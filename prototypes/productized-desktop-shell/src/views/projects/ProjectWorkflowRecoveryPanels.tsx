import { Badge } from "../../components/Badge";
import { DetailLine as PrimitiveDetailLine, SummaryTile } from "../../components/WorkbenchPrimitives";
import type { K3B1RecoveryReadModel } from "../../lib/types";
import { ProjectCanvasDetailLine } from "./ProjectCanvasDetailPrimitives";
import type { ProjectWorkflowCanvasSidePanelProps } from "./ProjectWorkflowCanvasView";

export function K3B1RecoveryCard({
  recovery,
  projectRoot,
  onRequestAction,
}: {
  recovery: K3B1RecoveryReadModel | null;
  projectRoot: string;
  onRequestAction: ProjectWorkflowCanvasSidePanelProps["onRequestAction"];
}) {
  if (!recovery || recovery.current_state !== "blocked_by_safety_review_again") return null;
  const manualOption = recovery.recovery_options.find((option) => option.option_id === "manual_exact_command_submission");
  const renewedOption = recovery.recovery_options.find((option) => option.option_id === "renewed_risk_approval_request");
  const bridgeOption = recovery.recovery_options.find((option) => option.option_id === "narrow_local_bridge_design");
  const resultCount =
    recovery.readback_boundary.result_count === null || recovery.readback_boundary.result_count === undefined
      ? "未知/不可用"
      : `${recovery.readback_boundary.result_count} 条`;
  const exactCommandText = recovery.manual_exact_command.command_lines.join("\n");

  return (
    <section className="project-canvas-detail-card k3-b1-recovery-card" aria-label="K3-B1 阻断恢复路径">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">K3-B1 恢复路径</p>
          <h3>K3-B1 被安全审查再次阻断</h3>
          <p className="path-text">这不是执行失败可自动重试，也不是已完成恢复。</p>
        </div>
        <Badge tone="warning">阻断</Badge>
      </div>
      <p className="muted small-note">
        阻断原因：真实 Codex resume 会向外部服务发送项目/session 派生提示词，并写入 Codex 本地状态。
      </p>
      <div className="workflow-draft-grid">
        <SummaryTile label="当前状态" value="安全审查再次阻断" hint="blocked_by_safety_review_again" />
        <SummaryTile label="K3-B2" value="继续阻断" hint={recovery.k3_b2_gate.reason} />
        <SummaryTile label="读回结果" value={`结果数：${resultCount}`} hint={recovery.readback_boundary.unavailable_reason} />
      </div>
      <div className="project-canvas-detail-sections">
        <article className="project-canvas-detail-section user_summary">
          <strong>现在能做什么</strong>
          <ProjectCanvasDetailLine
            item={{
              item_id: "manual-recovery",
              label: "手动运行并回交",
              value: manualOption?.user_visible_description ?? "用户手动运行并回交，主管线复核前不改变成功状态。",
              source_refs: [],
            }}
          />
          <ProjectCanvasDetailLine
            item={{
              item_id: "renewed-risk",
              label: "重新授权申请",
              value: renewedOption?.user_visible_description ?? "只进入待重新授权/待安全审查状态。",
              source_refs: [],
            }}
          />
          <ProjectCanvasDetailLine
            item={{
              item_id: "narrow-bridge",
              label: "更窄本地执行桥",
              value: bridgeOption?.user_visible_description ?? "只作为后续设计候选，不能绕过安全审查。",
              source_refs: [],
            }}
          />
        </article>
        <article className="project-canvas-detail-section project_director">
          <strong>回交要求</strong>
          <ProjectCanvasDetailLine
            item={{
              item_id: "exact-command-workdir",
              label: "执行目录",
              value: recovery.manual_exact_command.working_directory,
              source_refs: [],
            }}
          />
          <ProjectCanvasDetailLine
            item={{
              item_id: "exact-command-prompt",
              label: "Prompt 引用",
              value: `${recovery.manual_exact_command.prompt_ref} · ${recovery.manual_exact_command.prompt_hash}`,
              source_refs: [],
            }}
          />
          <p className="muted small-note">{recovery.manual_exact_command.boundary}</p>
          <pre className="task-preview-code" aria-label="K3-B1 exact command">{exactCommandText}</pre>
          <ProjectCanvasDetailLine
            item={{
              item_id: "required-fields",
              label: "必填字段",
              value: recovery.manual_submission_contract.required_fields.join(" / "),
              source_refs: [],
            }}
          />
          <ProjectCanvasDetailLine
            item={{
              item_id: "manual-review",
              label: "主管线复核",
              value: "exit_code=0 仍不自动成功；必须结合 last message、marker、hash 和主管复核。",
              source_refs: [],
            }}
          />
          <ProjectCanvasDetailLine
            item={{
              item_id: "sensitive-policy",
              label: "敏感材料",
              value: recovery.manual_submission_contract.sensitive_material_policy,
              source_refs: [],
            }}
          />
        </article>
      </div>
      <div className="workflow-state-actions">
        <button
          className="secondary-button"
          type="button"
          onClick={() =>
            onRequestAction({
              kind: "record-k3-b1-manual-recovery-submission",
              label: "记录 K3-B1 手动回交进入待复核",
              path: projectRoot,
              source: "索引内项目路径",
              boundary:
                "只记录用户准备按 exact command 回交材料的产品路径；不执行 codex exec/resume、不发送提示词、不写 .codex、不自动接受成功。",
              k3B1RecoveryAction: {
                execution_point_id: recovery.execution_point_id,
                recovery_choice: "manual_exact_command_submission",
                status_after_selection: recovery.manual_submission_contract.status_after_submit,
                risk_acknowledgement: "manual_submission_requires_supervisor_review",
                required_fields: recovery.manual_submission_contract.required_fields,
                readback_result_count: recovery.readback_boundary.result_count ?? null,
              },
            })
          }
        >
          记录手动回交待复核
        </button>
        <button
          className="secondary-button"
          type="button"
          onClick={() =>
            onRequestAction({
              kind: "request-k3-b1-renewed-risk-approval",
              label: "准备 K3-B1 重新授权申请",
              path: projectRoot,
              source: "索引内项目路径",
              boundary:
                "只记录重新授权申请意图；用户需另窗明确批准提示词外发和 .codex 写入风险，L1 不启动真实 retry。",
              k3B1RecoveryAction: {
                execution_point_id: recovery.execution_point_id,
                recovery_choice: "renewed_risk_approval_request",
                status_after_selection: recovery.renewed_risk_approval.status_after_request,
                risk_acknowledgement: recovery.renewed_risk_approval.warning,
                readback_result_count: recovery.readback_boundary.result_count ?? null,
              },
            })
          }
        >
          准备重新授权说明
        </button>
      </div>
      <details className="project-canvas-detail-layer technical_details">
        <summary>
          <span>开发者字段</span>
          <em>只显示引用和 hash，不显示提示词正文、完整会话记录或 .codex 内容</em>
        </summary>
        <div className="workflow-draft-grid">
          {recovery.developer_details.map((detail) => (
            <PrimitiveDetailLine label={detail.label} value={detail.value} key={detail.label} />
          ))}
          <PrimitiveDetailLine label="运行日志" value={recovery.runtime_boundary.allowed_summary} />
          <PrimitiveDetailLine label="审计事件" value={recovery.audit_boundary.event_type} />
          <PrimitiveDetailLine label="记忆捕获" value={recovery.memory_capture_boundary.suggested_candidate_text} />
        </div>
      </details>
      {recovery.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </section>
  );
}
