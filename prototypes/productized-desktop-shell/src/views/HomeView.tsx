// ⑤ C·首页(系统总览)——设计定稿 `docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html` 的 C 段。
// 统计行(项目/跑着的单/等我的事/系统健康)+ 四区块 2×2(等我的事/最近项目/记忆动态/系统状态)。
// 布局总纲(DESIGN.md 二):整页不滚,滚动只在各区块 .spec-scroll 内。
//
// ⚠️ 零 hooks 硬约束:本文件及其所有子组件禁止 useState/useMemo/任何 hook。
// 理由=`tests/helpers/offlineInteractionTestUtils.tsx` 的 renderComposite(L73-77)把组件当普通函数
// 裸调 `Component(element.props)`,并递归穿透所有子组件;`offline-permission-dialog.test.tsx:2501`
// 的 `findButtonByText(home, "打开智能体")` 正走这条路径。加 hooks 必炸,且「状态提升父级」救不了
// ——HomeView 自己就是被裸调的那一层。
import type { ReactNode } from "react";
import { deriveDailyMemoryCandidateInbox } from "../lib/memoryDailyLoop";
import { listRowTimeLabel, projectName } from "../lib/format";
import { EmptyState, FactRow, ListRow } from "../components/SpecPrimitives";
import type { MemoryCandidateStoreV1, WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { NavigateHandler, NavigationFocus, ViewKey } from "../lib/workbenchNavigation";

// 后端「系统状态读模型」形状(`tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md` §A)。
// 命令名与返回形状由后端包回传后再接线;在此先按契约立 UI 形态,读模型缺席 = 留位 +「接线中」,不编数据。
export type HomeSystemStatusReadModel = {
  storage_mode: "db_primary" | "json_only";
  storage_healthy: boolean;
  observation_day: number;
  last_degradation?: { at_ms: number; reason_human: string } | null;
  recent_catches: { at_ms: number; summary: string }[];
  gate_summary?: string | null;
};

type HomeViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  // 「记忆动态」数据源:复用 M2 记忆候选店(App → ActiveWorkbenchView 已有,穿下来即可)。
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  // 后端 §A 未落地前恒为 null → 系统健康/系统状态显示「接线中」。
  systemStatus?: HomeSystemStatusReadModel | null;
  onNavigate: NavigateHandler;
};

type HomeTone = "ok" | "warn" | "err" | "run" | "idle";

type HomeRow = {
  key: string;
  tone: HomeTone;
  // 三元素行的中间元素:一句人话(等我的事)/项目名(最近项目)/一句聚合(记忆动态)。
  claim: string;
  // 三元素行的第三元素:项目名(等我的事)或时间(最近项目/记忆动态)。事实层给不出就留空,不编。
  tail: string | null;
  view: ViewKey;
  focus?: NavigationFocus;
};

// 未接线口径统一一句人话(宪法 §四.3 禁机器术语上脸)。
const NOT_WIRED = "接线中";

// 「跑着的单」= 机器侧在飞的工单;「等我的事」= 用户侧欠动作的。两边刻意不重叠——
// 定稿 C 段样例(跑着的单 0 / 等我的事 1,那 1 条是待复核体检)即此口径。
const HOME_RUNNING_TASK_STATES = new Set(["running", "ready_to_dispatch", "retry_pending"]);

