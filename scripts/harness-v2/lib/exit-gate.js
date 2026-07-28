'use strict';

// Adaptive Harness v0.5 — 统一收尾门（AH-050-02）
//
// 需求溯源：EX-3 · EX-4 · EX-6 · EX-7 · GIT-4..GIT-10 · TS-1 · KP-14 · §13
//
// §13.1 总则，三条同时成立才算合格：
//   * 正向齐备必放行：十四项全齐时必须放行，放行后不得再要求任何补充。
//     本函数返回 {allowed, unmet}；unmet 为空时写入路径不得再查任何东西。
//   * 拒绝必指到项号：unmet 的每一项带 1–14 的固定编号，枚举里没有 OTHER。
//     产生不了编号的拒绝在类型层面就构造不出来。
//   * 只缺一项不得多报：十四个谓词在同一份不可变快照上各自独立求值、不短路、
//     不早返回，且任何谓词都不得把「另一个谓词失败了」当输入。
//
// 适用范围包含「做不下去」：被卡住的移出与完成后的退场用同一套判据（EX-6）。
//
// 本文件只判，不写。判定与写入分离，否则「齐备必放行」无法单独验证。

const lifecycle = require('./lifecycle');

const LEAF_GATE_ITEMS = Object.freeze([
  { item: 1, code: 'ITEM_01_DELIVERABLE_OR_TERMINATION', title: '交付物完成，或终止原因明确' },
  { item: 2, code: 'ITEM_02_WRITE_SCOPE_NO_BREACH', title: 'write-scope 没有越界' },
  { item: 3, code: 'ITEM_03_DIRECT_VERIFICATION_RAN', title: '直接验证已经运行且跑的是当前这版' },
  { item: 4, code: 'ITEM_04_RESULT_TRUTH_TABLE', title: 'result 符合真值表，且退场正文含一段自由文本的验收结论' },
  { item: 5, code: 'ITEM_05_COMMIT_REALITY', title: '有产品改动时提交真实；零改动时路径级核验通过' },
  { item: 6, code: 'ITEM_06_WORKTREE_STATE_CHECKED', title: 'task diff 与整个工作副本状态都已检查' },
  { item: 7, code: 'ITEM_07_ASSET_DISPOSITION', title: '新增测试 / fixture / diagnostic / evidence 已有去向' },
  { item: 8, code: 'ITEM_08_BLOCKING_QUESTIONS', title: 'blocking question 已解决或转交' },
  { item: 9, code: 'ITEM_09_INTEGRATION_OR_PARKED', title: '已正确集成，或明确停在等待集成' },
  { item: 10, code: 'ITEM_10_GIT_DISPOSITION', title: 'branch / worktree 有四种 disposition 之一' },
  { item: 11, code: 'ITEM_11_CURRENT_REPOINTED', title: 'CURRENT 已指向 successor / parent / idle' },
  { item: 12, code: 'ITEM_12_NO_UNAUTHORIZED_CLAIMS', title: '没把 push / 发布 / 物理删除 / 迁移写成已授权' },
  { item: 13, code: 'ITEM_13_DESCENDANTS_SETTLED', title: '子节点已全部处理完' },
  { item: 14, code: 'ITEM_14_NO_DEPENDENTS', title: '没有其他 non-history 任务依赖本任务' },
]);

const NON_LEAF_GATE_ITEMS = Object.freeze([
  { item: 1, code: 'NON_LEAF_01_DESCENDANTS_SETTLED', title: '全部子节点已 withdraw / transfer / 进入 history' },
  { item: 2, code: 'NON_LEAF_02_NO_DEPENDENTS', title: '无其他 non-history 任务依赖其产出或 branch' },
  { item: 3, code: 'NON_LEAF_03_BOUNDARY_MATCHES_DELIVERY', title: '阶段边界与实际交付一致' },
  { item: 4, code: 'NON_LEAF_04_OPEN_QUESTIONS_OWNED', title: '未决问题已转交明确 owner' },
]);

// 需要单独授权的四类高危动作（GIT-10）。流程里写过不算数，每次单独拿授权。
const SEPARATE_CONFIRMATION_ACTIONS = Object.freeze(['push', 'publish', 'physical-removal', 'project-migration']);

