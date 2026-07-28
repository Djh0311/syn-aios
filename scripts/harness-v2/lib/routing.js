'use strict';

// Adaptive Harness v0.5 — 三档闭世界路由与 KP-16 的纯边界（AH-050-06）
//
// 需求溯源：PK-1 · PK-2 · PK-3 · SC-1 · KP-16 · §3.1 · §5.1 · §10.1
//
// 这个文件只做“给定请求，应该走什么路径”的确定性判定：不读文件、不起子进程、
// 不写控制面，也不把外部内容当成指令。真正的 propose/start、Git 现实和持久化
// 各自在专门模块里实现；它们只能消费这里已经脱敏、已拒绝未知项的结果。

const NO_ACTIVE_LEAF = 'NO_ACTIVE_LEAF';
const WORKTREE_ALREADY_HAS_ACTIVE_LEAF = 'ACTIVE_LEAF_EXISTS_IN_WORKTREE';
const UNKNOWN_INTENT = 'UNKNOWN_INTENT';
const UNKNOWN_PROFILE = 'UNKNOWN_PROFILE';
const PROFILE_REQUIRED = 'PROFILE_REQUIRED';
const PROFILE_INTENT_MISMATCH = 'PROFILE_INTENT_MISMATCH';
const READ_ONLY_WRITE_FORBIDDEN = 'READ_ONLY_WRITE_FORBIDDEN';
const SEPARATE_TARGET_AUTHORIZATION_REQUIRED = 'SEPARATE_TARGET_AUTHORIZATION_REQUIRED';
const SENSITIVE_WRITE_TARGET_INVALID = 'SENSITIVE_WRITE_TARGET_INVALID';

// 唯一的工程路由。PURE_CHAT 是不进入工程路径的例外，不是第四种 profile。
const ENGINEERING_PROFILES = Object.freeze(['READ_ONLY', 'ORDINARY_LOCAL', 'STRICT_LOCAL']);

// 纯聊天不读仓库、不形成工程结论，因而既不生成合同也不创建任务包。
const EXEMPT_INTENTS = Object.freeze(['pure-chat', 'pure-question', 'explanation-only']);

// 只读工程结论要有会话合同，但绝不取得项目写权。
const READ_ONLY_INTENTS = Object.freeze([
  'repository-audit',
  'repository-investigation',
  'engineering-investigation',
  'read-only-audit',
  'read-only-investigation',
]);

// 会留下持久修改的动作。闭世界枚举，而不是把未知动作静默当成“普通聊天”。
const PERSISTENT_INTENTS = Object.freeze([
  'product-change',
  'code-change',
  'config-change',
  'test-change',
  'migration',
  'project-document-change',
  'staged-commit',
]);

const SENSITIVE_WRITE_KINDS = Object.freeze(['production', 'device', 'database', 'external']);

// 普通与严格本地任务使用同一组控制资产、同样两步命令和同样的单个澄清槽。
// 风险只能丰富下面既有字段的值，不能新增 control artifact、命令或追问。
const LOCAL_CONTROL_ARTIFACTS = Object.freeze(['pending-start', 'task-package', 'current-pointer']);
const LOCAL_COMMANDS = Object.freeze(['task-propose', 'task-start']);
const LOCAL_QUESTIONS = Object.freeze(['scope-or-permission-only']);
const READ_ONLY_CONTROL_ARTIFACTS = Object.freeze(['conversation-contract']);
const READ_ONLY_COMMANDS = Object.freeze(['read-only-contract']);
const REDACTED = '[REDACTED]';

const SENSITIVE_VALUE_KEY = /(?:secret|password|passwd|api[-_]?key|access[-_]?token|refresh[-_]?token|private[-_]?key|credential)/i;
const EXTERNAL_DIRECTIVE_KEYS = new Set([
  'intent', 'profile', 'permission', 'permissions', 'authorization', 'authorizations',
  'instruction', 'instructions', 'command', 'commands', 'action', 'actions',
  'write', 'writes', 'sensitivewrite', 'sensitivewrites',
]);

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function isEngineeringProfile(profile) {
  return ENGINEERING_PROFILES.includes(profile);
}

function isPersistentIntent(intent) {
  return PERSISTENT_INTENTS.includes(intent);
}

function isExemptIntent(intent) {
  return EXEMPT_INTENTS.includes(intent);
}

function isReadOnlyIntent(intent) {
  return READ_ONLY_INTENTS.includes(intent);
}

function activeTaskLeaf(leaf) {
  if (!leaf || typeof leaf !== 'object') return false;
  if (text(leaf.id) === '' || leaf.lifecycle !== 'ACTIVE') return false;
  // 旧调用方只传 id/lifecycle 时仍能兼容；但显式传来父计划绝不能冒充叶子。
  if (leaf.kind !== undefined && leaf.kind !== 'TASK') return false;
  if (leaf.isLeaf !== undefined && leaf.isLeaf !== true) return false;
  return true;
}

