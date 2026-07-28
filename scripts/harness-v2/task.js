#!/usr/bin/env node
'use strict';

// Adaptive Harness v0.5 — TASK 生命周期与退场 CLI（AH-050-05 / AH-050-08）
//
// 这个文件是薄壳：它只解析参数、读取同一份 live snapshot 与 Git 现实，
// 把纯 task-lifecycle 的 change set 交给 store。它不做 fast-forward、不删
// branch/worktree；08 只在确认收据仍匹配、且专用 integration worktree 现实
// 成立时写 canonical history。

const fs = require('node:fs');
const path = require('node:path');

const nodeSchema = require('./lib/node-schema');
const lifecycle = require('./lib/lifecycle');
const graph = require('./lib/graph');
const store = require('./lib/store');
const opening = require('./lib/opening');
const gitFacts = require('./lib/git-facts');
const scope = require('./lib/scope');
const taskLifecycle = require('./lib/task-lifecycle');
const taskCloseout = require('./lib/task-closeout');
const integrationGate = require('./lib/integration-gate');
const taskStart = require('./lib/task-start');
const routing = require('./lib/routing');
const evidenceTrace = require('./lib/evidence-trace');
const taskRecord = require('./lib/task-record');

const COMMANDS = Object.freeze([
  'create',
  'ready',
  'activate',
  'park',
  'resume',
  'withdraw',
  'finish',
  'cancel',
  'stop',
  'split',
  'replace',
  'propose',
  'start',
  'record',
]);

function usage() {
  return [
    `用法：task.js <${COMMANDS.join('|')}> [--help]`,
    '[--id <编号>] [--candidate <Markdown>] [--successor <Markdown>]',
    '[--result <COMPLETED|STOPPED|CANCELLED>] [--split-reason <原因>] [--replace-reason <CONTRACT_INVALID>]',
    '[--disposition <RETAINED|TRANSFERRED|READY_FOR_CONFIRMED_REMOVAL|REMOVED>] [--no-product-change]',
    '[--closeout <JSON>] [--inspect | --write --receipt <sha256>]',
    '[record --record <JSON|文件> | --product-commit <OID> | --wip-commit <OID>]',
    '[record --disposition <...> --verification-run <JSON|文件> ...]',
    '[--target <目录>]',
    '[--integration-ref <refs/heads/...>] [--closed-at <YYYY-MM-DD>]',
    '[propose --request <文本文件>|--request-stdin --profile <READ_ONLY|ORDINARY_LOCAL|STRICT_LOCAL> --verification <JSON数组|文件>]',
    '[start [doctor|recover] --proposal <JSON> --proposal-digest <sha256> --pending <ID> --action <...>]',
  ].join(' ');
}

function parseArgumentsUnsafe(argv) {
  const options = {
    command: null,
    startMode: null,
    id: null,
    candidatePath: null,
    successorPaths: [],
    result: null,
    splitReason: null,
    replaceReason: null,
    disposition: null,
    noProductChange: false,
    closedAt: null,
    closeoutPath: null,
    inspect: false,
    receipt: null,
    cwd: process.cwd(),
    integrationRef: null,
    json: false,
    write: false,
    proposalPath: null,
    proposalDigest: null,
    requestPath: null,
    requestStdin: false,
    profile: null,
    parentId: null,
    baseBranch: null,
    baseOid: null,
    taskBranch: null,
    worktree: null,
    writeScope: [],
    forbiddenScope: [],
    exclusiveResources: [],
    taskPackagePath: null,
    goal: null,
    acceptanceCriteria: [],
    localCommitAllowed: null,
    pushAllowed: null,
    pendingId: null,
    recoveryAction: null,
    help: false,
    proposalVerificationPath: undefined,
    recordPath: null,
    productCommit: undefined,
    wipCommit: undefined,
    verificationRunInputs: [],
  };
  const list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length > 0 && !list[0].startsWith('--')) options.command = list.shift();
  if (options.command === 'start' && list.length > 0 && !list[0].startsWith('--')) {
    const mode = list.shift();
    if (!['doctor', 'recover'].includes(mode)) {
      return { ok: false, error: `start 只接受 doctor 或 recover 子命令，收到 ${mode}`, options };
    }
    options.startMode = mode;
  }
  while (list.length > 0) {
    const token = list.shift();
    if (token === '--help' || token === '-h') { options.help = true; continue; }
    if (token === '--write') { options.write = true; continue; }
    if (token === '--request-stdin') { options.requestStdin = true; continue; }
    if (token === '--local-commit-allowed') { options.localCommitAllowed = true; continue; }
    if (token === '--push-allowed') {
      if (options.pushAllowed === false) return { ok: false, error: '--push-allowed 与 --no-push 互斥', options };
      options.pushAllowed = true;
      continue;
    }
    if (token === '--no-push') {
      if (options.pushAllowed === true) return { ok: false, error: '--push-allowed 与 --no-push 互斥', options };
      options.pushAllowed = false;
      continue;
    }
    if (token === '--inspect') { options.inspect = true; continue; }
    if (token === '--json') { options.json = true; continue; }
    if (token === '--no-product-change') { options.noProductChange = true; continue; }
    if (token === '--id' || token === '--task') { options.id = list.shift() || null; continue; }
    if (token === '--pending') { options.pendingId = list.shift() || null; continue; }
    if (token === '--proposal') { options.proposalPath = list.shift() || null; continue; }
    if (token === '--proposal-digest') { options.proposalDigest = list.shift() || null; continue; }
    if (token === '--request') { options.requestPath = list.shift() || null; continue; }
    if (token === '--verification' || token === '--verification-file') {
      if (options.proposalVerificationPath !== undefined) {
        return { ok: false, error: '--verification 只能给一次；proposal 只冻结一份验证合同', options };
      }
      options.proposalVerificationPath = list.shift() || null;
      continue;
    }
    if (token === '--profile') { options.profile = list.shift() || null; continue; }
    if (token === '--parent') { options.parentId = list.shift() || null; continue; }
    if (token === '--base-branch') { options.baseBranch = list.shift() || null; continue; }
    if (token === '--base-oid') { options.baseOid = list.shift() || null; continue; }
    if (token === '--task-branch') { options.taskBranch = list.shift() || null; continue; }
    if (token === '--worktree') { options.worktree = list.shift() || null; continue; }
    if (token === '--write-scope') { options.writeScope.push(list.shift() || null); continue; }
    if (token === '--forbidden-scope') { options.forbiddenScope.push(list.shift() || null); continue; }
    if (token === '--exclusive-resource') { options.exclusiveResources.push(list.shift() || null); continue; }
    if (token === '--task-package') { options.taskPackagePath = list.shift() || null; continue; }
    if (token === '--goal') { options.goal = list.shift() || null; continue; }
    if (token === '--acceptance') { options.acceptanceCriteria.push(list.shift() || null); continue; }
    if (token === '--action') { options.recoveryAction = list.shift() || null; continue; }
    if (token === '--candidate') { options.candidatePath = list.shift() || null; continue; }
    if (token === '--successor') { options.successorPaths.push(list.shift() || null); continue; }
    if (token === '--result') { options.result = list.shift() || null; continue; }
    if (token === '--split-reason') { options.splitReason = list.shift() || null; continue; }
    if (token === '--replace-reason') { options.replaceReason = list.shift() || null; continue; }
    if (token === '--disposition') { options.disposition = list.shift() || null; continue; }
    if (token === '--closed-at') { options.closedAt = list.shift() || null; continue; }
    if (token === '--closeout') { options.closeoutPath = list.shift() || null; continue; }
    if (token === '--record' || token === '--record-file') { options.recordPath = list.shift() || null; continue; }
    if (token === '--product-commit') { options.productCommit = list.shift() || null; continue; }
    if (token === '--wip-commit') { options.wipCommit = list.shift() || null; continue; }
    if (token === '--verification-run' || token === '--run') { options.verificationRunInputs.push(list.shift() || null); continue; }
    if (token === '--receipt') { options.receipt = list.shift() || null; continue; }
    if (token === '--target') { options.cwd = list.shift() || options.cwd; continue; }
    if (token === '--integration-ref') { options.integrationRef = list.shift() || null; continue; }
    return { ok: false, error: `未知参数 ${token}`, options };
  }
  if (options.help) {
    if (options.command && !COMMANDS.includes(options.command)) {
      return { ok: false, error: usage(), options };
    }
    return { ok: true, error: null, options };
  }
  if (!options.command || !COMMANDS.includes(options.command)) {
    return { ok: false, error: usage(), options };
  }
  if (options.successorPaths.some((item) => !item)) {
    return { ok: false, error: '--successor 必须给出 Markdown 路径', options };
  }
  if (options.requestPath && options.requestStdin) {
    return { ok: false, error: '--request 与 --request-stdin 互斥', options };
  }
  if (options.inspect && options.write) {
    return { ok: false, error: '--inspect 与 --write 互斥', options };
  }
  if (options.receipt && !/^[0-9a-f]{64}$/i.test(options.receipt)) {
    return { ok: false, error: '--receipt 必须是 64 位十六进制 sha256', options };
  }
  if (options.receipt && !options.write) {
    return { ok: false, error: '--receipt 只可与 --write 一起使用', options };
  }
  if (options.proposalDigest && !/^[0-9a-f]{64}$/i.test(options.proposalDigest)) {
    return { ok: false, error: '--proposal-digest 必须是 64 位十六进制 sha256', options };
  }
  return { ok: true, error: null, options };
}

function publicResult(value) {
  return routing.redactKnownSecrets(value);
}

function parseArguments(argv) {
  try {
    return publicResult(parseArgumentsUnsafe(argv));
  } catch (error) {
    return publicResult({
      ok: false,
      error: error && error.message ? error.message : String(error),
      options: null,
    });
  }
}

function loadPlanes(options, runtime) {
  if (runtime && runtime.planes) return runtime.planes;
  return store.resolvePlanes({
    cwd: options.cwd,
    integrationRef: options.integrationRef,
  });
}

function snapshotPrecondition(planes) {
  const snapshot = store.readLiveSnapshot(planes);
  const schema = [];
  for (const record of snapshot.records) {
    for (const issue of record.issues || []) schema.push({ path: record.path, ...issue });
  }
  const index = graph.buildGraphIndex(snapshot.records);
  const integrity = graph.graphIntegrityIssues(index);
  if (schema.length > 0 || integrity.length > 0) {
    return {
      ok: false,
      code: 'LIVE_GRAPH_PRECONDITION_FAILED',
      error: 'live graph 含坏节点；先修复被点名的问题，本次不规划写入',
      snapshot,
      index,
      schema,
      integrity,
    };
  }
  return { ok: true, snapshot, index };
}

function requireId(options) {
  return options.id
    ? null
    : { ok: false, code: 'TASK_ID_REQUIRED', error: `${options.command} 需要 --id` };
}

function requireCandidate(options, flag) {
  const candidatePath = options.candidatePath;
  if (!candidatePath) {
    return { ok: false, code: 'TASK_CANDIDATE_REQUIRED', error: `${flag || options.command} 需要 --candidate <Markdown>` };
  }
  return null;
}

function requireExplicitIntegrationRef(planes, options, command) {
  if (planes.tracked && !options.integrationRef) {
    return {
      ok: false,
      code: 'INTEGRATION_REF_REQUIRED',
      error: `${command} 必须显式提供 --integration-ref；历史编号只认固定引用，不能默认猜 main`,
    };
  }
  return null;
}

function readCandidate(filePath, cwd) {
  if (!filePath) return { ok: false, code: 'TASK_CANDIDATE_REQUIRED', error: '需要 candidate Markdown' };
  const absolute = path.resolve(cwd, filePath);
  let text;
  try {
    text = fs.readFileSync(absolute, 'utf8');
  } catch (error) {
    return { ok: false, code: 'TASK_CANDIDATE_READ_FAILED', error: `读不到 TASK candidate ${absolute}：${error.message}` };
  }
  return {
    ok: true,
    absolute,
    candidate: nodeSchema.parseNode(text, {
      relativePath: absolute,
      lifecycleValues: lifecycle.LIFECYCLE_VALUES,
    }),
  };
}

function historyExists(planes, options) {
  return (id) => Boolean(store.readHistoryNode(planes, id, { cwd: options.cwd }));
}

function findTaskRecord(index, id) {
  const record = index && index.byId ? index.byId.get(id) : null;
  if (!record) return { ok: false, code: 'TASK_NOT_FOUND', error: `在办平面里找不到 TASK ${id}` };
  if (!record.node || record.node.kind !== 'TASK') {
    return { ok: false, code: 'TASK_KIND_INVALID', error: `${id} 不是 TASK，不能走 task lifecycle` };
  }
  return { ok: true, record };
}

function actualOpeningReality(record, options) {
  const node = record.node;
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  return opening.reconcileWithGit({
    cwd: options.cwd,
    fields: {
      ...binding,
      'task-id': node.id,
      'write-scope': node['write-scope'],
      'forbidden-scope': node['forbidden-scope'],
      'exclusive-resources': node['exclusive-resources'],
    },
  });
}

