#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const expectedTopLevelKeys = [
  'schemaVersion',
  'project',
  'commands',
  'ecosystemDetection',
  'runtimeDocs',
  'protectedPaths',
  'policy',
  'ui',
  'tools',
  'gates',
  'preWork',
  'preCompletion',
  'autoRisk',
  'completionProtocol',
  'memoryIntegration',
  'verificationRunner',
  'taskLifecycle'
];

const optionalTopLevelKeys = [
  'memoryIntegration'
];

const requiredTopLevelObjects = [
  'project',
  'commands',
  'runtimeDocs',
  'policy',
  'tools',
  'gates',
  'preWork',
  'preCompletion'
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

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error };
  }
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function loadConfig(args) {
  const candidates = args.config
    ? [args.config]
    : [
        path.join(args.target, 'harness.config.json'),
        path.join(args.target, 'harness.config.example.json')
      ];

  for (const candidate of candidates) {
    const full = path.resolve(candidate);
    if (!fs.existsSync(full)) {
      if (args.config) return { path: full, data: null, error: 'Config file was not found' };
      continue;
    }

    const parsed = readJson(full);
    return {
      path: full,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null
    };
  }

  return { path: null, data: null, error: null };
}

function topLevelDetails(data) {
  const actual = isPlainObject(data) ? Object.keys(data) : [];
  const expected = expectedTopLevelKeys.slice();
  const missing = expected.filter((key) => !optionalTopLevelKeys.includes(key) && !Object.prototype.hasOwnProperty.call(data, key));
  const optionalMissing = expected.filter((key) => optionalTopLevelKeys.includes(key) && !Object.prototype.hasOwnProperty.call(data, key));
  const unknown = actual.filter((key) => !expected.includes(key));
  const requiredObjects = requiredTopLevelObjects.map((key) => ({
    key,
    present: isPlainObject(data[key])
  }));

  return { expected, actual, missing, optionalMissing, unknown, requiredObjects };
}

function policyDetails(data) {
  const policy = data.policy;
  const sections = ['git', 'ci', 'evidence', 'ui', 'hooks'];
  return {
    present: isPlainObject(policy),
    mode: isPlainObject(policy) ? policy.mode || null : null,
    sections: sections.map((key) => ({
      key,
      present: isPlainObject(policy && policy[key])
    })),
    disabledChecksPresent: Array.isArray(policy && policy.disabledChecks)
  };
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      configPath: null,
      schemaVersion: null,
      latestKnownSchemaVersion: 1,
      topLevel: null,
      policy: null
    }
  };

  if (!fs.existsSync(args.target)) {
    add(report, 'fail', `Target does not exist: ${args.target}`);
    return report;
  }
  if (!fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args);
  report.details.configPath = config.path ? rel(args.target, config.path) : null;

  if (!config.path) {
    add(report, args.strict ? 'fail' : 'warn', 'No harness.config.json or harness.config.example.json found in target');
    return report;
  }
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.path} (${config.error})`);
    return report;
  }
  if (!isPlainObject(config.data)) {
    add(report, 'fail', `Harness config root must be a JSON object: ${rel(args.target, config.path)}`);
    return report;
  }

  const data = config.data;
  const topLevel = topLevelDetails(data);
  const policy = policyDetails(data);
  report.details.schemaVersion = data.schemaVersion === undefined ? null : data.schemaVersion;
  report.details.topLevel = topLevel;
  report.details.policy = policy;

  add(report, 'pass', `Harness config parsed: ${rel(args.target, config.path)}`);

  if (data.schemaVersion === 1) add(report, 'pass', 'schemaVersion is current: 1');
  else if (data.schemaVersion === undefined) add(report, 'fail', 'schemaVersion is missing');
  else add(report, args.strict ? 'fail' : 'warn', `schemaVersion is not current: ${data.schemaVersion}`);

  const missingRequiredObjects = topLevel.requiredObjects.filter((entry) => !entry.present).map((entry) => entry.key);
  if (missingRequiredObjects.length === 0) add(report, 'pass', 'Required top-level objects are present');
  else add(report, 'fail', `Required top-level object(s) missing or invalid: ${missingRequiredObjects.join(', ')}`);

  if (topLevel.missing.length === 0) add(report, 'pass', 'No expected top-level keys are missing');
  else add(report, args.strict ? 'fail' : 'warn', `Expected top-level key(s) missing: ${topLevel.missing.join(', ')}`);

  if (topLevel.optionalMissing.length === 0) add(report, 'pass', 'No optional top-level keys are missing');
  else add(report, 'warn', `Optional top-level key(s) missing: ${topLevel.optionalMissing.join(', ')}`);

  if (topLevel.unknown.length === 0) add(report, 'pass', 'No unknown top-level keys found');
  else add(report, args.strict ? 'fail' : 'warn', `Unknown top-level key(s): ${topLevel.unknown.join(', ')}`);

  if (policy.present) add(report, 'pass', 'policy object is present');
  else add(report, 'fail', 'policy object is missing or invalid');

  const missingPolicySections = policy.sections.filter((entry) => !entry.present).map((entry) => entry.key);
  if (missingPolicySections.length === 0) add(report, 'pass', 'policy sections are present');
  else add(report, args.strict ? 'fail' : 'warn', `policy section(s) missing or invalid: ${missingPolicySections.join(', ')}`);

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness config schema: ${report.target}`);
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
