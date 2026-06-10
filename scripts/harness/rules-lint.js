#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const root = path.resolve(process.argv[2] || process.cwd());

const requiredRootFiles = [
  'README.md',
  'AGENTS.md',
  'codex-multi-agent-safe-collaboration.md',
  'skills/using-superpowers/SKILL.md',
  'templates/docs/current-state.md',
  'templates/docs/requirements-matrix.md',
  'templates/docs/task-queue.md',
  'templates/docs/decisions.md',
  'templates/docs/open-questions.md',
  'templates/docs/context-checkpoints.md',
  'templates/docs/sprint-contract.md',
  'templates/docs/agent-mistake-ledger.md',
  'templates/docs/tooling-and-mcp-registry.md',
  'templates/docs/evidence/README.md',
  'templates/docs/plans/README.md',
  'harness.config.example.json',
  'scripts/harness/rules-lint.js',
  'scripts/harness/install-harness.js',
  'scripts/harness/sync-harness.js',
  'scripts/harness/harness.js',
  'scripts/harness/config-check.js',
  'scripts/harness/config-schema.js',
  'scripts/harness/config-migrate.js',
  'scripts/harness/capability-scan.js',
  'scripts/harness/capability-map.js',
  'scripts/harness/evidence-freshness.js',
  'scripts/harness/browser-evidence-check.js',
  'scripts/harness/git-gate.js',
  'scripts/harness/ci-gate.js',
  'scripts/harness/ci-validate.js',
  'scripts/harness/managed-files-audit.js',
  'scripts/harness/runtime-docs-diff.js',
  'scripts/harness/runtime-docs-init.js',
  'scripts/harness/config-init.js',
  'scripts/harness/verification-suite.js',
  'scripts/harness/task-start.js',
  'scripts/harness/task-finish.js',
  'scripts/harness/task-risk.js',
  'scripts/harness/stale-control-check.js',
  'scripts/harness/harness-doctor.js',
  'scripts/harness/installed-health.js',
  'scripts/harness/hook-install.js',
  'scripts/harness/hook-uninstall.js',
  'scripts/harness/ci-init.js',
  'scripts/harness/project-profile.js',
  'scripts/harness/config-policy.js',
  'scripts/harness/evidence-index.js',
  'scripts/harness/evidence-query.js',
  'scripts/harness/task-status.js',
  'templates/hooks/pre-commit',
  'templates/hooks/pre-push',
  'templates/ci/github-actions/harness.yml',
  'templates/ci/gitlab/harness.yml',
  'scripts/harness/fixture-check.js',
  'scripts/harness/fixtures/README.md',
  'scripts/harness/verification-plan.js',
  'scripts/harness/verification-runner.js',
  'scripts/harness/mcp-doctor.js',
  'scripts/harness/status-snapshot.js',
  'scripts/harness/guard-state-files.js',
  'scripts/harness/evidence-check.js',
  'scripts/harness/evidence-command.js',
  'scripts/harness/evidence-new.js',
  'scripts/harness/ui-verify.js',
  'scripts/harness/mistake-check.js',
  'scripts/harness/mistake-new.js',
  'scripts/harness/pre-work.js',
  'scripts/harness/pre-completion.js',
  'scripts/harness/self-test.js',
  'scripts/harness/lib/check-runner.js',
  'scripts/harness/lib/manifest.js',
  'scripts/harness/lib/project-kind.js',
  'scripts/harness/lib/risk-classifier.js'
];

const ignoredNames = new Set(['.DS_Store']);
const projectSpecificPatterns = [
  /\bkt-erp\b/i,
  /\bkterp\b/i,
  /\badmin-web\b/i,
  /\brider-h5\b/i,
  /localhost:300[0-9]/i,
  /库存|骑手|门店|外卖|采购|仓库/
];

const report = {
  pass: [],
  warn: [],
  fail: []
};

function rel(filePath) {
  return path.relative(root, filePath) || '.';
}

function exists(relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}

