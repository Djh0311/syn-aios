#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { TextDecoder } = require('node:util');
const {
  MAP_INDEX_PATH,
  entriesForPath,
  loadCodeMap,
  searchEntries,
  stripFragment,
  validateCodeMap
} = require('./lib/code-map-model');
const { safeOutputRepoPath, sanitizeOutputText } = require('./lib/output-safety');
const stagedTree = require('./lib/staged-tree');

const MAP_PREFIX = 'docs/code-map/';
const DEFAULT_LIMIT = 10;
const MAX_LIMIT = 50;
const UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });

function parseArgs(argv) {
  const values = [...argv];
  const args = {
    command: values.shift() || null,
    target: process.cwd(),
    query: '',
    json: false,
    staged: false,
    shadow: false,
    limit: DEFAULT_LIMIT
  };
  for (let index = 0; index < values.length; index += 1) {
    const item = values[index];
    if (item === '--target') args.target = values[++index];
    else if (item === '--query') args.query = values[++index];
    else if (item === '--limit') args.limit = Number(values[++index]);
    else if (item === '--json') args.json = true;
    else if (item === '--staged') args.staged = true;
    else if (item === '--shadow') args.shadow = true;
    else throw new Error(`Unknown argument: ${item}`);
  }
  if (!['query', 'overlay', 'check'].includes(args.command)) {
    throw new Error('Command must be query, overlay, or check');
  }
  if (!args.target) throw new Error('--target requires a value');
  if (!Number.isInteger(args.limit) || args.limit < 1 || args.limit > MAX_LIMIT) {
    throw new Error(`--limit must be between 1 and ${MAX_LIMIT}`);
  }
  if (args.command === 'query' && !String(args.query || '').trim()) {
    throw new Error('query requires --query');
  }
  if (args.command === 'check' && !args.staged) throw new Error('check requires --staged');
  if (args.shadow && args.command !== 'check') throw new Error('--shadow is only valid for check');
  if (args.staged && args.command !== 'check') throw new Error('--staged is only valid for check');
  args.target = path.resolve(args.target);
  return args;
}

function baseReport(args) {
  return {
    command: args.command,
    status: 'PASS',
    mode: args.shadow ? 'shadow-advisory' : 'read-only',
    target: '.',
    pass: [],
    warn: [],
    details: {}
  };
}

function unique(items) {
  return [...new Set(items)];
}

function worktreeReader(target) {
  let root;
  try {
    root = fs.realpathSync(target);
  } catch (error) {
    return () => null;
  }
  return (relativePath) => {
    const safe = stagedTree.safeRepoPath(relativePath);
    if (!safe) return null;
    const full = path.join(root, safe);
    try {
      if (!fs.lstatSync(full).isFile()) return null;
      const real = fs.realpathSync(full);
      if (real !== root && !real.startsWith(`${root}${path.sep}`)) return null;
      if (!fs.lstatSync(real).isFile()) return null;
      // Buffer→fatal UTF-8 decode：含非法字节的 canonical 不能被替换字符悄悄
      // 当成已读源码。无法可靠解释的内容与读失败一样返回 UNKNOWN。
      return UTF8_DECODER.decode(fs.readFileSync(real));
    } catch (error) {
      return null;
    }
  };
}

function indexReader(target) {
  return (relativePath) => {
    const result = stagedTree.readIndexFile(target, relativePath);
    return result.ok && result.exists ? result.content : null;
  };
}

function pathExistsInWorktree(target, relativePath) {
  const result = stagedTree.worktreePathExists(target, relativePath);
  return result.ok && result.exists;
}

function pathExistsInIndex(target, relativePath) {
  const result = stagedTree.indexPathExists(target, relativePath);
  return result.ok && result.exists;
}

function committedMapAvailable(target) {
  return stagedTree.git(target, ['cat-file', '-e', `HEAD:${MAP_INDEX_PATH}`]).ok;
}

function loadForWorktree(args) {
  return loadCodeMap(args.target, { readFile: worktreeReader(args.target) });
}

function loadForIndex(args) {
  return loadCodeMap(args.target, { readFile: indexReader(args.target) });
}

