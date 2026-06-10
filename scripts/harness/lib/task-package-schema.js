const fs = require('fs');
const path = require('path');

const schemaVersion = 1;
const requiredFields = [
  'id',
  'mission',
  'path',
  'readScope',
  'writeScope',
  'forbiddenScope',
  'acceptance',
  'verification',
  'riskTags',
  'inputs',
  'relatedMistakes'
];
const arrayFields = [
  'readScope',
  'writeScope',
  'forbiddenScope',
  'acceptance',
  'verification',
  'riskTags',
  'inputs',
  'relatedMistakes'
];
const allowedPaths = new Set(['fast', 'standard', 'strict']);

function safeId(value) {
  const id = String(value || '').trim();
  if (!id) throw new Error('--id is required');
  if (/[\\/]/.test(id) || id.includes('..')) throw new Error('Task package id must be a safe file name segment');
  return id.replace(/[^A-Za-z0-9._-]+/g, '-').replace(/^-+|-+$/g, '');
}

function asList(value) {
  if (value === undefined || value === null) return [];
  if (Array.isArray(value)) return value.map((item) => String(item).trim()).filter(Boolean);
  const text = String(value).trim();
  return text ? [text] : [];
}

function normalizeTaskPackage(input) {
  const id = safeId(input.id);
  const taskPath = String(input.path || '').trim().toLowerCase();
  return {
    schemaVersion,
    id,
    mission: String(input.mission || '').trim(),
    path: taskPath,
    readScope: asList(input.readScope),
    writeScope: asList(input.writeScope),
    forbiddenScope: asList(input.forbiddenScope),
    acceptance: asList(input.acceptance),
    verification: asList(input.verification),
    riskTags: asList(input.riskTags),
    inputs: asList(input.inputs),
    relatedMistakes: asList(input.relatedMistakes)
  };
}

function validateTaskPackage(data, options = {}) {
  const errors = [];
  const warnings = [];
  const source = options.source || data && data.id || 'task package';

  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    return { source, valid: false, errors: [`${source}: JSON root must be an object`], warnings };
  }

  for (const field of requiredFields) {
    if (data[field] === undefined) errors.push(`${source}: missing required field ${field}`);
  }

  if (typeof data.id !== 'string' || data.id.trim() === '') errors.push(`${source}: id must be a non-empty string`);
  if (typeof data.mission !== 'string' || data.mission.trim() === '') errors.push(`${source}: mission must be a non-empty string`);
  if (typeof data.path !== 'string' || !allowedPaths.has(data.path)) errors.push(`${source}: path must be one of fast, standard, strict`);

  for (const field of arrayFields) {
    if (!Array.isArray(data[field])) {
      errors.push(`${source}: ${field} must be an array`);
      continue;
    }
    if (data[field].length === 0) errors.push(`${source}: ${field} must contain at least one item`);
    data[field].forEach((item, index) => {
      if (typeof item !== 'string' || item.trim() === '') errors.push(`${source}: ${field}[${index}] must be a non-empty string`);
    });
  }

  if (data.schemaVersion !== undefined && data.schemaVersion !== schemaVersion) {
    warnings.push(`${source}: schemaVersion ${data.schemaVersion} differs from supported version ${schemaVersion}`);
  }

  return { source, valid: errors.length === 0, errors, warnings };
}

function renderList(items) {
  return asList(items).map((item) => `- ${item}`).join('\n');
}

function renderTaskPackageMarkdown(data) {
  const pkg = normalizeTaskPackage(data);
  return `# Task Package: ${pkg.id}

## Mission
${pkg.mission}

## Path
${pkg.path}

## Allowed Read Paths
${renderList(pkg.readScope)}

## Allowed Write Paths
${renderList(pkg.writeScope)}

## Forbidden
${renderList(pkg.forbiddenScope)}

## Inputs
${renderList(pkg.inputs)}

## Acceptance
${renderList(pkg.acceptance)}

## Verification
${renderList(pkg.verification)}

## Risk Tags
${renderList(pkg.riskTags)}

## Related Mistakes
${renderList(pkg.relatedMistakes)}

## Required Output
STATUS: DONE | DONE_WITH_CONCERNS | NEEDS_CONTEXT | NEEDS_DECISION | BLOCKED

CHANGED_FILES:
- List actual changed files, or None.

SUMMARY:
- Short factual summary.

EVIDENCE:
- Commands/checks run and result summary; if not verified, explain why.

RISKS:
- Remaining risks, assumptions, conflicts.

REQUESTS:
- Decisions or authorizations needed, or None.
`;
}

function taskPackageDir(targetRoot) {
  return path.join(targetRoot, 'docs', 'task-packages');
}

function taskPackageFiles(targetRoot, id) {
  const safe = safeId(id);
  const dir = taskPackageDir(targetRoot);
  return {
    dir,
    json: path.join(dir, `${safe}.json`),
    markdown: path.join(dir, `${safe}.md`)
  };
}

function readTaskPackage(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

module.exports = {
  schemaVersion,
  requiredFields,
  normalizeTaskPackage,
  renderTaskPackageMarkdown,
  taskPackageDir,
  taskPackageFiles,
  validateTaskPackage,
  readTaskPackage
};
