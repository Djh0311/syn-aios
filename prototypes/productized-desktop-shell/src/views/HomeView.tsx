// M4C06 Secretary homepage.
//
// Hard constraint: this module intentionally has no React hooks. The offline
// interaction harness may evaluate it as a plain function, and all transport /
// action state therefore belongs to App. The only truth rendered here is the
// typed M4 Secretary home read model supplied by that host.
import type { FormEvent, ReactNode } from "react";
import { EmptyState, FactRow, ListRow } from "../components/SpecPrimitives";
import { listRowTimeLabel, projectName } from "../lib/format";
import { deriveDailyMemoryCandidateInbox } from "../lib/memoryDailyLoop";
import type {
  M4SecretaryCoordinationActionCode,
  M4SecretaryPersonalObjectActionCode,
  M4SecretaryPersonalObjectRequestDto,
  SecretaryHomeAttentionItem,
  SecretaryHomeReadModel,
  SecretaryTypedDeepLinkDescriptor,
} from "../lib/types/m4Secretary";
import type { MemoryCandidateStoreV1, WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { NavigateHandler, NavigationFocus, ViewKey } from "../lib/workbenchNavigation";

// Compatibility export consumed by SettingsView. M4C06 no longer uses this
// read model to construct the homepage, but keeping its exact public shape
// avoids changing the unrelated settings surface.
export type HomeSystemStatusReadModel = {
  storage_mode: "db_primary" | "json_only";
  storage_healthy: boolean;
  observation_day: number;
  last_degradation?: { at_ms: number; reason_human: string } | null;
  recent_catches: { at_ms: number; summary: string }[];
  gate_summary?: string | null;
  warnings: string[];
};

export type SecretaryCoordinationIntent = Readonly<{
  item: SecretaryHomeAttentionItem;
  action: M4SecretaryCoordinationActionCode;
  snoozed_until_utc?: string;
}>;

export type SecretaryCoordinationViewState =
  | Readonly<{ phase: "pending"; action: M4SecretaryCoordinationActionCode }>
  | Readonly<{
      phase: "succeeded";
      action: M4SecretaryCoordinationActionCode;
      command_receipt_ref: string;
      outcome_code: string;
    }>
  | Readonly<{ phase: "failed"; action: M4SecretaryCoordinationActionCode; error_code: string }>;

type StripIdempotency<T> = T extends Readonly<{ idempotency_key: string }>
  ? Omit<T, "idempotency_key">
  : never;

export type SecretaryPersonalObjectIntent = StripIdempotency<M4SecretaryPersonalObjectRequestDto>;

export type SecretaryPersonalObjectViewState =
  | Readonly<{ phase: "pending"; action: M4SecretaryPersonalObjectActionCode }>
  | Readonly<{
      phase: "succeeded";
      action: M4SecretaryPersonalObjectActionCode;
      command_receipt_ref: string;
      outcome_code: string;
    }>
  | Readonly<{ phase: "failed"; action: M4SecretaryPersonalObjectActionCode; error_code: string }>;

type HomeViewProps = {
  // M4C06 path. App always supplies this authoritative projection.
  secretaryHome?: SecretaryHomeReadModel;
  presentationState?: "loading" | "error" | null;
  coordinationStates?: Readonly<Record<string, SecretaryCoordinationViewState>>;
  personalObjectStates?: Readonly<Record<string, SecretaryPersonalObjectViewState>>;
  onOperateCoordination?: (intent: SecretaryCoordinationIntent) => void;
  onOperatePersonalObject?: (intent: SecretaryPersonalObjectIntent) => void;
  onOpenDeepLink?: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
  onReloadSecretaryHome?: () => void;
  // Compatibility-only fields keep the pre-M4 render dispatcher type-safe.
  // They are deliberately never used to reconstruct a Secretary identity,
  // scope, context, source owner, or coordination record.
  snapshot?: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  systemStatus?: HomeSystemStatusReadModel | null;
  onNavigate?: NavigateHandler;
};

type SecretaryHomeAction = Readonly<{
  action: M4SecretaryCoordinationActionCode;
  label: string;
  usesSnoozeTime?: boolean;
}>;

const EMPTY_COORDINATION_STATES: Readonly<Record<string, SecretaryCoordinationViewState>> = Object.freeze({});
const EMPTY_PERSONAL_OBJECT_STATES: Readonly<Record<string, SecretaryPersonalObjectViewState>> = Object.freeze({});

// This is a visibly loading compatibility shell only. It contains no identity,
// role, context, scope, attention item or source truth.
const LOADING_HOME: SecretaryHomeReadModel = Object.freeze({
  schema_version: "syn.m4.secretary.home.v1",
  state: "loading",
  source_authority: "NONE",
  context: null,
  deterministic_brief: null,
  scope_source_watermark: null,
  role_session_recovery: Object.freeze({
    status: "LOADING",
    role_session_ref: null,
    context_ref: null,
    recovery_code: null,
  }),
  attention_items: Object.freeze([]),
  personal_actions: Object.freeze([]),
  local_objects: Object.freeze({
    personal_actions: Object.freeze([]),
    notifications: Object.freeze([]),
    reminders: Object.freeze([]),
    decisions: Object.freeze([]),
    reminder_owner_refs: Object.freeze([]),
  }),
  module_entries: Object.freeze([]),
  model_enhancement: Object.freeze({
    status: "NOT_REQUESTED",
    invocation_ref: null,
    enhancement_ref: null,
    enhancement_hash: null,
    invocation_receipt: null,
    recovery_code: null,
  }),
  handoff: Object.freeze({
    status: "NOT_LOADED",
    handoff_ref: null,
    request_receipt_ref: null,
    returned_receipt: null,
    recovery_code: null,
  }),
  degradation_code: null,
});

export function HomeView({
  secretaryHome,
  presentationState = null,
  coordinationStates = EMPTY_COORDINATION_STATES,
  personalObjectStates = EMPTY_PERSONAL_OBJECT_STATES,
  onOperateCoordination,
  onOperatePersonalObject,
  onOpenDeepLink,
  onReloadSecretaryHome,
  snapshot,
  workflowState = null,
  memoryCandidateStore = null,
  systemStatus = null,
  onNavigate,
}: HomeViewProps) {
  if (!secretaryHome && snapshot && onNavigate) {
    return (
      <LegacyHomeView
        snapshot={snapshot}
        workflowState={workflowState}
        memoryCandidateStore={memoryCandidateStore}
        systemStatus={systemStatus}
        onNavigate={onNavigate}
      />
    );
  }
  const home = secretaryHome ?? LOADING_HOME;
  const state = presentationState ?? home.state;

  if (state === "loading") return <SecretaryHomeLoading />;
  if (state === "error") return <SecretaryHomeFailure home={home} onReload={onReloadSecretaryHome} />;
  if (home.state === "degraded") return <SecretaryHomeDegraded home={home} onReload={onReloadSecretaryHome} />;

  return (
    <main className="secretary-home" data-secretary-home-state={home.state} aria-labelledby="secretary-home-title">
      <header className="secretary-home-head">
        <div>
          <p className="eyebrow">持续 Secretary</p>
          <h1 id="secretary-home-title">现在要看住什么</h1>
          <p className="secretary-home-lede">每一条都保留原因、当前状态、负责人和回源入口；不在这里改写来源事实。</p>
        </div>
        <SecretaryContextStamp home={home} />
      </header>

      {home.state === "empty" ? (
        <section className="secretary-home-empty" aria-label="秘书情境为空">
          <strong>当前情境很干净。</strong>
          <p>已恢复同一 Secretary 情境，但没有需要持续看住的关注项或个人行动。</p>
          <SecretaryPersonalObjects
            home={home}
            states={personalObjectStates}
            onOperate={onOperatePersonalObject}
          />
          <SecretaryAvailability home={home} />
        </section>
      ) : (
        <div className="secretary-home-layout">
          <section className="secretary-attention-region" aria-labelledby="secretary-attention-title">
            <div className="secretary-section-head">
              <div>
                <p className="eyebrow">source-backed attention</p>
                <h2 id="secretary-attention-title">持续关注</h2>
              </div>
              <span className="secretary-section-count" aria-label={`${home.attention_items.length} 条关注`}>
                {home.attention_items.length}
              </span>
            </div>
            {home.attention_items.length ? (
              <div className="secretary-attention-list">
                {home.attention_items.map((item) => (
                  <SecretaryAttentionSpine
                    key={item.item_ref}
                    item={item}
                    actionState={coordinationStates[item.item_ref]}
                    onOperate={onOperateCoordination}
                    onOpenDeepLink={onOpenDeepLink}
                  />
                ))}
              </div>
            ) : (
              <p className="secretary-inline-empty">当前没有来源型关注项。</p>
            )}
          </section>

          <aside className="secretary-home-secondary" aria-label="持续情境与边界">
            <SecretaryPersonalObjects
              home={home}
              states={personalObjectStates}
              onOperate={onOperatePersonalObject}
            />
            <SecretaryAvailability home={home} />
            <SecretaryModuleEntries home={home} onOpenDeepLink={onOpenDeepLink} />
          </aside>
        </div>
      )}
    </main>
  );
}

// Pre-M4 dispatcher compatibility for existing static views/tests. App's M4
// path always passes `secretaryHome` and never consumes this legacy projection.
type LegacyHomeTone = "ok" | "warn" | "err" | "run" | "idle";
type LegacyHomeRow = {
  key: string;
  tone: LegacyHomeTone;
  claim: string;
  tail: string | null;
  view: ViewKey;
  focus?: NavigationFocus;
};

const LEGACY_NOT_WIRED = "接线中";
const LEGACY_RUNNING_STATES = new Set(["running", "ready_to_dispatch", "retry_pending"]);

function LegacyHomeView({
  snapshot,
  workflowState = null,
  memoryCandidateStore = null,
  systemStatus = null,
  onNavigate,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  systemStatus?: HomeSystemStatusReadModel | null;
  onNavigate: NavigateHandler;
}) {
  const workflows = workflowState?.project_workflows ?? [];
  const projectRootByWorkflowId = new Map(workflows.map((workflow) => [workflow.workflow_id, workflow.project_root]));
  const attentionRows: LegacyHomeRow[] = snapshot.runtime_session_attention
    .filter((item) => item.requires_user_action || item.blocks_continuation)
    .map((item) => {
      const root = item.workflow_id ? projectRootByWorkflowId.get(item.workflow_id) : undefined;
      return {
        key: `attention:${item.attention_id}`,
        tone: item.blocks_continuation ? "err" : "warn",
        claim: item.user_message || item.title,
        tail: root ? projectName(root) : null,
        view: "agents",
        focus: item.session_id ? { kind: "session", id: item.session_id } : undefined,
      };
    });
  const reviewRows: LegacyHomeRow[] = workflows.flatMap((workflow) => workflow.task_drafts
    .filter((task) => task.state === "ready_for_review")
    .map((task) => ({
      key: `review:${task.work_item_id}`,
      tone: "warn" as const,
      claim: task.title,
      tail: projectName(workflow.project_root),
      view: "projects" as const,
      focus: { kind: "work-item" as const, id: task.work_item_id },
    })));
  const permissionRows: LegacyHomeRow[] = workflows.flatMap((workflow) => workflow.permission_requests
    .filter((request) => request.status !== "approved")
    .map((request) => ({
      key: `permission:${request.request_id}`,
      tone: "warn" as const,
      claim: request.reason || "有一处权限等你批",
      tail: projectName(workflow.project_root),
      view: "projects" as const,
      focus: { kind: "permission-request" as const, id: request.request_id },
    })));
  const waitingRows = [...attentionRows, ...reviewRows, ...permissionRows];
  const recentProjectRows: LegacyHomeRow[] = [...snapshot.projects]
    .sort((left, right) => (right.latest_updated_at_ms ?? 0) - (left.latest_updated_at_ms ?? 0))
    .slice(0, 6)
    .map((project) => ({
      key: `project:${project.project_root}`,
      tone: project.context_warnings.length || project.warnings.length ? "warn" : project.active_hint ? "ok" : "idle",
      claim: project.name,
      tail: legacyMsTimeLabel(project.latest_updated_at_ms),
      view: "projects",
      focus: { kind: "project", id: project.project_root },
    }));
  const memoryInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const memoryLatestTime = listRowTimeLabel(memoryInbox.items[0]?.updated_at ?? null);
  const memoryRows: LegacyHomeRow[] = [
    ...(memoryInbox.items.filter((item) => item.can_confirm).length ? [{
      key: "memory:needs-review", tone: "warn" as const,
      claim: `${memoryInbox.items.filter((item) => item.can_confirm).length} 条候选待你确认`,
      tail: memoryLatestTime, view: "memory" as const,
    }] : []),
    ...(memoryInbox.adoptable_count ? [{
      key: "memory:adoptable", tone: "run" as const,
      claim: `${memoryInbox.adoptable_count} 条已确认，等你决定是否长期记住`,
      tail: memoryLatestTime, view: "memory" as const,
    }] : []),
  ];
  const runningTaskCount = workflows.reduce(
    (count, workflow) => count + workflow.task_drafts.filter((task) => LEGACY_RUNNING_STATES.has(task.state)).length,
    0,
  );
  return (
    <div className="home-overview-stage">
      <p className="sr-only">最近项来自索引近似口径，不是真实使用事件。</p>
      <button className="sr-only" type="button" onClick={() => onNavigate("agents")}>打开智能体</button>
      <div className="home-stat-row" aria-label="系统总览统计">
        <LegacyHomeStat n={`${snapshot.summary.project_count}`} t="项目" />
        <LegacyHomeStat n={`${runningTaskCount}`} t="跑着的单" />
        <LegacyHomeStat n={`${waitingRows.length}`} t="等我的事" tone="warn" />
        <LegacyHomeStat
          n={systemStatus ? `● ${legacyStorageHealthLabel(systemStatus)}` : `● ${LEGACY_NOT_WIRED}`}
          t={systemStatus ? legacySystemHealthDetail(systemStatus) : "系统状态读模型还没接上"}
          small
          tone={systemStatus ? (systemStatus.storage_healthy ? "ok" : "warn") : "idle"}
        />
      </div>
      <div className="home-overview-grid">
        <LegacyHomeBlock label="等我的事">{waitingRows.length ? <LegacyHomeRows rows={waitingRows} onNavigate={onNavigate} /> : <EmptyState what="现在没有需要你拍板的事" next="有工单要复核或权限要批时会出现在这里" />}</LegacyHomeBlock>
        <LegacyHomeBlock label="最近项目">{recentProjectRows.length ? <LegacyHomeRows rows={recentProjectRows} onNavigate={onNavigate} /> : <EmptyState what="索引里还没有项目" next="去「项目」页添加一个项目根目录" />}</LegacyHomeBlock>
        <LegacyHomeBlock label="记忆动态">{memoryRows.length ? <LegacyHomeRows rows={memoryRows} onNavigate={onNavigate} /> : <EmptyState what="没有待你确认的记忆候选" next="干活过程中攒出候选后会在这里排队" />}</LegacyHomeBlock>
        <LegacyHomeBlock label="系统状态">
          <FactRow k="存储">{systemStatus ? legacyStorageModeLabel(systemStatus.storage_mode) : LEGACY_NOT_WIRED}</FactRow>
          <FactRow k="观察期">{systemStatus ? legacyObservationDayLabel(systemStatus.observation_day) : LEGACY_NOT_WIRED}</FactRow>
          {systemStatus?.last_degradation ? <FactRow k="上次降级">{systemStatus.last_degradation.reason_human}</FactRow> : null}
          <FactRow k="安全闸">{systemStatus?.gate_summary || (systemStatus ? "没有额外解封，按默认闸走" : LEGACY_NOT_WIRED)}</FactRow>
          <FactRow k="最近拦截">{legacyRecentCatchLabel(systemStatus)}</FactRow>
          {systemStatus?.warnings.map((warning, index) => <p className="muted small-note" key={`${warning}:${index}`}>{warning}</p>)}
          {systemStatus ? null : <EmptyState what="系统状态读模型还没接上" next="后端读模型接上后这里显示存储、安全闸和最近拦截" />}
        </LegacyHomeBlock>
      </div>
    </div>
  );
}

function LegacyHomeStat({ n, t, tone = "plain", small = false }: { n: string; t: string; tone?: "plain" | "warn" | "ok" | "idle"; small?: boolean }) {
  return <div className="home-stat"><div className={`home-stat-n home-stat-${tone}${small ? " is-small" : ""}`}>{n}</div><div className="home-stat-t">{t}</div></div>;
}

function LegacyHomeBlock({ label, children }: { label: string; children: ReactNode }) {
  return <section className="home-overview-card" aria-label={label}><p className="home-overview-label">{label}</p><div className="spec-scroll home-overview-body">{children}</div></section>;
}

function LegacyHomeRows({ rows, onNavigate }: { rows: LegacyHomeRow[]; onNavigate: NavigateHandler }) {
  return <>{rows.map((row) => <ListRow key={row.key} badge={<i className={`home-dot home-dot-${row.tone}`} aria-hidden="true" />} claim={row.claim} time={row.tail} onSelect={() => onNavigate(row.view, row.focus)} />)}</>;
}

function legacyStorageModeLabel(mode: HomeSystemStatusReadModel["storage_mode"]) { return mode === "db_primary" ? "DB 主写" : "只用 JSON"; }
function legacyStorageHealthLabel(status: HomeSystemStatusReadModel) { return status.storage_healthy ? "正常" : "有问题"; }
function legacyObservationDayLabel(day: number) { return day === 0 ? "未进入观察期" : `观察期第 ${day} 天`; }
function legacySystemHealthDetail(status: HomeSystemStatusReadModel) {
  const parts = [`存储 ${legacyStorageModeLabel(status.storage_mode)}`, legacyObservationDayLabel(status.observation_day)];
  if (status.last_degradation) parts.push(`上次降级：${status.last_degradation.reason_human}`);
  return parts.join(" · ");
}
function legacyRecentCatchLabel(status: HomeSystemStatusReadModel | null) {
  if (!status) return LEGACY_NOT_WIRED;
  const latest = status.recent_catches[0];
  return latest ? `${latest.summary} · ${legacyMsTimeLabel(latest.at_ms) ?? "时间未知"}` : "无";
}
function legacyMsTimeLabel(ms?: number | null): string | null {
  if (!ms) return null;
  const date = new Date(ms);
  return Number.isNaN(date.getTime()) ? null : listRowTimeLabel(date.toISOString());
}

function SecretaryHomeLoading() {
  return (
    <main className="secretary-home secretary-home-state" data-secretary-home-state="loading" aria-live="polite">
      <p className="eyebrow">持续 Secretary</p>
      <h1>正在恢复同一情境</h1>
      <p>正在从应用服务读取 role session、context 与确定性 brief。</p>
    </main>
  );
}

function SecretaryHomeFailure({ home, onReload }: { home: SecretaryHomeReadModel; onReload?: () => void }) {
  return (
    <main className="secretary-home secretary-home-state is-error" data-secretary-home-state="error" aria-live="assertive">
      <p className="eyebrow">持续 Secretary</p>
      <h1>秘书情境暂时没读出来</h1>
      <p>桌面没有用本地缓存补造 role session、scope 或来源事实。请重新读取应用服务结果。</p>
      <SecretaryRecoveryCode code={home.degradation_code ?? home.role_session_recovery.recovery_code} />
      <button className="secondary-button secretary-home-retry" type="button" onClick={onReload}>
        重新读取
      </button>
    </main>
  );
}

function SecretaryHomeDegraded({ home, onReload }: { home: SecretaryHomeReadModel; onReload?: () => void }) {
  return (
    <main className="secretary-home secretary-home-state is-degraded" data-secretary-home-state="degraded" aria-live="polite">
      <p className="eyebrow">持续 Secretary</p>
      <h1>秘书情境处于降级状态</h1>
      <p>没有恢复可确认的 role session 或 context；这里不会用前端旧数据替代它。</p>
      <SecretaryRecoveryCode code={home.degradation_code ?? home.role_session_recovery.recovery_code} />
      <button className="secondary-button secretary-home-retry" type="button" onClick={onReload}>
        再次读取
      </button>
    </main>
  );
}

function SecretaryContextStamp({ home }: { home: SecretaryHomeReadModel }) {
  const recovery = home.role_session_recovery;
  const brief = home.deterministic_brief;
  return (
    <section className="secretary-context-stamp" aria-label="持续 Secretary 情境">
      <span className={`secretary-context-status is-${recovery.status.toLowerCase()}`}>{continuityLabel(recovery.status)}</span>
      {recovery.role_session_ref ? <ContextReference label="RoleSession" value={recovery.role_session_ref} /> : null}
      {recovery.context_ref ? <ContextReference label="Context" value={recovery.context_ref} /> : null}
      {brief ? <ContextReference label="Brief" value={brief.brief_ref} /> : null}
      {home.scope_source_watermark ? <ContextReference label="Watermark" value={home.scope_source_watermark} /> : null}
    </section>
  );
}

function ContextReference({ label, value }: { label: string; value: string }) {
  return (
    <span className="secretary-context-ref">
      <span>{label}</span>
      <code>{value}</code>
    </span>
  );
}

function SecretaryAttentionSpine({
  item,
  actionState,
  onOperate,
  onOpenDeepLink,
}: {
  item: SecretaryHomeAttentionItem;
  actionState?: SecretaryCoordinationViewState;
  onOperate?: (intent: SecretaryCoordinationIntent) => void;
  onOpenDeepLink?: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
}) {
  const actions = secretaryCoordinationActionsFor(item);
  const pending = actionState?.phase === "pending";
  return (
    <article className="secretary-attention-spine" data-item-kind={item.item_kind_code}>
      <div className="secretary-spine-priority">
        <span className="secretary-priority-rank">P{item.priority_rank}</span>
        <span>{item.priority_reason_code}</span>
        <span className="secretary-spine-why">因为 {item.why_code}</span>
      </div>
      <div className="secretary-spine-status">
        <strong>当前状态</strong>
        <code>{item.status_code}</code>
        <span>来源状态</span>
        <code>{item.source_status_code}</code>
      </div>
      <div className="secretary-spine-source">
        <span>负责人</span>
        <code>{item.source_owner.source_owner_ref ?? "来源摘要未提供 owner"}</code>
        <span>来源</span>
        <code>{item.source_object_type} · {item.source_object_ref}</code>
      </div>
      <div className="secretary-spine-change">
        <span>最后变化 {displayUtc(item.last_change_at_utc)}</span>
        <span>到期 {displayUtc(item.due_at_utc)}</span>
        <button
          className="secondary-button secretary-source-link"
          type="button"
          onClick={() => onOpenDeepLink?.(item.deep_link)}
          disabled={!onOpenDeepLink}
          aria-label="在来源模块中查看此关注项"
        >
          回到来源
        </button>
      </div>
      {actions.length ? (
        <div className="secretary-coordination-actions" aria-label="协调动作">
          {actions.map((entry) => (
            <button
              className="secondary-button secretary-home-action"
              key={entry.action}
              type="button"
              data-secretary-action={entry.action}
              disabled={pending || !onOperate}
              onClick={() => onOperate?.(secretaryCoordinationIntent(item, entry))}
            >
              {entry.label}
            </button>
          ))}
        </div>
      ) : (
        <p className="secretary-coordination-note">此状态没有可执行的本地协调动作。</p>
      )}
      <SecretaryCoordinationFeedback state={actionState} />
    </article>
  );
}

function SecretaryCoordinationFeedback({ state }: { state?: SecretaryCoordinationViewState }) {
  if (!state) return null;
  if (state.phase === "pending") {
    return <p className="secretary-coordination-feedback is-pending" aria-live="polite">正在记录协调动作…</p>;
  }
  if (state.phase === "succeeded") {
    return (
      <p className="secretary-coordination-feedback is-succeeded" aria-live="polite">
        已记录 {state.outcome_code} · <code>{state.command_receipt_ref}</code>；正在读取最新 brief。
      </p>
    );
  }
  return (
    <p className="secretary-coordination-feedback is-failed" aria-live="assertive">
      没有记录成功：<code>{state.error_code}</code>。可在此重试，或先回到来源确认状态。
    </p>
  );
}

function SecretaryPersonalObjects({
  home,
  states,
  onOperate,
}: {
  home: SecretaryHomeReadModel;
  states: Readonly<Record<string, SecretaryPersonalObjectViewState>>;
  onOperate?: (intent: SecretaryPersonalObjectIntent) => void;
}) {
  const reminderOwnerRefs = [...new Set([
    ...home.local_objects.reminder_owner_refs,
    ...home.local_objects.personal_actions.map((action) => action.personal_action_id),
  ])];

  const createPersonalAction = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!onOperate) return;
    const form = new FormData(event.currentTarget);
    const title = String(form.get("title") ?? "").trim();
    const dueAtUtc = localDateTimeToUtc(String(form.get("due_at_local") ?? ""));
    if (!title) return;
    onOperate({
      action: "PERSONAL_ACTION_CREATE",
      title,
      ...(dueAtUtc ? { due_at_utc: dueAtUtc } : {}),
    });
  };

  const createReminder = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!onOperate) return;
    const form = new FormData(event.currentTarget);
    const ownerRef = String(form.get("owner_ref") ?? "");
    const scheduledForUtc = localDateTimeToUtc(String(form.get("scheduled_for_local") ?? ""));
    const ianaTimezone = String(form.get("iana_timezone") ?? "").trim();
    if (!ownerRef || !scheduledForUtc || !ianaTimezone) return;
    onOperate({
      action: "REMINDER_CREATE",
      owner_ref: ownerRef,
      scheduled_for_utc: scheduledForUtc,
      iana_timezone: ianaTimezone,
    });
  };

  return (
    <div className="secretary-personal-object-stack">
      <section className="secretary-personal-actions" aria-labelledby="secretary-personal-actions-title">
        <div className="secretary-section-head compact">
          <div>
            <p className="eyebrow">explicit personal action</p>
            <h2 id="secretary-personal-actions-title">个人行动</h2>
          </div>
          <span className="secretary-section-count">{home.local_objects.personal_actions.length}</span>
        </div>
        <form className="secretary-local-create" onSubmit={createPersonalAction}>
          <label>
            <span>行动标题</span>
            <input name="title" type="text" maxLength={160} required placeholder="例如：整理周会结论" disabled={!onOperate} />
          </label>
          <label>
            <span>到期（可选）</span>
            <input name="due_at_local" type="datetime-local" disabled={!onOperate} />
          </label>
          <button className="secondary-button secretary-local-submit" type="submit" disabled={!onOperate || states["personal-action:create"]?.phase === "pending"}>
            新建个人行动
          </button>
          <SecretaryPersonalObjectFeedback state={states["personal-action:create"]} />
        </form>
        {home.local_objects.personal_actions.length ? (
          <ul>
            {home.local_objects.personal_actions.map((action) => {
              const state = states[action.personal_action_id];
              const pending = state?.phase === "pending";
              return (
                <li key={action.personal_action_id}>
                  <strong>{action.title}</strong>
                  <span><code>{action.status}</code> · 到期 {displayUtc(action.due_at_utc)}</span>
                  <small><code>{action.personal_action_id}</code> · 修订 {action.revision}</small>
                  <div className="secretary-local-actions">
                    {action.status === "OPEN" ? (
                      <>
                        <SecretaryPersonalObjectButton
                          action="PERSONAL_ACTION_COMPLETE"
                          label="完成"
                          disabled={pending || !onOperate}
                          onClick={() => onOperate?.({ action: "PERSONAL_ACTION_COMPLETE", item_ref: action.personal_action_id, expected_revision: action.revision })}
                        />
                        <SecretaryPersonalObjectButton
                          action="PERSONAL_ACTION_CANCEL"
                          label="取消"
                          disabled={pending || !onOperate}
                          onClick={() => onOperate?.({ action: "PERSONAL_ACTION_CANCEL", item_ref: action.personal_action_id, expected_revision: action.revision })}
                        />
                      </>
                    ) : (
                      <SecretaryPersonalObjectButton
                        action="PERSONAL_ACTION_REOPEN"
                        label="重新打开"
                        disabled={pending || !onOperate}
                        onClick={() => onOperate?.({ action: "PERSONAL_ACTION_REOPEN", item_ref: action.personal_action_id, expected_revision: action.revision })}
                      />
                    )}
                  </div>
                  <SecretaryPersonalObjectFeedback state={state} />
                </li>
              );
            })}
          </ul>
        ) : (
          <p className="secretary-inline-empty">没有独立创建的个人行动；OpenLoop 不会在这里被复制成 Todo。</p>
        )}
      </section>

      <section className="secretary-personal-actions secretary-local-object-card" aria-labelledby="secretary-reminders-title">
        <div className="secretary-section-head compact">
          <div>
            <p className="eyebrow">server clock reminder</p>
            <h2 id="secretary-reminders-title">提醒</h2>
          </div>
          <span className="secretary-section-count">{home.local_objects.reminders.length}</span>
        </div>
        <form className="secretary-local-create" onSubmit={createReminder}>
          <label>
            <span>关联对象</span>
            <select name="owner_ref" required disabled={!onOperate || reminderOwnerRefs.length === 0}>
              <option value="">选择一个现有对象</option>
              {reminderOwnerRefs.map((ownerRef) => <option key={ownerRef} value={ownerRef}>{ownerRef}</option>)}
            </select>
          </label>
          <label>
            <span>提醒时间</span>
            <input name="scheduled_for_local" type="datetime-local" required disabled={!onOperate || reminderOwnerRefs.length === 0} />
          </label>
          <label>
            <span>IANA 时区</span>
            <input name="iana_timezone" type="text" defaultValue="Asia/Shanghai" required maxLength={128} disabled={!onOperate || reminderOwnerRefs.length === 0} />
          </label>
          <button className="secondary-button secretary-local-submit" type="submit" disabled={!onOperate || reminderOwnerRefs.length === 0}>
            安排提醒
          </button>
          {reminderOwnerRefs.length === 0 ? <p className="secretary-inline-empty">先创建个人行动，或等待来源关注对象进入当前情境。</p> : null}
        </form>
        {home.local_objects.reminders.length ? (
          <ul>
            {home.local_objects.reminders.map((reminder) => {
              const state = states[reminder.reminder_id];
              const pending = state?.phase === "pending";
              const canSnooze = reminder.status === "SCHEDULED" || reminder.status === "FIRED";
              const canDismiss = reminder.status === "SCHEDULED" || reminder.status === "FIRED" || reminder.status === "SNOOZED";
              const canCancel = reminder.status === "SCHEDULED" || reminder.status === "SNOOZED";
              return (
                <li key={reminder.reminder_id}>
                  <strong>{displayUtc(reminder.snoozed_until_utc ?? reminder.scheduled_for_utc)}</strong>
                  <span><code>{reminder.status}</code> · {reminder.iana_timezone}</span>
                  <small>关联 <code>{reminder.owner_ref}</code> · 修订 {reminder.revision}</small>
                  <div className="secretary-local-actions">
                    {canSnooze ? (
                      <SecretaryPersonalObjectButton
                        action="REMINDER_SNOOZE"
                        label="稍后提醒"
                        disabled={pending || !onOperate}
                        onClick={() => onOperate?.({
                          action: "REMINDER_SNOOZE",
                          item_ref: reminder.reminder_id,
                          expected_revision: reminder.revision,
                          snoozed_until_utc: defaultSnoozeUtc(),
                        })}
                      />
                    ) : null}
                    {canDismiss ? (
                      <SecretaryPersonalObjectButton
                        action="REMINDER_DISMISS"
                        label="忽略"
                        disabled={pending || !onOperate}
                        onClick={() => onOperate?.({ action: "REMINDER_DISMISS", item_ref: reminder.reminder_id, expected_revision: reminder.revision })}
                      />
                    ) : null}
                    {canCancel ? (
                      <SecretaryPersonalObjectButton
                        action="REMINDER_CANCEL"
                        label="取消"
                        disabled={pending || !onOperate}
                        onClick={() => onOperate?.({ action: "REMINDER_CANCEL", item_ref: reminder.reminder_id, expected_revision: reminder.revision })}
                      />
                    ) : null}
                  </div>
                  <SecretaryPersonalObjectFeedback state={state} />
                </li>
              );
            })}
          </ul>
        ) : <p className="secretary-inline-empty">当前没有已安排的本地提醒。</p>}
      </section>

      <SecretaryNotificationList home={home} states={states} onOperate={onOperate} />
      <SecretaryDecisionList home={home} states={states} onOperate={onOperate} />
    </div>
  );
}

