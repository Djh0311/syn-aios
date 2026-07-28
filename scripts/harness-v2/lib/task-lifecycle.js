'use strict';

// Adaptive Harness v0.5 — TASK 显式生命周期、split 与 replace（AH-050-05）
//
// 需求溯源：LY-1 · LY-3 · LY-4 · LY-5 · GIT-1 · GIT-2 · GIT-6 · EX-3 ·
// EX-5 · EX-6 · DP-3 · KP-13 · KP-14 · WK-1 · WK-2。
//
// 纯决策模块：只计算 change set，绝不读写文件、绝不调用 Git。CLI 负责把同一
// generation 的快照、Git 现实和 --write 传进来；store 负责原子落盘。

const nodeSchema = require('./node-schema');
const lifecycle = require('./lifecycle');
const graph = require('./graph');
const opening = require('./opening');
const prepare = require('./prepare');
const scope = require('./scope');

const TASK_SPLIT_REASONS = Object.freeze(['GOAL', 'ACCEPTANCE', 'AUTHORITY', 'OWNER', 'BASELINE', 'WORK_SURFACE']);
const REPLACE_REASONS = Object.freeze(['CONTRACT_INVALID']);
const TERMINAL_ACTION_RESULTS = Object.freeze(['COMPLETED', 'STOPPED', 'CANCELLED']);
const RETAIN_OR_TRANSFER = Object.freeze(['RETAINED', 'TRANSFERRED']);
// split / replace 的资源承接仍只可能是 retain 或 transfer。普通 finish 只有
// AH-050-08 closeout 调用方显式开启后才接受完整退场处置域；默认保持 AH-050-05
// 的两值语义。终态真值表仍由 lifecycle.resultSupported 独立判定，不能借新
// disposition 绕过 Git 事实。
const FINISH_DISPOSITIONS = lifecycle.RETIRING_DISPOSITIONS;
const TRANSFER_RESOURCE_KEYS = Object.freeze(['task-branch', 'worktree', 'product-commit', 'wip-commit']);

function failure(code, error, detail) {
  return { ok: false, code, error, detail: detail || null, changes: [] };
}

function success(detail) {
  return { ok: true, code: null, error: null, ...(detail || {}) };
}

function taskRecord(index, id) {
  const record = index && index.byId ? index.byId.get(id) : null;
  if (!record) return failure('TASK_NOT_FOUND', `在办平面里找不到 TASK ${id}`);
  if (!record.node || record.node.kind !== 'TASK') {
    return failure('TASK_KIND_INVALID', `${id} 不是 TASK，不能走 task lifecycle`);
  }
  return success({ record });
}

function hasGitBinding(node) {
  return Boolean(node && node.git && typeof node.git === 'object' && !Array.isArray(node.git));
}

function hasRequiredVerification(node) {
  return Array.isArray(node && node.verification)
    && node.verification.some((entry) => entry && entry.required === true);
}

function bodyValid(body) {
  return prepare.validateTaskBody(body);
}

// AH-050-14 把新正文收束为五区块后，已经在办的 v0.4 六区块任务不能因此
// 失去唯一的 record / exit 通道。兼容只在这里被显式消费：它不用于 ready、
// activate、park、resume、split、replace，也不接纳 HISTORY 再次回写。
function continuationBodyValid(record) {
  const current = bodyValid(record && record.body);
  if (current.ok) return current;
  const node = record && record.node;
  if (!node || node.lifecycle === 'HISTORY') return current;
  return prepare.validateContinuationTaskBody(record.body);
}

