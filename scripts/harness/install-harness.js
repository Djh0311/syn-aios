#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { writeManifest, manifestRelativePath } = require('./lib/manifest');

const sourceRoot = path.resolve(__dirname, '..', '..');
const ignoredNames = new Set(['.DS_Store']);
const sourceOnlyHarnessPaths = [
  'scripts/harness/fixtures/'
];

function parseArgs(argv) {
  const args = {
    target: null,
    write: false,
    forceRuntimeDocs: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--force-runtime-docs') args.forceRuntimeDocs = true;
    else if (!args.target) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!args.target) throw new Error('Usage: node scripts/harness/install-harness.js --target <dir> [--write] [--force-runtime-docs]');
  return args;
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

function relFrom(base, filePath) {
  return path.relative(base, filePath);
}

function normalizeRelative(relativePath) {
  return relativePath.split(path.sep).join('/');
}

function isSourceOnlyHarnessPath(relativePath) {
  const normalized = normalizeRelative(relativePath);
  return sourceOnlyHarnessPaths.some((pattern) => normalized.startsWith(pattern));
}

function copyFile(source, target, write) {
  if (!write) return;
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
}

function addPlan(report, kind, source, target, reason) {
  report[kind].push({
    source: source ? path.relative(sourceRoot, source) : null,
    sourcePath: source || null,
    target,
    reason
  });
}

function planRegularFile(report, relativePath, targetRoot, write) {
  const source = path.join(sourceRoot, relativePath);
  const target = path.join(targetRoot, relativePath);
  if (!fs.existsSync(source)) {
    addPlan(report, 'missingSource', source, target, 'Source file is missing');
    return;
  }
  if (fs.existsSync(target)) {
    addPlan(report, 'skipExisting', source, target, 'Target file already exists');
    return;
  }
  addPlan(report, 'create', source, target, 'Create missing harness file');
  copyFile(source, target, write);
}

function planTree(report, sourceDirRelative, targetDirRelative, targetRoot, options) {
  const sourceDir = path.join(sourceRoot, sourceDirRelative);
  for (const source of walk(sourceDir)) {
    const relativeInside = relFrom(sourceDir, source);
    const sourceRelative = normalizeRelative(path.join(sourceDirRelative, relativeInside));
    if (isSourceOnlyHarnessPath(sourceRelative)) continue;
    const target = path.join(targetRoot, targetDirRelative, relativeInside);
    if (fs.existsSync(target)) {
      if (options.forceExisting) {
        addPlan(report, 'update', source, target, 'Overwrite requested explicitly');
        copyFile(source, target, options.write);
      } else {
        addPlan(report, 'skipExisting', source, target, 'Target file already exists');
      }
      continue;
    }
    addPlan(report, 'create', source, target, 'Create missing file');
    copyFile(source, target, options.write);
  }
}

function printReport(report, args) {
  console.log(`Harness install ${args.write ? 'write' : 'dry-run'} report`);
  console.log(`Source: ${sourceRoot}`);
  console.log(`Target: ${args.target}`);
  console.log(`Runtime docs overwrite: ${args.forceRuntimeDocs ? 'allowed' : 'blocked by default'}`);

  for (const [title, items] of [
    ['CREATE', report.create],
    ['UPDATE', report.update],
    ['SKIP_EXISTING', report.skipExisting],
    ['MISSING_SOURCE', report.missingSource]
  ]) {
    console.log(`\n${title} (${items.length})`);
    if (items.length === 0) {
      console.log('  None');
      continue;
    }
    for (const item of items) {
      console.log(`  - ${item.target}${item.reason ? ` (${item.reason})` : ''}`);
    }
  }

  console.log('\nMANIFEST');
  console.log(`  ${args.write ? 'Wrote' : 'Would write'} ${path.join(args.target, manifestRelativePath)}`);

  if (!args.write) console.log('\nDry run only. Re-run with --write to apply.');
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  args.target = path.resolve(args.target);

  const report = {
    create: [],
    update: [],
    skipExisting: [],
    missingSource: []
  };

  planRegularFile(report, 'AGENTS.md', args.target, args.write);
  planRegularFile(report, 'codex-multi-agent-safe-collaboration.md', args.target, args.write);
  planRegularFile(report, 'harness.config.example.json', args.target, args.write);
  planTree(report, 'skills', 'skills', args.target, { write: args.write, forceExisting: false });
  planTree(report, 'scripts/harness', 'scripts/harness', args.target, { write: args.write, forceExisting: false });
  planTree(report, 'templates', 'templates', args.target, { write: args.write, forceExisting: false });
  planTree(report, 'templates/docs', 'docs', args.target, { write: args.write, forceExisting: args.forceRuntimeDocs });

  if (args.write) {
    const installedItems = [...report.create, ...report.update, ...report.skipExisting]
      .map((item) => ({
        source: item.sourcePath || (item.source ? path.join(sourceRoot, item.source) : null),
        target: item.target
      }))
      .filter((item) => item.source && item.target && fs.existsSync(item.source) && fs.existsSync(item.target));
    writeManifest(sourceRoot, args.target, installedItems);
  }

  printReport(report, args);
}

try {
  main();
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
