#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const supportedHooks = ['pre-commit', 'pre-push'];
const managedStart = '# >>> standard-ai-engineering-harness >>>';
const managedEnd = '# <<< standard-ai-engineering-harness <<<';

function parseArgs(argv) {
  const args = {
    target: null,
    hook: 'all',
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--hook') args.hook = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (!args.target) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!args.target) throw new Error('Usage: node scripts/harness/hook-uninstall.js --target <dir> [--hook pre-commit|pre-push|all] [--write] [--json]');
  if (!['all'].concat(supportedHooks).includes(args.hook)) throw new Error(`Unsupported hook: ${args.hook}`);
  return args;
}

function selectedHooks(hook) {
  return hook === 'all' ? supportedHooks : [hook];
}

function hookPath(targetRoot, hook) {
  const hooksDir = gitHooksDir(targetRoot);
  return path.join(hooksDir || path.join(targetRoot, '.git', 'hooks'), hook);
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

function add(report, status, hook, target, message) {
  report[status].push({ hook, target, message });
}

function maybeRemove(target, write) {
  const existing = fs.readFileSync(target, 'utf8');
  const next = removeManagedBlock(existing);
  if (!write) return;
  if (next.trim() === '' || next.trim() === '#!/bin/sh') fs.unlinkSync(target);
  else {
    fs.writeFileSync(target, `${next}\n`, { mode: 0o755 });
    fs.chmodSync(target, 0o755);
  }
}

function buildReport(args) {
  const targetRoot = path.resolve(args.target);
  const report = {
    command: 'hook-uninstall',
    write: args.write,
    target: targetRoot,
    pass: [],
    warn: [],
    fail: []
  };

  for (const hook of selectedHooks(args.hook)) {
    const target = hookPath(targetRoot, hook);
    if (!fs.existsSync(target)) {
      add(report, 'warn', hook, target, 'Hook file does not exist');
      continue;
    }

    const existing = fs.readFileSync(target, 'utf8');
    if (!hasManagedBlock(existing)) {
      add(report, 'warn', hook, target, 'No harness-managed block found; preserved');
      continue;
    }

    add(report, 'pass', hook, target, `${args.write ? 'Removed' : 'Would remove'} harness-managed block`);
    maybeRemove(target, args.write);
  }

  return report;
}

function printText(report) {
  console.log(`Harness hook uninstall ${report.write ? 'write' : 'dry-run'} report`);
  console.log(`Target: ${report.target}`);

  for (const [title, items] of [['PASS', report.pass], ['WARN', report.warn], ['FAIL', report.fail]]) {
    console.log(`\n${title} (${items.length})`);
    if (items.length === 0) {
      console.log('  None');
      continue;
    }
    for (const item of items) console.log(`  - ${item.hook}: ${item.target} (${item.message})`);
  }

  if (!report.write) console.log('\nDry run only. Re-run with --write to uninstall managed hook blocks.');
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
