#!/usr/bin/env node

const path = require('path');
const { buildAggregateReport } = require('./lib/check-runner');

const fallbackCommands = [
  'node scripts/harness/config-check.js --target .',
  'node scripts/harness/capability-scan.js --target .',
  'node scripts/harness/verification-plan.js --target .',
  'node scripts/harness/status-snapshot.js --target .',
  'node scripts/harness/mcp-doctor.js --target .',
  'node scripts/harness/git-gate.js --target .',
  'node scripts/harness/ci-gate.js --target .',
  'node scripts/harness/guard-state-files.js --target .',
  'node scripts/harness/evidence-check.js --target .',
  'node scripts/harness/evidence-freshness.js --target .',
  'node scripts/harness/browser-evidence-check.js --target .',
  'node scripts/harness/mistake-check.js --target .'
];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    config: null,
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

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item}`);
}

function groupedChecks(report) {
  const planning = new Set([
    'config-check',
    'capability-scan',
    'verification-plan',
    'status-snapshot',
    'mcp-doctor',
    'git-gate',
    'ci-gate',
    'guard-state-files',
    'mistake-check'
  ]);
  const evidence = new Set([
    'evidence-check',
    'evidence-freshness',
    'browser-evidence-check'
  ]);
  const groups = {
    planningAndStatus: [],
    verificationEvidence: [],
    other: []
  };

  for (const check of report.details.checks) {
    if (planning.has(check.name)) groups.planningAndStatus.push(check);
    else if (evidence.has(check.name)) groups.verificationEvidence.push(check);
    else groups.other.push(check);
  }

  return groups;
}

function printCheckGroup(title, checks) {
  console.log(`\n${title} (${checks.length})`);
  if (checks.length === 0) {
    console.log('  None');
    return;
  }

  for (const check of checks) {
    const exit = check.exitCode === null ? '' : ` exit=${check.exitCode}`;
    console.log(`  - ${check.name}: ${check.status}${exit}`);
    for (const line of check.summary.evidence.slice(0, 4)) {
      console.log(`      ${line}`);
    }
  }
}

function printReport(report) {
  console.log(`Harness doctor: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  console.log('Project tests: not executed by this command');
  console.log(`Command source: ${report.details.commandSource || 'N/A'}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);

  const groups = groupedChecks(report);
  console.log('\nCHECK GROUPS');
  printCheckGroup('Planning and status checks', groups.planningAndStatus);
  printCheckGroup('Verification evidence checks', groups.verificationEvidence);
  printCheckGroup('Other checks', groups.other);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildAggregateReport({
    args,
    sectionName: 'harnessDoctor',
    fallbackCommands,
    harnessDir: __dirname,
    currentScript: 'harness-doctor.js'
  });
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
