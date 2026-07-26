#!/usr/bin/env node

const assert = require('assert/strict');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');
const test = require('node:test');

const repoRoot = path.resolve(__dirname, '..', '..');
const harnessPath = path.join(repoRoot, 'scripts', 'harness', 'harness.js');
const configTools = ['config-schema.js', 'config-check.js', 'config-policy.js'];

const defaultRoutes = [
  ['context', 'project-context.js', []],
  ['context diagnostic', 'project-context.js', ['--diagnostic']],
  ['map query', 'codebase-map.js', ['query']],
  ['map overlay', 'codebase-map.js', ['overlay']],
  ['map check', 'codebase-map.js', ['check']],
  ['checkpoint', 'checkpoint-audit.js', ['--current']],
  ['shape', 'workbench-shape-gate.js', []],
  ['stage-k', 'stage-k-architecture-gate.js', []],
  ['doctor', 'harness-doctor.js', []]
];

const defaultBoundaryCategories = {
  context: 'reportingOnly',
  'context diagnostic': 'explicitTool',
  'map query': 'explicitTool',
  'map overlay': 'explicitTool',
  'map check': 'explicitTool',
  checkpoint: 'reportingOnly',
  shape: 'mechanical',
  'stage-k': 'explicitTool',
  doctor: 'explicitTool'
};

const compatibilityRoutes = [
  ['doctor', 'harness-doctor.js'],
  ['pre-work', 'pre-work.js'],
  ['pre-completion', 'pre-completion.js'],
  ['init config', 'config-init.js'],
  ['init docs', 'runtime-docs-init.js'],
  ['init hooks', 'hook-install.js'],
  ['init ci', 'ci-init.js'],
  ['profile', 'project-profile.js'],
  ['policy', 'config-policy.js'],
  ['mistake query', 'mistake-query.js'],
  ['memory candidate new', 'memory-candidate-new.js'],
  ['memory candidate lint', 'memory-candidate-lint.js'],
  ['memory review', 'memory-review.js'],
  ['memory stale-check', 'memory-stale-check.js'],
  ['memory maintenance', 'memory-maintenance.js'],
  ['memory agentmemory query', 'memory-agentmemory-query.js'],
  ['memory agentmemory save', 'memory-agentmemory-save.js'],
  ['task start', 'task-start.js'],
  ['task finish', 'task-finish.js'],
  ['task status', 'task-status.js'],
  ['task risk', 'task-risk.js'],
  ['task package new', 'task-package-new.js'],
  ['task package lint', 'task-package-lint.js'],
  ['evidence new', 'evidence-new.js'],
  ['evidence retention', 'evidence-retention.js'],
  ['evidence compact', 'evidence-compact.js'],
  ['evidence index', 'evidence-index.js'],
  ['evidence query', 'evidence-query.js'],
  ['skill recommend', 'skill-recommend.js'],
  ['security scan', 'security-scan.js'],
  ['eval', 'eval-runner.js'],
  ['verify plan', 'verification-plan.js'],
  ['verify run', 'verification-runner.js'],
  ['verify suite', 'verification-suite.js'],
  ['capabilities', 'capability-map.js']
];

const boundaryKeys = ['mechanical', 'reportingOnly', 'explicitTool', 'legacyIgnored'];

function runNode(cwd, script, argv) {
  return spawnSync(process.execPath, [script, ...argv], {
    cwd,
    encoding: 'utf8',
    env: Object.assign({}, process.env, { HARNESS_PHASE5_STUB_STATUS: '23' })
  });
}

function parseJsonOutput(result) {
  assert.equal(result.stderr, '', `unexpected stderr: ${result.stderr}`);
  return JSON.parse(result.stdout);
}

function parseHelpAliases(stdout) {
  return stdout
    .split('\n')
    .map((line) => {
      const match = line.match(/^  ([a-z][a-z0-9-]*(?: [a-z][a-z0-9-]*)*)\s{2,}/);
      return match ? match[1] : null;
    })
    .filter(Boolean);
}

function createRouterFixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'harness-phase5-router-'));
  const harnessDir = path.join(root, 'scripts', 'harness');
  fs.mkdirSync(harnessDir, { recursive: true });
  fs.copyFileSync(harnessPath, path.join(harnessDir, 'harness.js'));

  const scriptNames = new Set([
    ...compatibilityRoutes.map(([, script]) => script),
    ...defaultRoutes.map(([, script]) => script)
  ]);
  for (const scriptName of scriptNames) {
    fs.writeFileSync(
      path.join(harnessDir, scriptName),
      [
        "const path = require('path');",
        "console.log(JSON.stringify({ script: path.basename(__filename), argv: process.argv.slice(2) }));",
        'process.exit(Number(process.env.HARNESS_PHASE5_STUB_STATUS || 23));',
        ''
      ].join('\n')
    );
  }

  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return { root, harness: path.join(harnessDir, 'harness.js') };
}

