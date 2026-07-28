'use strict';

// Adaptive Harness v0.5 — 子执行者完成事实与 verification 一跳追溯（AH-050-10）
//
// 需求溯源：KP-8 / KP-9 · G-20 / G-21。
//
// 这里故意是纯装配和判定：不读工作树、不调用 Git、不写文件。调用方把刚从
// git-facts 取得的 base..head 事实注入进来；repo: 引用也只能由调用方给一个
// “按 run.head-oid 读取”的函数。这样 agent 自报的 changed paths 永远不能成为
// 判定事实，外部文本也不能变成会执行的命令。

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function object(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : null;
}

function array(value) {
  return Array.isArray(value) ? value : [];
}

function firstText(values) {
  for (const value of values) {
    const candidate = text(value);
    if (candidate !== '') return candidate;
  }
  return '';
}

function normalizePath(value) {
  return text(value).replace(/\\/g, '/').replace(/^\.\//, '').replace(/\/+$/, '');
}

function pathList(value) {
  const input = Array.isArray(value) ? value : (typeof value === 'string' ? [value] : []);
  return input.map(normalizePath).filter(Boolean);
}

function claimId(claim, index) {
  const source = object(claim) || {};
  return firstText([
    source.id, source.agentId, source['agent-id'], source.executorId, source['executor-id'], source.name,
  ]) || `agent-${index + 1}`;
}

function claimCompleted(claim) {
  const source = object(claim) || {};
  if (source.claimedComplete === true || source['claimed-complete'] === true
    || source.completed === true || source.complete === true) return true;
  const value = firstText([source.claim, source.status, source.result, source.outcome]).toUpperCase();
  return ['COMPLETE', 'COMPLETED', 'DONE'].includes(value);
}

function delegatedScope(claim) {
  const source = object(claim) || {};
  return pathList(
    source.delegatedScope
    || source['delegated-scope']
    || source.writeScope
    || source['write-scope']
    || source.scope,
  );
}

function claimVerificationEntries(claim) {
  const source = object(claim) || {};
  const value = source.verification
    || source.verifications
    || source['verification-runs']
    || source.verificationRuns
    || source.runs;
  if (Array.isArray(value)) return value.filter((entry) => object(entry));
  return object(value) ? [value] : [];
}

function outputRefFrom(value) {
  const source = object(value) || {};
  const run = object(source.run) || source;
  return firstText([
    run['output-ref'], run.outputRef, run.rawOutputRef, run['raw-output-ref'], run.evidenceRef,
  ]);
}

function hasVerificationOutput(claim) {
  const source = object(claim) || {};
  if (firstText([
    source['output-ref'], source.outputRef, source.rawOutputRef, source['raw-output-ref'],
    source.verificationOutput, source['verification-output'], source.evidenceRef,
  ]) !== '') return true;
  const entries = claimVerificationEntries(source);
  if (entries.length === 0) return false;
  const required = entries.filter((entry) => entry.required !== false);
  if (required.length === 0) return false;
  return required.every((entry) => outputRefFrom(entry) !== '');
}

function claimSummary(claim) {
  const source = object(claim) || {};
  return firstText([source.summary, source['agent-summary'], source.note, source.conclusion]);
}

function agentClaimsFromCloseout(closeout) {
  const source = object(closeout) || {};
  const candidates = [
    source.agentResults,
    source['agent-results'],
    source.delegatedAgents,
    source['delegated-agents'],
    source.delegatedResults,
    source['delegated-results'],
    source.agents,
    source.agentSummaries,
    source['agent-summaries'],
  ];
  for (const candidate of candidates) {
    if (Array.isArray(candidate)) return candidate.filter((entry) => object(entry));
  }
  return [];
}

function actualForClaim(claim, id, index, settings) {
  const input = object(settings) || {};
  const byId = object(input.actualById) || {};
  const fromMap = object(byId[id]);
  const byIndex = array(input.actual).map(object).filter(Boolean)[index] || null;
  // claim.actual 是子执行者自报，绝不能回退采用。只有调用主流程刚从 Git
  // 取得并注入的 actualById / actual[index] 才能成为 KP-8 的事实输入。
  return fromMap || byIndex || null;
}

function withinPrefix(candidate, prefix) {
  const path = normalizePath(candidate);
  const base = normalizePath(prefix);
  return path !== '' && base !== '' && (path === base || path.startsWith(`${base}/`));
}

/**
 * 审计声称“完成”的子执行者。claims 只提供声明和 delegated scope；actualById
 * 只能来自主流程刚读取的 Git facts，典型条目为
 * { 'base-oid', 'head-oid', changedPaths }。没有可核对事实也 fail closed。
 */
function auditAgentClaims(input) {
  const settings = object(input) || {};
  const claims = Array.isArray(settings.claims)
    ? settings.claims.filter((entry) => object(entry))
    : agentClaimsFromCloseout(settings.closeout);
  const parentWriteScope = pathList(settings.parentWriteScope || settings['parent-write-scope']);
  const parentForbiddenScope = pathList(settings.parentForbiddenScope || settings['parent-forbidden-scope']);
  const records = claims.map((claim, index) => {
    const id = claimId(claim, index);
    const completed = claimCompleted(claim);
    const actual = actualForClaim(claim, id, index, settings);
    const paths = actual && Array.isArray(actual.changedPaths)
      ? actual.changedPaths.map(normalizePath).filter(Boolean)
      : null;
    const scope = delegatedScope(claim);
    const unableToVerify = completed && (!actual || paths === null || text(actual.error) !== '');
    const emptyDiff = completed && !unableToVerify && paths.length === 0;
    const delegatedScopeBreaches = completed && !unableToVerify
      ? paths.filter((candidate) => !scope.some((prefix) => withinPrefix(candidate, prefix)))
      : [];
    const parentScopeBreaches = completed && !unableToVerify
      ? paths.filter((candidate) => (
        parentForbiddenScope.some((prefix) => withinPrefix(candidate, prefix))
        || (parentWriteScope.length > 0 && !parentWriteScope.some((prefix) => withinPrefix(candidate, prefix)))
      ))
      : [];
    const scopeBreaches = [...new Set([...delegatedScopeBreaches, ...parentScopeBreaches])];
    const missingVerificationOutput = completed && !hasVerificationOutput(claim);
    const verified = completed
      && !unableToVerify
      && !emptyDiff
      && scopeBreaches.length === 0
      && !missingVerificationOutput;
    return {
      id,
      claimedComplete: completed,
      delegatedScope: scope,
      actual: actual ? {
        'base-oid': firstText([actual['base-oid'], actual.baseOid]),
        'head-oid': firstText([actual['head-oid'], actual.headOid]),
        changedPaths: paths === null ? null : paths,
        error: text(actual.error) || null,
      } : null,
      emptyDiff,
      delegatedScopeBreaches,
      parentScopeBreaches,
      scopeBreaches,
      missingVerificationOutput,
      unverifiable: unableToVerify,
      verified,
      summary: claimSummary(claim) || null,
      claim,
    };
  });
  const emptyDiffClaims = records.filter((entry) => entry.emptyDiff);
  const scopeBreachClaims = records.filter((entry) => entry.scopeBreaches.length > 0);
  const missingVerificationClaims = records.filter((entry) => entry.missingVerificationOutput);
  const unverifiableClaims = records.filter((entry) => entry.unverifiable);
  const problems = [
    ...emptyDiffClaims.map((entry) => ({ code: 'AGENT_CLAIMED_COMPLETE_DIFF_EMPTY', id: entry.id })),
    ...scopeBreachClaims.map((entry) => ({
      code: 'AGENT_DELEGATED_SCOPE_BREACH', id: entry.id, paths: entry.scopeBreaches.slice(),
    })),
    ...missingVerificationClaims.map((entry) => ({ code: 'AGENT_VERIFICATION_OUTPUT_MISSING', id: entry.id })),
    ...unverifiableClaims.map((entry) => ({ code: 'AGENT_ACTUAL_DIFF_UNAVAILABLE', id: entry.id })),
  ];
  return {
    allowed: problems.length === 0,
    records,
    verifiedClaims: records.filter((entry) => entry.verified),
    emptyDiffClaims,
    scopeBreachClaims,
    missingVerificationClaims,
    unverifiableClaims,
    problems,
  };
}

function decodeAnchor(value) {
  const raw = text(value).replace(/^#/, '');
  if (raw === '') return '';
  try { return decodeURIComponent(raw); } catch (_) { return raw; }
}

function anchorSlug(value) {
  return decodeAnchor(value)
    .toLocaleLowerCase()
    .replace(/[\s_]+/g, '-')
    .replace(/[^\p{L}\p{N}-]/gu, '')
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function rawOutputAnchor(entry) {
  const id = anchorSlug(text(entry && entry.id));
  return id === '' ? '' : `raw-output-${id}`;
}

function parseOutputRef(value, options) {
  const settings = object(options) || {};
  const raw = text(value);
  if (raw.startsWith('node:#')) {
    const anchor = decodeAnchor(raw.slice('node:#'.length));
    return anchor === ''
      ? { ok: false, code: 'TRACE_REFERENCE_INVALID', error: 'node: 引用必须带 #anchor' }
      : { ok: true, kind: 'node', location: raw, anchor };
  }
  if (raw.startsWith('repo:')) {
    const rest = raw.slice('repo:'.length);
    const split = rest.indexOf('#');
    const repoPath = split === -1 ? '' : normalizePath(rest.slice(0, split));
    const anchor = split === -1 ? '' : decodeAnchor(rest.slice(split + 1));
    const safe = repoPath !== ''
      && !repoPath.startsWith('/')
      && !repoPath.includes('\0')
      && repoPath.split('/').every((part) => part !== '' && part !== '.' && part !== '..');
    if (!safe || anchor === '') {
      return {
        ok: false,
        code: 'TRACE_REFERENCE_INVALID',
        error: 'repo: 引用必须是仓库内 repo:<path>#anchor，且不得含 .. 或绝对路径',
      };
    }
    return { ok: true, kind: 'repo', location: raw, path: repoPath, anchor };
  }
  // AH-050-10 前的历史节点已经有 local:/history: 的位置字符串。它们不是新
  // 格式的可切片 Markdown 原件，但保留为兼容位置，不能因升级把旧 history 变成
  // 不可读取；新的 repo/node 引用仍会逐段实解。
  if (raw.startsWith('local:') || raw.startsWith('history:')) {
    if (settings.allowLegacyRefs === true) {
      return { ok: true, kind: 'legacy', location: raw, anchor: null };
    }
    return {
      ok: false,
      code: 'TRACE_LEGACY_REFERENCE_UNRESOLVED',
      error: 'local:/history: 不是可在 history 后实解的原始输出位置；请改用 node:#anchor 或 repo:<path>#anchor',
    };
  }
  return {
    ok: false,
    code: raw === '' ? 'TRACE_OUTPUT_REF_MISSING' : 'TRACE_REFERENCE_UNSUPPORTED',
    error: raw === '' ? 'verification run 缺 output-ref' : `不支持的 output-ref：${raw}`,
  };
}

function headings(textValue) {
  const lines = String(textValue || '').replace(/\r\n/g, '\n').split('\n');
  const found = [];
  for (let offset = 0; offset < lines.length; offset += 1) {
    const match = /^(#{1,6})[ \t]+(.+?)[ \t]*#*[ \t]*$/.exec(lines[offset]);
    if (!match) continue;
    found.push({
      level: match[1].length,
      title: match[2].trim(),
      slug: anchorSlug(match[2]),
      lineIndex: offset,
    });
  }
  return { lines, found };
}

function locateHeading(textValue, anchor) {
  const source = headings(textValue);
  const wanted = anchorSlug(anchor);
  const matches = source.found.filter((entry) => entry.slug === wanted);
  if (wanted === '' || matches.length === 0) {
    return { ok: false, code: 'TRACE_ANCHOR_NOT_FOUND', error: `找不到 heading #${decodeAnchor(anchor)}` };
  }
  if (matches.length > 1) {
    return { ok: false, code: 'TRACE_ANCHOR_AMBIGUOUS', error: `heading #${decodeAnchor(anchor)} 不唯一` };
  }
  const heading = matches[0];
  let boundary = source.lines.length;
  for (const next of source.found) {
    if (next.lineIndex > heading.lineIndex && next.level <= heading.level) {
      boundary = next.lineIndex;
      break;
    }
  }
  return {
    ok: true,
    heading,
    raw: {
      heading: heading.title,
      headingLevel: heading.level,
      startLine: heading.lineIndex + 1,
      endLine: boundary,
      text: source.lines.slice(heading.lineIndex + 1, boundary).join('\n'),
    },
  };
}

function traceFailure(entry, code, error, reference) {
  const run = object(entry && entry.run);
  return {
    ok: false,
    id: text(entry && entry.id) || null,
    command: text(entry && entry.command) || null,
    run: run || null,
    code,
    error,
    reference: reference || null,
  };
}

function resolveVerification(entry, input) {
  const settings = object(input) || {};
  const run = object(entry && entry.run);
  if (!run) return traceFailure(entry, 'TRACE_RUN_MISSING', 'required verification 没有 run 记录');
  const headOid = firstText([run['head-oid'], run.headOid]);
  if (headOid === '') return traceFailure(entry, 'TRACE_HEAD_OID_MISSING', 'verification run 缺 head-oid');
  if (!/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(headOid)) {
    return traceFailure(entry, 'TRACE_HEAD_OID_INVALID', 'verification run.head-oid 必须是完整 40/64 位 commit OID');
  }
  const parsed = parseOutputRef(outputRefFrom(run), {
    allowLegacyRefs: settings.allowLegacyRefs === true,
  });
  if (!parsed.ok) return traceFailure(entry, parsed.code, parsed.error, parsed.location || null);
  if (parsed.kind === 'legacy') {
    // 老节点的字符串位置只供 history --id 展示，不是已经解析出的原始材料。
    // 因此这个结果仍是 unresolved，永远不能进入 close 的第 3 项正向证据。
    return {
      ok: false,
      id: text(entry && entry.id) || null,
      command: text(entry && entry.command) || null,
      run,
      code: 'TRACE_LEGACY_REFERENCE_UNRESOLVED',
      error: 'legacy output-ref 只能展示，不能作为一跳可解析原始输出',
      raw: {
        kind: 'legacy',
        location: parsed.location,
        ref: parsed.location,
        'head-oid': headOid,
        headOid,
        text: null,
      },
      legacy: true,
      unresolved: true,
    };
  }

  // `## 怎么验证` 一类章节只是人类说明，不是某次命令的原始输出。每条
  // required run 都必须落到以 verification id 命名的唯一 raw-output heading，
  // 这样同一段说明/输出不能被多个 run 互相借用。
  const expectedAnchor = rawOutputAnchor(entry);
  if (expectedAnchor === '') {
    return traceFailure(entry, 'TRACE_VERIFICATION_ID_MISSING', 'verification 缺稳定 id，无法定位唯一 raw-output heading', parsed.location);
  }
  if (anchorSlug(parsed.anchor) !== expectedAnchor) {
    return traceFailure(
      entry,
      'TRACE_RAW_OUTPUT_ANCHOR_INVALID',
      `output-ref 必须指向 #${expectedAnchor}，不能把验证说明章节当原始输出`,
      parsed.location,
    );
  }

  let source;
  if (parsed.kind === 'node') {
    source = String(settings.body || '');
  } else {
    if (typeof settings.readRepoAtHead !== 'function') {
      return traceFailure(entry, 'TRACE_REPO_READER_MISSING', 'repo: 引用没有按 run.head-oid 的只读读取器', parsed.location);
    }
    try {
      source = settings.readRepoAtHead({ headOid, path: parsed.path, entry, run });
    } catch (error) {
      return traceFailure(entry, 'TRACE_REPO_READ_FAILED', `读取 repo: 原件失败：${error.message}`, parsed.location);
    }
    if (typeof source !== 'string') {
      return traceFailure(entry, 'TRACE_REPO_READ_FAILED', '按 run.head-oid 读不到 repo: 原件', parsed.location);
    }
  }
  const located = locateHeading(source, parsed.anchor);
  if (!located.ok) return traceFailure(entry, located.code, located.error, parsed.location);
  // 标题本身不是原始输出。若它后面没有任何非空正文，history 只会留下一个
  // 可以伪造的锚点而没有可复核的材料，必须和找不到锚点一样 fail closed。
  if (located.raw.text.trim() === '') {
    return traceFailure(
      entry,
      'TRACE_RAW_OUTPUT_EMPTY',
      `heading #${parsed.anchor} 没有可复核的原始输出正文`,
      parsed.location,
    );
  }
  return {
    ok: true,
    id: text(entry && entry.id) || null,
    command: text(entry && entry.command) || null,
    run,
    raw: {
      kind: parsed.kind,
      location: parsed.location,
      ref: parsed.location,
      'head-oid': headOid,
      headOid,
      path: parsed.kind === 'repo' ? parsed.path : null,
      anchor: parsed.anchor,
      ...located.raw,
    },
    legacy: false,
  };
}

function resolveRequiredVerification(input) {
  const settings = object(input) || {};
  const node = object(settings.node) || {};
  const entries = array(node.verification).filter((entry) => object(entry) && entry.required === true);
  const results = entries.map((entry) => resolveVerification(entry, settings));
  const resolved = results.filter((entry) => entry.ok);
  const problems = results.filter((entry) => !entry.ok).map((entry) => ({
    id: entry.id,
    code: entry.code,
    error: entry.error,
    reference: entry.reference || null,
  }));
  return {
    allowed: problems.length === 0,
    resolved,
    results,
    problems,
    unresolvedCount: problems.length,
  };
}

function verifiedAgentSummaries(closeout, audit) {
  const summaries = [];
  const seen = new Set();
  for (const record of array(audit && audit.verifiedClaims)) {
    if (!record || !record.summary || seen.has(record.id)) continue;
    seen.add(record.id);
    summaries.push({ id: record.id, summary: record.summary });
  }
  // closeout.agentSummaries / item.verified 都是外部自报，不能让它们越过
  // G-20 的实际 Git 审计。保留 closeout 参数只是兼容既有调用签名；history
  // 只能折叠 audit.verifiedClaims 里的摘要。
  return summaries;
}

function traceReferences(closeout, traceAudit) {
  const result = [];
  const seen = new Set();
  const add = (value) => {
    const candidate = typeof value === 'string'
      ? text(value)
      : firstText([value && value.ref, value && value.location, value && value['output-ref']]);
    if (candidate !== '' && !seen.has(candidate)) {
      seen.add(candidate);
      result.push(candidate);
    }
  };
  // closeout.traceRefs 是调用方自报的位置；没有 resolve 结果时不能写进
  // HISTORY，更不能借一个字符串冒充“已一跳解析”。
  array(traceAudit && traceAudit.resolved).forEach((entry) => add(entry && entry.raw));
  return result;
}

module.exports = {
  normalizePath,
  withinPrefix,
  agentClaimsFromCloseout,
  auditAgentClaims,
  auditDelegatedClaims: auditAgentClaims,
  parseOutputRef,
  locateHeading,
  rawOutputAnchor,
  resolveVerification,
  resolveRequiredVerification,
  resolveVerificationTrace: resolveRequiredVerification,
  verifiedAgentSummaries,
  traceReferences,
};
