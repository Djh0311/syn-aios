#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const sourceRoot = path.resolve(__dirname, '..', '..');
const supportedHooks = ['pre-commit', 'pre-push'];
const managedStart = '# >>> standard-ai-engineering-harness >>>';
const managedEnd = '# <<< standard-ai-engineering-harness <<<';

function parseArgs(argv) {
  const args = {
    target: null,
    hook: 'all',
    write: false,
    json: false,
    force: false,
    allowNonGitTemplate: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--hook') args.hook = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--force') args.force = true;
    else if (arg === '--allow-non-git-template') args.allowNonGitTemplate = true;
    else if (!args.target) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!args.target) throw new Error('Usage: node scripts/harness/hook-install.js --target <dir> [--hook pre-commit|pre-push|all] [--write] [--json] [--force] [--allow-non-git-template]');
  if (!['all'].concat(supportedHooks).includes(args.hook)) throw new Error(`Unsupported hook: ${args.hook}`);
  return args;
}

function selectedHooks(hook) {
  return hook === 'all' ? supportedHooks : [hook];
}

function templatePath(hook) {
  return path.join(sourceRoot, 'templates', 'hooks', hook);
}

function hookPath(targetRoot, hook, allowNonGitTemplate) {
  const hooksDir = gitHooksDir(targetRoot);
  if (allowNonGitTemplate && !hooksDir) {
    return path.join(targetRoot, 'templates', 'hooks', hook);
  }
  return path.join(hooksDir || path.join(targetRoot, '.git', 'hooks'), hook);
}

function isGitRepo(targetRoot) {
  return Boolean(gitHooksDir(targetRoot));
}

function gitHooksDir(targetRoot) {
  const dotGit = path.join(targetRoot, '.git');
  if (!fs.existsSync(dotGit)) return null;
  const stat = fs.statSync(dotGit);
  if (stat.isDirectory()) return path.join(dotGit, 'hooks');
  if (!stat.isFile()) return null;

  const content = fs.readFileSync(dotGit, 'utf8').trim();
  const match = content.match(/^gitdir:\s*(.+)$/i);
  if (!match) return null;
  const gitDir = path.resolve(targetRoot, match[1].trim());
  return path.join(gitDir, 'hooks');
}

function hasManagedBlock(content) {
  return content.includes(managedStart) && content.includes(managedEnd);
}

function removeManagedBlock(content) {
  const escapedStart = managedStart.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const escapedEnd = managedEnd.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return content.replace(new RegExp(`\\n?${escapedStart}[\\s\\S]*?${escapedEnd}\\n?`, 'g'), '\n').replace(/\n{3,}/g, '\n\n').trimEnd();
}

function templateBody(hook) {
  const lines = fs.readFileSync(templatePath(hook), 'utf8').trimEnd().split(/\r?\n/);
  if (lines[0] && lines[0].startsWith('#!')) lines.shift();
  return lines.join('\n').trim();
}

function renderManagedBlock(hook) {
  return `${managedStart}\n${templateBody(hook)}\n${managedEnd}\n`;
}

function renderHookFile(hook) {
  return `#!/bin/sh\n\n${renderManagedBlock(hook)}`;
}

function isOnlyShebang(content) {
  return content.trim() === '#!/bin/sh';
}

function modeForExisting(target, force) {
  if (!fs.existsSync(target)) return 'create';
  const existing = fs.readFileSync(target, 'utf8');
  if (hasManagedBlock(existing)) return 'update-managed';
  if (force) return 'overwrite-forced';
  return 'skip-existing';
}

function writeHook(target, hook, mode) {
  fs.mkdirSync(path.dirname(target), { recursive: true });

  if (mode === 'create' || mode === 'overwrite-forced') {
    fs.writeFileSync(target, renderHookFile(hook), { mode: 0o755 });
  } else if (mode === 'update-managed') {
    const existing = fs.readFileSync(target, 'utf8');
    const unmanaged = removeManagedBlock(existing);
    const next = unmanaged && !isOnlyShebang(unmanaged) ? `${unmanaged}\n\n${renderManagedBlock(hook)}` : renderHookFile(hook);
    fs.writeFileSync(target, next, { mode: 0o755 });
  }

  fs.chmodSync(target, 0o755);
}

function add(report, status, hook, target, message) {
  report[status].push({ hook, target, message });
}

function buildReport(args) {
  const targetRoot = path.resolve(args.target);
  const gitRepo = isGitRepo(targetRoot);
  const report = {
    command: 'hook-install',
    write: args.write,
    target: targetRoot,
    gitRepo,
    allowNonGitTemplate: args.allowNonGitTemplate,
    pass: [],
    warn: [],
    fail: []
  };

  if (!gitRepo && !args.allowNonGitTemplate) {
    for (const hook of selectedHooks(args.hook)) {
      add(report, 'fail', hook, hookPath(targetRoot, hook, false), 'Target is not a Git repository; use --allow-non-git-template to generate template hooks');
    }
    return report;
  }

  for (const hook of selectedHooks(args.hook)) {
    const source = templatePath(hook);
    const target = hookPath(targetRoot, hook, args.allowNonGitTemplate);
    if (!fs.existsSync(source)) {
      add(report, 'fail', hook, target, `Template missing: ${source}`);
      continue;
    }

    const mode = modeForExisting(target, args.force);
    if (mode === 'skip-existing') {
      add(report, 'warn', hook, target, 'Existing non-managed hook preserved; re-run with --force to overwrite');
      continue;
    }

    add(report, 'pass', hook, target, `${args.write ? 'Installed' : 'Would install'} ${hook} (${mode})`);
    if (args.write) writeHook(target, hook, mode);
  }

  return report;
}

function printText(report) {
  console.log(`Harness hook install ${report.write ? 'write' : 'dry-run'} report`);
  console.log(`Target: ${report.target}`);
  console.log(`Git repository: ${report.gitRepo ? 'yes' : 'no'}`);

  for (const [title, items] of [['PASS', report.pass], ['WARN', report.warn], ['FAIL', report.fail]]) {
    console.log(`\n${title} (${items.length})`);
    if (items.length === 0) {
      console.log('  None');
      continue;
    }
    for (const item of items) console.log(`  - ${item.hook}: ${item.target} (${item.message})`);
  }

  if (!report.write) console.log('\nDry run only. Re-run with --write to install hooks.');
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printText(report);
  if (report.fail.length > 0) process.exit(1);
}

try {
  main();
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
