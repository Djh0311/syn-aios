#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const sourceRoot = path.resolve(__dirname, '..', '..');
const providers = {
  github: {
    template: path.join(sourceRoot, 'templates', 'ci', 'github-actions', 'harness.yml'),
    target: path.join('.github', 'workflows', 'harness.yml')
  },
  gitlab: {
    template: path.join(sourceRoot, 'templates', 'ci', 'gitlab', 'harness.yml'),
    target: '.gitlab-ci-harness.yml'
  }
};

function parseArgs(argv) {
  const args = {
    target: null,
    provider: null,
    write: false,
    json: false,
    force: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--provider') args.provider = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--force') args.force = true;
    else if (!args.target) args.target = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (!args.target || !args.provider) throw new Error('Usage: node scripts/harness/ci-init.js --target <dir> --provider github|gitlab [--write] [--json] [--force]');
  if (!providers[args.provider]) throw new Error(`Unsupported provider: ${args.provider}`);
  return args;
}

function readJson(filePath) {
  if (!fs.existsSync(filePath)) return null;
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (_error) {
    return null;
  }
}

function commandKeysFromConfig(targetRoot) {
  const config = readJson(path.join(targetRoot, 'harness.config.json')) || readJson(path.join(targetRoot, 'harness.config.example.json'));
  const commands = config && config.commands && typeof config.commands === 'object' ? config.commands : {};
  return ['lint', 'typecheck', 'test', 'testUnit', 'testIntegration', 'testE2E', 'build']
    .filter((key) => {
      const value = typeof commands[key] === 'string' ? commands[key].trim() : '';
      return value !== '' && !/\|/.test(value);
    });
}

function isSourcePackage(targetRoot) {
  return fs.existsSync(path.join(targetRoot, 'plans', '2026-05-13-harness-upgrade-phase-3.md'))
    && fs.existsSync(path.join(targetRoot, 'templates', 'docs'))
    && fs.existsSync(path.join(targetRoot, 'scripts', 'harness', 'self-test.js'));
}

function renderTemplate(provider, targetRoot) {
  let template = fs.readFileSync(providers[provider].template, 'utf8').trimEnd();
  const commandKeys = commandKeysFromConfig(targetRoot);

  if (provider === 'github' && commandKeys.length > 0) {
    const steps = commandKeys.map((key) => [
      `      - name: Harness ${key}`,
      `        run: node scripts/harness/verification-runner.js --target . --command-key ${key}`
    ].join('\n')).join('\n\n');
    template = template.replace(/      # Add project command checks[\s\S]*?      - name: Pre-completion gate/, `${steps}\n\n      - name: Pre-completion gate`);
  }

  if (provider === 'gitlab' && commandKeys.length > 0) {
    const lines = commandKeys.map((key) => `    - node scripts/harness/verification-runner.js --target . --command-key ${key}`).join('\n');
    template = template.replace(/    # Add project command checks[\s\S]*?    - node scripts\/harness\/pre-completion\.js --target \./, `${lines}\n    - node scripts/harness/pre-completion.js --target .`);
  }

  if (!isSourcePackage(targetRoot)) {
    if (provider === 'github') {
      template = template.replace(/\n\n      # Source package only[\s\S]*?run: node scripts\/harness\/self-test\.js/, '');
    } else {
      template = template.replace(/\n    # Source package only[\s\S]*?      fi/, '');
    }
  }

  return `${template}\n`;
}

function add(report, status, target, message) {
  report[status].push({ target, message });
}

function buildReport(args) {
  const targetRoot = path.resolve(args.target);
  const provider = providers[args.provider];
  const target = path.join(targetRoot, provider.target);
  const report = {
    command: 'ci-init',
    write: args.write,
    target: targetRoot,
    provider: args.provider,
    commandKeys: commandKeysFromConfig(targetRoot),
    sourcePackage: isSourcePackage(targetRoot),
    pass: [],
    warn: [],
    fail: []
  };

  if (!fs.existsSync(provider.template)) {
    add(report, 'fail', target, `Template missing: ${provider.template}`);
    return report;
  }

  if (fs.existsSync(target) && !args.force) {
    add(report, 'warn', target, 'Existing CI file preserved; re-run with --force to overwrite');
    return report;
  }

  add(report, 'pass', target, `${args.write ? 'Wrote' : 'Would write'} ${args.provider} harness CI`);
  if (args.write) {
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, renderTemplate(args.provider, targetRoot));
  }

  return report;
}

function printText(report) {
  console.log(`Harness CI init ${report.write ? 'write' : 'dry-run'} report`);
  console.log(`Target: ${report.target}`);
  console.log(`Provider: ${report.provider}`);
  console.log(`Configured command keys: ${report.commandKeys.length > 0 ? report.commandKeys.join(', ') : 'none'}`);
  console.log(`Source package self-test: ${report.sourcePackage ? 'included' : 'not included'}`);

  for (const [title, items] of [['PASS', report.pass], ['WARN', report.warn], ['FAIL', report.fail]]) {
    console.log(`\n${title} (${items.length})`);
    if (items.length === 0) {
      console.log('  None');
      continue;
    }
    for (const item of items) console.log(`  - ${item.target} (${item.message})`);
  }

  if (!report.write) console.log('\nDry run only. Re-run with --write to create CI.');
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
