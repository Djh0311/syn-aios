'use strict';

// Adaptive Harness v0.5 — 长期权威入口（AH-050-03）
//
// 需求溯源：LY-2 · KP-3 · G-25
//
// 长期权威文档（产品文档、技术文档、决策）**不是节点**：
// 它们没有 lifecycle，不属于任务与计划的四个目录中的任何一个，
// 也不随任何一个任务被搬走或改写。某根计划及其全部子节点结束之后，
// 它们仍在原位、仍被默认加载（G-25）。
//
// 本文件只声明「权威入口上有哪些位置」，不含任何写操作。

const AUTHORITY_DOC_KEYS = Object.freeze([
  'product-doc',
  'technical-doc',
  'decision-record',
  'code-map',
  'safety-boundary',
]);

// 默认可解引用的类型白名单（§7.2）。除这四类之外，引用一律只渲染四元组，
// 不打开正文——「写了一个链接」不等于要读它。
const DEREFERENCEABLE_TYPES = Object.freeze([
  'product-doc',
  'technical-doc',
  'decision-record',
  'required-verification-note',
]);

function isAuthorityDocKey(key) {
  return AUTHORITY_DOC_KEYS.includes(key);
}

function isDereferenceable(type) {
  return DEREFERENCEABLE_TYPES.includes(type);
}

/**
 * 从 AUTHORITY 入口取出长期权威文档的目标路径。
 * 只做取值，不判定这些文档处在哪一段——它们没有那个轴（LY-2）。
 */
function authorityDocTargets(authority) {
  const source = authority && typeof authority === 'object' ? authority : {};
  const targets = [];
  for (const key of AUTHORITY_DOC_KEYS) {
    const value = source[key];
    if (typeof value !== 'string') continue;
    const trimmed = value.trim();
    if (trimmed === '' || trimmed.toLowerCase() === 'none') continue;
    targets.push({ key, target: trimmed });
  }
  return targets;
}

module.exports = {
  AUTHORITY_DOC_KEYS,
  DEREFERENCEABLE_TYPES,
  isAuthorityDocKey,
  isDereferenceable,
  authorityDocTargets,
};
