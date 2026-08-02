#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');

const {
  expectedManagedBlock,
  inspectGitHooks,
  inspectManagedInstallationEntry,
} = require('./active-path-audit');

function runGit(target, arguments_) {
  const result = spawnSync('git', ['-C', target, ...arguments_], {
    encoding: 'utf8',
  });
  assert.equal(result.status, 0, result.stderr);
}

function makeHookFixture(t, hooksPath) {
  const target = fs.mkdtempSync(path.join(os.tmpdir(), 'harness-active-path-'));
  t.after(() => fs.rmSync(target, { recursive: true, force: true }));
  runGit(target, ['init', '--quiet']);

  const template = [
    '#!/bin/sh',
    '# HARNESS V2 MANAGED HOOK: pre-commit',
    'git diff --cached --check',
    'node scripts/harness-v2/git-gate.js --target . --strict',
    'node scripts/harness-v2/codebase-map.js check --target . --staged --shadow',
    '',
  ].join('\n');
  const templatePath = path.join(target, 'templates', 'hooks', 'pre-commit');
  fs.mkdirSync(path.dirname(templatePath), { recursive: true });
  fs.writeFileSync(templatePath, template);

  runGit(target, ['config', 'core.hooksPath', hooksPath]);
  return { target, template };
}

function installFixtureHook(directory, template) {
  fs.mkdirSync(directory, { recursive: true });
  const hookPath = path.join(directory, 'pre-commit');
  fs.writeFileSync(
    hookPath,
    `#!/bin/sh\n${expectedManagedBlock(template)}\n`,
    { mode: 0o755 },
  );
}

test('mutable managed files may change content but retain mode enforcement', () => {
  assert.equal(typeof inspectManagedInstallationEntry, 'function');
  const entry = {
    ownership: 'created',
    mutable: true,
    installedSha256: 'not-the-current-digest',
    installedMode: 0o644,
  };
  const content = Buffer.from('project-owned current state\n');

  const contentErrors = [];
  inspectManagedInstallationEntry(
    'docs/harness/CURRENT.md',
    entry,
    { content, mode: 0o644 },
    contentErrors,
  );
  assert.deepEqual(contentErrors, []);

  const modeErrors = [];
  inspectManagedInstallationEntry(
    'docs/harness/CURRENT.md',
    entry,
    { content, mode: 0o755 },
    modeErrors,
  );
  assert.deepEqual(modeErrors, [
    'docs/harness/CURRENT.md drifted from its managed installation mode',
  ]);
});

test('immutable managed files still enforce installed content digest', () => {
  const errors = [];
  inspectManagedInstallationEntry(
    'scripts/harness-v2/git-gate.js',
    {
      ownership: 'created',
      mutable: false,
      installedSha256: 'not-the-current-digest',
      installedMode: 0o755,
    },
    { content: Buffer.from('changed\n'), mode: 0o755 },
    errors,
  );
  assert.deepEqual(errors, [
    'scripts/harness-v2/git-gate.js drifted from its managed installation',
  ]);
});

test('repository-local core.hooksPath is an authoritative managed carrier', (t) => {
  const { target, template } = makeHookFixture(t, '.githooks');
  installFixtureHook(path.join(target, '.githooks'), template);
  const errors = [];
  const warnings = [];

  inspectGitHooks(
    target,
    new Set(['staged-git-safety', 'code-map']),
    true,
    errors,
    warnings,
  );

  assert.deepEqual(errors, []);
  assert.deepEqual(warnings, []);
});

test('core.hooksPath outside the repository remains rejected', (t) => {
  const outside = fs.mkdtempSync(path.join(os.tmpdir(), 'harness-hooks-outside-'));
  t.after(() => fs.rmSync(outside, { recursive: true, force: true }));
  const { target, template } = makeHookFixture(t, outside);
  installFixtureHook(outside, template);
  const errors = [];
  const warnings = [];

  inspectGitHooks(
    target,
    new Set(['staged-git-safety', 'code-map']),
    true,
    errors,
    warnings,
  );

  assert.deepEqual(errors, [
    'custom core.hooksPath must resolve inside the repository worktree',
  ]);
  assert.deepEqual(warnings, []);
});

test('repository-local symlink hook carrier remains rejected', (t) => {
  const { target, template } = makeHookFixture(t, '.hook-link');
  const realHooks = path.join(target, '.real-hooks');
  installFixtureHook(realHooks, template);
  fs.symlinkSync('.real-hooks', path.join(target, '.hook-link'));
  const errors = [];
  const warnings = [];

  inspectGitHooks(
    target,
    new Set(['staged-git-safety', 'code-map']),
    true,
    errors,
    warnings,
  );

  assert.deepEqual(errors, [
    'configured Git hooks carrier must be a regular non-symlink directory',
  ]);
  assert.deepEqual(warnings, []);
});
