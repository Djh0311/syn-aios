#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');

const { readManifest } = require('./lib/manifest');
const { sanitizeOutputText } = require('./lib/output-safety');

const CONFIG_PATH = 'harness.config.json';
// Generic engineering tutorials remain source assets only. This compatibility
// component must not claim that a project installation received any of them.
const ENGINEERING_SKILL_IDS = Object.freeze([]);
const SKILL_ID = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const FORBIDDEN_KEYS = new Set([
  'profile',
  'policy',
  'gates',
  'preWork',
  'preCompletion',
  'automaticLifecycle',
  'autoRisk',
  'completionProtocol',
  'taskLifecycle',
  'runtimeDocs',
  'verificationRunner',
  'runtimeLedger',
  'full',
  'legacy',
]);
const COMPONENT_FIELDS = {
  collaborationContract: ['enabled'],
  contextRouter: ['enabled'],
  agentAdapters: ['enabled'],
  stagedGitSafety: ['enabled', 'hook', 'scope'],
  codeMap: ['enabled', 'indexPath', 'hookAdvisory', 'limit'],
  taskPackageCollaboration: [
    'enabled',
    'templatePath',
    'maxBytes',
    'headerMaxBytes',
    'headerMaxLines',
    'bodySoftMaxBytes',
  ],
  gitTaskLifecycle: [
    'enabled',
    'defaultContext',
    'automaticStaging',
    'dryRunByDefault',
    'persistentLedger',
    'pushSupported',
    'forceCleanupSupported',
  ],
  mistakeLedger: [
    'enabled',
    'path',
    'defaultContext',
    'automaticAppend',
    'softMaxActiveEntries',
    'softMaxBytes',
  ],
  highRiskBoundary: [
    'enabled',
    'rulesPath',
    'runbookTemplatePath',
    'loadJustInTime',
    'preflightConsumesAuthorization',
  ],
  harnessObservation: [
    'enabled',
    'ledgerPath',
    'defaultContext',
    'automaticAppend',
    'blocksCompletion',
    'minimumSamples',
    'maximumSamples',
  ],
  engineeringSkills: ['enabled', 'defaultLoad', 'codexRoot', 'claudeRoot', 'skills'],
  installationAudit: ['enabled'],
  ktErp: ['enabled', 'loadByDefault', 'loadMatchingRunbookOnly'],
  mengchongBox: ['enabled', 'loadByDefault', 'loadMatchingRunbookOnly'],
};

function componentConfigKey(id) {
  return id.replace(/-([a-z0-9])/g, (_, character) => character.toUpperCase());
}

function usage() {
  return [
    'Usage: node scripts/harness-v2/config-check.js --target <project> [--strict] [--json]',
    '',
    'Read-only schema 2 configuration validation. No legacy or profile fallback is performed.',
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

function collectForbiddenKeys(value, location = 'config', findings = []) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectForbiddenKeys(item, `${location}[${index}]`, findings));
    return findings;
  }
  if (!value || typeof value !== 'object') return findings;
  for (const [key, child] of Object.entries(value)) {
    if (FORBIDDEN_KEYS.has(key)) findings.push(`${location}.${key} is a removed Harness v1 key`);
    collectForbiddenKeys(child, `${location}.${key}`, findings);
  }
  return findings;
}

function requireBoolean(value, location, errors) {
  if (typeof value !== 'boolean') errors.push(`${location} must be boolean`);
}