function SecretaryNotificationList({
  home,
  states,
  onOperate,
}: {
  home: SecretaryHomeReadModel;
  states: Readonly<Record<string, SecretaryPersonalObjectViewState>>;
  onOperate?: (intent: SecretaryPersonalObjectIntent) => void;
}) {
  return (
    <section className="secretary-personal-actions secretary-local-object-card" aria-labelledby="secretary-notifications-title">
      <div className="secretary-section-head compact">
        <div><p className="eyebrow">source event delivery</p><h2 id="secretary-notifications-title">通知</h2></div>
        <span className="secretary-section-count">{home.local_objects.notifications.length}</span>
      </div>
      {home.local_objects.notifications.length ? (
        <ul>
          {home.local_objects.notifications.map((notification) => {
            const state = states[notification.notification_id];
            const pending = state?.phase === "pending";
            return (
              <li key={notification.notification_id}>
                <strong>{notification.notification_purpose_code}</strong>
                <span><code>{notification.status}</code> · {displayUtc(notification.created_at_utc)}</span>
                <small>来源 <code>{notification.source_ref.source_owner_ref}</code> · 修订 {notification.revision}</small>
                <div className="secretary-local-actions">
                  {notification.status === "DELIVERED" ? (
                    <SecretaryPersonalObjectButton
                      action="NOTIFICATION_READ"
                      label="标为已读"
                      disabled={pending || !onOperate}
                      onClick={() => onOperate?.({ action: "NOTIFICATION_READ", item_ref: notification.notification_id, expected_revision: notification.revision })}
                    />
                  ) : null}
                  {notification.status === "DELIVERED" || notification.status === "READ" ? (
                    <SecretaryPersonalObjectButton
                      action="NOTIFICATION_DISMISS"
                      label="收起"
                      disabled={pending || !onOperate}
                      onClick={() => onOperate?.({ action: "NOTIFICATION_DISMISS", item_ref: notification.notification_id, expected_revision: notification.revision })}
                    />
                  ) : null}
                </div>
                <SecretaryPersonalObjectFeedback state={state} />
              </li>
            );
          })}
        </ul>
      ) : <p className="secretary-inline-empty">当前没有本地通知。</p>}
    </section>
  );
}

