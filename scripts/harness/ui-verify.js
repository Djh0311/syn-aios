#!/usr/bin/env node

const fs = require('fs');
const http = require('http');
const https = require('https');
const path = require('path');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    targetProvided: false,
    slug: null,
    url: null,
    name: null,
    write: false,
    json: false,
    strict: false,
    timeoutMs: 15000
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') {
      args.target = argv[++i];
      args.targetProvided = true;
    } else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--url') args.url = argv[++i];
    else if (arg === '--name') args.name = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--timeout-ms') args.timeoutMs = Number(argv[++i]);
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.write && (!args.targetProvided || !args.slug)) {
    throw new Error('--write requires explicit --target and --slug');
  }
  if (!Number.isInteger(args.timeoutMs) || args.timeoutMs <= 0) {
    throw new Error('--timeout-ms must be a positive integer');
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error };
  }
}

function loadConfig(targetRoot) {
  for (const name of ['harness.config.json', 'harness.config.example.json']) {
    const file = path.join(targetRoot, name);
    if (!fs.existsSync(file)) continue;
    const parsed = readJson(file);
    return {
      path: file,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null
    };
  }
  return { path: null, data: null, error: null };
}

function safeSlug(value) {
  const slug = String(value || '').trim();
  if (!slug) throw new Error('--slug is required');
  if (/[\\/]/.test(slug) || slug.includes('..')) {
    throw new Error('Slug must be a single safe path segment without traversal');
  }
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(slug)) {
    throw new Error('Slug may contain only letters, numbers, dots, underscores, and hyphens');
  }
  return slug;
}

function targetsFromConfig(configData) {
  const targets = configData && configData.ui && Array.isArray(configData.ui.targets)
    ? configData.ui.targets
    : [];
  return targets
    .filter((target) => target && typeof target.url === 'string' && target.url.trim())
    .map((target, index) => ({
      name: target.name || `ui-target-${index + 1}`,
      url: target.url.trim(),
      viewports: Array.isArray(target.viewports) ? target.viewports : [],
      requiredEvidence: Array.isArray(target.requiredEvidence) ? target.requiredEvidence : []
    }));
}

function normalizeTargets(args, configData) {
  if (args.url) {
    return [{
      name: args.name || 'explicit-url',
      url: args.url,
      viewports: [],
      requiredEvidence: []
    }];
  }
  return targetsFromConfig(configData);
}

function assertHttpUrl(value) {
  const url = new URL(value);
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error(`Only http/https URLs are supported: ${value}`);
  }
  return url;
}

function fetchUrl(target, timeoutMs) {
  return new Promise((resolve) => {
    let url;
    try {
      url = assertHttpUrl(target.url);
    } catch (error) {
      resolve({
        target,
        ok: false,
        error: error.message,
        statusCode: null,
        headers: {},
        bodySample: '',
        durationMs: 0
      });
      return;
    }

    const started = Date.now();
    const client = url.protocol === 'https:' ? https : http;
    const request = client.get(url, { timeout: timeoutMs }, (response) => {
      const chunks = [];
      let size = 0;

      response.on('data', (chunk) => {
        if (size < 2048) chunks.push(chunk);
        size += chunk.length;
      });

      response.on('end', () => {
        const statusCode = response.statusCode || null;
        resolve({
          target,
          ok: Boolean(statusCode && statusCode >= 200 && statusCode < 400),
          error: null,
          statusCode,
          headers: response.headers || {},
          bodySample: Buffer.concat(chunks).toString('utf8').slice(0, 2048),
          durationMs: Date.now() - started
        });
      });
    });

    request.on('timeout', () => {
      request.destroy(new Error(`Timed out after ${timeoutMs}ms`));
    });
    request.on('error', (error) => {
      resolve({
        target,
        ok: false,
        error: error.message,
        statusCode: null,
        headers: {},
        bodySample: '',
        durationMs: Date.now() - started
      });
    });
  });
}