function candidateValid(candidate) {
  if (!candidate || !candidate.node) {
    return failure('TASK_INPUT_INVALID', 'TASK candidate 无法解析', {
      issues: candidate && Array.isArray(candidate.issues) ? candidate.issues : [],
    });
  }
  if (candidate.node.kind !== 'TASK') {
    return failure('TASK_KIND_INVALID', `task 入口只接收 TASK，收到 ${candidate.node.kind}`);
  }
  if (Array.isArray(candidate.issues) && candidate.issues.length > 0) {
    return failure('TASK_INPUT_INVALID', 'TASK candidate 未通过 canonical schema', { issues: candidate.issues });
  }
  if (candidate.node.lifecycle !== 'DRAFT') {
    return failure('TASK_INPUT_NOT_DRAFT', `新 TASK 必须以 DRAFT 创建，收到 ${candidate.node.lifecycle}`);
  }
  const body = bodyValid(candidate.body);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', body);
  return success({ candidate, body });
}

function idAvailable(index, id, historyExists) {
  const wanted = String(id).normalize('NFC').toLowerCase();
  const live = index && index.byId ? [...index.byId.keys()].find(
    (existing) => String(existing).normalize('NFC').toLowerCase() === wanted,
  ) : null;
  if (live) return failure('TASK_ID_ALREADY_LIVE', `编号 ${id} 已在在办平面，不能重用`);
  let inHistory = false;
  if (typeof historyExists === 'function') inHistory = historyExists(id) === true;
  else if (historyExists instanceof Set) inHistory = historyExists.has(id);
  else if (historyExists && typeof historyExists === 'object') inHistory = historyExists[id] === true;
  else inHistory = historyExists === true;
  if (inHistory) return failure('TASK_ID_ALREADY_HISTORY', `编号 ${id} 已在固定 integration history，不能重用`);
  return success();
}

function changeFor(record, node) {
  return { node, body: record.body, previousPath: record.path };
}

function terminalNode(node, result, closedAt, facts) {
  const source = facts && typeof facts === 'object' ? facts : {};
  const binding = node.git && typeof node.git === 'object' ? node.git : null;
  const git = binding
    ? {
      ...binding,
      'product-commit': source.productCommit === undefined ? binding['product-commit'] : source.productCommit,
      'wip-commit': source.wipCommit === undefined ? binding['wip-commit'] : source.wipCommit,
      'no-product-change': source.noProductChange === undefined
        ? binding['no-product-change'] === true
        : source.noProductChange === true,
      disposition: source.disposition === undefined ? binding.disposition : source.disposition,
    }
    : null;
  return {
    ...node,
    ...(git ? { git } : {}),
    // 终态 source 不再保留任何正向边。split/replace 的可回查关系只存在
    // successor 上（SPLIT_FROM / REPLACES），避免 HISTORY 继续宣称依赖。
    relations: [],
    lifecycle: 'HISTORY',
    result,
    'closed-at': closedAt,
  };
}

function sourceResourceBinding(old, facts) {
  const binding = old && old.git && typeof old.git === 'object' ? old.git : {};
  return {
    ...binding,
    'product-commit': facts && facts.productCommit !== undefined ? facts.productCommit : binding['product-commit'],
    'wip-commit': facts && facts.wipCommit !== undefined ? facts.wipCommit : binding['wip-commit'],
  };
}

function sharesSourceResource(binding, source) {
  if (!binding || typeof binding !== 'object') return false;
  return TRANSFER_RESOURCE_KEYS.some((key) => {
    const value = source[key];
    return value !== null && value !== undefined && value !== '' && binding[key] === value;
  });
}

function fullyReceivesSource(binding, source) {
  if (!binding || typeof binding !== 'object' || binding.disposition !== 'TRANSFERRED') return false;
  return TRANSFER_RESOURCE_KEYS.every((key) => {
    const value = source[key];
    return value === null || value === undefined || value === '' || binding[key] === value;
  });
}

