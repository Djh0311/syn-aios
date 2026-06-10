#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { readManifest, writeManifest, sha256File, targetChangedSinceInstall, manifestRelativePath } = require('./lib/manifest');

const sourceRoot = path.resolve(__dirname, '..', '..');
const ignoredNames = new Set(['.DS_Store']);
const sourceOnlyHarnessPaths = [
  'scripts/harness/fixtures/'
];

function parseArgs(argv) {
  const args = {
    target: null,
    write: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--no-overwrite') continue;
    else if (!args.target) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!args.target) throw new Error('Usage: node scripts/harness/sync-harness.js --target <dir> [--write]');
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

function sameFile(a, b) {
  if (!fs.existsSync(a) || !fs.existsSync(b)) return false;
  const left = fs.readFileSync(a);
  const right = fs.readFileSync(b);
  return left.equals(right);
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

function add(report, kind, source, target, reason) {
  report[kind].push({
    source: source ? path.relative(sourceRoot, source) : null,
    sourcePath: source || null,
    target,
    reason
  });
}

function planFile(report, sourceRelative, targetRoot, args) {
  const source = path.join(sourceRoot, sourceRelative);
  const target = path.join(targetRoot, sourceRelative);
  if (!fs.existsSync(source)) {
    add(report, 'missingSource', source, target, 'Source missing');
    return;
  }
  if (!fs.existsSync(target)) {
    add(report, 'wouldCreate', source, target, 'Missing in target');
    copyFile(source, target, args.write);
    return;
  }
  if (sameFile(source, target)) {
    add(report, 'skipExisting', source, target, 'Already up to date');
    return;
  }
  if (args.manifest && targetChangedSinceInstall(targetRoot, args.manifest, sourceRelative)) {
    add(report, 'conflict', source, target, 'Target changed since last harness manifest; not overwriting');
    return;
  }
  add(report, 'wouldUpdate', source, target, 'Rule file differs');
  copyFile(source, target, args.write);
}

function planTree(report, sourceDirRelative, targetDirRelative, targetRoot, args) {
  const sourceDir = path.join(sourceRoot, sourceDirRelative);
  for (const source of walk(sourceDir)) {
    const relativeInside = path.relative(sourceDir, source);
    const sourceRelative = normalizeRelative(path.join(sourceDirRelative, relativeInside));
    if (isSourceOnlyHarnessPath(sourceRelative)) continue;
    const target = path.join(targetRoot, targetDirRelative, relativeInside);
    if (!fs.existsSync(target)) {
      add(report, 'wouldCreate', source, target, 'Missing in target');
      copyFile(source, target, args.write);
      continue;
    }
    if (sameFile(source, target)) {
      add(report, 'skipExisting', source, target, 'Already up to date');
      continue;
    }
    const targetRelative = path.relative(targetRoot, target).split(path.sep).join('/');
    if (args.manifest && targetChangedSinceInstall(targetRoot, args.manifest, targetRelative)) {
      add(report, 'conflict', source, target, 'Target changed since last harness manifest; not overwriting');
      continue;
    }
    add(report, 'wouldUpdate', source, target, 'Rule file differs');
    copyFile(source, target, args.write);
  }
}

function planRuntimeDocs(report, targetRoot) {
  const sourceDir = path.join(sourceRoot, 'templates/docs');
  for (const source of walk(sourceDir)) {
    const relativeInside = path.relative(sourceDir, source);
    const target = path.join(targetRoot, 'docs', relativeInside);
    if (!fs.existsSync(target)) {
      add(report, 'wouldCreate', source, target, 'Missing runtime doc can be initialized');
      continue;
    }
    add(report, 'skipRuntimeDoc', source, target, 'Protected installed-project runtime state');
  }
}

function printReport(report, args) {
  console.log(`Harness sync ${args.write ? 'write' : 'dry-run'} report`);
  console.log(`Source: ${sourceRoot}`);
  console.log(`Target: ${args.target}`);
  console.log('Runtime docs overwrite: never during sync');

  for (const [title, items] of [
    ['WOULD_CREATE', report.wouldCreate],
    ['WOULD_UPDATE', report.wouldUpdate],
    ['CONFLICT', report.conflict],
    ['SKIP_EXISTING', report.skipExisting],
    ['SKIP_RUNTIME_DOC', report.skipRuntimeDoc],
    ['MISSING_SOURCE', report.missingSource]
  ]) {
    console.log(`\n${title} (${items.length})`);
    if (items.length === 0) {
      console.log('  None');
      continue;
    }
    for (const item of items) console.log(`  - ${item.target} (${item.reason})`);
  }

  console.log('\nMANIFEST');
  if (report.manifestError) console.log(`  Could not read ${manifestRelativePath}: ${report.manifestError}`);
  else if (args.write && report.conflict.length > 0) console.log(`  Not written because ${report.conflict.length} conflict(s) require review`);
  else console.log(`  ${args.write ? 'Wrote' : 'Would write'} ${path.join(args.target, manifestRelativePath)}`);

  if (!args.write) console.log('\nDry run only. Re-run with --write to sync rule files. Runtime docs will still be protected.');
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  args.target = path.resolve(args.target);
  const manifest = readManifest(args.target);
  args.manifest = manifest.data;

  const report = {
    wouldCreate: [],
    wouldUpdate: [],
    conflict: [],
    skipExisting: [],
    skipRuntimeDoc: [],
    missingSource: [],
    manifestError: manifest.error
  };

  planFile(report, 'AGENTS.md', args.target, args);
  planFile(report, 'codex-multi-agent-safe-collaboration.md', args.target, args);
  planFile(report, 'harness.config.example.json', args.target, args);
  planTree(report, 'skills', 'skills', args.target, args);
  planTree(report, 'scripts/harness', 'scripts/harness', args.target, args);
  planTree(report, 'templates', 'templates', args.target, args);
  planRuntimeDocs(report, args.target);

  if (args.write && report.conflict.length === 0) {
    const installedItems = [...report.wouldCreate, ...report.wouldUpdate, ...report.skipExisting]
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
