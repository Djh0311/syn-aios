'use strict';

// Adaptive Harness v0.5 — 两个平面的落点与唯一写入口（AH-050-03）
//
// 需求溯源：EX-5 · EX-6 · EX-9 · LY-4 · WK-1 · §3.1 · §4.3
//
// 本文件是整个 harness-v2 里**唯一**会写文件的模块。「只读的就是只读的」要成立，
// 写必须只有一个入口。

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

const gitFacts = require('./git-facts');
const lifecycle = require('./lifecycle');
const nodeSchema = require('./node-schema');
const pendingStart = require('./pending-start');

// ---------------------------------------------------------------------------
// 裁决 A：控制面拆成两个平面
//
// 在办平面（drafts / current / parked / pending-start / 图谱锁与 generation /
// 每工作副本一份的 CURRENT）挂在 git common dir 下的私有目录，**不随 Git 跟踪**，
// 全仓所有 linked worktree 共享同一份物理文件——所以「不切分支即可读到全部在册声明」
// 与「任何工作副本读到的内容一致」这两条跨工作树要求物理上成立（EX-9 第 1、2 条）。
//
// 历史平面（history plane）挂在仓库内单一受管根路径，只由干净的 integration worktree
// 在集成那一刻写入一次，此后正文冻结；任何工作副本读它一律经固定的 integration ref。
// 当前入口（current）与历史（history）因此各有唯一一个说法，跨工作树读到的完全一致。
//
// 并发写入后写方报冲突、不静默覆盖（EX-9 第 4 条）：跨工作树的写事务绑定预检时的
// graph generation 与逐节点内容摘要，两者任一对不上就是冲突。
const CONFLICT_STALE_GENERATION = 'CONFLICT_STALE_GENERATION';
const CONFLICT_NODE_CHANGED = 'CONFLICT_NODE_CHANGED';
const CONFLICT_TARGET_CHANGED = 'CONFLICT_TARGET_CHANGED';
const CONFLICT_LOCK_HELD = 'CONFLICT_LOCK_HELD';
const STORE_READ_FAILED = 'STORE_READ_FAILED';
const TARGET_ALREADY_EXISTS = 'TARGET_ALREADY_EXISTS';
const WRITE_PLAN_PATH_COLLISION = 'WRITE_PLAN_PATH_COLLISION';
const TRANSACTION_ROLLBACK_INCOMPLETE = 'TRANSACTION_ROLLBACK_INCOMPLETE';
const LOCK_RELEASE_FAILED = 'LOCK_RELEASE_FAILED';
const COMMIT_APPLIED_LOCK_RELEASE_FAILED = 'COMMIT_APPLIED_LOCK_RELEASE_FAILED';
const PENDING_START_MALFORMED = 'PENDING_START_MALFORMED';
const PENDING_START_ID_CONFLICT = 'PENDING_START_ID_CONFLICT';
const PENDING_START_NOT_FOUND = 'PENDING_START_NOT_FOUND';
const CONFLICT_PENDING_START_DIGEST = 'CONFLICT_PENDING_START_DIGEST';
const PENDING_START_CAS_REQUIRED = 'PENDING_START_CAS_REQUIRED';
const PENDING_START_FINALIZATION_INVALID = 'PENDING_START_FINALIZATION_INVALID';
const PENDING_START_STANDALONE_CONSUME_FORBIDDEN = 'PENDING_START_STANDALONE_CONSUME_FORBIDDEN';
const PENDING_START_HISTORY_ID_CONFLICT = 'PENDING_START_HISTORY_ID_CONFLICT';
const PENDING_START_HISTORY_REALITY_UNAVAILABLE = 'PENDING_START_HISTORY_REALITY_UNAVAILABLE';
const PENDING_START_LIVE_ID_CONFLICT = 'PENDING_START_LIVE_ID_CONFLICT';
const PENDING_START_RESOURCE_REALITY_CONFLICT = 'PENDING_START_RESOURCE_REALITY_CONFLICT';
const OPENING_PACKAGE_PATH_INVALID = 'OPENING_PACKAGE_PATH_INVALID';
// 已进入 HISTORY 的那份正文冻结：不得被后写内容覆盖（LY-4）。
// 这不是流转层面的拒绝，而是对已结束节点**内容**本身的约束。
const HISTORY_BODY_FROZEN = 'HISTORY_BODY_FROZEN';
const HISTORY_REF_UNAVAILABLE = 'HISTORY_REF_UNAVAILABLE';
const HISTORY_BUCKET_INVALID = 'HISTORY_BUCKET_INVALID';
const ID_NOT_SAFE_PATH_SEGMENT = 'ID_NOT_SAFE_PATH_SEGMENT';
// 两阶段 canonical history 的第二步只消费“已经由 Git 固定引用承认”的候选正文。
// 这些错误码刻意和普通节点移动区分：这里绝不能把尚未提交、错误 ref 或被改过的
// working-tree candidate 当作可退场的历史事实。
const HISTORY_FINALIZATION_INVALID = 'HISTORY_FINALIZATION_INVALID';
const HISTORY_FINALIZATION_REF_MISMATCH = 'HISTORY_FINALIZATION_REF_MISMATCH';
const HISTORY_FINALIZATION_CANDIDATE_MISMATCH = 'HISTORY_FINALIZATION_CANDIDATE_MISMATCH';
const HISTORY_FINALIZATION_SOURCE_INVALID = 'HISTORY_FINALIZATION_SOURCE_INVALID';
const HISTORY_FINALIZATION_CHECKOUT_MISMATCH = 'HISTORY_FINALIZATION_CHECKOUT_MISMATCH';
const TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN = 'TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN';
const HISTORY_CANDIDATE_FREEZE_ACTION = 'history-candidate-freeze';

// 物理落点必须和生命周期一致（§4.3）。四个目录挂在两个根上。
// 卡住 / 暂停的东西真的被挪去 parked，不是只把字段改掉（EX-6）。
const LIFECYCLE_LOCATION = Object.freeze({
  DRAFT: 'adaptive-harness/drafts',
  READY: 'adaptive-harness/current',
  ACTIVE: 'adaptive-harness/current',
  PARKED: 'adaptive-harness/parked',
  HISTORY: 'docs/harness/history',
});

// LIVE_AREAS 描述 common-dir 的所有在办控制区域；PENDING_START 也在这里，
// 但它不是 node area。图谱读取只能解析后三个 canonical node 目录。
const LIVE_AREAS = Object.freeze(['drafts', 'current', 'parked', 'pending-start']);
const LIVE_NODE_AREAS = Object.freeze(['drafts', 'current', 'parked']);

const DEFAULT_LIVE_DIR_NAME = 'adaptive-harness';
const DEFAULT_HISTORY_ROOT = 'docs/harness/history';
const DEFAULT_INTEGRATION_REF = 'refs/heads/main';
// 只有 finalizePendingStart 能持有这份进程内 authority；普通 plan 调用方即使
// 伪造 role/remove 字段，也不能借 commitNodeWrite 绕过完整收口直接删 marker。
const PENDING_START_FINALIZATION_AUTHORITY = Symbol('pending-start-finalization');
const PENDING_START_FINALIZATION_PLANS = new WeakSet();
// 开工收口的 Git/资源现实不是一段可由 write plan 伪造的普通回调。调用方先向
// store 申请一枚进程内 opaque guard；finalize 把它绑定到自己刚创建的 plan，随后
// commitNodeWrite 才会在同一把 graph lock 内执行 pre/post 两次只读复核。
// WeakMap 不暴露 probe，也避免“给 plan 塞同名字段”绕过这一边界。
const PENDING_START_REALITY_GUARDS = new WeakMap();
const PENDING_START_REALITY_PLANS = new WeakMap();

class StoreError extends Error {
  constructor(code, message, detail) {
    super(message);
    this.name = 'StoreError';
    this.code = code;
    this.detail = detail || null;
  }
}

// ---------------------------------------------------------------------------
// 落点解析
// ---------------------------------------------------------------------------

function resolvePlanes(options) {
  const settings = options || {};
  if (settings.liveRoot) {
    return {
      liveRoot: settings.liveRoot,
      historyRoot: settings.historyRoot || path.join(settings.liveRoot, 'history'),
      integrationRef: settings.integrationRef || DEFAULT_INTEGRATION_REF,
      repoRoot: settings.repoRoot || null,
      tracked: false,
    };
  }
  const cwd = settings.cwd || process.cwd();
  const commonDir = gitFacts.gitCommonDir(cwd);
  const root = gitFacts.repoRoot(cwd);
  return {
    liveRoot: path.join(commonDir, DEFAULT_LIVE_DIR_NAME),
    historyRoot: path.join(root, settings.historyRoot || DEFAULT_HISTORY_ROOT),
    historyRepoPath: settings.historyRoot || DEFAULT_HISTORY_ROOT,
    integrationRef: settings.integrationRef || DEFAULT_INTEGRATION_REF,
    repoRoot: root,
    tracked: true,
  };
}

function areaFor(lifecycle) {
  if (lifecycle === 'DRAFT') return 'drafts';
  if (lifecycle === 'READY' || lifecycle === 'ACTIVE') return 'current';
  if (lifecycle === 'PARKED') return 'parked';
  if (lifecycle === 'HISTORY') return 'history';
  return null;
}

function areaDir(planes, area) {
  if (area === 'history') return planes.historyRoot;
  return path.join(planes.liveRoot, area);
}

function historyBucket(closedAt) {
  const text = String(closedAt || '').trim();
  const match = /^(\d{4})-(\d{2})/.exec(text);
  return match ? `${match[1]}-${match[2]}` : 'unknown';
}

function assertSafeIdSegment(id) {
  const safe = typeof id === 'string'
    && id.length > 0
    && id !== '.'
    && id !== '..'
    && !id.includes('/')
    && !id.includes('\\')
    && !id.includes('\0');
  if (!safe) {
    throw new StoreError(
      ID_NOT_SAFE_PATH_SEGMENT,
      '节点 id 必须是单一安全路径段，不得为空、`.`、`..` 或含路径分隔符 / NUL',
      { id },
    );
  }
  return id;
}

function assertSafeHistoryBucket(bucket) {
  const safe = typeof bucket === 'string'
    && bucket.length > 0
    && bucket !== '.'
    && bucket !== '..'
    && !bucket.includes('/')
    && !bucket.includes('\\')
    && !bucket.includes('\0');
  if (!safe) {
    throw new StoreError(
      HISTORY_BUCKET_INVALID,
      '历史 bucket 必须是单一安全路径段，不得为空、`.`、`..` 或含路径分隔符 / NUL',
      { bucket },
    );
  }
  return bucket;
}

function nodeFilePath(planes, node) {
  const id = assertSafeIdSegment(node && node.id);
  const area = areaFor(node.lifecycle);
  if (!area) throw new StoreError('LIFECYCLE_HAS_NO_LOCATION', `lifecycle ${node.lifecycle} 没有对应落点`);
  if (area === 'history') {
    return path.join(planes.historyRoot, historyBucket(node['closed-at']), `${id}.md`);
  }
  return path.join(planes.liveRoot, area, `${id}.md`);
}

function pendingStartPath(planes, id) {
  return path.join(planes.liveRoot, 'pending-start', `${assertSafeIdSegment(id)}.md`);
}

function currentViewPath(planes, worktreeKey) {
  return path.join(planes.liveRoot, 'worktrees', worktreeKey, 'current.md');
}

function worktreeKeyFor(worktreePath) {
  const normalized = path.resolve(String(worktreePath || ''));
  return crypto.createHash('sha256').update(normalized).digest('hex').slice(0, 16);
}

// ---------------------------------------------------------------------------
// 读（只读，绝不写）
// ---------------------------------------------------------------------------

function digestOf(text) {
  return crypto.createHash('sha256').update(String(text), 'utf8').digest('hex');
}

function isMissingError(error) {
  return error && (error.code === 'ENOENT' || error.code === 'ENOTDIR');
}

function storeReadFailed(operation, target, error) {
  return new StoreError(
    STORE_READ_FAILED,
    `读取控制面失败，不能把未知当成不存在：${operation} ${target}`,
    {
      operation,
      target,
      cause: error && error.message ? error.message : String(error),
      causeCode: error && error.code ? error.code : null,
    },
  );
}

function readTextOrNull(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf8');
  } catch (error) {
    if (isMissingError(error)) return null;
    throw storeReadFailed('readFile', filePath, error);
  }
}

function listArea(planes, area) {
  const directory = areaDir(planes, area);
  let entries;
  try {
    entries = fs.readdirSync(directory, { withFileTypes: true });
  } catch (error) {
    if (isMissingError(error)) return [];
    throw storeReadFailed('readdir', directory, error);
  }
  return entries
    .filter((entry) => entry.isFile() && entry.name.endsWith('.md'))
    .map((entry) => path.join(directory, entry.name))
    .sort();
}

function lifecycleLocationIssues(node, area) {
  if (!node || !nodeSchema.NODE_KINDS.includes(node.kind)) return [];
  const allowed = lifecycle.lifecycleValuesFor(node.kind);
  if (!allowed.includes(node.lifecycle)) {
    return [{
      code: 'LIFECYCLE_UNKNOWN',
      field: 'lifecycle',
      message: `${node.kind} 的 lifecycle 取值 ${node.lifecycle} 不在取值域内`,
    }];
  }
  if (!['drafts', 'current', 'parked'].includes(area)) return [];
  const expected = areaFor(node.lifecycle);
  if (expected === area) return [];
  return [{
    code: 'LIFECYCLE_LOCATION_MISMATCH',
    field: 'lifecycle',
    message: `lifecycle ${node.lifecycle} 应位于 ${expected || '未知目录'}，实际位于 ${area}`,
  }];
}

/**
 * 只读 canonical live nodes（drafts + current + parked）。PENDING_START 是
 * Git common-dir 的临时控制记录，不是 TASK lifecycle，绝不能让 node-schema
 * 把它当作节点解析；需要它的开工/doctor 路径必须显式调用 listPendingStarts。
 * 本函数不碰历史平面——历史只能按编号单点解析。
 */
function readLiveNodes(planes) {
  const out = [];
  for (const area of LIVE_NODE_AREAS) {
    for (const filePath of listArea(planes, area)) {
      const text = readTextOrNull(filePath);
      if (text === null) continue;
      const parsed = nodeSchema.parseNode(text, { relativePath: filePath });
      const issues = parsed.issues.concat(lifecycleLocationIssues(parsed.node, area));
      out.push({
        area,
        path: filePath,
        text,
        digest: digestOf(text),
        node: parsed.node,
        title: parsed.title,
        body: parsed.body,
        sections: parsed.sections,
        issues,
      });
    }
  }
  return out;
}

