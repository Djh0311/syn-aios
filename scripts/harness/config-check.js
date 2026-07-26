#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

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

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
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
        return { path: full, data: null, error: 'Config file was not found', explicitMissing: true };
      }
      continue;
    }

    const parsed = readJson(full);
    return {
      path: full,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null,
      explicitMissing: false
    };
  }

  return { path: null, data: null, error: null, explicitMissing: false };
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function hasString(value) {
  return typeof value === 'string';
}

function hasNonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0;
}

function isStringArray(value) {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}

const activeBoundaryKeys = ['mechanical', 'reportingOnly', 'explicitTool', 'legacyIgnored'];

function inspectActiveBoundary(value) {
  const details = {
    present: value !== undefined,
    keys: isPlainObject(value) ? Object.keys(value) : [],
    missingKeys: [],
    unknownKeys: [],
    invalidTypes: [],
    crossCategoryDuplicates: [],
    values: Object.fromEntries(activeBoundaryKeys.map((key) => [key, []]))
  };

  if (!isPlainObject(value)) {
    details.invalidTypes.push('activeBoundary');
    return details;
  }

  details.missingKeys = activeBoundaryKeys.filter((key) => !Object.prototype.hasOwnProperty.call(value, key));
  details.unknownKeys = details.keys.filter((key) => !activeBoundaryKeys.includes(key));
  const owners = new Map();
  for (const key of activeBoundaryKeys) {
    const entries = value[key];
    if (!isStringArray(entries)) {
      if (Object.prototype.hasOwnProperty.call(value, key)) details.invalidTypes.push(`activeBoundary.${key}`);
      continue;
    }
    details.values[key] = entries;
    for (const entry of entries) {
      const owner = owners.get(entry);
      if (owner && owner !== key) details.crossCategoryDuplicates.push(entry);
      else owners.set(entry, key);
    }
  }
  details.crossCategoryDuplicates = [...new Set(details.crossCategoryDuplicates)];
  return details;
}

function checkActiveBoundary(report, details, boundary, gates) {
  const boundaryDetails = inspectActiveBoundary(boundary);
  details.activeBoundary = boundaryDetails;
  if (boundaryDetails.invalidTypes.includes('activeBoundary')) {
    add(report, 'fail', '[ACTIVE_BOUNDARY_INVALID_TYPE] activeBoundary must be an object with the four boundary categories');
    return boundaryDetails;
  }
  if (boundaryDetails.missingKeys.length > 0) {
    add(report, 'fail', `[ACTIVE_BOUNDARY_MISSING_KEY] activeBoundary missing key(s): ${boundaryDetails.missingKeys.join(', ')}`);
  }
  if (boundaryDetails.unknownKeys.length > 0) {
    add(report, 'fail', `[ACTIVE_BOUNDARY_UNKNOWN_KEY] activeBoundary has unsupported key(s): ${boundaryDetails.unknownKeys.join(', ')}`);
  }
  if (boundaryDetails.invalidTypes.length > 0) {
    add(report, 'fail', `[ACTIVE_BOUNDARY_INVALID_TYPE] activeBoundary categories must be string arrays: ${boundaryDetails.invalidTypes.join(', ')}`);
  }
  if (boundaryDetails.crossCategoryDuplicates.length > 0) {
    add(report, 'fail', `[ACTIVE_BOUNDARY_CROSS_CATEGORY_DUPLICATE] activeBoundary entries appear in multiple categories: ${boundaryDetails.crossCategoryDuplicates.join(', ')}`);
  }

  const hardGateIds = new Set(Array.isArray(gates && gates.hard) ? gates.hard : []);
  const nonMechanicalHardGates = ['reportingOnly', 'explicitTool']
    .flatMap((key) => boundaryDetails.values[key])
    .filter((entry) => hardGateIds.has(entry));
  boundaryDetails.nonMechanicalHardGates = [...new Set(nonMechanicalHardGates)];
  if (boundaryDetails.nonMechanicalHardGates.length > 0) {
    add(report, 'fail', `[ACTIVE_BOUNDARY_NON_MECHANICAL_HARD_GATE] reportingOnly/explicitTool entries cannot be hard gates: ${boundaryDetails.nonMechanicalHardGates.join(', ')}`);
  }

  if (boundaryDetails.missingKeys.length === 0
    && boundaryDetails.unknownKeys.length === 0
    && boundaryDetails.invalidTypes.length === 0
    && boundaryDetails.crossCategoryDuplicates.length === 0
    && boundaryDetails.nonMechanicalHardGates.length === 0) {
    add(report, 'pass', 'activeBoundary has exactly four disjoint categories and no non-mechanical hard gate');
  }
  return boundaryDetails;
}

