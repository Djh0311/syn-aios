#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    requirementId: null,
    maxAgeHours: null,
    type: 'any',
    status: 'any',
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--requirement-id') args.requirementId = argv[++i];
    else if (arg === '--max-age-hours') args.maxAgeHours = Number(argv[++i]);
    else if (arg === '--type') args.type = argv[++i];
    else if (arg === '--status') args.status = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!['command', 'browser', 'summary', 'any'].includes(args.type)) throw new Error('--type must be command, browser, summary, or any');
  if (!['pass', 'fail', 'warn', 'unknown', 'any'].includes(args.status)) throw new Error('--status must be pass, fail, warn, unknown, or any');
  if (args.maxAgeHours !== null && (!Number.isFinite(args.maxAgeHours) || args.maxAgeHours < 0)) throw new Error('--max-age-hours must be a non-negative number');
  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (_error) {
    return null;
  }
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

function timestamps(text) {
  const matches = [];
  const pattern = /(?:createdAt|recordedAt|Last Updated):\s*([0-9]{4}-[0-9]{2}-[0-9]{2}(?:T[0-9:.Z+-]+)?)/gi;
  for (const match of text.matchAll(pattern)) matches.push(match[1]);
  return matches;
}

function typeFor(file, text) {
  if (/browser|console-network|screenshot|viewport/i.test(file) || /Chrome DevTools MCP|browser console|network|screenshot|viewport|interaction/i.test(text)) return 'browser';
  if (/test-output|command|verification/i.test(file) || /\$ |command:|exitCode|exit 0|exit [1-9]/i.test(text)) return 'command';
  return 'summary';
}

function statusFor(text) {
  if (/\b(fail|failed|failure|exit\s+[1-9][0-9]*)\b/i.test(text)) return 'fail';
  if (/\b(warn|warning|concern)\b/i.test(text)) return 'warn';
  if (/\b(pass|passed|success|exit\s+0)\b/i.test(text)) return 'pass';
  return 'unknown';
}

function requirements(text) {
  return Array.from(new Set(Array.from(text.matchAll(/\bR-[0-9A-Za-z._-]+\b/g)).map((match) => match[0]))).sort();
}

function buildIndex(targetRoot) {
  const evidenceRoot = path.join(targetRoot, 'docs', 'evidence');
  const indexPath = path.join(evidenceRoot, 'index.json');
  const existing = readJson(indexPath);
  if (existing && Array.isArray(existing.archives)) return existing;
  const archives = [];
  if (!fs.existsSync(evidenceRoot)) return { archives };
  const dirs = fs.readdirSync(evidenceRoot, { withFileTypes: true }).filter((entry) => entry.isDirectory()).map((entry) => path.join(evidenceRoot, entry.name));
  for (const dir of dirs) {
    const archive = {
      slug: path.basename(dir),
      latestTimestamp: null,
      requirementIds: [],
      files: []
    };
    const reqs = new Set();
    const allTimestamps = [];
    for (const file of walk(dir)) {
      const text = fs.readFileSync(file, 'utf8');
      const ts = timestamps(text);
      const fileReqs = requirements(text);
      for (const req of fileReqs) reqs.add(req);
      allTimestamps.push(...ts);
      archive.files.push({
        file: path.relative(targetRoot, file).split(path.sep).join('/'),
        type: typeFor(file, text),
        status: statusFor(text),
        timestamps: ts,
        requirementIds: fileReqs
      });
    }
    archive.latestTimestamp = allTimestamps.sort().pop() || null;
    archive.requirementIds = Array.from(reqs).sort();
    archives.push(archive);
  }
  return { archives };
}

function isFresh(latestTimestamp, maxAgeHours) {
  if (maxAgeHours === null) return true;
  if (!latestTimestamp) return false;
  const time = Date.parse(latestTimestamp);
  if (!Number.isFinite(time)) return false;
  return Date.now() - time <= maxAgeHours * 60 * 60 * 1000;
}

function matchesArchive(args, archive) {
  if (args.slug && archive.slug !== args.slug) return false;
  if (args.requirementId && !archive.requirementIds.includes(args.requirementId)) return false;
  if (!isFresh(archive.latestTimestamp, args.maxAgeHours)) return false;
  if (args.type !== 'any' && !archive.files.some((file) => file.type === args.type)) return false;
  if (args.status !== 'any' && !archive.files.some((file) => file.status === args.status)) return false;
  return true;
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
      query: {
        slug: args.slug,
        requirementId: args.requirementId,
        maxAgeHours: args.maxAgeHours,
        type: args.type,
        status: args.status
      },
      matches: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) {
    add(report, 'pass', 'Source package detected; runtime evidence query is not required');
    return report;
  }

  const index = buildIndex(args.target);
  report.details.matches = index.archives.filter((archive) => matchesArchive(args, archive));
  if (report.details.matches.length > 0) add(report, 'pass', `Matching evidence archive(s): ${report.details.matches.length}`);
  else add(report, args.strict ? 'fail' : 'warn', 'No evidence archives matched query');
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness evidence query: ${report.target}`);
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
