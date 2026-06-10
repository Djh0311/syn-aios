#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { projectPresetRecommendation } = require('./lib/risk-classifier');

const packageManagers = ['pnpm', 'npm', 'yarn', 'bun'];
const policyPresets = {
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
  balanced: {
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
  },
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
    preset: 'auto',
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--preset') args.preset = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.preset && args.preset !== 'auto' && !Object.prototype.hasOwnProperty.call(policyPresets, args.preset)) {
    throw new Error(`Unknown preset: ${args.preset}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error };
  }
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function detectPackageManager(targetRoot, pkg) {
  if (pkg && typeof pkg.packageManager === 'string') {
    const name = pkg.packageManager.split('@')[0];
    if (packageManagers.includes(name)) return name;
  }
  if (exists(targetRoot, 'pnpm-lock.yaml')) return 'pnpm';
  if (exists(targetRoot, 'yarn.lock')) return 'yarn';
  if (exists(targetRoot, 'bun.lockb') || exists(targetRoot, 'bun.lock')) return 'bun';
  if (exists(targetRoot, 'package-lock.json')) return 'npm';
  return 'npm';
}

function projectType(targetRoot, pkg) {
  const deps = Object.assign({}, pkg && pkg.dependencies, pkg && pkg.devDependencies);
  const names = Object.keys(deps);
  if (exists(targetRoot, 'pnpm-workspace.yaml') || exists(targetRoot, 'turbo.json') || exists(targetRoot, 'nx.json')) return 'monorepo';
  if (names.some((name) => /next|vite|react|vue|svelte|astro|remix/.test(name))) return 'web-app';
  if (names.some((name) => /express|fastify|hono|koa|@nestjs/.test(name))) return 'api';
  if (exists(targetRoot, 'pyproject.toml') || exists(targetRoot, 'requirements.txt')) return 'python';
  if (exists(targetRoot, 'Cargo.toml')) return 'rust';
  if (exists(targetRoot, 'go.mod')) return 'go';
  if (exists(targetRoot, 'pom.xml') || exists(targetRoot, 'build.gradle') || exists(targetRoot, 'build.gradle.kts')) return 'java';
  return 'other';
}

function commandFromScripts(pkg, key) {
  const scripts = pkg && pkg.scripts && typeof pkg.scripts === 'object' ? pkg.scripts : {};
  const scriptNames = {
    lint: ['lint'],
    typecheck: ['typecheck', 'type-check', 'tsc'],
    test: ['test'],
    testUnit: ['test:unit', 'unit'],
    testIntegration: ['test:integration', 'integration'],
    testE2E: ['test:e2e', 'e2e'],
    build: ['build'],
    dev: ['dev', 'start']
  }[key] || [];
  return scriptNames.find((name) => Object.prototype.hasOwnProperty.call(scripts, name)) || '';
}

function runCommand(packageManager, scriptName) {
  if (!scriptName) return '';
  if (packageManager === 'yarn') return `yarn ${scriptName}`;
  if (packageManager === 'bun') return `bun run ${scriptName}`;
  return `${packageManager} run ${scriptName}`;
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

function policyWithPreset(examplePolicy, existingPolicy, presetName) {
  const configuredMode = isObject(existingPolicy) && policyPresets[existingPolicy.mode] ? existingPolicy.mode : null;
  const exampleMode = isObject(examplePolicy) && policyPresets[examplePolicy.mode] ? examplePolicy.mode : null;
  const mode = presetName || configuredMode || exampleMode || 'balanced';
  const base = mergeObject(examplePolicy || {}, policyPresets[mode]);
  const merged = mergeObject(base, existingPolicy || {});
  if (presetName) merged.mode = presetName;
  if (!Array.isArray(merged.disabledChecks)) merged.disabledChecks = [];
  return merged;
}

function mergeConfig(base, existing, inferred, presetName, autoSelection) {
  const merged = Object.assign({}, base, existing || {});
  merged.project = Object.assign({}, base.project, existing && existing.project, inferred.project);
  merged.commands = Object.assign({}, base.commands, existing && existing.commands);
  for (const [key, value] of Object.entries(inferred.commands)) {
    if (!merged.commands[key] || /\|/.test(merged.commands[key])) merged.commands[key] = value;
  }
  merged.policy = policyWithPreset(base.policy, existing && existing.policy, presetName);
  merged.autoRisk = Object.assign({}, base.autoRisk, existing && existing.autoRisk, {
    enabled: true,
    configInit: {
      presetSource: autoSelection && autoSelection.explicit ? 'explicit' : 'auto',
      selectedPreset: presetName,
      recommendedPreset: autoSelection && autoSelection.recommendedPreset,
      rationale: autoSelection && autoSelection.rationale
    }
  });
  return merged;
}

function backupPath(targetRoot) {
  const stamp = new Date().toISOString().replace(/[^0-9A-Za-z]+/g, '-').replace(/-+$/g, '');
  return path.join(targetRoot, '.harness', `harness.config.backup.${stamp}.json`);
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      output: path.join(args.target, 'harness.config.json'),
      backup: null,
      preset: args.preset,
      autoSelection: null,
      inferred: null,
      config: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const examplePath = path.join(args.target, 'harness.config.example.json');
  if (!fs.existsSync(examplePath)) {
    add(report, 'fail', 'harness.config.example.json not found in target');
    return report;
  }

  const example = readJson(examplePath);
  if (example.error) {
    add(report, 'fail', `harness.config.example.json could not be parsed: ${example.error.message}`);
    return report;
  }

  const packagePath = path.join(args.target, 'package.json');
  const pkg = fs.existsSync(packagePath) ? readJson(packagePath) : { data: null, error: null };
  if (pkg.error) add(report, 'warn', `package.json could not be parsed: ${pkg.error.message}`);

  const pm = detectPackageManager(args.target, pkg.data);
  const inferred = {
    project: {
      name: pkg.data && pkg.data.name ? pkg.data.name : path.basename(args.target),
      type: projectType(args.target, pkg.data),
      description: pkg.data && pkg.data.description ? pkg.data.description : `Harness config for ${path.basename(args.target)}.`
    },
    commands: {
      packageManager: pm
    }
  };

  for (const key of ['install', 'lint', 'typecheck', 'test', 'testUnit', 'testIntegration', 'testE2E', 'build', 'dev']) {
    if (key === 'install') inferred.commands[key] = pm === 'npm' ? 'npm install' : `${pm} install`;
    else inferred.commands[key] = runCommand(pm, commandFromScripts(pkg.data, key));
  }

  const existingPath = path.join(args.target, 'harness.config.json');
  const existing = fs.existsSync(existingPath) ? readJson(existingPath) : { data: null, error: null };
  if (existing.error) {
    add(report, 'fail', `Existing harness.config.json could not be parsed: ${existing.error.message}`);
    return report;
  }

  const recommendation = projectPresetRecommendation(args.target);
  const explicitPreset = args.preset !== 'auto';
  const selectedPreset = explicitPreset ? args.preset : recommendation.preset;
  report.details.autoSelection = {
    explicit: explicitPreset,
    recommendedPreset: recommendation.preset,
    selectedPreset,
    score: recommendation.score,
    rationale: recommendation.rationale,
    signals: recommendation.profile.signals
  };
  report.details.inferred = inferred;
  report.details.config = mergeConfig(example.data, existing.data, inferred, selectedPreset, report.details.autoSelection);

  if (existing.data) {
    report.details.backup = backupPath(args.target);
    add(report, 'warn', 'Existing harness.config.json will be merged and backed up before write');
  } else {
    add(report, 'pass', 'harness.config.json does not exist; ready to create');
  }
  if (explicitPreset) add(report, 'pass', `Policy preset selected explicitly: ${selectedPreset}`);
  else add(report, 'pass', `Policy preset auto-selected: ${selectedPreset}`);

  if (!args.write) {
    add(report, 'warn', 'Dry run only; no config file written');
    return report;
  }

  if (existing.data) {
    fs.mkdirSync(path.dirname(report.details.backup), { recursive: true });
    fs.copyFileSync(existingPath, report.details.backup, fs.constants.COPYFILE_EXCL);
  }
  fs.writeFileSync(existingPath, `${JSON.stringify(report.details.config, null, 2)}\n`, 'utf8');
  add(report, 'pass', `Wrote ${path.relative(args.target, existingPath)}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness config init: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
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
