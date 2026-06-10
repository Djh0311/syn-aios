#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const {
  memoryCandidateFiles,
  normalizeMemoryCandidate,
  renderMemoryCandidateMarkdown,
  validateMemoryCandidate
} = require('./lib/memory-governance');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    approve: null,
    quarantine: null,
    revoke: null,
    reason: '',
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--approve') args.approve = argv[++i];
    else if (arg === '--quarantine') args.quarantine = argv[++i];
    else if (arg === '--revoke') args.revoke = argv[++i];
    else if (arg === '--reason') args.reason = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function add(report, kind, message) {
  report[kind].push(message);
}

function candidatesDir(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory', 'candidates');
}

function auditPath(targetRoot) {
  return path.join(targetRoot, '.harness', 'memory', 'audit.jsonl');
}

function listCandidateFiles(targetRoot) {
  const dir = candidatesDir(targetRoot);
  if (!fs.existsSync(dir)) return [];
  return fs.readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .map((name) => path.join(dir, name))
    .sort();
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
  }
}

function requestedAction(args) {
  const actions = [
    ['approved', args.approve],
    ['quarantined', args.quarantine],
    ['revoked', args.revoke]
  ].filter(([, id]) => id);
  if (actions.length > 1) throw new Error('Use only one of --approve, --quarantine, or --revoke');
  if (actions.length === 0) return null;
  return { status: actions[0][0], id: actions[0][1] };
}

function writeCandidate(targetRoot, candidate) {
  const files = memoryCandidateFiles(targetRoot, candidate.id);
  fs.mkdirSync(files.dir, { recursive: true });
  fs.writeFileSync(files.json, `${JSON.stringify(candidate, null, 2)}\n`, 'utf8');
  fs.writeFileSync(files.markdown, renderMemoryCandidateMarkdown(candidate), 'utf8');
}

function appendAudit(targetRoot, entry) {
  const filePath = auditPath(targetRoot);
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.appendFileSync(filePath, `${JSON.stringify(entry)}\n`, 'utf8');
}

function buildReport(args) {
  const action = requestedAction(args);
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      counts: {},
      candidates: [],
      action
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }
  if (action && !String(args.reason || '').trim()) {
    add(report, 'fail', '--reason is required for memory review actions');
    return report;
  }
  if (action && !args.write) {
    add(report, 'warn', 'Dry run only; review action was not written');
  }

  let files = listCandidateFiles(args.target);
  let candidates = [];
  function loadCandidates() {
    candidates = [];
    report.details.counts = {};
    report.details.candidates = [];
    for (const filePath of files) {
      const parsed = readJson(filePath);
      if (parsed.error) {
        add(report, 'fail', `${rel(args.target, filePath)} could not be parsed: ${parsed.error}`);
        continue;
      }
      const candidate = normalizeMemoryCandidate(parsed.data);
      const validation = validateMemoryCandidate(candidate, { projectContext: { targetRoot: args.target } });
      candidates.push({ filePath, candidate, validation });
      report.details.candidates.push({
        file: rel(args.target, filePath),
        id: candidate.id,
        status: candidate.status,
        valid: validation.valid
      });
      report.details.counts[candidate.status] = (report.details.counts[candidate.status] || 0) + 1;
    }
  }

  loadCandidates();
  if (files.length === 0) add(report, 'pass', 'No memory candidates found');
  else add(report, 'pass', `Memory candidates found: ${files.length}`);

  if (!action) return report;

  const item = candidates.find((entry) => entry.candidate.id === action.id);
  if (!item) {
    add(report, 'fail', `Memory candidate not found: ${action.id}`);
    return report;
  }
  const next = Object.assign({}, item.candidate, {
    status: action.status,
    reviewReason: String(args.reason || '').trim(),
    lastVerifiedAt: action.status === 'approved' ? new Date().toISOString() : item.candidate.lastVerifiedAt
  });
  const validation = validateMemoryCandidate(next, { projectContext: { targetRoot: args.target } });
  if (!validation.valid) {
    for (const error of validation.errors) add(report, 'fail', `${next.id}: ${error}`);
    return report;
  }
  if (action.status === 'approved' && validation.classification.recommendedStatus !== 'approved') {
    add(report, 'fail', `${next.id}: cannot approve memory while governance recommends ${validation.classification.recommendedStatus}`);
    for (const reason of validation.classification.reasons) add(report, 'fail', `${next.id}: ${reason}`);
    return report;
  }

  if (args.write) {
    writeCandidate(args.target, validation.normalized);
    appendAudit(args.target, {
      at: new Date().toISOString(),
      id: next.id,
      action: action.status,
      reason: String(args.reason || '').trim()
    });
    files = listCandidateFiles(args.target);
    loadCandidates();
    add(report, 'pass', `Updated memory candidate ${next.id} to ${action.status}`);
    report.details.audit = rel(args.target, auditPath(args.target));
  } else {
    add(report, 'pass', `Would update memory candidate ${next.id} to ${action.status}`);
  }

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness memory review: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
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
