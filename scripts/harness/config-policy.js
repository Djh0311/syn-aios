#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const balancedDefaults = {
  mode: 'balanced',
  git: {
    requireRepository: false,
    blockProtectedPathChanges: true,
    allowDirtyWorktree: true
  },
  ci: {
    required: false,
    providers: ['github', 'gitlab']
  },
  evidence: {
    requireFreshForCompletion: true,
    maxAgeHours: 24,
    writeIndexOnFinish: false
  },
  ui: {
    requireRealBrowserEvidence: true,
    allowHttpOnlyReachability: false
  },
  hooks: {
    enabled: false,
    preCommit: 'advisory',
    prePush: 'advisory'
  },
  disabledChecks: []
};

const presetDefaults = {
  advisory: {
    mode: 'advisory',
    git: {
      requireRepository: false,
      blockProtectedPathChanges: false,
      allowDirtyWorktree: true
    },
    ci: {
      required: false,
      providers: ['github', 'gitlab']
    },
    evidence: {
      requireFreshForCompletion: false,
      maxAgeHours: 168,
      writeIndexOnFinish: false
    },
    ui: {
      requireRealBrowserEvidence: false,
      allowHttpOnlyReachability: true
    },
    hooks: {
      enabled: false,
      preCommit: 'advisory',
      prePush: 'advisory'
    },
    disabledChecks: []
  },
  balanced: balancedDefaults,
  strict: {
    mode: 'strict',
    git: {
      requireRepository: true,
      blockProtectedPathChanges: true,
      allowDirtyWorktree: false
    },
    ci: {
      required: true,
      providers: ['github', 'gitlab']
    },
    evidence: {
      requireFreshForCompletion: true,
      maxAgeHours: 24,
      writeIndexOnFinish: true
    },
    ui: {
      requireRealBrowserEvidence: true,
      allowHttpOnlyReachability: false
    },
    hooks: {
      enabled: true,
      preCommit: 'hard',
      prePush: 'hard'
    },
    disabledChecks: []
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

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
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
    if (!fs.existsSync(candidate)) {
      if (args.config) return { path: candidate, data: null, error: 'Config file was not found' };
      continue;
    }
    const parsed = readJson(candidate);
    return { path: candidate, data: parsed.data, error: parsed.error };
  }

  return { path: null, data: null, error: null };
}

function isObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function mergeObject(base, override) {
  const result = Object.assign({}, base);
  if (!isObject(override)) return result;
  for (const [key, value] of Object.entries(override)) {
    if (isObject(value) && isObject(result[key])) result[key] = mergeObject(result[key], value);
    else result[key] = value;
  }
  return result;
}

function normalizePolicy(configData) {
  const configured = configData && isObject(configData.policy) ? configData.policy : null;
  const mode = configured && ['advisory', 'balanced', 'strict'].includes(configured.mode) ? configured.mode : 'balanced';
  const merged = mergeObject(presetDefaults[mode], configured || {});
  merged.mode = mode;
  if (!Array.isArray(merged.disabledChecks)) merged.disabledChecks = [];
  return {
    policy: merged,
    usedDefault: !configured,
    invalidMode: Boolean(configured && configured.mode && !['advisory', 'balanced', 'strict'].includes(configured.mode))
  };
}

function policyCheck(id, title, level, reason, source) {
  return { id, title, level, reason, source };
}

function normalizeLevel(value, fallback) {
  if (value === 'hard') return 'hard';
  if (value === 'soft' || value === 'advisory') return 'soft';
  if (value === 'disabled') return 'disabled';
  return fallback;
}

