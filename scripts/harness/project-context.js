#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');

const CONTEXT_PATH = 'docs/project-context.json';
const MAX_DEFAULT_LINES = 25;
const MAX_DEFAULT_BYTES = 4096;
const POINTER_FIELDS = ['ruleEntry', 'authorityIndex', 'decision', 'taskPackage', 'plan'];
const TEXT_FIELDS = ['currentWork', 'nextAction', 'blocker', 'safetyReminder'];
const MANUAL_ENTRIES = ['AGENTS.md', 'AUTHORITY.md', 'CURRENT.md'];

function parseArgs(argv) {
  const options = {
    target: process.cwd(),
    json: false,
    diagnostic: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];

    if (argument === '--target') {
      const target = argv[index + 1];
      if (!target || target.startsWith('--')) {
        throw new Error('--target requires a directory');
      }
      options.target = target;
      index += 1;
    } else if (argument.startsWith('--target=')) {
      const target = argument.slice('--target='.length);
      if (!target) {
        throw new Error('--target requires a directory');
      }
      options.target = target;
    } else if (argument === '--json') {
      options.json = true;
    } else if (argument === '--diagnostic') {
      options.diagnostic = true;
    } else if (argument === '--help' || argument === '-h') {
      options.help = true;
    } else {
      throw new Error(`unknown option: ${argument}`);
    }
  }

  options.target = path.resolve(options.target);
  return options;
}

function isSafeRelativePath(value) {
  if (typeof value !== 'string' || value.length === 0 || /[\0\\\r\n]/.test(value)) {
    return false;
  }

  if (path.isAbsolute(value) || value.startsWith('./') || value.endsWith('/') || value.includes('//')) {
    return false;
  }

  const segments = value.split('/');
  return segments.every((segment) => segment.length > 0 && segment !== '.' && segment !== '..');
}

function isNonEmptyText(value) {
  return typeof value === 'string' && value.trim().length > 0 && !/[\r\n]/.test(value);
}

function fallback(reason) {
  return {
    status: 'DEGRADED',
    reasons: [reason],
    manualEntries: MANUAL_ENTRIES,
    nextAction: '修复 docs/project-context.json；已有明确任务仍按自身授权、范围和证据推进。',
    safetyReminder: '本路由只读；DEGRADED 不自动阻塞已有明确任务。',
  };
}

function validateRoute(candidate) {
  if (!candidate || typeof candidate !== 'object' || Array.isArray(candidate)) {
    return 'CONTEXT_MUST_BE_AN_OBJECT';
  }

  if (candidate.schemaVersion !== 1) {
    return 'UNSUPPORTED_SCHEMA_VERSION';
  }

  for (const field of POINTER_FIELDS) {
    if (!isSafeRelativePath(candidate[field])) {
      return `INVALID_POINTER:${field}`;
    }
  }

  for (const field of TEXT_FIELDS) {
    if (!isNonEmptyText(candidate[field])) {
      return `INVALID_TEXT:${field}`;
    }
  }

  return null;
}

function loadRoute(target) {
  const contextPath = path.join(target, CONTEXT_PATH);
  let raw;

  try {
    raw = fs.readFileSync(contextPath, 'utf8');
  } catch (error) {
    if (error && error.code === 'ENOENT') {
      return fallback('CONTEXT_MISSING');
    }
    return fallback('CONTEXT_UNREADABLE');
  }

  let candidate;
  try {
    candidate = JSON.parse(raw);
  } catch {
    return fallback('CONTEXT_MALFORMED');
  }

  const validationError = validateRoute(candidate);
  if (validationError) {
    return fallback(validationError);
  }

  const pointerChecks = {};
  for (const field of POINTER_FIELDS) {
    const pointer = candidate[field];
    const exists = fs.existsSync(path.join(target, pointer));
    pointerChecks[field] = { path: pointer, exists };
    if (!exists) {
      return fallback(`POINTER_MISSING:${field}`);
    }
  }

  return {
    status: 'READY',
    route: {
      ruleEntry: candidate.ruleEntry,
      authorityIndex: candidate.authorityIndex,
      decision: candidate.decision,
      taskPackage: candidate.taskPackage,
      plan: candidate.plan,
      currentWork: candidate.currentWork,
      nextAction: candidate.nextAction,
      blocker: candidate.blocker,
      safetyReminder: candidate.safetyReminder,
    },
    pointerChecks,
  };
}

