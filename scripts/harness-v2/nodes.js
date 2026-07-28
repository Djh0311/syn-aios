#!/usr/bin/env node
'use strict';

// Adaptive Harness v0.5 — 节点 CLI 薄壳（AH-050-02 / AH-050-03）
//
// 需求溯源：EX-3 · EX-5 · EX-6 · EX-7 · EX-9 · LY-1 · LY-7 · KP-13
//
// 这是唯一对外入口。判定全部在 lib 里，本文件只负责取参数、取现实、印结果。
//
// KP-13：会真正改动文件的动作**默认只给预览（dry-run）**，必须显式 --write 才落盘。
// 落盘一律走 store 的原子写入（先写临时文件再 rename），绝不就地截断。
//
// 提供的动作：
//   inspect  只读：重建索引，报 cycle / orphan / duplicate id / 非法 parent kind /
//            祖先已进入历史，以及阶段边界的跨节点重复声明。
//   current  生成本工作副本的 CURRENT 视图（根计划 → 父阶段 → 当前叶子）。
//   history  按编号从历史平面单点取回一份已结束节点的正文。
//   park     把一个被卡住的叶子挪进 parked：真的换目录，不是只改一个状态字段。
//   resume   按同一个编号把 parked 的叶子拉回 current 区。
//   close    退场：现场向 Git 取事实，跑统一收尾门；十四项齐备才放行。

const fs = require('node:fs');
const path = require('node:path');

const nodeSchema = require('./lib/node-schema');
const lifecycle = require('./lib/lifecycle');
const gitFacts = require('./lib/git-facts');
const store = require('./lib/store');
const graph = require('./lib/graph');
const boundary = require('./lib/boundary');
const context = require('./lib/context');
const exitGate = require('./lib/exit-gate');
const assets = require('./lib/assets');
const authority = require('./lib/authority');
const planCli = require('./plan');
const evidenceTrace = require('./lib/evidence-trace');
const legacyDocs = require('./lib/legacy-docs');

const COMMANDS = ['inspect', 'current', 'history', 'park', 'resume', 'close'];

function parseArguments(argv) {
  const options = {
    command: null,
    id: null,
    cwd: process.cwd(),
    closeout: null,
    liveRoot: null,
    historyRoot: null,
    integrationRef: null,
    questionsInput: null,
    json: false,
    // dry-run 是默认：不给 --write 就只输出预览，不落盘（KP-13）。
    write: false,
  };
  const list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length && !list[0].startsWith('--')) options.command = list.shift();
  while (list.length) {
    const token = list.shift();
    if (token === '--write') { options.write = true; continue; }
    if (token === '--json') { options.json = true; continue; }
    if (token === '--id') { options.id = list.shift() || null; continue; }
    if (token === '--target') { options.cwd = list.shift() || options.cwd; continue; }
    if (token === '--closeout') { options.closeout = list.shift() || null; continue; }
    if (token === '--live-root') { options.liveRoot = list.shift() || null; continue; }
    if (token === '--history-root') { options.historyRoot = list.shift() || null; continue; }
    if (token === '--integration-ref') { options.integrationRef = list.shift() || null; continue; }
    if (token === '--questions-input') {
      options.questionsInput = list.shift() || null;
      if (!options.questionsInput) {
        return { ok: false, error: '--questions-input 需要 JSON 文件路径', options };
      }
      continue;
    }
    return { ok: false, error: `未知参数 ${token}`, options };
  }
  if (!options.command || !COMMANDS.includes(options.command)) {
    return { ok: false, error: `用法：nodes.js <${COMMANDS.join('|')}> [--id <编号>] [--target <目录>] [--write]`, options };
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

function loadGraph(planes) {
  const records = store.readLiveNodes(planes);
  const index = graph.buildGraphIndex(records);
  return { records, index };
}

function readJsonOrNull(filePath) {
  if (!filePath) return null;
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    return null;
  }
}

