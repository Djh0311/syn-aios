#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const commandNames = ['node', 'npm', 'pnpm', 'yarn', 'bun', 'git', 'codex', 'npx', 'playwright'];
const ignoredNames = new Set(['.DS_Store', '.git', 'node_modules', 'dist', 'build', '.next', 'coverage']);

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

function shellQuote(value) {
  return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function commandPath(command) {
  const result = spawnSync('sh', ['-lc', `command -v ${shellQuote(command)}`], {
    encoding: 'utf8',
    timeout: 3000
  });
  return {
    name: command,
    available: result.status === 0,
    path: result.status === 0 && result.stdout ? result.stdout.trim().split(/\r?\n/)[0] : null
  };
}

function commandOutput(command) {
  const result = spawnSync('sh', ['-lc', command], {
    encoding: 'utf8',
    timeout: 5000
  });
  return {
    status: result.status,
    signal: result.signal || null,
    stdout: result.stdout ? result.stdout.trim() : '',
    stderr: result.stderr ? result.stderr.trim() : ''
  };
}

function dependencyNames(pkg) {
  const deps = Object.assign({}, pkg && pkg.dependencies, pkg && pkg.devDependencies, pkg && pkg.optionalDependencies);
  return Object.keys(deps);
}

function scriptNames(pkg) {
  return pkg && pkg.scripts && typeof pkg.scripts === 'object' ? Object.keys(pkg.scripts) : [];
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

function collectCiFiles(targetRoot) {
  const files = [];
  const workflowRoot = path.join(targetRoot, '.github', 'workflows');
  if (fs.existsSync(workflowRoot)) {
    files.push(...walkLimited(workflowRoot, 2).filter((file) => /\.ya?ml$/i.test(file)));
  }
  for (const relativePath of ['.gitlab-ci.yml', '.circleci/config.yml', 'buildkite.yml', '.buildkite/pipeline.yml']) {
    const full = path.join(targetRoot, relativePath);
    if (fs.existsSync(full)) files.push(full);
  }
  return files;
}

function collectEvidenceState(targetRoot) {
  const evidenceRoot = path.join(targetRoot, 'docs', 'evidence');
  const existsOnDisk = fs.existsSync(evidenceRoot) && fs.statSync(evidenceRoot).isDirectory();
  const files = existsOnDisk ? walkLimited(evidenceRoot, 4) : [];
  return {
    root: rel(targetRoot, evidenceRoot),
    exists: existsOnDisk,
    fileCount: files.length,
    summaries: files.filter((file) => path.basename(file) === 'summary.md').map((file) => rel(targetRoot, file))
  };
}

function detectProjectProfile(targetRoot, configData, pkg) {
  const deps = dependencyNames(pkg);
  const scripts = scriptNames(pkg);
  const ecosystems = [];

  if (exists(targetRoot, 'package.json')) ecosystems.push('node');
  if (exists(targetRoot, 'pyproject.toml') || exists(targetRoot, 'requirements.txt') || exists(targetRoot, 'pytest.ini')) ecosystems.push('python');
  if (exists(targetRoot, 'go.mod')) ecosystems.push('go');
  if (exists(targetRoot, 'Cargo.toml')) ecosystems.push('rust');
  if (exists(targetRoot, 'pom.xml') || exists(targetRoot, 'build.gradle') || exists(targetRoot, 'build.gradle.kts') || exists(targetRoot, 'gradlew')) ecosystems.push('java');
  if (exists(targetRoot, 'Dockerfile') || exists(targetRoot, 'docker-compose.yml') || exists(targetRoot, 'compose.yaml')) ecosystems.push('docker');
  if (exists(targetRoot, 'pnpm-workspace.yaml') || exists(targetRoot, 'turbo.json') || exists(targetRoot, 'nx.json') || (pkg && pkg.workspaces)) ecosystems.push('monorepo');
  if (deps.some((name) => /^(next|vite|react|vue|svelte|astro|@remix-run\/)/.test(name)) || exists(targetRoot, 'vite.config.js') || exists(targetRoot, 'next.config.js')) ecosystems.push('frontend');
  if (deps.some((name) => /^(express|fastify|hono|koa|@nestjs\/)/.test(name))) ecosystems.push('backend');

  return {
    configuredType: configData && configData.project ? configData.project.type || null : null,
    ecosystems: Array.from(new Set(ecosystems)),
    packageScripts: scripts,
    ciFiles: collectCiFiles(targetRoot).map((file) => rel(targetRoot, file)),
    evidence: collectEvidenceState(targetRoot)
  };
}

function hasChromeDevToolsName(value) {
  return /chrome[-_ ]?devtools|chrome devtools mcp/i.test(String(value || ''));
}

function detectCodexMcp(commands) {
  if (!commands.codex || !commands.codex.available) {
    return {
      inspected: false,
      status: null,
      chromeDevTools: false,
      entries: [],
      error: 'codex command not found on PATH'
    };
  }

  const result = commandOutput('codex mcp list');
  const text = `${result.stdout}\n${result.stderr}`;
  const entries = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .slice(0, 100);

  return {
    inspected: true,
    status: result.status,
    chromeDevTools: result.status === 0 && hasChromeDevToolsName(text),
    entries,
    error: result.status === 0 ? null : result.stderr || result.stdout || 'codex mcp list did not complete successfully'
  };
}

function configuredTools(configData) {
  const tools = configData && configData.tools && typeof configData.tools === 'object' ? configData.tools : {};
  const browser = tools.browser && typeof tools.browser === 'object' ? tools.browser : {};
  const mcp = tools.mcp && typeof tools.mcp === 'object' ? tools.mcp : {};
  return {
    browserPreferred: browser.preferred || null,
    browserFallbacks: Array.isArray(browser.fallbacks) ? browser.fallbacks : [],
    mcpRequired: Array.isArray(mcp.required) ? mcp.required : [],
    mcpPreferred: Array.isArray(mcp.preferred) ? mcp.preferred : [],
    mcpOptional: Array.isArray(mcp.optional) ? mcp.optional : []
  };
}

function detectBrowserFallback(name, commands, pkg, projectProfile) {
  const deps = dependencyNames(pkg);
  const scripts = scriptNames(pkg);
  const normalized = String(name || '').toLowerCase();

  if (normalized.includes('playwright')) {
    const hasDep = deps.some((dep) => /playwright/.test(dep));
    const hasScript = scripts.some((script) => /e2e|playwright|browser/.test(script));
    const hasConfig = ['playwright.config.js', 'playwright.config.cjs', 'playwright.config.mjs', 'playwright.config.ts'].some((file) => exists(projectProfile.targetRoot, file));
    return {
      name,
      available: commands.playwright.available || commands.npx.available || hasDep || hasScript || hasConfig,
      confidence: commands.playwright.available || hasDep || hasConfig ? 'high' : commands.npx.available || hasScript ? 'medium' : 'low',
      evidence: [
        commands.playwright.available ? 'playwright command on PATH' : null,
        commands.npx.available ? 'npx command on PATH' : null,
        hasDep ? 'package dependency includes playwright' : null,
        hasScript ? 'package script mentions e2e/playwright/browser' : null,
        hasConfig ? 'playwright config file found' : null
      ].filter(Boolean)
    };
  }

  if (normalized.includes('project e2e')) {
    const hasScript = scripts.some((script) => /e2e|browser|cypress|playwright/.test(script));
    return {
      name,
      available: hasScript,
      confidence: hasScript ? 'medium' : 'low',
      evidence: hasScript ? ['package script mentions e2e/browser tooling'] : []
    };
  }

  if (normalized.includes('codex in-app browser')) {
    return {
      name,
      available: true,
      confidence: 'configured',
      evidence: ['configured fallback; availability depends on the active Codex app session']
    };
  }

  if (normalized.includes('manual')) {
    return {
      name,
      available: true,
      confidence: 'procedural',
      evidence: ['manual fallback is procedural, not locally executable']
    };
  }

  return {
    name,
    available: false,
    confidence: 'unknown',
    evidence: []
  };
}

function addTool(list, name, reason, evidence) {
  list.push({ name, reason, evidence: evidence || [] });
}

function addMissing(report, name, severity, reason, optional) {
  report.missing.push({ name, severity, optional: Boolean(optional), reason });
  if (severity === 'FAIL') add(report, 'fail', `${name}: ${reason}`);
  else add(report, 'warn', `${name}: ${reason}`);
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    available: [],
    preferred: [],
    fallback: [],
    missing: [],
    recommendations: [],
    pass: [],
    warn: [],
    fail: [],
    details: {}
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args);
  const pkgResult = packageJson(args.target);
  const pkg = pkgResult.data;
  const commands = Object.fromEntries(commandNames.map((name) => [name, commandPath(name)]));
  const mcp = detectCodexMcp(commands);
  const tools = configuredTools(config.data);
  const profile = detectProjectProfile(args.target, config.data, pkg);
  profile.targetRoot = args.target;

  report.details = {
    configPath: config.path ? rel(args.target, config.path) : null,
    configError: config.error,
    packageJson: {
      exists: pkgResult.exists,
      error: pkgResult.error
    },
    commands,
    mcp,
    configuredTools: tools,
    projectProfile: {
      configuredType: profile.configuredType,
      ecosystems: profile.ecosystems,
      packageScripts: profile.packageScripts,
      ciFiles: profile.ciFiles,
      evidence: profile.evidence
    }
  };

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
  else add(report, 'warn', 'No harness config found; using local detection only');

  if (pkgResult.error) add(report, 'fail', `package.json could not be parsed: ${pkgResult.error}`);

  for (const command of Object.values(commands)) {
    if (command.available) addTool(report.available, command.name, 'command on PATH', [command.path]);
    else addMissing(report, command.name, 'WARN', 'command not found on PATH', true);
  }

  if (profile.ecosystems.length > 0) addTool(report.available, 'project profile', 'local project signals detected', profile.ecosystems);
  if (profile.ciFiles.length > 0) addTool(report.available, 'ci', 'CI configuration detected', profile.ciFiles);
  else addMissing(report, 'ci', 'WARN', 'no CI configuration files detected', true);
  if (profile.evidence.exists) addTool(report.available, 'evidence archive', 'docs/evidence exists', [`${profile.evidence.fileCount} file(s)`]);
  else addMissing(report, 'evidence archive', 'WARN', 'docs/evidence archive not detected', true);

  const chromePreferred = hasChromeDevToolsName(tools.browserPreferred)
    || tools.mcpPreferred.some(hasChromeDevToolsName)
    || tools.mcpRequired.some(hasChromeDevToolsName);

  if (chromePreferred) {
    addTool(report.preferred, 'Chrome DevTools MCP', 'configured as preferred or required browser/MCP tool', [
      tools.browserPreferred ? `browser.preferred=${tools.browserPreferred}` : null,
      tools.mcpPreferred.length > 0 ? `mcp.preferred=${tools.mcpPreferred.join(', ')}` : null,
      tools.mcpRequired.length > 0 ? `mcp.required=${tools.mcpRequired.join(', ')}` : null
    ].filter(Boolean));
  } else if (tools.browserPreferred) {
    addTool(report.preferred, tools.browserPreferred, 'configured browser.preferred', ['harness config']);
  }

  if (mcp.chromeDevTools) {
    addTool(report.available, 'Chrome DevTools MCP', 'observed in codex mcp list output', ['codex mcp list']);
  } else if (chromePreferred) {
    const required = tools.mcpRequired.some(hasChromeDevToolsName);
    addMissing(
      report,
      'Chrome DevTools MCP',
      required && args.strict ? 'FAIL' : 'WARN',
      required
        ? 'required by config but not confirmed by codex mcp list'
        : 'preferred by config but not confirmed by codex mcp list',
      !required
    );
  } else {
    addMissing(report, 'Chrome DevTools MCP', 'WARN', 'not configured or confirmed; browser evidence should use an available fallback', true);
  }

  for (const name of tools.mcpRequired) {
    if (hasChromeDevToolsName(name)) continue;
    addMissing(report, name, args.strict ? 'FAIL' : 'WARN', 'required MCP cannot be confirmed by capability-map local detection', false);
  }

  for (const name of tools.mcpOptional) {
    if (hasChromeDevToolsName(name) && mcp.chromeDevTools) continue;
    addMissing(report, name, 'WARN', 'optional MCP not confirmed by bounded local detection', true);
  }

  const browserFallbacks = tools.browserFallbacks.length > 0
    ? tools.browserFallbacks
    : ['Codex in-app Browser', 'Playwright MCP', 'project E2E runner', 'manual user verification'];
  for (const name of browserFallbacks) {
    const fallback = detectBrowserFallback(name, commands, pkg, profile);
    report.fallback.push(fallback);
    if (fallback.available) add(report, 'pass', `Browser fallback available: ${name}`);
    else add(report, 'warn', `Browser fallback not detected: ${name}`);
  }

  if (mcp.chromeDevTools) {
    report.recommendations.push('Use Chrome DevTools MCP for browser evidence when UI/browser verification is required.');
  } else {
    const availableFallbacks = report.fallback.filter((item) => item.available).map((item) => item.name);
    report.recommendations.push(
      availableFallbacks.length > 0
        ? `Chrome DevTools MCP is not confirmed; use fallback(s): ${availableFallbacks.join(', ')}.`
        : 'Chrome DevTools MCP is not confirmed and no browser fallback was detected; arrange manual browser evidence before UI completion claims.'
    );
  }

  if (!commands.git.available) report.recommendations.push('Git-dependent checks may be unavailable because git is not on PATH.');
  if (profile.ciFiles.length === 0) report.recommendations.push('No CI configuration was detected; treat CI status as unavailable rather than passing.');
  if (!profile.evidence.exists) report.recommendations.push('No docs/evidence archive was detected; durable evidence may need to be created before Strict completion.');

  if (report.fail.length === 0) add(report, 'pass', 'Capability map completed without hard failures');
  return report;
}

function printList(title, items, format) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${format(item)}`);
}

function printReport(report) {
  console.log(`Harness capability map: ${report.target}`);
  if (report.strict) console.log('Mode: strict');
  printList('AVAILABLE', report.available, (item) => `${item.name}: ${item.reason}`);
  printList('PREFERRED', report.preferred, (item) => `${item.name}: ${item.reason}`);
  printList('FALLBACK', report.fallback, (item) => `${item.name}: ${item.available ? 'available' : 'not detected'} (${item.confidence})`);
  printList('MISSING', report.missing, (item) => `${item.severity} ${item.name}: ${item.reason}`);
  printList('RECOMMENDATIONS', report.recommendations, (item) => item);
  printList('PASS', report.pass, (item) => item);
  printList('WARN', report.warn, (item) => item);
  printList('FAIL', report.fail, (item) => item);
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
