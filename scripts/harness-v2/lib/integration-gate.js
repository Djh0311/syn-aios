'use strict';

// Adaptive Harness v0.5 — 并线前的证明与并线后的残留（AH-050-07）
//
// 需求溯源：
//   GIT-6  没干完的活也要落盘在自己的任务分支上，不能长期散在共享工作树里
//   GIT-7  并进主线之前要能证明：干净检出下装得上跑得起来测得过、没有任何凭据
//          混进提交、有一次跳出本机的独立验证跑过
//   GIT-8  合并完成后不许留残留改动；被忽略的每一类都要有说得出口的规则
//   GIT-9  迁移、脚本、配置这类产品真正依赖的东西必须进版本库；
//          凭据、环境文件、浏览器 profile、日志绝不进
//   §6.3   集成只支持本地快进，且只在专用、完全干净的 integration 工作副本里进行
//
// 本模块只读：全部结论来自只读 git 查询，一条写命令都不跑。

const gitFacts = require('./git-facts');
const scope = require('./scope');

// ---------------------------------------------------------------------------
// 两类路径：绝不进库的，和必须进库的
// ---------------------------------------------------------------------------

// 凭据、环境文件、浏览器 profile、日志：命中即拒，绝不进库（GIT-7 / GIT-9）。
const NEVER_COMMIT_PATTERNS = Object.freeze([
  { code: 'CREDENTIAL_ENV_FILE', pattern: /(?:^|\/)\.env(?:\.[^/]+)?$/i, label: '环境文件' },
  { code: 'CREDENTIAL_PRIVATE_KEY', pattern: /(?:^|\/)(?:id_rsa|id_ed25519|id_dsa)(?:\.[^/]+)?$/i, label: '私钥' },
  { code: 'CREDENTIAL_KEY_MATERIAL', pattern: /\.(?:pem|key|p12|pfx|jks|keystore)$/i, label: '密钥材料' },
  { code: 'CREDENTIAL_SECRET_STORE', pattern: /(?:^|\/)(?:secrets?|credentials?)(?:\.(?:ya?ml|json|toml|ini))?$/i, label: '凭据文件' },
  { code: 'CREDENTIAL_TOKEN_FILE', pattern: /(?:^|\/)\.(?:npmrc|netrc|pypirc|aws\/credentials|docker\/config\.json)$/i, label: '令牌文件' },
  { code: 'BROWSER_PROFILE', pattern: /(?:^|\/)(?:Default|Profile \d+)\/(?:Cookies|Login Data|Web Data)$/i, label: '浏览器 profile' },
  { code: 'BROWSER_PROFILE_DIR', pattern: /(?:^|\/)(?:chrome-profile|browser-profile|playwright\/\.auth)(?:\/|$)/i, label: '浏览器 profile' },
  { code: 'RUNTIME_LOG', pattern: /\.log$/i, label: '日志' },
]);

// 产品真正依赖的东西：迁移、脚本、配置。它们长期未跟踪就是产品跑不起来的那一侧（GIT-9）。
const PRODUCT_DEPENDENCY_PATTERNS = Object.freeze([
  { code: 'MIGRATION', pattern: /(?:^|\/)(?:migrations?|db\/migrate|schema)(?:\/|$)|\.sql$/i, label: '迁移' },
  { code: 'SCRIPT', pattern: /(?:^|\/)(?:scripts?|bin)(?:\/|$)|\.(?:sh|bash|zsh|ps1)$/i, label: '脚本' },
  { code: 'CONFIG', pattern: /(?:^|\/)(?:config|conf)(?:\/|$)|\.(?:ya?ml|toml|ini|conf)$|(?:^|\/)[a-z0-9.-]*(?:rc|\.config)\.(?:json|js|cjs|mjs)$/i, label: '配置' },
]);

function matchNeverCommit(filePath) {
  return NEVER_COMMIT_PATTERNS.find((entry) => entry.pattern.test(String(filePath))) || null;
}