function safeSeedHead(value) {
  return typeof value === 'string' && /^[0-9a-f]{40}$/i.test(value) ? value : null;
}

function sourcePosition(source, query) {
  const needle = String(query || '');
  let offset = source.indexOf(needle);
  if (offset < 0) {
    const foldedSource = source.toLocaleLowerCase();
    const foldedNeedle = needle.toLocaleLowerCase();
    if (foldedSource.length !== source.length || foldedNeedle.length !== needle.length) return null;
    offset = foldedSource.indexOf(foldedNeedle);
  }
  if (offset < 0) return null;
  const preceding = source.slice(0, offset);
  const line = preceding.split(/\r\n|\r|\n/).length;
  const lastLineBreak = Math.max(preceding.lastIndexOf('\n'), preceding.lastIndexOf('\r'));
  return { line, column: offset - lastLineBreak };
}

function sourceBackedMatches(entries, query, readFile) {
  const locations = [];
  const inspected = new Set();
  const foldedQuery = String(query || '').toLocaleLowerCase();
  for (const { entry } of entries) {
    for (const reference of Array.isArray(entry.canonical) ? entry.canonical : []) {
      const relativePath = typeof reference === 'string'
        ? stagedTree.safeRepoPath(stripFragment(reference))
        : null;
      if (!relativePath || inspected.has(relativePath)) continue;
      inspected.add(relativePath);
      const source = readFile(relativePath);
      if (source == null) continue;
      // Code Map 只负责给候选；命中事实必须来自刚刚实际读取成功的 canonical
      // 文件。正文命中返回精确行列；同名模块/文件路径命中则返回该已读文件的
      // 起点。索引里的 name/summary/symbol/consumer 本身永远不会被输出。
      const position = sourcePosition(source, query)
        || (relativePath.toLocaleLowerCase().includes(foldedQuery)
          ? { line: 1, column: 1 }
          : null);
      if (!position) continue;
      locations.push({
        path: safeOutputRepoPath(relativePath),
        line: position.line,
        column: position.column
      });
    }
  }
  return locations;
}

function unknownQuery(report, args, model, validation, warning) {
  report.status = 'UNKNOWN';
  report.warn.push(warning);
  report.details.query = sanitizeOutputText(args.query, 160);
  report.details.baselined = Boolean(model.index) && committedMapAvailable(args.target);
  report.details.seedHead = model.index ? safeSeedHead(model.index.seedHead) : null;
  report.details.matchCount = 0;
  report.details.matches = [];
  report.details.validationFindingCount = validation.fail.length;
  report.details.truncated = false;
  return report;
}

function buildQuery(args) {
  const report = baseReport(args);
  const model = loadForWorktree(args);
  const validation = validateCodeMap(model, {
    pathExists: (relativePath) => pathExistsInWorktree(args.target, relativePath)
  });
  if (!model.index) {
    return unknownQuery(
      report,
      args,
      model,
      validation,
      'MAP_ADVISORY_UNAVAILABLE Code Map index is unavailable'
    );
  }
  if (validation.fail.length > 0) {
    return unknownQuery(
      report,
      args,
      model,
      validation,
      'MAP_UNKNOWN Code Map candidate is stale or unreadable'
    );
  }
  const candidates = searchEntries(model.entries, args.query, model.entries.length);
  const matches = sourceBackedMatches(candidates, args.query, worktreeReader(args.target));
  if (matches.length === 0) {
    return unknownQuery(
      report,
      args,
      model,
      validation,
      'MAP_UNKNOWN No readable canonical source confirmed the query'
    );
  }
  report.details.query = sanitizeOutputText(args.query, 160);
  report.details.baselined = committedMapAvailable(args.target);
  report.details.seedHead = safeSeedHead(model.index.seedHead);
  report.details.matchCount = matches.length;
  report.details.matches = matches.slice(0, args.limit);
  report.details.validationFindingCount = validation.fail.length;
  report.details.truncated = matches.length > args.limit;
  if (!report.details.baselined) {
    report.status = 'WARN';
    report.warn.push('MAP_REVIEW_REQUIRED Query uses an uncommitted Code Map candidate');
  }
  report.pass.push(`Found ${matches.length} source-backed location${matches.length === 1 ? '' : 's'}`);
  return report;
}

