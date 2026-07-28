#!/usr/bin/env node
'use strict';

// v0.4 task-package compatibility reader.
//
// v0.5 owns all task and plan lifecycle writes in scripts/harness-v2/task.js
// and scripts/harness-v2/plan.js.  This old entrypoint deliberately retains
// only safe inspection of existing v3 packages.  Keeping a hidden write path
// here would reintroduce the old global active-task gate and the behaviour
// that retired a parent plan while opening a leaf.

const fs = require('node:fs');
const path = require('node:path');

const {
  AUTHORITY_PATH,
  CURRENT_PATH,
  packageRelativePath,
  parseAuthorityDocument,
  parseCurrentDocument,
  parseTaskPackage,
  sha256,
  validateId,
} = require('./lib/task-package-model');
const { safeOutputRepoPath, sanitizeOutputText } = require('./lib/output-safety');
const { loadCoreContext } = require('./lib/context-contract');

const COMMAND = 'task-package';
const READ_OPERATIONS = new Set(['validate', 'status', 'show']);
const RETIRED_LIFECYCLE_OPERATIONS = new Set([
  'create',
  'activate',
  'accept',
  'complete',
]);
const FIELD_FLAGS = new Map([
  ['--id', 'id'],
  ['--title', 'title'],
  ['--goal', 'goal'],
  ['--mode', 'mode'],
  ['--owner', 'owner'],
  ['--acceptance-owner', 'acceptanceOwner'],
  ['--accepted-by', 'acceptedBy'],
  ['--next-action', 'nextAction'],
  ['--outcome', 'outcome'],
  ['--git-disposition', 'gitDisposition'],
  ['--git-commits', 'gitCommits'],
  ['--git-reason', 'gitReason'],
  ['--next-goal', 'nextGoal'],
  ['--next-owner', 'nextOwner'],
  ['--expected-sha256', 'expectedSha256'],
  ['--from-plan', 'fromPlan'],
]);

class TaskPackageError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'TaskPackageError';
    this.code = code;
  }
}

function fail(code, message) {
  throw new TaskPackageError(code, message);
}

function usage() {
  return [
    'Usage:',
    '  node scripts/harness-v2/task-package.js validate --target <project> --id <ID> [--json]',
    '  node scripts/harness-v2/task-package.js status --target <project> [--id <ID>] [--json]',
    '  node scripts/harness-v2/task-package.js show --target <project> --id <ID> [--full] [--json]',
    '',
    'The legacy create, activate, accept, and complete commands are read-only compatibility errors.',
    'Use scripts/harness-v2/task.js for task lifecycle and scripts/harness-v2/plan.js for plan lifecycle.',
  ].join('\n');
}

function takeValue(argv, index, flag) {
  if (index + 1 >= argv.length || argv[index + 1].startsWith('--')) {
    fail('ARGUMENT_VALUE_REQUIRED', `${flag} requires a value`);
  }
  return argv[index + 1];
}

function parseArgs(argv) {
  const options = {
    operation: null,
    target: process.cwd(),
    targetProvided: false,
    write: false,
    json: false,
    full: false,
    help: false,
    values: {},
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (!options.operation && !argument.startsWith('-')) {
      options.operation = argument;
    } else if (argument === '--target') {
      options.target = takeValue(argv, index, argument);
      options.targetProvided = true;
      index += 1;
    } else if (FIELD_FLAGS.has(argument)) {
      const field = FIELD_FLAGS.get(argument);
      if (Object.prototype.hasOwnProperty.call(options.values, field)) {
        fail('ARGUMENT_DUPLICATE', `${argument} may be provided only once`);
      }
      options.values[field] = takeValue(argv, index, argument);
      index += 1;
    } else if (argument === '--write') {
      if (options.write) fail('ARGUMENT_DUPLICATE', '--write may be provided only once');
      options.write = true;
    } else if (argument === '--json') {
      if (options.json) fail('ARGUMENT_DUPLICATE', '--json may be provided only once');
      options.json = true;
    } else if (argument === '--full') {
      if (options.full) fail('ARGUMENT_DUPLICATE', '--full may be provided only once');
      options.full = true;
    } else if (argument === '--help' || argument === '-h') {
      options.help = true;
    } else {
      fail('ARGUMENT_UNSUPPORTED', 'unsupported argument');
    }
  }
  if (options.help) return options;
  if (!options.operation || (!READ_OPERATIONS.has(options.operation)
    && !RETIRED_LIFECYCLE_OPERATIONS.has(options.operation))) {
    fail('OPERATION_REQUIRED', usage());
  }
  if (options.full && options.operation !== 'show') {
    fail('FULL_NOT_ALLOWED', '--full is accepted only by show');
  }
  if (options.full && options.json) {
    fail('FULL_JSON_NOT_ALLOWED', '--full cannot be combined with --json');
  }
  if (READ_OPERATIONS.has(options.operation) && options.write) {
    fail('WRITE_NOT_ALLOWED', `${options.operation} is read-only and does not accept --write`);
  }
  if (['validate', 'show'].includes(options.operation) && !String(options.values.id || '').trim()) {
    fail('ARGUMENT_REQUIRED', 'missing required field: id');
  }
  if (options.values.id) {
    try {
      options.values.id = validateId(options.values.id);
    } catch (error) {
      fail('TASK_ID_INVALID', error.message);
    }
  }
  if (options.values.fromPlan) {
    try {
      options.values.fromPlan = validateId(options.values.fromPlan);
    } catch (error) {
      fail('PLAN_ID_INVALID', error.message);
    }
  }
  options.target = path.resolve(options.target);
  return options;
}

