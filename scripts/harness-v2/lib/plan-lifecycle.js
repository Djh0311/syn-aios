'use strict';

// Adaptive Harness v0.5 — 计划节点的显式生命周期（AH-050-04）
//
// 需求溯源：
//   LY-1  ROOT_PLAN → PHASE_PLAN → PHASE_PLAN 可以按需继续向下拆
//   LY-3  replace 新建 successor，不反复覆盖旧节点
//   LY-4  旧节点进入 HISTORY 后正文冻结；反向关系从 successor 推导
//   LY-5  子节点变化不顺带结束 parent
//   EX-3  非叶子退出前核对子孙、依赖、边界与未决问题
//   EX-5  退出现状的计划进入可按编号取回的 HISTORY
//   EX-7  已结束的 child 不阻塞仍在办的 parent；在办 child 必须阻塞 parent close
//   KP-13 写入由 CLI 显式 --write 控制；本模块只计算 change set，不做 IO

const nodeSchema = require('./node-schema');
const lifecycle = require('./lifecycle');
const graph = require('./graph');
const boundary = require('./boundary');
const exitGate = require('./exit-gate');

const PLAN_KINDS = Object.freeze(['ROOT_PLAN', 'PHASE_PLAN']);

function failure(code, error, detail) {
  return { ok: false, code, error, detail: detail || null, changes: [] };
}

function success(detail) {
  return { ok: true, code: null, error: null, ...(detail || {}) };
}

function isPlanKind(kind) {
  return PLAN_KINDS.includes(kind);
}

function planRecord(index, id) {
  const record = index && index.byId ? index.byId.get(id) : null;
  if (!record) return failure('PLAN_NOT_FOUND', `在办平面里找不到计划 ${id}`);
  if (!record.node || !isPlanKind(record.node.kind)) {
    return failure(
      'TASK_ROUTED_TO_AH_050_05',
      `${id} 是 ${record.node ? record.node.kind : '未知类型'}，TASK 生命周期归 AH-050-05`,
    );
  }
  return success({ record });
}

/**
 * candidate 是 node-schema.parseNode 的返回值。
 * 计划入口只接收 DRAFT 的 ROOT_PLAN / PHASE_PLAN；TASK 明确转交 AH-050-05。
 */
function validateCandidate(candidate) {
  if (!candidate || !candidate.node) {
    return failure('PLAN_INPUT_INVALID', '计划节点无法解析', {
      issues: candidate && Array.isArray(candidate.issues) ? candidate.issues : [],
    });
  }
  if (candidate.node.kind === 'TASK') {
    return failure('TASK_ROUTED_TO_AH_050_05', 'plan 入口不接收 TASK；TASK 生命周期归 AH-050-05');
  }
  if (!isPlanKind(candidate.node.kind)) {
    return failure('PLAN_KIND_INVALID', `plan 入口只接受 ${PLAN_KINDS.join(' / ')}`);
  }
  if (Array.isArray(candidate.issues) && candidate.issues.length > 0) {
    return failure('PLAN_INPUT_INVALID', '计划节点未通过 canonical schema', {
      issues: candidate.issues,
    });
  }
  if (candidate.node.lifecycle !== 'DRAFT') {
    return failure('PLAN_INPUT_NOT_DRAFT', `新计划必须以 DRAFT 创建，收到 ${candidate.node.lifecycle}`);
  }
  return success({ candidate });
}

function idAvailable(index, id, historyExists) {
  const wanted = String(id).normalize('NFC').toLowerCase();
  const collidingLiveId = [...index.byId.keys()].find(
    (existingId) => String(existingId).normalize('NFC').toLowerCase() === wanted,
  );
  if (collidingLiveId) {
    return failure('PLAN_ID_ALREADY_LIVE', `编号 ${id} 已在在办平面，不能重用`);
  }
  if (historyExists === true) {
    return failure('PLAN_ID_ALREADY_HISTORY', `编号 ${id} 已在历史平面，不能重用`);
  }
  return success();
}

function changeFor(record, node) {
  return {
    node,
    body: record.body,
    previousPath: record.path,
  };
}

function terminalNode(node, result, closedAt) {
  return {
    ...node,
    lifecycle: 'HISTORY',
    result,
    'closed-at': closedAt,
  };
}

function createRoot(input) {
  const raw = input || {};
  const checked = validateCandidate(raw.candidate);
  if (!checked.ok) return checked;
  const node = checked.candidate.node;
  if (node.kind !== 'ROOT_PLAN') {
    return failure('CREATE_REQUIRES_ROOT_PLAN', 'create 只创建 ROOT_PLAN；PHASE_PLAN 请用 add-child');
  }
  const available = idAvailable(raw.index, node.id, raw.historyExists);
  if (!available.ok) return available;
  return success({
    action: 'create',
    id: node.id,
    from: null,
    to: 'DRAFT',
    changes: [{ node, body: checked.candidate.body, previousPath: null }],
  });
}

