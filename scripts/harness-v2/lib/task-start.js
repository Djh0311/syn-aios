'use strict';

// Adaptive Harness v0.5 — propose / start / doctor / recover 编排（AH-050-06）
//
// 需求溯源：PK-1 · PK-2 · PK-3 · GIT-1 · GIT-2 · GIT-4 · GIT-10 · EX-9 ·
// KP-13 · KP-14 · KP-16 · R2 §5.1 / §6.1 / §6.4。
//
// 本模块只编排事实与固定动作：不直接读写文件、不直接起子进程。文件写入仍只经
// store，Git 写入仍只经 git-facts.runStartGit；因此单测可以注入 runtime，生产
// 路径也不会多出第二个 writer / spawner。

const crypto = require('node:crypto');
const path = require('node:path');

const defaultStore = require('./store');
const defaultPending = require('./pending-start');
const defaultOpening = require('./opening');
const defaultScope = require('./scope');
const defaultGitFacts = require('./git-facts');
const defaultContext = require('./context');
const defaultGraph = require('./graph');
const defaultLifecycle = require('./lifecycle');
const defaultNodeSchema = require('./node-schema');
const defaultRouting = require('./routing');
const defaultPrepare = require('./prepare');

const PROPOSAL_SCHEMA = 'adaptive-harness/task-proposal/v1';
const START_PHASES = Object.freeze(['branch', 'worktree', 'taskPackage', 'openingCommit']);
const RECOVERY_ACTIONS = Object.freeze(['RESUME', 'ADOPT', 'PREPARE_CONFIRMED_REMOVAL']);
const ACTION_PREVIEW = Object.freeze([
  'CREATE_BRANCH',
  'ADD_WORKTREE',
  'WRITE_TASK_PACKAGE',
  'STAGE_OPENING_PACKAGE',
  'COMMIT_OPENING',
]);

function dependencies(runtime) {
  const settings = runtime || {};
  return {
    store: settings.store || defaultStore,
    pending: settings.pending || defaultPending,
    opening: settings.opening || defaultOpening,
    scope: settings.scope || defaultScope,
    gitFacts: settings.gitFacts || defaultGitFacts,
    context: settings.context || defaultContext,
    graph: settings.graph || defaultGraph,
    lifecycle: settings.lifecycle || defaultLifecycle,
    nodeSchema: settings.nodeSchema || defaultNodeSchema,
    routing: settings.routing || defaultRouting,
    prepare: settings.prepare || defaultPrepare,
    packageWriter: settings.packageWriter || null,
    inspectResources: settings.inspectResources || null,
    planes: settings.planes || null,
  };
}

function stableJson(value) {
  if (value === null) return 'null';
  if (typeof value === 'string') return JSON.stringify(value);
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) throw new TypeError('proposal digest 不接受非有限数值');
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map((item) => stableJson(item)).join(',')}]`;
  if (!value || typeof value !== 'object') throw new TypeError('proposal digest 只接受 JSON 值');
  return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(',')}}`;
}

function sha256(value) {
  return crypto.createHash('sha256').update(String(value), 'utf8').digest('hex');
}

