import { useState } from "react";
import { Badge } from "../components/Badge";
import { EmptyState, FactRow, ListRow, SegTitle } from "../components/SpecPrimitives";
import { displayStatus, listRowTimeLabel, runtimeLogCategoryLabel } from "../lib/format";
import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { NavigationFocus } from "../lib/workbenchNavigation";

// ④ 审计账本页（2026-07-15 施工）。
// 宪法 §二：「审计/账本：任何态都不是主角；常驻"可查"位」→ 本页不进左导航，
//   可达路径 = 右栏 rail「管」抽屉点行直达（带 focus 落到那一条）。
// 宪法 §六 回顾面：唯一问题=「我要找的那件事多快找到」→ B1 同构（工具条+过滤+列表+详情），
//   永不打断、纯拉式。
// DESIGN.md §三·五：「开发者详情」折叠废除后，运行编号/账本事件流等机器信息一律归本页——
//   所以机器字段在本页的**详情栏**里是正主，不是违宪。
//
// 数据边界（诚实声明）：后端审计账本读模型仍在接线（backend-ui-support-readmodels 包 §B）。
// 本页只渲染现在真能拿到的三个来源，拿不到的字段留白写「未登记」，不编。
type AuditLedgerFilter = "all" | "workitem" | "runtime" | "health";

type AuditLedgerFact = {
  k: string;
  v: string;
  bad?: boolean;
};

type AuditLedgerRow = {
  key: string;
  kind: Exclude<AuditLedgerFilter, "all">;
  badgeLabel: string;
  badgeTone: "neutral" | "candidate" | "warning" | "unknown";
  claim: string;
  timeLabel: string | null;
  // 详情栏：事实行（左键右值）。
  facts: AuditLedgerFact[];
  boundary: string;
};

export function AuditLedgerView({
  snapshot,
  workflowState,
  workflowStateError,
  focus,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateError?: string | null;
  // 「点击带上下文直达」：右栏抽屉点某一行 → 本页开在那一条上。
  focus?: NavigationFocus | null;
}) {
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<AuditLedgerFilter>("all");
  const [selectedKey, setSelectedKey] = useState<string | null>(null);

  const allRows = buildAuditLedgerRows(snapshot, workflowState);
  const rows = allRows
    .filter((row) => (filter === "all" ? true : row.kind === filter))
    .filter((row) => (query.trim() ? matchesQuery(row, query) : true));

  // 选中优先级：用户手点 > 导航带来的 focus > 列表第一条（B1 样板同款回落）。
  const selected =
    (selectedKey ? rows.find((row) => row.key === selectedKey) : null) ??
    (focus ? rows.find((row) => row.key === focus.id) : null) ??
    rows[0] ??
    null;

  const counts = {
    all: allRows.length,
    workitem: allRows.filter((row) => row.kind === "workitem").length,
    runtime: allRows.filter((row) => row.kind === "runtime").length,
    health: allRows.filter((row) => row.kind === "health").length,
  };

  return (
    <section className="stage-pad audit-ledger" aria-label="审计账本">
      <div className="sr-only">
        <p>审计账本</p>
        <h1>审计账本</h1>
      </div>

      <div className="memory-b1-grid">
        <section className="memory-center-panel" aria-label="审计流">
          <div className="memory-b1-toolbar">
            <input
              type="text"
              className="jiaoban-session-search"
              placeholder="搜账本…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              aria-label="搜索审计账本"
            />
          </div>
          <div className="memory-b1-toolbar" role="group" aria-label="账本过滤">
            <button className={`jiaoban-chip ${filter === "all" ? "on" : ""}`} type="button" onClick={() => setFilter("all")}>
              全部 {counts.all}
            </button>
            <button className={`jiaoban-chip ${filter === "workitem" ? "on" : ""}`} type="button" onClick={() => setFilter("workitem")}>
              工单账本 {counts.workitem}
            </button>
            <button className={`jiaoban-chip ${filter === "runtime" ? "on" : ""}`} type="button" onClick={() => setFilter("runtime")}>
              运行日志 {counts.runtime}
            </button>
            <button className={`jiaoban-chip ${filter === "health" ? "on" : ""}`} type="button" onClick={() => setFilter("health")}>
              健康诊断 {counts.health}
            </button>
          </div>
          <div className="spec-scroll memory-b1-list audit-ledger-list" aria-label="审计流列表">
            {rows.map((row) => (
              <ListRow
                key={row.key}
                badge={<Badge tone={row.badgeTone}>{row.badgeLabel}</Badge>}
                claim={row.claim}
                time={row.timeLabel ?? "时间未登记"}
                selected={selected?.key === row.key}
                onSelect={() => setSelectedKey(row.key)}
              />
            ))}
            {!rows.length ? <AuditLedgerEmpty hasAnyRow={allRows.length > 0} querying={Boolean(query.trim())} /> : null}
          </div>
        </section>

        <section className="memory-center-panel memory-detail-panel" aria-label="账本详情">
          {selected ? (
            <>
              <SegTitle>{selected.badgeLabel}</SegTitle>
              <p className="audit-ledger-claim">{selected.claim}</p>
              {selected.facts.map((fact) => (
                <FactRow k={fact.k} key={fact.k} bad={fact.bad}>
                  {fact.v}
                </FactRow>
              ))}
              <p className="muted small-note">{selected.boundary}</p>
            </>
          ) : (
            <EmptyState what="暂无可展示详情" next="先在左侧选一条账本记录；列表为空时，去项目页交办一单活，跑起来就会记账" />
          )}
        </section>
      </div>

      {workflowStateError ? <p className="rail-error">工单账本读取失败：{workflowStateError}</p> : null}
      <p className="muted small-note">
        账本只记账、不改事实：这里不重跑、不批准、不修状态。完整账本读模型还在接线，当前只显示工单状态变化、运行日志和健康诊断三个已能读到的来源。
      </p>
    </section>
  );
}

