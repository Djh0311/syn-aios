'use strict';

// Adaptive Harness v0.5 — 测试与证据资产的去向（AH-050-02，退场门第 7 项的接口）
//
// 需求溯源：TS-1 · TS-5 · EX-1 · EX-8 · §8 · §12 #28
//
// 本文件只定**接口与失败码**：把本次任务新增或改动的那批资产当作一批对象来处理，
// 每一样在退场（closeout）时定下去向。资产识别器的实现属另一片叶子，
// 但「有没有去向」这条判据必须在收尾门里生效，否则东西就永远只进不出。
//
// 一份资产的处理窗口，四件事一次说完：
//   * 长期资产与用完即扔的临时脚本先分开——REGRESSION / FIXTURE / COMPATIBILITY
//     是长期的，DIAGNOSTIC 与 EVIDENCE_RUNNER 默认是临时的；
//   * 每一份都要有 disposition（去向）：保留、删除、转交、随任务搬进历史平面、
//     或迁出仓库到一个已登记的外部根路径；
//   * 退场之后当前区只留结论摘要与链接（summary + link），原件在历史平面
//     或已登记的仓库外路径，仍可按引用解析；
//   * 范围按本次改动定：清单来自 base-oid..HEAD 的新增 / 改动 / 删除，
//     不是每次扫描全仓历史测试。
//
// 本文件只读：分类与判定都不写文件。

// §8 的五分类。长期与临时先分开，这是 TS-1 的前半句。
const ASSET_CLASSES = Object.freeze([
  'REGRESSION',
  'FIXTURE',
  'COMPATIBILITY',
  'DIAGNOSTIC',
  'EVIDENCE_RUNNER',
]);

const LONG_TERM_CLASSES = Object.freeze(['REGRESSION', 'FIXTURE', 'COMPATIBILITY']);
const DISPOSABLE_CLASSES = Object.freeze(['DIAGNOSTIC', 'EVIDENCE_RUNNER']);

// 每一份资产在收尾时的去向。
const ASSET_DISPOSITIONS = Object.freeze([
  'RETAINED',
  'DELETED',
  'TRANSFERRED',
  'MOVED_TO_HISTORY',
  'RELOCATED_EXTERNAL',
]);

// 证据引用的三种登记方案，其余一律非法（EX-8 / GIT-9 / TS-5）。
// 刻意不接受指向任务 linked worktree 内临时文件的引用——那个目录会随 disposition 消失。
const EVIDENCE_REF_SCHEMES = Object.freeze(['history', 'repo', 'external']);

const EVIDENCE_REF_PATTERN = /^(history|repo|external):(.+)$/;

// C-TS-05 用这些稳定枚举逐码执行 bad -> fixed 行为向量；不是静态关键词清单。
const ASSET_AUDIT_REJECTION_CODES = Object.freeze([
  'ASSET_UNCLASSIFIED',
  'ASSET_CLASS_UNKNOWN',
  'ASSET_DISPOSITION_MISSING',
  'FIXTURE_OWNER_MISSING',
  'COMPATIBILITY_REVIEW_BY_MISSING',
  'TRACKED_DIAGNOSTIC_INCOMPLETE',
  'EVIDENCE_REF_UNREGISTERED',
  'REQUIRED_EVIDENCE_DESTINATION_MISSING',
  'NEW_TEST_RATIONALE_MISSING',
  'REGRESSION_REPLACEMENT_OR_REASON_MISSING',
]);
const IGNORE_AUDIT_REJECTION_CODES = Object.freeze(['IGNORE_RULE_WITHOUT_REASON']);
const UNTRACKED_AUDIT_REJECTION_CODES = Object.freeze(['PRODUCT_DEPENDENCY_UNTRACKED']);

function isLongTerm(assetClass) {
  return LONG_TERM_CLASSES.includes(assetClass);
}

function isDisposable(assetClass) {
  return DISPOSABLE_CLASSES.includes(assetClass);
}

/** 解析一条证据引用。返回 null 表示这条引用不被接受。 */
function parseEvidenceRef(value) {
  const text = typeof value === 'string' ? value.trim() : '';
  const match = EVIDENCE_REF_PATTERN.exec(text);
  if (!match) return null;
  const scheme = match[1];
  const target = match[2].trim();
  if (target === '') return null;
  if (target.startsWith('/') || target.includes('\\') || target.includes('\0')) return null;
  const segments = target.split('/');
  if (segments.some((segment) => segment === '' || segment === '.' || segment === '..')) return null;
  if (scheme === 'external' && segments.length < 2) return null;
  return { scheme, target };
}