function normalizedIdentityKey(id) {
  return assertSafeIdSegment(id).normalize('NFC').toLowerCase();
}

/**
 * PENDING_START 的最终收口不能只看将要落入 current 的同一路径。外部直接写入、
 * 老版本写入或故障恢复都可能把等价编号留在 drafts / current / parked 的另一处；
 * 因而这里同时检查文件名与解析出的 node.id。解析失败但文件名命中也必须拒绝，
 * 不能让坏正文成为绕过全局编号唯一性的洞。
 */
function liveIdentityCandidates(planes, id) {
  const wanted = normalizedIdentityKey(id);
  const candidates = [];
  for (const area of LIVE_NODE_AREAS) {
    for (const filePath of listArea(planes, area)) {
      const source = readTextOrNull(filePath);
      if (source === null) {
        throw new StoreError(
          CONFLICT_NODE_CHANGED,
          `live identity 枚举后 ${filePath} 消失；不能把并发漂移当成无冲突`,
          { id, path: filePath },
        );
      }
      const fileId = path.basename(filePath, '.md');
      const fileMatches = fileId.normalize('NFC').toLowerCase() === wanted;
      const parsed = nodeSchema.parseNode(source, { relativePath: filePath });
      const nodeId = parsed && parsed.node && typeof parsed.node.id === 'string'
        ? parsed.node.id
        : null;
      const nodeMatches = nodeId !== null
        && nodeId.normalize('NFC').toLowerCase() === wanted;
      if (fileMatches || nodeMatches) {
        candidates.push({
          area,
          path: filePath,
          fileId,
          nodeId,
          fileMatches,
          nodeMatches,
          digest: digestOf(source),
          issues: parsed && Array.isArray(parsed.issues) ? parsed.issues.map((issue) => issue.code || null) : [],
        });
      }
    }
  }
  return candidates;
}

// ---------------------------------------------------------------------------
// PENDING_START：独立临时控制记录，不进入 canonical node graph
// ---------------------------------------------------------------------------

function pendingStartDirectory(planes) {
  return areaDir(planes, 'pending-start');
}

function pendingStartMalformed(filePath, error) {
  return new StoreError(
    PENDING_START_MALFORMED,
    `PENDING_START ${filePath} 无法按固定模型读取；不能把损坏记录当成不存在`,
    {
      path: filePath,
      causeCode: error && error.code ? error.code : null,
      cause: error && error.message ? error.message : String(error),
    },
  );
}

/**
 * pending-start 目录是单一用途目录：任何非 marker 文件、不可安全映射的文件名、
 * 同 id 的大小写/NFC 变体，都会让开工方 fail closed，而不是忽略后绕过半成品。
 */
function listPendingStartFiles(planes) {
  const directory = pendingStartDirectory(planes);
  let entries;
  try {
    entries = fs.readdirSync(directory, { withFileTypes: true });
  } catch (error) {
    if (isMissingError(error)) return [];
    throw storeReadFailed('readdir', directory, error);
  }
  const out = [];
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const filePath = path.join(directory, entry.name);
    if (!entry.isFile() || !entry.name.endsWith('.md')) {
      throw new StoreError(
        PENDING_START_MALFORMED,
        `pending-start 目录出现非 marker 项 ${entry.name}；不能忽略未知控制面内容`,
        { path: filePath, name: entry.name },
      );
    }
    const id = entry.name.slice(0, -3);
    try {
      assertSafeIdSegment(id);
    } catch (error) {
      throw pendingStartMalformed(filePath, error);
    }
    out.push({ id, path: filePath });
  }
  return out;
}

function readPendingStartRecords(planes) {
  const records = [];
  const seen = new Map();
  for (const entry of listPendingStartFiles(planes)) {
    const text = readTextOrNull(entry.path);
    if (text === null) {
      throw new StoreError(
        PENDING_START_MALFORMED,
        `pending-start 枚举后 ${entry.path} 消失；不能把并发漂移当成空记录`,
        { path: entry.path },
      );
    }
    let record;
    try {
      record = pendingStart.parsePendingStart(text);
    } catch (error) {
      throw pendingStartMalformed(entry.path, error);
    }
    if (record.id !== entry.id) {
      throw new StoreError(
        PENDING_START_ID_CONFLICT,
        `PENDING_START 文件名 ${entry.id} 与记录 id ${record.id} 不一致，不能猜哪个身份为真`,
        { path: entry.path, fileId: entry.id, recordId: record.id },
      );
    }
    const idKey = record.id.normalize('NFC').toLowerCase();
    if (seen.has(idKey)) {
      throw new StoreError(
        PENDING_START_ID_CONFLICT,
        `PENDING_START id ${record.id} 存在大小写或 Unicode 等价冲突，不能选择其中一份`,
        { id: record.id, first: seen.get(idKey), second: entry.path },
      );
    }
    seen.set(idKey, entry.path);
    records.push({
      id: record.id,
      path: entry.path,
      text,
      contentDigest: digestOf(text),
      digest: record.digest,
      record,
    });
  }
  return records;
}

/**
 * 与 node snapshot 同样绑定 graph generation，并观察同一把跨 worktree 锁。
 * PENDING 的字符串摘要与 generation 都交给后续 CAS，避免“读到旧 marker 却写入
 * 新 generation”的偷换。
 */
function readPendingStartSnapshot(planes) {
  const beforeLock = inspectLock(planes);
  if (beforeLock.held) {
    throw new StoreError(CONFLICT_LOCK_HELD, '读取 PENDING_START 时图谱锁已被占用', beforeLock);
  }
  const generation = readGeneration(planes);
  const records = readPendingStartRecords(planes);
  const afterLock = inspectLock(planes);
  if (afterLock.held) {
    throw new StoreError(CONFLICT_LOCK_HELD, '读取 PENDING_START 期间图谱锁被占用', afterLock);
  }
  const current = readGeneration(planes);
  if (current !== generation) {
    throw new StoreError(
      CONFLICT_STALE_GENERATION,
      `读取 PENDING_START 期间 generation 已从 ${generation} 前进到 ${current}`,
      { expected: generation, actual: current },
    );
  }
  return {
    generation,
    records: records.map((record) => ({ ...record, generation })),
  };
}

function listPendingStarts(planes) {
  return readPendingStartSnapshot(planes).records;
}

function readPendingStart(planes, id) {
  const safeId = assertSafeIdSegment(id);
  const snapshot = readPendingStartSnapshot(planes);
  const exact = snapshot.records.find((entry) => entry.id === safeId) || null;
  if (exact) return exact;
  const equivalent = snapshot.records.find((entry) => (
    entry.id.normalize('NFC').toLowerCase() === safeId.normalize('NFC').toLowerCase()
  ));
  if (equivalent) {
    throw new StoreError(
      PENDING_START_ID_CONFLICT,
      `请求 id ${safeId} 与在册 PENDING_START ${equivalent.id} 只有大小写或 Unicode 差异，不能据此绕过记录`,
      { requestedId: safeId, registeredId: equivalent.id, path: equivalent.path },
    );
  }
  return null;
}

function localHistoryCandidates(planes, id, bucket) {
  const safeId = assertSafeIdSegment(id);
  const wantedIdKey = safeId.normalize('NFC').toLowerCase();
  const buckets = [];
  if (bucket) buckets.push(assertSafeHistoryBucket(bucket));
  else {
    let entries = [];
    try {
      entries = fs.readdirSync(planes.historyRoot, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => entry.name);
    } catch (error) {
      if (!isMissingError(error)) {
        throw storeReadFailed('readdir', planes.historyRoot, error);
      }
      entries = [];
    }
    buckets.push(...entries.sort());
  }
  const found = [];
  for (const candidate of buckets) {
    const directory = path.join(planes.historyRoot, candidate);
    let entries;
    try {
      entries = fs.readdirSync(directory, { withFileTypes: true });
    } catch (error) {
      if (isMissingError(error)) continue;
      throw storeReadFailed('readdir', directory, error);
    }
    for (const entry of entries) {
      if (!entry.isFile() || !entry.name.endsWith('.md')) continue;
      const candidateId = entry.name.slice(0, -3);
      if (candidateId.normalize('NFC').toLowerCase() !== wantedIdKey) continue;
      const filePath = path.join(directory, entry.name);
      const text = readTextOrNull(filePath);
      if (text !== null) found.push({ bucket: candidate, path: filePath, text });
    }
  }
  return found;
}

/**
 * 按编号单点解析一份已结束的节点。历史正文永不冒充 current（EX-5）：
 * 只有显式命令会走到这里，默认装配路径一次都不调用它。
 */
function readHistoryNode(planes, id, options) {
  const safeId = assertSafeIdSegment(id);
  const wantedIdKey = safeId.normalize('NFC').toLowerCase();
  const settings = options || {};
  const bucket = settings.bucket === undefined || settings.bucket === null
    ? null
    : assertSafeHistoryBucket(settings.bucket);

  // 受 Git 跟踪的历史只认固定 integration ref。不能先读当前工作树里的同名文件，
  // 否则任务分支上的旧副本、未提交副本都可能冒充已经集成的历史事实。
  if (planes.tracked) {
    const cwd = settings.cwd || planes.repoRoot;
    if (!cwd) {
      throw new StoreError(
        HISTORY_REF_UNAVAILABLE,
        `无法从 ${planes.integrationRef} 读取历史：缺少 Git 工作目录`,
      );
    }
    const repoRoot = String(planes.historyRepoPath || DEFAULT_HISTORY_ROOT)
      .replace(/\\/g, '/')
      .replace(/^\.\/+/, '')
      .replace(/\/+$/, '');
    const resolved = gitFacts.runGit([
      'rev-parse',
      '--verify',
      '--end-of-options',
      `${planes.integrationRef}^{commit}`,
    ], { cwd });
    const integrationOid = resolved.ok ? resolved.stdout.trim() : '';
    if (!/^[0-9a-f]{40,64}$/i.test(integrationOid)) {
      throw new StoreError(
        HISTORY_REF_UNAVAILABLE,
        `无法把固定 integration ref ${planes.integrationRef} 冻结成单一 commit OID`,
        { ref: planes.integrationRef, stderr: resolved.stderr || '' },
      );
    }
    const listed = gitFacts.runGit([
      'ls-tree',
      '-r',
      '-z',
      '--name-only',
      integrationOid,
      '--',
      repoRoot,
    ], { cwd });
    if (!listed.ok) {
      throw new StoreError(
        HISTORY_REF_UNAVAILABLE,
        `无法从固定 integration ref ${planes.integrationRef} 枚举历史，不能把“读不到”当成“编号不存在”`,
        { ref: planes.integrationRef, stderr: listed.stderr || '' },
      );
    }
    const prefix = `${repoRoot}/`;
    const candidates = listed.stdout
      .split('\0')
      .filter(Boolean)
      .map((repoRelative) => {
        if (!repoRelative.startsWith(prefix)) return null;
        const relative = repoRelative.slice(prefix.length);
        const slash = relative.indexOf('/');
        if (slash <= 0) return null;
        const candidateBucket = relative.slice(0, slash);
        const fileName = relative.slice(slash + 1);
        if (!fileName.endsWith('.md')) return null;
        const candidateId = fileName.slice(0, -3);
        if (!candidateBucket || candidateBucket.includes('/')) return null;
        if (bucket && candidateBucket !== bucket) return null;
        if (candidateId.normalize('NFC').toLowerCase() !== wantedIdKey) return null;
        return { bucket: candidateBucket, path: repoRelative };
      })
      .filter(Boolean)
      .sort((left, right) => left.path.localeCompare(right.path));
    if (candidates.length > 1) {
      throw new StoreError(
        'HISTORY_ID_AMBIGUOUS',
        `固定 integration ref ${planes.integrationRef} 中编号 ${safeId} 出现在多个历史 bucket，不能猜哪一份是真的`,
        { id: safeId, candidates },
      );
    }
    for (const candidate of candidates) {
      const shown = gitFacts.showFromRef(cwd, integrationOid, candidate.path);
      if (shown === null) {
        throw new StoreError(
          HISTORY_REF_UNAVAILABLE,
          `已经从 ${integrationOid} 枚举到历史 ${candidate.path}，但正文读取失败；不能把它当成编号不存在`,
          { ref: planes.integrationRef, oid: integrationOid, path: candidate.path },
        );
      }
      return { ...candidate, text: shown, integrationOid };
    }
    return null;
  }

  const found = localHistoryCandidates(planes, safeId, bucket);
  if (found.length > 1) {
    throw new StoreError(
      'HISTORY_ID_AMBIGUOUS',
      `编号 ${safeId} 出现在多个历史 bucket，不能猜哪一份是真的`,
      { id: safeId, candidates: found.map((entry) => entry.path) },
    );
  }
  return found[0] || null;
}

// ---------------------------------------------------------------------------
// generation 与图谱锁（跨工作树生效）
// ---------------------------------------------------------------------------

function generationPath(planes) {
  return path.join(planes.liveRoot, 'graph', 'generation');
}

function lockPath(planes) {
  return path.join(planes.liveRoot, 'graph', 'lock');
}

function readGeneration(planes) {
  const raw = readTextOrNull(generationPath(planes));
  if (raw === null) return 0;
  const value = Number.parseInt(raw.trim(), 10);
  return Number.isFinite(value) ? value : 0;
}

/**
 * 读取一份可拿去规划写事务的 live graph 快照。
 *
 * generation 不能在读完 records 之后才临时补取，否则调用方可能拿着旧图，却把
 * 新 generation 绑进写计划。这里同时观察图谱锁和前后 generation：读期间若有
 * 写事务经过，就明确报冲突；没有写事务经过时，返回的 generation 与 records
 * 属于同一份稳定快照。
 */
function readLiveSnapshot(planes) {
  const beforeLock = inspectLock(planes);
  if (beforeLock.held) {
    throw new StoreError(CONFLICT_LOCK_HELD, '读取 live graph 时图谱锁已被占用', beforeLock);
  }
  const generation = readGeneration(planes);
  const records = readLiveNodes(planes);
  const afterLock = inspectLock(planes);
  if (afterLock.held) {
    throw new StoreError(CONFLICT_LOCK_HELD, '读取 live graph 期间图谱锁被占用', afterLock);
  }
  const current = readGeneration(planes);
  if (current !== generation) {
    throw new StoreError(
      CONFLICT_STALE_GENERATION,
      `读取 live graph 期间 generation 已从 ${generation} 前进到 ${current}`,
      { expected: generation, actual: current },
    );
  }
  return { generation, records };
}

