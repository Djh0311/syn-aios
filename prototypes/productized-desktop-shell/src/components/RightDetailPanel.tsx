import { SecretaryBrief } from "./SecretaryBrief";
import { deriveRunQueueReadModel } from "../lib/runQueue";
import type { SecretaryContext } from "../lib/secretaryReadModel";
import type { MemoryCandidateStoreV1, MemoryCaptureStoreV1, WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { RightPanelKey, ViewKey } from "../lib/workbenchNavigation";

type RightFeedTone = "ok" | "warn" | "err" | "run";

type RightFeedItem = {
  title: string;
  meta: string;
  tone: RightFeedTone;
};

type RightProjectGroupItem = RightFeedItem & {
  id: string;
};

type RightProjectGroup = {
  id: string;
  title: string;
  projectRoot: string;
  todoItems: RightProjectGroupItem[];
  runningItems: RightProjectGroupItem[];
};

export function RightDetailPanel({
  activePanel,
  snapshot,
  workflowState,
  notice,
  error,
  workflowStateError,
  memoryCaptureStore = null,
  memoryCandidateStore = null,
  secretaryContext,
  onClose,
  onNavigate,
  onReloadWorkflowState,
}: {
  activePanel: RightPanelKey;
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  notice: string;
  error: boolean;
  workflowStateError: string | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  secretaryContext: SecretaryContext;
  onClose: () => void;
  onNavigate: (view: ViewKey) => void;
  onReloadWorkflowState: () => void;
}) {
  if (activePanel === "secretary") {
    return (
      <div className="right-detail">
        <section className="status-pane secretary-boundary-pane">
          <h2>
            秘书只读摘要
            <button className="pane-close" type="button" onClick={onClose} aria-label="收起右侧详情">
              ×
            </button>
          </h2>
          <p className="muted small-note">秘书入口只展示派生读模型；不写事实、不派发任务、不批准权限、不写正式记忆。</p>
        </section>
        <section className="status-pane">
          <SecretaryBrief context={secretaryContext} />
        </section>
      </div>
    );
  }

  const auditItems: RightFeedItem[] =
    workflowState?.project_workflows.flatMap((workflow) =>
      workflow.task_drafts.flatMap((task) =>
        task.recent_audit_events.map((event) => ({
          title: event.event_type,
          meta: `${task.title} · ${displayStatus(event.after_state || event.before_state || "state 未登记")}`,
          tone: event.after_state === "failed" ? "err" : "ok",
        })),
      ),
    ) ?? [];
  const permissionRequests = workflowState?.project_workflows.flatMap((workflow) => workflow.permission_requests) ?? [];
  const runningTasks: RightFeedItem[] =
    workflowState?.project_workflows.flatMap((workflow) =>
      workflow.task_drafts
        .filter((task) => ["running", "waiting_for_permission", "retry_pending", "ready_to_dispatch", "ready_for_review"].includes(task.state))
        .map((task) => ({
          title: task.title,
          meta: `${displayStatus(task.state)} · ${workflow.title}`,
          tone: task.state === "waiting_for_permission" ? "warn" : task.state === "running" ? "run" : "ok",
        })),
    ) ?? [];
  const runtimeAttentionItems: RightFeedItem[] = snapshot.runtime_session_attention.map((item) => ({
    title: item.title,
    meta: `${displayStatus(item.status)} · 读回 ${displayStatus(item.readback_boundary.status)} · ${item.readback_boundary.reason}`,
    tone: item.blocks_continuation ? "err" : item.requires_user_action ? "warn" : "run",
  }));
  const runtimeTodoItems: RightFeedItem[] = snapshot.runtime_session_attention
    .filter((item) => item.requires_user_action || item.blocks_continuation)
    .map((item) => ({
      title: item.title,
      meta: item.recommended_next_step,
      tone: item.blocks_continuation ? "err" : "warn",
    }));
  const runtimeRunningItems: RightFeedItem[] = snapshot.session_run_status_summaries.map((summary) => ({
    title: summary.session_id,
    meta: `${summary.current_status_label} · 关注 ${summary.attention_count} · 读回 ${displayStatus(summary.readback_status)}`,
    tone: summary.blocking_count ? "err" : summary.needs_user_count ? "warn" : "run",
  }));
  const productCommandReadModel = snapshot.real_execution_product_commands;
  const failureStopRetry = productCommandReadModel?.failure_stop_retry_summary ?? null;
  const failureStopRetryItems = failureStopRetry?.items ?? [];
  const runQueue = deriveRunQueueReadModel({ snapshot, workflowState, memoryCaptureStore, memoryCandidateStore });
  const projectGroups = buildRightProjectGroups(workflowState);
  const projectTodoGroups = projectGroups.filter((group) => group.todoItems.length);
  const projectRunningGroups = projectGroups.filter((group) => group.runningItems.length);
  const queueRunningItems: RightFeedItem[] = runQueue.run_queue_items.slice(0, 6).map((item) => ({
    title: item.user_visible_summary,
    meta: `${displayStatus(item.status)} · 下一步：${item.next_step_label} · 结果数 ${productCommandResultCountLabel(item.readback_result_count)}`,
    tone: item.status === "failed" || item.status === "blocked_by_guard" || item.status === "duplicate_blocked" ? "err" : item.requires_user_action ? "warn" : "run",
  }));
  const queueTodoItems: RightFeedItem[] = runQueue.user_confirmation_queue.slice(0, 8).map((item) => ({
    title: confirmationKindLabel(item.kind),
    meta: `${item.title} · ${item.summary}`,
    tone: item.risk_level === "high" ? "err" : "warn",
  }));
  const queueFailureItems: RightFeedItem[] = runQueue.failure_control_summaries.slice(0, 6).map((item) => ({
    title: failureClassificationLabel(item.classification),
    meta: `${displayStatus(item.status)} · ${item.recommended_next_step} · 结果数 ${productCommandResultCountLabel(item.readback_result_count)}`,
    tone: item.status === "readback_unavailable" ? "warn" : "err",
  }));
  const runtimeLogItems: RightFeedItem[] = snapshot.runtime_log_store.entries
    .filter((entry) => entry.user_visible)
    .slice(0, 8)
    .map((entry) => ({
      title: `${runtimeLogCategoryLabel(entry.category)} · ${displayStatus(entry.status)}`,
      meta: `${entry.summary} · 审计引用 ${entry.audit_refs.length}`,
      tone: entry.severity === "error" ? "err" : entry.severity === "warning" ? "warn" : "ok",
    }));
  const diagnosticItems: RightFeedItem[] = snapshot.diagnostic_summary.degraded_states.slice(0, 6).map((state) => ({
    title: state.title,
    meta: `${displayStatus(state.kind)} · ${state.summary}`,
    tone: state.blocks_real_execution ? "err" : state.severity === "warning" ? "warn" : "ok",
  }));
  const notificationItems: RightFeedItem[] = [
    { title: error ? "当前读取存在问题" : "读取状态", meta: notice, tone: error ? "err" : "ok" },
    ...runtimeAttentionItems.filter((item) => item.tone !== "run").slice(0, 4),
    ...diagnosticItems.slice(0, 3),
    ...snapshot.projects.flatMap((project) =>
      [...project.context_warnings, ...project.warnings].slice(0, 2).map((warning) => ({
        title: project.name,
        meta: warning,
        tone: "warn" as const,
      })),
    ),
    ...snapshot.diagnostics.notes.slice(0, 3).map((note) => ({ title: "诊断", meta: note, tone: "warn" as const })),
  ];
  const todoItems: RightFeedItem[] = [
    ...snapshot.tasks.slice(0, 6).map((task) => ({
      title: task.title,
      meta: displayStatus(task.status),
      tone: task.status.includes("done") ? "ok" as const : "warn" as const,
    })),
    ...permissionRequests.slice(0, 6).map((request) => ({
      title: request.reason || request.request_id,
      meta: `${displayStatus(request.status)} · ${displayStatus(request.permission_kind)}`,
      tone: request.status === "approved" ? "ok" as const : "warn" as const,
    })),
    ...queueTodoItems,
    ...runtimeTodoItems.slice(0, 6),
    ...runningTasks.filter((item) => item.meta.includes("ready_for_review")).slice(0, 4),
  ];
  const ideaItems: RightFeedItem[] = [
    ...snapshot.tasks.slice(0, 8).map((task) => ({
      title: task.title,
      meta: `来自待办索引 · ${displayStatus(task.status)}`,
      tone: task.status.includes("done") ? "ok" as const : "warn" as const,
    })),
    ...secretaryContext.suggestions.slice(0, 6).map((suggestion) => ({
      title: suggestion.title,
      meta: `秘书建议 · ${suggestion.summary}`,
      tone: suggestion.priority === "high" ? "warn" as const : "ok" as const,
    })),
    ...snapshot.projects.flatMap((project) =>
      [...project.context_warnings, ...project.warnings].slice(0, 2).map((warning) => ({
        title: project.name,
        meta: `项目线索 · ${warning}`,
        tone: "warn" as const,
      })),
    ),
  ];
  const list =
    activePanel === "notifications"
      ? notificationItems
      : activePanel === "todos"
        ? todoItems
        : activePanel === "ideas"
          ? ideaItems
          : activePanel === "audit"
            ? [...diagnosticItems, ...runtimeLogItems, ...auditItems]
            : [...queueFailureItems, ...queueRunningItems, ...runtimeRunningItems, ...runningTasks];

  return (
    <div className="right-detail">
      <section className="status-pane">
        <h2>
          {rightPanelTitle(activePanel)}
          <button className="pane-close" type="button" onClick={onClose} aria-label="收起右侧详情">
            ×
          </button>
        </h2>
        <div className="right-stat-grid">
          <RightStat label="项目" value={snapshot.projects.length} />
          <RightStat label="会话" value={snapshot.sessions.length} />
          <RightStat label="项目工作流" value={workflowState?.counts.workflows ?? 0} />
          <RightStat
            label={activePanel === "audit" ? "诊断状态" : activePanel === "ideas" ? "想法线索" : "运行关注"}
            value={
              activePanel === "audit"
                ? snapshot.diagnostic_summary.degraded_states.length
                : activePanel === "ideas"
                  ? ideaItems.length
                  : snapshot.runtime_session_attention.length
            }
          />
        </div>
        <p className="muted small-note">{rightPanelBoundaryNote(activePanel)}</p>
        {workflowStateError ? <p className="rail-error">事实层读取失败：{workflowStateError}</p> : null}
      </section>
      {activePanel === "ideas" ? (
        <section className="status-pane">
          <h2>
            想法箱入口
            <span>{ideaItems.length}</span>
          </h2>
          <p className="muted small-note">
            这里先把待办、项目 warning 和秘书建议收成线索列表；不新建事实、不写正式记忆、不派发任务。
          </p>
          <div className="right-stat-grid">
            <RightStat label="待办线索" value={snapshot.tasks.length} />
            <RightStat label="秘书建议" value={secretaryContext.suggestions.length} />
            <RightStat label="项目 warning" value={snapshot.projects.reduce((count, project) => count + project.context_warnings.length + project.warnings.length, 0)} />
            <RightStat label="占位入口" value={1} />
          </div>
          <button className="secondary-button pane-action" type="button" onClick={() => onNavigate("ideas")}>
            打开想法箱
          </button>
        </section>
      ) : null}
      {activePanel === "running" ? (
        <section className="status-pane">
          <h2>
            J4 队列
            <span>{runQueue.run_queue_items.length}</span>
          </h2>
          <p className="muted small-note">
            运行队列、待确认和失败控制来自同一套派生读模型；重试、停止、恢复、重启都只进入确认，不自动执行。
          </p>
          <div className="right-stat-grid">
            <RightStat label="运行项" value={runQueue.run_queue_items.length} />
            <RightStat label="待确认" value={runQueue.user_confirmation_queue.length} />
            <RightStat label="失败控制" value={runQueue.failure_control_summaries.length} />
            <RightStat label="捕获补偿" value={runQueue.capture_compensation_count} />
          </div>
          <div className="audit-summary-list">
            {runQueue.user_confirmation_queue.slice(0, 12).map((item) => (
              <div className="audit-summary-item" key={item.confirmation_item_id}>
                <strong>{confirmationKindLabel(item.kind)}</strong>
                <span>{item.title}</span>
                <em>{item.summary}</em>
              </div>
            ))}
          </div>
        </section>
      ) : null}
      {activePanel === "running" ? (
        <section className="status-pane">
          <h2>
            统一执行链路
            <span>{productCommandStatusLabel(productCommandReadModel)}</span>
          </h2>
          <p className="muted small-note">
            这里单独展示统一执行命令状态；它不是项目工作流待办，也不是会话运行关注。
          </p>
          <div className="right-stat-grid">
            <RightStat label="命令" value={productCommandReadModel?.command_count ?? 0} />
            <RightStat label="等确认" value={productCommandReadModel?.pending_decision_count ?? 0} />
            <RightStat label="受控记录" value={productCommandReadModel?.running_attempt_count ?? 0} />
            <RightStat label="阻断" value={productCommandReadModel?.blocked_attempt_count ?? 0} />
            <RightStat label="失败" value={failureStopRetry?.failure_count ?? 0} />
            <RightStat label="读回异常" value={failureStopRetry?.readback_issue_count ?? 0} />
            <RightStat label="停止请求" value={failureStopRetry?.manual_stop_requested_count ?? 0} />
            <RightStat label="需确认" value={failureStopRetry?.retry_requires_new_user_confirmation ? 1 : 0} />
          </div>
          <p className="muted small-note">
            最近状态：{productAttemptStatusLabel(productCommandReadModel?.last_attempt_status)}；读回未知 / 不可用不能显示成 0 条结果。
          </p>
          {failureStopRetryItems.length ? (
            <div className="audit-summary-list">
              {failureStopRetryItems.map((item) => (
                <div className="audit-summary-item" key={item.kind}>
                  <strong>{item.title}</strong>
                  <span>{item.count} 条 · {item.requires_new_user_confirmation ? "需要重新确认" : "只读查看"}</span>
                  <em>{item.summary} 读回结果：{productCommandResultCountLabel(item.result_count)}</em>
                </div>
              ))}
            </div>
          ) : (
            <p className="muted small-note">统一执行链路当前没有失败、停止或重试相关产品状态。</p>
          )}
          <details className="project-dev-details">
            <summary>开发者详情：统一命令读模型</summary>
            <div className="audit-summary-list">
              <div className="audit-summary-item">
                <strong>存储版本</strong>
                <span>{productCommandReadModel?.store_revision ?? 0}</span>
                <em>边车路径：{productCommandReadModel?.sidecar_path ?? "未生成"}</em>
              </div>
              <div className="audit-summary-item">
                <strong>运行器入口</strong>
                <span>{productEntryStatusLabel(productCommandReadModel?.runner_entry_status)}</span>
                <em>旧入口：{productEntryStatusLabel(productCommandReadModel?.legacy_entry_status)}</em>
              </div>
              {failureStopRetryItems.map((item) => (
                <div className="audit-summary-item" key={item.kind}>
                  <strong>{item.kind}</strong>
                  <span>{item.source_refs.join(" / ") || "无 refs"}</span>
                  <em>{item.warnings.join(" / ") || "无 warnings"}</em>
                </div>
              ))}
            </div>
          </details>
        </section>
      ) : null}
      {activePanel === "audit" ? (
        <section className="status-pane">
          <h2>
            健康 / 诊断边界
            <span>{snapshot.diagnostic_summary.status}</span>
          </h2>
          <p className="muted small-note">G2 只读解释问题：不自动修复状态存储、不自动重试、不调用供应方，也不替代 G3 真实 Tauri 验收。</p>
          <div className="right-stat-grid">
            <RightStat label="健康" value={snapshot.diagnostic_summary.healthy_count} />
            <RightStat label="警告" value={snapshot.diagnostic_summary.warning_count} />
            <RightStat label="降级" value={snapshot.diagnostic_summary.degraded_count} />
            <RightStat label="阻断" value={snapshot.diagnostic_summary.blocked_count} />
          </div>
          <div className="audit-summary-list">
            {snapshot.diagnostic_summary.store_integrity.slice(0, 8).map((finding) => (
              <div className="audit-summary-item" key={finding.store_id}>
                <strong>{finding.label}</strong>
                <span>{displayStatus(finding.status)}</span>
                <em>{finding.item_count} 项 · 警告 {finding.warning_count}</em>
              </div>
            ))}
            {snapshot.diagnostic_summary.degraded_states.slice(0, 6).map((state) => (
              <div className="audit-summary-item" key={state.state_id}>
                <strong>{state.title}</strong>
                <span>{displayStatus(state.kind)}</span>
                <em>{state.summary}</em>
              </div>
            ))}
          </div>
          {snapshot.diagnostic_summary.boundary_notes.slice(0, 3).map((note) => (
            <p className="muted small-note" key={note}>{note}</p>
          ))}
        </section>
      ) : null}
      {activePanel === "audit" ? (
        <section className="status-pane">
          <h2>
            日志 / 审计边界
            <span>{snapshot.runtime_log_store.summaries.length}</span>
          </h2>
          <p className="muted small-note">{snapshot.runtime_log_store.boundary.separation_rule}</p>
          <div className="runtime-log-filter-row" aria-label="运行日志过滤摘要">
            {["all", ...Array.from(new Set(snapshot.runtime_log_store.summaries.map((summary) => summary.category)))].map((category) => (
              <span className="runtime-log-filter-chip" key={category}>
                {category === "all" ? "全部" : runtimeLogCategoryLabel(category)}
              </span>
            ))}
          </div>
          <div className="audit-summary-list">
            {snapshot.runtime_log_store.summaries.slice(0, 8).map((summary) => (
              <div className="audit-summary-item" key={`${summary.category}-${summary.status}-${summary.severity}`}>
                <strong>{runtimeLogCategoryLabel(summary.category)}</strong>
                <span>{displayStatus(summary.status)}</span>
                <em>{summary.entry_count} 条 · {displayStatus(summary.severity)}</em>
              </div>
            ))}
          </div>
        </section>
      ) : null}
      {activePanel === "todos" || activePanel === "running" ? (
        <ProjectGroupedSummary
          groups={activePanel === "todos" ? projectTodoGroups : projectRunningGroups}
          mode={activePanel}
          onNavigate={onNavigate}
        />
      ) : null}
      <section className="status-pane">
        <h2>
          {rightPanelFeedTitle(activePanel)}
          <span>{list.length}</span>
        </h2>
        <div className="ink-feed">
          {list.length ? (
            list.slice(0, 10).map((item, index) => (
              <button className="feed-item" type="button" key={`${item.title}-${item.meta}-${index}`} onClick={() => onNavigate(rightPanelTargetView(activePanel))}>
                <i className={item.tone} />
                <span>
                  <b>{item.title}</b>
                  <small>{item.meta}</small>
                </span>
              </button>
            ))
          ) : (
            <p className="muted small-note">未接到真实数据；这里只显示当前索引和事实层能证明的内容。</p>
          )}
        </div>
      </section>
      {activePanel === "running" ? (
        <section className="status-pane">
          <h2>
            项目运行
            <span>{(workflowState?.counts.work_items ?? 0) + snapshot.session_run_status_summaries.length}</span>
          </h2>
          {snapshot.session_run_status_summaries.slice(0, 4).map((summary) => (
            <button className="run" key={`runtime-${summary.adapter_id}-${summary.session_id}`} type="button" onClick={() => onNavigate("agents")}>
              <span className="run-title">
                <b>{summary.session_id}</b>
                <span className="who">{summary.current_status_label}</span>
              </span>
              <span className="run-meta">
                <span>{summary.attention_count} 个关注</span>
                <span>{displayStatus(summary.readback_status)}</span>
              </span>
            </button>
          ))}
          {(workflowState?.project_workflows ?? []).slice(0, 4).map((workflow) => (
            <button className="run" key={workflow.workflow_id} type="button" onClick={() => onNavigate("projects")}>
              <span className="run-title">
                <b>{workflow.title}</b>
                <span className="who">{workflow.state}</span>
              </span>
              <span className="bar">
                <i style={{ width: `${Math.min(100, Math.max(8, workflow.task_draft_count * 22))}%` }} />
              </span>
              <span className="run-meta">
                <span>{workflow.node_count} 节点</span>
                <span>{workflow.task_draft_count} 任务</span>
              </span>
            </button>
          ))}
          <button className="secondary-button pane-action" type="button" onClick={onReloadWorkflowState}>
            重新读取事实层
          </button>
        </section>
      ) : null}
    </div>
  );
}

function ProjectGroupedSummary({
  groups,
  mode,
  onNavigate,
}: {
  groups: RightProjectGroup[];
  mode: "todos" | "running";
  onNavigate: (view: ViewKey) => void;
}) {
  const itemCount = groups.reduce((count, group) => count + projectGroupItemsForMode(group, mode).length, 0);

  return (
    <section className="status-pane right-project-groups">
      <h2>
        按项目
        <span>
          {groups.length} 项目 / {itemCount} 项
        </span>
      </h2>
      <p className="muted small-note">
        只按 workflow-state 的项目工作流归组；全局会话、运行队列和诊断摘要继续留在下方。
      </p>
      {groups.length ? (
        <div className="right-project-group-list">
          {groups.slice(0, 6).map((group) => {
            const items = projectGroupItemsForMode(group, mode);
            return (
              <button className="right-project-group" type="button" key={`${mode}-${group.id}`} onClick={() => onNavigate("projects")}>
                <span className="right-project-group-head">
                  <b>{group.title}</b>
                  <small>{shortProjectRoot(group.projectRoot)}</small>
                </span>
                <span className="right-project-group-counts">
                  <span>
                    {projectGroupModeLabel(mode)} {items.length}
                  </span>
                  <span>{items.filter((item) => item.tone === "warn").length} 需看</span>
                  <span>{items.filter((item) => item.tone === "run").length} 运行</span>
                </span>
                <span className="right-project-group-preview">
                  {items.slice(0, 3).map((item) => (
                    <span key={item.id}>
                      {item.title} · {item.meta}
                    </span>
                  ))}
                </span>
              </button>
            );
          })}
        </div>
      ) : (
        <p className="muted small-note">当前没有能归属到项目工作流的{projectGroupModeLabel(mode)}项。</p>
      )}
    </section>
  );
}

function buildRightProjectGroups(workflowState: WorkflowStateSnapshot | null): RightProjectGroup[] {
  return (workflowState?.project_workflows ?? []).map((workflow) => {
    const todoTaskItems: RightProjectGroupItem[] = workflow.task_drafts
      .filter((task) => RIGHT_PROJECT_TODO_TASK_STATES.has(task.state))
      .map((task) => ({
        id: `task:${task.work_item_id}`,
        title: task.title,
        meta: displayStatus(task.state),
        tone: task.state === "waiting_for_permission" || task.state === "retry_pending" ? "warn" : "ok",
      }));
    const permissionItems: RightProjectGroupItem[] = workflow.permission_requests
      .filter((request) => isPendingProjectPermission(request.status))
      .map((request) => {
        const relatedTask = workflow.task_drafts.find((task) => task.work_item_id === request.work_item_id);
        return {
          id: `permission:${request.request_id}`,
          title: request.reason || relatedTask?.title || request.request_id,
          meta: `${displayStatus(request.status)} · ${displayStatus(request.permission_kind)}`,
          tone: "warn" as const,
        };
      });
    const runningItems: RightProjectGroupItem[] = workflow.task_drafts
      .filter((task) => RIGHT_PROJECT_RUNNING_TASK_STATES.has(task.state))
      .map((task) => ({
        id: `running:${task.work_item_id}`,
        title: task.title,
        meta: `${displayStatus(task.state)} · ${workflow.title}`,
        tone: task.state === "waiting_for_permission" ? "warn" : task.state === "running" ? "run" : "ok",
      }));

    return {
      id: workflow.workflow_id,
      title: workflow.title,
      projectRoot: workflow.project_root,
      todoItems: [...todoTaskItems, ...permissionItems],
      runningItems,
    };
  });
}

function projectGroupItemsForMode(group: RightProjectGroup, mode: "todos" | "running") {
  return mode === "todos" ? group.todoItems : group.runningItems;
}

function projectGroupModeLabel(mode: "todos" | "running") {
  return mode === "todos" ? "待办" : "运行";
}

function shortProjectRoot(projectRoot: string) {
  const parts = projectRoot.split("/").filter(Boolean);
  return parts.slice(-2).join("/") || projectRoot || "未登记路径";
}

const RIGHT_PROJECT_TODO_TASK_STATES = new Set(["waiting_for_permission", "ready_for_review", "retry_pending"]);
const RIGHT_PROJECT_RUNNING_TASK_STATES = new Set(["running", "waiting_for_permission", "retry_pending", "ready_to_dispatch", "ready_for_review"]);

function isPendingProjectPermission(status: string) {
  return !["approved", "accepted", "done", "completed", "closed", "cancelled", "canceled"].includes(status);
}

function confirmationKindLabel(value: string) {
  const labels: Record<string, string> = {
    execute_confirmation: "执行确认",
    retry_confirmation: "重试确认",
    stop_cancel_confirmation: "停止 / 取消确认",
    result_confirmation: "结果确认",
    process_fact_confirmation: "过程事实确认",
    memory_candidate_confirmation: "记忆候选确认",
    memory_formalization_confirmation: "正式化确认",
    capture_compensation_confirmation: "捕获补偿确认",
  };
  return labels[value] ?? value;
}

function failureClassificationLabel(value: string) {
  const labels: Record<string, string> = {
    blocked_by_guard: "边界阻断",
    duplicate_blocked: "重复阻断",
    failed: "运行失败",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    timed_out: "超时",
    runner_failed: "runner 失败",
    memory_capture_compensation_needed: "记忆捕获补偿",
  };
  return labels[value] ?? value;
}

function RightStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="right-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function rightPanelTitle(panel: RightPanelKey) {
  if (panel === "notifications") return "通知";
  if (panel === "todos") return "待办";
  if (panel === "ideas") return "想法箱";
  if (panel === "audit") return "管理";
  if (panel === "secretary") return "秘书只读摘要";
  return "运行中";
}