// ---------------------------------------------------------------------------
// 路径前缀：按目录边界判，不按字符串相等（§6.4）
// ---------------------------------------------------------------------------

function normalizePath(value) {
  return String(value === null || value === undefined ? '' : value).replace(/\\/g, '/').replace(/^\.\//, '').replace(/\/+$/, '');
}

function withinPrefix(candidate, prefix) {
  const target = normalizePath(candidate);
  const base = normalizePath(prefix);
  if (base === '' || target === '') return false;
  return target === base || target.startsWith(`${base}/`);
}

function withinAny(candidate, prefixes) {
  const list = Array.isArray(prefixes) ? prefixes : [];
  return list.some((prefix) => withinPrefix(candidate, prefix));
}

// ---------------------------------------------------------------------------
// 快照
// ---------------------------------------------------------------------------

function textOf(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function listOf(value) {
  return Array.isArray(value) ? value : [];
}

function freezeSnapshot(input) {
  const raw = input || {};
  const node = raw.node || {};
  const snapshot = {
    node,
    binding: node.git && typeof node.git === 'object' ? node.git : {},
    closeout: raw.closeout || {},
    git: raw.git || {},
    assets: listOf(raw.assets),
    assetAudit: raw.assetAudit && typeof raw.assetAudit === 'object' ? raw.assetAudit : null,
    verificationAudit: raw.verificationAudit && typeof raw.verificationAudit === 'object'
      ? raw.verificationAudit
      : null,
    agentAudit: raw.agentAudit && typeof raw.agentAudit === 'object' ? raw.agentAudit : null,
    traceAudit: raw.traceAudit && typeof raw.traceAudit === 'object' ? raw.traceAudit : null,
    descendants: listOf(raw.nonHistoryDescendants),
    dependents: listOf(raw.nonHistoryDependents),
    current: raw.current || {},
    truth: raw.truth || {},
    transferClaim: raw.transferClaim && typeof raw.transferClaim === 'object'
      ? raw.transferClaim
      : null,
  };
  return Object.freeze(snapshot);
}

// 终态结果是 closeout 的提案，不是仍在 non-history 平面的 node 字段。
// HISTORY 写入时调用方会把这份提案落进终态 node；在等待 integration 的
// PARKED 分支，提案必须留在 closeout，不能提前把 node 伪造成终态。
function terminalProposal(snapshot) {
  return snapshot.closeout.result;
}

function targetLifecycle(snapshot) {
  return textOf(snapshot.closeout.targetLifecycle) || snapshot.node.lifecycle;
}

// ---------------------------------------------------------------------------
// 十四个谓词。每个只读快照，互不引用彼此的结论。
// ---------------------------------------------------------------------------

// 第 1 项：结果是什么。COMPLETED 时逐条 acceptance-criteria 要有对应结论行；
// 否则终止原因段非空（EX-3 第 1 问）。
function item01(snapshot) {
  const criteria = listOf(snapshot.node['acceptance-criteria']);
  const conclusions = listOf(snapshot.closeout.criteriaConclusions).filter((line) => textOf(line) !== '');
  if (terminalProposal(snapshot) === 'COMPLETED') {
    if (conclusions.length < criteria.length) {
      return { ok: false, evidence: `acceptance-criteria ${criteria.length} 条，逐条结论只有 ${conclusions.length} 条` };
    }
  } else if (textOf(snapshot.closeout.terminationReason) === '') {
    return { ok: false, evidence: '非 COMPLETED 结果必须写明终止原因' };
  }
  const audit = snapshot.agentAudit;
  const unverified = audit && Array.isArray(audit.unverifiableClaims)
    ? audit.unverifiableClaims
    : [];
  const empty = audit && Array.isArray(audit.emptyDiffClaims)
    ? audit.emptyDiffClaims
    : [];
  if (unverified.length > 0 || empty.length > 0) {
    const detail = [];
    if (empty.length > 0) detail.push(`自报完成但真实 base..head diff 为空：${empty.map((entry) => entry.id).join('，')}`);
    if (unverified.length > 0) detail.push(`自报完成但真实 base..head 事实不可核对：${unverified.map((entry) => entry.id).join('，')}`);
    return { ok: false, evidence: detail.join('；') };
  }
  return { ok: true, evidence: null };
}

// 第 2 项（退场当场核对现实）：改了哪些文件、有没有越界。
// 输入是现场向 Git 取来的 base..HEAD 改动路径清单（name-only 的那份文件清单）；
// 每条路径必须落在某个 write-scope 前缀内、且不落在 forbidden-scope 内。
// 唯一豁免是历史平面根——控制面不参与越界判定（§6.4）。
// TRANSFERRED split/replace 的越界由承接方 write-scope 认领：合并后的声明
// 能覆盖全部改动时，越界已经有人负责，不再判来源违约（与终态真值表同一规则）。
function item02(snapshot) {
  const changed = listOf(snapshot.git.changedPaths);
  const claim = snapshot.transferClaim && snapshot.transferClaim.active === true
    ? snapshot.transferClaim
    : null;
  const writeScope = claim ? listOf(claim.unionWriteScope) : listOf(snapshot.node['write-scope']);
  const forbidden = listOf(snapshot.node['forbidden-scope']);
  const exempt = listOf(snapshot.git.controlPlaneExemptPrefixes);
  const breaches = [];
  for (const filePath of changed) {
    if (withinAny(filePath, exempt)) continue;
    if (withinAny(filePath, forbidden)) { breaches.push(`${filePath}（落在禁区）`); continue; }
    if (!withinAny(filePath, writeScope)) breaches.push(`${filePath}（不在写面内）`);
  }
  const agentBreaches = snapshot.agentAudit && Array.isArray(snapshot.agentAudit.scopeBreachClaims)
    ? snapshot.agentAudit.scopeBreachClaims
    : [];
  if (agentBreaches.length > 0) {
    for (const claim of agentBreaches) {
      const paths = Array.isArray(claim.scopeBreaches) ? claim.scopeBreaches : [];
      breaches.push(`子执行者 ${claim.id || '（未命名）'} 越 delegated scope：${paths.join('，') || '（未给出范围）'}`);
    }
  }
  return breaches.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: `越界路径：${breaches.join('，')}` };
}

// 第 3 项：验证结论是什么、是不是这一版跑出来的结果（GIT-7 / TS-4）。
// required=true 的每条都要有 run 记录，且 run.head-oid 等于当前任务 HEAD。
function item03(snapshot) {
  const entries = listOf(snapshot.node.verification).filter((entry) => entry && entry.required === true);
  const head = textOf(snapshot.git.headOid);
  const missing = [];
  for (const entry of entries) {
    if (!entry.run) { missing.push(`${entry.id}：没有 run 记录`); continue; }
    if (textOf(entry.run['head-oid']) !== head) {
      missing.push(`${entry.id}：run 记的是 ${entry.run['head-oid']}，当前 HEAD 是 ${head}`);
    }
  }
  const selectionProblems = snapshot.verificationAudit
    && Array.isArray(snapshot.verificationAudit.problems)
    ? snapshot.verificationAudit.problems
    : [];
  if (selectionProblems.length > 0) {
    missing.push(
      `required verification 选择审计失败：${selectionProblems
        .map((problem) => problem.code || 'VERIFICATION_SELECTION_UNKNOWN')
        .join('，')}`,
    );
  }
  const agentMissing = snapshot.agentAudit && Array.isArray(snapshot.agentAudit.missingVerificationClaims)
    ? snapshot.agentAudit.missingVerificationClaims
    : [];
  if (agentMissing.length > 0) {
    missing.push(`子执行者完成声明缺 verification 输出：${agentMissing.map((entry) => entry.id).join('，')}`);
  }
  const traceProblems = snapshot.traceAudit && Array.isArray(snapshot.traceAudit.problems)
    ? snapshot.traceAudit.problems
    : [];
  if (traceProblems.length > 0) {
    missing.push(
      `required verification 一跳追溯失败：${traceProblems
        .map((problem) => `${problem.id || '（未命名）'}:${problem.code || 'TRACE_UNKNOWN'}`)
        .join('，')}`,
    );
  }
  return missing.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: missing.join('；') };
}

