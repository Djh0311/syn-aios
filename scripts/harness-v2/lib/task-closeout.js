'use strict';

// Adaptive Harness v0.5 — TASK 退场编排的纯 after-image 与确认收据（AH-050-08）
//
// 需求溯源：KP-13（写前预览）、EX-3/EX-4（完整 closeout 与自由文本验收）、
// EX-5/EX-6（history / parked）、GIT-10（资源处置不等于物理删除）。
//
// 本模块不改变 lifecycle，也不执行任何 Git 写操作。CLI 先用这里生成完整
// after-image 和 receipt；store 只在 receipt、generation、source digest 都仍
// 相同时提交那一份 after-image。

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const context = require('./context');
const exitGate = require('./exit-gate');
const gitFacts = require('./git-facts');
const lifecycle = require('./lifecycle');
const assets = require('./assets');
const testAudit = require('./test-audit');
const evidenceTrace = require('./evidence-trace');
const routing = require('./routing');
const safeguards = require('./safeguards');

const RESOURCE_KEYS = Object.freeze([
  'task-branch',
  'worktree',
  'product-commit',
  'wip-commit',
]);

const LOCAL_INTEGRATE_ACTION = 'local-integrate';
const PHYSICAL_REMOVAL_ACTION = 'physical-removal';

function failure(code, error, detail) {
  return { ok: false, code, error, detail: detail || null };
}

function digest(text) {
  return crypto.createHash('sha256').update(String(text), 'utf8').digest('hex');
}

function loadCloseout(filePath, cwd) {
  if (typeof filePath !== 'string' || filePath.trim() === '') {
    return failure('CLOSEOUT_REQUIRED', 'tracked 退场需要 --closeout <JSON>');
  }
  const absolute = path.resolve(cwd || process.cwd(), filePath);
  let text;
  try {
    text = fs.readFileSync(absolute, 'utf8');
  } catch (error) {
    return failure('CLOSEOUT_READ_FAILED', `读不到 closeout ${absolute}：${error.message}`);
  }
  let closeout;
  try {
    closeout = JSON.parse(text);
  } catch (error) {
    return failure('CLOSEOUT_JSON_INVALID', `closeout 不是合法 JSON：${error.message}`);
  }
  if (!closeout || typeof closeout !== 'object' || Array.isArray(closeout)) {
    return failure('CLOSEOUT_JSON_INVALID', 'closeout 顶层必须是 JSON object');
  }
  return { ok: true, absolute, text, digest: digest(text), closeout };
}

function terminalResult(command, explicitResult) {
  if (command === 'cancel' || command === 'withdraw') return 'CANCELLED';
  if (command === 'stop') return 'STOPPED';
  if (command === 'split') return 'DECOMPOSED';
  if (command === 'replace') return 'SUPERSEDED';
  return explicitResult || null;
}

function withObservedBinding(node, facts) {
  const binding = node.git && typeof node.git === 'object' ? node.git : null;
  if (!binding) return { ...node };
  return {
    ...node,
    git: {
      ...binding,
      'product-commit': facts.productCommit === undefined
        ? binding['product-commit']
        : facts.productCommit,
      'wip-commit': facts.wipCommit === undefined
        ? binding['wip-commit']
        : facts.wipCommit,
      'no-product-change': facts.noProductChange === undefined
        ? binding['no-product-change'] === true
        : facts.noProductChange === true,
      disposition: facts.disposition === undefined
        ? binding.disposition
        : facts.disposition,
      'integrated-observed': facts.integratedNow === true,
    },
  };
}

function parkedChange(record, facts) {
  const node = withObservedBinding(record.node, facts || {});
  delete node.result;
  delete node['closed-at'];
  return {
    node: {
      ...node,
      lifecycle: 'PARKED',
    },
    body: record.body,
    previousPath: record.path,
  };
}

function lines(values, emptyText) {
  const list = Array.isArray(values) ? values : [];
  if (list.length === 0) return `- ${emptyText}`;
  return list.map((value) => {
    if (value && typeof value === 'object') return `- ${JSON.stringify(value)}`;
    return `- ${String(value)}`;
  }).join('\n');
}

