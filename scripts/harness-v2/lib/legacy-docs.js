'use strict';

// Adaptive Harness v0.5 — 七类旧文档能力的轻量、只读模型（AH-050-10）
//
// 需求溯源：KP-5 / KP-7 / KP-17 / G-23 / G-24 · R2 §7.2 / §7.3。
//
// 这里故意只有纯函数：它不读盘、不写盘、不启动子进程，也不持有第二份权威
// 状态。Queue、CURRENT blocker 和 Evidence Index 都是调用方交来的 canonical
// 事实的投影；checkpoint / handoff 只生成临时文本，持久化必须由其它显式流程
// 决定。

const graph = require('./graph');
const boundary = require('./boundary');

const LEGACY_CAPABILITY_IDS = Object.freeze([
  'task-queue',
  'open-questions',
  'sprint-contract',
  'context-checkpoint',
  'agent-work-summary',
  'evidence-index',
  'mistake-ledger',
]);

const LEGACY_CAPABILITY_REGISTRY = Object.freeze([
  Object.freeze({
    id: 'task-queue',
    owner: 'node graph',
    updateWhen: 'any non-history node changes lifecycle, parent, or canonical summary',
    retirePath: 'derived view only; discard and rebuild from the non-history index',
  }),
  Object.freeze({
    id: 'open-questions',
    owner: 'nearest product, technical, plan, or task owner',
    updateWhen: 'a question is opened, resolved, transferred, or changes blocker status',
    retirePath: 'resolve or transfer; fold terminal disposition into task closeout/history',
  }),
  Object.freeze({
    id: 'sprint-contract',
    owner: 'owning phase plan',
    updateWhen: 'the phase boundary changes under its explicit plan operation',
    retirePath: 'remains embedded in the canonical phase body, then freezes with that history node',
  }),
  Object.freeze({
    id: 'context-checkpoint',
    owner: 'current task or phase holder',
    updateWhen: 'only PAUSE, HANDOFF, CROSS_THREAD, CONTEXT_COMPRESSION, or CONFLICT_RECOVERY',
    retirePath: 'replace or fold its useful facts into the next handoff/closeout; never auto-generate by time',
  }),
  Object.freeze({
    id: 'agent-work-summary',
    owner: 'current task owner',
    updateWhen: 'an executor changes or a verified child result is handed back',
    retirePath: 'temporary handoff only; verified result is folded into task closeout/history, never a global log',
  }),
  Object.freeze({
    id: 'evidence-index',
    owner: 'task owner for an explicit durable evidence manifest',
    updateWhen: 'a durable manifest is explicitly admitted or retired',
    retirePath: 'derived index can be rebuilt; manifest is referenced by history closeout when retained',
  }),
  Object.freeze({
    id: 'mistake-ledger',
    owner: 'technical or project owner',
    updateWhen: 'a confirmed reusable cause has no stronger prevention carrier',
    retirePath: 'replace by regression test, checker, project rule, or technical decision; then retire the card',
  }),
]);

const CAPABILITY_REQUIRED_FIELDS = Object.freeze(['owner', 'updateWhen', 'retirePath']);
const QUESTION_OWNER_KINDS = Object.freeze([
  'PRODUCT_DOC',
  'TECHNICAL_DOC',
  'ROOT_PLAN',
  'PHASE_PLAN',
  'TASK',
]);
const QUESTION_STATUSES = Object.freeze(['OPEN', 'RESOLVED', 'TRANSFERRED']);
const STAGE_CONTRACT_SECTION_KEYS = Object.freeze([...boundary.BOUNDARY_SECTION_KEYS]);
const CHECKPOINT_EVENTS = Object.freeze([
  'PAUSE',
  'HANDOFF',
  'CROSS_THREAD',
  'CONTEXT_COMPRESSION',
  'CONFLICT_RECOVERY',
]);
const STRONGER_PREVENTION_CARRIERS = Object.freeze([
  'regressionTest',
  'checker',
  'projectRule',
  'technicalDecision',
]);
const DEFAULT_CONTEXT_EXCLUSIONS = Object.freeze([...LEGACY_CAPABILITY_IDS, 'history']);

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function isObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function failure(code, error, detail) {
  return { ok: false, code, error, detail: detail || null };
}