function checkObject(report, data, key) {
  if (isPlainObject(data[key])) {
    add(report, 'pass', `Config object present: ${key}`);
    return true;
  }
  add(report, 'fail', `Config object missing or invalid: ${key}`);
  return false;
}

function checkStringArray(report, owner, keyPath, value, options = {}) {
  if (!Array.isArray(value)) {
    add(report, options.required === false ? 'warn' : 'fail', `Config array missing or invalid: ${keyPath}`);
    return false;
  }
  if (!isStringArray(value)) {
    add(report, 'fail', `Config array must contain only strings: ${keyPath}`);
    return false;
  }
  add(report, 'pass', `Config string array valid: ${keyPath} (${value.length})`);
  owner[keyPath] = value.length;
  return true;
}

function checkCommands(report, details, commands) {
  const expected = [
    'packageManager',
    'install',
    'lint',
    'typecheck',
    'test',
    'testUnit',
    'testIntegration',
    'testE2E',
    'build',
    'dev'
  ];

  const missing = expected.filter((key) => !Object.prototype.hasOwnProperty.call(commands, key));
  if (missing.length > 0) add(report, 'fail', `commands missing key(s): ${missing.join(', ')}`);
  else add(report, 'pass', 'commands keys present');

  const nonString = expected.filter((key) => Object.prototype.hasOwnProperty.call(commands, key) && !hasString(commands[key]));
  if (nonString.length > 0) add(report, 'fail', `commands key(s) must be strings: ${nonString.join(', ')}`);
  else add(report, 'pass', 'commands values are strings');

  if (/pnpm \| npm \| yarn \| bun \| none/.test(commands.packageManager || '')) {
    add(report, 'warn', 'commands.packageManager still contains the template choice list');
  }

  details.commands = {
    expected,
    missing,
    nonString
  };
}

function checkRuntimeDocs(report, details, runtimeDocs) {
  const requiredStrings = ['templateSource', 'projectTarget'];
  for (const key of requiredStrings) {
    if (hasNonEmptyString(runtimeDocs[key])) add(report, 'pass', `runtimeDocs.${key} is set`);
    else add(report, 'fail', `runtimeDocs.${key} is missing or empty`);
  }
  checkStringArray(report, details.arrays, 'runtimeDocs.protected', runtimeDocs.protected);
}

function checkTools(report, details, tools) {
  if (!isPlainObject(tools.browser)) add(report, 'fail', 'tools.browser is missing or invalid');
  else {
    if (hasNonEmptyString(tools.browser.preferred)) add(report, 'pass', 'tools.browser.preferred is set');
    else add(report, 'warn', 'tools.browser.preferred is missing or empty');
    checkStringArray(report, details.arrays, 'tools.browser.fallbacks', tools.browser.fallbacks, { required: false });
  }

  if (!isPlainObject(tools.mcp)) add(report, 'fail', 'tools.mcp is missing or invalid');
  else {
    checkStringArray(report, details.arrays, 'tools.mcp.required', tools.mcp.required, { required: false });
    checkStringArray(report, details.arrays, 'tools.mcp.preferred', tools.mcp.preferred, { required: false });
    checkStringArray(report, details.arrays, 'tools.mcp.optional', tools.mcp.optional, { required: false });
  }
}