function validBoundary() {
  return {
    mechanical: ['commit-msg catch:', 'config-schema', 'config-check', 'config-policy', 'shape'],
    reportingOnly: ['context', 'checkpoint'],
    explicitTool: ['context diagnostic', 'map query', 'map overlay', 'map check', 'stage-k', 'doctor'],
    legacyIgnored: [
      'memory',
      'task/evidence lifecycle',
      'runtime-doc init',
      'managed Hook/CI init',
      'legacy capability scan'
    ]
  };
}

function readProjectConfig() {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, 'harness.config.json'), 'utf8'));
}

function writeConfigFixture(t, mutate) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'harness-phase5-config-'));
  const configPath = path.join(root, 'harness.config.json');
  const data = readProjectConfig();
  data.activeBoundary = validBoundary();
  mutate(data);
  fs.writeFileSync(configPath, `${JSON.stringify(data, null, 2)}\n`);
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return configPath;
}

function runConfig(tool, configPath) {
  return runNode(repoRoot, path.join(repoRoot, 'scripts', 'harness', tool), [
    '--target',
    repoRoot,
    '--config',
    configPath,
    '--strict',
    '--json'
  ]);
}

function assertBoundaryFailure(result, code) {
  assert.notEqual(result.status, 0, 'strict validation should fail');
  const report = parseJsonOutput(result);
  assert.ok(
    report.fail.some((message) => message.includes(code)),
    `expected ${code} in: ${JSON.stringify(report.fail)}`
  );
}

test('default help exposes exactly the current nine entrypoints', () => {
  const result = runNode(repoRoot, harnessPath, ['--help']);
  assert.equal(result.status, 0);
  assert.deepEqual(parseHelpAliases(result.stdout), defaultRoutes.map(([alias]) => alias));

  const lower = result.stdout.toLowerCase();
  for (const forbidden of ['memory', 'task', 'evidence', 'init docs', 'init hooks', 'init ci', 'capabilit', 'maintenance']) {
    assert.equal(lower.includes(forbidden), false, `default help must not include ${forbidden}`);
  }
});

test('legacy help keeps only the 34 compatibility routes', () => {
  const result = runNode(repoRoot, harnessPath, ['--legacy']);
  assert.equal(result.status, 0);
  assert.match(result.stdout, /compatibility/i);
  assert.deepEqual(parseHelpAliases(result.stdout), compatibilityRoutes.slice(1).map(([alias]) => alias));
  assert.equal(parseHelpAliases(result.stdout).includes('doctor'), false);
});

test('all original 35 routes preserve target, arguments, and exit status', (t) => {
  const fixture = createRouterFixture(t);
  for (const [alias, script] of compatibilityRoutes) {
    const result = runNode(fixture.root, fixture.harness, [...alias.split(' '), '--opaque', 'value']);
    assert.equal(result.status, 23, `${alias} must preserve stub exit status`);
    assert.equal(result.stderr, '', `${alias} must not add stderr text`);
    assert.deepEqual(JSON.parse(result.stdout), {
      script,
      argv: ['--opaque', 'value']
    });
  }
});

test('new default aliases forward fixed prefixes and use longest matching', (t) => {
  const fixture = createRouterFixture(t);
  for (const [alias, script, prefix] of defaultRoutes) {
    const result = runNode(fixture.root, fixture.harness, [...alias.split(' '), '--target', '.']);
    assert.equal(result.status, 23, `${alias} must preserve stub exit status`);
    assert.equal(result.stderr, '', `${alias} must not add stderr text`);
    assert.deepEqual(JSON.parse(result.stdout), {
      script,
      argv: [...prefix, '--target', '.']
    });
  }
});

test('both shipped configs pass all strict contract tools', () => {
  for (const configName of ['harness.config.json', 'harness.config.example.json']) {
    const configPath = path.join(repoRoot, configName);
    for (const tool of configTools) {
      const result = runConfig(tool, configPath);
      assert.equal(result.status, 0, `${configName} must pass ${tool}: ${result.stdout}${result.stderr}`);
      const report = parseJsonOutput(result);
      assert.ok(report.details.activeBoundary, `${tool} must report activeBoundary details`);
    }
  }
});

