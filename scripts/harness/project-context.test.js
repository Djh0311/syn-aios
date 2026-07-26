#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const SCRIPT_PATH = path.join(__dirname, 'project-context.js');
const REPO_ROOT = path.resolve(__dirname, '..', '..');

function baseRoute() {
  return {
    schemaVersion: 1,
    ruleEntry: 'AGENTS.md',
    authorityIndex: 'AUTHORITY.md',
    decision: 'decisions/current-decision.md',
    taskPackage: 'tasks/current-task.md',
    plan: 'docs/plans/current-plan.md',
    currentWork: '当前业务主线。resident/private-home 只作历史保留。',
    nextAction: '先核对当前任务包、授权与验证证据。',
    blocker: '没有新的业务实施授权时，不进入业务代码。',
    safetyReminder: '本路由只读；不运行 Git、Hook、Code Map、源码扫描或历史全文。',
  };
}

function writeFile(target, relativePath, content = '# fixture\n') {
  const absolutePath = path.join(target, relativePath);
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true });
  fs.writeFileSync(absolutePath, content);
}

function createFixture(route = baseRoute(), { writeContext = true } = {}) {
  const target = fs.mkdtempSync(path.join(os.tmpdir(), 'project-context-'));
  writeFile(target, 'AGENTS.md');
  writeFile(target, 'AUTHORITY.md');
  writeFile(target, 'CURRENT.md');
  writeFile(target, 'decisions/current-decision.md');
  writeFile(target, 'tasks/current-task.md');
  writeFile(target, 'docs/plans/current-plan.md');

  if (writeContext) {
    writeFile(target, 'docs/project-context.json', `${JSON.stringify(route, null, 2)}\n`);
  }

  return target;
}

function runRoute(target, args = [], options = {}) {
  const result = spawnSync(
    process.execPath,
    [SCRIPT_PATH, '--target', target, ...args],
    {
      encoding: 'utf8',
      env: { ...process.env, ...options.env },
    },
  );

  assert.equal(result.error, undefined, result.error?.message);
  return result;
}

function assertDefaultBudget(output) {
  assert.ok(output.trimEnd().split(/\r?\n/).length <= 25, 'default route exceeds 25 lines');
  assert.ok(Buffer.byteLength(output, 'utf8') <= 4096, 'default route exceeds 4 KB');
}

function assertDegradedFallback(result) {
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /^ROUTE: DEGRADED/m);
  assert.match(result.stdout, /AGENTS\.md \/ AUTHORITY\.md \/ CURRENT\.md/);
  assertDefaultBudget(result.stdout);
}

test('valid route stays compact in text and default JSON modes', (t) => {
  const target = createFixture();
  t.after(() => fs.rmSync(target, { recursive: true, force: true }));

  const textResult = runRoute(target);
  assert.equal(textResult.status, 0, textResult.stderr);
  assert.match(textResult.stdout, /^ROUTE: READY/m);
  assert.match(textResult.stdout, /当前决策: decisions\/current-decision\.md/);
  assertDefaultBudget(textResult.stdout);

  const jsonResult = runRoute(target, ['--json']);
  assert.equal(jsonResult.status, 0, jsonResult.stderr);
  assertDefaultBudget(jsonResult.stdout);
  const payload = JSON.parse(jsonResult.stdout);
  assert.equal(payload.status, 'READY');
  assert.equal(payload.route.decision, 'decisions/current-decision.md');
  assert.equal(payload.route.taskPackage, 'tasks/current-task.md');
  assert.equal(payload.route.plan, 'docs/plans/current-plan.md');
  assert.equal(Object.hasOwn(payload, 'diagnostic'), false);
});

test('missing, malformed, dangling, multiline, and oversized route sources degrade without blocking', (t) => {
  const missingTarget = createFixture(baseRoute(), { writeContext: false });
  t.after(() => fs.rmSync(missingTarget, { recursive: true, force: true }));
  assertDegradedFallback(runRoute(missingTarget));

  const malformedTarget = createFixture();
  t.after(() => fs.rmSync(malformedTarget, { recursive: true, force: true }));
  writeFile(malformedTarget, 'docs/project-context.json', '{bad json\n');
  assertDegradedFallback(runRoute(malformedTarget));

  const danglingRoute = baseRoute();
  danglingRoute.taskPackage = 'tasks/missing-task.md';
  const danglingTarget = createFixture(danglingRoute);
  t.after(() => fs.rmSync(danglingTarget, { recursive: true, force: true }));
  assertDegradedFallback(runRoute(danglingTarget));

  const multilineRoute = baseRoute();
  multilineRoute.currentWork = '第一行\n第二行';
  const multilineTarget = createFixture(multilineRoute);
  t.after(() => fs.rmSync(multilineTarget, { recursive: true, force: true }));
  assertDegradedFallback(runRoute(multilineTarget));

  const multilinePointerRoute = baseRoute();
  multilinePointerRoute.decision = 'decisions/current\n-decision.md';
  const multilinePointerTarget = createFixture(multilinePointerRoute);
  t.after(() => fs.rmSync(multilinePointerTarget, { recursive: true, force: true }));
  const multilinePointerPayload = JSON.parse(runRoute(multilinePointerTarget, ['--json']).stdout);
  assert.equal(multilinePointerPayload.status, 'DEGRADED');
  assert.deepEqual(multilinePointerPayload.reasons, ['INVALID_POINTER:decision']);

  const oversizedRoute = baseRoute();
  oversizedRoute.currentWork = 'x'.repeat(5000);
  const oversizedTarget = createFixture(oversizedRoute);
  t.after(() => fs.rmSync(oversizedTarget, { recursive: true, force: true }));
  assertDegradedFallback(runRoute(oversizedTarget));
});