// 空态（宪法 D7：必答「下一步做什么」，不许只说"这里没有东西"）。
function AuditLedgerEmpty({ hasAnyRow, querying }: { hasAnyRow: boolean; querying: boolean }) {
  if (querying) return <EmptyState what="没有匹配的账本记录" next="换个词试试，或把过滤切回「全部」" />;
  if (hasAnyRow) return <EmptyState what="这一类暂无记录" next="把过滤切回「全部」看其它类别" />;
  return <EmptyState what="账本还没有记录" next="去项目页交办一单活；派发、运行和复核都会自动记账到这里" />;
}

function matchesQuery(row: AuditLedgerRow, query: string): boolean {
  const needle = query.trim().toLowerCase();
  if (row.claim.toLowerCase().includes(needle)) return true;
  if (row.badgeLabel.toLowerCase().includes(needle)) return true;
  return row.facts.some((fact) => fact.v.toLowerCase().includes(needle));
}

// 三个现成可得来源。后端账本读模型就绪后，这里换成读那一个读模型即可（行形状不变）。
function buildAuditLedgerRows(
  snapshot: WorkbenchSnapshot,
  workflowState: WorkflowStateSnapshot | null,
): AuditLedgerRow[] {
  const workItemRows: AuditLedgerRow[] = (workflowState?.project_workflows ?? []).flatMap((workflow) =>
    workflow.task_drafts.flatMap((task) =>
      task.recent_audit_events.map((event) => {
        const transition = `${displayStatus(event.before_state)} → ${displayStatus(event.after_state)}`;
        const failed = event.after_state === "failed";
        return {
          key: `audit-event:${event.event_id}`,
          kind: "workitem" as const,
          badgeLabel: "工单账本",
          badgeTone: failed ? ("warning" as const) : ("candidate" as const),
          // 一句人话：优先账本自带的 reason；没有就退成状态变化，不把 event_type 摆上脸。
          claim: `${task.title}：${event.reason || transition}`,
          timeLabel: listRowTimeLabel(event.created_at),
          facts: [
            { k: "工单", v: task.title },
            { k: "变化", v: transition, bad: failed },
            { k: "原因", v: event.reason || "未登记" },
            { k: "工作流", v: workflow.title || "未登记" },
            { k: "项目", v: workflow.project_root || "未登记" },
            { k: "时间", v: event.created_at || "未登记" },
            // 机器字段：本页就是它们的归处（DESIGN.md §三·五），故上脸不违宪。
            { k: "事件类型", v: event.event_type },
            { k: "事件编号", v: event.event_id },
            { k: "工作项编号", v: task.work_item_id },
          ],
          boundary: "账本只记录已经发生的状态变化；在这里看不改任何事实，重跑或复核仍回项目页。",
        };
      }),
    ),
  );

  const runtimeRows: AuditLedgerRow[] = snapshot.runtime_log_store.entries
    .filter((entry) => entry.user_visible)
    .map((entry) => ({
      key: `runtime-log:${entry.entry_id}`,
      kind: "runtime" as const,
      badgeLabel: runtimeLogCategoryLabel(entry.category),
      badgeTone: entry.severity === "error" ? ("warning" as const) : entry.severity === "warning" ? ("warning" as const) : ("neutral" as const),
      claim: entry.summary,
      timeLabel: listRowTimeLabel(entry.started_at ?? entry.finished_at),
      facts: [
        { k: "类别", v: runtimeLogCategoryLabel(entry.category) },
        { k: "状态", v: displayStatus(entry.status), bad: entry.severity === "error" },
        { k: "严重度", v: displayStatus(entry.severity), bad: entry.severity === "error" },
        { k: "详情", v: entry.detail || "未登记" },
        { k: "开始", v: entry.started_at || "未登记" },
        { k: "结束", v: entry.finished_at || "未登记" },
        { k: "耗时", v: entry.duration_ms === null || entry.duration_ms === undefined ? "未登记" : `${entry.duration_ms} 毫秒` },
        { k: "脱敏", v: entry.redaction_status === "redacted_safe_summary" ? "已脱敏摘要" : displayStatus(entry.redaction_status) },
        { k: "省略的敏感内容", v: entry.sensitive_omissions.length ? `${entry.sensitive_omissions.length} 处` : "无" },
        { k: "审计引用", v: entry.audit_refs.length ? entry.audit_refs.join(" / ") : "无" },
        { k: "条目编号", v: entry.entry_id },
      ],
      boundary: snapshot.runtime_log_store.boundary.separation_rule,
    }));

  const healthRows: AuditLedgerRow[] = snapshot.diagnostic_summary.degraded_states.map((state) => ({
    key: `degraded-state:${state.state_id}`,
    kind: "health" as const,
    badgeLabel: "健康诊断",
    badgeTone: state.blocks_real_execution ? ("warning" as const) : ("unknown" as const),
    claim: `${state.title}：${state.summary}`,
    // ServiceDegradedState 没有时间字段（后端未提供）→ 留白，不编。
    timeLabel: null,
    facts: [
      { k: "类型", v: displayStatus(state.kind) },
      { k: "严重度", v: displayStatus(state.severity), bad: state.blocks_real_execution },
      { k: "是否挡真实执行", v: state.blocks_real_execution ? "挡住了" : "没挡" },
      { k: "要你处理吗", v: state.user_action_required ? "需要你处理" : "不需要你处理" },
      { k: "建议下一步", v: state.recommended_next_step || "未登记" },
      { k: "来源引用", v: state.source_refs.length ? state.source_refs.join(" / ") : "无" },
      { k: "状态编号", v: state.state_id },
    ],
    boundary: "健康诊断只解释问题，不自动修复、不自动重试、不调用供应方。",
  }));

  return [...workItemRows, ...runtimeRows, ...healthRows];
}
