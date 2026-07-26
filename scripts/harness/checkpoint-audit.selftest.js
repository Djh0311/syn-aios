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
  return runAuditWithExit(extraArgs).report;
}
function runAuditWithExit(extraArgs, options = {}) {
  let stdout;
  let exitCode = 0;
  try {
    stdout = execFileSync(process.execPath, [AUDIT, '--json', '--skip-gates', ...extraArgs], {
      encoding: 'utf8',
      env: { ...process.env, ...(options.env || {}) },
    });
  } catch (error) {
    exitCode = typeof error.status === 'number' ? error.status : 1;
    stdout = error.stdout ? error.stdout.toString() : '';
  }
  try { return { exitCode, report: JSON.parse(stdout) }; }
  catch (_e) { return { exitCode, report: { verdict: 'PARSE_ERROR', failed_checks: [], checks: [], _raw: stdout } }; }
}
function checkOf(report, id) { return (report.checks || []).find((c) => c.id === id) || {}; }
function writeTmpJson(obj) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-json-'));
  tmps.push(dir);
  const f = path.join(dir, 'record.json');
  fs.writeFileSync(f, JSON.stringify(obj, null, 2));
  return f;
}

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

function alignmentFields(overrides = {}) {
  return {
    authority_chain: 'AUTHORITY.md -> decisions/current-decision.md',
    plan_anchor: 'docs/plans/current-plan.md#phase-4',
    existing_before_new: '复用 `conversation-transport.agent-manual-relay`。',
    capabilities_touched: 'conversation-transport.agent-manual-relay',
    forbidden_alternatives: 'resident/private-home 旧路线。',
    ...overrides,
  };
}

function writeCurrentAlignmentMap(repo) {
  write(repo, 'docs/code-map/index.json', JSON.stringify({
    domains: [{ id: 'fixture', path: 'docs/code-map/domains/fixture.json' }],
  }, null, 2));
  write(repo, 'docs/code-map/domains/fixture.json', JSON.stringify({
    capabilities: [
      { id: 'conversation-transport.agent-manual-relay', status: 'active' },
      { id: 'workflow-execution-governance.legacy-real-workflow-execution', status: 'legacy' },
      { id: 'syn-mcp-supervision.tracked-supervisor-orchestrator', status: 'needs-confirmation' },
    ],
  }, null, 2));
}

function writeCurrentAlignmentPackage(repo, rel, fields) {
  const lines = [
    '# Important task fixture',
    '',
    '## Authority and plan alignment',
    '',
    ...Object.entries(fields).map(([field, value]) => `- ${field}: ${value}`),
    '',
    '## Scope',
    '',
    'fixture only',
  ];
  write(repo, rel, `${lines.join('\n')}\n`);
}

function buildCurrentAlignmentRepo({ binding = null, fields = alignmentFields() } = {}) {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-current-'));
  const packagePath = 'tasks/current-important-task.md';
  write(repo, 'docs/project-context.json', JSON.stringify({
    schemaVersion: 1,
    taskPackage: 'tasks/historical-closed-contract.md',
    checkpoint: { currentImportantTask: binding },
  }, null, 2));
  write(repo, 'tasks/historical-closed-contract.md', '# historical route only\n');
  write(repo, 'tasks/2026-01-old-important-a.md', '# historical a\n');
  write(repo, 'tasks/2026-02-old-important-b.md', '# historical b\n');
  write(repo, 'docs/plans/current-plan.md', '# Current plan\n\n## Phase 4\n');
  writeCurrentAlignmentMap(repo);
  if (binding) writeCurrentAlignmentPackage(repo, binding.path || packagePath, fields);
  return { repo, packagePath };
}

function currentAudit(repo, options = {}) {
  return runAuditWithExit(['--target', repo, '--current'], options);
}

