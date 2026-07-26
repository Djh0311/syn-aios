#!/usr/bin/env node
'use strict';

/*
 * A deliberately small, map-only navigator.  It validates source references
 * against HEAD so a dirty/untracked implementation cannot become canonical by
 * accident.  Working-tree changes are reported separately through `overlay`.
 */

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const INDEX_PATH = 'docs/code-map/index.json';
const DOMAIN_IDS = [
  'conversation-transport',
  'syn-mcp-supervision',
  'workflow-execution-governance',
  'persistence-canonical-state',
  'ui-shared-foundation',
  'development-harness',
];
const CAPABILITY_FIELDS = [
  'id',
  'domain',
  'name',
  'status',
  'coverage',
  'canonical',
  'entrypoints',
  'publicSymbols',
  'consumers',
  'stateOwners',
  'contracts',
  'tests',
  'related',
  'knownDuplicates',
  'keywords',
  'verifiedAtCommit',
];
const REF_LIST_FIELDS = [
  'entrypoints',
  'publicSymbols',
  'consumers',
  'stateOwners',
  'contracts',
  'tests',
];
const STATUS_ORDER = {
  active: 0,
  candidate: 1,
  'needs-confirmation': 2,
  legacy: 3,
  dead: 4,
};
const COVERAGES = new Set(['seed-partial', 'verified-partial', 'verified']);
const SHA1 = /^[0-9a-f]{40}$/;

class ToolError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function normalize(value) {
  return String(value || '')
    .normalize('NFKC')
    .toLocaleLowerCase('en-US')
    .replace(/\s+/g, ' ')
    .trim();
}

function parseArgs(argv) {
  const args = [...argv];
  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    return { help: true };
  }

  const command = args.shift();
  if (!['query', 'overlay', 'check'].includes(command)) {
    throw new ToolError('USAGE_ERROR', `Unknown command: ${command}`);
  }

  const options = {
    command,
    target: '.',
    query: null,
    json: false,
    staged: false,
    strict: false,
  };
  while (args.length > 0) {
    const option = args.shift();
    if (option === '--target') {
      if (!args.length) throw new ToolError('USAGE_ERROR', '--target needs a directory');
      options.target = args.shift();
    } else if (option === '--query') {
      if (!args.length) throw new ToolError('USAGE_ERROR', '--query needs text');
      options.query = args.shift();
    } else if (option === '--json') {
      options.json = true;
    } else if (option === '--staged') {
      options.staged = true;
    } else if (option === '--strict') {
      options.strict = true;
    } else if (option === '--help' || option === '-h') {
      options.help = true;
    } else {
      throw new ToolError('USAGE_ERROR', `Unknown option: ${option}`);
    }
  }
  if (options.command === 'query' && !normalize(options.query)) {
    throw new ToolError('USAGE_ERROR', 'query requires --query <text>');
  }
  return options;
}

function git(target, args) {
  const result = spawnSync('git', args, { cwd: target, encoding: 'utf8' });
  if (result.error) {
    throw new ToolError('GIT_ERROR', result.error.message);
  }
  return {
    status: result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
  };
}

function repositoryRoot(target) {
  const result = git(path.resolve(target), ['rev-parse', '--show-toplevel']);
  if (result.status !== 0) {
    throw new ToolError('NOT_GIT_REPOSITORY', `Not a Git repository: ${target}`);
  }
  return result.stdout.trim();
}

function readJson(target, relativePath, errors) {
  const filePath = path.join(target, relativePath);
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    errors.push({
      code: 'SCHEMA_ERROR',
      path: relativePath,
      message: `Cannot read JSON: ${error.message}`,
    });
    return null;
  }
}

function error(errors, code, message, extra = {}) {
  errors.push({ code, message, ...extra });
}

function isSafeRelativePath(value) {
  if (typeof value !== 'string' || value.length === 0) return false;
  if (value.includes('\\') || value.includes('\0') || value.startsWith('/') || value.startsWith('~')) {
    return false;
  }
  if (value.split('/').some((part) => part === '' || part === '.' || part === '..')) return false;
  return path.posix.normalize(value) === value;
}

