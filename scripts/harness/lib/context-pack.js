const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./project-kind');
const { loadHarnessConfig } = require('./config-loader');
const { memoryConfig } = require('./agentmemory-client');

const runtimeDocFiles = [
  'docs/current-state.md',
  'docs/sprint-contract.md',
  'docs/requirements-matrix.md',
  'docs/task-queue.md',
  'docs/decisions.md',
  'docs/open-questions.md',
  'docs/context-checkpoints.md',
  'docs/tooling-and-mcp-registry.md',
  'docs/agent-work-summary.md'
];

const maxSnippetLength = 700;
const maxTldrEntries = 16;
const maxSnippets = 24;

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function readIfExists(targetRoot, relativePath) {
  const filePath = path.join(targetRoot, relativePath);
  if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) return null;
  return fs.readFileSync(filePath, 'utf8');
}

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function normalizeToken(value) {
  return String(value || '').toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim();
}

function tokenSet(value) {
  return new Set(normalizeToken(value).split(/\s+/).filter((token) => token.length >= 2));
}

function tokenOverlapScore(left, right) {
  const leftTokens = tokenSet(left);
  const rightTokens = tokenSet(right);
  if (leftTokens.size === 0 || rightTokens.size === 0) return 0;
  let overlap = 0;
  for (const token of leftTokens) {
    if (rightTokens.has(token)) overlap += 1;
  }
  return overlap / Math.max(leftTokens.size, rightTokens.size);
}

function editDistance(left, right) {
  const a = normalizeToken(left).replace(/\s+/g, '');
  const b = normalizeToken(right).replace(/\s+/g, '');
  if (!a || !b) return Math.max(a.length, b.length);
  const previous = Array.from({ length: b.length + 1 }, (_, index) => index);
  for (let i = 1; i <= a.length; i += 1) {
    let last = i - 1;
    previous[0] = i;
    for (let j = 1; j <= b.length; j += 1) {
      const old = previous[j];
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      previous[j] = Math.min(previous[j] + 1, previous[j - 1] + 1, last + cost);
      last = old;
    }
  }
  return previous[b.length];
}

function fuzzyScore(left, right) {
  const a = normalizeToken(left);
  const b = normalizeToken(right);
  if (!a || !b) return 0;
  if (a === b) return 1;
  if (a.includes(b) || b.includes(a)) return 0.9;
  const overlap = tokenOverlapScore(a, b);
  const compactA = a.replace(/\s+/g, '');
  const compactB = b.replace(/\s+/g, '');
  const distance = editDistance(compactA, compactB);
  const editScore = 1 - (distance / Math.max(compactA.length, compactB.length, 1));
  return Math.max(overlap, editScore);
}

function evidenceFiles(targetRoot, slugs) {
  const evidenceRoot = path.join(targetRoot, 'docs', 'evidence');
  if (!fs.existsSync(evidenceRoot)) return [];

  const requestedSlugs = Array.isArray(slugs) ? slugs.filter(Boolean) : [];
  if (requestedSlugs.length > 0) {
    const files = [];
    const entries = fs.readdirSync(evidenceRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name);
    for (const evidenceSlug of entries) {
      const matched = requestedSlugs.some((slug) => fuzzyScore(evidenceSlug, slug) >= 0.75);
      if (!matched) continue;
      files.push(...walk(path.join(evidenceRoot, evidenceSlug))
        .filter((file) => fs.statSync(file).isFile())
        .map((file) => rel(targetRoot, file)));
    }
    return files;
  }

  return walk(evidenceRoot)
    .filter((file) => fs.statSync(file).isFile())
    .map((file) => rel(targetRoot, file));
}

function candidateFiles(targetRoot, slugs) {
  const seen = new Set();
  const files = [];
  for (const relativePath of runtimeDocFiles.concat(evidenceFiles(targetRoot, slugs))) {
    if (seen.has(relativePath)) continue;
    seen.add(relativePath);
    if (readIfExists(targetRoot, relativePath) !== null) files.push(relativePath);
  }
  return files;
}

function cleanLine(line) {
  return String(line || '').replace(/\s+/g, ' ').trim();
}

function compactText(text, limit = maxSnippetLength) {
  const compact = cleanLine(text);
  if (compact.length <= limit) return compact;
  return `${compact.slice(0, limit - 3).trim()}...`;
}

