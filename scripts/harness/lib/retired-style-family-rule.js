#!/usr/bin/env node

// retired_style_family 规则（G2 定式扶正·2026-07-20）：退休式族禁再现 + spec-* 基座禁直连。
// 法源=DESIGN.md §三·五（2026-07-14 用户逐字段拍板：pill/事实行唯一定式）；规则语义与白名单留痕
// decisions/2026-07-20-g2-spec-primitives-restoration-and-retired-style-family-gate-rule-v1.md（不沉默豁免）。
//
// error 级（新增零容忍）：
//   退休族再现 — jiaoban-fact（不匹配 jiaoban-fact-btn/-done）/ memory-kv / settings-fact（含 -grid）/
//     badge（精确词界·不匹配 sc-badge 等复合名·badge-row 同属退休）/ jiaoban-step-badge /
//     project-canvas-status-pill / prsb-pill / jiaoban-done-pills / running-status-pill（仅 ts/tsx·
//     死视图引用面·styles.css 定义段留 G4）+ 对 components/Badge 的 import。
//   spec-* 直连 — tsx 字符串面含 spec-fact-row/-k/-v、spec-pill（含各 tone）/spec-pill-row、
//     spec-seg-title、spec-list-row/-badge/-claim/-time、spec-empty、spec-expand、spec-bad，
//     而文件 ≠ SpecPrimitives.tsx（基座外只许用组件，不许直写类名）。
// 扫描面：src/** .tsx/.ts（字符串字面量面·天然跳过注释与标识符）+ .css（原始行·跳注释行）。
// 去重粒度：pattern|path（同族同文件取首命中行，命中数进 detail.count）。

