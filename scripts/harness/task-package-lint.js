#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');
const {
  readTaskPackage,
  renderTaskPackageMarkdown,
  taskPackageDir,
  validateTaskPackage
} = require('./lib/task-package-schema');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function jsonFiles(dir) {
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
    .map((entry) => path.join(dir, entry.name))
    .sort();
}

function buildReport(args) {
  const report = {
    target: args.target,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      checked: 0,
      packages: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  const dir = taskPackageDir(args.target);

  if (!fs.existsSync(dir)) {
    const message = kind.isSourcePackage
      ? 'Skipped source package task-package lint: docs/task-packages is installed-project runtime state'
      : 'No task packages found: docs/task-packages is absent';
    add(report, kind.isSourcePackage ? 'warn' : 'pass', message);
    return report;
  }

  const files = jsonFiles(dir);
  if (files.length === 0) {
    add(report, 'pass', 'No task package JSON files found');
    return report;
  }

  for (const file of files) {
    const entry = {
      file: rel(args.target, file),
      markdownFile: rel(args.target, file.replace(/\.json$/, '.md')),
      valid: false,
      errors: [],
      warnings: []
    };

    try {
      const data = readTaskPackage(file);
      const validation = validateTaskPackage(data, { source: entry.file });
      entry.valid = validation.valid;
      entry.errors = validation.errors;
      entry.warnings = validation.warnings;
      report.details.checked += 1;

      const rendered = renderTaskPackageMarkdown(data);
      const markdownPath = file.replace(/\.json$/, '.md');
      if (!fs.existsSync(markdownPath)) {
        entry.errors.push(`${entry.markdownFile}: rendered Markdown file is missing`);
        entry.valid = false;
      } else {
        const actual = fs.readFileSync(markdownPath, 'utf8');
        if (actual !== rendered) {
          entry.errors.push(`${entry.markdownFile}: rendered Markdown is stale relative to JSON source`);
          entry.valid = false;
        }
      }
    } catch (error) {
      entry.errors.push(`${entry.file}: ${error.message}`);
    }

    for (const warning of entry.warnings) add(report, 'warn', warning);
    if (entry.valid) add(report, 'pass', `${entry.file}: valid task package`);
    else for (const error of entry.errors) add(report, 'fail', error);
    report.details.packages.push(entry);
  }

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness task-package lint: ${report.target}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
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