function branchName(value) {
  return String(value || '').replace(/^refs\/heads\//, '');
}

function declaredTaskBranchReality(record, options, action) {
  const reality = actualOpeningReality(record, options);
  if (!reality.ok) return reality;
  const binding = record.node.git && typeof record.node.git === 'object' ? record.node.git : {};
  const declaredBranch = branchName(binding['task-branch']);
  const currentBranch = branchName(reality.facts && reality.facts.currentBranch);
  if (reality.facts.taskBranchExists !== true || currentBranch !== declaredBranch) {
    return {
      ...reality,
      ok: false,
      conflicts: [...(reality.conflicts || []), {
        code: 'GIT_TASK_BRANCH_MISMATCH',
        field: 'task-branch',
        message: `${action} 必须在声明的 task-branch ${binding['task-branch']} 上；当前为 ${reality.facts.currentBranch || 'detached/unknown'}`,
      }],
    };
  }
  return reality;
}

function activeOpeningReality(record, options) {
  return declaredTaskBranchReality(record, options, '进入 ACTIVE');
}

function hasGitBinding(node) {
  return Boolean(node && node.git && typeof node.git === 'object' && !Array.isArray(node.git));
}

function terminalGitReality(record, options) {
  if (!hasGitBinding(record.node)) return { ok: true, reality: null };
  // 终态读取声明 task worktree 本身，而不是调用者 cwd。这样从 integration
  // worktree 写 history 时仍会核对 task branch 的实际 checkout / base / HEAD。
  const reality = taskCloseout.inspectTaskBindingReality({ node: record.node, cwd: options.cwd });
  if (reality.ok) return { ok: true, reality };
  return {
    ok: false,
    code: 'TASK_GIT_REALITY_CONFLICT',
    error: 'Git 现实无法与 TASK binding 对上，拒绝进入终态判定',
    detail: reality.detail || { conflicts: [], facts: {} },
  };
}

function strictDescendant(cwd, baseOid, commitOid) {
  return gitFacts.isAncestor(cwd, baseOid, commitOid)
    && !gitFacts.isAncestor(cwd, commitOid, baseOid);
}

function sameCommit(cwd, left, right) {
  return gitFacts.isAncestor(cwd, left, right)
    && gitFacts.isAncestor(cwd, right, left);
}

function isCanonicalCommitOid(cwd, value) {
  if (typeof value !== 'string' || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(value)) {
    return false;
  }
  const resolved = gitFacts.runGit([
    'rev-parse', '--verify', '--end-of-options', `${value}^{commit}`,
  ], { cwd });
  return resolved.ok && resolved.stdout.trim().toLowerCase() === value.toLowerCase();
}

function changesStayInsideFrozenScope(node, changedPaths) {
  const classified = scope.classifyOutOfScopePaths({
    changedPaths,
    declaration: node,
    registered: [],
  });
  return classified.inForbidden.length === 0
    && classified.collidingWithOthers.length === 0
    && classified.backfillable.length === 0;
}

function readRecordJson(value, cwd, label) {
  const candidate = typeof value === 'string' ? value.trim() : '';
  if (candidate === '') {
    return { ok: false, code: 'TASK_RECORD_INPUT_REQUIRED', error: `${label} 需要 JSON object 或 JSON 文件路径` };
  }
  let raw = candidate;
  let source = 'inline';
  if (!candidate.startsWith('{')) {
    const absolute = path.resolve(cwd, candidate);
    try {
      raw = fs.readFileSync(absolute, 'utf8');
      source = absolute;
    } catch (error) {
      return {
        ok: false,
        code: 'TASK_RECORD_INPUT_READ_FAILED',
        error: `读不到 ${label} ${absolute}：${error.message}`,
      };
    }
  }
  try {
    const valueObject = JSON.parse(raw);
    if (!valueObject || typeof valueObject !== 'object' || Array.isArray(valueObject)) {
      return { ok: false, code: 'TASK_RECORD_INPUT_INVALID', error: `${label} 顶层必须是 JSON object` };
    }
    return { ok: true, value: valueObject, source };
  } catch (error) {
    return { ok: false, code: 'TASK_RECORD_INPUT_INVALID', error: `${label} 不是合法 JSON：${error.message}` };
  }
}

// propose 的验证合同可内联成 JSON 数组，也可从一个 JSON 文件读取；它不是
// record payload，故只接受数组，且在开工前不能携带任何已执行的 run 事实。
function readJsonArray(value, cwd, label) {
  const candidate = typeof value === 'string' ? value.trim() : '';
  if (candidate === '') {
    return { ok: false, code: 'PROPOSE_VERIFICATION_REQUIRED', error: `${label} 需要 JSON 数组或 JSON 文件路径` };
  }
  let raw = candidate;
  let source = 'inline';
  if (!candidate.startsWith('[')) {
    const absolute = path.resolve(cwd, candidate);
    try {
      raw = fs.readFileSync(absolute, 'utf8');
      source = absolute;
    } catch (error) {
      return {
        ok: false,
        code: 'PROPOSE_VERIFICATION_READ_FAILED',
        error: `读不到 ${label} ${absolute}：${error.message}`,
      };
    }
  }
  try {
    const array = JSON.parse(raw);
    if (!Array.isArray(array)) {
      return { ok: false, code: 'PROPOSE_VERIFICATION_INVALID', error: `${label} 顶层必须是 JSON 数组` };
    }
    return { ok: true, value: array, source };
  } catch (error) {
    return { ok: false, code: 'PROPOSE_VERIFICATION_INVALID', error: `${label} 不是合法 JSON：${error.message}` };
  }
}

function proposalVerificationFromOptions(options) {
  const loaded = readJsonArray(options.proposalVerificationPath, options.cwd, '--verification');
  if (!loaded.ok) return loaded;
  if (loaded.value.length === 0) {
    return { ok: false, code: 'PROPOSE_VERIFICATION_INVALID', error: '--verification 至少要冻结一条检查' };
  }
  const entries = [];
  const seen = new Set();
  let hasRequired = false;
  for (let index = 0; index < loaded.value.length; index += 1) {
    const source = loaded.value[index];
    if (!isPlainObject(source)) {
      return { ok: false, code: 'PROPOSE_VERIFICATION_INVALID', error: `--verification #${index + 1} 必须是 JSON object` };
    }
    const allowed = new Set(['id', 'command', 'required', 'status']);
    for (const key of Object.keys(source)) {
      if (!allowed.has(key)) {
        return {
          ok: false,
          code: 'PROPOSE_VERIFICATION_INVALID',
          error: `--verification #${index + 1} 不接受字段 ${key}；run/output 必须在 ACTIVE 后单独 record`,
        };
      }
    }
    const id = typeof source.id === 'string' ? source.id.trim() : '';
    const command = typeof source.command === 'string' ? source.command.trim() : '';
    if (id === '' || source.id !== id || command === '' || source.command !== command
      || typeof source.required !== 'boolean' || source.status !== 'UNKNOWN') {
      return {
        ok: false,
        code: 'PROPOSE_VERIFICATION_INVALID',
        error: `--verification #${index + 1} 必须是 {id, command, required:boolean, status:"UNKNOWN"}`,
      };
    }
    if (seen.has(id)) {
      return { ok: false, code: 'PROPOSE_VERIFICATION_INVALID', error: `--verification id 重复：${id}` };
    }
    seen.add(id);
    if (source.required === true) hasRequired = true;
    entries.push({ id, command, required: source.required, status: 'UNKNOWN' });
  }
  if (!hasRequired) {
    return {
      ok: false,
      code: 'PROPOSE_REQUIRED_VERIFICATION_MISSING',
      error: '--verification 至少要有一条 required:true；未来 start 不接受零 required 合同',
    };
  }
  return { ok: true, entries, source: loaded.source };
}

function recordRequestFromOptions(options) {
  const hasDirectCarrier = options.productCommit !== undefined || options.wipCommit !== undefined;
  const hasDirectRun = Array.isArray(options.verificationRunInputs) && options.verificationRunInputs.length > 0;
  const hasDirectDisposition = options.disposition !== null && options.disposition !== undefined;
  if (options.recordPath && (hasDirectCarrier || hasDirectRun || hasDirectDisposition)) {
    return {
      ok: false,
      code: 'TASK_RECORD_INPUT_CONFLICT',
      error: '--record 与 product/WIP/disposition/verification-run 直传参数不可混用；请选择一个完整输入面',
    };
  }
  let payload = {};
  if (options.recordPath) {
    const loaded = readRecordJson(options.recordPath, options.cwd, '--record');
    if (!loaded.ok) return loaded;
    payload = loaded.value;
  } else {
    if (options.productCommit !== undefined) payload['product-commit'] = options.productCommit;
    if (options.wipCommit !== undefined) payload['wip-commit'] = options.wipCommit;
    if (hasDirectDisposition) payload.disposition = options.disposition;
    if (hasDirectRun) {
      const runs = [];
      for (const raw of options.verificationRunInputs) {
        const loaded = readRecordJson(raw, options.cwd, '--verification-run');
        if (!loaded.ok) return loaded;
        runs.push(loaded.value);
      }
      payload['verification-runs'] = runs;
    }
  }
  return taskRecord.normalizeRequest(payload);
}

function carrierRecordFacts(worktree, node, binding, bindingReality, label, oid) {
  const existing = binding[label] || null;
  const canonical = isCanonicalCommitOid(worktree, oid);
  const baseOid = bindingReality.baseOid;
  const taskBranchRef = bindingReality.taskBranch ? `refs/heads/${bindingReality.taskBranch}` : null;
  const afterBase = canonical && strictDescendant(worktree, baseOid, oid);
  const reachableOnTaskBranch = canonical && taskBranchRef
    ? gitFacts.isAncestor(worktree, oid, taskBranchRef)
    : false;
  let changedPaths = [];
  let withinWriteScope = false;
  if (canonical && afterBase && reachableOnTaskBranch) {
    try {
      changedPaths = gitFacts.changedPaths(worktree, baseOid, oid);
      withinWriteScope = changesStayInsideFrozenScope(node, changedPaths);
    } catch (error) {
      withinWriteScope = false;
    }
  }
  return {
    oid,
    canonical,
    afterBase,
    reachableOnTaskBranch,
    withinWriteScope,
    changedPaths,
    extendsPrevious: !existing || (isCanonicalCommitOid(worktree, existing)
      && gitFacts.isAncestor(worktree, existing, oid)),
    coversTaskHead: canonical && sameCommit(worktree, oid, bindingReality.headOid),
  };
}

function recordFactsForRequest(record, request, options) {
  const reality = taskCloseout.inspectTaskBindingReality({ node: record.node, cwd: options.cwd });
  if (!reality.ok) {
    return {
      ok: false,
      code: 'TASK_RECORD_GIT_REALITY_CONFLICT',
      error: 'record 前无法唯一核对声明 task worktree、branch、base 与 HEAD',
      detail: reality.detail || null,
    };
  }
  const observed = reality.facts;
  const worktree = observed.declaredWorktree;
  const binding = record.node.git;
  const facts = {
    binding: {
      baseOid: observed.baseOid,
      taskHeadOid: observed.headOid,
      taskBranchOid: observed.taskBranchOid,
      worktreeClean: Array.isArray(observed.statusNow) && observed.statusNow.length === 0,
      headMatchesTaskBranch: observed.headOid === observed.taskBranchOid,
      baseIsAncestor: Boolean(observed.baseOid && observed.taskBranchOid
        && gitFacts.isAncestor(worktree, observed.baseOid, observed.taskBranchOid)),
      declaredWorktree: worktree,
    },
    carriers: {},
  };
  if (request['product-commit']) {
    facts.carriers.product = carrierRecordFacts(
      worktree, record.node, binding, observed, 'product-commit', request['product-commit'],
    );
  }
  if (request['wip-commit']) {
    facts.carriers.wip = carrierRecordFacts(
      worktree, record.node, binding, observed, 'wip-commit', request['wip-commit'],
    );
  }
  const trackedAtHead = new Map();
  for (const entry of request['verification-runs'] || []) {
    const selected = taskRecord.canonicalTestPaths(entry.run && entry.run['test-paths']);
    if (!selected.ok) return selected;
    if (!selected.paths) continue; // ALL_TRACKED 是显式全量选择，不伪装成一条路径。
    const head = entry.run['head-oid'];
    let tracked = trackedAtHead.get(head);
    if (!tracked) {
      try {
        tracked = new Set(gitFacts.trackedPaths(worktree, head));
        trackedAtHead.set(head, tracked);
      } catch (error) {
        return {
          ok: false,
          code: 'TASK_RECORD_TEST_PATHS_UNAVAILABLE',
          error: `${entry.id} 无法在 run.head-oid 读取 tracked test paths：${error.message}`,
        };
      }
    }
    const missing = selected.paths.filter((testPath) => !tracked.has(testPath));
    if (missing.length > 0) {
      return {
        ok: false,
        code: 'TASK_RECORD_TEST_PATHS_UNTRACKED',
        error: `${entry.id} 的 run.test-paths 含不在 run.head-oid 的 tracked 路径`,
        detail: { id: entry.id, headOid: head, missing },
      };
    }
  }
  return {
    ok: true,
    facts,
    readRepoAtHead({ headOid, path: repoPath }) {
      return gitFacts.showFromRef(worktree, headOid, repoPath);
    },
  };
}

function recordPlan(options, planes, state) {
  const missing = requireId(options);
  if (missing) return missing;
  const requested = recordRequestFromOptions(options);
  if (!requested.ok) return requested;
  const found = findTaskRecord(state.index, options.id);
  if (!found.ok) return found;
  const facts = recordFactsForRequest(found.record, requested.request, options);
  if (!facts.ok) return facts;
  const planned = taskRecord.prepareRecord({
    record: found.record,
    request: requested.request,
    requestDigest: requested.digest,
    facts: facts.facts,
    readRepoAtHead: facts.readRepoAtHead,
  });
  if (!planned.ok) return planned;
  const writePlan = store.planNodeWrite(planes, planned.changes, {
    expectedGeneration: state.snapshot.generation,
    guardRecords: guardRecordsFor(state.index, planned.changes),
  });
  const observed = {
    requestDigest: planned.requestDigest,
    binding: planned.observed.binding,
    carriers: planned.observed.carriers,
    verification: planned.observed.verification,
  };
  const receipt = taskRecord.receiptFor(writePlan, planned.requestDigest, observed);
  return {
    ok: true,
    action: 'record',
    id: options.id,
    lifecycle: 'ACTIVE',
    generation: writePlan.generation,
    request: planned.request,
    observed,
    receipt,
    writePlan,
  };
}

function runRecord(options, planes, state) {
  if (!options.inspect && !options.write) {
    return publicFailure({
      ok: false,
      code: 'RECORD_INSPECTION_REQUIRED',
      error: 'record 先用 --inspect 取得 receipt，再以同一 payload 的 --write --receipt 落盘',
    }, state.snapshot.generation);
  }
  const planned = recordPlan(options, planes, state);
  if (!planned.ok) return publicFailure(planned, state.snapshot.generation);
  const output = {
    ok: true,
    action: 'record',
    id: planned.id,
    lifecycle: planned.lifecycle,
    generation: planned.generation,
    request: planned.request,
    observed: planned.observed,
    ...(options.inspect ? { receipt: planned.receipt } : {}),
    moves: planned.writePlan.entries.map((entry) => ({
      id: entry.id,
      from: entry.previousPath,
      to: entry.target,
      lifecycle: entry.lifecycle,
    })),
    afterImage: afterImage(planned.writePlan),
    written: false,
    preview: true,
  };
  if (!options.write) return output;
  if (!options.receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_REQUIRED',
      error: 'record --write 必须携带同一 after-image 的 --receipt',
    };
  }
  if (options.receipt.toLowerCase() !== planned.receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_MISMATCH',
      error: 'receipt 与当前 generation/source/Git 现实/after-image 不一致，必须重新 inspect',
    };
  }
  const committed = store.commitNodeWrite(planes, planned.writePlan);
  return {
    ...output,
    written: true,
    preview: false,
    generation: committed.generation,
    writtenPaths: committed.written,
  };
}

function claimText(claim, keys) {
  const source = claim && typeof claim === 'object' ? claim : {};
  for (const key of keys) {
    const value = source[key];
    if (typeof value === 'string' && value.trim() !== '') return value.trim();
  }
  return null;
}

function agentClaimLocator(claim) {
  return {
    worktree: claimText(claim, ['worktree', 'agent-worktree', 'agentWorktree']),
    branch: normalizedBranch(claimText(claim, ['branch', 'agent-branch', 'agentBranch'])).trim(),
  };
}

function agentFactFailure(id, baseOid, worktree, branch, error) {
  return {
    id,
    'base-oid': baseOid || null,
    'head-oid': null,
    worktree: worktree || null,
    branch: branch || null,
    changedPaths: null,
    error,
  };
}

function equalOid(left, right) {
  return typeof left === 'string' && typeof right === 'string'
    && left.toLowerCase() === right.toLowerCase();
}