function commandInspect(options) {
  const planes = loadPlanes(options);
  const { records, index } = loadGraph(planes);
  const integrity = graph.graphIntegrityIssues(index);
  const boundaries = boundary.boundaryUniquenessIssues(index, records);
  const schema = [];
  for (const record of records) {
    for (const problem of record.issues) {
      schema.push({ path: record.path, ...problem });
    }
  }
  return {
    ok: integrity.length === 0 && boundaries.length === 0 && schema.length === 0,
    generation: store.readGeneration(planes),
    lock: store.inspectLock(planes),
    counted: records.length,
    nodes: records.map((record) => ({
      id: record.node ? record.node.id : null,
      kind: record.node ? record.node.kind : null,
      lifecycle: record.node ? record.node.lifecycle : null,
      area: record.area,
      path: record.path,
    })),
    integrity,
    boundaries,
    schema,
  };
}

function activeLeafFor(index, worktreePath) {
  for (const record of index.byId.values()) {
    const node = record.node;
    if (!node || node.kind !== 'TASK' || node.lifecycle !== 'ACTIVE') continue;
    const binding = node.git && typeof node.git === 'object' ? node.git : {};
    if (!worktreePath || binding.worktree === worktreePath) return node.id;
  }
  return null;
}

function commandCurrent(options) {
  const planes = loadPlanes(options);
  const { index } = loadGraph(planes);
  const worktreePath = planes.repoRoot || options.cwd;
  const leafId = activeLeafFor(index, worktreePath);
  let questionProjection = null;
  if (options.questionsInput) {
    const inputPath = path.resolve(options.cwd, options.questionsInput);
    const input = readJsonOrNull(inputPath);
    if (!input || typeof input !== 'object' || Array.isArray(input)) {
      return {
        ok: false,
        code: 'QUESTIONS_INPUT_INVALID',
        error: `读不到合法 Questions JSON：${inputPath}`,
        written: false,
      };
    }
    questionProjection = legacyDocs.projectCurrentBlockers(input.questions, {
      nearestOwners: input.nearestOwners,
      nearestOwner: input.nearestOwner,
    });
    if (!questionProjection.ok) {
      return {
        ...questionProjection,
        written: false,
      };
    }
  }
  const text = context.renderCurrent({
    index,
    activeLeafId: leafId,
    worktreePath,
    blockers: questionProjection ? questionProjection.blockers : [],
  });
  const target = store.currentViewPath(planes, store.worktreeKeyFor(worktreePath));
  const result = {
    ok: true,
    target,
    preview: text,
    questionProjection,
    written: false,
  };
  if (options.write) {
    store.atomicWrite(target, `${text}\n`);
    result.written = true;
  }
  return result;
}

function commandHistory(options) {
  const planes = loadPlanes(options);
  if (!options.id) return { ok: false, error: 'history 需要 --id' };
  if (planes.tracked && !options.integrationRef) {
    return {
      ok: false,
      code: 'INTEGRATION_REF_REQUIRED',
      error: 'tracked history 必须显式提供 --integration-ref；不能默认猜 main',
      written: false,
    };
  }
  const found = store.readHistoryNode(planes, options.id, { cwd: options.cwd });
  if (!found) return { ok: false, error: `历史平面里找不到编号 ${options.id}` };
  const parsed = nodeSchema.parseNode(found.text, { relativePath: found.path });
  const verificationTrace = parsed.node
    ? evidenceTrace.resolveRequiredVerification({
      node: parsed.node,
      body: parsed.body,
      // 旧 history 可以展示 legacy 位置，但 resolver 仍标成 unresolved；它绝不
      // 会被 TASK close 的 item 03 当成已解析的原始输出。
      allowLegacyRefs: true,
      readRepoAtHead({ headOid, path: repoPath }) {
        return gitFacts.showFromRef(options.cwd, headOid, repoPath);
      },
    })
    : {
      allowed: false,
      resolved: [],
      results: [],
      problems: [{ id: options.id, code: 'TRACE_HISTORY_NODE_INVALID', error: '历史节点无法解析' }],
      unresolvedCount: 1,
    };
  return {
    ok: true,
    id: options.id,
    bucket: found.bucket,
    path: found.path,
    text: found.text,
    verificationTrace,
    // 简短别名便于 JSON 消费者按编号读取时直接取追溯结果；不增加新 CLI。
    trace: verificationTrace,
  };
}

