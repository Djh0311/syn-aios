#!/usr/bin/env node
'use strict';
const fs = require('fs');
const { spawnSync } = require('child_process');
const ZERO = /^0+$/;
const OID = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/;
const SECRET = /-----BEGIN [A-Z ]*PRIVATE KEY-----|\bAKIA[0-9A-Z]{16}\b|\b(?:gh[pousr]_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,}|xox[baprs]-[A-Za-z0-9-]{10,}|sk-(?:proj-)?[A-Za-z0-9_-]{20,})\b/;
function ranges(input) {
  const result = [];
  for (const line of input.trim().split(/\r?\n/).filter(Boolean)) {
    const [, local, , remote, ...rest] = line.trim().split(/\s+/);
    if (rest.length || !OID.test(local || '') || !OID.test(remote || '') || local.length !== remote.length) return null;
    if (!ZERO.test(local)) result.push(ZERO.test(remote) ? local : `${remote}..${local}`);
  }
  return result;
}
function read(range) {
  const r = spawnSync('git', ['log', '--format=', '--no-ext-diff', '--diff-merges=first-parent', '-p', '-U0', range], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  return r.status === 0 ? r.stdout : null;
}
function scan(input, get = read) {
  const items = ranges(input);
  return items !== null && items.every((range) => {
    let patch; try { patch = get(range); } catch { return false; }
    return typeof patch === 'string' && !patch.split('\n').some((line) => /^\+[^+]/.test(line) && SECRET.test(line));
  });
}
function main(argv) {
  const remote = argv && argv[0] ? argv[0] : 'origin';
  const allowed = require('./gate.js').evaluate(process.cwd(),
    { category: 'external', operation: 'push', target: remote }, { write: true });
  if (allowed.decision !== 'allow') {
    console.error(`pre-push: ${allowed.reason}; push stopped`);
    return 1;
  }
  if (scan(fs.readFileSync(0, 'utf8'))) return 0;
  console.error('pre-push: potential secret or unreadable pending commit; push stopped');
  return 1;
}
if (require.main === module) process.exit(main(process.argv.slice(2)));
module.exports = { ranges, scan, main };
