'use strict';

const crypto = require('node:crypto');

// Adaptive Harness v0.5 — required verification 选择审计（AH-050-09）
//
// 需求溯源：TS-4 · G-18
//
// 这是一个纯判定模块：
//   * changedPaths 与 trackedTestPaths 必须来自调用方刚取得的 Git 事实；
//   * selection.basis 是作出选择时看到的那份快照，必须与现实逐项相等；
//   * 不解析 command，更不会由一段 shell 文本猜测“测了什么”；
//   * ORDINARY_LOCAL 的 focused verification 必须非空、真实存在、覆盖每一条
//     changed path，并且是完整 tracked test inventory 的严格子集；
//   * STRICT_LOCAL，或带结构化 HIGH 风险声明的 ORDINARY_LOCAL，才可选择全量。
//
// 本文件不读文件、不调用 Git、不启动子进程，也不写任何状态轴。

const VERIFICATION_SELECTION_REJECTION_CODES = Object.freeze([
  'PROFILE_UNSUPPORTED',
  'BASIS_CHANGED_PATHS_DRIFT',
  'BASIS_TEST_INVENTORY_DRIFT',
  'MODE_UNSUPPORTED',
  'CHANGED_PATHS_EMPTY',
  'TEST_INVENTORY_EMPTY',
  'SELECTED_TESTS_EMPTY',
  'REQUIRED_VERIFICATION_BINDING_INVALID',
  'ALL_TRACKED_MODE_MISMATCH',
  'SELECTED_TEST_DUPLICATE',
  'SELECTED_TEST_UNKNOWN',
  'FOCUSED_NOT_STRICT_SUBSET',
  'ASSOCIATION_CHANGED_PATH_UNKNOWN',
  'ASSOCIATION_TEST_UNSELECTED',
  'CHANGED_PATH_UNCOVERED',
  'FULL_SUITE_REASON_MISSING',
  'FULL_SUITE_NOT_ALLOWED',
  'HIGH_RISK_DECLARATION_INVALID',
  'FULL_SUITE_INCOMPLETE',
]);

