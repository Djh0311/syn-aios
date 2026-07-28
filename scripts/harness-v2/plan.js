#!/usr/bin/env node
'use strict';

// Adaptive Harness v0.5 — 计划节点 CLI（AH-050-04 / AH-050-08）
//
// 需求溯源：LY-1 · LY-3 · LY-4 · LY-5 · EX-3 · EX-5 · EX-7 · EX-9 ·
//           GIT-2 · GIT-4 · GIT-5 · KP-13
//
// 所有状态变化都来自显式命令；默认 dry-run，只有 --write 才提交 store 事务。
// 这条入口只接收 ROOT_PLAN / PHASE_PLAN。TASK 生命周期归 AH-050-05。

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const nodeSchema = require('./lib/node-schema');
const lifecycle = require('./lib/lifecycle');
const graph = require('./lib/graph');
const store = require('./lib/store');
const planLifecycle = require('./lib/plan-lifecycle');
const gitFacts = require('./lib/git-facts');
const integrationGate = require('./lib/integration-gate');

const COMMANDS = Object.freeze([
  'create',
  'add-child',
  'activate',
  'park',
  'resume',
  'withdraw',
  'replace',
  'close',
]);

const TERMINAL_COMMANDS = Object.freeze(['withdraw', 'replace', 'close']);
const HISTORY_CANDIDATE_CONFIRMATION = 'history-candidate-freeze';

function usage() {
  return [
    `用法：plan.js <${COMMANDS.join('|')}>`,
    '[--id <编号>] [--parent-id <编号>] [--node <Markdown>]',
    '[--closeout <JSON>] [--target <目录>] [--live-root <目录>]',
    '[--history-root <目录>] [--integration-ref <固定引用>]',
    '[--carrier-task <已冻结 TASK 编号>]',
    '[--inspect] [--receipt <sha256> --write]',
  ].join(' ');
}

function parseArguments(argv) {
  const options = {
    command: null,
    id: null,
    parentId: null,
    nodePath: null,
    closeout: null,
    cwd: process.cwd(),
    targetExplicit: false,
    liveRoot: null,
    historyRoot: null,
    integrationRef: null,
    carrierTask: null,
    json: false,
    inspect: false,
    receipt: null,
    write: false,
  };
  const list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length && !list[0].startsWith('--')) options.command = list.shift();
  while (list.length) {
    const token = list.shift();
    if (token === '--write') { options.write = true; continue; }
    if (token === '--inspect') { options.inspect = true; continue; }
    if (token === '--json') { options.json = true; continue; }
    if (token === '--receipt') { options.receipt = list.shift() || null; continue; }
    if (token === '--id') { options.id = list.shift() || null; continue; }
    if (token === '--parent-id') { options.parentId = list.shift() || null; continue; }
    if (token === '--node') { options.nodePath = list.shift() || null; continue; }
    if (token === '--closeout') { options.closeout = list.shift() || null; continue; }
    if (token === '--target') {
      const target = list.shift() || null;
      if (target) {
        options.cwd = target;
        options.targetExplicit = true;
      }
      continue;
    }
    if (token === '--live-root') { options.liveRoot = list.shift() || null; continue; }
    if (token === '--history-root') { options.historyRoot = list.shift() || null; continue; }
    if (token === '--integration-ref') { options.integrationRef = list.shift() || null; continue; }
    if (token === '--carrier-task') { options.carrierTask = list.shift() || null; continue; }
    return { ok: false, error: `未知参数 ${token}`, options };
  }
  if (!options.command || !COMMANDS.includes(options.command)) {
    return { ok: false, error: usage(), options };
  }
  if (options.inspect && options.write) {
    return { ok: false, error: '--inspect 与 --write 不能同时使用；先 inspect 取得 receipt，再单独 write', options };
  }
  if (options.receipt && !options.write) {
    return { ok: false, error: '--receipt 只可与 --write 一起使用', options };
  }
  return { ok: true, error: null, options };
}

function loadPlanes(options) {
  return store.resolvePlanes({
    cwd: options.cwd,
    liveRoot: options.liveRoot,
    historyRoot: options.historyRoot,
    integrationRef: options.integrationRef,
  });
}

// live/history 根覆写是测试夹具的便利，不是 tracked HISTORY 的第二套落点。
// 一个非 Git --target 也不能借绝对覆写把写入导向任意 Git worktree；否则会把
// canonical history 静默降成 untracked。旧 nodes fixture 没有 --target：只有当
// 它的两个覆写根都确实在任一 Git worktree 之外时，才把它视作非 Git fixture。
function nearestExistingPath(value) {
  let cursor = path.resolve(String(value));
  while (!fs.existsSync(cursor)) {
    const parent = path.dirname(cursor);
    if (parent === cursor) return null;
    cursor = parent;
  }
  return cursor;
}

function gitRootForPath(value) {
  const probePath = nearestExistingPath(value);
  if (!probePath) return null;
  const probe = gitFacts.runGit(['rev-parse', '--show-toplevel'], { cwd: probePath });
  return probe.ok && probe.stdout.trim() ? probe.stdout.trim() : null;
}

function canonicalRootOverrideFailure(options) {
  if (!options.liveRoot && !options.historyRoot) return null;
  const targetRepoRoot = gitRootForPath(options.cwd);
  const overrideRoots = [options.liveRoot, options.historyRoot]
    .filter(Boolean)
    .map((root) => ({ path: path.resolve(String(root)), repoRoot: gitRootForPath(root) }));
  const externalFixture = Boolean(options.liveRoot)
    && !options.targetExplicit
    && overrideRoots.length > 0
    && overrideRoots.every((root) => !root.repoRoot);
  if (!targetRepoRoot && overrideRoots.every((root) => !root.repoRoot)) return null;
  if (externalFixture) return null;
  return {
    ok: false,
    code: 'PLAN_CANONICAL_ROOT_OVERRIDE_FORBIDDEN',
    error: 'Git worktree 中不得用 --live-root 或 --history-root 改写受管 canonical 平面；这些覆写只供非 Git fixture 使用',
    detail: {
      target: options.cwd,
      targetRepoRoot,
      overrideRoots,
    },
  };
}

function readJson(filePath, cwd) {
  if (!filePath) return { ok: true, value: {} };
  const absolute = path.resolve(cwd, filePath);
  let text;
  try {
    text = fs.readFileSync(absolute, 'utf8');
  } catch (error) {
    return { ok: false, code: 'CLOSEOUT_READ_FAILED', error: `读不到 closeout ${absolute}：${error.message}` };
  }
  try {
    const value = JSON.parse(text);
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { ok: false, code: 'CLOSEOUT_INVALID', error: `closeout ${absolute} 必须是 JSON object` };
    }
    return { ok: true, value };
  } catch (error) {
    return { ok: false, code: 'CLOSEOUT_INVALID', error: `closeout ${absolute} 不是合法 JSON：${error.message}` };
  }
}

