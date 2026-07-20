#!/usr/bin/env node

// Self-test for the machine_face_on_ui rule (人话工程②·2026-07-20) in workbench-shape-gate.js.
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

const mfErrors = (report) => report.findings.filter((f) => f.id === 'machine_face_on_ui');
const mfWarns = (report) => report.findings.filter((f) => f.id === 'machine_face_state_hint');

const results = [];
function check(name, cond, detail) {
  results.push({ name, ok: !!cond, detail });
  console.log(`${cond ? 'PASS' : 'FAIL'}: ${name}${detail ? ` — ${detail}` : ''}`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'shape-gate-machine-face-'));
try {
  // Case 1: {error.message} JSX 直渲（非白名单文件）→ 1 个 error 级 finding，check 模式 status=fail。
  const c1 = path.join(tmp, 'case-jsx-error-message');
  writeFile(c1, `${SRC_REL}/views/Foo.tsx`, 'export function Foo({ error }: { error: Error }) {\n  return <code>{error.message}</code>;\n}\n');
  const r1 = runGate(c1);
  check('{error.message} 直渲 -> exactly 1 machine_face_on_ui', mfErrors(r1).length === 1, `got ${mfErrors(r1).length}`);
  check('finding severity=error', mfErrors(r1).every((f) => f.severity === 'error'), JSON.stringify(mfErrors(r1).map((f) => f.severity)));
  check('check-mode Status=fail on new direct render', r1.summary.status === 'fail', `status=${r1.summary.status}`);
  check('finding points at Foo.tsx/jsx_error_message', mfErrors(r1)[0] && mfErrors(r1)[0].detail && mfErrors(r1)[0].detail.file.endsWith('Foo.tsx') && mfErrors(r1)[0].detail.pattern === 'jsx_error_message', JSON.stringify(mfErrors(r1)[0] && mfErrors(r1)[0].detail));

  // Case 2: <pre>stderr: {event.stderr}</pre> 形（非白名单文件）→ error 级。
  const c2 = path.join(tmp, 'case-stderr-pre');
  writeFile(c2, `${SRC_REL}/views/Bar.tsx`, 'export function Bar({ event }: { event: { stderr: string } }) {\n  return <pre>stderr: {event.stderr}</pre>;\n}\n');
  const r2 = runGate(c2);
  check('<pre>stderr: 形 -> exactly 1 machine_face_on_ui', mfErrors(r2).length === 1, `got ${mfErrors(r2).length}`);
  check('finding pattern=jsx_event_stderr_pre', mfErrors(r2)[0] && mfErrors(r2)[0].detail && mfErrors(r2)[0].detail.pattern === 'jsx_event_stderr_pre', JSON.stringify(mfErrors(r2)[0] && mfErrors(r2)[0].detail));

  // Case 3: 白名单内既有违规（main.tsx 启动失败屏形）→ 0 error，记入 deferred。
  const c3 = path.join(tmp, 'case-whitelisted-main');
  writeFile(c3, `${SRC_REL}/main.tsx`, 'export function Boot({ error }: { error: Error }) {\n  return <code>{error.message || "未知错误"}</code>;\n}\n');
  const r3 = runGate(c3);
  check('whitelisted main.tsx 形 -> 0 machine_face_on_ui', mfErrors(r3).length === 0, `got ${mfErrors(r3).length}`);
  check('whitelisted hit recorded as deferred', r3.metrics.machine_face.deferred.some((d) => d.file.endsWith('main.tsx') && d.pattern === 'jsx_error_message'), `deferred=${r3.metrics.machine_face.deferred.length}`);
  check('whitelisted tree Status stays pass', r3.summary.status === 'pass', `status=${r3.summary.status}`);

  // Case 4: <details> 下钻 raw_snippet 合规格板（JiaobanHistory 样板形）→ 不误伤（0 error / 0 warn）。
  const c4 = path.join(tmp, 'case-details-drilldown');
  writeFile(c4, `${SRC_REL}/views/projects/jiaoban/JiaobanHistory.tsx`, 'export function H({ entry }: { entry: { error: { raw_snippet: string } } }) {\n  return <details className="jiaoban-run-error-raw"><summary>查看原文</summary><pre>{entry.error.raw_snippet}</pre></details>;\n}\n');
  const r4 = runGate(c4);
  check('<details> raw_snippet 样板 -> 0 machine_face_on_ui', mfErrors(r4).length === 0, `got ${mfErrors(r4).length}`);
  check('<details> raw_snippet 样板 -> 0 machine_face_state_hint', mfWarns(r4).length === 0, `got ${mfWarns(r4).length}`);

  // Case 5: error.message 进 state 形（非白名单文件）→ warn-only，check 模式 status 仍 pass。
  const c5 = path.join(tmp, 'case-state-form');
  writeFile(c5, `${SRC_REL}/views/Baz.tsx`, 'export function Baz() {\n  const onErr = (error: unknown) => setLedgerError(error instanceof Error ? error.message : String(error));\n  return null;\n}\n');
  const r5 = runGate(c5);
  check('state 形 -> exactly 1 machine_face_state_hint', mfWarns(r5).length === 1, `got ${mfWarns(r5).length}`);
  check('state 形 severity=warn (never error)', mfWarns(r5).every((f) => f.severity === 'warn'), JSON.stringify(mfWarns(r5).map((f) => f.severity)));
  check('state 形 Status stays pass (warn-only)', r5.summary.status === 'pass', `status=${r5.summary.status}`);

  // Case 6: state 形在白名单文件（AuditLedgerView.tsx）→ 0 warn，记入 deferred。
  const c6 = path.join(tmp, 'case-state-whitelisted');
  writeFile(c6, `${SRC_REL}/views/AuditLedgerView.tsx`, 'export function A() {\n  const onErr = (error: unknown) => setLedgerError(error instanceof Error ? error.message : String(error));\n  return null;\n}\n');
  const r6 = runGate(c6);
  check('whitelisted state 形 -> 0 machine_face_state_hint', mfWarns(r6).length === 0, `got ${mfWarns(r6).length}`);
  check('whitelisted state hit recorded as deferred', r6.metrics.machine_face.deferred.some((d) => d.file.endsWith('AuditLedgerView.tsx') && d.pattern === 'state_error_message'), `deferred=${r6.metrics.machine_face.deferred.length}`);

  // Case 7: 干净树 → 0 hits。
  const c7 = path.join(tmp, 'case-clean');
  writeFile(c7, `${SRC_REL}/views/Clean.tsx`, 'export function Clean({ human }: { human: string }) {\n  return <p>{human}</p>;\n}\n');
  const r7 = runGate(c7);
  check('clean tree -> 0 machine_face_on_ui', mfErrors(r7).length === 0, `got ${mfErrors(r7).length}`);
  check('clean tree -> 0 machine_face_state_hint', mfWarns(r7).length === 0, `got ${mfWarns(r7).length}`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