function capabilityCoverageReport(registry) {
  const checked = validateCapabilityRegistry(registry);
  return {
    ...checked,
    requiredIds: LEGACY_CAPABILITY_IDS.slice(),
  };
}

/** G-24：七类落点少一类、或三问任一为空，都不能算已落地。 */
function validateCapabilityRegistry(registry) {
  const list = registry === undefined ? LEGACY_CAPABILITY_REGISTRY : registry;
  const problems = [];
  const seen = new Set();
  if (!Array.isArray(list)) {
    return {
      ok: false,
      capabilities: [],
      problems: [{ code: 'CAPABILITY_REGISTRY_INVALID', message: 'capability registry 必须是数组' }],
    };
  }

  const capabilities = [];
  for (const entry of list) {
    if (!isObject(entry)) {
      problems.push({ code: 'CAPABILITY_ENTRY_INVALID', id: null, message: 'capability entry 必须是 object' });
      continue;
    }
    const id = text(entry.id);
    if (id === '') {
      problems.push({ code: 'CAPABILITY_ID_REQUIRED', id: null, message: 'capability 缺 id' });
      continue;
    }
    if (!LEGACY_CAPABILITY_IDS.includes(id)) {
      problems.push({ code: 'CAPABILITY_ID_UNKNOWN', id, message: `${id} 不在冻结的七类能力里` });
    }
    if (seen.has(id)) {
      problems.push({ code: 'CAPABILITY_ID_DUPLICATE', id, message: `${id} 重复出现` });
    }
    seen.add(id);
    for (const field of CAPABILITY_REQUIRED_FIELDS) {
      if (text(entry[field]) === '') {
        problems.push({
          code: `CAPABILITY_${field === 'updateWhen' ? 'UPDATE_WHEN' : field === 'retirePath' ? 'RETIRE_PATH' : 'OWNER'}_REQUIRED`,
          id,
          message: `${id} 缺 ${field}`,
        });
      }
    }
    capabilities.push({
      id,
      owner: text(entry.owner),
      updateWhen: text(entry.updateWhen),
      retirePath: text(entry.retirePath),
    });
  }

  for (const id of LEGACY_CAPABILITY_IDS) {
    if (!seen.has(id)) {
      problems.push({ code: 'CAPABILITY_REQUIRED_ID_MISSING', id, message: `冻结能力 ${id} 不在 registry` });
    }
  }
  return { ok: problems.length === 0, capabilities, problems };
}

function indexFrom(input) {
  if (input && input.byId && typeof input.byId.values === 'function') return input;
  if (Array.isArray(input)) return graph.buildGraphIndex(input);
  if (isObject(input) && Array.isArray(input.records)) return graph.buildGraphIndex(input.records);
  return null;
}

/** Queue 不接受手工正文：只从当前 non-history graph index 做稳定投影。 */
function deriveQueue(indexOrRecords) {
  const index = indexFrom(indexOrRecords);
  if (!index) return failure('QUEUE_INDEX_REQUIRED', 'Queue 需要 non-history graph index 或 canonical records');
  const rows = [];
  for (const record of index.byId.values()) {
    const node = record && record.node;
    if (!node || !graph.isNonHistory(node)) continue;
    rows.push({
      id: node.id,
      kind: node.kind || null,
      lifecycle: node.lifecycle || null,
      parentId: text(node['parent-id']) || null,
      title: record.title || null,
      goal: typeof node.goal === 'string' ? node.goal : null,
    });
  }
  rows.sort((left, right) => String(left.id).localeCompare(String(right.id)));
  return { ok: true, derived: true, source: 'non-history-index', rows };
}

function normalizeOwner(owner) {
  if (!isObject(owner)) return null;
  const kind = text(owner.kind).toUpperCase();
  const id = text(owner.id);
  if (id === '' || !QUESTION_OWNER_KINDS.includes(kind)) return null;
  return { kind, id };
}

