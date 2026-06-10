#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const ignoredNames = new Set(['.DS_Store', '.git', 'node_modules', 'dist', 'build', '.next', 'coverage']);

const harnessRuleFiles = [
  'AGENTS.md',
  'codex-multi-agent-safe-collaboration.md',
  'skills/using-superpowers/SKILL.md'
];

const runtimeDocFiles = [
  'docs/current-state.md',
  'docs/requirements-matrix.md',
  'docs/task-queue.md',
  'docs/decisions.md',
  'docs/open-questions.md',
  'docs/context-checkpoints.md',
  'docs/sprint-contract.md',
  'docs/agent-mistake-ledger.md',
  'docs/tooling-and-mcp-registry.md',
  'docs/evidence/README.md',
  'docs/plans/README.md'
];

const questionScanFiles = [
  'docs/open-questions.md',
  'docs/current-state.md',
  'docs/task-queue.md',
  'docs/sprint-contract.md'
];

const defaultRecommendedChecks = [
  'node scripts/harness/capability-scan.js --target .',
  'node scripts/harness/mcp-doctor.js --target .',
  'node scripts/harness/status-snapshot.js --target .'
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

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function rel(root, filePath) {
  return path.relative(root, filePath) || '.';
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error };
  }
}

function loadConfig(args) {
  const candidates = args.config
    ? [args.config]
    : [
        path.join(args.target, 'harness.config.json'),
        path.join(args.target, 'harness.config.example.json')
      ];

  for (const candidate of candidates) {
    if (!candidate) continue;
    const full = path.resolve(candidate);
    if (fs.existsSync(full)) {
      const parsed = readJson(full);
      return {
        path: full,
        data: parsed.data,
        error: parsed.error ? parsed.error.message : null
      };
    }
  }

  return {
    path: args.config || null,
    data: null,
    error: args.config ? 'Config file was not found' : null
  };
}

function gitStatus(targetRoot) {
  const result = spawnSync('git', ['-C', targetRoot, 'status', '--short'], {
    encoding: 'utf8',
    timeout: 5000
  });

  if (result.status !== 0) {
    return {
      isRepo: false,
      dirtyCount: null,
      dirtySummary: [],
      error: (result.stderr || result.stdout || 'git status failed').trim()
    };
  }

  const lines = result.stdout.split('\n').map((line) => line.trimEnd()).filter(Boolean);
  return {
    isRepo: true,
    dirtyCount: lines.length,
    dirtySummary: lines.slice(0, 25),
    error: null
  };
}

function checkFiles(targetRoot, files) {
  return files.map((file) => ({
    file,
    exists: exists(targetRoot, file)
  }));
}

function walkLimited(dir, files = []) {
  if (!fs.existsSync(dir)) return files;

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walkLimited(full, files);
    else files.push(full);
  }

  return files;
}

function globToRegex(pattern) {
  const escaped = pattern.replace(/[.+^${}()|[\]\\]/g, '\\$&');
  const regexText = escaped
    .replace(/\\\*\\\*/g, '.*')
    .replace(/\\\*/g, '[^/]*');
  return new RegExp(`^${regexText}$`);
}

function globBase(pattern) {
  const firstGlob = pattern.search(/[*?]/);
  if (firstGlob === -1) return pattern;
  const slash = pattern.slice(0, firstGlob).lastIndexOf('/');
  return slash === -1 ? '.' : pattern.slice(0, slash);
}

function protectedPathExists(targetRoot, pattern) {
  if (!/[*?]/.test(pattern)) {
    return fs.existsSync(path.join(targetRoot, pattern));
  }

  if (pattern.endsWith('/**')) {
    return fs.existsSync(path.join(targetRoot, pattern.slice(0, -3)));
  }

  const base = globBase(pattern);
  const basePath = path.join(targetRoot, base);
  if (!fs.existsSync(basePath)) return false;

  const regex = globToRegex(pattern);
  return walkLimited(basePath).some((file) => regex.test(rel(targetRoot, file)));
}

