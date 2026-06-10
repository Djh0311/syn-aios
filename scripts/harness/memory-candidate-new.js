#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const {
  memoryCandidateFiles,
  normalizeMemoryCandidate,
  renderMemoryCandidateMarkdown,
  validateMemoryCandidate
} = require('./lib/memory-governance');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    id: null,
    claim: null,
    sourceType: 'model-summary',
    source: 'unknown',
    scope: 'project',
    project: null,
    evidenceRefs: [],
    relatedFiles: [],
    confidence: 'low',
    authority: 'candidate',
    status: 'candidate',
    riskTags: [],
    expiresAt: null,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--id') args.id = argv[++i];
    else if (arg === '--claim') args.claim = argv[++i];
    else if (arg === '--source-type') args.sourceType = argv[++i];
    else if (arg === '--source') args.source = argv[++i];
    else if (arg === '--scope') args.scope = argv[++i];
    else if (arg === '--project') args.project = argv[++i];
    else if (arg === '--evidence') args.evidenceRefs.push(argv[++i]);
    else if (arg === '--related-file') args.relatedFiles.push(argv[++i]);
    else if (arg === '--confidence') args.confidence = argv[++i];
    else if (arg === '--authority') args.authority = argv[++i];
    else if (arg === '--status') args.status = argv[++i];
    else if (arg === '--risk-tag') args.riskTags.push(argv[++i]);
    else if (arg === '--expires-at') args.expiresAt = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
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

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      candidate: null,
      files: null,
      validation: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }
  if (!args.claim || !String(args.claim).trim()) {
    add(report, 'fail', '--claim is required');
    return report;
  }

  const candidate = normalizeMemoryCandidate({
    id: args.id,
    project: args.project || path.basename(args.target),
    scope: args.scope,
    sourceType: args.sourceType,
    source: args.source,
    claim: args.claim,
    evidenceRefs: args.evidenceRefs,
    relatedFiles: args.relatedFiles,
    confidence: args.confidence,
    authority: args.authority,
    status: args.status,
    riskTags: args.riskTags,
    expiresAt: args.expiresAt
  });
  const validation = validateMemoryCandidate(candidate, {
    projectContext: { targetRoot: args.target }
  });
  const files = memoryCandidateFiles(args.target, candidate.id);
  report.details.candidate = validation.normalized;
  report.details.validation = validation;
  report.details.files = {
    json: rel(args.target, files.json),
    markdown: rel(args.target, files.markdown)
  };

  if (validation.valid) add(report, 'pass', `Memory candidate is valid: ${candidate.id}`);
  else for (const error of validation.errors) add(report, 'fail', error);
  for (const warning of validation.warnings) add(report, 'warn', warning);

  if (!args.write) {
    add(report, 'warn', 'Dry run only; memory candidate was not written');
    return report;
  }
  if (!validation.valid) return report;

  if (fs.existsSync(files.json) || fs.existsSync(files.markdown)) {
    add(report, 'fail', `Memory candidate already exists: ${rel(args.target, files.json)}`);
    return report;
  }

  fs.mkdirSync(files.dir, { recursive: true });
  fs.writeFileSync(files.json, `${JSON.stringify(validation.normalized, null, 2)}\n`, { encoding: 'utf8', flag: 'wx' });
  fs.writeFileSync(files.markdown, renderMemoryCandidateMarkdown(validation.normalized), { encoding: 'utf8', flag: 'wx' });
  add(report, 'pass', `Created memory candidate: ${rel(args.target, files.json)}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness memory candidate new: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
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