function SecretaryDecisionList({
  home,
  states,
  onOperate,
}: {
  home: SecretaryHomeReadModel;
  states: Readonly<Record<string, SecretaryPersonalObjectViewState>>;
  onOperate?: (intent: SecretaryPersonalObjectIntent) => void;
}) {
  return (
    <section className="secretary-personal-actions secretary-local-object-card" aria-labelledby="secretary-decisions-title">
      <div className="secretary-section-head compact">
        <div><p className="eyebrow">owner / local dual axis</p><h2 id="secretary-decisions-title">待决策</h2></div>
        <span className="secretary-section-count">{home.local_objects.decisions.length}</span>
      </div>
      {home.local_objects.decisions.length ? (
        <ul>
          {home.local_objects.decisions.map((decision) => {
            const state = states[decision.decision_projection_id];
            const pending = state?.phase === "pending";
            return (
              <li key={decision.decision_projection_id}>
                <strong>来源状态 <code>{decision.owner_status}</code></strong>
                <span>本地显示 <code>{decision.local_visibility_status}</code> · 截止 {displayUtc(decision.decision_by_utc)}</span>
                <small><code>{decision.source_ref}</code> · 来源修订 {decision.source_revision}</small>
                <div className="secretary-local-actions">
                  {decision.local_visibility_status === "UNREAD" ? (
                    <SecretaryPersonalObjectButton
                      action="DECISION_READ"
                      label="标为已读"
                      disabled={pending || !onOperate}
                      onClick={() => onOperate?.({ action: "DECISION_READ", item_ref: decision.decision_projection_id, expected_revision: decision.revision })}
                    />
                  ) : null}
                  {decision.local_visibility_status !== "DISMISSED" ? (
                    <SecretaryPersonalObjectButton
                      action="DECISION_DISMISS"
                      label="从本地收起"
                      disabled={pending || !onOperate}
                      onClick={() => onOperate?.({ action: "DECISION_DISMISS", item_ref: decision.decision_projection_id, expected_revision: decision.revision })}
                    />
                  ) : null}
                </div>
                <SecretaryPersonalObjectFeedback state={state} />
              </li>
            );
          })}
        </ul>
      ) : <p className="secretary-inline-empty">当前没有来源发布的决策请求。</p>}
    </section>
  );
}

