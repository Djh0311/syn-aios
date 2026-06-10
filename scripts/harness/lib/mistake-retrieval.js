const fs = require('fs');
const path = require('path');

const runtimeLedgerPath = path.join('docs', 'agent-mistake-ledger.md');

const riskTagPatterns = [
  ['auth', /\bauth(?:entication|orization)?\b|\blogin\b|\btoken\b/],
  ['security', /\bsecurity\b|\bsecret\b|\bpermission\b|\bpolicy\b/],
  ['database', /\bdb\b|\bdatabase\b|\bschema\b|\bmigration\b|\bprisma\b/],
  ['api', /\bapi\b|\bcontract\b|\bendpoint\b|\bdto\b/],
  ['ui', /\bui\b|\bbrowser\b|\bfrontend\b|\blayout\b|\bviewport\b/],
  ['verification', /\btest\b|\bverification\b|\bevidence\b|\bpassing\b|\bcompletion\b/],
  ['git', /\bgit\b|\bcommit\b|\bbranch\b|\bworktree\b/],
  ['docs', /\bdocs?\b|\bruntime docs\b|\btemplates?\b/],
  ['context', /\bcontext\b|\bcheckpoint\b|\brecovery\b/],
  ['harness', /\bharness\b|\bfixture\b|\bself-test\b/],
  ['mistake-ledger', /\bmistake\b|\bledger\b|\bprevention\b/]
];

function normalizeText(value) {
  return String(value || '').toLowerCase();
}

function unique(values) {
  return [...new Set(values.filter(Boolean))];
}

function tokenize(value) {
  return unique(normalizeText(value)
    .split(/[^a-z0-9._/-]+/)
    .map((token) => token.trim())
    .filter((token) => token.length >= 3 && !/^(the|and|for|with|that|this|from|into|task|tbd|fix|bug|issue|error|failure|regression|change|update|follow|follow-up)$/.test(token)));
}

function splitCsv(value) {
  return unique(String(value || '')
    .split(/[,;]/)
    .map((item) => item.trim())
    .filter(Boolean));
}

function detectRiskTags(value) {
  const text = normalizeText(value);
  return riskTagPatterns
    .filter((item) => item[1].test(text))
    .map((item) => item[0]);
}

function parseMetadataLine(line) {
  const match = line.match(/^\s*([A-Za-z][A-Za-z /-]*?)\s*:\s*(.*?)\s*$/);
  if (!match) return null;
  return {
    key: match[1].trim().toLowerCase().replace(/\s+/g, '-'),
    value: match[2].trim()
  };
}

