#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectPromptInjection, redactSecrets } = require('./lib/security');
const { appendSecretEscapeHatchAudit, normalizeReason } = require('./lib/evidence-audit');

function parseArgs(argv) {
  const args = {
    target: null,
    targetProvided: false,
    slug: null,
    command: null,
    result: null,
    notes: null,
    output: null,
    append: false,
    allowRedactedSecrets: false,
    redactionReason: null,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') {
      args.target = argv[++i];
      args.targetProvided = true;
    } else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--command') args.command = argv[++i];
    else if (arg === '--result') args.result = argv[++i];
    else if (arg === '--notes') args.notes = argv[++i];
    else if (arg === '--output') args.output = argv[++i];
    else if (arg === '--append') args.append = true;
    else if (arg === '--allow-redacted-secrets') args.allowRedactedSecrets = true;
    else if (arg === '--redaction-reason') args.redactionReason = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.write && !args.targetProvided) {
    throw new Error('--write requires an explicit --target installed-project directory');
  }

  args.target = path.resolve(args.target || process.cwd());
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function safeSlug(value) {
  const slug = String(value || '').trim();
  if (!slug) throw new Error('--slug is required');
  if (/[\\/]/.test(slug) || slug.includes('..')) {
    throw new Error('Slug must be a single safe path segment without traversal');
  }
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(slug)) {
    throw new Error('Slug may contain only letters, numbers, dots, underscores, and hyphens');
  }
  return slug;
}

function valueOrTbd(report, value, label) {
  if (typeof value === 'string' && value.trim()) return value.trim();
  add(report, 'warn', `${label} omitted; using TBD`);
  return 'TBD';
}

function fence(value) {
  return String(value || '').replace(/```/g, '`` `');
}

function appendSeparator(text) {
  if (!text) return '';
  if (text.endsWith('\n\n')) return '';
  if (text.endsWith('\n')) return '\n';
  return '\n\n';
}

function buildEntry(report, args, now) {
  const command = valueOrTbd(report, args.command, 'Command');
  const result = valueOrTbd(report, args.result, 'Result');
  const notes = valueOrTbd(report, args.notes, 'Notes');
  const output = typeof args.output === 'string' && args.output.trim()
    ? args.output.trim()
    : 'No command output was provided.';

  const entry = `## Command Evidence

- recordedAt: ${now.toISOString()}
- command: \`${command.replace(/`/g, '\\`')}\`
- result: ${result}
- notes: ${notes}

### Output

\`\`\`text
${fence(output)}
\`\`\`
`;
  const redaction = redactSecrets(entry);
  const injection = detectPromptInjection(entry);
  report.details.redaction = {
    redacted: redaction.redacted,
    findings: redaction.findings
  };
  report.details.promptInjection = {
    detected: injection.detected,
    findings: injection.findings
  };
  if (redaction.redacted) add(report, 'warn', 'Secret-like content was redacted from command evidence');
  if (injection.detected) add(report, 'warn', 'Prompt-injection-like content was recorded as untrusted evidence, not instructions');
  return redaction.text;
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    wrote: false,
    pass: [],
    warn: [],
    fail: [],
    details: {
      slug: null,
      evidenceDir: null,
      file: null,
      relativeFile: null,
      append: args.append,
      entry: null,
      redaction: null,
      promptInjection: null,
      redactionOverride: {
        allowed: args.allowRedactedSecrets,
        reason: normalizeReason(args.redactionReason),
        audit: null
      }
    }
  };

  const slug = safeSlug(args.slug);
  const evidenceDir = path.join(args.target, 'docs', 'evidence', slug);
  const file = path.join(evidenceDir, 'test-output.md');
  const relativeFile = rel(args.target, file);

  report.details.slug = slug;
  report.details.evidenceDir = evidenceDir;
  report.details.file = file;
  report.details.relativeFile = relativeFile;

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing installed-project directory: ${args.target}`);
    return report;
  }

  if (!fs.existsSync(evidenceDir) || !fs.statSync(evidenceDir).isDirectory()) {
    add(report, 'fail', `Evidence directory must already exist: ${rel(args.target, evidenceDir)}`);
    return report;
  }

  const expectedRelative = `docs/evidence/${slug}/test-output.md`;
  if (relativeFile !== expectedRelative) {
    add(report, 'fail', `Refusing to write outside ${expectedRelative}: ${relativeFile}`);
    return report;
  }

  const exists = fs.existsSync(file);
  if (exists && !args.append) {
    add(report, 'fail', `Evidence command output already exists; use --append to add a new record: ${relativeFile}`);
    return report;
  }

  const entry = buildEntry(report, args, new Date());
  report.details.entry = entry;

  if (report.details.redaction && report.details.redaction.redacted && args.write && !args.allowRedactedSecrets) {
    add(report, 'fail', 'Secret-like content detected in command evidence; refusing to write without --allow-redacted-secrets');
    return report;
  }
  if (report.details.redaction && report.details.redaction.redacted && args.write && args.allowRedactedSecrets && !normalizeReason(args.redactionReason)) {
    add(report, 'fail', 'Secret-like content detected in command evidence; --allow-redacted-secrets requires --redaction-reason');
    return report;
  }

  if (!args.write) {
    add(report, 'pass', 'Dry run only; no files modified');
    return report;
  }

  if (exists) {
    const current = fs.readFileSync(file, 'utf8');
    fs.appendFileSync(file, `${appendSeparator(current)}${entry}`, 'utf8');
    add(report, 'pass', `Appended command evidence to ${relativeFile}`);
  } else {
    fs.writeFileSync(file, `# Test Output: ${slug}\n\n${entry}`, { encoding: 'utf8', flag: 'wx' });
    add(report, 'pass', `Created command evidence: ${relativeFile}`);
  }

  report.wrote = true;
  if (report.details.redaction && report.details.redaction.redacted && args.allowRedactedSecrets) {
    const audit = appendSecretEscapeHatchAudit({
      target: args.target,
      source: 'evidence-command',
      slug,
      evidenceFile: relativeFile,
      command: args.command || 'TBD',
      reason: args.redactionReason,
      findings: report.details.redaction.findings,
      summary: 'evidence-command redacted secret-like content and wrote evidence after explicit override.'
    });
    report.details.redactionOverride.audit = audit;
    if (audit.wrote) add(report, 'warn', `Secret evidence escape hatch audited in ${audit.relativeLedgerPath}`);
    else if (audit.warning) add(report, 'warn', audit.warning);
  }
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
  console.log(`Harness evidence command recorder: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  console.log(`File: ${report.details.relativeFile || 'N/A'}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  if (report.details.entry) {
    console.log('\nINTENDED RECORD');
    console.log(report.details.entry);
  }
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