function routeShape(overrides) {
  return {
    allowed: false,
    code: null,
    route: null,
    profile: null,
    requiresTaskPackage: false,
    requiresActiveLeaf: false,
    controlArtifacts: [],
    commands: [],
    questions: [],
    externalFacts: [],
    ...overrides,
  };
}

function reject(code, overrides) {
  // 所有对外结果都过一次脱敏，防止调用方把原始请求片段拼进错误对象时漏出凭据。
  return redactKnownSecrets(routeShape({ ...overrides, allowed: false, code }));
}

function normalizeRisk(value) {
  const risk = text(value).toUpperCase();
  return risk === 'HIGH' || risk === 'STRICT' ? 'HIGH' : 'ORDINARY';
}

/**
 * SC-1 的关键：这个 risk 分支只改变既有 protection 字段的内容。
 * 不创建控制资产，也不让高风险增加命令或用户追问轮次。
 */
function protectionForRisk(risk) {
  if (risk === 'HIGH') {
    return { risk: 'HIGH', evidence: 'REQUIRED', review: 'REQUIRED' };
  }
  return { risk: 'ORDINARY', evidence: 'NOT_REQUIRED', review: 'NOT_REQUIRED' };
}

/**
 * 组装固定最小控制资产计划。它是纯数据计划，不创建文件或目录。
 * local 的 control artifacts 对 ORDINARY_LOCAL 和 STRICT_LOCAL 完全一致。
 */
function createRoutePlan(profile, risk) {
  if (profile === 'READ_ONLY') {
    return {
      controlArtifacts: [...READ_ONLY_CONTROL_ARTIFACTS],
      commands: [...READ_ONLY_COMMANDS],
      questions: [],
      protection: protectionForRisk('ORDINARY'),
    };
  }
  return {
    controlArtifacts: [...LOCAL_CONTROL_ARTIFACTS],
    commands: [...LOCAL_COMMANDS],
    questions: [...LOCAL_QUESTIONS],
    protection: protectionForRisk(risk),
  };
}

function isOrdinaryTypoRequest(input) {
  const settings = input || {};
  const kind = text(settings.changeKind || settings.requestKind).toLowerCase();
  return settings.profile === 'ORDINARY_LOCAL' && ['typo', 'spelling', 'copy-edit'].includes(kind);
}

