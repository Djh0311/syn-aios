#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const {
  classifyMemoryTrust,
  normalizeMemoryCandidate,
  validateMemoryCandidate
} = require('./lib/memory-governance');

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

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function candidateDir(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory', 'candidates');
}

function candidateFiles(targetRoot) {
  const dir = candidateDir(targetRoot);
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .map((name) => path.join(dir, name))
    .sort();
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
  }
}

function readAuthorityText(targetRoot) {
  const files = [
    'AGENTS.md',
    'docs/current-state.md',
    'docs/decisions.md',
    'docs/sprint-contract.md'
  ];
  return files.map((relativePath) => {
    const filePath = path.join(targetRoot, relativePath);
    if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) return '';
    return fs.readFileSync(filePath, 'utf8');
  }).join('\n');
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      candidateDir: rel(args.target, candidateDir(args.target)),
      candidates: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const files = candidateFiles(args.target);
  if (files.length === 0) {
    add(report, args.strict ? 'warn' : 'pass', 'No memory candidate files found');
    return report;
  }

  const projectContext = {
    targetRoot: args.target,
    authorityText: readAuthorityText(args.target),
    staleAfterDays: 30
  };
  for (const filePath of files) {
    const parsed = readJson(filePath);
    if (parsed.error) {
      add(report, 'fail', `${rel(args.target, filePath)} could not be parsed: ${parsed.error}`);
      continue;
    }
    const candidate = normalizeMemoryCandidate(parsed.data);
    const validation = validateMemoryCandidate(candidate, { projectContext });
    const classification = classifyMemoryTrust(candidate, projectContext);
    report.details.candidates.push({
      file: rel(args.target, filePath),
      id: candidate.id,
      status: candidate.status,
      recommendedStatus: classification.recommendedStatus,
      valid: validation.valid,
      errors: validation.errors,
      warnings: validation.warnings
    });

    if (validation.valid) add(report, 'pass', `${candidate.id}: valid candidate`);
    else for (const error of validation.errors) add(report, 'fail', `${candidate.id}: ${error}`);
    for (const warning of validation.warnings) add(report, 'warn', `${candidate.id}: ${warning}`);
    if (args.strict && classification.recommendedStatus === 'quarantined') {
      add(report, 'fail', `${candidate.id}: strict lint refuses quarantined memory candidate`);
    }
  }
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness memory candidate lint: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
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
