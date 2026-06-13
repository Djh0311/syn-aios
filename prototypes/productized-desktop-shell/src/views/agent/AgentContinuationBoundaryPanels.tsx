import { Badge } from "../../components/Badge";
import { pathTail } from "../../lib/format";
import type {
  H2RealResumeAuthorizationReadiness,
  H2RealResumeExecutionDecisionSurface,
  RuntimeSessionAttention,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionRunStatusSummary,
  WorkerProtocolReadModel,
} from "../../lib/types";
import { readbackCountLabel } from "./TranscriptViews";
import {
  adapterContractStatusLabel,
  adapterDisplayName,
  adapterHealthStatusLabel,
  auditImpactLabel,
  controlledContinuationLabel,
  controlledContinuationTone,
  degradedModeLabel,
  eventKindLabel,
  groupSessionContinuationPreviewsByAdapter,
  guardSeverityLabel,
  h2DecisionCheckStatusLabel,
  h2DecisionCheckTone,
  h2DecisionStatusLabel,
  h2ReadinessItemStatusLabel,
  h2ReadinessItemTone,
  h2ReadinessStatusLabel,
  latestAttemptByContinuation,
  persistenceKindLabel,
  providerAvailabilityStatusLabel,
  readbackStatusLabel,
  readbackStrategyLabel,
  retryPolicyLabel,
  runtimeAttentionLabel,
  runtimeAttentionTone,
  runtimeStatusLabel,
  sessionContinuationCommandPreview,
  sessionContinuationOperationLabel,
  sessionContinuationStatusLabel,
  sessionContinuationStatusTone,
  severityLabel,
  yesNoLabel,
} from "./agentLabels";