function checkGates(report, details, gates) {
  checkStringArray(report, details.arrays, 'gates.hard', gates.hard);
  checkStringArray(report, details.arrays, 'gates.soft', gates.soft, { required: false });
  if (!isPlainObject(gates.escapeHatch)) {
    add(report, 'fail', 'gates.escapeHatch is missing or invalid');
    return;
  }

  const requiresConfirmation = gates.escapeHatch.requiresExplicitUserConfirmation;
  const recordReason = gates.escapeHatch.recordReason;
  if (requiresConfirmation === true) add(report, 'pass', 'gates.escapeHatch.requiresExplicitUserConfirmation is true');
  else add(report, 'warn', 'gates.escapeHatch.requiresExplicitUserConfirmation is not true');
  if (recordReason === true) add(report, 'pass', 'gates.escapeHatch.recordReason is true');
  else add(report, 'warn', 'gates.escapeHatch.recordReason is not true');
}

function checkPolicy(report, details, policy) {
  if (!isPlainObject(policy)) {
    add(report, 'warn', 'policy object is missing or invalid');
    return;
  }

  const allowedModes = new Set(['advisory', 'balanced', 'strict']);
  if (allowedModes.has(policy.mode)) add(report, 'pass', `policy.mode is valid: ${policy.mode}`);
  else add(report, 'warn', 'policy.mode is missing or not one of advisory, balanced, strict');

  for (const key of ['git', 'ci', 'evidence', 'ui', 'hooks']) {
    if (isPlainObject(policy[key])) add(report, 'pass', `policy.${key} object is present`);
    else add(report, 'warn', `policy.${key} object is missing or invalid`);
  }

  checkStringArray(report, details.arrays, 'policy.disabledChecks', policy.disabledChecks, { required: false });
}

function isLocalEndpoint(value) {
  return /^https?:\/\/(?:127\.0\.0\.1|localhost|\[::1\])(?::\d+)?(?:\/|$)/i.test(String(value || ''));
}