function explicitDisposition(raw, old, candidates, facts) {
  const source = sourceResourceBinding(old, facts);
  const disposition = raw.disposition || source.disposition || null;
  if (!RETAIN_OR_TRANSFER.includes(disposition)) {
    return failure('TASK_DISPOSITION_REQUIRED', 'split/replace 必须明确把旧 branch/worktree 标为 RETAINED 或 TRANSFERRED');
  }
  if (!old.git || typeof old.git !== 'object') {
    return failure('TASK_DISPOSITION_REQUIRED', 'split/replace 的来源 TASK 缺 git binding，无法诚实处置 branch/worktree');
  }
  const overlapping = (candidates || []).filter((candidate) => sharesSourceResource(candidate && candidate.node && candidate.node.git, source));
  if (disposition === 'RETAINED' && overlapping.length > 0) {
    return failure(
      'TASK_TRANSFER_NOT_TRACEABLE',
      'RETAINED 表示旧任务继续持有 branch/worktree/product/WIP；successor 不得共享其中任一资源',
      { source, overlapping: overlapping.map((entry) => entry.node.id) },
    );
  }
  if (disposition === 'TRANSFERRED') {
    const receivers = (candidates || []).filter((candidate) => fullyReceivesSource(candidate && candidate.node && candidate.node.git, source));
    if (receivers.length !== 1 || overlapping.length !== 1 || overlapping[0] !== receivers[0]) {
      return failure(
        'TASK_TRANSFER_NOT_TRACEABLE',
        'TRANSFERRED 必须且只能有一个 successor 以 TRANSFERRED 处置承接全部 branch/worktree/product/WIP；不得分散或重复归属',
        {
          source,
          receivers: receivers.map((entry) => entry.node.id),
          overlapping: overlapping.map((entry) => entry.node.id),
        },
      );
    }
  }
  return success({ disposition, source });
}

function validateParent(index, node) {
  const allowed = graph.canAddChild(index, node['parent-id'], 'TASK');
  if (!allowed.allowed) {
    return failure('TASK_PARENT_NOT_ALLOWED', `TASK ${node.id} 的 parent 当前不能接 child：${allowed.reason}`, allowed);
  }
  return success({ parentRecord: index.byId.get(node['parent-id']) });
}

function create(input) {
  const raw = input || {};
  const checked = candidateValid(raw.candidate);
  if (!checked.ok) return checked;
  const node = checked.candidate.node;
  const available = idAvailable(raw.index, node.id, raw.historyExists);
  if (!available.ok) return available;
  const parent = validateParent(raw.index, node);
  if (!parent.ok) return parent;
  const forbidden = (node.relations || []).filter((relation) => relation
    && (relation.type === 'SPLIT_FROM' || relation.type === 'REPLACES'));
  if (forbidden.length > 0) {
    return failure('TASK_CREATE_RELATION_FORBIDDEN', '普通 create 不得预造 SPLIT_FROM 或 REPLACES', { relations: forbidden });
  }
  const copied = prepare.findNonTrivialBodyDuplicate(checked.candidate.body, parent.parentRecord.body);
  if (copied) return failure('TASK_BODY_COPIES_PARENT', 'candidate 复制了直接 parent 的长段正文', copied);
  return success({
    action: 'create', id: node.id, from: null, to: 'DRAFT',
    changes: [{ node, body: checked.candidate.body, previousPath: null }],
  });
}

function ready(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (node.lifecycle !== 'DRAFT') {
    return failure('TASK_ACTION_SOURCE_MISMATCH', `ready 只接收 DRAFT，${node.id} 当前是 ${node.lifecycle}`);
  }
  const body = bodyValid(found.record.body);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'DRAFT TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', body);
  const complete = Array.isArray(node['write-scope']) && node['write-scope'].length > 0
    && Array.isArray(node['acceptance-criteria']) && node['acceptance-criteria'].length > 0
    && hasRequiredVerification(node);
  if (!complete) {
    return failure('TASK_READY_CONTRACT_INCOMPLETE', 'ready 需要合同、write-scope、acceptance-criteria 与至少一条 required verification 齐备');
  }
  const step = lifecycle.transitionAllowed('TASK', 'DRAFT', 'READY');
  if (!step.allowed) return failure('TASK_TRANSITION_NOT_ALLOWED', step.reason, step);
  return success({
    action: 'ready', id: node.id, from: 'DRAFT', to: 'READY',
    changes: [changeFor(found.record, { ...node, lifecycle: 'READY' })],
  });
}

