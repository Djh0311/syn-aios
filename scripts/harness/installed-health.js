#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { readManifest, sha256File, manifestRelativePath } = require('./lib/manifest');
const { detectProjectKind } = require('./lib/project-kind');

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

const runtimeTrees = [
  'docs/evidence',
  'docs/plans'
];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
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

function add(report, kind, message) {
  report[kind].push(message);
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
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
  }
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function normalizeRelative(relativePath) {
  return relativePath.split(path.sep).join('/');
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function loadConfig(targetRoot) {
  const candidates = [
    path.join(targetRoot, 'harness.config.json'),
    path.join(targetRoot, 'harness.config.example.json')
  ];

  for (const candidate of candidates) {
    if (!fs.existsSync(candidate)) continue;
    const parsed = readJson(candidate);
    return {
      path: candidate,
      data: parsed.data,
      error: parsed.error
    };
  }

  return { path: null, data: null, error: null };
}

function hardForInstalled(args, kind) {
  return args.strict && !kind.isSourcePackage;
}

function managedFileItem(targetRoot, relativePath, entry) {
  const target = path.join(targetRoot, relativePath);
  const item = {
    file: normalizeRelative(relativePath),
    target,
    status: 'missing',
    installedSha256: entry.installedSha256 || null,
    currentSha256: null,
    source: entry.source || null,
    sourceSha256: entry.sourceSha256 || null
  };

  if (!fs.existsSync(target)) return item;

  item.currentSha256 = sha256File(target);
  item.status = item.currentSha256 === item.installedSha256 ? 'unchanged' : 'local-modified';
  return item;
}

function checkManifest(args, kind, report) {
  const manifest = readManifest(args.target);
  report.details.manifest = {
    path: manifest.path,
    relativePath: manifestRelativePath,
    error: manifest.error,
    schemaVersion: manifest.data ? manifest.data.schemaVersion : null,
    generatedAt: manifest.data ? manifest.data.generatedAt : null,
    sourceRoot: manifest.data ? manifest.data.sourceRoot : null,
    fileCount: 0
  };

  if (manifest.error) {
    add(report, 'fail', `Manifest could not be read: ${manifest.error}`);
    return null;
  }

  if (!manifest.data) {
    add(report, hardForInstalled(args, kind) ? 'fail' : 'warn', `${manifestRelativePath} was not found`);
    return null;
  }

  add(report, 'pass', `Manifest readable: ${manifestRelativePath}`);
  if (manifest.data.schemaVersion === 1) add(report, 'pass', 'Manifest schemaVersion is 1');
  else add(report, hardForInstalled(args, kind) ? 'fail' : 'warn', 'Manifest schemaVersion is missing or unsupported');

  if (!manifest.data.files || typeof manifest.data.files !== 'object' || Array.isArray(manifest.data.files)) {
    add(report, 'fail', 'Manifest files object is missing or invalid');
    return manifest.data;
  }

  report.details.manifest.fileCount = Object.keys(manifest.data.files).length;
  return manifest.data;
}

function checkManagedFiles(args, kind, manifest, report) {
  const files = manifest && manifest.files && typeof manifest.files === 'object' ? manifest.files : {};
  const items = Object.keys(files).sort().map((relativePath) => managedFileItem(args.target, relativePath, files[relativePath]));
  const unchanged = items.filter((item) => item.status === 'unchanged');
  const localModified = items.filter((item) => item.status === 'local-modified');
  const missing = items.filter((item) => item.status === 'missing');

  report.details.managedFiles = {
    counts: {
      total: items.length,
      unchanged: unchanged.length,
      localModified: localModified.length,
      missing: missing.length
    },
    unchanged,
    localModified,
    missing
  };

  if (items.length === 0) {
    add(report, hardForInstalled(args, kind) ? 'fail' : 'warn', 'Manifest contains no managed files');
    return;
  }

  if (unchanged.length > 0) add(report, 'pass', `Managed files unchanged: ${unchanged.length}`);
  if (localModified.length > 0) add(report, 'warn', `Managed files locally modified: ${localModified.length}`);
  if (missing.length > 0) add(report, 'fail', `Managed files missing: ${missing.length}`);
  if (localModified.length === 0 && missing.length === 0) add(report, 'pass', 'No managed file local conflicts detected');
}

function checkRuntimeDocs(args, kind, report) {
  const core = coreRuntimeDocs.map((file) => ({
    file,
    exists: exists(args.target, file)
  }));
  const trees = runtimeTrees.map((relativePath) => {
    const stat = statSafe(path.join(args.target, relativePath));
    return {
      path: `${relativePath}/**`,
      exists: Boolean(stat && stat.isDirectory())
    };
  });
  const missingCore = core.filter((item) => !item.exists);
  const missingTrees = trees.filter((item) => !item.exists);

  report.details.runtimeDocs = {
    core,
    trees
  };

  if (kind.isSourcePackage) {
    add(report, 'warn', 'Target looks like a source package; installed-project runtime docs checks are informational');
  }

  if (missingCore.length === 0) add(report, 'pass', 'All core runtime docs are present');
  else add(report, hardForInstalled(args, kind) ? 'fail' : 'warn', `Core runtime docs missing: ${missingCore.map((item) => item.file).join(', ')}`);

  if (missingTrees.length === 0) add(report, 'pass', 'Runtime docs state trees are present');
  else add(report, hardForInstalled(args, kind) ? 'fail' : 'warn', `Runtime docs state trees missing: ${missingTrees.map((item) => item.path).join(', ')}`);
}

function checkConfig(args, report) {
  const config = loadConfig(args.target);
  report.details.config = {
    path: config.path,
    relativePath: config.path ? rel(args.target, config.path) : null,
    error: config.error,
    hasProject: Boolean(config.data && config.data.project),
    hasRuntimeDocs: Boolean(config.data && config.data.runtimeDocs),
    hasProtectedPaths: Boolean(config.data && Array.isArray(config.data.protectedPaths))
  };

  if (!config.path) {
    add(report, args.strict ? 'fail' : 'warn', 'No harness.config.json or harness.config.example.json found');
    return;
  }

  if (config.error) {
    add(report, 'fail', `Harness config could not be parsed: ${rel(args.target, config.path)} (${config.error})`);
    return;
  }

  add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  if (report.details.config.hasRuntimeDocs) add(report, 'pass', 'Config runtimeDocs object is present');
  else add(report, args.strict ? 'fail' : 'warn', 'Config runtimeDocs object is missing');
  if (report.details.config.hasProtectedPaths) add(report, 'pass', 'Config protectedPaths array is present');
  else add(report, 'warn', 'Config protectedPaths array is missing');
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      manifest: null,
      managedFiles: null,
      runtimeDocs: null,
      config: null
    }
  };

  const targetStat = statSafe(args.target);
  if (!targetStat) {
    add(report, 'fail', `Target does not exist: ${args.target}`);
    return report;
  }
  if (!targetStat.isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind;
  if (kind.isInstalledProject) add(report, 'pass', 'Installed-project signals detected');
  else if (kind.isSourcePackage) add(report, 'warn', 'Source package detected; this command is intended for installed projects');
  else add(report, args.strict ? 'fail' : 'warn', 'Installed-project signals were not detected');

  const manifest = checkManifest(args, kind, report);
  if (manifest) checkManagedFiles(args, kind, manifest, report);
  else {
    report.details.managedFiles = {
      counts: {
        total: 0,
        unchanged: 0,
        localModified: 0,
        missing: 0
      },
      unchanged: [],
      localModified: [],
      missing: []
    };
  }
  checkRuntimeDocs(args, kind, report);
  checkConfig(args, report);

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

function printFileItems(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item.file}`);
}

function printReport(report) {
  console.log(`Installed harness health: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);

  if (report.details.managedFiles) {
    printFileItems('UNCHANGED', report.details.managedFiles.unchanged);
    printFileItems('LOCAL_MODIFIED', report.details.managedFiles.localModified);
    printFileItems('MISSING', report.details.managedFiles.missing);
  }

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