// KP-8：子执行者的 “改了什么” 不能取自其回报。支持两种可逐人核对的
// 协作载体：
//   A. 独立 linked worktree + branch：主流程实读 worktree list / checkout
//      HEAD，强制 TASK frozen base..实际 agent HEAD；
//   B. 同 TASK branch 的显式 commit range：主流程验证 frozen base <= range
//      base <= range head <= 实际 task HEAD 均在同一祖先链，再重算该 range。
// 两种模式都绝不采信 claim.actual / changedPaths；缺少定位信息就不可核对。
function actualAgentClaimFacts(node, closeout, taskHeadOid, options) {
  const binding = node && node.git && typeof node.git === 'object' ? node.git : {};
  const claims = evidenceTrace.agentClaimsFromCloseout(closeout);
  const records = evidenceTrace.auditAgentClaims({ claims }).records;
  const frozenBase = binding['base-oid'] && isCanonicalCommitOid(options.cwd, binding['base-oid'])
    ? binding['base-oid']
    : null;
  const taskWorktree = realpathOrNull(binding.worktree);
  const taskBranch = normalizedBranch(binding['task-branch']).trim();
  const actualTaskHead = taskHeadOid && isCanonicalCommitOid(options.cwd, taskHeadOid)
    ? taskHeadOid
    : null;
  let listedWorktrees = null;
  let listError = null;
  let worktreeListRead = false;
  function readWorktreeList() {
    if (worktreeListRead) return;
    worktreeListRead = true;
    try { listedWorktrees = gitFacts.worktreeList(options.cwd); } catch (error) {
      listError = `主流程读不到 git worktree list：${error.message}`;
    }
  }
  const usedLocators = new Set();
  const actual = claims.map((claim, index) => {
    const record = records[index];
    const id = record ? record.id : `agent-${index + 1}`;
    if (!record || record.claimedComplete !== true) {
      return {
        id,
        'base-oid': frozenBase,
        'head-oid': null,
        changedPaths: null,
        error: null,
      };
    }
    if (!frozenBase) {
      return agentFactFailure(id, binding['base-oid'], null, null,
        'TASK 冻结 base-oid 不是本仓完整可核对 commit，不能审计子执行者');
    }
    const locator = agentClaimLocator(claim);
    const declaredRangeBase = claimText(claim, ['base-oid', 'baseOid', 'base']);
    const declaredRangeHead = claimText(claim, ['head-oid', 'headOid', 'head']);
    const hasExplicitRange = Boolean(declaredRangeBase && declaredRangeHead);
    const hasAnyLocator = Boolean(locator.worktree || locator.branch);
    const canonicalLocatorWorktree = locator.worktree && path.isAbsolute(locator.worktree)
      ? realpathOrNull(locator.worktree)
      : null;
    const explicitTaskLocator = hasAnyLocator
      && canonicalLocatorWorktree === taskWorktree
      && locator.branch === taskBranch;

    // B：同一 TASK branch 的串行提交。range base/head 是声明，而真正的
    // 事实仍由主流程拿 frozen base、actual task HEAD 和 ancestry 三方复核。
    if (hasExplicitRange && (!hasAnyLocator || explicitTaskLocator)) {
      if (!actualTaskHead || !isCanonicalCommitOid(options.cwd, declaredRangeBase)
        || !isCanonicalCommitOid(options.cwd, declaredRangeHead)) {
        return agentFactFailure(id, frozenBase, taskWorktree, taskBranch,
          '同 TASK branch 的完成声明必须给出完整可核对 range base/head 与实际 task HEAD');
      }
      const chainValid = gitFacts.isAncestor(options.cwd, frozenBase, declaredRangeBase)
        && gitFacts.isAncestor(options.cwd, declaredRangeBase, declaredRangeHead)
        && gitFacts.isAncestor(options.cwd, declaredRangeHead, actualTaskHead);
      if (!chainValid) {
        return agentFactFailure(id, frozenBase, taskWorktree, taskBranch,
          '同 TASK branch 的 range 必须满足 frozen base <= range base <= range head <= 实际 task HEAD');
      }
      try {
        return {
          id,
          mode: 'TASK_BRANCH_RANGE',
          'base-oid': declaredRangeBase,
          'head-oid': declaredRangeHead,
          worktree: taskWorktree,
          branch: taskBranch,
          changedPaths: gitFacts.changedPaths(taskWorktree || options.cwd, declaredRangeBase, declaredRangeHead),
          error: null,
        };
      } catch (error) {
        return agentFactFailure(id, declaredRangeBase, taskWorktree, taskBranch,
          `主流程读取同 TASK branch ${declaredRangeBase}..${declaredRangeHead} 失败：${error.message}`);
      }
    }

    // A：独立 worktree / branch。两项缺一不可，且不能把 TASK 自己的总 diff
    // 复用给多个 agent。
    if (!locator.worktree || !locator.branch || !path.isAbsolute(locator.worktree)) {
      return agentFactFailure(id, frozenBase, locator.worktree, locator.branch,
        '完成声明必须给出独立 agent worktree+branch，或同 TASK branch 的显式 commit range');
    }
    const canonicalWorktree = canonicalLocatorWorktree;
    if (!canonicalWorktree) {
      return agentFactFailure(id, frozenBase, locator.worktree, locator.branch,
        'agent worktree 不是可实读的 canonical checkout root');
    }
    if (canonicalWorktree === taskWorktree || locator.branch === taskBranch) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        '复用 TASK worktree/branch 时必须改用完整同分支 commit range；不能复用 TASK 总 diff');
    }
    readWorktreeList();
    if (listError || !Array.isArray(listedWorktrees)) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        listError || '主流程没有可核对的 git worktree list');
    }
    const matches = listedWorktrees.filter((entry) => realpathOrNull(entry && entry.worktree) === canonicalWorktree);
    if (matches.length !== 1) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        `git worktree list 对 agent worktree 命中 ${matches.length} 条，身份不唯一`);
    }
    const listed = matches[0];
    const expectedBranchRef = `refs/heads/${locator.branch}`;
    if (listed.branch !== expectedBranchRef || listed.detached === true || listed.bare === true) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        `agent worktree 实际 branch ${listed.branch || 'detached/unknown'} 与声明 ${expectedBranchRef} 不一致`);
    }
    const locatorKey = `${canonicalWorktree}\u0000${locator.branch}`;
    if (usedLocators.has(locatorKey)) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        '多个完成声明复用了同一 agent worktree/branch，不能把一条 diff 重复归属');
    }
    usedLocators.add(locatorKey);

    const worktreeHead = gitFacts.headOid(canonicalWorktree);
    const branchHead = gitFacts.runGit([
      'rev-parse', '--verify', '--end-of-options', `${expectedBranchRef}^{commit}`,
    ], { cwd: canonicalWorktree });
    const branchOid = branchHead.ok ? branchHead.stdout.trim() : null;
    if (!worktreeHead || !branchOid || listed.head !== worktreeHead || branchOid !== worktreeHead
      || !isCanonicalCommitOid(canonicalWorktree, worktreeHead)) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        'agent worktree 的实际 HEAD 与 git worktree list / 声明 branch 无法三方核对');
    }
    if (!gitFacts.isAncestor(canonicalWorktree, frozenBase, worktreeHead)) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        'agent branch 不包含 TASK 冻结 base-oid，不能作为本任务子执行者事实');
    }
    if ((declaredRangeBase && !equalOid(declaredRangeBase, frozenBase))
      || (declaredRangeHead && !equalOid(declaredRangeHead, worktreeHead))) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        'claim 自报 base/head 与主流程冻结 base 或实读 agent HEAD 不一致');
    }
    try {
      return {
        id,
        'base-oid': frozenBase,
        'head-oid': worktreeHead,
        worktree: canonicalWorktree,
        branch: locator.branch,
        changedPaths: gitFacts.changedPaths(canonicalWorktree, frozenBase, worktreeHead),
        error: null,
      };
    } catch (error) {
      return agentFactFailure(id, frozenBase, canonicalWorktree, locator.branch,
        `主流程读取 ${frozenBase}..${worktreeHead} 失败：${error.message}`);
    }
  });
  return { claims, actual };
}

function actualTerminalFacts(node, options, planes, closeout, body) {
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const integrationStatusNow = gitFacts.porcelainStatus(options.cwd);
  const productCommit = binding['product-commit'] || null;
  const wipCommit = binding['wip-commit'] || null;
  const baseOid = binding['base-oid'] || null;
  const baseOidImmutable = baseOid
    ? isCanonicalCommitOid(options.cwd, baseOid)
    : undefined;
  const taskBranch = binding['task-branch'] || null;
  const baseBranch = binding['base-branch'] || null;
  const removedProof = binding.disposition === 'REMOVED'
    ? taskCloseout.requireExactPhysicalRemovalAuthorization(closeout, binding, options.cwd)
    : null;
  const taskBindingReality = hasGitBinding(node) && binding.disposition !== 'REMOVED'
    ? taskCloseout.inspectTaskBindingReality({ node, cwd: options.cwd })
    : null;
  const taskBranchRef = taskBranch
    ? `refs/heads/${normalizedBranch(taskBranch)}`
    : null;
  const taskBranchExists = taskBindingReality && taskBindingReality.ok
    ? Boolean(taskBindingReality.facts.taskBranchOid)
    : (taskBranch
      ? gitFacts.runGit(
        ['rev-parse', '--verify', '--end-of-options', `${taskBranchRef}^{commit}`],
        { cwd: options.cwd },
      ).ok
      : false);
  const taskRef = taskBranchExists
    ? taskBranchRef
    : (removedProof && removedProof.ok
      ? removedProof.proof.taskHeadOid
      : (wipCommit || productCommit || baseOid || 'HEAD'));
  const resolvedTaskHead = gitFacts.runGit(
    ['rev-parse', '--verify', `${taskRef}^{commit}`],
    { cwd: options.cwd },
  );
  const taskHeadOid = resolvedTaskHead.ok ? resolvedTaskHead.stdout.trim() : null;
  let taskStatusNow;
  if (taskBindingReality && taskBindingReality.ok) {
    taskStatusNow = taskBindingReality.facts.statusNow;
  } else if (removedProof && removedProof.ok) {
    taskStatusNow = removedProof.proof.taskStatusNow;
  } else if (!hasGitBinding(node)) {
    taskStatusNow = integrationStatusNow;
  } else {
    // worktree 消失、分支不一致或 base/head 不可核对都不是 clean。
    taskStatusNow = ['<TASK_BINDING_REALITY_UNAVAILABLE>'];
  }
  const worktreeClean = taskStatusNow.length === 0
    && (!hasGitBinding(node)
      || (taskBindingReality && taskBindingReality.ok)
      || (removedProof && removedProof.ok));
  let scopedDiffEmpty = worktreeClean;
  const productCommitExists = productCommit
    ? isCanonicalCommitOid(options.cwd, productCommit)
    : undefined;
  const wipCommitExists = wipCommit
    ? isCanonicalCommitOid(options.cwd, wipCommit)
    : undefined;
  let productCommitTouchesScope = false;
  let productCommitChangesWithinScope;
  let productCommitCoversTaskHead;
  let taskChangesWithinScope;
  let wipCommitAfterBase;
  let wipCommitChangesWithinScope;
  let wipCommitCoversTaskHead;
  let changedPaths = [];
  let changedEntries = [];
  let trackedPaths = [];
  let addedIgnoreRules = [];
  let untrackedProductAudit = {
    ok: true,
    mustEnterRepository: [],
    mustStayOut: [],
    outsideWriteScope: [],
    problems: [],
  };
  if (baseOid && baseOidImmutable === true && taskHeadOid) {
    const taskChangedPaths = gitFacts.changedPaths(options.cwd, baseOid, taskRef);
    changedPaths = taskChangedPaths;
    changedEntries = gitFacts.changedEntries(options.cwd, baseOid, taskRef);
    trackedPaths = gitFacts.trackedPaths(options.cwd, taskRef);
    const taskRealityCwd = taskBindingReality && taskBindingReality.ok
      ? taskBindingReality.facts.declaredWorktree
      : options.cwd;
    const ignoreAudit = integrationGate.auditIgnoreRules({
      cwd: taskRealityCwd,
      baseOid,
      headRef: taskRef,
      reasons: closeout && Array.isArray(closeout.ignoreRules) ? closeout.ignoreRules : [],
    });
    addedIgnoreRules = ignoreAudit.added;
    const untrackedCandidates = gitFacts.untrackedPathsIncludingIgnored(
      taskRealityCwd,
      node['write-scope'] || [],
    );
    untrackedProductAudit = integrationGate.auditUntrackedProductDependencies({
      cwd: taskRealityCwd,
      writeScope: node['write-scope'] || [],
      untracked: untrackedCandidates,
    });
    const scopedPaths = gitFacts.scopedChangedPaths(
      options.cwd,
      baseOid,
      taskRef,
      node['write-scope'] || [],
    );
    // base..task-branch 与任务工作副本 porcelain 都必须干净；从 integration
    // 工作树退场时不能把 integration HEAD 当成 task HEAD。
    // 让 NO_PRODUCT_CHANGE 掩盖正在手里的 WIP。
    scopedDiffEmpty = scopedPaths.length === 0 && worktreeClean;
    taskChangesWithinScope = changesStayInsideFrozenScope(node, taskChangedPaths);
  } else if (hasGitBinding(node)) {
    scopedDiffEmpty = false;
    taskChangesWithinScope = false;
  }
  if (productCommit && productCommitExists && baseOid && baseOidImmutable === true) {
    const productChangedPaths = gitFacts.changedPaths(options.cwd, baseOid, productCommit);
    productCommitTouchesScope = gitFacts.scopedChangedPaths(
      options.cwd, `${productCommit}^`, productCommit, node['write-scope'] || [],
    ).length > 0;
    productCommitChangesWithinScope = productChangedPaths.length > 0
      && changesStayInsideFrozenScope(node, productChangedPaths);
    productCommitCoversTaskHead = wipCommit && wipCommitExists
      ? gitFacts.isAncestor(options.cwd, productCommit, wipCommit)
      : Boolean(taskHeadOid && sameCommit(options.cwd, productCommit, taskHeadOid));
  }
  if (wipCommit && wipCommitExists && baseOid && baseOidImmutable === true) {
    const wipChangedPaths = gitFacts.changedPaths(options.cwd, baseOid, wipCommit);
    wipCommitAfterBase = strictDescendant(options.cwd, baseOid, wipCommit);
    wipCommitChangesWithinScope = wipChangedPaths.length > 0
      && changesStayInsideFrozenScope(node, wipChangedPaths);
    wipCommitCoversTaskHead = Boolean(taskHeadOid && sameCommit(options.cwd, wipCommit, taskHeadOid));
  }
  const productCommitOnTaskBranch = productCommit && productCommitExists && taskBranch && taskBranchExists
    ? gitFacts.isAncestor(options.cwd, productCommit, taskBranchRef)
    : undefined;
  const productCommitAfterBase = productCommit && productCommitExists && baseOid
    ? strictDescendant(options.cwd, baseOid, productCommit)
    : undefined;
  const wipCommitOnTaskBranch = wipCommit && wipCommitExists && taskBranch && taskBranchExists
    ? gitFacts.isAncestor(options.cwd, wipCommit, taskBranchRef)
    : undefined;
  // integration ref 可以是固定 OID，也可以是 base branch 的 ref；两种都必须位于
  // 冻结 base branch 的历史上。任务分支自己的 HEAD 绝不能借名冒充 integration。
  const integrationRefOnBase = baseBranch && planes.integrationRef
    ? gitFacts.isAncestor(options.cwd, planes.integrationRef, baseBranch)
    : undefined;
  const agentFacts = actualAgentClaimFacts(node, closeout, taskHeadOid, options);
  const agentAudit = evidenceTrace.auditAgentClaims({
    claims: agentFacts.claims,
    actual: agentFacts.actual,
    parentWriteScope: node['write-scope'],
    parentForbiddenScope: node['forbidden-scope'],
  });
  const traceAudit = evidenceTrace.resolveRequiredVerification({
    node,
    body,
    readRepoAtHead({ headOid, path: repoPath }) {
      return gitFacts.showFromRef(options.cwd, headOid, repoPath);
    },
  });
  return {
    productCommit,
    productCommitTouchesScope,
    productCommitExists,
    productCommitOnTaskBranch,
    productCommitAfterBase,
    productCommitChangesWithinScope,
    productCommitCoversTaskHead,
    changedPaths,
    changedEntries,
    trackedPaths,
    addedIgnoreRules,
    untrackedProductAudit,
    taskHeadOid,
    taskBindingReality: taskBindingReality && taskBindingReality.ok
      ? taskBindingReality.facts
      : null,
    taskBindingRealityError: taskBindingReality && !taskBindingReality.ok
      ? taskBindingReality.detail
      : null,
    removedProof: removedProof && removedProof.ok ? removedProof.proof : null,
    removedProofError: removedProof && !removedProof.ok ? removedProof.detail : null,
    taskStatusNow,
    integrationStatusNow,
    integrationWorktreeClean: integrationStatusNow.length === 0,
    integratedNow: productCommit && productCommitExists
      ? gitFacts.isAncestor(options.cwd, productCommit, planes.integrationRef)
      : false,
    integrationRefOnBase,
    agentAudit,
    traceAudit,
    requiredVerification: (node.verification || []).filter((entry) => entry && entry.required === true),
    requiredVerificationEmptyIsPass: true,
    noProductChange: options.noProductChange === true || binding['no-product-change'] === true,
    scopedDiffEmpty,
    taskChangesWithinScope,
    worktreeClean,
    baseOidImmutable,
    wipCommit,
    wipCommitExists,
    wipCommitOnTaskBranch,
    wipCommitAfterBase,
    wipCommitChangesWithinScope,
    wipCommitCoversTaskHead,
    wipCarriedIntoIntegration: Boolean(
      wipCommit
      && wipCommitExists
      && gitFacts.isAncestor(options.cwd, wipCommit, planes.integrationRef),
    ),
    disposition: options.disposition || binding.disposition || null,
  };
}

function terminalPrecondition(record, options, planes) {
  const reality = terminalGitReality(record, options);
  if (!reality.ok) return reality;
  return { ok: true, facts: actualTerminalFacts(record.node, options, planes, null, record.body) };
}

function chooseDecision(options, planes, state) {
  const { snapshot, index } = state;
  const closedAt = options.closedAt || new Date().toISOString().slice(0, 10);
  const taskInput = { index, records: snapshot.records, id: options.id, closedAt };

  if (options.command === 'create') {
    const required = requireCandidate(options, 'create');
    if (required) return required;
    const integration = requireExplicitIntegrationRef(planes, options, 'create');
    if (integration) return integration;
    const loaded = readCandidate(options.candidatePath, options.cwd);
    if (!loaded.ok) return loaded;
    return taskLifecycle.create({
      index,
      candidate: loaded.candidate,
      historyExists: historyExists(planes, options),
    });
  }

  const missing = requireId(options);
  if (missing) return missing;

  if (options.command === 'ready') return taskLifecycle.ready(taskInput);
  if (options.command === 'park') return taskLifecycle.park(taskInput);

  if (options.command === 'activate' || options.command === 'resume') {
    const found = findTaskRecord(index, options.id);
    if (!found.ok) return found;
    const gitReality = activeOpeningReality(found.record, options);
    return options.command === 'activate'
      ? taskLifecycle.activate({ ...taskInput, gitReality })
      : taskLifecycle.resume({ ...taskInput, gitReality });
  }

  const terminalCommands = ['withdraw', 'finish', 'cancel', 'stop', 'split', 'replace'];
  if (terminalCommands.includes(options.command)) {
    const integration = requireExplicitIntegrationRef(planes, options, options.command);
    if (integration) return integration;
    const found = findTaskRecord(index, options.id);
    if (!found.ok) return found;
    const terminal = options.terminalFacts
      ? { ok: true, facts: options.terminalFacts }
      : terminalPrecondition(found.record, options, planes);
    if (!terminal.ok) return terminal;
    const facts = terminal.facts;
    if (options.command === 'withdraw') return taskLifecycle.withdraw({ ...taskInput, facts });
    if (options.command === 'finish') {
      return taskLifecycle.finish({
        ...taskInput,
        result: options.result,
        facts,
        allowRetiringDisposition: options.closeoutMode === true,
      });
    }
    if (options.command === 'cancel') {
      return taskLifecycle.cancel({
        ...taskInput,
        facts,
        allowRetiringDisposition: options.closeoutMode === true,
      });
    }
    if (options.command === 'stop') {
      return taskLifecycle.stop({
        ...taskInput,
        facts,
        allowRetiringDisposition: options.closeoutMode === true,
      });
    }
    if (options.command === 'split') {
      if (options.successorPaths.length === 0) {
        return { ok: false, code: 'TASK_SUCCESSOR_REQUIRED', error: 'split 至少需要一个 --successor <Markdown>' };
      }
      const candidates = [];
      for (const candidatePath of options.successorPaths) {
        const loaded = readCandidate(candidatePath, options.cwd);
        if (!loaded.ok) return loaded;
        candidates.push(loaded.candidate);
      }
      return taskLifecycle.split({
        ...taskInput,
        candidates,
        historyExists: historyExists(planes, options),
        reason: options.splitReason,
        disposition: options.disposition,
        facts,
      });
    }
    const required = requireCandidate(options, 'replace');
    if (required) return required;
    const loaded = readCandidate(options.candidatePath, options.cwd);
    if (!loaded.ok) return loaded;
    return taskLifecycle.replace({
      ...taskInput,
      candidate: loaded.candidate,
      historyExists: historyExists(planes, options),
      reason: options.replaceReason,
      disposition: options.disposition,
      facts,
    });
  }

  return { ok: false, code: 'TASK_OPERATION_FAILED', error: `未实现的 TASK 命令 ${options.command}` };
}

