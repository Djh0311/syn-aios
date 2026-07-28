'use strict';

// Adaptive Harness v0.5 — PENDING_START 固定控制记录（AH-050-06）
//
// PENDING_START 不是 TASK，也不是另一条 lifecycle。它只是 start 在创建 Git
// 资源之前留下的可恢复事实：一旦正常开工完成，store 会把它和 ACTIVE/CURRENT
// 放在同一笔事务里消费。这里完全不读写文件；文件系统的唯一写者仍是 store。
//
// 需求溯源：GIT-1 / GIT-2 / GIT-4、EX-9、WK-1、R2 §5.1 / §6.1 / §6.4。

const crypto = require('node:crypto');
const path = require('node:path');

const PENDING_START_SCHEMA = 'adaptive-harness/PENDING_START/v1';
const PENDING_START_STATUSES = Object.freeze([
  'PENDING_START',
  'START_FAILED',
  'BLOCKED',
  'READY_FOR_CONFIRMED_REMOVAL',
]);
const PENDING_START_STEP_NAMES = Object.freeze([
  'branch',
  'worktree',
  'taskPackage',
  'openingCommit',
]);
const PENDING_START_STEP_STATES = Object.freeze(['PENDING', 'DONE', 'FAILED']);
const PENDING_START_RECOVERY_ACTIONS = Object.freeze([
  'RESUME',
  'ADOPT',
  'PREPARE_CONFIRMED_REMOVAL',
]);

const RECORD_KEYS = Object.freeze([
  'schema',
  'id',
  'proposal',
  'parent',
  'base',
  'branch',
  'worktree',
  'declaration',
  'status',
  'steps',
  'resources',
  'digest',
]);
const MUTABLE_KEYS = new Set(['status', 'steps', 'resources', 'worktreeCanonicalPath']);

class PendingStartError extends Error {
  constructor(code, message, detail) {
    super(message);
    this.name = 'PendingStartError';
    this.code = code;
    this.detail = detail || null;
  }
}

function isPlainObject(value) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function invalid(message, detail) {
  throw new PendingStartError('PENDING_START_RECORD_INVALID', message, detail);
}

function assertExactKeys(value, allowed, label) {
  if (!isPlainObject(value)) invalid(`${label} 必须是对象`, { label });
  const expected = new Set(allowed);
  for (const key of Object.keys(value)) {
    if (!expected.has(key)) {
      invalid(`${label} 出现未登记字段 ${key}`, { label, key });
    }
  }
  for (const key of allowed) {
    if (!Object.prototype.hasOwnProperty.call(value, key)) {
      invalid(`${label} 缺少固定字段 ${key}`, { label, key });
    }
  }
}

function nonEmptyString(value, label) {
  if (typeof value !== 'string' || value.trim() === '' || value.includes('\0')) {
    invalid(`${label} 必须是非空字符串`, { label });
  }
  return value;
}

function safeId(value) {
  const id = nonEmptyString(value, 'id');
  if (id === '.' || id === '..' || id.includes('/') || id.includes('\\')) {
    invalid('id 必须是单一安全路径段', { id });
  }
  return id;
}

function sha256(value, label) {
  const digest = nonEmptyString(value, label);
  if (!/^[0-9a-f]{64}$/.test(digest)) {
    invalid(`${label} 必须是小写 64 位 SHA-256`, { label, digest });
  }
  return digest;
}

function gitOid(value, label) {
  const oid = nonEmptyString(value, label);
  if (!/^[0-9a-f]{40}(?:[0-9a-f]{24})?$/.test(oid)) {
    invalid(`${label} 必须是小写 40 或 64 位 Git OID`, { label, oid });
  }
  return oid;
}

function absolutePath(value, label, allowNull) {
  if (value === null && allowNull === true) return null;
  const candidate = nonEmptyString(value, label);
  if (!path.isAbsolute(candidate)) {
    invalid(`${label} 必须是绝对路径`, { label, value: candidate });
  }
  return candidate;
}

