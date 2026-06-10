#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { loadHarnessConfig } = require('./lib/config-loader');
const { agentmemoryRemember, memoryConfig } = require('./lib/agentmemory-client');
const { normalizeMemoryCandidate, validateMemoryCandidate } = require('./lib/memory-governance');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    file: null,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--file') args.file = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (args.file) args.file = path.resolve(args.target, args.file);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function auditPath(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory', 'audit.jsonl');
}

function appendAudit(targetRoot, entry) {
  const filePath = auditPath(targetRoot);
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.appendFileSync(filePath, `${JSON.stringify(entry)}\n`, 'utf8');
  return filePath;
}

async function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      file: args.file,
      candidate: null
    }
  };

  if (!args.file || !fs.existsSync(args.file)) {
    add(report, 'fail', '--file must point to an existing memory candidate JSON file');
    return report;
  }
  const loaded = loadHarnessConfig(args.target);
  if (loaded.error) {
    add(report, 'fail', `Harness config could not be loaded: ${loaded.error}`);
    return report;
  }
  const config = loaded.data || {};
  const resolved = memoryConfig(config);
  if (!resolved.enabled) {
    add(report, 'warn', 'memoryIntegration.enabled is false; skipped agentmemory save');
    return report;
  }

  const candidate = normalizeMemoryCandidate(JSON.parse(fs.readFileSync(args.file, 'utf8')));
  report.details.candidate = candidate;
  const validation = validateMemoryCandidate(candidate, { projectContext: { targetRoot: args.target } });
  if (!validation.valid) {
    for (const error of validation.errors) add(report, 'fail', error);
    return report;
  }
  if (candidate.status !== 'approved') add(report, 'fail', 'Only approved memory candidates may be saved to agentmemory');
  if (candidate.authority === 'candidate') add(report, 'fail', 'Candidate authority is too weak for agentmemory save');
  if (candidate.evidenceRefs.length === 0 && candidate.authority !== 'user-confirmed') {
    add(report, 'fail', 'agentmemory save requires evidenceRefs or user-confirmed authority');
  }
  if (report.fail.length > 0) return report;
  if (!args.write) {
    add(report, 'warn', 'Dry run only; approved memory was not sent to agentmemory');
    return report;
  }

  const response = await agentmemoryRemember(config, candidate);
  if (!response.ok) {
    add(report, 'warn', `agentmemory remember failed: ${response.error || `HTTP ${response.statusCode}`}`);
    return report;
  }
  const audit = appendAudit(args.target, {
    at: new Date().toISOString(),
    id: candidate.id,
    action: 'agentmemory-save',
    sourceFile: path.relative(args.target, args.file).split(path.sep).join('/'),
    authority: candidate.authority,
    evidenceRefs: candidate.evidenceRefs
  });
  report.details.audit = path.relative(args.target, audit).split(path.sep).join('/');
  add(report, 'pass', `Saved approved memory candidate to agentmemory: ${candidate.id}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness agentmemory save: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
}

(async () => {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = await buildReport(args);
    if (args.json) console.log(JSON.stringify(report, null, 2));
    else printReport(report);
    if (report.fail.length > 0) process.exit(1);
  } catch (error) {
    console.error(`ERROR: ${error.message}`);
    process.exit(1);
  }
})();