function readCandidate(filePath, cwd) {
  if (!filePath) return { ok: false, code: 'NODE_INPUT_REQUIRED', error: '需要 --node <Markdown>' };
  const absolute = path.resolve(cwd, filePath);
  let text;
  try {
    text = fs.readFileSync(absolute, 'utf8');
  } catch (error) {
    return { ok: false, code: 'NODE_INPUT_READ_FAILED', error: `读不到计划节点 ${absolute}：${error.message}` };
  }
  const candidate = nodeSchema.parseNode(text, {
    relativePath: absolute,
    lifecycleValues: lifecycle.PLAN_LIFECYCLE_VALUES,
  });
  return { ok: true, candidate, absolute };
}

function snapshotPrecondition(planes) {
  const snapshot = store.readLiveSnapshot(planes);
  const schema = [];
  for (const record of snapshot.records) {
    for (const problem of record.issues || []) schema.push({ path: record.path, ...problem });
  }
  const index = graph.buildGraphIndex(snapshot.records);
  const integrity = graph.graphIntegrityIssues(index);
  if (schema.length > 0 || integrity.length > 0) {
    return {
      ok: false,
      code: 'LIVE_GRAPH_PRECONDITION_FAILED',
      error: 'live graph 含坏节点；先修复被点名的问题，本次不规划写入',
      generation: snapshot.generation,
      schema,
      integrity,
      snapshot,
      index,
    };
  }
  return { ok: true, snapshot, index };
}

function historyHas(planes, id, options) {
  return Boolean(store.readHistoryNode(planes, id, { cwd: options.cwd }));
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

function integrationBranchName(ref) {
  const prefix = 'refs/heads/';
  const name = typeof ref === 'string' && ref.startsWith(prefix)
    ? ref.slice(prefix.length)
    : '';
  return name === '' ? null : name;
}

// canonical history 的写入点必须可判定为一个 integration worktree。HEAD、短名和
// 裸 OID 都无法证明“现在检出的不是 task branch”，所以只接受显式本地分支 ref。
function requireCanonicalHistoryRef(planes, options, command) {
  const explicit = requireExplicitIntegrationRef(planes, options, command);
  if (explicit) return explicit;
  if (planes.tracked && !integrationBranchName(options.integrationRef)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_REF_INVALID',
      error: `${command} 写 canonical HISTORY 只接受 --integration-ref refs/heads/<integration>；不能用 HEAD、短名或裸 OID 冒充 integration worktree`,
    };
  }
  return null;
}

function requireId(options) {
  return options.id
    ? null
    : { ok: false, code: 'PLAN_ID_REQUIRED', error: `${options.command} 需要 --id`, written: false };
}

function chooseDecision(options, planes, state, closedAt) {
  const { snapshot, index } = state;
  if (options.command === 'create' || options.command === 'add-child' || options.command === 'replace') {
    const requiredRef = requireExplicitIntegrationRef(planes, options, options.command);
    if (requiredRef) return requiredRef;
    const loaded = readCandidate(options.nodePath, options.cwd);
    if (!loaded.ok) return loaded;
    const candidateId = loaded.candidate && loaded.candidate.node
      ? loaded.candidate.node.id
      : null;
    const historyExists = candidateId ? historyHas(planes, candidateId, options) : false;
    if (options.command === 'create') {
      return planLifecycle.createRoot({
        index,
        candidate: loaded.candidate,
        historyExists,
      });
    }
    if (options.command === 'add-child') {
      if (!options.parentId) {
        return { ok: false, code: 'PARENT_ID_REQUIRED', error: 'add-child 需要 --parent-id' };
      }
      return planLifecycle.addChild({
        index,
        candidate: loaded.candidate,
        parentId: options.parentId,
        historyExists,
      });
    }
    const missing = requireId(options);
    if (missing) return missing;
    return planLifecycle.replace({
      index,
      id: options.id,
      candidate: loaded.candidate,
      historyExists,
      closedAt,
    });
  }

  const missing = requireId(options);
  if (missing) return missing;
  if (options.command === 'withdraw' || options.command === 'close') {
    const requiredRef = requireExplicitIntegrationRef(planes, options, options.command);
    if (requiredRef) return requiredRef;
  }
  if (options.command === 'activate') {
    return planLifecycle.transition({ index, id: options.id, action: 'activate', to: 'ACTIVE' });
  }
  if (options.command === 'park') {
    return planLifecycle.transition({ index, id: options.id, action: 'park', to: 'PARKED' });
  }
  if (options.command === 'resume') {
    return planLifecycle.transition({ index, id: options.id, action: 'resume', to: 'ACTIVE' });
  }
  if (options.command === 'withdraw') {
    return planLifecycle.withdraw({ index, id: options.id, closedAt });
  }
  const closeout = readJson(options.closeout, options.cwd);
  if (!closeout.ok) return closeout;
  return planLifecycle.close({
    records: snapshot.records,
    index,
    id: options.id,
    closeout: closeout.value,
    closedAt,
  });
}

function publicFailure(outcome, fallbackGeneration) {
  const detail = outcome && outcome.detail && typeof outcome.detail === 'object'
    ? outcome.detail
    : {};
  const result = {
    ok: false,
    code: outcome && outcome.code ? outcome.code : 'PLAN_OPERATION_FAILED',
    error: outcome && outcome.error ? outcome.error : '计划操作失败',
    generation: outcome && outcome.generation !== undefined
      ? outcome.generation
      : fallbackGeneration,
    written: false,
    ...detail,
  };
  if (outcome && Array.isArray(outcome.schema)) result.schema = outcome.schema;
  if (outcome && Array.isArray(outcome.integrity)) result.integrity = outcome.integrity;
  return result;
}

function isCanonicalOid(value) {
  return /^[0-9a-f]{40}$|^[0-9a-f]{64}$/i.test(String(value || ''));
}

function resolveIntegrationOid(cwd, integrationRef) {
  const resolved = gitFacts.runGit([
    'rev-parse',
    '--verify',
    '--end-of-options',
    `${integrationRef}^{commit}`,
  ], { cwd });
  const oid = resolved.ok ? resolved.stdout.trim() : '';
  if (!isCanonicalOid(oid)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_REF_UNRESOLVED',
      error: `无法把固定 integration ref ${integrationRef} 解析为单一 commit OID`,
      detail: { ref: integrationRef, stderr: resolved.stderr || '' },
    };
  }
  return { ok: true, oid };
}

