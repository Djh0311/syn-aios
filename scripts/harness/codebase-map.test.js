#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const { execFileSync, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const SCRIPT_PATH = path.join(__dirname, 'codebase-map.js');
const DOMAINS = [
  'conversation-transport',
  'syn-mcp-supervision',
  'workflow-execution-governance',
  'persistence-canonical-state',
  'ui-shared-foundation',
  'development-harness',
];

function writeFile(root, relativePath, content = '') {
  const filePath = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function writeJson(root, relativePath, value) {
  writeFile(root, relativePath, `${JSON.stringify(value, null, 2)}\n`);
}

function readJson(root, relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), 'utf8'));
}

function git(root, args) {
  return execFileSync(
    'git',
    [
      '-c',
      'commit.gpgsign=false',
      '-c',
      'user.email=codebase-map@example.invalid',
      '-c',
      'user.name=Codebase Map Fixture',
      ...args,
    ],
    { cwd: root, encoding: 'utf8' },
  ).trim();
}

function sourceRef(relativePath, symbol) {
  return { path: relativePath, symbol, kind: 'fixture' };
}

function capability(verifiedAtCommit, overrides = {}) {
  return {
    id: 'conversation-transport.agent-manual-relay',
    domain: 'conversation-transport',
    name: 'Agent conversation manual relay',
    status: 'active',
    coverage: 'verified-partial',
    canonical: sourceRef('src/agent.ts', 'AgentSessionCenter'),
    entrypoints: [sourceRef('src/agent.ts', 'AgentSessionCenter')],
    publicSymbols: [
      sourceRef('src/agent.ts', 'AgentSessionCenter'),
      sourceRef('src/agent.ts', 'pollManualRelayAttempt'),
      sourceRef('src/agent.ts', 'stopManualRelayAttempt'),
    ],
    consumers: [sourceRef('src/consumer.ts', 'renderAgentConversation')],
    stateOwners: [sourceRef('src/state.ts', 'manualRelayState')],
    contracts: [sourceRef('src/agent.ts', 'manualRelayContract')],
    tests: [sourceRef('tests/agent.test.ts', 'agentRelayTest')],
    related: [],
    knownDuplicates: [],
    keywords: [
      '交办会话',
      'conversation transport',
      'AgentSessionCenter',
      'existing',
      'new',
      'event mapping',
      'Stop',
      'poll',
      'readback',
      'src/agent.ts',
    ],
    verifiedAtCommit,
    ...overrides,
  };
}

function domainPath(domain) {
  return `docs/code-map/domains/${domain}.json`;
}

function createFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'codebase-map-'));
  writeFile(root, 'README.md', '# fixture\n');
  writeFile(
    root,
    'src/agent.ts',
    [
      'export const AgentSessionCenter = true;',
      'export const pollManualRelayAttempt = true;',
      'export const stopManualRelayAttempt = true;',
      'export const prepareRealExecutionProductCommand = true;',
      'export const legacyProductCommandBoundarySpec = true;',
      'export const FabricatedPublicSymbolSuffix = true;',
      '',
    ].join('\n'),
  );
  writeFile(root, 'src/consumer.ts', 'export const renderAgentConversation = true;\n');
  writeFile(root, 'src/state.ts', 'export const manualRelayState = true;\n');
  writeFile(root, 'tests/agent.test.ts', 'export const agentRelayTest = true;\n');

  git(root, ['init', '-q']);
  git(root, ['add', 'README.md', 'src', 'tests']);
  git(root, ['commit', '-qm', 'fixture sources']);
  const verifiedAtCommit = git(root, ['rev-parse', 'HEAD']);

  const domains = DOMAINS.map((domain) => ({
    id: domain,
    name: domain,
    path: domainPath(domain),
    coverage: 'seed-partial',
  }));
  writeJson(root, 'docs/code-map/index.json', {
    schemaVersion: 1,
    coverage: 'seed-partial',
    domains,
  });

  for (const domain of DOMAINS) {
    writeJson(root, domainPath(domain), {
      schemaVersion: 1,
      domain,
      coverage: 'seed-partial',
      capabilities: domain === 'conversation-transport' ? [capability(verifiedAtCommit)] : [],
    });
  }

  git(root, ['add', 'docs/code-map']);
  git(root, ['commit', '-qm', 'fixture map']);
  return { root, verifiedAtCommit };
}

function runTool(root, args) {
  const result = spawnSync(process.execPath, [SCRIPT_PATH, ...args, '--target', root], {
    encoding: 'utf8',
  });
  assert.equal(result.error, undefined, result.error?.message);
  return result;
}

function conversationMap(root) {
  return domainPath('conversation-transport');
}

function replaceConversationMap(root, mutate) {
  const relativePath = conversationMap(root);
  const domain = readJson(root, relativePath);
  mutate(domain);
  writeJson(root, relativePath, domain);
}