function addChild(input) {
  const raw = input || {};
  const checked = validateCandidate(raw.candidate);
  if (!checked.ok) return checked;
  const node = checked.candidate.node;
  if (node.kind !== 'PHASE_PLAN') {
    return failure('ADD_CHILD_REQUIRES_PHASE_PLAN', 'add-child 只创建 PHASE_PLAN');
  }
  if (!raw.parentId) return failure('PARENT_ID_REQUIRED', 'add-child 需要 parent id');
  if (node['parent-id'] !== raw.parentId) {
    return failure(
      'PARENT_ID_MISMATCH',
      `proposal 的 parent-id 是 ${node['parent-id'] || '(空)'}，命令指定的是 ${raw.parentId}`,
    );
  }
  const available = idAvailable(raw.index, node.id, raw.historyExists);
  if (!available.ok) return available;
  const allowed = graph.canAddChild(raw.index, raw.parentId, node.kind);
  if (!allowed.allowed) {
    return failure('PLAN_CHILD_NOT_ALLOWED', `不能把 ${node.id} 挂到 ${raw.parentId}：${allowed.reason}`, allowed);
  }
  const boundaryIssues = boundary.addChildAfterImageIssues(raw.index, {
    node,
    body: checked.candidate.body,
    sections: checked.candidate.sections,
  });
  if (boundaryIssues.length > 0) {
    const first = boundaryIssues[0];
    return failure(
      first.code,
      `不能创建阶段 ${node.id}：${first.message}`,
      { issues: boundaryIssues },
    );
  }
  return success({
    action: 'add-child',
    id: node.id,
    parentId: raw.parentId,
    from: null,
    to: 'DRAFT',
    changes: [{ node, body: checked.candidate.body, previousPath: null }],
  });
}

function transition(input) {
  const raw = input || {};
  const found = planRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const from = found.record.node.lifecycle;
  const requiredFrom = {
    activate: 'DRAFT',
    park: 'ACTIVE',
    resume: 'PARKED',
  }[raw.action];
  if (!requiredFrom) {
    return failure('PLAN_ACTION_UNKNOWN', `未知计划流转动作 ${raw.action}`);
  }
  if (from !== requiredFrom) {
    return failure(
      'PLAN_ACTION_SOURCE_MISMATCH',
      `${raw.action} 只接收 ${requiredFrom} 计划，${raw.id} 当前是 ${from}`,
    );
  }
  const step = lifecycle.transitionAllowed(found.record.node.kind, from, raw.to);
  if (!step.allowed) {
    return failure(
      'PLAN_TRANSITION_NOT_ALLOWED',
      `${from} → ${raw.to} 不在计划生命周期表内（${step.reason}）`,
      step,
    );
  }
  const next = { ...found.record.node, lifecycle: raw.to };
  return success({
    action: raw.action,
    id: raw.id,
    from,
    to: raw.to,
    changes: [changeFor(found.record, next)],
  });
}

function settleGate(index, id) {
  const descendants = graph.nonHistoryDescendants(index, id);
  const dependents = graph.dependentsOf(index, id);
  if (descendants.length > 0) {
    return failure(
      'PLAN_HAS_NON_HISTORY_DESCENDANTS',
      `计划 ${id} 仍有 non-history 后代：${descendants.join('，')}`,
      { descendants, dependents },
    );
  }
  if (dependents.length > 0) {
    return failure(
      'PLAN_HAS_NON_HISTORY_DEPENDENTS',
      `计划 ${id} 仍被 non-history 节点依赖：${dependents.join('，')}`,
      { descendants, dependents },
    );
  }
  return success({ descendants, dependents });
}

function withdraw(input) {
  const raw = input || {};
  const found = planRecord(raw.index, raw.id);
  if (!found.ok) return found;
  if (found.record.node.lifecycle !== 'DRAFT') {
    return failure(
      'WITHDRAW_REQUIRES_DRAFT',
      `withdraw 只接收 DRAFT 计划，${raw.id} 当前是 ${found.record.node.lifecycle}`,
    );
  }
  const settled = settleGate(raw.index, raw.id);
  if (!settled.ok) return settled;
  const next = terminalNode(found.record.node, 'CANCELLED', raw.closedAt);
  return success({
    action: 'withdraw',
    id: raw.id,
    from: 'DRAFT',
    to: 'HISTORY',
    result: 'CANCELLED',
    changes: [changeFor(found.record, next)],
  });
}

