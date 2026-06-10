#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detectProjectKind } = require('./lib/project-kind');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    taskId: null,
    requirementId: null,
    keys: 'test',
    updateDocs: false,
    write: false,
    json: false,
    strict: false
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--task-id') args.taskId = argv[++i];
    else if (arg === '--requirement-id') args.requirementId = argv[++i];
    else if (arg === '--keys') args.keys = argv[++i];
    else if (arg === '--update-docs') args.updateDocs = true;
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (!args.slug) throw new Error('--slug is required');
  if (args.updateDocs && !args.write) throw new Error('--update-docs requires --write');
  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function run(scriptName, args, extraArgs) {
  const installed = path.join(args.target, 'scripts', 'harness', scriptName);
  const script = fs.existsSync(installed) ? installed : path.join(__dirname, scriptName);
  const result = spawnSync(process.execPath, [script, ...extraArgs], {
    cwd: args.target,
    encoding: 'utf8',
    timeout: 180000,
    maxBuffer: 1024 * 1024 * 20
  });
  return {
    script: scriptName,
    exitCode: typeof result.status === 'number' ? result.status : 1,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    error: result.error ? result.error.message : null
  };
}

function markdownEscape(value) {
  return String(value || '').replace(/\r?\n/g, ' ').replace(/\|/g, '\\|').trim();
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function marker(name, id, edge) {
  return `<!-- harness:${name}:${id}:${edge} -->`;
}

function boundedBlock(name, id, content) {
  return `${marker(name, id, 'start')}\n${content.trim()}\n${marker(name, id, 'end')}`;
}

function upsertBoundedBlock(text, name, id, content, fallbackHeading) {
  const block = boundedBlock(name, id, content);
  const start = marker(name, id, 'start');
  const end = marker(name, id, 'end');
  const startIndex = text.indexOf(start);
  const endIndex = text.indexOf(end);
  if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
    return `${text.slice(0, startIndex)}${block}${text.slice(endIndex + end.length)}`;
  }
  const trimmed = text.replace(/\s+$/, '');
  return `${trimmed}\n\n${fallbackHeading}\n\n${block}\n`;
}

function writeTextIfChanged(filePath, nextText) {
  const current = fs.existsSync(filePath) ? fs.readFileSync(filePath, 'utf8') : '';
  if (current === nextText) return false;
  fs.writeFileSync(filePath, nextText, 'utf8');
  return true;
}

function parseJsonOutput(result) {
  try {
    return JSON.parse(result.stdout || '{}');
  } catch (error) {
    return null;
  }
}

function hardChecksPassed(report) {
  const verification = report.details.verificationSuite;
  const freshness = report.details.evidenceFreshness;
  const browser = report.details.browserEvidenceCheck;
  const preCompletion = report.details.preCompletion;
  return Boolean(
    verification && verification.exitCode === 0 &&
    freshness && freshness.exitCode === 0 &&
    browser && browser.exitCode === 0 &&
    preCompletion && preCompletion.exitCode === 0 &&
    report.fail.length === 0
  );
}

function evidenceFiles(args) {
  const dir = path.join(args.target, 'docs', 'evidence', args.slug);
  if (!fs.existsSync(dir) || !fs.statSync(dir).isDirectory()) return [];
  return fs.readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => rel(args.target, path.join(dir, entry.name)))
    .sort();
}

function completionStatus(report) {
  if (hardChecksPassed(report)) return 'Done';
  if (report.fail.length > 0) return 'Done With Concerns';
  return 'Review';
}

function validateUpdateDocsTarget(args, report) {
  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (!kind.isInstalledProject || kind.isSourcePackage) {
    add(report, 'fail', '--update-docs is allowed only for installed-project targets');
    return false;
  }
  return true;
}