function hasText(value) {
  return typeof value === 'string' && value.trim() !== '';
}

/** 可执行测试文件；供真实 tracked inventory 筛选，刻意不把 fixture/evidence 混进来。 */
function isTestFilePath(filePath) {
  const value = typeof filePath === 'string' ? filePath : '';
  return /\.(?:test|spec)\.[cm]?[jt]sx?$/i.test(value);
}

/** §8 的宽资产识别：测试、fixture、诊断、一次性 runner、浏览器输出和 evidence。 */
function isTestAssetPath(filePath) {
  const value = typeof filePath === 'string' ? filePath : '';
  return isTestFilePath(value)
    || /(?:^|\/)(?:tests?|__tests__|fixtures?|testdata|diagnostics?|evidence|test-results|playwright-report)(?:\/|$)/i.test(value)
    || /(?:^|\/)(?:evidence[-_.]?runner|diagnostic[-_.]?probe)(?:[./_-]|$)/i.test(value);
}

function normalizedChangedEntries(settings) {
  if (Array.isArray(settings.changedEntries)
    && (settings.changedEntries.length > 0 || !Array.isArray(settings.changedPaths))) {
    return settings.changedEntries
      .filter((entry) => entry && ['A', 'M', 'D', 'R', 'C'].includes(entry.status))
      .map((entry) => ({
        status: entry.status,
        oldPath: typeof entry.oldPath === 'string' ? entry.oldPath : null,
        newPath: typeof entry.newPath === 'string' ? entry.newPath : null,
        ...(Number.isFinite(entry.score) ? { score: entry.score } : {}),
      }));
  }
  // 旧调用方只给 name-only 时保持兼容；它没有资格冒充 A/D/R 事实，因此只按 M 审计。
  return (Array.isArray(settings.changedPaths) ? settings.changedPaths : [])
    .filter((filePath) => typeof filePath === 'string' && filePath !== '')
    .map((filePath) => ({ status: 'M', oldPath: filePath, newPath: filePath }));
}

function auditIgnoreRuleReasons(input) {
  const settings = input || {};
  const addedRules = Array.isArray(settings.addedRules)
    ? settings.addedRules
    : (Array.isArray(settings.ignoreRules) ? settings.ignoreRules : []);
  const suppliedReasons = new Map(
    (Array.isArray(settings.reasons) ? settings.reasons : [])
      .filter((entry) => entry && hasText(entry.pattern) && hasText(entry.reason))
      .map((entry) => [entry.pattern, entry.reason.trim()]),
  );
  const problems = [];
  const resolved = [];
  for (const entry of addedRules) {
    if (!entry || !hasText(entry.pattern)) continue;
    const reason = hasText(entry.reason) ? entry.reason.trim() : suppliedReasons.get(entry.pattern);
    if (!hasText(reason)) {
      problems.push({
        code: 'IGNORE_RULE_WITHOUT_REASON',
        pattern: entry.pattern,
        path: hasText(entry.file) ? entry.file : '.gitignore',
        message: `本次新增的忽略规则「${entry.pattern}」没有理由条目`,
      });
      continue;
    }
    resolved.push({
      pattern: entry.pattern,
      path: hasText(entry.file) ? entry.file : '.gitignore',
      reason,
    });
  }
  return { ok: problems.length === 0, problems, resolved };
}

function pathWithinAny(filePath, scopes) {
  if (!Array.isArray(scopes) || scopes.length === 0) return true;
  return scopes.some((scope) => {
    const prefix = typeof scope === 'string' ? scope.replace(/\/+$/, '') : '';
    return prefix !== '' && (filePath === prefix || filePath.startsWith(`${prefix}/`));
  });
}

/**
 * GIT-9 的纯组合接口。调用方传入现场未跟踪事实，并以 productDependency=true
 * （或精确 callback / path 集合）完成分类；本层不读工作树、不猜附近文件。
 */