function isMapPath(relativePath) {
  return String(relativePath || '').startsWith(MAP_PREFIX);
}

function relevantChanges(changes) {
  return changes.filter((change) => {
    const paths = [change.oldPath, change.newPath, change.path].filter(Boolean);
    return paths.some((relativePath) => !isMapPath(relativePath) && stagedTree.isRelevantSourcePath(relativePath));
  });
}

function annotate(model, layer, change) {
  const paths = unique([change.oldPath, change.newPath, change.path].filter(Boolean));
  const mappedCapabilities = unique(
    paths.flatMap((relativePath) => entriesForPath(model.entries, relativePath).map((entry) => entry.id))
  );
  return {
    layer: sanitizeOutputText(layer, 40),
    status: sanitizeOutputText(change.status || '?', 8),
    oldPath: change.oldPath ? safeOutputRepoPath(change.oldPath) : null,
    path: change.newPath || change.path
      ? safeOutputRepoPath(change.newPath || change.path)
      : null,
    mappedCapabilities: mappedCapabilities.map((id) => sanitizeOutputText(id, 160)),
    mapState: mappedCapabilities.length > 0 ? 'MAPPED_POSSIBLY_STALE' : 'UNMAPPED_UNCOMMITTED'
  };
}

function buildOverlay(args) {
  const report = baseReport(args);
  const model = loadForWorktree(args);
  const validation = validateCodeMap(model, {
    pathExists: (relativePath) => pathExistsInWorktree(args.target, relativePath)
  });
  if (!model.index) {
    report.status = 'NOT_BASELINED';
    report.warn.push('MAP_ADVISORY_UNAVAILABLE Code Map index is unavailable');
  } else if (validation.fail.length > 0) {
    report.status = 'WARN';
    report.warn.push(`MAP_REVIEW_REQUIRED Code Map has ${validation.fail.length} validation finding(s)`);
  }

  const staged = stagedTree.stagedChanges(args.target);
  const unstaged = stagedTree.unstagedChanges(args.target);
  const untracked = stagedTree.untrackedPaths(args.target);
  const errors = [staged, unstaged, untracked].filter((result) => !result.ok);
  if (errors.length > 0) {
    report.status = 'INCOMPLETE_SHADOW';
    report.warn.push('MAP_ADVISORY_UNAVAILABLE One or more Git layers could not be inspected');
  }
  const stagedRelevant = relevantChanges(staged.ok ? staged.changes : []);
  const unstagedRelevant = relevantChanges(unstaged.ok ? unstaged.changes : []);
  const untrackedRelevant = (untracked.ok ? untracked.paths : [])
    .filter(stagedTree.isRelevantSourcePath)
    .map((relativePath) => ({ status: '?', path: relativePath }));
  const allItems = [
    ...stagedRelevant.map((change) => annotate(model, 'staged', change)),
    ...unstagedRelevant.map((change) => annotate(model, 'unstaged', change)),
    ...untrackedRelevant.map((change) => annotate(model, 'untracked', change))
  ];
  report.details.summary = {
    staged: stagedRelevant.length,
    unstaged: unstagedRelevant.length,
    untracked: untrackedRelevant.length,
    mapped: allItems.filter((item) => item.mappedCapabilities.length > 0).length,
    unmapped: allItems.filter((item) => item.mappedCapabilities.length === 0).length
  };
  report.details.itemCount = allItems.length;
  report.details.items = allItems.slice(0, args.limit);
  report.details.truncated = allItems.length > args.limit;
  if (allItems.length === 0) report.pass.push('No source-relevant uncommitted paths detected');
  else {
    if (report.status === 'PASS') report.status = 'WARN';
    report.warn.push(`MAP_REVIEW_REQUIRED ${allItems.length} uncommitted source path state(s) detected`);
  }
  return report;
}

function mapFilesChanged(changes) {
  return changes.some((change) => (
    [change.oldPath, change.newPath, change.path].filter(Boolean).some(isMapPath)
  ));
}

