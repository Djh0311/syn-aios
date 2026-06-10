#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

const sourceLedgerPath = 'templates/docs/agent-mistake-ledger.md';
const runtimeLedgerPath = 'docs/agent-mistake-ledger.md';

const requiredEntrySections = [
  'Prevention',
  'Actual Root Cause',
  'Detection Evidence'
];

const preventionLikeSections = [
  'Prevention',
  'Regression Protection',
  'Evidence',
  'Detection Evidence'
];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    config: null,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--config') args.config = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (!arg.startsWith('--')) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (args.config) args.config = path.resolve(args.config);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function statSafe(filePath) {
  try {
    return fs.statSync(filePath);
  } catch (error) {
    return null;
  }
}

function rel(root, filePath) {
  return path.relative(root, filePath) || '.';
}

function readJson(filePath) {
  try {
    return {
      data: JSON.parse(fs.readFileSync(filePath, 'utf8')),
      error: null
    };
  } catch (error) {
    return {
      data: null,
      error
    };
  }
}

function loadConfig(args) {
  const candidates = args.config
    ? [args.config]
    : [
        path.join(args.target, 'harness.config.json'),
        path.join(args.target, 'harness.config.example.json')
      ];

  for (const candidate of candidates) {
    if (!candidate) continue;
    const full = path.resolve(candidate);
    if (fs.existsSync(full)) {
      const parsed = readJson(full);
      return {
        path: full,
        data: parsed.data,
        error: parsed.error ? parsed.error.message : null
      };
    }
  }

  return {
    path: args.config || null,
    data: null,
    error: args.config ? 'Config file was not found' : null
  };
}

function detectSourcePackage(targetRoot) {
  return detectProjectKind(targetRoot).isSourcePackage;
}