// HISTORY 是长期可读面。子执行者 summary 与 trace location 都来自 closeout /
// Git 原件的外部文本，写入前必须复用 KP-16 的统一脱敏器，不能因为它们已经
// “核验过”就把敏感片段原样长期持久化。
function redactHistoryText(value) {
  return routing.redactKnownSecrets(String(value === null || value === undefined ? '' : value));
}

function appendHistoryCloseout(body, closeout, node, evidenceFacts) {
  const source = String(body || '').replace(/\s+$/, '');
  const retention = closeout.retention && typeof closeout.retention === 'object'
    ? closeout.retention
    : {};
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const assetSummaries = (Array.isArray(closeout.assets) ? closeout.assets : [])
    .map((asset) => assets.summarizeForCurrent(asset));
  const agentSummaries = evidenceTrace.verifiedAgentSummaries(
    closeout,
    evidenceFacts && evidenceFacts.agentAudit,
  );
  const traceRefs = evidenceTrace.traceReferences(
    closeout,
    evidenceFacts && evidenceFacts.traceAudit,
  );
  const verifiedAgentBlock = agentSummaries.length === 0 ? [] : [
    '### 已核验子执行者摘要',
    '',
    ...agentSummaries.map((entry) => `- ${redactHistoryText(entry.id)}: ${redactHistoryText(entry.summary)}`),
    '',
  ];
  const traceBlock = traceRefs.length === 0 ? [] : [
    '### Verification 追溯引用',
    '',
    ...traceRefs.map((ref) => `- ${redactHistoryText(ref)}`),
    '',
  ];
  const block = [
    '## 退场记录',
    '',
    '### 结果',
    '',
    String(closeout.result || node.result || '（未声明）'),
    '',
    '### 验收标准逐条结论',
    '',
    lines(closeout.criteriaConclusions, '不适用'),
    '',
    '### 终止原因',
    '',
    String(closeout.terminationReason || '不适用'),
    '',
    '### 验收结论',
    '',
    String(closeout.acceptanceStatement || '').trim(),
    '',
    '### 未决问题',
    '',
    lines(closeout.openQuestions, '无'),
    '',
    '### 测试与证据资产摘要',
    '',
    lines(assetSummaries, '无'),
    '',
    ...verifiedAgentBlock,
    ...traceBlock,
    '### Git 与资源去向',
    '',
    `- disposition: ${binding.disposition || '（未声明）'}`,
    `- product commit: ${binding['product-commit'] || 'NO_PRODUCT_CHANGE'}`,
    `- WIP commit: ${binding['wip-commit'] || '无'}`,
    `- retention owner: ${retention.owner || '不适用'}`,
    `- retention reason: ${retention.reason || '不适用'}`,
    `- retention review-by: ${retention['review-by'] || '不适用'}`,
    '',
    '### 下一入口',
    '',
    String(closeout.nextEntry || 'idle'),
    '',
  ].join('\n');
  return `${source}\n\n${block}`;
}

function historyChange(decision, closeout, evidenceFacts) {
  const change = decision && Array.isArray(decision.changes) ? decision.changes[0] : null;
  if (!change || !change.node) {
    return failure('CLOSEOUT_AFTER_IMAGE_INVALID', '终态决策没有生成 HISTORY after-image');
  }
  const node = withObservedBinding(change.node, decision.facts || {});
  return {
    ok: true,
    change: {
      ...change,
      node,
      body: appendHistoryCloseout(change.body, closeout, node, evidenceFacts),
    },
  };
}

function currentAfterImage(index, record, closeout, worktreePath) {
  const resolved = context.resolveExitCurrent(index, {
    sourceTaskId: record.node.id,
    successorId: closeout.successorId,
  });
  const text = context.renderCurrent({
    index,
    activeLeafId: null,
    sourceTaskId: record.node.id,
    successorId: closeout.successorId,
    worktreePath,
  });
  return {
    nextEntry: resolved.entry.label,
    entry: resolved.entry,
    source: resolved.source,
    text,
  };
}

