#!/usr/bin/env node

'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');

const {
  AUTHORITY_PATH,
  CURRENT_PATH,
  loadCoreContext,
  resolveSafeFile,
  sanitizeText
} = require('./lib/context-contract');

const ROUTE_MAX_BYTES = 4 * 1024;
const ROUTE_MAX_LINES = 25;
const DIAGNOSTIC_MAX_BYTES = 12 * 1024;
const DIAGNOSTIC_MAX_LINES = 160;
const GIT_TIMEOUT_MS = 3000;
const GIT_MAX_BUFFER = 4 * 1024 * 1024;

function usage() {
  return [
    'Usage: node scripts/harness-v2/project-context.js [options]',
    '',
    'Options:',
    '  --target <path>      Repository root (default: current directory)',
    '  --diagnostic         Add read-only Git, hook, and Code Map diagnostics',
    '  --json               Change output format only; does not enable diagnostics',
    `  --max-bytes <n>     Output byte budget (route always <= ${ROUTE_MAX_BYTES})`,
    `  --max-lines <n>     Output line budget (route always <= ${ROUTE_MAX_LINES})`,
    '  --help               Show this help'
  ].join('\n');
}

function positiveInteger(value, flag) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed <= 0) {
    throw new Error(`${flag} expects a positive integer`);
  }
  return parsed;
}

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    diagnostic: false,
    json: false,
    maxBytes: null,
    maxLines: null,
    help: false
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--target') {
      const value = argv[++index];
      if (!value) throw new Error('--target expects a path');
      args.target = value;
    } else if (arg === '--diagnostic') {
      args.diagnostic = true;
    } else if (arg === '--json') {
      args.json = true;
    } else if (arg === '--max-bytes') {
      args.maxBytes = positiveInteger(argv[++index], arg);
    } else if (arg === '--max-lines') {
      args.maxLines = positiveInteger(argv[++index], arg);
    } else if (arg === '--help' || arg === '-h') {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return args;
}

function git(root, args) {
  try {
    return {
      ok: true,
      stdout: execFileSync('git', ['--no-optional-locks', '-C', root, ...args], {
        encoding: 'utf8',
        env: { ...process.env, GIT_OPTIONAL_LOCKS: '0' },
        timeout: GIT_TIMEOUT_MS,
        maxBuffer: GIT_MAX_BUFFER,
        stdio: ['ignore', 'pipe', 'ignore']
      })
    };
  } catch {
    return { ok: false, stdout: '' };
  }
}

function workspaceState(root) {
  const result = git(root, ['status', '--porcelain=v1', '--untracked-files=normal']);
  if (!result.ok) {
    return {
      status: 'UNAVAILABLE',
      counts: { staged: 0, unstaged: 0, untracked: 0, conflicted: 0, total: 0 }
    };
  }

  const records = result.stdout.split('\n').filter(Boolean);
  const counts = { staged: 0, unstaged: 0, untracked: 0, conflicted: 0, total: records.length };
  const conflicts = new Set(['DD', 'AU', 'UD', 'UA', 'DU', 'AA', 'UU']);
  for (const record of records) {
    const code = record.slice(0, 2);
    if (code === '??') {
      counts.untracked += 1;
      continue;
    }
    if (code[0] && code[0] !== ' ') counts.staged += 1;
    if (code[1] && code[1] !== ' ') counts.unstaged += 1;
    if (conflicts.has(code) || code.includes('U')) counts.conflicted += 1;
  }
  return { status: records.length ? 'DIRTY' : 'CLEAN', counts };
}

function hookState(root) {
  const hookPath = git(root, ['rev-parse', '--git-path', 'hooks']);
  if (!hookPath.ok) return { status: 'UNAVAILABLE' };
  const configured = hookPath.stdout.trim();
  const directory = path.isAbsolute(configured) ? configured : path.resolve(root, configured);
  const scope = directory === root || directory.startsWith(`${root}${path.sep}`)
    ? 'repository'
    : 'external-redacted';

  function inspect(name) {
    try {
      const stat = fs.lstatSync(path.join(directory, name));
      return {
        installed: stat.isFile() && !stat.isSymbolicLink(),
        executable: stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o111) !== 0
      };
    } catch {
      return { installed: false, executable: false };
    }
  }

  return {
    status: 'OK',
    scope,
    preCommit: inspect('pre-commit'),
    prePush: inspect('pre-push')
  };
}