function warningCodes(report) {
  return (report.warnings || []).map((warning) => warning.code);
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
    expect('good: evidence_hash_format NA (no --record, zero regression)', checkOf(r, 'evidence_hash_format').status === 'na');
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

  // 6) NEW (C4 hardening): evidence_hash_format — check (7). Scans hash-named string
  //    fields in --record JSON: must be 64-lowercase-hex or a non-hex sentinel; a
  //    "looks like hash but isn't 64 hex" value (e.g. the real B1 59-char defect) -> FAIL.
  {
    const { repo } = buildGoodRepo(); tmps.push(repo);
    const HEX64 = 'a'.repeat(64);
    const HEX64B = 'b'.repeat(64);

    // 6a) zero false positives (most critical): valid 64-hex + sentinels + *_algorithm
    //     + 40-char git_head_* (key has no hash -> not picked) + uuid + nested sha256.
    const cleanRecord = writeTmpJson({
      source_root_hash_before: HEX64,
      source_root_hash_after: HEX64,
      source_root_hash_algorithm: 'workbench_source_aggregate_hash.v1:preflight_path_ref_file_hash_classification',
      production_db_hash_before: 'missing_before=true',
      production_db_hash_after: HEX64,
      pending_hash: 'pending_before=true',
      created_hash: 'not_created',
      b0_report_hash: 'not_applicable_b0',
      pre_commit_hash: 'not_recorded_before_report_commit',
      git_head_before: 'a'.repeat(40),
      git_head_after: 'f'.repeat(40),
      agent_id: '019ec1e4-f49c-7d90-9166-42c8a0b1d2e3',
      source_inventory_before_after: [{ sha256_before: HEX64B, sha256_after: HEX64B }],
      manifests: { backup_manifest_file_sha256: HEX64 }
    });
    const rOk = runAudit(['--target', repo, '--package', SLUG, '--record', cleanRecord]);
    expect('hash 6a: clean record -> evidence_hash_format PASS (zero false positives)',
      checkOf(rOk, 'evidence_hash_format').status === 'pass', JSON.stringify(checkOf(rOk, 'evidence_hash_format').detail));
    expect('hash 6a: clean record -> verdict PASS', rOk.verdict === 'PASS', rOk.failed_checks.join(','));

    // 6b) truncated 59-char hex in a *_hash field -> FAIL, detail names field + reason.
    const truncRecord = writeTmpJson({
      report_hash: HEX64,
      post_apply_preflight_report_hash: 'a'.repeat(59)
    });
    const rTrunc = runAudit(['--target', repo, '--package', SLUG, '--record', truncRecord]);
    const cT = checkOf(rTrunc, 'evidence_hash_format');
    expect('hash 6b: 59-char hash -> evidence_hash_format FAIL', cT.status === 'fail');
    expect('hash 6b: detail names the field + reason not_64_lowercase_hex',
      JSON.stringify(cT.detail || '').includes('post_apply_preflight_report_hash') && JSON.stringify(cT.detail || '').includes('not_64_lowercase_hex'));
    expect('hash 6b: -> verdict FAIL', rTrunc.verdict === 'FAIL');

    // 6c) 64-char but UPPERCASE hex -> FAIL.
    const upperRecord = writeTmpJson({ export_verification_hash: 'A'.repeat(64) });
    const rUp = runAudit(['--target', repo, '--package', SLUG, '--record', upperRecord]);
    expect('hash 6c: uppercase 64-hex -> evidence_hash_format FAIL', checkOf(rUp, 'evidence_hash_format').status === 'fail');
    expect('hash 6c: -> verdict FAIL', rUp.verdict === 'FAIL');

    // 6d) comma-separated --record, one clean + one bad -> FAIL.
    const rMulti = runAudit(['--target', repo, '--package', SLUG, '--record', `${cleanRecord},${truncRecord}`]);
    expect('hash 6d: multi --record with one bad -> evidence_hash_format FAIL', checkOf(rMulti, 'evidence_hash_format').status === 'fail');
  }

  // 7) Phase 4: --current only trusts an explicitly bound important task. A null
  //    binding must not inspect the historical taskPackage or scan tasks/*.md.
  {
    const { repo } = buildCurrentAlignmentRepo(); tmps.push(repo);
    const probeDir = fs.mkdtempSync(path.join(os.tmpdir(), 'ckpt-audit-current-probe-')); tmps.push(probeDir);
    const probe = path.join(probeDir, 'no-tasks-scan.js');
    fs.writeFileSync(probe, [
      "const fs = require('node:fs');",
      'const original = fs.readdirSync;',
      'fs.readdirSync = function guarded(target, ...args) {',
      "  if (String(target).endsWith('/tasks')) throw new Error('tasks directory scan is forbidden in --current no-package mode');",
      '  return original.call(this, target, ...args);',
      '};',
      '',
    ].join('\n'));
    const { exitCode, report } = currentAudit(repo, { env: { NODE_OPTIONS: `--require=${probe}` } });
    expect('current null -> NOT_APPLICABLE', report.status === 'NOT_APPLICABLE' && report.reason === 'NO_CURRENT_IMPORTANT_TASK_PACKAGE');
    expect('current null -> exit 0', exitCode === 0, String(exitCode));
    expect('current null -> does not scan historical tasks directory', report.verdict !== 'PARSE_ERROR', report._raw);
  }

  // 8) Advisory current packages only warn for omitted fields, while a route that
  //    explicitly marks strict makes those same missing fields fail.
  {
    const advisory = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({ existing_before_new: '', forbidden_alternatives: '' }),
    }); tmps.push(advisory.repo);
    const advisoryResult = currentAudit(advisory.repo);
    expect('current advisory missing existing/forbidden -> warnings',
      warningCodes(advisoryResult.report).filter((code) => code === 'ALIGNMENT_FIELD_MISSING').length === 2,
      JSON.stringify(advisoryResult.report.warnings));
    expect('current advisory missing fields -> exit 0', advisoryResult.exitCode === 0, String(advisoryResult.exitCode));

    const strict = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'strict' },
      fields: alignmentFields({ existing_before_new: '', forbidden_alternatives: '' }),
    }); tmps.push(strict.repo);
    const strictResult = currentAudit(strict.repo);
    expect('current strict missing existing/forbidden -> FAIL',
      strictResult.report.status === 'STRICT_ALIGNMENT_ERRORS' && (strictResult.report.errors || []).length === 2,
      JSON.stringify(strictResult.report));
    expect('current strict missing fields -> exit 1', strictResult.exitCode === 1, String(strictResult.exitCode));
  }

  // 9) Five populated fields are navigation evidence only, never a completion or
  //    semantic-correctness claim.
  {
    const { repo, packagePath } = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
    }); tmps.push(repo);
    const { exitCode, report } = currentAudit(repo);
    expect('current complete fields -> FIELDS_PRESENT', report.status === 'FIELDS_PRESENT', JSON.stringify(report));
    expect('current complete fields -> exit 0', exitCode === 0, String(exitCode));
    expect('current complete fields -> retains explicit package path', report.currentImportantTask?.path === packagePath);
    expect('current complete fields -> boundary rejects completion inference',
      /does not prove.*completion.*product acceptance/i.test(report.boundary || ''), report.boundary);
  }

  // 9b) Plan anchors may use a percent-encoded Chinese heading, as the current
  //     master plan does; Unicode headings must not be reduced to an empty anchor.
  {
    const heading = '当前串行线·共享 transport 与统一 MCP 能力层（07-22 重排）';
    const { repo } = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({ plan_anchor: `docs/plans/current-plan.md#${encodeURIComponent(heading)}` }),
    }); tmps.push(repo);
    write(repo, 'docs/plans/current-plan.md', `# Current plan\n\n### ${heading}\n`);
    const { report } = currentAudit(repo);
    expect('current percent-encoded Chinese plan anchor -> FIELDS_PRESENT', report.status === 'FIELDS_PRESENT', JSON.stringify(report));
  }

  // 10) Path/anchor and Code Map disagreements remain diagnostics: they do not
  //     auto-select another route or turn an advisory package into a completion claim.
  {
    const { repo } = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({
        plan_anchor: 'docs/plans/missing-plan.md#phase-4',
        existing_before_new: '复用 `unknown-domain.missing-reuse`。',
        capabilities_touched: 'unknown-domain.missing-capability',
      }),
    }); tmps.push(repo);
    const missingResult = currentAudit(repo);
    expect('current missing plan anchor file -> warning', warningCodes(missingResult.report).includes('PLAN_ANCHOR_FILE_MISSING'));
    expect('current unknown capability IDs -> warnings',
      warningCodes(missingResult.report).filter((code) => code === 'MAP_CAPABILITY_NOT_FOUND').length === 2,
      JSON.stringify(missingResult.report.warnings));

    const heading = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({ plan_anchor: 'docs/plans/current-plan.md#missing-phase' }),
    }); tmps.push(heading.repo);
    const headingResult = currentAudit(heading.repo);
    expect('current missing plan anchor heading -> warning', warningCodes(headingResult.report).includes('PLAN_ANCHOR_HEADING_MISSING'));
  }

  // 11) Legacy and needs-confirmation IDs stay visible as warnings; checkpoint
  //     reports their exact IDs and never substitutes a supposedly better route.
  {
    const { repo } = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({
        capabilities_touched: 'workflow-execution-governance.legacy-real-workflow-execution, syn-mcp-supervision.tracked-supervisor-orchestrator',
      }),
    }); tmps.push(repo);
    const { report } = currentAudit(repo);
    expect('current legacy capability -> warning', warningCodes(report).includes('MAP_CAPABILITY_LEGACY'));
    expect('current needs-confirmation capability -> warning', warningCodes(report).includes('MAP_CAPABILITY_NEEDS_CONFIRMATION'));
    expect('current legacy/needs -> preserves declared IDs without replacement',
      JSON.stringify(report).includes('workflow-execution-governance.legacy-real-workflow-execution')
      && JSON.stringify(report).includes('syn-mcp-supervision.tracked-supervisor-orchestrator'));
  }

  // 12) `none` is valid only with a reason; it is not a way to omit the capability
  //     field altogether.
  {
    const missingReason = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({ capabilities_touched: 'none' }),
    }); tmps.push(missingReason.repo);
    const missingReasonResult = currentAudit(missingReason.repo);
    expect('capabilities_touched none without explanation -> warning',
      warningCodes(missingReasonResult.report).includes('CAPABILITIES_TOUCHED_NONE_NEEDS_EXPLANATION'));

    const explained = buildCurrentAlignmentRepo({
      binding: { path: 'tasks/current-important-task.md', mode: 'advisory' },
      fields: alignmentFields({ capabilities_touched: 'none — documentation-only task; no capability boundary changes.' }),
    }); tmps.push(explained.repo);
    const explainedResult = currentAudit(explained.repo);
    expect('capabilities_touched none with explanation -> FIELDS_PRESENT', explainedResult.report.status === 'FIELDS_PRESENT');
  }
} finally {
  for (const t of tmps) fs.rmSync(t, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.ok);
console.log('');
console.log(`Self-test: ${results.length - failed.length}/${results.length} checks passed`);
process.exit(failed.length === 0 ? 0 : 1);