function updateRuntimeDocs(args, report) {
  if (!validateUpdateDocsTarget(args, report)) {
    return;
  }

  const taskQueuePath = path.join(args.target, 'docs', 'task-queue.md');
  const currentStatePath = path.join(args.target, 'docs', 'current-state.md');
  for (const filePath of [taskQueuePath, currentStatePath]) {
    if (!fs.existsSync(filePath)) {
      add(report, 'fail', `Runtime doc is missing: ${rel(args.target, filePath)}`);
      return;
    }
  }

  const taskId = markdownEscape(args.taskId || args.slug);
  const requirementId = markdownEscape(args.requirementId || 'Unspecified');
  const status = completionStatus(report);
  const updatedAt = new Date().toISOString();
  const evidence = evidenceFiles(args);
  if (evidence.length === 0) evidence.push(`docs/evidence/${args.slug}/`);
  report.details.completionStatus = status;
  report.details.evidenceLinks = evidence;

  const verification = parseJsonOutput(report.details.verificationSuite);
  const verificationNotes = verification && Array.isArray(verification.pass)
    ? verification.pass.join('; ')
    : `verification-suite exit ${report.details.verificationSuite ? report.details.verificationSuite.exitCode : 'unknown'}`;

  const taskBlock = `
## ${taskId}

Status: ${status}

Requirement IDs:

- ${requirementId}

Evidence:

${evidence.map((item) => `- ${item}`).join('\n')}

Verification:

- ${markdownEscape(verificationNotes)}
- evidence-freshness exit: ${report.details.evidenceFreshness ? report.details.evidenceFreshness.exitCode : 'unknown'}
- browser-evidence-check exit: ${report.details.browserEvidenceCheck ? report.details.browserEvidenceCheck.exitCode : 'unknown'}
- pre-completion exit: ${report.details.preCompletion ? report.details.preCompletion.exitCode : 'unknown'}

Last Updated: ${updatedAt}
`;

  const currentBlock = `
### ${taskId}

- Status: ${status} (evidence: ${evidence.join(', ')})
- Requirement IDs: ${requirementId}
- Evidence: ${evidence.join(', ')}
- Last Updated: ${updatedAt}
`;

  const taskQueueText = fs.readFileSync(taskQueuePath, 'utf8');
  const currentStateText = fs.readFileSync(currentStatePath, 'utf8');
  const nextTaskQueue = upsertBoundedBlock(taskQueueText, 'task', taskId, taskBlock, '## Harness Managed Active Tasks');
  const nextCurrentState = upsertBoundedBlock(currentStateText, 'current-task', taskId, currentBlock, '## Harness Managed Current Task');
  const changed = [];
  if (writeTextIfChanged(taskQueuePath, nextTaskQueue)) changed.push('docs/task-queue.md');
  if (writeTextIfChanged(currentStatePath, nextCurrentState)) changed.push('docs/current-state.md');
  if (changed.length > 0) add(report, 'pass', `Updated runtime docs: ${changed.join(', ')}`);
  else add(report, 'pass', 'Runtime docs already contained finished task state');
  if (status !== 'Done') add(report, 'warn', `Task status recorded as ${status}; hard gates did not all pass`);
  report.details.updatedDocs = changed;
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    slug: args.slug,
    taskId: args.taskId,
    requirementId: args.requirementId,
    updateDocs: args.updateDocs,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      updatedDocs: [],
      completionStatus: null,
      evidenceLinks: []
    }
  };
  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  if (args.updateDocs && !validateUpdateDocsTarget(args, report)) return report;

  const suiteArgs = ['--target', args.target, '--slug', args.slug, '--keys', args.keys, '--json'];
  if (args.write) suiteArgs.push('--write');
  if (args.strict) suiteArgs.push('--strict');
  report.details.verificationSuite = run('verification-suite.js', args, suiteArgs);
  if (report.details.verificationSuite.exitCode === 0) add(report, 'pass', 'verification-suite completed');
  else add(report, 'fail', 'verification-suite failed');

  for (const [name, scriptArgs] of [
    ['evidence-freshness.js', ['--target', args.target, '--slug', args.slug, '--strict', '--json']],
    ['browser-evidence-check.js', ['--target', args.target, '--slug', args.slug, '--strict', '--json']],
    ['pre-completion.js', ['--target', args.target, '--strict', '--json']]
  ]) {
    const result = run(name, args, scriptArgs);
    const detailKey = name.replace(/\.js$/, '').replace(/-([a-z])/g, (match, letter) => letter.toUpperCase());
    report.details[detailKey] = result;
    if (result.exitCode === 0) add(report, 'pass', `${name} completed`);
    else add(report, args.strict ? 'fail' : 'warn', `${name} exited ${result.exitCode}`);
  }
  if (args.updateDocs) updateRuntimeDocs(args, report);
  if (!args.write) add(report, 'warn', 'Dry run only; verification-suite did not execute commands');
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness task finish: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  console.log(`Slug: ${report.slug}`);
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