function auditUntrackedProductDependencies(input) {
  const settings = input || {};
  const candidates = Array.isArray(settings.untrackedEntries)
    ? settings.untrackedEntries
    : (Array.isArray(settings.untracked) ? settings.untracked : []);
  const exactPaths = new Set(Array.isArray(settings.productDependencyPaths)
    ? settings.productDependencyPaths
    : []);
  const classify = typeof settings.isProductDependencyPath === 'function'
    ? settings.isProductDependencyPath
    : null;
  const productDependencies = [];
  const outsideWriteScope = [];

  for (const candidate of candidates) {
    const filePath = typeof candidate === 'string'
      ? candidate
      : (candidate && typeof candidate.path === 'string' ? candidate.path : '');
    if (filePath === '') continue;
    if (!pathWithinAny(filePath, settings.writeScope)) {
      outsideWriteScope.push(filePath);
      continue;
    }
    const explicitlyClassified = candidate && typeof candidate === 'object'
      && candidate.productDependency === true;
    if (!explicitlyClassified && !exactPaths.has(filePath) && !(classify && classify(filePath) === true)) {
      continue;
    }
    productDependencies.push({
      path: filePath,
      kind: candidate && typeof candidate === 'object' && hasText(candidate.kind)
        ? candidate.kind
        : 'PRODUCT_DEPENDENCY',
    });
  }

  const problems = productDependencies.length === 0 ? [] : [{
    code: 'PRODUCT_DEPENDENCY_UNTRACKED',
    paths: productDependencies.map((entry) => entry.path),
    message: `写面内存在未跟踪的产品依赖类文件：${productDependencies.map((entry) => entry.path).join('，')}`,
  }];
  return {
    ok: problems.length === 0,
    problems,
    productDependencies,
    outsideWriteScope,
  };
}

/**
 * 退场门第 7 项的判定输入：本次真实 A/M/D/R/C 增量 + 每份资产的分类与去向。
 * 未分类或未定去向的数量 > 0 即失败，并逐份列出。
 */
