#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    maxLines: 12,
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--max-lines') args.maxLines = Number(argv[++i]);
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  if (!args.slug) throw new Error('--slug is required');
  if (/[\\/]/.test(args.slug) || args.slug.includes('..')) throw new Error('--slug must be a safe single path segment');
  if (!Number.isFinite(args.maxLines) || args.maxLines < 4) throw new Error('--max-lines must be at least 4');
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
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

function isProtected(text) {
  return /\bstrict\b|strict path|\bsecurity\b|auth|permission|secret|\b(fail|failed|failure|exit\s+[1-9][0-9]*)\b/i.test(String(text || ''));
}

function compactFence(text, maxLines) {
  let changed = false;
  const output = String(text || '').replace(/```text\n([\s\S]*?)```/g, (match, body) => {
    const lines = body.replace(/\n$/, '').split(/\r?\n/);
    if (lines.length <= maxLines) return match;
    changed = true;
    const headCount = Math.ceil(maxLines / 2);
    const tailCount = Math.floor(maxLines / 2);
    const omitted = lines.length - headCount - tailCount;
    const compacted = [
      ...lines.slice(0, headCount),
      `... ${omitted} line(s) omitted by evidence-compact ...`,
      ...lines.slice(lines.length - tailCount)
    ];
    return `\`\`\`text\n${compacted.join('\n')}\n\`\`\``;
  });
  return { text: output, changed };
}

function buildReport(args) {
  const archiveDir = path.join(args.target, 'docs', 'evidence', args.slug);
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      slug: args.slug,
      archiveDir: rel(args.target, archiveDir),
      maxLines: args.maxLines,
      files: [],
      wrote: []
    }
  };

  if (!fs.existsSync(archiveDir) || !fs.statSync(archiveDir).isDirectory()) {
    add(report, 'fail', `Evidence archive must exist: ${rel(args.target, archiveDir)}`);
    return report;
  }

  const files = walk(archiveDir).filter((file) => file.endsWith('.md')).sort((a, b) => a.localeCompare(b));
  const combined = files.map((file) => fs.readFileSync(file, 'utf8')).join('\n');
  if (isProtected(combined)) {
    add(report, 'pass', 'Protected strict/failed/security evidence kept unmodified by default');
    report.details.files = files.map((file) => ({ file: rel(args.target, file), action: 'keep-protected' }));
    return report;
  }

  for (const file of files) {
    const current = fs.readFileSync(file, 'utf8');
    const compacted = compactFence(current, args.maxLines);
    const entry = {
      file: rel(args.target, file),
      action: compacted.changed ? 'compact' : 'keep',
      originalBytes: Buffer.byteLength(current),
      compactedBytes: Buffer.byteLength(compacted.text)
    };
    report.details.files.push(entry);
    if (compacted.changed && args.write) {
      fs.writeFileSync(file, compacted.text, 'utf8');
      report.details.wrote.push(entry.file);
    }
  }

  const candidates = report.details.files.filter((file) => file.action === 'compact');
  if (candidates.length > 0) add(report, 'warn', `Compaction candidate(s): ${candidates.map((file) => file.file).join(', ')}`);
  else add(report, 'pass', 'No oversized command output fences found');
  if (!args.write) add(report, 'warn', 'Dry run only; no evidence files were changed');
  else add(report, 'pass', `Compacted ${report.details.wrote.length} file(s)`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness evidence compact: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
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
