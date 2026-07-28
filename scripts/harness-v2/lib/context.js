'use strict';

// Adaptive Harness v0.5 — 默认上下文装配与 CURRENT 生成（AH-050-03）
//
// 需求溯源：
//   EX-5  默认只加载当前这条线；历史再多默认上下文也不跟着变大
//   EX-6  parked 退出默认视野
//   LY-6  只有当前叶子读全，祖先只给汇总
//   KP-3  新对话开工不用翻历史就知道当前谱系、改动承载在哪、卡在哪
//   §7.2  定长装配序列 + 白名单解引用
//   §12 #18 退场后 CURRENT 只回到显式 successor、来源 parent 或 idle
//
// 本文件只读：装配与渲染都不写文件。

const authority = require('./authority');
const graph = require('./graph');
const routing = require('./routing');

// 默认装配序列是**定长**的：打开的文件数是 O(树深)，不是 O(节点数)。
// 唯一变长的是父链摘要，而摘要全部取自索引，不打开任何父节点正文。
const ASSEMBLY_SEQUENCE = Object.freeze([
  'AGENTS',
  'AUTHORITY',
  'CURRENT',
  'ACTIVE_LEAF_FULL',
  'PARENT_STAGE_SUMMARY',
  'ROOT_PLAN_SUMMARY',
  'WHITELISTED_AUTHORITY_REFERENCE',
]);

// 默认不加载的东西。它们各自有显式命令，但都不在默认路由上。
const NOT_LOADED_BY_DEFAULT = Object.freeze([
  'parked',
  'history',
  'queue',
  'checkpoint',
  'evidence',
  'agent-summary',
  'mistake-history',
]);

/**
 * 只渲染四元组的关系类型：ID、标题、状态、一行摘要，四项全部来自索引。
 * 装配路径**从不 open target 文件**——读正文必须走显式的 history / evidence 命令。
 */
const SUMMARY_ONLY_RELATIONS = Object.freeze(['SPLIT_FROM', 'REPLACES']);

function summaryLine(item) {
  const goal = item.goal === null || item.goal === undefined ? '' : String(item.goal);
  return `${item.id} | ${item.title || '（无标题）'} | ${item.lifecycle || '（不在册）'} | ${goal}`;
}

function blockerLine(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return routing.redactKnownSecrets(String(value));
  }
  const owner = value.owner && typeof value.owner === 'object'
    ? `${value.owner.kind || 'UNKNOWN'}:${value.owner.id || 'UNKNOWN'}`
    : (value.owner || 'UNKNOWN');
  const id = value.id || '（未编号）';
  const summary = value.summary || '（无摘要）';
  return routing.redactKnownSecrets(`${id} | ${summary} | owner ${owner}`);
}

/**
 * 默认上下文装配。
 *
 * 分层对待是本函数的核心：**当前叶子读全，祖先只给汇总**——
 * 父阶段与根计划只取索引里的 {id, 标题, lifecycle, goal} 四元组，
 * 父节点正文有五千行也不影响默认上下文的规模（LY-6）。
 *
 * 返回值里带 openedFiles：那是 §5.2 不变量唯一可测的输出——
 * 固定一条当前谱系，把已结束节点从 0 增到 1000，这份清单必须逐字相同。
 */
