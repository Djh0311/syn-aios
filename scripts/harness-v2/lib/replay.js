'use strict';

// Adaptive Harness v0.5 — 受确认的重放：被抢先并线之后的继续路径（AH-050-07）
//
// 需求溯源：
//   GIT-3  前一个任务并进基线之后，后一个必须有一条正常路径把自己搬到新基线上
//          继续做完（整理、压缩、重放都算）。可以要求单独确认，**但不能变成死路**。
//          Harness 自己不擅自改写历史。
//   §6.5   必须存在至少一条无需用户手工执行 Git 的命令路径；确认之后不得返回
//          「无可用出口」或停在 BLOCKED 终态；必须记录搬之前与之后的 OID、保留原 ref；
//          **绝不改写已被集成基线包含的历史**。
//   §6.4   基线漂移只提示、不做硬门：不得因基线落后而拒绝开工、提交或进入等待集成。
//
// 本模块**只产出计划并做前置校验，不执行任何 Git 写操作**。
// steps 里的命令交给调用方在取得用户单独确认之后执行；本文件用的全是只读查询。

const gitFacts = require('./git-facts');

// 四条出口。**这份清单永远不会是空的**——快进不可能时还有重放，
// 重放会碰到已集成历史时还有「把新基线并进来再快进」。
// 「停下」不是出口，它是需求明令禁止的死路。
const CONTINUATION_ROUTES = Object.freeze([
  'ALREADY_INTEGRATED',
  'FAST_FORWARD',
  'REPLAY_ONTO_NEW_BASE',
  'MERGE_NEW_BASE_FORWARD',
]);

class ReplayInvariantError extends Error {
  constructor(message) {
    super(message);
    this.name = 'ReplayInvariantError';
  }
}

function shortOid(value) {
  const text = String(value === null || value === undefined ? '' : value);
  return text.length > 12 ? text.slice(0, 12) : text;
}

// ---------------------------------------------------------------------------
// 现实：基线漂到哪了
// ---------------------------------------------------------------------------

function resolveRef(cwd, ref) {
  const result = gitFacts.runGit(['rev-parse', '--verify', `${ref}^{commit}`], { cwd });
  return result.ok ? result.stdout.trim() : null;
}

function commitsInRange(cwd, fromOid, toRef) {
  const result = gitFacts.runGit(['log', '--format=%H', `${fromOid}..${toRef}`], { cwd });
  if (!result.ok) return [];
  return result.stdout.split('\n').map((line) => line.trim()).filter((line) => line !== '');
}

/**
 * 读一次现实：任务分支现在停在哪、集成基线现在停在哪、快进还成不成立。
 *
 * 快进不可能（`FAST_FORWARD_NOT_POSSIBLE`）在旧实现里就是终点。
 * 这里它只是一个事实，不是结论——结论由 continuationRoutes 给。
 */
function assessBaseline(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const taskBranch = settings.taskBranch;
  const baseOid = settings.baseOid;
  const integrationRef = settings.integrationRef || 'refs/heads/main';

  const taskHead = resolveRef(cwd, taskBranch);
  const integrationHead = resolveRef(cwd, integrationRef);
  const facts = {
    cwd,
    taskBranch,
    baseOid,
    integrationRef,
    taskHead,
    integrationHead,
    readable: Boolean(taskHead && integrationHead),
    baselineMoved: false,
    fastForwardPossible: false,
    taskCommits: [],
    alreadyIntegratedTaskCommits: [],
  };
  if (!facts.readable) return facts;

  // 快进成立 ⟺ 集成基线仍是任务分支的祖先。
  facts.fastForwardPossible = gitFacts.isAncestor(cwd, integrationHead, taskBranch);
  // 基线前进过 ⟺ 冻结的 base 已经不是集成分支的顶端。
  facts.baselineMoved = integrationHead !== baseOid && gitFacts.isAncestor(cwd, baseOid, integrationRef);
  facts.taskCommits = commitsInRange(cwd, baseOid, taskBranch);
  // 本任务的提交里，哪些已经被集成基线包含。这批是**绝对不能搬、不能改写**的那批。
  facts.alreadyIntegratedTaskCommits = facts.taskCommits
    .filter((oid) => gitFacts.isAncestor(cwd, oid, integrationRef));
  return facts;
}

// ---------------------------------------------------------------------------
// 出口
// ---------------------------------------------------------------------------

/**
 * 给出全部可走的继续路径。返回值**永不为空**：
 * 出口清单为空意味着我们把需求禁止的死路又造了一遍，那是实现缺陷，当场抛错。
 *
 * 判据顺序是「代价从小到大」，不是「谁排在前面谁优先」：
 *   * 本任务提交已经全被基线包含   → 无事可搬，直接收尾；
 *   * 集成基线仍是任务分支的祖先   → 普通快进；
 *   * 分叉了、且要搬的提交都没被基线包含 → 受确认的重放（整理 / 压缩 / 换基线均可）；
 *   * 分叉了、但有提交已被基线包含 → 改用把新基线并进来再快进，
 *     因为重放它们等于改写已被集成基线包含的历史，那条线绝对不许走。
 */