function recordsFrom(index, supplied) {
  if (Array.isArray(supplied)) return supplied;
  if (!index || !index.byId) return [];
  return [...index.byId.values()];
}

function admission(input, record, reason) {
  const raw = input || {};
  const reality = raw.gitReality;
  if (!reality || reality.ok !== true) {
    return failure('TASK_GIT_REALITY_CONFLICT', 'Git 现实无法与 TASK binding 对上，拒绝进入 ACTIVE', {
      conflicts: reality && Array.isArray(reality.conflicts) ? reality.conflicts : [],
    });
  }
  const node = record.node;
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const fields = {
    ...binding,
    'task-id': node.id,
    'write-scope': node['write-scope'],
    'forbidden-scope': node['forbidden-scope'],
    'exclusive-resources': node['exclusive-resources'],
  };
  const registered = opening.declarationsFromNodes(
    recordsFrom(raw.index, raw.records),
    lifecycle.participatesInScopeJudgement,
  );
  const decision = opening.evaluateOpening({
    fields,
    reason,
    registered,
    controlPlaneExempt: raw.controlPlaneExempt,
    skipGitReality: true,
    facts: reality.facts || {},
  });
  if (!decision.ok) {
    return failure('TASK_ADMISSION_REJECTED', '进入 ACTIVE 未通过 Git binding 或写面判据', {
      admission: decision.admission,
      opening: decision,
    });
  }
  return success({ admission: decision.admission, opening: decision });
}

function enterActive(input, expected, reason) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (node.lifecycle !== expected) {
    return failure('TASK_ACTION_SOURCE_MISMATCH', `${reason.toLowerCase()} 只接收 ${expected}，${node.id} 当前是 ${node.lifecycle}`);
  }
  const body = bodyValid(found.record.body);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'non-history TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', body);
  if (!hasRequiredVerification(node)) {
    return failure('TASK_REQUIRED_VERIFICATION_MISSING', '进入 ACTIVE 前必须已有至少一条冻结 required verification；不能把 closeout 自报倒灌为合同');
  }
  const step = lifecycle.transitionAllowed('TASK', expected, 'ACTIVE');
  if (!step.allowed) return failure('TASK_TRANSITION_NOT_ALLOWED', step.reason, step);
  const admitted = admission(raw, found.record, reason);
  if (!admitted.ok) return admitted;
  return success({
    action: expected === 'READY' ? 'activate' : 'resume', reason,
    id: node.id, from: expected, to: 'ACTIVE', admission: admitted.admission,
    changes: [changeFor(found.record, { ...node, lifecycle: 'ACTIVE' })],
  });
}

function activate(input) {
  return enterActive(input, 'READY', 'START');
}

function park(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (!['READY', 'ACTIVE'].includes(node.lifecycle)) {
    return failure('TASK_ACTION_SOURCE_MISMATCH', `park 只接收 READY / ACTIVE，${node.id} 当前是 ${node.lifecycle}`);
  }
  const body = bodyValid(found.record.body);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'non-history TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', body);
  // READY 尚无 binding 时可以诚实 cancel / stop，但不能 park 成一个随后无法按照
  // GIT-4 重新开工的 PARKED 节点。schema 对 PARKED 的 binding 要求在这里提前守住。
  if (node.lifecycle === 'READY' && !hasGitBinding(node)) {
    return failure('TASK_ADMISSION_REJECTED', 'READY 尚未取得完整 Git binding 时只能 cancel/stop，不能进入 PARKED');
  }
  const step = lifecycle.transitionAllowed('TASK', node.lifecycle, 'PARKED');
  if (!step.allowed) return failure('TASK_TRANSITION_NOT_ALLOWED', step.reason, step);
  return success({
    action: 'park', id: node.id, from: node.lifecycle, to: 'PARKED',
    changes: [changeFor(found.record, { ...node, lifecycle: 'PARKED' })],
  });
}

function resume(input) {
  return enterActive(input, 'PARKED', 'RESUME');
}