export function SessionContinuationPreviewPanel({ previews }: { previews: SessionContinuationPreview[] }) {
  if (!previews.length) return null;
  const groups = groupSessionContinuationPreviewsByAdapter(previews);
  const blockedCount = previews.filter((preview) => preview.guard_result.status === "blocked").length;
  const needsConfirmationCount = previews.filter((preview) => preview.guard_result.status === "needs_user_confirmation").length;
  return (
    <section className="session-continuation-panel" aria-label="会话继续预览和权限预览">
      <div className="sec-head">
        <h2>会话继续预览 / 权限预览</h2>
        <span className="sec-meta">
          {previews.length} 个预览 · {needsConfirmationCount} 需要确认 · {blockedCount} 阻断
        </span>
      </div>
      <p className="session-continuation-note">
        这里是 E4 / H3.1 预览协议，不是执行入口；不会创建真实新会话，不会发送提示词，不会执行恢复，不会写 Codex 原生状态，不会写尝试、派发或读回。
      </p>
      <div className="session-continuation-grid">
        {groups.map((group) => (
          <article className="session-continuation-card" key={group.adapterId}>
            <div className="session-continuation-card-head">
              <div>
                <strong>{adapterDisplayName(group.adapterId)}</strong>
                <span>{group.adapterId} · {group.previews.length} 个继续预览</span>
              </div>
              <Badge tone={group.adapterId === "codex-local" ? "unknown" : "warning"}>
                {group.adapterId === "codex-local" ? "预览协议" : "计划中阻断"}
              </Badge>
            </div>
            <div className="session-continuation-list">
              {group.previews.map((preview) => (
                <div className={`session-continuation-item ${preview.guard_result.status}`} key={preview.preview_id}>
                  <div className="session-continuation-main">
                    <span>{sessionContinuationOperationLabel(preview.operation_id)}</span>
                    <Badge tone={sessionContinuationStatusTone(preview.guard_result.status)}>
                      {sessionContinuationStatusLabel(preview.guard_result.status)}
                    </Badge>
                    <em>{guardSeverityLabel(preview.guard_result.severity)}</em>
                  </div>
                  <div className="session-continuation-target">
                    <span>会话：{preview.target_session_title || preview.target_session_id || "未绑定"}</span>
                    <span>项目：{preview.project_id || "未绑定"}</span>
                    <span>工作流：{preview.workflow_id || "未绑定"}</span>
                    <span>节点：{preview.node_id || "未绑定"}</span>
                    <span>工作项：{preview.work_item_id || "未绑定"}</span>
                  </div>
                  <p>{preview.prompt_summary}</p>
                  <div className="session-continuation-scope">
                    <span>工作目录：{preview.target_cwd || "未定义"}</span>
                    <span>可写根目录：{preview.allowed_write_roots_summary.length ? preview.allowed_write_roots_summary.join(" / ") : "未定义"}</span>
                    <span>沙箱：{preview.sandbox_summary}</span>
                  </div>
                  <div className="session-continuation-contract">
                    <span>读回：{readbackStrategyLabel(preview.readback_expectation.strategy)}</span>
                    <span>失败边界：{retryPolicyLabel(preview.failure_handling.retry_policy)}</span>
                    <span>审计影响：{auditImpactLabel(preview.audit_impact.impact_kind)}</span>
                    <span>供应方：{preview.provider_availability_summary ? providerAvailabilityStatusLabel(preview.provider_availability_summary.availability_status) : "未登记"}</span>
                  </div>
                  {preview.operation_id === "new_session" ? (
                    <div className="session-continuation-contract">
                      <span>执行边界摘要：{sessionContinuationCommandPreview(preview)}</span>
                      <span>运行器：H3.1 空操作</span>
                      <span>提示词发送状态：否</span>
                      <span>真实 Codex 执行状态：否</span>
                      <span>写入 Codex 主目录：否</span>
                    </div>
                  ) : null}
                  <div className="session-continuation-reasons">
                    {preview.guard_result.reasons.slice(0, 5).map((reason) => (
                      <span key={reason}>{reason}</span>
                    ))}
                  </div>
                  {preview.guard_result.required_fixes.length ? (
                    <small>{preview.guard_result.required_fixes[0]}</small>
                  ) : null}
                  <div className="session-continuation-warnings">
                    {preview.user_visible_warnings.slice(0, 6).map((warning) => (
                      <span key={warning}>{warning}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

export function ControlledSessionContinuationPanel({
  store,
  previews,
}: {
  store: SessionContinuationStoreV1 | null;
  previews: SessionContinuationPreview[];
}) {
  if (!previews.length && !store?.continuations.length && !store?.attempts.length) return null;
  const continuations = store?.continuations ?? [];
  const attempts = store?.attempts ?? [];
  const latestAttempts = latestAttemptByContinuation(attempts);
  const codexPreviewCount = previews.filter((preview) => preview.adapter_id === "codex-local").length;
  const runnablePreviewCount = previews.filter(
    (preview) => preview.adapter_id === "codex-local" && preview.guard_result.status === "needs_user_confirmation",
  ).length;
  const readbackUnavailableCount = attempts.filter(
    (attempt) => attempt.readback_summary.status === "readback_unavailable" || attempt.readback_summary.status === "not_attempted_stub",
  ).length;
  return (
    <section className="controlled-continuation-panel" aria-label="E5 受控会话继续桩执行状态">
      <div className="sec-head">
        <h2>受控会话继续 / E5 Level A</h2>
        <span className="sec-meta">
          {continuations.length} 条继续记录 · {attempts.length} 次桩尝试 · 版本 {store?.revision ?? 0}
        </span>
      </div>
      <p className="controlled-continuation-note">
        这里只显示工作台自有会话继续记录和桩验收状态；真实执行未授权，不发送提示词，不执行真实恢复，不读写 Codex 原生状态。读回不可用是边界状态，不等于空读回结果。
      </p>
      <div className="controlled-continuation-summary">
        <span>codex-local 预览：{codexPreviewCount}</span>
        <span>等待用户确认：{runnablePreviewCount}</span>
        <span>读回不可用：{readbackUnavailableCount}</span>
        <span>辅助状态文件：{store?.scope.sidecar_path ? pathTail(store.scope.sidecar_path) : "session-continuations.v1.json"}</span>
      </div>
      {continuations.length ? (
        <div className="controlled-continuation-list">
          {continuations.map((continuation) => {
            const attempt = latestAttempts.get(continuation.continuation_id) ?? null;
            return (
              <article className={`controlled-continuation-card ${continuation.status}`} key={continuation.continuation_id}>
                <div className="controlled-continuation-card-head">
                  <div>
                    <strong>{sessionContinuationOperationLabel(continuation.operation_id)}</strong>
                    <span>{continuation.adapter_id} · {continuation.execution_level} · {continuation.runner_kind}</span>
                  </div>
                  <Badge tone={controlledContinuationTone(continuation.status)}>
                    {controlledContinuationLabel(continuation.status)}
                  </Badge>
                </div>
                <p>{continuation.prompt_summary}</p>
                <div className="controlled-continuation-facts">
                  <span>会话：{continuation.session_id}</span>
                  <span>项目：{continuation.project_id}</span>
                  <span>工作流：{continuation.workflow_id}</span>
                  <span>节点：{continuation.node_id}</span>
                  <span>工作目录：{continuation.target_cwd}</span>
                  <span>沙箱：{continuation.sandbox}</span>
                </div>
                <div className="controlled-continuation-facts">
                  <span>提示词发送状态：{yesNoLabel(attempt?.prompt_sent ?? false)}</span>
                  <span>真实 Codex 执行状态：{yesNoLabel(attempt?.real_codex_executed ?? false)}</span>
                  <span>写入 Codex 主目录：{yesNoLabel(attempt?.writes_codex_home ?? false)}</span>
                  <span>读回：{readbackStatusLabel(attempt?.readback_summary.status ?? "not_attempted_stub")}</span>
                </div>
                {attempt?.readback_summary.unavailable_reason ? (
                  <small>{attempt.readback_summary.unavailable_reason}</small>
                ) : (
                  <small>等待桩验收；Level B 真实执行仍需另行授权。</small>
                )}
                <div className="controlled-continuation-warnings">
                  {(attempt?.warnings ?? continuation.warnings).slice(0, 6).map((warning) => (
                    <span key={warning}>{warning}</span>
                  ))}
                </div>
              </article>
            );
          })}
        </div>
      ) : (
        <div className="controlled-continuation-empty">
          <strong>尚未创建 E5 会话继续记录</strong>
          <span>E4 预览可在用户确认后写入辅助状态文件；Level A 只能进入桩验收。</span>
          <span>计划中的适配器保持不可执行；真实发送 / 恢复仍未授权。</span>
        </div>
      )}
      {store?.warnings.length ? (
        <div className="controlled-continuation-warnings">
          {store.warnings.slice(0, 4).map((warning) => (
            <span key={warning}>{warning}</span>
          ))}
        </div>
      ) : null}
    </section>
  );
}

export function H2RealResumeAuthorizationPanel({ readiness }: { readiness: H2RealResumeAuthorizationReadiness }) {
  return (
    <section className="h2-resume-authorization-panel" aria-label="H2 真实恢复授权准备">
      <div className="sec-head">
        <h2>H2 真实恢复授权准备</h2>
        <span className="sec-meta">
          {readiness.confirmed_count} 已确认 · {readiness.missing_count} 待确认 · {readiness.blocked_count} 阻断
        </span>
      </div>
      <p className="h2-resume-authorization-note">
        {readiness.summary} 这个面板只展示执行前授权矩阵；不会发送提示词，不会执行 codex exec resume，不会读写 /Users/yoyi/.codex。
      </p>
      <div className="h2-resume-authorization-summary">
        <span>状态：{h2ReadinessStatusLabel(readiness.status)}</span>
        <span>目标会话：{readiness.target_session_id ?? "待确认"}</span>
        <span>项目目录：{readiness.target_project_root ?? "待确认"}</span>
        <span>测试样例：{readiness.recommended_fixture_path}</span>
      </div>
      <div className="h2-resume-authorization-grid">
        {readiness.readiness_items.map((item) => (
          <article className={`h2-resume-authorization-item ${item.status}`} key={item.item_id}>
            <div className="h2-resume-authorization-item-head">
              <strong>{item.label}</strong>
              <Badge tone={h2ReadinessItemTone(item.status)}>{h2ReadinessItemStatusLabel(item.status)}</Badge>
            </div>
            <span>{item.value ?? "待确认"}</span>
            <small>{item.user_visible_reason}</small>
          </article>
        ))}
      </div>
      <div className="h2-resume-authorization-warnings">
        {readiness.warnings.map((warning) => (
          <span key={warning}>{warning}</span>
        ))}
      </div>
    </section>
  );
}

export function H2RealResumeExecutionDecisionPanel({ surface }: { surface: H2RealResumeExecutionDecisionSurface }) {
  return (
    <section className="h2-execution-decision-panel" aria-label="H2.8 真实恢复最终批准前决策面">
      <div className="sec-head">
        <h2>H2.8 最终批准决策面</h2>
        <span className="sec-meta">
          {h2DecisionStatusLabel(surface.status)} · {surface.duplicate_attempt_count} 次重复尝试
        </span>
      </div>
      <p className="h2-execution-decision-note">
        {surface.summary} 这里是权限弹层、审计摘要、运行日志预览和读回边界的只读材料；不批准、不执行、不发送提示词、不读写 /Users/yoyi/.codex。
      </p>
      <div className="h2-execution-decision-summary">
        <span>适配器：{surface.adapter_id}</span>
        <span>操作：{sessionContinuationOperationLabel(surface.operation_id)}</span>
        <span>授权：{h2ReadinessStatusLabel(surface.authorization_status)}</span>
        <span>最终批准：{surface.final_approval_allowed ? "材料齐备但仍需明确确认" : "当前不可批准"}</span>
        <span>目标会话：{surface.target_session_id ?? "待确认"}</span>
        <span>重复保护：{surface.duplicate_attempt_blocked ? "阻断" : "无排队 / 运行中的真实尝试"}</span>
      </div>
      <div className="h2-execution-decision-grid">
        {surface.decision_checks.map((check) => (
          <article className={`h2-execution-decision-check ${check.status}`} key={check.check_id}>
            <div className="h2-execution-decision-check-head">
              <strong>{check.label}</strong>
              <Badge tone={h2DecisionCheckTone(check.status, check.blocks_final_approval)}>
                {h2DecisionCheckStatusLabel(check.status, check.blocks_final_approval)}
              </Badge>
            </div>
            <span>{check.value ?? "待确认"}</span>
            <small>{check.user_visible_reason}</small>
          </article>
        ))}
      </div>
      <div className="h2-execution-decision-columns">
        <article className="h2-execution-decision-card">
          <strong>权限弹层预览</strong>
          <span>操作：{surface.permission_preview.operation_label}</span>
          <span>项目：{surface.permission_preview.target_project}</span>
          <span>工作流 / 节点：{surface.permission_preview.workflow_label} / {surface.permission_preview.node_label}</span>
          <span>工作项：{surface.permission_preview.work_item_label}</span>
          <span>会话：{surface.permission_preview.target_session_summary}</span>
          <span>工作目录：{surface.permission_preview.target_cwd}</span>
          <span>可写根目录：{surface.permission_preview.allowed_write_roots.length ? surface.permission_preview.allowed_write_roots.join(" / ") : "待确认"}</span>
          <span>拒绝路径：{surface.permission_preview.denied_paths.join(" / ")}</span>
          <span>提示词：{surface.permission_preview.prompt_summary}</span>
          <span>提示词引用 / 哈希：{surface.permission_preview.prompt_ref} / {surface.permission_preview.prompt_hash}</span>
          <span>任务记忆包：{surface.permission_preview.task_memory_packet_summary}</span>
          <span>.codex：{surface.permission_preview.codex_home_scope_summary}</span>
          <span>沙箱 / 超时：{surface.permission_preview.sandbox_summary} / {surface.permission_preview.timeout_summary}</span>
          <span>重复保护：{surface.permission_preview.duplicate_guard_summary}</span>
          <span>批准后：{surface.permission_preview.approval_effect}</span>
          <span>拒绝后：{surface.permission_preview.rejection_effect}</span>
          <span>阻断后：{surface.permission_preview.blocked_effect}</span>
        </article>
        <article className="h2-execution-decision-card">
          <strong>审计 / 运行日志 / 读回预览</strong>
          <span>审计：{surface.audit_runtime_preview.audit_preview.join(" / ")}</span>
          <span>运行日志：{surface.audit_runtime_preview.runtime_log_preview.join(" / ")}</span>
          <span>读回：{surface.audit_runtime_preview.readback_preview.join(" / ")}</span>
          <span>证据：{surface.audit_runtime_preview.evidence_preview.join(" / ")}</span>
          <span>回滚：{surface.audit_runtime_preview.rollback_preview.join(" / ")}</span>
          <span>读回状态：{surface.readback_boundary.display_label}</span>
          <span>结果数：{readbackCountLabel(surface.readback_boundary.result_count)}</span>
          <span>{surface.readback_boundary.user_message}</span>
          <span>{surface.planned_adapter_boundary}</span>
        </article>
      </div>
      <div className="h2-execution-decision-warnings">
        {[...surface.permission_preview.warnings, ...surface.readback_boundary.warnings, ...surface.warnings].map((warning) => (
          <span key={warning}>{warning}</span>
        ))}
      </div>
    </section>
  );
}

export function RuntimeSessionAttentionPanel({
  attention,
  summaries,
}: {
  attention: RuntimeSessionAttention[];
  summaries: SessionRunStatusSummary[];
}) {
  if (!attention.length && !summaries.length) return null;
  const blockingCount = attention.filter((item) => item.blocks_continuation || item.severity === "blocking").length;
  const needsUserCount = attention.filter((item) => item.requires_user_action || item.severity === "needs_user").length;
  const readbackUnavailableCount = attention.filter((item) => item.readback_boundary.status === "readback_unavailable").length;
  const readbackFailedCount = attention.filter((item) => item.readback_boundary.status === "readback_failed").length;
  return (
    <section className="runtime-attention-panel" aria-label="E6 运行会话关注">
      <div className="sec-head">
        <h2>运行关注 / E6</h2>
        <span className="sec-meta">
          {attention.length} 条关注 · {summaries.length} 条会话摘要
        </span>
      </div>
      <p className="runtime-attention-note">
        这里聚合 E4 预览、E5 会话继续和读回边界；只解释等待、桩执行、边界保护、失败 / 不可用状态，不显示原始日志，不自动重试，不执行停止或恢复。
      </p>
      <div className="runtime-attention-summary">
        <span>阻断：{blockingCount}</span>
        <span>需要用户：{needsUserCount}</span>
        <span>读回不可用：{readbackUnavailableCount}</span>
        <span>读回失败：{readbackFailedCount}</span>
      </div>
      {summaries.length ? (
        <div className="runtime-session-summary-list">
          {summaries.slice(0, 4).map((summary) => (
            <article className="runtime-session-summary-card" key={`${summary.adapter_id}:${summary.session_id}`}>
              <div>
                <strong>{summary.session_id}</strong>
                <span>{summary.adapter_id} · {runtimeAttentionLabel(summary.current_status) || summary.current_status_label}</span>
              </div>
              <Badge tone={runtimeAttentionTone(summary.current_status)}>
                {runtimeAttentionLabel(summary.current_status)}
              </Badge>
              <small>
                关注 {summary.attention_count} · 阻断 {summary.blocking_count} · 需要用户 {summary.needs_user_count} · 读回 {readbackStatusLabel(summary.readback_status)}
              </small>
            </article>
          ))}
        </div>
      ) : null}
      <div className="runtime-attention-list">
        {attention.slice(0, 6).map((item) => (
          <article className={`runtime-attention-card ${item.status}`} key={item.attention_id}>
            <div className="runtime-attention-card-head">
              <div>
                <strong>{item.title}</strong>
                <span>{item.adapter_id} · {item.session_id ?? "未绑定会话"} · {runtimeAttentionLabel(item.status)}</span>
              </div>
              <Badge tone={runtimeAttentionTone(item.status)}>
                {runtimeAttentionLabel(item.status)}
              </Badge>
            </div>
            <p>{item.user_message}</p>
            <small>{item.recommended_next_step}</small>
            <div className="runtime-attention-flags">
              <span>结果数：{readbackCountLabel(item.readback_boundary.result_count)}</span>
              <span>真实读回：{yesNoLabel(item.readback_boundary.real_readback_performed)}</span>
              <span>{item.readback_boundary.reason}</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

export function AdapterSdkCliDiagnosticsPanel({ workerProtocol }: { workerProtocol?: WorkerProtocolReadModel | null }) {
  if (!workerProtocol) return null;
  const checklists = workerProtocol.adapter_contract_checklists ?? [];
  const semantics = workerProtocol.controlled_api_cli_semantics ?? [];
  const eventSchemas = workerProtocol.diagnostic_event_schemas ?? [];
  const healthSummaries = workerProtocol.adapter_health_summaries ?? [];
  const degradedModes = workerProtocol.adapter_degraded_modes ?? [];
  const dataLocations = workerProtocol.adapter_data_locations ?? [];
  if (
    !checklists.length &&
    !semantics.length &&
    !eventSchemas.length &&
    !healthSummaries.length &&
    !degradedModes.length &&
    !dataLocations.length
  ) {
    return null;
  }
  const blockedContractCount = checklists.filter((item) => item.status !== "ready_for_controlled_adapter_contract").length;
  const blockedHealthCount = healthSummaries.filter((item) => item.status !== "available_with_guard").length;
  const backdoorBlockedCount = semantics.filter((item) => item.universal_api_backdoor_blocked).length;
  return (
    <section className="adapter-sdk-diagnostics-panel" aria-label="I5 适配器 SDK 命令行诊断契约">
      <div className="sec-head">
        <h2>适配器 SDK / 命令行 / 诊断预留</h2>
        <span className="sec-meta">
          {checklists.length} 个清单 · {blockedContractCount} 个契约阻断 · {blockedHealthCount} 个健康阻断
        </span>
      </div>
      <p className="adapter-sdk-diagnostics-note">
        I5 只定义未来适配器接入的契约、命令行对齐和诊断事件结构；它不提供通用执行接口，不绕过控制核心、权限、运行日志或审计，也不读取密钥或会话原文。
      </p>
      <div className="adapter-sdk-diagnostics-summary">
        <span>命令行对齐：{backdoorBlockedCount} 个明确阻断通用 API 后门</span>
        <span>诊断结构：{eventSchemas.length} 个适配器预留</span>
        <span>数据位置：{dataLocations.length} 个只读位置描述</span>
        <span>降级模式：{degradedModes.filter((mode) => mode.blocks_real_execution).length} 个阻断真实执行</span>
      </div>
      <div className="adapter-sdk-diagnostics-grid">
        {checklists.map((checklist) => {
          const cli = semantics.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const health = healthSummaries.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const degraded = degradedModes.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const location = dataLocations.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          const schema = eventSchemas.find((item) => item.adapter_id === checklist.adapter_id) ?? null;
          return (
            <article className={`adapter-sdk-diagnostics-card ${checklist.status}`} key={checklist.checklist_id}>
              <div className="adapter-sdk-diagnostics-card-head">
                <div>
                  <strong>{adapterDisplayName(checklist.adapter_id)}</strong>
                  <span>{checklist.adapter_id} · {adapterContractStatusLabel(checklist.status)}</span>
                </div>
                <Badge tone={checklist.status === "ready_for_controlled_adapter_contract" ? "candidate" : "warning"}>
                  {adapterContractStatusLabel(checklist.status)}
                </Badge>
              </div>
              <div className="adapter-sdk-diagnostics-flags">
                <span>控制核心：{yesNoLabel(checklist.control_core_required)}</span>
                <span>权限：{yesNoLabel(checklist.permission_required)}</span>
                <span>审计：{yesNoLabel(checklist.audit_required)}</span>
                <span>运行日志：{yesNoLabel(checklist.runtime_log_required)}</span>
                <span>凭据边界：{yesNoLabel(checklist.credential_boundary_defined)}</span>
                <span>模型边界：{yesNoLabel(checklist.model_boundary_defined)}</span>
                <span>数据位置：{yesNoLabel(checklist.data_location_defined)}</span>
              </div>
              <p>{health?.degraded_reason ?? degraded?.user_visible_summary ?? "契约材料仅用于后续适配器接入设计。"}</p>
              <div className="adapter-sdk-diagnostics-contract">
                <span>命令行：{cli?.parity_status ?? "未登记"}</span>
                <span>控制核心路径：{cli?.control_core_path ?? "需要控制核心"}</span>
                <span>权限路径：{cli?.permission_path ?? "需要权限"}</span>
                <span>审计路径：{cli?.audit_path ?? "需要审计"}</span>
                <span>后门阻断：{yesNoLabel(cli?.universal_api_backdoor_blocked ?? true)}</span>
              </div>
              <div className="adapter-sdk-diagnostics-contract">
                <span>健康：{adapterHealthStatusLabel(health?.status)} / {severityLabel(health?.severity)}</span>
                <span>运行：{runtimeStatusLabel(health?.runtime_status)}</span>
                <span>降级：{degradedModeLabel(degraded?.mode)}</span>
                <span>持久化：{persistenceKindLabel(location?.persistence_kind)}</span>
                <span>结构：{schema?.event_kinds.slice(0, 3).map(eventKindLabel).join(" / ") ?? "未登记"}</span>
              </div>
              <div className="adapter-sdk-diagnostics-warnings">
                {[
                  ...checklist.missing_items,
                  ...checklist.warnings,
                  ...(cli?.warnings ?? []),
                  ...(health?.warnings ?? []),
                  ...(degraded?.warnings ?? []),
                  ...(schema?.warnings ?? []),
                  ...(location?.warnings ?? []),
                ]
                  .slice(0, 12)
                  .map((warning) => (
                    <span key={warning}>{warning}</span>
                  ))}
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