function sectionHeading(line) {
  const match = line.match(/^(#{2,6})\s+(.+?)\s*#*\s*$/);
  if (!match) return null;
  return {
    level: match[1].length,
    title: match[2].trim()
  };
}

function isMeaningfulSectionBody(lines) {
  return lines.some((line) => {
    const text = line.trim();
    if (!text) return false;
    if (/^-+\s*$/.test(text)) return false;
    if (/^[-*]\s*(none|n\/a|tbd|todo|what|command|test\/check|evidence location|skill\/rule\/checklist|new guardrail)\b/i.test(text)) return false;
    return true;
  });
}

function normalizeTitle(title) {
  return title
    .replace(/^M-\d+\s*:?\s*/i, '')
    .trim()
    .toLowerCase();
}

function parseLedger(text) {
  const lines = text.split(/\r?\n/);
  const result = {
    hasActiveMistakesSection: false,
    hasClosedMistakesSection: false,
    activeMistakesHasItems: false,
    activeMistakeLines: [],
    entries: [],
    duplicateTitles: []
  };

  let currentTopSection = null;
  let currentEntry = null;
  let currentSubsection = null;
  let inFence = false;

  function finishEntry(endLine) {
    if (!currentEntry) return;
    if (currentSubsection) {
      currentEntry.sections[currentSubsection].body = currentEntry.sections[currentSubsection].body.concat([]);
    }
    currentEntry.endLine = endLine;
    result.entries.push(currentEntry);
    currentEntry = null;
    currentSubsection = null;
  }

  lines.forEach((line, index) => {
    const lineNumber = index + 1;
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      return;
    }

    if (inFence) return;

    const heading = sectionHeading(line);

    if (heading) {
      if (heading.level === 2) {
        finishEntry(lineNumber - 1);
        currentSubsection = null;

        if (/^Active Mistakes$/i.test(heading.title)) {
          currentTopSection = 'active';
          result.hasActiveMistakesSection = true;
          return;
        }

        if (/^Closed Mistakes$/i.test(heading.title)) {
          currentTopSection = 'closed';
          result.hasClosedMistakesSection = true;
          return;
        }

        const entryMatch = heading.title.match(/^(M-\d+)\s*:?\s*(.*)$/i);
        if (entryMatch) {
          currentTopSection = null;
          currentEntry = {
            id: entryMatch[1],
            title: entryMatch[2].trim() || heading.title.trim(),
            line: lineNumber,
            endLine: null,
            status: null,
            sections: {}
          };
          return;
        }

        currentTopSection = null;
        return;
      }

      if (currentEntry && heading.level === 3) {
        currentSubsection = heading.title.trim();
        if (!currentEntry.sections[currentSubsection]) {
          currentEntry.sections[currentSubsection] = {
            line: lineNumber,
            body: []
          };
        }
        return;
      }
    }

    if (currentEntry) {
      const statusMatch = line.match(/^\s*Status\s*:\s*(.+?)\s*$/i);
      if (statusMatch) currentEntry.status = statusMatch[1].trim();
      if (currentSubsection && currentEntry.sections[currentSubsection]) {
        currentEntry.sections[currentSubsection].body.push(line);
      }
      return;
    }

    if (currentTopSection === 'active') {
      const text = line.trim();
      if (/^[-*]\s+(.+)/.test(text) && !/^[-*]\s+None\b/i.test(text)) {
        result.activeMistakesHasItems = true;
        result.activeMistakeLines.push({
          line: lineNumber,
          text
        });
      }
    }
  });

  finishEntry(lines.length);

  const titles = new Map();
  for (const entry of result.entries) {
    const key = normalizeTitle(entry.title);
    if (!key) continue;
    if (!titles.has(key)) titles.set(key, []);
    titles.get(key).push(entry);
  }

  result.duplicateTitles = [...titles.entries()]
    .filter((item) => item[1].length > 1)
    .map((item) => ({
      title: item[0],
      entries: item[1].map((entry) => ({
        id: entry.id,
        line: entry.line
      }))
    }));

  return result;
}

function sectionPresent(entry, sectionName) {
  return Object.keys(entry.sections).some((name) => name.toLowerCase() === sectionName.toLowerCase());
}

function sectionHasBody(entry, sectionName) {
  const key = Object.keys(entry.sections).find((name) => name.toLowerCase() === sectionName.toLowerCase());
  if (!key) return false;
  return isMeaningfulSectionBody(entry.sections[key].body);
}

function configRequiresMistakeLedger(configData) {
  const hits = [];
  const gates = configData && configData.gates ? configData.gates : null;

  function scanValue(value, location) {
    if (typeof value === 'string' && /mistake|ledger|agent-mistake-ledger/i.test(value)) {
      hits.push({
        location,
        value
      });
    } else if (Array.isArray(value)) {
      value.forEach((item, index) => scanValue(item, `${location}[${index}]`));
    } else if (value && typeof value === 'object') {
      Object.keys(value).forEach((key) => scanValue(value[key], `${location}.${key}`));
    }
  }

  if (gates && Array.isArray(gates.hard)) scanValue(gates.hard, 'gates.hard');
  if (gates && Array.isArray(gates.soft)) scanValue(gates.soft, 'gates.soft');
  if (gates && gates.rules) scanValue(gates.rules, 'gates.rules');
  if (configData && configData.rules) scanValue(configData.rules, 'rules');

  return hits;
}

function checkLedgerStructure(report, ledgerRelativePath, ledgerText) {
  const parsed = parseLedger(ledgerText);
  report.details.ledger = Object.assign({}, report.details.ledger, {
    path: ledgerRelativePath,
    hasActiveMistakesSection: parsed.hasActiveMistakesSection,
    hasClosedMistakesSection: parsed.hasClosedMistakesSection,
    activeMistakeLines: parsed.activeMistakeLines,
    entries: parsed.entries.map((entry) => ({
      id: entry.id,
      title: entry.title,
      line: entry.line,
      status: entry.status,
      sections: Object.keys(entry.sections)
    })),
    duplicateTitles: parsed.duplicateTitles
  });

  if (parsed.hasActiveMistakesSection) add(report, 'pass', 'Ledger contains Active Mistakes section');
  else add(report, 'warn', 'Ledger is missing Active Mistakes section');

  if (parsed.hasClosedMistakesSection) add(report, 'pass', 'Ledger contains Closed Mistakes section');
  else add(report, 'warn', 'Ledger is missing Closed Mistakes section');

  const preventionSections = preventionLikeSections.filter((section) => parsed.entries.some((entry) => sectionPresent(entry, section)));
  report.details.ledger.preventionLikeSectionsFound = preventionSections;
  if (preventionSections.length > 0) {
    add(report, 'pass', `Ledger entries include prevention/evidence sections: ${preventionSections.join(', ')}`);
  } else if (parsed.entries.length > 0) {
    add(report, 'warn', 'Ledger entries do not include Prevention/Regression Protection/Evidence sections');
  }

  if (parsed.activeMistakesHasItems) add(report, 'warn', `Active Mistakes has open item(s): ${parsed.activeMistakeLines.length}`);
  else add(report, 'pass', 'No listed Active Mistakes found');

  if (parsed.entries.length === 0) {
    add(report, 'pass', 'No ## M- entries found to validate');
  } else {
    add(report, 'pass', `Ledger entries found: ${parsed.entries.length}`);
  }

  for (const entry of parsed.entries) {
    if (!entry.status) add(report, 'warn', `${entry.id} is missing Status line`);

    for (const section of requiredEntrySections) {
      if (!sectionPresent(entry, section)) {
        add(report, 'warn', `${entry.id} is missing ${section} section`);
      } else if (!sectionHasBody(entry, section)) {
        add(report, 'warn', `${entry.id} has placeholder or empty ${section} section`);
      }
    }
  }

  for (const duplicate of parsed.duplicateTitles) {
    const where = duplicate.entries.map((entry) => `${entry.id} line ${entry.line}`).join(', ');
    add(report, 'warn', `Duplicate mistake title "${duplicate.title}" in ${where}`);
  }
}

function buildReport(args) {
  const report = {
    target: args.target,
    pass: [],
    warn: [],
    fail: [],
    details: {}
  };

  if (!fs.existsSync(args.target)) {
    add(report, 'fail', `Target does not exist: ${args.target}`);
    return report;
  }

  const targetStat = statSafe(args.target);
  if (!targetStat || !targetStat.isDirectory()) {
    add(report, 'fail', `Target is not a directory: ${args.target}`);
    return report;
  }

  const config = loadConfig(args);
  report.details.configPath = config.path;

  if (config.path && config.data) add(report, 'pass', `Harness config readable: ${rel(args.target, config.path)}`);
  else if (config.path) add(report, 'fail', `Harness config could not be loaded: ${config.path}${config.error ? ` (${config.error})` : ''}`);
  else add(report, 'warn', 'No harness.config.json or harness.config.example.json found in target');

  const gateHits = configRequiresMistakeLedger(config.data);
  report.details.completionGateInput = {
    requiredByConfig: gateHits.length > 0,
    matches: gateHits
  };

  if (gateHits.length > 0) {
    add(report, 'pass', 'Mistake ledger is a configured completion gate input');
  }

  const isSourcePackage = detectSourcePackage(args.target);
  report.details.isSourcePackage = isSourcePackage;

  if (isSourcePackage) {
    if (exists(args.target, sourceLedgerPath)) {
      add(report, 'pass', `Source package ledger template exists: ${sourceLedgerPath}`);
      const text = fs.readFileSync(path.join(args.target, sourceLedgerPath), 'utf8');
      checkLedgerStructure(report, sourceLedgerPath, text);
    } else {
      add(report, 'fail', `Source package ledger template is missing: ${sourceLedgerPath}`);
    }

    if (exists(args.target, runtimeLedgerPath)) {
      add(report, 'warn', `Source package has root runtime ledger; source packages should keep runtime state out of docs/**: ${runtimeLedgerPath}`);
    } else {
      add(report, 'pass', `Source package has no root runtime ledger: ${runtimeLedgerPath}`);
    }

    return report;
  }

  if (!exists(args.target, runtimeLedgerPath)) {
    add(report, args.strict ? 'fail' : 'warn', `Installed-project mistake ledger missing: ${runtimeLedgerPath}`);
    return report;
  }

  add(report, 'pass', `Installed-project mistake ledger exists: ${runtimeLedgerPath}`);
  const text = fs.readFileSync(path.join(args.target, runtimeLedgerPath), 'utf8');
  checkLedgerStructure(report, runtimeLedgerPath, text);

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness mistake check: ${report.target}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  console.log('\nDETAILS');
  console.log(JSON.stringify(report.details, null, 2));
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
