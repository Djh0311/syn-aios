#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');

const repoRoot = path.resolve(__dirname, '..', '..');
const auditSource = path.join(__dirname, 'maintenance-audit.js');
const mapSource = path.join(__dirname, 'codebase-map.js');

const DOMAIN_IDS = [
  'conversation-transport',
  'syn-mcp-supervision',
  'workflow-execution-governance',
  'persistence-canonical-state',
  'ui-shared-foundation',
  'development-harness',
];

const DEFAULT_ROUTES = [
  'context',
  'context diagnostic',
  'map query',
  'map overlay',
  'map check',
  'checkpoint',
  'shape',
  'stage-k',
  'doctor',
];

function git(root, args) {
  const result = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, 0, `git ${args.join(' ')} failed: ${result.stderr}`);
  return result.stdout.trim();
}

function write(root, relativePath, value) {
  const destination = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, value);
}

function readJson(root, relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), 'utf8'));
}

function writeJson(root, relativePath, value) {
  write(root, relativePath, `${JSON.stringify(value, null, 2)}\n`);
}

function activeBoundary() {
  return {
    mechanical: ['commit-msg catch:', 'config-schema', 'config-check', 'config-policy', 'shape'],
    reportingOnly: ['context', 'checkpoint'],
    explicitTool: ['context diagnostic', 'map query', 'map overlay', 'map check', 'stage-k', 'doctor'],
    legacyIgnored: [
      'memory',
      'task/evidence lifecycle',
      'runtime-doc init',
      'managed Hook/CI init',
      'legacy capability scan',
    ],
  };
}

function configFixture() {
  return {
    activeBoundary: activeBoundary(),
    preWork: {
      recommendedChecks: ['node scripts/harness/project-context.js --target .'],
      strictPathRecommendedChecks: [],
    },
    preCompletion: {
      recommendedChecks: ['node scripts/harness/config-check.js --target . --strict'],
      strictPathRecommendedChecks: [],
    },
    policy: { hooks: { enabled: false }, ci: { required: false } },
    memoryIntegration: { enabled: false },
  };
}

function catalogFixture() {
  const commandRows = DEFAULT_ROUTES.map((route) => `| \`${route}\` | explicit route |`).join('\n');
  return [
    '# Harness catalog',
    '',
    '## Active boundary',
    '',
    '| 分类 | 当前入口 |',
    '| --- | --- |',
    '| `mechanical` | `commit-msg catch:`、config schema/check/policy、shape |',
    '| `reportingOnly` | `context`、`checkpoint` |',
    '| `explicitTool` | `context diagnostic`、Code Map、Stage K、doctor |',
    '| `legacyIgnored` | memory、task/evidence lifecycle、runtime-doc init、managed Hook/CI init、旧 capability scan |',
    '',
    '## 根 CLI',
    '',
    '| 命令 | 说明 |',
    '| --- | --- |',
    commandRows,
    '',
  ].join('\n');
}

function harnessFixture() {
  return [
    '#!/usr/bin/env node',
    `const routes = ${JSON.stringify(DEFAULT_ROUTES)};`,
    "if (process.argv.includes('--help') || process.argv.length === 2) {",
    "  console.log('Current manual entrypoints:');",
    "  for (const route of routes) console.log(`  ${route.padEnd(30)} route`);",
    "  console.log('');",
    "  console.log('Examples:');",
    "  process.exit(0);",
    '}',
    "process.exit(0);",
    '',
  ].join('\n');
}

function authorityFixture() {
  return [
    '# Authority',
    '',
    '## 一级入口',
    '',
    '- AGENTS.md',
    '- CURRENT.md',
    '- README.md',
    '- backlog.md',
    '',
    '## 当前业务路由（默认）',
    '',
    '- decisions/current.md',
    '- docs/plans/current.md',
    '- tasks/current.md',
    '- evidence/current.md',
    '',
    '## Superseded / 停用',
    '',
    '- docs/superseded.md',
    '',
  ].join('\n');
}

function mapIndex() {
  return {
    schemaVersion: 1,
    coverage: 'seed-partial',
    domains: DOMAIN_IDS.map((id) => ({
      id,
      name: id,
      path: `docs/code-map/domains/${id}.json`,
      coverage: 'verified-partial',
    })),
  };
}

function mapDomain(id, commit, sourcePath = 'src/capability.js') {
  return {
    schemaVersion: 1,
    domain: id,
    coverage: 'verified-partial',
    capabilities: [{
      id: `${id}.fixture`,
      domain: id,
      name: `${id} fixture`,
      status: 'active',
      coverage: 'verified-partial',
      canonical: { path: sourcePath, kind: 'fixture source' },
      entrypoints: [],
      publicSymbols: [],
      consumers: [],
      stateOwners: [],
      contracts: [],
      tests: [],
      related: [],
      knownDuplicates: [],
      keywords: ['fixture'],
      verifiedAtCommit: commit,
    }],
  };
}

