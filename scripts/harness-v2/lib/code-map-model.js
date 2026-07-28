'use strict';

const fs = require('node:fs');
const path = require('node:path');

const MAP_INDEX_PATH = 'docs/code-map/index.json';
const PATH_FIELDS = ['canonical', 'entrypoints', 'consumers', 'state', 'tests'];
const STRING_ARRAY_FIELDS = [...PATH_FIELDS, 'publicSymbols', 'related', 'knownDuplicates', 'keywords'];
const STATUS_VALUES = new Set(['active', 'candidate', 'legacy', 'dead', 'needs-confirmation']);

function toPosix(value) {
  return String(value || '').split(path.sep).join('/').replace(/^\.\//, '');
}

function isObject(value) {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function safeRelativePath(value) {
  const normalized = toPosix(value);
  if (!normalized || normalized.startsWith('/') || normalized.includes('\\')) return null;
  const segments = normalized.split('/');
  if (segments.includes('..') || segments.includes('.') || path.posix.normalize(normalized) !== normalized) return null;
  return normalized;
}

function stripFragment(value) {
  return toPosix(value).split('#')[0];
}

function defaultReadFile(target, relativePath) {
  const full = path.join(target, relativePath);
  if (!fs.existsSync(full)) return null;
  return fs.readFileSync(full, 'utf8');
}

function parseJson(text, relativePath) {
  try {
    return { data: JSON.parse(text), error: null };
  } catch (error) {
    return { data: null, error: `${relativePath}: invalid JSON` };
  }
}

function loadCodeMap(target, options = {}) {
  const readFile = options.readFile || ((relativePath) => defaultReadFile(target, relativePath));
  const errors = [];
  const files = [];
  const indexText = readFile(MAP_INDEX_PATH);
  if (indexText == null) {
    return { index: null, domains: [], entries: [], files, errors: [`Missing ${MAP_INDEX_PATH}`] };
  }
  const parsedIndex = parseJson(indexText, MAP_INDEX_PATH);
  if (parsedIndex.error) {
    return { index: null, domains: [], entries: [], files, errors: [parsedIndex.error] };
  }
  const index = parsedIndex.data;
  files.push(MAP_INDEX_PATH);
  if (!isObject(index) || index.schemaVersion !== 1 || !Array.isArray(index.domains)) {
    errors.push(`${MAP_INDEX_PATH}: expected schemaVersion=1 and domains array`);
  }
  if (
    index && index.seedHead !== undefined && index.seedHead !== null &&
    (typeof index.seedHead !== 'string' || !/^[0-9a-f]{40}$/i.test(index.seedHead))
  ) {
    errors.push(`${MAP_INDEX_PATH}: seedHead must be null or a full Git commit id`);
  }

  const domains = [];
  const entries = [];
  const seenDomains = new Set();
  for (const descriptor of Array.isArray(index && index.domains) ? index.domains : []) {
    const domainFile = descriptor && safeRelativePath(descriptor.file);
    if (!descriptor || typeof descriptor.id !== 'string' || !descriptor.id.trim()) {
      errors.push(`${MAP_INDEX_PATH}: domain descriptor missing id`);
      continue;
    }
    if (seenDomains.has(descriptor.id)) {
      errors.push(`${MAP_INDEX_PATH}: duplicate domain ${descriptor.id}`);
      continue;
    }
    seenDomains.add(descriptor.id);
    if (!domainFile || !domainFile.startsWith('docs/code-map/domains/') || !domainFile.endsWith('.json')) {
      errors.push(`${MAP_INDEX_PATH}: invalid domain path`);
      continue;
    }
    const text = readFile(domainFile);
    if (text == null) {
      errors.push(`Missing domain map: ${domainFile}`);
      continue;
    }
    const parsed = parseJson(text, domainFile);
    if (parsed.error) {
      errors.push(parsed.error);
      continue;
    }
    const domain = parsed.data;
    files.push(domainFile);
    domains.push({ descriptor, file: domainFile, data: domain });
    if (
      !isObject(domain) || domain.schemaVersion !== 1 ||
      domain.domain !== descriptor.id || !Array.isArray(domain.capabilities)
    ) {
      errors.push(`${domainFile}: descriptor/domain/schema mismatch`);
      continue;
    }
    for (const capability of domain.capabilities) {
      entries.push(Object.assign({}, capability, {
        domain: descriptor.id,
        mapFile: domainFile
      }));
    }
  }
  return { index, domains, entries, files, errors };
}

function validateCodeMap(model, options = {}) {
  const fail = [...model.errors];
  const seenIds = new Map();
  const knownIds = new Set(model.entries.map((item) => item && item.id).filter(Boolean));
  for (const entry of model.entries) {
    if (!isObject(entry)) {
      fail.push('Capability entry must be an object');
      continue;
    }
    if (typeof entry.id !== 'string' || !entry.id.trim()) {
      fail.push(`${entry.mapFile}: capability missing id`);
      continue;
    }
    if (seenIds.has(entry.id)) fail.push(`Duplicate capability id: ${entry.id}`);
    else seenIds.set(entry.id, entry.mapFile);
    if (typeof entry.name !== 'string' || !entry.name.trim()) fail.push(`${entry.id}: name missing`);
    if (!STATUS_VALUES.has(entry.status)) fail.push(`${entry.id}: unsupported status`);

    for (const field of STRING_ARRAY_FIELDS) {
      if (entry[field] === undefined) continue;
      if (!Array.isArray(entry[field]) || entry[field].some((item) => typeof item !== 'string')) {
        fail.push(`${entry.id}: ${field} must be strings`);
      }
    }
    for (const field of PATH_FIELDS) {
      for (const reference of Array.isArray(entry[field]) ? entry[field] : []) {
        const relativePath = safeRelativePath(stripFragment(reference));
        if (!relativePath) fail.push(`${entry.id}: invalid ${field} path`);
        else if (options.pathExists && !options.pathExists(relativePath)) {
          fail.push(`${entry.id}: dangling ${field} path ${reference}`);
        }
      }
    }
    for (const reference of Array.isArray(entry.knownDuplicates) ? entry.knownDuplicates : []) {
      if (!reference.includes('/')) continue;
      const relativePath = safeRelativePath(stripFragment(reference));
      if (!relativePath) fail.push(`${entry.id}: invalid knownDuplicates path`);
      else if (options.pathExists && !options.pathExists(relativePath)) {
        fail.push(`${entry.id}: dangling knownDuplicates path ${reference}`);
      }
    }
    for (const related of Array.isArray(entry.related) ? entry.related : []) {
      if (!knownIds.has(related)) fail.push(`${entry.id}: missing related capability ${related}`);
    }
  }
  return { fail: [...new Set(fail)], warn: [] };
}

function entryPathReferences(entry) {
  const references = [];
  for (const field of PATH_FIELDS) {
    for (const value of Array.isArray(entry[field]) ? entry[field] : []) {
      references.push({ field, value, path: stripFragment(value) });
    }
  }
  for (const value of Array.isArray(entry.knownDuplicates) ? entry.knownDuplicates : []) {
    if (value.includes('/')) references.push({ field: 'knownDuplicates', value, path: stripFragment(value) });
  }
  return references;
}

function entriesForPath(entries, relativePath) {
  const normalized = toPosix(relativePath);
  return entries.filter((entry) => entryPathReferences(entry).some((reference) => reference.path === normalized));
}

function normalizeSearch(value) {
  return String(value || '')
    .toLocaleLowerCase()
    .replace(/[._/\\:-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function searchEntries(entries, query, limit = 20) {
  const normalized = normalizeSearch(query);
  const tokens = normalized.split(' ').filter(Boolean);
  if (tokens.length === 0) return [];
  return entries
    .map((entry) => {
      const values = [
        entry.id,
        entry.domain,
        entry.name,
        entry.summary,
        ...(entry.keywords || []),
        ...(entry.publicSymbols || []),
        ...entryPathReferences(entry).flatMap((reference) => [reference.value, path.posix.basename(reference.path)])
      ];
      const text = normalizeSearch(values.join(' '));
      if (!tokens.every((token) => text.includes(token))) return null;
      let score = tokens.reduce((sum, token) => sum + text.split(token).length - 1, 0);
      if (normalizeSearch(entry.id) === normalized || normalizeSearch(entry.name) === normalized) score += 100;
      return { entry, score };
    })
    .filter(Boolean)
    .sort((left, right) => right.score - left.score || left.entry.id.localeCompare(right.entry.id))
    .slice(0, limit);
}

module.exports = {
  MAP_INDEX_PATH,
  entriesForPath,
  entryPathReferences,
  loadCodeMap,
  safeRelativePath,
  searchEntries,
  stripFragment,
  validateCodeMap
};
