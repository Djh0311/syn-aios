'use strict';

// Adaptive Harness v0.5 — 生命周期与终态（AH-050-02）
//
// 需求溯源：
//   EX-5  进入 HISTORY 之后没有任何出边；历史按编号显式查询，永不冒充 current
//   EX-6  被卡住的任务用 PARKED 表达，并且必须真的挪到 parked 目录
//   KP-14 终态结果的真值表：结论文本不是任何一条的输入
//   EX-4  验收不是状态轴——本文件的纯函数拿不到验收段落
//   §6.4  参与写面相交判定的是派生谓词，不是一个独立状态值
//
// 纯函数、零 IO：Git 事实由调用方注入，本文件自己不去问版本库。
// 注入而非自取，才能把真值表的每一条分支都构造出负样本（TS-6）。

const LIFECYCLE_VALUES = Object.freeze(['DRAFT', 'READY', 'ACTIVE', 'PARKED', 'HISTORY']);

const PLAN_LIFECYCLE_VALUES = Object.freeze(['DRAFT', 'ACTIVE', 'PARKED', 'HISTORY']);

// 终态结果轴。它与生命周期是两个轴：生命周期说东西在哪，结果说这件事是怎么收的。
const RESULT_VALUES = Object.freeze(['COMPLETED', 'STOPPED', 'CANCELLED', 'DECOMPOSED', 'SUPERSEDED']);

// 合法流转全表。不在表内即拒绝；HISTORY 没有出边。
const PLAN_TRANSITIONS = Object.freeze({
  DRAFT: Object.freeze(['ACTIVE', 'PARKED', 'HISTORY']),
  ACTIVE: Object.freeze(['PARKED', 'HISTORY']),
  PARKED: Object.freeze(['ACTIVE', 'HISTORY']),
  HISTORY: Object.freeze([]),
});

const TASK_TRANSITIONS = Object.freeze({
  DRAFT: Object.freeze(['READY', 'HISTORY']),
  READY: Object.freeze(['ACTIVE', 'PARKED', 'HISTORY']),
  ACTIVE: Object.freeze(['PARKED', 'HISTORY']),
  PARKED: Object.freeze(['ACTIVE', 'HISTORY']),
  HISTORY: Object.freeze([]),
});

function transitionTableFor(kind) {
  return kind === 'TASK' ? TASK_TRANSITIONS : PLAN_TRANSITIONS;
}

function lifecycleValuesFor(kind) {
  return kind === 'TASK' ? LIFECYCLE_VALUES : PLAN_LIFECYCLE_VALUES;
}

/**
 * 合法流转判定。BLOCKED 不是生命周期取值——被卡住的任务用 PARKED 表达（§4.3）。
 */
function transitionAllowed(kind, from, to) {
  const table = transitionTableFor(kind);
  if (!Object.prototype.hasOwnProperty.call(table, from)) {
    return { allowed: false, reason: 'LIFECYCLE_FROM_UNKNOWN' };
  }
  if (!lifecycleValuesFor(kind).includes(to)) {
    return { allowed: false, reason: 'LIFECYCLE_TO_UNKNOWN' };
  }
  if (!table[from].includes(to)) {
    return { allowed: false, reason: 'LIFECYCLE_TRANSITION_NOT_IN_TABLE' };
  }
  return { allowed: true, reason: null };
}

// ---------------------------------------------------------------------------
// 终态结果真值表（§2.2 / KP-14）
// ---------------------------------------------------------------------------

/**
 * 判定输入只有三样：git 事实、verification 状态、图谱事实。
 *
 * 签名里根本不接收验收段落——不是「接收了但不用」，是拿不到。
 * 这就是 EX-4「验收不参与任何状态判定」的机械保证。
 *
 * @param {object} facts
 *   facts.productCommit          {string|null}  产品提交 OID
 *   facts.productCommitTouchesScope {boolean}   该提交对 write-scope 的 diff 非空
 *   facts.integratedNow          {boolean}      退场那一刻现跑的 ancestor 判定
 *   facts.requiredVerification   {Array<{id,status}>}
 *   facts.noProductChange        {boolean}
 *   facts.scopedDiffEmpty        {boolean}      写面路径级零 diff 现场核验
 *   facts.wipCommit              {string|null}
 *   facts.disposition            {string|null}
 *   facts.worktreeClean          {boolean}       未提交改动是否为零
 *   facts.baseOidImmutable       {boolean}       冻结 base 是否为不可移动的完整 OID
 *   facts.productCommitExists    {boolean}       声明的产品提交能否解析
 *   facts.productCommitOnTaskBranch {boolean}    产品提交是否属于任务分支
 *   facts.productCommitAfterBase {boolean}       产品提交是否在冻结 base 之后
 *   facts.productCommitChangesWithinScope {boolean} 产品提交范围是否完全落在冻结写面
 *   facts.productCommitCoversTaskHead {boolean}  product/WIP 是否覆盖任务分支最新提交
 *   facts.integrationRefOnBase   {boolean}       固定 integration ref 是否确属 base branch
 *   facts.wipCommitExists        {boolean}       声明的 WIP 提交能否解析
 *   facts.wipCommitOnTaskBranch  {boolean}       WIP 提交是否属于任务分支
 *   facts.wipCommitAfterBase      {boolean}       WIP 是否严格晚于冻结 base
 *   facts.wipCommitChangesWithinScope {boolean}  WIP 提交范围是否完全落在冻结写面
 *   facts.wipCommitCoversTaskHead {boolean}       WIP 是否覆盖任务分支最新提交
 *   facts.taskChangesWithinScope  {boolean}       base..HEAD 是否存在未回填的越界改动
 *   facts.hasNonHistoryDescendant {boolean}
 *   facts.hasNonHistoryDependent  {boolean}
 *   facts.splitSuccessors        {Array<string>}
 *   facts.replaceSuccessors      {Array<string>}
 */
