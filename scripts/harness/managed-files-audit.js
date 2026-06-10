#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { readManifest, sha256File, manifestRelativePath } = require('./lib/manifest');

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
    else if (!arg.startsWith('--')) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function normalizeRelative(relativePath) {
  return relativePath.split(path.sep).join('/');
}

function makeItem(targetRoot, relativePath, entry) {
  const target = path.join(targetRoot, relativePath);
  const item = {
    file: normalizeRelative(relativePath),
    target,
    status: 'missing',
    installedSha256: entry.installedSha256 || null,
    currentSha256: null,
    source: entry.source || null,
    sourceSha256: entry.sourceSha256 || null
  };

  if (!fs.existsSync(target)) return item;

  item.currentSha256 = sha256File(target);
  item.status = item.currentSha256 === item.installedSha256 ? 'unchanged' : 'local-modified';
  return item;
}

function buildReport(args) {
  const manifest = readManifest(args.target);
  const report = {
    target: args.target,
    manifest: manifest.path,
    manifestError: manifest.error,
    pass: [],
    warn: [],
    fail: [],
    counts: {
      unchanged: 0,
      localModified: 0,
      missing: 0
    },
    unchanged: [],
    localModified: [],
    missing: []
  };

  if (manifest.error) {
    report.fail.push(`Manifest could not be read: ${manifest.error}`);
    return report;
  }
  if (!manifest.data) {
    report.warn.push(`${manifestRelativePath} was not found`);
    return report;
  }

  const files = manifest.data.files || {};
  for (const relativePath of Object.keys(files).sort()) {
    const item = makeItem(args.target, relativePath, files[relativePath]);
    if (item.status === 'unchanged') {
      report.unchanged.push(item);
      report.counts.unchanged += 1;
    } else if (item.status === 'local-modified') {
      report.localModified.push(item);
      report.counts.localModified += 1;
    } else {
      report.missing.push(item);
      report.counts.missing += 1;
    }
  }

  if (report.counts.unchanged > 0) report.pass.push(`Managed files unchanged: ${report.counts.unchanged}`);
  if (report.counts.localModified > 0) report.warn.push(`Managed files locally modified: ${report.counts.localModified}`);
  if (report.counts.missing > 0) report.fail.push(`Managed files missing: ${report.counts.missing}`);
  if (Object.keys(files).length === 0) report.warn.push('Manifest contains no managed files');

  return report;
}

function printItems(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item.file}`);
}

function printReport(report) {
  console.log('Managed files audit');
  console.log(`Target: ${report.target}`);
  console.log(`Manifest: ${report.manifest}`);
  printItems('PASS', report.pass.map((message) => ({ file: message })));
  printItems('WARN', report.warn.map((message) => ({ file: message })));
  printItems('FAIL', report.fail.map((message) => ({ file: message })));

  if (report.manifestError) {
    console.log(`Manifest error: ${report.manifestError}`);
    return;
  }

  if (!fs.existsSync(report.manifest)) {
    console.log(`Manifest error: ${manifestRelativePath} was not found`);
    return;
  }

  printItems('UNCHANGED', report.unchanged);
  printItems('LOCAL_MODIFIED', report.localModified);
  printItems('MISSING', report.missing);
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