function expectCheckFailure(root, expectedCode) {
  const result = runTool(root, ['check', '--strict', '--json']);
  assert.notEqual(result.status, 0, result.stderr);
  const payload = JSON.parse(result.stdout);
  assert.ok(payload.errors.some((item) => item.code === expectedCode), result.stdout);
}

function addRealExecutionRoutingCapabilities(root, verifiedAtCommit) {
  const relativePath = domainPath('workflow-execution-governance');
  const domain = readJson(root, relativePath);
  domain.capabilities.push(
    capability(verifiedAtCommit, {
      id: 'workflow-execution-governance.guarded-real-execution-product-command',
      domain: 'workflow-execution-governance',
      name: 'Guarded real-execution product command',
      canonical: sourceRef('src/agent.ts', 'prepareRealExecutionProductCommand'),
      entrypoints: [sourceRef('src/agent.ts', 'prepareRealExecutionProductCommand')],
      publicSymbols: [sourceRef('src/agent.ts', 'prepareRealExecutionProductCommand')],
      consumers: [sourceRef('src/consumer.ts', 'renderAgentConversation')],
      stateOwners: [sourceRef('src/state.ts', 'manualRelayState')],
      contracts: [sourceRef('src/agent.ts', 'prepareRealExecutionProductCommand')],
      tests: [],
      related: [
        {
          id: 'workflow-execution-governance.legacy-real-workflow-execution',
          relationship: 'separate guarded product command from legacy boundary',
        },
      ],
      knownDuplicates: [],
      keywords: ['real execution', 'product command', 'prepareRealExecutionProductCommand'],
    }),
    capability(verifiedAtCommit, {
      id: 'workflow-execution-governance.legacy-real-workflow-execution',
      domain: 'workflow-execution-governance',
      name: 'Legacy workflow execution boundary',
      status: 'legacy',
      canonical: sourceRef('src/agent.ts', 'legacyProductCommandBoundarySpec'),
      entrypoints: [],
      publicSymbols: [sourceRef('src/agent.ts', 'legacyProductCommandBoundarySpec')],
      consumers: [],
      stateOwners: [],
      contracts: [sourceRef('src/agent.ts', 'legacyProductCommandBoundarySpec')],
      tests: [],
      related: [
        {
          id: 'workflow-execution-governance.guarded-real-execution-product-command',
          relationship: 'separate legacy boundary from guarded product command',
        },
      ],
      knownDuplicates: [],
      keywords: ['legacy workflow execution', 'run_workflow_machine', 'real execution', 'product command'],
    }),
  );
  writeJson(root, relativePath, domain);
}

test('healthy six-domain partial map validates and conversation queries find the active base', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const check = runTool(root, ['check', '--strict', '--json']);
  assert.equal(check.status, 0, check.stderr);
  assert.equal(JSON.parse(check.stdout).status, 'OK');

  for (const query of [
    '交办会话',
    'conversation transport',
    'Stop',
    'poll',
    'readback',
    'AgentSessionCenter',
    'src/agent.ts',
  ]) {
    const result = runTool(root, ['query', '--query', query, '--json']);
    assert.equal(result.status, 0, result.stderr);
    const payload = JSON.parse(result.stdout);
    assert.equal(payload.status, 'MATCH');
    assert.equal(payload.results[0].id, 'conversation-transport.agent-manual-relay');
  }
});

test('duplicate ids fail map validation', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  replaceConversationMap(root, (domain) => {
    domain.capabilities.push({ ...domain.capabilities[0], name: 'Duplicate relay' });
  });
  expectCheckFailure(root, 'DUPLICATE_CAPABILITY_ID');
});

test('schema errors and untracked canonical paths fail map validation', (t) => {
  const schemaFixture = createFixture();
  t.after(() => fs.rmSync(schemaFixture.root, { recursive: true, force: true }));
  replaceConversationMap(schemaFixture.root, (domain) => {
    delete domain.capabilities[0].name;
  });
  expectCheckFailure(schemaFixture.root, 'SCHEMA_ERROR');

  const pathFixture = createFixture();
  t.after(() => fs.rmSync(pathFixture.root, { recursive: true, force: true }));
  writeFile(pathFixture.root, 'src/untracked.ts', 'export const untracked = true;\n');
  replaceConversationMap(pathFixture.root, (domain) => {
    domain.capabilities[0].canonical = sourceRef('src/untracked.ts', 'untracked');
  });
  expectCheckFailure(pathFixture.root, 'MAP_PATH_NOT_TRACKED');
});

test('public symbols must be exact identifiers at their verified commit', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  fs.appendFileSync(path.join(root, 'src/agent.ts'), 'export const FabricatedPublicSymbol = true;\n');
  git(root, ['add', 'src/agent.ts']);
  git(root, ['commit', '-qm', 'later symbol only']);
  replaceConversationMap(root, (domain) => {
    domain.capabilities[0].publicSymbols[0] = sourceRef('src/agent.ts', 'FabricatedPublicSymbol');
  });

  expectCheckFailure(root, 'MAP_PUBLIC_SYMBOL_NOT_FOUND');
});