/**
 * HISTORY 是受 Git 跟踪的 canonical 真相。这里只读取现场：真正写入仍由 store 的
 * generation / source digest 事务完成。对于本地分支 ref，额外核对当前 worktree
 * 正在检出的就是那条 integration 分支，不能让同 OID 的 task branch 冒充它。
 */
function inspectIntegrationWorktree(planes, options) {
  if (!planes.tracked) return { ok: true, eligible: true, tracked: false };
  let checkoutRoot;
  try {
    checkoutRoot = realpathOrNull(gitFacts.repoRoot(options.cwd));
  } catch (error) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_WORKTREE_UNAVAILABLE',
      error: '无法解析 integration checkout root，不能把子目录或任意 cwd 当作独立 worktree',
      detail: { cwd: options.cwd },
    };
  }
  if (!checkoutRoot) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_WORKTREE_UNAVAILABLE',
      error: '无法解析 integration checkout root realpath，不能写 canonical history',
      detail: { cwd: options.cwd },
    };
  }
  const resolved = resolveIntegrationOid(checkoutRoot, options.integrationRef);
  if (!resolved.ok) return resolved;
  const headOid = gitFacts.headOid(checkoutRoot);
  if (!isCanonicalOid(headOid)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_HEAD_UNAVAILABLE',
      error: '无法读取当前 HEAD OID，不能把它当成 integration worktree',
    };
  }
  let status;
  try {
    status = gitFacts.porcelainStatus(checkoutRoot);
  } catch (error) {
    return {
      ok: false,
      code: error && error.code ? error.code : 'PLAN_HISTORY_STATUS_UNAVAILABLE',
      error: error && error.message ? error.message : '无法读取 integration worktree 状态',
    };
  }
  const expectedBranch = integrationBranchName(options.integrationRef);
  let currentBranch = null;
  if (expectedBranch) {
    const branch = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: checkoutRoot });
    if (!branch.ok || !branch.stdout.trim()) {
      return {
        ok: false,
        code: 'PLAN_HISTORY_BRANCH_UNAVAILABLE',
        error: '无法确认当前检出分支，不能把 task worktree 当 integration worktree',
      };
    }
    currentBranch = branch.stdout.trim();
  }
  const headMatches = headOid.toLowerCase() === resolved.oid.toLowerCase();
  const branchMatches = expectedBranch ? currentBranch === expectedBranch : true;
  const clean = status.length === 0;
  return {
    ok: true,
    tracked: true,
    ref: options.integrationRef,
    integrationOid: resolved.oid,
    headOid,
    checkoutRoot,
    currentBranch,
    expectedBranch,
    headMatches,
    branchMatches,
    clean,
    status,
    eligible: headMatches && branchMatches && clean,
  };
}

function realpathOrNull(value) {
  try {
    return fs.realpathSync(String(value));
  } catch (error) {
    return null;
  }
}

function bindingBranchName(value) {
  const raw = typeof value === 'string' ? value.trim() : '';
  if (raw === '' || raw === 'HEAD' || raw.startsWith('refs/') && !raw.startsWith('refs/heads/')) {
    return null;
  }
  const name = raw.replace(/^refs\/heads\//, '');
  return name === '' ? null : name;
}

function historyRecord(planes, id, options) {
  const found = store.readHistoryNode(planes, id, { cwd: options.cwd });
  if (!found) return null;
  const parsed = nodeSchema.parseNode(found.text, {
    relativePath: found.path,
    lifecycleValues: lifecycle.LIFECYCLE_VALUES,
  });
  return {
    node: parsed.node,
    body: parsed.body,
    title: parsed.title,
    sections: parsed.sections,
    issues: parsed.issues,
    path: found.path,
    area: 'history',
    source: 'history',
  };
}

function recordForCarrierLineage(state, planes, options, id) {
  const live = state && state.index && state.index.byId ? state.index.byId.get(id) : null;
  if (live) return { ok: true, record: { ...live, source: 'live' } };
  const historical = historyRecord(planes, id, options);
  if (!historical) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_LINEAGE_MISSING',
      error: `carrier 谱系中的节点 ${id} 不在 live 或固定 integration history；不能猜父计划`,
      detail: { id },
    };
  }
  if (!historical.node || historical.node.id !== id || historical.issues.length > 0) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_LINEAGE_INVALID',
      error: `carrier 谱系中的历史节点 ${id} 不是有效 canonical node`,
      detail: { id, path: historical.path, issues: historical.issues },
    };
  }
  return { ok: true, record: historical };
}

function carrierLineage(state, planes, options, carrierId) {
  const chain = [];
  const seen = new Set();
  let cursor = carrierId;
  while (typeof cursor === 'string' && cursor.trim() !== '') {
    if (seen.has(cursor)) {
      return {
        ok: false,
        code: 'PLAN_HISTORY_CARRIER_LINEAGE_CYCLE',
        error: `carrier 谱系在 ${cursor} 出现循环，不能把它当成 PLAN 退场 authority`,
        detail: { chain: chain.slice(), at: cursor },
      };
    }
    seen.add(cursor);
    const loaded = recordForCarrierLineage(state, planes, options, cursor);
    if (!loaded.ok) return loaded;
    const record = loaded.record;
    chain.push({ id: record.node.id, record });
    const parentId = record.node['parent-id'];
    cursor = typeof parentId === 'string' && parentId.trim() !== '' ? parentId.trim() : null;
  }
  return {
    ok: true,
    records: chain.slice().reverse(),
    lineage: chain.slice().reverse().map((entry) => entry.id),
  };
}

function terminalPlanIds(changes) {
  return (Array.isArray(changes) ? changes : [])
    .map((change) => change && change.node)
    .filter((node) => node && node.lifecycle === 'HISTORY'
      && (node.kind === 'ROOT_PLAN' || node.kind === 'PHASE_PLAN'))
    .map((node) => node.id);
}

function publicCarrier(carrier) {
  return {
    id: carrier.id,
    source: carrier.source,
    lineage: carrier.lineage.slice(),
    baseBranch: carrier.baseBranch,
    baseOid: carrier.baseOid,
    taskBranch: carrier.taskBranch,
    taskWorktree: carrier.taskWorktree,
    taskHead: carrier.taskHead,
  };
}

