'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { TextDecoder } = require('node:util');

const READ_ONLY_GIT_COMMANDS = new Set(['cat-file', 'diff', 'ls-files', 'rev-parse']);
const SOURCE_EXTENSIONS = new Set([
  '.astro', '.c', '.cc', '.cpp', '.cs', '.cxx', '.ex', '.exs', '.gql', '.go',
  '.graphql', '.h', '.hpp', '.java', '.js', '.jsx', '.kt', '.kts', '.mjs',
  '.cjs', '.php', '.prisma', '.proto', '.py', '.rb', '.rs', '.scala', '.sql',
  '.swift', '.svelte', '.ts', '.tsx', '.vue'
]);
const EXCLUDED_SEGMENTS = new Set([
  '.git', '.next', '.turbo', '__fixtures__', '__mocks__', '__snapshots__',
  '__tests__', 'build', 'coverage', 'cypress', 'dist', 'e2e', 'fixtures',
  'generated', 'mocks', 'node_modules', 'out', 'playwright', 'spec', 'specs',
  'target', 'test', 'tests', 'vendor'
]);
const CHANGE_KINDS = new Set(['A', 'B', 'C', 'D', 'M', 'R', 'T', 'U', 'X']);

function failure(prefix, result) {
  const detail = String(result.stderr || result.error || '').trim();
  if (detail) return `${prefix}: ${detail}`;
  if (typeof result.status === 'number') return `${prefix}: git exited with ${result.status}`;
  if (result.signal) return `${prefix}: git terminated by ${result.signal}`;
  return `${prefix}: git did not complete`;
}

function git(target, args, options = {}) {
  const empty = options.encoding === null ? Buffer.alloc(0) : '';
  const base = {
    ok: false,
    status: null,
    signal: null,
    stdout: empty,
    stderr: '',
    error: null
  };

  if (typeof target !== 'string' || !target.trim()) {
    return Object.assign(base, { error: 'Git target must be a non-empty path' });
  }
  if (!Array.isArray(args) || args.length === 0 || args.some((item) => typeof item !== 'string')) {
    return Object.assign(base, { error: 'Git arguments must be non-empty strings' });
  }
  if (!READ_ONLY_GIT_COMMANDS.has(args[0])) {
    return Object.assign(base, { error: `Unsupported read-only Git command: ${args[0]}` });
  }
  if (args.some((item) => item === '--output' || item.startsWith('--output='))) {
    return Object.assign(base, { error: 'Git output-file options are forbidden' });
  }

  const encoding = options.encoding === null ? null : 'utf8';
  const timeout = options.timeout === undefined ? 10000 : options.timeout;
  const maxBuffer = options.maxBuffer === undefined ? 16 * 1024 * 1024 : options.maxBuffer;
  if (!Number.isSafeInteger(timeout) || timeout < 1 || timeout > 60000) {
    return Object.assign(base, { error: 'Git timeout must be between 1 and 60000 ms' });
  }
  if (!Number.isSafeInteger(maxBuffer) || maxBuffer < 1024 || maxBuffer > 128 * 1024 * 1024) {
    return Object.assign(base, { error: 'Git maxBuffer must be between 1 KiB and 128 MiB' });
  }

  try {
    const result = spawnSync('git', args, {
      cwd: path.resolve(target),
      encoding,
      env: Object.assign({}, process.env, {
        GIT_LITERAL_PATHSPECS: '1',
        GIT_OPTIONAL_LOCKS: '0'
      }),
      input: options.input,
      maxBuffer,
      timeout,
      windowsHide: true
    });
    const stdout = encoding === null
      ? (Buffer.isBuffer(result.stdout) ? result.stdout : Buffer.alloc(0))
      : String(result.stdout || '');
    const stderr = Buffer.isBuffer(result.stderr)
      ? result.stderr.toString('utf8')
      : String(result.stderr || '');
    const response = {
      ok: result.status === 0 && !result.error,
      status: typeof result.status === 'number' ? result.status : null,
      signal: result.signal || null,
      stdout,
      stderr,
      error: result.error ? result.error.message : null
    };
    if (!response.ok) response.error = failure(`git ${args[0]} failed`, response);
    return response;
  } catch (error) {
    return Object.assign(base, { error: `git ${args[0]} failed: ${error.message}` });
  }
}