function withdraw(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (node.lifecycle !== 'DRAFT') {
    return failure('TASK_WITHDRAW_REQUIRES_DRAFT', `withdraw 只接收 DRAFT，${node.id} 当前是 ${node.lifecycle}`);
  }
  const body = continuationBodyValid(found.record);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'DRAFT TASK 正文必须是当前五区块，或既有旧六区块的受控 exit 兼容形态', body);
  const facts = terminalFacts(raw.index, node, raw);
  const graphBlocked = rejectTerminalGraph('TASK_FINISH_NOT_SUPPORTED', facts);
  if (graphBlocked) return graphBlocked;
  return success({
    action: 'withdraw', id: node.id, from: 'DRAFT', to: 'HISTORY', result: 'CANCELLED',
    changes: [changeFor(found.record, terminalNode(node, 'CANCELLED', raw.closedAt))],
  });
}

function terminalFacts(index, node, input) {
  const facts = input && input.facts ? input.facts : {};
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const descendants = graph.nonHistoryDescendants(index, node.id);
  const dependents = graph.dependentsOf(index, node.id);
  return {
    productCommit: facts.productCommit === undefined ? binding['product-commit'] || null : facts.productCommit,
    productCommitTouchesScope: facts.productCommitTouchesScope === undefined
      ? undefined
      : facts.productCommitTouchesScope === true,
    integratedNow: facts.integratedNow === true,
    requiredVerification: facts.requiredVerification === undefined
      ? (node.verification || []).filter((entry) => entry && entry.required === true)
      : facts.requiredVerification,
    requiredVerificationEmptyIsPass: facts.requiredVerificationEmptyIsPass === true,
    noProductChange: facts.noProductChange === undefined ? binding['no-product-change'] === true : facts.noProductChange === true,
    scopedDiffEmpty: facts.scopedDiffEmpty === true,
    wipCommit: facts.wipCommit === undefined ? binding['wip-commit'] || null : facts.wipCommit,
    disposition: facts.disposition === undefined ? binding.disposition || null : facts.disposition,
    worktreeClean: facts.worktreeClean,
    baseOidImmutable: facts.baseOidImmutable,
    taskChangesWithinScope: facts.taskChangesWithinScope,
    productCommitExists: facts.productCommitExists,
    productCommitOnTaskBranch: facts.productCommitOnTaskBranch,
    productCommitAfterBase: facts.productCommitAfterBase,
    productCommitChangesWithinScope: facts.productCommitChangesWithinScope,
    productCommitCoversTaskHead: facts.productCommitCoversTaskHead,
    integrationRefOnBase: facts.integrationRefOnBase,
    wipCommitExists: facts.wipCommitExists,
    wipCommitOnTaskBranch: facts.wipCommitOnTaskBranch,
    wipCommitAfterBase: facts.wipCommitAfterBase,
    wipCommitChangesWithinScope: facts.wipCommitChangesWithinScope,
    wipCommitCoversTaskHead: facts.wipCommitCoversTaskHead,
    hasNonHistoryDescendant: descendants.length > 0,
    hasNonHistoryDependent: dependents.length > 0,
    nonHistoryDescendants: descendants,
    nonHistoryDependents: dependents,
    splitSuccessors: facts.splitSuccessors || graph.reverseRelationsOf(index, node.id).SPLIT_INTO,
    replaceSuccessors: facts.replaceSuccessors || graph.reverseRelationsOf(index, node.id).SUPERSEDED_BY,
  };
}

function rejectTerminalGraph(code, facts) {
  const descendants = Array.isArray(facts && facts.nonHistoryDescendants) ? facts.nonHistoryDescendants : [];
  const dependents = Array.isArray(facts && facts.nonHistoryDependents) ? facts.nonHistoryDependents : [];
  if (descendants.length === 0 && dependents.length === 0) return null;
  return failure(
    code,
    `仍有在办后代或依赖者，不能进入 HISTORY：后代 ${descendants.join(', ') || '无'}；依赖者 ${dependents.join(', ') || '无'}`,
    { descendants, dependents },
  );
}