function collectProtectedPaths(targetRoot, configData) {
  const configured = Array.isArray(configData && configData.protectedPaths) ? configData.protectedPaths : [];
  const runtimeProtected = configData && configData.runtimeDocs && Array.isArray(configData.runtimeDocs.protected)
    ? configData.runtimeDocs.protected
    : [];
  const patterns = [...new Set([...configured, ...runtimeProtected])];

  return patterns.map((pattern) => ({
    pattern,
    exists: protectedPathExists(targetRoot, pattern)
  }));
}

function scanQuestionLines(targetRoot) {
  const matches = [];
  const pattern = /\b(Open|Blocked|TBD)\b/i;

  for (const file of questionScanFiles) {
    const full = path.join(targetRoot, file);
    if (!fs.existsSync(full)) continue;

    const lines = fs.readFileSync(full, 'utf8').split(/\r?\n/);
    lines.forEach((line, index) => {
      if (!pattern.test(line)) return;
      const text = line.trim();
      if (!text) return;
      matches.push({
        file,
        line: index + 1,
        text: text.slice(0, 240)
      });
    });
  }

  return matches.slice(0, 25);
}

function recommendedChecks(targetRoot, configData) {
  const configured = configData && configData.preWork && Array.isArray(configData.preWork.recommendedChecks)
    ? configData.preWork.recommendedChecks
    : defaultRecommendedChecks;

  return configured.map((command) => {
    const scriptMatch = String(command).match(/\bnode\s+([^\s]+\.js)\b/);
    const script = scriptMatch ? scriptMatch[1] : null;
    return {
      command,
      available: script ? fs.existsSync(path.join(targetRoot, script)) : null
    };
  });
}

function buildReport(args) {
  const report = {
    target: args.target,
    pass: [],
    warn: [],
    fail: [],
    details: {}
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
  const git = gitStatus(args.target);
  const ruleFiles = checkFiles(args.target, harnessRuleFiles);
  const runtimeDocs = checkFiles(args.target, runtimeDocFiles);
  const questionLines = scanQuestionLines(args.target);
  const protectedPaths = collectProtectedPaths(args.target, config.data);
  const nextChecks = recommendedChecks(args.target, config.data);

  report.details = {
    configPath: config.path,
    git,
    harnessRuleFiles: ruleFiles,
    runtimeDocs,
    openQuestionsAndBlockers: questionLines,
    protectedPaths,
    recommendedNextChecks: nextChecks
  };

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  if (git.isRepo) {
    add(report, 'pass', 'Git repository detected');
    if (git.dirtyCount > 0) add(report, 'warn', `Git working tree has ${git.dirtyCount} changed path(s)`);
    else add(report, 'pass', 'Git working tree is clean');
  } else {
    add(report, 'warn', `Not confirmed as a git repository: ${git.error || 'git status failed'}`);
  }

  for (const item of ruleFiles) {
    if (item.exists) add(report, 'pass', `Harness rule file found: ${item.file}`);
    else add(report, 'warn', `Harness rule file missing: ${item.file}`);
  }

  const presentRuntimeDocs = runtimeDocs.filter((item) => item.exists);
  if (presentRuntimeDocs.length > 0) add(report, 'pass', `Runtime docs present: ${presentRuntimeDocs.length}/${runtimeDocs.length}`);
  else add(report, 'warn', 'No installed-project runtime docs detected');

  if (questionLines.length > 0) add(report, 'warn', `Open question/blocker/TBD lines detected: ${questionLines.length}`);
  else add(report, 'pass', 'No Open/Blocked/TBD lines detected in runtime docs scan');

  if (protectedPaths.length > 0) {
    const presentProtected = protectedPaths.filter((item) => item.exists);
    add(report, 'pass', `Protected path patterns checked: ${protectedPaths.length}`);
    if (presentProtected.length > 0) add(report, 'warn', `Protected paths exist; keep read/write scope explicit: ${presentProtected.map((item) => item.pattern).join(', ')}`);
  } else {
    add(report, 'warn', 'No protected paths configured');
  }

  const unavailableChecks = nextChecks.filter((item) => item.available === false);
  if (nextChecks.length > 0) add(report, 'pass', `Recommended next checks listed: ${nextChecks.length}`);
  else add(report, 'warn', 'No recommended next checks configured');
  for (const item of unavailableChecks) {
    add(report, 'warn', `Recommended check script is not present: ${item.command}`);
  }

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
  console.log(`Harness status snapshot: ${report.target}`);
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
