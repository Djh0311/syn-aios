'use strict';

// AH-050-12: static, bounded, read-only project inspection.  This module must
// never execute an inspected project's scripts, access the network, or read a
// user's home/global instruction files.

const fs = require('node:fs');
const path = require('node:path');

const { loadCoreContext } = require('./context-contract');
const { safeOutputRepoPath, sanitizeOutputText } = require('./output-safety');

const MAX_JSON_BYTES = 64 * 1024;
const MAX_ROOT_ENTRIES = 256;
const MAX_SCRIPT_NAMES = 64;
const MAX_DEPENDENCY_NAMES = 128;
const MAX_DEFAULT_CONTEXT_COMPONENTS = 32;
const MAX_RISK_CANDIDATES = 128;
const MAX_RISK_ADVISORIES = 24;
const MAX_EXTERNAL_LABEL_INPUT = 512;
const MAX_GIT_METADATA_BYTES = 1024;
const FACT_STATUSES = new Set(['confirmed', 'inferred', 'unknown']);
const ACCEPTANCE_SCRIPT_NAMES = Object.freeze([
  'acceptance',
  'e2e',
  'test:e2e',
  'test:acceptance',
  'smoke',
  'test:smoke',
  'verify:real',
]);

const DIRECTORY_MARKERS = [
  'src',
  'app',
  'apps',
  'packages',
  'web',
  'frontend',
  'client',
  'backend',
  'server',
  'api',
  'database',
  'db',
  'migrations',
  'prisma',
  'test',
  'tests',
  '__tests__',
  'docs',
  'scripts',
  'tools',
];

const FILE_MARKERS = [
  'package.json',
  'pnpm-workspace.yaml',
  'turbo.json',
  'lerna.json',
  'pyproject.toml',
  'requirements.txt',
  'setup.py',
  'Cargo.toml',
  'go.mod',
  'pom.xml',
  'build.gradle',
  'build.gradle.kts',
  'settings.gradle',
  'settings.gradle.kts',
  'Dockerfile',
  'docker-compose.yml',
  'docker-compose.yaml',
  'Makefile',
  '.nvmrc',
  'tsconfig.json',
  'vite.config.js',
  'vite.config.ts',
  'next.config.js',
  'next.config.mjs',
  '.gitlab-ci.yml',
  'Jenkinsfile',
  'azure-pipelines.yml',
  'pytest.ini',
  'tox.ini',
  'harness.config.json',
  '.harness/manifest.json',
  'docs/harness/AUTHORITY.md',
  'docs/harness/CURRENT.md',
  'scripts/harness-v2/project-context.js',
  'scripts/harness-v2/adapt.js',
  'prisma/schema.prisma',
];

const RISK_PATTERNS = [
  { category: 'database', expression: /(?:database|\bdb\b|数据库)/iu },
  { category: 'migration', expression: /(?:migration|migrate|迁移)/iu },
  { category: 'authentication', expression: /(?:authentication|auth|认证|鉴权)/iu },
  { category: 'permission', expression: /(?:permission|authorization|权限)/iu },
  { category: 'security', expression: /(?:security|安全)/iu },
  { category: 'production', expression: /(?:production|\bprod\b|生产)/iu },
  { category: 'deploy', expression: /(?:deploy(?:ment)?|部署)/iu },
  { category: 'payment', expression: /(?:payment|pay|支付|付款)/iu },
];

