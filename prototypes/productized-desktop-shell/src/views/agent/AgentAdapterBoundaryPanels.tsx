import { Pill } from "../../components/SpecPrimitives";
import type {
  AgentAdapterDescriptor,
  ProviderAvailabilitySummary,
  SessionOperationDescriptor,
} from "../../lib/types";
import {
  adapterCredentialStatusLabel,
  adapterDisplayName,
  adapterExecutionStatusLabel,
  adapterModelStatusLabel,
  adapterStatusLabel,
  adapterStatusTone,
  capabilityStatusLabel,
  costRiskStatusLabel,
  credentialBoundaryStatusLabel,
  externalCallStatusLabel,
  groupSessionOperationsByAdapter,
  modelAvailabilityStatusLabel,
  providerAvailabilityStatusLabel,
  providerAvailabilityTone,
  sessionOperationFlags,
  sessionOperationRiskLabel,
  sessionOperationStatusLabel,
  sessionOperationStatusTone,
} from "./agentLabels";

export function AgentAdapterCapabilityPanel({ descriptors }: { descriptors: AgentAdapterDescriptor[] }) {
  if (!descriptors.length) return null;
  return (
    <section className="adapter-capability-panel" aria-label="适配器能力声明">
      <div className="sec-head">
        <h2>适配器能力</h2>
        <span className="sec-meta">{descriptors.length} 个适配器描述</span>
      </div>
      <div className="adapter-capability-grid">
        {descriptors.map((descriptor) => (
          <article className="adapter-card" key={descriptor.adapter_id}>
            <div className="adapter-card-head">
              <div>
                <strong>{descriptor.display_name}</strong>
                <span>{descriptor.provider}</span>
              </div>
              <Pill tone={adapterStatusTone(descriptor.status)}>{adapterStatusLabel(descriptor.status)}</Pill>
            </div>
            <div className="adapter-status-grid">
              <span>执行：{adapterExecutionStatusLabel(descriptor.execution_status)}</span>
              <span>凭据：{adapterCredentialStatusLabel(descriptor.credential_status)}</span>
              <span>模型：{adapterModelStatusLabel(descriptor.model_access_status)}</span>
            </div>
            <div className="adapter-capability-list">
              {descriptor.capabilities.length ? (
                descriptor.capabilities.map((capability) => (
                  <div className={`adapter-capability-item ${capability.status}`} key={capability.capability_id}>
                    <span>{capability.label}</span>
                    <strong>{capabilityStatusLabel(capability.status)}</strong>
                    <em>{capability.description}</em>
                    <small>{capability.boundary}</small>
                  </div>
                ))
              ) : (
                <div className="adapter-empty-state">
                  <span>当前不可执行</span>
                  <small>计划中的适配器只有只读描述；没有真实命令、会话、凭据或模型调用。</small>
                </div>
              )}
            </div>
            <div className="adapter-boundary-list">
              <span>已实现动作：{descriptor.implemented_action_kinds.length ? descriptor.implemented_action_kinds.join(" / ") : "无"}</span>
              {descriptor.hidden_unimplemented_adapters.length ? (
                <span>未实现适配器清单：{descriptor.hidden_unimplemented_adapters.join(" / ")}</span>
              ) : null}
              <span>{descriptor.permission_boundary}</span>
              {descriptor.requires_user_setup ? <span>需要后续授权任务或用户设置。</span> : null}
              {descriptor.unavailable_reason ? <span>不可用原因：{descriptor.unavailable_reason}</span> : null}
              {descriptor.warnings.map((warning) => (
                <span key={warning}>{warning}</span>
              ))}
            </div>
            <details className="agent-boundary-details nested-boundary-details">
              <summary className="agent-boundary-summary">开发者详情</summary>
              <div className="adapter-boundary-list">
                <span>adapter_id={descriptor.adapter_id}</span>
                <span>source_kind={descriptor.source_kind}</span>
              </div>
            </details>
          </article>
        ))}
      </div>
    </section>
  );
}