function defaultPayload(result) {
  if (result.status === 'READY') {
    return {
      status: result.status,
      route: result.route,
    };
  }

  return {
    status: result.status,
    reasons: result.reasons,
    manualEntries: result.manualEntries,
    nextAction: result.nextAction,
    safetyReminder: result.safetyReminder,
  };
}

function renderText(result) {
  if (result.status === 'READY') {
    const route = result.route;
    return [
      'ROUTE: READY — 导航有效，不等于业务实施授权',
      `规则: ${route.ruleEntry}`,
      `权威索引: ${route.authorityIndex}`,
      `当前决策: ${route.decision}`,
      `当前任务包: ${route.taskPackage}`,
      `当前计划: ${route.plan}`,
      `在做: ${route.currentWork}`,
      `唯一下一步: ${route.nextAction}`,
      `阻塞: ${route.blocker}`,
      `安全: ${route.safetyReminder}`,
    ].join('\n');
  }

  return [
    `ROUTE: DEGRADED — ${result.reasons.join(', ')}`,
    `人工入口: ${result.manualEntries.join(' / ')}`,
    `唯一下一步: ${result.nextAction}`,
    `安全: ${result.safetyReminder}`,
  ].join('\n');
}

function isWithinDefaultBudget(output) {
  const lineCount = output.length === 0 ? 0 : output.split(/\r?\n/).length;
  return (
    Buffer.byteLength(`${output}\n`, 'utf8') <= MAX_DEFAULT_BYTES
    && lineCount <= MAX_DEFAULT_LINES
  );
}

function enforceDefaultBudget(result) {
  if (result.status !== 'READY') {
    return result;
  }

  const textOutput = renderText(result);
  const jsonOutput = JSON.stringify(defaultPayload(result));
  if (!isWithinDefaultBudget(textOutput) || !isWithinDefaultBudget(jsonOutput)) {
    return fallback('ROUTE_OUTPUT_BUDGET_EXCEEDED');
  }

  return result;
}

function diagnosticFor(target, result) {
  const diagnostic = {
    target,
    contextPath: CONTEXT_PATH,
    pointerChecks: result.pointerChecks || {},
    workspaceMarkers: {
      gitDirectory: fs.existsSync(path.join(target, '.git')),
      commitMessageHook: fs.existsSync(path.join(target, '.githooks', 'commit-msg')),
      structuredCodeMap: fs.existsSync(path.join(target, 'docs', 'code-map', 'index.json')),
      legacyCodeMap: fs.existsSync(path.join(target, 'docs', '2026-07-09-codebase-capability-map-v2.md')),
    },
  };

  return diagnostic;
}

function renderDiagnosticText(result, diagnostic) {
  const output = [renderText(result), '诊断:', `- target: ${diagnostic.target}`, `- context: ${diagnostic.contextPath}`];

  for (const [field, value] of Object.entries(diagnostic.pointerChecks)) {
    output.push(`- pointer ${field}: ${value.exists ? 'present' : 'missing'} (${value.path})`);
  }

  for (const [field, value] of Object.entries(diagnostic.workspaceMarkers)) {
    output.push(`- ${field}: ${value ? 'present' : 'absent'}`);
  }

  return output.join('\n');
}

function helpText() {
  return [
    'Usage: node scripts/harness/project-context.js [--target <directory>] [--json] [--diagnostic]',
    'Read-only, fail-open route to the current rules, authority, task, plan, next action, blocker, and safety boundary.',
    'Default mode does not run Git, inspect hooks, scan Code Map/source, or read historical documents.',
  ].join('\n');
}

function main(argv = process.argv.slice(2)) {
  let options;
  try {
    options = parseArgs(argv);
  } catch (error) {
    process.stderr.write(`project-context: ${error.message}\n`);
    return 2;
  }

  if (options.help) {
    process.stdout.write(`${helpText()}\n`);
    return 0;
  }

  const result = enforceDefaultBudget(loadRoute(options.target));
  const diagnostic = options.diagnostic ? diagnosticFor(options.target, result) : null;
  const output = options.json
    ? JSON.stringify(diagnostic ? { ...defaultPayload(result), diagnostic } : defaultPayload(result))
    : diagnostic
      ? renderDiagnosticText(result, diagnostic)
      : renderText(result);

  process.stdout.write(`${output}\n`);
  return 0;
}

if (require.main === module) {
  process.exitCode = main();
}

module.exports = {
  CONTEXT_PATH,
  MAX_DEFAULT_BYTES,
  MAX_DEFAULT_LINES,
  defaultPayload,
  enforceDefaultBudget,
  isWithinDefaultBudget,
  loadRoute,
  main,
  parseArgs,
  renderText,
};