// 第 4 项：closeout 的 terminal proposal 符合真值表，另加一段自由文本的验收结论。
// 那段人话只判存在且非空——谁验收的、结论是什么，或明写「本任务无独立验收」。
// 它不参与 result 的计算：算 result 的纯函数签名里根本收不到它（EX-4 / KP-14）。
function item04(snapshot) {
  const statement = textOf(snapshot.closeout.acceptanceStatement);
  if (statement === '') {
    return { ok: false, evidence: '退场正文缺一段自由文本的验收结论（一段人话，不是一个姓名或一个取值）' };
  }
  const result = terminalProposal(snapshot);
  if (!lifecycle.RESULT_VALUES.includes(result)) {
    return { ok: false, evidence: `closeout result ${result} 不在五值之内` };
  }
  // 等待 integration 的 COMPLETED 还不能宣称已经进入 HISTORY；但它不能因此
  // 卡死在 item 04 / item 09 之间。这里仅把未来 integration 这一件事作为
  // proposal 的待满足条件，其余产品提交、verification、图谱和 Git 事实仍由
  // 同一张真值表逐项核验。真正写入 HISTORY 时 item 09 再核验现场已集成。
  let truth = targetLifecycle(snapshot) === 'PARKED' && result === 'COMPLETED'
    ? { ...snapshot.truth, integratedNow: true }
    : snapshot.truth;
  // TRANSFERRED split/replace：来源的越界改动由承接方 write-scope 认领，
  // 真值表按合并后的认领范围判定（与 task-lifecycle 的同一规则）。
  const claim = snapshot.transferClaim && snapshot.transferClaim.active === true
    ? snapshot.transferClaim
    : null;
  if (claim && (result === 'DECOMPOSED' || result === 'SUPERSEDED') && truth && truth.taskChangesWithinScope === false) {
    const changed = listOf(snapshot.git && snapshot.git.changedPaths);
    const exempt = listOf(snapshot.git && snapshot.git.controlPlaneExemptPrefixes);
    const forbidden = listOf(snapshot.node && snapshot.node['forbidden-scope']);
    const union = listOf(claim.unionWriteScope);
    const claimed = changed.every((filePath) => withinAny(filePath, exempt)
      || (!withinAny(filePath, forbidden) && withinAny(filePath, union)));
    truth = { ...truth, taskChangesWithinScope: claimed };
  }
  if (!lifecycle.resultSupported(result, truth)) {
    return { ok: false, evidence: `现实事实不支持 result=${result}` };
  }
  // STOPPED / CANCELLED 不能把已经形成的产品提交伪装成「未完成的可保留工作」。
  // 这两种结果只诚实地覆盖两条路：真正的零产品改动，或尚未集成、由 retain /
  // transfer 承接的 WIP。COMPLETED 才是 product commit 的唯一正常出口。
  if (result === 'STOPPED' || result === 'CANCELLED') {
    if (snapshot.binding['product-commit']) {
      return { ok: false, evidence: 'STOPPED / CANCELLED 不接受 product commit；只能是零改动或未集成 WIP 的 retain / transfer' };
    }
    const zeroDiff = snapshot.binding['no-product-change'] === true
      && snapshot.git.scopedDiffEmpty === true
      && !snapshot.binding['wip-commit'];
    const unintegratedWip = Boolean(snapshot.binding['wip-commit'])
      && snapshot.git.commitReachable === true
      && snapshot.git.wipCarriedIntoIntegration !== true
      && ['RETAINED', 'TRANSFERRED'].includes(snapshot.binding.disposition);
    if (!zeroDiff && !unintegratedWip) {
      return { ok: false, evidence: 'STOPPED / CANCELLED 只能是 zero-diff，或尚未集成且 RETAINED / TRANSFERRED 的 WIP' };
    }
  }
  return { ok: true, evidence: null };
}