const SUPPORTED_PROFILES = Object.freeze(['ORDINARY_LOCAL', 'STRICT_LOCAL']);
const SELECTION_MODES = Object.freeze(['FOCUSED', 'FULL_SUITE']);

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function normalizePath(value) {
  return text(value).replace(/\\/g, '/').replace(/^\.\//, '').replace(/\/+/g, '/');
}

function pathList(value) {
  if (!Array.isArray(value)) return [];
  return value.map(normalizePath).filter(Boolean);
}

// node-schema 的 canonical serializer 无法无损往返 verification[].run 里的嵌套
// 数组（合法 Git 路径可含逗号）。持久节点因此使用显式 JSON: 前缀标量；内存调用
// 仍可传数组。两者都逐项规范化，其余字符串与对象 fail closed。
function runTestPathList(value) {
  if (Array.isArray(value)) {
    const paths = pathList(value);
    return {
      ok: paths.length === value.length,
      paths,
      source: 'ARRAY',
    };
  }
  if (typeof value === 'string' && value.startsWith('JSON:')) {
    let raw;
    try {
      raw = JSON.parse(value.slice('JSON:'.length));
    } catch {
      return { ok: false, paths: [], source: 'CANONICAL_JSON' };
    }
    if (!Array.isArray(raw)) return { ok: false, paths: [], source: 'CANONICAL_JSON' };
    const paths = pathList(raw);
    return {
      ok: paths.length === raw.length,
      paths,
      source: 'CANONICAL_JSON',
    };
  }
  return { ok: false, paths: [], source: 'UNSUPPORTED' };
}

function unique(values) {
  return [...new Set(values)];
}

// closeout 只保存这枚内容指纹，不复制整仓测试清单。排序使 Git 返回顺序的变化
// 不会形成假 drift；重复项仍会进入序列，因此不会被悄悄吞掉。
function fingerprintPaths(values) {
  const normalized = pathList(values).slice().sort();
  return `sha256:${crypto.createHash('sha256').update(JSON.stringify(normalized)).digest('hex')}`;
}

function fingerprintCommand(value) {
  return `sha256:${crypto.createHash('sha256').update(text(value)).digest('hex')}`;
}

function sameMultiset(left, right) {
  const a = left.slice().sort();
  const b = right.slice().sort();
  return a.length === b.length && a.every((entry, index) => entry === b[index]);
}

function problem(code, message, details) {
  return {
    code,
    message,
    ...(details && typeof details === 'object' ? details : {}),
  };
}

function validHighRiskDeclaration(value) {
  return Boolean(value)
    && typeof value === 'object'
    && value.level === 'HIGH'
    && text(value.reason) !== '';
}

/**
 * 审计 required verification 的结构化选择。
 *
 * input:
 *   profile: ORDINARY_LOCAL | STRICT_LOCAL
 *   changedPaths: Git 现场得到的真实改动路径
 *   trackedTestPaths: task HEAD 上真实存在、受 Git 跟踪的完整测试 inventory
 *   requiredVerifications: node.verification 中 required=true 的真实条目
 *   selection:
 *     mode: FOCUSED | FULL_SUITE
 *     selectedTests: FOCUSED 为显式路径数组；FULL_SUITE 可写 "ALL_TRACKED"
 *     associations: [{ changedPath, tests: [...] }]
 *     basis: { changedPaths: [...], trackedTestInventoryFingerprint: "sha256:..." }
 *     requiredVerificationIds: 必须与 requiredVerifications 的 id 集合精确相等
 *     fullSuiteReason: FULL_SUITE 时必填
 *     risk: ORDINARY_LOCAL 选择 FULL_SUITE 时必填 { level: 'HIGH', reason }
 */
function auditVerificationSelection(input) {
  const settings = input && typeof input === 'object' ? input : {};
  const selection = settings.selection && typeof settings.selection === 'object'
    ? settings.selection
    : {};
  const profile = text(settings.profile);
  const mode = text(selection.mode);
  const changedPathsRaw = pathList(settings.changedPaths);
  const trackedTestsRaw = pathList(settings.trackedTestPaths);
  const changedPaths = unique(changedPathsRaw);
  const trackedTests = unique(trackedTestsRaw);
  const allTrackedRequested = selection.selectedTests === 'ALL_TRACKED';
  const selectedTestsRaw = allTrackedRequested && mode === 'FULL_SUITE'
    ? trackedTestsRaw.slice()
    : pathList(selection.selectedTests);
  const selectedTests = unique(selectedTestsRaw);
  const basis = selection.basis && typeof selection.basis === 'object' ? selection.basis : {};
  const basisChangedPaths = pathList(basis.changedPaths);
  const basisTrackedTestFingerprint = text(basis.trackedTestInventoryFingerprint);
  const associations = Array.isArray(selection.associations) ? selection.associations : [];
  const requiredVerifications = Array.isArray(settings.requiredVerifications)
    ? settings.requiredVerifications.filter((entry) => entry && entry.required === true)
    : [];
  const requiredVerificationIds = requiredVerifications.map((entry) => text(entry.id)).filter(Boolean);
  const selectedVerificationIds = Array.isArray(selection.requiredVerificationIds)
    ? selection.requiredVerificationIds.map(text).filter(Boolean)
    : [];
  const problems = [];

  if (!SUPPORTED_PROFILES.includes(profile)) {
    problems.push(problem(
      'PROFILE_UNSUPPORTED',
      `verification selection 不接受 profile=${profile || '(empty)'}`,
      { profile: profile || null },
    ));
  }

  if (!sameMultiset(changedPathsRaw, basisChangedPaths)) {
    problems.push(problem(
      'BASIS_CHANGED_PATHS_DRIFT',
      'selection.basis.changedPaths 与退场现场的真实 changed paths 不一致',
      { expected: changedPathsRaw, observed: basisChangedPaths },
    ));
  }

  const actualTrackedTestFingerprint = fingerprintPaths(trackedTestsRaw);
  if (basisTrackedTestFingerprint !== actualTrackedTestFingerprint) {
    problems.push(problem(
      'BASIS_TEST_INVENTORY_DRIFT',
      'selection.basis.trackedTestInventoryFingerprint 与 task HEAD 上的真实 tracked test inventory 不一致',
      { expected: actualTrackedTestFingerprint, observed: basisTrackedTestFingerprint || null },
    ));
  }

  if (!SELECTION_MODES.includes(mode)) {
    problems.push(problem(
      'MODE_UNSUPPORTED',
      `verification selection mode=${mode || '(empty)'} 不在 FOCUSED / FULL_SUITE 之内`,
      { mode: mode || null },
    ));
  }

  if (changedPaths.length === 0) {
    problems.push(problem('CHANGED_PATHS_EMPTY', '没有真实 changed path，不能构造 diff-based required verification'));
  }

  if (trackedTests.length === 0) {
    problems.push(problem('TEST_INVENTORY_EMPTY', '真实 tracked test inventory 为空'));
  }

  if (selectedTests.length === 0) {
    problems.push(problem('SELECTED_TESTS_EMPTY', 'required verification 没有选择任何真实测试'));
  }

  const runBindings = [];
  const bindingProblems = [];
  if (requiredVerificationIds.length === 0) {
    bindingProblems.push('node 中没有 required=true 的 verification 条目');
  }
  if (!sameMultiset(requiredVerificationIds, selectedVerificationIds)) {
    bindingProblems.push(
      `selection.requiredVerificationIds=${JSON.stringify(selectedVerificationIds)}`
      + ` 与真实 required ids=${JSON.stringify(requiredVerificationIds)} 不一致`,
    );
  }

  let executedAllTracked = false;
  const executedTestPaths = [];
  for (const entry of requiredVerifications) {
    const id = text(entry.id);
    const command = text(entry.command);
    const run = entry.run && typeof entry.run === 'object' && !Array.isArray(entry.run)
      ? entry.run
      : null;
    if (!id || !command || !run) {
      bindingProblems.push(`${id || '(missing id)'} 缺 command 或真实 run 元数据`);
      continue;
    }
    const expectedCommandDigest = fingerprintCommand(command);
    if (text(run['command-digest']) !== expectedCommandDigest) {
      bindingProblems.push(`${id} 的 run.command-digest 没有绑定实际 command`);
    }
    const declaredTestPaths = run['test-paths'];
    if (declaredTestPaths === 'ALL_TRACKED') {
      executedAllTracked = true;
      runBindings.push({ id, commandDigest: expectedCommandDigest, testPaths: 'ALL_TRACKED' });
      continue;
    }
    const decoded = runTestPathList(declaredTestPaths);
    if (!decoded.ok) {
      bindingProblems.push(`${id} 的 run.test-paths 必须是路径数组、JSON: 数组标量或 ALL_TRACKED`);
      continue;
    }
    const normalizedRunTests = decoded.paths;
    executedTestPaths.push(...normalizedRunTests);
    runBindings.push({
      id,
      commandDigest: expectedCommandDigest,
      testPaths: normalizedRunTests,
      source: decoded.source,
    });
  }

  const uniqueExecutedTests = unique(executedTestPaths);
  if (mode === 'FOCUSED') {
    if (executedAllTracked || !sameMultiset(uniqueExecutedTests, selectedTests)) {
      bindingProblems.push(
        'FOCUSED 的 selectedTests 必须与所有 required run 实际绑定的 test-paths 精确相等',
      );
    }
  }
  if (mode === 'FULL_SUITE'
    && !executedAllTracked
    && !sameMultiset(uniqueExecutedTests, trackedTests)) {
    bindingProblems.push(
      'FULL_SUITE 必须由 required run 的 ALL_TRACKED 或完整真实 inventory 证明',
    );
  }
  if (bindingProblems.length > 0) {
    problems.push(problem(
      'REQUIRED_VERIFICATION_BINDING_INVALID',
      `required verification 与 selection/run 未形成同一份结构化事实：${bindingProblems.join('；')}`,
      { issues: bindingProblems },
    ));
  }

  if (allTrackedRequested && mode !== 'FULL_SUITE') {
    problems.push(problem(
      'ALL_TRACKED_MODE_MISMATCH',
      'selectedTests="ALL_TRACKED" 只可用于 FULL_SUITE；FOCUSED 必须显式列出测试路径',
    ));
  }

  if (selectedTestsRaw.length !== selectedTests.length) {
    const duplicates = selectedTests.filter(
      (entry) => selectedTestsRaw.indexOf(entry) !== selectedTestsRaw.lastIndexOf(entry),
    );
    problems.push(problem(
      'SELECTED_TEST_DUPLICATE',
      `selectedTests 含重复路径：${duplicates.join('，')}`,
      { paths: duplicates },
    ));
  }

  const trackedSet = new Set(trackedTests);
  const selectedSet = new Set(selectedTests);
  const unknownTests = selectedTests.filter((filePath) => !trackedSet.has(filePath));
  if (unknownTests.length > 0) {
    problems.push(problem(
      'SELECTED_TEST_UNKNOWN',
      `selectedTests 含不在真实 tracked inventory 内的路径：${unknownTests.join('，')}`,
      { paths: unknownTests },
    ));
  }

  if (mode === 'FOCUSED') {
    if (selectedTests.length >= trackedTests.length) {
      problems.push(problem(
        'FOCUSED_NOT_STRICT_SUBSET',
        `FOCUSED 选择了 ${selectedTests.length}/${trackedTests.length} 条，不是完整测试 inventory 的严格子集`,
        { selectedCount: selectedTests.length, inventoryCount: trackedTests.length },
      ));
    }

    const changedSet = new Set(changedPaths);
    const covered = new Set();
    for (const association of associations) {
      const changedPath = normalizePath(association && association.changedPath);
      const tests = pathList(association && association.tests);
      if (!changedSet.has(changedPath)) {
        problems.push(problem(
          'ASSOCIATION_CHANGED_PATH_UNKNOWN',
          `association 指向不在真实 diff 内的 changedPath：${changedPath || '(empty)'}`,
          { path: changedPath || null },
        ));
        continue;
      }
      let hasSelectedRealTest = false;
      for (const testPath of tests) {
        if (!selectedSet.has(testPath)) {
          problems.push(problem(
            'ASSOCIATION_TEST_UNSELECTED',
            `${changedPath} 关联了未被 selectedTests 选中的测试：${testPath}`,
            { changedPath, testPath },
          ));
          continue;
        }
        if (trackedSet.has(testPath)) hasSelectedRealTest = true;
      }
      if (hasSelectedRealTest) covered.add(changedPath);
    }

    const uncovered = changedPaths.filter((filePath) => !covered.has(filePath));
    if (uncovered.length > 0) {
      problems.push(problem(
        'CHANGED_PATH_UNCOVERED',
        `真实 diff 中有路径没有关联到本次选中的 tracked test：${uncovered.join('，')}`,
        { paths: uncovered },
      ));
    }
  }

  if (mode === 'FULL_SUITE') {
    if (text(selection.fullSuiteReason) === '') {
      problems.push(problem('FULL_SUITE_REASON_MISSING', 'FULL_SUITE 必须给出结构化 fullSuiteReason'));
    }

    const riskWasSupplied = selection.risk !== undefined && selection.risk !== null;
    const highRisk = validHighRiskDeclaration(selection.risk);
    if (riskWasSupplied && !highRisk) {
      problems.push(problem(
        'HIGH_RISK_DECLARATION_INVALID',
        'risk 必须是 { level: "HIGH", reason: 非空字符串 }',
      ));
    }
    if (profile !== 'STRICT_LOCAL' && !highRisk) {
      problems.push(problem(
        'FULL_SUITE_NOT_ALLOWED',
        '只有 STRICT_LOCAL，或带有效 HIGH 风险声明的 ORDINARY_LOCAL，才可选择 FULL_SUITE',
      ));
    }

    if (!sameMultiset(selectedTests, trackedTests)) {
      const missing = trackedTests.filter((filePath) => !selectedSet.has(filePath));
      const extra = selectedTests.filter((filePath) => !trackedSet.has(filePath));
      problems.push(problem(
        'FULL_SUITE_INCOMPLETE',
        `FULL_SUITE 与真实 inventory 不相等；缺 ${missing.length} 条，多 ${extra.length} 条`,
        { missing, extra },
      ));
    }
  }

  const resolved = {
    profile,
    mode,
    changedPaths,
    trackedTestPaths: trackedTests,
    trackedTestInventoryFingerprint: actualTrackedTestFingerprint,
    selectedTests,
    requiredVerificationIds,
    runBindings,
    associations: associations.map((association) => ({
      changedPath: normalizePath(association && association.changedPath) || null,
      tests: pathList(association && association.tests),
    })),
    fullSuiteReason: text(selection.fullSuiteReason) || null,
    risk: validHighRiskDeclaration(selection.risk)
      ? { level: 'HIGH', reason: text(selection.risk.reason) }
      : null,
  };

  return {
    allowed: problems.length === 0,
    problems,
    resolved,
    unresolvedCount: problems.length,
  };
}

module.exports = {
  VERIFICATION_SELECTION_REJECTION_CODES,
  SUPPORTED_PROFILES,
  SELECTION_MODES,
  fingerprintPaths,
  fingerprintCommand,
  auditVerificationSelection,
};