function continuationRoutes(facts) {
  const routes = [];
  if (!facts || facts.readable !== true) {
    return [{
      route: 'MERGE_NEW_BASE_FORWARD',
      requiresConfirmation: true,
      why: '读不到分支或基线的当前位置；把新基线并进任务分支再快进这条路不依赖历史改写，任何情况下都走得通',
    }];
  }
  if (facts.taskCommits.length > 0
    && facts.alreadyIntegratedTaskCommits.length === facts.taskCommits.length) {
    routes.push({
      route: 'ALREADY_INTEGRATED',
      requiresConfirmation: false,
      why: '本任务的提交已全部被集成基线包含，直接进收尾即可',
    });
  }
  if (facts.fastForwardPossible) {
    routes.push({
      route: 'FAST_FORWARD',
      requiresConfirmation: true,
      why: '集成基线仍是任务分支的祖先，本地快进即可，不需要搬任何东西',
    });
  }
  if (!facts.fastForwardPossible && facts.alreadyIntegratedTaskCommits.length === 0) {
    routes.push({
      route: 'REPLAY_ONTO_NEW_BASE',
      requiresConfirmation: true,
      why: '基线被抢先推进、快进不成立；把本任务这几笔提交重放到新基线上继续做完',
    });
  }
  routes.push({
    route: 'MERGE_NEW_BASE_FORWARD',
    requiresConfirmation: true,
    why: '把新基线并进任务分支再快进。它不改写任何已有提交，因此在「有提交已被基线包含」时也成立',
  });
  if (routes.length === 0) {
    throw new ReplayInvariantError('继续路径清单为空——这是实现缺陷：GIT-3 明令这里不得成为死路');
  }
  return routes;
}

// ---------------------------------------------------------------------------
// 前置校验：绝不改写已被集成基线包含的历史
// ---------------------------------------------------------------------------

/**
 * 重放前置校验。**这条要拦得住**：要搬的那批提交里只要有一笔已经被集成基线包含，
 * 重放就会改写已并入基线的历史，直接拒绝这条路线并改走 MERGE_NEW_BASE_FORWARD。
 *
 * 同时要求新基线确实包含旧 base——否则「搬到新基线」搬的不是同一条线。
 */
function precheckReplay(facts) {
  const blockers = [];
  if (!facts || facts.readable !== true) {
    blockers.push({
      code: 'REPLAY_REALITY_UNREADABLE',
      message: '读不到任务分支或集成基线的位置，拒绝在不明现实上重放',
    });
    return { ok: false, blockers };
  }
  if (facts.alreadyIntegratedTaskCommits.length > 0) {
    blockers.push({
      code: 'REPLAY_WOULD_REWRITE_INTEGRATED_HISTORY',
      message: '以下提交已被集成基线包含，重放会改写已并入基线的历史，拒绝：'
        + facts.alreadyIntegratedTaskCommits.map(shortOid).join(', '),
      commits: facts.alreadyIntegratedTaskCommits.slice(),
    });
  }
  if (facts.taskCommits.length === 0) {
    blockers.push({
      code: 'REPLAY_NOTHING_TO_MOVE',
      message: '基线之上没有本任务的提交，没有需要搬的东西',
    });
  }
  return { ok: blockers.length === 0, blockers };
}

// ---------------------------------------------------------------------------
// 计划
// ---------------------------------------------------------------------------

/**
 * 产出一份受确认的继续计划。
 *
 * 三件事写死在计划里：
 *   * 搬之前与搬之后的 OID 记录点（postOid 由调用方执行完回填）；
 *   * 保留原 ref——搬之前的分支顶端另存一份，随时能回到原状；
 *   * 每一步的原始命令。**本模块不执行它们**：经用户单独确认之后由调用方执行，
 *     那属于需要授权的集成动作，不属于自动改写历史。
 *
 * 确认之后**不得返回「无可用出口」**：选中的路线做不了时，函数会自动落到
 * 不改写历史的那条路线上，返回值里永远有一条可走的 route。
 */