test('each default CLI entrypoint is declared exactly once in the active boundary', () => {
  const defaultAliases = defaultRoutes.map(([alias]) => alias).sort();
  for (const configName of ['harness.config.json', 'harness.config.example.json']) {
    const data = JSON.parse(fs.readFileSync(path.join(repoRoot, configName), 'utf8'));
    const declarations = Object.entries(data.activeBoundary)
      .flatMap(([category, entries]) => entries.map((entry) => [category, entry]));
    const defaultDeclarations = declarations.filter(([, entry]) => defaultAliases.includes(entry));

    assert.equal(defaultDeclarations.length, defaultAliases.length, `${configName} must declare only the nine default entrypoints once`);
    assert.deepEqual(defaultDeclarations.map(([, entry]) => entry).sort(), defaultAliases);
    for (const [alias, category] of Object.entries(defaultBoundaryCategories)) {
      assert.deepEqual(
        defaultDeclarations.filter(([, entry]) => entry === alias),
        [[category, alias]],
        `${configName} must classify ${alias} as ${category}`
      );
    }
  }
});

test('all strict contract tools reject each activeBoundary structural defect', (t) => {
  const cases = [
    ['missing key', 'ACTIVE_BOUNDARY_MISSING_KEY', (data) => delete data.activeBoundary.legacyIgnored],
    ['invalid type', 'ACTIVE_BOUNDARY_INVALID_TYPE', (data) => { data.activeBoundary.mechanical = ['config-check', 1]; }],
    ['unknown key', 'ACTIVE_BOUNDARY_UNKNOWN_KEY', (data) => { data.activeBoundary.unexpected = []; }],
    ['cross-category duplicate', 'ACTIVE_BOUNDARY_CROSS_CATEGORY_DUPLICATE', (data) => { data.activeBoundary.explicitTool.push('context'); }]
  ];

  for (const [label, code, mutate] of cases) {
    const configPath = writeConfigFixture(t, mutate);
    for (const tool of configTools) {
      assertBoundaryFailure(runConfig(tool, configPath), code, `${tool} must reject ${label}`);
    }
  }
});

test('legacy lifecycle metadata is optional and non-mechanical categories cannot become hard gates', (t) => {
  const optionalConfig = writeConfigFixture(t, (data) => {
    delete data.autoRisk;
    delete data.verificationRunner;
    delete data.taskLifecycle;
  });
  const optionalResult = runConfig('config-schema.js', optionalConfig);
  assert.equal(optionalResult.status, 0, optionalResult.stdout);
  const optionalReport = parseJsonOutput(optionalResult);
  assert.equal(optionalReport.details.topLevel.missing.includes('autoRisk'), false);
  assert.equal(optionalReport.details.topLevel.missing.includes('verificationRunner'), false);
  assert.equal(optionalReport.details.topLevel.missing.includes('taskLifecycle'), false);

  for (const [category, entry] of [
    ['reportingOnly', 'context'],
    ['explicitTool', 'context diagnostic'],
    ['explicitTool', 'doctor']
  ]) {
    const conflictingConfig = writeConfigFixture(t, (data) => {
      data.gates.hard.push(entry);
    });
    for (const tool of ['config-check.js', 'config-policy.js']) {
      assertBoundaryFailure(runConfig(tool, conflictingConfig), 'ACTIVE_BOUNDARY_NON_MECHANICAL_HARD_GATE');
    }
  }
});

test('recommended checks are narrow, existing, and free of retired workflow categories', () => {
  const forbidden = /memory|task(?:-|\s)|evidence|runtime-docs-init|hook-install|ci-init|capability-scan/i;
  for (const configName of ['harness.config.json', 'harness.config.example.json']) {
    const data = JSON.parse(fs.readFileSync(path.join(repoRoot, configName), 'utf8'));
    for (const phase of ['preWork', 'preCompletion']) {
      for (const key of ['recommendedChecks', 'strictPathRecommendedChecks']) {
        for (const command of data[phase][key]) {
          assert.equal(forbidden.test(command), false, `${configName} ${phase}.${key} must not recommend ${command}`);
          const match = command.match(/scripts\/harness\/([^\s]+\.js)/);
          if (match) assert.equal(fs.existsSync(path.join(repoRoot, 'scripts', 'harness', match[1])), true, `${command} must exist`);
        }
      }
    }
  }
});