function decodeUtf8(value, label) {
  if (typeof value === 'string') return { ok: true, text: value, error: null };
  if (!Buffer.isBuffer(value)) return { ok: false, text: '', error: `${label} is not text` };
  try {
    return {
      ok: true,
      text: new TextDecoder('utf-8', { fatal: true }).decode(value),
      error: null
    };
  } catch (error) {
    return { ok: false, text: '', error: `${label} is not valid UTF-8` };
  }
}

function safeRepoPath(value) {
  if (typeof value !== 'string' || !value || value.length > 32768 || value.includes('\0')) return null;
  if (value.startsWith('/') || value.startsWith('./') || value.endsWith('/') || value.includes('\\')) return null;
  const segments = value.split('/');
  if (segments.some((segment) => !segment || segment === '.' || segment === '..')) return null;
  if (path.posix.normalize(value) !== value) return null;
  return value;
}

function parseNameStatusZ(value) {
  const decoded = decodeUtf8(value, 'Git name-status output');
  if (!decoded.ok) return { ok: false, changes: [], error: decoded.error };
  if (!decoded.text) return { ok: true, changes: [], error: null };
  if (!decoded.text.endsWith('\0')) {
    return { ok: false, changes: [], error: 'Malformed Git name-status output: missing trailing NUL' };
  }

  const tokens = decoded.text.slice(0, -1).split('\0');
  const changes = [];
  let cursor = 0;
  while (cursor < tokens.length) {
    const statusToken = tokens[cursor++];
    const match = statusToken.match(/^([ABCDMRTUX])(\d{1,3})?$/);
    if (!match) return { ok: false, changes: [], error: 'Malformed Git change status' };
    const status = match[1];
    const score = match[2] === undefined ? null : Number(match[2]);
    const pair = status === 'R' || status === 'C';
    if (pair !== (score !== null) || (score !== null && score > 100)) {
      return { ok: false, changes: [], error: 'Malformed Git rename/copy score' };
    }
    const count = pair ? 2 : 1;
    if (cursor + count > tokens.length) {
      return { ok: false, changes: [], error: `Malformed Git output after ${statusToken}` };
    }
    const paths = tokens.slice(cursor, cursor + count).map(safeRepoPath);
    cursor += count;
    if (paths.some((item) => item === null)) {
      return { ok: false, changes: [], error: `Unsafe repository path after ${statusToken}` };
    }
    if (pair) {
      if (paths[0] === paths[1]) return { ok: false, changes: [], error: 'Rename/copy paths must differ' };
      changes.push({
        status,
        path: paths[1],
        oldPath: paths[0],
        newPath: paths[1],
        score
      });
    } else {
      changes.push({
        status,
        path: paths[0],
        oldPath: status === 'D' ? paths[0] : null,
        newPath: status === 'D' ? null : paths[0],
        score: null
      });
    }
  }
  return { ok: true, changes, error: null };
}

function readChanges(target, args, label) {
  const result = git(target, args, { encoding: null, maxBuffer: 32 * 1024 * 1024 });
  if (!result.ok) return { ok: false, changes: [], error: result.error };
  const parsed = parseNameStatusZ(result.stdout);
  if (!parsed.ok) return { ok: false, changes: [], error: `${label}: ${parsed.error}` };
  return parsed;
}

function stagedChanges(target) {
  const raw = stagedRawChanges(target);
  if (!raw.ok) return { ok: false, changes: [], error: raw.error };
  return {
    ok: true,
    changes: raw.changes.map((change) => ({
      status: change.status,
      path: change.path,
      oldPath: change.oldPath,
      newPath: change.newPath,
      score: change.score,
      oldOid: change.oldOid,
      newOid: change.newOid
    })),
    error: null
  };
}