const RETIRED_FAMILY_PATTERNS = [
  { id: 'jiaoban-fact', re: /jiaoban-fact(?!-(?:btn|done))/ },
  { id: 'memory-kv', re: /memory-kv(?![\w-])/ },
  { id: 'settings-fact', re: /settings-fact/ },
  { id: 'badge', re: /(?<![\w-])badge(?:-row)?(?![\w-])/ },
  { id: 'jiaoban-step-badge', re: /jiaoban-step-badge(?![\w-])/ },
  { id: 'project-canvas-status-pill', re: /project-canvas-status-pill(?![\w-])/ },
  { id: 'prsb-pill', re: /prsb-pill(?![\w-])/ },
  { id: 'jiaoban-done-pills', re: /jiaoban-done-pills(?![\w-])/ },
  // 死视图引用面（G4 整删时连带清）：只扫 ts/tsx——styles.css:287-308 定义段本包不动、挂账 G4。
  { id: 'running-status-pill', re: /running-status-pill(?![\w-])/, tsOnly: true }
];
const BADGE_IMPORT_ID = 'badge-import';
const BADGE_IMPORT_RE = /from\s+["'][^"']*components\/Badge["']/;
const SPEC_DIRECT_RE = /spec-(?:fact-row|fact-k|fact-v|pill(?:-[\w-]+)?|seg-title|list-row|list-badge|list-claim|list-time|empty|expand|bad)(?![\w-])/g;
const STRING_LITERAL_RE = /(["'`])(?:(?!\1)[^\\]|\\.)*\1/g;
const SPEC_PRIMITIVES_PATH = 'prototypes/productized-desktop-shell/src/components/SpecPrimitives.tsx';

// 白名单 2 条（全登记 decisions·不沉默豁免）：
const RETIRED_STYLE_FAMILY_DEFER_WHITELIST = new Set([
  // ① 死视图 RunningWorkflowsView 的 running-status-pill 引用面（死视图 1196 行·G4 整删时连带清）
  'running-status-pill|prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx',
  // ② ActiveWorkbenchView.tsx:277 spec-empty 有意例外（想法箱空态文案「。」「；」混排与基座
  //   what/next 拼接逐字对不上·不改基座·该处注释在档）
  'spec-direct:spec-empty|prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx'
]);

function cssLineIsComment(line, inComment) {
  const stripped = line.trim();
  if (inComment) return { comment: true, still: !stripped.includes('*/') };
  if (stripped.startsWith('/*')) return { comment: true, still: !stripped.includes('*/') };
  return { comment: false, still: false };
}

function record(bucket, seen, whitelist, kind, relative, line) {
  const key = `${kind}|${relative}`;
  if (seen.has(key)) {
    seen.get(key).count += 1;
    return;
  }
  const hit = { pattern: kind, file: relative, line, count: 1 };
  seen.set(key, hit);
  if (whitelist.has(key)) bucket.deferred.push(hit);
  else bucket.violations.push(hit);
}

function scanRetiredStyleFamily(root, helpers) {
  const { walkFiles, rel, readText } = helpers;
  const path = require('path');
  const scanRoot = path.join(root, 'prototypes/productized-desktop-shell', 'src');
  const bucket = { violations: [], deferred: [] };
  const seen = new Map();
  for (const filePath of walkFiles(scanRoot)) {
    const ext = path.extname(filePath);
    if (ext !== '.ts' && ext !== '.tsx' && ext !== '.css') continue;
    const relative = rel(root, filePath);
    const lines = readText(filePath).split('\n');
    let inComment = false;
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      if (ext === '.css') {
        const state = cssLineIsComment(line, inComment);
        inComment = state.still;
        if (state.comment) continue;
        for (const { id, re, tsOnly } of RETIRED_FAMILY_PATTERNS) {
          if (tsOnly) continue;
          if (re.test(line)) record(bucket, seen, RETIRED_STYLE_FAMILY_DEFER_WHITELIST, id, relative, i + 1);
        }
        continue;
      }
      // ts/tsx：先查 Badge import（整行），再抽字符串字面量面查退休族 + spec-* 直连。
      if (BADGE_IMPORT_RE.test(line)) record(bucket, seen, RETIRED_STYLE_FAMILY_DEFER_WHITELIST, BADGE_IMPORT_ID, relative, i + 1);
      const literals = line.match(STRING_LITERAL_RE) || [];
      for (const literal of literals) {
        for (const { id, re } of RETIRED_FAMILY_PATTERNS) {
          if (re.test(literal)) record(bucket, seen, RETIRED_STYLE_FAMILY_DEFER_WHITELIST, id, relative, i + 1);
        }
        if (ext === '.tsx' && relative !== SPEC_PRIMITIVES_PATH) {
          const direct = literal.match(SPEC_DIRECT_RE) || [];
          for (const klass of direct) {
            // spec-pill 各 tone 归一族去重（spec-pill-ok/-warn/… 同记 spec-direct:spec-pill）。
            const kind = `spec-direct:${klass.startsWith('spec-pill-') ? 'spec-pill' : klass}`;
            record(bucket, seen, RETIRED_STYLE_FAMILY_DEFER_WHITELIST, kind, relative, i + 1);
          }
        }
      }
    }
  }
  bucket.violations.sort((a, b) => `${a.file}:${a.line}`.localeCompare(`${b.file}:${b.line}`));
  bucket.deferred.sort((a, b) => `${a.file}:${a.line}`.localeCompare(`${b.file}:${b.line}`));
  return bucket;
}

// 挂载进 gate report：写 metrics.retired_style_family + 追加 findings（error=retired_style_family）。
function attachRetiredStyleFamily(report, root, helpers, addFinding) {
  const result = scanRetiredStyleFamily(root, helpers);
  report.metrics.retired_style_family = result;
  for (const hit of result.violations) {
    addFinding(report, 'error', 'retired_style_family', '退休式族再现 / spec-* 基座直连（G2 定式扶正新增零容忍）；改回 FactRow/Pill/PillRow 等基座组件或登记豁免（不许沉默）。', hit);
  }
}

module.exports = { RETIRED_STYLE_FAMILY_DEFER_WHITELIST, scanRetiredStyleFamily, attachRetiredStyleFamily };