/**
 * PLAN 本身没有 Git binding，不能让调用者提供的 ref 自证为 integration。
 * 因此 tracked terminal 必须锚定到谱系内一个已冻结 TASK：它的 base branch、
 * task branch 和 worktree 是开工时声明的事实，而不是本次 CLI 输入。
 *
 * v0.5 的信任边界是本地操作者显式拥有并选择 --integration-ref；本入口不试图
 * 建立远端保护分支/仓库配置这一层新 authority。这里要防的是把正确的本地 branch
 * 意外换成 task checkout、子目录或不相干 checkout，而不是抵御拥有本地 Git 写权的
 * 操作者伪造整条分支历史。
 */
function resolveCarrierAuthority(state, planes, options, changes) {
  if (!options.carrierTask) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_REQUIRED',
      error: 'tracked PLAN HISTORY 需要 --carrier-task <TASK_ID>，以冻结 TASK binding 证明 integration authority',
    };
  }
  const loaded = recordForCarrierLineage(state, planes, options, options.carrierTask);
  if (!loaded.ok) return loaded;
  const carrierRecord = loaded.record;
  const carrier = carrierRecord.node;
  if (!carrier || carrier.kind !== 'TASK' || carrier.lifecycle !== 'HISTORY'
    || (carrierRecord.issues || []).length > 0) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_INVALID',
      error: `carrier ${options.carrierTask} 必须是已冻结的 HISTORY TASK canonical node`,
      detail: { id: options.carrierTask, path: carrierRecord.path, issues: carrierRecord.issues || [] },
    };
  }
  const binding = carrier.git;
  const baseBranch = bindingBranchName(binding && binding['base-branch']);
  const taskBranch = bindingBranchName(binding && binding['task-branch']);
  const taskWorktree = realpathOrNull(binding && binding.worktree);
  if (!binding || typeof binding !== 'object' || Array.isArray(binding)
    || !baseBranch || !taskBranch || !taskWorktree
    || !isCanonicalOid(binding['base-oid'])) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_BINDING_INVALID',
      error: `carrier ${options.carrierTask} 缺少可回查的冻结 git binding`,
      detail: { id: options.carrierTask, path: carrierRecord.path },
    };
  }

  let taskCheckoutRoot;
  let taskHead;
  let taskStatus;
  let observedTaskBranch;
  try {
    taskCheckoutRoot = realpathOrNull(gitFacts.repoRoot(taskWorktree));
    const branch = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: taskWorktree });
    observedTaskBranch = branch.ok ? branch.stdout.trim() : null;
    taskHead = gitFacts.headOid(taskWorktree);
    taskStatus = gitFacts.porcelainStatus(taskWorktree);
  } catch (error) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_REALITY_UNAVAILABLE',
      error: `无法复核 carrier ${carrier.id} 声明 task checkout 的 branch/head/status：${error.message}`,
      detail: { carrier: carrier.id, taskWorktree },
    };
  }
  if (!taskCheckoutRoot || taskCheckoutRoot !== taskWorktree) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_CHECKOUT_MISMATCH',
      error: `carrier ${carrier.id} 的 git.worktree 必须精确指向其 checkout root，不能是子目录或别名`,
      detail: { carrier: carrier.id, declared: taskWorktree, checkoutRoot: taskCheckoutRoot },
    };
  }
  if (observedTaskBranch !== taskBranch) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_BRANCH_MISMATCH',
      error: `carrier ${carrier.id} 的 task checkout 当前分支不是冻结的 ${taskBranch}`,
      detail: { carrier: carrier.id, expected: taskBranch, actual: observedTaskBranch },
    };
  }
  if (!isCanonicalOid(taskHead) || !gitFacts.isAncestor(taskWorktree, binding['base-oid'], taskHead)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_BASE_OID_MISMATCH',
      error: `carrier ${carrier.id} 的 task checkout HEAD 不包含冻结 base OID`,
      detail: { carrier: carrier.id, baseOid: binding['base-oid'], taskHead },
    };
  }
  if (taskStatus.length > 0) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_WORKTREE_DIRTY',
      error: `carrier ${carrier.id} 的 task checkout 不干净，不能作为冻结 authority`,
      detail: { carrier: carrier.id, taskWorktree, status: taskStatus },
    };
  }

  const lineage = carrierLineage(state, planes, options, carrier.id);
  if (!lineage.ok) return lineage;
  const targets = terminalPlanIds(changes);
  const unrelated = targets.filter((id) => !lineage.lineage.includes(id));
  if (unrelated.length > 0) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_UNRELATED',
      error: `carrier ${carrier.id} 的显式 parent 谱系不包含待退场 PLAN ${unrelated.join(', ')}`,
      detail: { carrier: carrier.id, lineage: lineage.lineage, targets: unrelated },
    };
  }

  const expectedRef = `refs/heads/${baseBranch}`;
  if (options.integrationRef !== expectedRef) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_INTEGRATION_REF_MISMATCH',
      error: `--integration-ref 必须精确等于 carrier ${carrier.id} 冻结的 ${expectedRef}`,
      detail: { carrier: carrier.id, expectedRef, actualRef: options.integrationRef || null },
    };
  }

  let integrationWorktree;
  try {
    integrationWorktree = realpathOrNull(gitFacts.repoRoot(options.cwd));
  } catch (error) {
    integrationWorktree = null;
  }
  if (!integrationWorktree) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_WORKTREE_UNAVAILABLE',
      error: '无法解析当前 integration worktree realpath，不能写 canonical history',
      detail: { cwd: options.cwd },
    };
  }
  if (integrationWorktree === taskWorktree) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_WORKTREE',
      error: `carrier ${carrier.id} 的 task worktree 不得写 canonical PLAN history`,
      detail: { carrier: carrier.id, worktree: taskWorktree },
    };
  }
  let carrierCommon;
  let integrationCommon;
  try {
    carrierCommon = realpathOrNull(gitFacts.gitCommonDir(taskWorktree));
    integrationCommon = realpathOrNull(gitFacts.gitCommonDir(options.cwd));
  } catch (error) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_REPOSITORY_UNAVAILABLE',
      error: `无法核对 carrier/integration 的 git common dir：${error.message}`,
      detail: { carrier: carrier.id },
    };
  }
  if (!carrierCommon || !integrationCommon || carrierCommon !== integrationCommon) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_REPOSITORY_MISMATCH',
      error: `carrier ${carrier.id} 与当前 integration worktree 不属于同一 git common dir`,
      detail: { carrier: carrier.id, carrierCommon, integrationCommon },
    };
  }

  return {
    ok: true,
    id: carrier.id,
    source: carrierRecord.source,
    lineage: lineage.lineage,
    baseBranch,
    baseOid: binding['base-oid'],
    taskBranch,
    taskWorktree,
    taskHead,
    integrationWorktree,
  };
}