function writeMap(root, commit) {
  writeJson(root, 'docs/code-map/index.json', mapIndex());
  for (const id of DOMAIN_IDS) {
    writeJson(root, `docs/code-map/domains/${id}.json`, mapDomain(id, commit));
  }
}

function createFixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'maintenance-audit-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  git(root, ['init', '-q']);
  git(root, ['config', 'user.email', 'maintenance-audit@example.test']);
  git(root, ['config', 'user.name', 'Maintenance audit fixture']);

  write(root, 'AGENTS.md', '# Rules\n');
  write(root, 'CURRENT.md', '# Current\n\n- active\n');
  write(root, 'README.md', '# Product\n');
  write(root, 'backlog.md', '# Backlog\n');
  write(root, 'decisions/current.md', '# Current decision\n');
  write(root, 'docs/plans/current.md', '# Current plan\n');
  write(root, 'tasks/current.md', '# Current task\n');
  write(root, 'evidence/current.md', '# Current evidence\n');
  write(root, 'docs/superseded.md', '# Superseded\n');
  write(root, 'AUTHORITY.md', authorityFixture());
  writeJson(root, 'docs/project-context.json', {
    schemaVersion: 1,
    ruleEntry: 'AGENTS.md',
    authorityIndex: 'AUTHORITY.md',
    decision: 'decisions/current.md',
    taskPackage: 'tasks/current.md',
    plan: 'docs/plans/current.md',
  });
  writeJson(root, 'harness.config.json', configFixture());
  writeJson(root, 'harness.config.example.json', configFixture());
  write(root, 'docs/harness-catalog.md', catalogFixture());
  write(root, 'scripts/harness/harness.js', harnessFixture());
  write(root, '.githooks/commit-msg', '#!/bin/sh\ngrep -q "catch:" "$1"\n');
  write(root, 'src/capability.js', 'export const capability = true;\n');

  git(root, ['add', '.']);
  git(root, ['commit', '-qm', 'fixture baseline']);
  const commit = git(root, ['rev-parse', 'HEAD']);
  writeMap(root, commit);
  fs.mkdirSync(path.join(root, 'scripts', 'harness'), { recursive: true });
  fs.copyFileSync(auditSource, path.join(root, 'scripts', 'harness', 'maintenance-audit.js'));
  fs.copyFileSync(mapSource, path.join(root, 'scripts', 'harness', 'codebase-map.js'));
  return { root, commit };
}

function runAudit(root, args = []) {
  const result = spawnSync(process.execPath, [
    path.join(root, 'scripts', 'harness', 'maintenance-audit.js'),
    '--target',
    root,
    '--json',
    ...args,
  ], {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 1024 * 1024,
  });
  return {
    status: result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    report: JSON.parse(result.stdout),
  };
}

function findingCodes(report) {
  return report.checks.flatMap((check) => check.findings.map((finding) => finding.code));
}

test('baseline audit passes six checks with bounded output and ignores a large dirty overlay', (t) => {
  const { root } = createFixture(t);
  const dirtyDir = path.join(root, 'scratch', 'untracked');
  fs.mkdirSync(dirtyDir, { recursive: true });
  for (let index = 0; index < 240; index += 1) {
    fs.writeFileSync(path.join(dirtyDir, `very-long-untracked-path-${String(index).padStart(4, '0')}-${'x'.repeat(120)}.txt`), 'dirty\n');
  }

  const result = runAudit(root);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.report.status, 'OK');
  assert.deepEqual(result.report.checks.map((check) => check.id), [
    'authority',
    'project-context',
    'current',
    'code-map',
    'active-boundary',
    'legacy-consumers',
  ]);
  assert.ok(Buffer.byteLength(result.stdout) < 64 * 1024);
  assert.equal(result.stdout.includes('very-long-untracked-path'), false);
  assert.equal(result.report.readOnly, true);
  assert.equal(result.report.checks.find((check) => check.id === 'legacy-consumers').metrics.activeConsumerCount, 0);
});

test('authority check catches broken current links, duplicate current entries, and superseded overlap', (t) => {
  const { root } = createFixture(t);
  write(root, 'AUTHORITY.md', authorityFixture()
    .replace('- evidence/current.md', '- evidence/current.md\n- decisions/current.md\n- docs/missing.md')
    .replace('- docs/superseded.md', '- docs/superseded.md\n- decisions/current.md'));

  const result = runAudit(root);
  assert.equal(result.status, 1);
  const codes = findingCodes(result.report);
  assert.ok(codes.includes('AUTHORITY_POINTER_MISSING'));
  assert.ok(codes.includes('AUTHORITY_DUPLICATE_CURRENT'));
  assert.ok(codes.includes('AUTHORITY_SUPERSEDED_CURRENT'));
});

