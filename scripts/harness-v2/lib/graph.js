'use strict';

// Adaptive Harness v0.5 — 节点图谱与索引（AH-050-03）
//
// 需求溯源：
//   LY-1  整条谱系链可表达且允许继续往下分；叶子能沿 parent-id 回溯到根
//   LY-3  分裂 / 取代关系可查
//   LY-5  非 history 节点的每一层 ancestor 都仍在 non-history
//   EX-3  子节点是否都处理完 / 还有没有别人依赖它
//   EX-7  已完成的子叶子先进历史，父计划留在 current 并且还能继续挂新的子叶子
//   §12 #11 cycle、orphan、duplicate ID、非法 parent kind 全部拒绝
//   §12 #13 index 删掉能从 canonical 节点重建
//
// 本文件只读：从 canonical 节点重建索引，不写任何文件。

const nodeSchema = require('./node-schema');

const NON_HISTORY_LIFECYCLES = Object.freeze(['DRAFT', 'READY', 'ACTIVE', 'PARKED']);

function isNonHistory(node) {
  return Boolean(node) && node.lifecycle !== 'HISTORY';
}

// 正文与小节必须原样带进索引。
// 索引记录是改写落盘时唯一的正文来源：relocate / close 拿 index.byId.get(id) 取到的
// 那份 record 直接喂给 serializeNode(node, body)。这里少带一个 body，
// 换目录就等于把正文清空——一次 park 抹掉整份记录，close 更会把空正文写进历史平面
// 并就地冻结，再也取不回来。LY-4 要的正是「已完成的事实不被后写内容覆盖」。
function normalizeRecord(record) {
  if (!record) return null;
  if (record.node && typeof record.node === 'object') {
    return {
      node: record.node,
      area: record.area || null,
      path: record.path || null,
      title: record.title || null,
      body: typeof record.body === 'string' ? record.body : null,
      sections: record.sections || null,
    };
  }
  if (typeof record === 'object' && typeof record.id === 'string') {
    return { node: record, area: null, path: null, title: null, body: null, sections: null };
  }
  return null;
}

/**
 * 从 canonical 节点重建索引。索引本身不是权威——删掉之后调用本函数即可重建（§12 #13）。
 */
function buildGraphIndex(records) {
  const list = Array.isArray(records) ? records : [];
  const byId = new Map();
  const duplicates = [];
  const entries = [];
  for (const raw of list) {
    const record = normalizeRecord(raw);
    if (!record || !record.node || typeof record.node.id !== 'string') continue;
    const id = record.node.id;
    if (byId.has(id)) {
      duplicates.push({ id, paths: [byId.get(id).path, record.path] });
      continue;
    }
    byId.set(id, record);
    entries.push(record);
  }

  const children = new Map();
  for (const record of entries) {
    const parentId = record.node['parent-id'];
    if (typeof parentId !== 'string' || parentId.trim() === '') continue;
    if (!children.has(parentId)) children.set(parentId, []);
    children.get(parentId).push(record.node.id);
  }
  for (const bucket of children.values()) bucket.sort();

  const roots = entries
    .filter((record) => {
      const parentId = record.node['parent-id'];
      return typeof parentId !== 'string' || parentId.trim() === '';
    })
    .map((record) => record.node.id)
    .sort();

  return { byId, children, roots, duplicates, order: entries.map((record) => record.node.id) };
}

function nodeOf(index, id) {
  const record = index.byId.get(id);
  return record ? record.node : null;
}

/**
 * 沿着「我装在谁里面」（parent-id）一路向上走，直到走到根为止。
 * 谱系链不靠任何冗余的 root 引用——存一份可推导的量就是第二个说法（EX-9 / §4.1）。
 * 返回 [根, …, 本节点]；发现环时报 cycle 而不是无限往上爬。
 */