function matchProductDependency(filePath) {
  return PRODUCT_DEPENDENCY_PATTERNS.find((entry) => entry.pattern.test(String(filePath))) || null;
}

function lines(result) {
  if (!result || !result.ok) return [];
  return result.stdout.split('\n').map((line) => line.trim()).filter((line) => line !== '');
}

// ---------------------------------------------------------------------------
// GIT-6：没干完的活必须真的落在自己的分支上，而且这件事要能被核对
// ---------------------------------------------------------------------------

/**
 * 核对未完成工作的落盘。记录里声称的每一个 WIP 提交都必须：
 *   * 在本仓真实存在（不是编出来的 OID）；
 *   * 并且**可达于本任务自己的分支**（是 task-branch 的祖先）。
 * 任何一条对不上就拒绝退场——落盘不能是自报，必须现场问版本库。
 * 被卡住做不下去的任务同样适用：做不下去也要先把半成品提交到自己分支上。
 */
function verifyWipCommits(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const taskBranch = settings.taskBranch;
  const claimed = Array.isArray(settings.wipCommits)
    ? settings.wipCommits.filter((item) => typeof item === 'string' && item.trim() !== '')
    : [];
  const problems = [];
  const verified = [];

  if (settings.hasUncommittedWork === true && claimed.length === 0) {
    problems.push({
      code: 'WIP_NOT_COMMITTED',
      message: '工作副本里还有没干完的活，但记录里一个 WIP 提交都没有；未完成的东西不许散在共享工作树里，拒绝退场',
    });
  }
  for (const oid of claimed) {
    if (!gitFacts.objectExists(cwd, oid)) {
      problems.push({ code: 'WIP_COMMIT_UNREACHABLE', oid, message: `声称的未完成提交 ${oid} 在本仓找不到，拒绝退场` });
      continue;
    }
    // 存在还不够：它必须落在**本任务自己的分支**上，才算「落盘在自己的任务分支」。
    const onOwnBranch = gitFacts.isAncestor(cwd, oid, taskBranch);
    if (!onOwnBranch) {
      problems.push({
        code: 'WIP_COMMIT_OFF_TASK_BRANCH',
        oid,
        message: `未完成提交 ${oid} 不可达于本任务分支 ${taskBranch}，拒绝退场`,
      });
      continue;
    }
    verified.push(oid);
  }
  return { ok: problems.length === 0, verified, problems };
}

// ---------------------------------------------------------------------------
// GIT-7 第一证：本任务**全部提交**的凭据扫描（不是某一次的暂存快照）
// ---------------------------------------------------------------------------

function taskCommitList(cwd, baseOid, headRef) {
  return lines(gitFacts.runGit(['log', '--format=%H', `${baseOid}..${headRef}`], { cwd }));
}

/**
 * 并线前的凭据扫描，范围是本任务的**全部提交**：`base..head` 这一整段，
 * 逐个提交列出它动过的路径。只看某一次暂存内容是不够的——
 * 早先某个提交里混进去的密钥照样会随并线进主线。
 *
 * 正面一侧同样成立：不含凭据的正常提交必须放行，不能靠一律拒绝混过去。
 */
function scanCommitRangeForCredentials(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const baseOid = settings.baseOid;
  const headRef = settings.headRef || 'HEAD';
  const commits = Array.isArray(settings.commits) ? settings.commits : taskCommitList(cwd, baseOid, headRef);
  const hits = [];
  const scanned = [];
  for (const oid of commits) {
    const touched = settings.pathsOfCommit
      ? settings.pathsOfCommit(oid)
      : lines(gitFacts.runGit(['show', '--name-only', '--format=', oid], { cwd }));
    scanned.push({ oid, paths: touched.length });
    for (const filePath of touched) {
      const banned = matchNeverCommit(filePath);
      if (!banned) continue;
      hits.push({ code: banned.code, label: banned.label, oid, path: filePath });
    }
  }
  return {
    // 命中即拒：凭据、环境文件、浏览器 profile、日志一律不许并进主线。
    ok: hits.length === 0,
    range: `${baseOid}..${headRef}`,
    commitsScanned: scanned.length,
    scanned,
    hits,
    refusal: hits.length === 0 ? null : {
      code: 'CREDENTIAL_FOUND_IN_TASK_COMMITS',
      message: `本任务提交里发现禁入内容，拒绝并线：${hits.map((hit) => `${hit.path}@${hit.oid.slice(0, 8)}`).join('，')}`,
    },
  };
}