function relocate(options, targetLifecycle) {
  const planes = loadPlanes(options);
  const { index } = loadGraph(planes);
  if (!options.id) return { ok: false, error: `${options.command} 需要 --id` };
  const record = index.byId.get(options.id);
  if (!record) return { ok: false, error: `在办平面里找不到编号 ${options.id}` };
  const from = record.node.lifecycle;
  const step = lifecycle.transitionAllowed(record.node.kind, from, targetLifecycle);
  if (!step.allowed) return { ok: false, error: `${from} → ${targetLifecycle} 不在合法流转表内（${step.reason}）` };
  const next = { ...record.node, lifecycle: targetLifecycle };
  const writePlan = store.planNodeWrite(planes, [{ node: next, body: record.body, previousPath: record.path }]);
  const result = {
    ok: true,
    id: options.id,
    from,
    to: targetLifecycle,
    moves: writePlan.entries.map((entry) => ({ from: entry.previousPath, to: entry.target })),
    written: false,
  };
  if (options.write) {
    store.commitNodeWrite(planes, writePlan);
    result.written = true;
  }
  return result;
}

function commandClose(options) {
  const planes = loadPlanes(options);
  const { records, index } = loadGraph(planes);
  if (!options.id) return { ok: false, error: 'close 需要 --id' };
  const record = index.byId.get(options.id);
  if (!record) return { ok: false, error: `在办平面里找不到编号 ${options.id}` };
  const node = record.node;
  const closeout = readJsonOrNull(options.closeout) || {};

  const descendants = graph.nonHistoryDescendants(index, options.id);
  const dependents = graph.dependentsOf(index, options.id);

  if (node.kind !== 'TASK') {
    const gaps = boundary.boundaryUniquenessIssues(index, records)
      .filter((entry) => entry.id === options.id)
      .map((entry) => `${entry.section} 与 ${entry.ancestorId} 重复`);
    const verdictNonLeaf = exitGate.evaluateNonLeafExit({
      nonHistoryDescendants: descendants,
      nonHistoryDependents: dependents,
      boundaryGaps: closeout.boundaryGaps || gaps,
      openQuestions: closeout.openQuestions || [],
    });
    return { ok: verdictNonLeaf.allowed, kind: node.kind, gate: 'non-leaf', ...verdictNonLeaf };
  }

  // 退场必须核对现实：这里现场向版本库取事实，而不是读自己刚写的那份记录。
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const reality = { changedPaths: [], statusNow: [], headOid: null, scopedDiffEmpty: null, commitReachable: false, integratedNow: false };
  try {
    reality.headOid = gitFacts.headOid(options.cwd);
    reality.statusNow = gitFacts.porcelainStatus(options.cwd);
    if (binding['base-oid']) {
      reality.changedPaths = gitFacts.changedPaths(options.cwd, binding['base-oid'], 'HEAD');
      reality.scopedDiffEmpty = gitFacts
        .scopedChangedPaths(options.cwd, binding['base-oid'], 'HEAD', node['write-scope'] || []).length === 0;
    }
    const commit = binding['product-commit'] || binding['wip-commit'] || null;
    reality.commitReachable = commit ? gitFacts.objectExists(options.cwd, commit) : false;
    if (binding['product-commit']) {
      reality.integratedNow = gitFacts.isAncestor(options.cwd, binding['product-commit'], planes.integrationRef);
    }
  } catch (error) {
    return { ok: false, error: `读不到 Git 现实，报冲突并停：${error.message}` };
  }

  const assetAudit = assets.auditAssetDispositions({
    assets: closeout.assets || [],
    changedPaths: reality.changedPaths,
  });

  const nextEntry = context.resolveNextEntry(index, options.id);
  const verdict = exitGate.evaluateLeafExit({
    node: { ...node, result: closeout.result },
    closeout,
    git: {
      changedPaths: reality.changedPaths,
      controlPlaneExemptPrefixes: [planes.historyRepoPath || 'docs/harness/history'],
      headOid: reality.headOid,
      statusNow: reality.statusNow,
      statusAtInspect: closeout.statusAtInspect,
      scopedDiffEmpty: reality.scopedDiffEmpty,
      commitReachable: reality.commitReachable,
      integratedNow: reality.integratedNow,
      wipCarriedIntoIntegration: closeout.wipCarriedIntoIntegration === true,
    },
    assets: assetAudit.problems.length === 0
      ? (closeout.assets || [])
      : assetAudit.problems.map((problem) => ({ path: problem.path, assetClass: '', disposition: '' })),
    nonHistoryDescendants: descendants,
    nonHistoryDependents: dependents,
    current: { nextEntry: nextEntry.label },
    truth: {
      productCommit: binding['product-commit'] || null,
      productCommitTouchesScope: reality.scopedDiffEmpty === false,
      integratedNow: reality.integratedNow,
      requiredVerification: (node.verification || []).filter((entry) => entry && entry.required === true),
      requiredVerificationEmptyIsPass: true,
      noProductChange: binding['no-product-change'] === true,
      scopedDiffEmpty: reality.scopedDiffEmpty === true,
      wipCommit: binding['wip-commit'] || null,
      disposition: binding.disposition || null,
      hasNonHistoryDescendant: descendants.length > 0,
      hasNonHistoryDependent: dependents.length > 0,
      splitSuccessors: graph.reverseRelationsOf(index, options.id).SPLIT_INTO,
      replaceSuccessors: graph.reverseRelationsOf(index, options.id).SUPERSEDED_BY,
    },
  });

  const result = {
    ok: verdict.allowed,
    kind: node.kind,
    gate: 'leaf',
    allowed: verdict.allowed,
    unmet: verdict.unmet,
    assetProblems: assetAudit.problems,
    nextEntry: nextEntry.label,
    written: false,
  };
  if (!verdict.allowed || !options.write) return result;

  // 齐备之后不得再查任何东西：直接落盘。
  const target = closeout.targetLifecycle === 'PARKED' ? 'PARKED' : 'HISTORY';
  const next = { ...node, lifecycle: target };
  if (target === 'HISTORY') {
    next.result = closeout.result;
    next['closed-at'] = closeout.closedAt || new Date().toISOString().slice(0, 10);
  }
  const writePlan = store.planNodeWrite(planes, [{ node: next, body: record.body, previousPath: record.path }]);
  store.commitNodeWrite(planes, writePlan);
  result.written = true;
  return result;
}

