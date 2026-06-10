#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const providers = {
  github: {
    paths: [
      path.join('.github', 'workflows', 'harness.yml'),
      path.join('templates', 'ci', 'github-actions', 'harness.yml')
    ]
  },
  gitlab: {
    paths: [
      '.gitlab-ci-harness.yml',
      path.join('templates', 'ci', 'gitlab', 'harness.yml')
    ]
  }
};

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    provider: 'all',
    file: null,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--provider') args.provider = argv[++i];
    else if (arg === '--file') args.file = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (!arg.startsWith('--')) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!['all', 'github', 'gitlab'].includes(args.provider)) throw new Error(`Unsupported provider: ${args.provider}`);
  args.target = path.resolve(args.target);
  if (args.file) args.file = path.resolve(args.file);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function candidateFiles(args) {
  if (args.file) return [args.file];
  const selected = args.provider === 'all' ? Object.keys(providers) : [args.provider];
  const files = [];
  for (const provider of selected) {
    for (const relativePath of providers[provider].paths) {
      const full = path.join(args.target, relativePath);
      if (fs.existsSync(full)) files.push(full);
    }
  }
  return [...new Set(files)];
}

function runLines(text) {
  return text
    .split(/\r?\n/)
    .map((line, index) => ({ line: line.trim(), number: index + 1 }))
    .filter((item) => /^run:\s+/.test(item.line) || /^-\s+node\s+/.test(item.line) || /^node\s+/.test(item.line));
}

function commandFromRunLine(line) {
  if (line.startsWith('run:')) return line.replace(/^run:\s+/, '').trim();
  if (line.startsWith('- ')) return line.replace(/^-\s+/, '').trim();
  return line.trim();
}

function validateFile(report, args, filePath) {
  const text = fs.readFileSync(filePath, 'utf8');
  const relative = rel(args.target, filePath);
  const lines = runLines(text);
  const unsafe = [];
  const nonHarness = [];
  const harnessCommands = [];
  const projectCommandChecks = [];

  if (!/node scripts\/harness\/(?:harness-doctor|pre-work|pre-completion)\.js/.test(text)) {
    add(report, args.strict ? 'fail' : 'warn', `${relative}: missing core harness doctor/pre-work/pre-completion signal`);
  } else {
    add(report, 'pass', `${relative}: core harness command signal present`);
  }

  for (const item of lines) {
    const command = commandFromRunLine(item.line);
    if (/node scripts\/harness\/verification-runner\.js/.test(command)) projectCommandChecks.push(item.number);
    if (/node scripts\/harness\/[A-Za-z0-9._-]+\.js/.test(command)) harnessCommands.push(item.number);
    if (!/^node scripts\/harness\/[A-Za-z0-9._-]+\.js(?:\s|$)/.test(command)) nonHarness.push(`${item.number}: ${command}`);
    if (/[;&<>`$()]/.test(command)) unsafe.push(`${item.number}: ${command}`);
  }

  if (harnessCommands.length > 0) add(report, 'pass', `${relative}: harness command lines validated (${harnessCommands.length})`);
  else add(report, args.strict ? 'fail' : 'warn', `${relative}: no direct harness command lines found`);

  if (unsafe.length > 0) add(report, 'fail', `${relative}: unsafe shell syntax in command lines: ${unsafe.join('; ')}`);
  else add(report, 'pass', `${relative}: no unsafe shell syntax in direct command lines`);

  if (nonHarness.length > 0) add(report, 'fail', `${relative}: non-harness direct command lines: ${nonHarness.join('; ')}`);
  else add(report, 'pass', `${relative}: direct command lines are harness-scoped`);

  if (/Add project command checks/.test(text) && projectCommandChecks.length === 0) {
    add(report, 'pass', `${relative}: project command checks are documented but not executed by default`);
  } else if (projectCommandChecks.length > 0) {
    add(report, 'pass', `${relative}: project command checks use verification-runner (${projectCommandChecks.length})`);
  }

  report.details.files.push({
    file: relative,
    commandLineCount: lines.length,
    harnessCommandLines: harnessCommands,
    projectCommandChecks,
    unsafe,
    nonHarness
  });
}

function buildReport(args) {
  const report = {
    target: args.target,
    provider: args.provider,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      files: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const files = candidateFiles(args);
  if (files.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', 'No harness CI files found to validate');
    return report;
  }

  for (const filePath of files) validateFile(report, args, filePath);
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
  console.log(`Harness CI validate: ${report.target}`);
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
