#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const ledgerRelativePath = path.join('docs', 'agent-mistake-ledger.md');
const installHint = 'Install runtime docs with scripts/harness/install-harness.js or copy templates/docs/agent-mistake-ledger.md to docs/agent-mistake-ledger.md.';
const statusValues = ['Open', 'Encoded In Test', 'Encoded In Skill', 'Encoded In Rule', 'Accepted Risk', 'Obsolete'];

function parseArgs(argv) {
  const args = {
    target: null,
    targetProvided: false,
    title: null,
    kind: null,
    status: null,
    rootCause: null,
    evidence: null,
    prevention: null,
    regressionProtection: null,
    signatureKeywords: null,
    signaturePaths: null,
    riskTags: null,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') {
      args.target = argv[++i];
      args.targetProvided = true;
    } else if (arg === '--title') args.title = argv[++i];
    else if (arg === '--kind') args.kind = argv[++i];
    else if (arg === '--status') args.status = argv[++i];
    else if (arg === '--root-cause') args.rootCause = argv[++i];
    else if (arg === '--evidence') args.evidence = argv[++i];
    else if (arg === '--prevention') args.prevention = argv[++i];
    else if (arg === '--regression-protection') args.regressionProtection = argv[++i];
    else if (arg === '--signature-keywords') args.signatureKeywords = argv[++i];
    else if (arg === '--signature-paths') args.signaturePaths = argv[++i];
    else if (arg === '--risk-tags') args.riskTags = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.write && !args.targetProvided) {
    throw new Error('--write requires an explicit --target installed-project directory');
  }

  args.target = path.resolve(args.target || process.cwd());
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function valueOrTbd(report, args, key, label) {
  const value = args[key];
  if (typeof value === 'string' && value.trim()) return value.trim();
  add(report, 'warn', `${label} omitted; using TBD`);
  return 'TBD';
}

function nextMistakeId(text) {
  let max = 0;
  let inFence = false;

  for (const line of text.split(/\r?\n/)) {
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      continue;
    }

    if (inFence) continue;

    const match = line.match(/^##\s+M-(\d+)\b/i);
    if (!match) continue;

    const number = Number.parseInt(match[1], 10);
    if (Number.isFinite(number) && number > max) max = number;
  }

  return `M-${String(max + 1).padStart(3, '0')}`;
}

function appendSeparator(text) {
  if (!text) return '';
  if (text.endsWith('\n\n')) return '';
  if (text.endsWith('\n')) return '\n';
  return '\n\n';
}

function normalizeStatus(report, status) {
  if (!status || !status.trim()) {
    add(report, 'warn', 'Status omitted; using TBD');
    return 'TBD';
  }

  const trimmed = status.trim();
  const canonical = statusValues.find((value) => value.toLowerCase() === trimmed.toLowerCase());
  if (canonical) return canonical;

  add(report, 'warn', `Status "${trimmed}" is not one of the template values; using as provided`);
  return trimmed;
}

function optionalMetadataLines(args) {
  const lines = [];
  if (typeof args.signatureKeywords === 'string' && args.signatureKeywords.trim()) {
    lines.push(`Signature Keywords: ${args.signatureKeywords.trim()}`);
  }
  if (typeof args.signaturePaths === 'string' && args.signaturePaths.trim()) {
    lines.push(`Signature Paths: ${args.signaturePaths.trim()}`);
  }
  if (typeof args.riskTags === 'string' && args.riskTags.trim()) {
    lines.push(`Risk Tags: ${args.riskTags.trim()}`);
  }
  return lines;
}

function buildEntry(report, args, id) {
  const title = valueOrTbd(report, args, 'title', 'Title');
  const kind = valueOrTbd(report, args, 'kind', 'Kind');
  const rootCause = valueOrTbd(report, args, 'rootCause', 'Root cause');
  const evidence = valueOrTbd(report, args, 'evidence', 'Evidence');
  const prevention = valueOrTbd(report, args, 'prevention', 'Prevention');
  const regressionProtection = valueOrTbd(report, args, 'regressionProtection', 'Regression protection');
  const status = normalizeStatus(report, args.status);
  const date = new Date().toISOString().slice(0, 10);
  const metadataLines = optionalMetadataLines(args);

  const header = [
    `## ${id}: ${title}`,
    '',
    `Date: ${date}`,
    'Task / Requirement: TBD',
    `Affected Area: ${kind}`,
    'Detected By: mistake-new harness',
    ...metadataLines,
    ''
  ];

  return header.concat([
    '### Symptom',
    '',
    '- TBD',
    '',
    '### Wrong Assumption',
    '',
    '- TBD',
    '',
    '### Wrong Action',
    '',
    '- TBD',
    '',
    '### Actual Root Cause',
    '',
    `- ${rootCause}`,
    '',
    '### Detection Evidence',
    '',
    `- ${evidence}`,
    '',
    '### Correct Fix',
    '',
    '- TBD',
    '',
    '### Regression Protection',
    '',
    `- ${regressionProtection}`,
    '',
    '### Prevention',
    '',
    `- ${prevention}`,
    '',
    `Status: ${status}`,
    ''
  ]).join('\n');
}

function buildReport(args) {
  const ledgerPath = path.join(args.target, ledgerRelativePath);
  const report = {
    target: args.target,
    ledger: ledgerPath,
    dryRun: !args.write,
    wrote: false,
    pass: [],
    warn: [],
    fail: [],
    details: {
      nextId: null,
      appendEntry: null
    }
  };

  if (!fs.existsSync(args.target)) {
    add(report, 'fail', `Target directory does not exist: ${args.target}`);
    return report;
  }

  const targetStat = fs.statSync(args.target);
  if (!targetStat.isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${args.target}`);
    return report;
  }

  if (!fs.existsSync(ledgerPath)) {
    add(report, 'fail', `Target ledger does not exist: ${ledgerPath}`);
    add(report, 'fail', installHint);
    return report;
  }

  const stat = fs.statSync(ledgerPath);
  if (!stat.isFile()) {
    add(report, 'fail', `Target ledger is not a file: ${ledgerPath}`);
    return report;
  }

  const text = fs.readFileSync(ledgerPath, 'utf8');
  const id = nextMistakeId(text);
  const entry = buildEntry(report, args, id);
  report.details.nextId = id;
  report.details.appendEntry = entry;

  if (!args.write) {
    add(report, 'pass', 'Dry run only; no files modified');
    return report;
  }

  fs.appendFileSync(ledgerPath, `${appendSeparator(text)}${entry}`, 'utf8');
  report.wrote = true;
  add(report, 'pass', `Appended ${id} to ${ledgerPath}`);
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
  console.log(`Harness mistake new: ${report.target}`);
  console.log(`Ledger: ${report.ledger}`);
  console.log(`Mode: ${report.dryRun ? 'dry-run' : 'write'}`);
  console.log(`Next ID: ${report.details.nextId || 'N/A'}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);

  if (report.details.appendEntry) {
    console.log('\nINTENDED APPEND');
    console.log(report.details.appendEntry);
  }
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