function carrierWorktreeReality(carrier, integration, cwd) {
  if (integration.checkoutRoot !== carrier.integrationWorktree) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_INTEGRATION_WORKTREE_MISMATCH',
      error: 'integration checkout root 在 carrier 复核之间发生变化，不能继续写 canonical history',
      detail: { carrier: publicCarrier(carrier), integration },
    };
  }
  if (integration.checkoutRoot === carrier.taskWorktree) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_WORKTREE',
      error: `carrier ${carrier.id} 的 task checkout 不得写 canonical PLAN history`,
      detail: { carrier: publicCarrier(carrier), integration },
    };
  }
  if (integration.currentBranch === carrier.taskBranch) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_TASK_BRANCH',
      error: `当前分支 ${integration.currentBranch} 是 carrier ${carrier.id} 的 task branch，不能自报为 integration`,
      detail: { carrier: publicCarrier(carrier), integration },
    };
  }
  if (!gitFacts.isAncestor(integration.checkoutRoot || cwd, carrier.baseOid, integration.integrationOid)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CARRIER_BASE_OID_MISMATCH',
      error: `integration HEAD ${integration.integrationOid} 不包含 carrier ${carrier.id} 冻结的 base OID ${carrier.baseOid}`,
      detail: { carrier: publicCarrier(carrier), integration },
    };
  }
  return { ok: true };
}

function isTerminalCommand(command) {
  return TERMINAL_COMMANDS.includes(command);
}

function terminalHistoryChanges(changes) {
  return (Array.isArray(changes) ? changes : [])
    .filter((change) => change && change.node && change.node.lifecycle === 'HISTORY'
      && (change.node.kind === 'ROOT_PLAN' || change.node.kind === 'PHASE_PLAN'));
}

function historyCandidatePath(planes, node) {
  const target = store.nodeFilePath(planes, node);
  const relative = path.relative(planes.repoRoot, target);
  if (!relative || relative === '.' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_PATH_INVALID',
      error: 'HISTORY candidate 必须位于当前 integration repo 的 canonical history 路径内',
      detail: { target, repoRoot: planes.repoRoot },
    };
  }
  return { ok: true, target, repoPath: relative.split(path.sep).join('/') };
}

function withoutHistoryFields(node) {
  const next = { ...(node || {}) };
  delete next.result;
  delete next['closed-at'];
  return next;
}

function candidateFreezeMarker(candidate, integration, command) {
  return {
    action: HISTORY_CANDIDATE_CONFIRMATION,
    command,
    'candidate-path': candidate.repoPath,
    'candidate-digest': candidate.digest,
    'integration-ref': integration.ref,
    'phase1-integration-oid': integration.integrationOid,
  };
}

function freezeMarkers(node) {
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
    .flatMap((record) => freezeMarkers(record.node)
      .filter((marker) => marker['integration-ref']
        && isCanonicalOid(marker['phase1-integration-oid'])
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

function parkedNodeWithFreeze(historyNode, marker) {
  const base = withoutHistoryFields(historyNode);
  const confirmations = Array.isArray(base.confirmations) ? base.confirmations.slice() : [];
  return {
    ...base,
    lifecycle: 'PARKED',
    confirmations: [...confirmations, marker],
  };
}

function candidateFromHistoryChange(planes, change) {
  const pathResult = historyCandidatePath(planes, change.node);
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

function canonicalHistoryAlreadyExists(planes, options, id) {
  const existing = store.readHistoryNode(planes, id, { cwd: options.cwd });
  if (!existing) return null;
  return {
    ok: false,
    code: 'PLAN_HISTORY_CANDIDATE_ALREADY_COMMITTED',
    error: `固定 integration ref 已有编号 ${id} 的 HISTORY；不能再生成同编号 candidate`,
    detail: { id, path: existing.path, integrationOid: existing.integrationOid || null },
  };
}

/**
 * tracked terminal 的第一阶段：history 本体只是 Git worktree 中的候选；live 节点
 * 只转 PARKED，并带一份不可由 phase2 候选反推的冻结摘要。该 marker 不写 result，
 * 也会随着 live source 的最终移除消失。
 */
function prepareHistoryCandidatePhase(planes, options, changes, integration) {
  const terminals = terminalHistoryChanges(changes);
  if (terminals.length !== 1) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_COUNT_INVALID',
      error: `一次 tracked PLAN 退场必须恰有一份 HISTORY after-image，收到 ${terminals.length} 份`,
    };
  }
  const terminal = terminals[0];
  const existing = canonicalHistoryAlreadyExists(planes, options, terminal.node.id);
  if (existing) return existing;
  const candidate = candidateFromHistoryChange(planes, terminal);
  if (!candidate.ok) return candidate;
  if (fs.existsSync(candidate.target)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_ALREADY_PRESENT',
      error: `history candidate 目标 ${candidate.target} 已存在；不得覆盖或重写未提交候选`,
      detail: { id: terminal.node.id, target: candidate.target },
    };
  }
  if (freezeMarkers(terminal.node).length > 0) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_MARKER_CONFLICT',
      error: `计划 ${terminal.node.id} 已含保留的 history candidate marker，拒绝覆盖其冻结事实`,
    };
  }
  const marker = candidateFreezeMarker(candidate, integration, options.command);
  const parked = parkedNodeWithFreeze(terminal.node, marker);
  const phaseChanges = changes.map((change) => (change === terminal
    ? { ...change, node: parked }
    : change));
  return {
    ok: true,
    phase: 'HISTORY_CANDIDATE',
    changes: phaseChanges,
    historyChanges: [terminal],
    candidate,
    marker,
    candidateExtra: store.historyCandidateExtra(candidate.target, candidate.text),
  };
}

function markerMatchesCandidate(marker, candidate, options) {
  return marker
    && marker.command === options.command
    && marker['candidate-path'] === candidate.repoPath
    && marker['candidate-digest'] === candidate.digest
    && marker['integration-ref'] === options.integrationRef
    && isCanonicalOid(marker['phase1-integration-oid']);
}

