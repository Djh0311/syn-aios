#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detectProjectKind } = require('./lib/project-kind');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    taskId: null,
    slug: null,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--task-id') args.taskId = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (args.taskId) args.taskId = String(args.taskId).trim();
  if (args.slug) args.slug = String(args.slug).trim();
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function run(scriptName, args, extraArgs) {
  const installed = path.join(args.target, 'scripts', 'harness', scriptName);
  const script = fs.existsSync(installed) ? installed : path.join(__dirname, scriptName);
  const result = spawnSync(process.execPath, [script, ...extraArgs], {
    cwd: args.target,
    encoding: 'utf8',
    timeout: 60000,
    maxBuffer: 1024 * 1024 * 10
  });
  return {
    script: scriptName,
    exitCode: typeof result.status === 'number' ? result.status : 1,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    error: result.error ? result.error.message : null
  };
}

function parseJsonOutput(result) {
  try {
    return JSON.parse(result.stdout || '{}');
  } catch (error) {
    return null;
  }
}

function markerRegex(name) {
  return new RegExp(`<!-- harness:${name}:([^:]+):start -->([\\s\\S]*?)<!-- harness:${name}:\\1:end -->`, 'g');
}

function fieldFromBlock(block, label) {
  const pattern = new RegExp(`^[-*]?\\s*${label}:\\s*(.+)$`, 'im');
  const match = block.match(pattern);
  return match ? match[1].trim() : null;
}

function collectBlocks(targetRoot, relativePath, name) {
  const filePath = path.join(targetRoot, relativePath);
  if (!fs.existsSync(filePath)) return [];
  const text = fs.readFileSync(filePath, 'utf8');
  const blocks = [];
  for (const match of text.matchAll(markerRegex(name))) {
    blocks.push({
      id: match[1],
      file: relativePath,
      block: match[2].trim(),
      status: fieldFromBlock(match[2], 'Status'),
      lastUpdated: fieldFromBlock(match[2], 'Last Updated'),
      evidence: fieldFromBlock(match[2], 'Evidence')
    });
  }
  return blocks;
}

function collectTaskDocs(args) {
  const taskBlocks = collectBlocks(args.target, 'docs/task-queue.md', 'task');
  const currentBlocks = collectBlocks(args.target, 'docs/current-state.md', 'current-task');
  const filteredTasks = args.taskId ? taskBlocks.filter((item) => item.id === args.taskId) : taskBlocks;
  const filteredCurrent = args.taskId ? currentBlocks.filter((item) => item.id === args.taskId) : currentBlocks;
  return {
    taskQueue: filteredTasks,
    currentState: filteredCurrent
  };
}

function evidenceFiles(args) {
  const root = path.join(args.target, 'docs', 'evidence');
  const dirs = [];
  if (args.slug) {
    dirs.push(path.join(root, args.slug));
  } else if (fs.existsSync(root)) {
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
      if (entry.isDirectory()) dirs.push(path.join(root, entry.name));
    }
  }

  return dirs
    .filter((dir) => fs.existsSync(dir) && fs.statSync(dir).isDirectory())
    .map((dir) => ({
      slug: path.basename(dir),
      dir: rel(args.target, dir),
      files: fs.readdirSync(dir, { withFileTypes: true })
        .filter((entry) => entry.isFile())
        .map((entry) => rel(args.target, path.join(dir, entry.name)))
        .sort()
    }))
    .sort((a, b) => a.slug.localeCompare(b.slug));
}

function checkReports(args) {
  const slugArgs = args.slug ? ['--slug', args.slug] : [];
  const strictArgs = args.strict ? ['--strict'] : [];
  const checks = [
    ['evidenceFreshness', 'evidence-freshness.js', ['--target', args.target, ...slugArgs, ...strictArgs, '--json']],
    ['browserEvidenceCheck', 'browser-evidence-check.js', ['--target', args.target, ...slugArgs, ...strictArgs, '--json']],
    ['staleControlCheck', 'stale-control-check.js', ['--target', args.target, ...strictArgs, '--json']]
  ];

  const details = {};
  for (const [key, scriptName, scriptArgs] of checks) {
    const result = run(scriptName, args, scriptArgs);
    details[key] = {
      exitCode: result.exitCode,
      error: result.error,
      parsed: parseJsonOutput(result)
    };
  }
  return details;
}

function buildReport(args) {
  const report = {
    target: args.target,
    taskId: args.taskId,
    slug: args.slug,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      taskDocs: null,
      evidence: [],
      checks: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) add(report, 'pass', 'Source package detected; task-status is read-only and runtime docs are templates only');
  else if (kind.isInstalledProject) add(report, 'pass', 'Installed-project target detected');
  else add(report, args.strict ? 'fail' : 'warn', 'Target is not recognized as a harness source package or installed project');

  report.details.taskDocs = collectTaskDocs(args);
  const taskCount = report.details.taskDocs.taskQueue.length + report.details.taskDocs.currentState.length;
  if (taskCount > 0) add(report, 'pass', `Harness-managed task doc entries found: ${taskCount}`);
  else add(report, args.strict ? 'fail' : 'warn', args.taskId ? `No harness-managed task entries found for ${args.taskId}` : 'No harness-managed task entries found');

  report.details.evidence = evidenceFiles(args);
  if (report.details.evidence.length > 0) add(report, 'pass', `Evidence archive(s) found: ${report.details.evidence.map((item) => item.slug).join(', ')}`);
  else add(report, args.strict ? 'fail' : 'warn', args.slug ? `No evidence archive found for slug: ${args.slug}` : 'No evidence archives found');

  report.details.checks = checkReports(args);
  for (const [key, check] of Object.entries(report.details.checks)) {
    if (check.exitCode === 0) add(report, 'pass', `${key} completed`);
    else add(report, args.strict ? 'fail' : 'warn', `${key} exited ${check.exitCode}`);
  }

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness task status: ${report.target}`);
  if (report.taskId) console.log(`Task ID: ${report.taskId}`);
  if (report.slug) console.log(`Slug: ${report.slug}`);
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