function assertRoot(target) {
  let stat;
  try {
    stat = fs.lstatSync(target);
  } catch (error) {
    fail('TARGET_UNAVAILABLE', `target is unavailable: ${error.code || 'unknown error'}`);
  }
  if (stat.isSymbolicLink() || !stat.isDirectory()) {
    fail('TARGET_UNSAFE', 'target must be a real directory, not a symlink');
  }
  try {
    return fs.realpathSync(target);
  } catch {
    fail('TARGET_UNAVAILABLE', 'target identity could not be resolved');
  }
}

function absolutePath(target, relativePath) {
  const normalized = path.posix.normalize(relativePath);
  if (
    !relativePath
    || normalized !== relativePath
    || relativePath.startsWith('/')
    || relativePath.split('/').includes('..')
  ) {
    fail('PATH_UNSAFE', 'internal path is unsafe');
  }
  const absolute = path.resolve(target, ...relativePath.split('/'));
  if (absolute !== target && !absolute.startsWith(`${target}${path.sep}`)) {
    fail('PATH_OUTSIDE_TARGET', 'internal path escapes the target');
  }
  return absolute;
}

function inspectAncestors(target, relativePath, allowMissing) {
  const segments = relativePath.split('/');
  let current = target;
  for (let index = 0; index < segments.length; index += 1) {
    current = path.join(current, segments[index]);
    let stat;
    try {
      stat = fs.lstatSync(current);
    } catch (error) {
      if (allowMissing && error.code === 'ENOENT') return;
      if (error.code === 'ENOENT') fail('FILE_MISSING', `${relativePath} does not exist`);
      fail('FILE_UNAVAILABLE', `${relativePath} is unavailable`);
    }
    if (stat.isSymbolicLink()) fail('SYMLINK_REJECTED', `${relativePath} crosses a symlink`);
    if (index < segments.length - 1 && !stat.isDirectory()) {
      fail('PATH_NOT_DIRECTORY', `${relativePath} has a non-directory ancestor`);
    }
  }
}

function readSnapshot(target, relativePath, options = {}) {
  inspectAncestors(target, relativePath, true);
  const absolute = absolutePath(target, relativePath);
  let stat;
  try {
    stat = fs.lstatSync(absolute);
  } catch (error) {
    if (error.code === 'ENOENT' && options.optional) {
      return { relativePath, exists: false, buffer: null, mode: null };
    }
    if (error.code === 'ENOENT') fail('FILE_MISSING', `${relativePath} does not exist`);
    fail('FILE_UNAVAILABLE', `${relativePath} is unavailable`);
  }
  if (stat.isSymbolicLink()) fail('SYMLINK_REJECTED', `${relativePath} is a symlink`);
  if (!stat.isFile()) fail('FILE_TYPE_INVALID', `${relativePath} must be a regular file`);
  let descriptor;
  try {
    const noFollow = fs.constants.O_NOFOLLOW || 0;
    descriptor = fs.openSync(absolute, fs.constants.O_RDONLY | noFollow);
    const opened = fs.fstatSync(descriptor);
    if (!opened.isFile()) fail('FILE_TYPE_INVALID', `${relativePath} must be a regular file`);
    return {
      relativePath,
      exists: true,
      buffer: fs.readFileSync(descriptor),
      mode: opened.mode & 0o777,
    };
  } catch (error) {
    if (error instanceof TaskPackageError) throw error;
    fail('FILE_READ_FAILED', `${relativePath} could not be read safely`);
  } finally {
    if (descriptor !== undefined) fs.closeSync(descriptor);
  }
}