function sameOwner(left, right) {
  return Boolean(left && right) && left.kind === right.kind && left.id === right.id;
}

/** 由调用方给出已解析的最近 owner；这里绝不把 question 自报的 owner 当证明。 */
function validateQuestionOwner(question, nearestOwner) {
  if (!isObject(question) || text(question.id) === '') {
    return failure('QUESTION_ID_REQUIRED', 'question 需要非空 id');
  }
  if (!QUESTION_STATUSES.includes(text(question.status))) {
    return failure('QUESTION_STATUS_INVALID', `${question.id} 的 status 必须是 ${QUESTION_STATUSES.join(' / ')}`);
  }
  if (text(question.summary) === '') {
    return failure('QUESTION_SUMMARY_REQUIRED', `${question.id} 缺 summary`);
  }
  const owner = normalizeOwner(question.owner);
  if (!owner) return failure('QUESTION_OWNER_REQUIRED', `${question.id} 缺合法 owner`);
  const nearest = normalizeOwner(nearestOwner);
  if (!nearest) return failure('QUESTION_NEAREST_OWNER_REQUIRED', `${question.id} 没有已解析的最近 owner`);
  if (!sameOwner(owner, nearest)) {
    return failure('QUESTION_OWNER_NOT_NEAREST', `${question.id} 的 owner 不是最近 owner`, { owner, nearest });
  }
  return { ok: true, question: { ...question, owner }, nearestOwner: nearest };
}

function nearestOwnerFor(question, options) {
  const settings = options || {};
  if (typeof settings.nearestOwnerFor === 'function') return settings.nearestOwnerFor(question);
  if (isObject(settings.nearestOwners)) return settings.nearestOwners[question && question.id];
  return settings.nearestOwner || null;
}

/** CURRENT 只拿已核验的、尚未解决的 blocker；完整问题清单不进入默认上下文。 */
function projectCurrentBlockers(questions, options) {
  if (!Array.isArray(questions)) return failure('QUESTIONS_ARRAY_REQUIRED', 'questions 必须是数组');
  const unresolved = [];
  const blockers = [];
  for (const question of questions) {
    const validation = validateQuestionOwner(question, nearestOwnerFor(question, options));
    if (!validation.ok) return validation;
    if (text(question.status) !== 'RESOLVED') {
      const projected = redactExternalData({
        id: text(question.id),
        status: text(question.status),
        blocker: question.blocker === true,
        summary: text(question.summary),
        owner: validation.nearestOwner,
      });
      // 非 blocker 的未决问题仍留在显式 Questions 结果里；默认 CURRENT 只消费
      // blockers 子集，因此“不丢问题”和“不把全部问题塞进默认上下文”同时成立。
      unresolved.push(projected);
      if (projected.blocker) blockers.push(projected);
    }
  }
  return {
    ok: true,
    source: 'owner-bound-questions',
    questions: unresolved,
    blockers,
  };
}

/** Sprint Contract 是 PHASE_PLAN 正文四节的投影，不创建第二个 sprint 文档。 */
function projectStageContract(record) {
  const node = record && record.node;
  if (!node || node.kind !== 'PHASE_PLAN') {
    return failure('STAGE_CONTRACT_PHASE_REQUIRED', 'Sprint Contract 只能从 PHASE_PLAN 的 canonical 正文投影');
  }
  const sections = {};
  for (const key of STAGE_CONTRACT_SECTION_KEYS) {
    const value = boundary.sectionValue(record.sections, key);
    if (text(value) === '') {
      return failure('STAGE_CONTRACT_SECTION_MISSING', `${node.id} 缺阶段合同小节「${key}」`, { id: node.id, section: key });
    }
    sections[key] = value;
  }
  return {
    ok: true,
    source: 'embedded-phase-plan',
    owner: { kind: 'PHASE_PLAN', id: node.id },
    sections,
  };
}

