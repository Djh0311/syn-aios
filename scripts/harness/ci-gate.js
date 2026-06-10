#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const ciFiles = [
  '.github/workflows',
  '.gitlab-ci.yml',
  '.circleci/config.yml',
  'buildkite.yml',
  '.buildkite/pipeline.yml'
];

const verificationWords = ['lint', 'typecheck', 'type-check', 'test', 'build', 'pytest', 'cargo test', 'go test', 'mvn test', 'gradle test'];

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

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function collectCiFiles(targetRoot) {
  const files = [];
  for (const relativePath of ciFiles) {
    const full = path.join(targetRoot, relativePath);
    if (!fs.existsSync(full)) continue;
    const stat = fs.statSync(full);
    if (stat.isDirectory()) files.push(...walk(full).filter((file) => /\.ya?ml$/i.test(file)));
    else files.push(full);
  }
  return files;
}

function scanFile(filePath) {
  const text = fs.readFileSync(filePath, 'utf8');
  const lower = text.toLowerCase();
  return verificationWords.filter((word) => lower.includes(word));
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      ciFiles: [],
      verificationSignals: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const files = collectCiFiles(args.target);
  report.details.ciFiles = files.map((file) => rel(args.target, file));
  if (files.length === 0) {
    add(report, 'warn', 'No CI configuration files detected');
    return report;
  }

  add(report, 'pass', `CI configuration files detected: ${files.length}`);
  for (const file of files) {
    const signals = scanFile(file);
    if (signals.length > 0) {
      report.details.verificationSignals.push({
        file: rel(args.target, file),
        signals
      });
    }
  }

  if (report.details.verificationSignals.length > 0) {
    add(report, 'pass', `CI verification signals found: ${report.details.verificationSignals.length} file(s)`);
  } else {
    add(report, args.strict ? 'fail' : 'warn', 'CI exists but no obvious lint/typecheck/test/build signals were found');
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
  console.log(`Harness CI gate: ${report.target}`);
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