function validateComponentConfig(key, value, errors) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    errors.push(`components.${key} must be an object`);
    return;
  }
  const allowed = COMPONENT_FIELDS[key];
  if (!allowed) {
    errors.push(`components.${key} is not a known Harness v2 component`);
    return;
  }
  for (const field of Object.keys(value)) {
    if (!allowed.includes(field)) errors.push(`components.${key}.${field} is unsupported`);
  }
  requireBoolean(value.enabled, `components.${key}.enabled`, errors);
  for (const field of [
    'hookAdvisory',
    'defaultContext',
    'automaticAppend',
    'blocksCompletion',
    'loadJustInTime',
    'preflightConsumesAuthorization',
    'defaultLoad',
    'loadByDefault',
    'loadMatchingRunbookOnly',
    'automaticStaging',
    'dryRunByDefault',
    'persistentLedger',
    'pushSupported',
    'forceCleanupSupported',
  ]) {
    if (value[field] !== undefined) requireBoolean(value[field], `components.${key}.${field}`, errors);
  }
  for (const field of [
    'limit',
    'maxBytes',
    'headerMaxBytes',
    'headerMaxLines',
    'bodySoftMaxBytes',
    'softMaxActiveEntries',
    'softMaxBytes',
    'minimumSamples',
    'maximumSamples',
  ]) {
    if (
      value[field] !== undefined &&
      (!Number.isSafeInteger(value[field]) || value[field] <= 0)
    ) {
      errors.push(`components.${key}.${field} must be a positive integer`);
    }
  }
  for (const field of [
    'hook',
    'scope',
    'indexPath',
    'templatePath',
    'path',
    'rulesPath',
    'runbookTemplatePath',
    'ledgerPath',
    'codexRoot',
    'claudeRoot',
  ]) {
    if (value[field] !== undefined && (typeof value[field] !== 'string' || !value[field].trim())) {
      errors.push(`components.${key}.${field} must be a non-empty string`);
    }
  }
  if (key === 'stagedGitSafety') {
    if (value.hook !== 'pre-commit') errors.push('components.stagedGitSafety.hook must be pre-commit');
    if (value.scope !== 'staged-only') errors.push('components.stagedGitSafety.scope must be staged-only');
  }
  if (key === 'taskPackageCollaboration') {
    if (
      value.headerMaxBytes !== undefined &&
      value.headerMaxBytes !== 4096
    ) {
      errors.push('components.taskPackageCollaboration.headerMaxBytes must be 4096');
    }
    if (
      value.headerMaxLines !== undefined &&
      value.headerMaxLines !== 60
    ) {
      errors.push('components.taskPackageCollaboration.headerMaxLines must be 60');
    }
    if (
      value.bodySoftMaxBytes !== undefined &&
      value.bodySoftMaxBytes !== 32768
    ) {
      errors.push('components.taskPackageCollaboration.bodySoftMaxBytes must be 32768');
    }
  }
  if (key === 'gitTaskLifecycle') {
    for (const [field, expected] of [
      ['defaultContext', false],
      ['automaticStaging', false],
      ['dryRunByDefault', true],
      ['persistentLedger', false],
      ['pushSupported', false],
      ['forceCleanupSupported', false],
    ]) {
      if (value[field] !== expected) {
        errors.push(`components.gitTaskLifecycle.${field} must be ${expected}`);
      }
    }
  }
  if (key === 'harnessObservation') {
    if (value.ledgerPath !== 'docs/harness/observations/WAVE-1.tsv') {
      errors.push(
        'components.harnessObservation.ledgerPath must be docs/harness/observations/WAVE-1.tsv',
      );
    }
    if (value.defaultContext !== false) {
      errors.push('components.harnessObservation.defaultContext must be false');
    }
    if (value.automaticAppend !== false) {
      errors.push('components.harnessObservation.automaticAppend must be false');
    }
    if (value.blocksCompletion !== false) {
      errors.push('components.harnessObservation.blocksCompletion must be false');
    }
    if (value.minimumSamples !== 8 || value.maximumSamples !== 12) {
      errors.push('components.harnessObservation sample bounds must be 8 and 12');
    }
  }
  if (key === 'engineeringSkills') {
    if (value.codexRoot !== '.agents/skills') {
      errors.push('components.engineeringSkills.codexRoot must be .agents/skills');
    }
    if (value.claudeRoot !== '.claude/skills') {
      errors.push('components.engineeringSkills.claudeRoot must be .claude/skills');
    }
    if (
      !Array.isArray(value.skills) ||
      value.skills.some((skill) => typeof skill !== 'string' || !SKILL_ID.test(skill))
    ) {
      errors.push('components.engineeringSkills.skills must contain valid skill ids');
    } else {
      if (new Set(value.skills).size !== value.skills.length) {
        errors.push('components.engineeringSkills.skills must not contain duplicates');
      }
      const actual = [...new Set(value.skills)].sort();
      if (
        value.skills.length !== ENGINEERING_SKILL_IDS.length ||
        JSON.stringify(actual) !== JSON.stringify(ENGINEERING_SKILL_IDS)
      ) {
        errors.push(
          'components.engineeringSkills.skills must be empty because generic tutorials are source-only',
        );
      }
    }
  }
}

