#!/usr/bin/env node

// Self-test for checkpoint-audit.js. Builds throwaway git repos in os.tmpdir() and
// asserts: a consistent closed package -> PASS; forged reports (commit absent,
// dirty tree, out-of-bounds files, review without STATUS) -> FAIL. Never touches
// the real repo. Exit 0 = all checks pass.

const fs = require('fs');
const os = require('os');
const path = require('path');
const { execFileSync } = require('child_process');

const AUDIT = path.join(__dirname, 'checkpoint-audit.js');

function sh(repo, args) {
  return execFileSync('git', ['-c', 'commit.gpgsign=false', '-c', 'user.email=t@t', '-c', 'user.name=t', ...args],
    { cwd: repo, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
}
function write(repo, rel, content) {
  const full = path.join(repo, rel);
  fs.mkdirSync(path.dirname(full), { recursive: true });
  fs.writeFileSync(full, content);
}
function runAudit(extraArgs) {
  let stdout;
  try {
    stdout = execFileSync(process.execPath, [AUDIT, '--json', '--skip-gates', ...extraArgs], { encoding: 'utf8' });
  } catch (error) {
    stdout = error.stdout ? error.stdout.toString() : '';
  }
  return JSON.parse(stdout);
}
function checkOf(report, id) { return report.checks.find((c) => c.id === id) || {}; }

const results = [];
function expect(name, cond, detail) {
  results.push({ name, ok: !!cond });
  console.log(`${cond ? 'PASS' : 'FAIL'}: ${name}${detail ? ` — ${detail}` : ''}`);
}

const SLUG = 'demo-pkg-v1';

// Build a consistent, fully-closed package repo. Returns {repo, impl}.
function buildGoodRepo() {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-good-'));
  sh(repo, ['init', '-q']);
  // impl commit: only docs/ touched
  write(repo, 'docs/feature.md', '# feature\n');
  sh(repo, ['add', '-A']);
  sh(repo, ['commit', '-q', '-m', 'impl: demo feature']);
  const impl = sh(repo, ['rev-parse', 'HEAD']);
  // task package with an allow-list, review file with STATUS, CURRENT.md block
  write(repo, `tasks/2026-06-15-${SLUG}.md`, `# demo\n\n## 允许写入\n\n\`docs/**\`、\`tasks/**\`、\`evidence/**\`\n\n## 验收\n`);
  write(repo, `evidence/2026-06-15-${SLUG}-review-bot-v1.md`, `# review\n\n状态：\`STATUS: CLEAR\`\n`);
  write(repo, 'CURRENT.md',
    `# Current Authority\n\n## 当前结论\n\n最新 checkpoint：${SLUG} 已完成并经独立复核 \`STATUS: CLEAR\`；` +
    `task package commit 为 \`${impl}\`，implementation commit 为 \`${impl}\`。` +
    `记录见 \`tasks/2026-06-15-${SLUG}.md\`、\`evidence/2026-06-15-${SLUG}-review-bot-v1.md\`。\n`);
  sh(repo, ['add', '-A']);
  sh(repo, ['commit', '-q', '-m', 'docs: close demo package']);
  return { repo, impl };
}

const tmps = [];
try {
  // 1) GOOD closed package -> PASS, with each mechanical check green.
  {
    const { repo, impl } = buildGoodRepo(); tmps.push(repo);
    const r = runAudit(['--target', repo, '--package', SLUG]);
    expect('good package -> verdict PASS', r.verdict === 'PASS', r.failed_checks.join(','));
    expect('good: commits_reachable PASS', checkOf(r, 'commits_reachable').status === 'pass');
    expect('good: tree_clean PASS', checkOf(r, 'tree_clean').status === 'pass');
    expect('good: review_status PASS (CLEAR)', checkOf(r, 'review_status').status === 'pass' && checkOf(r, 'review_status').detail.status === 'CLEAR');
    expect('good: current_md_refs PASS', checkOf(r, 'current_md_refs').status === 'pass');
    expect('good: files_within_allow PASS (parsed docs/**)', checkOf(r, 'files_within_allow').status === 'pass');
    void impl;
  }

  // 2) FORGED: claimed commit does not exist -> FAIL (the 判据 case).
  {
    const { repo } = buildGoodRepo(); tmps.push(repo);
    const r = runAudit(['--target', repo, '--package', SLUG, '--commit', 'deadbeefdeadbeef']);
    expect('forged absent commit -> verdict FAIL', r.verdict === 'FAIL');
    const c = checkOf(r, 'commits_reachable');
    expect('forged absent commit -> commits_reachable FAIL', c.status === 'fail');
    expect('forged absent commit -> marked MISSING', JSON.stringify(c.detail).includes('MISSING'));
  }

  // 3) FORGED: dirty working tree -> FAIL.
  {
    const { repo } = buildGoodRepo(); tmps.push(repo);
    write(repo, 'docs/uncommitted.md', 'stray\n');
    const r = runAudit(['--target', repo, '--package', SLUG]);
    expect('dirty tree -> verdict FAIL', r.verdict === 'FAIL');
    expect('dirty tree -> tree_clean FAIL', checkOf(r, 'tree_clean').status === 'fail');
    // and --allow-dirty downgrades it
    const r2 = runAudit(['--target', repo, '--package', SLUG, '--allow-dirty']);
    expect('dirty tree + --allow-dirty -> tree_clean not FAIL', checkOf(r2, 'tree_clean').status !== 'fail');
  }

  // 4) FORGED: impl commit changes a file outside the allow-list -> FAIL.
  {
    const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-oob-')); tmps.push(repo);
    sh(repo, ['init', '-q']);
    write(repo, 'docs/ok.md', 'ok\n');
    write(repo, 'src/secret.rs', 'fn main() {}\n'); // out of bounds vs docs/**
    sh(repo, ['add', '-A']);
    sh(repo, ['commit', '-q', '-m', 'impl + stray product file']);
    const impl = sh(repo, ['rev-parse', 'HEAD']);
    const r = runAudit(['--target', repo, '--commit', impl, '--allow', 'docs/**']);
    const c = checkOf(r, 'files_within_allow');
    expect('out-of-bounds file -> files_within_allow FAIL', c.status === 'fail');
    expect('out-of-bounds file -> names src/secret.rs', JSON.stringify(c.detail).includes('src/secret.rs'));
  }

  // 5) FORGED: review evidence without a parseable STATUS -> FAIL.
  {
    const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-nostatus-')); tmps.push(repo);
    sh(repo, ['init', '-q']);
    write(repo, 'docs/x.md', 'x\n');
    sh(repo, ['add', '-A']); sh(repo, ['commit', '-q', '-m', 'impl']);
    const impl = sh(repo, ['rev-parse', 'HEAD']);
    write(repo, `tasks/2026-06-15-${SLUG}.md`, `# d\n\n## 允许写入\n\n\`docs/**\`\n`);
    write(repo, `evidence/2026-06-15-${SLUG}-review-bot-v1.md`, '# review\n\nlooks fine to me, no verdict token\n');
    write(repo, 'CURRENT.md', `## 当前结论\n\n${SLUG}: implementation commit 为 \`${impl}\`\n`);
    sh(repo, ['add', '-A']); sh(repo, ['commit', '-q', '-m', 'docs']);
    const r = runAudit(['--target', repo, '--package', SLUG]);
    expect('review without STATUS -> review_status FAIL', checkOf(r, 'review_status').status === 'fail');
    expect('review without STATUS -> verdict FAIL', r.verdict === 'FAIL');
  }
} finally {
  for (const t of tmps) fs.rmSync(t, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
