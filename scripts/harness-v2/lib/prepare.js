'use strict';

// Adaptive Harness v0.5 — 普通任务的准备：复用现成材料，不重写一遍需求（AH-050-02）
//
// 需求溯源：PK-3 · DP-3 · §3.1 · §12 #3
//
// 判据是「内容是复用来的还是重新造的」：
// 一条原始请求（request）+ 当前 AUTHORITY / 上层计划 + Git 现状，
// 直接变成一份 DRAFT 节点。产品背景、技术方案和完整计划只**引用**权威文档，
// 不复制正文进任务包。
//
// 本文件只读：它产出文本，不落盘。落盘由 store 负责。

const gitFacts = require('./git-facts');
const nodeSchema = require('./node-schema');
const graph = require('./graph');

// 子任务正文固定五个 user-readable 区块；“边界”内显式保留允许读写和
// 禁止两项，因此仍完整承载 DP-3 / G-29 的六项必要信息，且不增加 Git 第六/七节。
const TASK_BODY_SECTIONS = Object.freeze([
  '负责哪块',
  '边界（允许读写、禁止）',
  '交付什么',
  '怎么验证',
  '遇到什么必须停',
]);

const TASK_BOUNDARY_FACTS = Object.freeze(['允许读写', '禁止']);

// 仅供已开工旧节点继续 record / exit 的读取兼容。新 candidate、proposal、
// start、split、replace 一律仍调用 TASK_BODY_SECTIONS 的五区块校验，不能借
// 这个常量再生产 v0.4 的六区块正文。
const LEGACY_TASK_BODY_SECTIONS = Object.freeze([
  '负责哪块',
  '允许读写什么',
  '禁止什么',
  '交付什么',
  '怎么验证',
  '遇到什么必须停',
]);

function firstLine(value) {
  return String(value === null || value === undefined ? '' : value).split('\n')[0].trim();
}

function bodyText(value) {
  return String(value === null || value === undefined ? '' : value)
    .replace(/\r\n/g, '\n')
    .split('\n')
    // 外部请求里即便出现二级标题，也只能作为本节内容；不能让 render 凭输入
    // 制造第七个 TASK 正文块。
    .map((line) => (/^##(?:[\t ]|$)/.test(line) ? `###${line.slice(2)}` : line))
    .join('\n');
}

/**
 * 从 Git 现状取开工五项里能自动取到的那部分（GIT-4）。
 * 取不到就留空并交回调用方——生成不出可用范围时让用户在开工请求内补齐，
 * 绝不默认全仓、绝不默认省略字段（§6.4「声明的默认值不得恒相交」）。
 */
function readGitReality(cwd) {
  const reality = { baseBranch: null, baseOid: null, worktree: null, available: false };
  try {
    reality.worktree = gitFacts.repoRoot(cwd);
    reality.baseOid = gitFacts.headOid(cwd);
    const branch = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd });
    reality.baseBranch = branch.ok ? branch.stdout.trim() : null;
    reality.available = Boolean(reality.worktree && reality.baseOid);
  } catch (error) {
    reality.available = false;
  }
  return reality;
}

/**
 * 把「原始请求 + 当前权威 / 上层计划 + Git 现状」直接变成一份 DRAFT 任务节点。
 * 三个来源都是现成材料，没有一处要求先重写背景、需求、证据链或评审材料。
 */
function generateTaskDraft(input) {
  const settings = input || {};
  const request = typeof settings.request === 'string' ? settings.request : '';
  const parentId = typeof settings.parentId === 'string' ? settings.parentId : null;
  const index = settings.index || null;
  const authorityEntry = settings.authority || {};
  const reality = settings.gitReality || readGitReality(settings.cwd || process.cwd());

  const parentSummary = parentId && index ? graph.summaryOf(index, parentId) : null;
  const goal = firstLine(settings.goal || request) || firstLine(parentSummary && parentSummary.goal) || '（待补齐）';

  const node = {
    id: settings.id,
    kind: 'TASK',
    'parent-id': parentId,
    lifecycle: 'DRAFT',
    goal,
    profile: settings.profile || 'ORDINARY_LOCAL',
    'write-scope': Array.isArray(settings.writeScope) ? settings.writeScope : [],
    'forbidden-scope': Array.isArray(settings.forbiddenScope) ? settings.forbiddenScope : [],
    'exclusive-resources': Array.isArray(settings.exclusiveResources) ? settings.exclusiveResources : [],
    'acceptance-criteria': Array.isArray(settings.acceptanceCriteria) ? settings.acceptanceCriteria : [],
    verification: Array.isArray(settings.verification) ? settings.verification : [],
    relations: Array.isArray(settings.relations) ? settings.relations : [],
    confirmations: [],
  };

  const missing = [];
  if (node['write-scope'].length === 0) missing.push('write-scope');
  if (!reality.available) missing.push('git-reality');

  const body = renderNodeText({
    request,
    goal,
    parentSummary,
    authority: authorityEntry,
    reality,
    writeScope: node['write-scope'],
    forbiddenScope: node['forbidden-scope'],
    acceptanceCriteria: node['acceptance-criteria'],
    verification: node.verification,
  });

  return { node, body, reality, missing, text: nodeSchema.serializeNode(node, body) };
}