function validateConfig(config, manifest) {
  const errors = [];
  const warnings = [];
  if (!config || typeof config !== 'object' || Array.isArray(config)) {
    return { ok: false, errors: ['config root must be an object'], warnings };
  }
  errors.push(...collectForbiddenKeys(config));
  const allowedRoot = new Set(['schemaVersion', 'project', 'components']);
  for (const key of Object.keys(config)) {
    if (!allowedRoot.has(key)) errors.push(`config.${key} is unsupported`);
  }
  if (config.schemaVersion !== 2) errors.push('schemaVersion must be 2');
  if (!config.project || typeof config.project !== 'object' || Array.isArray(config.project)) {
    errors.push('project must be an object');
  } else {
    for (const key of Object.keys(config.project)) {
      if (!['name', 'pack'].includes(key)) errors.push(`project.${key} is unsupported`);
    }
    if (typeof config.project.name !== 'string' || !config.project.name.trim()) {
      errors.push('project.name must be a non-empty string');
    }
    if (
      config.project.pack !== null &&
      (typeof config.project.pack !== 'string' || !config.project.pack.trim())
    ) {
      errors.push('project.pack must be a non-empty string or null');
    }
  }
  if (!config.components || typeof config.components !== 'object' || Array.isArray(config.components)) {
    errors.push('components must be an object');
  } else {
    for (const [key, value] of Object.entries(config.components)) {
      validateComponentConfig(key, value, errors);
    }
  }
  if (manifest && config.project && config.components) {
    if (config.project.name !== manifest.projectName) {
      errors.push('project.name does not match the installed manifest');
    }
    if ((config.project.pack || null) !== (manifest.selection.pack || null)) {
      errors.push('project.pack does not match the installed manifest');
    }
    for (const componentId of Object.keys(manifest.components)) {
      const key = componentConfigKey(componentId);
      if (!config.components[key]) {
        errors.push(`selected component ${componentId} is missing from config`);
      } else if (config.components[key].enabled !== true) {
        errors.push(`selected component ${componentId} must be enabled`);
      }
    }
    for (const [key, value] of Object.entries(config.components)) {
      if (value.enabled !== true) continue;
      const installed = Object.keys(manifest.components)
        .some((id) => componentConfigKey(id) === key);
      if (!installed) errors.push(`enabled config component ${key} is not installed`);
    }
  }
  return { ok: errors.length === 0, errors, warnings };
}

function validateAgentEntrypoints(files, required = {}) {
  const errors = [];
  const activeTextPatterns = [
    /scripts\/harness\/pre-work\.js/i,
    /scripts\/harness\/pre-completion\.js/i,
    /scripts\/harness\/task-(?:start|finish|status|risk)\.js/i,
    /scripts\/harness\/runtime-ledger\.js/i,
    new RegExp(`using-${['super', 'powers'].join('')}`, 'i'),
    /\.claude\/hooks\/stop-precompletion\.sh/i,
  ];
  function rejectOldConsumers(relativePath, text) {
    if (activeTextPatterns.some((expression) => expression.test(text))) {
      errors.push(`${relativePath} contains a removed Harness consumer`);
    }
  }

  if (required.agents || files.agents !== undefined) {
    if (typeof files.agents !== 'string') {
      errors.push('AGENTS.md is missing from the predicted installation');
    } else {
      rejectOldConsumers('AGENTS.md', files.agents);
      if (!files.agents.includes('scripts/harness-v2/project-context.js')) {
        errors.push('AGENTS.md does not route new sessions through Harness v2');
      }
      for (const mode of ['Quick', 'Plan', 'Guidance', 'Development']) {
        if (!new RegExp(`^###\\s+${mode}\\s*$`, 'im').test(files.agents)) {
          errors.push(`AGENTS.md is missing the ${mode} working mode`);
        }
      }
    }
  }
  if (required.claude || files.claude !== undefined) {
    if (typeof files.claude !== 'string') {
      errors.push('CLAUDE.md is missing from the predicted installation');
    } else {
      rejectOldConsumers('CLAUDE.md', files.claude);
      if (!/^\s*@AGENTS\.md\s*$/m.test(files.claude)) {
        errors.push('CLAUDE.md does not delegate to AGENTS.md');
      }
      if (/docs\/harness\/(?:AUTHORITY|CURRENT)\.md/i.test(files.claude)) {
        errors.push('CLAUDE.md defines an independent authority route');
      }
    }
  }
  if (required.settings || files.settings !== undefined) {
    if (typeof files.settings !== 'string') {
      errors.push('.claude/settings.json is missing from the predicted installation');
    } else {
      rejectOldConsumers('.claude/settings.json', files.settings);
      let settings;
      try {
        settings = JSON.parse(files.settings);
      } catch (error) {
        errors.push(`.claude/settings.json is invalid JSON: ${error.message}`);
        return errors;
      }
      const hooks =
        settings && settings.hooks && typeof settings.hooks === 'object' && !Array.isArray(settings.hooks)
          ? settings.hooks
          : {};
      if (Object.prototype.hasOwnProperty.call(hooks, 'Stop')) {
        errors.push('.claude/settings.json must not configure the removed Stop hook');
      }
      if (!Object.prototype.hasOwnProperty.call(hooks, 'SessionStart')) {
        errors.push('.claude/settings.json is missing SessionStart');
      } else if (!JSON.stringify(hooks.SessionStart).includes('.claude/hooks/session-start.sh')) {
        errors.push('.claude/settings.json SessionStart does not invoke the Harness v2 adapter');
      }
    }
  }
  return errors;
}

