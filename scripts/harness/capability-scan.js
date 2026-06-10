#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const ignoredNames = new Set(['.DS_Store', 'node_modules', '.git', 'dist', 'build', '.next', 'coverage']);

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
  return args;
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    return null;
  }
}

function commandExists(command) {
  const result = spawnSync('sh', ['-lc', `command -v ${shellQuote(command)}`], {
    encoding: 'utf8'
  });
  return result.status === 0;
}

function shellQuote(value) {
  return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function walkLimited(dir, maxDepth, files = [], depth = 0) {
  if (!fs.existsSync(dir) || depth > maxDepth) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walkLimited(full, maxDepth, files, depth + 1);
    else files.push(full);
  }
  return files;
}

function rel(root, filePath) {
  return path.relative(root, filePath) || '.';
}

function detectPackageManager(targetRoot, pkg) {
  const locks = [
    ['pnpm', 'pnpm-lock.yaml'],
    ['npm', 'package-lock.json'],
    ['yarn', 'yarn.lock'],
    ['bun', 'bun.lockb']
  ].filter(([, file]) => exists(targetRoot, file));

  if (pkg && pkg.packageManager) {
    const name = String(pkg.packageManager).split('@')[0];
    return { detected: name, reason: `packageManager field: ${pkg.packageManager}`, locks };
  }
  if (locks.length > 0) return { detected: locks[0][0], reason: `lockfile: ${locks[0][1]}`, locks };
  return { detected: null, reason: 'No package manager field or lockfile found', locks };
}

function detectScripts(pkg) {
  const scripts = pkg && pkg.scripts ? pkg.scripts : {};
  const names = Object.keys(scripts);
  const byCapability = {
    lint: names.filter((name) => /(^|:)lint($|:)/.test(name)),
    typecheck: names.filter((name) => /type-?check|tsc/.test(name)),
    test: names.filter((name) => /(^|:)test($|:)|spec|unit|integration/.test(name)),
    e2e: names.filter((name) => /e2e|playwright|cypress/.test(name)),
    build: names.filter((name) => /(^|:)build($|:)/.test(name)),
    dev: names.filter((name) => /(^|:)dev($|:)|start/.test(name))
  };
  return { scripts, byCapability };
}