function SecretaryPersonalObjectButton({
  action,
  label,
  disabled,
  onClick,
}: {
  action: M4SecretaryPersonalObjectActionCode;
  label: string;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      className="secondary-button secretary-home-action"
      type="button"
      data-secretary-personal-action={action}
      disabled={disabled}
      onClick={onClick}
    >
      {label}
    </button>
  );
}

function SecretaryPersonalObjectFeedback({ state }: { state?: SecretaryPersonalObjectViewState }) {
  if (!state) return null;
  if (state.phase === "pending") return <p className="secretary-coordination-feedback is-pending" aria-live="polite">正在记录本地对象动作…</p>;
  if (state.phase === "succeeded") {
    return <p className="secretary-coordination-feedback is-succeeded" aria-live="polite">已记录 {state.outcome_code}；正在读回最新状态。</p>;
  }
  return <p className="secretary-coordination-feedback is-failed" aria-live="assertive">记录失败：<code>{state.error_code}</code>。可重试。</p>;
}

function localDateTimeToUtc(value: string): string | null {
  if (!value) return null;
  const timestamp = new Date(value);
  return Number.isFinite(timestamp.getTime()) ? timestamp.toISOString() : null;
}

function defaultSnoozeUtc(): string {
  return new Date(Date.now() + 60 * 60 * 1_000).toISOString();
}