function inspectConfig(target) {
  const targetRoot = path.resolve(target);
  if (!fs.existsSync(targetRoot) || !fs.statSync(targetRoot).isDirectory()) {
    throw new Error('--target must name an existing directory');
  }
  const manifestRead = readManifest(targetRoot);
  if (manifestRead.error) {
    throw new Error(
      `cannot validate config without a valid schema 2 manifest (${manifestRead.error.code}): ` +
      manifestRead.error.message,
    );
  }
  if (!manifestRead.data) throw new Error('installed schema 2 manifest is missing');
  const configPath = path.join(targetRoot, CONFIG_PATH);
  if (!fs.existsSync(configPath)) {
    return {
      ok: false,
      errors: [`${CONFIG_PATH} is missing`],
      warnings: [],
      config: null,
      manifest: manifestRead.data,
    };
  }
  if (fs.lstatSync(configPath).isSymbolicLink() || !fs.statSync(configPath).isFile()) {
    return {
      ok: false,
      errors: [`${CONFIG_PATH} must be a regular non-symlink file`],
      warnings: [],
      config: null,
      manifest: manifestRead.data,
    };
  }
  let config;
  try {
    config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  } catch (error) {
    return {
      ok: false,
      errors: [`${CONFIG_PATH} is invalid JSON: ${error.message}`],
      warnings: [],
      config: null,
      manifest: manifestRead.data,
    };
  }
  return { ...validateConfig(config, manifestRead.data), config, manifest: manifestRead.data };
}

function publicReport(result) {
  return {
    ok: result.ok,
    schemaVersion: result.config && result.config.schemaVersion === 2 ? 2 : null,
    errors: result.errors.map((error) => sanitizeOutputText(error, 320)),
    warnings: result.warnings.map((warning) => sanitizeOutputText(warning, 320)),
  };
}

function printReport(report, json) {
  if (json) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    return;
  }
  process.stdout.write(`Harness config: ${report.ok ? 'PASS' : 'FAIL'}\n`);
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
    const result = inspectConfig(options.target);
    const report = publicReport(result);
    printReport(report, options.json);
    return report.ok && (!options.strict || report.warnings.length === 0) ? 0 : 1;
  } catch (error) {
    const message = sanitizeOutputText(error.message, 320);
    if (argv.includes('--json')) {
      process.stdout.write(`${JSON.stringify({ ok: false, error: message }, null, 2)}\n`);
    } else {
      process.stderr.write(`Harness config check failed: ${message}\n`);
    }
    return 1;
  }
}

if (require.main === module) process.exitCode = runCli();

module.exports = {
  COMPONENT_FIELDS,
  CONFIG_PATH,
  ENGINEERING_SKILL_IDS,
  FORBIDDEN_KEYS,
  collectForbiddenKeys,
  componentConfigKey,
  inspectConfig,
  parseArgs,
  runCli,
  validateAgentEntrypoints,
  validateConfig,
};
