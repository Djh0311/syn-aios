#!/usr/bin/env node
'use strict';

// Adaptive Harness v0.5 — Git 判定与计划的 CLI 薄壳（AH-050-07）
//
// 需求溯源：GIT-1..GIT-11 · §6.1~§6.5 · KP-13
//
// 这个入口**不执行任何 Git 写操作**：它只读现实、只出判定与计划。
// 真正的 branch / commit / merge 由调用方在取得用户单独授权之后执行。
// 会落盘的只有判定记录，且默认只给预览，必须显式 --write 才写（KP-13）。
//
// 对外提供的动作（每一条都是某类禁令配套的那条正常出口，见 GIT-RUNBOOK.md）：
//
//   node scripts/harness-v2/git-plan.js inspect   --task <ID>
//   node scripts/harness-v2/git-plan.js admit     --request <FILE>
//   node scripts/harness-v2/git-plan.js commit    --task <ID> --paths <A,B> --message-file <FILE>
//   node scripts/harness-v2/git-plan.js wip       --task <ID>
//   node scripts/harness-v2/git-plan.js replay    --task <ID>
//   node scripts/harness-v2/git-plan.js integrate --task <ID> --evidence <FILE>
//   node scripts/harness-v2/git-plan.js retire    --task <ID> --closeout <FILE>
//   node scripts/harness-v2/git-plan.js guard     --action push --task <ID>

const fs = require('node:fs');

const store = require('./lib/store');
const graph = require('./lib/graph');
const lifecycle = require('./lib/lifecycle');
const gitFacts = require('./lib/git-facts');
const scope = require('./lib/scope');
const opening = require('./lib/opening');
const replay = require('./lib/replay');
const commitPlan = require('./lib/commit-plan');
const integrationGate = require('./lib/integration-gate');
const safeguards = require('./lib/safeguards');

const COMMANDS = ['inspect', 'admit', 'commit', 'wip', 'replay', 'integrate', 'retire', 'guard'];

const USAGE = [
  '用法：',
  '  node scripts/harness-v2/git-plan.js inspect --task <ID>',
  '  node scripts/harness-v2/git-plan.js admit --request <FILE> [--reason START|RESUME|ADOPT|TRANSFER_IN]',
  '  node scripts/harness-v2/git-plan.js commit --task <ID> --paths <A,B> --message-file <FILE>',
  '  node scripts/harness-v2/git-plan.js wip --task <ID>',
  '  node scripts/harness-v2/git-plan.js replay --task <ID>',
  '  node scripts/harness-v2/git-plan.js integrate --task <ID> --evidence <FILE>',
  '  node scripts/harness-v2/git-plan.js retire --task <ID> --closeout <FILE>',
  '  node scripts/harness-v2/git-plan.js guard --action push --task <ID>',
].join('\n');

function parseArguments(argv) {
  const options = {
    command: null,
    id: null,
    cwd: process.cwd(),
    request: null,
    evidence: null,
    closeout: null,
    messageFile: null,
    paths: [],
    action: null,
    // 本次为什么进入 ACTIVE：START / RESUME / ADOPT / TRANSFER_IN。
    // 四种一律走同一条 admit，判的都是**当前**在册声明；
    // 「恢复 / 承接 / 接管沿用上次结论」在这里根本表达不出来（§6.4）。
    reason: null,
    route: null,
    confirmed: false,
    liveRoot: null,
    historyRoot: null,
    // dry-run 是默认：不给 --write 就只输出预览，不落盘（KP-13）。
    write: false,
  };
  const list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length && !list[0].startsWith('--')) options.command = list.shift();
  while (list.length) {
    const token = list.shift();
    if (token === '--write') { options.write = true; continue; }
    if (token === '--confirmed') { options.confirmed = true; continue; }
    if (token === '--task' || token === '--id') { options.id = list.shift() || null; continue; }
    if (token === '--target') { options.cwd = list.shift() || options.cwd; continue; }
    if (token === '--request') { options.request = list.shift() || null; continue; }
    if (token === '--evidence') { options.evidence = list.shift() || null; continue; }
    if (token === '--closeout') { options.closeout = list.shift() || null; continue; }
    if (token === '--message-file') { options.messageFile = list.shift() || null; continue; }
    if (token === '--paths') {
      options.paths = String(list.shift() || '').split(',').map((item) => item.trim()).filter(Boolean);
      continue;
    }
    if (token === '--action') { options.action = list.shift() || null; continue; }
    if (token === '--reason') { options.reason = list.shift() || null; continue; }
    if (token === '--route') { options.route = list.shift() || null; continue; }
    if (token === '--live-root') { options.liveRoot = list.shift() || null; continue; }
    if (token === '--history-root') { options.historyRoot = list.shift() || null; continue; }
    return { ok: false, error: `未知参数 ${token}`, options };
  }
  if (!options.command || !COMMANDS.includes(options.command)) {
    return { ok: false, error: USAGE, options };
  }
  return { ok: true, error: null, options };
}