function safeRelativePath(value) {
  const candidate = String(value || '').split(path.sep).join('/').replace(/^\.\//, '');
  if (
    !candidate ||
    candidate.startsWith('/') ||
    candidate.includes('\\') ||
    candidate.includes('\0') ||
    candidate.split('/').includes('..') ||
    path.posix.normalize(candidate) !== candidate
  ) return null;
  return candidate;
}

function isInside(root, candidate) {
  return candidate === root || candidate.startsWith(`${root}${path.sep}`);
}

function sanitizeExternalLabel(value, maxChars = 120) {
  if (typeof value !== 'string' && typeof value !== 'number') return null;
  const raw = String(value);
  if (!raw || raw.length > MAX_EXTERNAL_LABEL_INPUT) return null;
  const text = sanitizeOutputText(raw, maxChars);
  if (!text || /<(?:[^>]*redacted|unsafe-path-redacted)>/i.test(text)) return null;
  return text;
}

function sanitizeExternalIdentifier(value, maxChars = 80) {
  const label = sanitizeExternalLabel(value, maxChars);
  if (!label || !/^[A-Za-z0-9][A-Za-z0-9._:-]{0,79}$/.test(label)) return null;
  return label;
}

function sanitizeGitBranch(value) {
  const label = sanitizeExternalLabel(value, 120);
  if (!label || !/^[A-Za-z0-9][A-Za-z0-9._/-]{0,119}$/.test(label)) return null;
  if (label.includes('..') || label.includes('//') || label.endsWith('.') || label.endsWith('.lock')) {
    return null;
  }
  return label;
}

function scanLabel(value) {
  return String(value || '').slice(0, MAX_EXTERNAL_LABEL_INPUT).toLowerCase();
}

function safeSource(relativePath) {
  const normalized = safeRelativePath(relativePath);
  if (!normalized) return null;
  const label = sanitizeExternalLabel(normalized, 160);
  if (!label) return null;
  const safe = safeOutputRepoPath(label, 160);
  return safe === '<unsafe-path-redacted>' ? null : safe;
}

function sanitizeName(value, fallback = 'target') {
  return sanitizeExternalLabel(value, 120) || fallback;
}

function fact(status, value, sources = [], note) {
  if (!FACT_STATUSES.has(status)) throw new Error(`invalid inspection status: ${status}`);
  const result = {
    status,
    value,
    sources: [...new Set(sources.map(safeSource).filter(Boolean))],
  };
  if (note) result.note = sanitizeOutputText(note, 240);
  return result;
}

function resolveTarget(target) {
  if (typeof target !== 'string' || !target.trim()) {
    throw new Error('--target requires a directory');
  }
  const requested = path.resolve(target);
  let root;
  try {
    const requestedStat = fs.lstatSync(requested);
    if (requestedStat.isSymbolicLink()) throw new Error('TARGET_SYMLINK');
    root = fs.realpathSync(requested);
  } catch {
    throw new Error('target directory is unavailable or symbolic link');
  }
  let stat;
  try {
    stat = fs.statSync(root);
  } catch {
    throw new Error('target directory is unavailable');
  }
  if (!stat.isDirectory()) throw new Error('target must be a directory');
  return root;
}

function inspectPath(root, relativePath) {
  const safePath = safeRelativePath(relativePath);
  if (!safePath) return { ok: false, code: 'UNSAFE_PATH' };
  const candidate = path.resolve(root, safePath);
  if (!isInside(root, candidate)) return { ok: false, code: 'OUTSIDE_TARGET' };
  let cursor = root;
  try {
    for (const segment of safePath.split('/')) {
      cursor = path.join(cursor, segment);
      const stat = fs.lstatSync(cursor);
      if (stat.isSymbolicLink()) return { ok: false, code: 'SYMLINK_FORBIDDEN' };
      if (cursor !== candidate && !stat.isDirectory()) return { ok: false, code: 'NOT_A_DIRECTORY' };
      if (cursor === candidate) return { ok: true, path: candidate, stat };
    }
    return { ok: false, code: 'UNAVAILABLE' };
  } catch (error) {
    if (error && error.code === 'ENOENT') return { ok: false, code: 'MISSING' };
    return { ok: false, code: 'UNAVAILABLE' };
  }
}

function readBoundedText(root, relativePath, maxBytes = MAX_JSON_BYTES) {
  const inspected = inspectPath(root, relativePath);
  if (!inspected.ok) return inspected;
  if (!inspected.stat.isFile()) return { ok: false, code: 'NOT_A_FILE' };
  if (inspected.stat.size > maxBytes) return { ok: false, code: 'TOO_LARGE' };
  try {
    const text = fs.readFileSync(inspected.path, 'utf8');
    if (text.includes('\0')) return { ok: false, code: 'INVALID_TEXT' };
    return { ok: true, text, bytes: Buffer.byteLength(text, 'utf8') };
  } catch {
    return { ok: false, code: 'READ_FAILED' };
  }
}

function readBoundedJson(root, relativePath, maxBytes = MAX_JSON_BYTES) {
  const text = readBoundedText(root, relativePath, maxBytes);
  if (!text.ok) return text;
  try {
    const value = JSON.parse(text.text);
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return { ok: false, code: 'NOT_AN_OBJECT' };
    }
    return { ok: true, value, bytes: text.bytes };
  } catch {
    return { ok: false, code: 'INVALID_JSON' };
  }
}

function hasDirectory(root, relativePath) {
  const inspected = inspectPath(root, relativePath);
  return inspected.ok && inspected.stat.isDirectory();
}

function hasFile(root, relativePath) {
  const inspected = inspectPath(root, relativePath);
  return inspected.ok && inspected.stat.isFile();
}

function rootEntries(root) {
  try {
    const entries = fs.readdirSync(root, { withFileTypes: true });
    const names = entries.slice(0, MAX_ROOT_ENTRIES).map((entry) => entry.name);
    return {
      ok: true,
      names,
      truncated: entries.length > MAX_ROOT_ENTRIES,
    };
  } catch {
    return { ok: false, names: [], truncated: false };
  }
}