function assembleDefaultContext(input) {
  const settings = input || {};
  const index = settings.index;
  const openedFiles = [];
  const notes = [];
  const open = (filePath) => {
    if (typeof filePath !== 'string' || filePath === '') return null;
    openedFiles.push(filePath);
    return typeof settings.readFile === 'function' ? settings.readFile(filePath) : null;
  };

  const sequence = [];

  sequence.push({ stage: 'AGENTS', path: settings.agentsPath || null, text: open(settings.agentsPath) });
  sequence.push({ stage: 'AUTHORITY', path: settings.authorityPath || null, text: open(settings.authorityPath) });
  sequence.push({ stage: 'CURRENT', path: settings.currentPath || null, text: open(settings.currentPath) });

  const leafId = settings.activeLeafId || null;
  let lineage = { ok: false, chain: [] };
  if (leafId && index) {
    lineage = graph.resolveLineage(index, leafId);
    const record = index.byId.get(leafId) || null;
    sequence.push({
      stage: 'ACTIVE_LEAF_FULL',
      id: leafId,
      path: record ? record.path : null,
      text: record ? open(record.path) : null,
    });
  } else {
    sequence.push({ stage: 'ACTIVE_LEAF_FULL', id: null, path: null, text: null });
    notes.push('idle：当前工作副本没有 ACTIVE 叶子');
  }

  // 祖先只给汇总：这里一次都不 open 父节点的正文。
  const ancestors = lineage.ok ? lineage.chain.filter((id) => id !== leafId) : [];
  const parentStageId = ancestors.length ? ancestors[ancestors.length - 1] : null;
  const rootPlanId = ancestors.length ? ancestors[0] : null;
  sequence.push({
    stage: 'PARENT_STAGE_SUMMARY',
    id: parentStageId,
    summary: parentStageId && index ? graph.summaryOf(index, parentStageId) : null,
  });
  sequence.push({
    stage: 'ROOT_PLAN_SUMMARY',
    id: rootPlanId,
    summary: rootPlanId && index ? graph.summaryOf(index, rootPlanId) : null,
  });

  // 白名单类型的当前权威引用。AUTHORITY 指向的产品 / 技术文档不是节点：
  // 它们没有 lifecycle，某个根计划全部结束之后仍在原位、仍被默认加载（LY-2 / G-25）。
  const references = [];
  for (const entry of authority.authorityDocTargets(settings.authority)) {
    if (!authority.isDereferenceable(entry.key)) {
      references.push({ key: entry.key, target: entry.target, dereferenced: false });
      continue;
    }
    references.push({ key: entry.key, target: entry.target, dereferenced: true, text: open(entry.target) });
  }
  sequence.push({ stage: 'WHITELISTED_AUTHORITY_REFERENCE', references });

  // 分裂 / 取代关系只出四元组，不解引用正文。
  const relations = [];
  if (leafId && index) {
    const record = index.byId.get(leafId);
    const declared = record && Array.isArray(record.node.relations) ? record.node.relations : [];
    for (const relation of declared) {
      if (!relation || !SUMMARY_ONLY_RELATIONS.includes(relation.type)) continue;
      relations.push({ type: relation.type, ...graph.summaryOf(index, relation['target-id']) });
    }
  }

  return {
    sequence,
    openedFiles,
    relations,
    notLoaded: NOT_LOADED_BY_DEFAULT,
    lineage: lineage.ok ? lineage.chain : [],
    notes,
  };
}

/**
 * CURRENT 是**生成视图**，不是权威。
 * 权威是节点图谱；CURRENT 只是全局事实按「我在哪个工作副本」过滤出来的投影，
 * 随时可重建，手改没有授权效力（§12 #13）。每工作副本一份。
 */
function renderCurrent(input) {
  const settings = input || {};
  const index = settings.index;
  const leafId = settings.activeLeafId || null;
  const lines = [];
  // 正常 CURRENT 以 ACTIVE leaf 为来源；退场后没有 ACTIVE leaf 时，调用方把
  // sourceTaskId 显式带进来。这样不会为了找“下一件事”扫描 READY / ACTIVE
  // 节点，也不会因为来源任务刚离开 current 就丢掉它的父链。
  const hasExitSource = typeof settings.sourceTaskId === 'string' && settings.sourceTaskId.trim() !== '';
  const sourceTaskId = hasExitSource ? settings.sourceTaskId : leafId || null;
  const exitCurrent = hasExitSource
    ? resolveExitCurrent(index, { sourceTaskId, successorId: settings.successorId })
    : {
      entry: resolveNextEntry(index, leafId),
      source: sourceSummary(index, sourceTaskId),
    };
  const sourceLineage = exitCurrent.source.lineage;
  const ancestors = sourceLineage.filter((id) => id !== sourceTaskId);
  const leaf = leafId && index ? graph.nodeOf(index, leafId) : null;

  lines.push('# CURRENT');
  lines.push('');
  lines.push(`worktree: ${settings.worktreePath || '（未指定）'}`);
  lines.push(`root plan: ${ancestors.length ? summaryLine(graph.summaryOf(index, ancestors[0])) : '（无）'}`);
  lines.push(`parent stage: ${ancestors.length ? summaryLine(graph.summaryOf(index, ancestors[ancestors.length - 1])) : '（无）'}`);
  lines.push(`active leaf: ${leafId ? summaryLine(graph.summaryOf(index, leafId)) : 'idle'}`);
  if (settings.sourceTaskId && !leafId) lines.push(`exit source: ${exitCurrent.source.id || '（无）'}`);
  lines.push(`goal: ${leaf && typeof leaf.goal === 'string' ? leaf.goal : '（无）'}`);

  const blockers = Array.isArray(settings.blockers) ? settings.blockers.slice(0, 3) : [];
  lines.push('blockers:');
  if (blockers.length === 0) lines.push('  - （无）');
  for (const blocker of blockers) lines.push(`  - ${blockerLine(blocker)}`);

  const binding = leaf && leaf.git && typeof leaf.git === 'object' ? leaf.git : {};
  lines.push(`branch: ${binding['task-branch'] || '（无）'}`);
  lines.push(`worktree binding: ${binding.worktree || '（无）'}`);
  lines.push(`base OID: ${binding['base-oid'] || '（无）'}`);

  const verification = Array.isArray(leaf && leaf.verification) ? leaf.verification : [];
  const latest = verification.filter((entry) => entry && entry.run).slice(-1)[0] || null;
  lines.push(`latest verification: ${latest ? `${latest.id} ${latest.status}` : '（尚未运行）'}`);

  // 退场入口只能是经过来源关系核对的显式 successor，或来源父阶段 / idle。
  // 绝不扫描 READY / ACTIVE 节点来猜“下一件事”（§12 #18）。
  lines.push(`next action: ${exitCurrent.entry.label}`);
  lines.push(`safety boundary: ${leaf && leaf.profile ? leaf.profile : '（无）'}`);
  lines.push('');
  lines.push('本视图由节点图谱生成，随时可重建；手改它没有授权效力。');
  return lines.join('\n');
}

