#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const ignoredNames = new Set(['.DS_Store', '.git', 'node_modules', 'dist', 'build', '.next', 'coverage']);
const timestampPattern = /\b(?:recordedAt|createdAt):\s*([0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:.+-]+Z?)\b/g;
const defaultMaxAgeHours = 24;

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    maxAgeHours: defaultMaxAgeHours,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--max-age-hours') args.maxAgeHours = Number(argv[++i]);
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!Number.isFinite(args.maxAgeHours) || args.maxAgeHours <= 0) {
    throw new Error('--max-age-hours must be a positive number');
  }

  args.target = path.resolve(args.target);
  if (args.slug) {
    args.slug = String(args.slug).trim();
    if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(args.slug)) {
      throw new Error('--slug must be a safe single path segment');
    }
  }
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function collectEvidenceFiles(args) {
  const evidenceRoot = path.join(args.target, 'docs', 'evidence');
  if (args.slug) {
    const slugDir = path.join(evidenceRoot, args.slug);
    return fs.existsSync(slugDir) ? walk(slugDir) : [];
  }
  return walk(evidenceRoot);
}

function timestampsFromText(text) {
  const timestamps = [];
  for (const match of text.matchAll(timestampPattern)) {
    const date = new Date(match[1]);
    if (!Number.isNaN(date.getTime())) timestamps.push(date);
  }
  return timestamps;
}

function scanFile(targetRoot, filePath, nowMs) {
  const text = fs.readFileSync(filePath, 'utf8');
  const timestamps = timestampsFromText(text);
  const newest = timestamps.length > 0
    ? timestamps.reduce((latest, value) => (value.getTime() > latest.getTime() ? value : latest), timestamps[0])
    : null;
  const ageHours = newest ? (nowMs - newest.getTime()) / (1000 * 60 * 60) : null;
  return {
    file: rel(targetRoot, filePath),
    timestamps: timestamps.map((value) => value.toISOString()),
    newest: newest ? newest.toISOString() : null,
    ageHours
  };
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    maxAgeHours: args.maxAgeHours,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      slug: args.slug,
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
    add(report, 'pass', 'Source package detected; runtime evidence freshness is not required');
    return report;
  }

  const files = collectEvidenceFiles(args).filter((file) => /\.md$/i.test(file));
  if (args.slug && files.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', `No evidence files found for slug: ${args.slug}`);
    return report;
  }
  if (!args.slug && files.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', 'No markdown evidence files found under docs/evidence/**');
    return report;
  }

  const nowMs = Date.now();
  report.details.files = files.map((file) => scanFile(args.target, file, nowMs));
  const timestamped = report.details.files.filter((file) => file.newest);
  const fresh = timestamped.filter((file) => file.ageHours <= args.maxAgeHours);
  const stale = timestamped.filter((file) => file.ageHours > args.maxAgeHours);
  const missingTimestamp = report.details.files.filter((file) => !file.newest);

  if (fresh.length > 0) add(report, 'pass', `Fresh timestamped evidence files: ${fresh.length}`);
  if (timestamped.length === 0) add(report, args.strict ? 'fail' : 'warn', 'Evidence files exist but no recordedAt/createdAt timestamps were found');
  if (stale.length > 0) add(report, args.strict ? 'fail' : 'warn', `Stale evidence files older than ${args.maxAgeHours} hour(s): ${stale.map((file) => file.file).join(', ')}`);
  if (missingTimestamp.length > 0) add(report, 'warn', `Evidence files without timestamps: ${missingTimestamp.map((file) => file.file).join(', ')}`);

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
  console.log(`Harness evidence freshness: ${report.target}`);
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
