#!/usr/bin/env node

const path = require('path');
const { spawnSync } = require('child_process');
const { loadHarnessConfig } = require('./lib/config-loader');
const { memoryConfig } = require('./lib/agentmemory-client');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    strict: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function run(scriptName, args, extraArgs) {
  const script = path.join(__dirname, scriptName);
  const result = spawnSync(process.execPath, [script, ...extraArgs], {
    cwd: args.target,
    encoding: 'utf8',
    timeout: 60000,
    maxBuffer: 1024 * 1024 * 10
  });
  let data = null;
  if (result.stdout && result.stdout.trim().startsWith('{')) {
    try {
      data = JSON.parse(result.stdout);
    } catch {
      data = null;
    }
  }
  return {
    script: scriptName,
    exitCode: typeof result.status === 'number' ? result.status : 1,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    error: result.error ? result.error.message : null,
    data
  };
}

function outputEvidence(command) {
  return String(`${command.stdout}\n${command.stderr}`)
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter(Boolean)
    .slice(0, 12);
}

function countStatuses(commands) {
  const counts = {
    candidate: 0,
    approved: 0,
    quarantined: 0,
    stale: 0,
    revoked: 0,
    unknown: 0
  };
  const seen = new Set();
  for (const command of commands) {
    const candidates = command.data && command.data.details && Array.isArray(command.data.details.candidates)
      ? command.data.details.candidates
      : [];
    for (const candidate of candidates) {
      const key = candidate.id || candidate.file || JSON.stringify(candidate);
      if (seen.has(key)) continue;
      seen.add(key);
      const status = Object.prototype.hasOwnProperty.call(counts, candidate.status) ? candidate.status : 'unknown';
      counts[status] += 1;
    }
  }
  return counts;
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      commands: [],
      statusCounts: null,
      memoryEnabled: false
    }
  };

  const lintArgs = ['--target', args.target, '--json'];
  if (args.strict) lintArgs.push('--strict');
  const lint = run('memory-candidate-lint.js', args, lintArgs);
  report.details.commands.push(Object.assign({ name: 'memory-candidate-lint' }, lint, { evidence: outputEvidence(lint) }));
  if (lint.exitCode === 0 && !lint.error) add(report, 'pass', 'memory-candidate-lint completed');
  else add(report, args.strict ? 'fail' : 'warn', 'memory-candidate-lint reported issues');

  const staleArgs = ['--target', args.target, '--json'];
  if (args.strict) staleArgs.push('--strict');
  const stale = run('memory-stale-check.js', args, staleArgs);
  report.details.commands.push(Object.assign({ name: 'memory-stale-check' }, stale, { evidence: outputEvidence(stale) }));
  if (stale.exitCode === 0 && !stale.error) add(report, 'pass', 'memory-stale-check completed');
  else add(report, args.strict ? 'fail' : 'warn', 'memory-stale-check reported issues');

  const loaded = loadHarnessConfig(args.target);
  if (loaded.error) {
    add(report, 'warn', `Harness config could not be loaded for memory backend check: ${loaded.error}`);
  } else {
    const resolved = memoryConfig(loaded.data || {});
    report.details.memoryEnabled = resolved.enabled;
    if (!resolved.enabled) {
      add(report, 'warn', 'memoryIntegration.enabled is false; skipped agentmemory backend health');
    } else {
      const health = run('memory-agentmemory-query.js', args, [
        '--target',
        args.target,
        '--query',
        'memory maintenance health check',
        '--limit',
        '1',
        '--json'
      ]);
      report.details.commands.push(Object.assign({ name: 'memory-agentmemory-query' }, health, { evidence: outputEvidence(health) }));
      if (health.exitCode === 0 && !health.error) add(report, 'pass', 'agentmemory wrapper check completed');
      else add(report, 'warn', 'agentmemory wrapper check was unavailable or reported issues');
    }
  }

  report.details.statusCounts = countStatuses(report.details.commands);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness memory maintenance: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log('\nSTATUS COUNTS');
  for (const [status, count] of Object.entries(report.details.statusCounts || {})) {
    console.log(`  - ${status}: ${count}`);
  }
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
