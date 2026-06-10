#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

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

function exists(root, relativePath) {
  return fs.existsSync(path.join(root, relativePath));
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

function packageJson(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) return { exists: false, data: null, error: null };
  const parsed = readJson(file);
  return { exists: true, data: parsed.data, error: parsed.error };
}

function evidence(relativePath, reason) {
  return { path: relativePath, reason };
}

function signal(name, confidence, evidenceItems, configured) {
  return {
    name,
    confidence,
    source: configured ? 'configured' : 'inferred',
    evidence: evidenceItems
  };
}

function dependencyNames(pkg) {
  const deps = Object.assign({}, pkg && pkg.dependencies, pkg && pkg.devDependencies, pkg && pkg.optionalDependencies);
  return Object.keys(deps);
}

function packageManagerName(packageManager) {
  return packageManager && packageManager.name ? packageManager.name : 'npm';
}

function scriptCommand(packageManager, scriptName) {
  const manager = packageManagerName(packageManager);
  if (manager === 'yarn') return `yarn ${scriptName}`;
  if (manager === 'bun') return `bun run ${scriptName}`;
  return `${manager} run ${scriptName}`;
}

function detectPackageManager(root, pkg) {
  if (pkg && typeof pkg.packageManager === 'string') {
    return signal(pkg.packageManager.split('@')[0], 'high', [evidence('package.json', 'packageManager field')], false);
  }
  for (const [name, file] of [
    ['pnpm', 'pnpm-lock.yaml'],
    ['yarn', 'yarn.lock'],
    ['bun', 'bun.lockb'],
    ['bun', 'bun.lock'],
    ['npm', 'package-lock.json']
  ]) {
    if (exists(root, file)) return signal(name, 'high', [evidence(file, 'lockfile')], false);
  }
  if (exists(root, 'package.json')) return signal('npm', 'medium', [evidence('package.json', 'Node package without lockfile')], false);
  return null;
}