function validateSourceRef(value, context, trackedAtHead, errors, requireSymbol = false) {
  if (!isObject(value)) {
    error(errors, 'SCHEMA_ERROR', `${context} must be a source reference object`);
    return;
  }
  if (!isSafeRelativePath(value.path)) {
    error(errors, 'INVALID_REPO_RELATIVE_PATH', `${context}.path must be a safe repository-relative path`);
  } else if (!trackedAtHead.has(value.path)) {
    error(errors, 'MAP_PATH_NOT_TRACKED', `${context}.path is not Git tracked at HEAD: ${value.path}`, {
      path: value.path,
    });
  }
  if (Object.hasOwn(value, 'symbol') && typeof value.symbol !== 'string') {
    error(errors, 'SCHEMA_ERROR', `${context}.symbol must be a string when present`);
  }
  if (requireSymbol && (typeof value.symbol !== 'string' || value.symbol.length === 0)) {
    error(errors, 'SCHEMA_ERROR', `${context}.symbol is required`);
  }
  if (Object.hasOwn(value, 'kind') && typeof value.kind !== 'string') {
    error(errors, 'SCHEMA_ERROR', `${context}.kind must be a string when present`);
  }
}

function collectRefs(capability) {
  const refs = [];
  if (isObject(capability.canonical)) refs.push(capability.canonical);
  for (const field of REF_LIST_FIELDS) {
    if (Array.isArray(capability[field])) refs.push(...capability[field].filter(isObject));
  }
  return refs;
}

function parseNameStatusZ(output) {
  const tokens = output.split('\0');
  if (tokens[tokens.length - 1] === '') tokens.pop();
  const changes = [];
  for (let index = 0; index < tokens.length;) {
    const status = tokens[index++];
    if (!status) continue;
    const code = status[0];
    if (code === 'R' || code === 'C') {
      const from = tokens[index++];
      const to = tokens[index++];
      if (from && to) changes.push({ code, status, from, to });
    } else {
      const filePath = tokens[index++];
      if (filePath) changes.push({ code, status, path: filePath });
    }
  }
  return changes;
}

function trackedPathsAtHead(target) {
  const result = git(target, ['ls-tree', '-r', '-z', '--name-only', 'HEAD']);
  if (result.status !== 0) throw new ToolError('GIT_ERROR', result.stderr.trim() || 'Cannot list HEAD paths');
  return new Set(result.stdout.split('\0').filter(Boolean));
}

function hasCommit(target, commit, cache) {
  if (cache.has(commit)) return cache.get(commit);
  const result = git(target, ['cat-file', '-e', `${commit}^{commit}`]);
  const exists = result.status === 0;
  cache.set(commit, exists);
  return exists;
}

