#!/usr/bin/env node

const childProcess = require('child_process');
const fs = require('fs');
const path = require('path');

const PRODUCT_ROOT = 'prototypes/productized-desktop-shell';
const R_PREFLIGHT_BASELINE_COMMIT = 'ed01c6f281e3fd7a38548da948046e8366cc368d';
const R0_PACKAGE_COMMIT = 'a40b7b56ab949cd26b145ba0eccf9f3921886ea0';
const RATCHET_POLICY = 'historical_lowest_closed_value';
const COMMAND_BASELINE_TOTAL = 97;
const COMMAND_BASELINE_TOTAL_DECISION =
  'R4-A2 query_workbench_page_read_model introduced one read-only skeleton Tauri command; strategy review P2-3 accepted 97 as the current command baseline.';
const COMMAND_BASELINE_LIB_RS = 0;
const JS_GATE_SOFT_LIMIT = 500;

const TEXT_EXTENSIONS = new Set(['.rs', '.ts', '.tsx', '.css', '.js', '.mjs']);
const SOURCE_EXTENSIONS = new Set(['.rs', '.ts', '.tsx', '.css']);
const EXCLUDED_DIRS = new Set(['.git', 'node_modules', 'target', 'dist', '.harness']);
const NEW_FILE_LIMITS = new Map([
  ['.rs', 3000],
  ['.ts', 2000],
  ['.tsx', 2000]
]);

const KEY_METRIC_FILES = [
  ['lib.rs', 'prototypes/productized-desktop-shell/src-tauri/src/lib.rs'],
  ['real_execution_command.rs', 'prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs'],
  ['ProjectsView.tsx', 'prototypes/productized-desktop-shell/src/views/ProjectsView.tsx'],
  ['AgentView.tsx', 'prototypes/productized-desktop-shell/src/views/AgentView.tsx'],
  ['types.rs', 'prototypes/productized-desktop-shell/src-tauri/src/types.rs'],
  ['types.ts', 'prototypes/productized-desktop-shell/src/lib/types.ts'],
  ['styles.css', 'prototypes/productized-desktop-shell/src/styles.css'],
  ['offline-permission-dialog.test.tsx', 'prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx']
];

const RATCHET_WATERLINES = new Map([
  ['prototypes/productized-desktop-shell/src-tauri/src/lib.rs', 5567],
  ['prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx', 3404],
  ['prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs', 8763],
  ['prototypes/productized-desktop-shell/src/styles.css', 8464],
  ['prototypes/productized-desktop-shell/src/views/ProjectsView.tsx', 5897],
  ['prototypes/productized-desktop-shell/src-tauri/src/types.rs', 5386],
  ['prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs', 5237],
  ['prototypes/productized-desktop-shell/src/lib/types.ts', 4998],
  ['prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs', 5059],
  ['prototypes/productized-desktop-shell/src/views/AgentView.tsx', 3118],
  ['prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs', 2429],
  ['prototypes/productized-desktop-shell/src/lib/projectCanvas.ts', 2050]
]);

