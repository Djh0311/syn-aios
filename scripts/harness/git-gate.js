#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const protectedPatterns = [/^\.env(?:\.|$)/, /lock$/i, /package-lock\.json$/, /pnpm-lock\.yaml$/, /yarn\.lock$/];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function git(args, gitArgs) {
  const result = spawnSync('git', gitArgs, {
    cwd: args.target,
    encoding: 'utf8',
    timeout: 10000,
    maxBuffer: 1024 * 1024
  });
  return {
    status: typeof result.status === 'number' ? result.status : 1,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    error: result.error ? result.error.message : null
  };
}

function parseStatus(output) {
  return String(output || '')
    .split(/\r?\n/)
    .filter(Boolean)
    .map((line) => ({
      code: line.slice(0, 2),
      file: line.slice(3)
    }));
}

function isProtected(file) {
  return protectedPatterns.some((pattern) => pattern.test(file));
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      isGitRepo: false,
      branch: null,
      changes: [],
      protectedChanges: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const inside = git(args, ['rev-parse', '--is-inside-work-tree']);
  if (inside.status !== 0 || inside.stdout.trim() !== 'true') {
    add(report, 'warn', `Not confirmed as a git repository: ${(inside.stderr || inside.error || '').trim() || 'unknown'}`);
    return report;
  }

  report.details.isGitRepo = true;
  add(report, 'pass', 'Git repository detected');

  const branch = git(args, ['branch', '--show-current']);
  report.details.branch = branch.stdout.trim() || null;
  if (report.details.branch) add(report, 'pass', `Current branch: ${report.details.branch}`);
  else add(report, 'warn', 'Current branch could not be determined');

  const status = git(args, ['status', '--short']);
  if (status.status !== 0) {
    add(report, args.strict ? 'fail' : 'warn', `git status failed: ${(status.stderr || status.error || '').trim()}`);
    return report;
  }

  report.details.changes = parseStatus(status.stdout);
  report.details.protectedChanges = report.details.changes.filter((item) => isProtected(item.file));
  if (report.details.changes.length === 0) add(report, 'pass', 'Working tree has no reported changes');
  else add(report, 'warn', `Working tree changes reported: ${report.details.changes.length}`);
  if (report.details.protectedChanges.length > 0) {
    add(report, args.strict ? 'fail' : 'warn', `Protected path changes detected: ${report.details.protectedChanges.map((item) => item.file).join(', ')}`);
  }

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
  console.log(`Harness git gate: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log('\nDETAILS');
  console.log(JSON.stringify(report.details, null, 2));
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
