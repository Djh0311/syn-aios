'use strict';

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const MANIFEST_SCHEMA_VERSION = 2;
const MANIFEST_RELATIVE_PATH = '.harness/manifest.json';
const OWNERSHIP_STATES = new Set(['created', 'adopted', 'external']);
const LEGACY_OWNERSHIP_STATES = new Set(['managed', 'preserved']);

function sha256Buffer(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function sha256File(filePath) {
  return sha256Buffer(fs.readFileSync(filePath));
}

function normalizeRelativePath(value, label = 'path') {
  if (typeof value !== 'string') throw new Error(`${label} must be a string`);
  const candidate = value.trim();
  if (
    !candidate ||
    candidate.includes('\0') ||
    candidate.includes('\\') ||
    path.posix.isAbsolute(candidate) ||
    candidate.endsWith('/') ||
    candidate.split('/').includes('..') ||
    path.posix.normalize(candidate) !== candidate
  ) {
    throw new Error(`${label} must be a normalized repository-relative path: ${value}`);
  }
  return candidate;
}

function manifestPath(targetRoot) {
  return path.join(path.resolve(targetRoot), MANIFEST_RELATIVE_PATH);
}

function lstatIfPresent(filePath) {
  try {
    return fs.lstatSync(filePath);
  } catch (error) {
    if (error && error.code === 'ENOENT') return null;
    throw error;
  }
}

function inspectManifestCarrier(targetRoot) {
  const root = path.resolve(targetRoot);
  const harnessDirectory = path.join(root, '.harness');
  const file = path.join(harnessDirectory, 'manifest.json');
  const harnessStat = lstatIfPresent(harnessDirectory);
  if (harnessStat) {
    const stat = harnessStat;
    if (stat.isSymbolicLink()) {
      return { ok: false, code: 'UNSAFE_MANIFEST_PATH', message: '.harness must not be a symlink' };
    }
    if (!stat.isDirectory()) {
      return { ok: false, code: 'UNSAFE_MANIFEST_PATH', message: '.harness must be a directory' };
    }
  }
  const manifestStat = lstatIfPresent(file);
  if (manifestStat) {
    const stat = manifestStat;
    if (stat.isSymbolicLink() || !stat.isFile()) {
      return {
        ok: false,
        code: 'UNSAFE_MANIFEST_PATH',
        message: '.harness/manifest.json must be a regular non-symlink file'
      };
    }
  }
  return { ok: true, file };
}

function validateSelection(selection, errors) {
  if (!selection || typeof selection !== 'object' || Array.isArray(selection)) {
    errors.push('selection must be an object');
    return;
  }
  for (const key of Object.keys(selection)) {
    if (!['pack', 'packVersion', 'requestedComponents', 'resolvedComponents'].includes(key)) {
      errors.push(`selection.${key} is unsupported`);
    }
  }
  if (selection.pack !== null && selection.pack !== undefined && typeof selection.pack !== 'string') {
    errors.push('selection.pack must be a string or null');
  }
  if (typeof selection.pack === 'string' && !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(selection.pack)) {
    errors.push('selection.pack has an invalid id');
  }
  if (
    selection.packVersion !== null &&
    selection.packVersion !== undefined &&
    (typeof selection.packVersion !== 'string' || !selection.packVersion.trim())
  ) {
    errors.push('selection.packVersion must be a non-empty string or null');
  }
  if (selection.pack && !selection.packVersion) {
    errors.push('selection.packVersion is required when selection.pack is set');
  }
  if (!selection.pack && selection.packVersion) {
    errors.push('selection.packVersion must be null when selection.pack is null');
  }
  for (const key of ['requestedComponents', 'resolvedComponents']) {
    if (
      !Array.isArray(selection[key]) ||
      selection[key].some(
        (item) => typeof item !== 'string' || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(item),
      )
    ) {
      errors.push(`selection.${key} must be an array of non-empty strings`);
    }
  }
  for (const key of ['requestedComponents', 'resolvedComponents']) {
    if (
      Array.isArray(selection[key]) &&
      new Set(selection[key]).size !== selection[key].length
    ) {
      errors.push(`selection.${key} must not contain duplicates`);
    }
  }
}

function validateComponents(components, errors) {
  if (!components || typeof components !== 'object' || Array.isArray(components)) {
    errors.push('components must be an object');
    return;
  }
  for (const [id, entry] of Object.entries(components)) {
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)) {
      errors.push(`components.${id} has an invalid component id`);
    }
    if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
      errors.push(`components.${id} must be an object`);
      continue;
    }
    for (const key of Object.keys(entry)) {
      if (!['version', 'kind'].includes(key)) errors.push(`components.${id}.${key} is unsupported`);
    }
    if (typeof entry.version !== 'string' || !entry.version.trim()) {
      errors.push(`components.${id}.version must be a non-empty string`);
    }
    if (typeof entry.kind !== 'string' || !entry.kind.trim()) {
      errors.push(`components.${id}.kind must be a non-empty string`);
    }
  }
}