function buildCheck(args) {
  const report = baseReport(args);
  report.mode = 'shadow-advisory';
  const staged = stagedTree.stagedChanges(args.target);
  if (!staged.ok) {
    report.status = 'INCOMPLETE_SHADOW';
    report.warn.push('MAP_ADVISORY_UNAVAILABLE Staged Git state could not be inspected');
    report.details = {
      stagedChangeCount: 0,
      relevantChangeCount: 0,
      mapFilesStaged: false,
      findingCount: 1,
      findings: [{
        code: 'MAP_ADVISORY_UNAVAILABLE',
        message: 'Staged Git state could not be inspected'
      }],
      truncated: false
    };
    return report;
  }

  const allChanges = staged.changes;
  const relevant = relevantChanges(allChanges);
  const classified = stagedTree.classifyStagedChanges(args.target, relevant);
  const impacts = [];
  for (const result of classified.results) {
    if (result.structural) {
      impacts.push({
        change: result.change,
        signal: result.signal || 'unknown-structural-impact',
        classificationError: result.ok ? null : result.error
      });
    }
  }
  const mapStaged = mapFilesChanged(allChanges);
  const findings = [];
  const addFinding = (code, message, extra = {}) => {
    findings.push(Object.assign({ code, message }, extra));
  };

  report.details.stagedChangeCount = allChanges.length;
  report.details.relevantChangeCount = impacts.length;
  report.details.mapFilesStaged = mapStaged;

  if (!pathExistsInIndex(args.target, MAP_INDEX_PATH)) {
    for (const impact of impacts) {
      const change = impact.change;
      addFinding(
        'MAP_REVIEW_REQUIRED',
        'Candidate structural impact while Code Map is unavailable',
        {
          status: change.status,
          path: change.newPath || change.path || change.oldPath,
          oldPath: change.oldPath || null,
          signal: impact.signal
        }
      );
    }
    if (findings.length === 0) {
      addFinding('MAP_ADVISORY_UNAVAILABLE', 'No staged-tree Code Map baseline exists');
    }
    report.status = 'NOT_BASELINED';
    report.warn.push('MAP_ADVISORY_UNAVAILABLE No staged-tree Code Map baseline exists');
  } else {
    const model = loadForIndex(args);
    const validation = validateCodeMap(model, {
      pathExists: (relativePath) => pathExistsInIndex(args.target, relativePath)
    });
    report.details.seedHead = model.index ? safeSeedHead(model.index.seedHead) : null;
    for (const message of validation.fail) {
      addFinding('MAP_UPDATE_REQUIRED', sanitizeOutputText(message, 320));
    }

    for (const impact of impacts) {
      const change = impact.change;
      const oldPath = change.oldPath || (change.status === 'D' ? change.path : null);
      const newPath = change.newPath || (change.status !== 'D' ? change.path : null);
      const oldCapabilities = oldPath
        ? entriesForPath(model.entries, oldPath).map((entry) => entry.id)
        : [];
      const newCapabilities = newPath
        ? entriesForPath(model.entries, newPath).map((entry) => entry.id)
        : [];

      if (oldPath && ['D', 'R'].includes(change.status) && oldCapabilities.length > 0) {
        addFinding('MAP_UPDATE_REQUIRED', 'Removed or moved path is still referenced', {
          status: change.status,
          path: oldPath,
          signal: impact.signal,
          capabilities: oldCapabilities.slice(0, 10)
        });
      }
      if (newPath && ['A', 'R', 'C'].includes(change.status) && newCapabilities.length === 0) {
        addFinding('MAP_UPDATE_REQUIRED', 'New source path has no capability mapping', {
          status: change.status,
          path: newPath,
          oldPath: oldPath || null,
          signal: impact.signal
        });
      }
      if (
        newPath &&
        !['A', 'R', 'C'].includes(change.status) &&
        newCapabilities.length === 0
      ) {
        addFinding('MAP_REVIEW_REQUIRED', 'Structurally changed source is not mapped', {
          status: change.status,
          path: newPath,
          signal: impact.signal
        });
      } else if (
        newPath &&
        change.status === 'M' &&
        newCapabilities.length > 0 &&
        !mapStaged
      ) {
        addFinding('MAP_REVIEW_REQUIRED', 'Mapped source boundary changed without a staged Code Map update', {
          status: change.status,
          path: newPath,
          signal: impact.signal,
          capabilities: newCapabilities.slice(0, 10)
        });
      }
      if (impact.classificationError) {
        addFinding('MAP_REVIEW_REQUIRED', 'Structural classification failed closed to advisory', {
          status: change.status,
          path: newPath || oldPath,
          signal: impact.signal
        });
      }
    }
    if (findings.length > 0) {
      report.status = 'WARN';
      const required = findings.filter((item) => item.code === 'MAP_UPDATE_REQUIRED').length;
      const review = findings.filter((item) => item.code === 'MAP_REVIEW_REQUIRED').length;
      if (required > 0) report.warn.push(`MAP_UPDATE_REQUIRED ${required} finding(s)`);
      if (review > 0) report.warn.push(`MAP_REVIEW_REQUIRED ${review} finding(s)`);
    } else if (impacts.length === 0) {
      report.status = 'NO_RELEVANT_STAGED_CHANGE';
      report.pass.push('No high-confidence staged Code Map impact detected');
    } else {
      report.pass.push('Staged Code Map is consistent with detected structural impacts');
    }
  }

  report.details.findingCount = findings.length;
  report.details.findings = findings.slice(0, args.limit).map((finding) => ({
    ...finding,
    message: sanitizeOutputText(finding.message, 320),
    path: finding.path ? safeOutputRepoPath(finding.path) : finding.path,
    oldPath: finding.oldPath ? safeOutputRepoPath(finding.oldPath) : finding.oldPath,
    signal: finding.signal ? sanitizeOutputText(finding.signal, 120) : finding.signal,
    capabilities: Array.isArray(finding.capabilities)
      ? finding.capabilities.map((id) => sanitizeOutputText(id, 160))
      : finding.capabilities
  }));
  report.details.truncated = findings.length > args.limit;
  return report;
}

