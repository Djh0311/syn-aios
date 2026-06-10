#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    config: null,
    json: false,
    write: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--config') args.config = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--write') args.write = true;
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

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function valueAt(root, parts) {
  let current = root;
  for (const part of parts) {
    if (!isPlainObject(current) || !Object.prototype.hasOwnProperty.call(current, part)) return undefined;
    current = current[part];
  }
  return current;
}

function setAt(root, parts, value) {
  let current = root;
  for (let i = 0; i < parts.length - 1; i += 1) {
    const part = parts[i];
    if (!isPlainObject(current[part])) current[part] = {};
    current = current[part];
  }
  current[parts[parts.length - 1]] = clone(value);
}

function collectMissingKeys(latest, current, prefix, result) {
  if (!isPlainObject(latest)) return;

  for (const [key, latestValue] of Object.entries(latest)) {
    const parts = prefix.concat(key);
    const currentValue = valueAt(current, parts);

    if (currentValue === undefined) {
      result.push({
        path: parts.join('.'),
        value: clone(latestValue)
      });
      continue;
    }

    if (isPlainObject(latestValue) && isPlainObject(currentValue)) {
      collectMissingKeys(latestValue, current, parts, result);
    }
  }
}

function collectUnknownTopLevel(latest, current) {
  if (!isPlainObject(latest) || !isPlainObject(current)) return [];
  const expected = Object.keys(latest);
  return Object.keys(current).filter((key) => !expected.includes(key));
}

function mergeMissing(latest, current, missing) {
  const merged = clone(current);
  for (const entry of missing) setAt(merged, entry.path.split('.'), entry.value);

  if (isPlainObject(latest) && Object.prototype.hasOwnProperty.call(latest, 'schemaVersion')) {
    merged.schemaVersion = Object.prototype.hasOwnProperty.call(current, 'schemaVersion')
      ? current.schemaVersion
      : latest.schemaVersion;
  }

  return merged;
}

function configCandidate(args) {
  if (args.config) return args.config;
  const projectConfig = path.join(args.target, 'harness.config.json');
  if (fs.existsSync(projectConfig)) return projectConfig;
  return path.join(args.target, 'harness.config.example.json');
}

function backupPath(targetRoot) {
  const stamp = new Date().toISOString().replace(/[^0-9A-Za-z]+/g, '-').replace(/-+$/g, '');
  return path.join(targetRoot, '.harness', `harness.config.backup.${stamp}.json`);
}

function buildReport(args) {
  const outputPath = path.join(args.target, 'harness.config.json');
  const latestExamplePath = path.join(args.target, 'harness.config.example.json');
  const sourceConfigPath = configCandidate(args);
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      latestExamplePath: rel(args.target, latestExamplePath),
      sourceConfigPath: rel(args.target, sourceConfigPath),
      outputPath: rel(args.target, outputPath),
      backupPath: null,
      schemaVersion: {
        current: null,
        latest: null
      },
      missingKeys: [],
      unknownTopLevelKeys: [],
      plannedConfig: null,
      willWrite: false
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

  if (!fs.existsSync(latestExamplePath)) {
    add(report, 'fail', 'harness.config.example.json not found in target');
    return report;
  }
  if (!fs.existsSync(sourceConfigPath)) {
    add(report, 'fail', `Source config not found: ${sourceConfigPath}`);
    return report;
  }

  const latest = readJson(latestExamplePath);
  if (latest.error) {
    add(report, 'fail', `Latest harness.config.example.json could not be parsed: ${latest.error.message}`);
    return report;
  }
  if (!isPlainObject(latest.data)) {
    add(report, 'fail', 'Latest harness.config.example.json root must be a JSON object');
    return report;
  }

  const source = readJson(sourceConfigPath);
  if (source.error) {
    add(report, 'fail', `Source config could not be parsed: ${source.error.message}`);
    return report;
  }
  if (!isPlainObject(source.data)) {
    add(report, 'fail', 'Source config root must be a JSON object');
    return report;
  }

  const missing = [];
  collectMissingKeys(latest.data, source.data, [], missing);
  const unknownTopLevel = collectUnknownTopLevel(latest.data, source.data);
  const planned = mergeMissing(latest.data, source.data, missing);
  const existingOutput = fs.existsSync(outputPath);
  const createFromExample = !existingOutput && path.resolve(sourceConfigPath) === path.resolve(latestExamplePath);

  report.details.schemaVersion.current = source.data.schemaVersion === undefined ? null : source.data.schemaVersion;
  report.details.schemaVersion.latest = latest.data.schemaVersion === undefined ? null : latest.data.schemaVersion;
  report.details.missingKeys = missing;
  report.details.unknownTopLevelKeys = unknownTopLevel;
  report.details.plannedConfig = planned;
  report.details.willWrite = args.write && (missing.length > 0 || createFromExample);

  add(report, 'pass', `Latest example parsed: ${rel(args.target, latestExamplePath)}`);
  add(report, 'pass', `Source config parsed: ${rel(args.target, sourceConfigPath)}`);

  if (missing.length === 0) add(report, 'pass', 'No missing keys found');
  else add(report, 'warn', `Missing key(s) planned for merge: ${missing.map((entry) => entry.path).join(', ')}`);

  if (unknownTopLevel.length === 0) add(report, 'pass', 'No unknown top-level keys found');
  else add(report, args.strict ? 'fail' : 'warn', `Unknown top-level key(s) will be preserved: ${unknownTopLevel.join(', ')}`);

  if (report.details.schemaVersion.current === report.details.schemaVersion.latest) {
    add(report, 'pass', `schemaVersion matches latest: ${report.details.schemaVersion.latest}`);
  } else {
    add(report, args.strict ? 'fail' : 'warn', `schemaVersion differs from latest: current=${report.details.schemaVersion.current}, latest=${report.details.schemaVersion.latest}`);
  }

  if (!args.write) {
    add(report, 'warn', 'Dry run only; no config file written');
    return report;
  }

  if (missing.length === 0 && !createFromExample) {
    add(report, 'pass', 'No write needed; harness.config.json is already aligned with latest example keys');
    return report;
  }

  if (existingOutput) {
    report.details.backupPath = rel(args.target, backupPath(args.target));
    fs.mkdirSync(path.dirname(path.join(args.target, report.details.backupPath)), { recursive: true });
    fs.copyFileSync(outputPath, path.join(args.target, report.details.backupPath), fs.constants.COPYFILE_EXCL);
    add(report, 'pass', `Backed up existing harness.config.json to ${report.details.backupPath}`);
  }

  fs.writeFileSync(outputPath, `${JSON.stringify(planned, null, 2)}\n`, 'utf8');
  add(report, 'pass', `Wrote ${rel(args.target, outputPath)}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness config migrate: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  if (report.strict) console.log('Strict: true');
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
