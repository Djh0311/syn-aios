const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detectProjectKind } = require('./project-kind');

const targetOptionScripts = new Set([
  'capability-scan.js',
  'browser-evidence-check.js',
  'ci-gate.js',
  'ci-init.js',
  'config-check.js',
  'config-init.js',
  'config-policy.js',
  'evidence-check.js',
  'evidence-freshness.js',
  'evidence-index.js',
  'evidence-query.js',
  'git-gate.js',
  'guard-state-files.js',
  'harness-doctor.js',
  'hook-install.js',
  'hook-uninstall.js',
  'installed-health.js',
  'managed-files-audit.js',
  'mcp-doctor.js',
  'mistake-check.js',
  'pre-completion.js',
  'pre-work.js',
  'project-profile.js',
  'runtime-docs-diff.js',
  'runtime-docs-init.js',
  'stale-control-check.js',
  'status-snapshot.js',
  'task-finish.js',
  'task-start.js',
  'task-status.js',
  'verification-plan.js',
  'verification-suite.js'
]);

const configOptionScripts = new Set([
  'capability-scan.js',
  'browser-evidence-check.js',
  'ci-init.js',
  'config-check.js',
  'config-init.js',
  'config-policy.js',
  'evidence-check.js',
  'evidence-freshness.js',
  'evidence-index.js',
  'evidence-query.js',
  'guard-state-files.js',
  'harness-doctor.js',
  'hook-install.js',
  'hook-uninstall.js',
  'installed-health.js',
  'managed-files-audit.js',
  'mcp-doctor.js',
  'mistake-check.js',
  'pre-completion.js',
  'pre-work.js',
  'project-profile.js',
  'runtime-docs-diff.js',
  'runtime-docs-init.js',
  'stale-control-check.js',
  'status-snapshot.js',
  'task-finish.js',
  'task-start.js',
  'task-status.js',
  'verification-plan.js',
  'verification-suite.js'
]);

const strictOptionScripts = new Set([
  'config-check.js',
  'browser-evidence-check.js',
  'ci-gate.js',
  'ci-init.js',
  'config-policy.js',
  'evidence-check.js',
  'evidence-freshness.js',
  'evidence-index.js',
  'evidence-query.js',
  'git-gate.js',
  'guard-state-files.js',
  'harness-doctor.js',
  'hook-install.js',
  'hook-uninstall.js',
  'installed-health.js',
  'managed-files-audit.js',
  'mistake-check.js',
  'pre-completion.js',
  'pre-work.js',
  'project-profile.js',
  'runtime-docs-diff.js',
  'stale-control-check.js',
  'task-status.js',
  'verification-plan.js',
  'verification-suite.js'
]);

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
      if (args.config) return { path: full, data: null, error: 'Config file was not found' };
      continue;
    }

    const parsed = readJson(full);
    return {
      path: full,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null
    };
  }

  return { path: null, data: null, error: null };
}

