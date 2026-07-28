'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { sanitizeOutputText } = require('./output-safety');

const AUTHORITY_PATH = 'docs/harness/AUTHORITY.md';
const CURRENT_PATH = 'docs/harness/CURRENT.md';

const AUTHORITY_SCHEMA = 'harness-authority/v2';
const CURRENT_SCHEMA = 'harness-current/v2';
const ACTIVE_SCHEMA = 'harness-active/v3';
const LEGACY_ACTIVE_SCHEMA = 'harness-active/v2';
const ACTIVE_SCHEMAS = new Set([LEGACY_ACTIVE_SCHEMA, ACTIVE_SCHEMA]);
const LEGACY_REVIEW_STATUS = ['ACCEP', 'TED'].join('');

const AUTHORITY_LIMITS = Object.freeze({ maxBytes: 4 * 1024, maxLines: 60 });
const CURRENT_LIMITS = Object.freeze({ maxBytes: 8 * 1024, maxLines: 100 });
const ACTIVE_LIMITS = Object.freeze({
  headerOnly: true,
  maxBytes: 4 * 1024,
  maxLines: 60,
  bodySoftMaxBytes: 32 * 1024
});

const ALLOWED_MODES = new Set(['QUICK', 'PLAN', 'GUIDANCE', 'DEVELOPMENT']);
const ALLOWED_WORK_STATES = new Set([
  'READY',
  'IN_PROGRESS',
  'WAITING_EXTERNAL_CONDITION',
  'BLOCKED',
  'COMPLETE'
]);
const REQUIRED_AUTHORITY_POINTERS = [
  'project-rules',
  'current-state',
  'active-authority',
  'master-plan',
  'stage-plan',
  'code-map'
];
const OPTIONAL_POINTERS = new Set(['master-plan', 'stage-plan', 'mistake-ledger']);
const KNOWN_AUTHORITY_POINTERS = new Set([
  ...REQUIRED_AUTHORITY_POINTERS,
  ...OPTIONAL_POINTERS
]);
const ACTIVE_PREFIXES = ['docs/plans/active/', 'docs/task-packages/active/'];
const TASK_PACKAGE_ACTIVE_PREFIX = 'docs/task-packages/active/';
const TASK_PACKAGE_HEADER_FIELDS = Object.freeze([
  'authority-schema',
  'authority-id',
  'authority-status',
  'outcome',
  'mode',
  'owner',
  'acceptance-owner',
  'accepted-by',
  'updated-at',
  'goal',
  'next-action',
  'git-disposition'
]);
const TASK_PACKAGE_OUTCOMES = new Set([
  'PENDING',
  'COMPLETED',
  'BLOCKED',
  'STOPPED'
]);
const TASK_PACKAGE_MODES = new Set(['PLAN', 'GUIDANCE', 'DEVELOPMENT']);
const TASK_PACKAGE_GIT_DISPOSITIONS = new Set([
  'PENDING',
  'NO_CODE',
  'LOCAL_COMMIT',
  'WIP',
  'INTEGRATED'
]);
const TASK_PACKAGE_FINAL_OUTCOMES = new Set(['COMPLETED', 'BLOCKED', 'STOPPED']);
const TASK_PACKAGE_COMPLETE_DISPOSITIONS = new Set([
  'NO_CODE',
  'LOCAL_COMMIT',
  'WIP',
  'INTEGRATED'
]);
const FORBIDDEN_PATH_SEGMENTS = new Set([
  'archive',
  'archived',
  'evidence',
  'historical',
  'history',
  'superseded'
]);

function issue(source, code) {
  return { source, code };
}

function readableActiveStatus(value) {
  return value === 'DRAFT'
    || value === 'ACTIVE'
    || value === 'COMPLETE'
    || value === LEGACY_REVIEW_STATUS;
}

function projectedActiveStatus(value) {
  // The retired v0.4 review marker is not a v0.5 lifecycle value.
  // Readers expose it only as source data and report an advisory; no write or
  // exit decision may depend on it.
  return value;
}

function lineCount(text) {
  if (!text) return 0;
  const normalized = text.endsWith('\n') ? text.slice(0, -1) : text;
  return normalized ? normalized.split(/\r?\n/).length : 0;
}