function isSourceIdentifier(value) {
  return typeof value === 'string' && /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(value);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function sourceAtCommit(target, commit, relativePath, cache) {
  const key = `${commit}:${relativePath}`;
  if (cache.has(key)) return cache.get(key);
  const result = git(target, ['show', key]);
  const source = result.status === 0 ? result.stdout : null;
  cache.set(key, source);
  return source;
}

function sourceContainsPublicDeclaration(source, symbol) {
  const escaped = escapeRegExp(symbol);
  const declarations = [
    `\\bexport\\s+(?:default\\s+)?(?:async\\s+)?(?:function|class|const|let|var|type|interface|enum)\\s+${escaped}\\b`,
    `\\bexport\\s*\\{[^}]*\\b${escaped}\\b[^}]*\\}`,
    `\\bpub(?:\\([^)]*\\))?\\s+(?:async\\s+)?(?:fn|struct|enum|trait|type|const|static)\\s+${escaped}\\b`,
  ];
  return declarations.some((pattern) => new RegExp(pattern).test(source));
}

function validateMap(target) {
  const errors = [];
  const warnings = [];
  const trackedAtHead = trackedPathsAtHead(target);
  const index = readJson(target, INDEX_PATH, errors);
  const capabilityEntries = [];
  const commits = new Map();
  const sourcesAtCommit = new Map();

  if (!isObject(index)) {
    return { errors, warnings, capabilities: [], trackedAtHead };
  }
  if (index.schemaVersion !== 1) error(errors, 'SCHEMA_ERROR', 'index.schemaVersion must be 1', { path: INDEX_PATH });
  if (!COVERAGES.has(index.coverage)) error(errors, 'SCHEMA_ERROR', 'index.coverage is invalid', { path: INDEX_PATH });
  if (!Array.isArray(index.domains)) {
    error(errors, 'SCHEMA_ERROR', 'index.domains must be an array', { path: INDEX_PATH });
    return { errors, warnings, capabilities: [], trackedAtHead };
  }

  const seenDomains = new Set();
  const domainRows = [];
  for (const entry of index.domains) {
    if (!isObject(entry) || typeof entry.id !== 'string') {
      error(errors, 'SCHEMA_ERROR', 'each index domain needs an id', { path: INDEX_PATH });
      continue;
    }
    if (seenDomains.has(entry.id)) error(errors, 'DUPLICATE_DOMAIN_ID', `duplicate domain id: ${entry.id}`);
    seenDomains.add(entry.id);
    const expectedPath = `docs/code-map/domains/${entry.id}.json`;
    if (!DOMAIN_IDS.includes(entry.id)) error(errors, 'SCHEMA_ERROR', `unexpected domain id: ${entry.id}`);
    if (entry.path !== expectedPath) error(errors, 'SCHEMA_ERROR', `domain ${entry.id} must use ${expectedPath}`);
    if (typeof entry.name !== 'string' || !COVERAGES.has(entry.coverage)) {
      error(errors, 'SCHEMA_ERROR', `domain ${entry.id} needs name and valid coverage`);
    }
    domainRows.push({ id: entry.id, mapPath: expectedPath });
  }
  for (const id of DOMAIN_IDS) {
    if (!seenDomains.has(id)) error(errors, 'MISSING_DOMAIN', `required domain missing: ${id}`);
  }
  if (index.domains.length !== DOMAIN_IDS.length) {
    error(errors, 'SCHEMA_ERROR', 'index must contain exactly the six required domains');
  }

  for (const row of domainRows) {
    if (!DOMAIN_IDS.includes(row.id)) continue;
    const domain = readJson(target, row.mapPath, errors);
    if (!isObject(domain)) continue;
    if (domain.schemaVersion !== 1) error(errors, 'SCHEMA_ERROR', `${row.mapPath}.schemaVersion must be 1`);
    if (domain.domain !== row.id) error(errors, 'SCHEMA_ERROR', `${row.mapPath}.domain must be ${row.id}`);
    if (!COVERAGES.has(domain.coverage)) error(errors, 'SCHEMA_ERROR', `${row.mapPath}.coverage is invalid`);
    if (!Array.isArray(domain.capabilities)) {
      error(errors, 'SCHEMA_ERROR', `${row.mapPath}.capabilities must be an array`);
      continue;
    }
    for (let position = 0; position < domain.capabilities.length; position += 1) {
      const capability = domain.capabilities[position];
      const context = `${row.mapPath}.capabilities[${position}]`;
      if (!isObject(capability)) {
        error(errors, 'SCHEMA_ERROR', `${context} must be an object`);
        continue;
      }
      for (const field of CAPABILITY_FIELDS) {
        if (!Object.hasOwn(capability, field)) error(errors, 'SCHEMA_ERROR', `${context}.${field} is required`);
      }
      if (typeof capability.id !== 'string' || !capability.id.startsWith(`${row.id}.`)) {
        error(errors, 'SCHEMA_ERROR', `${context}.id must be prefixed with ${row.id}.`);
      }
      if (capability.domain !== row.id) error(errors, 'SCHEMA_ERROR', `${context}.domain must be ${row.id}`);
      if (typeof capability.name !== 'string' || capability.name.length === 0) error(errors, 'SCHEMA_ERROR', `${context}.name is required`);
      if (!Object.hasOwn(STATUS_ORDER, capability.status)) error(errors, 'SCHEMA_ERROR', `${context}.status is invalid`);
      if (!COVERAGES.has(capability.coverage)) error(errors, 'SCHEMA_ERROR', `${context}.coverage is invalid`);
      if (capability.canonical !== null && !isObject(capability.canonical)) {
        error(errors, 'SCHEMA_ERROR', `${context}.canonical must be null or a source reference`);
      }
      if (isObject(capability.canonical)) {
        validateSourceRef(capability.canonical, `${context}.canonical`, trackedAtHead, errors);
      }
      if (capability.status === 'active' && !isObject(capability.canonical)) {
        error(errors, 'SCHEMA_ERROR', `${context}.canonical is required for active capability`);
      }
      for (const field of REF_LIST_FIELDS) {
        if (!Array.isArray(capability[field])) {
          error(errors, 'SCHEMA_ERROR', `${context}.${field} must be an array`);
          continue;
        }
        capability[field].forEach((ref, refPosition) => {
          validateSourceRef(
            ref,
            `${context}.${field}[${refPosition}]`,
            trackedAtHead,
            errors,
            field === 'publicSymbols',
          );
        });
      }
      for (const field of ['related', 'knownDuplicates']) {
        if (!Array.isArray(capability[field])) {
          error(errors, 'SCHEMA_ERROR', `${context}.${field} must be an array`);
          continue;
        }
        capability[field].forEach((relation, relationPosition) => {
          if (!isObject(relation) || typeof relation.id !== 'string' || typeof relation.relationship !== 'string' || !relation.relationship) {
            error(errors, 'SCHEMA_ERROR', `${context}.${field}[${relationPosition}] needs id and relationship`);
          }
        });
      }
      if (!Array.isArray(capability.keywords) || capability.keywords.some((keyword) => typeof keyword !== 'string' || !keyword)) {
        error(errors, 'SCHEMA_ERROR', `${context}.keywords must be a non-empty-string array`);
      }
      if (capability.verifiedAtCommit !== null) {
        if (typeof capability.verifiedAtCommit !== 'string' || !SHA1.test(capability.verifiedAtCommit)) {
          error(errors, 'SCHEMA_ERROR', `${context}.verifiedAtCommit must be a full commit SHA or null`);
        } else if (!hasCommit(target, capability.verifiedAtCommit, commits)) {
          error(errors, 'UNKNOWN_VERIFICATION_COMMIT', `${context}.verifiedAtCommit is not available: ${capability.verifiedAtCommit}`);
        }
      }
      if (capability.status === 'active' && capability.verifiedAtCommit === null) {
        error(errors, 'SCHEMA_ERROR', `${context}.verifiedAtCommit is required for active capability`);
      }
      if (Array.isArray(capability.publicSymbols) && capability.publicSymbols.length > 0) {
        const commit = capability.verifiedAtCommit;
        const commitIsAvailable = typeof commit === 'string' && SHA1.test(commit) && hasCommit(target, commit, commits);
        if (!commitIsAvailable) {
          error(errors, 'MAP_PUBLIC_SYMBOL_UNVERIFIABLE', `${context}.publicSymbols require an available verifiedAtCommit`);
        } else {
          capability.publicSymbols.forEach((ref, refPosition) => {
            const publicContext = `${context}.publicSymbols[${refPosition}]`;
            if (!isObject(ref) || !isSafeRelativePath(ref.path) || !trackedAtHead.has(ref.path)) return;
            if (!isSourceIdentifier(ref.symbol)) {
              error(errors, 'MAP_PUBLIC_SYMBOL_NOT_FOUND', `${publicContext}.symbol must be a source identifier present at ${commit}:${ref.path}`, {
                path: ref.path,
                symbol: ref.symbol,
                verifiedAtCommit: commit,
              });
              return;
            }
            const source = sourceAtCommit(target, commit, ref.path, sourcesAtCommit);
            if (source === null) {
              error(errors, 'MAP_PUBLIC_SYMBOL_SOURCE_UNAVAILABLE', `${publicContext} cannot read ${commit}:${ref.path}`, {
                path: ref.path,
                symbol: ref.symbol,
                verifiedAtCommit: commit,
              });
            } else if (!sourceContainsPublicDeclaration(source, ref.symbol)) {
              error(errors, 'MAP_PUBLIC_SYMBOL_NOT_FOUND', `${publicContext}.symbol is not declared at ${commit}:${ref.path}: ${ref.symbol}`, {
                path: ref.path,
                symbol: ref.symbol,
                verifiedAtCommit: commit,
              });
            }
          });
        }
      }
      capabilityEntries.push({ capability, context });
    }
  }

  const byId = new Map();
  for (const item of capabilityEntries) {
    const id = item.capability.id;
    if (typeof id !== 'string') continue;
    if (byId.has(id)) error(errors, 'DUPLICATE_CAPABILITY_ID', `duplicate capability id: ${id}`);
    else byId.set(id, item.capability);
  }
  for (const item of capabilityEntries) {
    for (const field of ['related', 'knownDuplicates']) {
      if (!Array.isArray(item.capability[field])) continue;
      for (const relation of item.capability[field]) {
        if (isObject(relation) && typeof relation.id === 'string' && !byId.has(relation.id)) {
          error(errors, 'UNRESOLVED_CAPABILITY_REFERENCE', `${item.capability.id}.${field} references ${relation.id}`);
        }
      }
    }
  }
  return {
    errors,
    warnings,
    capabilities: capabilityEntries.map((item) => item.capability),
    trackedAtHead,
  };
}

function queryMap(map, query) {
  const needle = normalize(query);
  const results = [];
  for (const capability of map.capabilities) {
    const fields = [
      ['id', capability.id],
      ['name', capability.name],
      ['keywords', ...(Array.isArray(capability.keywords) ? capability.keywords : [])],
      ['publicSymbols', ...(Array.isArray(capability.publicSymbols) ? capability.publicSymbols.map((ref) => ref.symbol || '') : [])],
      ['paths', ...collectRefs(capability).map((ref) => ref.path || '')],
    ];
    const matchedFields = [];
    let score = 0;
    for (const [field, ...values] of fields) {
      for (const rawValue of values) {
        const value = normalize(rawValue);
        if (!value || !value.includes(needle)) continue;
        if (!matchedFields.includes(field)) matchedFields.push(field);
        if (value === needle) score += field === 'keywords' ? 160 : 120;
        else score += field === 'keywords' ? 100 : 60;
      }
    }
    if (matchedFields.length > 0) {
      results.push({
        ...capability,
        matchedFields,
        score,
      });
    }
  }
  results.sort((left, right) =>
    (STATUS_ORDER[left.status] - STATUS_ORDER[right.status])
    || (right.score - left.score)
    || left.id.localeCompare(right.id),
  );
  return results;
}

function stagedImpacts(target, capabilities) {
  const result = git(target, ['diff', '--cached', '--name-status', '-z', '--find-renames']);
  if (result.status !== 0) throw new ToolError('GIT_ERROR', result.stderr.trim() || 'Cannot inspect staged changes');
  const impacts = [];
  const seen = new Set();
  for (const change of parseNameStatusZ(result.stdout)) {
    if (change.code !== 'R' && change.code !== 'D') continue;
    for (const capability of capabilities) {
      const paths = new Set(collectRefs(capability).map((ref) => ref.path));
      if (change.code === 'R' && paths.has(change.from)) {
        const key = `${capability.id}:rename:${change.from}:${change.to}`;
        if (!seen.has(key)) {
          seen.add(key);
          impacts.push({ kind: 'rename', capabilityId: capability.id, from: change.from, to: change.to });
        }
      }
      if (change.code === 'D' && paths.has(change.path)) {
        const key = `${capability.id}:delete:${change.path}`;
        if (!seen.has(key)) {
          seen.add(key);
          impacts.push({ kind: 'delete', capabilityId: capability.id, path: change.path });
        }
      }
    }
  }
  return impacts;
}

function overlay(target, capabilities) {
  const unstaged = git(target, ['diff', '--name-status', '-z']);
  const untracked = git(target, ['ls-files', '--others', '--exclude-standard', '-z']);
  if (unstaged.status !== 0 || untracked.status !== 0) throw new ToolError('GIT_ERROR', 'Cannot inspect working-tree overlay');
  const references = new Map();
  for (const capability of capabilities) {
    for (const ref of collectRefs(capability)) {
      if (!references.has(ref.path)) references.set(ref.path, new Set());
      references.get(ref.path).add(capability.id);
    }
  }
  const entries = [];
  for (const change of parseNameStatusZ(unstaged.stdout)) {
    if (change.code === 'R' || change.code === 'C') {
      for (const changedPath of [change.from, change.to]) {
        entries.push({
          kind: 'unstaged',
          status: change.status,
          path: changedPath,
          capabilityIds: [...(references.get(changedPath) || [])].sort(),
        });
      }
    } else {
      entries.push({
        kind: 'unstaged',
        status: change.status,
        path: change.path,
        capabilityIds: [...(references.get(change.path) || [])].sort(),
      });
    }
  }
  for (const changedPath of untracked.stdout.split('\0').filter(Boolean)) {
    entries.push({
      kind: 'untracked',
      status: '??',
      path: changedPath,
      capabilityIds: [...(references.get(changedPath) || [])].sort(),
    });
  }
  entries.sort((left, right) => left.path.localeCompare(right.path) || left.kind.localeCompare(right.kind));
  return entries;
}

function helpText() {
  return [
    'Usage: node scripts/harness/codebase-map.js <query|overlay|check> --target <repo> [options]',
    '  query   --query <text> [--json]       Search the partial structured map only.',
    '  overlay [--json]                      Report unstaged and untracked paths without changing the map.',
    '  check   [--staged] [--strict] [--json] Validate map structure and HEAD-tracked references.',
  ].join('\n');
}

function plain(payload) {
  if (payload.status === 'MATCH') {
    return [`MATCH ${payload.query}`, ...payload.results.map((result) => `${result.id} [${result.status}] ${result.name}`)].join('\n');
  }
  if (payload.status === 'NO_MATCH_IN_PARTIAL_MAP') return payload.message;
  if (payload.status === 'OVERLAY') {
    return ['OVERLAY', ...payload.entries.map((entry) => `${entry.kind} ${entry.status} ${entry.path}${entry.capabilityIds.length ? ` -> ${entry.capabilityIds.join(', ')}` : ''}`)].join('\n');
  }
  if (payload.status === 'OK') return `OK${payload.stagedImpacts?.length ? ` (${payload.stagedImpacts.length} staged impact(s))` : ''}`;
  return [payload.status || 'ERROR', ...(payload.errors || []).map((item) => `${item.code}: ${item.message}`)].join('\n');
}

function emit(payload, json) {
  process.stdout.write(`${json ? JSON.stringify(payload) : plain(payload)}\n`);
}

function run(options) {
  if (options.help) {
    process.stdout.write(`${helpText()}\n`);
    return 0;
  }
  const target = repositoryRoot(options.target);
  const map = validateMap(target);
  if (options.command === 'query') {
    if (map.errors.length > 0) {
      emit({ status: 'INVALID_MAP', errors: map.errors }, options.json);
      return 1;
    }
    const results = queryMap(map, options.query);
    if (results.length === 0) {
      emit({
        status: 'NO_MATCH_IN_PARTIAL_MAP',
        query: options.query,
        message: `NO_MATCH_IN_PARTIAL_MAP: ${options.query}; this is a partial map, not evidence that the capability is absent.`,
      }, options.json);
      return 0;
    }
    emit({
      status: 'MATCH',
      query: options.query,
      coverage: 'partial-map-only',
      results: results.map(({ score, ...result }) => result),
    }, options.json);
    return 0;
  }
  if (options.command === 'overlay') {
    if (map.errors.length > 0) {
      emit({ status: 'INVALID_MAP', errors: map.errors }, options.json);
      return 1;
    }
    emit({ status: 'OVERLAY', entries: overlay(target, map.capabilities) }, options.json);
    return 0;
  }

  const staged = options.staged ? stagedImpacts(target, map.capabilities) : [];
  const errors = [...map.errors];
  const warnings = [...map.warnings];
  if (staged.length > 0) {
    const sink = options.strict ? errors : warnings;
    for (const impact of staged) {
      sink.push({
        code: impact.kind === 'rename' ? 'STAGED_RENAME_AFFECTS_CAPABILITY' : 'STAGED_DELETE_AFFECTS_CAPABILITY',
        message: `${impact.kind} affects ${impact.capabilityId}`,
        ...impact,
      });
    }
  }
  emit({
    status: errors.length === 0 ? 'OK' : 'INVALID',
    errors,
    warnings,
    stagedImpacts: staged,
    reviewBoundary: 'Map validation proves only the listed HEAD-tracked navigation references; it does not prove runtime, product, or acceptance state.',
  }, options.json);
  return errors.length === 0 ? 0 : 1;
}

function main() {
  try {
    return run(parseArgs(process.argv.slice(2)));
  } catch (caught) {
    const failure = caught instanceof ToolError ? caught : new ToolError('UNEXPECTED_ERROR', caught.message);
    emit({ status: 'ERROR', errors: [{ code: failure.code, message: failure.message }] }, process.argv.includes('--json'));
    return 2;
  }
}

if (require.main === module) process.exitCode = main();

module.exports = { normalize, parseNameStatusZ, validateMap, queryMap };
