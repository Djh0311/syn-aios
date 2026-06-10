#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { scanSecurityFindings } = require('./lib/security');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    file: null,
    text: null,
    source: null,
    url: null,
    strict: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--file') args.file = argv[++i];
    else if (arg === '--text') args.text = argv[++i];
    else if (arg === '--source') args.source = argv[++i];
    else if (arg === '--url') args.url = argv[++i];
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function readInput(args) {
  if (typeof args.text === 'string') {
    return {
      text: args.text,
      path: null,
      source: args.source || 'direct-text'
    };
  }

  if (args.file) {
    const filePath = path.resolve(args.target, args.file);
    return {
      text: fs.readFileSync(filePath, 'utf8'),
      path: rel(args.target, filePath),
      source: args.source || 'file'
    };
  }

  return {
    text: '',
    path: null,
    source: args.source || 'empty'
  };
}

function buildReport(args) {
  const input = readInput(args);
  const scan = scanSecurityFindings(input.text, {
    source: input.source,
    path: input.path,
    url: args.url
  });

  const report = {
    target: args.target,
    strict: args.strict,
    input: {
      source: input.source,
      file: input.path,
      url: args.url || null
    },
    pass: [],
    warn: [],
    fail: [],
    details: scan
  };

  if (scan.findings.length === 0) report.pass.push('No prompt-injection or secret patterns detected');
  if (scan.redacted) report.warn.push('Secret-like content was redacted');
  if (scan.promptInjectionDetected) report.warn.push('Prompt-injection-like content detected');
  if (args.strict && scan.risk === 'high') report.fail.push('High-risk combination detected');
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
  console.log(`Harness security scan: ${report.target}`);
  console.log(`Input: ${report.input.file || report.input.source}`);
  console.log(`Trust: ${report.details.trust.trust} (${report.details.trust.reason})`);
  console.log(`Risk: ${report.details.risk}`);
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