function fence(value) {
  return String(value || '').replace(/```/g, '`` `');
}

function writeEvidence(report) {
  const slug = safeSlug(report.details.slug);
  const evidenceDir = path.join(report.target, 'docs', 'evidence', slug);
  const browserFile = path.join(evidenceDir, 'browser-check.md');
  const consoleFile = path.join(evidenceDir, 'console-network.md');

  if (!fs.existsSync(evidenceDir) || !fs.statSync(evidenceDir).isDirectory()) {
    throw new Error(`Evidence directory must already exist: docs/evidence/${slug}/`);
  }

  const browserRelative = rel(report.target, browserFile);
  const consoleRelative = rel(report.target, consoleFile);
  if (browserRelative !== `docs/evidence/${slug}/browser-check.md`) {
    throw new Error(`Refusing to write outside docs/evidence/${slug}/browser-check.md`);
  }
  if (consoleRelative !== `docs/evidence/${slug}/console-network.md`) {
    throw new Error(`Refusing to write outside docs/evidence/${slug}/console-network.md`);
  }

  const now = new Date().toISOString();
  const rows = report.details.results.map((result) => (
    `| ${result.target.name} | ${result.target.url} | ${result.statusCode || 'none'} | ${result.ok ? 'pass' : 'fail'} | ${result.durationMs} | ${result.error || ''} |`
  )).join('\n');
  const samples = report.details.results.map((result) => `### ${result.target.name}

- URL: ${result.target.url}
- Status: ${result.statusCode || 'none'}
- Error: ${result.error || 'none'}

\`\`\`text
${fence(result.bodySample)}
\`\`\`
`).join('\n');

  fs.writeFileSync(browserFile, `# Browser Check

- recordedAt: ${now}
- mode: HTTP-only reachability check

| Target | URL | Status | Result | Duration ms | Error |
| --- | --- | --- | --- | --- | --- |
${rows}

## Body Samples

${samples}
`, 'utf8');

  fs.writeFileSync(consoleFile, `# Console And Network Check

- recordedAt: ${now}
- mode: HTTP-only network evidence

This file records HTTP reachability and response metadata only. It cannot inspect browser console messages, client-side runtime errors, hydration behavior, or in-browser network waterfalls. Use Chrome DevTools MCP or the Codex in-app Browser for real browser console and interaction evidence before making UI completion claims.

## Requests

| Target | URL | Status | Result | Error |
| --- | --- | --- | --- | --- |
${report.details.results.map((result) => `| ${result.target.name} | ${result.target.url} | ${result.statusCode || 'none'} | ${result.ok ? 'pass' : 'fail'} | ${result.error || ''} |`).join('\n')}
`, 'utf8');

  report.details.browserFile = browserRelative;
  report.details.consoleNetworkFile = consoleRelative;
  report.wrote = true;
}

async function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    strict: args.strict,
    wrote: false,
    pass: [],
    warn: [],
    fail: [],
    details: {
      slug: args.slug || null,
      configPath: null,
      targets: [],
      results: [],
      browserFile: null,
      consoleNetworkFile: null
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args.target);
  report.details.configPath = config.path ? rel(args.target, config.path) : null;
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.error}`);
    return report;
  }

  const targets = normalizeTargets(args, config.data);
  report.details.targets = targets;
  if (targets.length === 0) {
    add(report, args.strict ? 'fail' : 'warn', 'No UI targets configured or provided');
    return report;
  }

  if (!args.write) {
    add(report, 'warn', `PLAN ONLY: dry run did not make HTTP requests or write browser evidence; would check ${targets.length} UI target(s)`);
    return report;
  }

  report.details.results = await Promise.all(targets.map((target) => fetchUrl(target, args.timeoutMs)));
  const failed = report.details.results.filter((result) => !result.ok);
  if (failed.length === 0) add(report, 'pass', `UI HTTP checks passed: ${targets.length}/${targets.length}`);
  else add(report, args.strict ? 'fail' : 'warn', `UI HTTP checks failed: ${failed.length}/${targets.length}`);

  try {
    writeEvidence(report);
    add(report, 'pass', 'UI evidence files written');
  } catch (error) {
    add(report, 'fail', error.message);
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
  console.log(`Harness UI verify: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log('\nDETAILS');
  console.log(JSON.stringify(report.details, null, 2));
}

(async () => {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = await buildReport(args);
    if (args.json) console.log(JSON.stringify(report, null, 2));
    else printReport(report);
    if (report.fail.length > 0) process.exit(1);
  } catch (error) {
    console.error(`ERROR: ${error.message}`);
    process.exit(1);
  }
})();
