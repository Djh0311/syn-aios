'use strict';

// Adaptive Harness v0.5 — 阶段边界的跨节点唯一性（AH-050-03）
//
// 需求溯源：LY-7 · G-26 · §4.4
//
// 阶段边界四节（做什么 / 不做什么 / 什么算完成 / 跑哪些验证）不得在同一条祖先链重复声明。
// 判据的作用域是跨节点的祖先链：把同一份边界在子节点里再抄一遍，检查必须发现重复声明，
// 要么拒绝，要么把子节点那一节改写成指向严格祖先 id 的引用行。
// 不从相似自然语言或兄弟节点推断边界归属。
//
// 不为此新增任何字段：检查器直接读正文里的四节。

const graph = require('./graph');

const BOUNDARY_SECTION_KEYS = Object.freeze([
  '做什么',
  '不做什么',
  '什么算完成',
  '跑哪些验证',
]);

const SECTION_ALIASES = Object.freeze({
  '做什么': ['做什么', 'in scope', 'scope', 'what this stage does'],
  '不做什么': ['不做什么', 'out of scope', 'non-goals', 'not in scope'],
  '什么算完成': ['什么算完成', 'definition of done', 'done when', 'completion'],
  '跑哪些验证': ['跑哪些验证', 'verification', 'required verification', 'checks to run'],
});

const REFERENCE_LINE = /^(?:见|参见|see)\s+([^\s，,。]+)/i;
const STRICT_REFERENCE_LINE = /^(?:见|参见|see)\s+([^\s，,。]+)$/i;

function normalizeBoundaryText(value) {
  return String(value === null || value === undefined ? '' : value)
    .toLowerCase()
    .replace(/[\s　]+/g, '')
    .replace(/[.,;:!?、，。；：！？「」“”"'()（）\-—_*`#>[\]]/g, '');
}

function sectionValue(sections, key) {
  if (!sections) return '';
  const aliases = SECTION_ALIASES[key] || [key];
  const lookup = typeof sections.get === 'function'
    ? (name) => sections.get(name)
    : (name) => sections[name];
  for (const alias of aliases) {
    const direct = lookup(alias);
    if (typeof direct === 'string' && direct.trim() !== '') return direct;
  }
  const entries = typeof sections.entries === 'function'
    ? [...sections.entries()]
    : Object.entries(sections || {});
  for (const [heading, body] of entries) {
    const normalizedHeading = String(heading).toLowerCase().replace(/\s+/g, '');
    // 只认「以别名开头」，不认包含：「不做什么」包含「做什么」，
    // 用包含匹配会把两节判成同一节。
    if (aliases.some((alias) => normalizedHeading.startsWith(String(alias).toLowerCase().replace(/\s+/g, '')))) {
      return typeof body === 'string' ? body : '';
    }
  }
  return '';
}

function isReferenceLine(text) {
  const trimmed = String(text || '').trim();
  if (trimmed === '') return false;
  if (trimmed.split('\n').length > 1) return false;
  return REFERENCE_LINE.test(trimmed);
}

function referenceTarget(text) {
  const trimmed = String(text || '').trim();
  if (trimmed === '' || trimmed.split('\n').length > 1) return null;
  const match = STRICT_REFERENCE_LINE.exec(trimmed);
  return match ? match[1] : null;
}

/**
 * 跨节点唯一性检查。
 * 对每个节点的边界四节做归一化摘要，与它**祖先链上**各节点的对应摘要比对；
 * 命中重复即为违反 LY-7，除非子节点那一节已经写成指向祖先 id 的引用行。
 */
function boundaryUniquenessIssues(index, records) {
  const issues = [];
  const list = Array.isArray(records) ? records : [];
  const sectionsById = new Map();
  for (const record of list) {
    if (!record || !record.node || typeof record.node.id !== 'string') continue;
    sectionsById.set(record.node.id, record.sections || new Map());
  }

  for (const record of list) {
    if (!record || !record.node || typeof record.node.id !== 'string') continue;
    const id = record.node.id;
    const lineage = graph.resolveLineage(index, id);
    if (!lineage.ok) continue;
    const ancestors = lineage.chain.filter((ancestorId) => ancestorId !== id);
    if (ancestors.length === 0) continue;
    for (const key of BOUNDARY_SECTION_KEYS) {
      const own = sectionValue(sectionsById.get(id), key);
      if (own.trim() === '') continue;
      if (isReferenceLine(own)) continue;
      const ownDigest = normalizeBoundaryText(own);
      if (ownDigest === '') continue;
      for (const ancestorId of ancestors) {
        const ancestorText = sectionValue(sectionsById.get(ancestorId), key);
        if (normalizeBoundaryText(ancestorText) !== ownDigest) continue;
        issues.push({
          code: 'BOUNDARY_DECLARED_TWICE',
          id,
          ancestorId,
          section: key,
          message: `节点 ${id} 的「${key}」与祖先 ${ancestorId} 的同一节重复；改成指向 ${ancestorId} 的引用行`,
        });
        break;
      }
    }
  }
  return issues;
}

/**
 * `add-child` 在真正写入前构造候选 after-image，只检查新 PHASE_PLAN 相对其严格祖先的
 * 边界。这里不把 sibling 的相同短语或自然语言相似度当成违反；可审计的判据只有
 * 归一化后与祖先同节完全相同，或显式引用没有指向严格祖先。
 */
function addChildAfterImageIssues(index, candidateRecord) {
  if (!candidateRecord || !candidateRecord.node || typeof candidateRecord.node.id !== 'string') {
    return [];
  }
  const liveRecords = index && index.byId ? [...index.byId.values()] : [];
  const afterRecords = [...liveRecords, candidateRecord];
  const afterIndex = graph.buildGraphIndex(afterRecords);
  const id = candidateRecord.node.id;
  const duplicateIssues = boundaryUniquenessIssues(afterIndex, afterRecords)
    .filter((issue) => issue.id === id);
  const lineage = graph.resolveLineage(afterIndex, id);
  if (!lineage.ok) return duplicateIssues;
  const ancestorIds = new Set(lineage.chain.filter((ancestorId) => ancestorId !== id));
  const referenceIssues = [];
  for (const section of BOUNDARY_SECTION_KEYS) {
    const value = sectionValue(candidateRecord.sections, section).trim();
    if (!isReferenceLine(value)) continue;
    const target = referenceTarget(value);
    if (target && ancestorIds.has(target)) continue;
    referenceIssues.push({
      code: 'BOUNDARY_REFERENCE_NOT_STRICT_ANCESTOR',
      id,
      section,
      referenceTarget: target,
      message: `节点 ${id} 的「${section}」引用 ${target || '(无法解析)'}；只允许引用严格祖先 ${[...ancestorIds].join('、') || '(无)'}`,
    });
  }
  return [...duplicateIssues, ...referenceIssues];
}

/** 把重复的那一节强制改写成指向祖先的引用行。返回新的正文段落。 */
function referenceLineFor(ancestorId) {
  return `见 ${ancestorId}`;
}

module.exports = {
  BOUNDARY_SECTION_KEYS,
  SECTION_ALIASES,
  normalizeBoundaryText,
  sectionValue,
  isReferenceLine,
  referenceTarget,
  boundaryUniquenessIssues,
  addChildAfterImageIssues,
  referenceLineFor,
};
