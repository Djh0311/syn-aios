#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const ignoredNames = new Set(['.DS_Store', 'node_modules', '.git', 'dist', 'build', '.next', 'coverage']);

const coreRuntimeDocs = [
  'docs/current-state.md',
  'docs/requirements-matrix.md',
  'docs/task-queue.md',
  'docs/decisions.md',
  'docs/open-questions.md',
  'docs/context-checkpoints.md',
  'docs/sprint-contract.md',
  'docs/agent-mistake-ledger.md',
  'docs/tooling-and-mcp-registry.md'
];

const protectedRuntimePaths = [
  ...coreRuntimeDocs,
  'docs/evidence/**',
  'docs/plans/**'
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

function statSafe(filePath) {
  try {
    return fs.statSync(filePath);
  } catch (error) {
    return null;
  }
}

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    return null;
  }
}

function rel(root, filePath) {
  return path.relative(root, filePath) || '.';
}

function loadConfig(args) {
  const candidates = [
    args.config,
    path.join(args.target, 'harness.config.json'),
    path.join(args.target, 'harness.config.example.json')
  ].filter(Boolean);

  for (const candidate of candidates) {
    const full = path.resolve(candidate);
    if (fs.existsSync(full)) return { path: full, data: readJson(full) };
  }
  return { path: null, data: null };
}

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function hasAnyFile(dir) {
  return walk(dir).length > 0;
}

function escapeRegex(value) {
  return String(value).replace(/[|\\{}()[\]^$+?.]/g, '\\$&');
}

function globToRegex(pattern) {
  const normalized = pattern.split(path.sep).join('/');
  const parts = normalized.split('*').map(escapeRegex);
  return new RegExp(`^${parts.join('.*')}$`);
}

function listRootEnvFiles(targetRoot) {
  const entries = fs.readdirSync(targetRoot, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && /^\.env($|\.)/.test(entry.name))
    .map((entry) => entry.name)
    .sort();
}

function listProtectedPathMatches(targetRoot, patterns) {
  const rootFiles = fs.readdirSync(targetRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name);
  const allFiles = walk(targetRoot).map((file) => rel(targetRoot, file).split(path.sep).join('/'));

  return patterns.map((pattern) => {
    if (/^\.env/.test(pattern)) {
      const regex = globToRegex(pattern);
      return {
        pattern,
        matches: rootFiles.filter((file) => regex.test(file)),
        contentRead: false
      };
    }

    const regex = globToRegex(pattern);
    return {
      pattern,
      matches: allFiles.filter((file) => regex.test(file)),
      contentRead: false
    };
  });
}

function detectSourcePackage(targetRoot) {
  return detectProjectKind(targetRoot).isSourcePackage;
}

function checkRuntimeDocs(args, report) {
  const docsPath = path.join(args.target, 'docs');
  const templatesDocsPath = path.join(args.target, 'templates', 'docs');
  const hasDocsDir = Boolean(statSafe(docsPath) && statSafe(docsPath).isDirectory());
  const hasTemplatesDocs = Boolean(statSafe(templatesDocsPath) && statSafe(templatesDocsPath).isDirectory());
  const isSourcePackage = detectSourcePackage(args.target);

  const runtimeDocs = coreRuntimeDocs.map((file) => ({
    file,
    exists: exists(args.target, file)
  }));

  const runtimeTrees = [
    {
      path: 'docs/evidence/**',
      exists: statSafe(path.join(args.target, 'docs', 'evidence')) !== null,
      hasFiles: hasAnyFile(path.join(args.target, 'docs', 'evidence'))
    },
    {
      path: 'docs/plans/**',
      exists: statSafe(path.join(args.target, 'docs', 'plans')) !== null,
      hasFiles: hasAnyFile(path.join(args.target, 'docs', 'plans'))
    }
  ];

  report.details.runtimeDocs = {
    protected: protectedRuntimePaths,
    core: runtimeDocs,
    trees: runtimeTrees,
    hasDocsDir,
    hasTemplatesDocs,
    isSourcePackage
  };

  if (isSourcePackage) {
    if (hasDocsDir) {
      add(report, 'fail', 'Source rule package contains root docs/; source-package state belongs under templates/docs/** or plans/**');
    } else {
      add(report, 'pass', 'Source rule package has no root docs/ runtime directory');
    }
    return;
  }

  const presentCore = runtimeDocs.filter((item) => item.exists);
  const missingCore = runtimeDocs.filter((item) => !item.exists);

  if (presentCore.length > 0) {
    add(report, 'pass', `Installed-project runtime docs present: ${presentCore.length}/${runtimeDocs.length}`);
  }

  if (missingCore.length > 0) {
    add(report, args.strict ? 'fail' : 'warn', `Core runtime docs missing: ${missingCore.map((item) => item.file).join(', ')}`);
  } else {
    add(report, 'pass', 'All core runtime docs are present');
  }

  for (const tree of runtimeTrees) {
    if (tree.exists) add(report, 'pass', `Runtime state tree exists: ${tree.path}`);
    else add(report, args.strict ? 'fail' : 'warn', `Runtime state tree missing: ${tree.path}`);
  }

  if (!hasDocsDir && !hasTemplatesDocs) {
    add(report, 'warn', 'No docs/ runtime state directory or templates/docs/ source directory detected');
  }

  if (hasDocsDir && hasTemplatesDocs) {
    add(report, 'pass', 'Both templates/docs/** and docs/** exist; docs/** is protected runtime state and must not be overwritten by templates');
  }
}

function checkProtectedPaths(args, config, report) {
  const protectedPaths = config.data && Array.isArray(config.data.protectedPaths)
    ? config.data.protectedPaths
    : null;

  report.details.protectedPaths = {
    configured: protectedPaths,
    matches: []
  };

  if (!protectedPaths) {
    add(report, args.strict ? 'fail' : 'warn', 'config.protectedPaths is missing or is not an array');
    return;
  }

  add(report, 'pass', `config.protectedPaths configured: ${protectedPaths.length} entries`);

  const matches = listProtectedPathMatches(args.target, protectedPaths);
  report.details.protectedPaths.matches = matches;

  for (const item of matches) {
    if (item.matches.length === 0) continue;
    if (/^\.env/.test(item.pattern)) {
      add(report, 'warn', `.env* protected path exists; content was not read: ${item.matches.join(', ')}`);
    } else {
      add(report, 'pass', `Protected path exists: ${item.pattern} -> ${item.matches.join(', ')}`);
    }
  }
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

  const targetStat = statSafe(args.target);
  if (!targetStat || !targetStat.isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args);
  report.details.configPath = config.path;

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config exists but could not be parsed: ${config.path}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  checkRuntimeDocs(args, report);
  checkProtectedPaths(args, config, report);

  const envFiles = listRootEnvFiles(args.target);
  report.details.envFiles = envFiles;
  if (envFiles.length > 0) add(report, 'warn', `.env* files present at target root; content was not read: ${envFiles.join(', ')}`);

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
  console.log(`Harness state-file guard: ${report.target}`);
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