function gateSummary(policy) {
  const checks = [
    policyCheck('no-false-completion', 'No false completion claims', 'hard', 'Universal completion gate', 'harness defaults'),
    policyCheck('no-runtime-doc-overwrite', 'Do not overwrite runtime docs', 'hard', 'Runtime docs are project-owned state', 'harness defaults'),
    policyCheck('no-unauthorized-write-scope', 'Stay inside allowed write scope', 'hard', 'Prevents scope creep and cross-worker collisions', 'harness defaults'),
    policyCheck('unsafe-command-execution', 'Block unsafe command execution', 'hard', 'Aggregate gates must not run unsafe commands', 'harness defaults'),
    policyCheck('configured-policy-violations', 'Configured policy violations', 'hard', 'Explicit policy violations are hard gates', 'harness defaults'),
    policyCheck(
      'git-repository',
      'Target is a Git repository',
      policy.git.requireRepository ? 'hard' : 'disabled',
      policy.git.requireRepository ? 'policy.git.requireRepository is true' : 'Git repository is optional',
      'policy.git.requireRepository'
    ),
    policyCheck(
      'protected-path-changes',
      'Protected paths are not changed',
      policy.git.blockProtectedPathChanges ? 'hard' : 'disabled',
      policy.git.blockProtectedPathChanges ? 'policy.git.blockProtectedPathChanges is true' : 'Protected path blocking is disabled',
      'policy.git.blockProtectedPathChanges'
    ),
    policyCheck(
      'dirty-worktree',
      'Clean worktree for gated runs',
      policy.git.allowDirtyWorktree ? 'disabled' : 'hard',
      policy.git.allowDirtyWorktree ? 'policy.git.allowDirtyWorktree is true' : 'policy.git.allowDirtyWorktree is false',
      'policy.git.allowDirtyWorktree'
    ),
    policyCheck(
      'ci-present',
      'CI configuration exists',
      policy.ci.required ? 'hard' : 'soft',
      policy.ci.required ? 'policy.ci.required is true' : 'CI is recommended but not required',
      'policy.ci.required'
    ),
    policyCheck(
      'fresh-evidence-for-completion',
      'Fresh evidence before completion',
      policy.evidence.requireFreshForCompletion ? 'hard' : 'soft',
      policy.evidence.requireFreshForCompletion ? `Max age: ${policy.evidence.maxAgeHours} hour(s)` : 'Fresh evidence is advisory',
      'policy.evidence.requireFreshForCompletion'
    ),
    policyCheck(
      'evidence-index-on-finish',
      'Evidence index writes on finish',
      policy.evidence.writeIndexOnFinish ? 'soft' : 'disabled',
      policy.evidence.writeIndexOnFinish ? 'policy.evidence.writeIndexOnFinish is true' : 'Evidence index writes are optional',
      'policy.evidence.writeIndexOnFinish'
    ),
    policyCheck(
      'browser-evidence-for-ui-completion',
      'Real browser evidence for UI completion',
      policy.ui.requireRealBrowserEvidence ? 'hard' : 'soft',
      policy.ui.requireRealBrowserEvidence ? 'policy.ui.requireRealBrowserEvidence is true' : 'Real browser evidence is advisory',
      'policy.ui.requireRealBrowserEvidence'
    ),
    policyCheck(
      'http-only-ui-reachability',
      'HTTP-only UI reachability',
      policy.ui.allowHttpOnlyReachability ? 'soft' : 'disabled',
      policy.ui.allowHttpOnlyReachability ? 'HTTP-only evidence is allowed as soft evidence' : 'HTTP-only evidence does not satisfy UI completion',
      'policy.ui.allowHttpOnlyReachability'
    ),
    policyCheck(
      'pre-commit-hook',
      'Pre-commit hook',
      policy.hooks.enabled ? normalizeLevel(policy.hooks.preCommit, 'soft') : 'disabled',
      policy.hooks.enabled ? `policy.hooks.preCommit is ${policy.hooks.preCommit}` : 'Hooks are disabled',
      'policy.hooks.preCommit'
    ),
    policyCheck(
      'pre-push-hook',
      'Pre-push hook',
      policy.hooks.enabled ? normalizeLevel(policy.hooks.prePush, 'soft') : 'disabled',
      policy.hooks.enabled ? `policy.hooks.prePush is ${policy.hooks.prePush}` : 'Hooks are disabled',
      'policy.hooks.prePush'
    )
  ];

  const disabledChecks = new Set(policy.disabledChecks);
  const effective = checks.map((item) => {
    if (!disabledChecks.has(item.id)) return item;
    return Object.assign({}, item, {
      level: 'disabled',
      reason: `Disabled by policy.disabledChecks; original level was ${item.level}`
    });
  });

  return {
    hard: effective.filter((item) => item.level === 'hard'),
    soft: effective.filter((item) => item.level === 'soft'),
    disabled: effective.filter((item) => item.level === 'disabled'),
    unknownDisabledChecks: policy.disabledChecks.filter((item) => !checks.some((check) => check.id === item))
  };
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
      effectivePolicy: null,
      gates: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args);
  report.details.configPath = config.path;
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.error}`);
    return report;
  }
  if (config.path) add(report, 'pass', `Harness config readable: ${path.relative(args.target, config.path) || config.path}`);
  else add(report, 'warn', 'No harness config found; using balanced policy defaults');

  const normalized = normalizePolicy(config.data);
  report.details.effectivePolicy = normalized.policy;
  report.details.gates = gateSummary(normalized.policy);
  if (normalized.usedDefault) add(report, 'warn', 'Config policy object missing; using balanced defaults');
  if (normalized.invalidMode) add(report, 'warn', 'Config policy mode invalid; using balanced defaults');
  else add(report, 'pass', `Policy mode: ${normalized.policy.mode}`);
  if (report.details.gates.disabled.length > 0) add(report, 'warn', `Disabled checks: ${report.details.gates.disabled.map((item) => item.id).join(', ')}`);
  else add(report, 'pass', 'No checks disabled by policy');
  if (report.details.gates.unknownDisabledChecks.length > 0) add(report, 'warn', `Unknown disabled checks: ${report.details.gates.unknownDisabledChecks.join(', ')}`);
  add(report, 'pass', `Hard gate count: ${report.details.gates.hard.length}`);
  add(report, 'pass', `Soft guide count: ${report.details.gates.soft.length}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness config policy: ${report.target}`);
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
