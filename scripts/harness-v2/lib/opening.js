'use strict';

// Adaptive Harness v0.5 — 开工请求：五项齐备 + 与 Git 现实核对（AH-050-07）
//
// 需求溯源：
//   GIT-4  开工就写明：在哪条分支干、可以改哪些路径、禁止碰哪些路径、
//          允不允许本地提交、允不允许推送。五项缺任一项，开工判定失败。
//   GIT-1  一任务一分支一独立工作副本；同一个工作副本同时只有一件在执行。
//   §6.1   开工冻结 integration branch / base OID / task branch / worktree realpath
//   §6.4   每一次进入 ACTIVE 都重判——新开工、恢复、承接转交、接管半成品一律重判
//   WK-1   记录与现实冲突时报冲突；不得以文档覆盖现实，也不得静默把记录改写成现实值
//
// 本模块**不执行任何 Git 写操作**：它只读现实、只出判定与冻结值。
// 真正建分支、建工作副本、提交，由调用方在取得授权后执行。

const fs = require('node:fs');
const path = require('node:path');

const gitFacts = require('./git-facts');
const scope = require('./scope');

// ---------------------------------------------------------------------------
// GIT-4 的五项。五项是开工的**前置条件**，不是可以留空的表单格。
// ---------------------------------------------------------------------------

const OPENING_REQUIRED_FIELDS = Object.freeze([
  { key: 'task-branch', label: '在哪条分支干', type: 'text' },
  { key: 'write-scope', label: '可以改哪些路径', type: 'list-nonempty' },
  { key: 'forbidden-scope', label: '禁止碰哪些路径', type: 'list' },
  { key: 'local-commit-allowed', label: '允不允许本地提交', type: 'flag' },
  { key: 'push-allowed', label: '允不允许推送', type: 'flag' },
]);

// 开工那一刻冻结、并且必须能与版本库现实对上的绑定身份（§6.1 / GIT-1）。
const BINDING_IDENTITY_FIELDS = Object.freeze([
  'base-branch', 'base-oid', 'task-branch', 'worktree',
]);

const LIST_FIELDS = Object.freeze(['write-scope', 'forbidden-scope', 'exclusive-resources']);
const FLAG_FIELDS = Object.freeze(['local-commit-allowed', 'push-allowed']);

// ---------------------------------------------------------------------------
// 开工请求的读取
// ---------------------------------------------------------------------------