/** 正文只写五个用户区块（内含六项信息），背景与方案一律写成权威引用行。 */
function renderNodeText(input) {
  const settings = input || {};
  const lines = [];
  lines.push(`# ${firstLine(settings.goal || '任务') || '任务'}`);
  lines.push('');
  lines.push('## 负责哪块');
  lines.push(settings.request ? `原始请求：${bodyText(settings.request)}` : '（原始请求缺失）');
  if (settings.parentSummary) {
    lines.push(`上层计划：${bodyText(settings.parentSummary.id)} | ${bodyText(settings.parentSummary.goal || '')}`);
  }
  for (const [key, target] of Object.entries(settings.authority || {})) {
    if (typeof target !== 'string' || target.trim() === '' || target.trim().toLowerCase() === 'none') continue;
    lines.push(`权威引用 ${bodyText(key)}：${bodyText(target)}`);
  }
  lines.push('');
  lines.push('## 边界（允许读写、禁止）');
  lines.push('### 允许读写');
  if (Array.isArray(settings.writeScope) && settings.writeScope.length > 0) {
    for (const scope of settings.writeScope) lines.push(`- ${bodyText(scope)}`);
  } else {
    lines.push('（write-scope 待在开工请求中补齐）');
  }
  lines.push('');
  lines.push('### 禁止');
  if (Array.isArray(settings.forbiddenScope) && settings.forbiddenScope.length > 0) {
    for (const scope of settings.forbiddenScope) lines.push(`- ${bodyText(scope)}`);
  } else {
    lines.push('（无额外禁区；仍受项目规则和 front matter 约束）');
  }
  lines.push('');
  lines.push('## 交付什么');
  if (Array.isArray(settings.acceptanceCriteria) && settings.acceptanceCriteria.length > 0) {
    for (const item of settings.acceptanceCriteria) lines.push(`- ${bodyText(item)}`);
  } else {
    lines.push('（acceptance-criteria 待补齐）');
  }
  lines.push('');
  lines.push('## 怎么验证');
  if (Array.isArray(settings.verification) && settings.verification.length > 0) {
    for (const entry of settings.verification) {
      lines.push(`- ${bodyText(entry.id)}: ${bodyText(entry.command)}${entry.required ? '（required）' : ''}`);
    }
  } else {
    lines.push('（verification 待补齐）');
  }
  lines.push('');
  lines.push('## 遇到什么必须停');
  lines.push('- 写面之外的路径需要改动');
  lines.push('- 声明与 Git 现实对不上');
  return lines.join('\n');
}

/**
 * non-history TASK 的正文只能有五个用户区块，而且边界区块必须显式保留
 * “允许读写 / 禁止”两项；总共仍是六项必要信息（DP-3 / G-29）。
 * 它是纯函数；调用者把拒绝放在 create / split / replace 入口，而不是靠模板碰运气。
 */