function resolveLineage(index, id) {
  const chain = [];
  const seen = new Set();
  let cursor = id;
  while (typeof cursor === 'string' && cursor !== '') {
    if (seen.has(cursor)) return { ok: false, reason: 'GRAPH_CYCLE', chain: chain.slice().reverse(), at: cursor };
    seen.add(cursor);
    const node = nodeOf(index, cursor);
    if (!node) {
      if (chain.length === 0) return { ok: false, reason: 'NODE_NOT_FOUND', chain: [], at: cursor };
      return { ok: false, reason: 'PARENT_NOT_FOUND', chain: chain.slice().reverse(), at: cursor };
    }
    chain.push(cursor);
    const parentId = node['parent-id'];
    cursor = typeof parentId === 'string' && parentId.trim() !== '' ? parentId.trim() : null;
  }
  const lineage = chain.slice().reverse();
  return { ok: true, reason: null, chain: lineage, root: lineage[0], at: null };
}

function descendantsOf(index, id) {
  const out = [];
  const queue = [...(index.children.get(id) || [])];
  const seen = new Set();
  while (queue.length) {
    const next = queue.shift();
    if (seen.has(next)) continue;
    seen.add(next);
    out.push(next);
    queue.push(...(index.children.get(next) || []));
  }
  return out.sort();
}

/** 完整 non-history 后代集合：drafts + current + parked + pending-start 全覆盖。 */
function nonHistoryDescendants(index, id) {
  return descendantsOf(index, id).filter((childId) => isNonHistory(nodeOf(index, childId)));
}

function relationsOf(node) {
  return Array.isArray(node && node.relations) ? node.relations : [];
}

/** 反向 DEPENDS_ON：还有没有在办任务依赖本任务（EX-3 第 7 问 / 退场第 14 项）。 */
function dependentsOf(index, id) {
  const out = [];
  for (const record of index.byId.values()) {
    if (!isNonHistory(record.node)) continue;
    if (record.node.id === id) continue;
    for (const relation of relationsOf(record.node)) {
      if (relation && relation.type === 'DEPENDS_ON' && relation['target-id'] === id) {
        out.push(record.node.id);
        break;
      }
    }
  }
  return out.sort();
}

/**
 * 反向关系全部由索引生成，不可手写。
 * 新节点指向旧节点是唯一不需要动那份已冻结正文的写法（LY-4）。
 */
function reverseRelationsOf(index, id) {
  const out = { SPLIT_INTO: [], SUPERSEDED_BY: [], DEPENDED_ON_BY: [] };
  for (const record of index.byId.values()) {
    for (const relation of relationsOf(record.node)) {
      if (!relation || relation['target-id'] !== id) continue;
      if (relation.type === 'SPLIT_FROM') out.SPLIT_INTO.push(record.node.id);
      if (relation.type === 'REPLACES') out.SUPERSEDED_BY.push(record.node.id);
      if (relation.type === 'DEPENDS_ON') out.DEPENDED_ON_BY.push(record.node.id);
    }
  }
  for (const key of Object.keys(out)) out[key].sort();
  return out;
}

/**
 * EX-7 的正向侧：父节点只要还是 non-history，就留在 current，
 * 并且仍可继续 add / create 新的子叶子。已经进入 history 的子叶子
 * 不得成为父节点关闭的前置条件——只写反向（挡住关闭）而不写正向，
 * 一个「父节点永远关不掉、也不许再挂子节点」的实现照样能混过去。
 */
function canAddChild(index, parentId, childKind) {
  const parent = nodeOf(index, parentId);
  if (!parent) return { allowed: false, reason: 'PARENT_NOT_FOUND' };
  if (!isNonHistory(parent)) return { allowed: false, reason: 'PARENT_IN_HISTORY' };
  const legal = nodeSchema.LEGAL_PARENT_KINDS[childKind];
  if (!legal) return { allowed: false, reason: 'CHILD_KIND_UNKNOWN' };
  if (!legal.includes(parent.kind)) return { allowed: false, reason: 'PARENT_KIND_ILLEGAL' };
  const lineage = resolveLineage(index, parentId);
  if (!lineage.ok) return { allowed: false, reason: lineage.reason };
  for (const ancestorId of lineage.chain) {
    if (!isNonHistory(nodeOf(index, ancestorId))) {
      return { allowed: false, reason: 'ANCESTOR_IN_HISTORY', at: ancestorId };
    }
  }
  return { allowed: true, reason: null };
}