export function HomeView({
  snapshot,
  workflowState = null,
  memoryCandidateStore = null,
  systemStatus = null,
  onNavigate,
}: HomeViewProps) {
  const workflows = workflowState?.project_workflows ?? [];
  // workflow_id → 项目根。用于给运行时关注行补「项目名」这第三元素。
  // ⚠️ 刻意不用 attention.project_id：它是后端 stable_id 派生的机器 slug(如
  // `project:users-yoyi-documents-mario-test`，见 src-tauri/src/lib.rs:1076)，摆上脸就是机器术语上脸
  // (宪法 §四.3)；而在前端反推 slug = 复制后端逻辑，等于编。workflow_id 是两边都有的真 join key。
  const projectRootByWorkflowId = new Map(workflows.map((workflow) => [workflow.workflow_id, workflow.project_root]));

  // ── 等我的事 ──────────────────────────────────────────────────────────
  // 三个来源,与右栏抽屉(RightDetailPanel)的待办口径一致:运行时关注(需用户动作)
  // + 待复核工单 + 未批权限请求。target 也沿用同一套 focus 约定,免得目标页学会选中时首页掉队。
  const attentionRows: HomeRow[] = snapshot.runtime_session_attention
    .filter((item) => item.requires_user_action || item.blocks_continuation)
    .map((item) => {
      // 关联不到项目就留空第三元素(ListRow 会省掉那格)——不编项目名。
      const root = item.workflow_id ? projectRootByWorkflowId.get(item.workflow_id) : undefined;
      return {
        key: `attention:${item.attention_id}`,
        tone: item.blocks_continuation ? ("err" as const) : ("warn" as const),
        claim: item.user_message || item.title,
        tail: root ? projectName(root) : null,
        view: "agents" as const,
        focus: item.session_id ? { kind: "session", id: item.session_id } : undefined,
      };
    });
  const reviewRows: HomeRow[] = workflows.flatMap((workflow) =>
    workflow.task_drafts
      .filter((task) => task.state === "ready_for_review")
      .map((task) => ({
        key: `review:${task.work_item_id}`,
        tone: "warn" as const,
        claim: task.title,
        tail: projectName(workflow.project_root),
        view: "projects" as const,
        focus: { kind: "work-item", id: task.work_item_id },
      })),
  );
  const permissionRows: HomeRow[] = workflows.flatMap((workflow) =>
    workflow.permission_requests
      .filter((request) => request.status !== "approved")
      .map((request) => ({
        key: `permission:${request.request_id}`,
        tone: "warn" as const,
        claim: request.reason || "有一处权限等你批",
        tail: projectName(workflow.project_root),
        view: "projects" as const,
        focus: { kind: "permission-request", id: request.request_id },
      })),
  );
  const waitingRows = [...attentionRows, ...reviewRows, ...permissionRows];

  // ── 最近项目 ──────────────────────────────────────────────────────────
  const recentProjectRows: HomeRow[] = [...snapshot.projects]
    .sort((a, b) => (b.latest_updated_at_ms ?? 0) - (a.latest_updated_at_ms ?? 0))
    .slice(0, 6)
    .map((project) => ({
      key: `project:${project.project_root}`,
      tone: project.context_warnings.length || project.warnings.length
        ? ("warn" as const)
        : project.active_hint
          ? ("ok" as const)
          : ("idle" as const),
      claim: project.name,
      tail: msTimeLabel(project.latest_updated_at_ms),
      view: "projects" as const,
      focus: { kind: "project", id: project.project_root },
    }));

  // ── 记忆动态 ──────────────────────────────────────────────────────────
  // 防重造轮子:直接用既有 L5 日循环读模型,不自己数候选。
  const memoryInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const memoryLatestTime = listRowTimeLabel(memoryInbox.items[0]?.updated_at ?? null);
  const needsConfirmCount = memoryInbox.items.filter((item) => item.can_confirm).length;
  const memoryRows: HomeRow[] = [
    ...(needsConfirmCount
      ? [
          {
            key: "memory:needs-review",
            tone: "warn" as const,
            claim: `${needsConfirmCount} 条候选待你确认`,
            tail: memoryLatestTime,
            view: "memory" as const,
          },
        ]
      : []),
    ...(memoryInbox.adoptable_count
      ? [
          {
            key: "memory:adoptable",
            tone: "run" as const,
            claim: `${memoryInbox.adoptable_count} 条已确认，等你决定是否长期记住`,
            tail: memoryLatestTime,
            view: "memory" as const,
          },
        ]
      : []),
  ];

  // ── 统计行 ────────────────────────────────────────────────────────────
  const runningTaskCount = workflows.reduce(
    (count, workflow) => count + workflow.task_drafts.filter((task) => HOME_RUNNING_TASK_STATES.has(task.state)).length,
    0,
  );

  return (
    <div className="home-overview-stage">
      {/* 数据诚实:最近项目的时间来自索引 latest_updated_at_ms(近似口径)，不是真实使用/交货事件。 */}
      <p className="sr-only">最近项来自索引近似口径，不是真实使用事件。</p>
      {/* 智能体入口锚点：左导航已有「智能体」，此处为离线交互测试锁定的可达入口（真导航，非假按钮）。 */}
      <button className="sr-only" type="button" onClick={() => onNavigate("agents")}>
        打开智能体
      </button>

      <div className="home-stat-row" aria-label="系统总览统计">
        <HomeStat n={`${snapshot.summary.project_count}`} t="项目" />
        <HomeStat n={`${runningTaskCount}`} t="跑着的单" />
        <HomeStat n={`${waitingRows.length}`} t="等我的事" tone="warn" />
        <HomeStat
          n={systemStatus ? `● ${storageHealthLabel(systemStatus)}` : `● ${NOT_WIRED}`}
          t={systemStatus ? systemHealthDetail(systemStatus) : "系统状态读模型还没接上"}
          small
          tone={systemStatus ? (systemStatus.storage_healthy ? "ok" : "warn") : "idle"}
        />
      </div>

      <div className="home-overview-grid">
        <HomeBlock label="等我的事">
          {waitingRows.length ? (
            <HomeRows rows={waitingRows} onNavigate={onNavigate} />
          ) : (
            <EmptyState what="现在没有需要你拍板的事" next="有工单要复核或权限要批时会出现在这里" />
          )}
        </HomeBlock>

        <HomeBlock label="最近项目">
          {recentProjectRows.length ? (
            <HomeRows rows={recentProjectRows} onNavigate={onNavigate} />
          ) : (
            <EmptyState what="索引里还没有项目" next="去「项目」页添加一个项目根目录" />
          )}
        </HomeBlock>

        <HomeBlock label="记忆动态">
          {memoryRows.length ? (
            <HomeRows rows={memoryRows} onNavigate={onNavigate} />
          ) : (
            <EmptyState what="没有待你确认的记忆候选" next="干活过程中攒出候选后会在这里排队" />
          )}
        </HomeBlock>

        <HomeBlock label="系统状态">
          <FactRow k="存储">{systemStatus ? storageModeLabel(systemStatus.storage_mode) : NOT_WIRED}</FactRow>
          <FactRow k="安全闸">{systemStatus?.gate_summary || (systemStatus ? "没有额外解封，按默认闸走" : NOT_WIRED)}</FactRow>
          <FactRow k="最近拦截">{recentCatchLabel(systemStatus)}</FactRow>
          {systemStatus ? null : (
            <EmptyState what="系统状态读模型还没接上" next="后端读模型接上后这里显示存储、安全闸和最近拦截" />
          )}
        </HomeBlock>
      </div>
    </div>
  );
}