function packageLocations(id) {
  return [
    packageRelativePath(id, 'DRAFT'),
    packageRelativePath(id, 'ACTIVE'),
    packageRelativePath(id, 'COMPLETE'),
  ];
}

function locatePackage(target, id, options = {}) {
  const found = packageLocations(id)
    .map((relativePath) => readSnapshot(target, relativePath, { optional: true }))
    .filter((snapshot) => snapshot.exists);
  if (found.length > 1) {
    fail('PACKAGE_DUPLICATE', 'the same task id exists in more than one lifecycle location');
  }
  if (found.length === 0) {
    if (options.optional) return null;
    fail('PACKAGE_NOT_FOUND', 'task package was not found');
  }
  return found[0];
}

function loadBinding(target) {
  const core = loadCoreContext(target);
  if (core.coreStatus !== 'OK') {
    const codes = core.issues.slice(0, 5).map((entry) => entry.code).join(', ');
    fail('CORE_CONTEXT_INVALID', `core Harness context is invalid: ${codes}`);
  }
  const authoritySnapshot = readSnapshot(target, AUTHORITY_PATH);
  const currentSnapshot = readSnapshot(target, CURRENT_PATH);
  const authority = parseAuthorityDocument(authoritySnapshot.buffer.toString('utf8'));
  const current = parseCurrentDocument(currentSnapshot.buffer.toString('utf8'));
  if (!authority.ok) fail('AUTHORITY_INVALID', `AUTHORITY is invalid: ${authority.issues.join(', ')}`);
  if (!current.ok) fail('CURRENT_INVALID', `CURRENT is invalid: ${current.issues.join(', ')}`);
  return { core, authoritySnapshot, currentSnapshot, authority, current };
}

function stateFingerprint(target, operation, id, values, snapshots) {
  const rootStat = fs.statSync(target);
  const rootIdentity = {
    realpath: fs.realpathSync(target),
    dev: String(rootStat.dev),
    ino: String(rootStat.ino),
  };
  const state = snapshots
    .map((snapshot) => ({
      path: snapshot.relativePath,
      exists: snapshot.exists,
      sha256: snapshot.exists ? sha256(snapshot.buffer) : null,
      mode: snapshot.mode,
    }))
    .sort((left, right) => left.path.localeCompare(right.path));
  return sha256(Buffer.from(JSON.stringify({
    operation,
    id: id || null,
    rootIdentity,
    values: Object.fromEntries(Object.entries(values).sort(([left], [right]) => left.localeCompare(right))),
    state,
  }), 'utf8'));
}

function summarizePackage(parsed) {
  if (!parsed) return null;
  return {
    id: parsed.id,
    title: sanitizeOutputText(parsed.title || '', 160),
    status: parsed.status,
    outcome: parsed.outcome,
    mode: parsed.mode,
    owner: sanitizeOutputText(parsed.owner || '', 120),
    acceptanceOwner: sanitizeOutputText(parsed.acceptanceOwner || '', 120),
    acceptedBy: sanitizeOutputText(parsed.acceptedBy || '', 120),
    updatedAt: parsed.updatedAt,
    goal: sanitizeOutputText(parsed.goal || '', 320),
    nextAction: sanitizeOutputText(parsed.nextAction || '', 320),
    gitDisposition: parsed.gitDisposition,
    path: safeOutputRepoPath(parsed.relativePath),
    sha256: parsed.sha256,
    bytes: parsed.bytes,
    lines: parsed.lines,
    bodyBytes: parsed.bodyBytes,
    bodyLines: parsed.bodyLines,
    legacy: parsed.legacy,
  };
}

