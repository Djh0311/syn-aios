#!/usr/bin/env node

// Self-test for the HG-3 U-Gate dedup check in workbench-shape-gate.js.
// Builds throwaway fixture trees in os.tmpdir() and runs the gate against them.
// It never reads or writes real product code. Exit 0 = all cases pass.

const fs = require('fs');
const os = require('os');
const path = require('path');
const { execFileSync } = require('child_process');

const GATE = path.join(__dirname, 'workbench-shape-gate.js');
const SRC_REL = 'prototypes/productized-desktop-shell/src-tauri/src';

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

function dedupWarnings(report) {
  return report.findings.filter((f) => f.id === 'converged_helper_redefined');
}

const results = [];
function check(name, cond, detail) {
  results.push({ name, ok: !!cond, detail });
  console.log(`${cond ? 'PASS' : 'FAIL'}: ${name}${detail ? ` — ${detail}` : ''}`);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'shape-gate-dedup-'));
try {
  // Case 1: duplicate `fn sha256_hex` outside utils/ -> 1 warning, status still pass (warning-only).
  const c1 = path.join(tmp, 'case-dup');
  writeFile(c1, `${SRC_REL}/dup_probe.rs`, 'pub(crate) fn sha256_hex(value: &str) -> String {\n    String::new()\n}\n');
  const r1 = runGate(c1);
  const w1 = dedupWarnings(r1);
  check('dup fn sha256_hex outside utils/ -> exactly 1 dedup warning', w1.length === 1, `got ${w1.length}`);
  check('the warning is severity=warn (never error)', w1.every((f) => f.severity === 'warn'), JSON.stringify(w1.map((f) => f.severity)));
  check('default-mode Status stays pass with a dedup warning', r1.summary.status === 'pass', `status=${r1.summary.status}`);
  check('warning points at dup_probe.rs/sha256_hex', w1[0] && w1[0].detail && w1[0].detail.file.endsWith('dup_probe.rs') && w1[0].detail.helper === 'sha256_hex', JSON.stringify(w1[0] && w1[0].detail));

  // Case 2: same kind of definition at a WHITELISTED path -> suppressed (0 warning), shows as deferred.
  const c2 = path.join(tmp, 'case-whitelist');
  writeFile(c2, `${SRC_REL}/observation_store.rs`, 'pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {\n    Ok(PathBuf::new())\n}\n');
  const r2 = runGate(c2);
  const w2 = dedupWarnings(r2);
  const deferred2 = r2.metrics.helper_duplicates.deferred;
  check('whitelisted sidecar_path -> 0 dedup warning', w2.length === 0, `got ${w2.length}`);
  check('whitelisted hit is recorded as deferred', deferred2.some((d) => d.file.endsWith('observation_store.rs') && d.helper === 'sidecar_path'), `deferred=${deferred2.length}`);

  // Case 3: canonical definition under utils/ -> exempt (0 warning).
  const c3 = path.join(tmp, 'case-utils');
  writeFile(c3, `${SRC_REL}/utils/hash.rs`, 'pub(crate) fn sha256_hex(value: &str) -> String {\n    String::new()\n}\n');
  const r3 = runGate(c3);
  check('canonical fn under utils/ -> 0 dedup warning (exempt)', dedupWarnings(r3).length === 0, `got ${dedupWarnings(r3).length}`);

  // Case 4: clean tree (no target helpers) -> 0 dedup warning.
  const c4 = path.join(tmp, 'case-clean');
  writeFile(c4, `${SRC_REL}/unrelated.rs`, 'pub(crate) fn unrelated_helper(value: &str) -> String {\n    String::new()\n}\n');
  const r4 = runGate(c4);
  check('clean tree -> 0 dedup warning', dedupWarnings(r4).length === 0, `got ${dedupWarnings(r4).length}`);
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