function add(kind, message) {
  report[kind].push(message);
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item}`);
}

function readText(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(full, files);
    } else {
      files.push(full);
    }
  }
  return files;
}

function parseFrontmatter(text) {
  if (!text.startsWith('---\n')) return null;
  const end = text.indexOf('\n---', 4);
  if (end === -1) return null;
  const block = text.slice(4, end).trim();
  const result = {};
  for (const line of block.split('\n')) {
    const match = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (match) result[match[1]] = match[2].trim();
  }
  return result;
}

function checkRequiredFiles() {
  for (const file of requiredRootFiles) {
    if (exists(file)) add('pass', `Found required file: ${file}`);
    else add('fail', `Missing required file: ${file}`);
  }
}

function checkNoRuntimeDocsDirectory() {
  const docsPath = path.join(root, 'docs');
  if (!fs.existsSync(docsPath)) {
    add('pass', 'Source package has no root docs/ runtime directory');
    return;
  }

  const files = walk(docsPath);
  if (files.length === 0) {
    add('fail', 'Source package contains root docs/ directory; remove it to avoid runtime-state confusion');
    return;
  }

  add('fail', `Source package contains root docs/ files: ${files.map(rel).join(', ')}`);
}

function checkSkills() {
  const skillsDir = path.join(root, 'skills');
  if (!fs.existsSync(skillsDir)) {
    add('fail', 'Missing skills/ directory');
    return;
  }

  for (const entry of fs.readdirSync(skillsDir, { withFileTypes: true })) {
    if (!entry.isDirectory() || ignoredNames.has(entry.name)) continue;
    const skillFile = path.join(skillsDir, entry.name, 'SKILL.md');
    if (!fs.existsSync(skillFile)) {
      add('fail', `Missing SKILL.md for skill directory: skills/${entry.name}`);
      continue;
    }

    const text = fs.readFileSync(skillFile, 'utf8');
    const fm = parseFrontmatter(text);
    if (!fm) {
      add('fail', `Missing YAML frontmatter: skills/${entry.name}/SKILL.md`);
      continue;
    }
    if (!fm.name) add('fail', `Missing frontmatter name: skills/${entry.name}/SKILL.md`);
    if (!fm.description) add('fail', `Missing frontmatter description: skills/${entry.name}/SKILL.md`);
    if (fm.name && fm.name !== entry.name) {
      add('warn', `Skill name differs from directory: skills/${entry.name}/SKILL.md declares "${fm.name}"`);
    } else {
      add('pass', `Skill frontmatter ok: skills/${entry.name}/SKILL.md`);
    }
  }
}

function checkSkillReferences() {
  if (!exists('AGENTS.md')) return;
  const text = readText('AGENTS.md');
  const matches = [...text.matchAll(/`([a-z0-9-]+)`/g)].map((match) => match[1]);
  const skillRefs = matches.filter((name) => exists(`skills/${name}`) || name.includes('-'));
  const unique = [...new Set(skillRefs)];

  for (const name of unique) {
    if (exists(`skills/${name}`) && !exists(`skills/${name}/SKILL.md`)) {
      add('fail', `AGENTS.md references skill without SKILL.md: ${name}`);
    }
  }
  add('pass', 'Skill reference scan completed');
}

function checkProjectSpecificTerms() {
  const scanFiles = [
    'AGENTS.md',
    'codex-multi-agent-safe-collaboration.md',
    ...walk(path.join(root, 'templates')).map(rel)
  ].filter((file) => exists(file));

  for (const file of scanFiles) {
    const text = readText(file);
    for (const pattern of projectSpecificPatterns) {
      if (pattern.test(text)) {
        add('warn', `Possible project-specific term in ${file}: ${pattern}`);
      }
    }
  }
  add('pass', 'Project-specific term scan completed');
}

console.log(`Harness rules lint: ${root}`);

if (!detectProjectKind(root).isSourcePackage) {
  add('pass', 'Skipped source-package lint because this target is not a standard rule source package');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  process.exit(0);
}

checkRequiredFiles();
checkNoRuntimeDocsDirectory();
checkSkills();
checkSkillReferences();
checkProjectSpecificTerms();

printSection('PASS', report.pass);
printSection('WARN', report.warn);
printSection('FAIL', report.fail);

if (report.fail.length > 0) process.exit(1);
