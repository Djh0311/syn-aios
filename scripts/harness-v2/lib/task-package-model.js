'use strict';

const crypto = require('node:crypto');
const path = require('node:path');

const PACKAGE_SCHEMA = 'harness-active/v3';
const LEGACY_PACKAGE_SCHEMA = 'harness-active/v2';
const AUTHORITY_SCHEMA = 'harness-authority/v2';
const CURRENT_SCHEMA = 'harness-current/v2';

const AUTHORITY_PATH = 'docs/harness/AUTHORITY.md';
const CURRENT_PATH = 'docs/harness/CURRENT.md';
const PACKAGE_ROOT = 'docs/task-packages';

const AUTHORITY_MAX_BYTES = 4 * 1024;
const AUTHORITY_MAX_LINES = 60;
const CURRENT_MAX_BYTES = 8 * 1024;
const CURRENT_MAX_LINES = 100;
const HEADER_MAX_BYTES = 4 * 1024;
const HEADER_MAX_LINES = 60;
const BODY_SOFT_MAX_BYTES = 32 * 1024;
const LEGACY_REVIEW_STATUS = ['ACCEP', 'TED'].join('');

const VALID_OUTCOMES = new Set(['PENDING', 'COMPLETED', 'BLOCKED', 'STOPPED']);
const VALID_MODES = new Set(['QUICK', 'PLAN', 'GUIDANCE', 'DEVELOPMENT']);
const VALID_TASK_MODES = new Set(['PLAN', 'GUIDANCE', 'DEVELOPMENT']);
const VALID_GIT_DISPOSITIONS = new Set([
  'PENDING',
  'NO_CODE',
  'LOCAL_COMMIT',
  'WIP',
  'INTEGRATED',
]);
const HEADER_FIELDS = Object.freeze([
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
  'git-disposition',
]);
const REQUIRED_V3_FIELDS = new Set(HEADER_FIELDS);
const ID_PATTERN = /^[A-Z0-9][A-Z0-9._-]{1,79}$/;

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function lineCount(text) {
  if (!text) return 0;
  const normalized = text.endsWith('\n') ? text.slice(0, -1) : text;
  return normalized ? normalized.split(/\r?\n/).length : 0;
}

function validTimestamp(value) {
  return (
    typeof value === 'string' &&
    /(?:Z|[+-]\d{2}:\d{2})$/.test(value) &&
    Number.isFinite(Date.parse(value))
  );
}

function validateId(value) {
  const id = String(value || '').trim().toUpperCase();
  if (!ID_PATTERN.test(id)) {
    throw new Error('task id must be 2-80 uppercase letters, numbers, dots, underscores, or hyphens');
  }
  return id;
}

function normalizeSingleLine(value, label, maxChars) {
  const text = String(value || '').replace(/\s+/g, ' ').trim();
  if (!text) throw new Error(`${label} must not be empty`);
  if (/[\0\r\n]/.test(String(value))) throw new Error(`${label} must be a single line`);
  if (/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/.test(text)) {
    throw new Error(`${label} contains unsupported control characters`);
  }
  if (text.length > maxChars) throw new Error(`${label} must be at most ${maxChars} characters`);
  return text;
}

function packageRelativePath(id, status) {
  const normalizedId = validateId(id);
  if (status === 'DRAFT') return `${PACKAGE_ROOT}/drafts/${normalizedId}.md`;
  if (status === 'ACTIVE' || status === LEGACY_REVIEW_STATUS) {
    return `${PACKAGE_ROOT}/active/${normalizedId}.md`;
  }
  if (status === 'COMPLETE') return `${PACKAGE_ROOT}/archive/${normalizedId}.md`;
  throw new Error(`unsupported task package status: ${status}`);
}

function readableStatus(value) {
  return value === 'DRAFT'
    || value === 'ACTIVE'
    || value === 'COMPLETE'
    || value === LEGACY_REVIEW_STATUS;
}

function compatibilityStatus(value) {
  // v0.5 does not have a review lifecycle axis. A persisted v0.4/v3 package
  // carrying the retired review marker remains readable from its old location, but is
  // exposed only as source data and can never authorize a write or gate an exit.
  return value;
}

