'use strict';

// Adaptive Harness v0.5 — 用户未提交内容的保护与高危动作的逐次授权（AH-050-07）
//
// 需求溯源：
//   GIT-10 任何时候都不擅自重置、清理、暂存或覆盖用户自己未提交的改动；
//          推送、远程合并、打标签、发布、删分支删工作树删用户文件，
//          **每次单独拿授权**，不因为流程里写过就算数。
//   GIT-11 不能只写一堆「不许动 Git」，却不给正常怎么提交、怎么并回主线、怎么收尾。
//   §6.2   不得覆盖用户已有 staged 内容，不得把用户原有未提交内容混进提交。
//   WK-1   动作前后对不上时报冲突，不静默覆盖。
//
// 本模块只读：登记与核对都靠读文件与只读 git 查询，不动用户的任何东西。

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const gitFacts = require('./git-facts');

// ---------------------------------------------------------------------------
// 一、绝不自动执行的 Git 动词
// ---------------------------------------------------------------------------

// 下面每一个动词都可能抹掉用户自己没提交的东西。Harness 永远不自动跑它们；
// 要跑，必须先拿到**本次这一个动作**的单独确认，而且确认是逐次的、用完即失效。
// 「流程文档里写过」不算授权，「上次批准过」也不算授权。
const NEVER_AUTOMATIC_GIT_VERBS = Object.freeze([
  'reset',
  'clean',
  'stash',
  'checkout -- ',
  'restore',
  'rebase',
  'push',
]);

// 需要逐次单独授权的高危动作（GIT-10 点名的那批）。
const HIGH_RISK_ACTIONS = Object.freeze([
  'push',
  'remote-merge',
  'tag',
  'publish',
  'release',
  'delete-branch',
  'delete-worktree',
  'delete-user-file',
  'physical-removal',
  'confirmed-replay',
  'local-integrate',
]);

/**
 * 一次高危动作的授权检查。缺少本次授权时**拒绝执行**，不给任何绕过。
 *
 * 三种东西一律不算授权：
 *   * 流程文档、runbook、任务模板里写过「本任务允许推送」；
 *   * 上一次同类动作的确认（授权逐次消耗，用过就失效）；
 *   * 目标对不上的确认（批的是 A 分支，不能拿来删 B 工作副本）。
 */
function requireFreshAuthorization(input) {
  const settings = input || {};
  const action = String(settings.action || '').trim();
  const target = settings.target === undefined || settings.target === null ? null : String(settings.target);
  const granted = Array.isArray(settings.confirmations) ? settings.confirmations : [];

  if (!HIGH_RISK_ACTIONS.includes(action)) {
    return { authorized: true, action, reason: 'NOT_HIGH_RISK', refusal: null };
  }
  const candidates = granted.filter((entry) => entry && String(entry.action || '').trim() === action);
  if (candidates.length === 0) {
    return {
      authorized: false,
      action,
      refusal: {
        code: 'SEPARATE_CONFIRMATION_REQUIRED',
        message: `高危动作「${action}」没有本次的单独授权，拒绝执行；流程里写过不算数`,
      },
    };
  }
  const usable = candidates.find((entry) => {
    if (entry.standing === true || entry.fromProcessDocument === true) return false;
    if (entry.consumed === true) return false;
    if (target !== null && entry.target !== undefined && entry.target !== null && String(entry.target) !== target) return false;
    return true;
  });
  if (!usable) {
    return {
      authorized: false,
      action,
      refusal: {
        code: 'SEPARATE_CONFIRMATION_NOT_FRESH',
        message: `「${action}」只有常设或已用过的授权记录，拒绝执行；每次都要单独拿授权`,
      },
    };
  }
  return { authorized: true, action, confirmation: usable, refusal: null };
}

/** 把一次授权标记为已消耗。逐次授权靠它，不靠自觉。 */
function consumeAuthorization(confirmations, confirmation) {
  return (Array.isArray(confirmations) ? confirmations : [])
    .map((entry) => (entry === confirmation ? { ...entry, consumed: true } : entry));
}

// ---------------------------------------------------------------------------
// 二、用户原有未提交内容：动作前登记，动作后核对
// ---------------------------------------------------------------------------

const DIRTY_STATES = Object.freeze({
  STAGED: 'STAGED',
  UNSTAGED: 'UNSTAGED',
  UNTRACKED: 'UNTRACKED',
});

function digestOfFile(absolutePath) {
  try {
    const buffer = fs.readFileSync(absolutePath);
    return crypto.createHash('sha256').update(buffer).digest('hex');
  } catch (error) {
    return null;
  }
}

function parsePorcelainLine(line) {
  const text = String(line);
  const code = text.slice(0, 2);
  const filePath = text.slice(3).trim();
  if (code === '??') return { path: filePath, state: DIRTY_STATES.UNTRACKED, code };
  if (code[1] !== ' ' && code[1] !== '') return { path: filePath, state: DIRTY_STATES.UNSTAGED, code };
  return { path: filePath, state: DIRTY_STATES.STAGED, code };
}

