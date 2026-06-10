const fs = require('fs');
const path = require('path');
const { scanSecurityFindings } = require('./security');

const scopes = new Set(['global', 'project', 'repo', 'task']);
const sourceTypes = new Set([
  'user-preference',
  'project-decision',
  'mistake',
  'tool-observation',
  'external-input',
  'model-summary'
]);
const confidenceLevels = new Set(['low', 'medium', 'high']);
const authorityLevels = new Set(['candidate', 'evidence-backed', 'user-confirmed', 'project-doc-backed']);
const statuses = new Set(['candidate', 'approved', 'quarantined', 'stale', 'revoked']);

function isPlainObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function slug(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .replace(/-+/g, '-')
    .slice(0, 60);
}

function nowIso() {
  return new Date().toISOString();
}

function uniqueStrings(values) {
  return Array.from(new Set((Array.isArray(values) ? values : [])
    .map((value) => String(value || '').trim())
    .filter(Boolean)));
}

function makeId(input) {
  const basis = slug(input.id || input.claim || input.source || 'memory');
  return String(input.id || `MEM-${basis || 'candidate'}`).trim();
}

function normalizeMemoryCandidate(input) {
  const raw = isPlainObject(input) ? input : {};
  const createdAt = raw.createdAt || nowIso();
  return {
    id: makeId(raw),
    project: String(raw.project || '').trim() || 'unknown',
    scope: scopes.has(raw.scope) ? raw.scope : 'project',
    sourceType: sourceTypes.has(raw.sourceType) ? raw.sourceType : 'model-summary',
    source: String(raw.source || 'unknown').trim(),
    claim: String(raw.claim || '').trim(),
    evidenceRefs: uniqueStrings(raw.evidenceRefs || raw.evidence || []),
    relatedFiles: uniqueStrings(raw.relatedFiles || raw.relatedFile || []),
    confidence: confidenceLevels.has(raw.confidence) ? raw.confidence : 'low',
    authority: authorityLevels.has(raw.authority) ? raw.authority : 'candidate',
    status: statuses.has(raw.status) ? raw.status : 'candidate',
    riskTags: uniqueStrings(raw.riskTags || raw.riskTag || []),
    createdAt,
    lastVerifiedAt: raw.lastVerifiedAt || null,
    expiresAt: raw.expiresAt || null,
    staleTriggers: uniqueStrings(raw.staleTriggers || []),
    reviewReason: raw.reviewReason ? String(raw.reviewReason).trim() : ''
  };
}

function sourceForTrust(candidate) {
  if (candidate.sourceType === 'external-input') return 'web';
  if (candidate.sourceType === 'user-preference') return 'user';
  if (candidate.sourceType === 'project-decision') return 'project-protocol';
  if (candidate.sourceType === 'mistake') return 'local-protocol';
  return 'unknown';
}

function scanCandidate(candidate) {
  const metadataText = [
    `claim: ${candidate.claim}`,
    `source: ${candidate.source}`,
    `evidenceRefs: ${candidate.evidenceRefs.join('\n')}`,
    `relatedFiles: ${candidate.relatedFiles.join('\n')}`,
    `riskTags: ${candidate.riskTags.join('\n')}`,
    `reviewReason: ${candidate.reviewReason || ''}`
  ].join('\n');
  return scanSecurityFindings(metadataText, {
    source: sourceForTrust(candidate),
    path: candidate.source || null
  });
}