export function ProviderAvailabilityPanel({ summaries }: { summaries: ProviderAvailabilitySummary[] }) {
  const visibleSummaries = summaries.filter((summary) => summary.safe_to_display);
  if (!visibleSummaries.length) return null;
  const plannedCount = visibleSummaries.filter((summary) => summary.availability_status === "planned").length;
  const blockedExternalCallCount = visibleSummaries.filter((summary) => summary.external_call_status === "external_call_blocked").length;
  return (
    <section className="provider-availability-panel" aria-label="供应方模型凭据边界">
      <div className="sec-head">
        <h2>供应方 / 模型 / 凭据边界</h2>
        <span className="sec-meta">{visibleSummaries.length} 个供应方 · {plannedCount} 个计划中 · {blockedExternalCallCount} 个外发阻断</span>
      </div>
      <p className="provider-availability-note">
        这里只显示只读供应方可用性。它不等于项目授权、任务授权或会话操作能力；工作台不读取密钥，不验证模型，也不发起供应方调用。
      </p>
      <div className="provider-availability-grid">
        {visibleSummaries.map((summary) => (
          <article className={`provider-availability-card ${summary.availability_status}`} key={summary.adapter_id}>
            <div className="provider-availability-card-head">
              <div>
                <strong>{summary.provider_label}</strong>
              </div>
              <Pill tone={providerAvailabilityTone(summary.availability_status)}>
                {providerAvailabilityStatusLabel(summary.availability_status)}
              </Pill>
            </div>
            <div className="provider-status-grid">
              <span>凭据：{credentialBoundaryStatusLabel(summary.credential_status)}</span>
              <span>模型：{modelAvailabilityStatusLabel(summary.model_status)}</span>
              <span>外发：{externalCallStatusLabel(summary.external_call_status)}</span>
              <span>成本：{costRiskStatusLabel(summary.cost_risk_status)}</span>
            </div>
            <p>{summary.user_visible_reason}</p>
            <div className="provider-boundary-list">
              {summary.requires_user_configuration ? <span>需要后续授权任务或用户设置。</span> : <span>工作台不要求读取凭据。</span>}
              {summary.requires_future_task ? <span>真实调用或会话发送需要后续任务。</span> : null}
              {summary.warnings.map((warning) => (
                <span key={warning}>{warning}</span>
              ))}
            </div>
            <details className="agent-boundary-details nested-boundary-details">
              <summary className="agent-boundary-summary">开发者详情</summary>
              <div className="provider-boundary-list">
                <span>adapter_id={summary.adapter_id}</span>
                <span>provider_id={summary.provider_id}</span>
                <span>provider_kind={summary.provider_kind}</span>
              </div>
            </details>
          </article>
        ))}
      </div>
    </section>
  );
}

export function SessionOperationBoundaryPanel({ operations }: { operations: SessionOperationDescriptor[] }) {
  if (!operations.length) return null;
  const operationIds = new Set(operations.map((operation) => operation.operation_id));
  const blockedCount = operations.filter((operation) => operation.current_status !== "readonly_available").length;
  const groups = groupSessionOperationsByAdapter(operations);
  return (
    <section className="session-operation-panel" aria-label="会话操作边界">
      <div className="sec-head">
        <h2>会话操作边界</h2>
        <span className="sec-meta">{operationIds.size} 个操作 · {blockedCount} 当前不可执行或计划中</span>
      </div>
      <p className="session-operation-note">
        会话中心仍是只读历史浏览器；这里定义权限、审计和数据影响边界，不执行新建会话、发消息、停止、重启、恢复、导出、删除或收藏。
      </p>
      <div className="session-operation-grid">
        {groups.map((group) => (
          <article className="session-operation-card" key={group.adapterId}>
            <div className="session-operation-card-head">
              <div>
                <strong>{adapterDisplayName(group.adapterId)}</strong>
                <span>{group.operations.length} 个操作边界</span>
              </div>
              <Pill tone={group.adapterId === "codex-local" ? "unknown" : "warn"}>
                {group.adapterId === "codex-local" ? "只读边界" : "计划中不可执行"}
                    </Pill>
            </div>
            <div className="session-operation-list">
              {group.operations.map((operation) => (
                <div className={`session-operation-item ${operation.current_status}`} key={`${operation.adapter_id}:${operation.operation_id}`}>
                  <div className="session-operation-main">
                    <span>{operation.label}</span>
                    <Pill tone={sessionOperationStatusTone(operation.current_status)}>
                      {sessionOperationStatusLabel(operation.current_status)}
              </Pill>
                    <em>{sessionOperationRiskLabel(operation.risk_level)}</em>
                  </div>
                  <p>{operation.unavailable_reason}</p>
                  <small>{operation.future_task_hint}</small>
                  <div className="session-operation-flags">
                    {sessionOperationFlags(operation).map((flag) => (
                      <span key={flag}>{flag}</span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
            <details className="agent-boundary-details nested-boundary-details">
              <summary className="agent-boundary-summary">开发者详情</summary>
              <div className="session-operation-flags">
                <span>adapter_id={group.adapterId}</span>
              </div>
            </details>
          </article>
        ))}
      </div>
    </section>
  );
}