function finish(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (node.lifecycle === 'DRAFT') {
    return failure('TASK_DRAFT_MUST_WITHDRAW', `DRAFT TASK ${node.id} 请用 withdraw，不用 finish`);
  }
  if (!['READY', 'ACTIVE', 'PARKED'].includes(node.lifecycle)) {
    return failure('TASK_ACTION_SOURCE_MISMATCH', `finish 只接收 READY / ACTIVE / PARKED，${node.id} 当前是 ${node.lifecycle}`);
  }
  const body = continuationBodyValid(found.record);
  if (!body.ok) return failure('TASK_BODY_INVALID', 'non-history TASK 正文必须是当前五区块，或既有旧六区块的受控 exit 兼容形态', body);
  const result = raw.result;
  if (!TERMINAL_ACTION_RESULTS.includes(result)) {
    return failure('TASK_FINISH_RESULT_INVALID', `finish 只接收 ${TERMINAL_ACTION_RESULTS.join(' / ')}；split/replace 必须走专属动作`);
  }
  const facts = terminalFacts(raw.index, node, raw);
  const allowedDispositions = raw.allowRetiringDisposition === true
    ? FINISH_DISPOSITIONS
    : RETAIN_OR_TRANSFER;
  if (facts.disposition !== undefined && facts.disposition !== null
    && !allowedDispositions.includes(facts.disposition)) {
    return failure(
      'TASK_FINISH_NOT_SUPPORTED',
      `finish 的资源 disposition 必须是 ${allowedDispositions.join(' / ')}`,
      facts,
    );
  }
  const graphBlocked = rejectTerminalGraph('TASK_FINISH_NOT_SUPPORTED', facts);
  if (graphBlocked) return graphBlocked;
  if (facts.noProductChange === true && facts.wipCommit) {
    return failure('TASK_NO_PRODUCT_CHANGE_WITH_WIP', '存在 WIP 时不得用 NO_PRODUCT_CHANGE 掩盖；必须 RETAIN 或 TRANSFER', facts);
  }
  if (facts.noProductChange === true && facts.productCommit) {
    return failure('TASK_FINISH_NOT_SUPPORTED', '存在 product commit 时不得把任务写成 NO_PRODUCT_CHANGE', facts);
  }
  if (!lifecycle.resultSupported(result, facts)) {
    return failure('TASK_FINISH_NOT_SUPPORTED', `${result} 不符合终态真值表`, facts);
  }
  return success({
    action: 'finish', id: node.id, from: node.lifecycle, to: 'HISTORY', result, facts,
    changes: [changeFor(found.record, terminalNode(node, result, raw.closedAt, facts))],
  });
}

function cancel(input) {
  return finish({ ...(input || {}), result: 'CANCELLED' });
}

function stop(input) {
  return finish({ ...(input || {}), result: 'STOPPED' });
}

function successorDuplicate(candidateBody, parentBody, sourceBody) {
  const parentCopy = prepare.findNonTrivialBodyDuplicate(candidateBody, parentBody);
  if (parentCopy) return failure('TASK_BODY_COPIES_PARENT', 'successor 复制了直接 parent 的长段正文', parentCopy);
  const sourceCopy = prepare.findNonTrivialBodyDuplicate(candidateBody, sourceBody);
  if (sourceCopy) return failure('TASK_BODY_COPIES_SOURCE', 'successor 复制了来源 TASK 的长段正文', sourceCopy);
  return success();
}

