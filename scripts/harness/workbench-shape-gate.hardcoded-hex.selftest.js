#!/usr/bin/env node

// Self-test for the hardcoded_hex_on_ui rule (G1·2026-07-20) in workbench-shape-gate.js.
// Builds throwaway fixture trees in os.tmpdir() and runs the gate against them.
// It never reads or writes real product code. Exit 0 = all cases pass.

const fs = require('fs');
const os = require('os');
const path = require('path');
const { execFileSync } = require('child_process');

const GATE = path.join(__dirname, 'workbench-shape-gate.js');
const SRC_REL = 'prototypes/productized-desktop-shell/src';

function runGate(target) {
  let stdout;
  try {
    stdout = execFileSync(process.execPath, [GATE, '--mode', 'check', '--json', '--target', target], { encoding: 'utf8' });
  } catch (error) {
    stdout = error.stdout || ''; // gate exits 1 on hard fail; JSON is still printed
  }
  return JSON.parse(stdout);
}

function writeFile(root, relPath, contents) {
  const full = path.join(root, relPath);
  fs.mkdirSync(path.dirname(full), { recursive: true });
  fs.writeFileSync(full, contents);
}

const hexFindings = (report) => report.findings.filter((f) => f.id === 'hardcoded_hex_on_ui');

const results = [];
function check(name, cond, detail) {
  results.push({ name, ok: !!cond, detail });
  console.log(`${cond ? 'PASS' : 'FAIL'}: ${name}${detail ? ` — ${detail}` : ''}`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'shape-gate-hardcoded-hex-'));
try {
  // Case 1: CSS 裸 hex（非白名单·非定义行）→ 1 error，check 模式 status=fail。
  const c1 = path.join(tmp, 'case-bare-hex');
  writeFile(c1, `${SRC_REL}/styles.css`, '.card {\n  background: #aabbcc;\n}\n');
  const r1 = runGate(c1);
  check('CSS 裸 hex -> exactly 1 hardcoded_hex_on_ui', hexFindings(r1).length === 1, `got ${hexFindings(r1).length}`);
  check('finding severity=error', hexFindings(r1).every((f) => f.severity === 'error'), JSON.stringify(hexFindings(r1).map((f) => f.severity)));
  check('check-mode Status=fail on new bare hex', r1.summary.status === 'fail', `status=${r1.summary.status}`);
  check('finding points at styles.css/#aabbcc', hexFindings(r1)[0] && hexFindings(r1)[0].detail && hexFindings(r1)[0].detail.file.endsWith('styles.css') && hexFindings(r1)[0].detail.hex === '#aabbcc', JSON.stringify(hexFindings(r1)[0] && hexFindings(r1)[0].detail));

  // Case 2: 正典定义行 --x: #hex → 不误伤（0 finding）。
  const c2 = path.join(tmp, 'case-def-line');
  writeFile(c2, `${SRC_REL}/styles.css`, ':root {\n  --paper-card: #fbfcf8;\n}\n.ok { color: var(--paper-card); }\n');
  const r2 = runGate(c2);
  check('token 定义行 -> 0 hardcoded_hex_on_ui', hexFindings(r2).length === 0, `got ${hexFindings(r2).length}`);

  // Case 3: 注释内 hex（含 #004 bug 引用形）→ 不误伤。
  const c3 = path.join(tmp, 'case-comment');
  writeFile(c3, `${SRC_REL}/styles.css`, '/* 记忆 ux-render-bugs #004 雷区 */\n.card {\n  /* background: #aabbcc; */\n  color: var(--ink);\n}\n');
  const r3 = runGate(c3);
  check('注释内 hex -> 0 hardcoded_hex_on_ui', hexFindings(r3).length === 0, `got ${hexFindings(r3).length}`);

  // Case 4: var() 回退位 hex → error（错误回退实证形）。
  const c4 = path.join(tmp, 'case-fallback');
  writeFile(c4, `${SRC_REL}/styles.css`, '.card {\n  color: var(--warning, #9b4a18);\n}\n');
  const r4 = runGate(c4);
  check('var() 回退位 hex -> exactly 1 hardcoded_hex_on_ui', hexFindings(r4).length === 1, `got ${hexFindings(r4).length}`);

  // Case 5: %23 转义形（非白名单）→ error。
  const c5 = path.join(tmp, 'case-escaped');
  writeFile(c5, `${SRC_REL}/styles.css`, '.x { background: url("data:image/svg+xml;utf8,<svg stroke=\'%23aabbcc\'/>"); }\n');
  const r5 = runGate(c5);
  check('%23 转义形 -> exactly 1 hardcoded_hex_on_ui', hexFindings(r5).length === 1, `got ${hexFindings(r5).length}`);

  // Case 6: 白名单 hex值|path → 0 error，记入 deferred。
  const c6 = path.join(tmp, 'case-whitelisted');
  writeFile(c6, `${SRC_REL}/styles.css`, '.boot {\n  color: #1a1c1a;\n}\n');
  const r6 = runGate(c6);
  check('whitelisted hex -> 0 hardcoded_hex_on_ui', hexFindings(r6).length === 0, `got ${hexFindings(r6).length}`);
  check('whitelisted hit recorded as deferred', r6.metrics.hardcoded_hex.deferred.some((d) => d.file.endsWith('styles.css') && d.hex === '#1a1c1a'), `deferred=${r6.metrics.hardcoded_hex.deferred.length}`);
  check('whitelisted tree Status stays pass', r6.summary.status === 'pass', `status=${r6.summary.status}`);

  // Case 7: TSX 内联 style 裸 hex（非白名单）→ error。
  const c7 = path.join(tmp, 'case-tsx-inline');
  writeFile(c7, `${SRC_REL}/views/Foo.tsx`, 'export function Foo() {\n  return <p style={{ color: "#aabbcc" }}>x</p>;\n}\n');
  const r7 = runGate(c7);
  check('TSX 内联裸 hex -> exactly 1 hardcoded_hex_on_ui', hexFindings(r7).length === 1, `got ${hexFindings(r7).length}`);

  // Case 8: 干净树 → 0 findings。
  const c8 = path.join(tmp, 'case-clean');
  writeFile(c8, `${SRC_REL}/styles.css`, '.card {\n  background: var(--paper-card);\n  color: var(--ink-deep);\n}\n');
  const r8 = runGate(c8);
  check('clean tree -> 0 hardcoded_hex_on_ui', hexFindings(r8).length === 0, `got ${hexFindings(r8).length}`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
