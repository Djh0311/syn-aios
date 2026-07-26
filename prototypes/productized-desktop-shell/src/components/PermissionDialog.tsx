import type { PendingAction } from "../lib/types";
import { SegTitle } from "./SpecPrimitives";
import { summarizeProjectDirectorTaskPlan } from "../lib/projectDirectorTaskPlan";

type PermissionDialogProps = {
  action: PendingAction | null;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function PermissionDialog({ action, busy, onCancel, onConfirm }: PermissionDialogProps) {
  if (!action) return null;

  const authorizedAutoDispatchSummary =
    action.kind === "prepare-authorized-auto-dispatch"
      ? summarizeProjectDirectorTaskPlan(action.authorizedAutoDispatchPreview ?? null)
      : null;
  const memoryCandidateCreationSource =
    action.kind === "create-memory-candidate" ? action.memoryCandidateCreation?.source_refs[0] ?? null : null;
  // E 定稿的三段对齐(标题「允许它动这些吗?」+「要动什么」)**只对高危授权 kind 生效**(用户 07-15 拍)。
  // 高危集合 = { execute-node-dispatch, run-workflow-machine } —— 判据不是我拍的,是源码自己已经分好的组:
  // 只有这两个 kind 会得到 realCodexBoundary(「会触发真实 Codex 恢复」/「会写 /Users/yoyi/.codex」),
  // 也只有这两个把取消键从「取消」升级成「拒绝」,确认键说「真实派发」/「启动多轮真实执行」。
  // 对齐 AGENTS.md 高危清单 #1(真实项目真执行)/#2(写 .codex)/#4(自动连环)。
  // 其余约 40 种通用确认 kind(确认复制/记录决定/创建候选…)一律走原形态——给它们套「允许它动这些吗?」是错的。
  const isHighRiskAuthorization = action.kind === "execute-node-dispatch" || action.kind === "run-workflow-machine";
  const realCodexBoundary =
    action.kind === "execute-node-dispatch" || action.kind === "run-workflow-machine"
      ? {
          trigger: "会触发真实 Codex 恢复",
          codexHome: "会写 /Users/yoyi/.codex；只有本弹层确认后才会进入真实执行。",
          failure: "失败、超时、读回不可用或读回失败必须记录边界；不能显示成 0 条结果，也不会自动重试。",
        }
      : null;
  const k3B1RecoveryBoundary =
    action.kind === "record-k3-b1-manual-recovery-submission" ||
    action.kind === "request-k3-b1-renewed-risk-approval"
      ? {
          status: action.k3B1RecoveryAction?.status_after_selection ?? "blocked_by_safety_review_again",
          risk: action.k3B1RecoveryAction?.risk_acknowledgement ?? "L1 只记录恢复路径，不执行真实 Codex。",
          readback:
            action.k3B1RecoveryAction?.readback_result_count === null ||
            action.k3B1RecoveryAction?.readback_result_count === undefined
              ? "结果数：未知/不可用"
              : `结果数：${action.k3B1RecoveryAction.readback_result_count} 条`,
        }
      : null;
  const operationControlBoundary =
    action.kind === "record-operation-control-decision" && action.operationControlAction
      ? {
          operationId: action.operationControlAction.operation_id,
          currentStatus: action.operationControlAction.current_status,
          afterStatus: action.operationControlAction.status_after_confirmation,
          gate: action.operationControlAction.current_gate,
          wouldWrite: action.operationControlAction.would_write_if_real,
          risk: action.operationControlAction.risk_disclosure,
          readback:
            action.operationControlAction.readback_result_count === null ||
            action.operationControlAction.readback_result_count === undefined
              ? `读回：${action.operationControlAction.readback_status}；结果数：未知/不可用`
              : `读回：${action.operationControlAction.readback_status}；结果数：${action.operationControlAction.readback_result_count} 条`,
          audit: action.operationControlAction.audit_event_type,
          runtime: action.operationControlAction.runtime_status_after_confirmation,
        }
      : null;

  return (
    <div
      className="dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && !busy) onCancel();
      }}
    >
      <section
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="permission-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        {/* Zone 1: What + Why。高危授权 = 定稿 E 的问句上脸(唯一问题:允许它动这些吗),
            具体动作降为 eyebrow 交代上下文;其余 kind 维持原形态不动。 */}
        <div className="dialog-zone-what">
          <p className="eyebrow">{isHighRiskAuthorization ? action.label : "本机动作确认"}</p>
          <h2 id="permission-title">{isHighRiskAuthorization ? "允许它动这些吗？" : action.label}</h2>
        </div>

        {/* Zone 2: Detail rows */}
        <div className="dialog-zone-details">
        {/* 定稿 E 第二段「要动什么」(第一屏永远可见)。下面这些行本来就是「要动什么」的事实,
            这里只把段落名给它们——不重排、不复制一份新行(复制=同一事实两处漂移)。
            定稿的「新建」「工具」两行**无源**故不做:`PendingAction` 没有「新建」字段;
            `allowed_tools` 只挂在 projectConsultationProposalCreation.scope_draft /
            globalBoundaryReviewPreview.toolsAndChecks 上,不在 nodeDispatch —— 编不得。 */}
        {isHighRiskAuthorization ? <SegTitle>要动什么</SegTitle> : null}
        <div className="permission-detail">
          <span>目标路径</span>
          <strong>{action.path}</strong>
        </div>
        <div className="permission-detail">
          <span>路径来源</span>
          <strong>{action.source}</strong>
        </div>
        {action.boundary ? (
          <div className="permission-detail">
            <span>写入边界</span>
            <strong>{action.boundary}</strong>
          </div>
        ) : null}
        {realCodexBoundary ? (
          <>
            <div className="permission-detail">
              <span>真实 Codex</span>
              <strong>{realCodexBoundary.trigger}</strong>
            </div>
            <div className="permission-detail">
              <span>Codex 主目录</span>
              <strong>{realCodexBoundary.codexHome}</strong>
            </div>
            <div className="permission-detail">
              <span>失败处理</span>
              <strong>{realCodexBoundary.failure}</strong>
            </div>
          </>
        ) : null}
        {k3B1RecoveryBoundary ? (
          <>
            <div className="permission-detail">
              <span>K3-B1 恢复状态</span>
              <strong>{k3B1RecoveryBoundary.status}</strong>
            </div>
            <div className="permission-detail">
              <span>读回边界</span>
              <strong>{k3B1RecoveryBoundary.readback}</strong>
            </div>
            <div className="permission-detail">
              <span>执行边界</span>
              <strong>本动作不执行 codex exec/resume、不发送提示词、不写 .codex、不解锁 K3-B2。</strong>
            </div>
            <div className="permission-detail">
              <span>风险说明</span>
              <strong>{k3B1RecoveryBoundary.risk}</strong>
            </div>
          </>
        ) : null}
        {operationControlBoundary ? (
          <>
            <div className="permission-detail">
              <span>L3 操作</span>
              <strong>{operationControlBoundary.operationId}</strong>
            </div>
            <div className="permission-detail">
              <span>当前状态</span>
              <strong>{operationControlBoundary.currentStatus}</strong>
            </div>
            <div className="permission-detail">
              <span>确认后状态</span>
              <strong>{operationControlBoundary.afterStatus}；只登记决策，不代表已运行。</strong>
            </div>
            <div className="permission-detail">
              <span>当前门</span>
              <strong>{operationControlBoundary.gate}</strong>
            </div>
            <div className="permission-detail">
              <span>真执行写入面</span>
              <strong>{operationControlBoundary.wouldWrite}</strong>
            </div>
            <div className="permission-detail">
              <span>读回边界</span>
              <strong>{operationControlBoundary.readback}</strong>
            </div>
            <div className="permission-detail">
              <span>审计 / 运行日志</span>
              <strong>{operationControlBoundary.audit} / {operationControlBoundary.runtime}</strong>
            </div>
            <div className="permission-detail">
              <span>风险说明</span>
              <strong>{operationControlBoundary.risk}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "create-task-draft" && action.taskDraft ? (
          <>
            <div className="permission-detail">
              <span>任务标题</span>
              <strong>{action.taskDraft.title}</strong>
            </div>
            <div className="permission-detail">
              <span>目标说明</span>
              <strong>{action.taskDraft.objective}</strong>
            </div>
            <div className="permission-detail">
              <span>默认指派</span>
              <strong>{action.taskDraft.assignedRole}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "copy-task-preview" && action.taskPreview ? (
          <div className="permission-detail">
            <span>复制对象</span>
            <strong>{action.taskPreview.workItemId}</strong>
          </div>
        ) : null}
        {action.kind === "update-task-fields" && action.taskFields ? (
          <>
            <div className="permission-detail">
              <span>更新对象</span>
              <strong>{action.taskFields.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>任务名</span>
              <strong>{action.taskFields.fields.task_name || "待补充"}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "correct-dispatch-fields" && action.dispatchFields ? (
          <>
            <div className="permission-detail">
              <span>修正对象</span>
              <strong>{action.dispatchFields.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>任务名</span>
              <strong>{action.dispatchFields.fields.task_name || "待补充"}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "generate-task-file" && action.taskFileGeneration ? (
          <>
            <div className="permission-detail">
              <span>生成对象</span>
              <strong>{action.taskFileGeneration.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>写入目录</span>
              <strong>/Users/yoyi/workspace/product-line/tasks/</strong>
            </div>
          </>
        ) : null}
        {action.kind === "advance-work-item-state" && action.workItemStateUpdate ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.workItemStateUpdate.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>目标状态</span>
              <strong>{stateLabel(action.workItemStateUpdate.next_state)}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "bind-node-session" && action.nodeSessionBinding ? (
          <>
            <div className="permission-detail">
              <span>节点</span>
              <strong>{action.nodeSessionBinding.node_id}</strong>
            </div>
            <div className="permission-detail">
              <span>Codex 会话</span>
              <strong>{action.nodeSessionBinding.thread_id}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "unbind-node-session" && action.nodeSessionUnbinding ? (
          <div className="permission-detail">
            <span>绑定对象</span>
            <strong>{action.nodeSessionUnbinding.binding_id}</strong>
          </div>
        ) : null}
        {action.kind === "execute-node-dispatch" && action.nodeDispatch ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.nodeDispatch.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>节点</span>
              <strong>{action.nodeDispatch.node_id}</strong>
            </div>
            <div className="permission-detail">
              <span>派发模式</span>
              <strong>{action.nodeDispatch.prompt_kind === "safe_probe" ? "安全测试模式" : "用户审核模式"}</strong>
            </div>
            {action.nodeDispatch.prompt_kind === "user_reviewed_instruction" && action.nodeDispatch.user_reviewed_instruction ? (
              <>
                <div className="permission-detail">
                  <span>执行目录</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.execution_cwd}</strong>
                </div>
                <div className="permission-detail">
                  <span>沙箱模式</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.sandbox_mode}</strong>
                </div>
                <div className="permission-detail">
                  <span>允许写入根目录</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.allowed_write_roots.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>允许读取</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.allowed_reads.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>允许写入</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.allowed_writes.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>禁止事项</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.forbidden_actions.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>超时 / 重试</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.timeout_seconds} 秒 / {action.nodeDispatch.user_reviewed_instruction.max_retries} 次</strong>
                </div>
                <div className="permission-detail">
                  <span>必须回传</span>
                  <strong>{action.nodeDispatch.user_reviewed_instruction.required_return.join("；") || "未登记"}</strong>
                </div>
              </>
            ) : null}
          </>
        ) : null}
        {action.kind === "record-director-review" && action.directorReview ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.directorReview.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>派发记录</span>
              <strong>{action.directorReview.dispatch_id}</strong>
            </div>
            <div className="permission-detail">
              <span>回收结论</span>
              <strong>{directorDecisionLabel(action.directorReview.decision)}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "preview-user-reviewed-instruction" && action.userReviewedInstruction ? (
          <>
            <div className="permission-detail">
              <span>指令摘要</span>
              <strong>{action.userReviewedInstruction.summary || "待补充"}</strong>
            </div>
            <div className="permission-detail">
              <span>审核状态</span>
              <strong>{action.userReviewedInstruction.approval_state}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-permission-decision" && action.permissionDecision ? (
          <>
            <div className="permission-detail">
              <span>权限请求</span>
              <strong>{action.permissionDecision.request_id}</strong>
            </div>
            <div className="permission-detail">
              <span>权限结论</span>
              <strong>{action.permissionDecision.decision === "approved" ? "批准" : "拒绝"}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "create-memory-candidate" && action.memoryCandidateCreation ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>memory-candidates.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>来源类型</span>
              <strong>{memoryCandidateCreationSource?.source_type ?? "knowledge_doc"}</strong>
            </div>
            <div className="permission-detail">
              <span>知识库资料</span>
              <strong>{memoryCandidateCreationSource?.source_title ?? action.memoryCandidateCreation.claim}</strong>
            </div>
            <div className="permission-detail">
              <span>生成来源</span>
              <strong>{action.memoryCandidateCreation.generated_from}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>该动作只会在你确认后写入 memory-candidates.v1.json；只生成候选，不写正式记忆；知识库材料仍需经候选与用户确认。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "adopt-memory-candidate-to-formal-memory" && action.memoryCandidateAdoption ? (
          <>
            <div className="permission-detail">
              <span>采纳类型</span>
              <strong>采纳候选为正式记忆</strong>
            </div>
            <div className="permission-detail">
              <span>记忆候选</span>
              <strong>{action.memoryCandidateAdoption.candidate_key}</strong>
            </div>
            <div className="permission-detail">
              <span>采纳理由</span>
              <strong>{action.memoryCandidateAdoption.adoption_reason}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "adopt-memory-candidates-to-formal-memory-batch" && action.memoryCandidateBatchAdoptions?.length ? (
          <>
            <div className="permission-detail">
              <span>批量采纳</span>
              <strong>{action.memoryCandidateBatchAdoptions.length} 条候选；逐条复用 M2 采纳门</strong>
            </div>
            <div className="permission-detail">
              <span>候选清单</span>
              <strong>{action.memoryCandidateBatchAdoptions.map((item) => item.candidate_key).join("；")}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>本动作只处理弹窗中列出的候选；不会自动采纳其他候选，不绕过用户确认门。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "knowledge-vault-ai-write" && action.knowledgeVaultWrite ? (
          <>
            <div className="permission-detail">
              <span>笔记标题</span>
              <strong>{action.knowledgeVaultWrite.note_title}</strong>
            </div>
            <div className="permission-detail">
              <span>来源</span>
              <strong>{action.knowledgeVaultWrite.source_summary}</strong>
            </div>
            <div className="permission-detail">
              <span>全文预览</span>
              <strong className="knowledge-vault-preview">{action.knowledgeVaultWrite.body}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>该动作只会在你允许后写入工作台自管的 knowledge-vault 目录（md 文件）；不写正式记忆、不碰你的其他文件夹、不会自动写第二条。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-formal-memory-lifecycle-operation" && action.formalMemoryLifecycle ? (
          <>
            <div className="permission-detail">
              <span>操作</span>
              <strong>{formalMemoryLifecycleLabel(action.formalMemoryLifecycle.operation_kind)}</strong>
            </div>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>formal-memories.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>目标记忆</span>
              <strong>
                {action.formalMemoryLifecycle.memory_id ??
                  (action.formalMemoryLifecycle.memory_ids.join("；") ||
                    action.formalMemoryLifecyclePreview?.target_memory_ids.join("；") ||
                    "未登记")}
              </strong>
            </div>
            <div className="permission-detail">
              <span>确认权</span>
              <strong>
                {action.formalMemoryLifecyclePreview?.required_approval.approval_kind ?? "待确认"} /{" "}
                {action.formalMemoryLifecycle.confirmed_by ?? "未登记"}
              </strong>
            </div>
            <div className="permission-detail">
              <span>影响面</span>
              <strong>{action.formalMemoryLifecyclePreview?.impact.display_text ?? "预览未返回影响面"}</strong>
            </div>
            <div className="permission-detail">
              <span>任务包影响</span>
              <strong>{action.formalMemoryLifecyclePreview?.impact.task_packet_eligibility_change ?? "按活跃状态 / 检查 / 范围规则重新评估"}</strong>
            </div>
            <div className="permission-detail">
              <span>版本变化</span>
              <strong>
                原版本 {action.formalMemoryLifecyclePreview?.before_records.map((record) => `v${record.record_version}`).join("；") || "新记录"} / 新版本{" "}
                {action.formalMemoryLifecyclePreview?.proposed_records.map((record) => `v${record.record_version}`).join("；") || "待写入"}
              </strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>正式记忆生命周期会写版本和审计；非启用态记忆默认不进任务包；知识库和观察来源不能绕过该确认。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-memory-entity-alias-decision" && action.memoryEntityAliasDecision ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>memory-entity-relations.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>实体候选</span>
              <strong>{action.memoryEntityAliasCandidate?.display_name ?? action.memoryEntityAliasDecision.entity_candidate_id}</strong>
            </div>
            <div className="permission-detail">
              <span>决定</span>
              <strong>{action.memoryEntityAliasDecision.decision}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>只登记实体 / 别名治理决定；不写正式记忆，不改任务包入选清单。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-memory-entity-merge-decision" && action.memoryEntityMergeDecision ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>memory-entity-relations.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>去重候选</span>
              <strong>
                {action.memoryEntityMergeCandidate
                  ? `${action.memoryEntityMergeCandidate.left_label} / ${action.memoryEntityMergeCandidate.right_label}`
                  : action.memoryEntityMergeDecision.merge_candidate_id}
              </strong>
            </div>
            <div className="permission-detail">
              <span>决定</span>
              <strong>{action.memoryEntityMergeDecision.decision}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>相似度命中仅作候选；确认只写治理审计，不改正式记忆内容。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-memory-relation-candidate-decision" && action.memoryRelationCandidateDecision ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>memory-entity-relations.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>关系候选</span>
              <strong>
                {action.memoryRelationCandidate
                  ? `${action.memoryRelationCandidate.subject_label} -> ${action.memoryRelationCandidate.object_label}`
                  : action.memoryRelationCandidateDecision.relation_candidate_id}
              </strong>
            </div>
            <div className="permission-detail">
              <span>决定</span>
              <strong>{action.memoryRelationCandidateDecision.decision}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>已确认关系用于解释召回原因；关系候选不会作为正式事实影响工作者。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "run-memory-maintenance" && action.memoryMaintenanceRun ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>memory-lint.v1.json</strong>
            </div>
            <div className="permission-detail">
              <span>维护意图</span>
              <strong>{action.memoryMaintenanceRun.lint_intent === "maintenance_run" ? "维护运行" : action.memoryMaintenanceRun.lint_intent}</strong>
            </div>
            <div className="permission-detail">
              <span>上下文</span>
              <strong>{action.memoryMaintenanceRun.project_id ?? "未登记"} / {action.memoryMaintenanceRun.workflow_id ?? "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>检查边界</span>
              <strong>维护任务只生成发现 / 报告；阻断级发现会阻止召回；不会自动修改正式记忆或调用生命周期。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-mature-pattern-decision" && action.maturePatternDecision ? (
          <>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>
                {action.maturePatternDecision.decision === "confirm_as_formal_memory"
                  ? "memory-patterns.v1.json / formal-memories.v1.json"
                  : "memory-patterns.v1.json"}
              </strong>
            </div>
            <div className="permission-detail">
              <span>成熟模式候选</span>
              <strong>{action.maturePatternCandidate?.title ?? action.maturePatternDecision.candidate_id}</strong>
            </div>
            <div className="permission-detail">
              <span>决定</span>
              <strong>{maturePatternDecisionLabel(action.maturePatternDecision.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>确认人</span>
              <strong>{action.maturePatternDecision.confirmed_by ?? "未确认正式化"}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>候选和跨项目主题报告未确认不进入任务包；只有用户确认正式化时才会通过正式记忆路径写版本、审计和来源引用。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "create-project-consultation-proposal" && action.projectConsultationProposalCreation ? (
          <>
            <div className="permission-detail">
              <span>方案标题</span>
              <strong>{action.projectConsultationProposalCreation.title}</strong>
            </div>
            <div className="permission-detail">
              <span>用户目标</span>
              <strong>{action.projectConsultationProposalCreation.goal_summary}</strong>
            </div>
            <div className="permission-detail">
              <span>允许读取</span>
              <strong>{action.projectConsultationProposalCreation.scope_draft.allowed_read_roots.join("；") || "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>允许写入</span>
              <strong>{action.projectConsultationProposalCreation.scope_draft.allowed_write_roots.join("；") || "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>工具 / 检查</span>
              <strong>
                {action.projectConsultationProposalCreation.scope_draft.allowed_tools.join("；") || "未登记"} /{" "}
                {action.projectConsultationProposalCreation.scope_draft.allowed_checks.join("；") || "未登记"}
              </strong>
            </div>
            <div className="permission-detail">
              <span>停止条件</span>
              <strong>{action.projectConsultationProposalCreation.scope_draft.stop_conditions.join("；") || "未登记"}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-project-consultation-proposal-decision" && action.projectConsultationProposalDecision ? (
          <>
            <div className="permission-detail">
              <span>方案决定</span>
              <strong>{proposalDecisionLabel(action.projectConsultationProposalDecision.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>决定摘要</span>
              <strong>{action.projectConsultationProposalDecision.summary}</strong>
            </div>
            {action.projectConsultationProposalPreview ? (
              <>
                <div className="permission-detail">
                  <span>目标摘要</span>
                  <strong>{action.projectConsultationProposalPreview.goalSummary}</strong>
                </div>
                <div className="permission-detail">
                  <span>允许读取</span>
                  <strong>{action.projectConsultationProposalPreview.allowedReadRoots.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>允许写入</span>
                  <strong>{action.projectConsultationProposalPreview.allowedWriteRoots.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>工具 / 检查</span>
                  <strong>
                    {action.projectConsultationProposalPreview.allowedTools.join("；") || "未登记"} /{" "}
                    {action.projectConsultationProposalPreview.allowedChecks.join("；") || "未登记"}
                  </strong>
                </div>
                <div className="permission-detail">
                  <span>停止条件</span>
                  <strong>{action.projectConsultationProposalPreview.stopConditions.join("；") || "未登记"}</strong>
                </div>
              </>
            ) : null}
          </>
        ) : null}
        {action.kind === "record-global-boundary-review" && action.globalBoundaryReview ? (
          <>
            <div className="permission-detail">
              <span>复核结论</span>
              <strong>{globalBoundaryReviewLabel(action.globalBoundaryReview.review_status)}</strong>
            </div>
            <div className="permission-detail">
              <span>复核摘要</span>
              <strong>{action.globalBoundaryReview.summary}</strong>
            </div>
            <div className="permission-detail">
              <span>授权对象</span>
              <strong>{action.globalBoundaryReview.authorization_id}</strong>
            </div>
            {action.globalBoundaryReviewPreview ? (
              <>
                <div className="permission-detail">
                  <span>方案标题</span>
                  <strong>{action.globalBoundaryReviewPreview.proposalTitle}</strong>
                </div>
                <div className="permission-detail">
                  <span>目标摘要</span>
                  <strong>{action.globalBoundaryReviewPreview.goalSummary}</strong>
                </div>
                <div className="permission-detail">
                  <span>读写范围</span>
                  <strong>{action.globalBoundaryReviewPreview.readWriteScope}</strong>
                </div>
                <div className="permission-detail">
                  <span>工具 / 检查</span>
                  <strong>{action.globalBoundaryReviewPreview.toolsAndChecks}</strong>
                </div>
                <div className="permission-detail">
                  <span>停止条件</span>
                  <strong>{action.globalBoundaryReviewPreview.stopConditions.join("；") || "未登记"}</strong>
                </div>
                <div className="permission-detail">
                  <span>发现</span>
                  <strong>{action.globalBoundaryReviewPreview.findings.map((finding) => finding.summary).join("；") || "无阻断发现"}</strong>
                </div>
              </>
            ) : null}
          </>
        ) : null}
        {action.kind === "prepare-authorized-auto-dispatch" && action.authorizedAutoDispatch ? (
          <>
            <div className="permission-detail">
              <span>授权对象</span>
              <strong>{action.authorizedAutoDispatch.authorization_id}</strong>
            </div>
            <div className="permission-detail">
              <span>方案对象</span>
              <strong>{action.authorizedAutoDispatch.proposal_id}</strong>
            </div>
            <div className="permission-detail">
              <span>执行角色</span>
              <strong>{action.authorizedAutoDispatch.actor_id}</strong>
            </div>
            {authorizedAutoDispatchSummary ? (
              <>
                <div className="permission-detail">
                  <span>计划摘要</span>
                  <strong>{authorizedAutoDispatchSummary.display_text}</strong>
                </div>
                <div className="permission-detail">
                  <span>任务计数</span>
                  <strong>
                    planned {authorizedAutoDispatchSummary.planned_task_count} / prepared{" "}
                    {authorizedAutoDispatchSummary.prepared_dispatch_count} / blocked{" "}
                    {authorizedAutoDispatchSummary.blocked_count} / needs_binding{" "}
                    {authorizedAutoDispatchSummary.needs_binding_count}
                  </strong>
                </div>
                <div className="permission-detail">
                  <span>记忆快照</span>
                  <strong>{authorizedAutoDispatchSummary.memory_text}</strong>
                </div>
                <div className="permission-detail">
                  <span>阻断原因</span>
                  <strong>{authorizedAutoDispatchSummary.blocked_reasons.join("；") || "无阻断原因"}</strong>
                </div>
              </>
            ) : null}
            <div className="permission-detail">
              <span>边界</span>
              <strong>只创建准备记录，不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-worker-structured-report" && action.workerStructuredReport ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.workerStructuredReport.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>工作者节点</span>
              <strong>{action.workerStructuredReport.workflow_node_id}</strong>
            </div>
            <div className="permission-detail">
              <span>汇报摘要</span>
              <strong>{action.workerStructuredReport.summary}</strong>
            </div>
            <div className="permission-detail">
              <span>证据</span>
              <strong>{action.workerStructuredReport.evidence_refs.join("；") || "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>只记录工作者汇报；不把汇报写成正式事实或正式记忆，不启动 Codex。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-project-director-process-fact-decision" && action.processFactDecision ? (
          <>
            <div className="permission-detail">
              <span>汇报对象</span>
              <strong>{action.processFactDecision.report_id}</strong>
            </div>
            <div className="permission-detail">
              <span>决定</span>
              <strong>{processFactDecisionLabel(action.processFactDecision.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>确认事实</span>
              <strong>{action.processFactDecision.accepted_facts.map((fact) => fact.summary).join("；") || "无"}</strong>
            </div>
            <div className="permission-detail">
              <span>摘要</span>
              <strong>{action.processFactDecision.summary}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>确认后只记录过程事实观察；不写正式记忆，不完成最终验收。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-global-final-result-review" && action.globalFinalResultReview ? (
          <>
            <div className="permission-detail">
              <span>工作流</span>
              <strong>{action.globalFinalResultReview.workflow_id}</strong>
            </div>
            <div className="permission-detail">
              <span>复核结论</span>
              <strong>{globalFinalReviewLabel(action.globalFinalResultReview.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>过程事实</span>
              <strong>{action.globalFinalResultReview.accepted_process_fact_ids.join("；") || "无"}</strong>
            </div>
            <div className="permission-detail">
              <span>摘要</span>
              <strong>{action.globalFinalResultReview.summary}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>这只是全局主管最终复核；不代表用户已接受，不写正式记忆，不代表中间版本整体完成。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "record-user-result-decision" && action.userResultDecision ? (
          <>
            <div className="permission-detail">
              <span>工作流</span>
              <strong>{action.userResultDecision.workflow_id}</strong>
            </div>
            <div className="permission-detail">
              <span>用户决定</span>
              <strong>{userResultDecisionLabel(action.userResultDecision.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>关联复核</span>
              <strong>{action.userResultDecision.accepted_review_id || "未引用"}</strong>
            </div>
            <div className="permission-detail">
              <span>摘要</span>
              <strong>{action.userResultDecision.summary}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>只记录本次结果决定；不代表未来任务默认接受，不写正式记忆。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "generate-stage-c-acceptance-summary" && action.stageCAcceptanceSummary ? (
          <>
            <div className="permission-detail">
              <span>工作流</span>
              <strong>{action.stageCAcceptanceSummary.workflow_id}</strong>
            </div>
            <div className="permission-detail">
              <span>项目</span>
              <strong>{action.stageCAcceptanceSummary.project_id}</strong>
            </div>
            <div className="permission-detail">
              <span>写入位置</span>
              <strong>只写 workflow-state.v0.json 既有产物和审计事件。</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>生成门禁摘要和后置项；不执行真实工作者，不写正式记忆，不代表中间版本整体完成。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "run-project-workflow-automation-phase-a" && action.projectWorkflowAutomation ? (
          <>
            <div className="permission-detail">
              <span>用户目标</span>
              <strong>{action.projectWorkflowAutomation.user_goal}</strong>
            </div>
            <div className="permission-detail">
              <span>目标会话</span>
              <strong>{action.projectWorkflowAutomation.target_session_id ?? "未绑定"}</strong>
            </div>
            <div className="permission-detail">
              <span>执行沙箱</span>
              <strong>{action.projectWorkflowAutomation.sandbox ?? "read-only"}</strong>
            </div>
            <div className="permission-detail">
              <span>边界</span>
              <strong>只记录 J2-A Phase A no-op；不发送提示词，不执行真实 Codex，不写项目文件。</strong>
            </div>
          </>
        ) : null}
        {action.kind === "offline-role-dispatch" && action.offlineRoleDispatch ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.offlineRoleDispatch.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>目标角色</span>
              <strong>{action.offlineRoleDispatch.target_role_label}</strong>
            </div>
            <div className="permission-detail">
              <span>任务名</span>
              <strong>{action.offlineRoleDispatch.task_title}</strong>
            </div>
            <div className="permission-detail">
              <span>目标</span>
              <strong>{action.offlineRoleDispatch.objective}</strong>
            </div>
            <div className="permission-detail">
              <span>执行目录</span>
              <strong>{action.offlineRoleDispatch.execution_cwd}</strong>
            </div>
            <div className="permission-detail">
              <span>允许读取</span>
              <strong>{action.offlineRoleDispatch.allowed_reads.join("；") || "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>允许写入</span>
              <strong>{action.offlineRoleDispatch.allowed_writes.join("；") || "未登记"}</strong>
            </div>
            <div className="permission-detail">
              <span>必须回传</span>
              <strong>{action.offlineRoleDispatch.required_return.join("；") || "未登记"}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "offline-role-result-handoff" && action.offlineRoleResultHandoff ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.offlineRoleResultHandoff.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>离线派发</span>
              <strong>{action.offlineRoleResultHandoff.dispatch_id}</strong>
            </div>
            <div className="permission-detail">
              <span>角色</span>
              <strong>{action.offlineRoleResultHandoff.target_role_id}</strong>
            </div>
            <div className="permission-detail">
              <span>回传摘要</span>
              <strong>{action.offlineRoleResultHandoff.summary}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "offline-director-review" && action.offlineDirectorReview ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.offlineDirectorReview.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>离线派发</span>
              <strong>{action.offlineDirectorReview.dispatch_id}</strong>
            </div>
            <div className="permission-detail">
              <span>回收结论</span>
              <strong>{directorDecisionLabel(action.offlineDirectorReview.decision)}</strong>
            </div>
            <div className="permission-detail">
              <span>摘要</span>
              <strong>{action.offlineDirectorReview.summary}</strong>
            </div>
          </>
        ) : null}
        {action.kind === "run-workflow-machine" && action.workflowMachineRun ? (
          <>
            <div className="permission-detail">
              <span>工作项</span>
              <strong>{action.workflowMachineRun.work_item_id}</strong>
            </div>
            <div className="permission-detail">
              <span>目标</span>
              <strong>{action.workflowMachineRun.objective}</strong>
            </div>
            {action.workflowMachineRun.execution_root ? (
              <div className="permission-detail">
                <span>目标执行目录</span>
                <strong>{action.workflowMachineRun.execution_root}</strong>
              </div>
            ) : null}
            <div className="permission-detail">
              <span>最大轮次</span>
              <strong>{action.workflowMachineRun.max_rounds}</strong>
            </div>
            <div className="permission-detail">
              <span>单步超时</span>
              <strong>{action.workflowMachineRun.timeout_seconds_per_step} 秒</strong>
            </div>
          </>
        ) : null}
        <p className="muted">
          {action.kind === "create-task-draft"
            ? "该动作只会在你确认后登记到工作台自己的状态文件；不生成真实任务包文件、不派发真实 Codex 会话。"
            : action.kind === "copy-task-preview"
            ? "该动作只会在你确认后复制 Markdown 预览文本；不写真实任务文件、不派发真实 Codex 会话。"
            : action.kind === "update-task-fields"
            ? "该动作只会在你确认后写入工作台自己的状态文件；不生成真实任务文件、不派发真实 Codex 会话。"
            : action.kind === "correct-dispatch-fields"
            ? "该动作只会在你确认后写入工作台自己的状态文件；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 /Users/yoyi/.codex 或 Codex 状态库。"
            : action.kind === "generate-task-file"
            ? "该动作只会在你确认后生成一个新的 product-line/tasks/*.md 文件，并更新工作台自己的状态文件；不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 /Users/yoyi/.codex 或 Codex 状态库。"
            : action.kind === "advance-work-item-state"
            ? "该动作只会在你确认后推进工作台自己的工作项状态并追加审计事件；不启动 Codex 命令行、不恢复会话、不派发真实 Codex 会话、不运行运行器、不写 /Users/yoyi/.codex 或 Codex 状态库。"
            : action.kind === "bind-node-session"
            ? "该动作只会在你确认后把已有索引 Codex 会话绑定到工作台自己的工作流状态；不写 Codex 状态库、不启动 Codex、不发送消息、不读取完整会话正文。"
            : action.kind === "unbind-node-session"
            ? "该动作只会在你确认后解除工作台自己的节点会话绑定；不删除、不移动、不归档 Codex 原始会话，不写 /Users/yoyi/.codex。"
            : action.kind === "execute-node-dispatch"
            ? action.nodeDispatch?.prompt_kind === "user_reviewed_instruction"
              ? "该动作会在你确认后执行 codex exec resume，会写 /Users/yoyi/.codex，会写工作台工作流状态，可能写入上方允许的业务路径；不会读取授权、密钥、.env 或完整会话记录，不会运行运行器，不会删除、移动、归档会话。"
              : "该动作会在你确认后向绑定的 Codex 会话发送一条消息，会写 /Users/yoyi/.codex，会写工作台工作流状态；不会读取授权或密钥，不会运行运行器，不会删除、移动、归档会话。"
            : action.kind === "record-director-review"
            ? "该动作只会在你确认后把总指导回收意见写入真实工作流状态的复核记录并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex、不读取会话原文。"
            : action.kind === "preview-user-reviewed-instruction"
            ? "该动作只确认结构化业务指令的边界预览；当前版本不执行真实业务任务、不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。"
            : action.kind === "record-permission-decision"
            ? "该动作只会在你确认后通过控制核心把权限结论写入工作台自己的工作流状态，并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。"
            : action.kind === "create-project-consultation-proposal"
            ? "该动作只会在你确认后写入 project-proposals.v1.json；不调用真实项目咨询智能体、不启动 Codex、不执行工作者、不写 /Users/yoyi/.codex。"
            : action.kind === "record-project-consultation-proposal-decision"
            ? "该动作只会在你确认后写入项目咨询方案辅助状态文件；确认方案时会联动 C1 方案授权并停在待全局复核，不会启动真实工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。"
            : action.kind === "record-global-boundary-review"
            ? "该动作只会在你确认后写入 plan-authorizations.v1.json 的全局边界复核；批准时只让授权有效，仍未派发工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。"
            : action.kind === "prepare-authorized-auto-dispatch"
            ? "该动作只会在你确认后创建准备记录；不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。"
            : action.kind === "record-worker-structured-report"
            ? "该动作只会在你确认后写入工作者汇报记录和审计事件；不启动工作者、不执行 Codex、不把汇报写成正式事实或正式记忆。"
            : action.kind === "record-project-director-process-fact-decision"
            ? "该动作只会在你确认后记录项目主管过程事实决定；确认低风险事实时写观察，仍不是正式记忆，不做最终结果复核。"
            : action.kind === "record-formal-memory-lifecycle-operation"
            ? "该动作只会在你确认后通过正式记忆生命周期写入 formal-memories.v1.json；会新增版本和审计，不会改候选、观察、检查或工作流状态。"
            : action.kind === "record-mature-pattern-decision"
            ? "该动作只会在你确认后写入 M12 成熟模式辅助状态文件；只有用户确认正式化时才会联动 formal-memories.v1.json，候选和报告不会直接进入任务包。"
            : action.kind === "run-project-workflow-automation-phase-a"
            ? "该动作只会在你确认后生成项目自动编排 Level A run units，并记录 Product Command Phase A no-op、runtime/audit/readback unavailable、worker report、捕获来源和 observation；不发送提示词、不执行真实 Codex、不写 /Users/yoyi/.codex、不写项目文件。"
            : action.kind === "record-operation-control-decision"
            ? "该动作只会记录 L3 操作控制决策和待处理状态；不调用 runner、不执行 Codex、不发送提示词、不停止或重启真实进程、不解锁 K3-B2。"
            : action.kind === "adopt-memory-candidate-to-formal-memory"
            ? "该动作只会在你确认后通过 M2 采纳门写入 formal-memories.v1.json，并保留候选、来源、版本和审计；不会自动采纳其他候选。"
            : action.kind === "adopt-memory-candidates-to-formal-memory-batch"
            ? "该动作只会在你确认后逐条调用 M2 采纳门；不会绕过确认门，不会自动正式化未列出的候选。"
            : action.kind === "offline-role-dispatch"
            ? "该动作只会在你确认后把离线角色派发写入工作台自己的工作流状态；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。"
            : action.kind === "offline-role-result-handoff"
            ? "该动作只会在你确认后把离线角色回传写入工作台自己的工作流状态；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。"
            : action.kind === "offline-director-review"
            ? "该动作只会在你确认后把离线总指导回收写入工作台自己的工作流状态，并推进工作项状态；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。"
            : action.kind === "run-workflow-machine"
            ? "该动作会在你确认后按总指导、开发线、验证线、回收线、总指导结论循环调用绑定 Codex 会话；会执行 codex exec resume、写 /Users/yoyi/.codex、写真实工作流状态，并允许开发线修改项目目录。"
            : action.kind === "initialize-workflow-state" || action.kind === "bootstrap-project-workflow"
            ? "该动作只会在你确认后写入工作台自己的状态文件；后端会追加审计事件，使用临时文件和原子替换，存在旧状态时先备份。"
            : "该动作只会在你确认后执行；后端仍会再次检查路径是否在索引白名单内。"}
        </p>
        </div>{/* end dialog-zone-details */}

        {/* Zone 3: Risk summary + actions */}
        <div className="dialog-zone-actions">
          <p className={`dialog-risk-summary ${realCodexBoundary ? "" : "safe"}`}>
            {realCodexBoundary
              ? `⚠ 高风险：会执行真实 Codex。批准后会写 /Users/yoyi/.codex。失败、超时或读回不可用必须记录边界，不会自动重试。`
              : action.kind === "record-operation-control-decision"
              ? "低风险：本轮只登记运行控制决策和待处理状态。不启动 Codex 命令行，不发送真实执行指令，不停止或重启进程。"
              : action.kind === "adopt-memory-candidate-to-formal-memory" ||
                action.kind === "adopt-memory-candidates-to-formal-memory-batch"
              ? "中风险：会在确认后写正式记忆、版本和审计；候选不会自动正式化，批量也逐条走 M2 门。"
              : action.kind === "knowledge-vault-ai-write"
              ? "低风险：只写工作台自管的 knowledge-vault 目录；AI 提议、你允许才落盘；不写正式记忆、不碰其他文件夹。"
              : "低风险：只写工作台自己的状态文件。不启动 Codex 命令行，不发送真实执行指令，不写 /Users/yoyi/.codex。"}
          </p>
        <div className="dialog-actions">
          <button className="secondary-button" type="button" onClick={onCancel} disabled={busy}>
            {realCodexBoundary ? "拒绝" : action.kind === "knowledge-vault-ai-write" ? "不要" : "取消"}
          </button>
          <button className="primary-button" type="button" onClick={onConfirm} disabled={busy}>
            {busy ? "执行中…" : confirmActionLabel(action.kind)}
          </button>
        </div>
        </div>{/* end dialog-zone-actions */}
      </section>
    </div>
  );
}

function confirmActionLabel(kind: PendingAction["kind"]) {
  if (kind === "run-workflow-machine") return "确认启动多轮真实执行";
  if (kind === "execute-node-dispatch") return "确认真实派发";
  if (kind === "copy-task-preview") return "确认复制";
  if (kind === "run-project-workflow-automation-phase-a") return "确认生成编排记录";
  if (kind === "record-operation-control-decision") return "确认记录决策";
  if (kind === "knowledge-vault-ai-write") return "允许写入";
  if (kind === "adopt-memory-candidate-to-formal-memory") return "确认采纳";
  if (kind === "adopt-memory-candidates-to-formal-memory-batch") return "确认批量采纳";
  if (
    kind === "initialize-workflow-state" ||
    kind === "bootstrap-project-workflow" ||
    kind === "update-task-fields" ||
    kind === "correct-dispatch-fields" ||
    kind === "advance-work-item-state" ||
    kind === "bind-node-session" ||
    kind === "unbind-node-session"
  ) {
    return "确认写入状态";
  }
  if (
    kind === "record-director-review" ||
    kind === "record-permission-decision" ||
    kind === "record-worker-structured-report" ||
    kind === "record-project-director-process-fact-decision" ||
    kind === "record-global-final-result-review" ||
    kind === "generate-stage-c-acceptance-summary" ||
    kind === "offline-role-dispatch" ||
    kind === "offline-role-result-handoff" ||
    kind === "offline-director-review"
  ) {
    return "确认记录";
  }
  if (
    kind === "record-blackboard-candidate-decision" ||
    kind === "record-memory-candidate-decision" ||
    kind === "record-memory-entity-alias-decision" ||
    kind === "record-memory-entity-merge-decision" ||
    kind === "record-memory-relation-candidate-decision" ||
    kind === "record-mature-pattern-decision" ||
    kind === "record-project-consultation-proposal-decision" ||
    kind === "record-global-boundary-review" ||
    kind === "record-user-result-decision"
  ) {
    return "确认提交决定";
  }
  if (kind === "create-task-draft") return "确认创建草稿";
  if (
    kind === "create-memory-candidate" ||
    kind === "create-memory-candidate-from-observation"
  ) {
    return "确认创建候选";
  }
  if (
    kind === "generate-task-file" ||
    kind === "record-formal-memory-lifecycle-operation" ||
    kind === "run-memory-maintenance" ||
    kind === "create-project-consultation-proposal" ||
    kind === "prepare-authorized-auto-dispatch"
  ) {
    return "确认创建记录";
  }
  if (kind === "preview-user-reviewed-instruction") return "确认边界预览";
  return "确认继续";
}

function directorDecisionLabel(decision: string) {
  if (decision === "accepted") return "接受";
  if (decision === "needs_changes") return "需要修改";
  if (decision === "paused") return "暂停";
  if (decision === "discarded") return "废弃";
  return decision;
}

function processFactDecisionLabel(decision: string) {
  if (decision === "confirm_process_fact") return "确认为过程事实";
  if (decision === "request_rework") return "要求返工";
  if (decision === "block_and_escalate") return "阻断并上报";
  return decision || "未知决定";
}

function globalFinalReviewLabel(decision: string) {
  if (decision === "accepted") return "最终复核通过";
  if (decision === "needs_changes") return "需要修改";
  if (decision === "blocked") return "已阻断";
  return decision || "未知复核结论";
}

function userResultDecisionLabel(decision: string) {
  if (decision === "accept_result") return "用户已接受";
  if (decision === "request_changes") return "用户要求修改";
  if (decision === "reject_result") return "用户拒绝结果";
  return decision || "未知用户决定";
}

function proposalDecisionLabel(decision: string) {
  if (decision === "confirm") return "确认方案范围";
  if (decision === "request_changes") return "要求修改";
  if (decision === "reject") return "拒绝方案";
  return decision;
}

function globalBoundaryReviewLabel(status: string) {
  if (status === "approved") return "批准并生效";
  if (status === "needs_changes") return "要求修改";
  if (status === "blocked") return "阻断方案";
  return status;
}

function maturePatternDecisionLabel(decision: string) {
  if (decision === "confirm_as_formal_memory") return "用户确认写入正式记忆";
  if (decision === "reject") return "拒绝候选";
  if (decision === "quarantine") return "隔离候选";
  if (decision === "request_changes") return "要求补充来源";
  return decision;
}

function formalMemoryLifecycleLabel(kind: string) {
  if (kind === "revise") return "编辑提案";
  if (kind === "deprecate") return "废弃";
  if (kind === "freeze") return "冻结";
  if (kind === "unfreeze") return "解冻";
  if (kind === "archive") return "归档";
  if (kind === "merge") return "合并";
  if (kind === "split") return "拆分";
  if (kind === "promote_to_global") return "上升为全局";
  if (kind === "demote_to_project") return "下沉为项目";
  return kind;
}

function stateLabel(state: string) {
  if (state === "draft") return "草稿";
  if (state === "ready_to_dispatch") return "待派发";
  if (state === "running") return "执行中";
  if (state === "ready_for_review") return "待回收";
  if (state === "accepted") return "已接受";
  if (state === "needs_changes") return "需修改";
  if (state === "paused") return "暂停";
  return state;
}