function resultSupported(result, facts) {
  const input = facts || {};
  const required = Array.isArray(input.requiredVerification) ? input.requiredVerification : [];
  const allRequiredPass = required.length === 0
    ? input.requiredVerificationEmptyIsPass === true
    : required.every((entry) => entry && entry.status === 'PASS');
  const transferredOrRetained = input.disposition === 'RETAINED' || input.disposition === 'TRANSFERRED';
  const graphClear = input.hasNonHistoryDescendant !== true && input.hasNonHistoryDependent !== true;
  const declaredCommitsAreReal = input.worktreeClean !== false
    && input.baseOidImmutable !== false
    && input.taskChangesWithinScope !== false
    && !(input.productCommit && input.productCommitExists === false)
    && !(input.productCommit && input.productCommitOnTaskBranch === false)
    && !(input.productCommit && input.productCommitAfterBase === false)
    && !(input.productCommit && input.productCommitTouchesScope === false)
    && !(input.productCommit && input.productCommitChangesWithinScope === false)
    && !(input.productCommit && input.productCommitCoversTaskHead === false)
    && !(input.wipCommit && input.wipCommitExists === false)
    && !(input.wipCommit && input.wipCommitOnTaskBranch === false)
    && !(input.wipCommit && input.wipCommitAfterBase === false)
    && !(input.wipCommit && input.wipCommitChangesWithinScope === false)
    && !(input.wipCommit && input.wipCommitCoversTaskHead === false);

  // 所有结果都必须面对同一份在办图和同一份 Git 事实。否则只要换一个
  // result 词就能让仍有后代/依赖、未提交 WIP 或伪造 OID 的任务逃离 live 图。
  if (!graphClear || !declaredCommitsAreReal) return false;

  switch (result) {
    case 'COMPLETED':
      return Boolean(
        input.productCommit
        && input.productCommitTouchesScope === true
        && input.integratedNow === true
        && allRequiredPass
        && !input.wipCommit
        && input.integrationRefOnBase !== false,
      );
    case 'STOPPED':
    case 'CANCELLED': {
      const cleanZeroDiff = input.noProductChange === true
        && input.scopedDiffEmpty === true
        && !input.wipCommit
        && !input.productCommit;
      const disposedWork = Boolean(input.productCommit || input.wipCommit) && transferredOrRetained;
      return cleanZeroDiff || disposedWork;
    }
    case 'DECOMPOSED':
      return Array.isArray(input.splitSuccessors)
        && input.splitSuccessors.length > 0
        && transferredOrRetained;
    case 'SUPERSEDED':
      return Array.isArray(input.replaceSuccessors)
        && input.replaceSuccessors.length > 0
        && transferredOrRetained;
    default:
      return false;
  }
}

/** 真值表允许的全部结果。空数组意味着现实还不支持任何一种诚实退场。 */
function supportedResults(facts) {
  return RESULT_VALUES.filter((result) => resultSupported(result, facts));
}

// ---------------------------------------------------------------------------
// §6.4 判定参与的派生谓词（§2.4）
// ---------------------------------------------------------------------------

const RETIRING_DISPOSITIONS = Object.freeze([
  'RETAINED', 'TRANSFERRED', 'READY_FOR_CONFIRMED_REMOVAL', 'REMOVED',
]);

/**
 * 参与 §6.4 写面相交判定 ⟺ 节点有 git binding
 *   ∧ ¬( lifecycle = HISTORY ∧ disposition ∈ 四值 )
 *
 * 推论：DRAFT / READY 没有 binding ⇒ 不参与（刚建好的一批叶子不互相预占写面）；
 * PARKED 且已有产品提交未集成 ⇒ 仍参与（这就是「等待集成」，不需要独立状态值）。
 *
 * 物理位置决定默认加载，不决定判定参与——这两件事必须用不同的谓词。
 */
function participatesInScopeJudgement(node) {
  if (!node || typeof node !== 'object') return false;
  const binding = node.git;
  if (!binding || typeof binding !== 'object') return false;
  if (node.lifecycle !== 'HISTORY') return true;
  return !RETIRING_DISPOSITIONS.includes(binding.disposition);
}

/** 默认视野只装 current 区。parked 与 history 都退出默认加载（EX-5 / EX-6）。 */
function inDefaultView(node) {
  if (!node || typeof node !== 'object') return false;
  return node.lifecycle === 'READY' || node.lifecycle === 'ACTIVE';
}

module.exports = {
  LIFECYCLE_VALUES,
  PLAN_LIFECYCLE_VALUES,
  RESULT_VALUES,
  PLAN_TRANSITIONS,
  TASK_TRANSITIONS,
  RETIRING_DISPOSITIONS,
  transitionTableFor,
  lifecycleValuesFor,
  transitionAllowed,
  resultSupported,
  supportedResults,
  participatesInScopeJudgement,
  inDefaultView,
};