function checkMemoryIntegration(report, details, memoryIntegration) {
  details.memoryIntegration = {
    present: isPlainObject(memoryIntegration),
    enabled: null,
    provider: null,
    endpoint: null
  };

  if (!isPlainObject(memoryIntegration)) {
    add(report, 'warn', 'memoryIntegration object is missing; agentmemory governance is disabled');
    return;
  }

  details.memoryIntegration.enabled = memoryIntegration.enabled;
  details.memoryIntegration.provider = memoryIntegration.provider || null;
  details.memoryIntegration.endpoint = memoryIntegration.endpoint || null;

  if (typeof memoryIntegration.enabled === 'boolean') add(report, 'pass', 'memoryIntegration.enabled is boolean');
  else add(report, 'fail', 'memoryIntegration.enabled must be boolean when present');

  if (memoryIntegration.provider === 'agentmemory') add(report, 'pass', 'memoryIntegration.provider is agentmemory');
  else add(report, 'fail', 'memoryIntegration.provider must be agentmemory');

  if (memoryIntegration.mode === 'governed') add(report, 'pass', 'memoryIntegration.mode is governed');
  else add(report, 'warn', 'memoryIntegration.mode should be governed');

  if (hasNonEmptyString(memoryIntegration.endpoint)) add(report, 'pass', 'memoryIntegration.endpoint is set');
  else add(report, 'warn', 'memoryIntegration.endpoint is missing or empty');

  const auth = isPlainObject(memoryIntegration.auth) ? memoryIntegration.auth : {};
  if (!isLocalEndpoint(memoryIntegration.endpoint) && !hasNonEmptyString(auth.secretEnv)) {
    add(report, 'warn', 'memoryIntegration endpoint is not local and auth.secretEnv is not set');
  }

  const readPolicy = isPlainObject(memoryIntegration.readPolicy) ? memoryIntegration.readPolicy : {};
  const writePolicy = isPlainObject(memoryIntegration.writePolicy) ? memoryIntegration.writePolicy : {};
  const retention = isPlainObject(memoryIntegration.retention) ? memoryIntegration.retention : {};
  if (Number.isInteger(readPolicy.maxMemoriesPerTask) && readPolicy.maxMemoriesPerTask >= 0) add(report, 'pass', 'memoryIntegration.readPolicy.maxMemoriesPerTask is valid');
  else add(report, 'warn', 'memoryIntegration.readPolicy.maxMemoriesPerTask should be a non-negative integer');
  if (Number.isInteger(readPolicy.maxMemoryChars) && readPolicy.maxMemoryChars > 0) add(report, 'pass', 'memoryIntegration.readPolicy.maxMemoryChars is valid');
  else add(report, 'warn', 'memoryIntegration.readPolicy.maxMemoryChars should be a positive integer');
  if (readPolicy.respectProjectDocsOverMemory === true) add(report, 'pass', 'memoryIntegration respects project docs over memory');
  else add(report, 'warn', 'memoryIntegration.readPolicy.respectProjectDocsOverMemory should be true');
  if (writePolicy.autoWrite === false) add(report, 'pass', 'memoryIntegration.writePolicy.autoWrite is false');
  else add(report, 'warn', 'memoryIntegration.writePolicy.autoWrite should stay false');
  if (writePolicy.candidateQueue === true) add(report, 'pass', 'memoryIntegration.writePolicy.candidateQueue is true');
  else add(report, 'warn', 'memoryIntegration.writePolicy.candidateQueue should be true');
  if (writePolicy.blockSecrets === true) add(report, 'pass', 'memoryIntegration.writePolicy.blockSecrets is true');
  else add(report, report.strict ? 'fail' : 'warn', 'memoryIntegration.writePolicy.blockSecrets must be true for strict mode');
  if (writePolicy.requirePromotion === true) add(report, 'pass', 'memoryIntegration.writePolicy.requirePromotion is true');
  else add(report, report.strict ? 'fail' : 'warn', 'memoryIntegration.writePolicy.requirePromotion must be true for strict mode');
  if (writePolicy.quarantinePromptInjection === true) add(report, 'pass', 'memoryIntegration.writePolicy.quarantinePromptInjection is true');
  else add(report, 'warn', 'memoryIntegration.writePolicy.quarantinePromptInjection should be true');
  if (Number.isInteger(retention.reviewAfterDays) && retention.reviewAfterDays > 0) add(report, 'pass', 'memoryIntegration.retention.reviewAfterDays is valid');
  else add(report, 'warn', 'memoryIntegration.retention.reviewAfterDays should be a positive integer');
}

function extractNodeScript(command) {
  const match = String(command).match(/(?:^|\s)node\s+([^\s]+scripts\/harness\/[^\s]+\.js|scripts\/harness\/[^\s]+\.js)/);
  return match ? match[1] : null;
}

function checkRecommendedCommands(report, details, targetRoot, ownerKey, owner) {
  if (!isPlainObject(owner)) {
    add(report, 'fail', `${ownerKey} is missing or invalid`);
    return;
  }

  const commandKeys = ['recommendedChecks', 'strictPathRecommendedChecks'];
  details.recommendedCommands[ownerKey] = {};

  for (const key of commandKeys) {
    const keyPath = `${ownerKey}.${key}`;
    const commands = owner[key];
    if (!checkStringArray(report, details.arrays, keyPath, commands, { required: key === 'recommendedChecks' })) continue;

    const commandResults = commands.map((command) => {
      const script = extractNodeScript(command);
      const exists = script ? fs.existsSync(path.join(targetRoot, script)) : null;
      if (script && exists) add(report, 'pass', `${keyPath} script exists: ${script}`);
      else if (script) add(report, 'fail', `${keyPath} script is missing: ${script}`);
      else add(report, 'warn', `${keyPath} command is not a recognized node harness script: ${command}`);
      return { command, script, exists };
    });

    details.recommendedCommands[ownerKey][key] = commandResults;
  }
}