function codeMapState(core) {
  const relativePath = core.pointers['code-map'];
  if (!relativePath) return { status: 'UNAVAILABLE', entry: null };
  const resolved = resolveSafeFile(core.root, relativePath);
  if (!resolved.ok) return { status: resolved.code, entry: relativePath };
  const baselined = git(core.root, ['cat-file', '-e', `HEAD:${relativePath}`]).ok;
  return {
    status: baselined ? 'OK' : 'NOT_BASELINED',
    entry: relativePath,
    baselined
  };
}

function collectDiagnostics(core) {
  const repositoryCheck = git(core.root, ['rev-parse', '--is-inside-work-tree']);
  const gitAvailable = repositoryCheck.ok && repositoryCheck.stdout.trim() === 'true';
  if (!gitAvailable) {
    return {
      status: 'WARN',
      git: {
        status: 'UNAVAILABLE',
        branch: 'UNKNOWN',
        head: 'UNKNOWN',
        workspace: {
          status: 'UNAVAILABLE',
          counts: { staged: 0, unstaged: 0, untracked: 0, conflicted: 0, total: 0 }
        }
      },
      hooks: { status: 'UNAVAILABLE' },
      codeMap: { status: 'UNAVAILABLE', entry: core.pointers['code-map'] || null },
      warnings: ['GIT_UNAVAILABLE']
    };
  }

  const branch = git(core.root, ['symbolic-ref', '--short', '-q', 'HEAD']);
  const head = git(core.root, ['rev-parse', '--short=12', 'HEAD']);
  const hooks = hookState(core.root);
  const codeMap = codeMapState(core);
  const warnings = [];
  if (hooks.status !== 'OK') warnings.push('HOOKS_UNAVAILABLE');
  if (codeMap.status !== 'OK') warnings.push(`CODE_MAP_${codeMap.status}`);

  return {
    status: warnings.length ? 'WARN' : 'OK',
    git: {
      status: 'OK',
      branch: branch.ok ? sanitizeText(branch.stdout.trim(), 120) || 'DETACHED' : 'UNKNOWN',
      head: head.ok ? head.stdout.trim() : 'UNKNOWN',
      workspace: workspaceState(core.root)
    },
    hooks,
    codeMap,
    warnings
  };
}

function routeAdvisories(core) {
  const advisories = Array.isArray(core.advisories)
    ? core.advisories
    : Array.isArray(core.warnings)
      ? core.warnings
      : [];
  return advisories.slice(0, 8).map((entry) => ({
    source: sanitizeText(entry && entry.source, 240),
    code: sanitizeText(entry && entry.code, 120)
  }));
}

function routeFromCore(core) {
  return {
    schemaVersion: 2,
    mode: 'route',
    status: core.coreStatus,
    coreStatus: core.coreStatus,
    readOnly: true,
    repository: { name: core.repository.name },
    activeAuthority: core.activeAuthority
      ? {
          path: core.activeAuthority.path,
          id: core.activeAuthority.id,
          schema: core.activeAuthority.schema,
          status: core.activeAuthority.status
        }
      : null,
    current: {
      mode: core.current.mode,
      workState: core.current.workState,
      authority: core.authority,
      goal: core.current.goal,
      phase: core.current.phase,
      status: core.current.status.slice(0, 3),
      blockers: core.current.blockers.slice(0, 3),
      nextAction: core.current.nextAction,
      safety: core.current.safety.slice(0, 2)
    },
    issues: core.issues.slice(0, 8),
    advisories: routeAdvisories(core)
  };
}

function buildRouteContext(target) {
  return routeFromCore(loadCoreContext(target));
}