function loadContext(options) {
  const planes = store.resolvePlanes({
    cwd: options.cwd,
    liveRoot: options.liveRoot,
    historyRoot: options.historyRoot,
  });
  const records = store.readLiveNodes(planes);
  const index = graph.buildGraphIndex(records);
  return { planes, records, index };
}

function readJsonOrNull(filePath) {
  if (!filePath) return null;
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    return null;
  }
}

function readTextOrNull(filePath) {
  if (!filePath) return null;
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch (error) {
    return null;
  }
}

function taskOf(context, id) {
  const record = context.index.byId.get(id);
  if (!record || !record.node || record.node.kind !== 'TASK') return null;
  return record.node;
}

function bindingOf(node) {
  return node && node.git && typeof node.git === 'object' ? node.git : {};
}

function declarationsExcept(context, id) {
  return opening
    .declarationsFromNodes(context.records, lifecycle.participatesInScopeJudgement)
    .filter((entry) => entry.id !== id);
}

// ---------------------------------------------------------------------------
// 动作
// ---------------------------------------------------------------------------

function commandInspect(options, context) {
  if (!options.id) return { ok: false, error: 'inspect 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  const baseline = safeguards.registerDirtyBaseline({ cwd: options.cwd, root: context.planes.repoRoot || options.cwd });
  const drift = replay.assessBaseline({
    cwd: options.cwd,
    taskBranch: binding['task-branch'],
    baseOid: binding['base-oid'],
    integrationRef: context.planes.integrationRef,
  });
  return {
    ok: true,
    task: options.id,
    binding,
    headOid: gitFacts.headOid(options.cwd),
    dirtyBaseline: baseline,
    baseline: drift,
    // 基线落后只提示、不做硬门：它不拒绝开工、不拒绝提交、不拒绝进入等待集成。
    advisory: replay.baselineDriftAdvisory(drift),
    declarationsInPlay: declarationsExcept(context, options.id).filter((entry) => entry.participates),
  };
}

function commandAdmit(options, context) {
  const text = readTextOrNull(options.request);
  if (text === null) return { ok: false, error: 'admit 需要 --request 指向一份开工请求' };
  const parsed = opening.parseOpeningRequest(text);
  if (!parsed.ok) return { ok: false, error: '开工请求读不出来', issues: parsed.issues };
  if (options.reason && !opening.ADMISSION_REASONS.includes(options.reason)) {
    return { ok: false, error: `--reason 只接受 ${opening.ADMISSION_REASONS.join(' / ')}` };
  }
  // 四种进入方式走的是同一条判定，吃的是同一份**当前**在册声明。
  // 恢复、承接转交、接管半成品在这里都重判一次，不存在「沿用首判结论」的分支。
  const verdict = opening.evaluateOpening({
    cwd: options.cwd,
    fields: parsed.fields,
    reason: options.reason || 'START',
    registered: declarationsExcept(context, parsed.fields['task-id'] || null),
  });
  return {
    ok: verdict.ok,
    task: parsed.fields['task-id'] || null,
    reason: verdict.reason,
    admitted: verdict.ok,
    fieldIssues: verdict.fieldIssues,
    conflicts: verdict.conflicts,
    refusals: verdict.admission.refusals,
    refusalText: scope.describeRefusals(verdict.admission.refusals),
    binding: verdict.binding,
  };
}

function commandCommit(options, context) {
  if (!options.id) return { ok: false, error: 'commit 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  if (binding['local-commit-allowed'] !== true) {
    return { ok: false, error: `任务 ${options.id} 的 local-commit-allowed 不是 true，拒绝提交` };
  }
  const message = readTextOrNull(options.messageFile);
  const verdict = commitPlan.planExactCommit({
    cwd: options.cwd,
    paths: options.paths,
    message: message === null ? '' : message,
    declaration: {
      id: node.id,
      'write-scope': node['write-scope'],
      'forbidden-scope': node['forbidden-scope'],
      'exclusive-resources': node['exclusive-resources'],
    },
    registered: declarationsExcept(context, options.id),
  });
  return { ok: verdict.allowed, task: options.id, ...verdict };
}

function commandWip(options, context) {
  if (!options.id) return { ok: false, error: 'wip 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  const wip = binding['wip-commit'];
  const verdict = integrationGate.verifyWipCommits({
    cwd: options.cwd,
    taskBranch: binding['task-branch'],
    wipCommits: wip ? [wip] : [],
    hasUncommittedWork: gitFacts.porcelainStatus(options.cwd).length > 0,
  });
  return { ok: verdict.ok, task: options.id, ...verdict };
}

function commandReplay(options, context) {
  if (!options.id) return { ok: false, error: 'replay 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  const facts = replay.assessBaseline({
    cwd: options.cwd,
    taskBranch: binding['task-branch'],
    baseOid: binding['base-oid'],
    integrationRef: context.planes.integrationRef,
  });
  const plan = replay.planConfirmedReplay({
    facts,
    taskId: options.id,
    route: options.route,
    confirmed: options.confirmed,
  });
  // 确认之后不得返回「无可用出口」：plan.hasExit 恒为 true，出口清单也永不为空。
  return { ok: true, task: options.id, facts, plan, executedHere: false };
}

function commandIntegrate(options, context) {
  if (!options.id) return { ok: false, error: 'integrate 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  const evidence = readJsonOrNull(options.evidence) || {};
  const authorization = safeguards.requireFreshAuthorization({
    action: 'local-integrate',
    target: binding['task-branch'],
    confirmations: node.confirmations || [],
  });
  const readiness = integrationGate.evaluateIntegrationReadiness({
    cwd: options.cwd,
    baseOid: binding['base-oid'],
    headRef: binding['task-branch'],
    headOid: gitFacts.headOid(options.cwd),
    taskWorktree: binding.worktree,
    cleanCheckout: evidence.cleanCheckout,
    offMachine: evidence.offMachine,
    integrationRef: context.planes.integrationRef,
    integrationsInFlight: evidence.integrationsInFlight || [],
  });
  const facts = replay.assessBaseline({
    cwd: options.cwd,
    taskBranch: binding['task-branch'],
    baseOid: binding['base-oid'],
    integrationRef: context.planes.integrationRef,
  });
  return {
    ok: readiness.allowed && authorization.authorized && facts.fastForwardPossible,
    task: options.id,
    authorization,
    readiness,
    fastForwardPossible: facts.fastForwardPossible,
    // 快进不成立不是终点：这里给出继续路径，不是停在「合不上」。
    continuation: facts.fastForwardPossible ? null : replay.planConfirmedReplay({ facts, taskId: options.id }),
    executedHere: false,
  };
}

function commandRetire(options, context) {
  if (!options.id) return { ok: false, error: 'retire 需要 --task' };
  const node = taskOf(context, options.id);
  if (!node) return { ok: false, error: `在办平面里找不到任务 ${options.id}` };
  const binding = bindingOf(node);
  const closeout = readJsonOrNull(options.closeout) || {};
  const residue = integrationGate.auditResidue({ cwd: options.cwd, dirtyBaseline: closeout.dirtyBaseline || [] });
  const ignores = integrationGate.auditIgnoreRules({
    cwd: options.cwd,
    baseOid: binding['base-oid'],
    headRef: binding['task-branch'] || 'HEAD',
    reasons: closeout.ignoreReasons || [],
  });
  const untracked = integrationGate.auditUntrackedProductDependencies({
    cwd: options.cwd,
    writeScope: node['write-scope'] || [],
  });
  const wip = integrationGate.verifyWipCommits({
    cwd: options.cwd,
    taskBranch: binding['task-branch'],
    wipCommits: binding['wip-commit'] ? [binding['wip-commit']] : [],
  });
  const problems = [...residue.problems, ...ignores.problems, ...untracked.problems, ...wip.problems];
  return { ok: problems.length === 0, task: options.id, residue, ignores, untracked, wip, problems };
}

function commandGuard(options, context) {
  const node = options.id ? taskOf(context, options.id) : null;
  const verdict = safeguards.requireFreshAuthorization({
    action: options.action,
    target: options.id,
    confirmations: node && Array.isArray(node.confirmations) ? node.confirmations : [],
  });
  return {
    ok: verdict.authorized,
    action: options.action,
    task: options.id,
    refusal: verdict.refusal,
    exits: safeguards.exitsForProhibitions(),
  };
}

function run(argv) {
  const parsed = parseArguments(argv);
  if (!parsed.ok) return { ok: false, error: parsed.error };
  const options = parsed.options;
  const context = loadContext(options);
  if (options.command === 'inspect') return commandInspect(options, context);
  if (options.command === 'admit') return commandAdmit(options, context);
  if (options.command === 'commit') return commandCommit(options, context);
  if (options.command === 'wip') return commandWip(options, context);
  if (options.command === 'replay') return commandReplay(options, context);
  if (options.command === 'integrate') return commandIntegrate(options, context);
  if (options.command === 'retire') return commandRetire(options, context);
  return commandGuard(options, context);
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
  USAGE,
  parseArguments,
  run,
};