function planConfirmedReplay(input) {
  const settings = input || {};
  const facts = settings.facts || assessBaseline(settings);
  const taskId = settings.taskId || 'TASK';
  const confirmed = settings.confirmed === true;
  const routes = continuationRoutes(facts);
  const precheck = precheckReplay(facts);

  const requested = settings.route || null;
  const fellBackBecause = [];
  let selected = requested && routes.some((entry) => entry.route === requested)
    ? routes.find((entry) => entry.route === requested)
    : routes[0];
  // 点名了一条走不通的路线，就把走不通的原因逐条记下来再落到别的路线上。
  // 「点了名却拿不到，也说不出为什么」等同于死路。
  if (requested && selected.route !== requested) {
    fellBackBecause.push(...precheck.blockers);
    if (fellBackBecause.length === 0) {
      fellBackBecause.push({
        code: 'ROUTE_NOT_APPLICABLE',
        message: `点名的路线 ${requested} 在当前现实下不适用，改走 ${selected.route}`,
      });
    }
  }
  if (selected.route === 'REPLAY_ONTO_NEW_BASE' && !precheck.ok) {
    fellBackBecause.push(...precheck.blockers);
    selected = routes.find((entry) => entry.route === 'MERGE_NEW_BASE_FORWARD');
  }
  if (!selected) {
    throw new ReplayInvariantError('选不出继续路径——确认之后返回「无可用出口」是 GIT-3 明令禁止的');
  }

  const preOid = facts.taskHead || null;
  const preservedRef = `refs/harness-v2/pre-continuation/${taskId}/${shortOid(preOid)}`;
  const steps = buildSteps(selected.route, facts, preservedRef);

  return {
    route: selected.route,
    why: selected.why,
    routes,
    requiresConfirmation: selected.requiresConfirmation,
    confirmed,
    // 确认之后一定给得出出口；这里恒为 true，不存在「确认了却无路可走」的返回值。
    hasExit: true,
    fellBackBecause,
    precheck,
    recordPoints: {
      // 搬之前的位置。
      preOid,
      // 搬到哪个基线上。
      newBaseOid: facts.integrationHead || null,
      // 搬之后的位置：由调用方执行完写回，写不回来就不算搬完。
      postOid: settings.postOid || null,
    },
    // 原 ref 保留：搬之前先给分支顶端另存一个只读引用，随时回得去。
    preservedRef,
    steps,
    // 执行属于需要授权的集成动作。本模块只出计划，一条写命令都不跑。
    executedHere: false,
  };
}

function buildSteps(route, facts, preservedRef) {
  const base = [{
    order: 1,
    intent: 'PRESERVE_ORIGINAL_REF',
    command: `git update-ref ${preservedRef} ${facts.taskHead || '<task-head>'}`,
    note: '保留原 ref：搬之前的分支顶端先另存一份',
  }];
  if (route === 'ALREADY_INTEGRATED') {
    return [{
      order: 1,
      intent: 'NOTHING_TO_MOVE',
      command: '（无需 Git 写操作）',
      note: '本任务的提交已全部被基线包含，直接进收尾',
    }];
  }
  if (route === 'FAST_FORWARD') {
    return base.concat([{
      order: 2,
      intent: 'FAST_FORWARD_ONLY',
      command: `git merge --ff-only ${facts.taskBranch}`,
      note: '在干净的专用 integration 工作副本里执行，只接受快进',
    }]);
  }
  if (route === 'REPLAY_ONTO_NEW_BASE') {
    return base.concat([
      {
        order: 2,
        intent: 'REPLAY_ONTO_NEW_BASE',
        command: `git rebase --onto ${facts.integrationHead || '<integration-head>'} ${facts.baseOid} ${facts.taskBranch}`,
        note: '把本任务这几笔提交搬到新基线上；只搬未被基线包含的那批',
      },
      {
        order: 3,
        intent: 'RECORD_POST_OID',
        command: `git rev-parse ${facts.taskBranch}`,
        note: '记录搬之后的 OID，与 preOid 一起入档',
      },
      {
        order: 4,
        intent: 'FAST_FORWARD_ONLY',
        command: `git merge --ff-only ${facts.taskBranch}`,
        note: '搬完之后快进并线，收尾',
      },
    ]);
  }
  return base.concat([
    {
      order: 2,
      intent: 'MERGE_NEW_BASE_FORWARD',
      command: `git merge --no-ff ${facts.integrationRef}`,
      note: '把新基线并进任务分支。不改写任何已有提交，已被基线包含的历史原样保留',
    },
    {
      order: 3,
      intent: 'RECORD_POST_OID',
      command: `git rev-parse ${facts.taskBranch}`,
      note: '记录搬之后的 OID，与 preOid 一起入档',
    },
    {
      order: 4,
      intent: 'FAST_FORWARD_ONLY',
      command: `git merge --ff-only ${facts.taskBranch}`,
      note: '并完之后快进并线，收尾',
    },
  ]);
}

/**
 * 基线漂移只提示、不做硬门（§6.4）：落后多少都不构成拒绝开工、拒绝提交、
 * 或拒绝进入等待集成的理由。这里只给一条指向继续路径的提示。
 */
function baselineDriftAdvisory(facts) {
  if (!facts || facts.readable !== true) return null;
  if (facts.fastForwardPossible) return null;
  return {
    code: 'BASELINE_MOVED_CONTINUATION_AVAILABLE',
    advisory: true,
    message: `基线已前进到 ${shortOid(facts.integrationHead)}，快进不再成立；`
      + `继续路径见 ${continuationRoutes(facts).map((entry) => entry.route).join(' / ')}，需要单独确认后执行`,
  };
}

module.exports = {
  CONTINUATION_ROUTES,
  ReplayInvariantError,
  assessBaseline,
  continuationRoutes,
  precheckReplay,
  planConfirmedReplay,
  baselineDriftAdvisory,
};