function parseLedger(text) {
  const entries = [];
  const lines = String(text || '').split(/\r?\n/);
  let current = null;
  let currentSection = null;
  let inFence = false;

  function finish(endLine) {
    if (!current) return;
    current.endLine = endLine;
    current.raw = current.lines.join('\n');
    current.searchText = [
      current.id,
      current.title,
      current.status,
      current.kind,
      current.raw,
      current.signatureKeywords.join(' '),
      current.signaturePaths.join(' '),
      current.riskTags.join(' ')
    ].join('\n');
    current.derivedRiskTags = unique(current.riskTags.concat(detectRiskTags(current.searchText)));
    entries.push(current);
    current = null;
    currentSection = null;
  }

  lines.forEach((line, index) => {
    const lineNumber = index + 1;
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      return;
    }
    if (inFence) return;

    const entryMatch = line.match(/^##\s+(M-\d+)\s*:?\s*(.*?)\s*$/i);
    if (entryMatch) {
      finish(lineNumber - 1);
      current = {
        id: entryMatch[1],
        title: entryMatch[2].trim(),
        line: lineNumber,
        endLine: null,
        status: null,
        kind: null,
        sections: {},
        metadata: {},
        signatureKeywords: [],
        signaturePaths: [],
        riskTags: [],
        lines: []
      };
      return;
    }

    if (!current) return;
    current.lines.push(line);

    const sectionMatch = line.match(/^###\s+(.+?)\s*#*\s*$/);
    if (sectionMatch) {
      currentSection = sectionMatch[1].trim();
      if (!current.sections[currentSection]) current.sections[currentSection] = [];
      return;
    }

    if (currentSection) current.sections[currentSection].push(line);

    const metadata = parseMetadataLine(line);
    if (!metadata) return;
    current.metadata[metadata.key] = metadata.value;
    if (metadata.key === 'status') current.status = metadata.value;
    if (metadata.key === 'affected-area') current.kind = metadata.value;
    if (metadata.key === 'signature-keywords') current.signatureKeywords = splitCsv(metadata.value);
    if (metadata.key === 'signature-paths') current.signaturePaths = splitCsv(metadata.value);
    if (metadata.key === 'risk-tags') current.riskTags = splitCsv(metadata.value).map((tag) => normalizeText(tag));
  });

  finish(lines.length);
  return entries;
}

function loadLedger(targetRoot) {
  const ledgerPath = path.join(targetRoot, runtimeLedgerPath);
  if (!fs.existsSync(ledgerPath)) {
    return {
      ledgerPath,
      exists: false,
      entries: [],
      error: null
    };
  }

  try {
    const text = fs.readFileSync(ledgerPath, 'utf8');
    return {
      ledgerPath,
      exists: true,
      entries: parseLedger(text),
      error: null
    };
  } catch (error) {
    return {
      ledgerPath,
      exists: true,
      entries: [],
      error: error.message
    };
  }
}

function normalizeQuery(query) {
  const paths = Array.isArray(query.paths)
    ? query.paths
    : splitCsv(query.path || '');
  const text = [query.title, query.description].filter(Boolean).join(' ');
  const explicitRiskTags = Array.isArray(query.riskTags)
    ? query.riskTags
    : splitCsv(query.riskTag || query.riskTags || '');
  const riskTags = unique(explicitRiskTags.map((tag) => normalizeText(tag)).concat(detectRiskTags(text)));

  return {
    title: query.title || '',
    description: query.description || '',
    paths: unique(paths.map((item) => item.split(path.sep).join('/'))),
    riskTags,
    titleTokens: tokenize(query.title || ''),
    descriptionTokens: tokenize(query.description || ''),
    allTokens: tokenize(text)
  };
}

function pathScore(queryPath, entryPath) {
  const left = normalizeText(queryPath).split(path.sep).join('/');
  const right = normalizeText(entryPath).split(path.sep).join('/');
  if (!left || !right) return 0;
  if (left === right) return 12;
  if (left.includes(right) || right.includes(left)) return 8;

  const leftParts = left.split('/').filter(Boolean);
  const rightParts = right.split('/').filter(Boolean);
  const overlap = leftParts.filter((part) => rightParts.includes(part)).length;
  return overlap >= 2 ? overlap * 2 : 0;
}

function scoreEntry(entry, query) {
  const reasons = [];
  let score = 0;
  const entryText = normalizeText(entry.searchText);
  const titleText = normalizeText(entry.title);

  for (const token of query.titleTokens) {
    if (titleText.includes(token)) {
      score += 5;
      reasons.push(`title:${token}`);
    } else if (entryText.includes(token)) {
      score += 3;
      reasons.push(`keyword:${token}`);
    }
  }

  for (const token of query.descriptionTokens) {
    if (entryText.includes(token)) {
      score += 1;
      reasons.push(`description:${token}`);
    }
  }

  for (const queryPath of query.paths) {
    let best = 0;
    for (const entryPath of entry.signaturePaths) {
      best = Math.max(best, pathScore(queryPath, entryPath));
    }
    if (best === 0 && entryText.includes(normalizeText(queryPath))) best = 6;
    if (best > 0) {
      score += best;
      reasons.push(`path:${queryPath}`);
    }
  }

  for (const tag of query.riskTags) {
    if (entry.derivedRiskTags.includes(tag)) {
      score += 5;
      reasons.push(`risk:${tag}`);
    }
  }

  return {
    score,
    reasons: unique(reasons)
  };
}

function queryMistakes(targetRoot, query, options = {}) {
  const limit = Number.isFinite(options.limit) ? options.limit : 5;
  const minimumScore = Number.isFinite(options.minimumScore) ? options.minimumScore : 1;
  const ledger = loadLedger(targetRoot);
  const normalized = normalizeQuery(query || {});
  const matches = ledger.entries
    .map((entry) => {
      const scored = scoreEntry(entry, normalized);
      return {
        id: entry.id,
        title: entry.title,
        status: entry.status,
        kind: entry.kind,
        line: entry.line,
        score: scored.score,
        reasons: scored.reasons,
        riskTags: entry.derivedRiskTags,
        signaturePaths: entry.signaturePaths
      };
    })
    .filter((entry) => entry.score >= minimumScore)
    .sort((left, right) => right.score - left.score || left.id.localeCompare(right.id))
    .slice(0, limit);

  return {
    ledgerPath: ledger.ledgerPath,
    ledgerExists: ledger.exists,
    ledgerError: ledger.error,
    query: normalized,
    matches
  };
}

module.exports = {
  runtimeLedgerPath,
  parseLedger,
  queryMistakes,
  detectRiskTags,
  tokenize
};
