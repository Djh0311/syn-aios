#!/usr/bin/env node

const path = require('path');
const { queryMistakes } = require('./lib/mistake-retrieval');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    title: '',
    description: '',
    paths: [],
    riskTags: [],
    limit: 5,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--title') args.title = argv[++i] || '';
    else if (arg === '--description') args.description = argv[++i] || '';
    else if (arg === '--path') args.paths.push(argv[++i] || '');
    else if (arg === '--risk-tag') args.riskTags.push(argv[++i] || '');
    else if (arg === '--limit') args.limit = Number.parseInt(argv[++i], 10);
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (!Number.isFinite(args.limit) || args.limit < 1) args.limit = 5;
  return args;
}

function buildReport(args) {
  const related = queryMistakes(args.target, {
    title: args.title,
    description: args.description,
    paths: args.paths,
    riskTags: args.riskTags
  }, {
    limit: args.limit
  });

  const report = {
    target: args.target,
    ledger: related.ledgerPath,
    pass: [],
    warn: [],
    fail: [],
    details: {
      query: related.query,
      matches: related.matches
    }
  };

  if (related.ledgerError) report.fail.push(`Mistake ledger could not be read: ${related.ledgerError}`);
  else if (!related.ledgerExists) report.warn.push(`Mistake ledger not found: ${related.ledgerPath}`);
  else report.pass.push(`Mistake ledger scanned: ${related.matches.length} related entr${related.matches.length === 1 ? 'y' : 'ies'}`);

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness mistake query: ${report.target}`);
  console.log(`Ledger: ${report.ledger}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log(`\nRELATED MISTAKES (${report.details.matches.length})`);
  if (report.details.matches.length === 0) {
    console.log('  None');
    return;
  }
  for (const entry of report.details.matches) {
    console.log(`  - ${entry.id}: ${entry.title} (score ${entry.score}; ${entry.reasons.join(', ') || 'related'})`);
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