function validMode(value) {
  return Number.isInteger(value) && value >= 0 && value <= 0o777;
}

function validateFileEntry(relativePath, entry, errors, options = {}) {
  const allowLegacyOwnership = options.allowLegacyOwnership !== false;
  try {
    normalizeRelativePath(relativePath, 'manifest file path');
  } catch (error) {
    errors.push(error.message);
  }
  if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
    errors.push(`files.${relativePath} must be an object`);
    return;
  }
  for (const key of Object.keys(entry)) {
    if (
      ![
        'components',
        'source',
        'policy',
        'ownership',
        'sourceSha256',
        'contentSha256',
        'installedSha256',
        'installedMode',
        'adoptionBaselineSha256',
        'adoptionBaselineMode',
        'executable',
        'mutable'
      ].includes(key)
    ) {
      errors.push(`files.${relativePath}.${key} is unsupported`);
    }
  }
  if (!Array.isArray(entry.components) || entry.components.some((item) => typeof item !== 'string')) {
    errors.push(`files.${relativePath}.components must be an array of strings`);
  } else if (entry.components.length === 0 || new Set(entry.components).size !== entry.components.length) {
    errors.push(`files.${relativePath}.components must be non-empty and unique`);
  }
  try {
    normalizeRelativePath(entry.source, `files.${relativePath}.source`);
  } catch (error) {
    errors.push(error.message);
  }
  if (!['replace-managed', 'create-if-missing'].includes(entry.policy)) {
    errors.push(`files.${relativePath}.policy is unsupported`);
  }
  const legacyOwnership = LEGACY_OWNERSHIP_STATES.has(entry.ownership);
  const legacyAdoption = entry.ownership === 'adopted' && (
    typeof entry.adoptionBaselineSha256 !== 'string' ||
    !validMode(entry.adoptionBaselineMode)
  );
  if (legacyOwnership && !allowLegacyOwnership) {
    errors.push(`files.${relativePath}.ownership is legacy and cannot be written`);
  } else if (!OWNERSHIP_STATES.has(entry.ownership) && !legacyOwnership) {
    errors.push(`files.${relativePath}.ownership is unsupported`);
  } else if (!allowLegacyOwnership && legacyAdoption) {
    errors.push(`files.${relativePath}.ownership is legacy and cannot be written`);
  }
  for (const key of ['sourceSha256', 'contentSha256', 'installedSha256']) {
    if (typeof entry[key] !== 'string' || !/^[a-f0-9]{64}$/.test(entry[key])) {
      errors.push(`files.${relativePath}.${key} must be a sha256 digest`);
    }
  }
  if (entry.installedMode === undefined) {
    if (!allowLegacyOwnership) {
      errors.push(`files.${relativePath}.installedMode is required`);
    }
  } else if (!validMode(entry.installedMode)) {
    errors.push(`files.${relativePath}.installedMode must be a file mode`);
  }
  if (entry.ownership === 'adopted' && !legacyAdoption) {
    if (!/^[a-f0-9]{64}$/.test(entry.adoptionBaselineSha256)) {
      errors.push(`files.${relativePath}.adoptionBaselineSha256 must be a sha256 digest`);
    }
    if (!validMode(entry.adoptionBaselineMode)) {
      errors.push(`files.${relativePath}.adoptionBaselineMode must be a file mode`);
    }
  } else if (!allowLegacyOwnership && entry.ownership === 'adopted') {
    errors.push(`files.${relativePath}.adopted ownership requires an immutable baseline`);
  } else if (
    entry.ownership !== 'adopted' &&
    (
      entry.adoptionBaselineSha256 !== undefined ||
      entry.adoptionBaselineMode !== undefined
    )
  ) {
    errors.push(`files.${relativePath}.adoption baseline is only valid for adopted ownership`);
  }
  if (typeof entry.executable !== 'boolean') {
    errors.push(`files.${relativePath}.executable must be boolean`);
  }
  if (typeof entry.mutable !== 'boolean') {
    errors.push(`files.${relativePath}.mutable must be boolean`);
  } else if (entry.mutable !== (entry.policy === 'create-if-missing')) {
    errors.push(`files.${relativePath}.mutable must match create-if-missing policy`);
  }
}