function SecretaryAvailability({ home }: { home: SecretaryHomeReadModel }) {
  return (
    <section className="secretary-availability" aria-label="模型与 Handoff 状态">
      <p className="eyebrow">增强边界</p>
      <dl>
        <div>
          <dt>模型增强</dt>
          <dd><code>{home.model_enhancement.status}</code></dd>
        </div>
        <div>
          <dt>Handoff</dt>
          <dd><code>{home.handoff.status}</code></dd>
        </div>
      </dl>
      {home.model_enhancement.recovery_code ? <SecretaryRecoveryCode code={home.model_enhancement.recovery_code} /> : null}
      {home.handoff.recovery_code ? <SecretaryRecoveryCode code={home.handoff.recovery_code} /> : null}
      <p>模型或 Handoff 不可用时，确定性 brief 仍可独立使用；这里不会把模型文本当作来源事实。</p>
    </section>
  );
}

function SecretaryModuleEntries({
  home,
  onOpenDeepLink,
}: {
  home: SecretaryHomeReadModel;
  onOpenDeepLink?: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
}) {
  return (
    <section className="secretary-module-entries" aria-labelledby="secretary-module-entries-title">
      <p className="eyebrow">专业模块入口</p>
      <h2 id="secretary-module-entries-title">按来源继续处理</h2>
      {home.module_entries.length ? (
        <ul>
          {home.module_entries.map((entry) => (
            <li key={entry.entry_ref}>
              <code>{entry.source_owner.source_owner_ref ?? "来源摘要未提供 owner"}</code>
              <button
                className="secondary-button secretary-module-link"
                type="button"
                onClick={() => onOpenDeepLink?.(entry.deep_link)}
                disabled={!onOpenDeepLink}
                aria-label="打开来源负责模块"
              >
                打开模块
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <p className="secretary-inline-empty">当前没有可打开的来源模块入口。</p>
      )}
    </section>
  );
}

function SecretaryRecoveryCode({ code }: { code: string | null }) {
  return code ? <p className="secretary-recovery-code">恢复码：<code>{code}</code></p> : null;
}

export function secretaryCoordinationActionsFor(item: SecretaryHomeAttentionItem): readonly SecretaryHomeAction[] {
  if (item.source_authority !== "M4_COORDINATION" || item.coordination_revision === null) return Object.freeze([]);

  if (item.item_kind_code === "INBOX_ITEM") {
    if (item.status_code === "NEW") {
      return Object.freeze([
        { action: "INBOX_MARK_READ", label: "标为已读" },
        { action: "INBOX_DISMISS", label: "忽略此提醒" },
      ]);
    }
    if (item.status_code === "READ") return Object.freeze([{ action: "INBOX_DISMISS", label: "忽略此提醒" }]);
    return Object.freeze([]);
  }

  if (item.item_kind_code !== "OPEN_LOOP") return Object.freeze([]);
  if (item.status_code === "OPEN") {
    return Object.freeze([
      { action: "OPEN_LOOP_ACKNOWLEDGE", label: "确认看见" },
      { action: "OPEN_LOOP_SNOOZE", label: "一小时后提醒", usesSnoozeTime: true },
      { action: "OPEN_LOOP_CLOSE", label: "停止跟踪" },
      { action: "OPEN_LOOP_DISMISS", label: "忽略此关注" },
      { action: "OPEN_LOOP_CARRY_OVER", label: "带到下一轮" },
    ]);
  }
  if (item.status_code === "ACKNOWLEDGED") {
    return Object.freeze([
      { action: "OPEN_LOOP_SNOOZE", label: "一小时后提醒", usesSnoozeTime: true },
      { action: "OPEN_LOOP_CLOSE", label: "停止跟踪" },
      { action: "OPEN_LOOP_DISMISS", label: "忽略此关注" },
      { action: "OPEN_LOOP_CARRY_OVER", label: "带到下一轮" },
    ]);
  }
  if (item.status_code === "SNOOZED") {
    return Object.freeze([
      { action: "OPEN_LOOP_CLOSE", label: "停止跟踪" },
      { action: "OPEN_LOOP_DISMISS", label: "忽略此关注" },
    ]);
  }
  if (item.status_code === "CLOSED" || item.status_code === "DISMISSED") {
    return Object.freeze([{ action: "OPEN_LOOP_REOPEN", label: "重新关注" }]);
  }
  return Object.freeze([]);
}

function secretaryCoordinationIntent(item: SecretaryHomeAttentionItem, action: SecretaryHomeAction): SecretaryCoordinationIntent {
  return Object.freeze({
    item,
    action: action.action,
    ...(action.usesSnoozeTime ? { snoozed_until_utc: oneHourFromNowUtc() } : {}),
  });
}

function oneHourFromNowUtc(): string {
  return new Date(Date.now() + 60 * 60 * 1000).toISOString();
}

function displayUtc(value: string | null): string {
  if (!value) return "未设置";
  return value.replace("T", " ").replace("Z", " UTC");
}

function continuityLabel(status: "LOADING" | "RESTORED" | "UNAVAILABLE"): string {
  if (status === "RESTORED") return "已继续同一情境";
  if (status === "LOADING") return "正在恢复情境";
  return "情境暂不可用";
}