function successorsValid(raw, oldRecord, candidates, relationType, relationNote) {
  const list = Array.isArray(candidates) ? candidates : [];
  if (list.length === 0) return failure('TASK_SUCCESSOR_REQUIRED', 'split/replace 至少需要一个 canonical successor');
  const seen = new Set();
  const prepared = [];
  for (const candidate of list) {
    const checked = candidateValid(candidate);
    if (!checked.ok) return checked;
    const node = checked.candidate.node;
    const key = node.id.normalize('NFC').toLowerCase();
    if (seen.has(key)) return failure('TASK_SUCCESSOR_ID_DUPLICATE', `successor 编号重复：${node.id}`);
    seen.add(key);
    const available = idAvailable(raw.index, node.id, raw.historyExists);
    if (!available.ok) return available;
    const parent = validateParent(raw.index, node);
    if (!parent.ok) return parent;
    const unwanted = (node.relations || []).filter((relation) => relation && (
      relation.type === (relationType === 'SPLIT_FROM' ? 'REPLACES' : 'SPLIT_FROM')
      || (relation.type === relationType && relation['target-id'] !== oldRecord.node.id)
    ));
    if (unwanted.length > 0) {
      return failure(
        relationType === 'REPLACES' ? 'TASK_REPLACEMENT_REPLACES_CONFLICT' : 'TASK_SUCCESSOR_RELATION_CONFLICT',
        `successor 不能预带与 ${relationType} 冲突的关系`,
        { relations: unwanted },
      );
    }
    const copied = successorDuplicate(checked.candidate.body, parent.parentRecord.body, oldRecord.body);
    if (!copied.ok) return copied;
    const relations = (node.relations || []).filter((relation) => relation && relation.type !== relationType);
    const relation = { type: relationType, 'target-id': oldRecord.node.id };
    if (relationNote) relation.note = relationNote;
    prepared.push({
      node: { ...node, relations: [...relations, relation] },
      body: checked.candidate.body,
      previousPath: null,
    });
  }
  return success({ successors: prepared });
}

// TRANSFERRED 的 split/replace 正是为「原合同已装不下既有改动」准备的出口：
// 只要来源与全部承接方的 write-scope 合并后，base..HEAD 的真实改动按同一套
// scope 语义全部有人认领（前缀覆盖、不触禁区），在办图就没有未回填的越界
// 工作。任一一条路径无人认领或落入禁区时仍然 fail closed。
function transferredScopeAccountsForChanges(rawFacts, taskChangesWithinScope, old, successors, disposition) {
  if (disposition !== 'TRANSFERRED' || taskChangesWithinScope !== false) return taskChangesWithinScope;
  const changedPaths = rawFacts && Array.isArray(rawFacts.changedPaths) ? rawFacts.changedPaths : null;
  if (!changedPaths) return false;
  const unionWriteScope = [];
  const claimants = [old && old['write-scope'], ...successors.map((entry) => entry.node && entry.node['write-scope'])];
  for (const entries of claimants) {
    for (const entry of Array.isArray(entries) ? entries : []) {
      if (!unionWriteScope.includes(entry)) unionWriteScope.push(entry);
    }
  }
  const classified = scope.classifyOutOfScopePaths({
    changedPaths,
    declaration: {
      id: old && old.id,
      'write-scope': unionWriteScope,
      'forbidden-scope': old && Array.isArray(old['forbidden-scope']) ? old['forbidden-scope'] : [],
    },
    registered: [],
  });
  return classified.refused === false && classified.backfillable.length === 0;
}