function validateManifest(data, options = {}) {
  const errors = [];
  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    return { ok: false, errors: ['manifest root must be an object'] };
  }
  for (const key of Object.keys(data)) {
    if (
      ![
        'schemaVersion',
        'harnessVersion',
        'installedAt',
        'projectName',
        'selection',
        'components',
        'files'
      ].includes(key)
    ) {
      errors.push(`${key} is unsupported in schema 2 manifest`);
    }
  }
  if (data.schemaVersion !== MANIFEST_SCHEMA_VERSION) {
    errors.push(`schemaVersion must be ${MANIFEST_SCHEMA_VERSION}`);
  }
  if (typeof data.harnessVersion !== 'string' || !data.harnessVersion.trim()) {
    errors.push('harnessVersion must be a non-empty string');
  }
  if (typeof data.installedAt !== 'string' || Number.isNaN(Date.parse(data.installedAt))) {
    errors.push('installedAt must be an ISO timestamp');
  }
  if (typeof data.projectName !== 'string' || !data.projectName.trim()) {
    errors.push('projectName must be a non-empty string');
  }
  validateSelection(data.selection, errors);
  validateComponents(data.components, errors);
  if (!data.files || typeof data.files !== 'object' || Array.isArray(data.files)) {
    errors.push('files must be an object');
  } else {
    for (const [relativePath, entry] of Object.entries(data.files)) {
      validateFileEntry(relativePath, entry, errors, options);
    }
  }
  if (
    data.selection &&
    Array.isArray(data.selection.resolvedComponents) &&
    data.components &&
    typeof data.components === 'object' &&
    !Array.isArray(data.components)
  ) {
    const resolved = [...data.selection.resolvedComponents].sort();
    const declared = Object.keys(data.components).sort();
    if (JSON.stringify(resolved) !== JSON.stringify(declared)) {
      errors.push('selection.resolvedComponents must exactly match components');
    }
    if (data.files && typeof data.files === 'object' && !Array.isArray(data.files)) {
      for (const [relativePath, entry] of Object.entries(data.files)) {
        if (!entry || !Array.isArray(entry.components)) continue;
        for (const owner of entry.components) {
          if (!Object.prototype.hasOwnProperty.call(data.components, owner)) {
            errors.push(`files.${relativePath} names undeclared component ${owner}`);
          }
        }
      }
    }
  }
  return { ok: errors.length === 0, errors };
}

function normalizeManifestOwnership(data) {
  const normalized = structuredClone(data);
  const advisories = [];
  for (const [relativePath, entry] of Object.entries(normalized.files || {})) {
    if (!validMode(entry.installedMode)) {
      entry.installedMode = entry.executable ? 0o755 : 0o644;
    }
    if (LEGACY_OWNERSHIP_STATES.has(entry.ownership)) {
      advisories.push(
        `${relativePath} legacy ownership ${entry.ownership} is treated as external`,
      );
      entry.ownership = 'external';
      delete entry.adoptionBaselineSha256;
      delete entry.adoptionBaselineMode;
    } else if (
      entry.ownership === 'adopted' &&
      (
        typeof entry.adoptionBaselineSha256 !== 'string' ||
        !/^[a-f0-9]{64}$/.test(entry.adoptionBaselineSha256) ||
        !validMode(entry.adoptionBaselineMode)
      )
    ) {
      advisories.push(
        `${relativePath} legacy adopted ownership lacks an explicit baseline and is treated as external`,
      );
      entry.ownership = 'external';
      delete entry.adoptionBaselineSha256;
      delete entry.adoptionBaselineMode;
    }
  }
  return { data: normalized, advisories };
}

function readManifest(targetRoot) {
  const carrier = inspectManifestCarrier(targetRoot);
  if (!carrier.ok) {
    return {
      path: manifestPath(targetRoot),
      data: null,
      error: { code: carrier.code, message: carrier.message }
    };
  }
  const file = carrier.file;
  if (!lstatIfPresent(file)) return { path: file, data: null, error: null };
  let descriptor;
  try {
    descriptor = fs.openSync(
      file,
      fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW || 0),
    );
    const stat = fs.fstatSync(descriptor);
    if (!stat.isFile()) {
      return {
        path: file,
        data: null,
        error: {
          code: 'UNSAFE_MANIFEST_PATH',
          message: '.harness/manifest.json must be a regular non-symlink file',
        },
      };
    }
    const content = fs.readFileSync(descriptor);
    const data = JSON.parse(content.toString('utf8'));
    const validation = validateManifest(data, { allowLegacyOwnership: true });
    if (!validation.ok) {
      return {
        path: file,
        data: null,
        error: {
          code: data && data.schemaVersion !== MANIFEST_SCHEMA_VERSION
            ? 'UNSUPPORTED_MANIFEST_SCHEMA'
            : 'INVALID_MANIFEST',
          message: validation.errors.join('; ')
        }
      };
    }
    const normalized = normalizeManifestOwnership(data);
    return {
      path: file,
      data: normalized.data,
      error: null,
      fileSha256: sha256Buffer(content),
      fileMode: stat.mode & 0o777,
      ownershipAdvisories: normalized.advisories,
    };
  } catch (error) {
    return {
      path: file,
      data: null,
      error: { code: 'INVALID_MANIFEST_JSON', message: error.message }
    };
  } finally {
    if (descriptor !== undefined) fs.closeSync(descriptor);
  }
}

