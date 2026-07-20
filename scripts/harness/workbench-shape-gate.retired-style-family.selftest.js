#!/usr/bin/env node

// Self-test for the retired_style_family rule (G2·2026-07-20) in workbench-shape-gate.js.
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

const familyFindings = (report) => report.findings.filter((f) => f.id === 'retired_style_family');

const results = [];
function check(name, cond, detail) {
  results.push({ name, ok: !!cond, detail });
  console.log(`${cond ? 'PASS' : 'FAIL'}: ${name}${detail ? ` — ${detail}` : ''}`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'shape-gate-retired-style-family-'));
try {
  // Case 1: tsx 再现退休类名（jiaoban-fact / memory-kv / settings-fact-grid / badge / jiaoban-step-badge /
  // project-canvas-status-pill / prsb-pill / jiaoban-done-pills）→ error。
  const c1 = path.join(tmp, 'case-retired-tsx');
  writeFile(
    c1,
    `${SRC_REL}/views/Foo.tsx`,
    [
      'export function Foo() {',
      '  return (',
      '    <div>',
      '      <p className="jiaoban-fact">a</p>',
      '      <p className="memory-kv">b</p>',
      '      <p className="settings-fact-grid">c</p>',
      '      <span className="badge">d</span>',
      '      <span className="jiaoban-step-badge tone-green">e</span>',
      '      <span className="project-canvas-status-pill ready">f</span>',
      '      <span className="prsb-pill warning">g</span>',
      '      <div className="jiaoban-done-pills">h</div>',
      '    </div>',
      '  );',
      '}',
      ''
    ].join('\n')
  );
  const r1 = runGate(c1);
  check('退休族 tsx 再现 8 族 -> exactly 8 retired_style_family', familyFindings(r1).length === 8, `got ${familyFindings(r1).length}`);
  check('finding severity=error', familyFindings(r1).every((f) => f.severity === 'error'), JSON.stringify(familyFindings(r1).map((f) => f.severity)));
  check('check-mode Status=fail on retired family', r1.summary.status === 'fail', `status=${r1.summary.status}`);

  // Case 2: 保留名/复合名不误伤（jiaoban-fact-btn / jiaoban-fact-done / sc-badge / badgeItem / badges / badge-tone）。
  const c2 = path.join(tmp, 'case-kept-names');
  writeFile(
    c2,
    `${SRC_REL}/views/Foo.tsx`,
    [
      'export function Foo() {',
      '  return (',
      '    <div>',
      '      <button className="jiaoban-fact-btn">a</button>',
      '      <span className="jiaoban-fact-done">b</span>',
      '      <span className="sc-badge">c</span>',
      '      <span className="rail-icon-badge">d</span>',
      '    </div>',
      '  );',
      '}',
      'const badgeItem = { badges: ["x"], badge_tone: "warn" };',
      'export default badgeItem;',
      ''
    ].join('\n')
  );
  writeFile(c2, `${SRC_REL}/styles.css`, '.session-card .sc-badge {\n  color: red;\n}\n.canvas-boundary-badges span {\n  color: blue;\n}\n');
  const r2 = runGate(c2);
  check('保留名/复合名 -> 0 retired_style_family', familyFindings(r2).length === 0, `got ${familyFindings(r2).length}`);
  check('保留名树 Status=pass', r2.summary.status === 'pass', `status=${r2.summary.status}`);

  // Case 3: Badge import 再现 → error。
  const c3 = path.join(tmp, 'case-badge-import');
  writeFile(c3, `${SRC_REL}/views/Foo.tsx`, 'import { Badge } from "../../components/Badge";\nexport const x = 1;\n');
  const r3 = runGate(c3);
  check('Badge import 再现 -> exactly 1 retired_style_family', familyFindings(r3).length === 1, `got ${familyFindings(r3).length}`);

  // Case 4: css 再现退休类名 → error；注释行不误伤。
  const c4 = path.join(tmp, 'case-retired-css');
  writeFile(
    c4,
    `${SRC_REL}/styles.css`,
    '/* 七律⑤事实行定式(对齐样板 .jiaoban-fact) */\n.badge {\n  color: red;\n}\n.memory-kv {\n  color: blue;\n}\n.badge-row,\n.action-row {\n  display: flex;\n}\n'
  );
  const r4 = runGate(c4);
  check('css 退休族（badge/memory-kv·badge-row 并入 badge 族去重）-> exactly 2 retired_style_family', familyFindings(r4).length === 2, `got ${familyFindings(r4).length}`);

  // Case 5: 白名单 2 条（running-status-pill 死视图引用面 + ActiveWorkbenchView spec-empty）→ 0 error、deferred 2。
  const c5 = path.join(tmp, 'case-whitelisted');
  writeFile(
    c5,
    `${SRC_REL}/views/RunningWorkflowsView.tsx`,
    [
      'export function Foo({ items }: { items: { badge_id: string; tone: string; label: string }[] }) {',
      '  return (',
      '    <div>',
      '      {items.map((badgeItem) => (',
      '        <span className={`running-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>',
      '          {badgeItem.label}',
      '        </span>',
      '      ))}',
      '      <span className="running-status-pill neutral">当前视图</span>',
      '    </div>',
      '  );',
      '}',
      ''
    ].join('\n')
  );
  writeFile(
    c5,
    `${SRC_REL}/components/ActiveWorkbenchView.tsx`,
    'export function Foo() {\n  return <p className="spec-empty muted small-note">想法箱是空的</p>;\n}\n'
  );
  writeFile(c5, `${SRC_REL}/styles.css`, '.running-status-pill {\n  color: red;\n}\n');
  const r5 = runGate(c5);
  check('白名单树 -> 0 retired_style_family error', familyFindings(r5).length === 0, `got ${familyFindings(r5).length}`);
  check('白名单树 Status=pass', r5.summary.status === 'pass', `status=${r5.summary.status}`);
  check(
    '白名单 deferred=2（死视图 2 span 去重 1 + spec-empty 1；styles.css 定义段 tsOnly 不入账）',
    r5.metrics.retired_style_family.deferred.length === 2,
    `deferred=${JSON.stringify(r5.metrics.retired_style_family.deferred)}`
  );

  // Case 6: spec-* 直连（非 SpecPrimitives 的 tsx 写 spec-fact-row/spec-pill-ok 等）→ error；
  // SpecPrimitives.tsx 本体与 spec-scroll 不误伤。
  const c6 = path.join(tmp, 'case-spec-direct');
  writeFile(
    c6,
    `${SRC_REL}/views/Foo.tsx`,
    [
      'export function Foo() {',
      '  return (',
      '    <div className="spec-scroll">',
      '      <p className="spec-fact-row">a</p>',
      '      <span className="spec-pill spec-pill-ok">b</span>',
      '      <p className="spec-seg-title">c</p>',
      '      <span className="spec-bad">d</span>',
      '    </div>',
      '  );',
      '}',
      ''
    ].join('\n')
  );
  writeFile(
    c6,
    `${SRC_REL}/components/SpecPrimitives.tsx`,
    [
      'export function FactRow() {',
      '  return (',
      '    <div className="spec-fact-row">',
      '      <span className="spec-fact-k">k</span>',
      '      <span className="spec-fact-v spec-bad">v</span>',
      '    </div>',
      '  );',
      '}',
      'export function Pill() {',
      '  return <span className="spec-pill spec-pill-warn">p</span>;',
      '}',
      ''
    ].join('\n')
  );
  const r6 = runGate(c6);
  check(
    'spec-* 直连 4 类（fact-row/pill-ok/seg-title/bad·spec-scroll 不算）-> exactly 4 retired_style_family',
    familyFindings(r6).length === 4,
    `got ${JSON.stringify(familyFindings(r6).map((f) => f.detail && f.detail.pattern))}`
  );
  check(
    'SpecPrimitives 本体零误伤',
    familyFindings(r6).every((f) => !f.detail || !f.detail.file.endsWith('SpecPrimitives.tsx')),
    JSON.stringify(familyFindings(r6).map((f) => f.detail && f.detail.file))
  );

  // Case 7: 干净树（基座组件用法 + 非退休类名）→ 0 findings。
  const c7 = path.join(tmp, 'case-clean');
  writeFile(
    c7,
    `${SRC_REL}/views/Foo.tsx`,
    'import { Pill, FactRow } from "../components/SpecPrimitives";\nexport function Foo() {\n  return <div className="jiaoban-step-row tone-green"><FactRow k="a">b</FactRow><Pill tone="ok">c</Pill></div>;\n}\n'
  );
  writeFile(c7, `${SRC_REL}/styles.css`, '.spec-pill {\n  color: var(--muted);\n}\n.action-row {\n  display: flex;\n}\n');
  const r7 = runGate(c7);
  check('干净树 -> 0 retired_style_family', familyFindings(r7).length === 0, `got ${familyFindings(r7).length}`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