/**
 * 报告残留锁（pid / host / age），**不自动清除**。
 * 自初始化地把别人的锁抹掉就是静默覆盖，AGENTS.md 明令不许。
 */
function inspectLock(planes) {
  const directory = lockPath(planes);
  try {
    fs.statSync(directory);
  } catch (error) {
    if (error && error.code === 'ENOENT') {
      return { held: false, owner: null, ageMs: null, malformed: false };
    }
    throw storeReadFailed('stat', directory, error);
  }
  const owner = readTextOrNull(path.join(directory, 'owner.json'));
  if (owner === null) {
    return {
      held: true,
      owner: null,
      ageMs: null,
      malformed: true,
      reason: 'LOCK_OWNER_MISSING',
    };
  }
  let parsed = null;
  try {
    parsed = JSON.parse(owner);
  } catch (error) {
    parsed = null;
  }
  const startedAt = parsed && typeof parsed.startedAt === 'number' ? parsed.startedAt : null;
  return {
    held: true,
    owner: parsed,
    ageMs: startedAt === null ? null : Date.now() - startedAt,
    malformed: parsed === null,
    reason: parsed === null ? 'LOCK_OWNER_MALFORMED' : null,
  };
}

function acquireLock(planes) {
  const directory = lockPath(planes);
  fs.mkdirSync(path.dirname(directory), { recursive: true });
  try {
    fs.mkdirSync(directory);
  } catch (error) {
    throw new StoreError(CONFLICT_LOCK_HELD, '图谱锁已被占用，后写方报冲突', inspectLock(planes));
  }
  const owner = {
    pid: process.pid,
    host: require('node:os').hostname(),
    startedAt: Date.now(),
  };
  try {
    fs.writeFileSync(path.join(directory, 'owner.json'), `${JSON.stringify(owner)}\n`, 'utf8');
  } catch (error) {
    let cleanupError = null;
    try {
      fs.rmdirSync(directory);
    } catch (cleanup) {
      if (!cleanup || cleanup.code !== 'ENOENT') cleanupError = cleanup;
    }
    if (cleanupError) {
      throw new StoreError(
        'LOCK_INITIALIZATION_INCOMPLETE',
        '图谱锁目录已建立，但 owner 写入失败且无法清理；保留异常锁等待人工核对',
        {
          cause: error && error.message ? error.message : String(error),
          cleanup: cleanupError && cleanupError.message ? cleanupError.message : String(cleanupError),
        },
      );
    }
    throw new StoreError(
      'LOCK_INITIALIZATION_FAILED',
      '图谱锁 owner 写入失败，未开始任何写事务',
      { cause: error && error.message ? error.message : String(error) },
    );
  }
  return owner;
}

function releaseLock(planes) {
  const directory = lockPath(planes);
  try {
    fs.rmSync(path.join(directory, 'owner.json'), { force: true });
    fs.rmdirSync(directory);
  } catch (error) {
    // ENOTDIR 不是“已经释放”：它说明锁路径或祖先被异常对象替代，必须 fail closed。
    if (error && error.code === 'ENOENT') return;
    throw new StoreError(
      LOCK_RELEASE_FAILED,
      '图谱锁释放失败；不能把仍被锁住的控制面报告成普通成功',
      {
        lockPath: directory,
        cause: error && error.message ? error.message : String(error),
        causeCode: error && error.code ? error.code : null,
      },
    );
  }
}

// ---------------------------------------------------------------------------
// 原子写入
// ---------------------------------------------------------------------------

function atomicWrite(filePath, text) {
  const directory = path.dirname(filePath);
  fs.mkdirSync(directory, { recursive: true });
  const temporary = path.join(
    directory,
    `.${path.basename(filePath)}.${process.pid}.${crypto.randomBytes(6).toString('hex')}.tmp`,
  );
  const handle = fs.openSync(temporary, 'wx');
  try {
    fs.writeFileSync(handle, text, 'utf8');
    fs.fsyncSync(handle);
  } finally {
    fs.closeSync(handle);
  }
  fs.renameSync(temporary, filePath);
}

// ---------------------------------------------------------------------------
// 写事务
// ---------------------------------------------------------------------------

function normalizedPathKey(filePath) {
  return path.resolve(filePath).normalize('NFC').toLowerCase();
}

function pathInsideRoot(filePath, rootPath) {
  const candidate = normalizedPathKey(filePath);
  const root = normalizedPathKey(rootPath);
  return candidate === root || candidate.startsWith(`${root}${path.sep}`);
}

function canonicalRealpathOrThrow(filePath, label) {
  let real;
  try {
    real = fs.realpathSync(filePath);
  } catch (error) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      `${label} 无法解析 realpath；opening package 不得猜测路径身份`,
      {
        path: filePath,
        cause: error && error.message ? error.message : String(error),
        causeCode: error && error.code ? error.code : null,
      },
    );
  }
  return real;
}

/**
 * opening package 是唯一会写入新 linked worktree 的 extra file。它必须从不存在
 * 开始，并且目标、根目录、父目录在 plan 与 commit 两次核验时都保持同一 canonical
 * 路径。这里只接受已经存在的父目录：让 atomicWrite 递归创建一条尚未核验的父链，
 * 会重新引入 symlink/alias 逃逸窗口。
 */
function assertOpeningPackageExtra(extra, phase) {
  if (!extra || extra.role !== 'opening-package') return;
  const target = typeof extra.target === 'string' ? extra.target : '';
  const allowedRoot = typeof extra.allowedRoot === 'string' ? extra.allowedRoot : '';
  const detail = {
    phase: phase || null,
    target: target || null,
    allowedRoot: allowedRoot || null,
  };
  if (extra.mustBeAbsent !== true || extra.remove === true) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package 必须以 mustBeAbsent 新建，且没有删除或覆盖表示方式',
      detail,
    );
  }
  if (!path.isAbsolute(target) || !path.isAbsolute(allowedRoot)
    || path.resolve(target) !== target || path.resolve(allowedRoot) !== allowedRoot) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package 的 target 与 allowedRoot 必须是无 `.` / `..` 别名的绝对路径',
      detail,
    );
  }
  const rootRealpath = canonicalRealpathOrThrow(allowedRoot, 'opening package allowedRoot');
  if (rootRealpath !== allowedRoot) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package allowedRoot 不是自身 realpath，拒绝经 symlink 或路径别名写入',
      { ...detail, rootRealpath },
    );
  }
  let rootStat;
  try {
    rootStat = fs.statSync(rootRealpath);
  } catch (error) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package allowedRoot 无法读取',
      {
        ...detail,
        cause: error && error.message ? error.message : String(error),
        causeCode: error && error.code ? error.code : null,
      },
    );
  }
  if (!rootStat.isDirectory() || normalizedPathKey(target) === normalizedPathKey(rootRealpath)
    || !pathInsideRoot(target, rootRealpath)) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package target 必须严格位于 allowedRoot 之内',
      { ...detail, rootRealpath },
    );
  }
  const parent = path.dirname(target);
  const parentRealpath = canonicalRealpathOrThrow(parent, 'opening package parent');
  if (parentRealpath !== parent || !pathInsideRoot(parentRealpath, rootRealpath)) {
    throw new StoreError(
      OPENING_PACKAGE_PATH_INVALID,
      'opening package 父目录含 symlink/alias，或 realpath 已越出 allowedRoot',
      { ...detail, parent, parentRealpath, rootRealpath },
    );
  }
  try {
    fs.lstatSync(target);
    throw new StoreError(
      TARGET_ALREADY_EXISTS,
      `opening package 目标 ${target} 已存在，拒绝覆盖或接管`,
      detail,
    );
  } catch (error) {
    if (error instanceof StoreError) throw error;
    if (!isMissingError(error)) {
      throw new StoreError(
        OPENING_PACKAGE_PATH_INVALID,
        '无法确认 opening package 目标是否不存在',
        {
          ...detail,
          cause: error && error.message ? error.message : String(error),
          causeCode: error && error.code ? error.code : null,
        },
      );
    }
  }
}

function expectedContentDigest(value, label) {
  if (value === null) return null;
  if (typeof value === 'string' && /^[0-9a-f]{64}$/.test(value)) return value;
  throw new StoreError(
    CONFLICT_TARGET_CHANGED,
    `${label} 必须是 null 或小写 64 位 SHA-256`,
    { expectedTargetDigest: value === undefined ? null : value },
  );
}

function assertNoOtherHistoryIdentity(planes, id, target) {
  const otherLocal = localHistoryCandidates(planes, id, null)
    .filter((entry) => normalizedPathKey(entry.path) !== normalizedPathKey(target));
  const official = planes.tracked
    ? readHistoryNode(planes, id, { cwd: planes.repoRoot })
    : null;
  if (otherLocal.length === 0 && !official) return;
  throw new StoreError(
    HISTORY_BODY_FROZEN,
    `历史编号 ${id} 已存在，正文冻结；不能省略来源后换 bucket 重写或重复创建`,
    {
      id,
      target,
      existing: otherLocal.map((entry) => entry.path),
      official: official ? official.path : null,
    },
  );
}

function caseFoldCollision(filePath) {
  const directory = path.dirname(filePath);
  let entries;
  try {
    entries = fs.readdirSync(directory, { withFileTypes: true });
  } catch (error) {
    if (isMissingError(error)) return null;
    throw storeReadFailed('readdir', directory, error);
  }
  const expectedName = path.basename(filePath);
  const expectedKey = expectedName.normalize('NFC').toLowerCase();
  const collision = entries.find((entry) => entry.name !== expectedName
    && entry.name.normalize('NFC').toLowerCase() === expectedKey);
  return collision ? path.join(directory, collision.name) : null;
}

function targetConflict(entry, actualText) {
  if (!entry.targetMustBeAbsent || actualText === null) return null;
  if (entry.area === 'history') {
    return new StoreError(
      HISTORY_BODY_FROZEN,
      `节点 ${entry.id} 已在历史平面，正文冻结，不接受后写内容覆盖`,
      { id: entry.id, target: entry.target },
    );
  }
  return new StoreError(
    TARGET_ALREADY_EXISTS,
    `节点 ${entry.id} 的目标位置已被占用，拒绝静默覆盖`,
    { id: entry.id, target: entry.target },
  );
}

function removeStrict(filePath) {
  try {
    fs.rmSync(filePath);
  } catch (error) {
    if (isMissingError(error)) {
      throw new StoreError(
        CONFLICT_NODE_CHANGED,
        `待移动的源文件 ${filePath} 在提交期间消失，拒绝把它当成已删除`,
        { path: filePath },
      );
    }
    throw error;
  }
}

function snapshotPaths(paths) {
  return paths.map((filePath) => ({
    path: filePath,
    text: readTextOrNull(filePath),
  }));
}

function restoreSnapshots(snapshots) {
  const errors = [];
  for (const snapshot of snapshots.slice().reverse()) {
    try {
      if (snapshot.text === null) {
        const current = readTextOrNull(snapshot.path);
        if (current !== null) fs.rmSync(snapshot.path, { force: true });
      } else {
        atomicWrite(snapshot.path, snapshot.text);
      }
    } catch (error) {
      errors.push({
        path: snapshot.path,
        code: error && error.code ? error.code : null,
        message: error && error.message ? error.message : String(error),
      });
    }
  }
  return errors;
}

/**
 * 预检：算出这次要落哪些文件，并记下 generation 与每个将被改写节点的内容摘要。
 * 本函数**不写任何东西**——它就是 dry-run 的输出。
 */
