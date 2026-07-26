#!/usr/bin/env node
'use strict';

/*
 * Explicit, read-only maintenance report for the small governance surfaces.
 * It deliberately does not inspect the working-tree overlay, install itself,
 * or rewrite any document.  A non-zero result means the caller asked for a
 * report and should review a recorded drift; this tool is not wired to a Hook,
 * CI, cron, or the default Harness CLI.
 */

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { validateMap, parseNameStatusZ } = require('./codebase-map.js');

const MAX_FINDINGS = 48;
const MAX_VALUE_BYTES = 512;
const MAX_JSON_BYTES = 64 * 1024;
const MAX_READ_BYTES = 512 * 1024;
const PROJECT_CONTEXT_BUDGET = { lines: 25, bytes: 4 * 1024 };
const CURRENT_BUDGET = { lines: 30, bytes: 12 * 1024 };
const POINTER_FIELDS = ['ruleEntry', 'authorityIndex', 'decision', 'taskPackage', 'plan'];
const BOUNDARY_KEYS = ['mechanical', 'reportingOnly', 'explicitTool', 'legacyIgnored'];
const CHECK_IDS = [
  'authority',
  'project-context',
  'current',
  'code-map',
  'active-boundary',
  'legacy-consumers',
];

class ToolError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

function parseArgs(argv) {
  const options = { target: '.', json: false, help: false };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--target') {
      const target = argv[index + 1];
      if (!target || target.startsWith('--')) throw new ToolError('USAGE_ERROR', '--target needs a directory');
      options.target = target;
      index += 1;
    } else if (value.startsWith('--target=')) {
      options.target = value.slice('--target='.length);
      if (!options.target) throw new ToolError('USAGE_ERROR', '--target needs a directory');
    } else if (value === '--json') {
      options.json = true;
    } else if (value === '--help' || value === '-h') {
      options.help = true;
    } else {
      throw new ToolError('USAGE_ERROR', `Unknown option: ${value}`);
    }
  }
  options.target = path.resolve(options.target);
  return options;
}

function helpText() {
  return [
    'Usage: node scripts/harness/maintenance-audit.js --target <directory> [--json]',
    'Explicit, read-only drift report for authority routing, short route budgets, Code Map references, active boundaries, and active legacy consumers.',
    'It does not inspect dirty paths, update documents, install hooks, or schedule itself.',
  ].join('\n');
}

function byteLength(value) {
  return Buffer.byteLength(String(value), 'utf8');
}

function lineCount(value) {
  if (value.length === 0) return 0;
  const lines = value.split(/\r?\n/).length;
  return /\r?\n$/.test(value) ? lines - 1 : lines;
}

function compact(value, maximum = MAX_VALUE_BYTES) {
  const text = String(value ?? '');
  if (byteLength(text) <= maximum) return text;
  let end = text.length;
  while (end > 0 && byteLength(text.slice(0, end)) > maximum - 3) end -= 1;
  return `${text.slice(0, end)}...`;
}

function isSafeRelativePath(value) {
  if (typeof value !== 'string' || value.length === 0) return false;
  if (value.includes('\0') || value.includes('\\') || path.isAbsolute(value) || value.startsWith('./')) return false;
  const parts = value.split('/');
  return parts.every((part) => part && part !== '.' && part !== '..') && path.posix.normalize(value) === value;
}

function resolveInside(target, relativePath) {
  if (!isSafeRelativePath(relativePath)) return null;
  const resolved = path.resolve(target, relativePath);
  return resolved.startsWith(`${target}${path.sep}`) || resolved === target ? resolved : null;
}

function readText(target, relativePath) {
  const location = resolveInside(target, relativePath);
  if (!location) throw new ToolError('UNSAFE_PATH', `Unsafe repository-relative path: ${relativePath}`);
  const stat = fs.statSync(location);
  if (!stat.isFile()) throw new ToolError('NOT_A_FILE', `Not a file: ${relativePath}`);
  if (stat.size > MAX_READ_BYTES) throw new ToolError('READ_LIMIT_EXCEEDED', `${relativePath} exceeds ${MAX_READ_BYTES} byte read limit`);
  return fs.readFileSync(location, 'utf8');
}

function readJson(target, relativePath) {
  return JSON.parse(readText(target, relativePath));
}

function exists(target, relativePath) {
  const location = resolveInside(target, relativePath);
  return Boolean(location && fs.existsSync(location));
}

