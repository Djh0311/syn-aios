#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

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

function shellQuote(value) {
  return `'${String(value).replace(/'/g, "'\\''")}'`;
}

function commandExists(command) {
  const result = spawnSync('sh', ['-lc', `command -v ${shellQuote(command)}`], {
    encoding: 'utf8'
  });
  return result.status === 0;
}

function commandOutput(command) {
  const result = spawnSync('sh', ['-lc', command], {
    encoding: 'utf8',
    timeout: 5000
  });
  return {
    status: result.status,
    stdout: result.stdout ? result.stdout.trim() : '',
    stderr: result.stderr ? result.stderr.trim() : ''
  };
}

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    return null;
  }
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

  const config = loadConfig(args);
  const tools = config.data && config.data.tools ? config.data.tools : {};
  const ui = config.data && config.data.ui ? config.data.ui : {};
  const gates = config.data && config.data.gates ? config.data.gates : {};
  const requiredMcp = tools.mcp && Array.isArray(tools.mcp.required) ? tools.mcp.required : [];
  const preferredMcp = tools.mcp && Array.isArray(tools.mcp.preferred) ? tools.mcp.preferred : [];
  const optionalMcp = tools.mcp && Array.isArray(tools.mcp.optional) ? tools.mcp.optional : [];
  const browser = tools.browser || {};
  const browserFallbacks = Array.isArray(browser.fallbacks) ? browser.fallbacks : [];
  const uiTargets = Array.isArray(ui.targets) ? ui.targets : [];
  const hardGates = Array.isArray(gates.hard) ? gates.hard : [];

  const commands = {
    node: commandExists('node'),
    npx: commandExists('npx'),
    pnpm: commandExists('pnpm'),
    codex: commandExists('codex'),
    playwright: commandExists('playwright')
  };

  const packageJson = path.join(args.target, 'package.json');
  const pkg = fs.existsSync(packageJson) ? readJson(packageJson) : null;
  const packageScripts = pkg && pkg.scripts ? Object.keys(pkg.scripts) : [];
  const deps = Object.assign({}, pkg && pkg.dependencies, pkg && pkg.devDependencies, pkg && pkg.optionalDependencies);
  const dependencyNames = Object.keys(deps);
  const hasPlaywrightDep = dependencyNames.some((name) => /playwright/.test(name));
  const hasChromeDevtoolsDep = dependencyNames.some((name) => /chrome-devtools|devtools-mcp/.test(name));
  const hasPlaywrightConfig = ['playwright.config.js', 'playwright.config.cjs', 'playwright.config.mjs', 'playwright.config.ts'].some((file) => fs.existsSync(path.join(args.target, file)));
  const e2eScripts = packageScripts.filter((name) => /e2e|playwright|browser/.test(name));

  const codexMcpList = commands.codex ? commandOutput('codex mcp list') : null;
  const codexMcpText = codexMcpList ? `${codexMcpList.stdout}\n${codexMcpList.stderr}` : '';
  const chromeDevtoolsListed = /chrome[-_ ]?devtools/i.test(codexMcpText);

  report.details = {
    configPath: config.path,
    configuredTools: {
      browser,
      requiredMcp,
      preferredMcp,
      optionalMcp
    },
    commands,
    dependencies: {
      hasPlaywrightDep,
      hasChromeDevtoolsDep
    },
    playwright: {
      hasConfig: hasPlaywrightConfig,
      e2eScripts
    },
    codexMcpList: codexMcpList
      ? {
          status: codexMcpList.status,
          observedChromeDevTools: chromeDevtoolsListed
        }
      : null
  };

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${path.relative(args.target, config.path) || config.path}`);
  else if (config.path) add(report, 'fail', `Harness config exists but could not be parsed: ${config.path}`);
  else add(report, 'warn', 'No harness config found; using conservative MCP checks');

  if (preferredMcp.length > 0) add(report, 'pass', `Preferred MCP tools configured: ${preferredMcp.join(', ')}`);
  else add(report, 'warn', 'No preferred MCP tools configured');

  if (requiredMcp.length > 0) add(report, 'pass', `Required MCP tools configured: ${requiredMcp.join(', ')}`);
  else add(report, 'warn', 'No required MCP tools configured');

  if (browser.preferred) add(report, 'pass', `Preferred browser harness: ${browser.preferred}`);
  else add(report, 'warn', 'No preferred browser harness configured');

  if (browserFallbacks.length > 0) add(report, 'pass', `Browser fallback tools configured: ${browserFallbacks.join(', ')}`);
  else add(report, 'warn', 'No browser fallback tools configured');

  for (const target of uiTargets) {
    const missing = [];
    if (!target.name) missing.push('name');
    if (!target.url) missing.push('url');
    if (!Array.isArray(target.viewports) || target.viewports.length === 0) missing.push('viewports');
    if (!Array.isArray(target.requiredEvidence) || target.requiredEvidence.length === 0) missing.push('requiredEvidence');
    if (missing.length === 0) add(report, 'pass', `UI target evidence fields complete: ${target.name}`);
    else add(report, args.strict ? 'fail' : 'warn', `UI target is missing fields (${missing.join(', ')}): ${target.name || '<unnamed>'}`);
  }

  if (hardGates.includes('browser-evidence-for-ui-completion') && !browser.preferred && browserFallbacks.length === 0) {
    add(report, 'fail', 'browser-evidence-for-ui-completion gate is enabled but no browser harness or fallback is configured');
  }

  if (commands.codex && codexMcpList && codexMcpList.status === 0) {
    add(report, 'pass', 'codex mcp list executed successfully');
    if (chromeDevtoolsListed) add(report, 'pass', 'Chrome DevTools MCP appears in codex mcp list output');
    else add(report, 'warn', 'Chrome DevTools MCP was not observed in codex mcp list output');
  } else if (commands.codex) {
    add(report, 'warn', 'codex command exists, but codex mcp list did not complete successfully');
  } else {
    add(report, 'warn', 'codex command not found on PATH; cannot inspect Codex MCP registration');
  }

  if (hasPlaywrightDep || commands.playwright || hasPlaywrightConfig || e2eScripts.length > 0) add(report, 'pass', 'Playwright/project E2E fallback has at least one detection signal');
  else add(report, 'warn', 'Playwright fallback not detected');

  if (commands.npx) add(report, 'pass', 'npx available for package-provided browser tools');
  else add(report, 'warn', 'npx not found; package fallback discovery may be limited');

  const requiredText = requiredMcp.join(' ');
  if (/chrome[-_ ]?devtools/i.test(requiredText) && !chromeDevtoolsListed) {
    add(report, args.strict ? 'fail' : 'warn', 'Chrome DevTools MCP is required by config but not confirmed by local inspection');
  }

  if (/chrome[-_ ]?devtools/i.test(String(browser.preferred || '')) && !chromeDevtoolsListed) {
    add(report, 'warn', 'Chrome DevTools MCP is preferred for browser checks but not confirmed; use fallback and report the gap if needed');
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

function printReport(report) {
  console.log(`Harness MCP doctor: ${report.target}`);
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