function planNodeWrite(planes, changes, options) {
  const settings = options || {};
  const list = Array.isArray(changes) ? changes : [];
  const generation = settings.expectedGeneration === undefined
    ? readGeneration(planes)
    : settings.expectedGeneration;
  if (!Number.isInteger(generation) || generation < 0) {
    throw new StoreError(
      'EXPECTED_GENERATION_INVALID',
      `expectedGeneration 必须是非负整数，收到 ${generation}`,
      { expectedGeneration: generation },
    );
  }
  const entries = [];
  const targetKeys = new Map();
  const historyIds = new Map();
  for (const change of list) {
    const node = change.node;
    const target = nodeFilePath(planes, node);
    const previousPath = change.previousPath || null;
    const sameTarget = Boolean(previousPath && normalizedPathKey(previousPath) === normalizedPathKey(target));
    const targetArea = areaFor(node.lifecycle);
    if (planes.tracked === true && targetArea === 'history') {
      throw new StoreError(
        TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN,
        'tracked canonical HISTORY 禁止经普通 node write 写入；只能先写 mustBeAbsent candidate，再走 history finalization',
        { id: node.id, target },
      );
    }
    const sourceAlreadyInHistory = Boolean(
      previousPath && pathInsideRoot(previousPath, planes.historyRoot),
    );
    if (sourceAlreadyInHistory || (targetArea === 'history' && sameTarget)) {
      throw new StoreError(
        HISTORY_BODY_FROZEN,
        `节点 ${node.id} 已在历史平面，正文冻结，不接受覆盖、搬移或恢复为 current`,
        { id: node.id, source: previousPath, target },
      );
    }
    if (targetArea === 'history') {
      const historyId = String(node.id).normalize('NFC').toLowerCase();
      if (historyIds.has(historyId)) {
        throw new StoreError(
          HISTORY_BODY_FROZEN,
          `同一写事务不能为历史编号 ${node.id} 规划多个 bucket`,
          { id: node.id, first: historyIds.get(historyId), second: target },
        );
      }
      historyIds.set(historyId, target);
      assertNoOtherHistoryIdentity(planes, node.id, target);
    }
    const sourceText = previousPath ? readTextOrNull(previousPath) : null;
    if (previousPath && sourceText === null) {
      throw new StoreError(
        'SOURCE_NODE_MISSING',
        `待变更节点 ${node.id} 的源文件不存在，不能凭空创建一份替代内容`,
        { id: node.id, source: previousPath },
      );
    }
    const targetText = sameTarget ? sourceText : readTextOrNull(target);
    const targetMustBeAbsent = !previousPath || !sameTarget;
    const targetKey = normalizedPathKey(target);
    if (targetKeys.has(targetKey)) {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        `同一写事务里多个节点会落到等价目标路径，拒绝继续`,
        { first: targetKeys.get(targetKey), second: target },
      );
    }
    targetKeys.set(targetKey, target);
    const foldedCollision = caseFoldCollision(target);
    if (foldedCollision && (!previousPath
      || normalizedPathKey(foldedCollision) !== normalizedPathKey(previousPath))) {
      throw new StoreError(
        TARGET_ALREADY_EXISTS,
        `节点 ${node.id} 的目标与现有文件发生大小写或 Unicode 等价碰撞`,
        { id: node.id, target, collision: foldedCollision },
      );
    }
    const text = nodeSchema.serializeNode(node, change.body);
    entries.push({
      id: node.id,
      lifecycle: node.lifecycle,
      area: targetArea,
      target,
      previousPath,
      removePrevious: Boolean(previousPath && !sameTarget),
      targetMustBeAbsent,
      expectedSourceDigest: sourceText === null ? null : digestOf(sourceText),
      expectedTargetDigest: targetText === null ? null : digestOf(targetText),
      // expectedDigest 是旧调用方/诊断输出的兼容别名；提交逻辑分别冻结 source/target。
      expectedDigest: sourceText === null ? null : digestOf(sourceText),
      text,
      digest: digestOf(text),
    });
  }
  const extraFiles = (Array.isArray(settings.extraFiles) ? settings.extraFiles : []).map((extra) => {
    const target = extra.target;
    if (typeof target !== 'string' || target.trim() === '') {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        '额外文件缺少可回查的 target，不能把未知落点纳入原子事务',
      );
    }
    assertOpeningPackageExtra(extra, 'PLAN');
    const targetKey = normalizedPathKey(target);
    if (targetKeys.has(targetKey)) {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        '额外文件与节点写入落到同一个等价目标路径',
        { first: targetKeys.get(targetKey), second: target },
      );
    }
    targetKeys.set(targetKey, target);
    const current = readTextOrNull(target);
    const mustBeAbsent = extra.mustBeAbsent === true;
    const remove = extra.remove === true;
    const inPendingStartDirectory = normalizedPathKey(path.dirname(target))
      === normalizedPathKey(pendingStartDirectory(planes)) && target.endsWith('.md');
    // 普通 extra file 从来没有“删除”能力。唯一例外是 PENDING_START 的消费，
    // 并且仍由本事务的 generation/摘要预检和回滚包住，不能借通用接口删任意路径。
    if (remove && (extra.role !== 'pending-start' || !inPendingStartDirectory || mustBeAbsent)) {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        'extra remove 只允许消费 pending-start 目录中的固定 marker，不能删除任意文件',
        { target, role: extra.role || null, mustBeAbsent },
      );
    }
    if (!remove && typeof extra.text !== 'string') {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        '额外文件必须给出完整文本；不能把 undefined 当成安全写入',
        { target },
      );
    }
    const targetsTrackedHistory = planes.tracked === true
      && pathInsideRoot(target, planes.historyRoot);
    if (targetsTrackedHistory
      && (mustBeAbsent !== true || extra.role !== 'history-candidate')) {
      throw new StoreError(
        TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN,
        'tracked historyRoot 下的 extra file 只允许 mustBeAbsent history candidate',
        { target, mustBeAbsent, role: extra.role || null },
      );
    }
    // history candidate 的第一阶段必须“新建”，不是趁已有正文存在时偷偷覆盖。
    // 这里提前拒绝，commit 时还会再核一次，覆盖 inspect → write 间的竞争。
    if (mustBeAbsent && current !== null) {
      throw new StoreError(
        TARGET_ALREADY_EXISTS,
        `额外文件 ${target} 声明 mustBeAbsent，但目标已经存在，拒绝覆盖`,
        { target },
      );
    }
    if (remove && current === null) {
      throw new StoreError(
        PENDING_START_NOT_FOUND,
        `待消费的 PENDING_START ${target} 在预检时不存在，不能把它当成已消费`,
        { target },
      );
    }
    const foldedCollision = caseFoldCollision(target);
    if (foldedCollision && normalizedPathKey(foldedCollision) !== normalizedPathKey(target)) {
      throw new StoreError(
        TARGET_ALREADY_EXISTS,
        `额外文件 ${target} 与现有文件发生大小写或 Unicode 等价碰撞`,
        { target, collision: foldedCollision },
      );
    }
    const observedTargetDigest = current === null ? null : digestOf(current);
    const callerBoundDigest = Object.prototype.hasOwnProperty.call(extra, 'expectedTargetDigest')
      ? expectedContentDigest(extra.expectedTargetDigest, `额外文件 ${target} 的 expectedTargetDigest`)
      : observedTargetDigest;
    if (callerBoundDigest !== observedTargetDigest) {
      throw new StoreError(
        CONFLICT_TARGET_CHANGED,
        `额外文件 ${target} 已不再是调用方读取到的内容，拒绝以较新的现场覆盖旧 CAS`,
        { target, expected: callerBoundDigest, actual: observedTargetDigest },
      );
    }
    return {
      ...extra,
      mustBeAbsent,
      remove,
      expectedTargetDigest: callerBoundDigest,
    };
  });
  const guardRecords = [];
  const guardKeys = new Set();
  for (const guard of Array.isArray(settings.guardRecords) ? settings.guardRecords : []) {
    const guardPath = guard && typeof guard.path === 'string' ? guard.path : null;
    if (!guardPath) {
      throw new StoreError(
        CONFLICT_NODE_CHANGED,
        'guard record 缺少可回查的路径，拒绝把 parent/关系前置条件当成稳定事实',
      );
    }
    const key = normalizedPathKey(guardPath);
    if (guardKeys.has(key)) continue;
    guardKeys.add(key);
    const text = readTextOrNull(guardPath);
    if (text === null) {
      throw new StoreError(
        'SOURCE_NODE_MISSING',
        `guard record ${guard.id || guardPath} 在预检时不存在，不能继续规划写入`,
        { id: guard.id || null, path: guardPath },
      );
    }
    const observedDigest = digestOf(text);
    const callerBoundDigest = Object.prototype.hasOwnProperty.call(guard, 'expectedDigest')
      ? expectedContentDigest(guard.expectedDigest, `guard record ${guard.id || guardPath} 的 expectedDigest`)
      : observedDigest;
    if (callerBoundDigest !== observedDigest) {
      throw new StoreError(
        CONFLICT_NODE_CHANGED,
        `guard record ${guard.id || guardPath} 已不再是调用方读取到的内容`,
        { id: guard.id || null, path: guardPath, expected: callerBoundDigest, actual: observedDigest },
      );
    }
    guardRecords.push({
      id: guard.id || null,
      path: guardPath,
      expectedDigest: callerBoundDigest,
    });
  }
  return {
    generation,
    entries,
    extraFiles,
    guardRecords,
  };
}

// ---------------------------------------------------------------------------
// 两阶段 canonical history：候选落 Git 后，只消费 live source
// ---------------------------------------------------------------------------

/**
 * 第一阶段的 HISTORY 候选是一份普通 extra file，但它必须从“不存在”开始。
 * 调用方可以把它交给 planNodeWrite/commitNodeWrite 写到 integration worktree，
 * 随后由外部、显式的 Git 提交把它固定。store 永远不做 Git 写入。
 */
function historyCandidateExtra(target, text) {
  return {
    target,
    text,
    mustBeAbsent: true,
    role: 'history-candidate',
  };
}

function isCanonicalOid(value) {
  return typeof value === 'string' && /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(value);
}

function normalizeFinalizationIntegration(planes, options) {
  const settings = options || {};
  const integration = settings.integration;
  if (!planes || planes.tracked !== true) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'canonical history finalize 只适用于受 Git 跟踪的双平面仓库',
    );
  }
  if (!integration || typeof integration !== 'object' || Array.isArray(integration)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'canonical history finalize 需要 integration { ref, oid, cwd } 的冻结身份',
    );
  }
  const ref = integration.ref;
  const oid = integration.oid;
  const cwd = integration.cwd || settings.cwd || planes.repoRoot;
  if (typeof ref !== 'string' || !/^refs\/heads\/[^\s]+$/.test(ref)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'integration ref 必须是显式 refs/heads/<branch>，拒绝 HEAD、短名或 OID',
      { ref: ref || null },
    );
  }
  if (ref !== planes.integrationRef) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'finalize 的 integration ref 必须精确等于当前 canonical history 的固定 ref',
      { expected: planes.integrationRef, actual: ref },
    );
  }
  if (!isCanonicalOid(oid)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'finalize 必须绑定固定 integration commit OID',
      { oid: oid || null },
    );
  }
  if (typeof cwd !== 'string' || cwd.trim() === '') {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'finalize 缺少可回查的 integration 工作目录',
    );
  }
  return { ref, oid: oid.toLowerCase(), cwd };
}

function realpathOrNull(filePath) {
  try {
    return fs.realpathSync(String(filePath));
  } catch (error) {
    return null;
  }
}

function inspectFinalizationCheckout(planes, integration) {
  let repoRoot = null;
  let statusNow = null;
  try {
    repoRoot = gitFacts.repoRoot(integration.cwd);
    statusNow = gitFacts.porcelainStatus(integration.cwd);
  } catch (error) {
    throw new StoreError(
      HISTORY_FINALIZATION_CHECKOUT_MISMATCH,
      `无法读取 integration checkout 现实：${error.message}`,
      { cwd: integration.cwd, causeCode: error.code || null },
    );
  }
  const cwdReal = realpathOrNull(integration.cwd);
  const repoRootReal = realpathOrNull(repoRoot);
  const planeRepoRootReal = realpathOrNull(planes.repoRoot);
  const branchResult = gitFacts.runGit(
    ['rev-parse', '--abbrev-ref', 'HEAD'],
    { cwd: integration.cwd },
  );
  const refResult = gitFacts.runGit([
    'rev-parse',
    '--verify',
    '--end-of-options',
    `${integration.ref}^{commit}`,
  ], { cwd: integration.cwd });
  const currentBranch = branchResult.ok ? branchResult.stdout.trim() : null;
  const headOid = gitFacts.headOid(integration.cwd);
  const refOid = refResult.ok ? refResult.stdout.trim().toLowerCase() : null;
  const expectedBranch = integration.ref.slice('refs/heads/'.length);
  const reality = {
    cwd: cwdReal,
    repoRoot: repoRootReal,
    expectedRepoRoot: planeRepoRootReal,
    expectedBranch,
    currentBranch,
    headOid: headOid ? headOid.toLowerCase() : null,
    refOid,
    statusNow,
  };
  const valid = cwdReal !== null
    && repoRootReal !== null
    && planeRepoRootReal !== null
    && cwdReal === repoRootReal
    && repoRootReal === planeRepoRootReal
    && currentBranch === expectedBranch
    && reality.headOid === integration.oid
    && refOid === integration.oid
    && Array.isArray(statusNow)
    && statusNow.length === 0;
  if (!valid) {
    throw new StoreError(
      HISTORY_FINALIZATION_CHECKOUT_MISMATCH,
      'canonical history finalize 必须在绑定 ref 的干净 repo-root integration checkout 执行',
      reality,
    );
  }
  return reality;
}

function assertCheckoutStillBound(planes, finalization) {
  const actual = inspectFinalizationCheckout(planes, finalization.integration);
  const expected = finalization.checkout;
  if (!expected || JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new StoreError(
      HISTORY_FINALIZATION_CHECKOUT_MISMATCH,
      'integration checkout 现实已偏离 plan 绑定值，必须重新 inspect',
      { expected: expected || null, actual },
    );
  }
  return actual;
}

function repoRelativeHistoryPath(planes, target) {
  if (!planes.repoRoot || !planes.historyRoot
    || !pathInsideRoot(target, planes.historyRoot)
    || !pathInsideRoot(target, planes.repoRoot)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'history candidate 必须位于受管 repo 内的 canonical history 根下',
      { target, historyRoot: planes.historyRoot, repoRoot: planes.repoRoot },
    );
  }
  const relative = path.relative(planes.repoRoot, target);
  if (!relative || relative === '.' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      '无法把 history candidate 解析成安全的 repo 相对路径',
      { target, repoRoot: planes.repoRoot },
    );
  }
  return relative.split(path.sep).join('/');
}

function isLiveNodeRecordPath(planes, filePath) {
  if (!pathInsideRoot(filePath, planes.liveRoot)) return false;
  const relative = path.relative(planes.liveRoot, filePath);
  const parts = relative.split(path.sep);
  return parts.length === 2 && LIVE_NODE_AREAS.includes(parts[0]) && parts[1].endsWith('.md');
}

function finalizationSource(planes, change) {
  const node = change && change.node;
  const previousPath = change && change.previousPath;
  if (!node || node.lifecycle !== 'HISTORY' || typeof previousPath !== 'string'
    || !isLiveNodeRecordPath(planes, previousPath)) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'finalize 只允许把一份现存 live 节点消费为 HISTORY；不得直接删任意文件',
      { id: node && node.id ? node.id : null, previousPath: previousPath || null },
    );
  }
  const sourceText = readTextOrNull(previousPath);
  if (sourceText === null) {
    throw new StoreError(
      'SOURCE_NODE_MISSING',
      `待 finalize 的 live source ${previousPath} 不存在`,
      { id: node.id, source: previousPath },
    );
  }
  const parsed = nodeSchema.parseNode(sourceText, { relativePath: previousPath });
  const markers = parsed.node && Array.isArray(parsed.node.confirmations)
    ? parsed.node.confirmations.filter((entry) => entry
      && typeof entry === 'object'
      && entry.action === HISTORY_CANDIDATE_FREEZE_ACTION)
    : [];
  if (!parsed.node || parsed.issues.length > 0
    || parsed.node.id !== node.id || parsed.node.kind !== node.kind
    || parsed.node.lifecycle !== 'PARKED' || markers.length !== 1) {
    throw new StoreError(
      HISTORY_FINALIZATION_SOURCE_INVALID,
      'finalize source 必须是同编号同 kind 的 PARKED 节点，且恰有一份 history-candidate-freeze marker',
      {
        expectedId: node.id,
        actualId: parsed.node && parsed.node.id ? parsed.node.id : null,
        actualKind: parsed.node && parsed.node.kind ? parsed.node.kind : null,
        lifecycle: parsed.node && parsed.node.lifecycle ? parsed.node.lifecycle : null,
        markerCount: markers.length,
        issues: parsed.issues,
        source: previousPath,
      },
    );
  }
  return { previousPath, sourceText, parsed, marker: markers[0] };
}