function pendingHistoryIntent(state, planes, options) {
  if (!planes.tracked || !isTerminalCommand(options.command) || !options.id) {
    return { ok: true, phase: null };
  }
  const record = state && state.index && state.index.byId ? state.index.byId.get(options.id) : null;
  if (!record || !record.node || record.node.lifecycle !== 'PARKED') {
    return { ok: true, phase: null };
  }
  const markers = freezeMarkers(record.node);
  if (markers.length === 0) return { ok: true, phase: null };
  if (markers.length !== 1) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_MARKER_AMBIGUOUS',
      error: `PARKED 计划 ${options.id} 同时有 ${markers.length} 份 history candidate marker，不能猜哪份可 finalize`,
    };
  }
  const marker = markers[0];
  if (marker.command !== options.command) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_COMMAND_MISMATCH',
      error: `PARKED 计划 ${options.id} 正等待 ${marker.command} candidate commit；不能用 ${options.command} 改写或 finalize`,
      detail: { expected: marker.command, actual: options.command },
    };
  }
  const historical = historyRecord(planes, options.id, options);
  if (!historical) {
    const candidatePath = typeof marker['candidate-path'] === 'string'
      ? path.join(planes.repoRoot, marker['candidate-path'])
      : null;
    return {
      ok: true,
      phase: 'AWAITING_HISTORY_COMMIT',
      marker,
      candidatePath,
      error: `HISTORY candidate 尚未从固定 integration ref ${options.integrationRef} 读回；请只提交 ${marker['candidate-path'] || '(候选路径缺失)'}`,
    };
  }
  if (!historical.node || historical.node.lifecycle !== 'HISTORY' || historical.issues.length > 0
    || historical.node.id !== record.node.id || historical.node.kind !== record.node.kind) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_INVALID',
      error: `固定 integration ref 中的 history candidate ${options.id} 不是同编号、同 kind 的有效 HISTORY 节点`,
      detail: { path: historical.path, issues: historical.issues },
    };
  }
  const candidate = candidateFromHistoryChange(planes, {
    node: historical.node,
    body: historical.body,
    previousPath: record.path,
  });
  if (!candidate.ok) return candidate;
  if (historical.path !== candidate.repoPath) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_PATH_MISMATCH',
      error: '固定 integration ref 中的 candidate 路径与其 closed-at 推导的 canonical 路径不一致',
      detail: { expected: candidate.repoPath, actual: historical.path },
    };
  }
  if (!markerMatchesCandidate(marker, candidate, options)) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_FREEZE_MISMATCH',
      error: '固定 ref 的 history candidate 与 phase1 在 PARKED source 中冻结的路径/摘要/ref 不一致，拒绝自洽重建',
      detail: { marker, candidate: { path: candidate.repoPath, digest: candidate.digest } },
    };
  }
  const local = (() => {
    try { return fs.readFileSync(candidate.target, 'utf8'); } catch (error) { return null; }
  })();
  if (local === null || local !== candidate.text) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_LOCAL_MISMATCH',
      error: 'integration worktree 中的 history candidate 与固定 ref after-image 不完全一致',
      detail: { target: candidate.target, expectedDigest: candidate.digest, actualDigest: local === null ? null : store.digestOf(local) },
    };
  }
  const projectedParked = parkedNodeWithFreeze(historical.node, marker);
  const expectedSource = nodeSchema.serializeNode(projectedParked, historical.body);
  let actualSource = null;
  try { actualSource = fs.readFileSync(record.path, 'utf8'); } catch (error) { actualSource = null; }
  if (actualSource === null || actualSource !== expectedSource) {
    return {
      ok: false,
      code: 'PLAN_HISTORY_CANDIDATE_SOURCE_MISMATCH',
      error: 'PARKED live source 不再是 phase1 生成的 candidate 投影；不能删除',
      detail: { source: record.path, expectedDigest: store.digestOf(expectedSource), actualDigest: actualSource === null ? null : store.digestOf(actualSource) },
    };
  }
  return {
    ok: true,
    phase: 'HISTORY_FINALIZE',
    marker,
    candidate,
    historyChanges: [{ node: historical.node, body: historical.body, previousPath: record.path }],
  };
}

function afterImage(writePlan) {
  return writePlan.entries.map((entry) => ({
    id: entry.id,
    lifecycle: entry.lifecycle,
    area: entry.area,
    from: entry.previousPath,
    path: entry.target,
    digest: entry.digest,
    text: entry.text,
  }));
}

/**
 * receipt 绑定一次 inspect 所看到的 generation、每份 source/target 摘要和完整
 * after-image。write 会重新规划并比对；任一 live source、目标或图谱 generation
 * 漂移都会变成不同摘要，不能悄悄拿新事实写历史。
 */
function historyReceipt(writePlan, integration, carrier, phase, freeze) {
  const material = {
    version: 3,
    phase: phase || null,
    generation: writePlan.generation,
    integration: {
      ref: integration && integration.ref ? integration.ref : null,
      oid: integration && integration.integrationOid ? integration.integrationOid : null,
      branch: integration && integration.expectedBranch ? integration.expectedBranch : null,
      checkoutRoot: integration && integration.checkoutRoot ? integration.checkoutRoot : null,
    },
    carrier: carrier ? publicCarrier(carrier) : null,
    entries: writePlan.entries.map((entry) => ({
      id: entry.id,
      lifecycle: entry.lifecycle,
      area: entry.area,
      from: entry.previousPath,
      target: entry.target,
      expectedSourceDigest: entry.expectedSourceDigest,
      expectedTargetDigest: entry.expectedTargetDigest,
      afterImageDigest: entry.digest,
    })),
    extraFiles: (writePlan.extraFiles || []).map((extra) => ({
      target: extra.target,
      expectedTargetDigest: extra.expectedTargetDigest,
      mustBeAbsent: extra.mustBeAbsent === true,
      digest: extra.text === undefined ? null : store.digestOf(extra.text),
      role: extra.role || null,
    })),
    freeze: freeze ? {
      marker: freeze.marker || null,
      candidate: freeze.candidate ? {
        path: freeze.candidate.repoPath,
        digest: freeze.candidate.digest,
      } : null,
    } : null,
  };
  return crypto.createHash('sha256').update(JSON.stringify(material)).digest('hex');
}

function terminalWriteFailure(output, code, error, detail) {
  return {
    ...output,
    ok: false,
    code,
    error,
    detail: detail || null,
    written: false,
  };
}

function receiptFailure(output, options, receipt) {
  if (!options.receipt) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_RECEIPT_REQUIRED',
      'tracked HISTORY 写入必须先用 --inspect 取得 receipt，并在单独的 --write 中带回同一摘要',
    );
  }
  if (!/^[0-9a-f]{64}$/i.test(options.receipt)) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_RECEIPT_INVALID',
      '--receipt 必须是 inspect 输出的 sha256',
    );
  }
  if (options.receipt.toLowerCase() !== receipt) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_RECEIPT_STALE',
      'receipt 与当前 generation / source digest / after-image 不一致；请重新 --inspect',
      { expected: receipt, actual: options.receipt },
    );
  }
  return null;
}