function normalizedRelations(relations, targetId) {
  const list = Array.isArray(relations) ? relations.map((entry) => ({ ...entry })) : [];
  const conflicting = list.filter((entry) => entry
    && entry.type === 'REPLACES'
    && entry['target-id'] !== targetId);
  if (conflicting.length > 0) {
    return failure(
      'REPLACEMENT_REPLACES_CONFLICT',
      `replacement 只能 REPLACES ${targetId}，不能预带其他替代关系`,
      { conflicting },
    );
  }
  const withoutReplaces = list.filter((entry) => !entry || entry.type !== 'REPLACES');
  return success({
    relations: [...withoutReplaces, { type: 'REPLACES', 'target-id': targetId }],
  });
}

function normalizedHeading(value) {
  return String(value || '').toLowerCase().replace(/\s+/g, '');
}

function boundaryHeadingCount(body, key) {
  const aliases = boundary.SECTION_ALIASES[key] || [key];
  const normalizedAliases = aliases.map(normalizedHeading);
  return String(body || '')
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => /^##[^\S\r\n]+(.+)$/.exec(line))
    .filter(Boolean)
    .map((match) => normalizedHeading(match[1]))
    .filter((heading) => normalizedAliases.some((alias) => heading.startsWith(alias)))
    .length;
}

function boundaryResolutionProblem(record, key, index, recordsById, visited) {
  const id = record && record.node ? record.node.id : '(未知节点)';
  if (visited.has(id)) {
    return `的引用链在 ${id} 形成循环，始终没有落到真实正文`;
  }
  visited.add(id);

  const count = boundaryHeadingCount(record && record.body, key);
  if (count === 0) return `在 ${id} 中缺失`;
  if (count > 1) return `在 ${id} 中重复声明 ${count} 次`;

  const value = boundary.sectionValue(record && record.sections, key).trim();
  if (value === '') return `在 ${id} 中只有空标题，没有真实内容`;
  if (!boundary.isReferenceLine(value)) return null;

  const referenceTarget = boundary.referenceTarget(value);
  const lineage = graph.resolveLineage(index, record.node.id);
  const ancestorIds = new Set(
    lineage.ok ? lineage.chain.filter((id) => id !== record.node.id) : [],
  );
  if (!referenceTarget || !ancestorIds.has(referenceTarget)) {
    return `在 ${id} 引用的 ${referenceTarget || '(无法解析)'} 不是其真实祖先`;
  }
  const ancestor = recordsById.get(referenceTarget);
  if (!ancestor) return `引用的祖先 ${referenceTarget} 不在当前图谱记录中`;
  return boundaryResolutionProblem(ancestor, key, index, recordsById, visited);
}

function boundaryStructureGaps(record, index, records) {
  const gaps = [];
  const recordsById = new Map(
    (Array.isArray(records) ? records : [])
      .filter((entry) => entry && entry.node && typeof entry.node.id === 'string')
      .map((entry) => [entry.node.id, entry]),
  );
  for (const key of boundary.BOUNDARY_SECTION_KEYS) {
    const problem = boundaryResolutionProblem(record, key, index, recordsById, new Set());
    if (problem) gaps.push(`「${key}」${problem}`);
  }
  return gaps;
}

function replace(input) {
  const raw = input || {};
  const found = planRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const checked = validateCandidate(raw.candidate);
  if (!checked.ok) return checked;
  const replacement = checked.candidate.node;
  const old = found.record.node;
  const available = idAvailable(raw.index, replacement.id, raw.historyExists);
  if (!available.ok) return available;
  const settled = settleGate(raw.index, raw.id);
  if (!settled.ok) {
    return failure(
      'REPLACE_WOULD_ORPHAN_OR_BREAK_DEPENDENT',
      `不能替换 ${raw.id}：必须先处理它的后代和依赖`,
      settled.detail,
    );
  }
  if (replacement.kind !== old.kind) {
    return failure(
      'REPLACEMENT_KIND_MISMATCH',
      `replacement 必须保持 kind=${old.kind}，收到 ${replacement.kind}`,
    );
  }
  const oldParent = old['parent-id'] || null;
  const newParent = replacement['parent-id'] || null;
  if (oldParent !== newParent) {
    return failure(
      'REPLACEMENT_PARENT_MISMATCH',
      `replacement 必须留在同一个 parent 下：原为 ${oldParent || '(root)'}，新为 ${newParent || '(root)'}`,
    );
  }
  if (replacement.kind === 'PHASE_PLAN') {
    const parentAllowed = graph.canAddChild(raw.index, newParent, replacement.kind);
    if (!parentAllowed.allowed) {
      return failure(
        'REPLACEMENT_PARENT_NOT_AVAILABLE',
        `replacement 的 parent ${newParent} 当前不能接 child：${parentAllowed.reason}`,
        parentAllowed,
      );
    }
  }
  const replacementRelations = normalizedRelations(replacement.relations, raw.id);
  if (!replacementRelations.ok) return replacementRelations;
  const nextReplacement = {
    ...replacement,
    relations: replacementRelations.relations,
  };
  const relationValidation = nodeSchema.validateNode(nextReplacement, {
    lifecycleValues: lifecycle.PLAN_LIFECYCLE_VALUES,
  });
  if (relationValidation.issues.length > 0) {
    return failure('REPLACEMENT_INPUT_INVALID', '补入 REPLACES 后节点不合法', {
      issues: relationValidation.issues,
    });
  }
  const retired = terminalNode(old, 'SUPERSEDED', raw.closedAt);
  return success({
    action: 'replace',
    id: raw.id,
    successorId: nextReplacement.id,
    from: old.lifecycle,
    to: 'HISTORY',
    result: 'SUPERSEDED',
    changes: [
      changeFor(found.record, retired),
      { node: nextReplacement, body: checked.candidate.body, previousPath: null },
    ],
  });
}