function publicFailure(outcome, fallbackGeneration) {
  const detail = outcome && outcome.detail && typeof outcome.detail === 'object' ? outcome.detail : {};
  const result = {
    ok: false,
    code: outcome && outcome.code ? outcome.code : 'TASK_OPERATION_FAILED',
    error: outcome && outcome.error ? outcome.error : 'TASK 操作失败',
    generation: outcome && outcome.generation !== undefined ? outcome.generation : fallbackGeneration,
    written: false,
    ...detail,
  };
  if (outcome && Array.isArray(outcome.schema)) result.schema = outcome.schema;
  if (outcome && Array.isArray(outcome.integrity)) result.integrity = outcome.integrity;
  return result;
}

// ---------------------------------------------------------------------------
// AH-050-06：propose / start 薄壳
// ---------------------------------------------------------------------------
// 这里刻意只做 argv、只读事实和 task-start 编排。控制面写仍只能走
// task-start -> store，Git 写仍只能走 task-start -> git-facts.runStartGit。

function isPlainObject(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function cliFailure(code, error, detail) {
  return {
    ok: false,
    code,
    error,
    written: false,
    ...(detail || {}),
  };
}

function safeProposalId(value) {
  const id = typeof value === 'string' ? value.trim() : '';
  return id !== '' && id !== '.' && id !== '..' && !/[\\/\0]/.test(id) ? id : null;
}

function safeRelativePackagePath(value) {
  const candidate = typeof value === 'string' ? value.trim().replace(/\\/g, '/') : '';
  if (candidate === '' || candidate.startsWith('/') || candidate.includes('\0')) return null;
  const normalized = path.posix.normalize(candidate).replace(/^\.\//, '');
  if (normalized === '.' || normalized === '..' || normalized.startsWith('../') || normalized.includes('/../')) return null;
  return normalized;
}

function pathInsideRoot(target, root) {
  const relative = path.relative(root, target);
  return relative !== ''
    && relative !== '..'
    && !relative.startsWith(`..${path.sep}`)
    && !path.isAbsolute(relative);
}

function requestInput(options, runtime) {
  if (!options.requestPath && !options.requestStdin) {
    return cliFailure('PROPOSE_REQUEST_REQUIRED', 'propose 需要 --request <文本文件> 或 --request-stdin');
  }
  try {
    const text = options.requestStdin
      ? (runtime && Object.prototype.hasOwnProperty.call(runtime, 'stdinText')
        ? String(runtime.stdinText)
        : fs.readFileSync(0, 'utf8'))
      : fs.readFileSync(path.resolve(options.cwd, options.requestPath), 'utf8');
    if (text.trim() === '') return cliFailure('PROPOSE_REQUEST_EMPTY', 'propose 请求不能为空');
    return { ok: true, text };
  } catch (error) {
    return cliFailure('PROPOSE_REQUEST_READ_FAILED', `读不到 propose 请求：${error.message}`);
  }
}

function proposalInput(options, requiredDigest) {
  if (!options.proposalPath) {
    return cliFailure('PROPOSAL_FILE_REQUIRED', 'start 需要 --proposal <JSON>');
  }
  let parsed;
  let absolute;
  try {
    absolute = path.resolve(options.cwd, options.proposalPath);
    parsed = JSON.parse(fs.readFileSync(absolute, 'utf8'));
  } catch (error) {
    return cliFailure('PROPOSAL_FILE_READ_FAILED', `读不到或解析不了 proposal：${error.message}`);
  }
  if (!isPlainObject(parsed)) {
    return cliFailure('PROPOSAL_FILE_INVALID', 'proposal 文件必须是 JSON 对象');
  }
  const wrapped = isPlainObject(parsed.proposal);
  const proposal = wrapped ? parsed.proposal : parsed;
  const proposalDigest = options.proposalDigest || (wrapped ? parsed.proposalDigest : null);
  if (requiredDigest && (typeof proposalDigest !== 'string' || !/^[0-9a-f]{64}$/i.test(proposalDigest))) {
    return cliFailure('PROPOSAL_DIGEST_REQUIRED', 'start 需要与 proposal 同一字节身份的 --proposal-digest，或 wrapper 内的 proposalDigest');
  }
  return {
    ok: true,
    absolute,
    proposal,
    proposalDigest: typeof proposalDigest === 'string' ? proposalDigest.toLowerCase() : null,
  };
}

function parentDigest(record) {
  if (record && typeof record.digest === 'string' && /^[0-9a-f]{64}$/i.test(record.digest)) return record.digest;
  if (record && typeof record.text === 'string') return store.digestOf(record.text);
  return null;
}

function activePlanRecord(record) {
  const node = record && record.node;
  return Boolean(node
    && node.lifecycle === 'ACTIVE'
    && (node.kind === 'ROOT_PLAN' || node.kind === 'PHASE_PLAN'));
}

function parentSummary(record) {
  if (!record || !record.node) return null;
  return {
    id: record.node.id,
    kind: record.node.kind,
    lifecycle: record.node.lifecycle,
    goal: record.node.goal || '',
    digest: parentDigest(record),
  };
}

function snapshotRecordForId(state, id) {
  return state && state.snapshot && Array.isArray(state.snapshot.records)
    ? state.snapshot.records.find((record) => record && record.node && record.node.id === id) || null
    : null;
}

function currentPathCandidates(planes, repoRoot) {
  const roots = [];
  const add = (value) => {
    if (typeof value !== 'string' || value.trim() === '') return;
    const normalized = path.resolve(value);
    if (!roots.includes(normalized)) roots.push(normalized);
  };
  add(repoRoot);
  try {
    add(fs.realpathSync(repoRoot));
  } catch (_) {
    // repoRoot 已由 Git 证明存在；realpath 失败时保留原始身份，后续按 CURRENT 缺失处理。
  }
  return roots.map((root) => ({
    root,
    path: store.currentViewPath(planes, store.worktreeKeyFor(root)),
  }));
}

function readCallingCurrent(planes, repoRoot) {
  const candidates = currentPathCandidates(planes, repoRoot);
  const readable = [];
  for (const candidate of candidates) {
    try {
      const text = fs.readFileSync(candidate.path, 'utf8');
      readable.push({
        ...candidate,
        text,
        digest: store.digestOf(text),
      });
    } catch (error) {
      if (!error || error.code === 'ENOENT' || error.code === 'ENOTDIR') continue;
      return {
        artifact: { path: candidate.path, digest: null, exists: false, unreadable: true },
        selection: { mode: 'AUTO_CURRENT_UNREADABLE', path: candidate.path },
      };
    }
  }
  if (readable.length === 0) {
    return {
      artifact: { path: candidates[0] ? candidates[0].path : null, digest: null, exists: false },
      selection: { mode: 'AUTO_CURRENT_MISSING', paths: candidates.map((candidate) => candidate.path) },
    };
  }
  // /var 与 /private/var 的别名可能映射到不同 key。若两个 key 都有 CURRENT，
  // 不以内容或时间戳猜“哪个才是调用者入口”。必须让用户显式 --parent。
  if (readable.length !== 1) {
    return {
      artifact: { path: readable[0].path, digest: readable[0].digest, exists: true },
      selection: { mode: 'AUTO_CURRENT_AMBIGUOUS', paths: readable.map((candidate) => candidate.path) },
    };
  }
  const current = readable[0];
  return {
    artifact: { path: current.path, digest: current.digest, exists: true },
    text: current.text,
    worktreeRoot: current.root,
    selection: null,
  };
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function currentLineValues(text, key) {
  const pattern = new RegExp(`^\\s*${escapeRegExp(key)}\\s*:\\s*(.*?)\\s*$`, 'i');
  return String(text || '').split(/\r?\n/)
    .map((line) => line.match(pattern))
    .filter(Boolean)
    .map((match) => match[1]);
}

function sameWorktreeIdentity(left, right) {
  if (typeof left !== 'string' || typeof right !== 'string' || left.trim() === '' || right.trim() === '') return false;
  const candidates = (value) => {
    const result = [path.resolve(value)];
    try {
      const real = fs.realpathSync(value);
      if (!result.includes(real)) result.push(real);
    } catch (_) {
      // CURRENT/节点事实必须能至少以绝对 lexical identity 对齐；不因 realpath 失败猜测。
    }
    return result;
  };
  const lefts = candidates(left);
  const rights = candidates(right);
  return lefts.some((value) => rights.includes(value));
}

function activeLeafFromCurrent(current, state) {
  if (!current || typeof current.text !== 'string') {
    return { ok: false, selection: current && current.selection
      ? current.selection
      : { mode: 'AUTO_CURRENT_MISSING' } };
  }
  const declaredWorktrees = currentLineValues(current.text, 'worktree');
  if (declaredWorktrees.length !== 1 || !sameWorktreeIdentity(declaredWorktrees[0], current.worktreeRoot)) {
    return {
      ok: false,
      selection: {
        mode: declaredWorktrees.length > 1 ? 'AUTO_CURRENT_AMBIGUOUS' : 'AUTO_CURRENT_WORKTREE_MISMATCH',
        path: current.artifact.path,
      },
    };
  }
  const leaves = currentLineValues(current.text, 'active leaf');
  if (leaves.length !== 1) {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_AMBIGUOUS', path: current.artifact.path, field: 'active leaf' },
    };
  }
  if (leaves[0].trim().toLowerCase() === 'idle') {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_IDLE', path: current.artifact.path },
    };
  }
  const matching = state.snapshot.records.filter((record) => {
    const id = record && record.node && record.node.id;
    return typeof id === 'string'
      && new RegExp(`^${escapeRegExp(id)}(?:\\s*\\||\\s*[（(])`).test(leaves[0]);
  });
  if (matching.length !== 1) {
    return {
      ok: false,
      selection: {
        mode: matching.length > 1 ? 'AUTO_CURRENT_AMBIGUOUS' : 'AUTO_CURRENT_LEAF_UNRESOLVED',
        path: current.artifact.path,
      },
    };
  }
  const leaf = matching[0];
  const node = leaf.node;
  if (!node || node.kind !== 'TASK' || node.lifecycle !== 'ACTIVE') {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_LEAF_NOT_ACTIVE', path: current.artifact.path, id: node && node.id ? node.id : null },
    };
  }
  if (!node.git || !sameWorktreeIdentity(node.git.worktree, current.worktreeRoot)) {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_LEAF_WORKTREE_MISMATCH', path: current.artifact.path, id: node.id },
    };
  }
  const lineage = graph.resolveLineage(state.index, node.id);
  const parentId = node['parent-id'];
  if (!lineage.ok || !parentId || lineage.chain[lineage.chain.length - 2] !== parentId) {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_LINEAGE_UNRESOLVED', path: current.artifact.path, id: node.id },
    };
  }
  const parent = snapshotRecordForId(state, parentId);
  if (!activePlanRecord(parent)) {
    return {
      ok: false,
      selection: { mode: 'AUTO_CURRENT_PARENT_UNRESOLVED', path: current.artifact.path, id: parentId },
    };
  }
  return { ok: true, leaf, parent };
}

function chooseProposeParent(options, state, current) {
  if (options.parentId) {
    const record = snapshotRecordForId(state, options.parentId);
    if (!activePlanRecord(record)) {
      return {
        record: null,
        selection: {
          mode: 'EXPLICIT_REJECTED',
          requestedId: options.parentId,
          reason: 'NOT_ACTIVE_PLAN',
        },
      };
    }
    return { record, selection: { mode: 'EXPLICIT', id: record.node.id } };
  }
  const resolved = activeLeafFromCurrent(current, state);
  if (!resolved.ok) return { record: null, selection: resolved.selection };
  return {
    record: resolved.parent,
    selection: {
      mode: 'AUTO_CURRENT_DIRECT',
      id: resolved.parent.node.id,
      activeLeafId: resolved.leaf.node.id,
      currentPath: current.artifact.path,
    },
  };
}

function gitOutput(args, cwd) {
  const result = gitFacts.runGit(args, { cwd });
  return result && result.ok === true ? result.stdout.trim() : null;
}

function gitRealityForPropose(options) {
  const repoRoot = gitFacts.repoRoot(options.cwd);
  const currentBranch = gitOutput(['rev-parse', '--abbrev-ref', 'HEAD'], options.cwd);
  const baseBranch = options.baseBranch || (currentBranch && currentBranch !== 'HEAD' ? currentBranch : null);
  const observedBaseOid = baseBranch
    ? gitOutput(['rev-parse', '--verify', '--end-of-options', `${baseBranch}^{commit}`], options.cwd)
    : null;
  const baseOid = options.baseOid || observedBaseOid;
  let status = null;
  try {
    status = gitFacts.porcelainStatus(options.cwd);
  } catch (error) {
    status = { unavailable: true, code: error && error.code ? error.code : null };
  }
  return {
    repoRoot,
    currentBranch,
    baseBranch,
    baseOid,
    observedBaseOid,
    status,
  };
}

function firstReadableArtifact(candidates) {
  const list = Array.isArray(candidates) ? candidates : [];
  for (const filePath of list) {
    try {
      const text = fs.readFileSync(filePath, 'utf8');
      return { path: filePath, digest: store.digestOf(text), exists: true };
    } catch (error) {
      if (!error || error.code !== 'ENOENT') {
        return { path: filePath, digest: null, exists: false, unreadable: true };
      }
    }
  }
  return { path: list[0] || null, digest: null, exists: false };
}

function inferWriteScope(request) {
  const matches = String(request || '').match(/(?:^|[^A-Za-z0-9_.-])((?:[A-Za-z0-9_.-]+\/)+[A-Za-z0-9_.-]+)/g) || [];
  const found = new Set();
  for (const match of matches) {
    const candidate = match.replace(/^[^A-Za-z0-9_.-]+/, '').replace(/^[./]+/, '');
    if (candidate && !candidate.includes('..') && !candidate.startsWith('/')) found.add(candidate);
  }
  return [...found].sort();
}

function automaticTaskId(request, parent, reality) {
  const material = {
    request: routing.redactKnownSecrets(String(request || '')),
    parent: parent && parent.node ? {
      id: parent.node.id,
      digest: parentDigest(parent),
    } : null,
    base: {
      branch: reality && reality.baseBranch ? reality.baseBranch : null,
      oid: reality && reality.baseOid ? reality.baseOid : null,
    },
  };
  const stable = taskStart.stableJson(material);
  return `TASK-${store.digestOf(stable).slice(0, 12).toUpperCase()}`;
}

function proposalWorktree(options, repoRoot) {
  const id = safeProposalId(options.id) || 'unknown';
  const requestedPath = options.worktree || path.join(path.dirname(repoRoot), id.toLowerCase());
  let canonicalPath = null;
  let preflight = null;
  try {
    preflight = opening.inspectWorktreeTargetBeforeCreate({ cwd: options.cwd, worktree: requestedPath });
    if (preflight && preflight.ok === true) canonicalPath = preflight.canonicalPath;
  } catch (error) {
    preflight = { ok: false, code: error && error.code ? error.code : 'WORKTREE_PREFLIGHT_FAILED' };
  }
  return { requestedPath, canonicalPath, preflight };
}