function entry(kind, id) {
  return { kind, id, label: kind === 'idle' ? 'idle' : `${kind} ${id}` };
}

function sourceSummary(index, sourceTaskId) {
  const id = typeof sourceTaskId === 'string' && sourceTaskId.trim() !== '' ? sourceTaskId.trim() : null;
  if (!index || !id) return { id, lineage: [], rootId: null, parentId: null };
  const lineage = graph.resolveLineage(index, id);
  if (!lineage.ok || lineage.chain.length === 0) return { id, lineage: [], rootId: null, parentId: null };
  return {
    id,
    lineage: lineage.chain,
    rootId: lineage.chain[0],
    parentId: lineage.chain.length > 1 ? lineage.chain[lineage.chain.length - 2] : null,
  };
}

function isDeclaredSuccessor(index, sourceTaskId, successorId) {
  if (!index || !sourceTaskId || typeof successorId !== 'string' || successorId.trim() === '') return false;
  const candidate = graph.nodeOf(index, successorId.trim());
  if (!candidate || !graph.isNonHistory(candidate)) return false;
  const relations = Array.isArray(candidate.relations) ? candidate.relations : [];
  return relations.some((relation) => relation
    && (relation.type === 'SPLIT_FROM' || relation.type === 'REPLACES')
    && relation['target-id'] === sourceTaskId);
}

/**
 * 退场后的 CURRENT 来源解析。
 *
 * successor 必须由 closeout / 调用方显式给出，并且在当前图里真实存在、以
 * SPLIT_FROM 或 REPLACES 指向来源任务；只凭一个同级 READY / ACTIVE 节点
 * 绝不能被猜成下一步。若没有有效 successor，唯一的回落路径是来源 parent，
 * 再不行才是 idle。
 *
 * source.lineage 是写 CURRENT 前冻结的来源父链：写方可在把 source 移出
 * current 前用它生成 after-image，而默认 context 一次也不用读取 history。
 */
function resolveExitCurrent(index, input) {
  const settings = input || {};
  const source = sourceSummary(index, settings.sourceTaskId);
  if (source.lineage.length === 0) return { entry: entry('idle', null), source };

  if (isDeclaredSuccessor(index, source.id, settings.successorId)) {
    return { entry: entry('successor', settings.successorId.trim()), source };
  }
  if (source.parentId) return { entry: entry('parent', source.parentId), source };
  return { entry: entry('idle', null), source };
}

/**
 * 既有单参数调用保持兼容：旧的只读展示可从唯一的图谱 successor 得到入口。
 * 新的退场路径传第三参数 successorId，随后走 resolveExitCurrent 的严格核对；
 * 多个 successor 且没有显式选择时不猜，回到 parent / idle。
 */
function resolveNextEntry(index, leafId, options) {
  const hasExplicitOptions = Boolean(options && typeof options === 'object'
    && Object.prototype.hasOwnProperty.call(options, 'successorId'));
  if (hasExplicitOptions) {
    return resolveExitCurrent(index, { sourceTaskId: leafId, successorId: options.successorId }).entry;
  }

  // AH-050-03 已有调用没有携带 closeout。只兼容唯一、可回查的 successor，
  // 多个 successor 绝不按排序挑第一个。
  const reverse = index && leafId ? graph.reverseRelationsOf(index, leafId) : null;
  const candidates = reverse
    ? [...reverse.SPLIT_INTO, ...reverse.SUPERSEDED_BY]
      .filter((id) => isDeclaredSuccessor(index, leafId, id))
    : [];
  if (candidates.length === 1) return entry('successor', candidates[0]);
  return resolveExitCurrent(index, { sourceTaskId: leafId }).entry;
}

module.exports = {
  ASSEMBLY_SEQUENCE,
  NOT_LOADED_BY_DEFAULT,
  SUMMARY_ONLY_RELATIONS,
  assembleDefaultContext,
  renderCurrent,
  resolveExitCurrent,
  resolveNextEntry,
  summaryLine,
  blockerLine,
};