function legacyRedirects(options) {
  const target = '--target .';
  const id = options.values.id || '<TASK-ID>';
  const redirects = [];
  if (options.values.fromPlan) {
    redirects.push({
      owner: 'plan',
      command: `node scripts/harness-v2/plan.js add-child --id ${options.values.fromPlan} --node <TASK_NODE.md> --inspect ${target}`,
      reason: 'A parent plan remains current while its child is opened.',
    });
  }
  if (options.operation === 'create') {
    redirects.push({
      owner: 'task',
      command: `node scripts/harness-v2/task.js propose --request-stdin --profile ORDINARY_LOCAL ${target}`,
      reason: 'Generate a v0.5 proposal from the request and current facts before any write.',
    });
  } else if (options.operation === 'activate') {
    redirects.push({
      owner: 'task',
      command: `node scripts/harness-v2/task.js start --proposal <PROPOSAL.json> --proposal-digest <SHA256> ${target} --write`,
      reason: 'Use v0.5 start, which checks the declared worktree, write scope, and exclusive resources.',
    });
  } else {
    redirects.push({
      owner: 'task',
      command: `node scripts/harness-v2/task.js finish --id ${id} --inspect ${target}`,
      reason: 'Use v0.5 closeout inspection and its separately confirmed write path.',
    });
  }
  redirects.push({
    owner: 'plan',
    command: `node scripts/harness-v2/plan.js <command> ${target}`,
    reason: 'Use v0.5 plan lifecycle for parent/child state; legacy activation never retires a parent.',
  });
  return redirects;
}

function retiredLifecycleOperation(options) {
  return {
    schemaVersion: 1,
    command: COMMAND,
    operation: options.operation,
    ok: false,
    code: 'LEGACY_TASK_PACKAGE_WRITE_RETIRED',
    error: 'legacy task-package lifecycle writes are retired; this compatibility entrypoint is read-only',
    mode: 'read-only-compatibility',
    readOnly: true,
    wrote: false,
    changed: false,
    transitionAllowed: false,
    changedPaths: [],
    redirects: legacyRedirects(options),
    stateSha256: stateFingerprint(options.target, options.operation, options.values.id, options.values, []),
    warnings: [],
    issues: ['USE_V2_TASK_OR_PLAN_ENTRYPOINT'],
  };
}

function validateOperation(options) {
  const snapshot = locatePackage(options.target, options.values.id);
  const parsed = parseTaskPackage(snapshot.buffer.toString('utf8'), {
    relativePath: snapshot.relativePath,
  });
  return {
    schemaVersion: 1,
    command: COMMAND,
    operation: 'validate',
    ok: parsed.ok,
    mode: 'read-only',
    readOnly: true,
    wrote: false,
    changed: false,
    package: summarizePackage(parsed),
    stateSha256: stateFingerprint(options.target, 'validate', options.values.id, options.values, [snapshot]),
    warnings: parsed.warnings,
    issues: parsed.issues,
  };
}

function statusOperation(options) {
  const binding = loadBinding(options.target);
  const taskAuthoritySelected = binding.authority.activeAuthority.startsWith('docs/task-packages/active/');
  const id = options.values.id || (
    taskAuthoritySelected && binding.current.activeId && binding.current.activeId !== 'NONE'
      ? binding.current.activeId
      : null
  );
  const snapshot = id ? locatePackage(options.target, id, { optional: !options.values.id }) : null;
  const parsed = snapshot
    ? parseTaskPackage(snapshot.buffer.toString('utf8'), { relativePath: snapshot.relativePath })
    : null;
  let bindingConsistent = true;
  if (parsed && (parsed.status === 'ACTIVE' || parsed.legacyReviewProjection)) {
    bindingConsistent = binding.authority.activeAuthority === parsed.relativePath
      && binding.current.activeId === parsed.id;
  } else if (parsed) {
    bindingConsistent = binding.authority.activeAuthority !== parsed.relativePath
      && binding.current.activeId !== parsed.id;
  } else if (taskAuthoritySelected) {
    bindingConsistent = false;
  } else if (binding.authority.activeAuthority === CURRENT_PATH) {
    bindingConsistent = binding.current.activeId === 'NONE';
  }
  const snapshots = [
    binding.authoritySnapshot,
    binding.currentSnapshot,
    ...(snapshot ? [snapshot] : []),
  ];
  return {
    schemaVersion: 1,
    command: COMMAND,
    operation: 'status',
    ok: (!parsed || parsed.ok) && bindingConsistent,
    mode: 'read-only',
    readOnly: true,
    wrote: false,
    changed: false,
    package: summarizePackage(parsed),
    binding: {
      activeAuthority: safeOutputRepoPath(binding.authority.activeAuthority),
      activeId: binding.current.activeId,
      managedByTaskPackages: taskAuthoritySelected,
      consistent: bindingConsistent,
    },
    stateSha256: stateFingerprint(options.target, 'status', id, options.values, snapshots),
    warnings: parsed ? parsed.warnings : [],
    issues: [
      ...(parsed && !parsed.ok ? parsed.issues : []),
      ...(!bindingConsistent ? ['ACTIVE_BINDING_MISMATCH'] : []),
    ],
  };
}