function recognizedRootFiles(root) {
  return FILE_MARKERS.filter((relativePath) => hasFile(root, relativePath));
}

function recognizedDirectories(root) {
  return DIRECTORY_MARKERS.filter((relativePath) => hasDirectory(root, relativePath));
}

function boundedStringKeys(value, maximum) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return { names: [], totalCount: 0, truncated: false };
  }
  const names = Object.keys(value)
    .filter((name) => typeof value[name] === 'string')
    .sort();
  return {
    names: names.slice(0, maximum),
    totalCount: names.length,
    truncated: names.length > maximum,
  };
}

function inspectControlledScriptEntries(value) {
  const result = {
    test: false,
    build: false,
    lint: false,
    acceptance: [],
  };
  if (!value || typeof value !== 'object' || Array.isArray(value)) return result;
  const acceptance = new Set();
  for (const rawName of Object.keys(value)) {
    if (typeof value[rawName] !== 'string') continue;
    const name = scanLabel(rawName);
    if (name === 'test' || name.startsWith('test:')) result.test = true;
    if (name === 'build' || name.startsWith('build:')) result.build = true;
    if (name === 'lint' || name.startsWith('lint:')) result.lint = true;
    if (ACCEPTANCE_SCRIPT_NAMES.includes(name)) acceptance.add(name);
  }
  result.acceptance = ACCEPTANCE_SCRIPT_NAMES.filter((name) => acceptance.has(name));
  return result;
}

function packageSignals(root) {
  const packageJson = readBoundedJson(root, 'package.json');
  if (!packageJson.ok) {
    return {
      result: packageJson,
      scripts: [],
      dependencies: [],
      devDependencies: [],
      scriptCount: 0,
      dependencyCount: 0,
      scriptsTruncated: false,
      dependenciesTruncated: false,
      controlledEntries: { test: false, build: false, lint: false, acceptance: [] },
    };
  }
  const scriptKeys = boundedStringKeys(packageJson.value.scripts, MAX_SCRIPT_NAMES);
  const dependencyKeys = boundedStringKeys(packageJson.value.dependencies, MAX_DEPENDENCY_NAMES);
  const devDependencyKeys = boundedStringKeys(
    packageJson.value.devDependencies,
    MAX_DEPENDENCY_NAMES,
  );
  return {
    result: packageJson,
    scripts: scriptKeys.names,
    dependencies: dependencyKeys.names,
    devDependencies: devDependencyKeys.names,
    scriptCount: scriptKeys.totalCount,
    dependencyCount: dependencyKeys.totalCount + devDependencyKeys.totalCount,
    scriptsTruncated: scriptKeys.truncated,
    dependenciesTruncated: dependencyKeys.truncated || devDependencyKeys.truncated,
    controlledEntries: inspectControlledScriptEntries(packageJson.value.scripts),
  };
}

function inspectProject(root, packageInfo, rootFiles, directories) {
  const types = [];
  const sources = [];
  if (packageInfo.result.ok) {
    types.push('node');
    sources.push('package.json');
  } else if (hasFile(root, 'package.json')) {
    types.push('node');
    sources.push('package.json');
  }
  const fileTypes = [
    ['python', ['pyproject.toml', 'requirements.txt', 'setup.py']],
    ['rust', ['Cargo.toml']],
    ['go', ['go.mod']],
    ['jvm', ['pom.xml', 'build.gradle', 'build.gradle.kts', 'settings.gradle', 'settings.gradle.kts']],
  ];
  for (const [type, markers] of fileTypes) {
    const found = markers.filter((marker) => rootFiles.includes(marker));
    if (found.length) {
      types.push(type);
      sources.push(...found);
    }
  }
  const entries = rootEntries(root);
  const solution = entries.names.find((name) => name.endsWith('.sln'));
  if (solution) {
    types.push('dotnet');
    sources.push('root-solution-marker');
  }
  if (['pnpm-workspace.yaml', 'turbo.json', 'lerna.json'].some((name) => rootFiles.includes(name))) {
    types.push('monorepo');
    sources.push(...rootFiles.filter((name) => ['pnpm-workspace.yaml', 'turbo.json', 'lerna.json'].includes(name)));
  }
  const uniqueTypes = [...new Set(types)].sort();
  const packageFailed = hasFile(root, 'package.json') && !packageInfo.result.ok;
  return {
    type: uniqueTypes.length
      ? fact(packageFailed ? 'inferred' : 'confirmed', { types: uniqueTypes }, sources)
      : fact('unknown', { types: [] }, [], 'No supported project marker was found in the bounded root scan.'),
    directories: directories.length
      ? fact('confirmed', { present: directories }, directories)
      : fact('unknown', { present: [] }, [], 'No known project directory was found in the bounded marker list.'),
  };
}