function validateCheckpoint(checkpoint) {
  if (!isObject(checkpoint)) return failure('CHECKPOINT_INPUT_REQUIRED', 'checkpoint 必须是 object');
  const taskId = text(checkpoint.taskId);
  const event = text(checkpoint.event).toUpperCase();
  const summary = text(checkpoint.summary);
  if (taskId === '') return failure('CHECKPOINT_TASK_REQUIRED', 'checkpoint 需要 taskId');
  if (!CHECKPOINT_EVENTS.includes(event)) {
    return failure('CHECKPOINT_EVENT_FORBIDDEN', `checkpoint event 只能是 ${CHECKPOINT_EVENTS.join(' / ')}`);
  }
  if (summary === '') return failure('CHECKPOINT_SUMMARY_REQUIRED', 'checkpoint 需要 summary');
  return {
    ok: true,
    checkpoint: {
      taskId,
      event,
      summary,
      nextEntry: text(checkpoint.nextEntry),
    },
  };
}

/** 只生成可交给显式流程的临时文本；本模块永远没有写入意图。 */
function renderCheckpoint(checkpoint) {
  const validated = validateCheckpoint(checkpoint);
  if (!validated.ok) return validated;
  const value = validated.checkpoint;
  const lines = [
    '# Context Checkpoint',
    '',
    `task: ${redactSensitiveText(value.taskId)}`,
    `event: ${value.event}`,
    `summary: ${redactSensitiveText(value.summary)}`,
    `next entry: ${redactSensitiveText(value.nextEntry || '（未声明）')}`,
  ];
  return {
    ok: true,
    action: 'CHECKPOINT',
    route: 'TEMPORARY_OUTPUT',
    temporary: true,
    written: false,
    text: lines.join('\n'),
  };
}

function validateHandoff(handoff) {
  if (!isObject(handoff)) return failure('HANDOFF_INPUT_REQUIRED', 'handoff 必须是 object');
  const required = ['taskId', 'from', 'to', 'summary'];
  const missing = required.filter((key) => text(handoff[key]) === '');
  if (missing.length) return failure('HANDOFF_FIELDS_REQUIRED', `handoff 缺 ${missing.join(' / ')}`);
  if (handoff.verifiedResults !== undefined && !Array.isArray(handoff.verifiedResults)) {
    return failure('HANDOFF_RESULTS_INVALID', 'verifiedResults 必须是数组');
  }
  return { ok: true };
}

/** Agent Work Summary 只面向一次换人；不返回日志路径、写操作或全局状态。 */
function renderHandoff(handoff) {
  const validated = validateHandoff(handoff);
  if (!validated.ok) return validated;
  const results = Array.isArray(handoff.verifiedResults) ? handoff.verifiedResults : [];
  const lines = [
    '# Temporary Handoff',
    '',
    `task: ${redactSensitiveText(text(handoff.taskId))}`,
    `from: ${redactSensitiveText(text(handoff.from))}`,
    `to: ${redactSensitiveText(text(handoff.to))}`,
    '',
    redactSensitiveText(text(handoff.summary)),
    '',
    'verified results:',
    ...(results.length === 0
      ? ['- （无）']
      : results.map((result) => `- ${redactSensitiveText(JSON.stringify(result))}`)),
  ];
  return {
    ok: true,
    action: 'HANDOFF',
    route: 'TEMPORARY_OUTPUT',
    temporary: true,
    globalLog: false,
    written: false,
    text: lines.join('\n'),
  };
}

function validEvidenceEntry(entry, offset) {
  if (!isObject(entry)) return failure('EVIDENCE_ENTRY_INVALID', `entries[${offset}] 必须是 object`);
  const fields = ['verificationId', 'command', 'headOid', 'outputRef'];
  const missing = fields.filter((field) => text(entry[field]) === '');
  if (missing.length) return failure('EVIDENCE_ENTRY_FIELDS_REQUIRED', `entries[${offset}] 缺 ${missing.join(' / ')}`);
  return { ok: true };
}

