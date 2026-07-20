import { useEffect, useMemo, useState } from "react";
import { EmptyState, FactRow, ListRow, Pill, SegTitle } from "../components/SpecPrimitives";
import { displayStatus, formatDate, listRowTimeLabel, runtimeLogCategoryLabel } from "../lib/format";
import {
  queryAuditLedgerReadModel,
  type AuditLedgerReadModel,
  type AuditLedgerReadModelItem,
} from "../lib/tauri";
import type { WorkbenchSnapshot } from "../lib/types";
import type { NavigationFocus } from "../lib/workbenchNavigation";

// ④ 审计账本页（2026-07-15 施工）。
// 宪法 §二：「审计/账本：任何态都不是主角；常驻"可查"位」→ 本页不进左导航，
//   可达路径 = 右栏 rail「管」抽屉点行直达（带 focus 落到那一条）。
// 宪法 §六 回顾面：唯一问题=「我要找的那件事多快找到」→ B1 同构（工具条+过滤+列表+详情），
//   永不打断、纯拉式。
// DESIGN.md §三·五：「开发者详情」折叠废除后，运行编号/账本事件流等机器信息一律归本页——
//   所以机器字段在本页的详情栏里是正主，不是违宪。
//
// 数据边界：B 是唯一的分页主流；运行日志和健康诊断保留为本地完整并列区，
// 不与 B 混页、不把当前页搜索伪装成全局搜索。

const LEDGER_PAGE_SIZE = 50;
export const AUDIT_EVENT_NOT_IN_CURRENT_PAGE_MESSAGE = "目标事件不在最新一页(账本按时间倒序分页),可翻页查找";

type ParallelAuditFilter = "all" | "runtime" | "health";

type AuditLedgerFact = {
  k: string;
  v: string;
  bad?: boolean;
};

export type AuditLedgerMainRow = {
  key: string;
  item: AuditLedgerReadModelItem;
};

type ParallelAuditRow = {
  key: string;
  kind: Exclude<ParallelAuditFilter, "all">;
  badgeLabel: string;
  badgeTone: "plain" | "candidate" | "warn" | "unknown";
  claim: string;
  timeLabel: string | null;
  facts: AuditLedgerFact[];
  boundary: string;
};