function taskStartRuntimeFor(options, runtime, planes, proposal) {
  const supplied = runtime && isPlainObject(runtime.taskStartRuntime) ? runtime.taskStartRuntime : {};
  const facts = supplied.gitFacts || gitFacts;
  const opener = supplied.opening || opening;
  const storeApi = supplied.store || store;
  const customInspector = typeof supplied.inspectResources === 'function' ? supplied.inspectResources : null;
  return {
    ...supplied,
    planes,
    inspectResources: customInspector
      ? (record, passedProposal) => customInspector(record, passedProposal || proposal || null)
      : (record, passedProposal) => inspectStartResources(record, passedProposal || proposal || null, {
        cwd: options.cwd,
        gitFacts: facts,
        opening: opener,
        store: storeApi,
      }),
  };
}

function inspectionFailure(error, detail) {
  return {
    ok: false,
    code: 'START_RESOURCE_REALITY_UNAVAILABLE',
    error,
    detail: detail || null,
  };
}

function inspectStartResources(record, proposal, deps) {
  const settings = deps || {};
  const facts = settings.gitFacts || gitFacts;
  const opener = settings.opening || opening;
  const storeApi = settings.store || store;
  const cwd = settings.cwd || process.cwd();
  if (!record || !record.branch || !record.base || !record.worktree) {
    return inspectionFailure('PENDING_START 缺少可核对的冻结 branch/base/worktree 身份');
  }
  const branchName = record.branch.name;
  const requestedPath = record.worktree.requestedPath;
  const baseOid = record.base.oid;
  if (typeof branchName !== 'string' || branchName.trim() === ''
    || typeof requestedPath !== 'string' || !path.isAbsolute(requestedPath)
    || typeof baseOid !== 'string' || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(baseOid)) {
    return inspectionFailure('PENDING_START 冻结身份格式不完整，不能猜测资源现实');
  }
  try {
    // 先证明调用者仍处于可读取的同一 Git 现实；后续每项都只读取，不修复。
    facts.repoRoot(cwd);
  } catch (error) {
    return inspectionFailure(`读不到开工仓库 Git 现实：${error.message}`, { causeCode: error && error.code ? error.code : null });
  }

  const branchRef = `refs/heads/${branchName}`;
  const branchResult = facts.runGit([
    'rev-parse', '--verify', '--end-of-options', `${branchRef}^{commit}`,
  ], { cwd });
  const branchOid = branchResult && branchResult.ok === true ? branchResult.stdout.trim() : null;
  const branch = branchOid
    ? {
      name: branchName,
      baseOid,
      containsBase: facts.isAncestor(cwd, baseOid, branchOid),
    }
    : null;

  let worktree = null;
  let canonicalWorktree = null;
  if (fs.existsSync(requestedPath)) {
    try {
      canonicalWorktree = fs.realpathSync(requestedPath);
      const expectedCanonical = record.worktree.canonicalPath
        || (proposal && proposal.worktree && proposal.worktree.canonicalPath)
        || path.resolve(requestedPath);
      const reconciled = opener.reconcileCreatedWorktree({
        cwd,
        worktree: requestedPath,
        canonicalPath: expectedCanonical,
        branch: branchName,
        baseOid,
      });
      if (!reconciled || reconciled.ok !== true || !reconciled.facts) {
        return inspectionFailure('linked worktree 的 realpath/Git 身份与冻结记录不一致', {
          worktree: requestedPath,
          reconciled: reconciled || null,
        });
      }
      const observed = reconciled.facts;
      if (observed.canonicalWorktree !== canonicalWorktree
        || observed.currentBranch !== branchName
        || observed.containsBase !== true) {
        return inspectionFailure('linked worktree 未提供完整强身份事实', { observed });
      }
      worktree = {
        path: requestedPath,
        canonicalPath: canonicalWorktree,
        branch: observed.currentBranch,
        baseOid,
        containsBase: true,
      };
    } catch (error) {
      return inspectionFailure(`读取 linked worktree 现实失败：${error.message}`, {
        causeCode: error && error.code ? error.code : null,
        worktree: requestedPath,
      });
    }
  }

  let taskPackage = null;
  const markerPackage = record.resources && record.resources.taskPackage ? record.resources.taskPackage : null;
  let packagePathValue = proposal && proposal.taskPackage ? proposal.taskPackage.path : null;
  if (!packagePathValue && markerPackage) {
    packagePathValue = markerPackage.path;
  }
  packagePathValue = safeRelativePackagePath(packagePathValue);
  // worktree 已存在而 package 尚未被 marker 冻结，是最容易在崩溃后被“看成没发生”
  // 的窗口。没有同一 proposal 就没有唯一 package 路径，必须明确停在 unknown，
  // 不能把 taskPackage:null 当作已证明的不存在。
  if (worktree && !packagePathValue && !markerPackage) {
    return inspectionFailure('linked worktree 已存在，但 marker 尚未冻结 task package；doctor 需携带同一 --proposal 才能只读核对', {
      pendingId: record.id || null,
      worktree: worktree.canonicalPath,
      required: '--proposal <同一 proposal JSON>',
    });
  }
  if (worktree && packagePathValue) {
    const target = path.resolve(worktree.canonicalPath, packagePathValue);
    if (!pathInsideRoot(target, worktree.canonicalPath)) {
      return inspectionFailure('task package 路径越出冻结 linked worktree', { packagePath: packagePathValue });
    }
    if (fs.existsSync(target)) {
      try {
        const resolvedTarget = fs.realpathSync(target);
        const stat = fs.statSync(resolvedTarget);
        if (resolvedTarget !== target || !stat.isFile()) {
          return inspectionFailure('task package 不是 worktree 内的普通文件', { target, resolvedTarget });
        }
        taskPackage = {
          path: packagePathValue,
          digest: storeApi.digestOf(fs.readFileSync(resolvedTarget, 'utf8')),
        };
      } catch (error) {
        return inspectionFailure(`读取 task package 现实失败：${error.message}`, {
          causeCode: error && error.code ? error.code : null,
          packagePath: packagePathValue,
        });
      }
    }
  }

  let openingCommit = null;
  if (worktree && taskPackage && branch) {
    try {
      const headOid = facts.headOid(worktree.canonicalPath);
      if (!headOid) return inspectionFailure('读不到 task worktree HEAD，不能核对 opening commit');
      if (headOid !== branchOid) {
        return inspectionFailure('task branch ref 与 task worktree HEAD 不一致，不能接管 opening commit', {
          branch: branchName,
          branchOid,
          headOid,
        });
      }
      if (headOid !== baseOid) {
        const changedPaths = [...new Set(facts.changedPaths(worktree.canonicalPath, baseOid, headOid))].sort();
        const commits = facts.runGit(['log', '--format=%H', `${baseOid}..${headOid}`], {
          cwd: worktree.canonicalPath,
        });
        const staged = facts.runGit(['diff', '--cached', '--name-only'], { cwd: worktree.canonicalPath });
        const committedPackage = facts.runGit(['show', `${headOid}:${taskPackage.path}`], {
          cwd: worktree.canonicalPath,
        });
        const worktreeStatus = facts.porcelainStatus(worktree.canonicalPath);
        if (!commits || commits.ok !== true || !staged || staged.ok !== true
          || !committedPackage || committedPackage.ok !== true) {
          return inspectionFailure('opening commit 的 Git 现实无法完整读取');
        }
        const commitOids = commits.stdout.split('\n').map((line) => line.trim()).filter(Boolean);
        const onlyTaskPackage = changedPaths.length === 1 && changedPaths[0] === taskPackage.path;
        const containsBase = facts.isAncestor(worktree.canonicalPath, baseOid, headOid);
        const singleCommitFromBase = commitOids.length === 1 && commitOids[0] === headOid;
        const indexClean = staged.stdout.trim() === '';
        const worktreeClean = Array.isArray(worktreeStatus) && worktreeStatus.length === 0;
        const headPackageDigest = storeApi.digestOf(committedPackage.stdout);
        const packageMatchesHead = headPackageDigest === taskPackage.digest;
        if (!onlyTaskPackage || !containsBase || !singleCommitFromBase || !indexClean || !worktreeClean || !packageMatchesHead) {
          return inspectionFailure('opening commit 未同时满足唯一 package、单提交、base ancestry 与干净 worktree', {
            headOid,
            changedPaths,
            commitOids,
            staged: staged.stdout.split('\n').map((line) => line.trim()).filter(Boolean),
            worktreeStatus,
            onlyTaskPackage,
            containsBase,
            singleCommitFromBase,
            indexClean,
            worktreeClean,
            packageMatchesHead,
          });
        }
        openingCommit = {
          oid: headOid,
          branch: branchName,
          packagePath: taskPackage.path,
          onlyTaskPackage: true,
          containsBase: true,
          singleCommitFromBase: true,
          indexClean: true,
          worktreeClean: true,
          headPackageDigest,
        };
      }
    } catch (error) {
      return inspectionFailure(`核对 opening commit 现实失败：${error.message}`, {
        causeCode: error && error.code ? error.code : null,
      });
    }
  }
  return { ok: true, branch, worktree, taskPackage, openingCommit };
}

function runPropose(options, runtime) {
  if (options.write) return cliFailure('PROPOSE_WRITE_FORBIDDEN', 'propose 严格只读；不得携带 --write');
  const request = requestInput(options, runtime);
  if (!request.ok) return request;
  // READ_ONLY 只形成会话合同。它不能因为控制面尚未初始化、图谱暂有坏节点或
  // 没有可选父计划而被迫进入 start 的读写前置条件；尤其不能先创建/接触任何
  // PENDING_START。这里最多读取调用目录下的权威候选和可用的 Git 事实。
  if (options.profile === 'READ_ONLY') {
    let repoRoot = options.cwd;
    let gitReality = { repoRoot, unavailable: true };
    try {
      gitReality = gitRealityForPropose(options);
      repoRoot = gitReality.repoRoot;
    } catch (error) {
      gitReality = {
        repoRoot,
        unavailable: true,
        code: error && error.code ? error.code : null,
      };
    }
    const authority = firstReadableArtifact([
      path.join(repoRoot, 'AUTHORITY.md'),
      path.join(repoRoot, 'docs', 'harness', 'AUTHORITY.md'),
    ]);
    const output = taskStart.propose({
      request: request.text,
      profile: 'READ_ONLY',
      goal: options.goal || request.text.trim(),
      authority,
      current: null,
      parentSummary: null,
      gitReality,
    }, taskStartRuntimeFor(options, runtime, runtime && runtime.planes ? runtime.planes : null, null));
    return {
      ...output,
      parentSelection: { mode: 'READ_ONLY_NO_PARENT' },
      worktreePreflight: null,
    };
  }
  const verification = proposalVerificationFromOptions(options);
  if (!verification.ok) return verification;
  try {
    const planes = loadPlanes(options, runtime);
    const state = snapshotPrecondition(planes);
    if (!state.ok) return publicFailure(state, state.snapshot.generation);
    const reality = gitRealityForPropose(options);
    // 自动父计划只能从“本调用 worktree 的 CURRENT -> ACTIVE leaf -> 直接 parent”
    // 推导；绝不扫描全图选择一条碰巧更深的无关计划。
    const current = readCallingCurrent(planes, reality.repoRoot);
    const parent = chooseProposeParent(options, state, current);
    const selectedParent = parent.record;
    const id = options.id || automaticTaskId(request.text, selectedParent, reality);
    const worktree = proposalWorktree({ ...options, id }, reality.repoRoot);
    const packagePathValue = options.taskPackagePath || (id ? `plans/v0.5.0/${id}.md` : '');
    // opening package 本身也是本次明确声明的本地控制写；把它以精确路径并入
    // write-scope，保留用户的产品路径，不用一个笼统的控制根替代它。
    const sourceWriteScope = options.writeScope.length > 0 ? options.writeScope.slice() : inferWriteScope(request.text);
    const canonicalPackagePath = safeRelativePackagePath(packagePathValue);
    const declaredWriteScope = [...new Set([
      ...sourceWriteScope,
      ...(canonicalPackagePath ? [canonicalPackagePath] : []),
    ])];
    const authority = firstReadableArtifact([
      path.join(reality.repoRoot, 'AUTHORITY.md'),
      path.join(reality.repoRoot, 'docs', 'harness', 'AUTHORITY.md'),
    ]);
    const goal = options.goal || request.text.trim();
    const output = taskStart.propose({
      request: request.text,
      id,
      profile: options.profile,
      parent: selectedParent
        ? { id: selectedParent.node.id, digest: parentDigest(selectedParent) }
        : { id: '', digest: '' },
      base: { branch: reality.baseBranch || '', oid: reality.baseOid || '' },
      branch: { name: options.taskBranch || (id ? `codex/${String(id).toLowerCase()}` : '') },
      worktree: { requestedPath: worktree.requestedPath, canonicalPath: worktree.canonicalPath },
      declaration: {
        writeScope: declaredWriteScope,
        forbiddenScope: options.forbiddenScope.slice(),
        exclusiveResources: options.exclusiveResources.slice(),
      },
      taskPackage: { path: packagePathValue },
      goal,
      acceptanceCriteria: options.acceptanceCriteria.length > 0
        ? options.acceptanceCriteria.slice()
        : (goal ? ['完成 proposal 中冻结的目标并给出直接验证。'] : []),
      verification: verification.entries,
      // Git 写权限只消费专用 CLI 旗标。请求正文可能包含引用、粘贴的外部内容
      // 或历史说明，不能靠词法命中把其中任何一句提升为本次直接授权。
      localCommitAllowed: options.localCommitAllowed,
      pushAllowed: options.pushAllowed,
      authority,
      current: current.artifact,
      parentSummary: selectedParent ? parentSummary(selectedParent) : null,
      gitReality: {
        repoRoot: reality.repoRoot,
        currentBranch: reality.currentBranch,
        baseBranch: reality.baseBranch,
        baseOid: reality.baseOid,
        observedBaseOid: reality.observedBaseOid,
        status: reality.status,
        worktreePreflight: worktree.preflight,
      },
    }, taskStartRuntimeFor(options, runtime, planes, null));
    return {
      ...output,
      parentSelection: parent.selection,
      worktreePreflight: worktree.preflight,
    };
  } catch (error) {
    return cliFailure(error && error.code ? error.code : 'PROPOSE_OPERATION_FAILED', error && error.message ? error.message : String(error));
  }
}

function runStartFamily(options, runtime) {
  try {
    const planes = loadPlanes(options, runtime);
    if (options.startMode === 'doctor') {
      if (options.write) return cliFailure('DOCTOR_WRITE_FORBIDDEN', 'doctor 只读，不得携带 --write');
      const loaded = options.proposalPath ? proposalInput(options, false) : null;
      if (loaded && !loaded.ok) return loaded;
      return taskStart.doctor({
        pendingId: options.pendingId || options.id,
        planes,
        proposal: loaded ? loaded.proposal : null,
      }, taskStartRuntimeFor(options, runtime, planes, loaded ? loaded.proposal : null));
    }

    if (options.startMode === 'recover') {
      const action = typeof options.recoveryAction === 'string' ? options.recoveryAction.trim().toUpperCase() : '';
      // 准备人工确认清理只更新 marker，不消费 proposal，也绝不物理删除资源。
      if (action === 'PREPARE_CONFIRMED_REMOVAL') {
        return taskStart.recover({
          pendingId: options.pendingId || options.id,
          action,
          write: options.write,
          planes,
          cwd: options.cwd,
        }, taskStartRuntimeFor(options, runtime, planes, null));
      }
      if (!action) {
        return taskStart.recover({
          pendingId: options.pendingId || options.id,
          action,
          write: options.write,
          planes,
          cwd: options.cwd,
        }, taskStartRuntimeFor(options, runtime, planes, null));
      }
      const loaded = proposalInput(options, true);
      if (!loaded.ok) return loaded;
      const state = snapshotPrecondition(planes);
      if (!state.ok) return publicFailure(state, state.snapshot.generation);
      return taskStart.recover({
        pendingId: options.pendingId || options.id,
        action,
        write: options.write,
        proposal: loaded.proposal,
        proposalDigest: loaded.proposalDigest,
        planes,
        snapshot: state.snapshot,
        cwd: options.cwd,
      }, taskStartRuntimeFor(options, runtime, planes, loaded.proposal));
    }

    const loaded = proposalInput(options, true);
    if (!loaded.ok) return loaded;
    const state = snapshotPrecondition(planes);
    if (!state.ok) return publicFailure(state, state.snapshot.generation);
    return taskStart.start({
      proposal: loaded.proposal,
      proposalDigest: loaded.proposalDigest,
      write: options.write,
      planes,
      snapshot: state.snapshot,
      cwd: options.cwd,
    }, taskStartRuntimeFor(options, runtime, planes, loaded.proposal));
  } catch (error) {
    return cliFailure(error && error.code ? error.code : 'TASK_START_CLI_FAILED', error && error.message ? error.message : String(error));
  }
}