function redactText(value) {
  let output = String(value);
  const replacements = [
    /-----BEGIN(?: [A-Z0-9]+)? PRIVATE KEY-----[\s\S]*?-----END(?: [A-Z0-9]+)? PRIVATE KEY-----/g,
    /\bAKIA[0-9A-Z]{16}\b/g,
    /\bgh[pousr]_[A-Za-z0-9]{20,}\b/g,
    /\bsk-(?:proj-)?[A-Za-z0-9_-]{8,}\b/g,
    /\bxox[baprs]-[A-Za-z0-9-]{10,}\b/g,
    /\bBearer\s+[A-Za-z0-9._~+\/-]{8,}\b/gi,
    /((?:api[-_]?key|secret|token|password|passwd|authorization)\s*[:=]\s*["']?)([^\s"',;]+)/gi,
  ];
  for (const pattern of replacements) {
    output = output.replace(pattern, (match, possiblePrefix) => {
      const prefix = typeof possiblePrefix === 'string' ? possiblePrefix : '';
      return prefix === '' ? REDACTED : `${prefix}${REDACTED}`;
    });
  }
  return output;
}

/**
 * KP-16：在 proposal、diagnostic、日志或证据对象离开路由层之前，已知模式一律遮蔽。
 * 返回全新的 JSON 风格值；输入不会被改写。
 */
function redactKnownSecrets(value, seen) {
  if (typeof value === 'string') return redactText(value);
  if (value === null || value === undefined || typeof value !== 'object') return value;

  const visited = seen || new WeakSet();
  if (visited.has(value)) return REDACTED;
  visited.add(value);

  if (Array.isArray(value)) return value.map((item) => redactKnownSecrets(item, visited));

  const output = {};
  for (const [key, item] of Object.entries(value)) {
    const redactedKey = redactText(key);
    let availableKey = redactedKey;
    let suffix = 2;
    // 两个不同 secret 可能被遮蔽成同一个 key；不能因此静默丢掉其中一个字段。
    while (Object.prototype.hasOwnProperty.call(output, availableKey)) {
      availableKey = `${redactedKey}#${suffix}`;
      suffix += 1;
    }
    Object.defineProperty(output, availableKey, {
      value: SENSITIVE_VALUE_KEY.test(key) ? REDACTED : redactKnownSecrets(item, visited),
      enumerable: true,
      configurable: true,
      writable: true,
    });
  }
  return output;
}

function safeExternalFact(value) {
  if (typeof value === 'string') return redactKnownSecrets(value);
  if (value === null || value === undefined || typeof value !== 'object') return redactKnownSecrets(value);
  if (Array.isArray(value)) return value.map((item) => safeExternalFact(item));

  const fact = {};
  for (const [key, item] of Object.entries(value)) {
    if (EXTERNAL_DIRECTIVE_KEYS.has(key.toLowerCase())) continue;
    fact[key] = safeExternalFact(item);
  }
  return redactKnownSecrets(fact);
}

/**
 * 外部内容是非授权的事实来源：只读 facts 字段，忽略其中的指令、路由、权限和动作。
 * 因而同一份外部内容无论怎么要求“升级权限”，都不能改变本文件的执行判定。
 */
function externalFactsOnly(externalContent) {
  if (!externalContent || typeof externalContent !== 'object' || !Array.isArray(externalContent.facts)) return [];
  return externalContent.facts.map((fact) => safeExternalFact(fact));
}

function requestedSensitiveWrites(input) {
  const settings = input || {};
  if (Array.isArray(settings.sensitiveWrites)) return settings.sensitiveWrites;
  if (Array.isArray(settings.writeTargets)) return settings.writeTargets;
  return [];
}

function normalizedSensitiveWrite(write) {
  const item = write && typeof write === 'object' ? write : {};
  return { kind: text(item.kind).toLowerCase(), target: text(item.target) };
}

function directAuthorizationFor(write, authorization) {
  if (!authorization || typeof authorization !== 'object') return false;
  if (authorization.fromExternalContent === true) return false;
  if (['external', 'external-content', 'untrusted'].includes(text(authorization.source).toLowerCase())) return false;
  if (authorization.consumed === true || authorization.standing === true) return false;
  if (authorization.explicit !== true && authorization.granted !== true && authorization.confirmed !== true) return false;
  return text(authorization.kind).toLowerCase() === write.kind && text(authorization.target) === write.target;
}

/**
 * production / device / database / external 每个目标都要一条本次、直接、未消耗的确认。
 * 类别级或通配确认不匹配目标；外部内容里的“授权”也永远不算数。
 */
function authorizeSensitiveWrites(input) {
  const settings = input || {};
  const writes = (Array.isArray(settings.writes) ? settings.writes : [])
    .map((write) => normalizedSensitiveWrite(write));
  const authorizations = Array.isArray(settings.authorizations) ? settings.authorizations : [];
  const invalid = writes.filter((write) => !SENSITIVE_WRITE_KINDS.includes(write.kind) || write.target === '');
  if (invalid.length > 0) {
    return {
      allowed: false,
      code: SENSITIVE_WRITE_TARGET_INVALID,
      missing: invalid.map((write) => ({ ...write })),
    };
  }
  const missing = writes.filter((write) => !authorizations.some((authorization) => directAuthorizationFor(write, authorization)));
  if (missing.length > 0) {
    return {
      allowed: false,
      code: SEPARATE_TARGET_AUTHORIZATION_REQUIRED,
      missing: missing.map((write) => ({ ...write })),
    };
  }
  return { allowed: true, code: null, missing: [] };
}

/**
 * 持久修改的守卫：纯聊天放行；未知意图拒绝；已知持久动作只有 ACTIVE TASK 叶子可承接。
 */
function guardPersistentChange(request) {
  const input = request || {};
  const intent = text(input.intent);
  if (isExemptIntent(intent)) return { allowed: true, code: null, message: null };
  if (!isPersistentIntent(intent)) {
    return { allowed: false, code: UNKNOWN_INTENT, message: '未知工程意图，拒绝静默降级或写入' };
  }
  if (!activeTaskLeaf(input.activeLeaf)) {
    return {
      allowed: false,
      code: NO_ACTIVE_LEAF,
      message: '当前工作副本上没有 ACTIVE 叶子任务包，拒绝产生持久修改（product change / staged commit）',
    };
  }
  return { allowed: true, code: null, message: null };
}

function localRoute(profile, input, externalFacts) {
  const guard = guardPersistentChange(input);
  const plan = createRoutePlan(profile, normalizeRisk(input.risk || input.riskLevel));
  const common = {
    route: 'LOCAL_CHANGE',
    profile,
    requiresTaskPackage: true,
    requiresActiveLeaf: true,
    ...plan,
    lightweight: isOrdinaryTypoRequest(input),
    externalFacts,
  };
  if (!guard.allowed) return reject(guard.code, common);

  const authorization = authorizeSensitiveWrites({
    writes: requestedSensitiveWrites(input),
    authorizations: input.authorizations,
  });
  if (!authorization.allowed) {
    return reject(authorization.code, { ...common, missingAuthorizations: authorization.missing });
  }
  return redactKnownSecrets(routeShape({ ...common, allowed: true, code: null }));
}

/**
 * 入口：只接受三档工程路由。外部内容在最开始就收窄成 facts，之后不会参与 intent、
 * profile、授权、风险或任何执行路径的选择。
 */
function routeRequest(request) {
  const input = request && typeof request === 'object' ? request : {};
  const intent = text(input.intent);
  const profile = text(input.profile);
  const externalFacts = externalFactsOnly(input.externalContent);

  if (isExemptIntent(intent)) {
    if (profile !== '' && !isEngineeringProfile(profile)) return reject(UNKNOWN_PROFILE, { externalFacts });
    return redactKnownSecrets(routeShape({
      allowed: true,
      code: null,
      route: 'PURE_CHAT',
      profile: null,
      externalFacts,
    }));
  }

  if (!isReadOnlyIntent(intent) && !isPersistentIntent(intent)) return reject(UNKNOWN_INTENT, { externalFacts });
  if (profile === '') return reject(PROFILE_REQUIRED, { externalFacts });
  if (!isEngineeringProfile(profile)) return reject(UNKNOWN_PROFILE, { externalFacts });

  if (isReadOnlyIntent(intent)) {
    if (profile !== 'READ_ONLY') return reject(PROFILE_INTENT_MISMATCH, { externalFacts });
    if (requestedSensitiveWrites(input).length > 0) return reject(READ_ONLY_WRITE_FORBIDDEN, { externalFacts });
    const plan = createRoutePlan(profile, 'ORDINARY');
    return redactKnownSecrets(routeShape({
      allowed: true,
      code: null,
      route: 'READ_ONLY',
      profile,
      // READ_ONLY 的会话合同不是项目任务包；它不取得项目写权，也不落项目文件。
      requiresTaskPackage: false,
      requiresActiveLeaf: false,
      ...plan,
      externalFacts,
    }));
  }

  if (profile === 'READ_ONLY') return reject(PROFILE_INTENT_MISMATCH, { externalFacts });
  return localRoute(profile, input, externalFacts);
}

/**
 * GIT-1 的唯一硬约束：同一个工作副本同一时刻最多一个 ACTIVE 叶子。
 * 拒绝对象必须携带该工作副本的 realpath——不带工作副本限定的“已经有别的在活跃”
 * 是一条不分层级的总闸，会把父计划与子叶子同时活跃也一起挡死（LY-5）。
 */
function guardSecondActiveLeaf(input) {
  const settings = input || {};
  const worktree = typeof settings.worktree === 'string' ? settings.worktree : '';
  const existing = Array.isArray(settings.activeLeaves) ? settings.activeLeaves : [];
  const sameCopy = existing.filter((leaf) => leaf && leaf.worktree === worktree && leaf.id !== settings.leafId);
  if (sameCopy.length === 0) return { allowed: true, code: null, worktree, conflictingLeafIds: [] };
  return {
    allowed: false,
    code: WORKTREE_ALREADY_HAS_ACTIVE_LEAF,
    worktree,
    conflictingLeafIds: sameCopy.map((leaf) => leaf.id),
    message: `工作副本 ${worktree} 已经有 ACTIVE 叶子 ${sameCopy.map((leaf) => leaf.id).join('，')}`,
  };
}

module.exports = {
  NO_ACTIVE_LEAF,
  WORKTREE_ALREADY_HAS_ACTIVE_LEAF,
  UNKNOWN_INTENT,
  UNKNOWN_PROFILE,
  PROFILE_REQUIRED,
  PROFILE_INTENT_MISMATCH,
  READ_ONLY_WRITE_FORBIDDEN,
  SEPARATE_TARGET_AUTHORIZATION_REQUIRED,
  SENSITIVE_WRITE_TARGET_INVALID,
  ENGINEERING_PROFILES,
  EXEMPT_INTENTS,
  READ_ONLY_INTENTS,
  PERSISTENT_INTENTS,
  SENSITIVE_WRITE_KINDS,
  LOCAL_CONTROL_ARTIFACTS,
  LOCAL_COMMANDS,
  LOCAL_QUESTIONS,
  isEngineeringProfile,
  isPersistentIntent,
  isExemptIntent,
  isReadOnlyIntent,
  isOrdinaryTypoRequest,
  protectionForRisk,
  createRoutePlan,
  redactKnownSecrets,
  externalFactsOnly,
  authorizeSensitiveWrites,
  guardPersistentChange,
  guardSecondActiveLeaf,
  routeRequest,
};