function buildDiagnosticContext(target) {
  const core = loadCoreContext(target);
  const route = routeFromCore(core);
  const diagnostics = collectDiagnostics(core);
  return {
    ...route,
    mode: 'diagnostic',
    diagnosticsStatus: diagnostics.status,
    diagnostics
  };
}

function routeTextLines(report, includeHint = true) {
  const active = report.activeAuthority;
  const lines = [
    `${report.repository.name} project route — ${report.coreStatus} (read-only)`,
    `Mode: ${report.current.mode || 'UNKNOWN'} | Work state: ${report.current.workState || 'UNKNOWN'}`,
    `Authority: ${report.current.authority || 'not declared'}`,
    `Active package: ${active ? `${active.id || 'UNKNOWN'} | ${active.schema || 'UNKNOWN'} | ${active.status || 'UNKNOWN'}` : 'none'}`,
    `Goal: ${report.current.goal || 'unavailable'}`,
    `Phase: ${report.current.phase || 'unavailable'}`
  ];

  if (report.current.status.length) {
    lines.push('', 'Status:');
    for (const item of report.current.status) lines.push(`- ${item}`);
  }
  if (report.current.blockers.length) {
    lines.push('', 'Blockers:');
    for (const item of report.current.blockers) lines.push(`- ${item}`);
  }
  lines.push('', `Next action: ${report.current.nextAction || 'repair the current context before acting'}`);
  for (const item of report.current.safety) lines.push(`Safety: ${item}`);
  if (report.issues.length) {
    lines.push(`Route warning: ${report.issues.map((entry) => entry.code).join(', ')}`);
  }
  lines.push(`Advisories: ${report.advisories.length
    ? report.advisories.map((entry) => entry.code).join(', ')
    : 'none'}`);
  if (includeHint) {
    lines.push('Diagnostics: node scripts/harness-v2/project-context.js --target . --diagnostic');
  }
  return lines;
}

function diagnosticTextLines(report) {
  const lines = routeTextLines(report, false);
  const counts = report.diagnostics.git.workspace.counts;
  lines.push(
    '',
    `Diagnostics: ${report.diagnosticsStatus}`,
    `Git: ${report.diagnostics.git.status} | ${report.diagnostics.git.branch}@${report.diagnostics.git.head}`,
    `Workspace: ${report.diagnostics.git.workspace.status} | staged ${counts.staged}, unstaged ${counts.unstaged}, untracked ${counts.untracked}, conflicts ${counts.conflicted}`,
    `Hooks: pre-commit ${report.diagnostics.hooks.preCommit && report.diagnostics.hooks.preCommit.installed ? 'installed' : 'absent'}, pre-push ${report.diagnostics.hooks.prePush && report.diagnostics.hooks.prePush.installed ? 'installed' : 'absent'}`,
    `Code map: ${report.diagnostics.codeMap.status} | ${report.diagnostics.codeMap.entry || 'not declared'}`
  );
  if (report.diagnostics.warnings.length) {
    lines.push('Diagnostic warnings:');
    for (const warning of report.diagnostics.warnings) lines.push(`- ${warning}`);
  }
  return lines;
}

function truncateUtf8(value, maxBytes) {
  if (maxBytes < 1) return '';
  if (Buffer.byteLength(value, 'utf8') <= maxBytes) return value;
  const marker = '…';
  if (maxBytes < Buffer.byteLength(marker, 'utf8')) return '.'.repeat(maxBytes);
  let result = '';
  for (const character of value) {
    if (Buffer.byteLength(`${result}${character}${marker}`, 'utf8') > maxBytes) break;
    result += character;
  }
  return `${result}${marker}`;
}

function fitText(lines, maxBytes, maxLines) {
  const output = [];
  const marker = '[DEGRADED: output truncated]';
  let truncated = false;
  for (const rawLine of lines) {
    const line = truncateUtf8(rawLine, maxBytes);
    const candidate = [...output, line].join('\n');
    if (
      output.length + 1 > maxLines ||
      Buffer.byteLength(candidate, 'utf8') > maxBytes
    ) {
      truncated = true;
      break;
    }
    output.push(line);
  }
  if (truncated) {
    while (
      output.length &&
      (
        output.length + 1 > maxLines ||
        Buffer.byteLength([...output, marker].join('\n'), 'utf8') > maxBytes
      )
    ) output.pop();
    if (maxLines >= 1 && Buffer.byteLength(marker, 'utf8') <= maxBytes) output.push(marker);
  }
  return output.length ? output.join('\n') : truncateUtf8('DEGRADED', maxBytes);
}

