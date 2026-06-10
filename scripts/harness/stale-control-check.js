#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const runtimeDocs = [
  'docs/current-state.md',
  'docs/requirements-matrix.md',
  'docs/task-queue.md',
  'docs/decisions.md',
  'docs/open-questions.md',
  'docs/context-checkpoints.md',
  'docs/sprint-contract.md',
  'docs/agent-mistake-ledger.md'
];

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

function scanFile(targetRoot, relativePath) {
  const file = path.join(targetRoot, relativePath);
  if (!fs.existsSync(file)) return null;
  const text = fs.readFileSync(file, 'utf8');
  const warnings = [];
  const failures = [];
  const lines = text.split(/\r?\n/);
  lines.forEach((line, index) => {
    if (/Last Updated:\s*TBD/i.test(line)) warnings.push({ line: index + 1, kind: 'last-updated-tbd', text: line });
    if (/\bTBD\b/i.test(line)) warnings.push({ line: index + 1, kind: 'tbd-placeholder', text: line });
    const looksLikeStatusRow = /^\s*\|.*\bDone\b.*\|/.test(line) || /^\s*[-*]\s*Status:\s*Done\b/i.test(line);
    const hasEvidenceMarker = /evidence|docs\/evidence|verification|verified/i.test(line);
    if (looksLikeStatusRow && !hasEvidenceMarker) {
      failures.push({ line: index + 1, kind: 'done-without-evidence-marker', text: line });
    }
  });
  const tbdCount = (text.match(/\bTBD\b/g) || []).length;
  return { file: relativePath, tbdCount, warnings, failures };
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      files: []
    }
  };
  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }
  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) {
    add(report, 'pass', 'Source package detected; runtime control docs are templates only');
    return report;
  }
  report.details.files = runtimeDocs.map((file) => scanFile(args.target, file)).filter(Boolean);
  if (report.details.files.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', 'No runtime control docs found');
    return report;
  }
  const placeholderFiles = report.details.files.filter((file) => file.tbdCount > 0 || file.warnings.length > 0);
  const weakDoneFiles = report.details.files.filter((file) => file.failures.length > 0);
  if (placeholderFiles.length === 0 && weakDoneFiles.length === 0) {
    add(report, 'pass', 'No stale control-doc markers found');
  }
  if (placeholderFiles.length > 0) {
    add(report, 'warn', `Runtime control docs contain placeholders to resolve: ${placeholderFiles.map((file) => file.file).join(', ')}`);
  }
  if (weakDoneFiles.length > 0) {
    add(report, args.strict ? 'fail' : 'warn', `Runtime control docs contain Done status without evidence markers: ${weakDoneFiles.map((file) => file.file).join(', ')}`);
  }
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness stale control check: ${report.target}`);
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