function split(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const old = found.record.node;
  if (!['READY', 'ACTIVE', 'PARKED'].includes(old.lifecycle)) {
    return failure('TASK_SPLIT_SOURCE_INVALID', `split 只接收 READY / ACTIVE / PARKED，${old.id} 当前是 ${old.lifecycle}`);
  }
  if (!TASK_SPLIT_REASONS.includes(raw.reason)) {
    return failure('TASK_SPLIT_REASON_INVALID', `split 只接受真实合同分裂原因：${TASK_SPLIT_REASONS.join(' / ')}`);
  }
  const oldBody = bodyValid(found.record.body);
  if (!oldBody.ok) return failure('TASK_BODY_INVALID', '来源 TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', oldBody);
  const sourceFacts = terminalFacts(raw.index, old, raw);
  const graphBlocked = rejectTerminalGraph('TASK_SPLIT_NOT_SUPPORTED', sourceFacts);
  if (graphBlocked) return graphBlocked;
  const successors = successorsValid(raw, found.record, raw.candidates, 'SPLIT_FROM');
  if (!successors.ok) return successors;
  const disposition = explicitDisposition(raw, old, successors.successors, sourceFacts);
  if (!disposition.ok) return disposition;
  const facts = {
    ...sourceFacts,
    disposition: disposition.disposition,
    splitSuccessors: successors.successors.map((entry) => entry.node.id),
  };
  facts.taskChangesWithinScope = transferredScopeAccountsForChanges(
    raw.facts, sourceFacts.taskChangesWithinScope, old, successors.successors, disposition.disposition,
  );
  if (!lifecycle.resultSupported('DECOMPOSED', facts)) {
    return failure('TASK_SPLIT_NOT_SUPPORTED', 'DECOMPOSED 不符合终态真值表', facts);
  }
  const retired = terminalNode(
    { ...old, git: { ...old.git, disposition: disposition.disposition } },
    'DECOMPOSED',
    raw.closedAt,
    facts,
  );
  return success({
    action: 'split', id: old.id, from: old.lifecycle, to: 'HISTORY', result: 'DECOMPOSED',
    facts,
    successorIds: successors.successors.map((entry) => entry.node.id),
    changes: [changeFor(found.record, retired), ...successors.successors],
  });
}

function replace(input) {
  const raw = input || {};
  const found = taskRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const old = found.record.node;
  if (!['READY', 'ACTIVE', 'PARKED'].includes(old.lifecycle)) {
    return failure('TASK_REPLACE_SOURCE_INVALID', `replace 只接收 READY / ACTIVE / PARKED，${old.id} 当前是 ${old.lifecycle}`);
  }
  if (!REPLACE_REASONS.includes(raw.reason)) {
    return failure('TASK_REPLACE_NOT_SUPPORTED', 'replace 只在原合同根本错误时使用；必须明确声明 CONTRACT_INVALID');
  }
  const oldBody = bodyValid(found.record.body);
  if (!oldBody.ok) return failure('TASK_BODY_INVALID', '来源 TASK 正文必须只有五个用户区块，并保留六项信息且每项非空', oldBody);
  const sourceFacts = terminalFacts(raw.index, old, raw);
  const graphBlocked = rejectTerminalGraph('TASK_REPLACE_NOT_SUPPORTED', sourceFacts);
  if (graphBlocked) return graphBlocked;
  const successors = successorsValid(raw, found.record, raw.candidate ? [raw.candidate] : [], 'REPLACES', raw.reason);
  if (!successors.ok) return successors;
  const disposition = explicitDisposition(raw, old, successors.successors, sourceFacts);
  if (!disposition.ok) return disposition;
  const facts = {
    ...sourceFacts,
    disposition: disposition.disposition,
    replaceSuccessors: successors.successors.map((entry) => entry.node.id),
  };
  facts.taskChangesWithinScope = transferredScopeAccountsForChanges(
    raw.facts, sourceFacts.taskChangesWithinScope, old, successors.successors, disposition.disposition,
  );
  if (!lifecycle.resultSupported('SUPERSEDED', facts)) {
    return failure('TASK_REPLACE_NOT_SUPPORTED', 'SUPERSEDED 不符合终态真值表', facts);
  }
  const retired = terminalNode(
    { ...old, git: { ...old.git, disposition: disposition.disposition } },
    'SUPERSEDED',
    raw.closedAt,
    facts,
  );
  return success({
    action: 'replace', id: old.id, from: old.lifecycle, to: 'HISTORY', result: 'SUPERSEDED',
    facts,
    reason: raw.reason,
    successorId: successors.successors[0].node.id,
    changes: [changeFor(found.record, retired), successors.successors[0]],
  });
}

module.exports = {
  TASK_SPLIT_REASONS,
  REPLACE_REASONS,
  TERMINAL_ACTION_RESULTS,
  RETAIN_OR_TRANSFER,
  create,
  ready,
  activate,
  park,
  resume,
  withdraw,
  finish,
  cancel,
  stop,
  split,
  replace,
};