function run(argv) {
  const parsed = parseArguments(argv);
  if (!parsed.ok) return { ok: false, error: parsed.error };
  const options = parsed.options;
  // AH-050-04 之后，plan 的变更只保留一个实现。这里先做一次只读路由；
  // plan.js 会重新读取绑定 generation 的一致快照，第一次读取绝不作为写依据。
  if (options.id && ['park', 'resume', 'close'].includes(options.command)) {
    const planes = loadPlanes(options);
    const { records } = loadGraph(planes);
    // duplicate id 本来就是坏图。只看 index.byId 的第一条会受目录遍历顺序影响：
    // TASK 恰好先出现时，旧入口可能绕过 plan 的完整性预检，甚至覆盖同编号计划。
    // 只要同编号里出现过 plan，就交给 plan 入口 fail closed。
    const includesPlan = records.some((record) => record
      && record.node
      && record.node.id === options.id
      && record.node.kind !== 'TASK');
    if (includesPlan) {
      return planCli.run(argv);
    }
    const includesTask = records.some((record) => record
      && record.node
      && record.node.id === options.id
      && record.node.kind === 'TASK');
    if (includesTask) {
      return {
        ok: false,
        code: 'TASK_ROUTED_TO_AH_050_05',
        error: 'TASK 的 park/resume/close 必须走 scripts/harness-v2/task.js；generic nodes 入口不再提供可写旁路',
        written: false,
      };
    }
  }
  if (options.command === 'inspect') return commandInspect(options);
  if (options.command === 'current') return commandCurrent(options);
  if (options.command === 'history') return commandHistory(options);
  if (options.command === 'park') return relocate(options, 'PARKED');
  if (options.command === 'resume') return relocate(options, 'ACTIVE');
  return commandClose(options);
}

if (require.main === module) {
  let outcome;
  try {
    outcome = run(process.argv.slice(2));
  } catch (error) {
    outcome = { ok: false, error: error.message };
  }
  process.stdout.write(`${JSON.stringify(outcome, null, 2)}\n`);
  process.exitCode = outcome && outcome.ok ? 0 : 1;
}

module.exports = {
  COMMANDS,
  parseArguments,
  run,
  nodeSchema,
  authority,
  path,
};
