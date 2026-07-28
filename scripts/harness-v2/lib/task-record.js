'use strict';

// Adaptive Harness v0.5 — ACTIVE TASK 的受控事实记录（AH-050-14）
//
// 需求溯源：GIT-4 / GIT-5 / GIT-6 / GIT-10、KP-13、KP-14、TS-6、
// R2 §3.2 / §5.3 / §10.2 / §12 #20 #21 #24。
//
// 本模块只组装 canonical node 的 after-image，不读 Git、不写文件，也不创造
// 第二份“记录 JSON”。调用方必须把刚读取的 Git 事实传入；CLI 再把 after-image
// 交给 store 的 generation/source-digest 事务。这样 public record 既能保留
// product/WIP carrier 和 verification run，又不会成为手工改 live plane 的后门。

const crypto = require('node:crypto');

const nodeSchema = require('./node-schema');
const testAudit = require('./test-audit');
const evidenceTrace = require('./evidence-trace');
const prepare = require('./prepare');
const routing = require('./routing');

const FULL_OID = /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i;
const MAX_RAW_OUTPUT_BYTES = 64 * 1024;
const ACTIVE_RECORD_DISPOSITIONS = Object.freeze(
  nodeSchema.GIT_DISPOSITIONS.filter((value) => value !== 'REMOVED'),
);

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function object(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : null;
}

function clone(value) {
  if (Array.isArray(value)) return value.map((entry) => clone(entry));
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).map((key) => [key, clone(value[key])]));
  }
  return value;
}

function sameOid(left, right) {
  return typeof left === 'string'
    && typeof right === 'string'
    && left.toLowerCase() === right.toLowerCase();
}

function own(source, key) {
  return Object.prototype.hasOwnProperty.call(source || {}, key);
}

function failure(code, error, detail) {
  return { ok: false, code, error, detail: detail || null, changes: [] };
}

function stableValue(value) {
  if (Array.isArray(value)) return value.map((entry) => stableValue(entry));
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, stableValue(value[key])]));
  }
  return value;
}

function digestOf(value) {
  return crypto.createHash('sha256').update(JSON.stringify(stableValue(value)), 'utf8').digest('hex');
}

function field(source, names) {
  const input = object(source) || {};
  for (const name of names) {
    if (own(input, name)) return { present: true, value: input[name] };
  }
  return { present: false, value: undefined };
}