function detectEcosystems(root, pkg) {
  const deps = dependencyNames(pkg);
  const ecosystems = [];
  if (exists(root, 'package.json')) ecosystems.push(signal('node', 'high', [evidence('package.json', 'Node package manifest')], false));
  if (exists(root, 'pyproject.toml') || exists(root, 'requirements.txt') || exists(root, 'pytest.ini')) {
    ecosystems.push(signal('python', 'high', ['pyproject.toml', 'requirements.txt', 'pytest.ini'].filter((file) => exists(root, file)).map((file) => evidence(file, 'Python project signal')), false));
  }
  if (exists(root, 'go.mod')) ecosystems.push(signal('go', 'high', [evidence('go.mod', 'Go module')], false));
  if (exists(root, 'Cargo.toml')) ecosystems.push(signal('rust', 'high', [evidence('Cargo.toml', 'Cargo manifest')], false));
  if (exists(root, 'pom.xml') || exists(root, 'build.gradle') || exists(root, 'build.gradle.kts') || exists(root, 'gradlew')) {
    ecosystems.push(signal('java', 'high', ['pom.xml', 'build.gradle', 'build.gradle.kts', 'gradlew'].filter((file) => exists(root, file)).map((file) => evidence(file, 'Java build file')), false));
  }
  if (exists(root, 'Dockerfile') || exists(root, 'docker-compose.yml') || exists(root, 'docker-compose.yaml') || exists(root, 'compose.yml') || exists(root, 'compose.yaml')) {
    ecosystems.push(signal('docker', 'medium', ['Dockerfile', 'docker-compose.yml', 'docker-compose.yaml', 'compose.yml', 'compose.yaml'].filter((file) => exists(root, file)).map((file) => evidence(file, 'Container project signal')), false));
  }
  if (exists(root, 'pnpm-workspace.yaml') || exists(root, 'turbo.json') || exists(root, 'nx.json') || exists(root, 'lerna.json') || (pkg && pkg.workspaces)) {
    ecosystems.push(signal('monorepo', 'high', ['pnpm-workspace.yaml', 'turbo.json', 'nx.json', 'lerna.json', 'package.json'].filter((file) => file === 'package.json' ? Boolean(pkg && pkg.workspaces) : exists(root, file)).map((file) => evidence(file, file === 'package.json' ? 'workspaces field' : 'Monorepo signal')), false));
  }
  if (deps.some((name) => /^(next|vite|react|vue|svelte|astro|@remix-run\/)/.test(name)) || exists(root, 'vite.config.js') || exists(root, 'vite.config.ts') || exists(root, 'next.config.js') || exists(root, 'index.html') || exists(root, 'src/App.jsx') || exists(root, 'src/App.tsx')) {
    ecosystems.push(signal('frontend', deps.some((name) => /^(next|vite|react|vue|svelte|astro|@remix-run\/)/.test(name)) ? 'high' : 'medium', [evidence('package.json or frontend files', 'Frontend framework dependency or app files')], false));
  }
  if (deps.some((name) => /^(express|fastify|hono|koa|@nestjs\/)/.test(name)) || exists(root, 'server.js') || exists(root, 'server.ts') || exists(root, 'src/server.js') || exists(root, 'src/server.ts')) {
    ecosystems.push(signal('backend', deps.some((name) => /^(express|fastify|hono|koa|@nestjs\/)/.test(name)) ? 'high' : 'medium', [evidence('package.json or server files', 'API framework dependency or server entrypoint')], false));
  }
  if (pkg && pkg.bin || exists(root, 'bin') || exists(root, 'cli.js') || exists(root, 'src/cli.js') || exists(root, 'src/cli.ts')) ecosystems.push(signal('cli', pkg && pkg.bin ? 'high' : 'medium', [evidence('package.json or CLI files', 'bin field or CLI entrypoint')], false));
  if (pkg && (pkg.main || pkg.module || pkg.types || pkg.exports) || exists(root, 'src/index.js') || exists(root, 'src/index.ts')) ecosystems.push(signal('library', pkg && (pkg.main || pkg.module || pkg.types || pkg.exports) ? 'medium' : 'low', [evidence('package.json or src/index', 'package export fields or index entrypoint')], false));
  return ecosystems;
}

function commandCandidates(root, pkg, configData, packageManager) {
  const scripts = pkg && pkg.scripts && typeof pkg.scripts === 'object' ? pkg.scripts : {};
  const configured = configData && configData.commands && typeof configData.commands === 'object' ? configData.commands : {};
  const keys = ['install', 'lint', 'typecheck', 'test', 'testUnit', 'testIntegration', 'testE2E', 'build', 'dev'];
  const result = [];

  for (const key of keys) {
    const configuredValue = typeof configured[key] === 'string' ? configured[key].trim() : '';
    if (configuredValue && !/\|/.test(configuredValue)) {
      result.push({ key, command: configuredValue, source: 'configured', confidence: 'high', evidence: [evidence('harness.config.json', `commands.${key}`)] });
      continue;
    }
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
    const scriptName = scriptNames.find((name) => Object.prototype.hasOwnProperty.call(scripts, name));
    if (scriptName) result.push({ key, command: scriptCommand(packageManager, scriptName), source: 'inferred', confidence: 'high', evidence: [evidence('package.json', `scripts.${scriptName}`)] });
  }

  if (exists(root, 'pyproject.toml') || exists(root, 'pytest.ini')) result.push({ key: 'test', command: 'pytest', source: 'inferred', confidence: 'medium', evidence: [evidence('pyproject.toml/pytest.ini', 'Python test signal')] });
  if (exists(root, 'go.mod')) result.push({ key: 'test', command: 'go test ./...', source: 'inferred', confidence: 'medium', evidence: [evidence('go.mod', 'Go module')] });
  if (exists(root, 'Cargo.toml')) result.push({ key: 'test', command: 'cargo test', source: 'inferred', confidence: 'medium', evidence: [evidence('Cargo.toml', 'Cargo manifest')] });
  if (exists(root, 'pom.xml')) result.push({ key: 'test', command: 'mvn test', source: 'inferred', confidence: 'medium', evidence: [evidence('pom.xml', 'Maven project signal')] });
  if (exists(root, 'gradlew') || exists(root, 'build.gradle') || exists(root, 'build.gradle.kts')) result.push({ key: 'test', command: exists(root, 'gradlew') ? './gradlew test' : 'gradle test', source: 'inferred', confidence: 'medium', evidence: [evidence('gradle files', 'Gradle project signal')] });
  if (exists(root, 'docker-compose.yml') || exists(root, 'docker-compose.yaml') || exists(root, 'compose.yml') || exists(root, 'compose.yaml')) result.push({ key: 'dockerConfig', command: 'docker compose config', source: 'inferred', confidence: 'medium', evidence: [evidence('compose file', 'Docker Compose signal')] });
  return result;
}