// ---------------------------------------------------------------------------
// GIT-7 第二、三证：干净检出跑得起来 + 一次跳出本机的独立验证
// ---------------------------------------------------------------------------

// 干净检出的验证记录：三项齐全才算数。长期污染的开发目录里跑一遍不算。
const CLEAN_CHECKOUT_RECORD_FIELDS = Object.freeze(['checkoutPath', 'checkoutOid', 'command', 'result']);
// 跳出本机的独立验证记录：命令、检出标识、结果三项齐全才算数。
const OFF_MACHINE_RECORD_FIELDS = Object.freeze(['command', 'checkoutOid', 'result']);

function missingFields(record, fields) {
  const source = record && typeof record === 'object' ? record : {};
  return fields.filter((key) => typeof source[key] !== 'string' || source[key].trim() === '');
}

/**
 * 干净检出的验证：必须在长期污染的开发目录**之外**重新检出一份，在那里装上、跑起来、
 * 测过，并把检出位置、检出 OID、命令原文与结果记录下来。记录缺项即视为没跑过，
 * 结果不是 PASS 同样拒绝并线。
 */
function requireCleanCheckoutVerification(input) {
  const settings = input || {};
  const record = settings.cleanCheckout;
  const missing = missingFields(record, CLEAN_CHECKOUT_RECORD_FIELDS);
  const problems = [];
  if (!record) {
    problems.push({ code: 'CLEAN_CHECKOUT_MISSING', message: '没有干净检出的验证记录：并线前必须在开发目录之外重新检出并跑一遍' });
  } else if (missing.length > 0) {
    problems.push({ code: 'CLEAN_CHECKOUT_RECORD_INCOMPLETE', message: `干净检出验证记录缺 ${missing.join(' / ')}` });
  } else if (String(record.result).trim().toUpperCase() !== 'PASS') {
    problems.push({ code: 'CLEAN_CHECKOUT_NOT_PASSING', message: `干净检出里的验证结果是 ${record.result}，拒绝并线` });
  } else if (settings.taskWorktree && scope.pathWithin(record.checkoutPath, settings.taskWorktree)) {
    problems.push({
      code: 'CLEAN_CHECKOUT_INSIDE_TASK_WORKTREE',
      message: `记录的检出位置 ${record.checkoutPath} 还在任务工作副本里，不构成一次干净检出`,
    });
  }
  return { ok: problems.length === 0, problems };
}

/**
 * 跳出本机的独立验证：必须有一次在本机之外跑过的记录（远端 runner、CI job、
 * 另一台机器都算），并记下命令、检出 OID、结果三项。缺记录即拒绝并线。
 */
function requireOffMachineVerification(input) {
  const settings = input || {};
  const record = settings.offMachine;
  const missing = missingFields(record, OFF_MACHINE_RECORD_FIELDS);
  const problems = [];
  if (!record) {
    problems.push({ code: 'OFF_MACHINE_VERIFICATION_MISSING', message: '没有跳出本机的独立验证记录：并线前必须有一次本机之外跑过的结果' });
  } else if (missing.length > 0) {
    problems.push({ code: 'OFF_MACHINE_RECORD_INCOMPLETE', message: `独立验证记录缺 ${missing.join(' / ')}` });
  } else if (String(record.result).trim().toUpperCase() !== 'PASS') {
    problems.push({ code: 'OFF_MACHINE_NOT_PASSING', message: `本机之外的独立验证结果是 ${record.result}，拒绝并线` });
  } else if (settings.headOid && record.checkoutOid !== settings.headOid) {
    problems.push({
      code: 'OFF_MACHINE_STALE',
      message: `独立验证跑的是 ${record.checkoutOid}，当前要并的是 ${settings.headOid}`,
    });
  }
  return { ok: problems.length === 0, problems };
}

