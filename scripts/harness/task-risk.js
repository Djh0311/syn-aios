#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { projectPresetRecommendation, taskPathRecommendation } = require('./lib/risk-classifier');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    title: '',
    description: '',
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--title') args.title = argv[++i];
    else if (arg === '--description') args.description = argv[++i];
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

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectPreset: null,
      taskPath: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const projectPreset = projectPresetRecommendation(args.target);
  const taskPath = taskPathRecommendation(args.target, {
    title: args.title,
    description: args.description
  });

  report.details.projectPreset = projectPreset;
  report.details.taskPath = taskPath;

  add(report, 'pass', `Recommended project preset: ${projectPreset.preset}`);
  add(report, 'pass', `Recommended task path: ${taskPath.path}`);
  if (!args.title && !args.description) add(report, 'warn', 'No task title or description supplied; task path is provisional');
  if (args.strict && taskPath.path !== 'strict') add(report, 'warn', `Strict flag requested; recommended task path remains ${taskPath.path}`);

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness task risk: ${report.target}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log('\nRECOMMENDATION');
  console.log(`  Project preset: ${report.details.projectPreset && report.details.projectPreset.preset}`);
  console.log(`  Task path: ${report.details.taskPath && report.details.taskPath.path}`);
  console.log(`  Rationale: ${report.details.taskPath && report.details.taskPath.rationale}`);
  console.log('\nEVIDENCE');
  const projectSignals = report.details.projectPreset && report.details.projectPreset.profile
    ? report.details.projectPreset.profile.signals
    : [];
  for (const signal of projectSignals) console.log(`  - project:${signal.name} weight=${signal.weight} (${signal.evidence.join('; ')})`);
  const taskEvidence = report.details.taskPath ? report.details.taskPath.evidence : [];
  for (const item of taskEvidence) console.log(`  - task:${item.name} -> ${item.path} (${item.evidence})`);
  console.log('\nHANDLING');
  const handling = report.details.taskPath ? report.details.taskPath.handling : [];
  for (const item of handling) console.log(`  - ${item}`);
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