function auditAssetDispositions(input) {
  const settings = input || {};
  const declared = Array.isArray(settings.assets) ? settings.assets : [];
  const changed = normalizedChangedEntries(settings);
  const known = new Map();
  for (const asset of declared) {
    if (!asset || typeof asset.path !== 'string') continue;
    known.set(asset.path, asset);
  }

  const problems = [];
  const resolved = [];
  const looksLikeAsset = typeof settings.isAssetPath === 'function'
    ? settings.isAssetPath
    : isTestAssetPath;
  const looksLikeTestFile = typeof settings.isTestFilePath === 'function'
    ? settings.isTestFilePath
    : isTestFilePath;

  for (const entry of changed) {
    const paths = [entry.newPath, entry.oldPath].filter((item) => typeof item === 'string');
    const asset = (entry.newPath && known.get(entry.newPath))
      || (entry.oldPath && known.get(entry.oldPath))
      || null;
    if (!asset && !paths.some(looksLikeAsset)) continue;
    const filePath = entry.newPath || entry.oldPath;
    if (!asset) {
      problems.push({ code: 'ASSET_UNCLASSIFIED', path: filePath, message: `${filePath} 没有分类与去向` });
      continue;
    }
    if (!ASSET_CLASSES.includes(asset.assetClass)) {
      problems.push({ code: 'ASSET_CLASS_UNKNOWN', path: filePath, message: `${filePath} 的分类 ${asset.assetClass} 不在五类之内` });
    }
    if (!ASSET_DISPOSITIONS.includes(asset.disposition)) {
      problems.push({ code: 'ASSET_DISPOSITION_MISSING', path: filePath, message: `${filePath} 没有可接受的去向` });
    }
    const remainsAvailable = ASSET_DISPOSITIONS.includes(asset.disposition)
      && asset.disposition !== 'DELETED';
    if (asset.assetClass === 'FIXTURE' && remainsAvailable && !hasText(asset.owner)) {
      problems.push({
        code: 'FIXTURE_OWNER_MISSING',
        path: filePath,
        message: `继续保留或转交的 FIXTURE ${filePath} 没有明确 test owner`,
      });
    }
    if (asset.assetClass === 'COMPATIBILITY' && remainsAvailable && !hasText(asset.reviewBy)) {
      problems.push({
        code: 'COMPATIBILITY_REVIEW_BY_MISSING',
        path: filePath,
        message: `继续保留或转交的 COMPATIBILITY ${filePath} 没有 review-by 复审点`,
      });
    }
    if (isDisposable(asset.assetClass) && asset.disposition === 'RETAINED') {
      for (const key of ['owner', 'reason', 'reviewBy']) {
        if (typeof asset[key] !== 'string' || asset[key].trim() === '') {
          problems.push({ code: 'TRACKED_DIAGNOSTIC_INCOMPLETE', path: filePath, message: `留下来的临时资产 ${filePath} 缺 ${key}` });
        }
      }
    }
    const evidenceRef = parseEvidenceRef(asset.evidenceRef);
    if (asset.evidenceRef !== undefined && asset.evidenceRef !== null) {
      if (!evidenceRef) {
        problems.push({ code: 'EVIDENCE_REF_UNREGISTERED', path: filePath, message: `${filePath} 的证据引用 ${asset.evidenceRef} 不是三种登记方案之一` });
      }
    }
    if ((asset.requiredEvidence === true || asset.evidenceRequired === true || asset.required === true)
      && (!evidenceRef || asset.disposition === 'DELETED')) {
      problems.push({
        code: 'REQUIRED_EVIDENCE_DESTINATION_MISSING',
        path: filePath,
        message: `${filePath} 是 required evidence，但没有历史、仓库或已登记外部路径这一稳定去向`,
      });
    }
    if ((entry.status === 'A' || entry.status === 'C')
      && entry.newPath
      && looksLikeTestFile(entry.newPath)
      && !hasText(asset.newFileRationale)) {
      problems.push({
        code: 'NEW_TEST_RATIONALE_MISSING',
        path: entry.newPath,
        message: `新建测试文件 ${entry.newPath} 没说明为何不能并入已有测试文件`,
      });
    }
    // 删除之后已经无法从 HEAD 读取原文件内的历史分类；若允许 closeout 当场把
    // *.test/*.spec 重标成 DIAGNOSTIC，就能绕过 TS-3。真实 Git oldPath 是测试文件
    // 时始终要求 replacement/removalReason；声明为 REGRESSION 的其他命名同样要求。
    const retiresExecutableTest = entry.oldPath && looksLikeTestFile(entry.oldPath);
    if ((entry.status === 'D' || entry.status === 'R')
      && (asset.assetClass === 'REGRESSION' || retiresExecutableTest)
      && !hasText(asset.replacement)
      && !hasText(asset.removalReason)) {
      problems.push({
        code: 'REGRESSION_REPLACEMENT_OR_REASON_MISSING',
        path: entry.oldPath || filePath,
        message: `正式回归 ${entry.oldPath || filePath} 被删除或合并，但没有 replacement 或理由`,
      });
    }
    resolved.push({
      path: filePath,
      oldPath: entry.oldPath,
      newPath: entry.newPath,
      status: entry.status,
      assetClass: asset.assetClass,
      disposition: asset.disposition,
    });
  }

  const ignoreAudit = auditIgnoreRuleReasons({
    addedRules: Array.isArray(settings.addedIgnoreRules)
      ? settings.addedIgnoreRules
      : settings.ignoreRules,
    reasons: settings.ignoreReasons,
  });
  const untrackedAudit = auditUntrackedProductDependencies({
    untrackedEntries: settings.untrackedEntries,
    untracked: settings.untracked,
    productDependencyPaths: settings.productDependencyPaths,
    isProductDependencyPath: settings.isProductDependencyPath,
    writeScope: settings.writeScope,
  });
  problems.push(...ignoreAudit.problems, ...untrackedAudit.problems);

  return {
    problems,
    resolved,
    unresolvedCount: problems.length,
    ignoreAudit,
    untrackedAudit,
  };
}

/** 退场后当前区只留摘要与链接：把原件的去处折成一行指针。 */
function summarizeForCurrent(asset) {
  const reference = parseEvidenceRef(asset && asset.evidenceRef);
  return {
    path: asset && asset.path ? asset.path : null,
    assetClass: asset && asset.assetClass ? asset.assetClass : null,
    disposition: asset && asset.disposition ? asset.disposition : null,
    link: reference ? `${reference.scheme}:${reference.target}` : null,
  };
}

module.exports = {
  ASSET_CLASSES,
  LONG_TERM_CLASSES,
  DISPOSABLE_CLASSES,
  ASSET_DISPOSITIONS,
  EVIDENCE_REF_SCHEMES,
  ASSET_AUDIT_REJECTION_CODES,
  IGNORE_AUDIT_REJECTION_CODES,
  UNTRACKED_AUDIT_REJECTION_CODES,
  isLongTerm,
  isDisposable,
  isTestFilePath,
  isTestAssetPath,
  parseEvidenceRef,
  auditIgnoreRuleReasons,
  auditUntrackedProductDependencies,
  auditAssetDispositions,
  summarizeForCurrent,
};