function validateTaskBodyLayout(body, expectedSections, boundaryFacts) {
  const lines = String(body || '').replace(/\r\n/g, '\n').split('\n');
  const headings = [];
  let current = null;
  let buffer = [];
  const prefix = [];
  const sections = new Map();
  const flush = () => {
    if (current !== null) sections.set(current, buffer.join('\n').trim());
  };
  for (const line of lines) {
    const match = /^##[^\S\r\n]+(.+)$/.exec(line);
    if (match) {
      flush();
      current = match[1].trim();
      headings.push(current);
      buffer = [];
      continue;
    }
    if (current !== null) buffer.push(line);
    else prefix.push(line);
  }
  flush();

  const issues = [];
  // TASK 正文可以有且只能有一个标题；标题之后、第一项之前不能藏进另一段
  // live 内容。否则复制父/来源正文的人只要把长段塞在首个 ## 前就能绕过固定
  // 区块与防复制判据（DP-3 / G-29）。
  const nonBlankPrefix = prefix.filter((line) => line.trim() !== '');
  const h1 = /^#[^#\S\r\n]+.+$/;
  if (nonBlankPrefix.length > 1 || (nonBlankPrefix.length === 1 && !h1.test(nonBlankPrefix[0]))) {
    issues.push({
      code: 'TASK_BODY_SECTION_EXTRA',
      section: '固定 TASK 正文之前的额外内容',
    });
  }
  for (const expected of expectedSections) {
    const count = headings.filter((heading) => heading === expected).length;
    if (count === 0) issues.push({ code: 'TASK_BODY_SECTION_MISSING', section: expected });
    if (count > 1) issues.push({ code: 'TASK_BODY_SECTION_DUPLICATE', section: expected, count });
    if (count === 1 && !sections.get(expected)) {
      issues.push({ code: 'TASK_BODY_SECTION_EMPTY', section: expected });
    }
  }
  for (const heading of headings) {
    if (!expectedSections.includes(heading)) {
      issues.push({ code: 'TASK_BODY_SECTION_EXTRA', section: heading });
    }
  }
  const boundary = boundaryFacts ? sections.get('边界（允许读写、禁止）') : null;
  if (boundary && Array.isArray(boundaryFacts)) {
    const boundaryHeadings = [];
    const boundarySections = new Map();
    let currentBoundary = null;
    let boundaryBuffer = [];
    const flushBoundary = () => {
      if (currentBoundary !== null) boundarySections.set(currentBoundary, boundaryBuffer.join('\n').trim());
    };
    for (const line of boundary.split('\n')) {
      const match = /^###[^\S\r\n]+(.+)$/.exec(line);
      if (match) {
        flushBoundary();
        currentBoundary = match[1].trim();
        boundaryHeadings.push(currentBoundary);
        boundaryBuffer = [];
        continue;
      }
      if (currentBoundary !== null) boundaryBuffer.push(line);
    }
    flushBoundary();
    for (const expected of boundaryFacts) {
      const count = boundaryHeadings.filter((heading) => heading === expected).length;
      if (count === 0) issues.push({ code: 'TASK_BODY_BOUNDARY_FACT_MISSING', section: expected });
      if (count > 1) issues.push({ code: 'TASK_BODY_BOUNDARY_FACT_DUPLICATE', section: expected, count });
      if (count === 1 && !boundarySections.get(expected)) {
        issues.push({ code: 'TASK_BODY_BOUNDARY_FACT_EMPTY', section: expected });
      }
    }
    for (const heading of boundaryHeadings) {
      if (!boundaryFacts.includes(heading)) {
        issues.push({ code: 'TASK_BODY_BOUNDARY_FACT_EXTRA', section: heading });
      }
    }
  }
  return { ok: issues.length === 0, headings, sections, issues };
}

/** 新 TASK 的唯一正文形态：五个用户区块，边界中包含两项事实。 */
function validateTaskBody(body) {
  return validateTaskBodyLayout(body, TASK_BODY_SECTIONS, TASK_BOUNDARY_FACTS);
}

/**
 * v0.4 已开工节点的只读识别器。它不被任何 create/start/split/replace 调用；
 * 调用方还必须自行确认节点尚未 HISTORY，才可把它用于 record 或 terminal exit。
 */
function validateLegacyTaskBody(body) {
  return validateTaskBodyLayout(body, LEGACY_TASK_BODY_SECTIONS, null);
}

function validateContinuationTaskBody(body) {
  const current = validateTaskBody(body);
  if (current.ok) return { ...current, format: 'V2_FIVE_BLOCKS' };
  const legacy = validateLegacyTaskBody(body);
  if (legacy.ok) return { ...legacy, format: 'LEGACY_SIX_BLOCKS' };
  return { ...current, format: null, legacyIssues: legacy.issues };
}

/**
 * 旧六区块正文 → 五区块正文的受控迁移。只重排结构：标题与每一节原文逐字
 * 保留，「允许读写什么」「禁止什么」并入边界区块的两个固定事实小节。
 * 迁移是 record 对既有旧节点补合同时的附带修复（AH-050-14 类过渡化石）；
 * 新 candidate、start、split、replace 仍然只生产五区块正文。
 */
