#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const {
  isMemoryStale,
  memoryCandidateFiles,
  normalizeMemoryCandidate,
  renderMemoryCandidateMarkdown
} = require('./lib/memory-governance');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    staleAfterDays: 30,
    write: false,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--stale-after-days') args.staleAfterDays = Number.parseInt(argv[++i], 10);
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function add(report, kind, message) {
  report[kind].push(message);
}

function candidatesDir(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory', 'candidates');
}

function listCandidateFiles(targetRoot) {
  const dir = candidatesDir(targetRoot);
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
  return ['AGENTS.md', 'docs/current-state.md', 'docs/decisions.md', 'docs/sprint-contract.md']
    .map((relativePath) => {
      const filePath = path.join(targetRoot, relativePath);
      if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) return '';
      return fs.readFileSync(filePath, 'utf8');
    })
    .join('\n');
}

function writeCandidate(targetRoot, candidate) {
  const files = memoryCandidateFiles(targetRoot, candidate.id);
  fs.writeFileSync(files.json, `${JSON.stringify(candidate, null, 2)}\n`, 'utf8');
  fs.writeFileSync(files.markdown, renderMemoryCandidateMarkdown(candidate), 'utf8');
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      staleAfterDays: args.staleAfterDays,
      candidates: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const projectContext = {
    targetRoot: args.target,
    staleAfterDays: args.staleAfterDays,
    authorityText: readAuthorityText(args.target)
  };
  const files = listCandidateFiles(args.target);
  if (files.length === 0) {
    add(report, 'pass', 'No memory candidate files found');
    return report;
  }

  let staleCount = 0;
  for (const filePath of files) {
    const parsed = readJson(filePath);
    if (parsed.error) {
      add(report, 'fail', `${rel(args.target, filePath)} could not be parsed: ${parsed.error}`);
      continue;
    }
    const candidate = normalizeMemoryCandidate(parsed.data);
    const stale = isMemoryStale(candidate, projectContext);
    report.details.candidates.push({
      file: rel(args.target, filePath),
      id: candidate.id,
      status: candidate.status,
      stale: stale.stale,
      reasons: stale.reasons
    });
    if (!stale.stale) {
      add(report, 'pass', `${candidate.id}: not stale`);
      continue;
    }

    staleCount += 1;
    add(report, args.strict ? 'fail' : 'warn', `${candidate.id}: stale (${stale.reasons.join('; ')})`);
    if (args.write && candidate.status !== 'stale') {
      const next = Object.assign({}, candidate, {
        status: 'stale',
        staleTriggers: Array.from(new Set(candidate.staleTriggers.concat(stale.reasons)))
      });
      writeCandidate(args.target, next);
      add(report, 'pass', `${candidate.id}: marked stale`);
    }
  }

  if (staleCount === 0) add(report, 'pass', 'No stale memory candidates found');
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness memory stale check: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
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