function checkUi(report, details, ui) {
  if (!isPlainObject(ui)) {
    add(report, 'warn', 'ui object is missing or invalid');
    return;
  }

  if (!Array.isArray(ui.targets)) {
    add(report, 'warn', 'ui.targets is missing or invalid');
    return;
  }

  details.uiTargets = ui.targets.map((target, index) => {
    const missing = [];
    if (!target || typeof target !== 'object') missing.push('target-object');
    else {
      if (!hasNonEmptyString(target.name)) missing.push('name');
      if (!hasNonEmptyString(target.url)) missing.push('url');
      if (!isStringArray(target.viewports) || target.viewports.length === 0) missing.push('viewports');
      if (!isStringArray(target.requiredEvidence) || target.requiredEvidence.length === 0) missing.push('requiredEvidence');
    }

    if (missing.length === 0) add(report, 'pass', `ui.targets[${index}] is structurally valid`);
    else add(report, 'warn', `ui.targets[${index}] is missing field(s): ${missing.join(', ')}`);
    return { index, missing };
  });
}

function checkTemplatePlaceholders(report, data, configPath) {
  const basename = configPath ? path.basename(configPath) : '';
  const isExample = basename === 'harness.config.example.json';

  if (data.project && data.project.name === 'example-project') {
    add(report, isExample ? 'warn' : 'fail', 'project.name is still example-project');
  }
  if (data.project && /Replace with/.test(data.project.description || '')) {
    add(report, isExample ? 'warn' : 'fail', 'project.description still contains template placeholder text');
  }
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      configPath: null,
      arrays: {},
      commands: null,
      recommendedCommands: {},
      uiTargets: [],
      activeBoundary: null
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

  const config = loadConfig(args);
  report.details.configPath = config.path ? rel(args.target, config.path) : null;

  if (!config.path) {
    add(report, args.strict ? 'fail' : 'warn', 'No harness.config.json or harness.config.example.json found in target');
    return report;
  }
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.path} (${config.error})`);
    return report;
  }

  const data = config.data;
  add(report, 'pass', `Harness config parsed: ${rel(args.target, config.path)}`);

  if (data.schemaVersion !== undefined) add(report, 'pass', 'schemaVersion is present');
  else add(report, 'fail', 'schemaVersion is missing');

  const requiredObjects = ['project', 'commands', 'runtimeDocs', 'tools', 'gates', 'preWork', 'preCompletion'];
  const objectOk = Object.fromEntries(requiredObjects.map((key) => [key, checkObject(report, data, key)]));

  if (objectOk.project) {
    if (hasNonEmptyString(data.project.name)) add(report, 'pass', 'project.name is set');
    else add(report, 'fail', 'project.name is missing or empty');
    if (hasNonEmptyString(data.project.type)) add(report, 'pass', 'project.type is set');
    else add(report, 'warn', 'project.type is missing or empty');
  }

  if (objectOk.commands) checkCommands(report, report.details, data.commands);
  if (objectOk.runtimeDocs) checkRuntimeDocs(report, report.details, data.runtimeDocs);
  checkStringArray(report, report.details.arrays, 'protectedPaths', data.protectedPaths, { required: false });
  if (objectOk.tools) checkTools(report, report.details, data.tools);
  checkPolicy(report, report.details, data.policy);
  if (objectOk.gates) checkGates(report, report.details, data.gates);
  checkActiveBoundary(report, report.details, data.activeBoundary, data.gates);
  if (objectOk.preWork) checkRecommendedCommands(report, report.details, args.target, 'preWork', data.preWork);
  if (objectOk.preCompletion) checkRecommendedCommands(report, report.details, args.target, 'preCompletion', data.preCompletion);
  checkMemoryIntegration(report, report.details, data.memoryIntegration);
  checkUi(report, report.details, data.ui);
  checkTemplatePlaceholders(report, data, config.path);

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
  console.log(`Harness config check: ${report.target}`);
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