function afterImage(writePlan) {
  return writePlan.entries.map((entry) => ({
    id: entry.id,
    lifecycle: entry.lifecycle,
    path: entry.target,
    text: entry.text,
  }));
}

function guardRecordsFor(index, changes) {
  const wanted = new Set();
  for (const change of Array.isArray(changes) ? changes : []) {
    const node = change && change.node;
    if (!node || typeof node !== 'object') continue;
    if (typeof node['parent-id'] === 'string' && node['parent-id'].trim() !== '') {
      wanted.add(node['parent-id']);
    }
    for (const relation of Array.isArray(node.relations) ? node.relations : []) {
      if (relation && typeof relation['target-id'] === 'string' && relation['target-id'].trim() !== '') {
        wanted.add(relation['target-id']);
      }
    }
  }
  const guards = [];
  for (const id of wanted) {
    const record = index && index.byId ? index.byId.get(id) : null;
    if (record && record.path) guards.push({ id, path: record.path });
  }
  return guards;
}

function realpathOrNull(value) {
  try {
    return fs.realpathSync(String(value));
  } catch (error) {
    return null;
  }
}

function normalizedBranch(value) {
  return String(value || '').replace(/^refs\/heads\//, '');
}

function isTaskWorktree(record, cwd) {
  const binding = record.node.git && typeof record.node.git === 'object'
    ? record.node.git
    : {};
  const declared = realpathOrNull(binding.worktree);
  const current = realpathOrNull(cwd);
  return declared !== null && current !== null && declared === current;
}

function integrationCloseoutReality(record, options, facts) {
  const conflicts = [];
  const binding = record.node.git && typeof record.node.git === 'object'
    ? record.node.git
    : {};
  const match = /^refs\/heads\/(.+)$/.exec(String(options.integrationRef || ''));
  if (!match) {
    conflicts.push({
      code: 'INTEGRATION_REF_NOT_CANONICAL',
      field: 'integration-ref',
      message: 'tracked history 只接受显式 refs/heads/<integration>，拒绝 HEAD、短名或 OID 冒充 integration branch',
    });
  }
  const integrationBranch = match ? match[1] : null;
  const declaredBaseBranch = normalizedBranch(binding['base-branch']);
  if (integrationBranch && declaredBaseBranch && integrationBranch !== declaredBaseBranch) {
    conflicts.push({
      code: 'INTEGRATION_BRANCH_MISMATCH',
      field: 'base-branch',
      message: `integration ref ${integrationBranch} 与 TASK 冻结的 base branch ${declaredBaseBranch} 不一致`,
    });
  }

  const targetPath = realpathOrNull(options.cwd);
  let repoRoot = null;
  let gitDir = null;
  let commonDir = null;
  let statusNow = null;
  try {
    repoRoot = realpathOrNull(gitFacts.repoRoot(options.cwd));
    const gitDirResult = gitFacts.runGit([
      'rev-parse', '--path-format=absolute', '--git-dir',
    ], { cwd: options.cwd });
    gitDir = gitDirResult.ok ? realpathOrNull(gitDirResult.stdout.trim()) : null;
    commonDir = realpathOrNull(gitFacts.gitCommonDir(options.cwd));
    statusNow = gitFacts.porcelainStatus(options.cwd);
  } catch (error) {
    conflicts.push({
      code: 'INTEGRATION_GIT_REALITY_UNREADABLE', field: 'worktree',
      message: `读不到 integration worktree 的 Git 现实：${error.message}`,
    });
  }
  if (!targetPath || !repoRoot || targetPath !== repoRoot) {
    conflicts.push({
      code: 'INTEGRATION_TARGET_NOT_REPO_ROOT', field: 'worktree',
      message: `--target 必须精确是 integration checkout root；收到 ${targetPath || options.cwd}，Git root 为 ${repoRoot || 'unknown'}`,
    });
  }
  if (!gitDir || !commonDir || gitDir === commonDir) {
    conflicts.push({
      code: 'INTEGRATION_NOT_LINKED_WORKTREE', field: 'worktree',
      message: 'canonical history 必须在独立 linked integration worktree 写入，不能使用 primary checkout 或无法解析 git-dir/common-dir 的目录',
    });
  }

  const currentBranchResult = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: options.cwd });
  const currentBranch = currentBranchResult.ok ? currentBranchResult.stdout.trim() : null;
  if (!integrationBranch || currentBranch !== integrationBranch) {
    conflicts.push({
      code: 'INTEGRATION_WORKTREE_BRANCH_MISMATCH',
      field: 'integration-ref',
      message: `canonical history 必须在 integration branch ${integrationBranch || '（无效）'} 的专用工作树写；当前为 ${currentBranch || 'detached/unknown'}`,
    });
  }

  const headOid = gitFacts.headOid(options.cwd);
  const refOidResult = match
    ? gitFacts.runGit(['rev-parse', '--verify', `${options.integrationRef}^{commit}`], { cwd: options.cwd })
    : { ok: false, stdout: '' };
  const refOid = refOidResult.ok ? refOidResult.stdout.trim() : null;
  if (!headOid || !refOid || headOid !== refOid) {
    conflicts.push({
      code: 'INTEGRATION_HEAD_MISMATCH',
      field: 'integration-ref',
      message: `integration worktree HEAD ${headOid || 'unknown'} 与 ${options.integrationRef || 'unknown'} ${refOid || 'unknown'} 不一致`,
    });
  }
  if (!Array.isArray(statusNow) || statusNow.length !== 0) {
    conflicts.push({
      code: 'INTEGRATION_WORKTREE_DIRTY',
      field: 'worktree',
      message: 'canonical history 只能写进 staged/unstaged/untracked 全部为空的专用 integration worktree',
    });
  }

  const taskReality = facts && facts.taskBindingReality ? facts.taskBindingReality : null;
  const removedProof = facts && facts.removedProof ? facts.removedProof : null;
  const declaredTaskWorktree = taskReality && taskReality.declaredWorktree
    ? taskReality.declaredWorktree
    : String(binding.worktree || '');
  const integrationWorktree = repoRoot;
  const samePath = declaredTaskWorktree && integrationWorktree && (
    (realpathOrNull(declaredTaskWorktree) && realpathOrNull(declaredTaskWorktree) === integrationWorktree)
    || path.resolve(String(declaredTaskWorktree)) === path.resolve(String(integrationWorktree))
  );
  if (samePath) {
    conflicts.push({
      code: 'TASK_WORKTREE_CANNOT_WRITE_HISTORY',
      field: 'worktree',
      message: '任务工作树不得伪造 canonical history；必须使用另一个专用 integration worktree',
    });
  }

  const taskCommon = taskReality && taskReality.commonDir
    ? taskReality.commonDir
    : (removedProof && removedProof.commonDir ? removedProof.commonDir : null);
  if (!taskCommon || !commonDir || taskCommon !== commonDir) {
    conflicts.push({
      code: 'INTEGRATION_REPOSITORY_MISMATCH',
      field: 'worktree',
      message: '任务 binding 现实（或 REMOVED 的冻结 common dir）与 integration worktree 不属于同一个 git common dir',
    });
  }

  // 这些值会进入 receipt。write 会重新读取全部字段并要求 SHA 精确相同；
  // 所以 inspect 后换分支、移动 ref、切到子目录或制造脏改动都必须重新 inspect。
  if (facts && typeof facts === 'object') {
    facts.integrationStatusNow = Array.isArray(statusNow) ? statusNow : ['<INTEGRATION_STATUS_UNAVAILABLE>'];
    facts.integrationWorktreeClean = Array.isArray(statusNow) && statusNow.length === 0;
  }

  return conflicts.length === 0
    ? {
      ok: true,
      integrationRef: options.integrationRef,
      repoRoot,
      gitDir,
      commonDir,
      currentBranch,
      headOid,
      refOid,
      statusNow,
      taskWorktree: declaredTaskWorktree,
      taskReality: taskReality || removedProof || null,
    }
    : {
      ok: false,
      code: 'INTEGRATION_CLOSEOUT_REALITY_CONFLICT',
      error: 'canonical history 写入前的 integration worktree 现实核对失败',
      detail: { conflicts, targetPath, repoRoot, gitDir, commonDir, headOid, refOid, currentBranch, statusNow },
    };
}

function exitGateFailure(unmet, generation) {
  return {
    ok: false,
    code: 'TASK_EXIT_GATE_REJECTED',
    error: 'TASK 退场未通过十四项统一收尾门',
    generation,
    written: false,
    unmet,
  };
}

function resourceGateFailure(reality, generation) {
  return exitGateFailure([{
    item: 10,
    code: 'ITEM_10_GIT_DISPOSITION',
    title: 'branch / worktree 有四种 disposition 之一',
    evidence: reality.error,
    detail: reality.detail || null,
  }], generation);
}

function integrationGateFailure(reality, generation) {
  return exitGateFailure([{
    item: 9,
    code: 'ITEM_09_INTEGRATION_OR_PARKED',
    title: '已正确集成，或明确停在等待集成',
    evidence: reality.error,
    detail: reality.detail || null,
  }], generation);
}

function recordsWithProposals(snapshot, changes, sourceId) {
  const replacements = new Map(
    (Array.isArray(changes) ? changes : [])
      .filter((change) => change && change.node && change.node.id !== sourceId)
      .map((change) => [change.node.id, {
        node: change.node,
        body: change.body,
        path: change.previousPath || `proposal:${change.node.id}`,
        title: change.node.goal || change.node.id,
        issues: [],
      }]),
  );
  const records = snapshot.records.map((record) => replacements.get(record.node.id) || record);
  for (const [id, record] of replacements) {
    if (!snapshot.records.some((existing) => existing.node.id === id)) records.push(record);
  }
  return records;
}

// tracked TASK history 和 PLAN 一样分两段落地：第一段只把候选正文写到
// integration checkout，并把 live source 停在 PARKED；第二段只在固定 ref 已经
// 读回同字节 candidate 后，才消费 live source。这样 "本地文件存在" 永远不能
// 冒充 canonical HISTORY。
const HISTORY_CANDIDATE_CONFIRMATION = 'history-candidate-freeze';

function canonicalOid(value) {
  return typeof value === 'string' && /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(value);
}

function taskHistoryCandidatePath(planes, node) {
  const target = store.nodeFilePath(planes, node);
  const relative = planes.repoRoot ? path.relative(planes.repoRoot, target) : '';
  if (!relative || relative === '.' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_PATH_INVALID',
      error: 'TASK HISTORY candidate 必须位于当前 integration repo 的 canonical history 根下',
      detail: { target, repoRoot: planes.repoRoot || null },
    };
  }
  return { ok: true, target, repoPath: relative.split(path.sep).join('/') };
}

function taskHistoryCandidateFromChange(planes, change) {
  const pathResult = taskHistoryCandidatePath(planes, change.node);
  if (!pathResult.ok) return pathResult;
  const text = nodeSchema.serializeNode(change.node, change.body);
  return {
    ok: true,
    ...pathResult,
    node: change.node,
    body: change.body,
    text,
    digest: store.digestOf(text),
  };
}

function taskRealityFreeze(facts) {
  const taskStatus = Array.isArray(facts && facts.taskStatusNow)
    ? facts.taskStatusNow
    : ['<TASK_STATUS_UNAVAILABLE>'];
  const rawBindingReality = facts && (facts.taskBindingReality || facts.removedProof)
    ? (facts.taskBindingReality || facts.removedProof)
    : null;
  // baseBranchOid 会在外部提交 history candidate 后按预期前进，不能把它冻结成
  // “task 现实”。这里只绑定 task checkout / task ref 自身与任务声明中的冻结 base，
  // REMOVED 则绑定删除前证明；status 单独摘要。
  const stableRealityKeys = [
    'declaredWorktree',
    'repoRoot',
    'gitDir',
    'commonDir',
    'currentBranch',
    'headOid',
    'taskBranch',
    'taskBranchOid',
    'baseBranch',
    'baseOid',
    'taskHeadOid',
    'capturedAt',
  ];
  const bindingReality = rawBindingReality
    ? Object.fromEntries(stableRealityKeys
      .filter((key) => Object.prototype.hasOwnProperty.call(rawBindingReality, key))
      .map((key) => [key, rawBindingReality[key]]))
    : null;
  return {
    'task-head-oid': facts && facts.taskHeadOid ? facts.taskHeadOid : null,
    'task-status-digest': store.digestOf(JSON.stringify(taskStatus)),
    'task-binding-reality-digest': store.digestOf(JSON.stringify(bindingReality)),
  };
}

function taskRealityStillFrozen(marker, facts) {
  const observed = taskRealityFreeze(facts);
  return marker
    && marker['task-head-oid'] === observed['task-head-oid']
    && marker['task-status-digest'] === observed['task-status-digest']
    && marker['task-binding-reality-digest'] === observed['task-binding-reality-digest'];
}

function taskCandidateFreezeMarker(
  candidate,
  integrationReality,
  options,
  result,
  closeoutDigest,
  facts,
) {
  return {
    action: HISTORY_CANDIDATE_CONFIRMATION,
    command: options.command,
    result,
    'candidate-path': candidate.repoPath,
    'candidate-digest': candidate.digest,
    'closeout-digest': closeoutDigest,
    'integration-ref': options.integrationRef,
    'phase1-integration-oid': integrationReality.refOid,
    ...taskRealityFreeze(facts),
  };
}

function taskCandidateFreezeMarkers(node) {
  return (Array.isArray(node && node.confirmations) ? node.confirmations : [])
    .filter((entry) => entry && typeof entry === 'object'
      && entry.action === HISTORY_CANDIDATE_CONFIRMATION);
}

function concurrentHistoryIntegration(state, options, sourceId) {
  const entries = (state && state.snapshot && Array.isArray(state.snapshot.records)
    ? state.snapshot.records
    : [])
    .filter((record) => record && record.node && record.node.id !== sourceId
      && record.node.lifecycle === 'PARKED')
    .flatMap((record) => taskCandidateFreezeMarkers(record.node)
      .filter((marker) => marker['integration-ref']
        && canonicalOid(marker['phase1-integration-oid'])
        && typeof marker['candidate-path'] === 'string'
        && /^[0-9a-f]{64}$/i.test(String(marker['candidate-digest'] || '')))
      .map((marker) => ({
        taskId: record.node.id,
        integrationRef: marker['integration-ref'],
        confirmed: true,
        started: true,
        finished: false,
        phase: 'IN_PROGRESS',
      })));
  return integrationGate.refuseSecondIntegration(options.integrationRef, entries);
}

function taskParkedCandidateProjection(historyNode, marker) {
  const parked = { ...(historyNode || {}) };
  delete parked.result;
  delete parked['closed-at'];
  const confirmations = Array.isArray(parked.confirmations) ? parked.confirmations.slice() : [];
  return {
    ...parked,
    lifecycle: 'PARKED',
    confirmations: [...confirmations, marker],
  };
}

function taskHistoryCandidateAlreadyExists(planes, options, id) {
  const existing = store.readHistoryNode(planes, id, { cwd: options.cwd });
  if (!existing) return null;
  return {
    ok: false,
    code: 'TASK_HISTORY_CANDIDATE_ALREADY_COMMITTED',
    error: `固定 integration ref 已有编号 ${id} 的 HISTORY；不能再生成同编号 candidate`,
    detail: { id, path: existing.path, integrationOid: existing.integrationOid || null },
  };
}

function prepareTaskHistoryCandidatePhase(
  planes,
  options,
  proposalChanges,
  sourceId,
  integrationReality,
  result,
  closeoutDigest,
  facts,
) {
  const sourceChanges = (Array.isArray(proposalChanges) ? proposalChanges : [])
    .filter((change) => change && change.node && change.node.id === sourceId
      && change.node.lifecycle === 'HISTORY');
  if (sourceChanges.length !== 1) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_COUNT_INVALID',
      error: `一次 tracked TASK 退场必须恰有一份 source HISTORY after-image，收到 ${sourceChanges.length} 份`,
    };
  }
  const source = sourceChanges[0];
  const existing = taskHistoryCandidateAlreadyExists(planes, options, sourceId);
  if (existing) return existing;
  const candidate = taskHistoryCandidateFromChange(planes, source);
  if (!candidate.ok) return candidate;
  if (fs.existsSync(candidate.target)) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_ALREADY_PRESENT',
      error: `history candidate 目标 ${candidate.target} 已存在；不得覆盖或重写未提交候选`,
      detail: { id: sourceId, target: candidate.target },
    };
  }
  if (taskCandidateFreezeMarkers(source.node).length > 0) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_MARKER_CONFLICT',
      error: `TASK ${sourceId} 已含保留的 history candidate marker，拒绝覆盖其冻结事实`,
    };
  }
  const marker = taskCandidateFreezeMarker(
    candidate,
    integrationReality,
    options,
    result,
    closeoutDigest,
    facts,
  );
  const parked = taskParkedCandidateProjection(candidate.node, marker);
  return {
    ok: true,
    phase: 'HISTORY_CANDIDATE',
    candidate,
    marker,
    historyChanges: [source],
    changes: proposalChanges.map((change) => (
      change === source ? { ...change, node: parked, body: candidate.body } : change
    )),
  };
}