function validComponentId(value) {
  return typeof value === 'string' && /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value);
}

function safeVersion(value) {
  const text = sanitizeExternalLabel(value, 80);
  return text && /^[A-Za-z0-9][A-Za-z0-9._+-]{0,79}$/.test(text) ? text : null;
}

function inspectHarness(root) {
  const manifest = readBoundedJson(root, '.harness/manifest.json');
  const config = readBoundedJson(root, 'harness.config.json');
  const manifestFile = hasFile(root, '.harness/manifest.json');
  const configFile = hasFile(root, 'harness.config.json');
  const authority = hasFile(root, 'docs/harness/AUTHORITY.md');
  const current = hasFile(root, 'docs/harness/CURRENT.md');
  const entryPaths = [
    'scripts/harness-v2/project-context.js',
    'scripts/harness-v2/adapt.js',
  ].filter((relativePath) => hasFile(root, relativePath));
  const carriers = [];
  if (manifest.ok) carriers.push('.harness/manifest.json');
  if (config.ok) carriers.push('harness.config.json');
  if (manifestFile && !manifest.ok) carriers.push('.harness/manifest.json');
  if (configFile && !config.ok) carriers.push('harness.config.json');
  if (authority) carriers.push('docs/harness/AUTHORITY.md');
  if (current) carriers.push('docs/harness/CURRENT.md');
  if (entryPaths.length) carriers.push(...entryPaths);

  let existing;
  if (manifest.ok) {
    existing = fact('confirmed', { detected: true, carriers }, carriers);
  } else if (carriers.length) {
    existing = fact('inferred', { detected: true, carriers }, carriers,
      'Harness markers were found without a readable manifest.');
  } else {
    existing = fact('unknown', { detected: false, carriers: [] }, [],
      'No bounded Harness marker was found.');
  }

  const versionValue = {};
  const versionSources = [];
  if (manifest.ok) {
    if (Number.isInteger(manifest.value.schemaVersion)) {
      versionValue.manifestSchemaVersion = manifest.value.schemaVersion;
      versionSources.push('.harness/manifest.json');
    }
    const selection = manifest.value.selection;
    if (selection && typeof selection === 'object' && !Array.isArray(selection)) {
      if (validComponentId(selection.pack)) versionValue.pack = selection.pack;
      const packVersion = safeVersion(selection.packVersion);
      if (packVersion) versionValue.packVersion = packVersion;
      if (versionValue.pack || versionValue.packVersion) versionSources.push('.harness/manifest.json');
    }
  }
  if (config.ok && Number.isInteger(config.value.schemaVersion)) {
    versionValue.configSchemaVersion = config.value.schemaVersion;
    versionSources.push('harness.config.json');
  }
  const version = Object.keys(versionValue).length
    ? fact('confirmed', versionValue, versionSources)
    : carriers.length
      ? fact('inferred', {}, carriers, 'Harness markers do not expose a bounded version value.')
      : fact('unknown', {}, []);
  const entry = entryPaths.length
    ? fact('confirmed', { paths: entryPaths }, entryPaths)
    : carriers.length
      ? fact('inferred', { paths: [] }, carriers, 'No known Harness command entry was found.')
      : fact('unknown', { paths: [] }, []);
  return { existing, version, entry, manifest, config };
}

function unknownGitFacts(note) {
  return {
    repository: fact('unknown', { present: false, linkedWorktree: false }, [], note),
    worktrees: fact('unknown', { linkedWorktree: false, count: null, truncated: false }, []),
    branch: fact('unknown', { name: null }, []),
    head: fact('unknown', { value: null }, []),
    workspace: fact('unknown', { state: null, counts: null }, [],
      'Workspace status is intentionally not derived by static adapt inspect.'),
  };
}

function validGitSha(value) {
  return /^[a-f0-9]{40,64}$/i.test(value);
}

function parseStaticGitHead(text) {
  const value = String(text || '').trim();
  if (validGitSha(value)) return { kind: 'detached', sha: value.toLowerCase() };
  const match = /^ref:\s+(refs\/heads\/[^\s]+)$/.exec(value);
  if (!match) return { kind: 'unknown' };
  const ref = match[1];
  const branch = sanitizeGitBranch(ref.slice('refs/heads/'.length));
  if (!branch || !safeRelativePath(ref)) return { kind: 'unknown' };
  return { kind: 'branch', branch, ref };
}