function toPosix(value) {
  return String(value || '').split(path.sep).join('/').replace(/^\.\//, '');
}

function safeRelativePath(value) {
  const normalized = toPosix(value);
  if (
    !normalized ||
    normalized.startsWith('/') ||
    normalized.includes('\\') ||
    normalized.includes('\0')
  ) return null;
  if (
    normalized.split('/').includes('..') ||
    path.posix.normalize(normalized) !== normalized
  ) return null;
  return normalized;
}

function inside(root, candidate) {
  return candidate === root || candidate.startsWith(`${root}${path.sep}`);
}

function resolveRoot(target) {
  const resolved = path.resolve(target);
  try {
    return fs.realpathSync(resolved);
  } catch {
    return resolved;
  }
}

function resolveSafeFile(root, relativePath) {
  const safePath = safeRelativePath(relativePath);
  if (!safePath) return { ok: false, code: 'UNSAFE_PATH' };

  const joined = path.resolve(root, safePath);
  if (!inside(root, joined)) return { ok: false, code: 'OUTSIDE_REPOSITORY' };

  let cursor = root;
  try {
    for (const segment of safePath.split('/')) {
      cursor = path.join(cursor, segment);
      const stat = fs.lstatSync(cursor);
      if (stat.isSymbolicLink()) return { ok: false, code: 'SYMLINK_FORBIDDEN' };
    }
    const stat = fs.statSync(joined);
    if (!stat.isFile()) return { ok: false, code: 'NOT_A_FILE' };
    return { ok: true, path: joined, stat };
  } catch {
    return { ok: false, code: 'MISSING' };
  }
}

function readSafeText(root, relativePath, limits) {
  const resolved = resolveSafeFile(root, relativePath);
  if (!resolved.ok) return resolved;
  if (limits.headerOnly) {
    let descriptor;
    try {
      descriptor = fs.openSync(resolved.path, 'r');
      const openedStat = fs.fstatSync(descriptor);
      const bodyMarkerBytes = Buffer.byteLength('## ', 'utf8');
      const capacity = Math.min(
        openedStat.size,
        limits.maxBytes + bodyMarkerBytes
      );
      const buffer = Buffer.allocUnsafe(capacity);
      let bytesRead = 0;
      while (bytesRead < capacity) {
        const chunkBytes = fs.readSync(
          descriptor,
          buffer,
          bytesRead,
          capacity - bytesRead,
          bytesRead
        );
        if (!chunkBytes) break;
        bytesRead += chunkBytes;
      }
      const prefix = buffer.subarray(0, bytesRead).toString('utf8');
      const section = /^##[^\S\r\n]+.*$/m.exec(prefix);

      if (!section && openedStat.size > limits.maxBytes) {
        return { ok: false, code: 'HEADER_TOO_LARGE', bytes: openedStat.size };
      }

      const text = section ? prefix.slice(0, section.index) : prefix;
      const bytes = Buffer.byteLength(text, 'utf8');
      const lines = lineCount(text);
      if (bytes > limits.maxBytes) {
        return { ok: false, code: 'HEADER_TOO_LARGE', bytes };
      }
      if (lines > limits.maxLines) {
        return { ok: false, code: 'HEADER_TOO_MANY_LINES', lines };
      }
      if (text.includes('\0')) return { ok: false, code: 'INVALID_TEXT' };

      const bodyBytes = Math.max(0, openedStat.size - bytes);
      const advisories = bodyBytes > limits.bodySoftMaxBytes
        ? ['BODY_SOFT_LIMIT_EXCEEDED']
        : [];
      return {
        ok: true,
        text,
        bytes,
        lines,
        bodyBytes,
        hasBodySection: Boolean(section),
        advisories
      };
    } catch {
      return { ok: false, code: 'READ_FAILED' };
    } finally {
      if (descriptor !== undefined) fs.closeSync(descriptor);
    }
  }
  if (resolved.stat.size > limits.maxBytes) {
    return { ok: false, code: 'TOO_LARGE', bytes: resolved.stat.size };
  }
  try {
    const text = fs.readFileSync(resolved.path, 'utf8');
    if (lineCount(text) > limits.maxLines) {
      return { ok: false, code: 'TOO_MANY_LINES', lines: lineCount(text) };
    }
    if (text.includes('\0')) return { ok: false, code: 'INVALID_TEXT' };
    return {
      ok: true,
      text,
      bytes: Buffer.byteLength(text, 'utf8'),
      lines: lineCount(text)
    };
  } catch {
    return { ok: false, code: 'READ_FAILED' };
  }
}

function sanitizeText(value, maxChars = 240) {
  return sanitizeOutputText(value, maxChars);
}

function parseMetadata(text) {
  const values = {};
  const duplicates = [];
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (/^##\s+/.test(line)) break;
    const match = line.match(/^([a-z][a-z0-9-]*):\s*(.*?)\s*$/);
    if (!match) continue;
    if (Object.prototype.hasOwnProperty.call(values, match[1])) duplicates.push(match[1]);
    else values[match[1]] = match[2];
  }
  return { values, duplicates };
}

function sectionLines(text, heading) {
  const lines = text.split(/\r?\n/);
  const starts = lines
    .map((line, index) => (line.trim() === `## ${heading}` ? index : -1))
    .filter((index) => index >= 0);
  const start = starts[0];
  if (start === undefined) return { found: false, count: 0, lines: [] };
  const selected = [];
  for (let index = start + 1; index < lines.length; index += 1) {
    const line = lines[index].trim();
    if (/^##\s+/.test(line)) break;
    if (line) selected.push(line);
  }
  return { found: true, count: starts.length, lines: selected };
}

function parseBulletList(text, heading) {
  const section = sectionLines(text, heading);
  if (!section.found) return { found: false, count: 0, items: [], invalid: [] };
  const items = [];
  const invalid = [];
  for (const line of section.lines) {
    if (line.startsWith('<!--')) continue;
    const match = line.match(/^[-*]\s+(.+)$/);
    if (!match) {
      invalid.push(line);
      continue;
    }
    const value = sanitizeText(match[1]);
    if (value && value.toLowerCase() !== 'none') items.push(value);
  }
  return { found: true, count: section.count, items, invalid };
}

function parsePointerSection(text) {
  const section = sectionLines(text, 'Canonical');
  const pointers = {};
  const duplicates = [];
  const invalid = [];
  if (!section.found) {
    return { found: false, count: 0, pointers, duplicates, invalid };
  }
  for (const line of section.lines) {
    if (line.startsWith('<!--')) continue;
    const match = line.match(/^[-*]\s+([a-z][a-z0-9-]*):\s*(\S+)\s*$/);
    if (!match) {
      invalid.push(line);
      continue;
    }
    if (Object.prototype.hasOwnProperty.call(pointers, match[1])) duplicates.push(match[1]);
    else pointers[match[1]] = match[2];
  }
  return { found: true, count: section.count, pointers, duplicates, invalid };
}

function validTimestamp(value) {
  return (
    typeof value === 'string' &&
    /(?:Z|[+-]\d{2}:\d{2})$/.test(value) &&
    Number.isFinite(Date.parse(value))
  );
}

function validActiveId(value) {
  return value === 'NONE' || /^[A-Z0-9][A-Z0-9._-]{1,79}$/.test(value || '');
}

function pathHasForbiddenSegment(relativePath) {
  return relativePath
    .toLowerCase()
    .split('/')
    .some((segment) => FORBIDDEN_PATH_SEGMENTS.has(segment));
}

function allowedActiveAuthority(relativePath) {
  if (relativePath === CURRENT_PATH) return true;
  if (!relativePath.endsWith('.md') || pathHasForbiddenSegment(relativePath)) return false;
  return ACTIVE_PREFIXES.some((prefix) => relativePath.startsWith(prefix));
}

function validatePointer(name, value, source, issues) {
  if (OPTIONAL_POINTERS.has(name) && value === 'none') return null;
  const safePath = safeRelativePath(value);
  if (!safePath) {
    issues.push(issue(source, `INVALID_${name.toUpperCase().replace(/-/g, '_')}_PATH`));
    return null;
  }
  if (pathHasForbiddenSegment(safePath)) {
    issues.push(issue(source, `${name.toUpperCase().replace(/-/g, '_')}_PATH_FORBIDDEN`));
  }
  return safePath;
}

function parseAuthority(text) {
  const source = AUTHORITY_PATH;
  const issues = [];
  const metadata = parseMetadata(text);
  const canonical = parsePointerSection(text);

  if (metadata.duplicates.length) issues.push(issue(source, 'DUPLICATE_METADATA'));
  if (metadata.values.schema !== AUTHORITY_SCHEMA) {
    issues.push(issue(source, 'UNSUPPORTED_SCHEMA'));
  }
  if (!sanitizeText(metadata.values.project, 120)) {
    issues.push(issue(source, 'PROJECT_REQUIRED'));
  }
  if (!validTimestamp(metadata.values['updated-at'])) {
    issues.push(issue(source, 'UPDATED_AT_INVALID'));
  }
  if (!canonical.found) issues.push(issue(source, 'CANONICAL_SECTION_REQUIRED'));
  if (canonical.count > 1) issues.push(issue(source, 'DUPLICATE_CANONICAL_SECTION'));
  if (canonical.duplicates.length) issues.push(issue(source, 'DUPLICATE_CANONICAL_POINTER'));
  if (canonical.invalid.length) issues.push(issue(source, 'INVALID_CANONICAL_LINE'));
  if (Object.keys(canonical.pointers).some((key) => !KNOWN_AUTHORITY_POINTERS.has(key))) {
    issues.push(issue(source, 'UNKNOWN_CANONICAL_POINTER'));
  }

  for (const key of REQUIRED_AUTHORITY_POINTERS) {
    if (!Object.prototype.hasOwnProperty.call(canonical.pointers, key)) {
      issues.push(issue(source, `MISSING_${key.toUpperCase().replace(/-/g, '_')}`));
    }
  }

  const pointers = {};
  for (const key of KNOWN_AUTHORITY_POINTERS) {
    if (Object.prototype.hasOwnProperty.call(canonical.pointers, key)) {
      pointers[key] = validatePointer(key, canonical.pointers[key], source, issues);
    }
  }

  if (pointers['project-rules'] && pointers['project-rules'] !== 'AGENTS.md') {
    issues.push(issue(source, 'PROJECT_RULES_MUST_BE_AGENTS'));
  }
  if (pointers['current-state'] && pointers['current-state'] !== CURRENT_PATH) {
    issues.push(issue(source, 'CURRENT_STATE_PATH_MISMATCH'));
  }
  if (
    pointers['active-authority'] &&
    !allowedActiveAuthority(pointers['active-authority'])
  ) {
    issues.push(issue(source, 'ACTIVE_AUTHORITY_FORBIDDEN'));
  }
  if (
    pointers['code-map'] &&
    pathHasForbiddenSegment(pointers['code-map'])
  ) {
    issues.push(issue(source, 'CODE_MAP_PATH_FORBIDDEN'));
  }
  if (
    pointers['code-map'] &&
    pointers['code-map'] !== 'docs/code-map/index.json'
  ) {
    issues.push(issue(source, 'CODE_MAP_PATH_MISMATCH'));
  }
  if (
    pointers['mistake-ledger'] &&
    pathHasForbiddenSegment(pointers['mistake-ledger'])
  ) {
    issues.push(issue(source, 'MISTAKE_LEDGER_PATH_FORBIDDEN'));
  }
  if (
    pointers['mistake-ledger'] &&
    pointers['mistake-ledger'] !== 'docs/harness/MISTAKES.md'
  ) {
    issues.push(issue(source, 'MISTAKE_LEDGER_PATH_MISMATCH'));
  }

  return {
    value: {
      project: sanitizeText(metadata.values.project, 120),
      updatedAt: metadata.values['updated-at'] || null,
      pointers
    },
    issues
  };
}

function parseCurrent(text) {
  const source = CURRENT_PATH;
  const issues = [];
  const metadata = parseMetadata(text);
  const status = parseBulletList(text, 'Status');
  const blockers = parseBulletList(text, 'Blockers');
  const nextAction = parseBulletList(text, 'Next action');
  const safety = parseBulletList(text, 'Safety');

  if (metadata.duplicates.length) issues.push(issue(source, 'DUPLICATE_METADATA'));
  if (metadata.values.schema !== CURRENT_SCHEMA) {
    issues.push(issue(source, 'UNSUPPORTED_SCHEMA'));
  }
  if (!validTimestamp(metadata.values['updated-at'])) {
    issues.push(issue(source, 'UPDATED_AT_INVALID'));
  }
  if (!ALLOWED_MODES.has(metadata.values.mode)) {
    issues.push(issue(source, 'MODE_INVALID'));
  }
  if (!ALLOWED_WORK_STATES.has(metadata.values['work-state'])) {
    issues.push(issue(source, 'WORK_STATE_INVALID'));
  }
  if (!validActiveId(metadata.values['active-id'])) {
    issues.push(issue(source, 'ACTIVE_ID_INVALID'));
  }
  if (!sanitizeText(metadata.values.phase)) issues.push(issue(source, 'PHASE_REQUIRED'));
  if (!sanitizeText(metadata.values.goal)) issues.push(issue(source, 'GOAL_REQUIRED'));

  for (const [name, parsed] of [
    ['STATUS', status],
    ['BLOCKERS', blockers],
    ['NEXT_ACTION', nextAction],
    ['SAFETY', safety]
  ]) {
    if (!parsed.found) issues.push(issue(source, `${name}_SECTION_REQUIRED`));
    if (parsed.count > 1) issues.push(issue(source, `DUPLICATE_${name}_SECTION`));
    if (parsed.invalid.length) issues.push(issue(source, `${name}_SECTION_INVALID`));
  }
  if (!status.items.length || status.items.length > 5) {
    issues.push(issue(source, 'STATUS_COUNT_INVALID'));
  }
  if (blockers.items.length > 3) issues.push(issue(source, 'BLOCKER_COUNT_INVALID'));
  if (nextAction.items.length !== 1) {
    issues.push(issue(source, 'NEXT_ACTION_COUNT_INVALID'));
  }
  if (safety.items.length > 2) issues.push(issue(source, 'SAFETY_COUNT_INVALID'));

  const forbiddenHeadings = text
    .split(/\r?\n/)
    .filter((line) => /^##\s+/.test(line.trim()))
    .some((line) => /(?:history|historical|archive|历史|归档)/i.test(line));
  if (forbiddenHeadings) issues.push(issue(source, 'HISTORY_SECTION_FORBIDDEN'));

  return {
    value: {
      updatedAt: metadata.values['updated-at'] || null,
      mode: metadata.values.mode || null,
      workState: metadata.values['work-state'] || null,
      activeId: metadata.values['active-id'] || null,
      phase: sanitizeText(metadata.values.phase),
      goal: sanitizeText(metadata.values.goal),
      status: status.items,
      blockers: blockers.items,
      nextAction: nextAction.items[0] || null,
      safety: safety.items
    },
    issues
  };
}

function taskPackageTitle(text) {
  const lines = text.split(/\r?\n/);
  const bodyIndex = lines.findIndex((line) => /^##\s+/.test(line));
  const headerLines = bodyIndex === -1 ? lines : lines.slice(0, bodyIndex);
  const headings = headerLines
    .filter((line) => /^#(?!#)\s+/.test(line.trim()));
  if (headings.length !== 1) {
    return { id: null, title: null, issue: 'TITLE_HEADING_REQUIRED' };
  }
  const match = headings[0].trim().match(
    /^#\s+Task Package:\s+([A-Z0-9][A-Z0-9._-]{1,79})\s+(?:—|-)\s+(.+?)\s*$/
  );
  if (!match) return { id: null, title: null, issue: 'TITLE_FORMAT_INVALID' };
  return { id: match[1], title: match[2], issue: null };
}

function validTaskPackageLine(value, maxChars) {
  return (
    typeof value === 'string' &&
    value.length > 0 &&
    value.length <= maxChars &&
    !/[\0\r\n]/.test(value)
  );
}

/*
 * harness-active/v3 is deliberately path-sensitive:
 * - docs/plans/active keeps the small generic routing tuple;
 * - docs/task-packages/active must satisfy the complete task-package header.
 *
 * Keep this standalone because the task-package component is optional while the
 * context router is part of the generic pack.
 */
function validateTaskPackageActiveHeader(
  text,
  relativePath,
  metadata,
  options = {}
) {
  const issues = [];
  const fields = Object.keys(metadata.values);
  const unknown = fields.filter((field) => !TASK_PACKAGE_HEADER_FIELDS.includes(field));
  if (metadata.duplicates.length) {
    issues.push(issue(relativePath, 'PACKAGE_DUPLICATE_METADATA'));
  }
  if (unknown.length) issues.push(issue(relativePath, 'PACKAGE_UNKNOWN_METADATA'));

  for (const field of TASK_PACKAGE_HEADER_FIELDS) {
    if (!Object.prototype.hasOwnProperty.call(metadata.values, field)) {
      issues.push(issue(
        relativePath,
        `PACKAGE_${field.toUpperCase().replace(/-/g, '_')}_REQUIRED`
      ));
    }
  }

  const title = taskPackageTitle(text);
  if (title.issue) issues.push(issue(relativePath, title.issue));
  if (
    title.id &&
    metadata.values['authority-id'] &&
    title.id !== metadata.values['authority-id']
  ) {
    issues.push(issue(relativePath, 'PACKAGE_TITLE_ID_MISMATCH'));
  }

  const hasBodySection = options.hasBodySection === undefined
    ? text.split(/\r?\n/).some((line) => /^##\s+/.test(line))
    : options.hasBodySection;
  if (!hasBodySection) issues.push(issue(relativePath, 'PACKAGE_BODY_SECTION_REQUIRED'));

  const outcome = metadata.values.outcome;
  const mode = metadata.values.mode;
  const sourceStatus = metadata.values['authority-status'];
  const legacyReviewProjection = sourceStatus === LEGACY_REVIEW_STATUS;
  const status = projectedActiveStatus(sourceStatus);
  const gitDisposition = metadata.values['git-disposition'];
  const acceptedBy = metadata.values['accepted-by'];
  if (!validActiveId(metadata.values['authority-id'])) {
    issues.push(issue(relativePath, 'PACKAGE_ID_INVALID'));
  }
  if (!readableActiveStatus(sourceStatus)) {
    issues.push(issue(relativePath, 'PACKAGE_STATUS_INVALID'));
  }
  if (!TASK_PACKAGE_OUTCOMES.has(outcome)) {
    issues.push(issue(relativePath, 'PACKAGE_OUTCOME_INVALID'));
  }
  if (!TASK_PACKAGE_MODES.has(mode)) {
    issues.push(issue(relativePath, 'PACKAGE_MODE_INVALID'));
  }
  if (!TASK_PACKAGE_GIT_DISPOSITIONS.has(gitDisposition)) {
    issues.push(issue(relativePath, 'PACKAGE_GIT_DISPOSITION_INVALID'));
  }
  if (!validTimestamp(metadata.values['updated-at'])) {
    issues.push(issue(relativePath, 'PACKAGE_UPDATED_AT_INVALID'));
  }
  for (const [value, field, maxChars] of [
    [title.title, 'TITLE', 160],
    [metadata.values.owner, 'OWNER', 120],
    [metadata.values['acceptance-owner'], 'ACCEPTANCE_OWNER', 120],
    [metadata.values.goal, 'GOAL', 320],
    [metadata.values['next-action'], 'NEXT_ACTION', 320]
  ]) {
    if (!validTaskPackageLine(value, maxChars)) {
      issues.push(issue(relativePath, `PACKAGE_${field}_INVALID`));
    }
  }

  if (
    !legacyReviewProjection &&
    (status === 'DRAFT' || status === 'ACTIVE') &&
    outcome !== 'PENDING'
  ) {
    issues.push(issue(relativePath, 'PACKAGE_OPEN_STATUS_REQUIRES_PENDING_OUTCOME'));
  }
  if (
    status === 'COMPLETE' &&
    !TASK_PACKAGE_FINAL_OUTCOMES.has(outcome)
  ) {
    issues.push(issue(relativePath, 'PACKAGE_TERMINAL_STATUS_REQUIRES_FINAL_OUTCOME'));
  }
  if (!legacyReviewProjection && status !== 'COMPLETE' && gitDisposition !== 'PENDING') {
    issues.push(issue(relativePath, 'PACKAGE_GIT_DISPOSITION_PREMATURE'));
  }
  if (status === 'COMPLETE' && !TASK_PACKAGE_COMPLETE_DISPOSITIONS.has(gitDisposition)) {
    issues.push(issue(relativePath, 'PACKAGE_COMPLETE_REQUIRES_GIT_DISPOSITION'));
  }
  if (
    !legacyReviewProjection &&
    (status === 'DRAFT' || status === 'ACTIVE') &&
    acceptedBy !== 'PENDING'
  ) {
    issues.push(issue(relativePath, 'PACKAGE_OPEN_STATUS_REQUIRES_PENDING_ACCEPTED_BY'));
  }
  if (
    status === 'COMPLETE' &&
    (
      !acceptedBy ||
      acceptedBy === 'PENDING' ||
      acceptedBy.length > 120 ||
      /[\0\r\n]/.test(acceptedBy)
    )
  ) {
    issues.push(issue(relativePath, 'PACKAGE_TERMINAL_STATUS_REQUIRES_ACCEPTED_BY'));
  }

  const id = metadata.values['authority-id'];
  if (
    validActiveId(id) &&
    id !== 'NONE' &&
    (status === 'ACTIVE' || legacyReviewProjection) &&
    relativePath !== `${TASK_PACKAGE_ACTIVE_PREFIX}${id}.md`
  ) {
    issues.push(issue(relativePath, 'PACKAGE_LOCATION_STATUS_MISMATCH'));
  } else if (
    validActiveId(id) &&
    id !== 'NONE' &&
    (status === 'DRAFT' || status === 'COMPLETE')
  ) {
    issues.push(issue(relativePath, 'PACKAGE_LOCATION_STATUS_MISMATCH'));
  }

  return issues;
}

function parseActiveAuthority(text, relativePath, options = {}) {
  const issues = [];
  const metadata = parseMetadata(text);
  const schema = metadata.values['authority-schema'];
  const id = metadata.values['authority-id'];
  const sourceStatus = metadata.values['authority-status'];
  const status = projectedActiveStatus(sourceStatus);
  const statusValid = schema === LEGACY_ACTIVE_SCHEMA
    ? sourceStatus === 'ACTIVE'
    : readableActiveStatus(sourceStatus);
  if (metadata.duplicates.length) issues.push(issue(relativePath, 'DUPLICATE_METADATA'));
  if (!ACTIVE_SCHEMAS.has(schema)) {
    issues.push(issue(relativePath, 'ACTIVE_SCHEMA_INVALID'));
  }
  if (!validActiveId(id) || id === 'NONE') {
    issues.push(issue(relativePath, 'ACTIVE_ID_INVALID'));
  }
  if (!statusValid) {
    issues.push(issue(relativePath, 'ACTIVE_STATUS_INVALID'));
  }
  if (
    schema === ACTIVE_SCHEMA &&
    relativePath.startsWith(TASK_PACKAGE_ACTIVE_PREFIX)
  ) {
    issues.push(...validateTaskPackageActiveHeader(
      text,
      relativePath,
      metadata,
      options
    ));
  }
  return {
    value: {
      schema: ACTIVE_SCHEMAS.has(schema) ? schema : null,
      id: validActiveId(id) && id !== 'NONE' ? id : null,
      status: statusValid ? status : null
    },
    issues
  };
}

function loadCoreContext(target) {
  const root = resolveRoot(target);
  const issues = [];
  const advisories = [];

  const authorityDocument = readSafeText(root, AUTHORITY_PATH, AUTHORITY_LIMITS);
  const currentDocument = readSafeText(root, CURRENT_PATH, CURRENT_LIMITS);

  let authority = {
    project: sanitizeText(path.basename(root), 120) || 'project',
    updatedAt: null,
    pointers: {}
  };
  let activeAuthority = null;
  let current = {
    updatedAt: null,
    mode: null,
    workState: null,
    activeId: null,
    phase: '',
    goal: '',
    status: [],
    blockers: [],
    nextAction: null,
    safety: []
  };

  if (!authorityDocument.ok) {
    issues.push(issue(AUTHORITY_PATH, `AUTHORITY_${authorityDocument.code}`));
  } else {
    const parsed = parseAuthority(authorityDocument.text);
    authority = parsed.value;
    issues.push(...parsed.issues);
  }

  if (!currentDocument.ok) {
    issues.push(issue(CURRENT_PATH, `CURRENT_${currentDocument.code}`));
  } else {
    const parsed = parseCurrent(currentDocument.text);
    current = parsed.value;
    issues.push(...parsed.issues);
  }

  const activePath = authority.pointers['active-authority'] || null;
  if (activePath === CURRENT_PATH) {
    if (current.activeId !== 'NONE') {
      issues.push(issue(CURRENT_PATH, 'SELF_AUTHORITY_REQUIRES_NONE_ACTIVE_ID'));
    }
  } else if (activePath && allowedActiveAuthority(activePath)) {
    const activeDocument = readSafeText(root, activePath, ACTIVE_LIMITS);
    if (!activeDocument.ok) {
      issues.push(issue(activePath, `ACTIVE_AUTHORITY_${activeDocument.code}`));
    } else {
      for (const code of activeDocument.advisories) {
        advisories.push(issue(activePath, `ACTIVE_AUTHORITY_${code}`));
      }
      const parsed = parseActiveAuthority(activeDocument.text, activePath, {
        hasBodySection: activeDocument.hasBodySection
      });
      activeAuthority = {
        path: activePath,
        ...parsed.value
      };
      issues.push(...parsed.issues);
      if (parsed.value.id && current.activeId && parsed.value.id !== current.activeId) {
        issues.push(issue(activePath, 'ACTIVE_ID_MISMATCH'));
      }
      if (
        parsed.value.status
        && parsed.value.status !== 'ACTIVE'
        && parsed.value.status !== LEGACY_REVIEW_STATUS
      ) {
        issues.push(issue(
          activePath,
          parsed.value.status === 'DRAFT'
            ? 'DRAFT_AUTHORITY_CANNOT_BE_CURRENT'
            : 'COMPLETE_AUTHORITY_CANNOT_BE_CURRENT'
        ));
      } else if (parsed.value.status === LEGACY_REVIEW_STATUS) {
        advisories.push(issue(activePath, 'ACCEPTED_AUTHORITY_PENDING_EXIT'));
      }
    }
  }

  const uniqueIssues = issues.filter((entry, index) => (
    issues.findIndex((candidate) => (
      candidate.source === entry.source && candidate.code === entry.code
    )) === index
  ));
  const uniqueAdvisories = advisories.filter((entry, index) => (
    advisories.findIndex((candidate) => (
      candidate.source === entry.source && candidate.code === entry.code
    )) === index
  ));

  return {
    schemaVersion: 2,
    coreStatus: uniqueIssues.length ? 'DEGRADED' : 'OK',
    readOnly: true,
    root,
    repository: {
      name: authority.project || sanitizeText(path.basename(root), 120) || 'project'
    },
    authority: uniqueIssues.length ? null : activePath,
    activeAuthority,
    current,
    pointers: authority.pointers,
    issues: uniqueIssues,
    advisories: uniqueAdvisories
  };
}

module.exports = {
  ACTIVE_LIMITS,
  ACTIVE_SCHEMA,
  AUTHORITY_LIMITS,
  AUTHORITY_PATH,
  AUTHORITY_SCHEMA,
  CURRENT_LIMITS,
  CURRENT_PATH,
  CURRENT_SCHEMA,
  LEGACY_ACTIVE_SCHEMA,
  allowedActiveAuthority,
  lineCount,
  loadCoreContext,
  parseActiveAuthority,
  parseAuthority,
  parseCurrent,
  readSafeText,
  resolveRoot,
  resolveSafeFile,
  safeRelativePath,
  sanitizeText
};