function integrationWriteFailure(output, integration) {
  if (!integration.headMatches) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_INTEGRATION_HEAD_MISMATCH',
      `当前 HEAD ${integration.headOid} 不等于 integration ref ${integration.ref} 的 ${integration.integrationOid}`,
      integration,
    );
  }
  if (!integration.branchMatches) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_INTEGRATION_BRANCH_MISMATCH',
      `当前分支 ${integration.currentBranch} 不是 integration 分支 ${integration.expectedBranch}`,
      integration,
    );
  }
  if (!integration.clean) {
    return terminalWriteFailure(
      output,
      'PLAN_HISTORY_INTEGRATION_WORKTREE_DIRTY',
      'integration worktree 不干净，不能在未知改动上写 canonical history',
      integration,
    );
  }
  return null;
}

function run(argv) {
  const parsed = parseArguments(argv);
  if (!parsed.ok) return { ok: false, code: 'ARGUMENT_ERROR', error: parsed.error, written: false };
  const options = parsed.options;
  try {
    const rootOverride = canonicalRootOverrideFailure(options);
    if (rootOverride) return publicFailure(rootOverride, null);
    const planes = loadPlanes(options);
    const state = snapshotPrecondition(planes);
    if (!state.ok) return publicFailure(state, state.generation);
    const terminalCommand = planes.tracked && isTerminalCommand(options.command);
    const requiredRef = terminalCommand ? requireCanonicalHistoryRef(planes, options, options.command) : null;
    if (requiredRef) return publicFailure(requiredRef, state.snapshot.generation);

    const pending = terminalCommand ? pendingHistoryIntent(state, planes, options) : { ok: true, phase: null };
    if (!pending.ok) return publicFailure(pending, state.snapshot.generation);
    if (terminalCommand) {
      const serialization = concurrentHistoryIntegration(state, options, options.id);
      if (!serialization.ok) {
        return publicFailure({
          ok: false,
          code: serialization.refusal.code,
          error: serialization.refusal.message,
          detail: { running: serialization.running },
        }, state.snapshot.generation);
      }
    }
    if (pending.phase === 'AWAITING_HISTORY_COMMIT') {
      return {
        ok: true,
        code: 'AWAITING_HISTORY_COMMIT',
        phase: pending.phase,
        id: options.id,
        generation: state.snapshot.generation,
        integrationRef: options.integrationRef,
        historyCandidate: {
          path: pending.candidatePath,
          repoPath: pending.marker && pending.marker['candidate-path'] ? pending.marker['candidate-path'] : null,
          digest: pending.marker && pending.marker['candidate-digest'] ? pending.marker['candidate-digest'] : null,
        },
        error: pending.error,
        written: false,
        preview: true,
      };
    }

    let phase = null;
    let summary;
    let phaseChanges;
    let authorityChanges;
    let freeze = null;
    let candidate = null;
    let integration = null;
    let carrier = null;
    let writePlan;

    if (pending.phase === 'HISTORY_FINALIZE') {
      phase = pending.phase;
      phaseChanges = pending.historyChanges;
      authorityChanges = pending.historyChanges;
      freeze = { marker: pending.marker, candidate: pending.candidate };
      candidate = pending.candidate;
      summary = {
        ok: true,
        action: options.command,
        id: options.id,
        from: 'PARKED',
        to: 'HISTORY',
        result: pending.historyChanges[0].node.result,
      };
    } else {
      const decision = chooseDecision(
        options,
        planes,
        state,
        new Date().toISOString().slice(0, 10),
      );
      if (!decision.ok) return publicFailure(decision, state.snapshot.generation);
      const { changes, ...decisionSummary } = decision;
      summary = decisionSummary;
      const terminal = terminalHistoryChanges(changes).length > 0;
      if (terminal && planes.tracked) {
        authorityChanges = terminalHistoryChanges(changes);
        phase = 'HISTORY_CANDIDATE';
        integration = inspectIntegrationWorktree(planes, options);
        if (!integration.ok) return publicFailure(integration, state.snapshot.generation);
        carrier = resolveCarrierAuthority(state, planes, options, authorityChanges);
        if (!carrier.ok) return publicFailure(carrier, state.snapshot.generation);
        const carrierReality = carrierWorktreeReality(carrier, integration, options.cwd);
        if (!carrierReality.ok) return publicFailure(carrierReality, state.snapshot.generation);
        const candidatePhase = prepareHistoryCandidatePhase(planes, options, changes, integration);
        if (!candidatePhase.ok) return publicFailure(candidatePhase, state.snapshot.generation);
        phaseChanges = candidatePhase.changes;
        authorityChanges = candidatePhase.historyChanges;
        candidate = candidatePhase.candidate;
        freeze = { marker: candidatePhase.marker, candidate };
      } else {
        phaseChanges = changes;
        authorityChanges = terminalHistoryChanges(changes);
      }
    }

    const trackedTerminal = Boolean(phase && planes.tracked);
    if (trackedTerminal && !integration) {
      integration = inspectIntegrationWorktree(planes, options);
      if (!integration.ok) return publicFailure(integration, state.snapshot.generation);
    }
    if (trackedTerminal && !carrier) {
      carrier = resolveCarrierAuthority(state, planes, options, authorityChanges);
      if (!carrier.ok) return publicFailure(carrier, state.snapshot.generation);
      const carrierReality = carrierWorktreeReality(carrier, integration, options.cwd);
      if (!carrierReality.ok) return publicFailure(carrierReality, state.snapshot.generation);
    }

    if (phase === 'HISTORY_FINALIZE') {
      writePlan = store.planHistoryFinalization(planes, phaseChanges, {
        expectedGeneration: state.snapshot.generation,
        integration: {
          ref: integration.ref,
          oid: integration.integrationOid,
          cwd: integration.checkoutRoot,
        },
      });
    } else {
      writePlan = store.planNodeWrite(planes, phaseChanges, {
        expectedGeneration: state.snapshot.generation,
        extraFiles: phase === 'HISTORY_CANDIDATE'
          ? [store.historyCandidateExtra(candidate.target, candidate.text)]
          : [],
      });
    }
    const receipt = trackedTerminal ? historyReceipt(writePlan, integration, carrier, phase, freeze) : null;
    const output = {
      ...summary,
      ...(phase === 'HISTORY_CANDIDATE' ? { code: 'AWAITING_HISTORY_COMMIT' } : {}),
      ...(phase ? { phase } : {}),
      generation: writePlan.generation,
      moves: writePlan.entries.map((entry) => ({
        id: entry.id,
        from: entry.previousPath,
        to: entry.target,
        lifecycle: entry.lifecycle,
      })),
      afterImage: afterImage(writePlan),
      ...(candidate ? {
        historyCandidate: {
          path: candidate.target,
          repoPath: candidate.repoPath,
          digest: candidate.digest,
          commitPath: candidate.repoPath,
        },
      } : {}),
      ...(integration ? { integration } : {}),
      ...(carrier ? { carrier: publicCarrier(carrier) } : {}),
      ...(options.inspect && receipt ? { receipt } : {}),
      written: false,
      preview: true,
    };
    // 第二阶段的 inspect 本身就是确认 gate；候选尚未被固定、或 worktree 仍脏时
    // 不能签出一个可供 --write 使用的 receipt。
    if (phase === 'HISTORY_FINALIZE') {
      const initialIntegrationProblem = integrationWriteFailure(output, integration);
      if (initialIntegrationProblem) return initialIntegrationProblem;
    }
    if (!options.write || options.inspect) return output;
    if (trackedTerminal) {
      // receipt 比对之后、真正进入 store 事务之前再读取一次 Git / live 现场。两个
      // phase 都重建 write plan，使 generation、source、candidate OID 或 marker 的
      // 任一漂移都把旧 receipt 变成 stale。
      const finalIntegration = inspectIntegrationWorktree(planes, options);
      if (!finalIntegration.ok) {
        return terminalWriteFailure(
          output,
          finalIntegration.code || 'PLAN_HISTORY_INTEGRATION_CHECK_FAILED',
          finalIntegration.error || '无法复核 integration worktree 现场',
          finalIntegration.detail,
        );
      }
      const finalState = snapshotPrecondition(planes);
      if (!finalState.ok) {
        return terminalWriteFailure(
          output,
          finalState.code || 'PLAN_HISTORY_CARRIER_SNAPSHOT_FAILED',
          finalState.error || '无法复核 carrier 所在 live graph',
          { schema: finalState.schema || [], integrity: finalState.integrity || [] },
        );
      }

      let finalPlan;
      let finalAuthorityChanges;
      let finalFreeze;
      let finalCandidate;
      if (phase === 'HISTORY_CANDIDATE') {
        const finalCandidatePhase = prepareHistoryCandidatePhase(
          planes,
          options,
          // phaseChanges 已经是 PARKED after-image；原始 terminal after-image 保留在
          // authorityChanges，配合非 terminal changes 才能得到同一份 candidate 事务。
          phaseChanges.map((change) => {
            const original = authorityChanges.find((entry) => entry.previousPath === change.previousPath);
            return original || change;
          }),
          finalIntegration,
        );
        if (!finalCandidatePhase.ok) {
          return terminalWriteFailure(
            output,
            finalCandidatePhase.code || 'PLAN_HISTORY_CANDIDATE_CHECK_FAILED',
            finalCandidatePhase.error || '无法复核 history candidate phase',
            finalCandidatePhase.detail,
          );
        }
        finalPlan = store.planNodeWrite(planes, finalCandidatePhase.changes, {
          expectedGeneration: finalState.snapshot.generation,
          extraFiles: [store.historyCandidateExtra(
            finalCandidatePhase.candidate.target,
            finalCandidatePhase.candidate.text,
          )],
        });
        finalAuthorityChanges = finalCandidatePhase.historyChanges;
        finalCandidate = finalCandidatePhase.candidate;
        finalFreeze = { marker: finalCandidatePhase.marker, candidate: finalCandidate };
      } else {
        const finalPending = pendingHistoryIntent(finalState, planes, options);
        if (!finalPending.ok || finalPending.phase !== 'HISTORY_FINALIZE') {
          return terminalWriteFailure(
            output,
            (finalPending && finalPending.code) || 'PLAN_HISTORY_FINALIZE_STALE',
            (finalPending && finalPending.error) || 'history candidate 在 finalize 前不再满足固定 ref / PARKED source 条件',
            finalPending && finalPending.detail,
          );
        }
        finalPlan = store.planHistoryFinalization(planes, finalPending.historyChanges, {
          expectedGeneration: finalState.snapshot.generation,
          integration: {
            ref: finalIntegration.ref,
            oid: finalIntegration.integrationOid,
            cwd: finalIntegration.checkoutRoot,
          },
        });
        finalAuthorityChanges = finalPending.historyChanges;
        finalCandidate = finalPending.candidate;
        finalFreeze = { marker: finalPending.marker, candidate: finalCandidate };
      }
      const finalCarrier = resolveCarrierAuthority(finalState, planes, options, finalAuthorityChanges);
      if (!finalCarrier.ok) {
        return terminalWriteFailure(
          output,
          finalCarrier.code || 'PLAN_HISTORY_CARRIER_CHECK_FAILED',
          finalCarrier.error || '无法复核 carrier authority',
          finalCarrier.detail,
        );
      }
      const finalCarrierReality = carrierWorktreeReality(finalCarrier, finalIntegration, options.cwd);
      if (!finalCarrierReality.ok) {
        return terminalWriteFailure(
          output,
          finalCarrierReality.code || 'PLAN_HISTORY_CARRIER_WORKTREE_CHECK_FAILED',
          finalCarrierReality.error || 'carrier/integration worktree 不满足 canonical history 条件',
          finalCarrierReality.detail,
        );
      }
      const finalReceipt = historyReceipt(finalPlan, finalIntegration, finalCarrier, phase, finalFreeze);
      const receiptProblem = receiptFailure(output, options, finalReceipt);
      if (receiptProblem) return receiptProblem;
      const integrationProblem = integrationWriteFailure(output, finalIntegration);
      if (integrationProblem) return integrationProblem;
      const committed = phase === 'HISTORY_FINALIZE'
        ? store.commitHistoryFinalization(planes, finalPlan)
        : store.commitNodeWrite(planes, finalPlan);
      output.written = true;
      output.preview = false;
      output.generation = committed.generation;
      output.writtenPaths = committed.written;
      if (phase === 'HISTORY_FINALIZE') output.removedPaths = committed.removed;
      return output;
    }
    const committed = store.commitNodeWrite(planes, writePlan);
    output.written = true;
    output.preview = false;
    output.generation = committed.generation;
    output.writtenPaths = committed.written;
    return output;
  } catch (error) {
    return {
      ok: false,
      code: error && error.code ? error.code : 'PLAN_OPERATION_FAILED',
      error: error && error.message ? error.message : String(error),
      detail: error && error.detail ? error.detail : null,
      written: false,
    };
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