function markerMatchesTaskCandidate(marker, candidate, historical, options, result, closeoutDigest) {
  if (!marker || !candidate || !historical) return false;
  if (marker.command !== options.command
    || marker.result !== result
    || marker['candidate-path'] !== candidate.repoPath
    || marker['candidate-digest'] !== candidate.digest
    || marker['closeout-digest'] !== closeoutDigest
    || marker['integration-ref'] !== options.integrationRef
    || !canonicalOid(marker['phase1-integration-oid'])
    || !canonicalOid(historical.integrationOid)) {
    return false;
  }
  return gitFacts.isAncestor(
    options.cwd,
    marker['phase1-integration-oid'],
    historical.integrationOid,
  );
}

function pendingTaskHistoryIntent(state, planes, options, loaded, result, facts) {
  if (!planes.tracked || !options.id) return { ok: true, phase: null };
  const record = state && state.index && state.index.byId ? state.index.byId.get(options.id) : null;
  if (!record || !record.node || record.node.lifecycle !== 'PARKED') return { ok: true, phase: null };
  const markers = taskCandidateFreezeMarkers(record.node);
  if (markers.length === 0) return { ok: true, phase: null };
  if (markers.length !== 1) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_MARKER_AMBIGUOUS',
      error: `PARKED TASK ${options.id} 同时有 ${markers.length} 份 history candidate marker，不能猜哪份可 finalize`,
    };
  }
  const marker = markers[0];
  if (marker.command !== options.command) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_COMMAND_MISMATCH',
      error: `PARKED TASK ${options.id} 正等待 ${marker.command} candidate commit；不能用 ${options.command} 改写或 finalize`,
      detail: { expected: marker.command, actual: options.command },
    };
  }
  if (marker.result !== result) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_RESULT_MISMATCH',
      error: `PARKED TASK ${options.id} 冻结的 result ${marker.result || '（空）'} 与本次 ${result} 不一致`,
      detail: { expected: marker.result || null, actual: result },
    };
  }
  if (marker['closeout-digest'] !== loaded.digest) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_CLOSEOUT_MISMATCH',
      error: 'phase1 已冻结的 closeout digest 与本次 closeout 不一致；不得用改写后的说明 finalize',
      detail: { expected: marker['closeout-digest'] || null, actual: loaded.digest },
    };
  }
  if (marker['integration-ref'] !== options.integrationRef) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_REF_MISMATCH',
      error: 'phase1 冻结的 integration ref 与本次 --integration-ref 不一致',
      detail: { expected: marker['integration-ref'] || null, actual: options.integrationRef || null },
    };
  }
  if (!taskRealityStillFrozen(marker, facts)) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_TASK_REALITY_CHANGED',
      error: 'phase1 后 task HEAD、工作树状态或 binding 现实已变化；不能用旧 candidate 删除 live source',
      detail: {
        expected: {
          taskHeadOid: marker['task-head-oid'] || null,
          taskStatusDigest: marker['task-status-digest'] || null,
          taskBindingRealityDigest: marker['task-binding-reality-digest'] || null,
        },
        actual: taskRealityFreeze(facts),
      },
    };
  }

  let historical;
  try {
    historical = store.readHistoryNode(planes, options.id, { cwd: options.cwd });
  } catch (error) {
    return {
      ok: false,
      code: error && error.code ? error.code : 'TASK_HISTORY_CANDIDATE_READ_FAILED',
      error: error && error.message ? error.message : '无法从固定 integration ref 读取 history candidate',
      detail: error && error.detail ? error.detail : null,
    };
  }
  if (!historical) {
    const candidatePath = typeof marker['candidate-path'] === 'string'
      ? path.join(planes.repoRoot, marker['candidate-path'])
      : null;
    return {
      ok: true,
      phase: 'AWAITING_HISTORY_COMMIT',
      marker,
      candidatePath,
      error: `HISTORY candidate 尚未从固定 integration ref ${options.integrationRef} 读回；请只提交 ${marker['candidate-path'] || '（候选路径缺失）'}`,
    };
  }
  const parsed = nodeSchema.parseNode(historical.text, {
    relativePath: historical.path,
    lifecycleValues: lifecycle.LIFECYCLE_VALUES,
  });
  if (!parsed.node || parsed.issues.length > 0
    || parsed.node.id !== record.node.id || parsed.node.kind !== 'TASK'
    || parsed.node.lifecycle !== 'HISTORY' || parsed.node.result !== result) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_INVALID',
      error: `固定 integration ref 中的 history candidate ${options.id} 不是同编号、同 result 的有效 HISTORY TASK`,
      detail: { path: historical.path, issues: parsed.issues, result: parsed.node && parsed.node.result },
    };
  }
  const candidate = taskHistoryCandidateFromChange(planes, {
    node: parsed.node,
    body: parsed.body,
    previousPath: record.path,
  });
  if (!candidate.ok) return candidate;
  if (historical.path !== candidate.repoPath) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_PATH_MISMATCH',
      error: '固定 integration ref 中的 candidate 路径与其 closed-at 推导的 canonical 路径不一致',
      detail: { expected: candidate.repoPath, actual: historical.path },
    };
  }
  if (!markerMatchesTaskCandidate(marker, candidate, historical, options, result, loaded.digest)) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_FREEZE_MISMATCH',
      error: '固定 ref 的 history candidate 与 phase1 冻结的路径/摘要/ref/closeout 不一致，拒绝自洽重建',
      detail: { marker, candidate: { path: candidate.repoPath, digest: candidate.digest, integrationOid: historical.integrationOid } },
    };
  }
  const consumedIntegration = (Array.isArray(candidate.node.confirmations) ? candidate.node.confirmations : [])
    .some((entry) => entry && entry.action === 'local-integrate' && entry.consumed === true);
  if (!consumedIntegration) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_AUTHORIZATION_MISSING',
      error: '固定 HISTORY candidate 未记录已消耗的 local-integrate 授权，不能删除 live source',
    };
  }
  let local = null;
  try { local = fs.readFileSync(candidate.target, 'utf8'); } catch (error) { local = null; }
  if (local === null || local !== candidate.text) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_LOCAL_MISMATCH',
      error: 'integration worktree 中的 history candidate 与固定 ref after-image 不完全一致',
      detail: {
        target: candidate.target,
        expectedDigest: candidate.digest,
        actualDigest: local === null ? null : store.digestOf(local),
      },
    };
  }
  const projectedParked = taskParkedCandidateProjection(candidate.node, marker);
  const expectedSource = nodeSchema.serializeNode(projectedParked, candidate.body);
  let actualSource = null;
  try { actualSource = fs.readFileSync(record.path, 'utf8'); } catch (error) { actualSource = null; }
  if (actualSource === null || actualSource !== expectedSource) {
    return {
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_SOURCE_MISMATCH',
      error: 'PARKED live source 不再是 phase1 生成的 candidate 投影；不能删除',
      detail: {
        source: record.path,
        expectedDigest: store.digestOf(expectedSource),
        actualDigest: actualSource === null ? null : store.digestOf(actualSource),
      },
    };
  }
  return {
    ok: true,
    phase: 'HISTORY_FINALIZE',
    marker,
    candidate,
    historical,
    historyChanges: [{ node: candidate.node, body: candidate.body, previousPath: record.path }],
  };
}

function integrationReceiptReality(integrationReality) {
  if (!integrationReality || !integrationReality.ok) return null;
  return {
    integrationRef: integrationReality.integrationRef,
    repoRoot: integrationReality.repoRoot,
    gitDir: integrationReality.gitDir,
    commonDir: integrationReality.commonDir,
    currentBranch: integrationReality.currentBranch,
    headOid: integrationReality.headOid,
    refOid: integrationReality.refOid,
    statusNow: integrationReality.statusNow,
    taskWorktree: integrationReality.taskWorktree,
  };
}

function finalizeGateReality(state, found, candidateNode, closeout, facts, current, options) {
  facts.nonHistoryDescendants = graph.nonHistoryDescendants(
    state.index,
    found.record.node.id,
  );
  facts.nonHistoryDependents = graph.dependentsOf(state.index, found.record.node.id);
  const reverse = graph.reverseRelationsOf(state.index, found.record.node.id);
  facts.splitSuccessors = reverse.SPLIT_INTO;
  facts.replaceSuccessors = reverse.SUPERSEDED_BY;
  facts.hasNonHistoryDescendant = facts.nonHistoryDescendants.length > 0;
  facts.hasNonHistoryDependent = facts.nonHistoryDependents.length > 0;
  const dispositionReality = taskCloseout.verifyResourceDisposition({
    node: candidateNode,
    closeout,
    index: state.index,
    cwd: options.cwd,
  });
  if (!dispositionReality.ok) {
    return {
      ok: false,
      outcome: resourceGateFailure(dispositionReality, state.snapshot.generation),
    };
  }
  const gated = taskCloseout.evaluateGate({
    node: candidateNode,
    facts,
    closeout,
    current,
    transferSuccessors: transferSuccessorsForGate(facts, state.index),
  });
  if (!gated.verdict.allowed) {
    return {
      ok: false,
      outcome: exitGateFailure(gated.verdict.unmet, state.snapshot.generation),
    };
  }
  return { ok: true, gated, dispositionReality };
}


function transferSuccessorsForGate(facts, index) {
  // 终态门的 TRANSFERRED 认领判定需要承接方的 write-scope；按 reverse relation
  // 里的 successor id 从当前图谱取回节点。
  const ids = [
    ...(Array.isArray(facts.replaceSuccessors) ? facts.replaceSuccessors : []),
    ...(Array.isArray(facts.splitSuccessors) ? facts.splitSuccessors : []),
  ];
  const nodes = [];
  for (const id of ids) {
    const record = index && index.byId ? index.byId.get(id) : null;
    if (record && record.node) nodes.push(record.node);
  }
  return nodes;
}

function trackedHistoryFinalizePlan(options, planes, state, found, loaded, result, pending) {
  const serialization = concurrentHistoryIntegration(state, options, found.record.node.id);
  if (!serialization.ok) {
    return publicFailure({
      ok: false,
      code: serialization.refusal.code,
      error: serialization.refusal.message,
      detail: { running: serialization.running },
    }, state.snapshot.generation);
  }
  const binding = found.record.node.git && typeof found.record.node.git === 'object'
    ? found.record.node.git
    : {};
  const facts = actualTerminalFacts(found.record.node, options, planes, loaded.closeout, found.record.body);
  const closeout = {
    ...loaded.closeout,
    result,
    targetLifecycle: 'HISTORY',
  };
  const integrationReality = integrationCloseoutReality(found.record, options, facts);
  if (!integrationReality.ok) return integrationGateFailure(integrationReality, state.snapshot.generation);
  if (!pending.historical || pending.historical.integrationOid !== integrationReality.refOid) {
    return publicFailure({
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_REF_CHANGED',
      error: 'fixed history candidate 读取后 integration ref 已变化；必须重新 inspect',
      detail: {
        candidateOid: pending.historical && pending.historical.integrationOid
          ? pending.historical.integrationOid
          : null,
        integrationOid: integrationReality.refOid || null,
      },
    }, state.snapshot.generation);
  }

  const previewIndex = graph.buildGraphIndex(recordsWithProposals(
    state.snapshot,
    [],
    found.record.node.id,
  ));
  const current = taskCloseout.currentAfterImage(
    previewIndex,
    found.record,
    closeout,
    binding.worktree || options.cwd,
  );
  const currentTarget = store.currentViewPath(
    planes,
    store.worktreeKeyFor(binding.worktree || options.cwd),
  );
  const gateReality = finalizeGateReality(
    state,
    found,
    pending.candidate.node,
    closeout,
    facts,
    current,
    options,
  );
  if (!gateReality.ok) return gateReality.outcome;
  const writePlan = store.planHistoryFinalization(planes, pending.historyChanges, {
    expectedGeneration: state.snapshot.generation,
    guardRecords: guardRecordsFor(state.index, pending.historyChanges),
    integration: {
      ref: integrationReality.integrationRef,
      oid: integrationReality.refOid,
      cwd: integrationReality.repoRoot,
    },
    extraFiles: [{ target: currentTarget, text: `${current.text.replace(/\s+$/, '')}\n` }],
  });
  const observed = {
    phase: 'HISTORY_FINALIZE',
    integrationRef: planes.integrationRef,
    result,
    targetLifecycle: 'HISTORY',
    taskBindingReality: facts.taskBindingReality || facts.removedProof || null,
    integrationReality: integrationReceiptReality(integrationReality),
    marker: pending.marker,
    candidate: {
      path: pending.candidate.repoPath,
      digest: pending.candidate.digest,
      phase1IntegrationOid: pending.marker['phase1-integration-oid'],
      fixedIntegrationOid: pending.historical.integrationOid,
    },
    gate: gateReality.gated.gateInput,
  };
  const receipt = taskCloseout.receiptFor(writePlan, loaded.digest, observed);
  const output = {
    ok: true,
    action: options.command,
    id: options.id,
    result,
    targetLifecycle: 'HISTORY',
    phase: 'HISTORY_FINALIZE',
    generation: writePlan.generation,
    ...(options.inspect ? { receipt } : {}),
    current: {
      nextEntry: current.nextEntry,
      path: currentTarget,
      afterImage: current.text,
    },
    historyCandidate: {
      path: pending.candidate.target,
      repoPath: pending.candidate.repoPath,
      digest: pending.candidate.digest,
      fixedIntegrationOid: pending.historical.integrationOid,
    },
    git: {
      taskHeadOid: facts.taskHeadOid,
      taskStatusNow: facts.taskStatusNow,
      integrationStatusNow: facts.integrationStatusNow,
      integrationReality: integrationReceiptReality(integrationReality),
    },
    moves: writePlan.entries.map((entry) => ({
      id: entry.id,
      from: entry.previousPath,
      to: entry.target,
      lifecycle: entry.lifecycle,
    })),
    afterImage: afterImage(writePlan),
    written: false,
    preview: true,
  };
  if (!options.write) return output;
  if (!options.receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_REQUIRED',
      error: 'tracked HISTORY finalize 必须携带同一 after-image 的 --receipt',
    };
  }
  if (options.receipt.toLowerCase() !== receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_MISMATCH',
      error: 'receipt 与当前 generation/source/fixed ref/Git 现实/after-image 不一致，必须重新 inspect',
    };
  }

  // store 会在锁内复读 fixed ref 与 candidate 字节；这里先复查 integration checkout
  // 仍干净且身份未漂移，避免把 receipt 签给一个刚变脏/换分支的 worktree。
  const finalFacts = actualTerminalFacts(found.record.node, options, planes, loaded.closeout, found.record.body);
  if (!taskRealityStillFrozen(pending.marker, finalFacts)) {
    return {
      ...output,
      ok: false,
      code: 'TASK_HISTORY_CANDIDATE_TASK_REALITY_CHANGED',
      error: 'finalize 提交前 task HEAD、工作树状态或 binding 现实已变化，必须重新处理 candidate',
      detail: {
        expected: {
          taskHeadOid: pending.marker['task-head-oid'] || null,
          taskStatusDigest: pending.marker['task-status-digest'] || null,
          taskBindingRealityDigest: pending.marker['task-binding-reality-digest'] || null,
        },
        actual: taskRealityFreeze(finalFacts),
      },
    };
  }
  const finalIntegration = integrationCloseoutReality(found.record, options, finalFacts);
  if (!finalIntegration.ok
    || JSON.stringify(integrationReceiptReality(finalIntegration))
      !== JSON.stringify(integrationReceiptReality(integrationReality))) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_MISMATCH',
      error: 'finalize 前 integration worktree 现实已变化，必须重新 inspect',
      detail: finalIntegration.ok ? {
        expected: integrationReceiptReality(integrationReality),
        actual: integrationReceiptReality(finalIntegration),
      } : finalIntegration.detail || null,
    };
  }
  const finalGateReality = finalizeGateReality(
    state,
    found,
    pending.candidate.node,
    closeout,
    finalFacts,
    current,
    options,
  );
  if (!finalGateReality.ok) return finalGateReality.outcome;
  const committed = store.commitHistoryFinalization(planes, writePlan);
  return {
    ...output,
    written: true,
    preview: false,
    generation: committed.generation,
    writtenPaths: committed.written,
    removedPaths: committed.removed,
  };
}

