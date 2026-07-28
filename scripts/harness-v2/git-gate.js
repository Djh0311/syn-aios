#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { TextDecoder } = require('node:util');
const { safeOutputRepoPath, sanitizeOutputText } = require('./lib/output-safety');
const stagedTree = require('./lib/staged-tree');

const FINDING_LIMIT = 20;
const SAFE_ENV_TEMPLATES = new Set(['.env.example', '.env.sample', '.env.template']);
const PRIVATE_KEY_BASENAMES = new Set(['id_rsa', 'id_dsa', 'id_ecdsa', 'id_ed25519']);
const PROTECTED_EXACT = new Set([
  'AGENTS.md',
  'CLAUDE.md',
  'harness.config.json',
  '.claude/settings.json'
]);
const PROTECTED_PREFIXES = [
  '.claude/hooks/',
  'scripts/harness/',
  'scripts/harness-v2/',
  'templates/hooks/'
];
const PRIVATE_KEY_MARKER = /-----BEGIN(?: [A-Z0-9]+)* PRIVATE KEY(?: BLOCK)?-----/i;
const HIGH_CONFIDENCE_TOKENS = [
  /\bAKIA[0-9A-Z]{16}\b/,
  /\bgh[pousr]_[A-Za-z0-9]{20,}\b/,
  /\bxox[baprs]-[A-Za-z0-9-]{20,}\b/,
  /\bsk-(?:proj-|live-)?[A-Za-z0-9_-]{20,}\b/
];