function stringList(value, label, nonEmpty) {
  if (!Array.isArray(value)) invalid(`${label} 必须是列表`, { label });
  if (nonEmpty === true && value.length === 0) {
    invalid(`${label} 不得为空；生成不出可用范围时必须停下`, { label });
  }
  const seen = new Set();
  return value.map((entry, index) => {
    const item = nonEmptyString(entry, `${label}[${index}]`);
    if (seen.has(item)) invalid(`${label} 不得重复声明 ${item}`, { label, item });
    seen.add(item);
    return item;
  });
}

function stableJson(value) {
  if (value === null) return 'null';
  if (typeof value === 'string') return JSON.stringify(value);
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) invalid('digest 输入不能包含非有限数值');
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map((item) => stableJson(item)).join(',')}]`;
  if (!isPlainObject(value)) invalid('digest 输入必须由 JSON 标量、数组和对象组成');
  return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(',')}}`;
}

function payloadWithoutDigest(record) {
  return {
    schema: record.schema,
    id: record.id,
    proposal: record.proposal,
    parent: record.parent,
    base: record.base,
    branch: record.branch,
    worktree: record.worktree,
    declaration: record.declaration,
    status: record.status,
    steps: record.steps,
    resources: record.resources,
  };
}

function digestPendingStartRecord(record) {
  return crypto.createHash('sha256').update(stableJson(payloadWithoutDigest(record)), 'utf8').digest('hex');
}

function canonicalProposal(value) {
  assertExactKeys(value, ['digest'], 'proposal');
  return { digest: sha256(value.digest, 'proposal.digest') };
}

function canonicalParent(value) {
  assertExactKeys(value, ['id', 'digest'], 'parent');
  return {
    id: safeId(value.id),
    digest: sha256(value.digest, 'parent.digest'),
  };
}

function canonicalBase(value) {
  assertExactKeys(value, ['oid'], 'base');
  return { oid: gitOid(value.oid, 'base.oid') };
}