// 第 5 项：改动最终落在哪个提交上（GIT-5 / G-24）。
function item05(snapshot) {
  const noProductChange = snapshot.binding['no-product-change'] === true;
  if (noProductChange) {
    if (snapshot.binding['product-commit']) {
      return { ok: false, evidence: '声明 no-product-change 时不得同时保留 product commit' };
    }
    if (snapshot.git.scopedDiffEmpty !== true) {
      return { ok: false, evidence: 'no-product-change 声明为真，但写面路径级 diff 非空' };
    }
    if (snapshot.binding['wip-commit']) {
      return { ok: false, evidence: '存在 WIP 提交时不得用 no-product-change 掩盖' };
    }
    return { ok: true, evidence: null };
  }
  const commit = snapshot.binding['product-commit'] || snapshot.binding['wip-commit'] || null;
  if (!commit) {
    // TRANSFERRED split/replace：product/WIP 随 branch/worktree 整体转交承接方，
    // 绑定动作发生在承接方的 record。前提是合并声明确实覆盖全部改动——无人
    // 认领的改动不能借 TRANSFERRED 一词消失。
    const claim = snapshot.transferClaim && snapshot.transferClaim.active === true
      ? snapshot.transferClaim
      : null;
    const result = terminalProposal(snapshot);
    if (claim && (result === 'DECOMPOSED' || result === 'SUPERSEDED')) {
      const changed = listOf(snapshot.git && snapshot.git.changedPaths);
      const exempt = listOf(snapshot.git && snapshot.git.controlPlaneExemptPrefixes);
      const forbidden = listOf(snapshot.node && snapshot.node['forbidden-scope']);
      const union = listOf(claim.unionWriteScope);
      const claimed = changed.every((filePath) => withinAny(filePath, exempt)
        || (!withinAny(filePath, forbidden) && withinAny(filePath, union)));
      if (claimed) return { ok: true, evidence: null };
    }
    return { ok: false, evidence: '有产品改动但既无 product commit 也无 WIP commit' };
  }
  if (snapshot.git.commitReachable !== true) {
    return { ok: false, evidence: `提交 ${commit} 在本仓不可达` };
  }
  return { ok: true, evidence: null };
}

