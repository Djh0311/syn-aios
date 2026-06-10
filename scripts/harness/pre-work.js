#!/usr/bin/env node

const path = require('path');
const { buildAggregateReport } = require('./lib/check-runner');

const fallbackCommands = [
  'node scripts/harness/config-check.js --target .',
  'node scripts/harness/capability-scan.js --target .',
  'node scripts/harness/verification-plan.js --target .',
  'node scripts/harness/mcp-doctor.js --target .',
  'node scripts/harness/status-snapshot.js --target .',
  'node scripts/harness/guard-state-files.js --target .'
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
    else if (arg === '--config') args.config = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (!arg.startsWith('--')) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (args.config) args.config = path.resolve(args.config);
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

function printDetails(report) {
  console.log('\nDETAILS');
  console.log(`Command source: ${report.details.commandSource || 'N/A'}`);
  for (const check of report.details.checks) {
    console.log(`\n${check.name} [${check.status}]`);
    for (const line of check.summary.evidence) {
      console.log(`  ${line}`);
    }
  }
}

function printReport(report) {
  console.log(`Harness pre-work gate: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  printDetails(report);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildAggregateReport({
    args,
    sectionName: 'preWork',
    fallbackCommands,
    harnessDir: __dirname,
    currentScript: 'pre-work.js'
  });
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