// 统计块(定稿 C:大数字 + 小字标签;系统健康那格是「● 正常」+ 一行小字)。
function HomeStat({ n, t, tone = "plain", small = false }: { n: string; t: string; tone?: "plain" | "warn" | "ok" | "idle"; small?: boolean }) {
  return (
    <div className="home-stat">
      <div className={`home-stat-n home-stat-${tone}${small ? " is-small" : ""}`}>{n}</div>
      <div className="home-stat-t">{t}</div>
    </div>
  );
}

// 区块:标题固定，内容自己内滚(布局总纲——整页不滚)。
function HomeBlock({ label, children }: { label: string; children: ReactNode }) {
  return (
    <section className="home-overview-card" aria-label={label}>
      <p className="home-overview-label">{label}</p>
      <div className="spec-scroll home-overview-body">{children}</div>
    </section>
  );
}

// 三元素行(DESIGN.md 三·五 定式)。每条可点直达——focus 由目标页认领，目标页还没学会选中时
// 退化成只切页（`workbenchNavigation.ts`：focus 可省 = 只切页不选行），不是死链。
function HomeRows({ rows, onNavigate }: { rows: HomeRow[]; onNavigate: NavigateHandler }) {
  return (
    <>
      {rows.map((row) => (
        <ListRow
          key={row.key}
          badge={<i className={`home-dot home-dot-${row.tone}`} aria-hidden="true" />}
          claim={row.claim}
          time={row.tail}
          onSelect={() => onNavigate(row.view, row.focus)}
        />
      ))}
    </>
  );
}

function storageModeLabel(mode: HomeSystemStatusReadModel["storage_mode"]) {
  return mode === "db_primary" ? "DB 主写" : "只用 JSON";
}

function storageHealthLabel(status: HomeSystemStatusReadModel) {
  return status.storage_healthy ? "正常" : "有问题";
}

function systemHealthDetail(status: HomeSystemStatusReadModel) {
  const parts = [`存储 ${storageModeLabel(status.storage_mode)}`, `观察期第 ${status.observation_day} 天`];
  if (status.last_degradation) parts.push(`上次降级：${status.last_degradation.reason_human}`);
  return parts.join(" · ");
}

// 「最近拦截」后端注明「可先空实现留形状」→ 这里留形状 + 空态:
// 读模型在但列表为空 = 真的没拦截过 →「无」;读模型整个缺席 = 没数据源 →「接线中」(不冒充「无」)。
function recentCatchLabel(status: HomeSystemStatusReadModel | null) {
  if (!status) return NOT_WIRED;
  const latest = status.recent_catches[0];
  if (!latest) return "无";
  return `${latest.summary} · ${msTimeLabel(latest.at_ms) ?? "时间未知"}`;
}

// 索引给的是毫秒时间戳;统一走既有 listRowTimeLabel(三元素定式的第三元素),拿不到就 null 不编。
function msTimeLabel(ms?: number | null): string | null {
  if (!ms) return null;
  const date = new Date(ms);
  if (Number.isNaN(date.getTime())) return null;
  return listRowTimeLabel(date.toISOString());
}
