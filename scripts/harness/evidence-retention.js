#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const DEFAULT_MAX_AGE_DAYS = 30;

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    maxAgeDays: DEFAULT_MAX_AGE_DAYS,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--max-age-days') args.maxAgeDays = Number(argv[++i]);
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (!Number.isFinite(args.maxAgeDays) || args.maxAgeDays < 0) throw new Error('--max-age-days must be a non-negative number');
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
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function timestampMs(text) {
  let latest = null;
  const pattern = /(?:createdAt|recordedAt|Last Updated):\s*([0-9]{4}-[0-9]{2}-[0-9]{2}(?:T[0-9:.Z+-]+)?)/gi;
  for (const match of String(text || '').matchAll(pattern)) {
    const ms = Date.parse(match[1]);
    if (Number.isFinite(ms) && (latest === null || ms > latest)) latest = ms;
  }
  return latest;
}

function classify(text) {
  const value = String(text || '');
  const lower = value.toLowerCase();
  const reasons = [];
  if (/\bstrict\b|strict path/.test(lower)) reasons.push('strict');
  if (/\bsecurity\b|auth|permission|secret/.test(lower)) reasons.push('security');
  if (/\b(fail|failed|failure|exit\s+[1-9][0-9]*)\b/.test(lower)) reasons.push('failed');
  return {
    protected: reasons.length > 0,
    reasons,
    status: reasons.includes('failed') ? 'failed' : 'passing'
  };
}

function archiveInfo(targetRoot, dir, now, maxAgeDays) {
  const files = walk(dir).sort((a, b) => a.localeCompare(b));
  const combined = files.map((file) => fs.readFileSync(file, 'utf8')).join('\n');
  const latestTimestampMs = timestampMs(combined);
  const ageDays = latestTimestampMs === null ? null : Math.floor((now.getTime() - latestTimestampMs) / 86400000);
  const type = classify(combined);
  const stale = ageDays !== null && ageDays > maxAgeDays;
  const action = type.protected || !stale ? 'keep' : 'archive';

  return {
    slug: path.basename(dir),
    dir: rel(targetRoot, dir),
    files: files.map((file) => rel(targetRoot, file)),
    latestTimestamp: latestTimestampMs === null ? null : new Date(latestTimestampMs).toISOString(),
    ageDays,
    status: type.status,
    protectedReasons: type.reasons,
    action,
    reason: action === 'archive'
      ? `older than ${maxAgeDays} day(s) and not protected`
      : (type.reasons.length ? `protected: ${type.reasons.join(', ')}` : 'fresh or undated')
  };
}

function retentionRoot(targetRoot) {
  return path.join(targetRoot, 'docs', 'evidence', '.retained');
}

function moveArchive(targetRoot, item) {
  const source = path.join(targetRoot, item.dir);
  const destination = path.join(retentionRoot(targetRoot), item.slug);
  if (!fs.existsSync(source)) return null;
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  if (fs.existsSync(destination)) throw new Error(`Retention destination already exists: ${rel(targetRoot, destination)}`);
  fs.renameSync(source, destination);
  return rel(targetRoot, destination);
}

function buildReport(args) {
  const now = new Date();
  const evidenceRoot = path.join(args.target, 'docs', 'evidence');
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      maxAgeDays: args.maxAgeDays,
      evidenceRoot: rel(args.target, evidenceRoot),
      archives: [],
      wrote: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing installed-project directory: ${args.target}`);
    return report;
  }

  if (!fs.existsSync(evidenceRoot)) {
    add(report, 'warn', 'No docs/evidence directory found');
    return report;
  }

  const dirs = fs.readdirSync(evidenceRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
    .map((entry) => path.join(evidenceRoot, entry.name));
  report.details.archives = dirs.map((dir) => archiveInfo(args.target, dir, now, args.maxAgeDays));
  const archiveItems = report.details.archives.filter((item) => item.action === 'archive');
  const keptProtected = report.details.archives.filter((item) => item.action === 'keep' && item.protectedReasons.length > 0);

  add(report, 'pass', `Scanned ${report.details.archives.length} evidence archive(s)`);
  add(report, 'pass', `Protected archive(s) kept by default: ${keptProtected.length}`);
  if (archiveItems.length > 0) add(report, 'warn', `Archive candidate(s): ${archiveItems.map((item) => item.slug).join(', ')}`);
  else add(report, 'pass', 'No archive candidates found');
  if (!args.write) {
    add(report, 'warn', 'Dry run only; no evidence files were moved');
    return report;
  }

  for (const item of archiveItems) {
    const movedTo = moveArchive(args.target, item);
    if (movedTo) report.details.wrote.push(movedTo);
  }
  add(report, 'pass', `Moved ${report.details.wrote.length} archive candidate(s) into ${rel(args.target, retentionRoot(args.target))}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness evidence retention: ${report.target}`);
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