function git(target, args) {
  const result = spawnSync('git', args, {
    cwd: target,
    encoding: 'utf8',
    timeout: 5000,
    maxBuffer: 64 * 1024,
  });
  if (result.error) throw new ToolError('GIT_ERROR', result.error.message);
  return { status: result.status, stdout: result.stdout || '', stderr: result.stderr || '' };
}

function createCheck(id) {
  return { id, status: 'PASS', findings: [], metrics: {} };
}

function createReporter() {
  return {
    errors: 0,
    warnings: 0,
    omittedFindings: 0,
    findingCount: 0,
    add(check, severity, code, message, extra = {}) {
      if (severity === 'error') {
        this.errors += 1;
        check.status = 'DRIFT';
      } else {
        this.warnings += 1;
        if (check.status === 'PASS') check.status = 'WARN';
      }
      if (this.findingCount >= MAX_FINDINGS) {
        this.omittedFindings += 1;
        check.metrics.omittedFindings = (check.metrics.omittedFindings || 0) + 1;
        return;
      }
      this.findingCount += 1;
      const finding = { severity, code, message: compact(message) };
      for (const [key, value] of Object.entries(extra)) {
        if (typeof value === 'string') finding[key] = compact(value);
        else if (Array.isArray(value)) finding[key] = value.slice(0, 8).map((item) => compact(item));
        else if (value !== undefined && value !== null) finding[key] = value;
      }
      check.findings.push(finding);
    },
  };
}