function unstagedChanges(target) {
  return readChanges(
    target,
    ['diff', '--name-status', '-z', '--find-renames', '--no-ext-diff', '--no-textconv'],
    'Unable to parse unstaged changes'
  );
}

function parseNulPaths(value, label) {
  const decoded = decodeUtf8(value, label);
  if (!decoded.ok) return { ok: false, paths: [], error: decoded.error };
  if (!decoded.text) return { ok: true, paths: [], error: null };
  if (!decoded.text.endsWith('\0')) return { ok: false, paths: [], error: `${label}: missing trailing NUL` };
  const paths = decoded.text.slice(0, -1).split('\0').map(safeRepoPath);
  if (paths.some((item) => item === null)) return { ok: false, paths: [], error: `${label}: unsafe path` };
  return { ok: true, paths: [...new Set(paths)], error: null };
}

function untrackedPaths(target) {
  const result = git(target, ['ls-files', '--others', '--exclude-standard', '-z'], {
    encoding: null,
    maxBuffer: 32 * 1024 * 1024
  });
  if (!result.ok) return { ok: false, paths: [], error: result.error };
  return parseNulPaths(result.stdout, 'Unable to parse untracked paths');
}

function parseRawDiffZ(value) {
  const decoded = decodeUtf8(value, 'Git raw diff output');
  if (!decoded.ok) return { ok: false, changes: [], error: decoded.error };
  if (!decoded.text) return { ok: true, changes: [], error: null };
  if (!decoded.text.endsWith('\0')) {
    return { ok: false, changes: [], error: 'Malformed Git raw diff: missing trailing NUL' };
  }
  const tokens = decoded.text.slice(0, -1).split('\0');
  const changes = [];
  let cursor = 0;
  while (cursor < tokens.length) {
    const metadata = tokens[cursor++].match(
      /^:(\d{6}) (\d{6}) ([0-9a-f]{40}|[0-9a-f]{64}) ([0-9a-f]{40}|[0-9a-f]{64}) ([ABCDMRTUX])(\d{1,3})?$/
    );
    if (!metadata) return { ok: false, changes: [], error: 'Malformed Git raw diff metadata' };
    const status = metadata[5];
    const score = metadata[6] === undefined ? null : Number(metadata[6]);
    const pair = status === 'R' || status === 'C';
    const pathCount = pair ? 2 : 1;
    if (cursor + pathCount > tokens.length) {
      return { ok: false, changes: [], error: 'Malformed Git raw diff paths' };
    }
    const paths = tokens.slice(cursor, cursor + pathCount).map(safeRepoPath);
    cursor += pathCount;
    if (paths.some((item) => item === null)) {
      return { ok: false, changes: [], error: 'Unsafe path in Git raw diff' };
    }
    const oldPath = pair ? paths[0] : (status === 'D' ? paths[0] : null);
    const newPath = pair ? paths[1] : (status === 'D' ? null : paths[0]);
    changes.push({
      status,
      path: newPath || oldPath,
      oldPath,
      newPath,
      score,
      oldMode: metadata[1],
      newMode: metadata[2],
      oldOid: metadata[3],
      newOid: metadata[4]
    });
  }
  return { ok: true, changes, error: null };
}

function stagedRawChanges(target) {
  const result = git(target, [
    'diff',
    '--cached',
    '--raw',
    '-z',
    '--no-abbrev',
    '--find-renames',
    '--no-ext-diff',
    '--no-textconv'
  ], { encoding: null, maxBuffer: 32 * 1024 * 1024 });
  if (!result.ok) return { ok: false, changes: [], error: result.error };
  return parseRawDiffZ(result.stdout);
}