function renderText(report) {
  const lines = [`code-map ${report.command}: ${report.status}`];
  for (const item of report.pass) lines.push(`PASS ${item}`);
  for (const item of report.warn) lines.push(`WARN ${item}`);
  if (report.command === 'query') {
    for (const match of report.details.matches || []) {
      lines.push(`MATCH ${JSON.stringify(match.path)}:${match.line}:${match.column}`);
    }
  } else if (report.command === 'overlay') {
    for (const item of report.details.items || []) {
      lines.push(`${item.layer.toUpperCase()} ${item.status} ${JSON.stringify(item.path || item.oldPath)} ${item.mapState}`);
    }
  } else {
    for (const finding of report.details.findings || []) {
      const location = finding.path ? ` ${JSON.stringify(finding.path)}` : '';
      lines.push(`${finding.code}${location} | ${finding.message}`);
    }
  }
  if (report.details.truncated) {
    const total = report.command === 'query'
      ? report.details.matchCount
      : (report.details.findingCount || report.details.itemCount);
    lines.push(`WARN Output limited; ${total} total item(s)`);
  }
  return `${lines.join('\n')}\n`;
}

function exitCode(report, args) {
  if (args.command === 'check' && args.shadow) return 0;
  return 0;
}

function main(argv = process.argv.slice(2)) {
  let args;
  try {
    args = parseArgs(argv);
  } catch (error) {
    process.stderr.write(`${sanitizeOutputText(error.message, 240)}\n`);
    return 2;
  }
  let report;
  try {
    if (args.command === 'query') report = buildQuery(args);
    else if (args.command === 'overlay') report = buildOverlay(args);
    else report = buildCheck(args);
  } catch (error) {
    report = baseReport(args);
    report.status = 'INCOMPLETE_SHADOW';
    report.warn.push('MAP_ADVISORY_UNAVAILABLE Code Map command failed safely');
    report.details = {
      findingCount: 1,
      findings: [{
        code: 'MAP_ADVISORY_UNAVAILABLE',
        message: 'Code Map command failed safely'
      }],
      truncated: false
    };
  }
  process.stdout.write(args.json ? `${JSON.stringify(report, null, 2)}\n` : renderText(report));
  return exitCode(report, args);
}

if (require.main === module) process.exitCode = main();

module.exports = {
  buildCheck,
  buildOverlay,
  buildQuery,
  exitCode,
  main,
  parseArgs,
  relevantChanges,
  renderText
};