function detectFiles(targetRoot) {
  const files = walkLimited(targetRoot, 4);
  const has = (patterns) => files.filter((file) => patterns.some((pattern) => pattern.test(rel(targetRoot, file))));
  const hasExact = (names) => names.filter((name) => exists(targetRoot, name)).map((name) => path.join(targetRoot, name));
  return {
    testFiles: has([/(\.|\/)(test|spec)\.[jt]sx?$/, /__tests__\//]),
    e2eFiles: has([/e2e\//, /playwright\.config\.[cm]?[jt]s$/, /cypress\.config\.[cm]?[jt]s$/]),
    tsConfigs: has([/(^|\/)tsconfig[^/]*\.json$/]),
    envExamples: has([/(^|\/)\.env\.example$/, /(^|\/)\.env\.sample$/]),
    ciFiles: has([/^\.github\/workflows\/.+\.ya?ml$/, /^\.gitlab-ci\.ya?ml$/]),
    githubWorkflows: has([/^\.github\/workflows\/.+\.ya?ml$/]),
    pythonFiles: hasExact(['pyproject.toml', 'requirements.txt', 'pytest.ini', 'setup.py', 'setup.cfg', 'tox.ini']),
    rustFiles: hasExact(['Cargo.toml']),
    goFiles: hasExact(['go.mod']),
    javaFiles: hasExact(['pom.xml', 'build.gradle', 'build.gradle.kts', 'gradlew']),
    makeFiles: hasExact(['Makefile', 'makefile', 'GNUmakefile']),
    dockerFiles: hasExact(['Dockerfile', 'docker-compose.yml', 'docker-compose.yaml', 'compose.yml', 'compose.yaml'])
  };
}

function detectDependencies(pkg) {
  const deps = Object.assign({}, pkg && pkg.dependencies, pkg && pkg.devDependencies, pkg && pkg.optionalDependencies);
  const names = Object.keys(deps);
  const has = (patterns) => names.filter((name) => patterns.some((pattern) => pattern.test(name)));
  return {
    all: names,
    testFrameworks: has([/vitest/, /jest/, /mocha/, /ava/, /node:test/]),
    browserE2E: has([/playwright/, /cypress/, /puppeteer/]),
    uiFrameworks: has([/next/, /vite/, /astro/, /remix/, /react/, /vue/, /svelte/]),
    apiFrameworks: has([/express/, /fastify/, /@nestjs/, /hono/, /koa/]),
    typescript: has([/typescript/])
  };
}

function detectProjectType(targetRoot, pkg, configData, deps, files) {
  if (configData && configData.project && configData.project.type) {
    return { type: configData.project.type, reason: 'harness config project.type' };
  }
  if (exists(targetRoot, 'pnpm-workspace.yaml') || exists(targetRoot, 'turbo.json') || exists(targetRoot, 'nx.json') || exists(targetRoot, 'lerna.json')) {
    return { type: 'monorepo', reason: 'workspace config detected' };
  }
  if (deps.uiFrameworks.length > 0 || files.e2eFiles.length > 0) return { type: 'web-app', reason: 'UI framework or browser test assets detected' };
  if (deps.apiFrameworks.length > 0) return { type: 'api', reason: 'API framework dependency detected' };
  if (pkg) return { type: 'library | app | other', reason: 'package.json found but project type is not explicit' };
  if (files.pythonFiles.length > 0) return { type: 'python', reason: 'Python project files detected' };
  if (files.rustFiles.length > 0) return { type: 'rust', reason: 'Cargo.toml detected' };
  if (files.goFiles.length > 0) return { type: 'go', reason: 'go.mod detected' };
  if (files.javaFiles.length > 0) return { type: 'java', reason: 'Maven or Gradle files detected' };
  if (files.dockerFiles.length > 0) return { type: 'containerized', reason: 'Docker or Compose files detected' };
  return { type: 'other', reason: 'No package.json or harness project type found' };
}

function checkRuleInstall(targetRoot) {
  const required = [
    'AGENTS.md',
    'codex-multi-agent-safe-collaboration.md',
    'skills/using-superpowers/SKILL.md'
  ];
  return required.map((file) => ({
    file,
    exists: exists(targetRoot, file)
  }));
}

function checkRuntimeDocs(targetRoot) {
  const docs = [
    'docs/current-state.md',
    'docs/requirements-matrix.md',
    'docs/task-queue.md',
    'docs/decisions.md',
    'docs/open-questions.md',
    'docs/context-checkpoints.md',
    'docs/sprint-contract.md',
    'docs/agent-mistake-ledger.md',
    'docs/tooling-and-mcp-registry.md',
    'docs/evidence/README.md',
    'docs/plans/README.md'
  ];
  return docs.map((file) => ({
    file,
    exists: exists(targetRoot, file)
  }));
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

function add(report, kind, message) {
  report[kind].push(message);
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

  const packageJsonPath = path.join(args.target, 'package.json');
  const pkg = fs.existsSync(packageJsonPath) ? readJson(packageJsonPath) : null;
  const config = loadConfig(args);
  const packageManager = detectPackageManager(args.target, pkg);
  const scripts = detectScripts(pkg);
  const files = detectFiles(args.target);
  const deps = detectDependencies(pkg);
  const projectType = detectProjectType(args.target, pkg, config.data, deps, files);
  const ruleInstall = checkRuleInstall(args.target);
  const runtimeDocs = checkRuntimeDocs(args.target);

  report.details = {
    configPath: config.path,
    projectType,
    packageManager,
    scriptCapabilities: scripts.byCapability,
    dependencies: {
      testFrameworks: deps.testFrameworks,
      browserE2E: deps.browserE2E,
      uiFrameworks: deps.uiFrameworks,
      apiFrameworks: deps.apiFrameworks,
      typescript: deps.typescript
    },
    ruleInstall,
    runtimeDocs,
    files: Object.fromEntries(Object.entries(files).map(([key, value]) => [key, value.map((file) => rel(args.target, file))])),
    commandsOnPath: {
      node: commandExists('node'),
      git: commandExists('git'),
      pnpm: commandExists('pnpm'),
      npm: commandExists('npm'),
      yarn: commandExists('yarn'),
      bun: commandExists('bun'),
      python: commandExists('python'),
      python3: commandExists('python3'),
      pytest: commandExists('pytest'),
      cargo: commandExists('cargo'),
      go: commandExists('go'),
      mvn: commandExists('mvn'),
      gradle: commandExists('gradle'),
      make: commandExists('make'),
      docker: commandExists('docker')
    }
  };

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config exists but could not be parsed: ${config.path}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  add(report, 'pass', `Project type inference: ${projectType.type} (${projectType.reason})`);

  if (pkg) add(report, 'pass', 'package.json found');
  else add(report, 'warn', 'No package.json found; command detection is limited to files and PATH');

  if (packageManager.detected) add(report, 'pass', `Package manager detected: ${packageManager.detected} (${packageManager.reason})`);
  else add(report, 'warn', packageManager.reason);

  for (const [capability, names] of Object.entries(scripts.byCapability)) {
    if (names.length > 0) add(report, 'pass', `${capability} scripts: ${names.join(', ')}`);
    else add(report, 'warn', `No ${capability} script detected`);
  }

  if (files.testFiles.length > 0) add(report, 'pass', `Test files detected: ${files.testFiles.length}`);
  else add(report, 'warn', 'No test files detected in shallow scan');

  if (files.e2eFiles.length > 0) add(report, 'pass', `E2E/browser test assets detected: ${files.e2eFiles.length}`);
  else add(report, 'warn', 'No E2E/browser test assets detected in shallow scan');

  if (files.ciFiles.length > 0) add(report, 'pass', `CI files detected: ${files.ciFiles.length}`);
  else add(report, 'warn', 'No CI workflow detected');

  if (files.githubWorkflows.length > 0) add(report, 'pass', `GitHub Actions workflows detected: ${files.githubWorkflows.length}`);
  if (files.pythonFiles.length > 0) add(report, 'pass', `Python project files detected: ${files.pythonFiles.map((file) => rel(args.target, file)).join(', ')}`);
  if (files.rustFiles.length > 0) add(report, 'pass', `Rust project files detected: ${files.rustFiles.map((file) => rel(args.target, file)).join(', ')}`);
  if (files.goFiles.length > 0) add(report, 'pass', `Go project files detected: ${files.goFiles.map((file) => rel(args.target, file)).join(', ')}`);
  if (files.javaFiles.length > 0) add(report, 'pass', `Java/Maven/Gradle project files detected: ${files.javaFiles.map((file) => rel(args.target, file)).join(', ')}`);
  if (files.makeFiles.length > 0) add(report, 'pass', `Makefile entrypoints detected: ${files.makeFiles.map((file) => rel(args.target, file)).join(', ')}`);
  if (files.dockerFiles.length > 0) add(report, 'pass', `Docker/Compose files detected: ${files.dockerFiles.map((file) => rel(args.target, file)).join(', ')}`);

  for (const item of ruleInstall) {
    if (item.exists) add(report, 'pass', `Harness rule file found: ${item.file}`);
    else add(report, args.strict ? 'fail' : 'warn', `Harness rule file missing: ${item.file}`);
  }

  const presentRuntimeDocs = runtimeDocs.filter((item) => item.exists);
  if (presentRuntimeDocs.length > 0) add(report, 'pass', `Runtime docs present: ${presentRuntimeDocs.length}/${runtimeDocs.length}`);
  else add(report, 'warn', 'No installed-project runtime docs detected');

  if (deps.testFrameworks.length > 0) add(report, 'pass', `Test framework dependencies: ${deps.testFrameworks.join(', ')}`);
  if (deps.browserE2E.length > 0) add(report, 'pass', `Browser/E2E dependencies: ${deps.browserE2E.join(', ')}`);
  if (deps.uiFrameworks.length > 0) add(report, 'pass', `UI framework dependencies: ${deps.uiFrameworks.join(', ')}`);
  if (deps.apiFrameworks.length > 0) add(report, 'pass', `API framework dependencies: ${deps.apiFrameworks.join(', ')}`);

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
  console.log(`Harness capability scan: ${report.target}`);
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