function splitHeaderAndBody(text) {
  const lines = String(text || '').replace(/\r\n/g, '\n').split('\n');
  const bodyIndex = lines.findIndex((line) => /^##[^\S\r\n]+/.test(line));
  const headerLines = bodyIndex === -1 ? lines : lines.slice(0, bodyIndex);
  const bodyLines = bodyIndex === -1 ? [] : lines.slice(bodyIndex);
  return {
    headerText: headerLines.join('\n'),
    headerLines,
    bodyText: bodyLines.join('\n'),
    bodyLines,
    bodyIndex,
  };
}

function parseMetadata(headerLines) {
  const values = {};
  const duplicates = [];
  const unknown = [];
  for (const rawLine of headerLines) {
    const line = rawLine.trim();
    const match = line.match(/^([a-z][a-z0-9-]*):\s*(.*?)\s*$/);
    if (!match) continue;
    const key = match[1];
    if (!HEADER_FIELDS.includes(key)) unknown.push(key);
    if (Object.prototype.hasOwnProperty.call(values, key)) duplicates.push(key);
    else values[key] = match[2];
  }
  return { values, duplicates, unknown };
}

function parseTitle(headerLines) {
  const headings = headerLines.filter((line) => /^#(?!#)\s+/.test(line.trim()));
  if (headings.length !== 1) {
    return { id: null, title: null, issue: 'TITLE_HEADING_REQUIRED' };
  }
  const match = headings[0].trim().match(
    /^#\s+Task Package:\s+([A-Z0-9][A-Z0-9._-]{1,79})\s+(?:—|-)\s+(.+?)\s*$/,
  );
  if (!match) return { id: null, title: null, issue: 'TITLE_FORMAT_INVALID' };
  return { id: match[1], title: match[2], issue: null };
}

function parseCloseout(bodyText) {
  const lines = String(bodyText || '').replace(/\r\n/g, '\n').split('\n');
  const starts = [];
  for (let index = 0; index < lines.length; index += 1) {
    if (lines[index] === '## Closeout') starts.push(index);
  }
  if (starts.length !== 1) {
    return { values: {}, commits: [], issues: ['PACKAGE_CLOSEOUT_SECTION_REQUIRED'] };
  }
  const start = starts[0];
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (/^##[^\S\r\n]+/.test(lines[index])) {
      end = index;
      break;
    }
  }
  const allowed = new Set([
    'Outcome',
    'Accepted by',
    'Git disposition',
    'Git commits',
    'Git reason',
    'Carryover goal',
    'Carryover action',
    'Carryover owner',
    'Harness closeout commit',
  ]);
  const values = {};
  const issues = [];
  for (const line of lines.slice(start + 1, end)) {
    if (!line.trim()) continue;
    const match = line.match(/^-\s+([^:]+):\s*(.*?)\s*$/);
    if (!match || !allowed.has(match[1])) {
      issues.push('PACKAGE_CLOSEOUT_FORMAT_INVALID');
      continue;
    }
    if (Object.prototype.hasOwnProperty.call(values, match[1])) {
      issues.push('PACKAGE_CLOSEOUT_DUPLICATE_FIELD');
    } else {
      values[match[1]] = match[2];
    }
  }
  for (const key of allowed) {
    if (!Object.prototype.hasOwnProperty.call(values, key)) {
      issues.push('PACKAGE_CLOSEOUT_FIELD_REQUIRED');
    }
  }
  const commits = String(values['Git commits'] || '')
    .split(',')
    .map((value) => value.trim().toLowerCase())
    .filter((value) => value && value !== 'none');
  if (commits.some((value) => !/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(value))) {
    issues.push('PACKAGE_CLOSEOUT_COMMIT_OID_INVALID');
  }
  if (new Set(commits).size !== commits.length) {
    issues.push('PACKAGE_CLOSEOUT_COMMIT_OID_DUPLICATE');
  }
  if (values['Harness closeout commit'] !== 'REQUIRED_AFTER_COMPLETE') {
    issues.push('PACKAGE_CLOSEOUT_COMMIT_REQUIRED');
  }
  return { values, commits, issues };
}

function validateLocation(relativePath, id, status, issues) {
  if (!relativePath) return;
  let expected;
  try {
    expected = packageRelativePath(id, status);
  } catch {
    return;
  }
  if (path.posix.normalize(relativePath) !== relativePath || relativePath !== expected) {
    issues.push('PACKAGE_LOCATION_STATUS_MISMATCH');
  }
}

function parseTaskPackage(text, options = {}) {
  const raw = String(text || '');
  const normalized = raw.replace(/\r\n/g, '\n');
  const bytes = Buffer.byteLength(raw, 'utf8');
  const lines = lineCount(raw);
  const issues = [];
  const warnings = [];
  if (normalized.includes('\0')) issues.push('PACKAGE_INVALID_TEXT');

  const split = splitHeaderAndBody(normalized);
  const rawBodyMatch = /^##[^\S\r\n]+/m.exec(raw);
  const rawHeader = rawBodyMatch ? raw.slice(0, rawBodyMatch.index) : raw;
  const rawBody = rawBodyMatch ? raw.slice(rawBodyMatch.index) : '';
  const headerBytes = Buffer.byteLength(rawHeader, 'utf8');
  const headerLines = lineCount(rawHeader);
  if (headerBytes > HEADER_MAX_BYTES) issues.push('PACKAGE_HEADER_TOO_LARGE');
  if (headerLines > HEADER_MAX_LINES) issues.push('PACKAGE_HEADER_TOO_MANY_LINES');
  if (split.bodyIndex === -1) issues.push('PACKAGE_BODY_SECTION_REQUIRED');

  const metadata = parseMetadata(split.headerLines);
  if (metadata.duplicates.length) issues.push('PACKAGE_DUPLICATE_METADATA');
  const schema = metadata.values['authority-schema'] || null;
  const legacy = schema === LEGACY_PACKAGE_SCHEMA;
  if (schema !== PACKAGE_SCHEMA && !legacy) issues.push('PACKAGE_SCHEMA_INVALID');
  if (!legacy && metadata.unknown.length) issues.push('PACKAGE_UNKNOWN_METADATA');

  const id = metadata.values['authority-id'] || null;
  if (!ID_PATTERN.test(id || '')) issues.push('PACKAGE_ID_INVALID');
  const sourceStatus = metadata.values['authority-status'] || null;
  const legacyReviewProjection = sourceStatus === LEGACY_REVIEW_STATUS;
  const status = compatibilityStatus(sourceStatus);
  if (!readableStatus(sourceStatus)) issues.push('PACKAGE_STATUS_INVALID');
  if (legacy && sourceStatus !== 'ACTIVE') issues.push('LEGACY_PACKAGE_STATUS_INVALID');

  const title = parseTitle(split.headerLines);
  if (!legacy && title.issue) issues.push(title.issue);
  if (!legacy && title.id && id && title.id !== id) issues.push('PACKAGE_TITLE_ID_MISMATCH');

  let outcome = metadata.values.outcome || (legacy ? 'PENDING' : null);
  let mode = metadata.values.mode || null;
  let owner = metadata.values.owner || null;
  let acceptanceOwner = metadata.values['acceptance-owner'] || null;
  let acceptedBy = metadata.values['accepted-by'] || null;
  let updatedAt = metadata.values['updated-at'] || null;
  let goal = metadata.values.goal || null;
  let nextAction = metadata.values['next-action'] || null;
  let gitDisposition = metadata.values['git-disposition'] || (legacy ? 'PENDING' : null);

  if (!legacy) {
    for (const field of REQUIRED_V3_FIELDS) {
      if (!Object.prototype.hasOwnProperty.call(metadata.values, field)) {
        issues.push(`PACKAGE_${field.toUpperCase().replace(/-/g, '_')}_REQUIRED`);
      }
    }
    if (!VALID_OUTCOMES.has(outcome)) issues.push('PACKAGE_OUTCOME_INVALID');
    if (!VALID_TASK_MODES.has(mode)) issues.push('PACKAGE_MODE_INVALID');
    if (!VALID_GIT_DISPOSITIONS.has(gitDisposition)) {
      issues.push('PACKAGE_GIT_DISPOSITION_INVALID');
    }
    if (!validTimestamp(updatedAt)) issues.push('PACKAGE_UPDATED_AT_INVALID');
    for (const [value, field, maxChars] of [
      [title.title, 'TITLE', 160],
      [owner, 'OWNER', 120],
      [acceptanceOwner, 'ACCEPTANCE_OWNER', 120],
      [goal, 'GOAL', 320],
      [nextAction, 'NEXT_ACTION', 320],
    ]) {
      if (!value || String(value).length > maxChars || /[\0\r\n]/.test(String(value || ''))) {
        issues.push(`PACKAGE_${field}_INVALID`);
      }
    }
    if (
      !legacyReviewProjection &&
      (status === 'DRAFT' || status === 'ACTIVE') &&
      outcome !== 'PENDING'
    ) {
      issues.push('PACKAGE_OPEN_STATUS_REQUIRES_PENDING_OUTCOME');
    }
    if (
      status === 'COMPLETE' &&
      !['COMPLETED', 'BLOCKED', 'STOPPED'].includes(outcome)
    ) {
      issues.push('PACKAGE_TERMINAL_STATUS_REQUIRES_FINAL_OUTCOME');
    }
    if (!legacyReviewProjection && status !== 'COMPLETE' && gitDisposition !== 'PENDING') {
      issues.push('PACKAGE_GIT_DISPOSITION_PREMATURE');
    }
    if (status === 'COMPLETE' && gitDisposition === 'PENDING') {
      issues.push('PACKAGE_COMPLETE_REQUIRES_GIT_DISPOSITION');
    }
    if (
      !legacyReviewProjection &&
      (status === 'DRAFT' || status === 'ACTIVE') &&
      acceptedBy !== 'PENDING'
    ) {
      issues.push('PACKAGE_OPEN_STATUS_REQUIRES_PENDING_ACCEPTED_BY');
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
      issues.push('PACKAGE_TERMINAL_STATUS_REQUIRES_ACCEPTED_BY');
    }
  }

  if (id && readableStatus(sourceStatus)) {
    validateLocation(options.relativePath || null, id, sourceStatus, issues);
  }

  const bodyBytes = Buffer.byteLength(rawBody, 'utf8');
  const bodyLines = split.bodyLines.length;
  if (bodyBytes > BODY_SOFT_MAX_BYTES) warnings.push('BODY_SOFT_LIMIT_EXCEEDED');
  let closeout = null;
  if (status === 'COMPLETE') {
    closeout = parseCloseout(split.bodyText);
    issues.push(...closeout.issues);
    const values = closeout.values;
    if (values.Outcome !== outcome) issues.push('PACKAGE_CLOSEOUT_OUTCOME_MISMATCH');
    if (values['Accepted by'] !== acceptedBy) {
      issues.push('PACKAGE_CLOSEOUT_ACCEPTED_BY_MISMATCH');
    }
    if (values['Git disposition'] !== gitDisposition) {
      issues.push('PACKAGE_CLOSEOUT_GIT_DISPOSITION_MISMATCH');
    }
    const hasCommits = closeout.commits.length > 0;
    const reason = values['Git reason'];
    if (gitDisposition === 'NO_CODE') {
      if (hasCommits) issues.push('PACKAGE_NO_CODE_FORBIDS_COMMITS');
      if (!reason || reason === 'none') issues.push('PACKAGE_NO_CODE_REASON_REQUIRED');
    } else if (!hasCommits) {
      issues.push('PACKAGE_GIT_COMMITS_REQUIRED');
    }
    const carryoverGoal = values['Carryover goal'];
    const carryoverAction = values['Carryover action'];
    const carryoverOwner = values['Carryover owner'];
    const carryoverRequired = gitDisposition === 'WIP' || outcome === 'BLOCKED';
    if (
      carryoverRequired &&
      (
        !carryoverGoal ||
        carryoverGoal === 'none' ||
        !carryoverAction ||
        carryoverAction === 'none' ||
        !carryoverOwner ||
        carryoverOwner === 'none'
      )
    ) {
      issues.push('PACKAGE_CARRYOVER_REQUIRED');
    }
    if (
      (carryoverGoal === 'none') !== (carryoverAction === 'none') ||
      (carryoverGoal === 'none') !== (carryoverOwner === 'none')
    ) {
      issues.push('PACKAGE_CARRYOVER_FIELDS_INCOMPLETE');
    }
    if (gitDisposition === 'WIP' && outcome === 'COMPLETED') {
      issues.push('PACKAGE_COMPLETED_OUTCOME_FORBIDS_WIP');
    }
  }

  return {
    ok: issues.length === 0,
    schema,
    legacy,
    id,
    status,
    sourceStatus,
    legacyReviewProjection,
    outcome,
    mode,
    owner,
    acceptanceOwner,
    acceptedBy,
    updatedAt,
    goal,
    nextAction,
    gitDisposition,
    title: title.title,
    relativePath: options.relativePath || null,
    bytes,
    lines,
    headerBytes,
    headerLines,
    bodyBytes,
    bodyLines,
    sha256: sha256(Buffer.from(raw, 'utf8')),
    issues: [...new Set(issues)],
    warnings: [...new Set(warnings)],
    text: normalized,
    bodyText: split.bodyText,
    closeout: closeout
      ? {
        outcome: closeout.values.Outcome || null,
        acceptedBy: closeout.values['Accepted by'] || null,
        gitDisposition: closeout.values['Git disposition'] || null,
        gitCommits: closeout.commits,
        gitReason: closeout.values['Git reason'] || null,
        carryoverGoal: closeout.values['Carryover goal'] || null,
        carryoverAction: closeout.values['Carryover action'] || null,
        carryoverOwner: closeout.values['Carryover owner'] || null,
        harnessCloseoutCommit:
          closeout.values['Harness closeout commit'] || null,
      }
      : null,
  };
}

function replaceTemplateToken(text, token, value) {
  const marker = `{{${token}}}`;
  if (!text.includes(marker)) throw new Error(`task package template is missing ${marker}`);
  return text.replaceAll(marker, value);
}

function renderDraftFromTemplate(template, input) {
  const id = validateId(input.id);
  const title = normalizeSingleLine(input.title, 'title', 160);
  const goal = normalizeSingleLine(input.goal, 'goal', 320);
  const mode = String(input.mode || '').trim().toUpperCase();
  if (!VALID_TASK_MODES.has(mode)) {
    throw new Error(`mode must be one of: ${[...VALID_TASK_MODES].join(', ')}`);
  }
  const owner = normalizeSingleLine(input.owner || 'N/A', 'owner', 120);
  const acceptanceOwner = normalizeSingleLine(
    input.acceptanceOwner,
    'acceptance owner',
    120,
  );
  const nextAction = normalizeSingleLine(
    input.nextAction || 'Review the contract and activate it only after its scope is authorized.',
    'next action',
    320,
  );
  const updatedAt = input.updatedAt || new Date().toISOString();
  if (!validTimestamp(updatedAt)) throw new Error('updated-at must be an ISO timestamp with timezone');

  let rendered = String(template || '').replace(/\r\n/g, '\n');
  for (const [token, value] of [
    ['TASK_ID', id],
    ['TASK_TITLE', title],
    ['MODE', mode],
    ['OWNER', owner],
    ['ACCEPTANCE_OWNER', acceptanceOwner],
    ['UPDATED_AT', updatedAt],
    ['GOAL', goal],
    ['NEXT_ACTION', nextAction],
  ]) {
    rendered = replaceTemplateToken(rendered, token, value);
  }
  if (/{{[A-Z0-9_]+}}/.test(rendered)) {
    throw new Error('task package template contains unresolved placeholders');
  }
  if (!rendered.endsWith('\n')) rendered += '\n';
  const relativePath = packageRelativePath(id, 'DRAFT');
  const parsed = parseTaskPackage(rendered, { relativePath });
  if (!parsed.ok) {
    throw new Error(`generated task package is invalid: ${parsed.issues.join(', ')}`);
  }
  return { text: rendered, parsed, relativePath };
}

function documentMetadata(text) {
  const split = splitHeaderAndBody(text);
  const values = {};
  const duplicates = [];
  for (const rawLine of split.headerLines) {
    const match = rawLine.trim().match(/^([a-z][a-z0-9-]*):\s*(.*?)\s*$/);
    if (!match) continue;
    if (Object.prototype.hasOwnProperty.call(values, match[1])) duplicates.push(match[1]);
    else values[match[1]] = match[2];
  }
  return { values, duplicates };
}

function sectionRange(lines, heading) {
  const indexes = [];
  for (let index = 0; index < lines.length; index += 1) {
    if (lines[index].trim() === `## ${heading}`) indexes.push(index);
  }
  if (indexes.length !== 1) {
    throw new Error(`${heading} section must appear exactly once`);
  }
  const start = indexes[0];
  let end = lines.length;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (/^##[^\S\r\n]+/.test(lines[index].trim())) {
      end = index;
      break;
    }
  }
  return { start, end };
}