export function AuditLedgerView({
  snapshot,
  focus,
}: {
  snapshot: WorkbenchSnapshot;
  // 「点击带上下文直达」：右栏抽屉点某一行 → 本页开在那一条上。
  focus?: NavigationFocus | null;
}) {
  const [page, setPage] = useState(0);
  const [kindFilter, setKindFilter] = useState("");
  const [ledger, setLedger] = useState<AuditLedgerReadModel | null>(null);
  const [ledgerLoading, setLedgerLoading] = useState(true);
  const [ledgerError, setLedgerError] = useState<string | null>(null);
  const [selectedLedgerKey, setSelectedLedgerKey] = useState<string | null>(null);
  const [parallelQuery, setParallelQuery] = useState("");
  const [parallelFilter, setParallelFilter] = useState<ParallelAuditFilter>("all");
  const [selectedParallelKey, setSelectedParallelKey] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    setLedger(null);
    setLedgerLoading(true);
    setLedgerError(null);
    setSelectedLedgerKey(null);
    void queryAuditLedgerReadModel({
      page,
      page_size: LEDGER_PAGE_SIZE,
      kind_filter: kindFilter || undefined,
    })
      .then((nextLedger) => {
        if (!active) return;
        setLedger(nextLedger);
      })
      .catch((error: unknown) => {
        if (!active) return;
        setLedgerError(error instanceof Error ? error.message : String(error));
      })
      .finally(() => {
        if (active) setLedgerLoading(false);
      });

    return () => {
      active = false;
    };
  }, [kindFilter, page, snapshot]);

  const mainRows = useMemo(() => buildAuditLedgerMainRows(ledger?.items ?? []), [ledger?.items]);
  const parallelRows = useMemo(() => buildParallelAuditRows(snapshot), [snapshot]);
  const visibleParallelRows = useMemo(
    () => filterParallelAuditRows(parallelRows, parallelFilter, parallelQuery),
    [parallelFilter, parallelQuery, parallelRows],
  );
  const missingAuditEventFocus = isMissingAuditEventFocus(focus, mainRows, Boolean(ledger));
  const focusedLedgerRow = focus?.kind === "audit-event" ? mainRows.find((row) => row.key === focus.id) ?? null : null;
  const selectedLedger =
    (selectedLedgerKey ? mainRows.find((row) => row.key === selectedLedgerKey) : null) ??
    focusedLedgerRow ??
    (missingAuditEventFocus ? null : mainRows[0] ?? null);
  const selectedParallel =
    (selectedParallelKey ? visibleParallelRows.find((row) => row.key === selectedParallelKey) : null) ??
    (focus ? visibleParallelRows.find((row) => row.key === focus.id) : null) ??
    visibleParallelRows[0] ??
    null;
  const pageCount = ledger ? Math.max(1, Math.ceil(ledger.total / ledger.page_size)) : 1;
  const canMoveForward = Boolean(ledger && (ledger.page + 1) * ledger.page_size < ledger.total);

  function changeKindFilter(nextFilter: string) {
    setKindFilter(nextFilter);
    setPage(0);
    setSelectedLedgerKey(null);
  }

  function changePage(nextPage: number) {
    setPage(nextPage);
    setSelectedLedgerKey(null);
  }

  return (
    <section className="stage-pad audit-ledger" aria-label="审计账本">
      <div className="sr-only">
        <p>审计账本</p>
        <h1>审计账本</h1>
      </div>

      <div className="view-stack">
        <div className="memory-b1-grid">
          <section className="memory-center-panel" aria-label="账本主流">
          <SegTitle>账本主流</SegTitle>
          <div className="memory-b1-toolbar">
            <label className="sr-only" htmlFor="audit-ledger-kind-filter">
              账本类型过滤
            </label>
            <select
              id="audit-ledger-kind-filter"
              value={kindFilter}
              onChange={(event) => changeKindFilter(event.target.value)}
              aria-label="账本类型过滤"
            >
              <option value="">全部类型</option>
              {(ledger?.kinds ?? []).map((kind) => (
                <option key={kind} value={kind}>
                  {kind}
                </option>
              ))}
            </select>
          </div>
          <div className="memory-b1-toolbar" aria-label="账本分页">
            <button type="button" className="jiaoban-chip" disabled={!ledger || ledger.page === 0} onClick={() => changePage(page - 1)}>
              上一页
            </button>
            <span className="muted small-note">{ledger ? "第 " + (ledger.page + 1) + " / " + pageCount + " 页" : "正在读取…"}</span>
            <button type="button" className="jiaoban-chip" disabled={!canMoveForward} onClick={() => changePage(page + 1)}>
              下一页
            </button>
          </div>
          <div className="spec-scroll memory-b1-list audit-ledger-list" aria-label="账本主流列表">
            {mainRows.map((row) => (
              <ListRow
                key={row.key}
                badge={<Pill tone={row.item.event_type === "unknown" ? "warn" : "plain"}>账本</Pill>}
                claim={row.item.human_summary}
                time={formatDate(row.item.at_ms)}
                selected={selectedLedger?.key === row.key}
                onSelect={() => setSelectedLedgerKey(row.key)}
              />
            ))}
            {!ledgerLoading && !ledgerError && !mainRows.length ? <MainLedgerEmpty hasKindFilter={Boolean(kindFilter)} /> : null}
          </div>
          {ledger ? (
            <p className="muted small-note">
              过滤后共 {ledger.total} 条；按时间倒序，每页 {ledger.page_size} 条。
            </p>
          ) : null}
          {ledger?.warnings.map((warning, index) => (
            <p className="muted small-note" key={index + ":" + warning}>
              账本读取提示：{warning}
            </p>
          ))}
          {missingAuditEventFocus ? (
            <p className="rail-error">{AUDIT_EVENT_NOT_IN_CURRENT_PAGE_MESSAGE}</p>
          ) : null}
          </section>

          <section className="memory-center-panel memory-detail-panel" aria-label="账本主流详情">
            {selectedLedger ? (
              <MainLedgerDetail row={selectedLedger} />
            ) : (
              <EmptyState
                what={ledgerLoading ? "账本正在读取" : "暂无可展示详情"}
                next={missingAuditEventFocus ? "按时间倒序翻页查找目标事件" : "先在左侧账本主流选一条记录"}
              />
            )}
          </section>
        </div>

        <div className="memory-b1-grid">
          <section className="memory-center-panel" aria-label="并列运行与健康记录">
          <SegTitle>运行日志与健康诊断</SegTitle>
          <div className="memory-b1-toolbar">
            <input
              type="text"
              className="jiaoban-session-search"
              placeholder="搜运行日志与健康诊断…"
              value={parallelQuery}
              onChange={(event) => setParallelQuery(event.target.value)}
              aria-label="搜索运行日志与健康诊断"
            />
          </div>
          <div className="memory-b1-toolbar" role="group" aria-label="并列记录过滤">
            <button
              className={"jiaoban-chip " + (parallelFilter === "all" ? "on" : "")}
              type="button"
              onClick={() => setParallelFilter("all")}
            >
              全部 {parallelRows.length}
            </button>
            <button
              className={"jiaoban-chip " + (parallelFilter === "runtime" ? "on" : "")}
              type="button"
              onClick={() => setParallelFilter("runtime")}
            >
              运行日志 {parallelRows.filter((row) => row.kind === "runtime").length}
            </button>
            <button
              className={"jiaoban-chip " + (parallelFilter === "health" ? "on" : "")}
              type="button"
              onClick={() => setParallelFilter("health")}
            >
              健康诊断 {parallelRows.filter((row) => row.kind === "health").length}
            </button>
          </div>
          <div className="spec-scroll memory-b1-list audit-ledger-list" aria-label="并列记录列表">
            {visibleParallelRows.map((row) => (
              <ListRow
                key={row.key}
                badge={<Pill tone={row.badgeTone}>{row.badgeLabel}</Pill>}
                claim={row.claim}
                time={row.timeLabel ?? "时间未登记"}
                selected={selectedParallel?.key === row.key}
                onSelect={() => setSelectedParallelKey(row.key)}
              />
            ))}
            {!visibleParallelRows.length ? <ParallelLedgerEmpty hasAnyRow={parallelRows.length > 0} querying={Boolean(parallelQuery.trim())} /> : null}
          </div>
          <p className="muted small-note">共 {parallelRows.length} 条，本地完整展示，不参与账本主流分页。</p>
          </section>

          <section className="memory-center-panel memory-detail-panel" aria-label="并列记录详情">
            {selectedParallel ? (
              <ParallelLedgerDetail row={selectedParallel} />
            ) : (
              <EmptyState what="暂无可展示详情" next="先在左侧运行日志与健康诊断中选一条记录" />
            )}
          </section>
        </div>
      </div>

      {ledgerError ? <p className="rail-error">账本主流读取失败：{ledgerError}</p> : null}
      <p className="muted small-note">
        账本只记账、不改事实：这里不重跑、不批准、不修状态。账本主流只读 B 聚合流；运行日志和健康诊断保持独立并列，不混入其分页或总数。
      </p>
    </section>
  );
}