function close(input) {
  const raw = input || {};
  const found = planRecord(raw.index, raw.id);
  if (!found.ok) return found;
  const node = found.record.node;
  if (node.lifecycle === 'DRAFT') {
    return failure('DRAFT_MUST_WITHDRAW', `DRAFT 计划 ${raw.id} 请用 withdraw，不用 close`);
  }
  const step = lifecycle.transitionAllowed(node.kind, node.lifecycle, 'HISTORY');
  if (!step.allowed) {
    return failure(
      'PLAN_TRANSITION_NOT_ALLOWED',
      `${node.lifecycle} → HISTORY 不在计划生命周期表内（${step.reason}）`,
      step,
    );
  }

  const closeout = raw.closeout || {};
  if (!Object.prototype.hasOwnProperty.call(closeout, 'result')) {
    return failure(
      'PLAN_CLOSE_RESULT_REQUIRED',
      'close 必须明确写 result；ACTIVE/PARKED 计划可选 COMPLETED、STOPPED 或 CANCELLED',
    );
  }
  const allowedResults = ['COMPLETED', 'STOPPED', 'CANCELLED'];
  if (!allowedResults.includes(closeout.result)) {
    return failure(
      'PLAN_CLOSE_RESULT_INVALID',
      `plan close 的 result 必须是 ${allowedResults.join(' / ')}，收到 ${closeout.result}`,
      { result: closeout.result, allowedResults },
    );
  }
  const descendants = graph.nonHistoryDescendants(raw.index, raw.id);
  const dependents = graph.dependentsOf(raw.index, raw.id);
  const detectedBoundaryGaps = boundaryStructureGaps(
    found.record,
    raw.index,
    raw.records || [],
  ).concat(
    boundary.boundaryUniquenessIssues(raw.index, raw.records || [])
    .filter((entry) => entry.id === raw.id)
    .map((entry) => `${entry.section} 与 ${entry.ancestorId} 重复`),
  );
  const boundaryReported = Object.prototype.hasOwnProperty.call(closeout, 'boundaryGaps')
    && Array.isArray(closeout.boundaryGaps);
  const questionsReported = Object.prototype.hasOwnProperty.call(closeout, 'openQuestions')
    && Array.isArray(closeout.openQuestions);
  // §13.2 第 3 / 4 项需要明确事实；字段缺失不能被当作「已经确认没有」。
  // 用对应谓词能识别的输入表达缺口，拒绝仍然落在固定项号内。
  const boundaryGaps = boundaryReported
    ? [...closeout.boundaryGaps, ...detectedBoundaryGaps]
    : ['closeout 未明确提供 boundaryGaps'];
  const openQuestions = questionsReported
    ? closeout.openQuestions
    : [{ id: 'CLOSEOUT_OPEN_QUESTIONS_NOT_REPORTED', status: 'UNKNOWN' }];
  const verdict = exitGate.evaluateNonLeafExit({
    nonHistoryDescendants: descendants,
    nonHistoryDependents: dependents,
    boundaryGaps,
    openQuestions,
  });
  if (!verdict.allowed) {
    return failure('PLAN_CLOSE_GATE_BLOCKED', `计划 ${raw.id} 未通过非叶子收尾门`, {
      gate: 'non-leaf',
      ...verdict,
    });
  }
  const next = terminalNode(node, closeout.result, raw.closedAt);
  return success({
    action: 'close',
    id: raw.id,
    from: node.lifecycle,
    to: 'HISTORY',
    result: closeout.result,
    gate: 'non-leaf',
    allowed: true,
    unmet: [],
    checked: verdict.checked,
    changes: [changeFor(found.record, next)],
  });
}

module.exports = {
  PLAN_KINDS,
  isPlanKind,
  validateCandidate,
  createRoot,
  addChild,
  transition,
  withdraw,
  replace,
  close,
};