function assertFreezeMarkerMatches(source, candidate, integration) {
  const marker = source && source.marker;
  const phase1Oid = marker && marker['phase1-integration-oid'];
  const markerMatches = marker
    && marker['candidate-path'] === candidate.repoPath
    && marker['candidate-digest'] === candidate.digest
    && marker['integration-ref'] === integration.ref
    && isCanonicalOid(phase1Oid);
  const phase1Exists = markerMatches
    && gitFacts.objectExists(integration.cwd, phase1Oid);
  const phase1IsAncestor = phase1Exists
    && gitFacts.isAncestor(integration.cwd, phase1Oid, integration.oid);
  if (!markerMatches || !phase1Exists || !phase1IsAncestor) {
    throw new StoreError(
      HISTORY_FINALIZATION_SOURCE_INVALID,
      'PARKED source 的 freeze marker 未绑定同一 candidate/ref，或 phase1 OID 不是固定 P2 OID 的可解析祖先',
      {
        id: candidate.id,
        marker: marker || null,
        expected: {
          candidatePath: candidate.repoPath,
          candidateDigest: candidate.digest,
          integrationRef: integration.ref,
          fixedIntegrationOid: integration.oid,
        },
        phase1Exists: Boolean(phase1Exists),
        phase1IsAncestor: Boolean(phase1IsAncestor),
      },
    );
  }
  const parentResult = gitFacts.runGit([
    'log',
    '-1',
    '--format=%P',
    integration.oid,
  ], { cwd: integration.cwd });
  const parents = parentResult.ok
    ? parentResult.stdout.trim().split(/\s+/).filter(Boolean)
    : [];
  let changedPaths = [];
  try {
    changedPaths = [...new Set(gitFacts.changedPaths(
      integration.cwd,
      phase1Oid,
      integration.oid,
    ))].sort();
  } catch (error) {
    changedPaths = [];
  }
  if (parents.length !== 1
    || parents[0].toLowerCase() !== phase1Oid.toLowerCase()
    || changedPaths.length !== 1
    || changedPaths[0] !== candidate.repoPath) {
    throw new StoreError(
      HISTORY_FINALIZATION_CANDIDATE_MISMATCH,
      'phase1 到 fixed OID 必须恰好是一份只改 history candidate 路径的外部提交',
      {
        id: candidate.id,
        phase1Oid,
        fixedIntegrationOid: integration.oid,
        expectedPath: candidate.repoPath,
        parents,
        changedPaths,
        parentReadError: parentResult.stderr || '',
      },
    );
  }
  return marker;
}

function parseFinalizationCandidate(candidate) {
  const parsed = nodeSchema.parseNode(candidate.text, {
    relativePath: candidate.repoPath,
  });
  if (!parsed.node || parsed.issues.length > 0
    || parsed.node.id !== candidate.id
    || parsed.node.lifecycle !== 'HISTORY') {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'history candidate after-image 必须是无 schema 问题的同编号 HISTORY 节点',
      {
        id: candidate.id,
        actualId: parsed.node && parsed.node.id ? parsed.node.id : null,
        lifecycle: parsed.node && parsed.node.lifecycle ? parsed.node.lifecycle : null,
        issues: parsed.issues,
      },
    );
  }
  return parsed;
}

/**
 * 规划 canonical history 的第二阶段。
 *
 * 这里不写 history target：目标正文应已由第一阶段写到 integration worktree，并由
 * 外部 Git commit 固定。写计划只保留“删哪份 live source、写哪份 CURRENT”以及
 * 期望的历史候选字节，真正 commit 时会用固定 ref + OID 复读验证。
 */
function planHistoryFinalization(planes, changes, options) {
  const settings = options || {};
  const integration = normalizeFinalizationIntegration(planes, settings);
  const checkout = inspectFinalizationCheckout(planes, integration);
  const list = Array.isArray(changes) ? changes : [];
  if (list.length === 0) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'canonical history finalize 至少需要一份 live→HISTORY after-image',
    );
  }

  // 借用普通计划器处理 extra CURRENT / guard 的 generation、摘要与路径碰撞；
  // history candidates 不作为普通 extra 传入，否则第二阶段会错误地重写它们。
  const base = planNodeWrite(planes, [], {
    expectedGeneration: settings.expectedGeneration,
    extraFiles: settings.extraFiles,
    guardRecords: settings.guardRecords,
  });
  const entries = [];
  const candidates = [];
  const sourceKeys = new Set();
  const targetKeys = new Set();
  const idKeys = new Set();
  for (const change of list) {
    const node = change && change.node;
    if (!node || node.lifecycle !== 'HISTORY') {
      throw new StoreError(
        HISTORY_FINALIZATION_INVALID,
        'canonical history finalize 的每个 after-image 都必须是 HISTORY 节点',
        { id: node && node.id ? node.id : null, lifecycle: node && node.lifecycle ? node.lifecycle : null },
      );
    }
    const source = finalizationSource(planes, change);
    const target = nodeFilePath(planes, node);
    const targetKey = normalizedPathKey(target);
    const sourceKey = normalizedPathKey(source.previousPath);
    const idKey = String(node.id).normalize('NFC').toLowerCase();
    if (targetKeys.has(targetKey) || sourceKeys.has(sourceKey) || idKeys.has(idKey)) {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        '同一 canonical history finalize 不能重复消费 source、目标路径或历史编号',
        { id: node.id, source: source.previousPath, target },
      );
    }
    targetKeys.add(targetKey);
    sourceKeys.add(sourceKey);
    idKeys.add(idKey);
    const text = nodeSchema.serializeNode(node, change.body);
    const candidate = {
      id: node.id,
      target,
      repoPath: repoRelativeHistoryPath(planes, target),
      text,
      digest: digestOf(text),
    };
    parseFinalizationCandidate(candidate);
    const freezeMarker = assertFreezeMarkerMatches(source, candidate, integration);
    entries.push({
      id: node.id,
      lifecycle: node.lifecycle,
      area: 'history',
      target,
      previousPath: source.previousPath,
      removePrevious: true,
      removeOnly: true,
      targetMustBeAbsent: false,
      expectedSourceDigest: digestOf(source.sourceText),
      expectedTargetDigest: null,
      expectedDigest: digestOf(source.sourceText),
      text,
      digest: digestOf(text),
    });
    candidates.push({
      ...candidate,
      freeze: {
        action: HISTORY_CANDIDATE_FREEZE_ACTION,
        'candidate-path': freezeMarker['candidate-path'],
        'candidate-digest': freezeMarker['candidate-digest'],
        'integration-ref': freezeMarker['integration-ref'],
        'phase1-integration-oid': freezeMarker['phase1-integration-oid'],
      },
    });
  }
  const reserved = new Set([...sourceKeys, ...targetKeys]);
  for (const extra of base.extraFiles) {
    const key = normalizedPathKey(extra.target);
    if (reserved.has(key)) {
      throw new StoreError(
        WRITE_PLAN_PATH_COLLISION,
        'canonical history candidate/source 不得与 extra CURRENT 或其他额外文件重叠',
        { target: extra.target },
      );
    }
    reserved.add(key);
  }
  return {
    ...base,
    entries,
    historyFinalization: {
      integration,
      checkout,
      candidates,
    },
  };
}

function resolvedIntegrationOid(integration) {
  const resolved = gitFacts.runGit([
    'rev-parse',
    '--verify',
    '--end-of-options',
    `${integration.ref}^{commit}`,
  ], { cwd: integration.cwd });
  const actual = resolved.ok ? resolved.stdout.trim().toLowerCase() : null;
  if (!actual || actual !== integration.oid) {
    throw new StoreError(
      HISTORY_FINALIZATION_REF_MISMATCH,
      `fixed integration ref ${integration.ref} 未指向 inspect 绑定的 ${integration.oid}`,
      {
        ref: integration.ref,
        expectedOid: integration.oid,
        actualOid: actual,
        stderr: resolved.stderr || '',
      },
    );
  }
  return actual;
}

function historyIdentityPathsAtOid(planes, integration, id) {
  const root = String(planes.historyRepoPath || DEFAULT_HISTORY_ROOT)
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '')
    .replace(/\/+$/, '');
  const listed = gitFacts.runGit([
    'ls-tree',
    '-r',
    '-z',
    '--name-only',
    integration.oid,
    '--',
    root,
  ], { cwd: integration.cwd });
  if (!listed.ok) {
    throw new StoreError(
      HISTORY_FINALIZATION_REF_MISMATCH,
      `无法从固定 integration OID ${integration.oid} 枚举 history candidate`,
      { ref: integration.ref, oid: integration.oid, stderr: listed.stderr || '' },
    );
  }
  const wanted = String(id).normalize('NFC').toLowerCase();
  const prefix = `${root}/`;
  return listed.stdout
    .split('\0')
    .filter(Boolean)
    .filter((repoPath) => repoPath.startsWith(prefix) && repoPath.endsWith('.md'))
    .filter((repoPath) => {
      const relative = repoPath.slice(prefix.length);
      const slash = relative.indexOf('/');
      if (slash <= 0) return false;
      const fileName = relative.slice(slash + 1);
      return fileName.slice(0, -3).normalize('NFC').toLowerCase() === wanted;
    })
    .sort();
}

/**
 * 第二阶段提交前的历史现实核验。使用固定 OID show 正文，并在前后各读一次 ref，
 * 因此不会把“刚才还指向候选”的可移动 branch 当成已冻结事实。
 */
function verifyCommittedHistoryCandidates(planes, finalization) {
  const integration = finalization && finalization.integration;
  const candidates = finalization && Array.isArray(finalization.candidates)
    ? finalization.candidates
    : [];
  if (!integration || candidates.length === 0) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'history finalize 缺少冻结 integration 或候选 history after-image',
    );
  }
  resolvedIntegrationOid(integration);
  for (const candidate of candidates) {
    const paths = historyIdentityPathsAtOid(planes, integration, candidate.id);
    if (paths.length !== 1 || paths[0] !== candidate.repoPath) {
      throw new StoreError(
        HISTORY_FINALIZATION_CANDIDATE_MISMATCH,
        `固定 integration ref 中 history 编号 ${candidate.id} 的路径不等于预期 after-image`,
        {
          id: candidate.id,
          expectedPath: candidate.repoPath,
          actualPaths: paths,
          oid: integration.oid,
        },
      );
    }
    const fromRef = gitFacts.showFromRef(integration.cwd, integration.oid, candidate.repoPath);
    if (fromRef === null || digestOf(fromRef) !== candidate.digest || fromRef !== candidate.text) {
      throw new StoreError(
        HISTORY_FINALIZATION_CANDIDATE_MISMATCH,
        `固定 integration ref 中 history candidate ${candidate.repoPath} 的正文不等于预期 after-image`,
        {
          id: candidate.id,
          path: candidate.repoPath,
          expectedDigest: candidate.digest,
          actualDigest: fromRef === null ? null : digestOf(fromRef),
          oid: integration.oid,
        },
      );
    }
    const local = readTextOrNull(candidate.target);
    if (local === null || digestOf(local) !== candidate.digest || local !== candidate.text) {
      throw new StoreError(
        HISTORY_FINALIZATION_CANDIDATE_MISMATCH,
        `integration worktree 中 history candidate ${candidate.target} 不等于已提交 after-image`,
        {
          id: candidate.id,
          path: candidate.target,
          expectedDigest: candidate.digest,
          actualDigest: local === null ? null : digestOf(local),
        },
      );
    }
  }
  // 读完正文后再核 ref，避免 ref 在 ls-tree/show 期间前移却被当成同一份固定事实。
  resolvedIntegrationOid(integration);
}

/**
 * 消费已经提交并核验过的 canonical history candidate。
 *
 * 注意：本函数绝不调用 atomicWrite(candidate.target, ...)，也绝不删除 candidate。
 * 它只删除 shared live source、写 extra CURRENT，并推进 generation；任一步失败都
 * 用同一事务的快照回滚这三类控制面文件，Git history 完全不在写集合里。
 */