function splitCommand(command) {
  const input = String(command || '').trim();
  if (!input) throw new Error('Command is empty');
  if (/[|;&<>`$()]/.test(input)) {
    throw new Error('Command contains shell control syntax');
  }

  const tokens = [];
  let current = '';
  let quote = null;

  for (let i = 0; i < input.length; i += 1) {
    const char = input[i];

    if (quote) {
      if (char === quote) quote = null;
      else current += char;
      continue;
    }

    if (char === '"' || char === "'") {
      quote = char;
      continue;
    }

    if (/\s/.test(char)) {
      if (current) {
        tokens.push(current);
        current = '';
      }
      continue;
    }

    current += char;
  }

  if (quote) throw new Error('Command has an unterminated quote');
  if (current) tokens.push(current);
  return tokens;
}

function hasOption(tokens, option) {
  return tokens.includes(option);
}

function replaceOptionValue(tokens, option, value) {
  const index = tokens.indexOf(option);
  if (index === -1) return false;
  if (index === tokens.length - 1) {
    tokens.push(value);
  } else {
    tokens[index + 1] = value;
  }
  return true;
}

function normalizeChildArgs(scriptName, rawArgs, args) {
  const childArgs = rawArgs.slice();

  if (scriptName === 'rules-lint.js') {
    const targetIndex = childArgs.indexOf('--target');
    if (targetIndex !== -1) childArgs.splice(targetIndex, 2);
    const dotIndex = childArgs.indexOf('.');
    if (dotIndex !== -1) childArgs[dotIndex] = args.target;
    if (!childArgs.some((item) => !item.startsWith('--'))) childArgs.push(args.target);
    return childArgs;
  }

  if (targetOptionScripts.has(scriptName)) {
    if (!replaceOptionValue(childArgs, '--target', args.target)) {
      childArgs.push('--target', args.target);
    }
  }

  if (args.config && configOptionScripts.has(scriptName) && !hasOption(childArgs, '--config')) {
    childArgs.push('--config', args.config);
  }

  if (args.strict && strictOptionScripts.has(scriptName) && !hasOption(childArgs, '--strict')) {
    childArgs.push('--strict');
  }

  return childArgs;
}

function parseHarnessCommand(command, args, harnessDir, currentScript) {
  const tokens = splitCommand(command);
  if (tokens[0] !== 'node') {
    throw new Error('Only node scripts/harness/*.js commands are allowed');
  }

  const scriptToken = tokens[1];
  if (!/^scripts\/harness\/[A-Za-z0-9._-]+\.js$/.test(scriptToken || '')) {
    throw new Error('Only scripts/harness/*.js commands are allowed');
  }

  const scriptName = path.basename(scriptToken);
  if (scriptName === currentScript) {
    return {
      skip: true,
      name: scriptName.replace(/\.js$/, ''),
      script: scriptName,
      evidence: [`Skipped self-reference command: ${command}`]
    };
  }

  if (scriptName === 'rules-lint.js' && !detectProjectKind(args.target).isSourcePackage) {
    return {
      skip: true,
      name: 'rules-lint',
      script: scriptName,
      evidence: ['Skipped because rules-lint is source-package-only and this target is not a source package']
    };
  }

  const scriptPath = path.join(harnessDir, scriptName);
  return {
    skip: false,
    name: scriptName.replace(/\.js$/, ''),
    script: scriptName,
    scriptPath,
    args: normalizeChildArgs(scriptName, tokens.slice(2), args)
  };
}

function outputLines(output) {
  return String(output || '')
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter(Boolean);
}

function evidenceFromOutput(stdout, stderr) {
  const lines = outputLines(`${stdout || ''}\n${stderr || ''}`);
  return lines.slice(0, 18);
}

function sectionCount(output, sectionName) {
  const pattern = new RegExp(`^${sectionName}\\s*\\((\\d+)\\)`, 'im');
  const match = String(output || '').match(pattern);
  return match ? Number(match[1]) : 0;
}

function summarizeOutput(stdout, stderr) {
  const combined = `${stdout || ''}\n${stderr || ''}`;
  return {
    passCount: sectionCount(combined, 'PASS'),
    warnCount: sectionCount(combined, 'WARN'),
    failCount: sectionCount(combined, 'FAIL'),
    evidence: evidenceFromOutput(stdout, stderr)
  };
}

function skipCheck(parsed) {
  return {
    name: parsed.name,
    script: parsed.script,
    command: null,
    status: 'PASS',
    exitCode: null,
    summary: {
      passCount: 1,
      warnCount: 0,
      failCount: 0,
      evidence: parsed.evidence
    }
  };
}

function policySkipCheck(parsed, reason) {
  return {
    name: parsed.name,
    script: parsed.script,
    command: null,
    status: 'SKIP_BY_POLICY',
    exitCode: null,
    summary: {
      passCount: 0,
      warnCount: 1,
      failCount: 0,
      evidence: [reason]
    }
  };
}

function invalidCheck(command, message) {
  return {
    name: 'invalid-command',
    script: null,
    command,
    status: 'FAIL',
    exitCode: null,
    summary: {
      passCount: 0,
      warnCount: 0,
      failCount: 1,
      evidence: [`${message}: ${command}`]
    }
  };
}

function disabledChecks(configData) {
  const values = configData && configData.policy && Array.isArray(configData.policy.disabledChecks)
    ? configData.policy.disabledChecks
    : [];
  return new Set(values.map((item) => String(item).replace(/\.js$/, '').trim()).filter(Boolean));
}

function runCheck(command, args, harnessDir, currentScript, configData) {
  let parsed;
  try {
    parsed = parseHarnessCommand(command, args, harnessDir, currentScript);
  } catch (error) {
    return invalidCheck(command, error.message);
  }

  if (parsed.skip) return skipCheck(parsed);

  const disabled = disabledChecks(configData);
  if (disabled.has(parsed.name) || disabled.has(parsed.script)) {
    return policySkipCheck(parsed, `Skipped by policy.disabledChecks: ${parsed.name}`);
  }

  if (!fs.existsSync(parsed.scriptPath)) {
    return {
      name: parsed.name,
      script: parsed.script,
      command,
      status: 'WARN',
      exitCode: null,
      summary: {
        passCount: 0,
        warnCount: 1,
        failCount: 0,
        evidence: [`Harness script is missing: ${parsed.script}`]
      }
    };
  }

  const result = spawnSync(process.execPath, [parsed.scriptPath, ...parsed.args], {
    encoding: 'utf8',
    timeout: 30000,
    maxBuffer: 1024 * 1024 * 10
  });
  const summary = summarizeOutput(result.stdout, result.stderr);
  const exitCode = typeof result.status === 'number' ? result.status : 1;

  let status = 'PASS';
  if (exitCode !== 0 || summary.failCount > 0 || result.error) status = 'FAIL';
  else if (summary.warnCount > 0) status = 'WARN';

  if (result.error) summary.evidence.unshift(`Execution error: ${result.error.message}`);

  return {
    name: parsed.name,
    script: parsed.script,
    command,
    status,
    exitCode,
    summary
  };
}

function configuredCommands(configData, sectionName, strict, fallbackCommands) {
  const section = configData && configData[sectionName] && typeof configData[sectionName] === 'object'
    ? configData[sectionName]
    : null;
  const strictCommands = section && Array.isArray(section.strictPathRecommendedChecks)
    ? section.strictPathRecommendedChecks
    : null;
  const recommendedCommands = section && Array.isArray(section.recommendedChecks)
    ? section.recommendedChecks
    : null;

  if (strict && strictCommands && strictCommands.length > 0) {
    return { commands: strictCommands, source: `${sectionName}.strictPathRecommendedChecks` };
  }

  if (recommendedCommands && recommendedCommands.length > 0) {
    return { commands: recommendedCommands, source: `${sectionName}.recommendedChecks` };
  }

  return { commands: fallbackCommands, source: 'built-in fallback' };
}

function applyStrictEscalation(report) {
  if (!report.strict) return;
  const childWarns = report.details.checks.filter((check) => check.status === 'WARN');
  if (childWarns.length > 0) {
    add(report, 'warn', `Strict mode preserved non-failing WARN check(s): ${childWarns.map((check) => check.name).join(', ')}. Child checks own hard-gate escalation.`);
  }
}

function buildAggregateReport(options) {
  const report = {
    target: options.args.target,
    strict: options.args.strict,
    pass: [],
    warn: [],
    fail: [],
    details: {
      configPath: null,
      commandSource: null,
      commands: [],
      checks: []
    }
  };

  const targetStat = statSafe(options.args.target);
  if (!targetStat) {
    add(report, 'fail', `Target does not exist: ${options.args.target}`);
    return report;
  }
  if (!targetStat.isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${options.args.target}`);
    return report;
  }

  const config = loadConfig(options.args);
  report.details.configPath = config.path;
  if (config.error) {
    add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
    return report;
  }
  if (config.path) add(report, 'pass', `Harness config readable: ${rel(options.args.target, config.path)}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found; using built-in fallback checks');

  const selected = configuredCommands(
    config.data,
    options.sectionName,
    options.args.strict,
    options.fallbackCommands
  );
  report.details.commandSource = selected.source;
  report.details.commands = selected.commands.slice();
  report.details.checks = selected.commands.map((command) => runCheck(
    command,
    options.args,
    options.harnessDir,
    options.currentScript,
    config.data
  ));

  applyStrictEscalation(report);

  for (const check of report.details.checks) {
    const line = `${check.name}: ${check.status}${check.exitCode === null ? '' : ` (exit ${check.exitCode})`}`;
    if (check.status === 'FAIL') add(report, 'fail', line);
    else if (check.status === 'WARN') add(report, 'warn', line);
    else if (check.status === 'SKIP_BY_POLICY') add(report, 'warn', line);
    else add(report, 'pass', line);
  }

  return report;
}

module.exports = {
  buildAggregateReport,
  loadConfig,
  rel
};
