#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const commandKeys = [
  'lint',
  'typecheck',
  'test',
  'testUnit',
  'testIntegration',
  'testE2E',
  'e2e',
  'build',
  'dev'
];

const meaningfulKeys = new Set(['lint', 'typecheck', 'test', 'testUnit', 'testIntegration', 'testE2E', 'e2e', 'build']);
const strictMeaningfulKeys = new Set(['lint', 'typecheck', 'test', 'build']);
const packageManagers = new Set(['pnpm', 'npm', 'yarn', 'bun']);

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

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function fileText(targetRoot, relativePath) {
  try {
    return fs.readFileSync(path.join(targetRoot, relativePath), 'utf8');
  } catch (error) {
    return '';
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

function loadPackage(targetRoot) {
  const packagePath = path.join(targetRoot, 'package.json');
  if (!fs.existsSync(packagePath)) return { path: packagePath, data: null, error: null, exists: false };

  const parsed = readJson(packagePath);
  return {
    path: packagePath,
    data: parsed.data,
    error: parsed.error ? parsed.error.message : null,
    exists: true
  };
}

function isConcretePackageManager(value) {
  return packageManagers.has(String(value || '').trim());
}

function packageManagerFromPackageField(pkg) {
  if (!pkg || typeof pkg.packageManager !== 'string') return null;
  const name = pkg.packageManager.trim().split('@')[0];
  return isConcretePackageManager(name) ? name : null;
}

function detectPackageManager(targetRoot, configData, pkg) {
  const configured = configData && configData.commands ? configData.commands.packageManager : null;
  if (isConcretePackageManager(configured)) {
    return { name: configured.trim(), source: 'config.commands.packageManager', evidence: configured };
  }

  const packageField = packageManagerFromPackageField(pkg);
  if (packageField) {
    return { name: packageField, source: 'package.json packageManager', evidence: pkg.packageManager };
  }

  const locks = [
    ['pnpm', 'pnpm-lock.yaml'],
    ['npm', 'package-lock.json'],
    ['yarn', 'yarn.lock'],
    ['bun', 'bun.lockb'],
    ['bun', 'bun.lock']
  ];
  for (const [name, lockfile] of locks) {
    if (exists(targetRoot, lockfile)) return { name, source: 'lockfile', evidence: lockfile };
  }

  return { name: 'npm', source: 'fallback', evidence: 'No configured package manager, packageManager field, or lockfile found' };
}

function scriptCommand(packageManager, scriptName) {
  if (packageManager === 'yarn') return `yarn ${scriptName}`;
  if (packageManager === 'bun') return `bun run ${scriptName}`;
  return `${packageManager} run ${scriptName}`;
}

function normalizeCommand(value) {
  if (typeof value !== 'string') return '';
  return value.trim();
}

function configCommand(configData, key) {
  if (!configData || !configData.commands) return '';
  return normalizeCommand(configData.commands[key]);
}

function scriptNamesForKey(key) {
  if (key === 'testUnit') return ['test:unit', 'unit'];
  if (key === 'testIntegration') return ['test:integration', 'integration'];
  if (key === 'testE2E' || key === 'e2e') return ['test:e2e', 'e2e'];
  if (key === 'typecheck') return ['typecheck', 'type-check', 'tsc'];
  return [key];
}

function findPackageScript(scripts, key) {
  for (const name of scriptNamesForKey(key)) {
    if (Object.prototype.hasOwnProperty.call(scripts, name)) return name;
  }
  return null;
}

function candidateForKey(key, configData, pkg, packageManager) {
  const scripts = pkg && pkg.scripts && typeof pkg.scripts === 'object' ? pkg.scripts : {};
  const configKey = key === 'e2e' && !configCommand(configData, 'e2e') ? 'testE2E' : key;
  const configuredCommand = configCommand(configData, configKey);

  if (configuredCommand) {
    return {
      key,
      command: configuredCommand,
      status: 'configured',
      source: `harness config commands.${configKey}`,
      runnable: key !== 'dev'
    };
  }

  const scriptName = findPackageScript(scripts, key);
  if (scriptName) {
    return {
      key,
      command: scriptCommand(packageManager, scriptName),
      status: 'existing',
      source: `package.json scripts.${scriptName}`,
      runnable: key !== 'dev'
    };
  }

  return {
    key,
    command: null,
    status: 'missing',
    source: null,
    runnable: false
  };
}

function inferredCandidate(key, command, source) {
  return {
    key,
    command,
    status: 'inferred',
    source,
    runnable: true
  };
}

function hasAny(targetRoot, relativePaths) {
  return relativePaths.some((relativePath) => exists(targetRoot, relativePath));
}

function hasMakeTarget(targetRoot, targetName) {
  const makefile = ['Makefile', 'makefile', 'GNUmakefile'].find((name) => exists(targetRoot, name));
  if (!makefile) return false;
  const text = fileText(targetRoot, makefile);
  const pattern = new RegExp(`^${targetName}\\s*:`, 'm');
  return pattern.test(text);
}

function inferCandidatesForKey(targetRoot, key) {
  const candidates = [];
  const hasPython = hasAny(targetRoot, ['pyproject.toml', 'requirements.txt', 'pytest.ini']);
  const hasPytestConfig = hasAny(targetRoot, ['pytest.ini']) || /\[tool\.pytest/.test(fileText(targetRoot, 'pyproject.toml'));
  const hasRust = exists(targetRoot, 'Cargo.toml');
  const hasGo = exists(targetRoot, 'go.mod');
  const hasMaven = exists(targetRoot, 'pom.xml');
  const hasGradle = hasAny(targetRoot, ['build.gradle', 'build.gradle.kts', 'gradlew']);
  const gradleCommand = exists(targetRoot, 'gradlew') ? './gradlew' : 'gradle';
  const composeFile = ['docker-compose.yml', 'docker-compose.yaml', 'compose.yml', 'compose.yaml'].find((name) => exists(targetRoot, name));

  if (key === 'test') {
    if (hasPython || hasPytestConfig) candidates.push(inferredCandidate(key, 'pytest', 'Python project files'));
    if (hasRust) candidates.push(inferredCandidate(key, 'cargo test', 'Cargo.toml'));
    if (hasGo) candidates.push(inferredCandidate(key, 'go test ./...', 'go.mod'));
    if (hasMaven) candidates.push(inferredCandidate(key, 'mvn test', 'pom.xml'));
    if (hasGradle) candidates.push(inferredCandidate(key, `${gradleCommand} test`, 'Gradle build file'));
    if (hasMakeTarget(targetRoot, 'test')) candidates.push(inferredCandidate(key, 'make test', 'Makefile target'));
  }

  if (key === 'lint' && hasMakeTarget(targetRoot, 'lint')) {
    candidates.push(inferredCandidate(key, 'make lint', 'Makefile target'));
  }

  if (key === 'typecheck' && hasRust) {
    candidates.push(inferredCandidate(key, 'cargo check', 'Cargo.toml'));
  }

  if (key === 'build') {
    if (hasRust) candidates.push(inferredCandidate(key, 'cargo build', 'Cargo.toml'));
    if (hasMaven) candidates.push(inferredCandidate(key, 'mvn package', 'pom.xml'));
    if (hasGradle) candidates.push(inferredCandidate(key, `${gradleCommand} build`, 'Gradle build file'));
    if (hasMakeTarget(targetRoot, 'build')) candidates.push(inferredCandidate(key, 'make build', 'Makefile target'));
  }

  if (key === 'composeConfig' && composeFile) {
    candidates.push(inferredCandidate(key, 'docker compose config', composeFile));
  }

  return candidates;
}

function inferProjectEntryPoints(targetRoot) {
  return [
    'package.json',
    'pyproject.toml',
    'requirements.txt',
    'pytest.ini',
    'Cargo.toml',
    'go.mod',
    'pom.xml',
    'build.gradle',
    'build.gradle.kts',
    'gradlew',
    'Makefile',
    'makefile',
    'GNUmakefile',
    'docker-compose.yml',
    'docker-compose.yaml',
    'compose.yml',
    'compose.yaml'
  ].filter((relativePath) => exists(targetRoot, relativePath));
}

function expandInferredCandidates(targetRoot, candidates) {
  const expanded = candidates.slice();
  for (let i = 0; i < expanded.length; i += 1) {
    const candidate = expanded[i];
    if (candidate.command) continue;
    const inferred = inferCandidatesForKey(targetRoot, candidate.key);
    if (inferred.length > 0) expanded[i] = inferred[0];
    if (inferred.length > 1) expanded.push(...inferred.slice(1));
  }
  expanded.push(...inferCandidatesForKey(targetRoot, 'composeConfig'));
  return expanded;
}

function dedupeCandidates(candidates) {
  const seen = new Set();
  return candidates.map((candidate) => {
    if (!candidate.command) return candidate;
    const identity = candidate.command;
    if (seen.has(identity)) {
      return Object.assign({}, candidate, {
        duplicate: true,
        runnable: false
      });
    }
    seen.add(identity);
    return candidate;
  });
}

function isMeaningful(candidate) {
  return meaningfulKeys.has(candidate.key) && candidate.command && !candidate.duplicate;
}

function isStrictMeaningful(candidate) {
  return strictMeaningfulKeys.has(candidate.key) && candidate.command && !candidate.duplicate;
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
      packageJsonPath: null,
      packageManager: null,
      projectEntryPoints: [],
      candidates: []
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
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
    return report;
  }
  if (config.path) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  const pkg = loadPackage(args.target);
  report.details.packageJsonPath = pkg.exists ? rel(args.target, pkg.path) : null;
  if (pkg.error) {
    add(report, 'fail', `package.json could not be parsed: ${rel(args.target, pkg.path)} (${pkg.error})`);
    return report;
  }
  if (pkg.exists) add(report, 'pass', 'package.json readable');
  else add(report, 'warn', 'No package.json found; planning will use harness config and common project entrypoint inference');

  const packageManager = detectPackageManager(args.target, config.data, pkg.data);
  report.details.packageManager = packageManager;
  add(report, 'pass', `Package manager selected: ${packageManager.name} (${packageManager.source})`);

  report.details.projectEntryPoints = inferProjectEntryPoints(args.target);
  if (report.details.projectEntryPoints.length > 0) {
    add(report, 'pass', `Project verification entrypoints detected: ${report.details.projectEntryPoints.join(', ')}`);
  } else {
    add(report, 'warn', 'No common project verification entrypoints detected');
  }

  report.details.candidates = dedupeCandidates(expandInferredCandidates(args.target, commandKeys.map((key) => (
    candidateForKey(key, config.data, pkg.data, packageManager.name)
  ))));

  const meaningful = report.details.candidates.filter(isMeaningful);
  const strictMeaningful = report.details.candidates.filter(isStrictMeaningful);
  const anyRunnableCandidate = report.details.candidates.some((candidate) => (
    candidate.command && !candidate.duplicate && candidate.runnable
  ));
  const dev = report.details.candidates.find((candidate) => candidate.key === 'dev' && candidate.command);

  if (meaningful.length > 0) {
    add(report, 'pass', `Verification command candidates found: ${meaningful.map((candidate) => candidate.key).join(', ')}`);
  } else {
    add(report, 'warn', 'No meaningful verification command candidates found');
  }

  if (dev) add(report, 'warn', 'Dev command is listed for convenience and marked runnable=false by default');

  for (const candidate of report.details.candidates) {
    if (candidate.status === 'missing') {
      add(report, 'warn', `Missing command candidate: ${candidate.key}`);
    } else if (candidate.duplicate) {
      add(report, 'warn', `Duplicate command candidate suppressed: ${candidate.key} -> ${candidate.command}`);
    } else {
      add(report, 'pass', `${candidate.key}: ${candidate.status} (${candidate.command})`);
    }
  }

  if (args.strict && report.details.projectEntryPoints.length > 0 && strictMeaningful.length === 0 && !anyRunnableCandidate) {
    add(report, 'fail', 'Strict mode found project verification entrypoints but no runnable command candidates');
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

function printDetails(report) {
  console.log('\nDETAILS');
  console.log(`Config: ${report.details.configPath || 'None'}`);
  console.log(`package.json: ${report.details.packageJsonPath || 'None'}`);
  console.log(`Project entrypoints: ${report.details.projectEntryPoints.length > 0 ? report.details.projectEntryPoints.join(', ') : 'None'}`);
  if (report.details.packageManager) {
    console.log(`Package manager: ${report.details.packageManager.name} (${report.details.packageManager.source})`);
  }
  console.log('\nCommand menu:');
  for (const candidate of report.details.candidates) {
    const command = candidate.command || 'None';
    const flags = [
      candidate.status,
      `runnable=${candidate.runnable ? 'true' : 'false'}`,
      candidate.duplicate ? 'duplicate=true' : null
    ].filter(Boolean).join(', ');
    console.log(`  - ${candidate.key}: ${command} [${flags}]`);
  }
}

function printReport(report) {
  console.log(`Harness verification plan: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  printDetails(report);
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