function commitHistoryFinalization(planes, writePlan) {
  const finalization = writePlan && writePlan.historyFinalization;
  if (!finalization) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'commitHistoryFinalization 只能提交 planHistoryFinalization 生成的写计划',
    );
  }
  const candidateById = new Map(
    (Array.isArray(finalization.candidates) ? finalization.candidates : [])
      .map((candidate) => [candidate && candidate.id, candidate]),
  );
  if (!Array.isArray(writePlan.entries) || writePlan.entries.length === 0
    || candidateById.size !== writePlan.entries.length) {
    throw new StoreError(
      HISTORY_FINALIZATION_INVALID,
      'history finalize 的 live removals 与 history candidates 必须一一对应',
    );
  }
  acquireLock(planes);
  let releaseWhenDone = true;
  let applied = null;
  try {
    const current = readGeneration(planes);
    if (current !== writePlan.generation) {
      throw new StoreError(
        CONFLICT_STALE_GENERATION,
        `图谱 generation 已从 ${writePlan.generation} 前进到 ${current}，finalize 拒绝使用旧快照`,
        { expected: writePlan.generation, actual: current },
      );
    }
    assertCheckoutStillBound(planes, finalization);
    for (const entry of writePlan.entries || []) {
      if (entry.removeOnly !== true || entry.area !== 'history' || !entry.previousPath) {
        throw new StoreError(
          HISTORY_FINALIZATION_INVALID,
          'history finalize 写计划含有非“只移除 live source”的条目，拒绝降级为普通 history 写入',
          { id: entry && entry.id ? entry.id : null },
        );
      }
      const candidate = candidateById.get(entry.id);
      if (!candidate || candidate.target !== entry.target
        || candidate.text !== entry.text || candidate.digest !== entry.digest
        || candidate.repoPath !== repoRelativeHistoryPath(planes, entry.target)) {
        throw new StoreError(
          HISTORY_FINALIZATION_INVALID,
          'history finalize 的 candidate 必须逐项等于要消费 source 的 HISTORY after-image',
          { id: entry.id },
        );
      }
      const parsedCandidate = parseFinalizationCandidate(candidate);
      const source = finalizationSource(planes, {
        node: parsedCandidate.node,
        previousPath: entry.previousPath,
      });
      const marker = assertFreezeMarkerMatches(source, candidate, finalization.integration);
      const frozenMarker = {
        action: HISTORY_CANDIDATE_FREEZE_ACTION,
        'candidate-path': marker['candidate-path'],
        'candidate-digest': marker['candidate-digest'],
        'integration-ref': marker['integration-ref'],
        'phase1-integration-oid': marker['phase1-integration-oid'],
      };
      if (JSON.stringify(frozenMarker) !== JSON.stringify(candidate.freeze)) {
        throw new StoreError(
          HISTORY_FINALIZATION_SOURCE_INVALID,
          'commit 时 PARKED source marker 与 plan 绑定的 freeze marker 不一致',
          { id: entry.id, expected: candidate.freeze || null, actual: frozenMarker },
        );
      }
      const sourceDigest = digestOf(source.sourceText);
      if (sourceDigest !== entry.expectedSourceDigest) {
        throw new StoreError(
          CONFLICT_NODE_CHANGED,
          `finalize source ${entry.id} 在预检之后被改过，拒绝删除`,
          { id: entry.id, expected: entry.expectedSourceDigest, actual: sourceDigest },
        );
      }
    }
    for (const extra of writePlan.extraFiles || []) {
      const currentText = readTextOrNull(extra.target);
      if (extra.mustBeAbsent === true && currentText !== null) {
        throw new StoreError(
          TARGET_ALREADY_EXISTS,
          `额外文件 ${extra.target} 声明 mustBeAbsent，但目标已在 finalize 前出现`,
          { target: extra.target },
        );
      }
      const currentDigest = currentText === null ? null : digestOf(currentText);
      if (currentDigest !== extra.expectedTargetDigest) {
        throw new StoreError(
          CONFLICT_TARGET_CHANGED,
          `finalize 的额外文件 ${extra.target} 在预检之后被改过，拒绝覆盖`,
          { target: extra.target, expected: extra.expectedTargetDigest, actual: currentDigest },
        );
      }
    }
    for (const guard of writePlan.guardRecords || []) {
      const currentText = readTextOrNull(guard.path);
      const currentDigest = currentText === null ? null : digestOf(currentText);
      if (currentDigest !== guard.expectedDigest) {
        throw new StoreError(
          CONFLICT_NODE_CHANGED,
          `finalize guard ${guard.id || guard.path} 在预检之后被改过`,
          { id: guard.id || null, path: guard.path, expected: guard.expectedDigest, actual: currentDigest },
        );
      }
    }

    // 这一步紧邻任何控制面写入：固定 ref、OID、路径、正文都必须在此刻仍一致。
    verifyCommittedHistoryCandidates(planes, finalization);
    assertCheckoutStillBound(planes, finalization);

    const affected = new Map();
    const remember = (filePath) => {
      const key = path.resolve(filePath);
      if (!affected.has(key)) affected.set(key, filePath);
    };
    remember(generationPath(planes));
    for (const entry of writePlan.entries || []) remember(entry.previousPath);
    for (const extra of writePlan.extraFiles || []) remember(extra.target);
    const before = snapshotPaths([...affected.values()]);
    const removed = [];
    const written = [];
    try {
      for (const entry of writePlan.entries || []) {
        removeStrict(entry.previousPath);
        removed.push(entry.previousPath);
      }
      for (const extra of writePlan.extraFiles || []) {
        atomicWrite(extra.target, extra.text);
        written.push(extra.target);
      }
      const nextGeneration = writePlan.generation + 1;
      atomicWrite(generationPath(planes), `${nextGeneration}\n`);
      // Git ref / checkout 不在 shared graph lock 的管辖内；控制面写完后立即再读。
      // 若外部恰在窗口内移动 ref、换分支或改 candidate，下面抛错会进入同一回滚。
      verifyCommittedHistoryCandidates(planes, finalization);
      assertCheckoutStillBound(planes, finalization);
      applied = {
        ok: true,
        generation: nextGeneration,
        written,
        removed,
        historyCandidates: finalization.candidates.map((candidate) => candidate.target),
      };
      return applied;
    } catch (error) {
      const rollbackErrors = restoreSnapshots(before);
      if (rollbackErrors.length > 0) {
        releaseWhenDone = false;
        throw new StoreError(
          TRANSACTION_ROLLBACK_INCOMPLETE,
          'history finalize 中途失败且无法完整回滚 live source/CURRENT；保留图谱锁等待人工核对',
          {
            cause: error && error.message ? error.message : String(error),
            causeCode: error && error.code ? error.code : null,
            rollbackErrors,
          },
        );
      }
      throw error;
    }
  } finally {
    if (releaseWhenDone) {
      try {
        releaseLock(planes);
      } catch (error) {
        if (applied) {
          throw new StoreError(
            COMMIT_APPLIED_LOCK_RELEASE_FAILED,
            `history finalize 已完整落地到 generation ${applied.generation}，但图谱锁释放失败；不得重试写入`,
            {
              generation: applied.generation,
              written: applied.written,
              removed: applied.removed,
              releaseError: error && error.detail ? error.detail : {
                message: error && error.message ? error.message : String(error),
              },
            },
          );
        }
        throw error;
      }
    }
  }
}

/**
 * 提交：加锁 → 重验 generation → 逐节点比对摘要 → 临时文件 + rename → generation 递增。
 * 任何一步对不上就报冲突并**不写**，绝不把别人的内容静默覆盖（EX-9 第 4 条、WK-1）。
 */
function commitNodeWrite(planes, writePlan) {
  acquireLock(planes);
  let releaseWhenDone = true;
  let applied = null;
  try {
    const current = readGeneration(planes);
    if (current !== writePlan.generation) {
      throw new StoreError(
        CONFLICT_STALE_GENERATION,
        `图谱 generation 已从 ${writePlan.generation} 前进到 ${current}，后写方报冲突`,
        { expected: writePlan.generation, actual: current },
      );
    }
    // marker removal 只接受 finalizePendingStart 内部产生的 authority，同时复核
    // 规划时冻结的 canonical history absence。
    assertPendingFinalizationPlan(planes, writePlan);
    if (writePlan.pendingStartFinalization) {
      // 这两项都必须位于 graph lock 内，不能让任务层的“刚才看过”替代最后事实。
      assertPendingStartFinalizationLiveIdentity(planes, writePlan, 'PRE');
      assertPendingStartFinalizationReality(planes, writePlan, 'PRE');
    }
    for (const entry of writePlan.entries) {
      if (planes.tracked === true && entry.area === 'history') {
        throw new StoreError(
          TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN,
          'tracked canonical HISTORY 禁止经普通 commitNodeWrite 落盘',
          { id: entry.id, target: entry.target },
        );
      }
      if (entry.area === 'history') {
        assertNoOtherHistoryIdentity(planes, entry.id, entry.target);
      }
      if (entry.previousPath) {
        const sourceText = readTextOrNull(entry.previousPath);
        const sourceDigest = sourceText === null ? null : digestOf(sourceText);
        if (sourceDigest !== entry.expectedSourceDigest) {
          throw new StoreError(
            CONFLICT_NODE_CHANGED,
            `节点 ${entry.id} 的源内容在预检之后被改过，后写方报冲突`,
            { id: entry.id, expected: entry.expectedSourceDigest, actual: sourceDigest },
          );
        }
      }
      const targetText = entry.previousPath === entry.target
        ? readTextOrNull(entry.target)
        : readTextOrNull(entry.target);
      const occupied = targetConflict(entry, targetText);
      if (occupied) throw occupied;
      const targetDigest = targetText === null ? null : digestOf(targetText);
      if (targetDigest !== entry.expectedTargetDigest) {
        throw new StoreError(
          CONFLICT_TARGET_CHANGED,
          `节点 ${entry.id} 的目标位置在预检之后被改过，后写方报冲突`,
          { id: entry.id, expected: entry.expectedTargetDigest, actual: targetDigest },
        );
      }
      const foldedCollision = caseFoldCollision(entry.target);
      if (foldedCollision && (!entry.previousPath
        || normalizedPathKey(foldedCollision) !== normalizedPathKey(entry.previousPath))) {
        throw new StoreError(
          TARGET_ALREADY_EXISTS,
          `节点 ${entry.id} 的目标与现有文件发生大小写或 Unicode 等价碰撞`,
          { id: entry.id, target: entry.target, collision: foldedCollision },
        );
      }
    }
    for (const extra of writePlan.extraFiles || []) {
      const remove = extra.remove === true;
      assertOpeningPackageExtra(extra, 'COMMIT');
      const targetsTrackedHistory = planes.tracked === true
        && pathInsideRoot(extra.target, planes.historyRoot);
      if (targetsTrackedHistory
        && (extra.mustBeAbsent !== true || extra.role !== 'history-candidate')) {
        throw new StoreError(
          TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN,
          'tracked historyRoot 下的 extra file 只允许 mustBeAbsent history candidate',
          { target: extra.target, mustBeAbsent: extra.mustBeAbsent === true, role: extra.role || null },
        );
      }
      const currentText = readTextOrNull(extra.target);
      if (remove) {
        const inPendingStartDirectory = normalizedPathKey(path.dirname(extra.target))
          === normalizedPathKey(pendingStartDirectory(planes)) && extra.target.endsWith('.md');
        if (extra.role !== 'pending-start' || !inPendingStartDirectory || extra.mustBeAbsent === true) {
          throw new StoreError(
            WRITE_PLAN_PATH_COLLISION,
            'extra remove 只允许消费 pending-start 目录中的固定 marker，不能删除任意文件',
            { target: extra.target, role: extra.role || null },
          );
        }
        if (currentText === null) {
          throw new StoreError(
            PENDING_START_NOT_FOUND,
            `待消费的 PENDING_START ${extra.target} 在提交前消失，拒绝把它当成已消费`,
            { target: extra.target },
          );
        }
      }
      if (extra.mustBeAbsent === true && currentText !== null) {
        throw new StoreError(
          TARGET_ALREADY_EXISTS,
          `额外文件 ${extra.target} 声明 mustBeAbsent，但目标已在提交前出现`,
          { target: extra.target },
        );
      }
      const currentDigest = currentText === null ? null : digestOf(currentText);
      if (currentDigest !== extra.expectedTargetDigest) {
        throw new StoreError(
          CONFLICT_TARGET_CHANGED,
          `额外文件 ${extra.target} 在预检之后被改过，后写方报冲突`,
          { target: extra.target, expected: extra.expectedTargetDigest, actual: currentDigest },
        );
      }
      const foldedCollision = caseFoldCollision(extra.target);
      if (foldedCollision && normalizedPathKey(foldedCollision) !== normalizedPathKey(extra.target)) {
        throw new StoreError(
          TARGET_ALREADY_EXISTS,
          `额外文件 ${extra.target} 与现有文件发生大小写或 Unicode 等价碰撞`,
          { target: extra.target, collision: foldedCollision },
        );
      }
    }
    for (const guard of writePlan.guardRecords || []) {
      const currentText = readTextOrNull(guard.path);
      const currentDigest = currentText === null ? null : digestOf(currentText);
      if (currentDigest !== guard.expectedDigest) {
        throw new StoreError(
          CONFLICT_NODE_CHANGED,
          `guard record ${guard.id || guard.path} 在预检之后被改过，拒绝把失效 parent/关系前置条件写入`,
          { id: guard.id || null, path: guard.path, expected: guard.expectedDigest, actual: currentDigest },
        );
      }
    }

    const affected = new Map();
    const remember = (filePath) => {
      const key = path.resolve(filePath);
      if (!affected.has(key)) affected.set(key, filePath);
    };
    // generation 最后写、回滚时最后恢复，外部不会看到“旧文件 + 新 generation”。
    remember(generationPath(planes));
    for (const entry of writePlan.entries) {
      remember(entry.target);
      if (entry.removePrevious) remember(entry.previousPath);
    }
    for (const extra of writePlan.extraFiles || []) remember(extra.target);
    const before = snapshotPaths([...affected.values()]);

    const written = [];
    try {
      for (const entry of writePlan.entries) {
        atomicWrite(entry.target, entry.text);
        if (entry.removePrevious) removeStrict(entry.previousPath);
        written.push(entry.target);
      }
      for (const extra of writePlan.extraFiles || []) {
        if (extra.remove === true) removeStrict(extra.target);
        else atomicWrite(extra.target, extra.text);
        written.push(extra.target);
      }
      const nextGeneration = writePlan.generation + 1;
      atomicWrite(generationPath(planes), `${nextGeneration}\n`);
      // history plane / Git ref 不受 graph lock 管辖；落盘后再读一次，漂移即回滚。
      if (writePlan.pendingStartFinalization) {
        assertPendingHistoryAbsence(planes, writePlan.pendingStartFinalization);
        // 外部直接写入和 Git/工作树现实都不受 graph lock 管辖，故控制面写完后
        // 仍要在 rollback 包围内重验；任何一项不成立都不能返回 ACTIVE。
        assertPendingStartFinalizationLiveIdentity(planes, writePlan, 'POST');
        assertPendingStartFinalizationReality(planes, writePlan, 'POST');
      }
      applied = { ok: true, generation: nextGeneration, written };
      return applied;
    } catch (error) {
      const rollbackErrors = restoreSnapshots(before);
      if (rollbackErrors.length > 0) {
        releaseWhenDone = false;
        throw new StoreError(
          TRANSACTION_ROLLBACK_INCOMPLETE,
          '写事务中途失败，而且无法完整回滚；保留图谱锁，必须人工核对',
          {
            cause: error && error.message ? error.message : String(error),
            causeCode: error && error.code ? error.code : null,
            rollbackErrors,
          },
        );
      }
      throw error;
    }
  } finally {
    PENDING_START_REALITY_PLANS.delete(writePlan);
    if (releaseWhenDone) {
      try {
        releaseLock(planes);
      } catch (error) {
        if (applied) {
          throw new StoreError(
            COMMIT_APPLIED_LOCK_RELEASE_FAILED,
            `写事务已经完整落地到 generation ${applied.generation}，但图谱锁释放失败；不得重试写入`,
            {
              generation: applied.generation,
              written: applied.written,
              releaseError: error && error.detail ? error.detail : {
                message: error && error.message ? error.message : String(error),
              },
            },
          );
        }
        throw error;
      }
    }
  }
}

// ---------------------------------------------------------------------------
// PENDING_START 原子接口
// ---------------------------------------------------------------------------

function requirePendingGeneration(options) {
  const settings = options || {};
  const value = settings.expectedGeneration;
  if (!Number.isInteger(value) || value < 0) {
    throw new StoreError(
      PENDING_START_CAS_REQUIRED,
      'PENDING_START 写入必须携带读取快照的 expectedGeneration；不能把并发记录当作当前值覆盖',
      { expectedGeneration: value === undefined ? null : value },
    );
  }
  return value;
}

function requirePendingDigest(options) {
  const settings = options || {};
  const value = settings.expectedDigest;
  if (typeof value !== 'string' || !/^[0-9a-f]{64}$/.test(value)) {
    throw new StoreError(
      PENDING_START_CAS_REQUIRED,
      'PENDING_START 更新/消费必须携带读取到的 expectedDigest',
      { expectedDigest: value === undefined ? null : value },
    );
  }
  return value;
}