function buildGateInput(input) {
  const settings = input || {};
  const node = settings.node || {};
  const facts = settings.facts || {};
  const closeout = settings.closeout || {};
  const current = settings.current || {};
  const status = Array.isArray(facts.taskStatusNow) ? facts.taskStatusNow : [];
  // TRANSFERRED split/replace 的承接方 write-scope：退场门的越界、真值表与
  // 提交现实三项按「来源 + 全部承接方」的合并声明判定，规则与
  // task-lifecycle 的 transferredScopeAccountsForChanges 一致。
  const transferSuccessors = Array.isArray(settings.transferSuccessors) ? settings.transferSuccessors : [];
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const transferActive = binding.disposition === 'TRANSFERRED' && transferSuccessors.length > 0;
  const unionWriteScope = [];
  if (transferActive) {
    const scopes = [node['write-scope'], ...transferSuccessors.map((successor) => successor && successor['write-scope'])];
    for (const entries of scopes) {
      for (const entry of Array.isArray(entries) ? entries : []) {
        if (!unionWriteScope.includes(entry)) unionWriteScope.push(entry);
      }
    }
  }
  const transferClaim = { active: transferActive, unionWriteScope, successorIds: transferSuccessors.map((successor) => successor && successor.id).filter(Boolean) };
  const assetAudit = assets.auditAssetDispositions({
    changedPaths: Array.isArray(facts.changedPaths) ? facts.changedPaths : [],
    changedEntries: Array.isArray(facts.changedEntries) ? facts.changedEntries : [],
    assets: Array.isArray(closeout.assets) ? closeout.assets : [],
    addedIgnoreRules: Array.isArray(facts.addedIgnoreRules) ? facts.addedIgnoreRules : [],
    ignoreReasons: Array.isArray(closeout.ignoreRules) ? closeout.ignoreRules : [],
    untrackedEntries: facts.untrackedProductAudit
      && Array.isArray(facts.untrackedProductAudit.mustEnterRepository)
      ? facts.untrackedProductAudit.mustEnterRepository.map((entry) => ({
        ...entry,
        productDependency: true,
      }))
      : [],
    writeScope: Array.isArray(node['write-scope']) ? node['write-scope'] : [],
  });
  const trackedTestPaths = (Array.isArray(facts.trackedPaths) ? facts.trackedPaths : [])
    .filter((filePath) => assets.isTestFilePath(filePath));
  const zeroProductDiff = facts.noProductChange === true && facts.scopedDiffEmpty === true;
  const verificationAudit = zeroProductDiff
    ? {
      allowed: true,
      problems: [],
      resolved: {
        mode: 'NOT_APPLICABLE_ZERO_PRODUCT_DIFF',
        changedPaths: [],
        trackedTestPaths,
      },
      unresolvedCount: 0,
    }
    : testAudit.auditVerificationSelection({
      profile: node.profile,
      changedPaths: Array.isArray(facts.changedPaths) ? facts.changedPaths : [],
      trackedTestPaths,
      requiredVerifications: (Array.isArray(node.verification) ? node.verification : [])
        .filter((entry) => entry && entry.required === true),
      selection: closeout.verificationSelection,
    });
  return {
    node,
    closeout,
    git: {
      changedPaths: Array.isArray(facts.changedPaths) ? facts.changedPaths : [],
      controlPlaneExemptPrefixes: ['docs/harness/history'],
      headOid: facts.taskHeadOid || null,
      statusNow: status,
      statusAtInspect: status,
      scopedDiffEmpty: facts.scopedDiffEmpty === true,
      commitReachable: facts.productCommit
        ? facts.productCommitExists === true
        : facts.wipCommitExists === true,
      integratedNow: facts.integratedNow === true,
      wipCarriedIntoIntegration: facts.wipCarriedIntoIntegration === true,
    },
    assets: Array.isArray(closeout.assets) ? closeout.assets : [],
    assetAudit,
    verificationAudit,
    agentAudit: facts.agentAudit && typeof facts.agentAudit === 'object'
      ? facts.agentAudit
      : null,
    traceAudit: facts.traceAudit && typeof facts.traceAudit === 'object'
      ? facts.traceAudit
      : null,
    nonHistoryDescendants: Array.isArray(facts.nonHistoryDescendants)
      ? facts.nonHistoryDescendants
      : [],
    nonHistoryDependents: Array.isArray(facts.nonHistoryDependents)
      ? facts.nonHistoryDependents
      : [],
    current: { nextEntry: current.nextEntry || '' },
    truth: facts,
    transferClaim,
  };
}

