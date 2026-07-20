#!/usr/bin/env node

// hardcoded_hex_on_ui 规则（G1 token 归真·2026-07-20）：UI 样式禁新增裸 hex，一律走正典 token。
// 法源=DESIGN.md §一（零换皮拍板值）；白名单条目与规则语义留痕
// decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md（不沉默豁免·白名单只减不增）。
//
// error 级（新增零容忍）：裸 hex、var() 回退位 hex（错误回退实证）、%23 转义形。
// 豁免：正典定义行（^\s*--[\w-]+\s*:）、注释行、HARDCODED_HEX_WHITELIST（`hex值|path` 粒度）。
// 扫描面：6 个 CSS + src/** .ts/.tsx（含内联 style 与数据字面量，数据类经白名单登记）。

const HEX_RE = /#[0-9a-fA-F]{3,8}\b|%23[0-9a-fA-F]{3,6}/g;
const DEF_LINE_RE = /^\s*--[\w-]+\s*:/;

const CSS_FILES = [
  'prototypes/productized-desktop-shell/src/styles.css',
  'prototypes/productized-desktop-shell/src/manualRelay.css',
  'prototypes/productized-desktop-shell/src/components/sourceStylePlaceholder.css',
  'prototypes/productized-desktop-shell/src/views/memory/memoryCenter.css',
  'prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  'prototypes/productized-desktop-shell/src/views/projects/projectReferencePanels.css'
];

// 预登记 86 条（勘察 §6 口径：styles.css 49 + sidePanel 25 + ActiveWorkbenchView 3 +
// canvasNodeData 6 + ProjectWorkflowCanvasView 1 + WorkflowCommandConsoleView 2），
// G1 施工后实际 42 条（治平一批核销一批·只减不增）：
const HARDCODED_HEX_WHITELIST = new Set([
  // styles.css 26 = boot 诊断屏 6 + SVG 转义 2 + 无等值 token 的 live 零散值 18（b 类兜底·值未调）
  '#1a1c1a|prototypes/productized-desktop-shell/src/styles.css',
  '#f7f1e3|prototypes/productized-desktop-shell/src/styles.css',
  '#f1ead9|prototypes/productized-desktop-shell/src/styles.css',
  '#e8dfcd|prototypes/productized-desktop-shell/src/styles.css',
  '#5c3a1f|prototypes/productized-desktop-shell/src/styles.css',
  '#2a2419|prototypes/productized-desktop-shell/src/styles.css',
  '%231c1f24|prototypes/productized-desktop-shell/src/styles.css',
  '%23a14242|prototypes/productized-desktop-shell/src/styles.css',
  '#d8d3c5|prototypes/productized-desktop-shell/src/styles.css',
  '#faf8f3|prototypes/productized-desktop-shell/src/styles.css',
  '#ccc|prototypes/productized-desktop-shell/src/styles.css',
  '#8a8275|prototypes/productized-desktop-shell/src/styles.css',
  '#ddd|prototypes/productized-desktop-shell/src/styles.css',
  '#c9bfa6|prototypes/productized-desktop-shell/src/styles.css',
  '#faf7f0|prototypes/productized-desktop-shell/src/styles.css',
  '#b14422|prototypes/productized-desktop-shell/src/styles.css',
  '#c8a05a|prototypes/productized-desktop-shell/src/styles.css',
  '#8a7f6a|prototypes/productized-desktop-shell/src/styles.css',
  '#f7f1e6|prototypes/productized-desktop-shell/src/styles.css',
  '#fffdf8|prototypes/productized-desktop-shell/src/styles.css',
  '#18211f|prototypes/productized-desktop-shell/src/styles.css',
  '#edf7f1|prototypes/productized-desktop-shell/src/styles.css',
  '#cfc8b6|prototypes/productized-desktop-shell/src/styles.css',
  '#3f5235|prototypes/productized-desktop-shell/src/styles.css',
  '#7a2e2e|prototypes/productized-desktop-shell/src/styles.css',
  '#f7e8e8|prototypes/productized-desktop-shell/src/styles.css',
  // projectWorkflowSidePanel.css 9（状态色零散值·无等值 token）
  '#2e7d4f|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#4caf72|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#666|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#6b6b6b|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#a86a00|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#b0b0b0|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#b23b3b|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#d9a441|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  '#e05656|prototypes/productized-desktop-shell/src/views/projects/projectWorkflowSidePanel.css',
  // lib/canvasNodeData.ts 6（节点调色板数据·数据非样式）
  '#c8602b|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  '#5a6f4a|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  '#3a6a77|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  '#8a7f6a|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  '#b9b3a6|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  '#a14242|prototypes/productized-desktop-shell/src/lib/canvasNodeData.ts',
  // ProjectWorkflowCanvasView.tsx 1（SVG 依赖边 stroke·无等值 token）
  '#9aa0a6|prototypes/productized-desktop-shell/src/views/projects/ProjectWorkflowCanvasView.tsx'
]);

function lineIsComment(line, inComment) {
  const stripped = line.trim();
  if (inComment) return { comment: true, still: !stripped.includes('*/') };
  if (stripped.startsWith('/*')) return { comment: true, still: !stripped.includes('*/') };
  if (stripped.startsWith('//')) return { comment: true, still: false };
  return { comment: false, still: false };
}

function scanHardcodedHex(root, helpers) {
  const { walkFiles, rel, readText } = helpers;
  const path = require('path');
  const targets = [];
  for (const relCss of CSS_FILES) {
    const full = path.join(root, relCss);
    if (require('fs').existsSync(full)) targets.push(full);
  }
  for (const filePath of walkFiles(path.join(root, 'prototypes/productized-desktop-shell', 'src'))) {
    const ext = path.extname(filePath);
    if (ext === '.ts' || ext === '.tsx') targets.push(filePath);
  }
  const violations = [];
  const deferred = [];
  for (const filePath of targets) {
    const relative = rel(root, filePath);
    const lines = readText(filePath).split('\n');
    let inComment = false;
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      const state = lineIsComment(line, inComment);
      inComment = state.still;
      if (state.comment || DEF_LINE_RE.test(line)) continue;
      const matches = line.match(HEX_RE) || [];
      for (const hex of matches) {
        const hit = { hex: hex.toLowerCase(), file: relative, line: i + 1 };
        if (HARDCODED_HEX_WHITELIST.has(`${hex.toLowerCase()}|${relative}`)) deferred.push(hit);
        else violations.push(hit);
      }
    }
  }
  violations.sort((a, b) => `${a.file}:${a.line}`.localeCompare(`${b.file}:${b.line}`));
  return { violations, deferred };
}

// 挂载进 gate report：写 metrics.hardcoded_hex + 追加 findings（error=hardcoded_hex_on_ui）。
function attachHardcodedHex(report, root, helpers, addFinding) {
  const result = scanHardcodedHex(root, helpers);
  report.metrics.hardcoded_hex = result;
  for (const hit of result.violations) {
    addFinding(report, 'error', 'hardcoded_hex_on_ui', 'UI 样式新增裸 hex（G1 新增零容忍）；改走正典 token（styles.css :root）或登记白名单（只减不增·不许沉默豁免）。', hit);
  }
}

module.exports = { HARDCODED_HEX_WHITELIST, scanHardcodedHex, attachHardcodedHex };