// 第 6 项：task diff 与整个工作副本状态都已检查；两者不一致时报冲突而不是覆盖（WK-1）。
function item06(snapshot) {
  const now = listOf(snapshot.git.statusNow).slice().sort();
  const atInspect = snapshot.git.statusAtInspect;
  if (!Array.isArray(atInspect)) {
    return { ok: false, evidence: '没有开工时的工作副本状态快照可供对账' };
  }
  const before = atInspect.slice().sort();
  if (before.length !== now.length || before.some((line, offset) => line !== now[offset])) {
    return { ok: false, evidence: '工作副本状态与 inspect 快照不一致，报冲突而不是覆盖' };
  }
  return { ok: true, evidence: null };
}

// 第 7 项：这次新增或改动的测试 / fixture / diagnostic / runner / evidence 逐份定去向（TS-1 / EX-8）。
function item07(snapshot) {
  const unresolved = snapshot.assets.filter((asset) => !asset
    || textOf(asset.assetClass) === ''
    || textOf(asset.disposition) === '');
  const auditProblems = snapshot.assetAudit && Array.isArray(snapshot.assetAudit.problems)
    ? snapshot.assetAudit.problems
    : [];
  if (unresolved.length === 0 && auditProblems.length === 0) {
    return { ok: true, evidence: null };
  }
  const evidence = [];
  if (unresolved.length > 0) evidence.push(`${unresolved.length} 份资产没有分类或没有去向`);
  if (auditProblems.length > 0) {
    evidence.push(`本次 changed paths 的资产审计失败：${auditProblems.map((problem) => `${problem.path} (${problem.code})`).join('，')}`);
  }
  return { ok: false, evidence: evidence.join('；') };
}

// 第 8 项：未做完的那些交给谁（carry-over 的 owner）。
// 每条未决项要么 RESOLVED，要么 TRANSFERRED 且带非空 owner（KP-5 / EX-3 第 5 问）。
function item08(snapshot) {
  const questions = listOf(snapshot.closeout.openQuestions);
  const bad = [];
  for (const question of questions) {
    const status = textOf(question && question.status);
    if (status === 'RESOLVED') continue;
    if (status === 'TRANSFERRED' && textOf(question.owner) !== '') continue;
    bad.push(textOf(question && question.id) || '(未命名未决项)');
  }
  return bad.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: `未解决也未转交（或转交没写负责人）：${bad.join('，')}` };
}