function readObjects(target, objectIds) {
  const zero = /^(?:0{40}|0{64})$/;
  const ids = [...new Set(objectIds.filter((oid) => typeof oid === 'string' && !zero.test(oid)))];
  if (ids.length === 0) return { ok: true, objects: new Map(), error: null };
  const result = git(target, ['cat-file', '--batch'], {
    encoding: null,
    input: Buffer.from(`${ids.join('\n')}\n`, 'utf8'),
    maxBuffer: 128 * 1024 * 1024
  });
  if (!result.ok) return { ok: false, objects: new Map(), error: result.error };
  const objects = new Map();
  let offset = 0;
  for (const expected of ids) {
    const newline = result.stdout.indexOf(0x0a, offset);
    if (newline < 0) return { ok: false, objects: new Map(), error: 'Malformed cat-file header' };
    const header = result.stdout.subarray(offset, newline).toString('utf8').split(' ');
    const oid = header[0];
    const type = header[1];
    const size = Number(header[2]);
    if (oid !== expected || type !== 'blob' || !Number.isSafeInteger(size) || size < 0) {
      return { ok: false, objects: new Map(), error: 'Unexpected cat-file object' };
    }
    const start = newline + 1;
    const end = start + size;
    if (end > result.stdout.length) {
      return { ok: false, objects: new Map(), error: 'Truncated cat-file object' };
    }
    objects.set(oid, result.stdout.subarray(start, end));
    offset = end + 1;
  }
  return { ok: true, objects, error: null };
}

function indexEntry(target, relativePath) {
  const file = safeRepoPath(relativePath);
  if (!file) return { ok: false, exists: false, entry: null, error: 'Unsafe index path' };
  const result = git(target, ['ls-files', '--stage', '-z', '--', file], { encoding: null });
  if (!result.ok) return { ok: false, exists: false, entry: null, error: result.error };
  const decoded = decodeUtf8(result.stdout, `Index metadata for ${file}`);
  if (!decoded.ok) return { ok: false, exists: false, entry: null, error: decoded.error };
  if (!decoded.text) return { ok: true, exists: false, entry: null, error: null };
  if (!decoded.text.endsWith('\0')) {
    return { ok: false, exists: false, entry: null, error: `Malformed index metadata for ${file}` };
  }
  const records = decoded.text.slice(0, -1).split('\0');
  if (records.length !== 1) return { ok: false, exists: false, entry: null, error: `Unmerged index path: ${file}` };
  const tab = records[0].indexOf('\t');
  const metadata = tab >= 0
    ? records[0].slice(0, tab).match(/^(\d{6}) ([0-9a-f]{40}|[0-9a-f]{64}) ([0-3])$/)
    : null;
  const entryPath = tab >= 0 ? safeRepoPath(records[0].slice(tab + 1)) : null;
  if (!metadata || entryPath !== file || metadata[3] !== '0') {
    return { ok: false, exists: false, entry: null, error: `Unexpected index entry for ${file}` };
  }
  return {
    ok: true,
    exists: true,
    entry: { mode: metadata[1], oid: metadata[2], stage: 0, path: file },
    error: null
  };
}

function indexPathExists(target, relativePath) {
  const result = indexEntry(target, relativePath);
  return { ok: result.ok, exists: result.exists, error: result.error };
}

function readIndexFile(target, relativePath) {
  const indexed = indexEntry(target, relativePath);
  if (!indexed.ok || !indexed.exists) {
    return {
      ok: indexed.ok,
      exists: indexed.exists,
      path: safeRepoPath(relativePath),
      oid: null,
      content: null,
      error: indexed.error
    };
  }
  const blob = git(target, ['cat-file', 'blob', indexed.entry.oid], {
    encoding: null,
    maxBuffer: 64 * 1024 * 1024
  });
  if (!blob.ok) {
    return {
      ok: false,
      exists: true,
      path: indexed.entry.path,
      oid: indexed.entry.oid,
      content: null,
      error: blob.error
    };
  }
  const decoded = decodeUtf8(blob.stdout, `Index blob for ${indexed.entry.path}`);
  return {
    ok: decoded.ok,
    exists: true,
    path: indexed.entry.path,
    oid: indexed.entry.oid,
    content: decoded.ok ? decoded.text : null,
    error: decoded.error
  };
}

