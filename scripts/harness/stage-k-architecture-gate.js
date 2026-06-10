#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const TEXT_EXTENSIONS = new Set(['.rs', '.ts', '.tsx', '.js', '.jsx', '.md']);
const EXCLUDED_DIRS = new Set(['node_modules', 'target', 'dist', '.git']);

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    json: false,
    strict: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function walkFiles(root, files = []) {
  if (!fs.existsSync(root)) return files;
  const entries = fs.readdirSync(root, { withFileTypes: true });
  for (const entry of entries) {
    if (EXCLUDED_DIRS.has(entry.name)) continue;
    const fullPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      walkFiles(fullPath, files);
    } else if (TEXT_EXTENSIONS.has(path.extname(entry.name))) {
      files.push(fullPath);
    }
  }
  return files;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function read(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function add(report, severity, id, message, file, line = null, detail = null) {
  report.findings.push({
    severity,
    id,
    message,
    file,
    line,
    detail
  });
}

function lineOf(text, index) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function matches(text, pattern) {
  const results = [];
  const regex = pattern.global ? pattern : new RegExp(pattern.source, `${pattern.flags}g`);
  let match;
  while ((match = regex.exec(text)) !== null) {
    results.push({ index: match.index, value: match[0] });
    if (match[0].length === 0) regex.lastIndex += 1;
  }
  return results;
}

function scanRawCodexSpawn(report, root, filePath, text) {
  for (const match of matches(text, /Command::new\("codex"\)/g)) {
    const relative = rel(root, filePath);
    if (relative.startsWith('docs/')) {
      add(report, 'info', 'documented_codex_spawn_boundary', '文档中提到裸 Codex spawn 边界，不作为源码违规。', relative, lineOf(text, match.index));
      continue;
    }
    if (relative === 'prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs') {
      add(report, 'info', 'approved_codex_spawn', '裸 Codex spawn 仅位于批准 runner。', relative, lineOf(text, match.index));
    } else {
      add(report, 'error', 'raw_codex_spawn_outside_runner', '裸 Command::new("codex") 不得出现在批准 runner 之外。', relative, lineOf(text, match.index));
    }
  }
}

function scanLegacyFrontendCalls(report, root, filePath, text) {
  const relative = rel(root, filePath);
  if (!relative.startsWith('prototypes/productized-desktop-shell/src/')) return;
  if (relative === 'prototypes/productized-desktop-shell/src/lib/tauri.ts') return;

  const patterns = [
    ['legacy_workflow_dispatch_ui_call', /\bexecuteLegacyWorkflowNodeDispatch\s*\(/g],
    ['legacy_workflow_machine_ui_call', /\brunLegacyWorkflowMachine\s*\(/g],
    ['sealed_canvas_start_ui_call', /\bcanvasStartRun\s*\(/g],
    ['sealed_canvas_tick_ui_call', /\bcanvasTickRun\s*\(/g]
  ];

  for (const [id, pattern] of patterns) {
    for (const match of matches(text, pattern)) {
      add(report, 'error', id, '普通 UI 不应调用 legacy / sealed real-run wrapper。', relative, lineOf(text, match.index), match.value);
    }
  }
}

function scanPromptBody(report, root, filePath, text) {
  const relative = rel(root, filePath);
  if (!/prompt_body/.test(text)) return;
  const testStart = text.search(/#\[cfg\(test\)\]|mod tests\s*\{/);
  if (relative.startsWith('docs/')) {
    add(report, 'info', 'prompt_body_documented_boundary', '文档中描述 prompt_body 边界，不作为持久化风险。', relative);
    return;
  }
  const firstPromptBody = text.search(/prompt_body/);
  if (testStart >= 0 && firstPromptBody > testStart) {
    add(report, 'info', 'prompt_body_test_boundary', '测试模块中出现 prompt_body，用于边界断言。', relative);
    return;
  }
  const allowed = [
    'prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/commands.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/types.rs',
    'prototypes/productized-desktop-shell/src/lib/types.ts',
    'prototypes/productized-desktop-shell/src/views/AgentView.tsx'
  ];
  if (allowed.includes(relative)) {
    add(report, 'info', 'prompt_body_runtime_boundary', 'prompt_body 仅允许作为 Phase B runtime input 或类型/UI 入参边界出现。', relative);
    return;
  }
  add(report, 'warn', 'prompt_body_unclassified_location', '发现 prompt_body 未分类位置，需要确认没有持久化 prompt body。', relative);
}

function scanReadbackZero(report, root, filePath, text) {
  const relative = rel(root, filePath);
  const testStart = text.search(/#\[cfg\(test\)\]|mod tests\s*\{/);
  const patterns = [
    /result_count\s*:\s*0/g,
    /result_count\s*=\s*Some\(0\)/g,
    /readback_result_count\s*:\s*Some\(0\)/g
  ];
  for (const pattern of patterns) {
    for (const match of matches(text, pattern)) {
      if (testStart >= 0 && match.index > testStart) {
        add(report, 'info', 'readback_zero_count_test_fixture', '测试 fixture 中出现 result_count=0；仍需由产品代码把失败/不可用归一为 null。', relative, lineOf(text, match.index), match.value);
        continue;
      }
      add(report, 'warn', 'readback_zero_count_requires_review', 'readback unavailable / failed / timed_out 不能显示成真实 0 条；0 count 只能用于真实成功且可证明为空的场景。', relative, lineOf(text, match.index), match.value);
    }
  }
}

function scanFixtureConstants(report, root, filePath, text) {
  const relative = rel(root, filePath);
  const fixtureHits = matches(text, /\b(K2_|J2_B|J2-B|K3_B|K3-B|H5_LEVEL_B|PCR9)\b/g);
  if (!fixtureHits.length) return;
  const testStart = text.search(/#\[cfg\(test\)\]|mod tests\s*\{/);
  const allowed = [
    'prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs',
    'prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs'
  ];
  if (allowed.includes(relative) || relative.startsWith('tasks/') || relative.startsWith('evidence/') || relative.startsWith('handoffs/') || relative.startsWith('docs/') || (testStart >= 0 && fixtureHits[0].index > testStart)) {
    add(report, 'info', 'fixture_constant_boundary', '阶段/fixture 常量存在于 fixture、测试、任务或文档边界内。', relative);
  } else {
    add(report, 'warn', 'fixture_constant_in_product_surface', '阶段/fixture 常量出现在普通产品源码中，需确认未变成用户主路径。', relative);
  }
}

function scanFormalMemoryWording(report, root, filePath, text) {
  const relative = rel(root, filePath);
  if (!relative.startsWith('prototypes/productized-desktop-shell/src/')) return;
  const risky = matches(text, /候选已写入正式记忆|observation 已写入正式记忆|knowledge hit 已写入正式记忆|自动写正式记忆/g);
  for (const match of risky) {
    const line = text.split(/\r?\n/)[lineOf(text, match.index) - 1] || '';
    if (/不自动写正式记忆|不会自动写正式记忆|不能自动写正式记忆/.test(line)) {
      add(report, 'info', 'formal_memory_negative_boundary', '否定句明确说明不会自动写正式记忆。', relative, lineOf(text, match.index), match.value);
      continue;
    }
    add(report, 'error', 'candidate_or_observation_as_formal_memory', '候选 / observation / knowledge hit 不得被写成已自动进入正式记忆。', relative, lineOf(text, match.index), match.value);
  }
}

function buildReport(args) {
  const report = {
    target: args.target,
    strict: args.strict,
    boundary: 'Stage K architecture gate is read-only; it does not execute Codex, does not send prompts, and does not read/write /Users/yoyi/.codex.',
    findings: []
  };

  const roots = [
    path.join(args.target, 'prototypes/productized-desktop-shell/src-tauri/src'),
    path.join(args.target, 'prototypes/productized-desktop-shell/src'),
    path.join(args.target, 'docs/plans')
  ];

  for (const root of roots) {
    for (const filePath of walkFiles(root)) {
      const text = read(filePath);
      scanRawCodexSpawn(report, args.target, filePath, text);
      scanLegacyFrontendCalls(report, args.target, filePath, text);
      scanPromptBody(report, args.target, filePath, text);
      scanReadbackZero(report, args.target, filePath, text);
      scanFixtureConstants(report, args.target, filePath, text);
      scanFormalMemoryWording(report, args.target, filePath, text);
    }
  }

  const errors = report.findings.filter((finding) => finding.severity === 'error');
  const warnings = report.findings.filter((finding) => finding.severity === 'warn');
  report.summary = {
    error_count: errors.length,
    warning_count: warnings.length,
    info_count: report.findings.filter((finding) => finding.severity === 'info').length,
    status: errors.length > 0 || (args.strict && warnings.length > 0) ? 'fail' : 'pass'
  };

  return report;
}

function printReport(report) {
  console.log(`Stage K architecture gate: ${report.target}`);
  console.log(report.boundary);
  console.log(`Status: ${report.summary.status}`);
  console.log(`Errors: ${report.summary.error_count}`);
  console.log(`Warnings: ${report.summary.warning_count}`);
  console.log(`Info: ${report.summary.info_count}`);
  for (const finding of report.findings) {
    const location = finding.line ? `${finding.file}:${finding.line}` : finding.file;
    console.log(`- [${finding.severity}] ${finding.id} ${location} ${finding.message}`);
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.summary.status === 'fail') process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