function parseArgs(argv) {
  const args = { target: process.cwd(), json: false, strict: false };
  for (let index = 0; index < argv.length; index += 1) {
    const item = argv[index];
    if (item === '--target') args.target = argv[++index];
    else if (item === '--json') args.json = true;
    else if (item === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${item}`);
  }
  if (!args.target) throw new Error('--target requires a value');
  args.target = path.resolve(args.target);
  return args;
}

function normalizePath(value) {
  return String(value || '').replace(/^\.\//, '');
}

function parseOverrideList(raw) {
  const allowed = [];
  const invalid = [];
  for (const original of String(raw || '').split(',').map((item) => item.trim()).filter(Boolean)) {
    const candidate = normalizePath(original);
    const segments = candidate.split('/');
    const unsafe = !candidate ||
      path.posix.isAbsolute(candidate) ||
      candidate.includes('\\') ||
      candidate.endsWith('/') ||
      segments.includes('..') ||
      /[*?\[\]{}]/.test(candidate) ||
      path.posix.normalize(candidate) !== candidate;
    if (unsafe) invalid.push(original);
    else allowed.push(candidate);
  }
  return {
    allowed: [...new Set(allowed)],
    invalid: [...new Set(invalid)]
  };
}

function isEnvPath(file) {
  const basename = path.posix.basename(file);
  return basename === '.env' || basename.startsWith('.env.');
}

function isSafeEnvTemplate(file) {
  return SAFE_ENV_TEMPLATES.has(path.posix.basename(file));
}

function isPrivateKeyPath(file) {
  const basename = path.posix.basename(file);
  return PRIVATE_KEY_BASENAMES.has(basename) ||
    /\.(?:key|p12|pfx|ppk)$/i.test(basename) ||
    /(?:^|[._-])private(?:[._-]?key)?\.pem$/i.test(basename);
}

function isProtectedPath(file) {
  return PROTECTED_EXACT.has(file) || PROTECTED_PREFIXES.some((prefix) => file.startsWith(prefix));
}

function placeholderValue(raw) {
  const value = String(raw || '').trim().replace(/^(['"])([\s\S]*)\1$/, '$2').trim();
  if (!value) return true;
  const reference = value.match(/^\$\{[A-Za-z_][A-Za-z0-9_]*(?::-(.*))?\}$/);
  if (reference) return reference[1] === undefined || placeholderValue(reference[1]);
  if (/^<[^>]+>$/.test(value)) return true;
  return /^(?:your(?:[-_ ].*)?|change[-_ ]?me|replace(?:[-_ ]?me|[-_ ]?with[-_ ].*)?|dev[-_ ]?secret|placeholder|example|sample|dummy|fake|test(?:[-_ ]?only)?|local|development|x{3,}|none|null|not[-_ ]?set|redacted)$/i.test(value);
}

function contentFinding(content, scanAssignments) {
  if (!content) return null;
  if (PRIVATE_KEY_MARKER.test(content)) return 'private-key marker';
  if (HIGH_CONFIDENCE_TOKENS.some((pattern) => pattern.test(content))) {
    return 'high-confidence secret token';
  }
  if (!scanAssignments) return null;
  if (content.includes('\0')) return 'NUL byte in environment template';
  for (const line of content.split(/\r?\n/)) {
    const match = line.match(/^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*[=:]\s*(.*?)\s*$/);
    if (!match) continue;
    if (!/(?:^|_)(?:SECRET|SECRET_KEY|TOKEN|PASSWORD|PASSWD|API_KEY|ACCESS_KEY|PRIVATE_KEY|SHARED_SECRET)(?:_(?:MD5|HASH))?$/i.test(match[1])) {
      continue;
    }
    if (!placeholderValue(match[2])) return `non-placeholder value assigned to ${match[1]}`;
  }
  return null;
}

function stagedBlobs(target, files) {
  if (files.length === 0) return { ok: true, blobs: new Map(), error: null };
  const index = stagedTree.git(target, ['ls-files', '--stage', '-z'], {
    encoding: null,
    maxBuffer: 32 * 1024 * 1024
  });
  if (!index.ok) return { ok: false, blobs: new Map(), error: index.error };

  let text;
  try {
    text = new TextDecoder('utf-8', { fatal: true }).decode(index.stdout);
  } catch (error) {
    return { ok: false, blobs: new Map(), error: 'Git index paths are not valid UTF-8' };
  }
  const wanted = new Set(files);
  const oidByFile = new Map();
  for (const record of text.split('\0').filter(Boolean)) {
    const tab = record.indexOf('\t');
    const metadata = tab >= 0
      ? record.slice(0, tab).match(/^(\d{6}) ([0-9a-f]{40}|[0-9a-f]{64}) ([0-3])$/)
      : null;
    const file = tab >= 0 ? stagedTree.safeRepoPath(record.slice(tab + 1)) : null;
    if (metadata && file && metadata[3] === '0' && wanted.has(file)) {
      oidByFile.set(file, metadata[2]);
    }
  }
  const missing = files.filter((file) => !oidByFile.has(file));
  if (missing.length > 0) {
    return { ok: false, blobs: new Map(), error: 'One or more staged blobs were unavailable' };
  }

  const oids = [...new Set(files.map((file) => oidByFile.get(file)))];
  const batch = stagedTree.git(target, ['cat-file', '--batch'], {
    encoding: null,
    input: Buffer.from(`${oids.join('\n')}\n`, 'utf8'),
    maxBuffer: 128 * 1024 * 1024
  });
  if (!batch.ok) return { ok: false, blobs: new Map(), error: batch.error };

  const contentByOid = new Map();
  let offset = 0;
  for (const oid of oids) {
    const newline = batch.stdout.indexOf(0x0a, offset);
    if (newline < 0) return { ok: false, blobs: new Map(), error: 'Malformed cat-file header' };
    const header = batch.stdout.subarray(offset, newline).toString('utf8').split(' ');
    const size = Number(header[header.length - 1]);
    if (!Number.isSafeInteger(size) || size < 0) {
      return { ok: false, blobs: new Map(), error: 'Malformed cat-file size' };
    }
    const start = newline + 1;
    const end = start + size;
    if (end > batch.stdout.length) {
      return { ok: false, blobs: new Map(), error: 'Truncated staged blob' };
    }
    contentByOid.set(oid, batch.stdout.subarray(start, end));
    offset = end + 1;
  }

  const blobs = new Map();
  for (const file of files) blobs.set(file, contentByOid.get(oidByFile.get(file)));
  return { ok: true, blobs, error: null };
}

function limited(items) {
  return items.slice(0, FINDING_LIMIT);
}

function publicFinding(finding) {
  return {
    file: safeOutputRepoPath(finding.file),
    reason: finding.reason ? sanitizeOutputText(finding.reason, 160) : undefined
  };
}

function buildReport(args) {
  const overrides = parseOverrideList(process.env.HARNESS_ALLOW_PROTECTED_STAGED);
  const report = {
    command: 'git-gate',
    status: 'PASS',
    target: '.',
    scope: 'staged-only',
    strict: args.strict,
    pass: [],
    fail: [],
    details: {
      stagedFileCount: 0,
      inspectedBlobCount: 0,
      unstagedInspected: false,
      blockedSecretPaths: [],
      blockedContent: [],
      unauthorizedProtectedChanges: [],
      authorizedProtectedChanges: [],
      invalidOverrides: [],
      findingsTruncated: false
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    report.fail.push('Target must be an existing directory');
    report.status = 'FAIL';
    return report;
  }
  const inside = stagedTree.git(args.target, ['rev-parse', '--is-inside-work-tree']);
  if (!inside.ok || inside.stdout.trim() !== 'true') {
    if (args.strict) report.fail.push('Target is not a confirmed Git worktree');
    else report.pass.push('No Git worktree was inspected');
    report.status = report.fail.length ? 'FAIL' : 'PASS';
    return report;
  }
  if (overrides.invalid.length > 0) report.fail.push('Protected-path override contains non-exact paths');

  const staged = stagedTree.stagedChanges(args.target);
  if (!staged.ok) {
    report.fail.push('Unable to inspect staged paths');
    report.status = 'FAIL';
    return report;
  }
  const changes = staged.changes;
  report.details.stagedFileCount = changes.length;
  if (changes.length === 0) {
    report.pass.push('No staged changes to inspect');
    return report;
  }
  if (changes.some((change) => ['B', 'U', 'X'].includes(change.status))) {
    report.fail.push('Unresolved or unsupported staged state');
  }

  const contentFiles = [];
  const protectedChanges = new Set();
  const blockedSecretPaths = [];
  for (const change of changes) {
    const current = change.newPath || (change.status !== 'D' ? change.path : null);
    const old = change.oldPath || (change.status === 'D' ? change.path : null);
    for (const candidate of [old, current].filter(Boolean)) {
      if (isProtectedPath(candidate)) protectedChanges.add(candidate);
    }
    if (!current) continue;
    if (isEnvPath(current) && !isSafeEnvTemplate(current)) {
      blockedSecretPaths.push({ file: current, reason: 'environment file' });
      continue;
    }
    if (isPrivateKeyPath(current)) {
      blockedSecretPaths.push({ file: current, reason: 'private-key path' });
      continue;
    }
    contentFiles.push(current);
  }

  const inspected = stagedBlobs(args.target, [...new Set(contentFiles)]);
  const blockedContent = [];
  if (!inspected.ok) {
    report.fail.push('Staged content could not be fully inspected');
  } else {
    report.details.inspectedBlobCount = inspected.blobs.size;
    for (const file of contentFiles) {
      const blob = inspected.blobs.get(file);
      const content = blob ? blob.toString('utf8') : '';
      const reason = contentFinding(content, isSafeEnvTemplate(file));
      if (reason) blockedContent.push({ file, reason });
    }
  }

  const unauthorized = [...protectedChanges].filter((file) => !overrides.allowed.includes(file));
  const authorized = [...protectedChanges].filter((file) => overrides.allowed.includes(file));
  let remainingFindingBudget = FINDING_LIMIT;
  const boundedFindings = (items) => {
    const selected = items.slice(0, remainingFindingBudget);
    remainingFindingBudget -= selected.length;
    return selected;
  };
  report.details.invalidOverrides = boundedFindings(overrides.invalid)
    .map((item) => safeOutputRepoPath(item));
  report.details.blockedSecretPaths = boundedFindings(blockedSecretPaths).map(publicFinding);
  report.details.blockedContent = boundedFindings(blockedContent).map(publicFinding);
  report.details.unauthorizedProtectedChanges = boundedFindings(unauthorized)
    .map((item) => safeOutputRepoPath(item));
  report.details.authorizedProtectedChanges = limited(authorized)
    .map((item) => safeOutputRepoPath(item));
  const totalFindings = blockedSecretPaths.length + blockedContent.length + unauthorized.length + overrides.invalid.length;
  report.details.findingsTruncated = totalFindings > FINDING_LIMIT;

  if (blockedSecretPaths.length > 0) report.fail.push(`${blockedSecretPaths.length} secret-bearing staged path(s) rejected`);
  if (blockedContent.length > 0) report.fail.push(`${blockedContent.length} staged secret-content finding(s) rejected`);
  if (unauthorized.length > 0) {
    report.fail.push(`${unauthorized.length} Harness-protected path(s) lack exact per-invocation authorization`);
  }
  if (authorized.length > 0) report.pass.push(`${authorized.length} exact protected-path authorization(s) applied`);
  if (report.fail.length === 0) {
    report.pass.push(`Staged-only checks passed for ${changes.length} path(s)`);
  } else {
    report.status = 'FAIL';
  }
  return report;
}

function renderText(report) {
  const lines = [`staged-git-gate ${report.status}`];
  for (const item of report.pass) lines.push(`PASS ${item}`);
  for (const item of report.fail) lines.push(`FAIL ${item}`);
  for (const [kind, findings] of [
    ['SECRET_PATH', report.details.blockedSecretPaths],
    ['SECRET_CONTENT', report.details.blockedContent],
    ['PROTECTED_PATH', report.details.unauthorizedProtectedChanges]
  ]) {
    for (const finding of findings) {
      if (typeof finding === 'string') {
        lines.push(`${kind} ${JSON.stringify(finding)}`);
      } else {
        lines.push(`${kind} ${JSON.stringify(finding.file)}${finding.reason ? ` (${finding.reason})` : ''}`);
      }
    }
  }
  if (report.details.findingsTruncated) lines.push('WARN Additional findings omitted');
  return `${lines.join('\n')}\n`;
}

function main(argv = process.argv.slice(2)) {
  let args;
  try {
    args = parseArgs(argv);
  } catch (error) {
    process.stderr.write(`${sanitizeOutputText(error.message, 240)}\n`);
    return 2;
  }
  let report;
  try {
    report = buildReport(args);
  } catch (error) {
    report = {
      command: 'git-gate',
      status: 'FAIL',
      target: '.',
      scope: 'staged-only',
      strict: args.strict,
      pass: [],
      fail: ['Gate failed before staged content could be proven safe'],
      details: {
        stagedFileCount: 0,
        inspectedBlobCount: 0,
        unstagedInspected: false,
        blockedSecretPaths: [],
        blockedContent: [],
        unauthorizedProtectedChanges: [],
        authorizedProtectedChanges: [],
        invalidOverrides: [],
        findingsTruncated: false
      }
    };
  }
  process.stdout.write(args.json ? `${JSON.stringify(report, null, 2)}\n` : renderText(report));
  return report.status === 'FAIL' ? 1 : 0;
}

if (require.main === module) process.exitCode = main();

module.exports = {
  buildReport,
  contentFinding,
  isEnvPath,
  isPrivateKeyPath,
  isProtectedPath,
  main,
  parseArgs,
  parseOverrideList,
  placeholderValue,
  stagedBlobs
};