function canonicalBranch(value) {
  assertExactKeys(value, ['name'], 'branch');
  const name = nonEmptyString(value.name, 'branch.name');
  if (name.startsWith('-') || name.includes('..') || /[~^:?*[\\\s]/.test(name)) {
    invalid('branch.name 不是安全的 Git branch 名', { name });
  }
  return { name };
}

function canonicalWorktree(value) {
  assertExactKeys(value, ['requestedPath', 'canonicalPath'], 'worktree');
  const requestedPath = absolutePath(value.requestedPath, 'worktree.requestedPath', false);
  const canonicalPath = absolutePath(value.canonicalPath, 'worktree.canonicalPath', true);
  return { requestedPath, canonicalPath };
}

function canonicalDeclaration(value) {
  assertExactKeys(value, ['writeScope', 'forbiddenScope', 'exclusiveResources'], 'declaration');
  return {
    writeScope: stringList(value.writeScope, 'declaration.writeScope', true),
    forbiddenScope: stringList(value.forbiddenScope, 'declaration.forbiddenScope', false),
    exclusiveResources: stringList(value.exclusiveResources, 'declaration.exclusiveResources', false),
  };
}

function canonicalStatus(value) {
  const status = nonEmptyString(value, 'status');
  if (!PENDING_START_STATUSES.includes(status)) {
    invalid(`status ${status} 不在固定取值域内`, { status });
  }
  return status;
}

function canonicalSteps(value) {
  assertExactKeys(value, PENDING_START_STEP_NAMES, 'steps');
  const steps = {};
  for (const name of PENDING_START_STEP_NAMES) {
    const state = nonEmptyString(value[name], `steps.${name}`);
    if (!PENDING_START_STEP_STATES.includes(state)) {
      invalid(`steps.${name} ${state} 不在固定取值域内`, { step: name, state });
    }
    steps[name] = state;
  }
  return steps;
}

function optionalResource(value, label, keys, canonicalize) {
  if (value === null) return null;
  assertExactKeys(value, keys, label);
  return canonicalize(value);
}

function canonicalResources(value, binding, worktree) {
  assertExactKeys(value, PENDING_START_STEP_NAMES, 'resources');
  const resources = {
    branch: optionalResource(value.branch, 'resources.branch', ['name'], (source) => {
      const name = nonEmptyString(source.name, 'resources.branch.name');
      if (name !== binding.name) {
        invalid('resources.branch.name 必须等于冻结的 branch.name', { expected: binding.name, actual: name });
      }
      return { name };
    }),
    worktree: optionalResource(value.worktree, 'resources.worktree', ['path', 'canonicalPath'], (source) => {
      const resourcePath = absolutePath(source.path, 'resources.worktree.path', false);
      const canonicalPath = absolutePath(source.canonicalPath, 'resources.worktree.canonicalPath', false);
      if (resourcePath !== worktree.requestedPath) {
        invalid('resources.worktree.path 必须等于冻结的 worktree.requestedPath', {
          expected: worktree.requestedPath,
          actual: resourcePath,
        });
      }
      if (worktree.canonicalPath !== canonicalPath) {
        invalid('resources.worktree.canonicalPath 必须等于冻结的 worktree.canonicalPath', {
          expected: worktree.canonicalPath,
          actual: canonicalPath,
        });
      }
      return { path: resourcePath, canonicalPath };
    }),
    taskPackage: optionalResource(value.taskPackage, 'resources.taskPackage', ['path', 'digest'], (source) => ({
      path: nonEmptyString(source.path, 'resources.taskPackage.path'),
      digest: sha256(source.digest, 'resources.taskPackage.digest'),
    })),
    openingCommit: optionalResource(value.openingCommit, 'resources.openingCommit', ['oid'], (source) => ({
      oid: gitOid(source.oid, 'resources.openingCommit.oid'),
    })),
  };
  return resources;
}

function validateProgress(record) {
  const doneRequires = {
    branch: () => record.resources.branch !== null,
    worktree: () => record.resources.worktree !== null && record.worktree.canonicalPath !== null,
    taskPackage: () => record.resources.taskPackage !== null,
    openingCommit: () => record.resources.openingCommit !== null,
  };
  for (const name of PENDING_START_STEP_NAMES) {
    if (record.steps[name] === 'DONE' && !doneRequires[name]()) {
      invalid(`steps.${name} 为 DONE 时必须有同名真实 resources 事实`, { step: name });
    }
  }
  const firstNotDone = PENDING_START_STEP_NAMES.findIndex((name) => record.steps[name] !== 'DONE');
  if (firstNotDone >= 0) {
    for (let index = firstNotDone + 1; index < PENDING_START_STEP_NAMES.length; index += 1) {
      const prior = PENDING_START_STEP_NAMES[index - 1];
      const current = PENDING_START_STEP_NAMES[index];
      if (record.steps[current] === 'DONE' && record.steps[prior] !== 'DONE') {
        invalid(`steps.${current} 不能越过未完成的 ${prior}`, { prior, current });
      }
    }
  }
  const hasFailure = PENDING_START_STEP_NAMES.some((name) => record.steps[name] === 'FAILED');
  if (record.status === 'START_FAILED' && !hasFailure) {
    invalid('START_FAILED 必须明确标出一个失败步骤');
  }
  if (record.status === 'PENDING_START' && hasFailure) {
    invalid('PENDING_START 不能同时保留 FAILED 步骤；恢复必须显式重置步骤');
  }
}

function canonicalRecord(source, options) {
  const settings = options || {};
  if (!isPlainObject(source)) invalid('PENDING_START 必须是对象');
  const input = { ...source };
  if (!Object.prototype.hasOwnProperty.call(input, 'schema')) input.schema = PENDING_START_SCHEMA;
  if (!Object.prototype.hasOwnProperty.call(input, 'digest') && settings.requireDigest !== true) input.digest = null;
  assertExactKeys(input, RECORD_KEYS, 'PENDING_START');
  if (input.schema !== PENDING_START_SCHEMA) {
    invalid(`schema 必须精确等于 ${PENDING_START_SCHEMA}`, { actual: input.schema });
  }
  const record = {
    schema: PENDING_START_SCHEMA,
    id: safeId(input.id),
    proposal: canonicalProposal(input.proposal),
    parent: canonicalParent(input.parent),
    base: canonicalBase(input.base),
    branch: canonicalBranch(input.branch),
    worktree: canonicalWorktree(input.worktree),
    declaration: canonicalDeclaration(input.declaration),
    status: canonicalStatus(input.status),
    steps: canonicalSteps(input.steps),
    resources: null,
    digest: null,
  };
  record.resources = canonicalResources(input.resources, record.branch, record.worktree);
  validateProgress(record);
  const computedDigest = digestPendingStartRecord(record);
  if (input.digest !== null && input.digest !== undefined) {
    const supplied = sha256(input.digest, 'digest');
    if (supplied !== computedDigest) {
      throw new PendingStartError(
        'PENDING_START_DIGEST_MISMATCH',
        'PENDING_START digest 与固定字段不一致，不能把被改过的记录当成同一 proposal',
        { expected: computedDigest, actual: supplied, id: record.id },
      );
    }
  } else if (settings.requireDigest === true) {
    invalid('持久化 PENDING_START 缺少 digest');
  }
  record.digest = computedDigest;
  return record;
}

function createPendingStartRecord(input) {
  return canonicalRecord(input, { requireDigest: false });
}

function parsePendingStart(text) {
  let value;
  try {
    value = JSON.parse(String(text));
  } catch (error) {
    throw new PendingStartError(
      'PENDING_START_RECORD_INVALID',
      'PENDING_START 不是可解析的 JSON；不能把损坏记录当成不存在',
      { cause: error && error.message ? error.message : String(error) },
    );
  }
  return canonicalRecord(value, { requireDigest: true });
}

function serializePendingStart(record) {
  const canonical = canonicalRecord(record, { requireDigest: true });
  return `${JSON.stringify(canonical, null, 2)}\n`;
}

function statusTransitionAllowed(from, to) {
  if (from === to) return true;
  if (from === 'PENDING_START') return ['START_FAILED', 'BLOCKED', 'READY_FOR_CONFIRMED_REMOVAL'].includes(to);
  if (from === 'START_FAILED' || from === 'BLOCKED') {
    return ['PENDING_START', 'START_FAILED', 'BLOCKED', 'READY_FOR_CONFIRMED_REMOVAL'].includes(to);
  }
  return false;
}

function updatePendingStartRecord(record, patch) {
  const current = canonicalRecord(record, { requireDigest: true });
  if (!isPlainObject(patch)) invalid('PENDING_START 更新必须是对象');
  for (const key of Object.keys(patch)) {
    if (!MUTABLE_KEYS.has(key)) {
      if (RECORD_KEYS.includes(key)) {
        throw new PendingStartError(
          'PENDING_START_IMMUTABLE_FIELD',
          `PENDING_START 的 ${key} 是开工时冻结的身份，不能在恢复中改写`,
          { key, id: current.id },
        );
      }
      invalid(`PENDING_START 更新出现未登记字段 ${key}`, { key });
    }
  }
  if (Object.prototype.hasOwnProperty.call(patch, 'worktreeCanonicalPath')
    && current.worktree.canonicalPath !== null
    && patch.worktreeCanonicalPath !== current.worktree.canonicalPath) {
    throw new PendingStartError(
      'PENDING_START_IMMUTABLE_FIELD',
      'worktree.canonicalPath 只能在创建后从 null 回填一次，不能改写已冻结的真实身份',
      {
        expected: current.worktree.canonicalPath,
        actual: patch.worktreeCanonicalPath,
        id: current.id,
      },
    );
  }
  const next = {
    ...current,
    status: Object.prototype.hasOwnProperty.call(patch, 'status') ? patch.status : current.status,
    steps: Object.prototype.hasOwnProperty.call(patch, 'steps') ? patch.steps : current.steps,
    resources: Object.prototype.hasOwnProperty.call(patch, 'resources') ? patch.resources : current.resources,
    worktree: Object.prototype.hasOwnProperty.call(patch, 'worktreeCanonicalPath')
      ? { ...current.worktree, canonicalPath: patch.worktreeCanonicalPath }
      : current.worktree,
    // digest 是派生值，不是 patch 的可写输入；先清掉旧摘要再重新计算。
    digest: null,
  };
  const canonical = canonicalRecord(next, { requireDigest: false });
  if (!statusTransitionAllowed(current.status, canonical.status)) {
    throw new PendingStartError(
      'PENDING_START_STATUS_TRANSITION_INVALID',
      `PENDING_START 不允许从 ${current.status} 静默改成 ${canonical.status}`,
      { from: current.status, to: canonical.status, id: current.id },
    );
  }
  return canonical;
}

function preparePendingStartRecovery(record, action) {
  const current = canonicalRecord(record, { requireDigest: true });
  if (!PENDING_START_RECOVERY_ACTIONS.includes(action)) {
    throw new PendingStartError(
      'PENDING_START_RECOVERY_ACTION_INVALID',
      `recover action 只能是 ${PENDING_START_RECOVERY_ACTIONS.join(' / ')}`,
      { action },
    );
  }
  if (action === 'PREPARE_CONFIRMED_REMOVAL') {
    return updatePendingStartRecord(current, { status: 'READY_FOR_CONFIRMED_REMOVAL' });
  }
  const steps = { ...current.steps };
  for (const name of PENDING_START_STEP_NAMES) {
    if (steps[name] === 'FAILED') steps[name] = 'PENDING';
  }
  return updatePendingStartRecord(current, { status: 'PENDING_START', steps });
}

function projectPendingStartDeclaration(record) {
  const current = canonicalRecord(record, { requireDigest: true });
  return {
    id: current.id,
    // 创建前只能用由 canonical parent 派生的 requested path；创建后必须回填
    // canonicalPath。scope 本身会继续以同一工作副本硬约束与交集判据处理它。
    worktree: current.worktree.canonicalPath || current.worktree.requestedPath,
    'write-scope': current.declaration.writeScope.slice(),
    'forbidden-scope': current.declaration.forbiddenScope.slice(),
    'exclusive-resources': current.declaration.exclusiveResources.slice(),
    participates: true,
  };
}

function pendingStartDoctorView(record) {
  const current = canonicalRecord(record, { requireDigest: true });
  return {
    id: current.id,
    schema: current.schema,
    status: current.status,
    digest: current.digest,
    proposalDigest: current.proposal.digest,
    parent: { ...current.parent },
    baseOid: current.base.oid,
    branch: current.branch.name,
    worktree: { ...current.worktree },
    declaration: {
      writeScope: current.declaration.writeScope.slice(),
      forbiddenScope: current.declaration.forbiddenScope.slice(),
      exclusiveResources: current.declaration.exclusiveResources.slice(),
    },
    steps: { ...current.steps },
    resources: {
      branch: current.resources.branch && { ...current.resources.branch },
      worktree: current.resources.worktree && { ...current.resources.worktree },
      taskPackage: current.resources.taskPackage && { ...current.resources.taskPackage },
      openingCommit: current.resources.openingCommit && { ...current.resources.openingCommit },
    },
  };
}

module.exports = {
  PendingStartError,
  PENDING_START_SCHEMA,
  PENDING_START_STATUSES,
  PENDING_START_STEP_NAMES,
  PENDING_START_STEP_STATES,
  PENDING_START_RECOVERY_ACTIONS,
  createPendingStartRecord,
  parsePendingStart,
  serializePendingStart,
  digestPendingStartRecord,
  updatePendingStartRecord,
  preparePendingStartRecovery,
  projectPendingStartDeclaration,
  pendingStartDoctorView,
};