function safeRepoPath(value) {
  const candidate = text(value).replace(/\\/g, '/').replace(/^\.\//, '');
  if (candidate === '' || candidate.startsWith('/') || candidate.includes('\0')) return null;
  const pieces = candidate.split('/');
  if (pieces.some((piece) => piece === '' || piece === '.' || piece === '..')) return null;
  return candidate;
}

// node-schema 的 canonical serializer 无法无损往返 verification[].run 里的嵌套
// 数组（合法 Git 路径可含逗号）。持久节点因此使用显式 JSON: 前缀标量；内存调用
// 仍可传数组。两者都逐项规范化，其余字符串与对象 fail closed。
// 与 lib/test-audit.js 的同名解析保持同一约定；本模块只用安全 repo 路径。
function runTestPathList(value) {
  if (Array.isArray(value)) {
    const paths = value.map(safeRepoPath).filter(Boolean);
    return { ok: paths.length === value.length, paths, source: 'ARRAY' };
  }
  if (typeof value === 'string' && value.startsWith('JSON:')) {
    let raw;
    try {
      raw = JSON.parse(value.slice('JSON:'.length));
    } catch {
      return { ok: false, paths: [], source: 'CANONICAL_JSON' };
    }
    if (!Array.isArray(raw)) return { ok: false, paths: [], source: 'CANONICAL_JSON' };
    const paths = raw.map(safeRepoPath).filter(Boolean);
    return { ok: paths.length === raw.length, paths, source: 'CANONICAL_JSON' };
  }
  return { ok: false, paths: [], source: 'INVALID' };
}

function canonicalTestPaths(value) {
  if (value === 'ALL_TRACKED') {
    return { ok: true, stored: 'ALL_TRACKED', paths: null };
  }
  const decoded = runTestPathList(value);
  if (!decoded.ok || decoded.paths.length === 0) {
    return failure(
      'TASK_RECORD_TEST_PATHS_INVALID',
      'verification run.test-paths 必须是非空相对路径数组，或显式 ALL_TRACKED',
    );
  }
  const seen = new Set();
  const paths = [];
  for (const valuePath of decoded.paths) {
    const path = safeRepoPath(valuePath);
    if (!path || seen.has(path)) {
      return failure(
        'TASK_RECORD_TEST_PATHS_INVALID',
        'verification run.test-paths 必须是唯一、仓库内且不含 .. 的路径列表',
        { value: valuePath },
      );
    }
    seen.add(path);
    paths.push(path);
  }
  return { ok: true, stored: `JSON:${JSON.stringify(paths)}`, paths };
}

// 只为升级前“没有任何 required verification”的既有六区块节点保留的窄修复
// 输入。它刻意没有 status/run：先冻结 UNKNOWN 合同，再在下一次 record 登记
// 实际运行，不能把 closeout 或同一请求的自报倒灌成已经冻结的验证事实。
function normalizeLegacyVerificationContract(raw, index) {
  const source = object(raw);
  if (!source) {
    return failure('TASK_RECORD_LEGACY_CONTRACT_INVALID', `legacy-verification-contract #${index + 1} 必须是 object`);
  }
  const allowed = new Set(['id', 'command', 'required']);
  for (const key of Object.keys(source)) {
    if (!allowed.has(key)) {
      return failure(
        'TASK_RECORD_LEGACY_CONTRACT_INVALID',
        'legacy-verification-contract 只接受 id、command、required；status/run 必须在后续独立 record 登记',
        { key },
      );
    }
  }
  const id = text(source.id);
  const command = text(source.command);
  if (id === '' || command === '' || source.required !== true) {
    return failure(
      'TASK_RECORD_LEGACY_CONTRACT_INVALID',
      'legacy-verification-contract 必须给出非空 id、command 和 required:true',
      { id: id || null, command: command || null, required: source.required },
    );
  }
  return { ok: true, entry: { id, command, required: true } };
}

function normalizeRun(raw, index) {
  const source = object(raw);
  if (!source) {
    return failure('TASK_RECORD_VERIFICATION_INVALID', `verification run #${index + 1} 必须是 object`);
  }
  const runFields = new Set([
    'head-oid', 'headOid', 'exit-code', 'exitCode', 'output-ref', 'outputRef',
    'command-digest', 'commandDigest', 'test-paths', 'testPaths',
    'output', 'evidence-text', 'evidenceText',
  ]);
  const outerFields = new Set(['id', 'command', 'required', 'status', 'run', ...runFields]);
  for (const key of Object.keys(source)) {
    if (!outerFields.has(key)) {
      return failure(
        'TASK_RECORD_VERIFICATION_FIELD_UNKNOWN',
        `verification run #${index + 1} 不接受字段 ${key}；验收/结论不能混入事实记录`,
        { key },
      );
    }
  }
  const nested = own(source, 'run') ? object(source.run) : null;
  if (own(source, 'run') && !nested) {
    return failure('TASK_RECORD_VERIFICATION_INVALID', `verification run #${index + 1}.run 必须是 object`);
  }
  if (nested) {
    for (const key of Object.keys(nested)) {
      if (!runFields.has(key)) {
        return failure(
          'TASK_RECORD_VERIFICATION_FIELD_UNKNOWN',
          `verification run #${index + 1}.run 不接受字段 ${key}`,
          { key },
        );
      }
    }
    const flatRunFacts = [...runFields].filter((key) => (
      !['output', 'evidence-text', 'evidenceText'].includes(key) && own(source, key)
    ));
    if (flatRunFacts.length > 0) {
      return failure(
        'TASK_RECORD_VERIFICATION_AMBIGUOUS',
        `verification run #${index + 1} 不能同时使用 run 与顶层执行字段`,
        { fields: flatRunFacts },
      );
    }
  }
  const run = nested || source;
  const id = text(source.id);
  const command = text(source.command);
  const required = source.required;
  const status = text(source.status);
  const head = field(run, ['head-oid', 'headOid']);
  const exitCode = field(run, ['exit-code', 'exitCode']);
  const outputRef = field(run, ['output-ref', 'outputRef']);
  const commandDigest = field(run, ['command-digest', 'commandDigest']);
  const testPaths = field(run, ['test-paths', 'testPaths']);
  const sourceOutput = field(source, ['output', 'evidence-text', 'evidenceText']);
  const nestedOutput = nested ? field(nested, ['output', 'evidence-text', 'evidenceText']) : { present: false, value: undefined };
  if (sourceOutput.present && nestedOutput.present) {
    return failure(
      'TASK_RECORD_VERIFICATION_AMBIGUOUS',
      `verification run #${index + 1} 不能同时在顶层与 run 提供 output/evidence-text`,
    );
  }
  const output = sourceOutput.present ? sourceOutput : nestedOutput;

  if (id === '' || command === '' || typeof required !== 'boolean'
    || !nodeSchema.VERIFICATION_STATUSES.includes(status)) {
    return failure(
      'TASK_RECORD_VERIFICATION_INVALID',
      'verification run 必须完整给出 id、command、required 与五档 status',
      { id: id || null, command: command || null, required, status: status || null },
    );
  }
  if (!head.present || !FULL_OID.test(text(head.value))) {
    return failure('TASK_RECORD_HEAD_OID_INVALID', `${id} 的 run.head-oid 必须是完整 40/64 位 commit OID`);
  }
  if (!exitCode.present || !Number.isInteger(exitCode.value)) {
    return failure('TASK_RECORD_EXIT_CODE_INVALID', `${id} 的 run.exit-code 必须是整数`);
  }
  if (status === 'PASS' && exitCode.value !== 0) {
    return failure('TASK_RECORD_STATUS_CONFLICT', `${id} 标记 PASS 时 run.exit-code 必须为 0`);
  }
  if (!outputRef.present || text(outputRef.value) === '' || /[\r\n]/.test(String(outputRef.value))) {
    return failure('TASK_RECORD_OUTPUT_REF_INVALID', `${id} 的 run.output-ref 必须是单行可解析引用`);
  }
  const parsedRef = evidenceTrace.parseOutputRef(text(outputRef.value));
  if (!parsedRef.ok || parsedRef.kind === 'legacy') {
    return failure(
      'TASK_RECORD_OUTPUT_REF_INVALID',
      `${id} 的 run.output-ref 不是可实解的 node:/repo: 原始输出引用`,
      { outputRef: text(outputRef.value), reason: parsedRef.error || null },
    );
  }
  const expectedDigest = testAudit.fingerprintCommand(command);
  if (!commandDigest.present || text(commandDigest.value) !== expectedDigest) {
    return failure(
      'TASK_RECORD_COMMAND_DIGEST_MISMATCH',
      `${id} 的 run.command-digest 必须精确绑定 command 原文`,
      { expected: expectedDigest, actual: text(commandDigest.value) || null },
    );
  }
  if (!testPaths.present) {
    return failure('TASK_RECORD_TEST_PATHS_INVALID', `${id} 的 run.test-paths 不得缺失`);
  }
  const paths = canonicalTestPaths(testPaths.value);
  if (!paths.ok) return paths;
  let sanitizedOutput;
  if (output.present) {
    if (typeof output.value !== 'string' || output.value.trim() === '') {
      return failure('TASK_RECORD_OUTPUT_INVALID', `${id} 的 output/evidence-text 必须是非空文本`);
    }
    const rawBytes = Buffer.byteLength(output.value, 'utf8');
    if (rawBytes > MAX_RAW_OUTPUT_BYTES) {
      return failure(
        'TASK_RECORD_OUTPUT_TOO_LARGE',
        `${id} 的原始 output 超过 ${MAX_RAW_OUTPUT_BYTES} bytes 上限`,
        { bytes: rawBytes, maximum: MAX_RAW_OUTPUT_BYTES },
      );
    }
    sanitizedOutput = String(routing.redactKnownSecrets(output.value)).replace(/\r\n/g, '\n');
    const sanitizedBytes = Buffer.byteLength(sanitizedOutput, 'utf8');
    if (sanitizedBytes === 0 || sanitizedBytes > MAX_RAW_OUTPUT_BYTES) {
      return failure(
        'TASK_RECORD_OUTPUT_INVALID',
        `${id} 的脱敏后原始 output 为空或超过保存上限`,
        { bytes: sanitizedBytes, maximum: MAX_RAW_OUTPUT_BYTES },
      );
    }
  }
  return {
    ok: true,
    entry: {
      id,
      command,
      required,
      status,
      run: {
        'head-oid': text(head.value).toLowerCase(),
        'exit-code': exitCode.value,
        'output-ref': text(outputRef.value),
        'command-digest': expectedDigest,
        'test-paths': paths.stored,
      },
      testPaths: paths.paths,
      ...(output.present ? { output: sanitizedOutput } : {}),
    },
  };
}

/**
 * 接受 file/CLI 都可表达的极小 payload，并将可能的 camelCase 入口收束到
 * canonical kebab-case。这里不接受任意附带“结论”字段：事实记录不是验收入口。
 */
function normalizeRequest(input) {
  const source = object(input);
  if (!source) return failure('TASK_RECORD_INPUT_INVALID', 'record payload 顶层必须是 object');
  const allowed = new Set([
    'product-commit', 'productCommit', 'wip-commit', 'wipCommit',
    'disposition', 'verification-runs', 'verificationRuns', 'verification', 'runs',
    'legacy-verification-contract', 'legacyVerificationContract',
  ]);
  for (const key of Object.keys(source)) {
    if (!allowed.has(key)) {
      return failure('TASK_RECORD_FIELD_UNKNOWN', `record payload 不接受字段 ${key}`);
    }
  }
  const product = field(source, ['product-commit', 'productCommit']);
  const wip = field(source, ['wip-commit', 'wipCommit']);
  const disposition = field(source, ['disposition']);
  const listed = field(source, ['verification-runs', 'verificationRuns', 'verification', 'runs']);
  const legacyContract = field(source, ['legacy-verification-contract', 'legacyVerificationContract']);

  const normalizeCarrier = (label, candidate) => {
    if (!candidate.present) return { ok: true, value: undefined };
    const oid = text(candidate.value);
    if (!FULL_OID.test(oid)) {
      return failure('TASK_RECORD_CARRIER_OID_INVALID', `${label} 必须是完整 40/64 位 commit OID`, { value: candidate.value });
    }
    return { ok: true, value: oid.toLowerCase() };
  };
  const productCarrier = normalizeCarrier('product-commit', product);
  if (!productCarrier.ok) return productCarrier;
  const wipCarrier = normalizeCarrier('wip-commit', wip);
  if (!wipCarrier.ok) return wipCarrier;

  let normalizedDisposition;
  if (disposition.present) {
    normalizedDisposition = text(disposition.value);
    if (!ACTIVE_RECORD_DISPOSITIONS.includes(normalizedDisposition)) {
      return failure(
        'TASK_RECORD_DISPOSITION_INVALID',
        `ACTIVE record disposition 只接受 ${ACTIVE_RECORD_DISPOSITIONS.join(' / ')}；REMOVED 必须走已确认的物理退场流程`,
      );
    }
  }

  let verificationRuns = [];
  if (listed.present) {
    if (!Array.isArray(listed.value)) {
      return failure('TASK_RECORD_VERIFICATION_INVALID', 'verification-runs 必须是数组');
    }
    const seen = new Set();
    for (let index = 0; index < listed.value.length; index += 1) {
      const normalized = normalizeRun(listed.value[index], index);
      if (!normalized.ok) return normalized;
      if (seen.has(normalized.entry.id)) {
        return failure('TASK_RECORD_VERIFICATION_DUPLICATE', `record payload 中 verification id 重复：${normalized.entry.id}`);
      }
      seen.add(normalized.entry.id);
      verificationRuns.push(normalized.entry);
    }
  }
  let legacyVerificationContract = [];
  if (legacyContract.present) {
    if (!Array.isArray(legacyContract.value) || legacyContract.value.length === 0) {
      return failure(
        'TASK_RECORD_LEGACY_CONTRACT_INVALID',
        'legacy-verification-contract 必须是至少一条 required 合同的数组',
      );
    }
    const seen = new Set();
    for (let index = 0; index < legacyContract.value.length; index += 1) {
      const normalized = normalizeLegacyVerificationContract(legacyContract.value[index], index);
      if (!normalized.ok) return normalized;
      if (seen.has(normalized.entry.id)) {
        return failure('TASK_RECORD_LEGACY_CONTRACT_DUPLICATE', `legacy-verification-contract id 重复：${normalized.entry.id}`);
      }
      seen.add(normalized.entry.id);
      legacyVerificationContract.push(normalized.entry);
    }
  }
  if (!product.present && !wip.present && !disposition.present
    && verificationRuns.length === 0 && legacyVerificationContract.length === 0) {
    return failure('TASK_RECORD_EMPTY', 'record 至少要登记 product/WIP carrier、disposition、verification run 或受控 legacy verification 合同之一');
  }
  const request = {
    ...(product.present ? { 'product-commit': productCarrier.value } : {}),
    ...(wip.present ? { 'wip-commit': wipCarrier.value } : {}),
    ...(disposition.present ? { disposition: normalizedDisposition } : {}),
    'verification-runs': verificationRuns.map(({ testPaths, ...entry }) => entry),
    ...(legacyContract.present ? { 'legacy-verification-contract': legacyVerificationContract } : {}),
  };
  return { ok: true, request, digest: digestOf(request) };
}

function bindingFacts(raw) {
  const source = object(raw) || {};
  return {
    baseOid: text(source.baseOid || source['base-oid']),
    taskHeadOid: text(source.taskHeadOid || source['task-head-oid']),
    taskBranchOid: text(source.taskBranchOid || source['task-branch-oid']),
    worktreeClean: source.worktreeClean === true,
    headMatchesTaskBranch: source.headMatchesTaskBranch === true,
    baseIsAncestor: source.baseIsAncestor === true,
    declaredWorktree: text(source.declaredWorktree || source.worktree) || null,
  };
}

function carrierFacts(raw) {
  const source = object(raw) || {};
  return {
    oid: text(source.oid),
    canonical: source.canonical === true,
    afterBase: source.afterBase === true,
    reachableOnTaskBranch: source.reachableOnTaskBranch === true,
    withinWriteScope: source.withinWriteScope === true,
    changedPaths: Array.isArray(source.changedPaths) ? source.changedPaths.slice() : [],
    extendsPrevious: source.extendsPrevious !== false,
    coversTaskHead: source.coversTaskHead === true,
  };
}

function validateCarrier(label, oid, existing, facts, requireTip) {
  const carrier = carrierFacts(facts);
  if (!sameOid(carrier.oid, oid) || !carrier.canonical || !carrier.afterBase
    || !carrier.reachableOnTaskBranch || !carrier.withinWriteScope || carrier.changedPaths.length === 0) {
    return failure(
      'TASK_RECORD_CARRIER_REALITY_INVALID',
      `${label} 必须是冻结 base 之后、在 task branch 可达、且全部落在 write-scope 内的非空完整 commit`,
      { label, oid, carrier },
    );
  }
  if (existing && !carrier.extendsPrevious) {
    return failure(
      'TASK_RECORD_CARRIER_REWRITE_FORBIDDEN',
      `${label} 不得把已记录 carrier 改写到不包含旧事实的提交`,
      { previous: existing, next: oid },
    );
  }
  if (requireTip && !carrier.coversTaskHead) {
    return failure(
      'TASK_RECORD_WIP_NOT_TASK_TIP',
      'wip-commit 必须精确承载当前 task branch HEAD，不能遗漏已完成的本任务改动',
      { oid, carrier },
    );
  }
  return { ok: true, carrier };
}

function validationEntryMap(entries) {
  const map = new Map();
  for (const entry of Array.isArray(entries) ? entries : []) {
    if (entry && typeof entry === 'object' && typeof entry.id === 'string') map.set(entry.id, entry);
  }
  return map;
}

function validateTrace(nextNode, body, entry, readRepoAtHead) {
  const candidate = (nextNode.verification || []).find((value) => value && value.id === entry.id);
  const resolved = evidenceTrace.resolveVerification(candidate, {
    node: nextNode,
    body,
    readRepoAtHead,
  });
  if (resolved.ok) return { ok: true, trace: resolved.raw };
  return failure(
    'TASK_RECORD_OUTPUT_REF_UNRESOLVED',
    `${entry.id} 的 output-ref 未能在本次 head 上实解为唯一原始输出`,
    { id: entry.id, trace: resolved },
  );
}

function rawOutputBlock(value) {
  // 将原始命令输出作为 Markdown 缩进代码块保存。这样输出里的 `# pass` 或恶意
  // `### raw-output-*` 都不会变成新的 heading，证据定位器也不会被输出内容劫持。
  return String(value || '')
    .replace(/\r\n/g, '\n')
    .replace(/\s+$/, '')
    .split('\n')
    .map((line) => `    ${line}`)
    .join('\n');
}

function appendNodeOutputs(body, runs) {
  let nextBody = String(body || '');
  const appended = [];
  const anchors = new Set();
  for (const entry of Array.isArray(runs) ? runs : []) {
    if (!own(entry, 'output')) continue;
    const parsed = evidenceTrace.parseOutputRef(entry.run && entry.run['output-ref']);
    const anchor = evidenceTrace.rawOutputAnchor(entry);
    if (!parsed.ok || parsed.kind !== 'node' || anchor === '') {
      return failure(
        'TASK_RECORD_OUTPUT_REF_OUTPUT_CONFLICT',
        `${entry.id} 携带 output 时 output-ref 必须是同一 node 的 raw-output anchor`,
        { id: entry.id, outputRef: entry.run && entry.run['output-ref'] ? entry.run['output-ref'] : null },
      );
    }
    if (anchors.has(anchor)) {
      return failure(
        'TASK_RECORD_OUTPUT_ANCHOR_DUPLICATE',
        `同一 record payload 不能向 #${anchor} 追加两份原始输出`,
        { id: entry.id, anchor },
      );
    }
    anchors.add(anchor);
    const existing = evidenceTrace.locateHeading(nextBody, anchor);
    if (existing.ok || existing.code === 'TRACE_ANCHOR_AMBIGUOUS') {
      return failure(
        'TASK_RECORD_OUTPUT_HEADING_EXISTS',
        `#${anchor} 已存在；原始输出不可覆盖、不可追加，必须保留既有证据或使用新的 verification id`,
        { id: entry.id, anchor },
      );
    }
    const output = rawOutputBlock(entry.output);
    if (output.trim() === '') {
      return failure('TASK_RECORD_OUTPUT_INVALID', `${entry.id} 的脱敏 output 为空，不能写入 raw-output section`);
    }
    nextBody = `${nextBody.replace(/\s+$/, '')}\n\n### ${anchor}\n\n${output}\n`;
    appended.push({ id: entry.id, anchor, bytes: Buffer.byteLength(entry.output, 'utf8') });
  }
  return { ok: true, body: nextBody, appended };
}

/**
 * 生成 ACTIVE 节点的事实记录 after-image。facts 必须来自同一次 Git 实读；本函数
 * 明确检查布尔事实而非“缺省当真”，让测试夹具和 CLI 都无法把未知伪装为通过。
 */
function prepareRecord(input) {
  const settings = object(input) || {};
  const record = object(settings.record);
  const request = settings.request;
  if (!record || !record.node || typeof record.body !== 'string') {
    return failure('TASK_RECORD_SOURCE_INVALID', 'record 需要当前 live TASK 的 node、body 与 source path');
  }
  const node = record.node;
  if (node.kind !== 'TASK' || node.lifecycle !== 'ACTIVE') {
    return failure('TASK_RECORD_ACTIVE_REQUIRED', `record 只接受 ACTIVE TASK，收到 ${node.id || 'unknown'} / ${node.lifecycle || 'unknown'}`);
  }
  const binding = object(node.git);
  if (!binding) return failure('TASK_RECORD_BINDING_REQUIRED', 'ACTIVE TASK 缺少 canonical git binding，不能登记 carrier');
  const body = prepare.validateContinuationTaskBody(record.body);
  if (!body.ok) {
    return failure(
      'TASK_RECORD_BODY_INVALID',
      'record 只接受当前五区块正文，或已在办旧六区块节点的受控兼容正文',
      body,
    );
  }
  const normalized = normalizeRequest(request);
  if (!normalized.ok) return normalized;

  const facts = object(settings.facts) || {};
  const reality = bindingFacts(facts.binding);
  if (!FULL_OID.test(reality.baseOid) || !FULL_OID.test(reality.taskHeadOid)
    || !FULL_OID.test(reality.taskBranchOid) || !reality.worktreeClean
    || !reality.headMatchesTaskBranch || !reality.baseIsAncestor
    || !sameOid(binding['base-oid'], reality.baseOid)) {
    return failure(
      'TASK_RECORD_GIT_REALITY_INVALID',
      'record 必须在干净且仍 checkout 声明 task branch 的 worktree 上，且冻结 base/head/branch 可完整核对',
      { binding: clone(binding), reality },
    );
  }

  const requested = normalized.request;
  if (requested['product-commit']) {
    const checked = validateCarrier(
      'product-commit', requested['product-commit'], binding['product-commit'],
      object(facts.carriers) && facts.carriers.product, false,
    );
    if (!checked.ok) return checked;
  }
  if (requested['wip-commit']) {
    const checked = validateCarrier(
      'wip-commit', requested['wip-commit'], binding['wip-commit'],
      object(facts.carriers) && facts.carriers.wip, true,
    );
    if (!checked.ok) return checked;
  }

  const known = validationEntryMap(node.verification);
  const runs = Array.isArray(requested['verification-runs']) ? requested['verification-runs'] : [];
  const legacyContracts = Array.isArray(requested['legacy-verification-contract'])
    ? requested['legacy-verification-contract']
    : [];
  if (legacyContracts.length > 0) {
    const hasRequired = (Array.isArray(node.verification) ? node.verification : [])
      .some((entry) => entry && entry.required === true);
    const alsoRecordsFacts = Boolean(
      requested['product-commit'] || requested['wip-commit'] || requested.disposition || runs.length > 0,
    );
    if (body.format !== 'LEGACY_SIX_BLOCKS' || hasRequired) {
      return failure(
        'TASK_RECORD_LEGACY_CONTRACT_NOT_ELIGIBLE',
        '仅既有旧六区块、且尚无任何 required verification 的 ACTIVE TASK 可以补一次验证合同',
        { format: body.format, hasRequired },
      );
    }
    if (alsoRecordsFacts) {
      return failure(
        'TASK_RECORD_LEGACY_CONTRACT_ISOLATED',
        '补 legacy verification 合同必须单独 inspect→write；不得同次登记 carrier、disposition 或运行结果',
      );
    }
    for (const contract of legacyContracts) {
      if (known.has(contract.id)) {
        return failure(
          'TASK_RECORD_LEGACY_CONTRACT_EXISTS',
          `legacy verification 合同 ${contract.id} 已存在；不得覆盖或改写既有合同`,
        );
      }
    }
  }
  // 过渡化石修复（AH-050-14 类）：补合同的同时把旧六区块正文无损迁移为五区块，
  // 否则该节点永远无法通过 split/replace 的来源正文校验，合同将再无可修正路径。
  let workingBody = record.body;
  let legacyBodyMigrated = false;
  if (legacyContracts.length > 0) {
    const migrated = prepare.migrateLegacyTaskBody(record.body);
    if (!migrated.ok) {
      return failure('TASK_RECORD_LEGACY_BODY_MIGRATION_INVALID', '旧六区块正文无法无损迁移为五区块', {
        issues: migrated.issues,
      });
    }
    workingBody = migrated.body;
    legacyBodyMigrated = true;
  }
  const replacement = new Map();
  for (const incoming of runs) {
    const original = known.get(incoming.id);
    if (!original) {
      return failure('TASK_RECORD_VERIFICATION_UNKNOWN', `verification ${incoming.id} 不在冻结合同中，事实记录不得新增测试合同`);
    }
    if (original.command !== incoming.command || original.required !== incoming.required) {
      return failure(
        'TASK_RECORD_VERIFICATION_CONTRACT_MISMATCH',
        `${incoming.id} 的 command/required 必须与冻结 verification 合同完全一致`,
        {
          expected: { command: original.command, required: original.required },
          actual: { command: incoming.command, required: incoming.required },
        },
      );
    }
    if (!sameOid(incoming.run['head-oid'], reality.taskHeadOid)) {
      return failure(
        'TASK_RECORD_RUN_HEAD_MISMATCH',
        `${incoming.id} 的 run.head-oid 必须等于当前 task branch HEAD`,
        { expected: reality.taskHeadOid, actual: incoming.run['head-oid'] },
      );
    }
    replacement.set(incoming.id, incoming);
  }

  const verification = (Array.isArray(node.verification) ? node.verification : []).map((entry) => {
    const incoming = replacement.get(entry && entry.id);
    return incoming
      ? { ...clone(entry), status: incoming.status, run: clone(incoming.run) }
      : clone(entry);
  });
  for (const contract of legacyContracts) {
    verification.push({ ...clone(contract), status: 'UNKNOWN' });
  }
  const nextGit = {
    ...clone(binding),
    ...(requested['product-commit'] ? { 'product-commit': requested['product-commit'] } : {}),
    ...(requested['wip-commit'] ? { 'wip-commit': requested['wip-commit'] } : {}),
    ...(requested.disposition ? { disposition: requested.disposition } : {}),
    ...((requested['product-commit'] || requested['wip-commit']) ? { 'no-product-change': false } : {}),
  };
  const nextNode = { ...clone(node), git: nextGit, verification };
  const outputBody = appendNodeOutputs(workingBody, runs);
  if (!outputBody.ok) return outputBody;
  const traces = [];
  for (const incoming of runs) {
    const checked = validateTrace(nextNode, outputBody.body, incoming, settings.readRepoAtHead);
    if (!checked.ok) return checked;
    traces.push({ id: incoming.id, output: checked.trace });
  }

  return {
    ok: true,
    action: 'record',
    id: node.id,
    request: clone(requested),
    requestDigest: normalized.digest,
    observed: {
      binding: reality,
      carriers: clone(facts.carriers || {}),
      verification: traces,
      legacyVerificationContract: legacyContracts.map((entry) => entry.id),
      legacyBodyMigrated,
      appendedRawOutput: outputBody.appended,
    },
    changes: [{ node: nextNode, body: outputBody.body, previousPath: record.path }],
  };
}

function receiptPayload(writePlan, requestDigest, observed) {
  return {
    generation: writePlan && writePlan.generation,
    entries: (writePlan && Array.isArray(writePlan.entries) ? writePlan.entries : []).map((entry) => ({
      id: entry.id,
      lifecycle: entry.lifecycle,
      target: entry.target,
      previousPath: entry.previousPath,
      expectedSourceDigest: entry.expectedSourceDigest,
      expectedTargetDigest: entry.expectedTargetDigest,
      digest: entry.digest,
    })),
    guards: (writePlan && Array.isArray(writePlan.guardRecords) ? writePlan.guardRecords : []).map((entry) => ({
      id: entry.id,
      path: entry.path,
      expectedDigest: entry.expectedDigest,
    })),
    requestDigest,
    observed: stableValue(observed || {}),
  };
}

function receiptFor(writePlan, requestDigest, observed) {
  return digestOf(receiptPayload(writePlan, requestDigest, observed));
}

module.exports = {
  ACTIVE_RECORD_DISPOSITIONS,
  canonicalTestPaths,
  normalizeRequest,
  prepareRecord,
  receiptFor,
  receiptPayload,
  digestOf,
};
