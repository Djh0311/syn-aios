#!/usr/bin/env node

// machine_face_on_ui 规则（人话工程②·2026-07-20）：UI 组件禁直渲机器格式错误串。
// 法源=交互宪法 §四.3「禁机器内部术语上脸」；规则语义与白名单条目留痕
// decisions/2026-07-20-machine-face-gate-rule-and-defer-whitelist-v1.md（不沉默豁免）。
//
// error 级（新增零容忍）：
//   jsx_error_message    — JSX 直渲 {error.message} / {this.state.error.message} 形；
//   jsx_event_stderr_pre — <pre>stderr: {…stderr}</pre> 形。
// warn-only（先观察·照 converged_helper_redefined 先例不拦）：
//   state_error_message  — `error instanceof Error ? error.message : String(error)` 进 state 形。
// 既有违规全部登记 MACHINE_FACE_DEFER_WHITELIST（`pattern|path` 形·粒度照 DEDUP 先例）；
// 新违规不得塞名单（人话工程包 §六.5）。<details> 下钻 raw_snippet 合规格板不匹配以上三形,天然豁免。

const MACHINE_FACE_ERROR_PATTERNS = [
  { id: 'jsx_error_message', re: /\{\s*(?:this\.state\.)?error\.message\b/ },
  { id: 'jsx_event_stderr_pre', re: /<pre>stderr:\s*\{[^}]*\.stderr\s*\}<\/pre>/ }
];
const MACHINE_FACE_WARN_PATTERNS = [
  { id: 'state_error_message', re: /\berror instanceof Error \? error\.message : String\(error\)/ }
];

const MACHINE_FACE_DEFER_WHITELIST = new Set([
  // 启动失败屏 <code>{this.state.error.message}</code>（main.tsx:46·随③清单另包治平）
  'jsx_error_message|prototypes/productized-desktop-shell/src/main.tsx',
  // 转录详情 <pre>stderr: {event.stderr}</pre>（TranscriptViews.tsx:428/:603·另包治理）
  'jsx_event_stderr_pre|prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx',
  // warn 档既有观察件（人话工程包 §2.4 点名·先观察不拦）
  'state_error_message|prototypes/productized-desktop-shell/src/views/AuditLedgerView.tsx',
  'state_error_message|prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx',
  'state_error_message|prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx',
  // §2.4「等」覆盖的既有 warn 档观察件（HEAD 既有·勘察漏列·2026-07-20 执行线补登并留痕）
  'state_error_message|prototypes/productized-desktop-shell/src/views/WorkflowCommandConsoleView.tsx'
]);

function scanMachineFace(root, helpers) {
  const { walkFiles, rel, readText } = helpers;
  const path = require('path');
  const scanRoot = path.join(root, 'prototypes/productized-desktop-shell', 'src');
  const violations = [];
  const warnings = [];
  const deferred = [];
  for (const filePath of walkFiles(scanRoot)) {
    const ext = path.extname(filePath);
    if (ext !== '.ts' && ext !== '.tsx') continue;
    const relative = rel(root, filePath);
    const lines = readText(filePath).split('\n');
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      for (const { id, re } of MACHINE_FACE_ERROR_PATTERNS) {
        if (ext === '.tsx' && re.test(line)) {
          const hit = { pattern: id, file: relative, line: i + 1 };
          if (MACHINE_FACE_DEFER_WHITELIST.has(`${id}|${relative}`)) deferred.push(hit);
          else violations.push(hit);
        }
      }
      for (const { id, re } of MACHINE_FACE_WARN_PATTERNS) {
        if (re.test(line)) {
          const hit = { pattern: id, file: relative, line: i + 1 };
          if (MACHINE_FACE_DEFER_WHITELIST.has(`${id}|${relative}`)) deferred.push(hit);
          else warnings.push(hit);
        }
      }
    }
  }
  violations.sort((a, b) => `${a.file}:${a.line}`.localeCompare(`${b.file}:${b.line}`));
  warnings.sort((a, b) => `${a.file}:${a.line}`.localeCompare(`${b.file}:${b.line}`));
  return { violations, warnings, deferred };
}

// 挂载进 gate report：写 metrics.machine_face + 追加 findings（error=machine_face_on_ui / warn=machine_face_state_hint）。
function attachMachineFace(report, root, helpers, addFinding) {
  const result = scanMachineFace(root, helpers);
  report.metrics.machine_face = result;
  for (const hit of result.violations) {
    addFinding(report, 'error', 'machine_face_on_ui', 'UI 组件直渲机器格式错误串（人话工程②新增零容忍）；改走 src/lib/humanize.ts 人话层或登记豁免（不许沉默）。', hit);
  }
  for (const hit of result.warnings) {
    addFinding(report, 'warn', 'machine_face_state_hint', '原始 error.message 进 state（warn-only 先观察·照 dedup 先例不拦）；后续进人话层或登记豁免。', hit);
  }
}

module.exports = { MACHINE_FACE_DEFER_WHITELIST, scanMachineFace, attachMachineFace };
