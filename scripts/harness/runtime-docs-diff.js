#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { sha256File } = require('./lib/manifest');

const ignoredNames = new Set(['.DS_Store']);

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    templateRoot: null,
    docsRoot: null,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--template-root') args.templateRoot = argv[++i];
    else if (arg === '--docs-root') args.docsRoot = argv[++i];
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

function fileMap(root) {
  const map = {};
  for (const file of walk(root)) map[rel(root, file)] = file;
  return map;
}

function compareFile(templateRoot, docsRoot, relativePath, templateFile, docsFile) {
  const item = {
    file: relativePath,
    template: templateFile || path.join(templateRoot, relativePath),
    docs: docsFile || path.join(docsRoot, relativePath),
    status: 'missing-doc',
    templateSha256: null,
    docsSha256: null
  };

  if (templateFile) item.templateSha256 = sha256File(templateFile);
  if (docsFile) item.docsSha256 = sha256File(docsFile);

  if (!templateFile && docsFile) item.status = 'extra-doc';
  else if (templateFile && !docsFile) item.status = 'missing-doc';
  else if (item.templateSha256 === item.docsSha256) item.status = 'same';
  else item.status = 'different';

  return item;
}

function buildReport(args) {
  const templates = fileMap(args.templateRoot);
  const docs = fileMap(args.docsRoot);
  const allRelative = Array.from(new Set([...Object.keys(templates), ...Object.keys(docs)])).sort();
  const report = {
    target: args.target,
    templateRoot: args.templateRoot,
    docsRoot: args.docsRoot,
    pass: [],
    warn: [],
    fail: [],
    counts: {
      same: 0,
      different: 0,
      missingDoc: 0,
      extraDoc: 0
    },
    same: [],
    different: [],
    missingDoc: [],
    extraDoc: []
  };

  for (const relativePath of allRelative) {
    const item = compareFile(args.templateRoot, args.docsRoot, relativePath, templates[relativePath], docs[relativePath]);
    if (item.status === 'same') {
      report.same.push(item);
      report.counts.same += 1;
    } else if (item.status === 'different') {
      report.different.push(item);
      report.counts.different += 1;
    } else if (item.status === 'missing-doc') {
      report.missingDoc.push(item);
      report.counts.missingDoc += 1;
    } else {
      report.extraDoc.push(item);
      report.counts.extraDoc += 1;
    }
  }

  if (!fs.existsSync(args.templateRoot)) report.fail.push(`Template docs root missing: ${args.templateRoot}`);
  if (!fs.existsSync(args.docsRoot)) report.warn.push(`Runtime docs root missing: ${args.docsRoot}`);
  if (report.counts.same > 0) report.pass.push(`Runtime docs matching templates: ${report.counts.same}`);
  if (report.counts.different > 0) report.warn.push(`Runtime docs differ from templates: ${report.counts.different}`);
  if (report.counts.missingDoc > 0) report.warn.push(`Runtime docs missing from installed project: ${report.counts.missingDoc}`);
  if (report.counts.extraDoc > 0) report.pass.push(`Project-owned extra runtime docs: ${report.counts.extraDoc}`);

  return report;
}

function printItems(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) {
    const hashes = item.templateSha256 || item.docsSha256
      ? ` template=${item.templateSha256 || 'missing'} docs=${item.docsSha256 || 'missing'}`
      : '';
    console.log(`  - ${item.file}${hashes}`);
  }
}

function printReport(report) {
  console.log('Runtime docs diff');
  console.log(`Template root: ${report.templateRoot}`);
  console.log(`Docs root: ${report.docsRoot}`);
  printItems('PASS', report.pass.map((message) => ({ file: message })));
  printItems('WARN', report.warn.map((message) => ({ file: message })));
  printItems('FAIL', report.fail.map((message) => ({ file: message })));
  printItems('SAME', report.same);
  printItems('DIFFERENT', report.different);
  printItems('MISSING_DOC', report.missingDoc);
  printItems('EXTRA_DOC', report.extraDoc);
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