// 第 9 项：已正确集成，或明确停在等待集成。二者互斥，不能混写。
function item09(snapshot) {
  const target = targetLifecycle(snapshot);
  const result = terminalProposal(snapshot);
  if (target === 'HISTORY') {
    if (snapshot.git.integratedNow !== true && result === 'COMPLETED') {
      return { ok: false, evidence: 'COMPLETED 进入 history 必须现场核验集成事实' };
    }
    if ((result === 'STOPPED' || result === 'CANCELLED')
      && snapshot.git.wipCarriedIntoIntegration === true) {
      return { ok: false, evidence: 'STOPPED / CANCELLED 不得把废弃 WIP 带进 integration' };
    }
    return { ok: true, evidence: null };
  }
  if (target === 'PARKED') {
    if (snapshot.node.result !== undefined && snapshot.node.result !== null) {
      return { ok: false, evidence: '停在等待集成的 PARKED node 不得写 result；terminal proposal 必须留在 closeout' };
    }
    if (snapshot.git.integratedNow === true) {
      return { ok: false, evidence: '已集成的任务不得停在等待集成的 PARKED，应进入 history' };
    }
    if (!lifecycle.participatesInScopeJudgement(snapshot.node)) {
      return { ok: false, evidence: '等待集成的声明必须继续参与写面判定' };
    }
    return { ok: true, evidence: null };
  }
  return { ok: false, evidence: `退场目标只能是 HISTORY 或 PARKED，收到 ${target}` };
}

// 第 10 项：branch / worktree 的去向（§6.3）。
function item10(snapshot) {
  const disposition = textOf(snapshot.binding.disposition);
  if (disposition === '') return { ok: false, evidence: 'branch / worktree 没有 disposition' };
  if (!lifecycle.RETIRING_DISPOSITIONS.includes(disposition)) {
    return { ok: false, evidence: `disposition ${disposition} 不在四值之内` };
  }
  if (disposition === 'RETAINED') {
    const detail = snapshot.closeout.retention || {};
    const missing = ['owner', 'reason', 'review-by'].filter((key) => textOf(detail[key]) === '');
    if (missing.length) return { ok: false, evidence: `RETAINED 缺 ${missing.join(' / ')}` };
  }
  if (disposition === 'TRANSFERRED' && textOf(snapshot.closeout.successorId) === '') {
    return { ok: false, evidence: 'TRANSFERRED 必须给出 successor 的编号' };
  }
  return { ok: true, evidence: null };
}

// 第 11 项：下一个入口是什么。重生成本工作副本的 CURRENT，其 next entry 必须与 closeout
// 写下的下一个入口一致；不一致就是两个说法。
function item11(snapshot) {
  const declared = textOf(snapshot.closeout.nextEntry);
  const regenerated = textOf(snapshot.current.nextEntry);
  if (declared === '') return { ok: false, evidence: 'closeout 没有写下一个入口（successor / parent / idle）' };
  if (regenerated === '') return { ok: false, evidence: '没有重生成 CURRENT 可供对账' };
  if (declared !== regenerated) {
    return { ok: false, evidence: `closeout 写的是 ${declared}，重生成的 CURRENT 指向 ${regenerated}` };
  }
  return { ok: true, evidence: null };
}

// 第 12 项：高危动作没有被写成已授权（GIT-10）。
function item12(snapshot) {
  const claims = listOf(snapshot.closeout.completionClaims).map((claim) => textOf(claim)).filter(Boolean);
  const confirmations = listOf(snapshot.node.confirmations)
    .map((entry) => textOf(entry && entry.action))
    .filter(Boolean);
  const unbacked = claims.filter((claim) => SEPARATE_CONFIRMATION_ACTIONS.includes(claim)
    && !confirmations.includes(claim));
  return unbacked.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: `声称已完成但没有对应授权记录：${unbacked.join('，')}` };
}

// 第 13 项：子节点全部处理完——不得存在 DRAFT / READY / ACTIVE / PARKED 的后代悬着
// （EX-3 第 6 问）。范围覆盖 drafts + current + parked + pending-start 的完整集合。
function item13(snapshot) {
  return snapshot.descendants.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: `仍有 non-history 后代：${snapshot.descendants.join('，')}` };
}

// 第 14 项：还有没有在办任务依赖本任务的产出或 branch（EX-3 第 7 问）。
// 有就拒绝并指名；无人依赖时必须放行——只写拒绝侧不算通过（G-2 双侧）。
function item14(snapshot) {
  return snapshot.dependents.length === 0
    ? { ok: true, evidence: null }
    : { ok: false, evidence: `以下 non-history 任务仍依赖本任务：${snapshot.dependents.join('，')}` };
}

const LEAF_PREDICATES = Object.freeze([
  item01, item02, item03, item04, item05, item06, item07,
  item08, item09, item10, item11, item12, item13, item14,
]);

