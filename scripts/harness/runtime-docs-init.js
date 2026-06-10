#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const ignoredNames = new Set(['.DS_Store']);

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    templateRoot: null,
    docsRoot: null,
    write: false,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--template-root') args.templateRoot = argv[++i];
    else if (arg === '--docs-root') args.docsRoot = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (!arg.startsWith('--')) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  args.templateRoot = path.resolve(args.templateRoot || path.join(args.target, 'templates', 'docs'));
  args.docsRoot = path.resolve(args.docsRoot || path.join(args.target, 'docs'));
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

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/');
}

function copyMissing(source, target, write) {
  if (!write) return;
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target, fs.constants.COPYFILE_EXCL);
}

function buildReport(args) {
  const report = {
    mode: args.write ? 'write' : 'dry-run',
    target: args.target,
    templateRoot: args.templateRoot,
    docsRoot: args.docsRoot,
    pass: [],
    warn: [],
    fail: [],
    create: [],
    skipExisting: [],
    missingTemplateRoot: false
  };

  if (!fs.existsSync(args.templateRoot)) {
    report.missingTemplateRoot = true;
    report.fail.push(`Template root was not found: ${args.templateRoot}`);
    return report;
  }

  for (const source of walk(args.templateRoot)) {
    const relativePath = rel(args.templateRoot, source);
    const target = path.join(args.docsRoot, relativePath);
    const item = {
      file: relativePath,
      source,
      target,
      reason: fs.existsSync(target) ? 'Runtime doc already exists; never overwritten' : 'Missing runtime doc'
    };

    if (fs.existsSync(target)) {
      report.skipExisting.push(item);
      continue;
    }

    report.create.push(item);
    copyMissing(source, target, args.write);
  }

  if (report.create.length > 0) report.warn.push(`${args.write ? 'Created' : 'Would create'} missing runtime docs: ${report.create.length}`);
  if (report.skipExisting.length > 0) report.pass.push(`Existing runtime docs protected: ${report.skipExisting.length}`);
  if (report.create.length === 0 && report.skipExisting.length > 0) report.pass.push('No missing runtime docs detected');

  return report;
}

function printItems(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item.target} (${item.reason})`);
}

function printReport(report) {
  console.log(`Runtime docs init ${report.mode} report`);
  console.log(`Template root: ${report.templateRoot}`);
  console.log(`Docs root: ${report.docsRoot}`);
  console.log('Overwrite policy: never overwrite existing docs/**');
  printItems('PASS', report.pass.map((message) => ({ target: message, reason: '' })));
  printItems('WARN', report.warn.map((message) => ({ target: message, reason: '' })));
  printItems('FAIL', report.fail.map((message) => ({ target: message, reason: '' })));

  if (report.missingTemplateRoot) {
    console.log('\nERROR');
    console.log('  Template root was not found');
    return;
  }

  printItems(report.mode === 'write' ? 'CREATE' : 'WOULD_CREATE', report.create);
  printItems('SKIP_EXISTING', report.skipExisting);

  if (report.mode === 'dry-run') console.log('\nDry run only. Re-run with --write to create missing docs/** files.');
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.missingTemplateRoot) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