function showOperation(options) {
  const snapshot = locatePackage(options.target, options.values.id);
  const parsed = parseTaskPackage(snapshot.buffer.toString('utf8'), {
    relativePath: snapshot.relativePath,
  });
  return {
    report: {
      schemaVersion: 1,
      command: COMMAND,
      operation: 'show',
      ok: parsed.ok,
      mode: 'read-only',
      readOnly: true,
      wrote: false,
      changed: false,
      package: summarizePackage(parsed),
      stateSha256: stateFingerprint(options.target, 'show', options.values.id, options.values, [snapshot]),
      warnings: parsed.warnings,
      issues: parsed.issues,
    },
    fullText: options.full ? snapshot.buffer.toString('utf8') : null,
  };
}

function runOperation(options) {
  options.target = assertRoot(options.target);
  if (RETIRED_LIFECYCLE_OPERATIONS.has(options.operation)) {
    return { report: retiredLifecycleOperation(options), fullText: null };
  }
  if (options.operation === 'validate') return { report: validateOperation(options), fullText: null };
  if (options.operation === 'status') return { report: statusOperation(options), fullText: null };
  if (options.operation === 'show') return showOperation(options);
  fail('OPERATION_UNSUPPORTED', 'unsupported operation');
}

function sanitizedFullText(text) {
  return String(text || '')
    .replace(/\r\n/g, '\n')
    .split('\n')
    .map((line) => sanitizeOutputText(line, 1000))
    .join('\n');
}

function humanOutput(report) {
  const lines = [
    `task-package ${report.operation}: ${report.ok ? 'OK' : 'INVALID'}`,
    `mode=${report.mode} wrote=${report.wrote ? 'yes' : 'no'} changed=${report.changed ? 'yes' : 'no'}`,
  ];
  if (report.package) {
    lines.push(
      `package=${report.package.id} status=${report.package.status} outcome=${report.package.outcome}`,
      `path=${report.package.path} lines=${report.package.lines} body-lines=${report.package.bodyLines}`,
    );
  }
  for (const redirect of report.redirects || []) lines.push(`redirect=${redirect.command}`);
  for (const issue of report.issues || []) lines.push(`issue=${issue}`);
  return `${lines.join('\n')}\n`;
}

function errorReport(operation, error) {
  return {
    schemaVersion: 1,
    command: COMMAND,
    operation: operation || null,
    ok: false,
    code: error instanceof TaskPackageError ? error.code : 'UNEXPECTED_ERROR',
    error: sanitizeOutputText(error.message || 'unexpected error', 320),
  };
}

function main(argv = process.argv.slice(2)) {
  let options = { operation: null, json: argv.includes('--json') };
  try {
    options = parseArgs(argv);
    if (options.help) {
      process.stdout.write(`${usage()}\n`);
      return 0;
    }
    const result = runOperation(options);
    if (options.json) {
      process.stdout.write(`${JSON.stringify(result.report, null, 2)}\n`);
    } else {
      process.stdout.write(humanOutput(result.report));
      if (result.fullText !== null) process.stdout.write(`\n${sanitizedFullText(result.fullText)}\n`);
    }
    return result.report.ok ? 0 : 1;
  } catch (error) {
    const report = errorReport(options.operation, error);
    if (options.json) process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    else process.stderr.write(`task-package error [${report.code}]: ${report.error}\n`);
    return 1;
  }
}

if (require.main === module) process.exitCode = main();

module.exports = {
  TaskPackageError,
  errorReport,
  legacyRedirects,
  main,
  parseArgs,
  runOperation,
  stateFingerprint,
};