function canonicalPointer(text) {
  const lines = String(text || '').replace(/\r\n/g, '\n').split('\n');
  const { start, end } = sectionRange(lines, 'Canonical');
  const matches = [];
  for (let index = start + 1; index < end; index += 1) {
    const match = lines[index].trim().match(/^[-*]\s+active-authority:\s*(\S+)\s*$/);
    if (match) matches.push({ index, value: match[1] });
  }
  if (matches.length !== 1) throw new Error('active-authority pointer must appear exactly once');
  return matches[0];
}

function parseAuthorityDocument(text) {
  const metadata = documentMetadata(text);
  const issues = [];
  if (Buffer.byteLength(String(text || ''), 'utf8') > AUTHORITY_MAX_BYTES) {
    issues.push('AUTHORITY_TOO_LARGE');
  }
  if (lineCount(String(text || '')) > AUTHORITY_MAX_LINES) {
    issues.push('AUTHORITY_TOO_MANY_LINES');
  }
  if (metadata.duplicates.length) issues.push('AUTHORITY_DUPLICATE_METADATA');
  if (metadata.values.schema !== AUTHORITY_SCHEMA) issues.push('AUTHORITY_SCHEMA_INVALID');
  if (!validTimestamp(metadata.values['updated-at'])) issues.push('AUTHORITY_UPDATED_AT_INVALID');
  let activeAuthority = null;
  try {
    activeAuthority = canonicalPointer(text).value;
  } catch {
    issues.push('AUTHORITY_ACTIVE_POINTER_INVALID');
  }
  return {
    ok: issues.length === 0,
    activeAuthority,
    updatedAt: metadata.values['updated-at'] || null,
    issues,
  };
}