/** Evidence 是按需的；一旦要保留，就要求可说明为什么保留、以及怎么退场。 */
function validateEvidenceManifest(manifest) {
  if (manifest === null || manifest === undefined) {
    return { ok: true, admitted: false, optional: true, reason: 'NOT_REQUESTED' };
  }
  if (!isObject(manifest)) return failure('EVIDENCE_MANIFEST_INVALID', 'evidence manifest 必须是 object');
  const required = ['taskId', 'reason'];
  const missing = required.filter((key) => text(manifest[key]) === '');
  if (missing.length) return failure('EVIDENCE_MANIFEST_FIELDS_REQUIRED', `manifest 缺 ${missing.join(' / ')}`);
  if (!Array.isArray(manifest.entries) || manifest.entries.length === 0) {
    return failure('EVIDENCE_ENTRIES_REQUIRED', '持久 evidence manifest 至少要有一条 entry');
  }
  for (let offset = 0; offset < manifest.entries.length; offset += 1) {
    const entry = validEvidenceEntry(manifest.entries[offset], offset);
    if (!entry.ok) return entry;
  }
  const retirement = manifest.retirement;
  if (!isObject(retirement)
    || text(retirement.onClose) === ''
    || text(retirement.index) !== 'rebuild-from-manifest') {
    return failure(
      'EVIDENCE_RETIREMENT_REQUIRED',
      '持久 evidence manifest 必须声明 onClose 和 index=rebuild-from-manifest；index 不能成为手工权威',
    );
  }
  return {
    ok: true,
    admitted: true,
    taskId: text(manifest.taskId),
    entryCount: manifest.entries.length,
    retirement: { onClose: text(retirement.onClose), index: text(retirement.index) },
  };
}

/** Evidence Index 只是显式 manifests 的可重建投影，普通任务可以返回空索引。 */
function renderEvidenceIndex(manifests) {
  if (!Array.isArray(manifests)) return failure('EVIDENCE_MANIFESTS_ARRAY_REQUIRED', 'manifests 必须是数组');
  const rows = [];
  for (const manifest of manifests) {
    const validation = validateEvidenceManifest(manifest);
    if (!validation.ok) return validation;
    if (!validation.admitted) continue;
    for (const entry of manifest.entries) {
      rows.push(redactExternalData({
        taskId: text(manifest.taskId),
        verificationId: text(entry.verificationId),
        command: text(entry.command),
        headOid: text(entry.headOid),
        outputRef: text(entry.outputRef),
        note: text(entry.note),
      }));
    }
  }
  rows.sort((left, right) => {
    const first = `${left.taskId}:${left.verificationId}`;
    const second = `${right.taskId}:${right.verificationId}`;
    return first.localeCompare(second);
  });
  return {
    ok: true,
    action: 'EVIDENCE_INDEX',
    route: 'READ_ONLY',
    derived: true,
    written: false,
    rows,
  };
}

function carrierIsExplicitlyUnavailable(value) {
  return isObject(value) && value.applicable === false && text(value.reason) !== '';
}

/** prevention card 是最后手段：四种更强载体必须逐一证明不适用。 */
function validatePreventionCard(card) {
  if (card === null || card === undefined) {
    return { ok: true, admitted: false, optional: true, reason: 'NOT_REQUESTED' };
  }
  if (!isObject(card)) return failure('PREVENTION_CARD_INVALID', 'prevention card 必须是 object');
  const fields = ['id', 'cause', 'action', 'owner', 'updateWhen', 'retirePath'];
  const missing = fields.filter((field) => text(card[field]) === '');
  if (missing.length) return failure('PREVENTION_CARD_FIELDS_REQUIRED', `prevention card 缺 ${missing.join(' / ')}`);
  if (!isObject(card.strongerCarriers)) {
    return failure('PREVENTION_STRONGER_CARRIERS_REQUIRED', 'prevention card 必须逐项说明更强载体为何不适用');
  }
  for (const carrier of STRONGER_PREVENTION_CARRIERS) {
    const value = card.strongerCarriers[carrier];
    if (isObject(value) && value.applicable === true) {
      return failure('PREVENTION_STRONGER_CARRIER_AVAILABLE', `${carrier} 已可用时不得再保留 prevention card`, { carrier });
    }
    if (!carrierIsExplicitlyUnavailable(value)) {
      return failure('PREVENTION_STRONGER_CARRIER_UNPROVEN', `${carrier} 没有明确证明不适用`, { carrier });
    }
  }
  return { ok: true, admitted: true, id: text(card.id) };
}