test('diagnostics are opt-in and do not alter the default JSON contract', (t) => {
  const target = createFixture();
  t.after(() => fs.rmSync(target, { recursive: true, force: true }));

  const defaultPayload = JSON.parse(runRoute(target, ['--json']).stdout);
  assert.equal(Object.hasOwn(defaultPayload, 'diagnostic'), false);

  const diagnosticPayload = JSON.parse(runRoute(target, ['--json', '--diagnostic']).stdout);
  assert.equal(diagnosticPayload.status, 'READY');
  assert.equal(typeof diagnosticPayload.diagnostic, 'object');
  assert.equal(diagnosticPayload.diagnostic.target, target);
  assert.equal(typeof diagnosticPayload.diagnostic.pointerChecks, 'object');
});

test('default path does not invoke Git, Hook, Code Map, source scan, or child processes', (t) => {
  const target = createFixture();
  const probeDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'project-context-probe-'));
  const probePath = path.join(probeDirectory, 'probe.js');
  const logPath = path.join(probeDirectory, 'calls.log');
  t.after(() => {
    fs.rmSync(target, { recursive: true, force: true });
    fs.rmSync(probeDirectory, { recursive: true, force: true });
  });

  fs.writeFileSync(
    probePath,
    [
      "const fs = require('node:fs');",
      "const childProcess = require('node:child_process');",
      "const append = fs.appendFileSync.bind(fs);",
      "const logPath = process.env.PROJECT_CONTEXT_PROBE_LOG;",
      "function log(kind, value) { append(logPath, `${kind}\\t${String(value)}\\n`); }",
      "for (const name of ['readFileSync', 'existsSync', 'statSync', 'readdirSync']) {",
      '  const original = fs[name];',
      '  fs[name] = function patched(target, ...args) {',
      '    log(`fs.${name}`, target);',
      '    return original.call(this, target, ...args);',
      '  };',
      '}',
      "for (const name of ['exec', 'execFile', 'execFileSync', 'execSync', 'fork', 'spawn', 'spawnSync']) {",
      '  childProcess[name] = function blocked(...args) {',
      '    log(`child.${name}`, args[0]);',
      "    throw new Error('child process is forbidden in default route');",
      '  };',
      '}',
      '',
    ].join('\n'),
  );

  const result = runRoute(target, [], {
    env: {
      NODE_OPTIONS: `--require=${probePath}`,
      PROJECT_CONTEXT_PROBE_LOG: logPath,
    },
  });
  assert.equal(result.status, 0, result.stderr);

  const calls = fs.readFileSync(logPath, 'utf8');
  assert.doesNotMatch(calls, /^child\./m);
  assert.doesNotMatch(calls, /^fs\.readdirSync/m);
  assert.doesNotMatch(calls, /(?:\.git(?:\/|$)|\.githooks(?:\/|$)|docs\/code-map|2026-07-09-codebase-capability-map-v2)/);
  assert.doesNotMatch(calls, /^fs\.readFileSync.*(?:AGENTS\.md|AUTHORITY\.md|CURRENT\.md|decisions\/|tasks\/|evidence\/|handoffs\/)/m);
});

test('repository route names the current shared transport line and historical resident route', () => {
  const result = runRoute(REPO_ROOT, ['--json']);
  assert.equal(result.status, 0, result.stderr);
  const payload = JSON.parse(result.stdout);
  assert.equal(payload.status, 'READY');
  assert.equal(
    payload.route.decision,
    'decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md',
  );
  assert.equal(
    payload.route.taskPackage,
    'tasks/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-implementation-package-v1.md',
  );
  assert.equal(payload.route.plan, 'docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md');
  assert.match(payload.route.currentWork, /resident\/private-home.*历史/);
  assert.match(payload.route.nextAction, /真实 App 替代性验收/);
  assert.match(payload.route.nextAction, /不得由已收口离线包自动续跑/);
});