function MainLedgerDetail({ row }: { row: AuditLedgerMainRow }) {
  const { item } = row;

  return (
    <article>
      <SegTitle>账本记录</SegTitle>
      <p className="audit-ledger-claim">{item.human_summary}</p>
      <FactRow k="时间">{formatDate(item.at_ms)}</FactRow>
      <FactRow k="来源">{item.source}</FactRow>
      <FactRow k="事件类型">{item.event_type}</FactRow>
      <FactRow k="归属对象">{item.target_ref || "未登记"}</FactRow>
      <details className="agent-boundary-details">
        <summary>查看原始记录</summary>
        <pre className="task-preview-code">{rawJsonText(item.raw_json)}</pre>
      </details>
      <p className="muted small-note">本页只展示已发生的账本事实；原始机器字段只在此处下钻，不参与卡面主显示。</p>
    </article>
  );
}

function ParallelLedgerDetail({ row }: { row: ParallelAuditRow }) {
  return (
    <article>
      <SegTitle>{row.badgeLabel}</SegTitle>
      <p className="audit-ledger-claim">{row.claim}</p>
      {row.facts.map((fact) => (
        <FactRow k={fact.k} key={fact.k} bad={fact.bad}>
          {fact.v}
        </FactRow>
      ))}
      <p className="muted small-note">{row.boundary}</p>
    </article>
  );
}

function MainLedgerEmpty({ hasKindFilter }: { hasKindFilter: boolean }) {
  return (
    <EmptyState
      what={hasKindFilter ? "这个类型暂无账本记录" : "账本还没有记录"}
      next={hasKindFilter ? "换一个类型或切回「全部类型」" : "去项目页交办一单活；派发、运行和复核都会自动记账到这里"}
    />
  );
}

function ParallelLedgerEmpty({ hasAnyRow, querying }: { hasAnyRow: boolean; querying: boolean }) {
  if (querying) return <EmptyState what="没有匹配的并列记录" next="换个词试试，或清空搜索" />;
  if (hasAnyRow) return <EmptyState what="这一类暂无记录" next="把过滤切回「全部」看其它类别" />;
  return <EmptyState what="暂无运行日志或健康诊断" next="运行状态变化后会在这里留下独立记录" />;
}