function evaluateGate(input) {
  const gateInput = buildGateInput(input);
  const verdict = exitGate.evaluateLeafExit(gateInput);
  return { gateInput, verdict };
}

function stableReceiptPayload(writePlan, closeoutDigest, observed) {
  return {
    generation: writePlan.generation,
    closeoutDigest,
    observed: observed || {},
    entries: (writePlan.entries || []).map((entry) => ({
      id: entry.id,
      lifecycle: entry.lifecycle,
      target: entry.target,
      previousPath: entry.previousPath,
      expectedSourceDigest: entry.expectedSourceDigest,
      expectedTargetDigest: entry.expectedTargetDigest,
      digest: entry.digest,
    })),
    extraFiles: (writePlan.extraFiles || []).map((entry) => ({
      target: entry.target,
      expectedTargetDigest: entry.expectedTargetDigest,
      digest: digest(entry.text),
    })),
    guards: (writePlan.guardRecords || []).map((entry) => ({
      id: entry.id,
      path: entry.path,
      expectedDigest: entry.expectedDigest,
    })),
  };
}

function receiptFor(writePlan, closeoutDigest, observed) {
  return digest(JSON.stringify(stableReceiptPayload(writePlan, closeoutDigest, observed)));
}

function realpathOrNull(target) {
  try {
    return fs.realpathSync(String(target));
  } catch (error) {
    return null;
  }
}

