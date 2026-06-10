#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const fixturesRoot = path.join(__dirname, 'fixtures');
const expectedFixtures = [
  'unsafe-aggregate-config',
  'stale-evidence',
  'http-only-browser-evidence',
  'malformed-manifest',
  'mixed-templates-docs-classification',
  'node-project',
  'python-project',
  'go-project',
  'existing-ci-project',
  'lifecycle-runtime-docs',
  'security-redaction',
  'context-pack-runtime-docs',
  'skill-recommend',
  'evidence-retention',
  'task-package-schema'
];

function parseArgs(argv) {
  const args = {
    target: fixturesRoot,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = path.resolve(argv[++i]);
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function exists(root, relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}

function readText(root, relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

function readJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function fixtureDir(root, name) {
  return path.join(root, name);
}

function walk(dir, files) {
  if (!fs.existsSync(dir)) return;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
}

function checkFixtureManifest(report, root, name) {
  const file = path.join(fixtureDir(root, name), 'fixture.json');
  if (!fs.existsSync(file)) {
    add(report, 'fail', `${name}: fixture.json is missing`);
    return;
  }

  try {
    const data = readJsonFile(file);
    const missing = ['name', 'description', 'kind', 'expectedFindings', 'recommendedCommands']
      .filter((key) => data[key] === undefined);
    if (missing.length > 0) add(report, 'fail', `${name}: fixture.json missing key(s): ${missing.join(', ')}`);
    else add(report, 'pass', `${name}: fixture metadata is present`);

    if (data.name !== name) add(report, 'fail', `${name}: fixture.json name does not match directory`);
    if (!Array.isArray(data.expectedFindings) || data.expectedFindings.length === 0) {
      add(report, 'fail', `${name}: expectedFindings must be a non-empty array`);
    }
    if (!Array.isArray(data.recommendedCommands) || data.recommendedCommands.length === 0) {
      add(report, 'fail', `${name}: recommendedCommands must be a non-empty array`);
    }
  } catch (error) {
    add(report, 'fail', `${name}: fixture.json could not be parsed (${error.message})`);
  }
}

function checkNoSourceRootDocs(report, root) {
  const sourceRootDocs = path.resolve(__dirname, '..', '..', 'docs');
  if (fs.existsSync(sourceRootDocs)) {
    add(report, 'fail', `Source package root docs/** exists unexpectedly: ${sourceRootDocs}`);
  } else {
    add(report, 'pass', 'Source package root docs/** is absent');
  }

  const files = [];
  walk(root, files);
  const outsideFixtures = files.filter((file) => !path.resolve(file).startsWith(path.resolve(root) + path.sep));
  if (outsideFixtures.length > 0) {
    add(report, 'fail', `Fixture scan escaped root: ${outsideFixtures.map((file) => rel(root, file)).join(', ')}`);
  }
}

function checkUnsafeAggregateConfig(report, root) {
  const name = 'unsafe-aggregate-config';
  const dir = fixtureDir(root, name);
  const configPath = path.join(dir, 'harness.config.json');
  if (!fs.existsSync(configPath)) {
    add(report, 'fail', `${name}: harness.config.json is missing`);
    return;
  }

  const config = readJsonFile(configPath);
  const commands = []
    .concat(config.preCompletion && config.preCompletion.recommendedChecks || [])
    .concat(config.preCompletion && config.preCompletion.strictPathRecommendedChecks || []);
  const hasShellSyntax = commands.some((command) => /[|;&<>`$()]/.test(command));
  const hasNonHarness = commands.some((command) => !/^node scripts\/harness\/[A-Za-z0-9._-]+\.js(?:\s|$)/.test(command));
  const hasSelfReference = commands.some((command) => /^node scripts\/harness\/pre-completion\.js(?:\s|$)/.test(command));

  if (hasShellSyntax) add(report, 'pass', `${name}: unsafe shell syntax fixture present`);
  else add(report, 'fail', `${name}: expected unsafe shell syntax command`);
  if (hasNonHarness) add(report, 'pass', `${name}: non-harness command fixture present`);
  else add(report, 'fail', `${name}: expected non-harness command`);
  if (hasSelfReference) add(report, 'pass', `${name}: aggregate self-reference fixture present`);
  else add(report, 'fail', `${name}: expected aggregate self-reference command`);
}

function checkStaleEvidence(report, root) {
  const name = 'stale-evidence';
  const dir = fixtureDir(root, name);
  const summary = 'docs/evidence/stale-task/summary.md';
  const output = 'docs/evidence/stale-task/test-output.md';
  const stalePattern = /(?:createdAt|recordedAt): 2000-01-01T00:00:00\.000Z/;

  if (exists(dir, summary) && stalePattern.test(readText(dir, summary))) {
    add(report, 'pass', `${name}: stale summary timestamp present`);
  } else {
    add(report, 'fail', `${name}: stale summary timestamp missing`);
  }

  if (exists(dir, output) && stalePattern.test(readText(dir, output))) {
    add(report, 'pass', `${name}: stale command timestamp present`);
  } else {
    add(report, 'fail', `${name}: stale command timestamp missing`);
  }
}

function checkHttpOnlyBrowserEvidence(report, root) {
  const name = 'http-only-browser-evidence';
  const dir = fixtureDir(root, name);
  const browser = 'docs/evidence/ui-task/browser-check.md';
  const consoleNetwork = 'docs/evidence/ui-task/console-network.md';
  const httpOnlyPattern = /HTTP-only|reachability/i;

  if (exists(dir, browser) && httpOnlyPattern.test(readText(dir, browser))) {
    add(report, 'pass', `${name}: HTTP-only browser-check fixture present`);
  } else {
    add(report, 'fail', `${name}: HTTP-only browser-check marker missing`);
  }

  if (exists(dir, consoleNetwork) && httpOnlyPattern.test(readText(dir, consoleNetwork))) {
    add(report, 'pass', `${name}: HTTP-only console-network fixture present`);
  } else {
    add(report, 'fail', `${name}: HTTP-only console-network marker missing`);
  }
}

function checkMalformedManifest(report, root) {
  const name = 'malformed-manifest';
  const manifest = path.join(fixtureDir(root, name), '.harness', 'manifest.json');
  if (!fs.existsSync(manifest)) {
    add(report, 'fail', `${name}: malformed manifest file is missing`);
    return;
  }

  try {
    JSON.parse(fs.readFileSync(manifest, 'utf8'));
    add(report, 'fail', `${name}: manifest unexpectedly parses as JSON`);
  } catch (error) {
    add(report, 'pass', `${name}: manifest is intentionally malformed`);
  }
}

function checkMixedClassification(report, root) {
  const name = 'mixed-templates-docs-classification';
  const dir = fixtureDir(root, name);
  const required = [
    'AGENTS.md',
    'codex-multi-agent-safe-collaboration.md',
    'skills/using-superpowers/SKILL.md',
    'docs/current-state.md',
    'docs/evidence/README.md',
    'docs/evidence/mixed-task/summary.md',
    'templates/docs/evidence/README.md'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: mixed templates/docs installed-project shape present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
  }

  const sourceOnlySignals = [
    'plans/2026-05-12-harness-upgrade-phase-1.md',
    'scripts/harness/rules-lint.js',
    'scripts/harness/install-harness.js',
    'scripts/harness/sync-harness.js',
    'templates/docs/current-state.md'
  ];
  const accidentalSourceSignals = sourceOnlySignals.filter((relativePath) => exists(dir, relativePath));
  if (accidentalSourceSignals.length === 0) {
    add(report, 'pass', `${name}: source-package-only signals are absent`);
  } else {
    add(report, 'fail', `${name}: accidental source-package signal(s): ${accidentalSourceSignals.join(', ')}`);
  }
}

function checkNodeProject(report, root) {
  const name = 'node-project';
  const dir = fixtureDir(root, name);
  const packageJson = 'package.json';

  if (!exists(dir, packageJson)) {
    add(report, 'fail', `${name}: package.json is missing`);
    return;
  }

  const manifest = readJsonFile(path.join(dir, packageJson));
  const scripts = manifest.scripts || {};
  const requiredScripts = ['test', 'build', 'lint'];
  const missing = requiredScripts.filter((script) => typeof scripts[script] !== 'string' || scripts[script].length === 0);

  if (missing.length === 0) {
    add(report, 'pass', `${name}: package.json scripts fixture present`);
  } else {
    add(report, 'fail', `${name}: missing package script(s): ${missing.join(', ')}`);
  }

  if (manifest.private === true) add(report, 'pass', `${name}: fixture package is private`);
  else add(report, 'fail', `${name}: package.json must mark the fixture private`);
}

function checkPythonProject(report, root) {
  const name = 'python-project';
  const dir = fixtureDir(root, name);
  const pyproject = 'pyproject.toml';

  if (!exists(dir, pyproject)) {
    add(report, 'fail', `${name}: pyproject.toml is missing`);
    return;
  }

  const content = readText(dir, pyproject);
  if (/\[project\][\s\S]*name\s*=/.test(content)) {
    add(report, 'pass', `${name}: pyproject project metadata present`);
  } else {
    add(report, 'fail', `${name}: pyproject project metadata missing`);
  }

  if (/\[tool\.pytest\.ini_options\]/.test(content)) {
    add(report, 'pass', `${name}: pytest configuration signal present`);
  } else {
    add(report, 'fail', `${name}: pytest configuration signal missing`);
  }
}

function checkGoProject(report, root) {
  const name = 'go-project';
  const dir = fixtureDir(root, name);
  const gomod = 'go.mod';

  if (!exists(dir, gomod)) {
    add(report, 'fail', `${name}: go.mod is missing`);
    return;
  }

  const content = readText(dir, gomod);
  if (/^module\s+\S+/m.test(content)) {
    add(report, 'pass', `${name}: go module declaration present`);
  } else {
    add(report, 'fail', `${name}: go module declaration missing`);
  }

  if (/^go\s+\d+\.\d+/m.test(content)) {
    add(report, 'pass', `${name}: go version declaration present`);
  } else {
    add(report, 'fail', `${name}: go version declaration missing`);
  }
}

function checkExistingCiProject(report, root) {
  const name = 'existing-ci-project';
  const dir = fixtureDir(root, name);
  const workflow = '.github/workflows/harness.yml';

  if (!exists(dir, workflow)) {
    add(report, 'fail', `${name}: existing GitHub Actions workflow is missing`);
    return;
  }

  const content = readText(dir, workflow);
  if (/Existing Harness Workflow/.test(content) && /preserve me/.test(content)) {
    add(report, 'pass', `${name}: project-owned CI workflow fixture present`);
  } else {
    add(report, 'fail', `${name}: existing CI preservation marker missing`);
  }

  if (!/HARNESS MANAGED/.test(content)) {
    add(report, 'pass', `${name}: workflow is not harness-managed`);
  } else {
    add(report, 'fail', `${name}: workflow should model non-managed existing CI`);
  }
}

function checkLifecycleRuntimeDocs(report, root) {
  const name = 'lifecycle-runtime-docs';
  const dir = fixtureDir(root, name);
  const required = [
    'docs/task-queue.md',
    'docs/current-state.md',
    'docs/evidence/README.md',
    'docs/evidence/lifecycle-task/summary.md',
    'docs/evidence/lifecycle-task/task-status.md'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: runtime docs and evidence files present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
    return;
  }

  const taskQueue = readText(dir, 'docs/task-queue.md');
  const currentState = readText(dir, 'docs/current-state.md');
  const summary = readText(dir, 'docs/evidence/lifecycle-task/summary.md');
  const taskStatus = readText(dir, 'docs/evidence/lifecycle-task/task-status.md');

  if (/fixture-lifecycle-task/.test(taskQueue) && /docs\/evidence\/lifecycle-task\/summary\.md/.test(taskQueue)) {
    add(report, 'pass', `${name}: task queue links lifecycle evidence`);
  } else {
    add(report, 'fail', `${name}: task queue lifecycle evidence link missing`);
  }

  if (/Current task:\s*`fixture-lifecycle-task`/.test(currentState)) {
    add(report, 'pass', `${name}: current state references lifecycle task`);
  } else {
    add(report, 'fail', `${name}: current state lifecycle task reference missing`);
  }

  if (/slug:\s*lifecycle-task/.test(summary) && /status:\s*pass/.test(summary)) {
    add(report, 'pass', `${name}: lifecycle evidence summary metadata present`);
  } else {
    add(report, 'fail', `${name}: lifecycle evidence summary metadata missing`);
  }

  if (/task-status\.js/.test(taskStatus) && /Result:/.test(taskStatus)) {
    add(report, 'pass', `${name}: lifecycle task-status evidence present`);
  } else {
    add(report, 'fail', `${name}: lifecycle task-status evidence missing`);
  }
}

function checkSecurityRedaction(report, root) {
  const name = 'security-redaction';
  const dir = fixtureDir(root, name);
  const required = [
    'fixture.json',
    'input.txt'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: security fixture files present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
    return;
  }

  const input = readText(dir, 'input.txt');
  if (/Ignore previous instructions/i.test(input)) {
    add(report, 'pass', `${name}: prompt-injection fixture present`);
  } else {
    add(report, 'fail', `${name}: prompt-injection fixture missing`);
  }

  if (/ghp_[A-Za-z0-9_]+/.test(input) && /sk-[A-Za-z0-9_]+/.test(input)) {
    add(report, 'pass', `${name}: secret-like fixture values present`);
  } else {
    add(report, 'fail', `${name}: secret-like fixture values missing`);
  }
}

function checkContextPackRuntimeDocs(report, root) {
  const name = 'context-pack-runtime-docs';
  const dir = fixtureDir(root, name);
  const required = [
    'docs/current-state.md',
    'docs/task-queue.md',
    'docs/decisions.md',
    'docs/open-questions.md',
    'docs/context-checkpoints.md',
    'docs/sprint-contract.md',
    'docs/evidence/context-pack-task/summary.md',
    'docs/evidence/context-pack-task/test-output.md'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: runtime docs for context-pack fixture present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
    return;
  }

  const taskQueue = readText(dir, 'docs/task-queue.md');
  const currentState = readText(dir, 'docs/current-state.md');
  const evidenceSummary = readText(dir, 'docs/evidence/context-pack-task/summary.md');

  if (/T-CONTEXT-PACK/.test(taskQueue) && /context-pack-task/.test(taskQueue)) {
    add(report, 'pass', `${name}: task queue includes task-id and slug`);
  } else {
    add(report, 'fail', `${name}: task queue task-id or slug missing`);
  }

  if (/TL;DR/i.test(currentState) && /T-CONTEXT-PACK/.test(currentState)) {
    add(report, 'pass', `${name}: current state includes TL;DR task context`);
  } else {
    add(report, 'fail', `${name}: current state TL;DR task context missing`);
  }

  if (/slug:\s*context-pack-task/.test(evidenceSummary) && /status:\s*pass/.test(evidenceSummary)) {
    add(report, 'pass', `${name}: context-pack evidence summary metadata present`);
  } else {
    add(report, 'fail', `${name}: context-pack evidence summary metadata missing`);
  }
}

function checkSkillRecommend(report, root) {
  const name = 'skill-recommend';
  const dir = fixtureDir(root, name);
  const required = [
    'fixture.json',
    'skills/custom-review/SKILL.md'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: skill recommendation fixture files present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
    return;
  }

  const skill = readText(dir, 'skills/custom-review/SKILL.md');
  if (/name:\s*custom-review/.test(skill) && /description:\s*Use when reviewing CLI utility behavior/.test(skill)) {
    add(report, 'pass', `${name}: custom skill metadata present`);
  } else {
    add(report, 'fail', `${name}: custom skill metadata missing`);
  }
}

function checkEvidenceRetention(report, root) {
  const name = 'evidence-retention';
  const dir = fixtureDir(root, name);
  const required = [
    'docs/evidence/old-pass/summary.md',
    'docs/evidence/old-pass/test-output.md',
    'docs/evidence/recent-pass/summary.md',
    'docs/evidence/strict-keep/summary.md',
    'docs/evidence/failed-keep/summary.md',
    'docs/evidence/security-keep/summary.md'
  ];
  const missing = required.filter((relativePath) => !exists(dir, relativePath));

  if (missing.length === 0) {
    add(report, 'pass', `${name}: retention evidence fixture files present`);
  } else {
    add(report, 'fail', `${name}: missing file(s): ${missing.join(', ')}`);
    return;
  }

  const oldOutput = readText(dir, 'docs/evidence/old-pass/test-output.md');
  const outputLines = oldOutput.split(/\r?\n/).filter((line) => /^line \d+ old passing output$/.test(line));
  if (outputLines.length >= 20) add(report, 'pass', `${name}: large command output fixture present`);
  else add(report, 'fail', `${name}: large command output fixture is too small`);

  const strict = readText(dir, 'docs/evidence/strict-keep/summary.md');
  const failed = readText(dir, 'docs/evidence/failed-keep/summary.md');
  const security = readText(dir, 'docs/evidence/security-keep/summary.md');
  if (/kind:\s*strict/.test(strict)) add(report, 'pass', `${name}: strict evidence marker present`);
  else add(report, 'fail', `${name}: strict evidence marker missing`);
  if (/result:\s*fail/.test(failed)) add(report, 'pass', `${name}: failed evidence marker present`);
  else add(report, 'fail', `${name}: failed evidence marker missing`);
  if (/kind:\s*security/.test(security)) add(report, 'pass', `${name}: security evidence marker present`);
  else add(report, 'fail', `${name}: security evidence marker missing`);
}

function checkTaskPackageSchema(report, root) {
  const name = 'task-package-schema';
  const dir = fixtureDir(root, name);
  const jsonFile = 'docs/task-packages/T-SCHEMA-FIXTURE.json';
  const mdFile = 'docs/task-packages/T-SCHEMA-FIXTURE.md';
  const requiredFields = [
    'id',
    'mission',
    'path',
    'readScope',
    'writeScope',
    'forbiddenScope',
    'acceptance',
    'verification',
    'riskTags',
    'inputs',
    'relatedMistakes'
  ];

  if (!exists(dir, jsonFile) || !exists(dir, mdFile)) {
    add(report, 'fail', `${name}: task package JSON/Markdown pair missing`);
    return;
  }

  const data = readJsonFile(path.join(dir, jsonFile));
  const missing = requiredFields.filter((field) => data[field] === undefined);
  if (missing.length === 0) add(report, 'pass', `${name}: required structured fields present`);
  else add(report, 'fail', `${name}: missing required field(s): ${missing.join(', ')}`);

  if (data.id === 'T-SCHEMA-FIXTURE' && data.path === 'strict') {
    add(report, 'pass', `${name}: fixture package id and path present`);
  } else {
    add(report, 'fail', `${name}: fixture package id/path mismatch`);
  }

  const rendered = readText(dir, mdFile);
  if (/## Mission/.test(rendered) && /## Allowed Read Paths/.test(rendered) && /## Required Output/.test(rendered)) {
    add(report, 'pass', `${name}: rendered Markdown has task-package sections`);
  } else {
    add(report, 'fail', `${name}: rendered Markdown missing expected sections`);
  }
}

function buildReport(args) {
  const root = path.resolve(args.target);
  const report = {
    target: root,
    pass: [],
    warn: [],
    fail: [],
    details: {
      expectedFixtures
    }
  };

  if (!fs.existsSync(root) || !fs.statSync(root).isDirectory()) {
    add(report, 'fail', `Fixture root must be an existing directory: ${root}`);
    return report;
  }

  for (const name of expectedFixtures) {
    if (fs.existsSync(fixtureDir(root, name))) add(report, 'pass', `${name}: fixture directory exists`);
    else add(report, 'fail', `${name}: fixture directory missing`);
    checkFixtureManifest(report, root, name);
  }

  checkNoSourceRootDocs(report, root);
  checkUnsafeAggregateConfig(report, root);
  checkStaleEvidence(report, root);
  checkHttpOnlyBrowserEvidence(report, root);
  checkMalformedManifest(report, root);
  checkMixedClassification(report, root);
  checkNodeProject(report, root);
  checkPythonProject(report, root);
  checkGoProject(report, root);
  checkExistingCiProject(report, root);
  checkLifecycleRuntimeDocs(report, root);
  checkSecurityRedaction(report, root);
  checkContextPackRuntimeDocs(report, root);
  checkSkillRecommend(report, root);
  checkEvidenceRetention(report, root);
  checkTaskPackageSchema(report, root);

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness fixture check: ${report.target}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