test('project-context and CURRENT budget or pointer drift is reported without rewriting either file', (t) => {
  const { root } = createFixture(t);
  const context = readJson(root, 'docs/project-context.json');
  context.plan = 'docs/plans/missing.md';
  context.padding = 'x'.repeat(5000);
  writeJson(root, 'docs/project-context.json', context);
  write(root, 'CURRENT.md', `${Array.from({ length: 32 }, (_, index) => `line ${index}`).join('\n')}\n`);

  const result = runAudit(root);
  assert.equal(result.status, 1);
  const codes = findingCodes(result.report);
  assert.ok(codes.includes('PROJECT_CONTEXT_BYTES_EXCEEDED'));
  assert.ok(codes.includes('PROJECT_CONTEXT_POINTER_MISSING'));
  assert.ok(codes.includes('CURRENT_LINES_EXCEEDED'));
});

test('Code Map validation reports schema/path errors and stale verified commits separately', (t) => {
  const { root } = createFixture(t);
  write(root, 'src/capability.js', 'export const capability = false;\n');
  git(root, ['add', 'src/capability.js']);
  git(root, ['commit', '-qm', 'advance source']);

  let result = runAudit(root);
  assert.equal(result.status, 0);
  assert.ok(findingCodes(result.report).includes('STALE_VERIFIED_COMMIT'));

  const domainPath = 'docs/code-map/domains/conversation-transport.json';
  const domain = readJson(root, domainPath);
  domain.capabilities[0].canonical.path = 'src/missing.js';
  writeJson(root, domainPath, domain);
  result = runAudit(root);
  assert.equal(result.status, 1);
  assert.ok(findingCodes(result.report).includes('CODE_MAP_INVALID'));

  write(root, 'docs/code-map/index.json', '{not json\n');
  result = runAudit(root);
  assert.equal(result.status, 1);
  assert.ok(findingCodes(result.report).includes('CODE_MAP_INVALID'));
});

test('staged canonical rename and deletion return drift with affected capabilities', (t) => {
  const renamed = createFixture(t);
  git(renamed.root, ['mv', 'src/capability.js', 'src/capability-renamed.js']);

  let result = runAudit(renamed.root);
  assert.equal(result.status, 1);
  let findings = result.report.checks.find((check) => check.id === 'code-map').findings;
  const rename = findings.find((finding) => finding.code === 'STAGED_RENAME_AFFECTS_CAPABILITY');
  assert.ok(rename);
  assert.equal(rename.capabilityId, 'conversation-transport.fixture');
  assert.equal(rename.from, 'src/capability.js');
  assert.equal(rename.to, 'src/capability-renamed.js');

  const deleted = createFixture(t);
  git(deleted.root, ['rm', '-q', 'src/capability.js']);

  result = runAudit(deleted.root);
  assert.equal(result.status, 1);
  findings = result.report.checks.find((check) => check.id === 'code-map').findings;
  const deletion = findings.find((finding) => finding.code === 'STAGED_DELETE_AFFECTS_CAPABILITY');
  assert.ok(deletion);
  assert.equal(deletion.capabilityId, 'conversation-transport.fixture');
  assert.equal(deletion.path, 'src/capability.js');
});

test('active boundary drift across CLI, config, and catalog is rejected', (t) => {
  const { root } = createFixture(t);
  const config = readJson(root, 'harness.config.json');
  config.activeBoundary.explicitTool = config.activeBoundary.explicitTool.filter((entry) => entry !== 'doctor');
  writeJson(root, 'harness.config.json', config);
  write(root, 'docs/harness-catalog.md', catalogFixture().replace('Code Map', 'mapping removed'));

  const result = runAudit(root);
  assert.equal(result.status, 1);
  const codes = findingCodes(result.report);
  assert.ok(codes.includes('DEFAULT_CLI_BOUNDARY_DRIFT'));
  assert.ok(codes.includes('CATALOG_ACTIVE_BOUNDARY_DRIFT'));
});

test('empty or unparseable default CLI help returns a boundary drift', (t) => {
  const empty = createFixture(t);
  write(empty.root, 'scripts/harness/harness.js', [
    '#!/usr/bin/env node',
    "console.log('Current manual entrypoints:');",
    "console.log('');",
    "console.log('Examples:');",
    '',
  ].join('\n'));

  let result = runAudit(empty.root);
  assert.equal(result.status, 1);
  assert.ok(findingCodes(result.report).includes('DEFAULT_CLI_BOUNDARY_DRIFT'));

  const unparseable = createFixture(t);
  write(unparseable.root, 'scripts/harness/harness.js', "#!/usr/bin/env node\nconsole.log('not a route table');\n");

  result = runAudit(unparseable.root);
  assert.equal(result.status, 1);
  assert.ok(findingCodes(result.report).includes('DEFAULT_CLI_BOUNDARY_DRIFT'));
});

test('only active automatic legacy consumers block the audit', (t) => {
  const { root } = createFixture(t);
  const config = readJson(root, 'harness.config.json');
  config.preWork.recommendedChecks.push('node scripts/harness/memory-maintenance.js');
  writeJson(root, 'harness.config.json', config);

  const result = runAudit(root);
  assert.equal(result.status, 1);
  const legacyCheck = result.report.checks.find((check) => check.id === 'legacy-consumers');
  assert.equal(legacyCheck.metrics.activeConsumerCount, 1);
  assert.ok(findingCodes(result.report).includes('ACTIVE_LEGACY_CONSUMER'));
});