function pendingResult(record, filePath, text, applied) {
  return {
    record,
    id: record.id,
    path: filePath,
    digest: record.digest,
    contentDigest: digestOf(text),
    generation: applied.generation,
    written: applied.written.slice(),
  };
}

function assertPendingGeneration(entry, expectedGeneration) {
  if (entry.generation !== expectedGeneration) {
    throw new StoreError(
      CONFLICT_STALE_GENERATION,
      `PENDING_START ${entry.id} 的 generation 已从 ${expectedGeneration} 前进到 ${entry.generation}`,
      { id: entry.id, expected: expectedGeneration, actual: entry.generation },
    );
  }
}

function assertPendingDigest(entry, expectedDigest) {
  if (entry.digest !== expectedDigest) {
    throw new StoreError(
      CONFLICT_PENDING_START_DIGEST,
      `PENDING_START ${entry.id} 的 digest 已变化；不能用旧 proposal 身份继续`,
      { id: entry.id, expected: expectedDigest, actual: entry.digest },
    );
  }
}

/**
 * start 的第一笔控制面写。先读完整 pending 目录以发现损坏/重复 marker，再把
 * “目标必须不存在”连同 generation 放进同一把 graph lock 的原子事务。
 */
function createPendingStart(planes, input, options) {
  const expectedGeneration = requirePendingGeneration(options);
  const record = pendingStart.createPendingStartRecord(input);
  const snapshot = readPendingStartSnapshot(planes);
  if (snapshot.generation !== expectedGeneration) {
    throw new StoreError(
      CONFLICT_STALE_GENERATION,
      `创建 PENDING_START 前 generation 已从 ${expectedGeneration} 前进到 ${snapshot.generation}`,
      { expected: expectedGeneration, actual: snapshot.generation, id: record.id },
    );
  }
  const existing = snapshot.records.find((entry) => (
    entry.id.normalize('NFC').toLowerCase() === record.id.normalize('NFC').toLowerCase()
  ));
  if (existing) {
    throw new StoreError(
      TARGET_ALREADY_EXISTS,
      `PENDING_START ${record.id} 已存在，不能用第二份 proposal 覆盖第一份`,
      { id: record.id, existing: existing.path },
    );
  }
  const target = pendingStartPath(planes, record.id);
  const text = pendingStart.serializePendingStart(record);
  const plan = planNodeWrite(planes, [], {
    expectedGeneration,
    extraFiles: [{
      target,
      text,
      mustBeAbsent: true,
      role: 'pending-start',
    }],
  });
  const applied = commitNodeWrite(planes, plan);
  return pendingResult(record, target, text, applied);
}

/** 更新只允许 PENDING_START 自身的 status / steps / resources，绑定身份不可改。 */
function updatePendingStart(planes, id, patch, options) {
  const expectedGeneration = requirePendingGeneration(options);
  const expectedDigest = requirePendingDigest(options);
  const current = readPendingStart(planes, id);
  if (!current) {
    throw new StoreError(
      PENDING_START_NOT_FOUND,
      `PENDING_START ${id} 不存在，不能凭空更新或把缺失当成已恢复`,
      { id },
    );
  }
  assertPendingGeneration(current, expectedGeneration);
  assertPendingDigest(current, expectedDigest);
  const next = pendingStart.updatePendingStartRecord(current.record, patch);
  const text = pendingStart.serializePendingStart(next);
  const plan = planNodeWrite(planes, [], {
    expectedGeneration,
    extraFiles: [{
      target: current.path,
      text,
      role: 'pending-start',
      expectedTargetDigest: current.contentDigest,
    }],
  });
  const applied = commitNodeWrite(planes, plan);
  return pendingResult(next, current.path, text, applied);
}

/**
 * 保留导出名只为让旧内部调用 fail closed，而不是在升级后变成 `undefined`。
 *
 * 成功开工只能由 finalizePendingStart 在同一事务中写 ACTIVE/CURRENT 并消费 marker。
 * READY_FOR_CONFIRMED_REMOVAL 也不能只凭 generation/digest 删除：未来若实现物理清理，
 * 必须另设同时核验物理现实与本次单独确认收据的专用流程。
 */
function consumePendingStart(_planes, id, _options) {
  throw new StoreError(
    PENDING_START_STANDALONE_CONSUME_FORBIDDEN,
    `禁止单独消费 PENDING_START ${id || '（未指定）'}；成功只允许 finalize，清理仍需物理现实与单独确认收据`,
    { id: id || null },
  );
}

function sameOrderedStrings(left, right) {
  return Array.isArray(left)
    && Array.isArray(right)
    && left.length === right.length
    && left.every((item, index) => typeof item === 'string' && item === right[index]);
}

function pendingFinalizationInvalid(message, detail) {
  throw new StoreError(
    PENDING_START_FINALIZATION_INVALID,
    message,
    detail || null,
  );
}

function pendingHistoryUnavailable(id, message, detail, cause) {
  throw new StoreError(
    PENDING_START_HISTORY_REALITY_UNAVAILABLE,
    message,
    {
      id,
      ...(detail || {}),
      causeCode: cause && cause.code ? cause.code : null,
      cause: cause && cause.message ? cause.message : null,
    },
  );
}

function pendingHistoryConflict(id, candidates) {
  throw new StoreError(
    PENDING_START_HISTORY_ID_CONFLICT,
    `canonical history 已存在与 ${id} 等价的编号，不能再把同一身份收口为 ACTIVE`,
    { id, candidates },
  );
}

function normalizedHistoryRepoPath(planes) {
  const raw = String(planes.historyRepoPath || DEFAULT_HISTORY_ROOT)
    .replace(/\\/g, '/')
    .replace(/^\.\/+/, '')
    .replace(/\/+$/, '');
  const segments = raw.split('/');
  if (!raw || path.posix.isAbsolute(raw)
    || segments.some((segment) => !segment || segment === '.' || segment === '..')) {
    pendingHistoryUnavailable(
      null,
      'canonical history repo path 不是安全的 repo 相对路径，不能证明编号不存在',
      { historyRepoPath: raw || null },
    );
  }
  return segments.join('/');
}

function resolvePendingHistoryOid(ref, cwd, id) {
  let resolved;
  try {
    resolved = gitFacts.runGit([
      'rev-parse',
      '--verify',
      '--end-of-options',
      `${ref}^{commit}`,
    ], { cwd });
  } catch (error) {
    pendingHistoryUnavailable(
      id,
      `无法读取固定 integration ref ${ref}，不能把 history 读失败当成编号不存在`,
      { ref, cwd },
      error,
    );
  }
  const oid = resolved && resolved.ok ? resolved.stdout.trim().toLowerCase() : '';
  if (!isCanonicalOid(oid)) {
    pendingHistoryUnavailable(
      id,
      `无法把固定 integration ref ${ref} 冻结成单一 commit OID`,
      { ref, cwd, stderr: resolved && resolved.stderr ? resolved.stderr : '' },
    );
  }
  return oid;
}

/**
 * 规划开工收口所依赖的“历史中无同 ID”事实。tracked history 不读任务工作树，
 * 而是冻结 canonical integration ref 的 OID、repo realpath 与 history repo path。
 */
function planPendingHistoryAbsence(planes, id) {
  const safeId = assertSafeIdSegment(id);
  if (planes.tracked !== true) {
    const guard = {
      authority: PENDING_START_FINALIZATION_AUTHORITY,
      version: 1,
      tracked: false,
      id: safeId,
      historyRoot: path.resolve(planes.historyRoot),
    };
    assertPendingHistoryAbsence(planes, guard);
    return guard;
  }

  const ref = planes.integrationRef;
  const repoRoot = realpathOrNull(planes.repoRoot);
  const historyRepoPath = normalizedHistoryRepoPath(planes);
  if (typeof ref !== 'string' || !ref.startsWith('refs/heads/') || !repoRoot) {
    pendingHistoryUnavailable(
      safeId,
      '缺少可固定的 canonical integration ref 或 repo realpath，不能证明历史编号不存在',
      { ref: ref || null, repoRoot: planes.repoRoot || null },
    );
  }
  const expectedHistoryRoot = path.resolve(repoRoot, ...historyRepoPath.split('/'));
  if (normalizedPathKey(expectedHistoryRoot) !== normalizedPathKey(planes.historyRoot)) {
    pendingHistoryUnavailable(
      safeId,
      'historyRoot 与 canonical repo/historyRepoPath 身份不一致，不能证明历史编号不存在',
      {
        repoRoot,
        historyRepoPath,
        expectedHistoryRoot,
        actualHistoryRoot: planes.historyRoot,
      },
    );
  }
  const guard = {
    authority: PENDING_START_FINALIZATION_AUTHORITY,
    version: 1,
    tracked: true,
    id: safeId,
    ref,
    oid: resolvePendingHistoryOid(ref, repoRoot, safeId),
    repoRoot,
    historyRepoPath,
  };
  assertPendingHistoryAbsence(planes, guard);
  return guard;
}

/**
 * plan → commit 的固定事实复核。写前调用阻止旧计划落盘；写后调用时若外部在
 * 窗口内新增 history 或移动 integration ref，会抛错并进入同一事务回滚。
 */
function assertPendingHistoryAbsence(planes, guard) {
  const id = guard && guard.id;
  if (!guard || guard.authority !== PENDING_START_FINALIZATION_AUTHORITY
    || guard.version !== 1 || assertSafeIdSegment(id) !== id) {
    pendingFinalizationInvalid(
      'pending finalization write plan 缺少不可伪造的 history-absence authority',
      { id: id || null },
    );
  }

  try {
    if (guard.tracked === false) {
      if (planes.tracked === true
        || normalizedPathKey(planes.historyRoot) !== normalizedPathKey(guard.historyRoot)) {
        pendingHistoryUnavailable(
          id,
          'history plane 身份在 plan 后变化，不能继续消费 PENDING_START',
          {
            expectedTracked: false,
            actualTracked: planes.tracked === true,
            expectedHistoryRoot: guard.historyRoot,
            actualHistoryRoot: planes.historyRoot,
          },
        );
      }
      const candidates = localHistoryCandidates(
        { ...planes, historyRoot: guard.historyRoot },
        id,
        null,
      );
      if (candidates.length > 0) {
        pendingHistoryConflict(id, candidates.map((entry) => entry.path));
      }
      return true;
    }

    const repoRoot = realpathOrNull(planes.repoRoot);
    if (guard.tracked !== true || planes.tracked !== true
      || planes.integrationRef !== guard.ref
      || repoRoot !== guard.repoRoot
      || normalizedHistoryRepoPath(planes) !== guard.historyRepoPath
      || normalizedPathKey(planes.historyRoot)
        !== normalizedPathKey(path.resolve(guard.repoRoot, ...guard.historyRepoPath.split('/')))) {
      pendingHistoryUnavailable(
        id,
        'canonical history plane 的 ref/repo/path 身份在 plan 后变化',
        {
          expectedRef: guard.ref,
          actualRef: planes.integrationRef || null,
          expectedRepoRoot: guard.repoRoot,
          actualRepoRoot: repoRoot,
          expectedHistoryRepoPath: guard.historyRepoPath,
        },
      );
    }
    const beforeOid = resolvePendingHistoryOid(guard.ref, guard.repoRoot, id);
    if (beforeOid !== guard.oid) {
      pendingHistoryUnavailable(
        id,
        `canonical integration ref ${guard.ref} 已从 plan 绑定 OID 前移`,
        { ref: guard.ref, expectedOid: guard.oid, actualOid: beforeOid },
      );
    }
    const candidates = historyIdentityPathsAtOid(
      { ...planes, historyRepoPath: guard.historyRepoPath },
      { ref: guard.ref, oid: guard.oid, cwd: guard.repoRoot },
      id,
    );
    const afterOid = resolvePendingHistoryOid(guard.ref, guard.repoRoot, id);
    if (afterOid !== guard.oid) {
      pendingHistoryUnavailable(
        id,
        `canonical integration ref ${guard.ref} 在 history 枚举期间前移`,
        { ref: guard.ref, expectedOid: guard.oid, actualOid: afterOid },
      );
    }
    if (candidates.length > 0) pendingHistoryConflict(id, candidates);
    return true;
  } catch (error) {
    if (error && (
      error.code === PENDING_START_HISTORY_ID_CONFLICT
      || error.code === PENDING_START_HISTORY_REALITY_UNAVAILABLE
      || error.code === PENDING_START_FINALIZATION_INVALID
    )) {
      throw error;
    }
    pendingHistoryUnavailable(
      id || null,
      `读取 canonical history 时失败，不能把 ${id || '未知编号'} 当成不存在`,
      null,
      error,
    );
  }
}

function pendingRealityPlaneIdentity(planes) {
  return {
    liveRoot: normalizedPathKey(planes.liveRoot),
    historyRoot: normalizedPathKey(planes.historyRoot),
    tracked: planes.tracked === true,
  };
}

/**
 * 申请一枚只供当前进程内 finalize 使用的资源现实 guard。它不提供任何文件/Git
 * 写能力；probe 只会在 store 已持有 graph lock 时由 commitNodeWrite 读取两次。
 */
function createPendingStartFinalizationRealityGuard(planes, id, options) {
  const settings = options || {};
  const safeId = assertSafeIdSegment(id);
  const expectedGeneration = requirePendingGeneration(settings);
  const expectedDigest = requirePendingDigest(settings);
  if (typeof settings.probe !== 'function') {
    pendingFinalizationInvalid(
      'pending finalization resource reality guard 必须提供只读 probe 函数',
      { id: safeId },
    );
  }
  const guard = Object.freeze(Object.create(null));
  PENDING_START_REALITY_GUARDS.set(guard, {
    id: safeId,
    expectedGeneration,
    expectedDigest,
    planes: pendingRealityPlaneIdentity(planes),
    probe: settings.probe,
  });
  return guard;
}

function bindPendingStartFinalizationRealityGuard(planes, plan, entry, guard) {
  if (guard === null || guard === undefined) {
    pendingFinalizationInvalid(
      'finalizePendingStart 必须携带该 marker/generation 的受控 resource reality guard',
      { id: entry && entry.record ? entry.record.id : null },
    );
  }
  const binding = PENDING_START_REALITY_GUARDS.get(guard);
  if (!binding
    || binding.id !== entry.record.id
    || binding.expectedGeneration !== entry.generation
    || binding.expectedDigest !== entry.digest
    || JSON.stringify(binding.planes) !== JSON.stringify(pendingRealityPlaneIdentity(planes))) {
    pendingFinalizationInvalid(
      'pending finalization resource reality guard 不是该 marker/generation 的受控绑定',
      { id: entry && entry.record ? entry.record.id : null },
    );
  }
  PENDING_START_REALITY_PLANS.set(plan, binding);
}