function trackedCloseoutPlan(options, planes, state) {
  const found = findTaskRecord(state.index, options.id);
  if (!found.ok) return publicFailure(found, state.snapshot.generation);
  const loaded = taskCloseout.loadCloseout(options.closeoutPath, options.cwd);
  if (!loaded.ok) return publicFailure(loaded, state.snapshot.generation);

  const result = taskCloseout.terminalResult(options.command, options.result);
  if (!result || !lifecycle.RESULT_VALUES.includes(result)) {
    return publicFailure({
      ok: false,
      code: 'TASK_FINISH_RESULT_INVALID',
      error: `tracked closeout 需要五值 result；收到 ${result || '空'}`,
    }, state.snapshot.generation);
  }
  if (loaded.closeout.result && loaded.closeout.result !== result) {
    return publicFailure({
      ok: false,
      code: 'CLOSEOUT_RESULT_MISMATCH',
      error: `CLI result ${result} 与 closeout result ${loaded.closeout.result} 不一致`,
    }, state.snapshot.generation);
  }

  // 先识别已经由 phase1 停到 PARKED 的 source。第二阶段绝不重新从 closeout
  // 生成 history：只能从固定 integration ref 读回 marker 绑定的同字节 candidate。
  const facts = actualTerminalFacts(found.record.node, options, planes, loaded.closeout, found.record.body);
  const pending = pendingTaskHistoryIntent(state, planes, options, loaded, result, facts);
  if (!pending.ok) return publicFailure(pending, state.snapshot.generation);
  if (pending.phase === 'AWAITING_HISTORY_COMMIT') {
    return {
      ok: true,
      code: 'AWAITING_HISTORY_COMMIT',
      phase: pending.phase,
      action: options.command,
      id: options.id,
      result,
      generation: state.snapshot.generation,
      integrationRef: options.integrationRef,
      historyCandidate: {
        path: pending.candidatePath,
        repoPath: pending.marker['candidate-path'] || null,
        digest: pending.marker['candidate-digest'] || null,
      },
      error: pending.error,
      written: false,
      preview: true,
    };
  }
  if (pending.phase === 'HISTORY_FINALIZE') {
    return trackedHistoryFinalizePlan(options, planes, state, found, loaded, result, pending);
  }

  facts.nonHistoryDescendants = graph.nonHistoryDescendants(state.index, found.record.node.id);
  facts.nonHistoryDependents = graph.dependentsOf(state.index, found.record.node.id);
  facts.hasNonHistoryDescendant = facts.nonHistoryDescendants.length > 0;
  facts.hasNonHistoryDependent = facts.nonHistoryDependents.length > 0;

  const taskWorktree = isTaskWorktree(found.record, options.cwd);
  const targetLifecycle = result === 'COMPLETED' && facts.integratedNow !== true
    ? 'PARKED'
    : (taskWorktree && (result === 'STOPPED' || result === 'CANCELLED')
      ? 'PARKED'
      : 'HISTORY');
  const closeout = {
    ...loaded.closeout,
    result,
    targetLifecycle,
  };
  const binding = found.record.node.git && typeof found.record.node.git === 'object'
    ? found.record.node.git
    : {};
  // HISTORY 的 inspect 本身就必须读取同一份 integration 现实；否则 receipt
  // 只能绑定文档 after-image，无法约束 inspect 之后换 checkout / 移动 ref。
  let integrationReality = null;
  let integrationAuthorization = null;
  if (targetLifecycle === 'HISTORY') {
    const serialization = concurrentHistoryIntegration(state, options, found.record.node.id);
    if (!serialization.ok) {
      return publicFailure({
        ok: false,
        code: serialization.refusal.code,
        error: serialization.refusal.message,
        detail: { running: serialization.running },
      }, state.snapshot.generation);
    }
    integrationReality = integrationCloseoutReality(found.record, options, facts);
    if (!integrationReality.ok) return integrationGateFailure(integrationReality, state.snapshot.generation);
    integrationAuthorization = taskCloseout.requireExactLocalIntegrateAuthorization(
      closeout,
      binding,
      integrationReality,
    );
    if (!integrationAuthorization.ok) return publicFailure(integrationAuthorization, state.snapshot.generation);
  }

  // 先求终态 proposal；等待集成的 COMPLETED 是唯一尚不能由 HISTORY 真值表
  // 直接构造的形状，交给 exit gate 用相同事实（只暂缓 integration 一项）核验。
  let decision = null;
  if (!(targetLifecycle === 'PARKED' && result === 'COMPLETED')) {
    decision = chooseDecision({
      ...options,
      result,
      terminalFacts: facts,
      closeoutMode: true,
    }, planes, state);
    if (!decision.ok) return publicFailure(decision, state.snapshot.generation);
    if (decision.facts) {
      Object.assign(facts, {
        splitSuccessors: decision.facts.splitSuccessors,
        replaceSuccessors: decision.facts.replaceSuccessors,
      });
    }
  }

  let proposalChanges;
  let gateNode;
  if (targetLifecycle === 'PARKED') {
    const parked = taskCloseout.parkedChange(found.record, facts);
    proposalChanges = [parked];
    gateNode = parked.node;
  } else {
    const finished = taskCloseout.historyChange(decision, closeout, facts);
    if (!finished.ok) return publicFailure(finished, state.snapshot.generation);
    const consumedNode = taskCloseout.consumeCloseoutAuthorization(
      finished.change.node,
      integrationAuthorization && integrationAuthorization.authorization,
    );
    const consumedChange = { ...finished.change, node: consumedNode };
    proposalChanges = decision.changes.map((change) => (
      change.node && change.node.id === found.record.node.id ? consumedChange : change
    ));
    gateNode = consumedNode;
  }

  const previewRecords = recordsWithProposals(
    state.snapshot,
    decision && decision.changes,
    found.record.node.id,
  );
  const previewIndex = graph.buildGraphIndex(previewRecords);
  const current = taskCloseout.currentAfterImage(
    previewIndex,
    found.record,
    closeout,
    found.record.node.git && found.record.node.git.worktree
      ? found.record.node.git.worktree
      : options.cwd,
  );
  const dispositionReality = taskCloseout.verifyResourceDisposition({
    node: gateNode,
    closeout,
    index: previewIndex,
    cwd: options.cwd,
  });
  if (!dispositionReality.ok) {
    return resourceGateFailure(dispositionReality, state.snapshot.generation);
  }
  if (targetLifecycle === 'HISTORY' && dispositionReality.removalAuthorization) {
    const consumedNode = taskCloseout.consumeCloseoutAuthorization(
      gateNode,
      dispositionReality.removalAuthorization,
    );
    proposalChanges = proposalChanges.map((change) => (
      change.node && change.node.id === found.record.node.id ? { ...change, node: consumedNode } : change
    ));
    gateNode = consumedNode;
  }

  const gated = taskCloseout.evaluateGate({
    node: gateNode,
    facts,
    closeout,
    current,
    transferSuccessors: transferSuccessorsForGate(facts, previewIndex),
  });
  if (!gated.verdict.allowed) {
    return exitGateFailure(gated.verdict.unmet, state.snapshot.generation);
  }

  const currentTarget = store.currentViewPath(
    planes,
    store.worktreeKeyFor(binding.worktree || options.cwd),
  );
  let phase = null;
  let historyCandidate = null;
  let freezeMarker = null;
  // tracked HISTORY 的第一段绝不调用普通 history move：它只在 integration
  // checkout 新建 candidate，并把 live source 投影为无 result 的 PARKED。
  if (planes.tracked && targetLifecycle === 'HISTORY') {
    const candidatePhase = prepareTaskHistoryCandidatePhase(
      planes,
      options,
      proposalChanges,
      found.record.node.id,
      integrationReality,
      result,
      loaded.digest,
      facts,
    );
    if (!candidatePhase.ok) return publicFailure(candidatePhase, state.snapshot.generation);
    phase = candidatePhase.phase;
    historyCandidate = candidatePhase.candidate;
    freezeMarker = candidatePhase.marker;
    proposalChanges = candidatePhase.changes;
  }
  const writePlan = store.planNodeWrite(planes, proposalChanges, {
    expectedGeneration: state.snapshot.generation,
    guardRecords: guardRecordsFor(state.index, proposalChanges),
    // phase1 已把 source 原子地移到 PARKED；CURRENT 必须在同一事务退出旧 ACTIVE
    // 入口，不能等 candidate commit 后才修。phase2 会按同一 after-image 再核一次。
    extraFiles: phase === 'HISTORY_CANDIDATE'
      ? [
        store.historyCandidateExtra(historyCandidate.target, historyCandidate.text),
        { target: currentTarget, text: `${current.text.replace(/\s+$/, '')}\n` },
      ]
      : [{ target: currentTarget, text: `${current.text.replace(/\s+$/, '')}\n` }],
  });
  const observed = {
    phase,
    integrationRef: planes.integrationRef,
    targetLifecycle,
    result,
    taskBindingReality: facts.taskBindingReality || facts.removedProof || null,
    integrationReality: integrationReceiptReality(integrationReality),
    marker: freezeMarker,
    candidate: historyCandidate ? {
      path: historyCandidate.repoPath,
      digest: historyCandidate.digest,
      phase1IntegrationOid: integrationReality && integrationReality.ok ? integrationReality.refOid : null,
    } : null,
    gate: gated.gateInput,
  };
  const receipt = taskCloseout.receiptFor(writePlan, loaded.digest, observed);
  const output = {
    ok: true,
    action: options.command,
    id: options.id,
    result,
    targetLifecycle,
    ...(phase === 'HISTORY_CANDIDATE' ? { code: 'AWAITING_HISTORY_COMMIT' } : {}),
    ...(phase ? { phase } : {}),
    generation: writePlan.generation,
    ...(options.inspect ? { receipt } : {}),
    gate: gated.verdict,
    current: {
      nextEntry: current.nextEntry,
      path: currentTarget,
      afterImage: current.text,
      deferred: false,
    },
    ...(historyCandidate ? {
      historyCandidate: {
        path: historyCandidate.target,
        repoPath: historyCandidate.repoPath,
        digest: historyCandidate.digest,
        commitPath: historyCandidate.repoPath,
      },
    } : {}),
    git: {
      taskHeadOid: facts.taskHeadOid,
      changedPaths: facts.changedPaths,
      taskStatusNow: facts.taskStatusNow,
      integrationStatusNow: facts.integrationStatusNow,
      integratedNow: facts.integratedNow,
      integrationReality: integrationReceiptReality(integrationReality),
    },
    moves: writePlan.entries.map((entry) => ({
      id: entry.id,
      from: entry.previousPath,
      to: entry.target,
      lifecycle: entry.lifecycle,
    })),
    afterImage: afterImage(writePlan),
    written: false,
    preview: true,
  };

  if (!options.write) return output;
  if (!options.receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_REQUIRED',
      error: 'tracked --write 必须携带同一 after-image 的 --receipt',
    };
  }
  if (options.receipt.toLowerCase() !== receipt) {
    return {
      ...output,
      ok: false,
      code: 'INSPECTION_RECEIPT_MISMATCH',
      error: 'receipt 与当前 generation/source/Git 现实/after-image 不一致，必须重新 inspect',
    };
  }
  if (phase === 'HISTORY_CANDIDATE') {
    // receipt 校验之后再读取一次 task/integration 现场。store 会重验 source、
    // target 与 generation；这里补上 store 无法知道的 checkout/ref/clean 现实。
    const finalFacts = actualTerminalFacts(found.record.node, options, planes, loaded.closeout, found.record.body);
    const finalIntegration = integrationCloseoutReality(found.record, options, finalFacts);
    const factsStable = finalFacts.taskHeadOid === facts.taskHeadOid
      && JSON.stringify(finalFacts.taskStatusNow) === JSON.stringify(facts.taskStatusNow)
      && JSON.stringify(finalFacts.changedPaths) === JSON.stringify(facts.changedPaths)
      && finalFacts.integratedNow === facts.integratedNow;
    if (!finalIntegration.ok
      || !factsStable
      || JSON.stringify(integrationReceiptReality(finalIntegration))
        !== JSON.stringify(integrationReceiptReality(integrationReality))) {
      return {
        ...output,
        ok: false,
        code: 'INSPECTION_RECEIPT_MISMATCH',
        error: 'phase1 写入前 task/integration Git 现实已变化，必须重新 inspect',
        detail: finalIntegration.ok ? {
          expectedIntegration: integrationReceiptReality(integrationReality),
          actualIntegration: integrationReceiptReality(finalIntegration),
          expectedTaskHead: facts.taskHeadOid,
          actualTaskHead: finalFacts.taskHeadOid,
        } : finalIntegration.detail || null,
      };
    }
  }
  if (targetLifecycle !== 'HISTORY') {
    const taskReality = terminalGitReality(found.record, options);
    if (!taskReality.ok) return publicFailure(taskReality, state.snapshot.generation);
  }

  const committed = store.commitNodeWrite(planes, writePlan);
  return {
    ...output,
    written: true,
    preview: false,
    generation: committed.generation,
    writtenPaths: committed.written,
  };
}

function runUnsafe(argv, runtime) {
  const parsed = parseArgumentsUnsafe(argv);
  if (!parsed.ok) return { ok: false, code: 'ARGUMENT_ERROR', error: parsed.error, written: false };
  const options = parsed.options;
  if (options.help) {
    return {
      ok: true,
      action: 'help',
      command: options.command || null,
      usage: usage(),
      written: false,
      preview: true,
    };
  }
  if (options.command === 'propose') return runPropose(options, runtime);
  if (options.command === 'start') return runStartFamily(options, runtime);
  try {
    const planes = loadPlanes(options, runtime);
    const state = snapshotPrecondition(planes);
    if (!state.ok) return publicFailure(state, state.snapshot.generation);
    if (options.command === 'record') return runRecord(options, planes, state);
    const terminalCommands = ['withdraw', 'finish', 'cancel', 'stop', 'split', 'replace'];
    if (planes.tracked && terminalCommands.includes(options.command) && options.closeoutPath) {
      const integration = requireExplicitIntegrationRef(planes, options, options.command);
      if (integration) return publicFailure(integration, state.snapshot.generation);
      return trackedCloseoutPlan(options, planes, state);
    }
    const decision = chooseDecision(options, planes, state);
    if (!decision.ok) return publicFailure(decision, state.snapshot.generation);

    const { changes, ...summary } = decision;
    const writePlan = store.planNodeWrite(planes, changes, {
      expectedGeneration: state.snapshot.generation,
      guardRecords: guardRecordsFor(state.index, changes),
    });
    const terminal = writePlan.entries.some((entry) => entry.area === 'history');
    const output = {
      ...summary,
      generation: writePlan.generation,
      moves: writePlan.entries.map((entry) => ({
        id: entry.id,
        from: entry.previousPath,
        to: entry.target,
        lifecycle: entry.lifecycle,
      })),
      afterImage: afterImage(writePlan),
      written: false,
      preview: true,
    };

    // tracked 终态从 08 起必须经过 closeout + inspect receipt；旧 05 seam 仍
    // 可先完成真值判定，但不能再靠 awaiting 标记绕过统一收尾门。
    if (planes.tracked && terminal) {
      return {
        ...output,
        ok: false,
        code: 'CLOSEOUT_REQUIRED',
        error: 'tracked 终态必须提供 --closeout，先 --inspect，再以同一 --receipt --write',
      };
    }
    if (!options.write) return output;
    const committed = store.commitNodeWrite(planes, writePlan);
    return {
      ...output,
      written: true,
      preview: false,
      generation: committed.generation,
      writtenPaths: committed.written,
    };
  } catch (error) {
    return {
      ok: false,
      code: error && error.code ? error.code : 'TASK_OPERATION_FAILED',
      error: error && error.message ? error.message : String(error),
      detail: error && error.detail ? error.detail : null,
      written: false,
    };
  }
}

function run(argv, runtime) {
  try {
    return publicResult(runUnsafe(argv, runtime));
  } catch (error) {
    return publicResult({
      ok: false,
      code: error && error.code ? error.code : 'TASK_OPERATION_FAILED',
      error: error && error.message ? error.message : String(error),
      detail: error && error.detail ? error.detail : null,
      written: false,
    });
  }
}

if (require.main === module) {
  const outcome = run(process.argv.slice(2));
  process.stdout.write(`${JSON.stringify(outcome, null, 2)}\n`);
  process.exitCode = outcome.ok ? 0 : 1;
}

module.exports = {
  COMMANDS,
  parseArguments,
  run,
};