function countStaticWorktrees(root) {
  const carrier = inspectPath(root, '.git/worktrees');
  if (!carrier.ok && carrier.code === 'MISSING') {
    return fact('confirmed', { linkedWorktree: false, count: 1, truncated: false }, ['.git']);
  }
  if (!carrier.ok || !carrier.stat.isDirectory()) {
    return fact('unknown', { linkedWorktree: false, count: null, truncated: false }, []);
  }
  let handle;
  let count = 0;
  let truncated = false;
  try {
    handle = fs.opendirSync(carrier.path);
    for (;;) {
      const entry = handle.readSync();
      if (!entry) break;
      if (!entry.isDirectory()) continue;
      if (count >= MAX_ROOT_ENTRIES) {
        truncated = true;
        break;
      }
      count += 1;
    }
    return fact('confirmed', {
      linkedWorktree: false,
      count: 1 + count,
      truncated,
    }, ['.git']);
  } catch {
    return fact('unknown', { linkedWorktree: false, count: null, truncated: false }, []);
  } finally {
    if (handle) handle.closeSync();
  }
}

function inspectStaticGitDirectory(root) {
  const headDocument = readBoundedText(root, '.git/HEAD', MAX_GIT_METADATA_BYTES);
  const parsed = headDocument.ok ? parseStaticGitHead(headDocument.text) : { kind: 'unknown' };
  const headUsable = parsed.kind === 'detached' || parsed.kind === 'branch';
  const repository = fact(headUsable ? 'confirmed' : 'inferred', {
    present: true,
    linkedWorktree: false,
  }, ['.git'], headUsable ? undefined : 'Target .git directory exists but HEAD is missing, unsafe, or malformed.');
  const countedWorktrees = countStaticWorktrees(root);
  const worktrees = headUsable || countedWorktrees.status !== 'confirmed'
    ? countedWorktrees
    : fact('inferred', countedWorktrees.value, countedWorktrees.sources,
      'Worktree count is not confirmed until target-local HEAD is safely parsed.');
  const workspace = fact('unknown', { state: null, counts: null }, [],
    'Workspace status is intentionally not derived by static adapt inspect.');
  if (parsed.kind === 'detached') {
    return {
      repository,
      worktrees,
      branch: fact('confirmed', { name: 'DETACHED' }, ['.git/HEAD']),
      head: fact('confirmed', { value: parsed.sha }, ['.git/HEAD']),
      workspace,
    };
  }
  if (parsed.kind !== 'branch') {
    return {
      repository,
      worktrees,
      branch: fact('unknown', { name: null }, []),
      head: fact('unknown', { value: null }, []),
      workspace,
    };
  }
  const looseRef = readBoundedText(root, `.git/${parsed.ref}`, MAX_GIT_METADATA_BYTES);
  const looseSha = looseRef.ok && validGitSha(looseRef.text.trim())
    ? looseRef.text.trim().toLowerCase()
    : null;
  return {
    repository,
    worktrees,
    branch: fact('confirmed', { name: parsed.branch }, ['.git/HEAD']),
    head: looseSha
      ? fact('confirmed', { value: looseSha }, ['.git/HEAD'])
      : fact('unknown', { value: null }, ['.git/HEAD'],
        'No safe bounded loose ref was available; packed refs are not read.'),
    workspace,
  };
}

function inspectGit(root) {
  const carrier = inspectPath(root, '.git');
  if (!carrier.ok) return unknownGitFacts('No safe target-local .git carrier was found.');
  if (carrier.stat.isDirectory()) return inspectStaticGitDirectory(root);
  if (!carrier.stat.isFile()) return unknownGitFacts('Target .git carrier is not a regular directory or pointer file.');
  const pointer = readBoundedText(root, '.git', MAX_GIT_METADATA_BYTES);
  if (!pointer.ok || !/^gitdir:\s+\S[^\r\n]*\s*$/m.test(pointer.text)) {
    return unknownGitFacts('Target .git pointer file is malformed or unavailable.');
  }
  return {
    repository: fact('confirmed', { present: true, linkedWorktree: true }, ['.git']),
    worktrees: fact('confirmed', { linkedWorktree: true, count: null, truncated: false }, ['.git']),
    branch: fact('unknown', { name: null }, []),
    head: fact('unknown', { value: null }, []),
    workspace: fact('unknown', { state: null, counts: null }, [],
      'Linked worktree pointer is not followed by static adapt inspect.'),
  };
}

function hasAny(value, candidates) {
  const normalized = new Set(value.map(scanLabel));
  return candidates.some((candidate) => normalized.has(candidate));
}