function compactJson(report, maxBytes) {
  const candidates = [
    report,
    {
      ...report,
      current: {
        ...report.current,
        status: report.current.status.slice(0, 1),
        blockers: report.current.blockers.slice(0, 1),
        safety: report.current.safety.slice(0, 1)
      },
      issues: report.issues.slice(0, 3)
    },
    {
      schemaVersion: 2,
      mode: report.mode,
      status: 'DEGRADED',
      coreStatus: report.coreStatus,
      readOnly: true,
      activeAuthority: report.activeAuthority,
      current: {
        authority: report.current.authority,
        nextAction: report.current.nextAction
      },
      advisories: report.advisories,
      truncated: true
    },
    { status: 'DEGRADED', readOnly: true, truncated: true }
  ];
  for (const candidate of candidates) {
    const output = JSON.stringify(candidate);
    if (Buffer.byteLength(output, 'utf8') <= maxBytes) return output;
  }
  return maxBytes >= 2 ? '{}' : '';
}

function formatContext(report, options = {}) {
  const route = report.mode === 'route';
  const defaultBytes = route ? ROUTE_MAX_BYTES : DIAGNOSTIC_MAX_BYTES;
  const defaultLines = route ? ROUTE_MAX_LINES : DIAGNOSTIC_MAX_LINES;
  const requestedBytes = options.maxBytes || defaultBytes;
  const requestedLines = options.maxLines || defaultLines;
  const maxBytes = route ? Math.min(requestedBytes, ROUTE_MAX_BYTES) : requestedBytes;
  const maxLines = route ? Math.min(requestedLines, ROUTE_MAX_LINES) : requestedLines;

  if (options.json) return compactJson(report, maxBytes);
  const lines = route ? routeTextLines(report) : diagnosticTextLines(report);
  return fitText(lines, maxBytes, maxLines);
}

function degradedArgumentReport() {
  return {
    schemaVersion: 2,
    mode: 'route',
    status: 'DEGRADED',
    coreStatus: 'DEGRADED',
    readOnly: true,
    repository: { name: 'unknown' },
    activeAuthority: null,
    current: {
      mode: null,
      workState: null,
      authority: null,
      goal: '',
      phase: '',
      status: [],
      blockers: [],
      nextAction: null,
      safety: []
    },
    issues: [{ source: 'project-context', code: 'INVALID_ARGUMENTS' }],
    advisories: []
  };
}

function main(argv = process.argv.slice(2)) {
  let args;
  try {
    args = parseArgs(argv);
  } catch {
    const report = degradedArgumentReport();
    const wantsJson = argv.includes('--json');
    process.stdout.write(formatContext(report, { json: wantsJson }));
    return 0;
  }

  if (args.help) {
    process.stdout.write(`${usage()}\n`);
    return 0;
  }

  try {
    const report = args.diagnostic
      ? buildDiagnosticContext(args.target)
      : buildRouteContext(args.target);
    process.stdout.write(formatContext(report, args));
  } catch {
    const report = degradedArgumentReport();
    report.issues = [{ source: 'project-context', code: 'UNEXPECTED_ERROR' }];
    process.stdout.write(formatContext(report, args));
  }
  return 0;
}

if (require.main === module) process.exitCode = main();

module.exports = {
  DIAGNOSTIC_MAX_BYTES,
  DIAGNOSTIC_MAX_LINES,
  ROUTE_MAX_BYTES,
  ROUTE_MAX_LINES,
  AUTHORITY_PATH,
  CURRENT_PATH,
  buildDiagnosticContext,
  buildRouteContext,
  collectDiagnostics,
  formatContext,
  main,
  parseArgs,
  routeFromCore
};