function uiTargets(root, configData, pkg) {
  const targets = [];
  const configured = configData && configData.ui && Array.isArray(configData.ui.targets) ? configData.ui.targets : [];
  for (const target of configured) {
    if (target && target.url) {
      targets.push({ name: target.name || 'configured-ui', url: target.url, source: 'configured', confidence: 'high', evidence: [evidence('harness.config.json', 'ui.targets')] });
    }
  }

  const deps = dependencyNames(pkg);
  if (deps.some((name) => /^vite$|^@vitejs\//.test(name)) || exists(root, 'vite.config.js') || exists(root, 'vite.config.ts')) {
    targets.push({ name: 'vite', url: 'http://localhost:5173', source: 'inferred', confidence: 'medium', evidence: [evidence('package.json/vite.config', 'Vite signal')] });
  }
  if (deps.includes('next') || exists(root, 'next.config.js') || exists(root, 'next.config.mjs')) {
    targets.push({ name: 'next', url: 'http://localhost:3000', source: 'inferred', confidence: 'medium', evidence: [evidence('package.json/next.config', 'Next.js signal')] });
  }
  return targets;
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
      configPath: null,
      packageManager: null,
      ecosystems: [],
      commands: [],
      uiTargets: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) add(report, 'pass', 'Source package detected; profile is informational');
  else if (kind.isInstalledProject) add(report, 'pass', 'Installed-project signals detected');
  else add(report, 'warn', 'Target is not recognized as a harness source package or installed project');

  const config = loadConfig(args);
  report.details.configPath = config.path;
  if (config.error) add(report, 'fail', `Harness config could not be loaded: ${config.error}`);
  else if (config.path) add(report, 'pass', `Harness config readable: ${path.relative(args.target, config.path) || config.path}`);
  else add(report, 'warn', 'No harness config found; using filesystem inference only');

  const pkg = packageJson(args.target);
  if (pkg.error) add(report, 'fail', `package.json could not be parsed: ${pkg.error}`);
  else if (pkg.exists) add(report, 'pass', 'package.json parsed');
  else add(report, 'warn', 'No package.json found');

  report.details.packageManager = detectPackageManager(args.target, pkg.data);
  if (report.details.packageManager) add(report, 'pass', `Package manager candidate: ${report.details.packageManager.name}`);
  else add(report, 'warn', 'No package manager candidate detected');

  report.details.ecosystems = detectEcosystems(args.target, pkg.data);
  if (report.details.ecosystems.length > 0) add(report, 'pass', `Ecosystem signals: ${report.details.ecosystems.map((item) => item.name).join(', ')}`);
  else add(report, 'warn', 'No ecosystem signals detected');

  report.details.commands = commandCandidates(args.target, pkg.data, config.data, report.details.packageManager);
  if (report.details.commands.length > 0) add(report, 'pass', `Command candidates found: ${report.details.commands.length}`);
  else add(report, 'warn', 'No command candidates found');

  report.details.uiTargets = uiTargets(args.target, config.data, pkg.data);
  if (report.details.uiTargets.length > 0) add(report, 'pass', `UI target candidates found: ${report.details.uiTargets.length}`);
  else add(report, 'warn', 'No UI target candidates found');

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness project profile: ${report.target}`);
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