function worktreePathExists(target, relativePath) {
  const file = safeRepoPath(relativePath);
  if (!file) return { ok: false, exists: false, error: 'Unsafe worktree path' };
  let root;
  try {
    root = fs.realpathSync(path.resolve(target));
  } catch (error) {
    return { ok: false, exists: false, error: 'Unable to resolve worktree root' };
  }
  let cursor = root;
  for (const segment of file.split('/')) {
    cursor = path.join(cursor, segment);
    try {
      const stat = fs.lstatSync(cursor);
      if (stat.isSymbolicLink()) return { ok: false, exists: false, error: `Refusing symlink path: ${file}` };
    } catch (error) {
      if (error.code === 'ENOENT' || error.code === 'ENOTDIR') {
        return { ok: true, exists: false, error: null };
      }
      return { ok: false, exists: false, error: `Unable to inspect worktree path: ${file}` };
    }
  }
  return { ok: true, exists: true, error: null };
}

function isRelevantSourcePath(relativePath) {
  const file = safeRepoPath(relativePath);
  if (
    !file ||
    file.startsWith('scripts/harness/') ||
    file.startsWith('scripts/harness-v2/')
  ) {
    return false;
  }
  const segments = file.toLocaleLowerCase().split('/');
  if (segments.some((segment) => EXCLUDED_SEGMENTS.has(segment))) return false;
  const basename = segments[segments.length - 1];
  if (
    /(?:^|[._-])(?:fixture|mock|spec|test)s?(?:[._-]|$)/i.test(basename) ||
    /(?:^|\.)generated\./i.test(basename) ||
    /_test\.[^.]+$/i.test(basename) ||
    basename.endsWith('.d.ts') ||
    basename.endsWith('.snap') ||
    basename.endsWith('.map')
  ) {
    return false;
  }
  return SOURCE_EXTENSIONS.has(path.posix.extname(basename));
}

function changedContentLines(patch) {
  const lines = [];
  for (const raw of String(patch || '').split(/\r?\n/)) {
    if (!raw || raw.startsWith('+++') || raw.startsWith('---')) continue;
    if (raw[0] === '+' || raw[0] === '-') lines.push(raw.slice(1));
  }
  return lines;
}