/**
 * 叶子收尾门。十四个谓词各自独立求值、不短路，缺失集合与构造的缺失集合逐项相等。
 * unmet 为空 ⇒ 放行，写入路径不得再查任何东西。
 */
function evaluateLeafExit(input) {
  const snapshot = freezeSnapshot(input);
  const results = [];
  for (let offset = 0; offset < LEAF_PREDICATES.length; offset += 1) {
    const spec = LEAF_GATE_ITEMS[offset];
    let outcome;
    try {
      outcome = LEAF_PREDICATES[offset](snapshot);
    } catch (error) {
      outcome = { ok: false, evidence: `谓词求值失败：${error.message}` };
    }
    results.push({ item: spec.item, code: spec.code, title: spec.title, ok: outcome.ok === true, evidence: outcome.evidence || null });
  }
  const unmet = results.filter((entry) => !entry.ok).map((entry) => ({
    item: entry.item,
    code: entry.code,
    title: entry.title,
    evidence: entry.evidence,
  }));
  return { allowed: unmet.length === 0, unmet, checked: results };
}

// ---------------------------------------------------------------------------
// 非叶子收尾门（§13.2）
// ---------------------------------------------------------------------------

/**
 * 阶段计划与根计划同样要退场。
 * 让一个计划或阶段结束之前，必须先确认它下面没有还悬着的后代——
 * 全部子节点都已 withdraw / transfer / 进入 history，一个都不得存在于
 * drafts / current / parked / pending-start；并且没有其它 non-history
 * 任务还依赖它的产出或 branch，有就拒绝并指名（EX-3 第 6 / 7 问）。
 *
 * 反过来同等重要：父节点有 non-history 子节点时必须留在 current，
 * 已进入 history 的子叶子不得成为父节点关闭的前置条件（EX-7 正向侧）。
 */
function evaluateNonLeafExit(input) {
  const raw = input || {};
  const descendants = listOf(raw.nonHistoryDescendants);
  const dependents = listOf(raw.nonHistoryDependents);
  const boundaryGaps = listOf(raw.boundaryGaps);
  const questions = listOf(raw.openQuestions);

  const outcomes = [
    descendants.length === 0
      ? { ok: true, evidence: null }
      : { ok: false, evidence: `仍有 non-history 子节点：${descendants.join('，')}` },
    dependents.length === 0
      ? { ok: true, evidence: null }
      : { ok: false, evidence: `以下 non-history 任务仍依赖它：${dependents.join('，')}` },
    boundaryGaps.length === 0
      ? { ok: true, evidence: null }
      : { ok: false, evidence: `阶段边界与实际交付对不上：${boundaryGaps.join('，')}` },
    (() => {
      const bad = questions.filter((question) => {
        const status = textOf(question && question.status);
        if (status === 'RESOLVED') return false;
        return !(status === 'TRANSFERRED' && textOf(question.owner) !== '');
      });
      return bad.length === 0
        ? { ok: true, evidence: null }
        : { ok: false, evidence: `${bad.length} 条未决问题没有明确 owner` };
    })(),
  ];

  const checked = outcomes.map((outcome, offset) => ({
    item: NON_LEAF_GATE_ITEMS[offset].item,
    code: NON_LEAF_GATE_ITEMS[offset].code,
    title: NON_LEAF_GATE_ITEMS[offset].title,
    ok: outcome.ok,
    evidence: outcome.evidence,
  }));
  const unmet = checked.filter((entry) => !entry.ok).map((entry) => ({
    item: entry.item,
    code: entry.code,
    title: entry.title,
    evidence: entry.evidence,
  }));
  return { allowed: unmet.length === 0, unmet, checked };
}

/** 父节点只要还有 non-history 子节点，就留在 current，不必等整棵树结束再一起搬（EX-7）。 */
function parentStaysCurrent(nonHistoryDescendants) {
  return listOf(nonHistoryDescendants).length > 0;
}

module.exports = {
  LEAF_GATE_ITEMS,
  NON_LEAF_GATE_ITEMS,
  SEPARATE_CONFIRMATION_ACTIONS,
  normalizePath,
  withinPrefix,
  withinAny,
  evaluateLeafExit,
  evaluateNonLeafExit,
  parentStaysCurrent,
};