function inspectEntries(root, packageInfo) {
  if (packageInfo.result.ok) {
    const controlled = packageInfo.controlledEntries;
    const acceptanceNames = controlled.acceptance;
    return {
      test: fact('confirmed', { available: controlled.test }, ['package.json']),
      build: fact('confirmed', { available: controlled.build }, ['package.json']),
      lint: fact('confirmed', { available: controlled.lint }, ['package.json']),
      acceptance: acceptanceNames.length
        ? fact('confirmed', {
          available: true,
          names: acceptanceNames,
          inspectedScriptCount: packageInfo.scripts.length,
          totalScriptCount: packageInfo.scriptCount,
          truncated: packageInfo.scriptsTruncated,
        }, ['package.json'])
        : fact('unknown', {
          available: false,
          names: [],
          inspectedScriptCount: packageInfo.scripts.length,
          totalScriptCount: packageInfo.scriptCount,
          truncated: packageInfo.scriptsTruncated,
        }, [], 'No bounded explicit real-acceptance script name was found.'),
    };
  }
  const pythonTest = hasFile(root, 'pytest.ini') || hasFile(root, 'tox.ini');
  const markerSources = [
    ...(hasFile(root, 'pytest.ini') ? ['pytest.ini'] : []),
    ...(hasFile(root, 'tox.ini') ? ['tox.ini'] : []),
  ];
  return {
    test: pythonTest
      ? fact('inferred', { available: true }, markerSources, 'Static test-runner marker only; no command was executed.')
      : fact('unknown', { available: null }, []),
    build: fact('unknown', { available: null }, []),
    lint: fact('unknown', { available: null }, []),
    acceptance: fact('unknown', { available: false, names: [] }, []),
  };
}

function capabilityFact(found, sources, inferred = false) {
  if (found) return fact(inferred ? 'inferred' : 'confirmed', { present: true }, sources);
  return fact('unknown', { present: false }, []);
}

function inspectCapabilities(root, packageInfo, directories, rootFiles) {
  const packages = [...packageInfo.dependencies, ...packageInfo.devDependencies].map(scanLabel);
  const scriptNames = packageInfo.scripts.map(scanLabel);
  const frontendPackages = ['react', 'next', 'vue', '@angular/core', 'svelte', 'nuxt', 'vite'];
  const backendPackages = ['express', '@nestjs/core', 'fastify', 'koa', 'hapi'];
  const frontendDirectories = ['frontend', 'client', 'web', 'app'];
  const backendDirectories = ['backend', 'server', 'api'];
  const databaseDirectories = ['database', 'db', 'migrations', 'prisma'];
  const frontendSources = [
    ...directories.filter((item) => frontendDirectories.includes(item)),
    ...(hasAny(packages, frontendPackages) ? ['package.json'] : []),
    ...rootFiles.filter((item) => ['vite.config.js', 'vite.config.ts', 'next.config.js', 'next.config.mjs'].includes(item)),
  ];
  const backendSources = [
    ...directories.filter((item) => backendDirectories.includes(item)),
    ...(hasAny(packages, backendPackages) ? ['package.json'] : []),
  ];
  const databaseSources = [
    ...directories.filter((item) => databaseDirectories.includes(item)),
    ...rootFiles.filter((item) => item === 'prisma/schema.prisma'),
  ];
  const ciSources = [
    ...(hasDirectory(root, '.github/workflows') ? ['.github/workflows'] : []),
    ...rootFiles.filter((item) => ['.gitlab-ci.yml', 'Jenkinsfile', 'azure-pipelines.yml'].includes(item)),
  ];
  const toolSources = rootFiles.filter((item) => [
    'Dockerfile', 'docker-compose.yml', 'docker-compose.yaml', 'Makefile', '.nvmrc', 'tsconfig.json',
  ].includes(item));
  return {
    frontend: capabilityFact(frontendSources.length > 0, frontendSources),
    backend: capabilityFact(backendSources.length > 0, backendSources),
    database: capabilityFact(databaseSources.length > 0, databaseSources),
    ci: capabilityFact(ciSources.length > 0, ciSources),
    tools: capabilityFact(toolSources.length > 0, toolSources),
    scriptCatalog: packageInfo.result.ok
      ? fact('confirmed', {
        available: packageInfo.scriptCount > 0,
        inspectedCount: scriptNames.length,
        totalCount: packageInfo.scriptCount,
        truncated: packageInfo.scriptsTruncated,
        recognized: packageInfo.controlledEntries.acceptance,
      }, ['package.json'])
      : fact('unknown', {
        available: false,
        inspectedCount: 0,
        totalCount: 0,
        truncated: false,
        recognized: [],
      }, []),
  };
}