function splitInline(value) {
  return String(value)
    .replace(/^\[/, '')
    .replace(/\]$/, '')
    .split(',')
    .map((item) => item.trim().replace(/^["']|["']$/g, ''))
    .filter((item) => item !== '');
}

/**
 * 读一份开工请求。顶部 front matter 是机读部分，下面是给人看的说明。
 *
 * 本函数**不做任何默认填充**：请求里没写的字段读出来就是缺失（undefined），
 * 绝不替用户补一个全仓、通配或省略值——那样判据再对也等于全串行（§6.4）。
 */
function parseOpeningRequest(text) {
  const lines = String(text === null || text === undefined ? '' : text).replace(/\r\n/g, '\n').split('\n');
  if (lines[0] !== '---') {
    return { ok: false, fields: {}, body: String(text || ''), issues: [{ code: 'OPENING_FRONT_MATTER_MISSING', field: '', message: '开工请求顶部必须有 --- 包住的 front matter' }] };
  }
  let end = -1;
  for (let cursor = 1; cursor < lines.length; cursor += 1) {
    if (lines[cursor] === '---') { end = cursor; break; }
  }
  if (end === -1) {
    return { ok: false, fields: {}, body: '', issues: [{ code: 'OPENING_FRONT_MATTER_MALFORMED', field: '', message: 'front matter 没有收尾的 ---' }] };
  }

  const fields = {};
  let pendingKey = null;
  for (let cursor = 1; cursor < end; cursor += 1) {
    const line = lines[cursor];
    if (line.trim() === '' || line.trim().startsWith('#')) continue;
    const sequence = /^\s+-\s+(.*)$/.exec(line);
    if (sequence && pendingKey) {
      if (!Array.isArray(fields[pendingKey])) fields[pendingKey] = [];
      const item = sequence[1].trim().replace(/^["']|["']$/g, '');
      if (item !== '') fields[pendingKey].push(item);
      continue;
    }
    const match = /^([a-z][a-z0-9-]*):\s*(.*)$/.exec(line);
    if (!match) continue;
    const key = match[1];
    const raw = match[2].trim();
    pendingKey = key;
    if (raw === '') { fields[key] = LIST_FIELDS.includes(key) ? [] : null; continue; }
    if (LIST_FIELDS.includes(key)) {
      fields[key] = raw === '[]' ? [] : splitInline(raw);
      continue;
    }
    if (FLAG_FIELDS.includes(key)) {
      if (raw === 'true') { fields[key] = true; continue; }
      if (raw === 'false') { fields[key] = false; continue; }
      fields[key] = raw;
      continue;
    }
    fields[key] = raw.replace(/^["']|["']$/g, '');
  }

  return { ok: true, fields, body: lines.slice(end + 1).join('\n'), issues: [] };
}

// ---------------------------------------------------------------------------
// 五项齐备校验：缺任一项，开工判定失败，理由指名缺的是哪一项
// ---------------------------------------------------------------------------

/**
 * 五项——分支、可改路径、禁止路径、能否本地提交、能否推送——逐项检查。
 * 任何一项为空、缺失或写成非布尔值，都让开工失败并指名该项；
 * 不允许「先开工再补」。push-allowed 尤其不得省略：省略等于把推送权限留成未知，
 * 而推送是必须逐次单独授权的高危动作（GIT-10）。
 */
function validateOpeningRequest(fields) {
  const source = fields && typeof fields === 'object' ? fields : {};
  const issues = [];
  for (const spec of OPENING_REQUIRED_FIELDS) {
    const value = source[spec.key];
    if (value === undefined || value === null) {
      issues.push({
        code: 'OPENING_FIELD_MISSING',
        field: spec.key,
        message: `开工五项缺「${spec.label}」（${spec.key}）：字段缺失，开工判定失败`,
      });
      continue;
    }
    if (spec.type === 'text' && String(value).trim() === '') {
      issues.push({ code: 'OPENING_FIELD_EMPTY', field: spec.key, message: `开工五项的「${spec.label}」（${spec.key}）为空，开工判定失败` });
      continue;
    }
    if (spec.type === 'flag' && typeof value !== 'boolean') {
      issues.push({ code: 'OPENING_FIELD_NOT_EXPLICIT', field: spec.key, message: `开工五项的「${spec.label}」（${spec.key}）必须显式写成 true 或 false，收到「${value}」，开工判定失败` });
      continue;
    }
    if (spec.type === 'list-nonempty' && (!Array.isArray(value) || value.length === 0)) {
      issues.push({ code: 'OPENING_FIELD_EMPTY', field: spec.key, message: `开工五项的「${spec.label}」（${spec.key}）为空表，开工判定失败；生成不出可用范围时请在开工请求里补齐` });
      continue;
    }
    if (spec.type === 'list' && !Array.isArray(value)) {
      issues.push({ code: 'OPENING_FIELD_MALFORMED', field: spec.key, message: `开工五项的「${spec.label}」（${spec.key}）必须是列表（可显式写成 []），开工判定失败` });
    }
  }
  for (const key of BINDING_IDENTITY_FIELDS) {
    const value = source[key];
    if (typeof value !== 'string' || value.trim() === '') {
      issues.push({ code: 'OPENING_BINDING_MISSING', field: key, message: `开工即冻结的绑定身份 ${key} 缺失，开工判定失败` });
    }
  }
  if (source['exclusive-resources'] === undefined || source['exclusive-resources'] === null) {
    issues.push({
      code: 'OPENING_FIELD_MISSING',
      field: 'exclusive-resources',
      message: '独占资源清单必填、可显式为空；字段缺失按与一切相交处理，开工判定失败',
    });
  }
  return issues;
}

// ---------------------------------------------------------------------------
// 与 Git 现实核对
// ---------------------------------------------------------------------------

function realpathOrNull(value) {
  try {
    return fs.realpathSync(String(value));
  } catch (error) {
    return null;
  }
}

function safeTargetBasename(value) {
  const text = typeof value === 'string' ? value : '';
  return text !== ''
    && text !== '.'
    && text !== '..'
    && !text.includes('/')
    && !text.includes('\\')
    && !text.includes('\0');
}

/**
 * worktree 尚不存在时只能信任两样：已存在父目录的 realpath 和一个安全 basename。
 * 输入如果经 symlink、`..` 或别名才到同一位置，直接拒绝；创建后还会再读 Git 现实。
 */
function inspectWorktreeTargetBeforeCreate(input) {
  const settings = input || {};
  const requested = typeof settings.worktree === 'string' ? settings.worktree : '';
  const conflicts = [];
  if (!path.isAbsolute(requested)) {
    conflicts.push({
      code: 'WORKTREE_TARGET_NOT_ABSOLUTE',
      field: 'worktree',
      message: 'linked worktree 目标必须是绝对路径',
    });
    return { ok: false, requested, canonicalPath: null, parentRealpath: null, conflicts };
  }
  const resolved = path.resolve(requested);
  const basename = path.basename(resolved);
  if (!safeTargetBasename(basename)) {
    conflicts.push({
      code: 'WORKTREE_TARGET_BASENAME_INVALID',
      field: 'worktree',
      message: 'linked worktree 目标必须以单一安全目录名结尾',
    });
    return { ok: false, requested, canonicalPath: null, parentRealpath: null, conflicts };
  }
  let parentRealpath = null;
  try {
    parentRealpath = fs.realpathSync(path.dirname(resolved));
    if (!fs.statSync(parentRealpath).isDirectory()) {
      throw new Error('父路径不是目录');
    }
  } catch (error) {
    conflicts.push({
      code: 'WORKTREE_PARENT_UNRESOLVED',
      field: 'worktree',
      message: `linked worktree 的父目录无法唯一解析：${error.message}`,
    });
    return { ok: false, requested, canonicalPath: null, parentRealpath: null, conflicts };
  }
  const canonicalPath = path.join(parentRealpath, basename);
  if (resolved !== canonicalPath || requested !== resolved) {
    conflicts.push({
      code: 'WORKTREE_TARGET_ALIAS_FORBIDDEN',
      field: 'worktree',
      message: `worktree 输入 ${requested} 不是父目录 realpath 下的 canonical path ${canonicalPath}`,
    });
  }
  try {
    fs.lstatSync(canonicalPath);
    conflicts.push({
      code: 'WORKTREE_TARGET_ALREADY_EXISTS',
      field: 'worktree',
      message: `worktree 目标 ${canonicalPath} 已存在，不能静默接管`,
    });
  } catch (error) {
    if (!error || error.code !== 'ENOENT') {
      conflicts.push({
        code: 'WORKTREE_TARGET_UNREADABLE',
        field: 'worktree',
        message: `无法确认 worktree 目标是否存在：${error.message}`,
      });
    }
  }

  let registered = [];
  try {
    registered = gitFacts.worktreeList(settings.cwd || process.cwd(), settings.gitOptions);
  } catch (error) {
    conflicts.push({
      code: 'GIT_WORKTREE_LIST_UNAVAILABLE',
      field: 'worktree',
      message: error.message,
    });
  }
  const listed = registered.filter((entry) => path.resolve(entry.worktree || '') === canonicalPath);
  if (listed.length > 0) {
    conflicts.push({
      code: 'WORKTREE_TARGET_ALREADY_REGISTERED',
      field: 'worktree',
      message: `git worktree list 已登记 ${canonicalPath}，不能重复建立`,
    });
  }
  return {
    ok: conflicts.length === 0,
    requested,
    canonicalPath,
    parentRealpath,
    registered,
    conflicts,
  };
}

/**
 * 创建后不再信任 proposal 字符串：以 fs realpath、Git worktree 清单、
 * repo root、当前 branch 和 base ancestry 五方交叉核对。
 */
function reconcileCreatedWorktree(input) {
  const settings = input || {};
  const worktree = typeof settings.worktree === 'string' ? settings.worktree : '';
  const branch = String(settings.branch || '').replace(/^refs\/heads\//, '');
  const baseOid = typeof settings.baseOid === 'string' ? settings.baseOid : '';
  const expectedCanonical = settings.canonicalPath || worktree;
  const conflicts = [];
  const facts = {
    canonicalWorktree: realpathOrNull(worktree),
    repoRoot: null,
    currentBranch: null,
    headOid: null,
    listed: null,
    containsBase: false,
  };
  const expectedReal = path.resolve(expectedCanonical);
  if (!facts.canonicalWorktree || facts.canonicalWorktree !== expectedReal) {
    conflicts.push({
      code: 'WORKTREE_REALPATH_MISMATCH',
      field: 'worktree',
      message: `创建后的 worktree realpath ${facts.canonicalWorktree || 'unknown'} 与冻结值 ${expectedReal} 不一致`,
    });
  }
  try {
    facts.repoRoot = realpathOrNull(gitFacts.repoRoot(worktree));
  } catch (error) {
    conflicts.push({ code: 'WORKTREE_REPO_ROOT_UNREADABLE', field: 'worktree', message: error.message });
  }
  if (!facts.repoRoot || facts.repoRoot !== facts.canonicalWorktree) {
    conflicts.push({
      code: 'WORKTREE_REPO_ROOT_MISMATCH',
      field: 'worktree',
      message: `Git repo root ${facts.repoRoot || 'unknown'} 与 worktree realpath 不一致`,
    });
  }
  try {
    const list = gitFacts.worktreeList(settings.cwd || worktree, settings.gitOptions);
    const matches = list.filter((entry) => realpathOrNull(entry.worktree) === facts.canonicalWorktree);
    if (matches.length === 1) facts.listed = matches[0];
    else {
      conflicts.push({
        code: 'WORKTREE_LIST_IDENTITY_AMBIGUOUS',
        field: 'worktree',
        message: `git worktree list 对该 realpath 命中 ${matches.length} 条，身份不唯一`,
      });
    }
  } catch (error) {
    conflicts.push({ code: 'GIT_WORKTREE_LIST_UNAVAILABLE', field: 'worktree', message: error.message });
  }
  const current = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: worktree });
  facts.currentBranch = current.ok ? current.stdout.trim() : null;
  facts.headOid = gitFacts.headOid(worktree);
  if (facts.currentBranch !== branch) {
    conflicts.push({
      code: 'WORKTREE_BRANCH_MISMATCH',
      field: 'task-branch',
      message: `创建后的 worktree 当前 branch 是 ${facts.currentBranch || 'unknown'}，冻结值是 ${branch}`,
    });
  }
  if (facts.listed && facts.listed.branch !== `refs/heads/${branch}`) {
    conflicts.push({
      code: 'WORKTREE_LIST_BRANCH_MISMATCH',
      field: 'task-branch',
      message: `git worktree list 的 branch 是 ${facts.listed.branch || 'unknown'}，冻结值是 refs/heads/${branch}`,
    });
  }
  facts.containsBase = Boolean(baseOid) && gitFacts.isAncestor(worktree, baseOid, 'HEAD');
  if (!facts.containsBase) {
    conflicts.push({
      code: 'WORKTREE_BASE_MISMATCH',
      field: 'base-oid',
      message: `创建后的 HEAD 不包含冻结 base OID ${baseOid || '（空）'}`,
    });
  }
  return { ok: conflicts.length === 0, facts, conflicts };
}

/**
 * 冻结的绑定身份必须能与版本库现实对上，对不上就**报冲突并拒绝**继续。
 *
 * 这里读的是 task-branch、worktree 与 base-oid 三样冻结值，
 * 逐样拿真正的 git 查询去核对：
 *   * worktree —— git 报的仓库根 realpath 必须与记录里的 worktree 完全一致；
 *   * base-oid —— 必须是本仓真实可达的提交对象；
 *   * task-branch —— 已经存在时必须包含 base-oid，不存在则如实报「尚未建立」。
 * 任何一处不一致都返回 conflicts 并让开工失败；本函数**绝不**把记录改写成现实值。
 */
function reconcileWithGit(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const fields = settings.fields && typeof settings.fields === 'object' ? settings.fields : {};
  const conflicts = [];
  const facts = {
    repoRoot: null,
    headOid: null,
    currentBranch: null,
    baseOidExists: false,
    baseBranchExists: false,
    taskBranchExists: false,
    taskBranchContainsBase: null,
  };

  try {
    facts.repoRoot = gitFacts.repoRoot(cwd);
  } catch (error) {
    conflicts.push({ code: 'GIT_REALITY_UNREADABLE', field: 'worktree', message: `读不到版本库现实，报冲突并停：${error.message}` });
    return { ok: false, facts, conflicts };
  }
  facts.headOid = gitFacts.headOid(cwd);
  const branch = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd });
  facts.currentBranch = branch.ok ? branch.stdout.trim() : null;

  const declaredWorktree = realpathOrNull(fields.worktree);
  const actualWorktree = realpathOrNull(facts.repoRoot);
  if (declaredWorktree === null || actualWorktree === null || declaredWorktree !== actualWorktree) {
    conflicts.push({
      code: 'GIT_WORKTREE_MISMATCH',
      field: 'worktree',
      message: `记录里的 worktree 是 ${fields.worktree}，这里的工作副本 realpath 是 ${facts.repoRoot}；两个说法对不上，拒绝继续`,
    });
  }

  facts.baseOidExists = gitFacts.objectExists(cwd, fields['base-oid']);
  if (!facts.baseOidExists) {
    conflicts.push({
      code: 'GIT_BASE_OID_UNREACHABLE',
      field: 'base-oid',
      message: `记录里的 base-oid ${fields['base-oid']} 在本仓不可达；拒绝继续，不把记录静默改写成现实值`,
    });
  }

  facts.baseBranchExists = gitFacts.runGit(['rev-parse', '--verify', `${fields['base-branch']}^{commit}`], { cwd }).ok;
  if (!facts.baseBranchExists) {
    conflicts.push({
      code: 'GIT_BASE_BRANCH_UNKNOWN',
      field: 'base-branch',
      message: `记录里的 base-branch ${fields['base-branch']} 在本仓解析不出来，拒绝继续`,
    });
  }

  facts.taskBranchExists = gitFacts.runGit(['rev-parse', '--verify', `${fields['task-branch']}^{commit}`], { cwd }).ok;
  if (facts.taskBranchExists && facts.baseOidExists) {
    facts.taskBranchContainsBase = gitFacts.isAncestor(cwd, fields['base-oid'], fields['task-branch']);
    if (facts.taskBranchContainsBase !== true) {
      conflicts.push({
        code: 'GIT_TASK_BRANCH_NOT_ON_BASE',
        field: 'task-branch',
        message: `task-branch ${fields['task-branch']} 不包含冻结的 base-oid ${fields['base-oid']}，绑定与现实冲突，拒绝继续`,
      });
    }
  }

  return { ok: conflicts.length === 0, facts, conflicts };
}

// ---------------------------------------------------------------------------
// 开工判定
// ---------------------------------------------------------------------------

/**
 * 一次完整的开工判定。三段各自独立，**都要过**：
 *   1. 五项齐备（GIT-4）；
 *   2. 冻结的绑定身份与 Git 现实一致（GIT-1 / WK-1）；
 *   3. 写面与独占资源的相交判定（GIT-2，见 lib/scope.js）。
 *
 * `reason` 说明本次为什么要判：START / RESUME / ADOPT / TRANSFER_IN。
 * 四种情形都必须走完整的三段——每一次进入 ACTIVE 都重判，
 * 只在首次开工判一次的实现不合格（§6.4）。
 */
const ADMISSION_REASONS = Object.freeze(['START', 'RESUME', 'ADOPT', 'TRANSFER_IN']);

function evaluateOpening(input) {
  const settings = input || {};
  const fields = settings.fields && typeof settings.fields === 'object' ? settings.fields : {};
  const reason = ADMISSION_REASONS.includes(settings.reason) ? settings.reason : 'START';

  const fieldIssues = validateOpeningRequest(fields);
  const reality = settings.skipGitReality === true
    ? { ok: true, facts: settings.facts || {}, conflicts: [] }
    : reconcileWithGit({ cwd: settings.cwd, fields });

  const admission = scope.decideAdmission({
    request: {
      id: fields['task-id'] || settings.taskId || null,
      worktree: fields.worktree || null,
      'write-scope': fields['write-scope'],
      'forbidden-scope': fields['forbidden-scope'],
      'exclusive-resources': fields['exclusive-resources'],
    },
    registered: settings.registered || [],
    controlPlaneExempt: settings.controlPlaneExempt,
    dependsOn: settings.dependsOn,
  });

  const ok = fieldIssues.length === 0 && reality.ok && admission.admitted;
  return {
    ok,
    reason,
    fieldIssues,
    conflicts: reality.conflicts,
    facts: reality.facts,
    admission,
    binding: ok ? freezeBinding(fields) : null,
  };
}

/** 判定通过之后冻结成任务节点的 git binding。此后这份 binding 是唯一说法。 */
function freezeBinding(fields) {
  const source = fields && typeof fields === 'object' ? fields : {};
  return {
    'base-branch': source['base-branch'],
    'base-oid': source['base-oid'],
    'task-branch': source['task-branch'],
    worktree: source.worktree,
    'local-commit-allowed': source['local-commit-allowed'] === true,
    'push-allowed': source['push-allowed'] === true,
  };
}

/**
 * 把在册节点折成判定输入。
 * 参与与否只看 lifecycle.participatesInScopeJudgement：尚未取得分支与工作副本的
 * 草稿与就绪叶子一律不参与，否则刚建好的一批叶子会互相预占写面，谁都起不来。
 */
function declarationsFromNodes(records, participates) {
  const list = Array.isArray(records) ? records : [];
  const out = [];
  for (const record of list) {
    const node = record && record.node ? record.node : record;
    if (!node || node.kind !== 'TASK') continue;
    const binding = node.git && typeof node.git === 'object' ? node.git : null;
    out.push({
      id: node.id,
      worktree: binding ? binding.worktree : null,
      'write-scope': node['write-scope'],
      'forbidden-scope': node['forbidden-scope'],
      'exclusive-resources': node['exclusive-resources'],
      participates: typeof participates === 'function' ? participates(node) === true : Boolean(binding),
    });
  }
  return out;
}

module.exports = {
  OPENING_REQUIRED_FIELDS,
  BINDING_IDENTITY_FIELDS,
  ADMISSION_REASONS,
  parseOpeningRequest,
  validateOpeningRequest,
  inspectWorktreeTargetBeforeCreate,
  reconcileCreatedWorktree,
  reconcileWithGit,
  evaluateOpening,
  freezeBinding,
  declarationsFromNodes,
};