test('needs-confirmation may omit a canonical source without making the map dishonest', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const relativePath = domainPath('syn-mcp-supervision');
  const domain = readJson(root, relativePath);
  domain.capabilities.push(
    capability(null, {
      id: 'syn-mcp-supervision.shared-plane',
      domain: 'syn-mcp-supervision',
      name: 'Shared capability plane pending tracked proof',
      status: 'needs-confirmation',
      canonical: null,
      entrypoints: [],
      publicSymbols: [],
      consumers: [],
      stateOwners: [],
      contracts: [],
      tests: [],
      keywords: ['shared capability plane', 'needs confirmation'],
      verifiedAtCommit: null,
    }),
  );
  writeJson(root, relativePath, domain);

  const result = runTool(root, ['check', '--strict', '--json']);
  assert.equal(result.status, 0, result.stderr);
});

test('partial map misses stay explicit and non-blocking', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const result = runTool(root, ['query', '--query', '不存在的能力', '--json']);
  assert.equal(result.status, 0, result.stderr);
  const payload = JSON.parse(result.stdout);
  assert.equal(payload.status, 'NO_MATCH_IN_PARTIAL_MAP');
  assert.match(payload.message, /partial/i);
});

test('real-execution queries prefer the active guarded product command over the legacy boundary', (t) => {
  const { root, verifiedAtCommit } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  addRealExecutionRoutingCapabilities(root, verifiedAtCommit);

  const check = runTool(root, ['check', '--strict', '--json']);
  assert.equal(check.status, 0, check.stderr);

  for (const [query, expectedId] of [
    ['real execution', 'workflow-execution-governance.guarded-real-execution-product-command'],
    ['product command', 'workflow-execution-governance.guarded-real-execution-product-command'],
    ['prepareRealExecutionProductCommand', 'workflow-execution-governance.guarded-real-execution-product-command'],
    ['legacy workflow execution', 'workflow-execution-governance.legacy-real-workflow-execution'],
    ['run_workflow_machine', 'workflow-execution-governance.legacy-real-workflow-execution'],
  ]) {
    const result = runTool(root, ['query', '--query', query, '--json']);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(JSON.parse(result.stdout).results[0].id, expectedId);
  }
});

test('overlay reports only uncommitted paths and never changes the committed map', (t) => {
  const { root } = createFixture();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const mapBefore = fs.readFileSync(path.join(root, conversationMap(root)), 'utf8');

  writeFile(root, 'src/agent.ts', 'export const AgentSessionCenter = false;\n');
  writeFile(root, 'src/shared-untracked.ts', 'export const shared = true;\n');
  const result = runTool(root, ['overlay', '--json']);
  assert.equal(result.status, 0, result.stderr);
  const payload = JSON.parse(result.stdout);
  assert.equal(payload.status, 'OVERLAY');
  assert.ok(payload.entries.some((entry) => entry.path === 'src/agent.ts' && entry.kind === 'unstaged'));
  assert.ok(payload.entries.some((entry) => entry.path === 'src/shared-untracked.ts' && entry.kind === 'untracked'));
  assert.equal(fs.readFileSync(path.join(root, conversationMap(root)), 'utf8'), mapBefore);

  const check = runTool(root, ['check', '--strict', '--json']);
  assert.equal(check.status, 0, check.stderr);
});

test('staged rename and delete in fixture repositories identify affected capabilities', (t) => {
  const renamed = createFixture();
  t.after(() => fs.rmSync(renamed.root, { recursive: true, force: true }));
  git(renamed.root, ['mv', 'src/agent.ts', 'src/renamed-agent.ts']);
  const renameCheck = runTool(renamed.root, ['check', '--staged', '--strict', '--json']);
  assert.notEqual(renameCheck.status, 0, renameCheck.stderr);
  const renamePayload = JSON.parse(renameCheck.stdout);
  assert.ok(
    renamePayload.stagedImpacts.some(
      (impact) =>
        impact.kind === 'rename'
        && impact.capabilityId === 'conversation-transport.agent-manual-relay'
        && impact.from === 'src/agent.ts'
        && impact.to === 'src/renamed-agent.ts',
    ),
  );

  const deleted = createFixture();
  t.after(() => fs.rmSync(deleted.root, { recursive: true, force: true }));
  git(deleted.root, ['rm', '-q', 'src/state.ts']);
  const deleteCheck = runTool(deleted.root, ['check', '--staged', '--strict', '--json']);
  assert.notEqual(deleteCheck.status, 0, deleteCheck.stderr);
  const deletePayload = JSON.parse(deleteCheck.stdout);
  assert.ok(
    deletePayload.stagedImpacts.some(
      (impact) =>
        impact.kind === 'delete'
        && impact.capabilityId === 'conversation-transport.agent-manual-relay'
        && impact.path === 'src/state.ts',
    ),
  );
});
