#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const defaultKeys = ['lint', 'typecheck', 'test', 'build'];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    keys: defaultKeys,
    write: false,
    json: false,
    strict: false,
    timeoutMs: 120000
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--keys') args.keys = argv[++i].split(',').map((item) => item.trim()).filter(Boolean);
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--timeout-ms') args.timeoutMs = Number(argv[++i]);
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (args.write && !args.slug) throw new Error('--write requires --slug');
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function runRunner(args, key) {
  const runner = path.join(args.target, 'scripts', 'harness', 'verification-runner.js');
  const script = fs.existsSync(runner) ? runner : path.join(__dirname, 'verification-runner.js');
  const childArgs = ['--target', args.target, '--command-key', key, '--timeout-ms', String(args.timeoutMs), '--json'];
  if (args.slug) childArgs.push('--slug', args.slug);
  if (args.write) childArgs.push('--write');
  if (args.strict) childArgs.push('--strict');
  const result = spawnSync(process.execPath, [script, ...childArgs], {
    cwd: args.target,
    encoding: 'utf8',
    timeout: args.timeoutMs + 5000,
    maxBuffer: 1024 * 1024 * 20
  });
  let parsed = null;
  try {
    parsed = JSON.parse(result.stdout || '{}');
  } catch (error) {
    parsed = { parseError: error.message };
  }
  return {
    key,
    exitCode: typeof result.status === 'number' ? result.status : 1,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    error: result.error ? result.error.message : null,
    parsed
  };
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    slug: args.slug,
    pass: [],
    warn: [],
    fail: [],
    details: {
      keys: args.keys,
      results: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }
  if (args.write) {
    const evidenceDir = path.join(args.target, 'docs', 'evidence', args.slug);
    if (!fs.existsSync(evidenceDir) || !fs.statSync(evidenceDir).isDirectory()) {
      add(report, 'fail', `Evidence directory must exist before --write: docs/evidence/${args.slug}/`);
      return report;
    }
  }

  for (const key of args.keys) {
    const result = runRunner(args, key);
    report.details.results.push({
      key,
      exitCode: result.exitCode,
      error: result.error,
      pass: result.parsed && result.parsed.pass ? result.parsed.pass : [],
      warn: result.parsed && result.parsed.warn ? result.parsed.warn : [],
      fail: result.parsed && result.parsed.fail ? result.parsed.fail : []
    });
    if (result.exitCode === 0 && (!result.parsed.fail || result.parsed.fail.length === 0)) {
      add(report, 'pass', `${key}: verification runner exit 0`);
    } else {
      add(report, args.strict || args.write ? 'fail' : 'warn', `${key}: verification runner exit ${result.exitCode}`);
    }
  }

  if (!args.write) add(report, 'warn', 'PLAN ONLY: dry run did not execute verification commands');
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness verification suite: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
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