function headingLevel(line) {
  const match = line.match(/^(#{1,6})\s+/);
  return match ? match[1].length : null;
}

function headingText(line) {
  return line.replace(/^#{1,6}\s+/, '').trim();
}

function sectionAfterHeading(lines, startIndex, level) {
  const body = [];
  for (let i = startIndex + 1; i < lines.length; i += 1) {
    const nextLevel = headingLevel(lines[i]);
    if (nextLevel && nextLevel <= level) break;
    if (cleanLine(lines[i])) body.push(lines[i]);
    if (body.length >= 8) break;
  }
  return body.join('\n');
}

function findTldrSections(relativePath, text) {
  const lines = text.split(/\r?\n/);
  const entries = [];
  for (let i = 0; i < lines.length; i += 1) {
    const level = headingLevel(lines[i]);
    const title = level ? headingText(lines[i]) : '';
    if (level && /tl;?dr|summary|status|next safe task/i.test(title)) {
      const body = sectionAfterHeading(lines, i, level);
      if (body) {
        entries.push({
          file: relativePath,
          heading: title,
          text: compactText(body, 500)
        });
      }
    }
  }
  return entries;
}

function splitParagraphs(text) {
  return String(text || '')
    .split(/\n{2,}/)
    .map((part) => part.trim())
    .filter(Boolean);
}

function queryTerms(args) {
  const terms = [];
  if (args.taskId) terms.push({ value: args.taskId, matchType: 'task-id', source: 'taskId' });
  if (args.slug) terms.push({ value: args.slug, matchType: 'slug', source: 'slug' });
  for (const alias of args.aliases || []) {
    const trimmed = String(alias || '').trim();
    if (trimmed) terms.push({ value: trimmed, matchType: 'alias', source: 'alias' });
  }
  const seen = new Set();
  return terms.filter((term) => {
    const key = `${term.matchType}:${term.value}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function paragraphMatch(paragraph, term) {
  if (!term.value) return null;
  if (paragraph.includes(term.value)) {
    return {
      method: 'exact',
      score: 1
    };
  }
  const termCompact = normalizeToken(term.value).replace(/\s+/g, '');
  const paragraphCompact = normalizeToken(paragraph).replace(/\s+/g, '');
  const identifierLike = /[A-Z]+-[A-Z0-9-]+|[a-z0-9]+-[a-z0-9-]+/.test(term.value);
  if (!identifierLike || paragraphCompact.length > Math.max(termCompact.length * 5, 80)) {
    return null;
  }
  const score = fuzzyScore(paragraph, term.value);
  if (score >= 0.84) {
    return {
      method: 'fuzzy',
      score
    };
  }
  return null;
}

function matchingParagraphs(relativePath, text, term) {
  if (!term || !term.value) return [];
  return splitParagraphs(text)
    .map((paragraph) => ({ paragraph, match: paragraphMatch(paragraph, term) }))
    .filter((item) => item.match)
    .sort((left, right) => right.match.score - left.match.score)
    .slice(0, 4)
    .map((item) => ({
      matchType: term.matchType,
      value: term.value,
      method: item.match.method,
      score: Number(item.match.score.toFixed(3)),
      file: relativePath,
      text: compactText(item.paragraph)
    }));
}

function buildMarkdown(report) {
  const lines = [];
  lines.push(`# Context Pack`);
  lines.push('');
  lines.push(`- target: ${report.target}`);
  if (report.taskId) lines.push(`- taskId: ${report.taskId}`);
  if (report.slug) lines.push(`- slug: ${report.slug}`);
  if (report.aliases.length > 0) lines.push(`- aliases: ${report.aliases.join(', ')}`);
  lines.push(`- projectKind: ${report.details.projectKind}`);
  if (report.details.skipped) lines.push('- skipped: true');
  lines.push('');

  lines.push('## TL;DR');
  if (report.details.compact.tldr.length === 0) {
    lines.push('- None found.');
  } else {
    for (const entry of report.details.compact.tldr) {
      lines.push(`- ${entry.file} / ${entry.heading}: ${entry.text}`);
    }
  }

  lines.push('');
  lines.push('## Matching Snippets');
  if (report.details.snippets.length === 0) {
    lines.push('- None found.');
  } else {
    for (const snippet of report.details.snippets) {
      lines.push(`- ${snippet.matchType} ${snippet.value} in ${snippet.file} (${snippet.method}; score ${snippet.score}): ${snippet.text}`);
    }
  }

  lines.push('');
  lines.push('## Governed Memory Candidates');
  if (!report.details.memory || report.details.memory.governed.length === 0) {
    lines.push('- None included.');
  } else {
    for (const entry of report.details.memory.governed) {
      lines.push(`- ${entry.candidate.id} (${entry.candidate.status}; ${entry.candidate.authority}): ${entry.candidate.claim}`);
    }
  }

  if (report.warn.length > 0) {
    lines.push('');
    lines.push('## Warnings');
    for (const item of report.warn) lines.push(`- ${item}`);
  }

  if (report.fail.length > 0) {
    lines.push('');
    lines.push('## Failures');
    for (const item of report.fail) lines.push(`- ${item}`);
  }

  return `${lines.join('\n')}\n`;
}

function buildReport(options) {
  const args = {
    target: path.resolve(options.target || process.cwd()),
    taskId: options.taskId ? String(options.taskId).trim() : null,
    slug: options.slug ? String(options.slug).trim() : null,
    aliases: Array.isArray(options.aliases) ? options.aliases.map((alias) => String(alias || '').trim()).filter(Boolean) : []
  };
  const report = {
    target: args.target,
    taskId: args.taskId,
    slug: args.slug,
    aliases: args.aliases,
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      skipped: false,
      filesRead: [],
      compact: {
        tldr: []
      },
      snippets: [],
      memory: {
        enabled: false,
        skippedReason: null,
        governed: [],
        warnings: []
      }
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    report.markdown = buildMarkdown(report);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) {
    report.details.skipped = true;
    add(report, 'warn', 'Source package detected; context-pack reads installed runtime docs only, so this target is an informational skip');
    report.markdown = buildMarkdown(report);
    return report;
  }

  if (!kind.isInstalledProject) {
    add(report, 'warn', 'Target is not recognized as a harness installed project; scanning runtime doc paths if present');
  } else {
    add(report, 'pass', 'Installed-project target detected');
  }

  const terms = queryTerms(args);
  const loadedConfig = loadHarnessConfig(args.target);
  if (loadedConfig.data) {
    const resolvedMemory = memoryConfig(loadedConfig.data);
    report.details.memory.enabled = resolvedMemory.enabled;
    if (!resolvedMemory.enabled) report.details.memory.skippedReason = 'memoryIntegration.enabled is false';
    else report.details.memory.skippedReason = 'agentmemory live query is handled by task-start wrapper in this release';
  } else if (loadedConfig.error) {
    report.details.memory.skippedReason = `config load failed: ${loadedConfig.error}`;
    report.details.memory.warnings.push(report.details.memory.skippedReason);
  } else {
    report.details.memory.skippedReason = 'no harness config found';
  }
  if (report.details.memory.skippedReason) add(report, 'warn', `memory: ${report.details.memory.skippedReason}`);

  const slugTerms = terms.filter((term) => term.matchType === 'slug' || /slug|task/i.test(term.value)).map((term) => term.value);
  const files = candidateFiles(args.target, slugTerms);
  report.details.filesRead = files;
  if (files.length === 0) {
    add(report, 'warn', 'No runtime docs or matching evidence files found');
    report.markdown = buildMarkdown(report);
    return report;
  }

  for (const relativePath of files) {
    const text = readIfExists(args.target, relativePath);
    if (text === null) continue;
    report.details.compact.tldr.push(...findTldrSections(relativePath, text));
    for (const term of terms) {
      report.details.snippets.push(...matchingParagraphs(relativePath, text, term));
    }
  }

  report.details.compact.tldr = report.details.compact.tldr.slice(0, maxTldrEntries);
  report.details.snippets = report.details.snippets.slice(0, maxSnippets);

  if (report.details.compact.tldr.length > 0) add(report, 'pass', `TL;DR section(s) found: ${report.details.compact.tldr.length}`);
  else add(report, 'warn', 'No TL;DR-style sections found');

  if (args.taskId || args.slug) {
    if (report.details.snippets.length > 0) add(report, 'pass', `Matching snippet(s) found: ${report.details.snippets.length}`);
    else add(report, 'warn', 'No task-id or slug matching snippets found');
  } else {
    add(report, 'warn', 'No --task-id or --slug provided; matching snippets were not requested');
  }

  report.markdown = buildMarkdown(report);
  return report;
}

module.exports = {
  buildReport,
  buildMarkdown
};
