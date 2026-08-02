#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const { inspectConfig, validateAgentEntrypoints } = require('./config-check');
const { loadCoreContext } = require('./lib/context-contract');
const { readManifest, sha256Buffer } = require('./lib/manifest');
const { sanitizeOutputText } = require('./lib/output-safety');

const START_MARKER = '# >>> ADAPTIVE HARNESS V2 PRE-COMMIT >>>';
const END_MARKER = '# <<< ADAPTIVE HARNESS V2 PRE-COMMIT <<<';
const OBSERVATION_COMMAND = /scripts\/harness-v2\/harness-observation\.js/i;
const AUTOMATIC_OBSERVATION_CONSUMERS = new Set([
  'AGENTS.md',
  'CLAUDE.md',
  '.claude/settings.json',
  '.claude/hooks/session-start.sh',
  'templates/hooks/pre-commit',
]);
const AUTOMATIC_COMMAND_PATTERNS = [
  {
    label: 'task-package write-capable command',
    expression: /(?:^|[\s;&|("'`])(?:node(?:\.exe)?\s+)?(?:\.\/)?scripts\/harness-v2\/task-package\.js\s+(?:create|activate|accept|complete)\b/im,
  },
  {
    label: 'git-task write-capable command',
    expression: /(?:^|[\s;&|("'`])(?:node(?:\.exe)?\s+)?(?:\.\/)?scripts\/harness-v2\/git-task\.js\s+(?:begin|commit|integrate|retire)\b/im,
  },
];
const FIXED_CI_CONSUMERS = [
  '.circleci/config.yml',
  '.circleci/config.yaml',
  '.gitlab-ci.yml',
  '.gitlab-ci.yaml',
  '.buildkite/pipeline.yml',
  '.buildkite/pipeline.yaml',
  'azure-pipelines.yml',
  'azure-pipelines.yaml',
  'bitbucket-pipelines.yml',
  'bitbucket-pipelines.yaml',
  'Jenkinsfile',
];
const CI_WORKFLOW_MAX_FILES = 64;
const CI_CONSUMER_MAX_BYTES = 512 * 1024;
const TEXT_DEPENDENCY_PATTERNS = [
  { label: 'old Harness runtime invocation', expression: /\bnode\s+scripts\/harness\/(?!harness-v2)/i },
  { label: 'pre-work lifecycle', expression: /scripts\/harness\/pre-work\.js/i },
  { label: 'pre-completion lifecycle', expression: /scripts\/harness\/pre-completion\.js/i },
  { label: 'task lifecycle runtime', expression: /scripts\/harness\/task-(?:start|finish|status|risk)\.js/i },
  { label: 'runtime ledger', expression: /scripts\/harness\/runtime-ledger\.js/i },
  {
    label: 'Superpower router',
    expression: new RegExp(`skills/using-${['super', 'powers'].join('')}|using-${['super', 'powers'].join('')}`, 'i'),
  },
  { label: 'Claude Stop completion hook', expression: /\.claude\/hooks\/stop-precompletion\.sh/i },
  { label: 'legacy forced path modes', expression: /^#{1,6}\s+(?:Fast|Standard|Strict)\s+Path\b/im },
  {
    label: 'donor Harness checkout',
    expression: /\/harness engineering\/(?:harness-engineering|game-harness)\//i,
  },
];

function usage() {
  return [
    'Usage: node scripts/harness-v2/active-path-audit.js --target <project> [--strict] [--json]',
    '',
    'Read-only audit of schema 2 manifest files, active adapters, config, and Git hooks.',
    'Historical files outside active consumers are not scanned.',
  ].join('\n');
}

function parseArgs(argv) {
  const options = { target: null, strict: false, json: false, help: false };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--target') {
      if (index + 1 >= argv.length || argv[index + 1].startsWith('--')) {
        throw new Error('--target requires a value');
      }
      options.target = argv[index + 1];
      index += 1;
    } else if (argument === '--strict') {
      options.strict = true;
    } else if (argument === '--json') {
      options.json = true;
    } else if (argument === '--help' || argument === '-h') {
      options.help = true;
    } else {
      throw new Error(`Unsupported argument: ${argument}`);
    }
  }
  if (!options.help && !options.target) throw new Error('--target is required');
  return options;
}

function lstatIfPresent(filePath) {
  try {
    return fs.lstatSync(filePath);
  } catch (error) {
    if (error && error.code === 'ENOENT') return null;
    throw error;
  }
}

function inspectRelativeFile(targetRoot, relativePath) {
  let cursor = targetRoot;
  for (const segment of relativePath.split('/')) {
    cursor = path.join(cursor, segment);
    const stat = lstatIfPresent(cursor);
    if (!stat) continue;
    if (stat.isSymbolicLink()) {
      return { ok: false, error: `${relativePath} traverses a symlink` };
    }
  }
  const absolute = path.join(targetRoot, relativePath);
  const stat = lstatIfPresent(absolute);
  if (!stat) return { ok: false, error: `${relativePath} is missing` };
  if (!stat.isFile()) {
    return { ok: false, error: `${relativePath} is not a regular file` };
  }
  return {
    ok: true,
    absolute,
    content: fs.readFileSync(absolute),
    mode: stat.mode & 0o777,
  };
}

function scanText(relativePath, content, errors) {
  if (content.includes(0)) return;
  const text = content.toString('utf8');
  for (const pattern of TEXT_DEPENDENCY_PATTERNS) {
    if (pattern.expression.test(text)) {
      errors.push(`${relativePath} contains active ${pattern.label}`);
    }
  }
}

function scanAutomaticCommands(relativePath, content, errors) {
  if (content.includes(0)) return;
  const text = content
    .toString('utf8')
    .split(/\r?\n/)
    .filter((line) => !/^\s*#/.test(line))
    .join('\n')
    .replace(/\\\r?\n/g, ' ');
  for (const pattern of AUTOMATIC_COMMAND_PATTERNS) {
    if (pattern.expression.test(text)) {
      errors.push(`${relativePath} contains automatic ${pattern.label}`);
    }
  }
}

function inspectAutomaticConsumer(targetRoot, relativePath, errors) {
  const candidateStat = lstatIfPresent(path.join(targetRoot, relativePath));
  if (
    candidateStat &&
    candidateStat.isFile() &&
    candidateStat.size > CI_CONSUMER_MAX_BYTES
  ) {
    errors.push(`${relativePath} exceeds the bounded CI consumer audit size`);
    return;
  }
  const state = inspectRelativeFile(targetRoot, relativePath);
  if (!state.ok) {
    errors.push(state.error);
    return;
  }
  scanText(relativePath, state.content, errors);
  scanAutomaticCommands(relativePath, state.content, errors);
}

function inspectCiConsumers(targetRoot, errors) {
  for (const relativePath of FIXED_CI_CONSUMERS) {
    if (!lstatIfPresent(path.join(targetRoot, relativePath))) continue;
    inspectAutomaticConsumer(targetRoot, relativePath, errors);
  }

  const githubDirectory = path.join(targetRoot, '.github');
  const githubStat = lstatIfPresent(githubDirectory);
  if (githubStat && (githubStat.isSymbolicLink() || !githubStat.isDirectory())) {
    errors.push('.github must be a regular non-symlink directory');
    return;
  }
  const workflowDirectory = path.join(githubDirectory, 'workflows');
  const directoryStat = lstatIfPresent(workflowDirectory);
  if (!directoryStat) return;
  if (directoryStat.isSymbolicLink() || !directoryStat.isDirectory()) {
    errors.push('.github/workflows must be a regular non-symlink directory');
    return;
  }
  const names = fs.readdirSync(workflowDirectory)
    .filter((name) => /\.ya?ml$/i.test(name))
    .sort();
  if (names.length > CI_WORKFLOW_MAX_FILES) {
    errors.push(
      `.github/workflows exceeds the ${CI_WORKFLOW_MAX_FILES}-file audit ceiling`,
    );
    return;
  }
  for (const name of names) {
    inspectAutomaticConsumer(
      targetRoot,
      path.posix.join('.github/workflows', name),
      errors,
    );
  }
}

function inspectRootEntrypoints(targetRoot, errors) {
  for (const relativePath of ['README.md', 'package.json', 'harness.config.example.json']) {
    const absolutePath = path.join(targetRoot, relativePath);
    if (!lstatIfPresent(absolutePath)) continue;
    const state = inspectRelativeFile(targetRoot, relativePath);
    if (!state.ok) {
      errors.push(state.error);
      continue;
    }
    scanText(relativePath, state.content, errors);
  }
}

function gitPath(targetRoot, ...arguments_) {
  const result = spawnSync(
    'git',
    ['-C', targetRoot, 'rev-parse', ...arguments_],
    { encoding: 'utf8', timeout: 10_000 },
  );
  if (result.error || result.status !== 0) return null;
  const value = result.stdout.trim();
  if (!value || value.includes('\0') || value.includes('\n')) return null;
  return path.resolve(targetRoot, value);
}

function pathIsWithin(parent, candidate) {
  const relative = path.relative(parent, candidate);
  return (
    relative === '' ||
    (
      relative !== '..' &&
      !relative.startsWith(`..${path.sep}`) &&
      !path.isAbsolute(relative)
    )
  );
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function findManagedBlock(content) {
  const starts = [...content.matchAll(new RegExp(`^${escapeRegex(START_MARKER)}$`, 'gm'))];
  const ends = [...content.matchAll(new RegExp(`^${escapeRegex(END_MARKER)}$`, 'gm'))];
  if (starts.length !== 1 || ends.length !== 1 || ends[0].index < starts[0].index) {
    return null;
  }
  return content.slice(starts[0].index, ends[0].index + END_MARKER.length);
}

function expectedManagedBlock(templateContent) {
  const body = templateContent
    .replace(/^#![^\n]*\n/, '')
    .replaceAll(`${START_MARKER}\n`, '')
    .replaceAll(`${END_MARKER}\n`, '')
    .trimEnd();
  return `${START_MARKER}\n${body}\n${END_MARKER}`;
}

function inspectGitHooks(targetRoot, selectedComponents, strict, errors, warnings) {
  const configuredHooks = gitPath(targetRoot, '--git-path', 'hooks');
  const commonDirectory = gitPath(targetRoot, '--git-common-dir');
  if (!configuredHooks || !commonDirectory) {
    const message = 'Git hook carrier is unavailable';
    (strict ? errors : warnings).push(message);
    return;
  }
  const expectedHooks = path.join(commonDirectory, 'hooks');
  const customHooks = path.normalize(configuredHooks) !== path.normalize(expectedHooks);
  if (customHooks && !pathIsWithin(targetRoot, configuredHooks)) {
    errors.push('custom core.hooksPath must resolve inside the repository worktree');
    return;
  }
  const hooksStat = lstatIfPresent(configuredHooks);
  if (hooksStat && (hooksStat.isSymbolicLink() || !hooksStat.isDirectory())) {
    errors.push('configured Git hooks carrier must be a regular non-symlink directory');
    return;
  }
  if (customHooks && hooksStat) {
    const realTargetRoot = fs.realpathSync(targetRoot);
    const realConfiguredHooks = fs.realpathSync(configuredHooks);
    if (!pathIsWithin(realTargetRoot, realConfiguredHooks)) {
      errors.push('custom core.hooksPath must resolve inside the repository worktree');
      return;
    }
  }
  const hookCarrierLabel = customHooks
    ? path.relative(targetRoot, configuredHooks).split(path.sep).join('/')
    : '.git/hooks';
  const prePushPath = path.join(configuredHooks, 'pre-push');
  const prePushStat = lstatIfPresent(prePushPath);
  if (prePushStat) {
    if (prePushStat.isSymbolicLink() || !prePushStat.isFile()) {
      warnings.push('pre-push exists but is not a regular file; it was not inspected');
    } else {
      const hookText = fs.readFileSync(prePushPath);
      const findings = [];
      scanText(`${hookCarrierLabel}/pre-push`, hookText, findings);
      const text = hookText.toString('utf8');
      if (
        text.includes(START_MARKER) ||
        /scripts\/harness-v2\//i.test(text)
      ) {
        findings.push(`${hookCarrierLabel}/pre-push contains a Harness consumer`);
      }
      scanAutomaticCommands(`${hookCarrierLabel}/pre-push`, Buffer.from(text), findings);
      errors.push(...findings);
    }
  }
  const preCommitPath = path.join(configuredHooks, 'pre-commit');
  const preCommitStat = lstatIfPresent(preCommitPath);
  let hook = null;
  if (preCommitStat) {
    if (preCommitStat.isSymbolicLink() || !preCommitStat.isFile()) {
      const message = 'pre-commit exists but is not a regular file; automatic commands were not inspected';
      (strict ? errors : warnings).push(message);
    } else {
      hook = fs.readFileSync(preCommitPath, 'utf8');
      scanAutomaticCommands(`${hookCarrierLabel}/pre-commit`, Buffer.from(hook), errors);
    }
  }
  if (!selectedComponents.has('staged-git-safety')) return;
  if (!preCommitStat) {
    (strict ? errors : warnings).push('managed pre-commit hook is not installed');
    return;
  }
  if (preCommitStat.isSymbolicLink() || !preCommitStat.isFile()) {
    errors.push('pre-commit hook must be a regular non-symlink file');
    return;
  }
  if (OBSERVATION_COMMAND.test(hook)) {
    errors.push('pre-commit makes bounded Harness observation an automatic consumer');
  }
  if ((preCommitStat.mode & 0o111) === 0) {
    errors.push('pre-commit hook is not executable');
  }
  if (!/^#!\S+[^\n]*\n/.test(hook)) {
    errors.push('pre-commit hook is missing an executable shebang');
  }
  const installedTemplate = inspectRelativeFile(targetRoot, 'templates/hooks/pre-commit');
  if (!installedTemplate.ok) {
    errors.push(installedTemplate.error);
  } else {
    const actualBlock = findManagedBlock(hook);
    if (!actualBlock) {
      errors.push('pre-commit has malformed or duplicate Harness v2 markers');
    } else if (
      actualBlock !== expectedManagedBlock(installedTemplate.content.toString('utf8'))
    ) {
      errors.push('pre-commit managed block differs from the installed template');
    }
  }
  const requirements = [
    [START_MARKER, 'Harness v2 start marker'],
    [END_MARKER, 'Harness v2 end marker'],
    ['git diff --cached --check', 'staged whitespace gate'],
    ['node scripts/harness-v2/git-gate.js --target . --strict', 'staged Harness gate'],
  ];
  if (selectedComponents.has('code-map')) {
    requirements.push([
      'node scripts/harness-v2/codebase-map.js check --target . --staged --shadow',
      'Code Map staged advisory',
    ]);
  }
  for (const [needle, label] of requirements) {
    if (!hook.includes(needle)) errors.push(`pre-commit is missing ${label}`);
  }
  scanText(`${hookCarrierLabel}/pre-commit`, Buffer.from(hook), errors);
}

function inspectManagedInstallationEntry(relativePath, entry, state, errors) {
  const digest = sha256Buffer(state.content);
  const harnessOwned = ['created', 'adopted'].includes(entry.ownership);
  if (harnessOwned && entry.mutable !== true && digest !== entry.installedSha256) {
    errors.push(`${relativePath} drifted from its managed installation`);
  }
  if (
    harnessOwned &&
    Number.isInteger(entry.installedMode) &&
    state.mode !== entry.installedMode
  ) {
    errors.push(`${relativePath} drifted from its managed installation mode`);
  }
}

function inspectAgentEntrypoints(targetRoot, manifest, errors) {
  const files = {};
  const mapping = {
    'AGENTS.md': 'agents',
    'CLAUDE.md': 'claude',
    '.claude/settings.json': 'settings',
  };
  for (const [relativePath, key] of Object.entries(mapping)) {
    if (!manifest.files[relativePath]) continue;
    const state = inspectRelativeFile(targetRoot, relativePath);
    if (state.ok) files[key] = state.content.toString('utf8');
  }
  errors.push(...validateAgentEntrypoints(files, {
    agents: Boolean(manifest.files['AGENTS.md']),
    claude: Boolean(manifest.files['CLAUDE.md']),
    settings: Boolean(manifest.files['.claude/settings.json']),
  }));
}

function auditTarget(target, options = {}) {
  const targetRoot = path.resolve(target);
  if (!fs.existsSync(targetRoot) || !fs.statSync(targetRoot).isDirectory()) {
    throw new Error('--target must name an existing directory');
  }
  const errors = [];
  const warnings = [];
  const manifestRead = readManifest(targetRoot);
  if (manifestRead.error) {
    return {
      ok: false,
      errors: [
        `manifest ${manifestRead.error.code}: ${manifestRead.error.message}`,
      ],
      warnings,
      checkedFiles: 0,
      manifest: null,
    };
  }
  if (!manifestRead.data) {
    return {
      ok: false,
      errors: ['schema 2 manifest is missing'],
      warnings,
      checkedFiles: 0,
      manifest: null,
    };
  }
  const manifest = manifestRead.data;
  const selectedComponents = new Set(Object.keys(manifest.components));
  let checkedFiles = 0;
  for (const [relativePath, entry] of Object.entries(manifest.files)) {
    for (const owner of entry.components) {
      if (!selectedComponents.has(owner)) {
        errors.push(`${relativePath} names unselected owner component ${owner}`);
      }
    }
    if (relativePath.startsWith('scripts/harness/')) {
      errors.push(`${relativePath} is an old active runtime target; use scripts/harness-v2`);
    }
    const state = inspectRelativeFile(targetRoot, relativePath);
    if (!state.ok) {
      errors.push(state.error);
      continue;
    }
    checkedFiles += 1;
    if (
      AUTOMATIC_OBSERVATION_CONSUMERS.has(relativePath) &&
      !state.content.includes(0) &&
      OBSERVATION_COMMAND.test(state.content.toString('utf8'))
    ) {
      errors.push(
        `${relativePath} makes bounded Harness observation an automatic or default consumer`,
      );
    }
    if (AUTOMATIC_OBSERVATION_CONSUMERS.has(relativePath)) {
      scanAutomaticCommands(relativePath, state.content, errors);
    }
    inspectManagedInstallationEntry(relativePath, entry, state, errors);
    scanText(relativePath, state.content, errors);
  }
  let configResult;
  try {
    configResult = inspectConfig(targetRoot);
    errors.push(...configResult.errors.map((message) => `config: ${message}`));
    warnings.push(...configResult.warnings.map((message) => `config: ${message}`));
  } catch (error) {
    errors.push(`config: ${error.message}`);
  }
  if (selectedComponents.has('context-router')) {
    const context = loadCoreContext(targetRoot);
    if (context.coreStatus !== 'OK') {
      errors.push(
        ...context.issues.map((entry) => `context: ${entry.source} ${entry.code}`),
      );
    }
  }
  inspectAgentEntrypoints(targetRoot, manifest, errors);
  inspectRootEntrypoints(targetRoot, errors);
  inspectCiConsumers(targetRoot, errors);
  inspectGitHooks(targetRoot, selectedComponents, options.strict === true, errors, warnings);
  return {
    ok: errors.length === 0 && (!options.strict || warnings.length === 0),
    errors,
    warnings,
    checkedFiles,
    manifest,
  };
}

function publicReport(result) {
  return {
    ok: result.ok,
    schemaVersion: result.manifest ? result.manifest.schemaVersion : null,
    components: result.manifest
      ? Object.keys(result.manifest.components)
        .sort()
        .map((id) => sanitizeOutputText(id, 120))
      : [],
    checkedFiles: result.checkedFiles,
    errors: result.errors.map((error) => sanitizeOutputText(error, 320)),
    warnings: result.warnings.map((warning) => sanitizeOutputText(warning, 320)),
  };
}

function printReport(report, json) {
  if (json) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    return;
  }
  process.stdout.write(
    `Harness active path audit: ${report.ok ? 'PASS' : 'FAIL'}\n` +
    `Checked files: ${report.checkedFiles}\n`,
  );
  report.errors.forEach((error) => process.stderr.write(`ERROR ${error}\n`));
  report.warnings.forEach((warning) => process.stderr.write(`WARN ${warning}\n`));
}

function runCli(argv = process.argv.slice(2)) {
  try {
    const options = parseArgs(argv);
    if (options.help) {
      process.stdout.write(`${usage()}\n`);
      return 0;
    }
    const result = auditTarget(options.target, options);
    const report = publicReport(result);
    printReport(report, options.json);
    return report.ok ? 0 : 1;
  } catch (error) {
    const message = sanitizeOutputText(error.message, 320);
    if (argv.includes('--json')) {
      process.stdout.write(`${JSON.stringify({ ok: false, error: message }, null, 2)}\n`);
    } else {
      process.stderr.write(`Harness active path audit failed: ${message}\n`);
    }
    return 1;
  }
}

if (require.main === module) process.exitCode = runCli();

module.exports = {
  AUTOMATIC_COMMAND_PATTERNS,
  FIXED_CI_CONSUMERS,
  TEXT_DEPENDENCY_PATTERNS,
  auditTarget,
  gitPath,
  expectedManagedBlock,
  findManagedBlock,
  inspectCiConsumers,
  inspectGitHooks,
  inspectManagedInstallationEntry,
  inspectRelativeFile,
  pathIsWithin,
  parseArgs,
  runCli,
  scanAutomaticCommands,
  scanText,
};