// ---------------------------------------------------------------------------
// GIT-8：合并后不留残留 + 本次新增的每一类忽略都要有理由
// ---------------------------------------------------------------------------

function ignoreFilesInRange(cwd, baseOid, headRef) {
  const changed = lines(gitFacts.runGit(['diff', '--name-only', `${baseOid}..${headRef}`], { cwd }));
  return changed.filter((filePath) => /(?:^|\/)\.gitignore$|(?:^|\/)\.git\/info\/exclude$/.test(filePath));
}

/**
 * 本次新增的忽略规则逐条要有说得出口的理由。
 * 先把 base..head 之间 ignore 文件的**新增行**取出来，再逐条对着理由清单核对；
 * 找不到理由的那条就拒绝收尾——不许随手加一条忽略把脏东西藏起来。
 */
function auditIgnoreRules(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const baseOid = settings.baseOid;
  const headRef = settings.headRef || 'HEAD';
  const reasons = new Map();
  for (const entry of Array.isArray(settings.reasons) ? settings.reasons : []) {
    if (!entry || typeof entry.pattern !== 'string') continue;
    if (typeof entry.reason === 'string' && entry.reason.trim() !== '') reasons.set(entry.pattern.trim(), entry.reason.trim());
  }

  const added = [];
  const files = Array.isArray(settings.ignoreFiles) ? settings.ignoreFiles : ignoreFilesInRange(cwd, baseOid, headRef);
  for (const filePath of files) {
    const diff = settings.diffOfFile
      ? settings.diffOfFile(filePath)
      : (gitFacts.runGit(['diff', '--unified=0', `${baseOid}..${headRef}`, '--', filePath], { cwd }).stdout || '');
    for (const line of String(diff).split('\n')) {
      if (!line.startsWith('+') || line.startsWith('+++')) continue;
      const pattern = line.slice(1).trim();
      if (pattern === '' || pattern.startsWith('#')) continue;
      added.push({ file: filePath, pattern });
    }
  }

  const problems = [];
  for (const entry of added) {
    if (reasons.has(entry.pattern)) continue;
    problems.push({
      code: 'IGNORE_RULE_WITHOUT_REASON',
      pattern: entry.pattern,
      file: entry.file,
      message: `本次新增的忽略规则「${entry.pattern}」（${entry.file}）没有理由条目，拒绝收尾`,
    });
  }
  return { ok: problems.length === 0, added, problems };
}

/**
 * 合并完成之后不许留残留改动。
 * 退场 / 收尾那一刻现场读整个工作副本的状态；除了开工时登记在册的用户原有未提交内容，
 * 一条残留都不许剩。剩了就拒绝，不去猜、更不去替用户清理。
 */
function auditResidue(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const registered = new Set((Array.isArray(settings.dirtyBaseline) ? settings.dirtyBaseline : [])
    .map((entry) => (entry && entry.path ? String(entry.path) : '')).filter(Boolean));
  const status = Array.isArray(settings.statusNow) ? settings.statusNow : gitFacts.porcelainStatus(cwd);
  const residue = [];
  for (const line of status) {
    const filePath = line.slice(3).trim();
    if (registered.has(filePath)) continue;
    residue.push({ path: filePath, status: line.slice(0, 2) });
  }
  return {
    ok: residue.length === 0,
    residue,
    problems: residue.length === 0 ? [] : [{
      code: 'RESIDUE_AFTER_INTEGRATION',
      message: `收尾时工作副本仍有残留改动，拒绝退场：${residue.map((entry) => entry.path).join('，')}`,
      paths: residue.map((entry) => entry.path),
    }],
  };
}

