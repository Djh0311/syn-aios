#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./lib/project-kind');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    slug: null,
    write: false,
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
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

function timestamps(text) {
  const matches = [];
  const pattern = /(?:createdAt|recordedAt|Last Updated):\s*([0-9]{4}-[0-9]{2}-[0-9]{2}(?:T[0-9:.Z+-]+)?)/gi;
  for (const match of text.matchAll(pattern)) matches.push(match[1]);
  return matches;
}

function requirementIds(text) {
  const ids = new Set();
  for (const match of text.matchAll(/\bR-[0-9A-Za-z._-]+\b/g)) ids.add(match[0]);
  return Array.from(ids).sort();
}

function resultStatus(text) {
  if (/\b(fail|failed|failure|exit\s+[1-9][0-9]*)\b/i.test(text)) return 'fail';
  if (/\b(warn|warning|concern)\b/i.test(text)) return 'warn';
  if (/\b(pass|passed|success|exit\s+0)\b/i.test(text)) return 'pass';
  return 'unknown';
}

function evidenceType(file, text) {
  const base = path.basename(file);
  if (/browser|console-network|screenshot|viewport/i.test(base) || /Chrome DevTools MCP|browser console|network|screenshot|viewport|interaction/i.test(text)) return 'browser';
  if (/test-output|command|verification/i.test(base) || /\$ |command:|exitCode|exit 0|exit [1-9]/i.test(text)) return 'command';
  return 'summary';
}

function archive(root, dir) {
  const files = walk(dir).filter((file) => path.basename(file) !== 'index.json' && path.basename(file) !== 'index.md');
  const item = {
    slug: path.basename(dir),
    dir: rel(root, dir),
    title: path.basename(dir),
    timestamps: [],
    latestTimestamp: null,
    requirementIds: [],
    files: [],
    counts: {
      command: 0,
      browser: 0,
      summary: 0,
      pass: 0,
      warn: 0,
      fail: 0,
      unknown: 0
    }
  };
  const allRequirements = new Set();

  for (const file of files) {
    const text = fs.readFileSync(file, 'utf8');
    const type = evidenceType(file, text);
    const status = resultStatus(text);
    const ts = timestamps(text);
    const reqs = requirementIds(text);
    if (path.basename(file) === 'summary.md') {
      const heading = text.match(/^#\s+(.+)$/m);
      if (heading) item.title = heading[1].trim();
    }
    for (const req of reqs) allRequirements.add(req);
    item.timestamps.push(...ts);
    item.counts[type] += 1;
    item.counts[status] += 1;
    item.files.push({
      file: rel(root, file),
      type,
      status,
      timestamps: ts,
      requirementIds: reqs
    });
  }

  item.timestamps = Array.from(new Set(item.timestamps)).sort();
  item.latestTimestamp = item.timestamps[item.timestamps.length - 1] || null;
  item.requirementIds = Array.from(allRequirements).sort();
  item.files.sort((a, b) => a.file.localeCompare(b.file));
  return item;
}

function buildIndex(targetRoot, slug) {
  const evidenceRoot = path.join(targetRoot, 'docs', 'evidence');
  const archives = [];
  if (!fs.existsSync(evidenceRoot)) return { evidenceRoot, archives };
  const dirs = slug
    ? [path.join(evidenceRoot, slug)]
    : fs.readdirSync(evidenceRoot, { withFileTypes: true }).filter((entry) => entry.isDirectory()).map((entry) => path.join(evidenceRoot, entry.name));
  for (const dir of dirs) {
    if (fs.existsSync(dir) && fs.statSync(dir).isDirectory()) archives.push(archive(targetRoot, dir));
  }
  archives.sort((a, b) => a.slug.localeCompare(b.slug));
  return { evidenceRoot, archives };
}

function indexMarkdown(index) {
  const lines = [
    '# Evidence Index',
    '',
    `Generated At: ${index.generatedAt}`,
    '',
    '| Slug | Latest Timestamp | Command | Browser | Status Counts |',
    '| --- | --- | ---: | ---: | --- |'
  ];
  for (const archiveItem of index.archives) {
    lines.push(`| ${archiveItem.slug} | ${archiveItem.latestTimestamp || 'None'} | ${archiveItem.counts.command} | ${archiveItem.counts.browser} | pass=${archiveItem.counts.pass}, warn=${archiveItem.counts.warn}, fail=${archiveItem.counts.fail}, unknown=${archiveItem.counts.unknown} |`);
  }
  lines.push('');
  return `${lines.join('\n')}\n`;
}

function writeIndex(targetRoot, index) {
  const root = path.join(targetRoot, 'docs', 'evidence');
  fs.mkdirSync(root, { recursive: true });
  fs.writeFileSync(path.join(root, 'index.json'), `${JSON.stringify(index, null, 2)}\n`, 'utf8');
  fs.writeFileSync(path.join(root, 'index.md'), indexMarkdown(index), 'utf8');
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    pass: [],
    warn: [],
    fail: [],
    details: {
      projectKind: null,
      index: null,
      wrote: []
    }
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  const kind = detectProjectKind(args.target);
  report.details.projectKind = kind.kind;
  if (kind.isSourcePackage) {
    add(report, 'pass', 'Source package detected; runtime evidence index is not required');
    report.details.index = { generatedAt: new Date().toISOString(), archives: [] };
    return report;
  }

  const built = buildIndex(args.target, args.slug);
  const index = {
    generatedAt: new Date().toISOString(),
    target: args.target,
    slug: args.slug,
    archives: built.archives
  };
  report.details.index = index;
  if (built.archives.length > 0) add(report, 'pass', `Evidence archives indexed: ${built.archives.length}`);
  else add(report, args.strict ? 'fail' : 'warn', args.slug ? `No evidence archive found for slug: ${args.slug}` : 'No evidence archives found');
  if (!args.write) add(report, 'warn', 'Dry run only; index files were not written');
  else {
    writeIndex(args.target, index);
    report.details.wrote = ['docs/evidence/index.json', 'docs/evidence/index.md'];
    add(report, 'pass', 'Wrote docs/evidence/index.json and docs/evidence/index.md');
  }
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness evidence index: ${report.target}`);
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