const ALLOWED_SIDECAR_JSON = new Set([
  'blackboard-candidates.v1.json',
  'formal-memories.v1.json',
  'memory-candidates.v1.json',
  'memory-capture-events.v1.json',
  'memory-entity-relations.v1.json',
  'memory-lint.v1.json',
  'memory-patterns.v1.json',
  'observations.v1.json',
  'plan-authorizations.v1.json',
  'project-proposals.v1.json',
  'real-execution-product-commands.v1.json',
  'runtime-log.v1.json',
  'runtime-logs.v1.json',
  'session-continuations.v1.json'
]);

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    mode: 'check',
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--mode') args.mode = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!['baseline', 'check'].includes(args.mode)) {
    throw new Error(`Unsupported mode: ${args.mode}. Expected baseline or check.`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function printHelp() {
  console.log(`Usage: node scripts/harness/workbench-shape-gate.js --mode baseline|check [--target PATH] [--json] [--strict]

Modes:
  baseline  Print current shape metrics without failing for existing debt.
  check     Apply R0 ratchet rules and fail on hard shape regressions.

Boundary:
  This gate is read-only. It does not execute Codex, send prompts, read/write /Users/yoyi/.codex, start Tauri, or inspect secrets.`);
}

function walkFiles(root, files = []) {
  if (!fs.existsSync(root)) return files;
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (EXCLUDED_DIRS.has(entry.name)) continue;
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) walkFiles(fullPath, files);
    else if (TEXT_EXTENSIONS.has(path.extname(entry.name))) files.push(fullPath);
  }
  return files;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function readText(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function countLines(text) {
  if (text.length === 0) return 0;
  const newlineCount = (text.match(/\n/g) || []).length;
  return text.endsWith('\n') ? newlineCount : newlineCount + 1;
}

function git(root, args) {
  try {
    return childProcess.execFileSync('git', args, {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore']
    }).trim();
  } catch (_error) {
    return null;
  }
}

function addFinding(report, severity, id, message, detail = null) {
  report.findings.push({ severity, id, message, detail });
}

function scanCommands(root) {
  const commandFiles = walkFiles(path.join(root, PRODUCT_ROOT, 'src-tauri', 'src'))
    .filter((filePath) => path.extname(filePath) === '.rs');
  const byFile = [];
  let total = 0;

  for (const filePath of commandFiles) {
    const text = readText(filePath);
    const count = (text.match(/^\s*#\[tauri::command\]/gm) || []).length;
    if (count > 0) {
      byFile.push({ file: rel(root, filePath), count });
      total += count;
    }
  }

  return {
    total,
    lib_rs: byFile.find((entry) => entry.file === `${PRODUCT_ROOT}/src-tauri/src/lib.rs`)?.count || 0,
    by_file: byFile.sort((a, b) => a.file.localeCompare(b.file))
  };
}

function scanSidecars(root) {
  const sourceRoots = [
    path.join(root, PRODUCT_ROOT, 'src-tauri', 'src'),
    path.join(root, PRODUCT_ROOT, 'src')
  ];
  const locations = new Map();
  const workflowStateNames = new Set();

  for (const sourceRoot of sourceRoots) {
    for (const filePath of walkFiles(sourceRoot)) {
      if (!SOURCE_EXTENSIONS.has(path.extname(filePath))) continue;
      const relative = rel(root, filePath);
      const text = readText(filePath);
      const matches = text.match(/[A-Za-z0-9][A-Za-z0-9_.-]*\.v[0-9]+\.json/g) || [];
      for (const name of matches) {
        if (/^workflow-state/.test(name) || /^missing-workflow-state/.test(name)) {
          workflowStateNames.add(name);
          continue;
        }
        if (!locations.has(name)) locations.set(name, []);
        locations.get(name).push(relative);
      }
    }
  }

  const names = Array.from(locations.keys()).sort();
  const unknown = names.filter((name) => !ALLOWED_SIDECAR_JSON.has(name));
  return {
    detected_count: names.length,
    names,
    unknown,
    allowed_missing: Array.from(ALLOWED_SIDECAR_JSON).filter((name) => !locations.has(name)).sort(),
    workflow_state_names: Array.from(workflowStateNames).sort(),
    locations: Object.fromEntries(
      names.map((name) => [name, Array.from(new Set(locations.get(name))).sort().slice(0, 8)])
    )
  };
}

function scanLines(root) {
  const metricFiles = KEY_METRIC_FILES.map(([label, relative]) => {
    const filePath = path.join(root, relative);
    return {
      label,
      file: relative,
      lines: fs.existsSync(filePath) ? countLines(readText(filePath)) : null
    };
  });

  const productFiles = walkFiles(path.join(root, PRODUCT_ROOT))
    .filter((filePath) => SOURCE_EXTENSIONS.has(path.extname(filePath)));
  const overLimit = [];
  const unknownOverLimit = [];

  for (const filePath of productFiles) {
    const relative = rel(root, filePath);
    const ext = path.extname(filePath);
    const limit = NEW_FILE_LIMITS.get(ext);
    if (!limit) continue;
    const lines = countLines(readText(filePath));
    if (lines > limit) {
      const entry = { file: relative, ext, lines, limit };
      overLimit.push(entry);
      if (!RATCHET_WATERLINES.has(relative)) unknownOverLimit.push(entry);
    }
  }

  const ratchet = Array.from(RATCHET_WATERLINES.entries()).map(([relative, waterline]) => {
    const filePath = path.join(root, relative);
    const lines = fs.existsSync(filePath) ? countLines(readText(filePath)) : null;
    return {
      file: relative,
      lines,
      waterline,
      delta: lines === null ? null : lines - waterline,
      status: lines === null ? 'missing' : lines > waterline ? 'increased' : lines < waterline ? 'decreased' : 'same'
    };
  });

  return {
    metric_files: metricFiles,
    over_limit_files: overLimit.sort((a, b) => b.lines - a.lines),
    unknown_over_limit_files: unknownOverLimit.sort((a, b) => b.lines - a.lines),
    ratchet_files: ratchet
  };
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.mode,
    strict: args.strict,
    boundary: 'Workbench shape gate is read-only; it does not execute Codex, send prompts, read/write /Users/yoyi/.codex, start Tauri, or inspect secrets.',
    baselines: {
      r_preflight_baseline_commit: R_PREFLIGHT_BASELINE_COMMIT,
      r0_package_commit: R0_PACKAGE_COMMIT,
      ratchet_policy: RATCHET_POLICY,
      command_total: COMMAND_BASELINE_TOTAL,
      command_total_decision: COMMAND_BASELINE_TOTAL_DECISION,
      lib_rs_command_count: COMMAND_BASELINE_LIB_RS,
      new_file_limits: Object.fromEntries(NEW_FILE_LIMITS),
      sidecar_json_kinds: Array.from(ALLOWED_SIDECAR_JSON).sort()
    },
    git: {
      toplevel: git(args.target, ['rev-parse', '--show-toplevel']),
      head: git(args.target, ['rev-parse', 'HEAD']),
      status_short: git(args.target, ['status', '--short']) || ''
    },
    metrics: {},
    findings: []
  };

  report.metrics.lines = scanLines(args.target);
  report.metrics.commands = scanCommands(args.target);
  report.metrics.sidecars = scanSidecars(args.target);

  const scriptPath = path.join(args.target, 'scripts/harness/workbench-shape-gate.js');
  if (fs.existsSync(scriptPath)) {
    report.metrics.gate_script_lines = countLines(readText(scriptPath));
  }

  addFinding(report, 'info', 'baseline_commits', 'R0 records both governance baseline and task-package start commit.', {
    r_preflight_baseline_commit: R_PREFLIGHT_BASELINE_COMMIT,
    r0_package_commit: R0_PACKAGE_COMMIT,
    current_head: report.git.head
  });

  if (!report.git.toplevel) {
    addFinding(report, 'warn', 'no_git_repository', 'No git repository detected; R2/R3 must be marked blocked until commit hashes are available.');
  }

  for (const entry of report.metrics.lines.ratchet_files) {
    if (entry.status === 'missing') {
      addFinding(report, 'warn', 'ratchet_file_missing', 'A ratchet file is missing; confirm this is an intentional governance change.', entry);
    } else if (entry.status === 'increased') {
      addFinding(report, 'error', 'ratchet_file_increased', 'A ratchet file is above its historical-low ratchet waterline.', entry);
    } else if (entry.lines > (NEW_FILE_LIMITS.get(path.extname(entry.file)) || Infinity) || entry.file.endsWith('styles.css')) {
      addFinding(report, 'info', 'ratchet_file_existing_debt', 'Existing oversized file is tracked as ratchet debt and must not grow.', entry);
    }
  }

  for (const entry of report.metrics.lines.unknown_over_limit_files) {
    addFinding(report, 'error', 'file_over_limit_not_in_ratchet', 'A source file exceeds the new-file limit but is not in the R0 ratchet list.', entry);
  }

  if (report.metrics.commands.lib_rs > COMMAND_BASELINE_LIB_RS) {
    addFinding(report, 'error', 'tauri_command_added_to_lib_rs', 'lib.rs contains #[tauri::command]; new Tauri commands must not be added to lib.rs.', {
      current: report.metrics.commands.lib_rs,
      baseline: COMMAND_BASELINE_LIB_RS
    });
  }

  if (report.metrics.commands.total > COMMAND_BASELINE_TOTAL) {
    addFinding(report, 'warn', 'tauri_command_total_increased', 'Tauri command total increased; confirm task package shape impact and non-lib.rs placement.', {
      current: report.metrics.commands.total,
      baseline: COMMAND_BASELINE_TOTAL
    });
  }

  for (const name of report.metrics.sidecars.unknown) {
    addFinding(report, 'error', 'unknown_sidecar_json_kind', 'Detected sidecar JSON kind is not in the R0 allowed baseline; new sidecars require user confirmation and a decision record.', {
      name,
      locations: report.metrics.sidecars.locations[name]
    });
  }

  if (report.metrics.gate_script_lines > JS_GATE_SOFT_LIMIT) {
    addFinding(report, 'warn', 'gate_script_soft_limit_exceeded', 'The R0 gate script is above the requested 500-line soft limit; consider splitting before it reaches 800 lines.', {
      lines: report.metrics.gate_script_lines,
      soft_limit: JS_GATE_SOFT_LIMIT
    });
  }

  const errors = report.findings.filter((finding) => finding.severity === 'error');
  const warnings = report.findings.filter((finding) => finding.severity === 'warn');
  report.summary = {
    status: errors.length > 0 || (args.strict && warnings.length > 0) ? 'fail' : 'pass',
    error_count: errors.length,
    warning_count: warnings.length,
    info_count: report.findings.filter((finding) => finding.severity === 'info').length
  };

  if (args.mode === 'baseline') {
    report.summary.status = 'pass';
  }

  return report;
}

function printReport(report) {
  console.log(`Workbench shape gate: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  console.log(report.boundary);
  console.log(`Status: ${report.summary.status}`);
  console.log(`Errors: ${report.summary.error_count}`);
  console.log(`Warnings: ${report.summary.warning_count}`);
  console.log(`Info: ${report.summary.info_count}`);
  console.log(`Git HEAD: ${report.git.head || 'unavailable'}`);
  console.log(`Ratchet policy: ${report.baselines.ratchet_policy}`);
  console.log('');
  console.log('Key metrics:');
  for (const entry of report.metrics.lines.metric_files) {
    console.log(`- ${entry.label}: ${entry.lines === null ? 'missing' : entry.lines} lines (${entry.file})`);
  }
  console.log(`- Tauri commands: ${report.metrics.commands.total} total; ${report.metrics.commands.lib_rs} in lib.rs`);
  console.log(`- Sidecar JSON kinds: ${report.metrics.sidecars.detected_count} detected; ${report.metrics.sidecars.unknown.length} unknown`);
  console.log(`- Ratchet files: ${report.metrics.lines.ratchet_files.length}`);
  console.log(`- Gate script lines: ${report.metrics.gate_script_lines || 'unavailable'}`);
  console.log('');
  console.log('Ratchet waterlines:');
  for (const entry of report.metrics.lines.ratchet_files) {
    console.log(`- ${entry.file}: ${entry.lines === null ? 'missing' : entry.lines}/${entry.waterline} (${entry.status})`);
  }
  console.log('');
  console.log('Sidecar JSON baseline:');
  for (const name of report.metrics.sidecars.names) {
    const marker = report.metrics.sidecars.unknown.includes(name) ? 'unknown' : 'allowed';
    console.log(`- ${name} (${marker})`);
  }
  console.log('');
  console.log('Findings:');
  if (report.findings.length === 0) {
    console.log('- none');
  } else {
    for (const finding of report.findings) {
      const detail = finding.detail ? ` ${JSON.stringify(finding.detail)}` : '';
      console.log(`- [${finding.severity}] ${finding.id}: ${finding.message}${detail}`);
    }
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (args.mode === 'check' && report.summary.status === 'fail') process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