function parseCurrentDocument(text) {
  const metadata = documentMetadata(text);
  const issues = [];
  if (Buffer.byteLength(String(text || ''), 'utf8') > CURRENT_MAX_BYTES) {
    issues.push('CURRENT_TOO_LARGE');
  }
  if (lineCount(String(text || '')) > CURRENT_MAX_LINES) {
    issues.push('CURRENT_TOO_MANY_LINES');
  }
  if (metadata.duplicates.length) issues.push('CURRENT_DUPLICATE_METADATA');
  if (metadata.values.schema !== CURRENT_SCHEMA) issues.push('CURRENT_SCHEMA_INVALID');
  if (!validTimestamp(metadata.values['updated-at'])) issues.push('CURRENT_UPDATED_AT_INVALID');
  if (!VALID_MODES.has(metadata.values.mode)) issues.push('CURRENT_MODE_INVALID');
  if (
    !['READY', 'IN_PROGRESS', 'WAITING_EXTERNAL_CONDITION', 'BLOCKED', 'COMPLETE']
      .includes(metadata.values['work-state'])
  ) {
    issues.push('CURRENT_WORK_STATE_INVALID');
  }
  if (
    metadata.values['active-id'] !== 'NONE' &&
    !ID_PATTERN.test(metadata.values['active-id'] || '')
  ) {
    issues.push('CURRENT_ACTIVE_ID_INVALID');
  }
  for (const heading of ['Status', 'Blockers', 'Next action', 'Safety']) {
    try {
      sectionRange(String(text || '').replace(/\r\n/g, '\n').split('\n'), heading);
    } catch {
      issues.push(`CURRENT_${heading.toUpperCase().replace(/\s+/g, '_')}_SECTION_INVALID`);
    }
  }
  return {
    ok: issues.length === 0,
    mode: metadata.values.mode || null,
    workState: metadata.values['work-state'] || null,
    activeId: metadata.values['active-id'] || null,
    phase: metadata.values.phase || null,
    goal: metadata.values.goal || null,
    updatedAt: metadata.values['updated-at'] || null,
    issues,
  };
}