/**
 * 图谱完整性：cycle、orphan、duplicate id、非法 parent kind 全部拒绝，
 * 外加 LY-5 的「non-history 节点的每一层 ancestor 都仍在 non-history」。
 */
function graphIntegrityIssues(index) {
  const issues = [];
  for (const duplicate of index.duplicates) {
    issues.push({
      code: 'GRAPH_DUPLICATE_ID',
      id: duplicate.id,
      message: `编号 ${duplicate.id} 出现在多处：${duplicate.paths.filter(Boolean).join(' , ')}`,
    });
  }
  for (const record of index.byId.values()) {
    const node = record.node;
    const parentId = node['parent-id'];
    const hasParent = typeof parentId === 'string' && parentId.trim() !== '';
    const legal = nodeSchema.LEGAL_PARENT_KINDS[node.kind] || [];
    if (!hasParent) {
      if (node.kind !== 'ROOT_PLAN') {
        issues.push({ code: 'GRAPH_ORPHAN', id: node.id, message: `${node.kind} ${node.id} 没有 parent-id` });
      }
      continue;
    }
    if (node.kind === 'ROOT_PLAN') {
      issues.push({ code: 'GRAPH_PARENT_KIND_ILLEGAL', id: node.id, message: `ROOT_PLAN ${node.id} 不得有 parent-id` });
      continue;
    }
    const parent = nodeOf(index, parentId);
    if (!parent) {
      issues.push({ code: 'GRAPH_ORPHAN', id: node.id, message: `${node.id} 的 parent-id ${parentId} 不在册` });
      continue;
    }
    if (!legal.includes(parent.kind)) {
      issues.push({
        code: 'GRAPH_PARENT_KIND_ILLEGAL',
        id: node.id,
        message: `${node.kind} ${node.id} 不能挂在 ${parent.kind} ${parentId} 下面`,
      });
    }
    const lineage = resolveLineage(index, node.id);
    if (!lineage.ok && lineage.reason === 'GRAPH_CYCLE') {
      issues.push({ code: 'GRAPH_CYCLE', id: node.id, message: `${node.id} 的父链成环，环点 ${lineage.at}` });
      continue;
    }
    if (!lineage.ok) continue;
    if (!isNonHistory(node)) continue;
    for (const ancestorId of lineage.chain) {
      if (ancestorId === node.id) continue;
      if (!isNonHistory(nodeOf(index, ancestorId))) {
        issues.push({
          code: 'GRAPH_ANCESTOR_IN_HISTORY',
          id: node.id,
          message: `${node.id} 仍在办，但祖先 ${ancestorId} 已进入 history（LY-5）`,
        });
      }
    }
  }
  return issues;
}

/** 一行摘要四元组：关系与父链渲染只用它，装配路径从不 open target 文件（§7.2）。 */
function summaryOf(index, id) {
  const record = index.byId.get(id);
  if (!record) return { id, title: null, lifecycle: null, goal: null, known: false };
  return {
    id,
    title: record.title || null,
    lifecycle: record.node.lifecycle || null,
    goal: typeof record.node.goal === 'string' ? record.node.goal : null,
    known: true,
  };
}

module.exports = {
  NON_HISTORY_LIFECYCLES,
  isNonHistory,
  buildGraphIndex,
  nodeOf,
  resolveLineage,
  descendantsOf,
  nonHistoryDescendants,
  dependentsOf,
  reverseRelationsOf,
  canAddChild,
  graphIntegrityIssues,
  summaryOf,
};
