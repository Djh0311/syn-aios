#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const realBrowserPatterns = [
  /Chrome DevTools MCP/i,
  /Codex in-app Browser/i,
  /Playwright/i,
  /browser console/i,
  /\bconsole\b/i,
  /\bnetwork\b/i,
  /\bscreenshot\b/i,
  /\bviewport\b/i,
  /\binteraction\b/i,
  /\bclick\b/i,
  /\btyped?\b/i
];

const httpOnlyPattern = /HTTP-only|reachability/i;

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
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

function evidenceDirs(args) {
  const root = path.join(args.target, 'docs', 'evidence');
  if (args.slug) return [path.join(root, args.slug)];
  if (!fs.existsSync(root)) return [];
  return fs.readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(root, entry.name));
}

function readIfExists(filePath) {
  if (!fs.existsSync(filePath)) return null;
  return fs.readFileSync(filePath, 'utf8');
}

function scanDir(targetRoot, dir) {
  const browserFile = path.join(dir, 'browser-check.md');
  const consoleFile = path.join(dir, 'console-network.md');
  const files = [browserFile, consoleFile].filter((file) => fs.existsSync(file));
  const text = files.map((file) => readIfExists(file) || '').join('\n\n');
  const matchedSignals = realBrowserPatterns
    .filter((pattern) => pattern.test(text))
    .map((pattern) => String(pattern));
  const httpOnly = httpOnlyPattern.test(text);
  return {
    slug: path.basename(dir),
    dir: rel(targetRoot, dir),
    files: files.map((file) => rel(targetRoot, file)),
    hasBrowserFile: fs.existsSync(browserFile),
    hasConsoleNetworkFile: fs.existsSync(consoleFile),
    httpOnly,
    matchedSignals,
    hasRealBrowserEvidence: matchedSignals.length >= 3 && !httpOnly
  };
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
      slug: args.slug,
      evidence: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) {
    add(report, 'pass', 'Source package detected; runtime browser evidence is not required');
    return report;
  }

  const dirs = evidenceDirs(args).filter((dir) => fs.existsSync(dir) && fs.statSync(dir).isDirectory());
  if (dirs.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', args.slug ? `No evidence directory found for slug: ${args.slug}` : 'No evidence directories found');
    return report;
  }

  report.details.evidence = dirs.map((dir) => scanDir(args.target, dir));
  const real = report.details.evidence.filter((item) => item.hasRealBrowserEvidence);
  const httpOnly = report.details.evidence.filter((item) => item.httpOnly && !item.hasRealBrowserEvidence);
  const missingFiles = report.details.evidence.filter((item) => !item.hasBrowserFile || !item.hasConsoleNetworkFile);

  if (real.length > 0) add(report, 'pass', `Real browser evidence candidates found: ${real.map((item) => item.slug).join(', ')}`);
  if (httpOnly.length > 0) add(report, args.strict ? 'fail' : 'warn', `HTTP-only UI evidence is not complete browser verification: ${httpOnly.map((item) => item.slug).join(', ')}`);
  if (missingFiles.length > 0) add(report, args.strict ? 'fail' : 'warn', `Missing browser evidence files: ${missingFiles.map((item) => item.slug).join(', ')}`);
  if (real.length === 0 && httpOnly.length === 0 && missingFiles.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', 'No recognizable real browser evidence markers found');
  }

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
  console.log(`Harness browser evidence check: ${report.target}`);
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