function replaceMetadata(text, updates) {
  const lines = String(text || '').replace(/\r\n/g, '\n').split('\n');
  const bodyIndex = lines.findIndex((line) => /^##[^\S\r\n]+/.test(line.trim()));
  const limit = bodyIndex === -1 ? lines.length : bodyIndex;
  for (const [key, value] of Object.entries(updates)) {
    const indexes = [];
    for (let index = 0; index < limit; index += 1) {
      if (new RegExp(`^${key}:\\s*`).test(lines[index].trim())) indexes.push(index);
    }
    if (indexes.length !== 1) throw new Error(`${key} metadata must appear exactly once`);
    lines[indexes[0]] = `${key}: ${value}`;
  }
  return lines.join('\n');
}

function replaceSectionItems(text, heading, items) {
  const lines = String(text || '').replace(/\r\n/g, '\n').split('\n');
  const range = sectionRange(lines, heading);
  const normalizedItems = items.length ? items : ['none'];
  return [
    ...lines.slice(0, range.start + 1),
    '',
    ...normalizedItems.map((item) => `- ${normalizeSingleLine(item, heading, 320)}`),
    '',
    ...lines.slice(range.end).filter((line, index) => index !== 0 || line !== ''),
  ].join('\n');
}

function updateAuthorityDocument(text, updates) {
  const parsed = parseAuthorityDocument(text);
  if (!parsed.ok) throw new Error(`authority document is invalid: ${parsed.issues.join(', ')}`);
  let next = replaceMetadata(text, { 'updated-at': updates.updatedAt });
  const lines = next.replace(/\r\n/g, '\n').split('\n');
  const pointer = canonicalPointer(next);
  lines[pointer.index] = `- active-authority: ${updates.activeAuthority}`;
  next = lines.join('\n');
  if (!next.endsWith('\n')) next += '\n';
  const checked = parseAuthorityDocument(next);
  if (!checked.ok || checked.activeAuthority !== updates.activeAuthority) {
    throw new Error('authority update failed validation');
  }
  return next;
}

function updateCurrentDocument(text, updates) {
  const parsed = parseCurrentDocument(text);
  if (!parsed.ok) throw new Error(`current document is invalid: ${parsed.issues.join(', ')}`);
  let next = replaceMetadata(text, {
    'updated-at': updates.updatedAt,
    mode: updates.mode,
    'work-state': updates.workState,
    'active-id': updates.activeId,
    phase: updates.phase,
    goal: updates.goal,
  });
  next = replaceSectionItems(next, 'Status', updates.status);
  next = replaceSectionItems(next, 'Blockers', updates.blockers);
  next = replaceSectionItems(next, 'Next action', [updates.nextAction]);
  if (updates.safety) next = replaceSectionItems(next, 'Safety', updates.safety);
  if (!next.endsWith('\n')) next += '\n';
  const checked = parseCurrentDocument(next);
  if (!checked.ok || checked.activeId !== updates.activeId) {
    throw new Error('current update failed validation');
  }
  return next;
}

module.exports = {
  AUTHORITY_MAX_BYTES,
  AUTHORITY_MAX_LINES,
  AUTHORITY_PATH,
  BODY_SOFT_MAX_BYTES,
  CURRENT_MAX_BYTES,
  CURRENT_MAX_LINES,
  CURRENT_PATH,
  HEADER_MAX_BYTES,
  HEADER_MAX_LINES,
  LEGACY_PACKAGE_SCHEMA,
  PACKAGE_ROOT,
  PACKAGE_SCHEMA,
  VALID_GIT_DISPOSITIONS,
  VALID_MODES,
  VALID_OUTCOMES,
  VALID_TASK_MODES,
  lineCount,
  normalizeSingleLine,
  packageRelativePath,
  parseAuthorityDocument,
  parseCurrentDocument,
  parseTaskPackage,
  renderDraftFromTemplate,
  sha256,
  updateAuthorityDocument,
  updateCurrentDocument,
  validTimestamp,
  validateId,
};