function markdownSection(text, headingMatcher) {
  const lines = text.split(/\r?\n/);
  let active = false;
  const body = [];
  for (const line of lines) {
    const heading = line.match(/^##\s+(.+)$/);
    if (heading) {
      if (active) break;
      active = headingMatcher(heading[1].trim());
      continue;
    }
    if (active) body.push(line);
  }
  return active ? body.join('\n') : null;
}

function authorityPointers(text) {
  const result = { current: [], superseded: [] };
  const lines = text.split(/\r?\n/);
  let section = null;
  const pointerPattern = /(?:AGENTS|CURRENT|README|backlog)\.md|(?:decisions|docs|tasks|evidence|handoffs|archive)\/[A-Za-z0-9._/-]+\.(?:md|json)/g;

  for (const line of lines) {
    const heading = line.match(/^##\s+(.+)$/);
    if (heading) {
      const name = heading[1].trim().toLocaleLowerCase('en-US');
      if (name.startsWith('一级入口') || name.startsWith('当前业务路由')) section = 'current';
      else if (name.includes('superseded') || name.includes('停用')) section = 'superseded';
      else section = null;
      continue;
    }
    if (!section) continue;
    for (const pointer of line.matchAll(pointerPattern)) {
      if (isSafeRelativePath(pointer[0])) result[section].push(pointer[0]);
    }
  }
  return result;
}

function checkAuthority(target, check, reporter) {
  let text;
  try {
    text = readText(target, 'AUTHORITY.md');
  } catch (error) {
    reporter.add(check, 'error', 'AUTHORITY_INDEX_UNREADABLE', error.message);
    return;
  }
  const pointers = authorityPointers(text);
  const counts = new Map();
  for (const pointer of pointers.current) counts.set(pointer, (counts.get(pointer) || 0) + 1);
  check.metrics.currentPointerCount = pointers.current.length;
  check.metrics.supersededPointerCount = pointers.superseded.length;
  check.metrics.duplicateCurrentCount = [...counts.values()].filter((count) => count > 1).length;

  for (const [pointer, count] of counts) {
    if (count > 1) {
      reporter.add(check, 'error', 'AUTHORITY_DUPLICATE_CURRENT', `Current authority route repeats ${pointer}`, { path: pointer, occurrences: count });
    }
    if (!exists(target, pointer)) {
      reporter.add(check, 'error', 'AUTHORITY_POINTER_MISSING', `Current authority pointer is missing: ${pointer}`, { path: pointer });
    }
  }
  const current = new Set(pointers.current);
  for (const pointer of new Set(pointers.superseded)) {
    if (current.has(pointer)) {
      reporter.add(check, 'error', 'AUTHORITY_SUPERSEDED_CURRENT', `Superseded document is still in a current authority route: ${pointer}`, { path: pointer });
    }
  }
}

function checkProjectContext(target, check, reporter) {
  let text;
  try {
    text = readText(target, 'docs/project-context.json');
  } catch (error) {
    reporter.add(check, 'error', 'PROJECT_CONTEXT_UNREADABLE', error.message);
    return;
  }
  const lines = lineCount(text);
  const bytes = byteLength(text);
  check.metrics = { lines, bytes, budget: PROJECT_CONTEXT_BUDGET };
  if (lines > PROJECT_CONTEXT_BUDGET.lines) {
    reporter.add(check, 'error', 'PROJECT_CONTEXT_LINES_EXCEEDED', `project-context has ${lines} lines; budget is ${PROJECT_CONTEXT_BUDGET.lines}`);
  }
  if (bytes > PROJECT_CONTEXT_BUDGET.bytes) {
    reporter.add(check, 'error', 'PROJECT_CONTEXT_BYTES_EXCEEDED', `project-context has ${bytes} bytes; budget is ${PROJECT_CONTEXT_BUDGET.bytes}`);
  }
  let context;
  try {
    context = JSON.parse(text);
  } catch (error) {
    reporter.add(check, 'error', 'PROJECT_CONTEXT_MALFORMED', `Cannot parse project-context: ${error.message}`);
    return;
  }
  for (const field of POINTER_FIELDS) {
    const pointer = context[field];
    if (!isSafeRelativePath(pointer)) {
      reporter.add(check, 'error', 'PROJECT_CONTEXT_POINTER_INVALID', `project-context.${field} is not a safe repository-relative path`, { field });
    } else if (!exists(target, pointer)) {
      reporter.add(check, 'error', 'PROJECT_CONTEXT_POINTER_MISSING', `project-context.${field} points to a missing file`, { field, path: pointer });
    }
  }
}

function checkCurrent(target, check, reporter) {
  let text;
  try {
    text = readText(target, 'CURRENT.md');
  } catch (error) {
    reporter.add(check, 'error', 'CURRENT_UNREADABLE', error.message);
    return;
  }
  const lines = lineCount(text);
  const bytes = byteLength(text);
  check.metrics = { lines, bytes, budget: CURRENT_BUDGET };
  if (lines > CURRENT_BUDGET.lines) {
    reporter.add(check, 'error', 'CURRENT_LINES_EXCEEDED', `CURRENT.md has ${lines} lines; budget is ${CURRENT_BUDGET.lines}`);
  }
  if (bytes > CURRENT_BUDGET.bytes) {
    reporter.add(check, 'error', 'CURRENT_BYTES_EXCEEDED', `CURRENT.md has ${bytes} bytes; budget is ${CURRENT_BUDGET.bytes}`);
  }
}

function mapReferencePaths(capability) {
  const references = [];
  if (capability && capability.canonical && typeof capability.canonical.path === 'string') references.push(capability.canonical.path);
  for (const field of ['entrypoints', 'publicSymbols', 'consumers', 'stateOwners', 'contracts', 'tests']) {
    if (!Array.isArray(capability && capability[field])) continue;
    for (const reference of capability[field]) {
      if (reference && typeof reference.path === 'string') references.push(reference.path);
    }
  }
  return [...new Set(references.filter(isSafeRelativePath))];
}

function stagedCanonicalImpacts(target, capabilities) {
  const canonicalByPath = new Map();
  for (const capability of capabilities) {
    const canonical = capability && capability.canonical;
    if (!canonical || !isSafeRelativePath(canonical.path)) continue;
    const entries = canonicalByPath.get(canonical.path) || [];
    entries.push(capability);
    canonicalByPath.set(canonical.path, entries);
  }
  const impacts = [];
  const pendingDeletes = [];
  for (const [canonicalPath, mappedCapabilities] of canonicalByPath) {
    const result = git(target, ['diff', '--cached', '--name-status', '-z', '--find-renames', '--', canonicalPath]);
    if (result.status !== 0) throw new ToolError('GIT_ERROR', result.stderr.trim() || 'Cannot inspect staged canonical path');
    for (const change of parseNameStatusZ(result.stdout)) {
      if (change.code === 'R' && change.from === canonicalPath) {
        for (const capability of mappedCapabilities) {
          impacts.push({ kind: 'rename', from: change.from, to: change.to, capabilityId: capability.id });
        }
      } else if (change.code === 'D' && change.path === canonicalPath) {
        pendingDeletes.push({ path: change.path, mappedCapabilities });
      }
    }
  }
  if (pendingDeletes.length === 0) return impacts;

  // A path-limited diff represents a rename of its source as a deletion.  Ask
  // Git for rename records only, rather than the entire staged change list.
  const renameResult = git(target, ['diff', '--cached', '--name-status', '-z', '--find-renames', '--diff-filter=R']);
  if (renameResult.status !== 0) throw new ToolError('GIT_ERROR', renameResult.stderr.trim() || 'Cannot inspect staged renames');
  const renamesByFrom = new Map();
  for (const change of parseNameStatusZ(renameResult.stdout)) {
    if (change.code === 'R' && change.from && change.to) renamesByFrom.set(change.from, change.to);
  }
  for (const pending of pendingDeletes) {
    const to = renamesByFrom.get(pending.path);
    for (const capability of pending.mappedCapabilities) {
      impacts.push(to
        ? { kind: 'rename', from: pending.path, to, capabilityId: capability.id }
        : { kind: 'delete', path: pending.path, capabilityId: capability.id });
    }
  }
  return impacts;
}

function checkCodeMap(target, check, reporter) {
  let map;
  try {
    map = validateMap(target);
  } catch (error) {
    reporter.add(check, 'error', 'CODE_MAP_INVALID', `Code Map validation could not run: ${error.message}`);
    return;
  }
  const errors = Array.isArray(map.errors) ? map.errors : [];
  const capabilities = Array.isArray(map.capabilities) ? map.capabilities : [];
  check.metrics.capabilityCount = capabilities.length;
  check.metrics.validationErrorCount = errors.length;
  if (errors.length > 0) {
    reporter.add(check, 'error', 'CODE_MAP_INVALID', `Code Map validation reported ${errors.length} error(s)`, {
      sourceCodes: [...new Set(errors.map((error) => error.code).filter(Boolean))],
    });
    return;
  }
  let stagedImpacts;
  try {
    stagedImpacts = stagedCanonicalImpacts(target, capabilities);
  } catch (error) {
    reporter.add(check, 'error', 'STAGED_CANONICAL_IMPACT_UNAVAILABLE', error.message);
    return;
  }
  check.metrics.stagedCanonicalImpactCount = stagedImpacts.length;
  for (const impact of stagedImpacts) {
    if (impact.kind === 'rename') {
      reporter.add(check, 'error', 'STAGED_RENAME_AFFECTS_CAPABILITY', `Staged rename affects canonical capability ${impact.capabilityId}`, {
        capabilityId: impact.capabilityId,
        from: impact.from,
        to: impact.to,
      });
    } else {
      reporter.add(check, 'error', 'STAGED_DELETE_AFFECTS_CAPABILITY', `Staged deletion affects canonical capability ${impact.capabilityId}`, {
        capabilityId: impact.capabilityId,
        path: impact.path,
      });
    }
  }
  let head;
  try {
    const result = git(target, ['rev-parse', 'HEAD']);
    if (result.status !== 0) throw new ToolError('GIT_ERROR', result.stderr.trim() || 'Cannot resolve HEAD');
    head = result.stdout.trim();
  } catch (error) {
    reporter.add(check, 'error', 'CODE_MAP_GIT_UNAVAILABLE', error.message);
    return;
  }

  let staleCount = 0;
  let changedReferenceCount = 0;
  for (const capability of capabilities) {
    const verified = capability.verifiedAtCommit;
    if (typeof verified !== 'string' || !verified || verified === head) continue;
    staleCount += 1;
    reporter.add(check, 'warning', 'STALE_VERIFIED_COMMIT', `${capability.id} was verified at an earlier commit`, {
      capabilityId: capability.id,
      verifiedAtCommit: verified,
      head,
    });
    for (const referencePath of mapReferencePaths(capability)) {
      let diff;
      try {
        diff = git(target, ['diff', '--quiet', verified, 'HEAD', '--', referencePath]);
      } catch {
        continue;
      }
      if (diff.status === 1) {
        changedReferenceCount += 1;
        reporter.add(check, 'warning', 'MAPPED_REFERENCE_CHANGED_SINCE_VERIFICATION', `${capability.id} has a mapped reference changed since verification`, {
          capabilityId: capability.id,
          path: referencePath,
        });
      }
    }
  }
  check.metrics.staleVerificationCount = staleCount;
  check.metrics.changedReferenceCount = changedReferenceCount;
}

function sortedStrings(values) {
  return [...values].sort((left, right) => left.localeCompare(right));
}

function equalStringArrays(left, right) {
  return JSON.stringify(sortedStrings(left)) === JSON.stringify(sortedStrings(right));
}

function validateBoundary(value) {
  const errors = [];
  if (!value || typeof value !== 'object' || Array.isArray(value)) return { errors: ['activeBoundary must be an object'], groups: null };
  const groups = {};
  const seen = new Map();
  for (const key of BOUNDARY_KEYS) {
    const entries = value[key];
    if (!Array.isArray(entries) || entries.some((entry) => typeof entry !== 'string' || !entry.trim())) {
      errors.push(`${key} must be a string array`);
      continue;
    }
    groups[key] = entries;
    for (const entry of entries) {
      if (seen.has(entry)) errors.push(`${entry} is duplicated across ${seen.get(entry)} and ${key}`);
      else seen.set(entry, key);
    }
  }
  for (const key of Object.keys(value)) {
    if (!BOUNDARY_KEYS.includes(key)) errors.push(`unexpected activeBoundary key: ${key}`);
  }
  return { errors, groups };
}

function parseDefaultRoutes(output) {
  const start = output.indexOf('Current manual entrypoints:');
  if (start < 0) return null;
  const remainder = output.slice(start + 'Current manual entrypoints:'.length);
  const end = remainder.indexOf('Examples:');
  const block = end < 0 ? remainder : remainder.slice(0, end);
  const routes = [];
  for (const line of block.split(/\r?\n/)) {
    const match = line.match(/^\s{2}(.+?)\s{2,}/);
    if (match) routes.push(match[1].trim());
  }
  return routes;
}

function defaultHelp(target) {
  const script = resolveInside(target, 'scripts/harness/harness.js');
  if (!script || !fs.existsSync(script)) throw new ToolError('DEFAULT_CLI_UNAVAILABLE', 'scripts/harness/harness.js is missing');
  const result = spawnSync(process.execPath, [script, '--help'], {
    cwd: target,
    encoding: 'utf8',
    timeout: 5000,
    maxBuffer: 32 * 1024,
  });
  if (result.error) throw new ToolError('DEFAULT_CLI_UNAVAILABLE', result.error.message);
  if (result.status !== 0) throw new ToolError('DEFAULT_CLI_UNAVAILABLE', (result.stderr || '').trim() || 'harness.js --help failed');
  const routes = parseDefaultRoutes(result.stdout || '');
  if (!routes || routes.length === 0) {
    throw new ToolError('DEFAULT_CLI_UNPARSEABLE', 'harness.js --help has no parseable default manual entrypoints');
  }
  return routes;
}

function checkActiveBoundary(target, check, reporter) {
  let projectConfig;
  let exampleConfig;
  try {
    projectConfig = readJson(target, 'harness.config.json');
    exampleConfig = readJson(target, 'harness.config.example.json');
  } catch (error) {
    reporter.add(check, 'error', 'ACTIVE_BOUNDARY_CONFIG_INVALID', error.message);
    return;
  }
  const project = validateBoundary(projectConfig.activeBoundary);
  const example = validateBoundary(exampleConfig.activeBoundary);
  if (project.errors.length || example.errors.length) {
    reporter.add(check, 'error', 'ACTIVE_BOUNDARY_CONFIG_INVALID', 'activeBoundary must have exactly four non-overlapping string-array categories', {
      projectErrors: project.errors,
      exampleErrors: example.errors,
    });
  }
  const projectGroupsComplete = project.groups && BOUNDARY_KEYS.every((key) => Array.isArray(project.groups[key]));
  const exampleGroupsComplete = example.groups && BOUNDARY_KEYS.every((key) => Array.isArray(example.groups[key]));
  if (projectGroupsComplete && exampleGroupsComplete) {
    const drift = BOUNDARY_KEYS.some((key) => !equalStringArrays(project.groups[key], example.groups[key]));
    if (drift) reporter.add(check, 'error', 'ACTIVE_BOUNDARY_CONFIG_DRIFT', 'Project and example activeBoundary declarations differ');
  }

  let routes = [];
  try {
    routes = defaultHelp(target);
  } catch (error) {
    reporter.add(check, 'error', 'DEFAULT_CLI_BOUNDARY_DRIFT', error.message);
  }
  check.metrics.defaultCliRouteCount = routes.length;
  if (projectGroupsComplete && routes.length > 0) {
    const expected = [
      ...project.groups.reportingOnly,
      ...project.groups.explicitTool,
      'shape',
    ];
    const noDuplicates = new Set(routes).size === routes.length;
    if (!noDuplicates || !equalStringArrays(routes, expected)) {
      reporter.add(check, 'error', 'DEFAULT_CLI_BOUNDARY_DRIFT', 'Default CLI routes do not match activeBoundary reportingOnly + explicitTool + shape', {
        expected,
        actual: routes,
      });
    }
  }

  let catalog;
  try {
    catalog = readText(target, 'docs/harness-catalog.md');
  } catch (error) {
    reporter.add(check, 'error', 'CATALOG_ACTIVE_BOUNDARY_DRIFT', error.message);
    return;
  }
  const activeSection = markdownSection(catalog, (heading) => heading === 'Active boundary');
  const cliSection = markdownSection(catalog, (heading) => heading === '根 CLI');
  const requiredMarkers = ['`mechanical`', '`reportingOnly`', '`explicitTool`', '`legacyIgnored`', 'Code Map', 'Stage K', 'context diagnostic', 'doctor'];
  const catalogMatchesRoutes = routes.every((route) => cliSection && cliSection.includes(`\`${route}\``));
  if (!activeSection || !cliSection || requiredMarkers.some((marker) => !activeSection.includes(marker)) || !catalogMatchesRoutes) {
    reporter.add(check, 'error', 'CATALOG_ACTIVE_BOUNDARY_DRIFT', 'Harness catalog does not accurately describe the active boundary and default CLI routes');
  }
}

function configuredCommands(config) {
  const values = [];
  for (const phase of ['preWork', 'preCompletion']) {
    for (const field of ['recommendedChecks', 'strictPathRecommendedChecks']) {
      const entries = config && config[phase] && config[phase][field];
      if (Array.isArray(entries)) values.push(...entries.filter((entry) => typeof entry === 'string'));
    }
  }
  return values;
}

function isLegacyConsumer(value) {
  return /scripts\/harness\/(?:memory-[\w-]+|task-(?:start|finish|status|risk|package-[\w-]+)|evidence-[\w-]+|runtime-docs-init|hook-(?:install|uninstall)|ci-(?:init|validate)|capability-(?:map|scan)|pre-work|pre-completion)\.js\b/i.test(value);
}

function effectiveHookFiles(target) {
  let configured;
  try {
    const result = git(target, ['config', '--get', 'core.hooksPath']);
    configured = result.status === 0 ? result.stdout.trim() : '';
  } catch {
    configured = '';
  }
  if (!configured || !isSafeRelativePath(configured)) return [];
  const hookDirectory = resolveInside(target, configured);
  if (!hookDirectory || !fs.existsSync(hookDirectory)) return [];
  let entries;
  try {
    entries = fs.readdirSync(hookDirectory, { withFileTypes: true }).filter((entry) => entry.isFile()).slice(0, 32);
  } catch {
    return [];
  }
  return entries.map((entry) => path.join(configured, entry.name)).filter(isSafeRelativePath);
}

function workflowFiles(target) {
  const directory = resolveInside(target, '.github/workflows');
  if (!directory || !fs.existsSync(directory)) return [];
  try {
    return fs.readdirSync(directory, { withFileTypes: true })
      .filter((entry) => entry.isFile() && /\.ya?ml$/i.test(entry.name))
      .slice(0, 32)
      .map((entry) => `.github/workflows/${entry.name}`);
  } catch {
    return [];
  }
}

function checkLegacyConsumers(target, check, reporter) {
  let config;
  try {
    config = readJson(target, 'harness.config.json');
  } catch (error) {
    reporter.add(check, 'error', 'LEGACY_CONSUMER_CONFIG_UNREADABLE', error.message);
    return;
  }
  const sources = [];
  for (const command of configuredCommands(config)) sources.push({ kind: 'config', value: command });
  try {
    for (const route of defaultHelp(target)) sources.push({ kind: 'default-cli', value: route });
  } catch {
    // active-boundary owns default CLI parsing failures; do not misclassify
    // them as an active legacy consumer.
    check.metrics.defaultCliRoutesUnavailable = true;
  }
  for (const file of [...effectiveHookFiles(target), ...workflowFiles(target)]) {
    try {
      sources.push({ kind: 'automatic-file', value: readText(target, file) });
    } catch {
      // A missing or unreadable optional automation file is not inferred as a consumer.
    }
  }
  const legacy = sources.filter((source) => isLegacyConsumer(source.value));
  check.metrics.activeConsumerCount = legacy.length;
  check.metrics.scannedSourceCount = sources.length;
  for (const source of legacy) {
    reporter.add(check, 'error', 'ACTIVE_LEGACY_CONSUMER', `An active ${source.kind} surface references a legacy consumer`);
  }
}

function runAudit(target) {
  if (!fs.existsSync(target) || !fs.statSync(target).isDirectory()) {
    throw new ToolError('TARGET_NOT_DIRECTORY', `Target is not a directory: ${target}`);
  }
  const reporter = createReporter();
  const checks = CHECK_IDS.map(createCheck);
  const runners = [checkAuthority, checkProjectContext, checkCurrent, checkCodeMap, checkActiveBoundary, checkLegacyConsumers];
  for (let index = 0; index < runners.length; index += 1) {
    try {
      runners[index](target, checks[index], reporter);
    } catch (error) {
      reporter.add(checks[index], 'error', 'AUDIT_CHECK_FAILED', error.message || String(error));
    }
  }
  return {
    schemaVersion: 1,
    tool: 'maintenance-audit',
    status: reporter.errors === 0 ? 'OK' : 'DRIFT',
    readOnly: true,
    dirtyTreeInspected: false,
    outputBounded: true,
    limits: { maxFindings: MAX_FINDINGS, maxJsonBytes: MAX_JSON_BYTES },
    summary: {
      errorCount: reporter.errors,
      warningCount: reporter.warnings,
      omittedFindingCount: reporter.omittedFindings,
    },
    checks,
  };
}

function boundedJson(report) {
  let serialized = JSON.stringify(report);
  if (byteLength(serialized) <= MAX_JSON_BYTES) return serialized;
  const compactReport = {
    schemaVersion: report.schemaVersion,
    tool: report.tool,
    status: report.status,
    readOnly: true,
    dirtyTreeInspected: false,
    outputBounded: true,
    truncated: true,
    limits: report.limits,
    summary: report.summary,
    checks: report.checks.map((check) => ({
      id: check.id,
      status: check.status,
      metrics: check.metrics,
      findings: [],
    })),
  };
  serialized = JSON.stringify(compactReport);
  if (byteLength(serialized) <= MAX_JSON_BYTES) return serialized;
  return JSON.stringify({
    schemaVersion: 1,
    tool: 'maintenance-audit',
    status: report.status,
    readOnly: true,
    dirtyTreeInspected: false,
    outputBounded: true,
    truncated: true,
    summary: report.summary,
  });
}

function renderText(report) {
  const lines = [
    `maintenance-audit: ${report.status}`,
    `errors=${report.summary.errorCount} warnings=${report.summary.warningCount} dirtyTreeInspected=false`,
  ];
  for (const check of report.checks) {
    lines.push(`- ${check.id}: ${check.status}`);
  }
  return lines.join('\n');
}

function failureReport(error) {
  const message = error && error.message ? error.message : String(error);
  return {
    schemaVersion: 1,
    tool: 'maintenance-audit',
    status: 'ERROR',
    readOnly: true,
    dirtyTreeInspected: false,
    outputBounded: true,
    limits: { maxFindings: MAX_FINDINGS, maxJsonBytes: MAX_JSON_BYTES },
    summary: { errorCount: 1, warningCount: 0, omittedFindingCount: 0 },
    checks: [],
    errors: [{ code: error && error.code ? error.code : 'UNEXPECTED_ERROR', message: compact(message) }],
  };
}

function main(argv = process.argv.slice(2)) {
  let options;
  try {
    options = parseArgs(argv);
  } catch (error) {
    const report = failureReport(error);
    process.stdout.write(`${JSON.stringify(report)}\n`);
    return 2;
  }
  if (options.help) {
    process.stdout.write(`${helpText()}\n`);
    return 0;
  }
  try {
    const report = runAudit(options.target);
    process.stdout.write(options.json ? `${boundedJson(report)}\n` : `${renderText(report)}\n`);
    return report.status === 'OK' ? 0 : 1;
  } catch (error) {
    const report = failureReport(error);
    process.stdout.write(options.json ? `${JSON.stringify(report)}\n` : `maintenance-audit: ERROR\n`);
    return 2;
  }
}

if (require.main === module) process.exitCode = main();

module.exports = { parseArgs, runAudit, authorityPointers, validateBoundary };