export function buildAuditLedgerMainRows(items: AuditLedgerReadModelItem[]): AuditLedgerMainRow[] {
  return items.map((item, index) => ({
    key: auditLedgerItemKey(item, index),
    item,
  }));
}

export function isMissingAuditEventFocus(
  focus: NavigationFocus | null | undefined,
  rows: AuditLedgerMainRow[],
  loaded: boolean,
): boolean {
  return Boolean(loaded && focus?.kind === "audit-event" && !rows.some((row) => row.key === focus.id));
}

function auditLedgerItemKey(item: AuditLedgerReadModelItem, index: number): string {
  // 右栏旧工单事件来自 workflow_state.audit_events，B 的同源项才能按 event_id 精确命中。
  // target_ref 是工单/工作流/运行对象，绝不能拿它猜事件编号。
  const eventId =
    item.source === "workflow_state" ? rawJsonString(item.raw_json, "event_id") ?? rawJsonString(item.raw_json, "audit_event_id") : null;
  return eventId
    ? "audit-event:" + eventId
    : "audit-ledger:" + item.source + ":" + item.event_type + ":" + item.at_ms + ":" + index;
}

function buildParallelAuditRows(snapshot: WorkbenchSnapshot): ParallelAuditRow[] {
  const runtimeRows: ParallelAuditRow[] = snapshot.runtime_log_store.entries
    .filter((entry) => entry.user_visible)
    .map((entry) => ({
      key: "runtime-log:" + entry.entry_id,
      kind: "runtime" as const,
      badgeLabel: runtimeLogCategoryLabel(entry.category),
      badgeTone: entry.severity === "error" ? ("warn" as const) : entry.severity === "warning" ? ("warn" as const) : ("plain" as const),
      claim: entry.summary,
      timeLabel: listRowTimeLabel(entry.started_at ?? entry.finished_at),
      facts: [
        { k: "类别", v: runtimeLogCategoryLabel(entry.category) },
        { k: "状态", v: displayStatus(entry.status), bad: entry.severity === "error" },
        { k: "严重度", v: displayStatus(entry.severity), bad: entry.severity === "error" },
        { k: "详情", v: entry.detail || "未登记" },
        { k: "开始", v: entry.started_at || "未登记" },
        { k: "结束", v: entry.finished_at || "未登记" },
        { k: "耗时", v: entry.duration_ms === null || entry.duration_ms === undefined ? "未登记" : String(entry.duration_ms) + " 毫秒" },
        { k: "脱敏", v: entry.redaction_status === "redacted_safe_summary" ? "已脱敏摘要" : displayStatus(entry.redaction_status) },
        { k: "省略的敏感内容", v: entry.sensitive_omissions.length ? String(entry.sensitive_omissions.length) + " 处" : "无" },
        { k: "审计引用", v: entry.audit_refs.length ? entry.audit_refs.join(" / ") : "无" },
        { k: "条目编号", v: entry.entry_id },
      ],
      boundary: snapshot.runtime_log_store.boundary.separation_rule,
    }));

  const healthRows: ParallelAuditRow[] = snapshot.diagnostic_summary.degraded_states.map((state) => ({
    key: "degraded-state:" + state.state_id,
    kind: "health" as const,
    badgeLabel: "健康诊断",
    badgeTone: state.blocks_real_execution ? ("warn" as const) : ("unknown" as const),
    claim: state.title + "：" + state.summary,
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

  return [...runtimeRows, ...healthRows];
}

function filterParallelAuditRows(rows: ParallelAuditRow[], filter: ParallelAuditFilter, query: string): ParallelAuditRow[] {
  const needle = query.trim().toLowerCase();
  return rows
    .filter((row) => (filter === "all" ? true : row.kind === filter))
    .filter((row) => {
      if (!needle) return true;
      return (
        row.claim.toLowerCase().includes(needle) ||
        row.badgeLabel.toLowerCase().includes(needle) ||
        row.facts.some((fact) => fact.v.toLowerCase().includes(needle))
      );
    });
}

function rawJsonString(rawJson: unknown, key: string): string | null {
  if (!rawJson || typeof rawJson !== "object" || Array.isArray(rawJson)) return null;
  const value = (rawJson as Record<string, unknown>)[key];
  return typeof value === "string" && value.trim() ? value : null;
}

function rawJsonText(rawJson: unknown): string {
  try {
    return JSON.stringify(rawJson, null, 2) ?? String(rawJson);
  } catch {
    return String(rawJson);
  }
}