function inspectBusinessRisks(root, packageInfo, rootFiles, directories) {
  const candidates = [
    ...rootFiles.map((source) => ({ source, label: source, signal: source })),
    ...directories.map((source) => ({ source, label: source, signal: source })),
    ...packageInfo.scripts.map((name) => ({
      source: 'package.json',
      label: name,
      signal: 'script-name',
    })),
  ].slice(0, MAX_RISK_CANDIDATES);
  const advisories = [];
  candidateLoop: for (const candidate of candidates) {
    const label = scanLabel(candidate.label);
    for (const pattern of RISK_PATTERNS) {
      if (pattern.expression.test(label)) {
        const key = `${pattern.category}:${candidate.source}:${candidate.signal}`;
        if (!advisories.some((entry) => entry.key === key)) {
          advisories.push({
            key,
            category: pattern.category,
            source: safeSource(candidate.source),
            signal: candidate.signal,
          });
          if (advisories.length >= MAX_RISK_ADVISORIES) break candidateLoop;
        }
      }
    }
  }
  const value = {
    advisories: advisories.map(({ category, source, signal }) => ({ category, source, signal })),
    effect: 'advisory-only',
    inspectedCandidateCount: candidates.length,
    totalCandidateCount: rootFiles.length + directories.length + packageInfo.scriptCount,
    truncated: rootFiles.length + directories.length + packageInfo.scriptCount > candidates.length
      || packageInfo.scriptsTruncated,
  };
  return advisories.length
    ? fact('inferred', value, advisories.map((entry) => entry.source),
      'Static name matches are not authorization, execution, or a confirmed business-risk finding.')
    : fact('unknown', value, [], 'No bounded Chinese or English risk-name match was found.');
}

function inspectAuthority(root, harness) {
  const authorityFile = hasFile(root, 'docs/harness/AUTHORITY.md');
  const currentFile = hasFile(root, 'docs/harness/CURRENT.md');
  if (!authorityFile && !currentFile) {
    return fact('unknown', { activeAuthority: null, mode: null, workState: null }, []);
  }
  let core;
  try {
    core = loadCoreContext(root);
  } catch {
    return fact('inferred', {
      activeAuthority: null,
      activeId: null,
      authorityStatus: null,
      mode: null,
      workState: null,
    }, [
      ...(authorityFile ? ['docs/harness/AUTHORITY.md'] : []),
      ...(currentFile ? ['docs/harness/CURRENT.md'] : []),
    ], 'Authority carriers could not be parsed by the bounded read-only context loader.');
  }
  const value = {
    activeAuthority: core.authority ? safeSource(core.authority) : null,
    activeId: core.activeAuthority && core.activeAuthority.id
      ? sanitizeName(core.activeAuthority.id, 'UNKNOWN')
      : null,
    authorityStatus: core.activeAuthority && core.activeAuthority.status
      ? sanitizeName(core.activeAuthority.status, 'UNKNOWN')
      : null,
    mode: core.current && core.current.mode ? sanitizeName(core.current.mode, 'UNKNOWN') : null,
    workState: core.current && core.current.workState
      ? sanitizeName(core.current.workState, 'UNKNOWN')
      : null,
  };
  const sources = [
    ...(authorityFile ? ['docs/harness/AUTHORITY.md'] : []),
    ...(currentFile ? ['docs/harness/CURRENT.md'] : []),
  ];
  return core.coreStatus === 'OK'
    ? fact('confirmed', value, sources)
    : fact('inferred', value, sources, 'Authority carriers were found but their bounded context is incomplete.');
}

function inspectDefaultContext(harness) {
  if (!harness.config.ok) {
    return harness.existing.status === 'unknown'
      ? fact('unknown', { configured: null }, [])
      : fact('inferred', { configured: null }, harness.existing.sources,
        'No readable Harness configuration exposes a default-context setting.');
  }
  const components = harness.config.value.components;
  if (!components || typeof components !== 'object' || Array.isArray(components)) {
    return fact('inferred', { configured: null }, ['harness.config.json'],
      'Harness configuration has no readable component section.');
  }
  const componentNames = Object.keys(components).sort();
  const defaults = [];
  const seen = new Set();
  for (const component of componentNames.slice(0, MAX_DEFAULT_CONTEXT_COMPONENTS)) {
    const config = components[component];
    if (config && typeof config === 'object' && !Array.isArray(config)
      && typeof config.defaultContext === 'boolean') {
      const safeComponent = sanitizeExternalIdentifier(component);
      if (safeComponent && !seen.has(safeComponent)) {
        seen.add(safeComponent);
        defaults.push({ component: safeComponent, defaultContext: config.defaultContext });
      }
    }
  }
  const value = {
    configured: defaults,
    inspectedComponentCount: Math.min(componentNames.length, MAX_DEFAULT_CONTEXT_COMPONENTS),
    totalComponentCount: componentNames.length,
    truncated: componentNames.length > MAX_DEFAULT_CONTEXT_COMPONENTS,
  };
  return defaults.length
    ? fact('confirmed', value, ['harness.config.json'])
    : fact('inferred', value, ['harness.config.json'],
      'No component declares a static default-context flag.');
}