function renderPreventionCard(card) {
  const validation = validatePreventionCard(card);
  if (!validation.ok || !validation.admitted) return validation;
  return {
    ok: true,
    action: 'PREVENTION_CARD',
    route: 'EXPLICIT_OUTPUT',
    written: false,
    temporary: false,
    text: [
      `# Prevention Card ${redactSensitiveText(text(card.id))}`,
      '',
      `cause: ${redactSensitiveText(text(card.cause))}`,
      `action: ${redactSensitiveText(text(card.action))}`,
      `owner: ${redactSensitiveText(text(card.owner))}`,
      `update when: ${redactSensitiveText(text(card.updateWhen))}`,
      `retire path: ${redactSensitiveText(text(card.retirePath))}`,
    ].join('\n'),
  };
}

/** history 的唯一新 helper：没有 ID 时不读；有 ID 时也只调一次精确 lookup。 */
function recoverHistoryById(input) {
  const settings = input || {};
  const id = text(settings.id);
  if (id === '') return failure('HISTORY_ID_REQUIRED', 'history 恢复必须显式提供 id');
  if (typeof settings.readById !== 'function') {
    return failure('HISTORY_READER_REQUIRED', 'history 恢复需要显式的 readById');
  }
  let record;
  try {
    record = settings.readById(id);
  } catch (error) {
    return failure('HISTORY_READ_FAILED', `history ${id} 读取失败：${error.message}`);
  }
  if (!record) return failure('HISTORY_NOT_FOUND', `history 中找不到 ${id}`);
  return { ok: true, explicit: true, id, record };
}

/**
 * G-23：外部输入只能作为数据显示。这里不解释其中的祈使句，所有输出 action / route
 * 都由各渲染函数固定；同时遮蔽常见 token、Bearer、GitHub token、AWS access key。
 */
function redactSensitiveText(value) {
  let output = String(value === null || value === undefined ? '' : value);
  output = output.replace(/\b(authorization\s*:\s*bearer\s+)([^\s,;]+)/gi, '$1[REDACTED]');
  output = output.replace(/\b((?:api[_-]?key|token|secret|password)\s*[=:]\s*)([^\s,;]+)/gi, '$1[REDACTED]');
  output = output.replace(/\b(?:sk-[A-Za-z0-9_-]{8,}|ghp_[A-Za-z0-9]{8,}|github_pat_[A-Za-z0-9_]{8,}|AKIA[0-9A-Z]{12,})\b/g, '[REDACTED]');
  return output;
}

function redactExternalData(value) {
  if (typeof value === 'string') return redactSensitiveText(value);
  if (Array.isArray(value)) return value.map((entry) => redactExternalData(entry));
  if (!isObject(value)) return value;
  const out = {};
  for (const [key, item] of Object.entries(value)) out[key] = redactExternalData(item);
  return out;
}

module.exports = {
  LEGACY_CAPABILITY_IDS,
  LEGACY_CAPABILITY_REGISTRY,
  CAPABILITY_REQUIRED_FIELDS,
  QUESTION_OWNER_KINDS,
  QUESTION_STATUSES,
  STAGE_CONTRACT_SECTION_KEYS,
  CHECKPOINT_EVENTS,
  STRONGER_PREVENTION_CARRIERS,
  DEFAULT_CONTEXT_EXCLUSIONS,
  validateCapabilityRegistry,
  capabilityCoverageReport,
  deriveQueue,
  validateQuestionOwner,
  projectCurrentBlockers,
  projectStageContract,
  validateCheckpoint,
  renderCheckpoint,
  validateHandoff,
  renderHandoff,
  validateEvidenceManifest,
  renderEvidenceIndex,
  validatePreventionCard,
  renderPreventionCard,
  recoverHistoryById,
  redactSensitiveText,
  redactExternalData,
};
