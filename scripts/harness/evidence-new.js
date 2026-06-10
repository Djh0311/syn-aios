#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { redactSecrets } = require('./lib/security');

function parseArgs(argv) {
  const args = {
    target: null,
    slug: null,
    title: null,
    kind: 'verification',
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--title') args.title = argv[++i];
    else if (arg === '--kind') args.kind = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.write && !args.target) {
    throw new Error('Writing evidence requires an explicit --target installed-project directory');
  }

  args.target = path.resolve(args.target || process.cwd());
  return args;
}

function sanitizeSlug(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .replace(/-+/g, '-');
}

function timestampSlug(date) {
  return date.toISOString()
    .replace(/\.\d{3}Z$/, 'z')
    .replace(/[^0-9a-z]+/gi, '-')
    .toLowerCase()
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function makeSlug(args, now) {
  const source = args.slug || args.title || timestampSlug(now);

  if (/[\\/]/.test(String(source)) || String(source).includes('..')) {
    throw new Error('Slug must be a single safe path segment without traversal');
  }

  const slug = sanitizeSlug(source);
  if (!slug) throw new Error('Slug is empty after sanitization');
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error(`Slug is not safe after sanitization: ${slug}`);
  return slug;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function markdownEscape(value) {
  return String(value || '').replace(/\r?\n/g, ' ').trim();
}

function buildContent(args, slug, filePath, now) {
  const title = markdownEscape(args.title || slug);
  const kind = markdownEscape(args.kind || 'verification');
  const createdAt = now.toISOString();
  const target = rel(args.target, filePath);

  const content = `# Evidence: ${title}

- title: ${title}
- kind: ${kind}
- createdAt: ${createdAt}
- target: ${target}

## Claims

- Claim: TBD
  - Evidence: TBD
  - Result: TBD

## Commands

| Command | Result | Notes |
| --- | --- | --- |
| TBD | TBD | TBD |

## Browser

| Route | Viewport | Interaction | Console | Network | Evidence |
| --- | --- | --- | --- | --- | --- |
| TBD | TBD | TBD | TBD | TBD | TBD |

## Failures/Gaps

- TBD

## Links

- Requirements: TBD
- Mistake ledger: TBD
- PR / CI: TBD
`;
  return redactSecrets(content).text;
}

function buildPlan(args) {
  const now = new Date();
  const slug = makeSlug(args, now);
  const evidenceDir = path.join(args.target, 'docs', 'evidence', slug);
  const filePath = path.join(evidenceDir, 'summary.md');
  const evidenceRoot = path.join(args.target, 'docs', 'evidence');

  const targetStat = fs.existsSync(args.target) ? fs.statSync(args.target) : null;
  if (!targetStat || !targetStat.isDirectory()) {
    throw new Error(`Target must be an existing installed-project directory: ${args.target}`);
  }

  const relativeFile = rel(args.target, filePath);
  if (!relativeFile.startsWith('docs/evidence/') || !relativeFile.endsWith('/summary.md')) {
    throw new Error(`Refusing to write outside docs/evidence/<slug>/summary.md: ${relativeFile}`);
  }

  if (fs.existsSync(filePath)) {
    throw new Error(`Evidence summary already exists; refusing to overwrite: ${filePath}`);
  }

  const content = buildContent(args, slug, filePath, now);
  return {
    mode: args.write ? 'write' : 'dry-run',
    target: args.target,
    slug,
    title: args.title || slug,
    kind: args.kind,
    evidenceRoot,
    file: filePath,
    relativeFile,
    content
  };
}

function writePlan(plan) {
  fs.mkdirSync(path.dirname(plan.file), { recursive: true });
  fs.writeFileSync(plan.file, plan.content, { encoding: 'utf8', flag: 'wx' });
}

function printText(plan) {
  console.log('Harness evidence archive scaffold');
  console.log('Purpose: create installed-project docs/evidence/<slug>/summary.md without overwriting existing evidence.');
  console.log(`Mode: ${plan.mode}`);
  console.log(`Target: ${plan.target}`);
  console.log(`File: ${plan.relativeFile}`);
  console.log('\nContent preview:');
  console.log(plan.content);
  if (plan.mode === 'dry-run') {
    console.log('Dry run only. Re-run with --write and an explicit --target to create the file.');
  } else {
    console.log('Created evidence summary.');
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  const plan = buildPlan(args);
  if (args.write) writePlan(plan);

  if (args.json) {
    console.log(JSON.stringify({
      mode: plan.mode,
      target: plan.target,
      slug: plan.slug,
      title: plan.title,
      kind: plan.kind,
      file: plan.file,
      relativeFile: plan.relativeFile,
      content: plan.content
    }, null, 2));
  } else {
    printText(plan);
  }
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