function sortedUnique(values) {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function makeManifest(options) {
  const files = {};
  for (const entry of [...options.files].sort((left, right) => left.target.localeCompare(right.target))) {
    files[normalizeRelativePath(entry.target, 'installed target')] = {
      components: sortedUnique(entry.components || []),
      source: normalizeRelativePath(entry.source, 'source path'),
      policy: entry.policy,
      ownership: entry.ownership,
      sourceSha256: entry.sourceSha256,
      contentSha256: entry.contentSha256,
      installedSha256: entry.installedSha256,
      installedMode: entry.installedMode,
      ...(entry.ownership === 'adopted'
        ? {
          adoptionBaselineSha256: entry.adoptionBaselineSha256,
          adoptionBaselineMode: entry.adoptionBaselineMode,
        }
        : {}),
      executable: entry.executable === true,
      mutable: entry.mutable === true
    };
  }
  const components = {};
  for (const id of Object.keys(options.components || {}).sort((left, right) => left.localeCompare(right))) {
    const component = options.components[id];
    components[id] = {
      version: component.version,
      kind: component.kind,
    };
  }
  const manifest = {
    schemaVersion: MANIFEST_SCHEMA_VERSION,
    harnessVersion: options.harnessVersion,
    installedAt: options.installedAt,
    projectName: options.projectName,
    selection: {
      pack: options.selection.pack || null,
      packVersion: options.selection.packVersion || null,
      requestedComponents: sortedUnique(options.selection.requestedComponents || []),
      resolvedComponents: sortedUnique(options.selection.resolvedComponents || [])
    },
    components,
    files
  };
  const validation = validateManifest(manifest, { allowLegacyOwnership: false });
  if (!validation.ok) throw new Error(`Refusing invalid manifest: ${validation.errors.join('; ')}`);
  return manifest;
}

function writeFileAtomic(filePath, content, mode) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporary = path.join(
    path.dirname(filePath),
    `.${path.basename(filePath)}.${process.pid}.${crypto.randomBytes(6).toString('hex')}.tmp`
  );
  try {
    fs.writeFileSync(temporary, content, { mode });
    fs.renameSync(temporary, filePath);
    if (mode !== undefined) fs.chmodSync(filePath, mode);
  } catch (error) {
    try {
      if (fs.existsSync(temporary)) fs.unlinkSync(temporary);
    } catch {
      // Preserve the original write error.
    }
    throw error;
  }
}

function writeManifest(targetRoot, manifest) {
  const validation = validateManifest(manifest, { allowLegacyOwnership: false });
  if (!validation.ok) throw new Error(`Refusing invalid manifest: ${validation.errors.join('; ')}`);
  const carrier = inspectManifestCarrier(targetRoot);
  if (!carrier.ok) throw new Error(carrier.message);
  const file = carrier.file;
  writeFileAtomic(file, `${JSON.stringify(manifest, null, 2)}\n`, 0o644);
  return file;
}

function targetChangedSinceInstall(targetRoot, manifest, relativePath) {
  const target = normalizeRelativePath(relativePath);
  const entry = manifest && manifest.files ? manifest.files[target] : null;
  const file = path.join(path.resolve(targetRoot), target);
  if (!entry || !fs.existsSync(file) || !fs.statSync(file).isFile()) return true;
  const stat = fs.statSync(file);
  return (
    sha256File(file) !== entry.installedSha256 ||
    (validMode(entry.installedMode) && (stat.mode & 0o777) !== entry.installedMode)
  );
}

module.exports = {
  MANIFEST_RELATIVE_PATH,
  MANIFEST_SCHEMA_VERSION,
  inspectManifestCarrier,
  lstatIfPresent,
  makeManifest,
  manifestPath,
  normalizeRelativePath,
  readManifest,
  sha256Buffer,
  sha256File,
  targetChangedSinceInstall,
  validateManifest,
  writeFileAtomic,
  writeManifest
};