function assertPendingStartFinalizationReality(planes, writePlan, phase) {
  const binding = PENDING_START_REALITY_PLANS.get(writePlan);
  if (!binding) return;
  if (JSON.stringify(binding.planes) !== JSON.stringify(pendingRealityPlaneIdentity(planes))) {
    pendingFinalizationInvalid(
      'pending finalization resource reality guard 的控制面身份在提交期间变化',
      { id: binding.id, phase },
    );
  }
  let result;
  try {
    result = binding.probe(phase);
  } catch (error) {
    throw new StoreError(
      PENDING_START_RESOURCE_REALITY_CONFLICT,
      `PENDING_START ${binding.id} 在 ${phase} 资源现实复核时失败；不能报告 ACTIVE`,
      {
        id: binding.id,
        phase,
        causeCode: error && error.code ? error.code : null,
        cause: error && error.message ? error.message : String(error),
      },
    );
  }
  if (!result || result.ok !== true) {
    throw new StoreError(
      PENDING_START_RESOURCE_REALITY_CONFLICT,
      `PENDING_START ${binding.id} 在 ${phase} 资源现实复核不成立；不能报告 ACTIVE`,
      {
        id: binding.id,
        phase,
        code: result && result.code ? result.code : null,
        reason: result && typeof result.reason === 'string' ? result.reason : null,
      },
    );
  }
}

/**
 * 这一步必须在 graph lock 内做。pre 阶段目标编号在 live graph 中必须完全不存在；
 * post 阶段则必须只剩本事务刚写出的、摘要匹配的唯一 ACTIVE after-image。这样即使
 * 有不经 store 的旧写者在锁窗口里塞入 DRAFT/PARKED 副本，也会触发同一事务回滚。
 */
function assertPendingStartFinalizationLiveIdentity(planes, writePlan, phase) {
  const guard = writePlan && writePlan.pendingStartFinalization;
  const entries = writePlan && Array.isArray(writePlan.entries) ? writePlan.entries : [];
  const entry = entries.length === 1 ? entries[0] : null;
  if (!guard || !entry || entry.id !== guard.id || entry.lifecycle !== 'ACTIVE') {
    pendingFinalizationInvalid(
      'pending finalization 缺少唯一 ACTIVE live identity after-image',
      { id: guard && guard.id ? guard.id : null, phase },
    );
  }
  const candidates = liveIdentityCandidates(planes, guard.id);
  if (phase === 'PRE') {
    if (candidates.length === 0) return;
    throw new StoreError(
      PENDING_START_LIVE_ID_CONFLICT,
      `PENDING_START ${guard.id} 在收口前已存在等价 live 编号，不能覆盖为 ACTIVE`,
      { id: guard.id, phase, candidates },
    );
  }
  const expectedTarget = normalizedPathKey(entry.target);
  const isExactAfterImage = candidates.length === 1
    && normalizedPathKey(candidates[0].path) === expectedTarget
    && candidates[0].digest === entry.digest
    && candidates[0].nodeMatches === true;
  if (!isExactAfterImage) {
    throw new StoreError(
      PENDING_START_LIVE_ID_CONFLICT,
      `PENDING_START ${guard.id} 在收口后未保持唯一、冻结的 ACTIVE 编号；已回滚控制面写入`,
      {
        id: guard.id,
        phase,
        expectedTarget: entry.target,
        expectedDigest: entry.digest,
        candidates,
      },
    );
  }
}

function assertPendingFinalizationPlan(planes, writePlan) {
  const removals = (Array.isArray(writePlan.extraFiles) ? writePlan.extraFiles : [])
    .filter((extra) => extra && extra.remove === true);
  const guard = writePlan.pendingStartFinalization || null;
  const registered = PENDING_START_FINALIZATION_PLANS.has(writePlan);
  if (removals.length === 0 && !guard && !registered) return;
  if (!registered || removals.length !== 1 || !guard
    || removals[0].role !== 'pending-start'
    || normalizedPathKey(removals[0].target)
      !== normalizedPathKey(pendingStartPath(planes, guard.id))) {
    pendingFinalizationInvalid(
      'PENDING_START marker removal 必须来自唯一 finalize plan，普通 node write 无权消费 marker',
      {
        removalCount: removals.length,
        id: guard && guard.id ? guard.id : null,
      },
    );
  }
  // finalize plan 是一次性 capability：即使本次后续因竞态失败，也必须重新读取
  // marker/history 并重新规划，不能拿旧 plan 重试。
  PENDING_START_FINALIZATION_PLANS.delete(writePlan);
  assertPendingHistoryAbsence(planes, guard);
}

/**
 * finalize 是 PENDING_START 身份的最终安全边界，不能只依赖 task-start 的调用顺序。
 * marker 的每一个冻结字段都必须与唯一 after-image 对上；否则持有某份 digest 的调用方
 * 可能消费 A 的 marker，却写入 B 的 ACTIVE/CURRENT。
 */
function assertPendingFinalizationIdentity(planes, entry, settings) {
  const record = entry.record;
  if (record.status !== 'PENDING_START') {
    pendingFinalizationInvalid(
      `PENDING_START ${record.id} 的 status ${record.status} 不可收口；必须先显式恢复到 PENDING_START`,
      { id: record.id, status: record.status },
    );
  }
  const incompleteSteps = pendingStart.PENDING_START_STEP_NAMES.filter((name) => (
    record.steps[name] !== 'DONE' || record.resources[name] === null
  ));
  if (incompleteSteps.length > 0 || record.worktree.canonicalPath === null) {
    pendingFinalizationInvalid(
      `PENDING_START ${record.id} 四个步骤和资源尚未全部完成，不能消费 marker`,
      {
        id: record.id,
        incompleteSteps,
        canonicalWorktree: record.worktree.canonicalPath,
      },
    );
  }
  if (settings.openingCommitOid !== record.resources.openingCommit.oid) {
    pendingFinalizationInvalid(
      'finalize 提供的 opening commit 与 PENDING_START 记录不一致',
      {
        id: record.id,
        expected: record.resources.openingCommit.oid,
        actual: settings.openingCommitOid || null,
      },
    );
  }
  if (!Array.isArray(settings.nodeChanges) || settings.nodeChanges.length !== 1) {
    pendingFinalizationInvalid(
      'finalizePendingStart 必须恰好写入一个 ACTIVE TASK after-image',
      { nodeChanges: Array.isArray(settings.nodeChanges) ? settings.nodeChanges.length : null },
    );
  }
  const change = settings.nodeChanges[0] || {};
  const node = change.node;
  const binding = node && node.git && typeof node.git === 'object' ? node.git : null;
  const identityMatches = node
    && node.kind === 'TASK'
    && node.lifecycle === 'ACTIVE'
    && node.id === record.id
    && node['parent-id'] === record.parent.id
    && (!change.previousPath)
    && binding
    && binding['base-oid'] === record.base.oid
    && binding['task-branch'] === record.branch.name
    && binding.worktree === record.worktree.canonicalPath
    && sameOrderedStrings(node['write-scope'], record.declaration.writeScope)
    && sameOrderedStrings(node['forbidden-scope'], record.declaration.forbiddenScope)
    && sameOrderedStrings(node['exclusive-resources'], record.declaration.exclusiveResources);
  if (!identityMatches) {
    pendingFinalizationInvalid(
      'ACTIVE TASK after-image 与 PENDING_START 冻结的 id/parent/base/branch/worktree/scope 不一致',
      { id: record.id },
    );
  }
  const packageText = nodeSchema.serializeNode(node, change.body);
  const packageDigest = digestOf(packageText);
  if (packageDigest !== record.resources.taskPackage.digest) {
    pendingFinalizationInvalid(
      'ACTIVE TASK after-image 与 opening package 的冻结摘要不一致',
      {
        id: record.id,
        expected: record.resources.taskPackage.digest,
        actual: packageDigest,
        packagePath: record.resources.taskPackage.path,
      },
    );
  }
  const parentGuards = (Array.isArray(settings.guardRecords) ? settings.guardRecords : [])
    .filter((guard) => guard && guard.id === record.parent.id);
  if (parentGuards.length !== 1 || parentGuards[0].expectedDigest !== record.parent.digest) {
    pendingFinalizationInvalid(
      'finalize 必须以 PENDING_START 冻结的 parent id/digest 守卫直接父节点',
      {
        id: record.id,
        parentId: record.parent.id,
        expectedDigest: record.parent.digest,
      },
    );
  }
  if (!Array.isArray(settings.extraFiles) || settings.extraFiles.length !== 1) {
    pendingFinalizationInvalid(
      'finalizePendingStart 必须恰好写入一份该 worktree 的 CURRENT',
      { extraFiles: Array.isArray(settings.extraFiles) ? settings.extraFiles.length : null },
    );
  }
  const current = settings.extraFiles[0] || {};
  const expectedCurrentTarget = currentViewPath(
    planes,
    worktreeKeyFor(record.worktree.canonicalPath),
  );
  if (current.target !== expectedCurrentTarget || current.remove === true || typeof current.text !== 'string') {
    pendingFinalizationInvalid(
      'CURRENT after-image 必须唯一落在冻结 canonical worktree 对应的 currentViewPath',
      {
        id: record.id,
        expectedTarget: expectedCurrentTarget,
        actualTarget: current.target || null,
      },
    );
  }
}

/**
 * 成功开工的唯一收口：ACTIVE node、worktree CURRENT 与 marker removal 共用一个
 * graph lock、同一 generation 和同一份 rollback snapshot。任何一项失败，marker
 * 都会和 node/CURRENT/generation 一起恢复，调用方不能谎称 ACTIVE。
 */
function finalizePendingStart(planes, id, options) {
  const settings = options || {};
  const expectedGeneration = requirePendingGeneration(settings);
  const expectedDigest = requirePendingDigest(settings);
  if (settings.resourceRealityGuard === null || settings.resourceRealityGuard === undefined) {
    pendingFinalizationInvalid(
      'finalizePendingStart 缺少受控 resource reality guard；最终 Git 现实不得留在锁外或被省略',
      { id: id || null },
    );
  }
  const current = readPendingStart(planes, id);
  if (!current) {
    throw new StoreError(
      PENDING_START_NOT_FOUND,
      `PENDING_START ${id} 不存在，不能在没有冻结身份时写 ACTIVE`,
      { id },
    );
  }
  assertPendingGeneration(current, expectedGeneration);
  assertPendingDigest(current, expectedDigest);
  assertPendingFinalizationIdentity(planes, current, settings);
  // 这份事实必须在任何 plan 读取之前冻结；commitNodeWrite 会在锁内写前/写后复核。
  const historyAbsence = planPendingHistoryAbsence(planes, current.id);
  const plan = planNodeWrite(planes, settings.nodeChanges, {
    expectedGeneration,
    guardRecords: settings.guardRecords,
    // marker 必须最后落在 extraFiles 序列，确保 node 和 CURRENT 先完成；但所有
    // 目标仍属于同一 rollback snapshot，因此这不是跨事务的“先后承诺”。
    extraFiles: settings.extraFiles.concat([{
      target: current.path,
      remove: true,
      role: 'pending-start',
      expectedTargetDigest: current.contentDigest,
    }]),
  });
  plan.pendingStartFinalization = historyAbsence;
  PENDING_START_FINALIZATION_PLANS.add(plan);
  bindPendingStartFinalizationRealityGuard(
    planes,
    plan,
    current,
    settings.resourceRealityGuard,
  );
  const applied = commitNodeWrite(planes, plan);
  return {
    ok: true,
    generation: applied.generation,
    written: applied.written.slice(),
    consumed: { id: current.id, digest: current.digest, path: current.path },
  };
}

module.exports = {
  StoreError,
  CONFLICT_STALE_GENERATION,
  CONFLICT_NODE_CHANGED,
  CONFLICT_TARGET_CHANGED,
  CONFLICT_LOCK_HELD,
  STORE_READ_FAILED,
  TARGET_ALREADY_EXISTS,
  WRITE_PLAN_PATH_COLLISION,
  TRANSACTION_ROLLBACK_INCOMPLETE,
  LOCK_RELEASE_FAILED,
  COMMIT_APPLIED_LOCK_RELEASE_FAILED,
  PENDING_START_MALFORMED,
  PENDING_START_ID_CONFLICT,
  PENDING_START_NOT_FOUND,
  CONFLICT_PENDING_START_DIGEST,
  PENDING_START_CAS_REQUIRED,
  PENDING_START_FINALIZATION_INVALID,
  PENDING_START_STANDALONE_CONSUME_FORBIDDEN,
  PENDING_START_HISTORY_ID_CONFLICT,
  PENDING_START_HISTORY_REALITY_UNAVAILABLE,
  PENDING_START_LIVE_ID_CONFLICT,
  PENDING_START_RESOURCE_REALITY_CONFLICT,
  OPENING_PACKAGE_PATH_INVALID,
  HISTORY_BODY_FROZEN,
  HISTORY_REF_UNAVAILABLE,
  HISTORY_BUCKET_INVALID,
  ID_NOT_SAFE_PATH_SEGMENT,
  HISTORY_FINALIZATION_INVALID,
  HISTORY_FINALIZATION_REF_MISMATCH,
  HISTORY_FINALIZATION_CANDIDATE_MISMATCH,
  HISTORY_FINALIZATION_SOURCE_INVALID,
  HISTORY_FINALIZATION_CHECKOUT_MISMATCH,
  TRACKED_HISTORY_DIRECT_WRITE_FORBIDDEN,
  LIFECYCLE_LOCATION,
  LIVE_AREAS,
  LIVE_NODE_AREAS,
  resolvePlanes,
  areaFor,
  areaDir,
  historyBucket,
  assertSafeIdSegment,
  assertSafeHistoryBucket,
  nodeFilePath,
  pendingStartPath,
  currentViewPath,
  worktreeKeyFor,
  digestOf,
  listArea,
  readLiveNodes,
  readLiveSnapshot,
  readPendingStartSnapshot,
  listPendingStarts,
  readPendingStart,
  readHistoryNode,
  readGeneration,
  inspectLock,
  acquireLock,
  releaseLock,
  atomicWrite,
  planNodeWrite,
  commitNodeWrite,
  createPendingStart,
  updatePendingStart,
  consumePendingStart,
  createPendingStartFinalizationRealityGuard,
  finalizePendingStart,
  historyCandidateExtra,
  planHistoryFinalization,
  commitHistoryFinalization,
};