// ---------------------------------------------------------------------------
// GIT-9：写面里躺着的未跟踪文件，一类必须进库，一类绝不进库
// ---------------------------------------------------------------------------

function untrackedIn(cwd, prefixes) {
  const scopePrefixes = Array.isArray(prefixes) ? prefixes.filter((item) => String(item).trim() !== '') : [];
  const args = ['ls-files', '--others', '--exclude-standard'];
  if (scopePrefixes.length > 0) args.push('--', ...scopePrefixes);
  return lines(gitFacts.runGit(args, { cwd }));
}

/**
 * 退场前扫一遍**本任务写面之内**还没进版本库的文件，按两类分开处置：
 *
 *   * 属产品依赖类型（迁移、脚本、配置）的未跟踪文件 —— 拒绝退场，并逐条列出清单。
 *     漏进版本库才是真正会让产品跑不起来的那一侧，光有「不许进」的一半不够。
 *   * 属凭据、环境文件、浏览器 profile、日志的 —— 绝不进库，列为「保持在库外」，
 *     并且不得因为它们躺在写面里就被顺手提交。
 *   * 其余的照常放行，不因为「看着眼生」就拦。
 */
function auditUntrackedProductDependencies(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const writeScope = Array.isArray(settings.writeScope) ? settings.writeScope : [];
  const candidates = Array.isArray(settings.untracked) ? settings.untracked : untrackedIn(cwd, writeScope);

  const mustEnterRepository = [];
  const mustStayOut = [];
  const ignoredHere = [];
  for (const filePath of candidates) {
    if (writeScope.length > 0 && !scope.pathWithinAny(filePath, writeScope)) { ignoredHere.push(filePath); continue; }
    const banned = matchNeverCommit(filePath);
    if (banned) {
      mustStayOut.push({ code: banned.code, label: banned.label, path: filePath });
      continue;
    }
    const dependency = matchProductDependency(filePath);
    if (dependency) {
      mustEnterRepository.push({ code: dependency.code, label: dependency.label, path: filePath });
    }
  }

  const problems = [];
  if (mustEnterRepository.length > 0) {
    problems.push({
      code: 'PRODUCT_DEPENDENCY_UNTRACKED',
      message: '本任务写面里躺着未跟踪的产品依赖类文件（迁移 / 脚本 / 配置），必须进版本库，拒绝退场：'
        + mustEnterRepository.map((entry) => `${entry.path}（${entry.label}）`).join('，'),
      items: mustEnterRepository,
    });
  }
  return {
    ok: problems.length === 0,
    mustEnterRepository,
    // 这一类**绝不进库**：既不提交，也不因为它们挡着退场就顺手加进来。
    mustStayOut,
    outsideWriteScope: ignoredHere,
    problems,
  };
}

// ---------------------------------------------------------------------------
// 并行的是执行，不是集成
// ---------------------------------------------------------------------------

// 一次集成的三个阶段。**只有 IN_PROGRESS 那一次**算「正在进行」。
// AWAITING_CONFIRMATION（等着用户确认）、CONFIRMED_NOT_STARTED（确认了还没动手）、
// 以及叶子自己处在「等待集成」这个状态，一律不算——拿它们去拦别人的集成或开工，
// 就是把已经拆掉的那道总闸换个名字装回来。
const INTEGRATION_PHASES = Object.freeze([
  'AWAITING_CONFIRMATION',
  'CONFIRMED_NOT_STARTED',
  'IN_PROGRESS',
  'FINISHED',
]);

const INTEGRATION_IN_PROGRESS_PHASE = 'IN_PROGRESS';

/**
 * 同一条 integration 分支同一时刻只能有一次正在进行的集成（§6.3）。
 *
 * 判据只认「已取得用户确认、已经开始、尚未结束」的那一次。等待确认的、
 * 确认了还没开始的、处于等待集成状态的叶子都不构成阻断理由，
 * 也不得据此拒绝他人的集成或开工。
 */