function detectAuthorityConflict(candidate, authorityText) {
  const normalized = normalizeMemoryCandidate(candidate);
  const claim = String(normalized.claim || '').toLowerCase();
  const text = String(authorityText || '').toLowerCase();
  const reasons = [];

  if (!text || !claim) return reasons;

  if (text.includes('supersedes memory') && text.includes(normalized.id.toLowerCase())) {
    reasons.push('project authority text supersedes this memory');
  }

  const verificationRequired = /fresh verification|verification before completion|completion claim.*verification|no false completion/.test(text);
  const claimSkipsVerification = /(?:no longer|not|never).{0,40}require.{0,40}verification|skip.{0,40}verification|verification.{0,40}(?:optional|unnecessary|not required)/.test(claim);
  if (verificationRequired && claimSkipsVerification) {
    reasons.push('memory claim conflicts with current verification requirements');
  }

  const singleAgentDefault = /default to a single main agent|default single agent|single agent plus tools|most tasks are safer as one agent/.test(text);
  const claimRequiresMultiAgent = /always.{0,40}(?:use|dispatch|spawn).{0,40}(?:multi-agent|subagent|agent [a-e])|default.{0,40}(?:multi-agent|subagent)/.test(claim);
  if (singleAgentDefault && claimRequiresMultiAgent) {
    reasons.push('memory claim conflicts with single-agent-first protocol');
  }

  const packageManagerMatch = claim.match(/\bpackage manager is (pnpm|npm|yarn|bun)\b/);
  if (packageManagerMatch) {
    const claimed = packageManagerMatch[1];
    const packageManagerText = text.match(/"packageManager"\s*:\s*"(pnpm|npm|yarn|bun)@/i)
      || text.match(/\bpackageManager["']?\s*[:=]\s*["']?(pnpm|npm|yarn|bun)\b/i);
    if (packageManagerText && packageManagerText[1].toLowerCase() !== claimed) {
      reasons.push(`memory claim conflicts with current package manager: ${packageManagerText[1].toLowerCase()}`);
    }
  }

  return reasons;
}

function classifyMemoryTrust(candidate, projectContext = {}) {
  const normalized = normalizeMemoryCandidate(candidate);
  const scan = scanCandidate(normalized);
  const reasons = [];
  let recommendedStatus = normalized.status;

  if (scan.redacted) {
    reasons.push('secret-like content detected');
    recommendedStatus = 'quarantined';
  }
  if (scan.promptInjectionDetected) {
    reasons.push('prompt-injection-like memory content detected');
    recommendedStatus = 'quarantined';
  }
  if (normalized.sourceType === 'external-input' && normalized.evidenceRefs.length === 0) {
    reasons.push('external input lacks evidence reference');
    if (recommendedStatus === 'approved') recommendedStatus = 'candidate';
  }
  if (normalized.authority === 'candidate' && normalized.status === 'approved') {
    reasons.push('approved status requires stronger authority than candidate');
    recommendedStatus = 'candidate';
  }
  if (normalized.status === 'approved' && normalized.authority !== 'user-confirmed' && normalized.evidenceRefs.length === 0) {
    reasons.push('approved status requires evidence refs unless authority is user-confirmed');
    recommendedStatus = 'candidate';
  }
  const stale = isMemoryStale(normalized, projectContext);
  if (stale.stale) {
    reasons.push(...stale.reasons);
    recommendedStatus = 'stale';
  }

  return {
    candidate: normalized,
    scan,
    recommendedStatus,
    reasons,
    trustedForContext: recommendedStatus === 'approved'
      && !scan.redacted
      && !stale.stale
      && normalized.authority !== 'candidate'
  };
}

function shouldPromoteMemory(candidate, projectContext = {}) {
  const normalized = normalizeMemoryCandidate(candidate);
  const classified = classifyMemoryTrust(normalized, projectContext);
  const hasAuthority = normalized.authority === 'user-confirmed'
    || normalized.authority === 'project-doc-backed'
    || (normalized.authority === 'evidence-backed' && normalized.evidenceRefs.length > 0);
  const promote = hasAuthority
    && classified.recommendedStatus !== 'quarantined'
    && classified.recommendedStatus !== 'stale'
    && !classified.scan.redacted;
  return {
    promote,
    reasons: promote ? ['candidate has sufficient authority and no blocking findings'] : classified.reasons.concat(hasAuthority ? [] : ['candidate lacks promotion authority'])
  };
}

function shouldQuarantineMemory(candidate, projectContext = {}) {
  const classified = classifyMemoryTrust(candidate, projectContext);
  return {
    quarantine: classified.recommendedStatus === 'quarantined',
    reasons: classified.reasons
  };
}

function parseTime(value) {
  const time = Date.parse(value || '');
  return Number.isFinite(time) ? time : null;
}

function relativeExists(targetRoot, relativePath) {
  if (!targetRoot || !relativePath) return true;
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function isMemoryStale(candidate, projectContext = {}) {
  const normalized = normalizeMemoryCandidate(candidate);
  const targetRoot = projectContext.targetRoot || null;
  const now = projectContext.now ? parseTime(projectContext.now) : Date.now();
  const reasons = [];
  const expiresAt = parseTime(normalized.expiresAt);
  if (expiresAt !== null && expiresAt < now) reasons.push('expiresAt is in the past');

  const staleAfterDays = Number.isInteger(projectContext.staleAfterDays) ? projectContext.staleAfterDays : null;
  if (staleAfterDays !== null) {
    const verifiedAt = parseTime(normalized.lastVerifiedAt || normalized.createdAt);
    if (verifiedAt !== null && now - verifiedAt > staleAfterDays * 24 * 60 * 60 * 1000) {
      reasons.push(`last verification is older than ${staleAfterDays} day(s)`);
    }
  }

  for (const file of normalized.relatedFiles) {
    if (!relativeExists(targetRoot, file)) reasons.push(`related file is missing: ${file}`);
  }
  for (const file of normalized.evidenceRefs) {
    if (!relativeExists(targetRoot, file)) reasons.push(`evidence ref is missing: ${file}`);
  }

  reasons.push(...detectAuthorityConflict(normalized, projectContext.authorityText || ''));

  return {
    stale: reasons.length > 0,
    reasons
  };
}

function validateMemoryCandidate(candidate, options = {}) {
  const normalized = normalizeMemoryCandidate(candidate);
  const errors = [];
  const warnings = [];
  if (!normalized.id) errors.push('memory id is required');
  if (!normalized.claim) errors.push('memory claim is required');
  if (!scopes.has(normalized.scope)) errors.push(`memory scope is invalid: ${normalized.scope}`);
  if (!sourceTypes.has(normalized.sourceType)) errors.push(`memory sourceType is invalid: ${normalized.sourceType}`);
  if (!confidenceLevels.has(normalized.confidence)) errors.push(`memory confidence is invalid: ${normalized.confidence}`);
  if (!authorityLevels.has(normalized.authority)) errors.push(`memory authority is invalid: ${normalized.authority}`);
  if (!statuses.has(normalized.status)) errors.push(`memory status is invalid: ${normalized.status}`);
  if (!parseTime(normalized.createdAt)) errors.push('memory createdAt must be an ISO-like timestamp');
  if (normalized.status === 'approved' && normalized.authority === 'candidate') {
    errors.push('approved memory requires stronger authority than candidate');
  }
  if (normalized.status === 'approved' && normalized.authority !== 'user-confirmed' && normalized.evidenceRefs.length === 0) {
    errors.push('approved memory requires evidenceRefs unless authority is user-confirmed');
  }

  const classified = classifyMemoryTrust(normalized, options.projectContext || {});
  if (classified.scan.redacted) errors.push('memory claim contains secret-like content');
  if (classified.scan.promptInjectionDetected) warnings.push('memory claim contains prompt-injection-like content');
  if (classified.recommendedStatus !== normalized.status) {
    warnings.push(`recommended status is ${classified.recommendedStatus}: ${classified.reasons.join('; ')}`);
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    normalized,
    classification: classified
  };
}

function memoryRoot(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory');
}

function memoryCandidateFiles(targetRoot, id) {
  const safeId = slug(id || 'memory') || 'memory';
  const dir = path.join(memoryRoot(targetRoot), 'candidates');
  return {
    dir,
    json: path.join(dir, `${safeId}.json`),
    markdown: path.join(dir, `${safeId}.md`)
  };
}

function renderMemoryCandidateMarkdown(candidate) {
  const normalized = normalizeMemoryCandidate(candidate);
  const lines = [
    `# ${normalized.id}`,
    '',
    `- status: ${normalized.status}`,
    `- scope: ${normalized.scope}`,
    `- sourceType: ${normalized.sourceType}`,
    `- source: ${normalized.source}`,
    `- authority: ${normalized.authority}`,
    `- confidence: ${normalized.confidence}`,
    `- createdAt: ${normalized.createdAt}`,
    `- lastVerifiedAt: ${normalized.lastVerifiedAt || 'Unverified'}`,
    `- expiresAt: ${normalized.expiresAt || 'None'}`,
    '',
    '## Claim',
    '',
    normalized.claim,
    '',
    '## Evidence Refs',
    ''
  ];
  if (normalized.evidenceRefs.length === 0) lines.push('- None');
  else for (const ref of normalized.evidenceRefs) lines.push(`- ${ref}`);
  lines.push('', '## Related Files', '');
  if (normalized.relatedFiles.length === 0) lines.push('- None');
  else for (const file of normalized.relatedFiles) lines.push(`- ${file}`);
  if (normalized.riskTags.length > 0) {
    lines.push('', '## Risk Tags', '');
    for (const tag of normalized.riskTags) lines.push(`- ${tag}`);
  }
  if (normalized.reviewReason) {
    lines.push('', '## Review Reason', '', normalized.reviewReason);
  }
  return `${lines.join('\n')}\n`;
}

module.exports = {
  classifyMemoryTrust,
  isMemoryStale,
  memoryCandidateFiles,
  normalizeMemoryCandidate,
  renderMemoryCandidateMarkdown,
  shouldPromoteMemory,
  shouldQuarantineMemory,
  validateMemoryCandidate
};