function structuralSignalForLine(relativePath, line) {
  const text = line.trim();
  if (!text || /^(?:\/\/|\/\*|\*|#(?!\[)|--)/.test(text)) return null;
  const extension = path.posix.extname(relativePath).toLocaleLowerCase();
  if (extension === '.prisma') return 'prisma-schema-line';
  if (['.graphql', '.gql'].includes(extension) && /^(?:extend\s+)?(?:schema|type|input|interface|enum|union|scalar|directive)\b/.test(text)) {
    return 'graphql-schema-declaration';
  }
  if (extension === '.proto' && /^(?:service|rpc|message|enum|extend)\b/.test(text)) return 'protobuf-contract-declaration';
  if (extension === '.sql' && /^(?:CREATE|ALTER|DROP|RENAME)\s+(?:TABLE|VIEW|FUNCTION|PROCEDURE|TYPE|INDEX|SCHEMA)\b/i.test(text)) {
    return 'sql-ddl';
  }
  if (/\.module\.[cm]?[jt]sx?$/.test(relativePath) && !/^import\b/.test(text)) return 'module-wiring';

  const patterns = [
    ['module-export', /^(?:export\s+(?:default\s+)?(?:abstract\s+)?(?:async\s+)?(?:class|interface|enum|type|function|const|let|var|namespace|module)\b|export\s+(?:\*|\{)|module\.exports\b|exports\.[A-Za-z_$])/],
    ['type-declaration', /^(?:declare\s+)?(?:abstract\s+)?(?:class|interface|enum|type|namespace)\s+[A-Za-z_$][\w$]*/],
    ['function-declaration', /^(?:async\s+)?function\s+[A-Za-z_$][\w$]*\s*\(/],
    ['nest-route', /^@(?:Get|Post|Put|Patch|Delete|Options|Head|All|Controller|Resolver|Mutation|Query|Subscription)\s*\(/],
    ['dependency-wiring', /^@(?:Module|Inject|Injectable|Global)\b/],
    ['common-route', /^(?:app|router|server)\.(?:use|get|post|put|patch|delete|options|head|route)\s*\(/],
    ['python-public', /^(?:async\s+)?def\s+[^_\s][A-Za-z0-9_]*\s*\(|^class\s+[^_\s][A-Za-z0-9_]*\b/],
    ['python-route', /^@(?:app|router|blueprint)\.(?:route|get|post|put|patch|delete)\s*\(/],
    ['go-public', /^(?:func|type|var|const)\s+(?:\([^)]*\)\s*)?[A-Z][A-Za-z0-9_]*\b/],
    ['rust-public', /^pub(?:\([^)]*\))?\s+(?:async\s+)?(?:fn|struct|enum|trait|type|mod|use|const|static)\b/],
    ['http-contract', /^@(?:RequestMapping|GetMapping|PostMapping|PutMapping|PatchMapping|DeleteMapping|Path|GET|POST|PUT|PATCH|DELETE)\b/]
  ];
  for (const [name, pattern] of patterns) {
    if (pattern.test(text)) return name;
  }
  if (/\.(?:service|controller|repository|provider)\.[cm]?[jt]sx?$/.test(relativePath)) {
    const method = text.match(/^(?:(?:public|protected)\s+)?(?:async\s+)?([A-Za-z_$][\w$]*)\s*\([^;{}]*\)\s*(?::[^=]+)?\s*\{/);
    if (method && !new Set(['catch', 'for', 'if', 'switch', 'while', 'with']).has(method[1])) {
      return 'service-public-method';
    }
  }
  return null;
}

function structuralSignatures(relativePath, content) {
  const signatures = [];
  for (const line of String(content || '').split(/\r?\n/)) {
    const signal = structuralSignalForLine(relativePath, line);
    if (!signal) continue;
    let text = line.trim();
    if ([
      'module-export',
      'type-declaration',
      'function-declaration',
      'python-public',
      'go-public',
      'rust-public',
      'service-public-method'
    ].includes(signal)) {
      const brace = text.indexOf('{');
      if (brace >= 0) text = text.slice(0, brace).trim();
    }
    if (signal === 'module-export') {
      const variable = text.match(
        /^(export\s+(?:(?:declare|default)\s+)?(?:const|let|var)\s+[A-Za-z_$][\w$]*(?:\s*:[^=]+)?)\s*=/
      );
      if (variable) text = variable[1].trim();
    }
    signatures.push({ signal, text });
  }
  return signatures.sort((left, right) => (
    left.signal.localeCompare(right.signal) || left.text.localeCompare(right.text)
  ));
}

function classifyStagedChanges(target, changes) {
  const directSignals = {
    A: 'source-added',
    C: 'source-copied',
    D: 'source-deleted',
    R: 'source-renamed',
    T: 'source-type-changed'
  };
  const modified = changes.filter((change) => (
    change && change.status === 'M' &&
    [change.oldPath, change.newPath, change.path].filter(Boolean).some(isRelevantSourcePath)
  ));
  const loaded = readObjects(
    target,
    modified.flatMap((change) => [change.oldOid, change.newOid])
  );
  const results = [];

  for (const change of changes) {
    if (!change || !CHANGE_KINDS.has(change.status)) {
      results.push({
        change,
        ok: false,
        structural: true,
        signal: 'malformed-change',
        error: 'Unsupported staged change'
      });
      continue;
    }
    const paths = [change.oldPath, change.newPath, change.path].filter(Boolean);
    if (paths.length === 0 || paths.some((item) => !safeRepoPath(item))) {
      results.push({
        change,
        ok: false,
        structural: true,
        signal: 'unsafe-path',
        error: 'Unsafe staged path'
      });
      continue;
    }
    if (!paths.some(isRelevantSourcePath)) {
      results.push({ change, ok: true, structural: false, signal: null, error: null });
      continue;
    }
    if (directSignals[change.status]) {
      results.push({
        change,
        ok: true,
        structural: true,
        signal: directSignals[change.status],
        error: null
      });
      continue;
    }
    if (change.status !== 'M') {
      results.push({
        change,
        ok: false,
        structural: true,
        signal: 'unresolved-index-state',
        error: 'Unresolved staged state'
      });
      continue;
    }
    if (!loaded.ok) {
      results.push({
        change,
        ok: false,
        structural: true,
        signal: 'staged-blob-unavailable',
        error: loaded.error
      });
      continue;
    }
    const oldBlob = loaded.objects.get(change.oldOid);
    const newBlob = loaded.objects.get(change.newOid);
    const oldText = decodeUtf8(oldBlob, `Old blob for ${change.path}`);
    const newText = decodeUtf8(newBlob, `New blob for ${change.path}`);
    if (!oldText.ok || !newText.ok) {
      results.push({
        change,
        ok: false,
        structural: true,
        signal: 'non-utf8-source',
        error: 'Modified source could not be compared as UTF-8'
      });
      continue;
    }
    const before = structuralSignatures(change.path, oldText.text);
    const after = structuralSignatures(change.path, newText.text);
    const beforeKey = JSON.stringify(before);
    const afterKey = JSON.stringify(after);
    if (beforeKey === afterKey) {
      results.push({ change, ok: true, structural: false, signal: null, error: null });
      continue;
    }
    const beforeSet = new Set(before.map((item) => `${item.signal}\0${item.text}`));
    const changed = after.find((item) => !beforeSet.has(`${item.signal}\0${item.text}`));
    results.push({
      change,
      ok: true,
      structural: true,
      signal: changed ? changed.signal : (before[0] ? before[0].signal : 'public-structure-changed'),
      error: null
    });
  }
  return { ok: loaded.ok, results, error: loaded.error };
}

function stagedChangeHasStructuralSignal(target, change) {
  if (!change || !CHANGE_KINDS.has(change.status)) {
    return { ok: false, structural: true, signal: 'malformed-change', error: 'Unsupported staged change' };
  }
  const paths = [change.oldPath, change.newPath, change.path].filter(Boolean);
  if (paths.length === 0 || paths.some((item) => !safeRepoPath(item))) {
    return { ok: false, structural: true, signal: 'unsafe-path', error: 'Unsafe staged path' };
  }
  if (!paths.some(isRelevantSourcePath)) {
    return { ok: true, structural: false, signal: null, error: null };
  }
  const direct = {
    A: 'source-added',
    C: 'source-copied',
    D: 'source-deleted',
    R: 'source-renamed',
    T: 'source-type-changed'
  };
  if (direct[change.status]) {
    return { ok: true, structural: true, signal: direct[change.status], error: null };
  }
  if (change.status !== 'M') {
    return { ok: false, structural: true, signal: 'unresolved-index-state', error: 'Unresolved staged state' };
  }
  const file = safeRepoPath(change.path || change.newPath);
  const diff = git(target, [
    'diff', '--cached', '--unified=0', '--no-ext-diff', '--no-textconv', '--', file
  ], { maxBuffer: 32 * 1024 * 1024 });
  if (!diff.ok || !diff.stdout) {
    return {
      ok: false,
      structural: true,
      signal: 'staged-diff-unavailable',
      error: diff.error || `Empty staged diff for ${file}`
    };
  }
  for (const line of changedContentLines(diff.stdout)) {
    const signal = structuralSignalForLine(file, line);
    if (signal) return { ok: true, structural: true, signal, error: null };
  }
  return { ok: true, structural: false, signal: null, error: null };
}

module.exports = {
  classifyStagedChanges,
  git,
  indexPathExists,
  isRelevantSourcePath,
  parseRawDiffZ,
  parseNameStatusZ,
  readIndexFile,
  safeRepoPath,
  stagedRawChanges,
  stagedChangeHasStructuralSignal,
  stagedChanges,
  unstagedChanges,
  untrackedPaths,
  worktreePathExists
};