function integrationIsInProgress(entry) {
  if (!entry || typeof entry !== 'object') return false;
  // 已结束的不算。phase 写着 FINISHED 与 finished 标记，任一为真都算结束。
  if (entry.finished === true || entry.phase === 'FINISHED') return false;
  // 「正在进行」= 已确认 ∧ 已开始。phase 只是这两件事的一个说法，
  // **不是额外的必要条件**：少写一个 phase 字段就能开第二次集成，那道串行门等于没有。
  const confirmed = entry.confirmed === true || entry.phase === INTEGRATION_IN_PROGRESS_PHASE;
  const started = entry.started === true || entry.phase === INTEGRATION_IN_PROGRESS_PHASE;
  return confirmed && started;
}

function integrationInProgressOn(integrationRef, entries) {
  const list = Array.isArray(entries) ? entries : [];
  return list.find((entry) => entry
    && entry.integrationRef === integrationRef
    && integrationIsInProgress(entry)) || null;
}

function refuseSecondIntegration(integrationRef, entries) {
  const running = integrationInProgressOn(integrationRef, entries);
  if (!running) return { ok: true, refusal: null };
  return {
    ok: false,
    running,
    refusal: {
      code: 'INTEGRATION_ALREADY_IN_PROGRESS',
      message: `${integrationRef} 上有一次已确认、已开始、尚未结束的集成（${running.taskId}）正在进行，请等它结束`,
    },
  };
}

// ---------------------------------------------------------------------------
// 并线前的整体判定
// ---------------------------------------------------------------------------

/**
 * 三证齐了才允许并线（GIT-7）：
 *   1. 本任务全部提交的凭据扫描通过；
 *   2. 干净检出里装得上、跑得起来、测得过；
 *   3. 有一次跳出本机的独立验证跑过。
 * 三项各自独立求值，缺哪一项报哪一项；三项都齐时必须放行，不得再加条件。
 */
function evaluateIntegrationReadiness(input) {
  const settings = input || {};
  const credentials = settings.credentialScan
    || scanCommitRangeForCredentials({
      cwd: settings.cwd,
      baseOid: settings.baseOid,
      headRef: settings.headRef,
      commits: settings.commits,
      pathsOfCommit: settings.pathsOfCommit,
    });
  const cleanCheckout = requireCleanCheckoutVerification(settings);
  const offMachine = requireOffMachineVerification(settings);
  const serialization = refuseSecondIntegration(settings.integrationRef, settings.integrationsInFlight);

  const problems = [];
  if (!credentials.ok) problems.push(credentials.refusal);
  problems.push(...cleanCheckout.problems);
  problems.push(...offMachine.problems);
  if (!serialization.ok) problems.push(serialization.refusal);

  return {
    allowed: problems.length === 0,
    proofs: {
      credentialScanOverAllCommits: credentials,
      cleanCheckoutVerification: cleanCheckout,
      offMachineVerification: offMachine,
    },
    serialization,
    problems,
  };
}

module.exports = {
  NEVER_COMMIT_PATTERNS,
  PRODUCT_DEPENDENCY_PATTERNS,
  CLEAN_CHECKOUT_RECORD_FIELDS,
  OFF_MACHINE_RECORD_FIELDS,
  INTEGRATION_PHASES,
  INTEGRATION_IN_PROGRESS_PHASE,
  integrationIsInProgress,
  integrationInProgressOn,
  refuseSecondIntegration,
  matchNeverCommit,
  matchProductDependency,
  taskCommitList,
  verifyWipCommits,
  scanCommitRangeForCredentials,
  requireCleanCheckoutVerification,
  requireOffMachineVerification,
  auditIgnoreRules,
  auditResidue,
  auditUntrackedProductDependencies,
  evaluateIntegrationReadiness,
};
