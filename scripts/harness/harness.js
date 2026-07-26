#!/usr/bin/env node

const path = require('path');
const { spawnSync } = require('child_process');

const defaultCommands = [
  ['context', 'project-context.js', 'Read the current short route.', []],
  ['context diagnostic', 'project-context.js', 'Read the short route with diagnostics.', ['--diagnostic']],
  ['map query', 'codebase-map.js', 'Query the partial Code Map.', ['query']],
  ['map overlay', 'codebase-map.js', 'Report uncommitted Code Map overlays.', ['overlay']],
  ['map check', 'codebase-map.js', 'Check the partial Code Map.', ['check']],
  ['checkpoint', 'checkpoint-audit.js', 'Check the explicitly bound important work item.', ['--current']],
  ['shape', 'workbench-shape-gate.js', 'Run the workbench shape gate explicitly.', []],
  ['stage-k', 'stage-k-architecture-gate.js', 'Run the Stage K architecture gate explicitly.', []],
  ['doctor', 'harness-doctor.js', 'Run the explicit manual harness diagnostic.', []]
];

const compatibilityCommands = [
  ['pre-work', 'pre-work.js', 'Run pre-work readiness checks.'],
  ['pre-completion', 'pre-completion.js', 'Run pre-completion checks.'],
  ['init config', 'config-init.js', 'Initialize harness config.'],
  ['init docs', 'runtime-docs-init.js', 'Initialize runtime docs from templates.'],
  ['init hooks', 'hook-install.js', 'Install managed Git hooks.'],
  ['init ci', 'ci-init.js', 'Initialize CI templates.'],
  ['profile', 'project-profile.js', 'Inspect project profile signals.'],
  ['policy', 'config-policy.js', 'Inspect harness policy configuration.'],
  ['mistake query', 'mistake-query.js', 'Query the mistake ledger for related prior failures.'],
  ['memory candidate new', 'memory-candidate-new.js', 'Create a governed memory candidate.'],
  ['memory candidate lint', 'memory-candidate-lint.js', 'Lint governed memory candidates.'],
  ['memory review', 'memory-review.js', 'Review or change governed memory candidate status.'],
  ['memory stale-check', 'memory-stale-check.js', 'Check governed memory candidates for staleness.'],
  ['memory maintenance', 'memory-maintenance.js', 'Run governed memory maintenance.'],
  ['memory agentmemory query', 'memory-agentmemory-query.js', 'Query agentmemory through the governance wrapper.'],
  ['memory agentmemory save', 'memory-agentmemory-save.js', 'Save approved governed memory to agentmemory.'],
  ['task start', 'task-start.js', 'Start a task record.'],
  ['task finish', 'task-finish.js', 'Finish a task record.'],
  ['task status', 'task-status.js', 'Report task status.'],
  ['task risk', 'task-risk.js', 'Recommend project preset and task path.'],
  ['task package new', 'task-package-new.js', 'Create a structured task package.'],
  ['task package lint', 'task-package-lint.js', 'Lint structured task packages.'],
  ['evidence new', 'evidence-new.js', 'Create an evidence archive entry.'],
  ['evidence retention', 'evidence-retention.js', 'Plan or apply evidence archive retention.'],
  ['evidence compact', 'evidence-compact.js', 'Compact oversized command output evidence.'],
  ['evidence index', 'evidence-index.js', 'Index evidence archives.'],
  ['evidence query', 'evidence-query.js', 'Query evidence archives.'],
  ['skill recommend', 'skill-recommend.js', 'Recommend required skills for task text.'],
  ['security scan', 'security-scan.js', 'Scan text or files for prompt injection and secret patterns.'],
  ['eval', 'eval-runner.js', 'Run harness eval instrumentation suites.'],
  ['verify plan', 'verification-plan.js', 'Plan verification commands.'],
  ['verify run', 'verification-runner.js', 'Run one verification command.'],
  ['verify suite', 'verification-suite.js', 'Run a verification suite.'],
  ['capabilities', 'capability-map.js', 'Map local agent/tool capabilities.']
];

const commands = [...defaultCommands, ...compatibilityCommands].map(([alias, script, description, prefixArgs = []], index) => ({
  alias,
  script,
  description,
  prefixArgs,
  index
}));

const commandsBySpecificity = commands.slice().sort((left, right) => {
  const tokenDifference = right.alias.split(' ').length - left.alias.split(' ').length;
  return tokenDifference || left.index - right.index;
});

function printCommandList(commandsToPrint) {
  for (const command of commandsToPrint) {
    console.log(`  ${command.alias.padEnd(30)} ${command.description} (${command.script})`);
  }
}

function printHelp() {
  console.log('Usage: node scripts/harness/harness.js <command> [args]');
  console.log('');
  console.log('Current manual entrypoints:');
  printCommandList(commands.slice(0, defaultCommands.length));
  console.log('');
  console.log('Examples:');
  console.log('  node scripts/harness/harness.js context --target .');
  console.log('  node scripts/harness/harness.js map query --target . --query "conversation transport"');
  console.log('');
  console.log('Use --legacy to list hidden compatibility commands.');
}

function printLegacyHelp() {
  console.log('Compatibility commands (hidden from default help):');
  console.log('They remain directly callable with their existing arguments and exit codes.');
  printCommandList(commands.slice(defaultCommands.length));
}

function isHelp(argv) {
  return argv.length === 0 || argv[0] === '--help' || argv[0] === '-h';
}

function isLegacyHelp(argv) {
  return argv.length === 1 && argv[0] === '--legacy'
    || argv.length === 2 && argv.includes('--legacy') && (argv.includes('--help') || argv.includes('-h'));
}

function matchCommand(argv) {
  for (const command of commandsBySpecificity) {
    const parts = command.alias.split(' ');
    const matches = parts.every((part, index) => argv[index] === part);
    if (matches) {
      return {
        alias: command.alias,
        script: command.script,
        prefixArgs: command.prefixArgs,
        rest: argv.slice(parts.length)
      };
    }
  }
  return null;
}

function main() {
  const argv = process.argv.slice(2);
  if (isLegacyHelp(argv)) {
    printLegacyHelp();
    return 0;
  }
  if (isHelp(argv)) {
    printHelp();
    return 0;
  }

  const match = matchCommand(argv);
  if (!match) {
    console.error(`Unknown harness command: ${argv[0] || '(none)'}`);
    console.error('');
    printHelp();
    return 1;
  }

  const scriptPath = path.join(__dirname, match.script);
  const result = spawnSync(process.execPath, [scriptPath, ...match.prefixArgs, ...match.rest], {
    cwd: process.cwd(),
    stdio: 'inherit'
  });

  if (result.error) {
    console.error(`ERROR: Failed to run ${match.alias}: ${result.error.message}`);
    return 1;
  }

  if (typeof result.status === 'number') return result.status;
  if (result.signal) {
    console.error(`ERROR: ${match.alias} terminated by signal ${result.signal}`);
    return 1;
  }
  return 1;
}

process.exitCode = main();