function rightPanelFeedTitle(panel: RightPanelKey) {
  if (panel === "notifications") return "通知摘要";
  if (panel === "todos") return "待处理事项";
  if (panel === "ideas") return "想法线索";
  if (panel === "audit") return "管理摘要";
  if (panel === "running") return "运行中摘要";
  return "摘要";
}

function rightPanelBoundaryNote(panel: RightPanelKey) {
  if (panel === "notifications") return "通知只解释当前读取、风险和降级状态；不触发修复或执行。";
  if (panel === "todos") return "待办只整理需要用户查看的事项；不替用户批准、派发或写入状态。";
  if (panel === "ideas") return "想法箱只收纳可见线索；不创建任务、不写事实、不替代用户确认。";
  if (panel === "audit") return "管理收纳健康、诊断、日志和审计摘要；原始材料仍在开发者区或详情中查看。";
  return "运行中只汇总工作流和会话关注项；不停止、恢复、重试或启动真实执行。";
}

function rightPanelTargetView(panel: RightPanelKey): ViewKey {
  if (panel === "running") return "projects";
  if (panel === "audit") return "settings";
  if (panel === "todos") return "projects";
  if (panel === "ideas") return "ideas";
  return "home";
}

function runtimeLogCategoryLabel(category: string) {
  if (category === "app_session") return "应用会话";
  if (category === "workflow_run") return "工作流运行";
  if (category === "dispatch_attempt") return "派发尝试";
  if (category === "readback") return "读回";
  if (category === "permission_wait") return "权限等待";
  if (category === "diagnostic_event") return "诊断事件";
  return category;
}