function inspectSuggestions(harness) {
  let pack;
  if (harness.manifest.ok) {
    const selection = harness.manifest.value.selection;
    if (selection && validComponentId(selection.pack)) {
      pack = fact('confirmed', { id: selection.pack }, ['.harness/manifest.json']);
    }
  }
  if (!pack && harness.config.ok) {
    const project = harness.config.value.project;
    if (project && validComponentId(project.pack)) {
      pack = fact('confirmed', { id: project.pack }, ['harness.config.json']);
    }
  }
  if (!pack) {
    pack = fact('inferred', { id: 'generic' }, [],
      'Generic is a review suggestion, not an installation action.');
  }
  const migrationScope = harness.existing.status === 'confirmed'
    ? fact('inferred', { action: 'review-existing-installation' }, harness.existing.sources,
      'Inspect the existing managed files before any explicit upgrade.')
    : fact('inferred', { action: 'review-static-findings-before-install' }, [],
      'No installation is performed by adapt inspect.');
  return { projectPack: pack, migrationScope };
}

function buildAdaptReport(target) {
  const root = resolveTarget(target);
  const packageInfo = packageSignals(root);
  const rootFiles = recognizedRootFiles(root);
  const directories = recognizedDirectories(root);
  const harness = inspectHarness(root);
  const authority = inspectAuthority(root, harness);
  const report = {
    schemaVersion: 1,
    command: 'adapt inspect',
    readOnly: true,
    target: fact('confirmed', { name: sanitizeName(path.basename(root)) }, []),
    project: inspectProject(root, packageInfo, rootFiles, directories),
    harness: {
      existing: harness.existing,
      version: harness.version,
      entry: harness.entry,
    },
    git: inspectGit(root),
    authority,
    defaultContext: inspectDefaultContext(harness),
    entries: inspectEntries(root, packageInfo),
    capabilities: inspectCapabilities(root, packageInfo, directories, rootFiles),
    businessRisks: inspectBusinessRisks(root, packageInfo, rootFiles, directories),
    suggested: inspectSuggestions(harness),
    globalInstructions: fact('unknown', {
      inspected: false,
      reason: 'Global instructions are outside the explicit target and are not read by adapt inspect.',
    }, []),
    safety: fact('confirmed', {
      projectScripts: 'not-executed',
      network: 'not-used',
      services: 'not-started',
      writes: 'not-performed',
      permissionChanges: 'not-performed',
    }, []),
  };
  return report;
}

function factSummary(item) {
  if (!item || !item.value) return 'unknown';
  if (Array.isArray(item.value.types)) return item.value.types.length ? item.value.types.join(', ') : 'none';
  if (typeof item.value.detected === 'boolean') return item.value.detected ? 'detected' : 'not detected';
  if (typeof item.value.repository === 'boolean') return item.value.repository ? 'repository' : 'unavailable';
  return item.status;
}

function formatAdaptReport(report, options = {}) {
  if (options.json) return `${JSON.stringify(report, null, 2)}\n`;
  const riskCount = report.businessRisks.value.advisories.length;
  const gitSummary = ['repository', 'worktrees', 'branch', 'head', 'workspace']
    .map((name) => `${name}:${report.git[name].status}`)
    .join(', ');
  return [
    'Adaptive Harness adapt inspect (static, read-only)',
    `Target: ${report.target.value.name}`,
    `Project types: ${factSummary(report.project.type)} (${report.project.type.status})`,
    `Harness: ${factSummary(report.harness.existing)} (${report.harness.existing.status})`,
    `Git: ${gitSummary}`,
    `Authority: ${report.authority.status}`,
    `Test/build/lint/acceptance entries: ${report.entries.test.status}/${report.entries.build.status}/${report.entries.lint.status}/${report.entries.acceptance.status}`,
    `Risk advisories: ${riskCount} (${report.businessRisks.status}; advisory-only)`,
    `Suggested pack: ${report.suggested.projectPack.value.id} (${report.suggested.projectPack.status})`,
    'Safety: no project scripts, network, services, writes, permission changes, or global instructions were used.',
    '',
  ].join('\n');
}

module.exports = {
  DIRECTORY_MARKERS,
  FILE_MARKERS,
  MAX_JSON_BYTES,
  MAX_RISK_ADVISORIES,
  MAX_SCRIPT_NAMES,
  buildAdaptReport,
  formatAdaptReport,
  inspectBusinessRisks,
  inspectGit,
  readBoundedJson,
  resolveTarget,
  sanitizeExternalLabel,
};