function isPlainObject(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function clone(value) {
  if (value === null || value === undefined) return value;
  return JSON.parse(JSON.stringify(value));
}

function redact(value, deps) {
  return deps.routing && typeof deps.routing.redactKnownSecrets === 'function'
    ? deps.routing.redactKnownSecrets(value)
    : clone(value);
}

function publicResult(value, deps) {
  return redact(value, deps);
}

function failure(code, error, detail, deps) {
  return publicResult({
    ok: false,
    code,
    error,
    written: false,
    ...(detail || {}),
  }, deps);
}

function safeId(value) {
  const id = text(value);
  return id !== '' && id !== '.' && id !== '..' && !/[\\/\0]/.test(id) ? id : null;
}

function safeBranch(value) {
  const branch = text(value).replace(/^refs\/heads\//, '');
  return branch !== ''
    && !branch.startsWith('-')
    && !branch.startsWith('/')
    && !branch.endsWith('/')
    && !branch.endsWith('.')
    && !branch.endsWith('.lock')
    && !branch.includes('..')
    && !branch.includes('@{')
    && !/[\s~^:?*[\]\\\x00-\x1f\x7f]/.test(branch)
    && branch.split('/').every((part) => part !== '' && part !== '.' && part !== '..')
    ? branch
    : null;
}

function safeShortBranch(value) {
  const candidate = text(value);
  const branch = safeBranch(candidate);
  return branch
    && candidate === branch
    && branch !== 'HEAD'
    && branch !== '@'
    && !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(branch)
    ? branch
    : null;
}

function safeDigest(value) {
  const digest = text(value).toLowerCase();
  return /^[0-9a-f]{64}$/.test(digest) ? digest : null;
}

function safeOid(value) {
  const oid = text(value).toLowerCase();
  return /^[0-9a-f]{40}(?:[0-9a-f]{24})?$/.test(oid) ? oid : null;
}

function list(value) {
  return Array.isArray(value) ? value.slice() : null;
}

function unknown(label, value, out, predicate) {
  if (!predicate(value)) out.push(label);
  return value;
}

function plainBody(input) {
  if (typeof input === 'string' && input.trim() !== '') return input;
  return [
    '# 任务', '',
    '## 负责哪块', '按 proposal 中冻结的目标执行。', '',
    '## 边界（允许读写、禁止）',
    '### 允许读写', '只修改已声明 write-scope。', '',
    '### 禁止', '不触及 proposal 的 forbidden-scope。', '',
    '## 交付什么', '满足 proposal 中的 acceptance-criteria。', '',
    '## 怎么验证', '运行 proposal 中的 required verification。', '',
    '## 遇到什么必须停', '身份、范围、授权或 Git 现实无法唯一核对时停止；integrate、push、发布和物理清理仍需分别确认。', '',
  ].join('\n');
}

function packagePath(value) {
  const candidate = text(value).replace(/\\/g, '/');
  if (candidate === '' || candidate.startsWith('/') || candidate.includes('\0')) return null;
  const normalized = path.posix.normalize(candidate).replace(/^\.\//, '');
  if (normalized === '.' || normalized === '..' || normalized.startsWith('../') || normalized.includes('/../')) return null;
  return normalized;
}

function taskPackagePath(value, id) {
  const candidate = packagePath(value);
  const taskId = safeId(id);
  if (!candidate || !taskId || !candidate.startsWith('plans/v0.5.0/')) return null;
  const basename = path.posix.basename(candidate);
  if (basename === `${taskId}.md` || (basename.startsWith(`${taskId}-`) && basename.endsWith('.md'))) {
    return candidate;
  }
  return null;
}

function relativeTarget(root, relativePath) {
  const target = path.resolve(root, relativePath);
  const relation = path.relative(root, target);
  if (relation === '' || relation === '..' || relation.startsWith(`..${path.sep}`) || path.isAbsolute(relation)) return null;
  return target;
}

function cleanList(value) {
  if (!Array.isArray(value)) return null;
  const items = [];
  const seen = new Set();
  for (const entry of value) {
    const item = text(entry);
    if (item === '' || seen.has(item)) return null;
    seen.add(item);
    items.push(item);
  }
  return items;
}

// 开工 proposal 的 verification 是冻结合同，不是执行回执。新任务必须先声明
// 至少一条 required 检查；真实 status/run 只能由 ACTIVE 后的 record 写回。
function frozenVerificationIssues(entries) {
  const issues = [];
  if (!Array.isArray(entries)) {
    return ['verification 必须是至少一条冻结检查的数组'];
  }
  const seen = new Set();
  let hasRequired = false;
  for (let index = 0; index < entries.length; index += 1) {
    const entry = entries[index];
    const prefix = `verification[${index}]`;
    if (!isPlainObject(entry)) {
      issues.push(`${prefix} 必须是 object`);
      continue;
    }
    const allowed = new Set(['id', 'command', 'required', 'status']);
    for (const key of Object.keys(entry)) {
      if (!allowed.has(key)) {
        issues.push(`${prefix}.${key} 不属于冻结 verification 合同；run/output/evidence 必须在 ACTIVE 后独立 record`);
      }
    }
    const id = text(entry.id);
    const command = text(entry.command);
    if (id === '' || entry.id !== id) issues.push(`${prefix}.id 必须是非空、无首尾空白文本`);
    if (id !== '' && seen.has(id)) issues.push(`${prefix}.id 与前一条重复：${id}`);
    if (id !== '') seen.add(id);
    if (command === '' || entry.command !== command) issues.push(`${prefix}.command 必须是非空、无首尾空白文本`);
    if (typeof entry.required !== 'boolean') issues.push(`${prefix}.required 必须显式为 boolean`);
    if (entry.required === true) hasRequired = true;
    if (entry.status !== 'UNKNOWN') issues.push(`${prefix}.status 在 propose 时必须是 UNKNOWN`);
  }
  if (!hasRequired) issues.push('verification 至少要有一条 required:true');
  return issues;
}

function frozenVerificationValid(entries) {
  return frozenVerificationIssues(entries).length === 0;
}

function proposalNode(source, fields) {
  const verification = Array.isArray(source.verification) ? clone(source.verification) : [];
  const acceptance = Array.isArray(source.acceptanceCriteria) ? clone(source.acceptanceCriteria) : [];
  return {
    id: fields.id,
    kind: 'TASK',
    'parent-id': fields.parent.id,
    lifecycle: 'ACTIVE',
    goal: fields.goal,
    profile: fields.profile,
    'write-scope': fields.declaration.writeScope.slice(),
    'forbidden-scope': fields.declaration.forbiddenScope.slice(),
    'exclusive-resources': fields.declaration.exclusiveResources.slice(),
    'acceptance-criteria': acceptance,
    verification,
    git: {
      'base-branch': fields.base.branch,
      'base-oid': fields.base.oid,
      'task-branch': fields.branch.name,
      worktree: fields.worktree.requestedPath,
      'local-commit-allowed': fields.localCommitAllowed,
      'push-allowed': fields.pushAllowed,
      'product-commit': null,
      'wip-commit': null,
      'no-product-change': false,
      disposition: 'RETAINED',
      'integrated-observed': false,
    },
    relations: [],
    confirmations: [],
  };
}

/**
 * 生成可由用户自行保存的 proposal。这里不碰 store、Git 写入口或项目文件；四类
 * 读取事实由 task.js 传入，故同一输入永远产生同一 digest。
 */
function propose(input, runtime) {
  const deps = dependencies(runtime);
  const source = redact(isPlainObject(input) ? input : {}, deps);
  // READ_ONLY 不是“少填几个 Git 字段的 TASK”。它只形成一份可以在对话中继续
  // 澄清的合同，绝不带出可被 start 复用的 task proposal / digest。
  if (text(source.profile) === 'READ_ONLY') {
    const reads = {
      authority: clone(source.authority || null),
      current: clone(source.current || null),
      parentSummary: clone(source.parentSummary || null),
      gitReality: clone(source.gitReality || null),
    };
    return publicResult({
      ok: true,
      written: false,
      preview: true,
      route: 'READ_ONLY',
      conversationContract: {
        request: text(source.request),
        goal: text(source.goal) || null,
        boundary: '只读澄清；不创建 TASK、PENDING_START、branch、worktree 或 task package。',
      },
      reads: [
        { kind: 'AUTHORITY', value: reads.authority },
        { kind: 'CURRENT', value: reads.current },
        { kind: 'PARENT', value: reads.parentSummary },
        { kind: 'GIT_REALITY', value: reads.gitReality },
      ],
      actions: [],
    }, deps);
  }
  const unknowns = [];
  const id = unknown('id', safeId(source.id), unknowns, Boolean);
  const profile = text(source.profile);
  unknown('profile', profile, unknowns, (value) => ['READ_ONLY', 'ORDINARY_LOCAL', 'STRICT_LOCAL'].includes(value));
  const parent = isPlainObject(source.parent) ? source.parent : {};
  const parentId = unknown('parent.id', safeId(parent.id), unknowns, Boolean);
  const parentDigest = unknown('parent.digest', safeDigest(parent.digest), unknowns, Boolean);
  const base = isPlainObject(source.base) ? source.base : {};
  const baseBranch = unknown('base.branch', safeBranch(base.branch), unknowns, Boolean);
  const baseOid = unknown('base.oid', safeOid(base.oid), unknowns, Boolean);
  const branch = isPlainObject(source.branch) ? source.branch : {};
  const branchName = unknown('branch.name', safeBranch(branch.name), unknowns, Boolean);
  const worktree = isPlainObject(source.worktree) ? source.worktree : {};
  const requestedPath = unknown('worktree.requestedPath', text(worktree.requestedPath), unknowns, (value) => path.isAbsolute(value));
  const declaredCanonical = worktree.canonicalPath === null || worktree.canonicalPath === undefined
    ? null
    : text(worktree.canonicalPath);
  if (declaredCanonical !== null && !path.isAbsolute(declaredCanonical)) unknowns.push('worktree.canonicalPath');
  const declaration = isPlainObject(source.declaration) ? source.declaration : {};
  const writeScope = cleanList(declaration.writeScope);
  const forbiddenScope = cleanList(declaration.forbiddenScope);
  const exclusiveResources = cleanList(declaration.exclusiveResources);
  unknown('declaration.writeScope', writeScope, unknowns, (value) => Array.isArray(value) && value.length > 0);
  unknown('declaration.forbiddenScope', forbiddenScope, unknowns, Array.isArray);
  unknown('declaration.exclusiveResources', exclusiveResources, unknowns, Array.isArray);
  const packageInput = isPlainObject(source.taskPackage) ? source.taskPackage : {};
  const taskPackagePathValue = unknown('taskPackage.path', taskPackagePath(packageInput.path, id), unknowns, Boolean);
  const goal = unknown('goal', text(source.goal), unknowns, (value) => value !== '');
  const acceptanceCriteria = cleanList(source.acceptanceCriteria);
  unknown('acceptanceCriteria', acceptanceCriteria, unknowns, (value) => Array.isArray(value) && value.length > 0);
  const verification = Array.isArray(source.verification) ? clone(source.verification) : null;
  unknown('verification', verification, unknowns, frozenVerificationValid);
  const localCommitAllowed = source.localCommitAllowed;
  const pushAllowed = source.pushAllowed;
  unknown('localCommitAllowed', localCommitAllowed, unknowns, (value) => typeof value === 'boolean');
  unknown('pushAllowed', pushAllowed, unknowns, (value) => typeof value === 'boolean');

  const fields = {
    id: id || '',
    profile,
    parent: { id: parentId || '', digest: parentDigest || '' },
    base: { branch: baseBranch, oid: baseOid || '' },
    branch: { name: branchName || '' },
    worktree: { requestedPath: requestedPath || '', canonicalPath: declaredCanonical },
    declaration: {
      writeScope: writeScope || [],
      forbiddenScope: forbiddenScope || [],
      exclusiveResources: exclusiveResources || [],
    },
    goal: goal || '',
    localCommitAllowed: localCommitAllowed === true,
    pushAllowed: pushAllowed === true,
  };
  const node = proposalNode({ acceptanceCriteria: acceptanceCriteria || [], verification: verification || [] }, fields);
  const proposal = {
    schema: PROPOSAL_SCHEMA,
    request: text(source.request),
    id: fields.id,
    profile: fields.profile,
    parent: fields.parent,
    base: fields.base,
    branch: fields.branch,
    worktree: fields.worktree,
    declaration: fields.declaration,
    taskPackage: {
      path: taskPackagePathValue || '',
      body: plainBody(packageInput.body),
    },
    node,
    opening: {
      subject: text(source.opening && source.opening.subject) || `task(${fields.id || 'unknown'}): start`,
      why: text(source.opening && source.opening.why) || fields.goal || '开始执行已冻结任务',
      what: text(source.opening && source.opening.what) || '写入任务开工包',
      verification: text(source.opening && source.opening.verification) || 'opening package only',
    },
    reads: {
      authority: clone(source.authority || null),
      current: clone(source.current || null),
      parentSummary: clone(source.parentSummary || null),
      gitReality: clone(source.gitReality || null),
    },
    unknowns: [...new Set(unknowns)].sort(),
  };
  const proposalDigest = digestProposal(proposal, runtime);
  return publicResult({
    ok: true,
    written: false,
    preview: true,
    proposal,
    proposalDigest,
    unknowns: proposal.unknowns.slice(),
    reads: [
      { kind: 'AUTHORITY', value: proposal.reads.authority },
      { kind: 'CURRENT', value: proposal.reads.current },
      { kind: 'PARENT', value: proposal.reads.parentSummary },
      { kind: 'GIT_REALITY', value: proposal.reads.gitReality },
    ],
    actions: ACTION_PREVIEW.map((action) => ({ action })),
  }, deps);
}

function unwrapProposal(value) {
  if (!isPlainObject(value)) return null;
  if (isPlainObject(value.proposal)) return value.proposal;
  return value;
}

function digestProposal(value, runtime) {
  const deps = dependencies(runtime);
  // digest 的字节表示必须等于后续 start 真正会消费的表示。不能先对脱敏副本
  // 算摘要、再把原始对象交给 writer；否则两个不同 secret 文本会共用同一个
  // digest，却让落盘内容悄悄改变。
  const raw = unwrapProposal(value);
  const proposal = raw && isPlainObject(raw) ? redact(raw, deps) : null;
  if (!proposal) return null;
  return sha256(stableJson(redact(proposal, deps)));
}

function checkedProposal(input, deps) {
  const rawProposal = unwrapProposal(input && input.proposal);
  const proposal = rawProposal && isPlainObject(rawProposal) ? redact(rawProposal, deps) : null;
  if (!proposal || proposal.schema !== PROPOSAL_SCHEMA) {
    return { ok: false, result: failure('PROPOSAL_INVALID', 'proposal 不是当前可解析的 task proposal', null, deps) };
  }
  const digest = digestProposal(proposal, deps);
  const requested = safeDigest(input && input.proposalDigest);
  if (!requested || requested !== digest) {
    return {
      ok: false,
      result: failure('PROPOSAL_DIGEST_MISMATCH', '提供的 proposal digest 与当前 proposal 内容不一致，拒绝复用身份', {
        expectedDigest: digest,
        actualDigest: requested,
      }, deps),
    };
  }
  if (Array.isArray(proposal.unknowns) && proposal.unknowns.length > 0) {
    return {
      ok: false,
      result: failure('PROPOSAL_UNKNOWN_FIELDS', 'proposal 仍有会改变范围或权限的未知项，不能开工', {
        unknowns: proposal.unknowns.slice(),
      }, deps),
    };
  }
  const issues = validateStartableProposal(proposal, deps);
  if (issues.length > 0) {
    return {
      ok: false,
      result: failure('PROPOSAL_FIELD_INVALID', 'proposal 含不安全、越权或彼此不一致的字段，拒绝在 PENDING_START 前继续', {
        issues,
      }, deps),
    };
  }
  return { ok: true, proposal, digest };
}

function validationIssue(field, message) {
  return { field, message };
}

function sameList(left, right) {
  return Array.isArray(left) && Array.isArray(right) && stableJson(left) === stableJson(right);
}

function validateStartableProposal(proposal, deps) {
  const issues = [];
  if (!proposal || !Array.isArray(proposal.unknowns) || proposal.unknowns.length !== 0) {
    issues.push(validationIssue('unknowns', 'proposal 必须显式携带空 unknowns 清单'));
  }
  const id = safeId(proposal && proposal.id);
  if (!id) issues.push(validationIssue('id', 'TASK id 缺失或含路径字符'));
  if (!['ORDINARY_LOCAL', 'STRICT_LOCAL'].includes(text(proposal && proposal.profile))) {
    issues.push(validationIssue('profile', 'start 只接受 ORDINARY_LOCAL 或 STRICT_LOCAL'));
  }
  const parent = isPlainObject(proposal && proposal.parent) ? proposal.parent : {};
  if (!safeId(parent.id)) issues.push(validationIssue('parent.id', '父节点 id 无效'));
  if (!safeDigest(parent.digest)) issues.push(validationIssue('parent.digest', '父节点摘要必须是完整 sha256'));
  const base = isPlainObject(proposal && proposal.base) ? proposal.base : {};
  if (!safeShortBranch(base.branch)) issues.push(validationIssue('base.branch', 'base branch 必须是安全短分支名'));
  if (!safeOid(base.oid)) issues.push(validationIssue('base.oid', 'base OID 必须是完整 commit OID'));
  const branch = isPlainObject(proposal && proposal.branch) ? proposal.branch : {};
  if (!safeShortBranch(branch.name)) {
    issues.push(validationIssue('branch.name', 'task branch 必须是安全短名'));
  }
  const worktree = isPlainObject(proposal && proposal.worktree) ? proposal.worktree : {};
  if (!path.isAbsolute(text(worktree.requestedPath))) {
    issues.push(validationIssue('worktree.requestedPath', 'worktree 必须是绝对路径'));
  }
  if (worktree.canonicalPath !== null && worktree.canonicalPath !== undefined
    && !path.isAbsolute(text(worktree.canonicalPath))) {
    issues.push(validationIssue('worktree.canonicalPath', '预声明 canonical worktree 必须为空或绝对路径'));
  }
  const declaration = isPlainObject(proposal && proposal.declaration) ? proposal.declaration : {};
  const writeScope = cleanList(declaration.writeScope);
  const forbiddenScope = cleanList(declaration.forbiddenScope);
  const exclusiveResources = cleanList(declaration.exclusiveResources);
  if (!writeScope || writeScope.length === 0) issues.push(validationIssue('declaration.writeScope', 'write-scope 必须是非空、无重复清单'));
  if (!forbiddenScope) issues.push(validationIssue('declaration.forbiddenScope', 'forbidden-scope 必须是显式清单'));
  if (!exclusiveResources) issues.push(validationIssue('declaration.exclusiveResources', 'exclusive-resources 必须是显式清单'));
  if (writeScope && forbiddenScope && exclusiveResources
    && deps.scope && typeof deps.scope.declarationDefects === 'function') {
    const defects = deps.scope.declarationDefects({
      id,
      worktree: worktree.requestedPath,
      'write-scope': writeScope,
      'forbidden-scope': forbiddenScope,
      'exclusive-resources': exclusiveResources,
    });
    for (const defect of defects) {
      issues.push(validationIssue(`declaration.${defect.field}`, defect.message));
    }
  }
  const packageValue = taskPackagePath(
    proposal && proposal.taskPackage && proposal.taskPackage.path,
    id,
  );
  if (!packageValue) {
    issues.push(validationIssue('taskPackage.path', 'task package 只能是 plans/v0.5.0 下以同一 TASK id 命名的 Markdown'));
  } else if (writeScope && deps.scope && typeof deps.scope.pathWithinAny === 'function'
    && !deps.scope.pathWithinAny(packageValue, writeScope)) {
    issues.push(validationIssue('taskPackage.path', 'task package 必须由本 proposal 的 write-scope 明确覆盖'));
  }
  if (packageValue && forbiddenScope && deps.scope && typeof deps.scope.pathWithinAny === 'function'
    && deps.scope.pathWithinAny(packageValue, forbiddenScope)) {
    issues.push(validationIssue('taskPackage.path', 'task package 不得落入本 proposal 的 forbidden-scope'));
  }
  if (!proposal || !isPlainObject(proposal.taskPackage) || typeof proposal.taskPackage.body !== 'string'
    || proposal.taskPackage.body.trim() === '') {
    issues.push(validationIssue('taskPackage.body', 'task package 正文不能为空'));
  } else if (deps.prepare && typeof deps.prepare.validateTaskBody === 'function') {
    const body = deps.prepare.validateTaskBody(proposal.taskPackage.body);
    for (const issue of body && Array.isArray(body.issues) ? body.issues : []) {
      issues.push(validationIssue('taskPackage.body', `${issue.code || 'TASK_BODY_INVALID'}: ${issue.section || '正文结构无效'}`));
    }
  }
  if (!proposal || !isPlainObject(proposal.opening)
    || ['subject', 'why', 'what', 'verification'].some((key) => text(proposal.opening[key]) === '')) {
    issues.push(validationIssue('opening', 'opening commit 的 subject/Why/What/Verification 必须齐全'));
  }
  const node = isPlainObject(proposal && proposal.node) ? proposal.node : {};
  if (node.id !== id || node.kind !== 'TASK' || node['parent-id'] !== parent.id
    || node.lifecycle !== 'ACTIVE' || node.profile !== proposal.profile
    || !sameList(node['write-scope'], writeScope)
    || !sameList(node['forbidden-scope'], forbiddenScope)
    || !sameList(node['exclusive-resources'], exclusiveResources)) {
    issues.push(validationIssue('node', 'TASK after-image 与 proposal 身份、状态或声明不一致'));
  }
  for (const message of frozenVerificationIssues(node.verification)) {
    issues.push(validationIssue('node.verification', message));
  }
  const binding = isPlainObject(node.git) ? node.git : {};
  if (binding['base-branch'] !== base.branch || binding['base-oid'] !== base.oid
    || binding['task-branch'] !== branch.name || binding.worktree !== worktree.requestedPath
    || typeof binding['local-commit-allowed'] !== 'boolean'
    || typeof binding['push-allowed'] !== 'boolean') {
    issues.push(validationIssue('node.git', 'TASK Git binding 与 proposal 身份或显式权限不一致'));
  }
  if (binding['local-commit-allowed'] !== true) {
    issues.push(validationIssue('node.git.local-commit-allowed', 'start 必须创建 opening commit，proposal 未授权本地提交'));
  }
  if (deps.nodeSchema && typeof deps.nodeSchema.validateNode === 'function') {
    const validated = deps.nodeSchema.validateNode(node, {
      lifecycleValues: deps.lifecycle && deps.lifecycle.LIFECYCLE_VALUES,
    });
    for (const issue of validated && Array.isArray(validated.issues) ? validated.issues : []) {
      issues.push(validationIssue(`node.${issue.field || issue.path || 'schema'}`, issue.message || issue.code || 'TASK node schema 无效'));
    }
  }
  return issues;
}

function getSnapshot(input, deps) {
  if (input && input.snapshot) return input.snapshot;
  return deps.store.readLiveSnapshot(input && input.planes ? input.planes : deps.planes);
}

function recordDigest(record, deps) {
  if (record && safeDigest(record.digest)) return record.digest;
  if (record && typeof record.text === 'string' && typeof deps.store.digestOf === 'function') return deps.store.digestOf(record.text);
  return null;
}

function findRecord(snapshot, id) {
  const records = snapshot && Array.isArray(snapshot.records) ? snapshot.records : [];
  return records.find((record) => record && record.node && record.node.id === id) || null;
}

function equivalentId(left, right) {
  return typeof left === 'string' && typeof right === 'string'
    && left.normalize('NFC').toLowerCase() === right.normalize('NFC').toLowerCase();
}

/**
 * admission 中会故意排除“正要恢复的 marker id”，避免 marker 自己与自己冲突；
 * 但 canonical live graph 从不该有同编号 TASK。recover 若只依赖 excludedId，
 * 会把另一个 drafts/PARKED/current 副本误当成自己的旧现场，因此先单独做无排除的
 * 全编号检查，大小写/NFC 变体同样拒绝。
 */
function liveIdPrecondition(proposal, snapshot, deps) {
  const records = snapshot && Array.isArray(snapshot.records) ? snapshot.records : [];
  const conflicts = records
    .filter((record) => record && record.node && equivalentId(record.node.id, proposal.id))
    .map((record) => ({
      id: record.node.id,
      kind: record.node.kind || null,
      lifecycle: record.node.lifecycle || null,
      path: record.path || null,
    }));
  if (conflicts.length === 0) return { ok: true };
  return failure(
    'START_TASK_ID_EXISTS',
    `TASK ${proposal.id} 已在 live graph 中存在同编号或大小写/Unicode 等价副本，不能恢复或覆盖`,
    { id: proposal.id, conflicts },
    deps,
  );
}

function historyIdentityPrecondition(proposal, planes, input, deps) {
  if (!deps.store || typeof deps.store.readHistoryNode !== 'function') {
    return failure(
      'START_HISTORY_REALITY_UNAVAILABLE',
      '缺少 canonical history 读取能力，不能证明 TASK 编号从未使用',
      null,
      deps,
    );
  }
  const history = deps.store.readHistoryNode(planes, proposal.id, {
    cwd: input && input.cwd ? input.cwd : (planes && planes.repoRoot ? planes.repoRoot : undefined),
  });
  if (history) {
    return failure(
      'START_TASK_ID_HISTORY_EXISTS',
      `TASK ${proposal.id} 已存在于 canonical history；编号全局唯一且永不重用`,
      { id: proposal.id },
      deps,
    );
  }
  return { ok: true };
}

function parentPrecondition(proposal, snapshot, deps) {
  const parent = findRecord(snapshot, proposal.parent.id);
  if (!parent || !parent.node) {
    return failure('START_PARENT_NOT_FOUND', `找不到 proposal 指定的父节点 ${proposal.parent.id}`, null, deps);
  }
  if (parent.node.kind !== 'ROOT_PLAN' && parent.node.kind !== 'PHASE_PLAN') {
    return failure('START_PARENT_KIND_INVALID', `${proposal.parent.id} 不是可容纳 TASK 的计划节点`, null, deps);
  }
  if (parent.node.lifecycle !== 'ACTIVE') {
    return failure('START_PARENT_NOT_ACTIVE', `父节点 ${proposal.parent.id} 当前不是 ACTIVE`, { lifecycle: parent.node.lifecycle }, deps);
  }
  const digest = recordDigest(parent, deps);
  if (!digest || digest !== proposal.parent.digest) {
    return failure('START_PARENT_DIGEST_MISMATCH', '父节点正文已变化或无法唯一核对，必须重新 propose', {
      expectedDigest: proposal.parent.digest,
      actualDigest: digest,
    }, deps);
  }
  return { ok: true, parent };
}

function nodeParticipates(node, deps) {
  if (!node || node.kind !== 'TASK') return false;
  if (deps.lifecycle && typeof deps.lifecycle.participatesInScopeJudgement === 'function') {
    return deps.lifecycle.participatesInScopeJudgement(node) === true;
  }
  return Boolean(node.git);
}

function registeredDeclarations(snapshot, planes, deps, excludedId) {
  const records = snapshot && Array.isArray(snapshot.records) ? snapshot.records : [];
  const declarations = [];
  for (const record of records) {
    const node = record && record.node;
    if (!node || node.kind !== 'TASK' || node.id === excludedId) continue;
    const binding = node.git && typeof node.git === 'object' ? node.git : {};
    declarations.push({
      id: node.id,
      worktree: binding.worktree || null,
      'write-scope': node['write-scope'],
      'forbidden-scope': node['forbidden-scope'],
      'exclusive-resources': node['exclusive-resources'],
      participates: nodeParticipates(node, deps),
    });
  }
  const pendingStarts = deps.store.listPendingStarts(planes);
  for (const entry of pendingStarts) {
    const record = entry && entry.record ? entry.record : entry;
    if (!record || record.id === excludedId) continue;
    declarations.push(deps.pending.projectPendingStartDeclaration(record));
  }
  return declarations;
}

function admissionFor(proposal, worktreePath, snapshot, planes, deps, excludedId) {
  const request = {
    id: proposal.id,
    worktree: worktreePath,
    'write-scope': proposal.declaration.writeScope,
    'forbidden-scope': proposal.declaration.forbiddenScope,
    'exclusive-resources': proposal.declaration.exclusiveResources,
  };
  const registered = registeredDeclarations(snapshot, planes, deps, excludedId);
  const admission = deps.scope.decideAdmission({ request, registered });
  if (admission.admitted === true || admission.allowed === true) return { ok: true, admission, registered };
  return {
    ok: false,
    result: failure('START_ADMISSION_REJECTED', '开工声明与在册任务或 PENDING_START 冲突', {
      refusals: admission.refusals || [],
      overlaps: admission.overlaps || [],
      registered,
    }, deps),
  };
}

function inspectTarget(proposal, input, deps) {
  const inspected = deps.opening.inspectWorktreeTargetBeforeCreate({
    cwd: input.cwd,
    worktree: proposal.worktree.requestedPath,
  });
  if (!inspected || inspected.ok !== true || !inspected.canonicalPath) {
    return {
      ok: false,
      result: failure('START_WORKTREE_TARGET_REJECTED', 'worktree 目标无法以唯一 canonical identity 建立', {
        conflicts: inspected && inspected.conflicts ? inspected.conflicts : [],
      }, deps),
    };
  }
  return { ok: true, inspected };
}

function basePrecondition(proposal, input, deps) {
  if (typeof deps.gitFacts.objectExists === 'function'
    && deps.gitFacts.objectExists(input.cwd, proposal.base.oid) !== true) {
    return failure('START_BASE_OID_UNREACHABLE', 'proposal 冻结的 base OID 在当前仓库不可达，必须重新 propose', null, deps);
  }
  const branch = deps.gitFacts.runGit && deps.gitFacts.runGit([
    'rev-parse', '--verify', '--end-of-options', `refs/heads/${proposal.base.branch}^{commit}`,
  ], { cwd: input.cwd });
  if (!branch || branch.ok !== true) {
    return failure('START_BASE_BRANCH_UNREACHABLE', 'proposal 冻结的 integration/base branch 无法解析，必须停止', null, deps);
  }
  const actualOid = safeOid(branch.stdout);
  if (!actualOid) {
    return failure('START_BASE_BRANCH_UNREACHABLE', 'proposal 冻结的 integration/base branch 未返回完整 commit OID，必须停止', null, deps);
  }
  if (actualOid !== proposal.base.oid) {
    return {
      ok: true,
      drifted: true,
      frozenOid: proposal.base.oid,
      actualOid,
      branch: proposal.base.branch,
      continuation: '继续从冻结 base OID 开工；集成前按 R2 §6.5 走受确认重放并重新核对。',
    };
  }
  return {
    ok: true,
    drifted: false,
    frozenOid: proposal.base.oid,
    actualOid,
    branch: proposal.base.branch,
    continuation: null,
  };
}

function pendingRecordFor(proposal, digest) {
  return {
    id: proposal.id,
    proposal: { digest },
    parent: { ...proposal.parent },
    base: { oid: proposal.base.oid },
    branch: { name: proposal.branch.name },
    worktree: { requestedPath: proposal.worktree.requestedPath, canonicalPath: null },
    declaration: {
      writeScope: proposal.declaration.writeScope.slice(),
      forbiddenScope: proposal.declaration.forbiddenScope.slice(),
      exclusiveResources: proposal.declaration.exclusiveResources.slice(),
    },
    status: 'PENDING_START',
    steps: { branch: 'PENDING', worktree: 'PENDING', taskPackage: 'PENDING', openingCommit: 'PENDING' },
    resources: { branch: null, worktree: null, taskPackage: null, openingCommit: null },
  };
}

function readEntry(result) {
  if (result && result.record) return result;
  return null;
}

function resourceList(record) {
  const resources = record && record.resources ? record.resources : {};
  const list = [];
  for (const name of START_PHASES) {
    if (resources[name] !== null && resources[name] !== undefined) {
      list.push({ kind: name, value: clone(resources[name]) });
    }
  }
  return list;
}

function updateRecord(state, patch, deps) {
  const result = deps.store.updatePendingStart(state.planes, state.record.id, patch, {
    expectedGeneration: state.generation,
    expectedDigest: state.record.digest,
  });
  const entry = readEntry(result);
  if (!entry) throw new Error('store.updatePendingStart 未返回 record');
  return { ...state, record: entry.record, generation: entry.generation };
}

function markFailed(state, phase, error, deps, status) {
  const steps = { ...state.record.steps, [phase]: 'FAILED' };
  try {
    const next = updateRecord(state, { status: status || 'START_FAILED', steps, resources: state.record.resources }, deps);
    return failure('START_PHASE_FAILED', `开工阶段 ${phase} 失败；PENDING_START 已保留供 doctor/recover 核对`, {
      phase,
      active: false,
      pending: deps.pending.pendingStartDoctorView(next.record),
      resources: resourceList(next.record),
      cause: error && error.message ? error.message : String(error),
      causeCode: error && error.code ? error.code : null,
    }, deps);
  } catch (recordError) {
    return failure('START_FAILURE_RECORDING_FAILED', `开工阶段 ${phase} 失败且 PENDING_START 更新也失败；绝不报告 ACTIVE`, {
      phase,
      active: false,
      cause: error && error.message ? error.message : String(error),
      recordError: recordError && recordError.message ? recordError.message : String(recordError),
    }, deps);
  }
}

function activeNodeFor(proposal, state) {
  const node = clone(proposal.node || {});
  node.id = proposal.id;
  node.kind = 'TASK';
  node['parent-id'] = proposal.parent.id;
  node.lifecycle = 'ACTIVE';
  node.profile = proposal.profile;
  node['write-scope'] = proposal.declaration.writeScope.slice();
  node['forbidden-scope'] = proposal.declaration.forbiddenScope.slice();
  node['exclusive-resources'] = proposal.declaration.exclusiveResources.slice();
  node.git = {
    'base-branch': proposal.base.branch,
    'base-oid': proposal.base.oid,
    'task-branch': proposal.branch.name,
    worktree: state.record.worktree.canonicalPath,
    'local-commit-allowed': proposal.node && proposal.node.git
      ? proposal.node.git['local-commit-allowed'] === true
      : false,
    'push-allowed': proposal.node && proposal.node.git
      ? proposal.node.git['push-allowed'] === true
      : false,
    'product-commit': null,
    'wip-commit': null,
    'no-product-change': false,
    disposition: 'RETAINED',
    'integrated-observed': false,
  };
  node.relations = Array.isArray(node.relations) ? node.relations : [];
  node.confirmations = Array.isArray(node.confirmations) ? node.confirmations : [];
  return node;
}

function packageTextFor(node, proposal, deps) {
  if (deps.nodeSchema && typeof deps.nodeSchema.serializeNode === 'function') {
    return deps.nodeSchema.serializeNode(node, proposal.taskPackage.body);
  }
  return proposal.taskPackage.body;
}

function writePackage(state, proposal, deps) {
  const root = state.record.worktree.canonicalPath;
  const relative = taskPackagePath(proposal.taskPackage.path, proposal.id);
  const target = root && relative ? relativeTarget(root, relative) : null;
  if (!target) throw Object.assign(new Error('opening package 路径越出 linked worktree'), { code: 'START_PACKAGE_PATH_INVALID' });
  const node = activeNodeFor(proposal, state);
  const text = packageTextFor(node, proposal, deps);
  if (deps.packageWriter) {
    const written = deps.packageWriter(target, text);
    if (written === false || (written && written.ok === false)) {
      throw Object.assign(new Error('opening package writer 拒绝写入'), {
        code: written && written.code ? written.code : 'START_PACKAGE_WRITE_FAILED',
      });
    }
    return { target, path: relative, text, node, generation: state.generation };
  }
  // package 是真正的开工产物，不能经 atomicWrite 静默覆盖已有文件。把“必须不存在”
  // 和当前 generation 放进同一 store 事务；若进程在此后崩溃，recover 只会读取并
  // 核对字节摘要，绝不会重写该文件。
  if (typeof deps.store.planNodeWrite !== 'function' || typeof deps.store.commitNodeWrite !== 'function') {
    throw Object.assign(new Error('store 未提供 opening package 的受保护写事务'), {
      code: 'START_PACKAGE_WRITER_UNAVAILABLE',
    });
  }
  const plan = deps.store.planNodeWrite(state.planes, [], {
    expectedGeneration: state.generation,
    extraFiles: [{
      target,
      text,
      mustBeAbsent: true,
      role: 'opening-package',
      allowedRoot: root,
    }],
  });
  const applied = deps.store.commitNodeWrite(state.planes, plan);
  return { target, path: relative, text, node, generation: applied.generation };
}

function startGit(deps, action, fields, cwd) {
  const result = deps.gitFacts.runStartGit(action, fields, { authorized: true, cwd });
  if (!result || result.ok !== true) {
    const detail = result && typeof result === 'object' ? result : null;
    const error = new Error(`Git 开工动作 ${action} 返回失败结果`);
    error.code = 'START_GIT_ACTION_FAILED';
    error.detail = detail;
    throw error;
  }
  return result;
}

function phaseBranch(state, proposal, input, deps) {
  if (state.record.steps.branch === 'DONE') return { ok: true, state };
  try {
    startGit(deps, 'CREATE_BRANCH', { branch: proposal.branch.name, baseOid: proposal.base.oid }, input.cwd);
    const next = updateRecord(state, {
      status: 'PENDING_START',
      steps: { ...state.record.steps, branch: 'DONE' },
      resources: { ...state.record.resources, branch: { name: proposal.branch.name } },
    }, deps);
    return { ok: true, state: next };
  } catch (error) {
    return { ok: false, result: markFailed(state, 'branch', error, deps) };
  }
}

function phaseWorktree(state, proposal, input, deps) {
  if (state.record.steps.worktree === 'DONE') return { ok: true, state };
  try {
    const rechecked = inspectTarget(proposal, input, deps);
    if (!rechecked.ok || rechecked.inspected.canonicalPath !== state.targetCanonicalPath) {
      throw Object.assign(new Error('worktree 创建前的目标 identity 已在 PENDING_START 后变化'), {
        code: 'START_WORKTREE_TARGET_DRIFT',
        detail: rechecked.ok ? {
          expected: state.targetCanonicalPath,
          actual: rechecked.inspected.canonicalPath,
        } : rechecked.result,
      });
    }
    startGit(deps, 'ADD_WORKTREE', {
      branch: proposal.branch.name,
      worktree: proposal.worktree.requestedPath,
    }, input.cwd);
    const reconciled = deps.opening.reconcileCreatedWorktree({
      cwd: input.cwd,
      worktree: proposal.worktree.requestedPath,
      canonicalPath: state.targetCanonicalPath,
      branch: proposal.branch.name,
      baseOid: proposal.base.oid,
    });
    if (!reconciled || reconciled.ok !== true || !reconciled.facts || !reconciled.facts.canonicalWorktree) {
      const error = Object.assign(new Error('worktree 创建后现实与冻结身份不一致'), {
        code: 'START_WORKTREE_REALITY_CONFLICT',
        detail: reconciled && reconciled.conflicts ? reconciled.conflicts : [],
      });
      return { ok: false, result: markFailed(state, 'worktree', error, deps, 'BLOCKED') };
    }
    const canonicalPath = reconciled.facts.canonicalWorktree;
    const next = updateRecord(state, {
      status: 'PENDING_START',
      worktreeCanonicalPath: canonicalPath,
      steps: { ...state.record.steps, worktree: 'DONE' },
      resources: {
        ...state.record.resources,
        worktree: { path: proposal.worktree.requestedPath, canonicalPath },
      },
    }, deps);
    return { ok: true, state: { ...next, targetCanonicalPath: canonicalPath } };
  } catch (error) {
    return { ok: false, result: markFailed(state, 'worktree', error, deps) };
  }
}

function phasePackage(state, proposal, deps) {
  if (state.record.steps.taskPackage === 'DONE') return { ok: true, state };
  try {
    const written = writePackage(state, proposal, deps);
    const packageState = written.generation === state.generation
      ? state
      : { ...state, generation: written.generation };
    const next = updateRecord(packageState, {
      status: 'PENDING_START',
      steps: { ...packageState.record.steps, taskPackage: 'DONE' },
      resources: {
        ...packageState.record.resources,
        taskPackage: { path: written.path, digest: sha256(written.text) },
      },
    }, deps);
    return { ok: true, state: { ...next, package: written } };
  } catch (error) {
    return { ok: false, result: markFailed(state, 'taskPackage', error, deps) };
  }
}

function phaseOpeningCommit(state, proposal, input, deps) {
  if (state.record.steps.openingCommit === 'DONE') return { ok: true, state };
  try {
    const packagePathValue = state.record.resources.taskPackage && state.record.resources.taskPackage.path;
    if (!packagePathValue) throw Object.assign(new Error('opening package 尚未有可核对的资源事实'), { code: 'START_PACKAGE_FACT_MISSING' });
    const worktree = state.record.worktree.canonicalPath;
    startGit(deps, 'STAGE_OPENING_PACKAGE', { worktree, packagePath: packagePathValue }, worktree);
    startGit(deps, 'COMMIT_OPENING', {
      worktree,
      packagePath: packagePathValue,
      subject: proposal.opening.subject,
      why: proposal.opening.why,
      what: proposal.opening.what,
      verification: proposal.opening.verification,
    }, worktree);
    const oid = safeOid(deps.gitFacts.headOid(worktree));
    if (!oid) throw Object.assign(new Error('opening commit 后无法读到完整 HEAD OID'), { code: 'START_OPENING_COMMIT_UNVERIFIABLE' });
    const provisional = {
      ...state,
      record: {
        ...state.record,
        steps: { ...state.record.steps, openingCommit: 'DONE' },
        resources: { ...state.record.resources, openingCommit: { oid } },
      },
    };
    const reality = reconcileRecoveryReality(provisional, proposal, deps);
    if (!reality.ok || reality.allDone !== true
      || !reality.resources.openingCommit
      || reality.resources.openingCommit.oid !== oid) {
      throw Object.assign(new Error('opening commit 后的 branch/worktree/package/commit 现实未通过完整复核'), {
        code: 'START_OPENING_COMMIT_REALITY_CONFLICT',
        detail: reality,
      });
    }
    const next = updateRecord(state, {
      status: 'PENDING_START',
      steps: { ...state.record.steps, openingCommit: 'DONE' },
      resources: { ...state.record.resources, openingCommit: reality.resources.openingCommit },
    }, deps);
    return { ok: true, state: next };
  } catch (error) {
    return { ok: false, result: markFailed(state, 'openingCommit', error, deps) };
  }
}

function finalAdmission(state, proposal, snapshot, input, deps, reason) {
  const registered = registeredDeclarations(snapshot, state.planes, deps, state.record.id);
  const fields = {
    'task-id': proposal.id,
    'base-branch': proposal.base.branch,
    'base-oid': proposal.base.oid,
    'task-branch': proposal.branch.name,
    worktree: state.record.worktree.canonicalPath,
    'write-scope': proposal.declaration.writeScope,
    'forbidden-scope': proposal.declaration.forbiddenScope,
    'exclusive-resources': proposal.declaration.exclusiveResources,
    'local-commit-allowed': proposal.node.git['local-commit-allowed'],
    'push-allowed': proposal.node.git['push-allowed'],
  };
  const evaluated = deps.opening.evaluateOpening({
    cwd: state.record.worktree.canonicalPath,
    fields,
    registered,
    reason: reason || 'START',
  });
  if (!evaluated || evaluated.ok !== true) {
    return {
      ok: false,
      result: markFailed(state, 'openingCommit', Object.assign(new Error('进入 ACTIVE 前的 opening/admission 重判失败'), {
        code: 'START_FINAL_ADMISSION_REJECTED',
        detail: evaluated,
      }), deps, 'BLOCKED'),
    };
  }
  return { ok: true, evaluated, registered };
}

function finalizationRealityGuard(state, proposal, deps) {
  if (!deps.store || typeof deps.store.createPendingStartFinalizationRealityGuard !== 'function') {
    throw Object.assign(new Error('store 未提供受控的 finalize resource reality guard；不能把最终 Git 现实留在锁外'), {
      code: 'START_FINAL_REALITY_GUARD_UNAVAILABLE',
    });
  }
  return deps.store.createPendingStartFinalizationRealityGuard(
    state.planes,
    state.record.id,
    {
      expectedGeneration: state.generation,
      expectedDigest: state.record.digest,
      // store 在 graph lock 内于写前和写后各调用一次；这里仍只读 inspectResources，
      // 不引入任何新的 Git 写入口。post 若发现 HEAD/package/worktree 漂移，store
      // 会在同一事务中回滚 ACTIVE/CURRENT/marker/generation。
      probe(phase) {
        const reality = reconcileRecoveryReality(state, proposal, deps);
        if (!reality.ok || reality.allDone !== true) {
          return {
            ok: false,
            code: 'START_FINAL_REALITY_CONFLICT',
            reason: `phase ${phase} 的 branch/worktree/package/opening commit 现实未通过完整复核`,
          };
        }
        return { ok: true };
      },
    },
  );
}

function finalizeActive(state, proposal, snapshot, input, deps, reason) {
  const historyIdentity = historyIdentityPrecondition(proposal, state.planes, input, deps);
  if (!historyIdentity.ok) {
    return markFailed(state, 'openingCommit', Object.assign(new Error(historyIdentity.error), {
      code: historyIdentity.code,
      detail: historyIdentity,
    }), deps, 'BLOCKED');
  }
  const base = basePrecondition(proposal, input, deps);
  if (!base.ok) {
    return markFailed(state, 'openingCommit', Object.assign(new Error(base.error), {
      code: base.code,
      detail: base,
    }), deps, 'BLOCKED');
  }
  const check = finalAdmission(state, proposal, snapshot, input, deps, reason);
  if (!check.ok) return check.result;
  const node = activeNodeFor(proposal, state);
  const validated = deps.nodeSchema.validateNode
    ? deps.nodeSchema.validateNode(node, { lifecycleValues: deps.lifecycle.LIFECYCLE_VALUES })
    : { issues: [] };
  if (validated && Array.isArray(validated.issues) && validated.issues.length > 0) {
    return markFailed(state, 'openingCommit', Object.assign(new Error('ACTIVE task node 未通过 schema'), {
      code: 'START_ACTIVE_NODE_INVALID', detail: validated.issues,
    }), deps, 'BLOCKED');
  }
  let index = null;
  if (deps.graph && typeof deps.graph.buildGraphIndex === 'function') {
    index = deps.graph.buildGraphIndex((snapshot.records || []).concat([{ node, path: null, issues: [] }]));
  }
  const currentText = deps.context.renderCurrent({
    index,
    activeLeafId: node.id,
    worktreePath: state.record.worktree.canonicalPath,
    blockers: [],
  });
  const currentPath = deps.store.currentViewPath(
    state.planes,
    deps.store.worktreeKeyFor(state.record.worktree.canonicalPath),
  );
  const finalReality = reconcileRecoveryReality(state, proposal, deps);
  if (!finalReality.ok || finalReality.allDone !== true) {
    return markFailed(state, 'openingCommit', Object.assign(
      new Error('ACTIVE 原子收口前的 branch/worktree/package/opening commit 现实已漂移'),
      {
        code: 'START_FINAL_REALITY_CONFLICT',
        detail: finalReality,
      },
    ), deps, 'BLOCKED');
  }
  let resourceRealityGuard;
  try {
    resourceRealityGuard = finalizationRealityGuard(state, proposal, deps);
  } catch (error) {
    return markFailed(state, 'openingCommit', error, deps, 'BLOCKED');
  }
  try {
    const applied = deps.store.finalizePendingStart(state.planes, state.record.id, {
      expectedGeneration: state.generation,
      expectedDigest: state.record.digest,
      nodeChanges: [{ node, body: proposal.taskPackage.body, previousPath: null }],
      guardRecords: state.parent && state.parent.path ? [{
        id: state.parent.node.id,
        path: state.parent.path,
        expectedDigest: state.record.parent.digest,
      }] : [],
      openingCommitOid: state.record.resources.openingCommit && state.record.resources.openingCommit.oid,
      extraFiles: [{ target: currentPath, text: `${String(currentText).replace(/\s+$/, '')}\n` }],
      resourceRealityGuard,
    });
    return publicResult({
      ok: true,
      code: null,
      active: true,
      written: true,
      preview: false,
      id: node.id,
      worktree: state.record.worktree.canonicalPath,
      allowedPaths: node['write-scope'].slice(),
      verification: node.verification,
      openingCommit: state.record.resources.openingCommit,
      baseReality: base,
      generation: applied.generation,
      writtenPaths: applied.written,
    }, deps);
  } catch (error) {
    return failure('START_FINALIZATION_FAILED', 'ACTIVE/CURRENT/PENDING 原子收口失败；绝不报告 ACTIVE', {
      active: false,
      cause: error && error.message ? error.message : String(error),
      causeCode: error && error.code ? error.code : null,
    }, deps);
  }
}

function runPhases(state, proposal, snapshot, input, deps, reason) {
  let current = state;
  const branch = phaseBranch(current, proposal, input, deps);
  if (!branch.ok) return branch.result;
  current = branch.state;
  const worktree = phaseWorktree(current, proposal, input, deps);
  if (!worktree.ok) return worktree.result;
  current = worktree.state;
  const taskPackage = phasePackage(current, proposal, deps);
  if (!taskPackage.ok) return taskPackage.result;
  current = taskPackage.state;
  const openingCommit = phaseOpeningCommit(current, proposal, input, deps);
  if (!openingCommit.ok) return openingCommit.result;
  current = openingCommit.state;
  return finalizeActive(current, proposal, snapshot, input, deps, reason);
}

function start(input, runtime) {
  const deps = dependencies(runtime);
  const settings = input || {};
  const rawProposal = unwrapProposal(settings.proposal);
  if (rawProposal && rawProposal.profile === 'READ_ONLY') {
    return failure('START_READ_ONLY_FORBIDDEN', 'READ_ONLY 只允许形成会话合同，不能进入 PENDING_START 或执行任何 Git 开工动作', null, deps);
  }
  if (settings.write !== true) {
    return failure('START_WRITE_REQUIRED', 'start 会创建 PENDING_START / branch / worktree；必须显式 --write', null, deps);
  }
  const checked = checkedProposal(settings, deps);
  if (!checked.ok) return checked.result;
  const proposal = checked.proposal;
  const planes = settings.planes || deps.planes;
  try {
    const snapshot = getSnapshot(settings, deps);
    const parent = parentPrecondition(proposal, snapshot, deps);
    if (!parent.ok) return parent;
    const liveIdentity = liveIdPrecondition(proposal, snapshot, deps);
    if (!liveIdentity.ok) return liveIdentity;
    const historyIdentity = historyIdentityPrecondition(proposal, planes, settings, deps);
    if (!historyIdentity.ok) return historyIdentity;
    const base = basePrecondition(proposal, settings, deps);
    if (!base.ok) return base;
    const target = inspectTarget(proposal, settings, deps);
    if (!target.ok) return target.result;
    const admission = admissionFor(proposal, target.inspected.canonicalPath, snapshot, planes, deps, proposal.id);
    if (!admission.ok) return admission.result;
    // 这里是 start 的第一笔写。此前所有分支都是只读预检。
    const created = deps.store.createPendingStart(planes, pendingRecordFor(proposal, checked.digest), {
      expectedGeneration: snapshot.generation,
    });
    const entry = readEntry(created);
    if (!entry) throw new Error('store.createPendingStart 未返回 record');
    const state = {
      planes,
      record: entry.record,
      generation: entry.generation,
      targetCanonicalPath: target.inspected.canonicalPath,
      parent: parent.parent,
      baseReality: base,
    };
    return runPhases(state, proposal, snapshot, settings, deps, 'START');
  } catch (error) {
    return failure(error && error.code ? error.code : 'START_OPERATION_FAILED', error && error.message ? error.message : String(error), {
      active: false,
    }, deps);
  }
}

function doctor(input, runtime) {
  const deps = dependencies(runtime);
  const settings = input || {};
  const id = safeId(settings.pendingId || settings.id);
  if (!id) return failure('PENDING_START_ID_REQUIRED', 'doctor 需要 --pending <ID>', null, deps);
  try {
    const entry = deps.store.readPendingStart(settings.planes || deps.planes, id);
    if (!entry) return failure('PENDING_START_NOT_FOUND', `找不到 PENDING_START ${id}`, null, deps);
    const record = entry.record || entry;
    const actual = deps.inspectResources ? deps.inspectResources(record) : null;
    return publicResult({
      ok: true,
      written: false,
      preview: true,
      pending: deps.pending.pendingStartDoctorView(record),
      resources: resourceList(record),
      reality: actual,
    }, deps);
  } catch (error) {
    return failure(error && error.code ? error.code : 'PENDING_START_DOCTOR_FAILED', error && error.message ? error.message : String(error), null, deps);
  }
}

function sameProposalIdentity(record, proposal, digest) {
  return record.id === proposal.id
    && record.proposal.digest === digest
    && record.parent.id === proposal.parent.id
    && record.parent.digest === proposal.parent.digest
    && record.base.oid === proposal.base.oid
    && record.branch.name === proposal.branch.name
    && record.worktree.requestedPath === proposal.worktree.requestedPath
    && JSON.stringify(record.declaration.writeScope) === JSON.stringify(proposal.declaration.writeScope)
    && JSON.stringify(record.declaration.forbiddenScope) === JSON.stringify(proposal.declaration.forbiddenScope)
    && JSON.stringify(record.declaration.exclusiveResources) === JSON.stringify(proposal.declaration.exclusiveResources);
}

function hasOwn(object, key) {
  return Boolean(object) && Object.prototype.hasOwnProperty.call(object, key);
}

function resourcesFromReality(value) {
  if (!isPlainObject(value)) return null;
  if (value.ok === false) return null;
  if (isPlainObject(value.resources)) return value.resources;
  return value;
}

function recoveryReality(record, proposal, deps) {
  if (typeof deps.inspectResources !== 'function') {
    return {
      ok: false,
      code: 'RECOVERY_REALITY_UNAVAILABLE',
      error: 'recover 缺少只读资源核对器；不能凭 marker 猜测 Git/文件现实',
      detail: null,
    };
  }
  let observed;
  try {
    observed = deps.inspectResources(record, proposal);
  } catch (error) {
    return {
      ok: false,
      code: error && error.code ? error.code : 'RECOVERY_REALITY_UNAVAILABLE',
      error: error && error.message ? error.message : String(error),
      detail: error && error.detail ? error.detail : null,
    };
  }
  if (!observed || observed.ok === false) {
    return {
      ok: false,
      code: observed && observed.code ? observed.code : 'RECOVERY_REALITY_UNAVAILABLE',
      error: observed && observed.error ? observed.error : '资源现实无法完整读取，拒绝用旧 marker 猜测',
      detail: observed && observed.detail ? observed.detail : observed || null,
    };
  }
  const resources = resourcesFromReality(observed);
  if (!resources) {
    return {
      ok: false,
      code: 'RECOVERY_REALITY_MALFORMED',
      error: '资源核对器未返回可解释的 branch/worktree/package/opening commit 事实',
      detail: observed,
    };
  }
  return { ok: true, observed, resources };
}

function actualExists(resources, phase) {
  return hasOwn(resources, phase) && resources[phase] !== null && resources[phase] !== undefined;
}

function sameValue(left, right) {
  return stableJson(left) === stableJson(right);
}

function recoveryMismatch(phase, message, expected, actual) {
  return {
    ok: false,
    phase,
    message,
    expected: expected === undefined ? null : clone(expected),
    actual: actual === undefined ? null : clone(actual),
  };
}

function normalizeRecoveredBranch(actual, state, proposal) {
  if (!isPlainObject(actual) || safeBranch(actual.name) !== proposal.branch.name) {
    return recoveryMismatch('branch', '实际 branch 与冻结 task branch 不一致', { name: proposal.branch.name }, actual);
  }
  if (safeOid(actual.baseOid) !== proposal.base.oid) {
    return recoveryMismatch('branch', '实际 branch 的 base identity 与 proposal 不一致', { baseOid: proposal.base.oid }, actual);
  }
  if (actual.containsBase !== true) {
    return recoveryMismatch('branch', '实际 branch 不包含冻结 base OID', { baseOid: proposal.base.oid }, actual);
  }
  const normalized = { name: proposal.branch.name };
  if (state.record.resources.branch && !sameValue(state.record.resources.branch, normalized)) {
    return recoveryMismatch('branch', 'marker 中的 branch 事实与现实不一致', state.record.resources.branch, normalized);
  }
  return { ok: true, resource: normalized };
}

function expectedCanonicalWorktree(state, proposal) {
  const current = state.record && state.record.worktree ? state.record.worktree.canonicalPath : null;
  if (current) return current;
  if (state.targetCanonicalPath) return state.targetCanonicalPath;
  if (proposal.worktree && proposal.worktree.canonicalPath) return proposal.worktree.canonicalPath;
  return path.resolve(state.record.worktree.requestedPath);
}

function normalizeRecoveredWorktree(actual, state, proposal) {
  if (!isPlainObject(actual)) {
    return recoveryMismatch('worktree', '实际 worktree 事实不是对象', {
      path: state.record.worktree.requestedPath,
      canonicalPath: expectedCanonicalWorktree(state, proposal),
    }, actual);
  }
  const resourcePath = text(actual.path || actual.requestedPath);
  const canonicalPath = text(actual.canonicalPath);
  const expectedCanonical = expectedCanonicalWorktree(state, proposal);
  if (resourcePath !== state.record.worktree.requestedPath
    || !path.isAbsolute(canonicalPath)
    || canonicalPath !== expectedCanonical) {
    return recoveryMismatch('worktree', '实际 worktree 路径或 canonical identity 与冻结值不一致', {
      path: state.record.worktree.requestedPath,
      canonicalPath: expectedCanonical,
    }, actual);
  }
  if (safeBranch(actual.branch) !== proposal.branch.name) {
    return recoveryMismatch('worktree', '实际 worktree checkout 的 branch 与冻结值不一致', { branch: proposal.branch.name }, actual);
  }
  if (safeOid(actual.baseOid) !== proposal.base.oid) {
    return recoveryMismatch('worktree', '实际 worktree 的 base identity 与冻结值不一致', { baseOid: proposal.base.oid }, actual);
  }
  if (actual.containsBase !== true) {
    return recoveryMismatch('worktree', '实际 worktree HEAD 不包含冻结 base OID', { baseOid: proposal.base.oid }, actual);
  }
  const normalized = { path: state.record.worktree.requestedPath, canonicalPath };
  if (state.record.resources.worktree && !sameValue(state.record.resources.worktree, normalized)) {
    return recoveryMismatch('worktree', 'marker 中的 worktree 事实与现实不一致', state.record.resources.worktree, normalized);
  }
  return { ok: true, resource: normalized, canonicalPath };
}

function expectedRecoveredPackageDigest(state, proposal, deps, canonicalPath) {
  if (state.record.resources.taskPackage && safeDigest(state.record.resources.taskPackage.digest)) {
    return state.record.resources.taskPackage.digest;
  }
  const record = {
    ...state.record,
    worktree: { ...state.record.worktree, canonicalPath: canonicalPath || state.record.worktree.canonicalPath },
  };
  return sha256(packageTextFor(activeNodeFor(proposal, { ...state, record }), proposal, deps));
}

function normalizeRecoveredPackage(actual, state, proposal, deps, canonicalPath) {
  const expectedPath = packagePath(proposal.taskPackage && proposal.taskPackage.path);
  const actualPath = isPlainObject(actual) ? packagePath(actual.path) : null;
  const actualDigest = isPlainObject(actual) ? safeDigest(actual.digest) : null;
  if (!expectedPath || actualPath !== expectedPath || !actualDigest) {
    return recoveryMismatch('taskPackage', '实际 task package 路径或字节摘要不可与 proposal 唯一核对', {
      path: expectedPath,
      digest: state.record.resources.taskPackage ? state.record.resources.taskPackage.digest : null,
    }, actual);
  }
  let expectedDigest;
  try {
    expectedDigest = expectedRecoveredPackageDigest(state, proposal, deps, canonicalPath);
  } catch (error) {
    return recoveryMismatch('taskPackage', '无法重建 task package 的冻结摘要，拒绝覆盖或接管', null, {
      cause: error && error.message ? error.message : String(error),
    });
  }
  if (actualDigest !== expectedDigest) {
    return recoveryMismatch('taskPackage', '实际 task package 摘要与冻结 proposal/marker 不一致；不允许覆盖', {
      path: expectedPath,
      digest: expectedDigest,
    }, actual);
  }
  const normalized = { path: expectedPath, digest: actualDigest };
  if (state.record.resources.taskPackage && !sameValue(state.record.resources.taskPackage, normalized)) {
    return recoveryMismatch('taskPackage', 'marker 中的 task package 事实与现实不一致', state.record.resources.taskPackage, normalized);
  }
  return { ok: true, resource: normalized };
}

function normalizeRecoveredOpeningCommit(actual, state, proposal) {
  const oid = isPlainObject(actual) ? safeOid(actual.oid) : null;
  const frozenPackageDigest = state.record.resources.taskPackage
    ? safeDigest(state.record.resources.taskPackage.digest)
    : null;
  if (!oid || oid === proposal.base.oid) {
    return recoveryMismatch('openingCommit', '实际 opening commit OID 不可验证或仍等于 base', state.record.resources.openingCommit, actual);
  }
  if (safeBranch(actual.branch) !== proposal.branch.name) {
    return recoveryMismatch('openingCommit', '实际 opening commit 不在冻结 task branch 上', { branch: proposal.branch.name }, actual);
  }
  if (taskPackagePath(actual.packagePath, proposal.id) !== taskPackagePath(proposal.taskPackage.path, proposal.id)) {
    return recoveryMismatch('openingCommit', '实际 opening commit 的路径不等于唯一 task package 路径', {
      packagePath: taskPackagePath(proposal.taskPackage.path, proposal.id),
    }, actual);
  }
  if (!frozenPackageDigest || safeDigest(actual.headPackageDigest) !== frozenPackageDigest) {
    return recoveryMismatch('openingCommit', 'HEAD tree 中的 task package 字节摘要与 marker 不一致', {
      headPackageDigest: frozenPackageDigest,
    }, actual);
  }
  if (actual.onlyTaskPackage !== true || actual.containsBase !== true
    || actual.singleCommitFromBase !== true || actual.indexClean !== true
    || actual.worktreeClean !== true) {
    return recoveryMismatch('openingCommit', '实际 opening commit 的范围或 ancestry 不满足冻结身份', {
      packagePath: taskPackagePath(proposal.taskPackage.path, proposal.id),
      baseOid: proposal.base.oid,
      singleCommitFromBase: true,
      indexClean: true,
      worktreeClean: true,
      headPackageDigest: frozenPackageDigest,
    }, actual);
  }
  const normalized = { oid };
  if (state.record.resources.openingCommit && !sameValue(state.record.resources.openingCommit, normalized)) {
    return recoveryMismatch('openingCommit', 'marker 中的 opening commit 事实与现实不一致', state.record.resources.openingCommit, normalized);
  }
  return { ok: true, resource: normalized };
}

/**
 * 这里只读现实并得出 marker 应有的 after-image；调用者随后才能决定是否写 marker。
 * 若某个已记录资源消失/变形，或后续资源越过前序资源存在，直接失败而不是“修正”
 * marker，也绝不写 package 来弥补未知现场。
 */
function reconcileRecoveryReality(state, proposal, deps) {
  const read = recoveryReality(state.record, proposal, deps);
  if (!read.ok) return read;
  const resources = { ...state.record.resources };
  const steps = { ...state.record.steps };
  let canonicalPath = state.record.worktree.canonicalPath;
  let absentBefore = false;
  for (const phase of START_PHASES) {
    const present = actualExists(read.resources, phase);
    if (!present) {
      if (state.record.steps[phase] === 'DONE' || state.record.resources[phase] !== null) {
        return {
          ok: false,
          code: 'RECOVERY_REALITY_CONFLICT',
          error: `marker 说 ${phase} 已存在，但只读现实中它不存在；不能自动重建或覆盖`,
          detail: { phase, marker: state.record.resources[phase], reality: null },
        };
      }
      absentBefore = true;
      continue;
    }
    if (absentBefore) {
      return {
        ok: false,
        code: 'RECOVERY_REALITY_CONFLICT',
        error: `只读现实中 ${phase} 已存在，但它的前序资源不存在；现场不可解释`,
        detail: { phase, reality: read.resources[phase] },
      };
    }
    let normalized;
    if (phase === 'branch') normalized = normalizeRecoveredBranch(read.resources[phase], state, proposal);
    else if (phase === 'worktree') normalized = normalizeRecoveredWorktree(read.resources[phase], state, proposal);
    else if (phase === 'taskPackage') {
      normalized = normalizeRecoveredPackage(read.resources[phase], state, proposal, deps, canonicalPath);
    } else normalized = normalizeRecoveredOpeningCommit(read.resources[phase], state, proposal);
    if (!normalized.ok) {
      return {
        ok: false,
        code: 'RECOVERY_REALITY_CONFLICT',
        error: normalized.message,
        detail: {
          phase: normalized.phase,
          expected: normalized.expected,
          actual: normalized.actual,
        },
      };
    }
    resources[phase] = normalized.resource;
    steps[phase] = 'DONE';
    if (phase === 'worktree') canonicalPath = normalized.canonicalPath;
  }
  return {
    ok: true,
    reality: read.observed,
    steps,
    resources,
    worktreeCanonicalPath: canonicalPath,
    allDone: START_PHASES.every((phase) => steps[phase] === 'DONE' && resources[phase]),
  };
}

function recoveryBase(record, deps) {
  // RESUME 的“重试”语义先在内存中将 FAILED 归回 PENDING。ADOPT 也借相同
  // 规范化 after-image 消掉 FAILED；它随后必须四项全部由现实证明才会 finalize。
  return deps.pending.preparePendingStartRecovery(record, 'RESUME');
}

function recoveryStateAfterReality(state, reconciliation, action, deps) {
  const reset = recoveryBase(state.record, deps);
  const steps = { ...reset.steps };
  const resources = { ...reset.resources };
  for (const phase of START_PHASES) {
    if (reconciliation.steps[phase] === 'DONE') {
      steps[phase] = 'DONE';
      resources[phase] = reconciliation.resources[phase];
    }
  }
  const status = action === 'ADOPT' && !reconciliation.allDone ? 'BLOCKED' : 'PENDING_START';
  const patch = { status, steps, resources };
  if (reconciliation.worktreeCanonicalPath
    && state.record.worktree.canonicalPath !== reconciliation.worktreeCanonicalPath) {
    patch.worktreeCanonicalPath = reconciliation.worktreeCanonicalPath;
  }
  const unchanged = state.record.status === patch.status
    && sameValue(state.record.steps, patch.steps)
    && sameValue(state.record.resources, patch.resources)
    && !hasOwn(patch, 'worktreeCanonicalPath');
  if (unchanged) {
    return {
      ...state,
      targetCanonicalPath: reconciliation.worktreeCanonicalPath || state.targetCanonicalPath,
    };
  }
  const next = updateRecord(state, patch, deps);
  return {
    ...next,
    targetCanonicalPath: reconciliation.worktreeCanonicalPath || next.record.worktree.canonicalPath || state.targetCanonicalPath,
  };
}

function blockRecoveryState(state, detail, deps) {
  try {
    const blocked = updateRecord(state, {
      status: 'BLOCKED',
      steps: state.record.steps,
      resources: state.record.resources,
    }, deps);
    return failure('RECOVERY_REALITY_CONFLICT', 'recover 的只读现实与 PENDING_START 冻结身份冲突；已标记 BLOCKED，不执行 Git 写入', {
      pending: deps.pending.pendingStartDoctorView(blocked.record),
      resources: resourceList(blocked.record),
      detail,
    }, deps);
  } catch (error) {
    return failure('RECOVERY_REALITY_CONFLICT', 'recover 的只读现实与 PENDING_START 冻结身份冲突；marker 更新失败，仍未执行 Git 写入', {
      detail,
      recordError: error && error.message ? error.message : String(error),
    }, deps);
  }
}

function recover(input, runtime) {
  const deps = dependencies(runtime);
  const settings = input || {};
  const action = text(settings.action).toUpperCase();
  if (!RECOVERY_ACTIONS.includes(action)) {
    return failure('RECOVERY_ACTION_INVALID', `recover action 只能是 ${RECOVERY_ACTIONS.join(' / ')}`, null, deps);
  }
  if (settings.write !== true) {
    return failure('RECOVERY_WRITE_REQUIRED', 'recover 会更新 PENDING_START；必须显式 --write', null, deps);
  }
  const id = safeId(settings.pendingId || settings.id);
  if (!id) return failure('PENDING_START_ID_REQUIRED', 'recover 需要 --pending <ID>', null, deps);
  const planes = settings.planes || deps.planes;
  try {
    const entry = deps.store.readPendingStart(planes, id);
    if (!entry) return failure('PENDING_START_NOT_FOUND', `找不到 PENDING_START ${id}`, null, deps);
    const record = entry.record || entry;
    if (action === 'PREPARE_CONFIRMED_REMOVAL') {
      const nextRecord = deps.pending.preparePendingStartRecovery(record, action);
      const updated = deps.store.updatePendingStart(planes, id, {
        status: nextRecord.status,
        steps: nextRecord.steps,
        resources: nextRecord.resources,
      }, { expectedGeneration: entry.generation, expectedDigest: record.digest });
      const next = updated.record || updated;
      return publicResult({
        ok: true,
        written: true,
        preview: false,
        action,
        pending: deps.pending.pendingStartDoctorView(next),
        resources: resourceList(next),
        physicalRemovalExecuted: false,
      }, deps);
    }
    const checked = checkedProposal(settings, deps);
    if (!checked.ok) return checked.result;
    if (checked.proposal.profile === 'READ_ONLY') {
      return failure('RECOVERY_READ_ONLY_FORBIDDEN', 'READ_ONLY 没有可恢复的 Git 开工现场，拒绝创建或接管 PENDING_START', null, deps);
    }
    if (!sameProposalIdentity(record, checked.proposal, checked.digest)) {
      return failure('RECOVERY_IDENTITY_MISMATCH', 'recover 只能使用同一 proposal digest 和冻结身份，不能借重试换范围/父节点/基线', null, deps);
    }
    const historyIdentity = historyIdentityPrecondition(checked.proposal, planes, settings, deps);
    if (!historyIdentity.ok) return historyIdentity;
    if (record.status === 'READY_FOR_CONFIRMED_REMOVAL') {
      return failure('RECOVERY_STATUS_NOT_RESUMABLE', '已准备人工确认清理的 PENDING_START 不能再 RESUME 或 ADOPT；需先走独立的确认流程', null, deps);
    }
    const snapshot = getSnapshot(settings, deps);
    const parent = parentPrecondition(checked.proposal, snapshot, deps);
    if (!parent.ok) return parent;
    // 这一步必须早于 admissionFor(..., record.id)。后者为避免 marker 自冲突会
    // 排除同 id 的在册声明，不能被它拿来掩盖另一个 canonical live TASK。
    const liveIdentity = liveIdPrecondition(checked.proposal, snapshot, deps);
    if (!liveIdentity.ok) return liveIdentity;
    const base = basePrecondition(checked.proposal, settings, deps);
    if (!base.ok) return base;
    const initial = {
      planes,
      record,
      generation: entry.generation,
      targetCanonicalPath: record.worktree.canonicalPath || checked.proposal.worktree.canonicalPath,
      parent: parent.parent,
      baseReality: base,
    };
    // 任何 recover 先只读现实。不能先“重置 marker”再看现场，更不能根据
    // FAILED/PENDING 猜测 Git 是否已经实际成功。
    const reconciliation = reconcileRecoveryReality(initial, checked.proposal, deps);
    if (!reconciliation.ok) return blockRecoveryState(initial, reconciliation.detail || reconciliation, deps);
    const state = recoveryStateAfterReality(initial, reconciliation, action, deps);
    if (action === 'ADOPT' && !reconciliation.allDone) {
      return failure('RECOVERY_ADOPT_UNEXPLAINABLE', 'ADOPT 只能接管 branch/worktree/package/opening commit 四项均由现实唯一证明的现场', {
        pending: deps.pending.pendingStartDoctorView(state.record),
        resources: resourceList(state.record),
        reality: reconciliation.reality,
      }, deps);
    }
    const admission = admissionFor(
      checked.proposal,
      state.record.worktree.canonicalPath || state.record.worktree.requestedPath,
      snapshot,
      planes,
      deps,
      record.id,
    );
    if (!admission.ok) return admission.result;
    if (action === 'ADOPT') {
      return finalizeActive(state, checked.proposal, snapshot, settings, deps, 'ADOPT');
    }
    return runPhases(state, checked.proposal, snapshot, settings, deps, 'RESUME');
  } catch (error) {
    return failure(error && error.code ? error.code : 'PENDING_START_RECOVERY_FAILED', error && error.message ? error.message : String(error), null, deps);
  }
}

module.exports = {
  PROPOSAL_SCHEMA,
  START_PHASES,
  RECOVERY_ACTIONS,
  ACTION_PREVIEW,
  stableJson,
  digestProposal,
  propose,
  start,
  doctor,
  recover,
};