/**
 * 开工时登记用户自己原有的未提交内容：**逐条路径 + 逐条内容摘要**。
 *
 * 只记索引指纹是不够的——索引指纹证明不了 unstaged 与 untracked 的内容有没有被动过，
 * 而那恰恰是最容易被「顺手清理干净」的一批。这里对每条路径的实际文件内容取摘要。
 */
function registerDirtyBaseline(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const root = settings.root || cwd;
  const status = Array.isArray(settings.statusNow) ? settings.statusNow : gitFacts.porcelainStatus(cwd);
  const entries = [];
  for (const line of status) {
    const parsed = parsePorcelainLine(line);
    if (parsed.path === '') continue;
    entries.push({
      path: parsed.path,
      state: parsed.state,
      code: parsed.code,
      digest: digestOfFile(path.join(root, parsed.path)),
    });
  }
  entries.sort((left, right) => (left.path < right.path ? -1 : 1));
  return { registeredAt: settings.now || null, root, entries };
}

/**
 * 动作之后再核对一遍：登记过的每一条路径都必须原样还在，内容摘要一模一样。
 * 不一致就**报冲突**——不猜、不修、不覆盖，也不把记录改写成现实值。
 */
function verifyDirtyBaselineUnchanged(input) {
  const settings = input || {};
  const baseline = settings.baseline || { entries: [] };
  const root = settings.root || baseline.root || process.cwd();
  const conflicts = [];
  for (const entry of Array.isArray(baseline.entries) ? baseline.entries : []) {
    const after = digestOfFile(path.join(root, entry.path));
    if (after === null && entry.digest !== null) {
      conflicts.push({
        code: 'USER_WORK_DISAPPEARED',
        path: entry.path,
        message: `事后核对：用户原有未提交内容 ${entry.path} 不见了，报冲突`,
      });
      continue;
    }
    if (after !== entry.digest) {
      conflicts.push({
        code: 'USER_WORK_MODIFIED',
        path: entry.path,
        before: entry.digest,
        after,
        message: `事后核对：用户原有未提交内容 ${entry.path} 的摘要变了，报冲突`,
      });
    }
  }
  return { unchanged: conflicts.length === 0, conflicts };
}

/** 用户原有的脏检出可以被登记、保留和只读观察，但永远不能成为集成目标（§6.3）。 */
function assertNotIntegrationTarget(input) {
  const settings = input || {};
  const baseline = settings.baseline || { entries: [] };
  if ((baseline.entries || []).length === 0) return { ok: true, refusal: null };
  return {
    ok: false,
    refusal: {
      code: 'DIRTY_CHECKOUT_NOT_INTEGRATION_TARGET',
      message: '这个工作副本里有用户原有的未提交内容；它可以被登记与只读观察，但不能当集成目标',
    },
  };
}

// ---------------------------------------------------------------------------
// 三、每类禁令都指得到一条可执行的正常出口（GIT-11）
// ---------------------------------------------------------------------------

// 安全约束与「改动最终要合回去」是同一套设计。左边每一条禁令，右边都有一条
// 真的跑得起来的出口命令；出口跑不通时右边写的是「换哪条出口」，不是「到此为止」。
const PROHIBITION_EXITS = Object.freeze([
  {
    prohibition: '不整树暂存、不把不明来源的改动一把捞进提交',
    exit: 'node scripts/harness-v2/git-plan.js commit --task <ID> --paths <逐条点名>',
    note: '提交只装点名的那一批，并要求三段自述',
  },
  {
    prohibition: '不自动 rebase、不擅自改写历史',
    exit: 'node scripts/harness-v2/git-plan.js replay --task <ID>',
    note: '产出受确认的重放计划；经用户单独确认后由调用方执行，保留原 ref',
  },
  {
    prohibition: '不自动 push、不自动打标签发布',
    exit: 'node scripts/harness-v2/git-plan.js guard --action push --task <ID>',
    note: '逐次单独授权；没有本次授权就拒绝执行',
  },
  {
    prohibition: '不自动 reset / clean / stash 用户未提交的改动',
    exit: 'node scripts/harness-v2/git-plan.js inspect --task <ID>',
    note: '先登记用户原有未提交内容，动作后逐条核对；不一致报冲突',
  },
  {
    prohibition: '不在脏检出上集成、不接受非快进',
    exit: 'node scripts/harness-v2/git-plan.js integrate --task <ID>',
    note: '干净的专用 integration 工作副本 + 三证齐备 + 只快进',
  },
  {
    prohibition: '不留残留改动、不随手加忽略把脏东西藏起来',
    exit: 'node scripts/harness-v2/git-plan.js retire --task <ID>',
    note: '收尾时核对残留、忽略规则理由、写面内未跟踪的产品依赖',
  },
]);

function exitsForProhibitions() {
  return PROHIBITION_EXITS.map((entry) => ({ ...entry }));
}

module.exports = {
  NEVER_AUTOMATIC_GIT_VERBS,
  HIGH_RISK_ACTIONS,
  DIRTY_STATES,
  PROHIBITION_EXITS,
  requireFreshAuthorization,
  consumeAuthorization,
  registerDirtyBaseline,
  verifyDirtyBaselineUnchanged,
  assertNotIntegrationTarget,
  exitsForProhibitions,
};