function migrateLegacyTaskBody(body) {
  const parsed = validateLegacyTaskBody(body);
  if (!parsed.ok) return { ok: false, issues: parsed.issues };
  const lines = String(body || '').replace(/\r\n/g, '\n').split('\n');
  const title = lines.find((line) => line.trim() !== '') || '';
  const text = (section) => String(parsed.sections.get(section) || '').trim();
  const migrated = [
    title,
    '',
    '## 负责哪块',
    '',
    text('负责哪块'),
    '',
    '## 边界（允许读写、禁止）',
    '',
    '### 允许读写',
    '',
    text('允许读写什么'),
    '',
    '### 禁止',
    '',
    text('禁止什么'),
    '',
    '## 交付什么',
    '',
    text('交付什么'),
    '',
    '## 怎么验证',
    '',
    text('怎么验证'),
    '',
    '## 遇到什么必须停',
    '',
    text('遇到什么必须停'),
    '',
  ].join('\n');
  const checked = validateTaskBody(migrated);
  if (!checked.ok) return { ok: false, issues: checked.issues };
  return { ok: true, body: migrated };
}

function normalizedComparableText(value) {
  return String(value || '')
    .normalize('NFC')
    .toLowerCase()
    .replace(/[a-z]{1,12}-\d+(?:-\d+)?/gi, ' ')
    .replace(/(?:[a-z0-9_.-]+\/)+[a-z0-9_.-]+/gi, ' ')
    .replace(/(?:见|参见|see)\s+[^\s，,。]+/gi, ' ')
    .replace(/[^\p{L}\p{N}]+/gu, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function comparableFragments(body) {
  const parsed = validateTaskBody(body);
  const sections = parsed.sections.size > 0 ? parsed.sections : nodeSchema.extractSections(body);
  const fragments = [];
  const append = (section, original) => {
    const normalized = normalizedComparableText(original);
    if (normalized.length >= 48) fragments.push({ section, normalized, original: String(original).trim() });
  };
  for (const [section, text] of sections.entries()) {
    for (const paragraph of String(text || '').split(/\n\s*\n/)) {
      append(section, paragraph);
      // 连续 Markdown 列表没有空行时，整节会被合成一个超长 paragraph。
      // 候选只复制其中一条完整长 bullet，不能靠总长度比例把它稀释掉；
      // 缩进续行和 Markdown 的 lazy continuation 也属于同一个列表项。
      let listItem = [];
      const flushListItem = () => {
        if (listItem.length > 0) append(section, listItem.join(' '));
        listItem = [];
      };
      for (const line of paragraph.split('\n')) {
        const item = /^\s*(?:[-+*]|\d+[.)])\s+(.+)$/.exec(line);
        if (item) {
          flushListItem();
          listItem.push(item[1]);
        } else if (listItem.length > 0 && line.trim() !== '') {
          listItem.push(line.trim());
        }
      }
      flushListItem();
    }
  }
  return fragments;
}

function characterOverlapRatio(left, right) {
  const source = String(left || '').replace(/\s+/g, '');
  const target = String(right || '').replace(/\s+/g, '');
  if (!source || !target) return 0;
  const counts = new Map();
  for (const character of source) counts.set(character, (counts.get(character) || 0) + 1);
  let shared = 0;
  for (const character of target) {
    const available = counts.get(character) || 0;
    if (available <= 0) continue;
    counts.set(character, available - 1);
    shared += 1;
  }
  return shared / Math.min(source.length, target.length);
}

/**
 * 只拦真正的长段或高比例复制：标题、短标签、ID、路径和必要引用已在标准化时移除。
 * 返回原始片段，使调用方能给出可回查的拒绝理由。
 */
function findNonTrivialBodyDuplicate(candidateBody, sourceBody) {
  const candidate = comparableFragments(candidateBody);
  const source = comparableFragments(sourceBody);
  for (const left of candidate) {
    for (const right of source) {
      const shorter = Math.min(left.normalized.length, right.normalized.length);
      const longer = Math.max(left.normalized.length, right.normalized.length);
      const exact = left.normalized === right.normalized;
      const contained = left.normalized.includes(right.normalized) || right.normalized.includes(left.normalized);
      const highOverlap = shorter >= 64
        && shorter / longer >= 0.72
        && characterOverlapRatio(left.normalized, right.normalized) >= 0.9;
      if (exact || (contained && shorter >= 64 && shorter / longer >= 0.72) || highOverlap) {
        return {
          candidateSection: left.section,
          sourceSection: right.section,
          fragment: left.original.length <= right.original.length ? left.original : right.original,
        };
      }
    }
  }
  return null;
}

module.exports = {
  TASK_BODY_SECTIONS,
  TASK_BOUNDARY_FACTS,
  LEGACY_TASK_BODY_SECTIONS,
  readGitReality,
  generateTaskDraft,
  renderNodeText,
  validateTaskBody,
  validateLegacyTaskBody,
  validateContinuationTaskBody,
  migrateLegacyTaskBody,
  findNonTrivialBodyDuplicate,
};
