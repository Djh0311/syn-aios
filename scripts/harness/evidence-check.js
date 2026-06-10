#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const ignoredNames = new Set(['.DS_Store', '.git', 'node_modules', 'dist', 'build', '.next', 'coverage']);

const summarySectionChecks = [
  { key: 'claims', label: 'Claims', pattern: /^##\s+.*Claims?/im },
  { key: 'commands', label: 'Commands', pattern: /^##\s+.*Commands?/im },
  { key: 'browser', label: 'Browser', pattern: /^##\s+.*Browser/im },
  { key: 'failures', label: 'Failures', pattern: /^##\s+.*(Failures?|Gaps?)/im },
  { key: 'links', label: 'Links', pattern: /^##\s+.*Links?/im }
];

const evidenceFileNames = new Set([
  'summary.md',
  'test-output.md',
  'browser-check.md',
  'console-network.md'
]);

const gateEvidence = {
  'fresh-verification-for-completion': {
    label: 'fresh verification evidence',
    files: ['summary.md', 'test-output.md']
  },
  'browser-evidence-for-ui-completion': {
    label: 'browser/UI evidence',
    files: ['browser-check.md', 'console-network.md']
  }
};

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
    return { data: null, error };
  }
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
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
    if (!fs.existsSync(full)) {
      if (args.config) {
        return {
          path: full,
          data: null,
          error: 'Config file was not found'
        };
      }
      continue;
    }

    const parsed = readJson(full);
    return {
      path: full,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null
    };
  }

  return {
    path: null,
    data: null,
    error: null
  };
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

function detectSourcePackage(targetRoot) {
  return detectProjectKind(targetRoot).isSourcePackage;
}

function collectEvidence(targetRoot) {
  const evidenceRoot = path.join(targetRoot, 'docs', 'evidence');
  const evidenceStat = statSafe(evidenceRoot);
  const files = evidenceStat && evidenceStat.isDirectory() ? walk(evidenceRoot) : [];

  const summaries = files.filter((file) => path.basename(file) === 'summary.md');
  const knownEvidenceFiles = files.filter((file) => evidenceFileNames.has(path.basename(file)));

  return {
    root: evidenceRoot,
    exists: Boolean(evidenceStat && evidenceStat.isDirectory()),
    readmeExists: exists(targetRoot, 'docs/evidence/README.md'),
    files,
    summaries,
    knownEvidenceFiles
  };
}

function scanSummary(targetRoot, summaryFile) {
  const text = fs.readFileSync(summaryFile, 'utf8');
  const sections = {};
  const missing = [];

  for (const check of summarySectionChecks) {
    const present = check.pattern.test(text);
    sections[check.key] = present;
    if (!present) missing.push(check.label);
  }

  return {
    file: rel(targetRoot, summaryFile),
    sections,
    missing
  };
}

function configuredHardGates(configData) {
  const hard = configData && configData.gates && Array.isArray(configData.gates.hard)
    ? configData.gates.hard
    : [];
  return hard.filter((gate) => Object.prototype.hasOwnProperty.call(gateEvidence, gate));
}

function hasEvidenceFile(evidence, fileNames) {
  return evidence.knownEvidenceFiles.some((file) => fileNames.includes(path.basename(file)));
}

function checkSourcePackage(args, report) {
  const templateReadme = 'templates/docs/evidence/README.md';
  const templateExists = exists(args.target, templateReadme);

  report.details.sourcePackage = {
    templateEvidenceReadme: {
      file: templateReadme,
      exists: templateExists
    }
  };

  if (templateExists) add(report, 'pass', `Source package evidence template found: ${templateReadme}`);
  else add(report, 'fail', `Source package evidence template missing: ${templateReadme}`);

  if (!exists(args.target, 'docs/evidence')) {
    add(report, 'pass', 'Source package has no root docs/evidence runtime archive');
  } else {
    add(report, 'warn', 'Source package contains root docs/evidence; source packages normally keep evidence templates under templates/docs/evidence');
  }
}

function checkInstalledProject(args, report, evidence) {
  report.details.evidence = {
    root: rel(args.target, evidence.root),
    exists: evidence.exists,
    readmeExists: evidence.readmeExists,
    fileCount: evidence.files.length,
    files: evidence.knownEvidenceFiles.map((file) => rel(args.target, file)),
    summaries: []
  };

  if (evidence.readmeExists) {
    add(report, 'pass', 'Evidence archive README found: docs/evidence/README.md');
  } else {
    add(report, args.strict ? 'fail' : 'warn', 'Evidence archive README missing: docs/evidence/README.md');
  }

  if (!evidence.exists) {
    add(report, 'warn', 'No docs/evidence archive directory detected');
    return;
  }

  add(report, 'pass', `Evidence archive directory found with ${evidence.files.length} file(s)`);

  if (evidence.summaries.length === 0) {
    add(report, 'warn', 'Evidence archive exists but no docs/evidence/**/summary.md files were found');
  } else {
    add(report, 'pass', `Evidence summary files found: ${evidence.summaries.length}`);
  }

  for (const summary of evidence.summaries) {
    const scanned = scanSummary(args.target, summary);
    report.details.evidence.summaries.push(scanned);

    if (scanned.missing.length > 0) {
      add(report, 'warn', `${scanned.file} is missing key section(s): ${scanned.missing.join(', ')}`);
    } else {
      add(report, 'pass', `Evidence summary sections complete: ${scanned.file}`);
    }
  }
}

function checkGates(args, report, config, evidence) {
  const gates = configuredHardGates(config.data);
  report.details.gates = {
    hardEvidenceGates: gates
  };

  if (gates.length === 0) {
    add(report, 'pass', 'No hard evidence gates configured');
    return;
  }

  for (const gate of gates) {
    const expectation = gateEvidence[gate];
    const hasEvidence = hasEvidenceFile(evidence, expectation.files);
    const message = `Hard gate "${gate}" requires ${expectation.label}; this check does not prove evidence is fresh`;

    if (hasEvidence) {
      add(report, 'pass', `${message}. Matching archive file is present.`);
    } else {
      add(report, args.strict ? 'fail' : 'warn', `${message}. No matching archive file found.`);
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
  report.details.configPath = config.path ? rel(args.target, config.path) : null;

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  const sourcePackage = detectSourcePackage(args.target);
  const evidence = collectEvidence(args.target);
  report.details.isSourcePackage = sourcePackage;

  if (sourcePackage) checkSourcePackage(args, report);
  else {
    checkInstalledProject(args, report, evidence);
    checkGates(args, report, config, evidence);
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
  console.log(`Harness evidence check: ${report.target}`);
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