function productCommandStatusLabel(readModel: WorkbenchSnapshot["real_execution_product_commands"] | null | undefined) {
  if (!readModel) return "未知 / 不可用";
  if (readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return productAttemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

function productAttemptStatusLabel(status?: string | null) {
  if (!status) return "未见 attempt";
  if (status === "running_stub") return "受控记录可见";
  if (status === "succeeded_stub") return "受控记录已写入";
  if (status === "failed_stub") return "受控记录失败";
  if (status === "blocked") return "已阻断";
  if (status === "timed_out") return "读回超时";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  return status;
}

function productCommandResultCountLabel(value?: number | null) {
  return value === null || value === undefined ? "未知 / 不可用" : String(value);
}

function productEntryStatusLabel(status?: string | null) {
  if (!status) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "等待统一执行与 Level B",
  };
  return labels[status] ?? status;
}

function displayStatus(value: string | null | undefined) {
  if (!value) return "未记录";
  const labels: Record<string, string> = {
    active: "活跃",
    approved: "已批准",
    archived: "已归档",
    blocked: "阻断",
    completed: "已完成",
    degraded: "降级",
    degraded_readonly: "只读降级",
    done: "已完成",
    err: "错误",
    error: "错误",
    failed: "失败",
    healthy: "健康",
    info: "信息",
    missing: "缺失",
    neutral: "中性",
    ok: "正常",
    open: "打开",
    pending: "待处理",
    ready_for_review: "待复核",
    ready_to_dispatch: "待派发",
    retry_pending: "待重试",
    run: "运行",
    running: "运行中",
    state_未登记: "状态未登记",
    succeeded: "成功",
    timed_out: "超时",
    unknown: "未知",
    waiting_for_permission: "等待权限",
    warning: "警告",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    blocked_by_guard: "被边界阻断",
    needs_user: "需要用户处理",
    needs_user_confirmation: "需要用户确认",
    needs_review: "需要复核",
  };
  return labels[value] ?? value;
}