function normalizedBranch(value) {
  return String(value || '').replace(/^refs\/heads\//, '').trim();
}

function fullCommitOid(cwd, value) {
  if (typeof value !== 'string' || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(value)) {
    return null;
  }
  const resolved = gitFacts.runGit([
    'rev-parse', '--verify', '--end-of-options', `${value}^{commit}`,
  ], { cwd });
  if (!resolved.ok) return null;
  const oid = resolved.stdout.trim();
  return oid.toLowerCase() === value.toLowerCase() ? oid : null;
}

function gitDirRealpath(cwd) {
  const result = gitFacts.runGit([
    'rev-parse', '--path-format=absolute', '--git-dir',
  ], { cwd });
  return result.ok ? realpathOrNull(result.stdout.trim()) : null;
}

function bindingOf(node) {
  return node && node.git && typeof node.git === 'object' && !Array.isArray(node.git)
    ? node.git
    : null;
}

function branchOid(cwd, branch) {
  const name = normalizedBranch(branch);
  if (name === '') return null;
  const result = gitFacts.runGit([
    'rev-parse', '--verify', '--end-of-options', `refs/heads/${name}^{commit}`,
  ], { cwd });
  return result.ok ? result.stdout.trim() : null;
}

function failureList(code, error, conflicts, facts) {
  return {
    ok: false,
    code,
    error,
    detail: { conflicts: Array.isArray(conflicts) ? conflicts : [], facts: facts || {} },
  };
}

/**
 * 读取已声明 task worktree 的 Git 现实。这里刻意不使用调用者 cwd：历史写入
 * 在 integration worktree 进行，TASK 的冻结身份仍必须在它自己的 worktree 中
 * 重新核实。目录不是仓库根、分支没实际 checkout、base 不可回查，都会失败。
 */
function inspectTaskBindingReality(input) {
  const settings = input || {};
  const node = settings.node || {};
  const binding = bindingOf(node);
  if (!binding) {
    return failureList('TASK_BINDING_MISSING', 'TASK 没有可核对的 git binding', [{
      code: 'TASK_BINDING_MISSING', field: 'git', message: 'TASK 没有可核对的 git binding',
    }]);
  }
  if (binding.disposition === 'REMOVED') {
    return failureList('TASK_REMOVED_REQUIRES_FROZEN_PROOF', 'REMOVED 的 task worktree 已不应再现场读取；必须走带冻结 head/status 证明的专门退场核验', [{
      code: 'TASK_REMOVED_REQUIRES_FROZEN_PROOF', field: 'git.disposition', message: 'REMOVED 不能把读不到的 task worktree 当作干净；需要冻结证明',
    }]);
  }

  const declaredWorktree = realpathOrNull(binding.worktree);
  const facts = {
    declaredWorktree,
    repoRoot: null,
    gitDir: null,
    commonDir: null,
    currentBranch: null,
    headOid: null,
    taskBranch: normalizedBranch(binding['task-branch']),
    taskBranchOid: null,
    baseBranch: normalizedBranch(binding['base-branch']),
    baseBranchOid: null,
    baseOid: null,
    statusNow: null,
  };
  const conflicts = [];
  if (!declaredWorktree) {
    conflicts.push({
      code: 'TASK_WORKTREE_UNRESOLVED', field: 'git.worktree',
      message: `声明的 task worktree ${binding.worktree || '（空）'} 不存在或无法 realpath`,
    });
    return failureList('TASK_BINDING_REALITY_CONFLICT', 'TASK binding 的 worktree 无法现场核对', conflicts, facts);
  }

  try {
    facts.repoRoot = realpathOrNull(gitFacts.repoRoot(declaredWorktree));
    facts.gitDir = gitDirRealpath(declaredWorktree);
    facts.commonDir = realpathOrNull(gitFacts.gitCommonDir(declaredWorktree));
    facts.headOid = gitFacts.headOid(declaredWorktree);
    const branch = gitFacts.runGit(['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: declaredWorktree });
    facts.currentBranch = branch.ok ? branch.stdout.trim() : null;
    facts.statusNow = gitFacts.porcelainStatus(declaredWorktree);
  } catch (error) {
    conflicts.push({
      code: 'TASK_WORKTREE_GIT_UNREADABLE', field: 'git.worktree',
      message: `读不到声明 task worktree 的 Git 现实：${error.message}`,
    });
    return failureList('TASK_BINDING_REALITY_CONFLICT', 'TASK binding 的 Git 现实不可读', conflicts, facts);
  }

  if (facts.repoRoot !== declaredWorktree) {
    conflicts.push({
      code: 'TASK_WORKTREE_NOT_REPO_ROOT', field: 'git.worktree',
      message: `声明 worktree ${declaredWorktree} 不是 Git checkout root ${facts.repoRoot || 'unknown'}；拒绝子目录伪装`,
    });
  }
  if (!facts.gitDir || !facts.commonDir) {
    conflicts.push({
      code: 'TASK_GIT_DIR_UNRESOLVED', field: 'git.worktree',
      message: '无法解析 task worktree 的 git-dir/common-dir',
    });
  }
  facts.taskBranchOid = branchOid(declaredWorktree, binding['task-branch']);
  if (!facts.taskBranch || !facts.taskBranchOid) {
    conflicts.push({
      code: 'TASK_BRANCH_UNRESOLVED', field: 'git.task-branch',
      message: `声明的 task-branch ${binding['task-branch'] || '（空）'} 无法解析`,
    });
  }
  if (facts.currentBranch !== facts.taskBranch) {
    conflicts.push({
      code: 'TASK_WORKTREE_BRANCH_MISMATCH', field: 'git.task-branch',
      message: `声明 task worktree 实际 checkout ${facts.currentBranch || 'detached/unknown'}，不是 ${facts.taskBranch || '（空）'}`,
    });
  }
  if (!facts.headOid || !facts.taskBranchOid || facts.headOid !== facts.taskBranchOid) {
    conflicts.push({
      code: 'TASK_WORKTREE_HEAD_MISMATCH', field: 'git.task-branch',
      message: `task worktree HEAD ${facts.headOid || 'unknown'} 与 task branch ${facts.taskBranchOid || 'unknown'} 不一致`,
    });
  }

  facts.baseOid = fullCommitOid(declaredWorktree, binding['base-oid']);
  facts.baseBranchOid = branchOid(declaredWorktree, binding['base-branch']);
  if (!facts.baseOid) {
    conflicts.push({
      code: 'TASK_BASE_OID_UNRESOLVED', field: 'git.base-oid',
      message: `冻结 base-oid ${binding['base-oid'] || '（空）'} 不是本仓完整可达 commit`,
    });
  }
  if (!facts.baseBranch || !facts.baseBranchOid) {
    conflicts.push({
      code: 'TASK_BASE_BRANCH_UNRESOLVED', field: 'git.base-branch',
      message: `冻结 base-branch ${binding['base-branch'] || '（空）'} 无法解析`,
    });
  }
  if (facts.baseOid && facts.taskBranchOid
    && !gitFacts.isAncestor(declaredWorktree, facts.baseOid, `refs/heads/${facts.taskBranch}`)) {
    conflicts.push({
      code: 'TASK_BRANCH_NOT_DESCENDED_FROM_BASE', field: 'git.base-oid',
      message: `task branch ${facts.taskBranch} 不包含冻结 base-oid ${facts.baseOid}`,
    });
  }
  if (facts.baseOid && facts.baseBranchOid
    && !gitFacts.isAncestor(declaredWorktree, facts.baseOid, `refs/heads/${facts.baseBranch}`)) {
    conflicts.push({
      code: 'TASK_BASE_BRANCH_DIVERGED', field: 'git.base-branch',
      message: `当前 base branch ${facts.baseBranch} 不包含冻结 base-oid ${facts.baseOid}`,
    });
  }

  return conflicts.length === 0
    ? { ok: true, facts }
    : failureList('TASK_BINDING_REALITY_CONFLICT', 'TASK binding 与声明 worktree 的 Git 现实不一致', conflicts, facts);
}

function physicalRemovalTarget(binding) {
  return JSON.stringify({
    taskBranch: normalizedBranch(binding && binding['task-branch']),
    worktree: String((binding && binding.worktree) || ''),
  });
}

function confirmationFailure(code, error, detail) {
  return failure(code, error, detail);
}

function requireExactPhysicalRemovalAuthorization(closeout, binding, cwd) {
  const target = physicalRemovalTarget(binding);
  const confirmations = closeout && Array.isArray(closeout.confirmations)
    ? closeout.confirmations
    : [];
  const authorization = safeguards.requireFreshAuthorization({
    action: PHYSICAL_REMOVAL_ACTION,
    target,
    confirmations,
  });
  if (!authorization.authorized || !authorization.confirmation) {
    return confirmationFailure(
      'PHYSICAL_REMOVAL_CONFIRMATION_REQUIRED',
      'REMOVED 需要本次、未消耗且精确指向 task branch/worktree 的 physical-removal 授权',
      { expectedTarget: target, refusal: authorization.refusal || null },
    );
  }
  const confirmation = authorization.confirmation;
  const expectedBranch = normalizedBranch(binding['task-branch']);
  if (confirmation.target !== target
    || normalizedBranch(confirmation['task-branch']) !== expectedBranch
    || String(confirmation.worktree || '') !== String(binding.worktree || '')) {
    return confirmationFailure(
      'PHYSICAL_REMOVAL_CONFIRMATION_TARGET_MISMATCH',
      'physical-removal 授权没有精确匹配声明的 task branch 与 worktree',
      { expectedTarget: target, confirmation },
    );
  }
  const proofHead = fullCommitOid(cwd, confirmation['task-head-oid']);
  const expectedHeadValue = binding['wip-commit']
    || binding['product-commit']
    || binding['base-oid']
    || null;
  const expectedHead = fullCommitOid(cwd, expectedHeadValue);
  const proofStatus = confirmation['task-status'];
  const proofCommonDir = realpathOrNull(confirmation['git-common-dir']);
  const capturedAt = confirmation['captured-at'];
  const capturedAtMillis = typeof capturedAt === 'string' ? Date.parse(capturedAt) : Number.NaN;
  if (!proofHead || !expectedHead || proofHead !== expectedHead
    || !Array.isArray(proofStatus) || proofStatus.length !== 0
    || !proofCommonDir
    || !Number.isFinite(capturedAtMillis)) {
    return confirmationFailure(
      'PHYSICAL_REMOVAL_FROZEN_PROOF_REQUIRED',
      'REMOVED 需要删除前冻结的精确 task tip、空 task status、git common dir 与 captured-at；task tip 必须等于 binding 中的 WIP/product/base 终点',
      { confirmation, expectedTaskHeadOid: expectedHead || expectedHeadValue },
    );
  }
  return {
    ok: true,
    authorization,
    proof: {
      taskHeadOid: proofHead,
      taskStatusNow: proofStatus.slice(),
      commonDir: proofCommonDir,
      capturedAt,
    },
  };
}

function localIntegrateExpected(binding, integrationReality) {
  return {
    target: integrationReality.integrationRef,
    taskBranch: normalizedBranch(binding && binding['task-branch']),
    taskWorktree: integrationReality.taskWorktree,
    integrationWorktree: integrationReality.repoRoot,
    integrationOid: integrationReality.refOid,
  };
}

function requireExactLocalIntegrateAuthorization(closeout, binding, integrationReality) {
  const expected = localIntegrateExpected(binding, integrationReality);
  const confirmations = closeout && Array.isArray(closeout.confirmations)
    ? closeout.confirmations
    : [];
  const authorization = safeguards.requireFreshAuthorization({
    action: LOCAL_INTEGRATE_ACTION,
    target: expected.target,
    confirmations,
  });
  if (!authorization.authorized || !authorization.confirmation) {
    return confirmationFailure(
      'LOCAL_INTEGRATE_CONFIRMATION_REQUIRED',
      'canonical history 写入需要 closeout.confirmations 中本次、未消耗的 local-integrate 单独授权',
      { expected, refusal: authorization.refusal || null },
    );
  }
  const confirmation = authorization.confirmation;
  const exact = confirmation.target === expected.target
    && normalizedBranch(confirmation['task-branch']) === expected.taskBranch
    && String(confirmation['task-worktree'] || '') === String(expected.taskWorktree || '')
    && String(confirmation['integration-worktree'] || '') === String(expected.integrationWorktree || '')
    && String(confirmation['integration-oid'] || '') === String(expected.integrationOid || '');
  if (!exact) {
    return confirmationFailure(
      'LOCAL_INTEGRATE_CONFIRMATION_TARGET_MISMATCH',
      'local-integrate 授权必须精确绑定 task branch/worktree、integration checkout root 与已解析 OID',
      { expected, confirmation },
    );
  }
  return { ok: true, authorization, expected };
}

function consumeCloseoutAuthorization(node, authorization) {
  if (!authorization || !authorization.confirmation) return node;
  const existing = Array.isArray(node.confirmations) ? node.confirmations.slice() : [];
  return {
    ...node,
    confirmations: [...existing, { ...authorization.confirmation, consumed: true }],
  };
}

function sameResource(expected, actual, key) {
  if (expected === null || expected === undefined || expected === '') {
    return actual === null || actual === undefined || actual === '';
  }
  if (key !== 'worktree') return expected === actual;
  const left = realpathOrNull(expected);
  const right = realpathOrNull(actual);
  return left !== null && right !== null && left === right;
}

function verifyResourceDisposition(input) {
  const settings = input || {};
  const node = settings.node || {};
  const binding = node.git && typeof node.git === 'object' ? node.git : {};
  const disposition = binding.disposition;
  if (!lifecycle.RETIRING_DISPOSITIONS.includes(disposition)) {
    return failure('RESOURCE_DISPOSITION_INVALID', `资源 disposition ${disposition} 不在四值之内`);
  }
  if (disposition === 'REMOVED') {
    const confirmed = requireExactPhysicalRemovalAuthorization(settings.closeout, binding, settings.cwd);
    const taskBranch = normalizedBranch(binding['task-branch']);
    const branchExists = taskBranch
      ? gitFacts.runGit(
        ['rev-parse', '--verify', '--end-of-options', `refs/heads/${taskBranch}^{commit}`],
        { cwd: settings.cwd },
      ).ok
      : false;
    const pathExists = fs.existsSync(String(binding.worktree || ''));
    if (!confirmed.ok || branchExists || pathExists) {
      return failure(
        'RESOURCE_REMOVAL_REALITY_MISMATCH',
        'REMOVED 只记录已单独确认、已冻结删除前 head/status 且 postcheck 证明 branch/worktree 都不存在的现实；本命令不会执行删除',
        {
          confirmation: confirmed.ok ? 'ok' : confirmed.detail,
          branchExists,
          pathExists,
        },
      );
    }
    return { ok: true, confirmed: true, removalAuthorization: confirmed.authorization, removalProof: confirmed.proof };
  }

  const sourceReality = inspectTaskBindingReality({ node, cwd: settings.cwd });
  if (!sourceReality.ok) {
    return failure(
      'RESOURCE_DISPOSITION_REALITY_MISMATCH',
      '资源 disposition 需要声明 task branch/worktree 的实际 checkout、base 与 HEAD 全部仍可核对',
      sourceReality.detail,
    );
  }
  const branchExists = Boolean(sourceReality.facts.taskBranchOid);
  const worktreeRegistered = sourceReality.facts.repoRoot === sourceReality.facts.declaredWorktree;

  if (disposition === 'RETAINED' || disposition === 'READY_FOR_CONFIRMED_REMOVAL') {
    if (!branchExists || !worktreeRegistered) {
      return failure(
        'RESOURCE_DISPOSITION_REALITY_MISMATCH',
        `${disposition} 要求声明的 task branch 与 worktree 仍真实存在`,
        { branchExists, worktreeRegistered },
      );
    }
    return { ok: true, branchExists, worktreeRegistered };
  }

  if (disposition === 'TRANSFERRED') {
    const successorId = settings.closeout && settings.closeout.successorId;
    const successor = settings.index && settings.index.byId
      ? settings.index.byId.get(successorId)
      : null;
    const successorBinding = successor && successor.node && bindingOf(successor.node);
    const successorRelations = successor && successor.node && Array.isArray(successor.node.relations)
      ? successor.node.relations
      : [];
    const tracesSource = successorRelations.some((relation) => relation
      && (relation.type === 'SPLIT_FROM' || relation.type === 'REPLACES')
      && relation['target-id'] === node.id);
    const complete = successorBinding && RESOURCE_KEYS.every(
      (key) => sameResource(binding[key], successorBinding[key], key),
    );
    const successorReality = successor && successor.node
      ? inspectTaskBindingReality({ node: successor.node, cwd: settings.cwd })
      : null;
    if (!complete
      || successorBinding.disposition !== 'TRANSFERRED'
      || !tracesSource
      || !branchExists
      || !worktreeRegistered
      || !successorReality
      || !successorReality.ok) {
      return failure(
        'RESOURCE_TRANSFER_REALITY_MISMATCH',
        'TRANSFERRED 必须由可回查 relation、唯一完整承接资源、且 source/successor worktree 都实际 checkout 对应 task branch 的 successor 完成',
        {
          successorId: successorId || null,
          tracesSource,
          branchExists,
          worktreeRegistered,
          successorReality: successorReality && successorReality.detail ? successorReality.detail : null,
        },
      );
    }
    return { ok: true, successorId, sourceReality: sourceReality.facts, successorReality: successorReality.facts };
  }
}

module.exports = {
  RESOURCE_KEYS,
  LOCAL_INTEGRATE_ACTION,
  PHYSICAL_REMOVAL_ACTION,
  digest,
  loadCloseout,
  terminalResult,
  parkedChange,
  appendHistoryCloseout,
  historyChange,
  currentAfterImage,
  buildGateInput,
  evaluateGate,
  stableReceiptPayload,
  receiptFor,
  normalizedBranch,
  fullCommitOid,
  inspectTaskBindingReality,
  physicalRemovalTarget,
  requireExactPhysicalRemovalAuthorization,
  localIntegrateExpected,
  requireExactLocalIntegrateAuthorization,
  consumeCloseoutAuthorization,
  verifyResourceDisposition,
};
