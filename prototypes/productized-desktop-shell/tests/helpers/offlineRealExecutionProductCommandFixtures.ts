import type {
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandFailureStopRetryItem,
  RealExecutionProductCommandReadModel,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../../src/lib/types";

interface RealExecutionProductCommandFixtureInput {
  snapshot: WorkbenchSnapshot;
  readModel: RealExecutionProductCommandReadModel;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  projectRoot: string;
}

export function realExecutionProductCommandFixtures(input: RealExecutionProductCommandFixtureInput): {
  activeReadModel: RealExecutionProductCommandReadModel;
  activeAutomation: ProjectWorkflowAutomationReadModel;
  snapshotWithProductCommands: WorkbenchSnapshot;
} {
  const { snapshot, readModel, workflowStateWithProjectWorkflow, projectRoot } = input;
  const pcr7FailureStopRetryItems: RealExecutionProductCommandFailureStopRetryItem[] = (
    [
      ["user_rejected", "用户已拒绝", "用户拒绝或要求修改，当前不能继续执行。", "high", true, ["decision:decision:pcr7:user_rejected"], ["decision:rejected"]],
      ["blocked_by_guard", "被安全边界阻断", "安全边界或准备状态阻断了统一执行链路。", "high", false, ["preview:preview:pcr7:guard"], ["guard_policy_blocked"]],
      ["blocked_by_diagnostics", "被诊断阻断", "诊断降级或阻断状态要求先查看诊断。", "high", false, ["preview:preview:pcr7:diagnostics"], ["diagnostics:blocking_fixture"]],
      ["duplicate_blocked", "重复执行已阻断", "已有重复命令或运行记录，不能并行继续。", "medium", false, ["preview:preview:pcr7:duplicate"], ["duplicate_active"]],
      ["blocked_stale_memory", "记忆包缺失或过期", "任务记忆包缺失或过期，重新确认前需要先检查。", "medium", false, ["preview:preview:pcr7:memory"], ["memory_packet_stale"]],
      ["timed_out", "执行超时", "执行或读回超时，不能解释为已经完成停止。", "high", true, ["attempt:attempt:pcr7:timed_out"], ["readback_timed_out"]],
      ["readback_unavailable", "读回不可用", "没有可用读回来源，结果数未知。", "medium", false, ["attempt:attempt:pcr7:readback_unavailable"], ["unknown_readback_result_count_must_remain_null"]],
      ["readback_failed", "读回失败", "读回尝试失败或不可信，结果数未知。", "high", true, ["attempt:attempt:pcr7:readback_failed"], ["readback_parser_failed"]],
      ["runner_failed", "运行失败", "运行记录失败，不能自动重新执行。", "high", true, ["attempt:attempt:pcr7:runner_failed"], ["failure_reason:runner_failed_fixture"]],
      ["manual_stop_requested", "停止请求需受控处理", "用户请求停止仅作为产品状态，本任务不会停止真实进程。", "medium", false, ["decision:decision:pcr7:manual_stop"], ["manual_stop_requested_from_decision_reason"]],
      ["retry_requires_new_user_confirmation", "需要重新确认", "再次执行前需要新的用户确认；不会自动重试。", "high", true, ["product_command_retry_boundary"], ["pcr7_no_auto_retry_requires_new_user_confirmation"]],
    ] as Array<[string, string, string, string, boolean, string[], string[]]>
  ).map(([kind, title, summary, severity, requiresNewUserConfirmation, sourceRefs, warnings]) => ({
    kind,
    title,
    summary,
    count: 1,
    severity,
    requires_new_user_confirmation: requiresNewUserConfirmation,
    result_count: null,
    source_refs: sourceRefs,
    warnings,
  }));

  const activeReadModel: RealExecutionProductCommandReadModel = {
    ...readModel,
    store_available: true,
    command_count: 2,
    pending_decision_count: 1,
    running_attempt_count: 1,
    blocked_attempt_count: 1,
    last_attempt_status: "succeeded_stub",
    failure_stop_retry_summary: {
      schema_version: "real_execution_product_command_failure_stop_retry.v1",
      item_count: pcr7FailureStopRetryItems.length,
      failure_count: 4,
      blocked_count: 4,
      readback_issue_count: 3,
      manual_stop_requested_count: 1,
      retry_requires_new_user_confirmation: true,
      items: pcr7FailureStopRetryItems,
      warnings: ["pcr7_failure_stop_retry_summary_is_read_model_only", "retry_requires_new_user_confirmation_no_auto_retry"],
    },
    warnings: ["readback_unavailable_is_not_zero_results"],
  };
  const workflowId = workflowStateWithProjectWorkflow.project_workflows[0]?.workflow_id ?? "workflow:k3:fixture";
  const activeAutomation: ProjectWorkflowAutomationReadModel = {
    schema_version: "project_workflow_automation.v1",
    available: true,
    generated_at: "2026-06-09T00:00:00Z",
    latest_automation_id: "project-workflow-automation:offline",
    latest_status: "phase_a_closed_loop_recorded",
    latest_plan: {
      schema_version: "project_workflow_automation.v1",
      automation_id: "project-workflow-automation:offline",
      project_id: "project:offline",
      project_root: projectRoot,
      workflow_id: workflowId,
      user_goal: "离线验证 K3 Level A 项目自动编排摘要。",
      current_phase: "collector_summary",
      next_step: "等待主管复核 K3 Level A evidence / handoff。",
      run_units: ["director_plan", "developer_execution", "verifier_check", "collector_summary", "director_final_review"].map((kind, index) => ({
        run_unit_id: `run-unit:k3:${kind}`,
        run_unit_kind: kind,
        role: kind === "developer_execution" ? "developer_execution" : kind,
        status: kind === "director_final_review" ? "needs_review" : kind === "developer_execution" ? "readback_unavailable" : "completed",
        project_id: "project:offline",
        project_root: projectRoot,
        workflow_id: workflowId,
        workflow_node_id: `${workflowId}:node:codex-dev`,
        work_item_id: "work-item:k3:offline",
        task_package_ref: "task-package:k3:offline",
        memory_packet_ref: "memory-packet:k3:offline",
        product_command_preview_ref: `preview:k3:${kind}`,
        product_command_ref: index === 1 ? "product-command:k3:developer" : null,
        runtime_log_refs: index === 1 ? ["runtime-log:k3:phase-a"] : [],
        audit_refs: [`audit:k3:${kind}`],
        readback_ref: index === 1 ? "readback:k3:phase-a" : null,
        readback_status: "readback_unavailable",
        readback_result_count: null,
        worker_report_ref: index === 1 || kind === "collector_summary" ? "worker-report:k3:offline" : null,
        capture_event_refs: ["memory-capture:k3:offline"],
        observation_refs: kind === "collector_summary" || kind === "director_final_review" ? ["observation:k3:process-fact"] : [],
        memory_candidate_refs: [],
        runner_call_allowed: false,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        summary: `${kind} 摘要`,
        next_step: kind === "director_final_review" ? "等待主管复核。" : "进入下一阶段。",
        blocked_reasons: [],
        warnings: ["k3_level_a_no_real_codex_execution"],
      })),
      blocked_reasons: [],
      warnings: ["k3_level_a_phase_a_only"],
    },
    run_unit_count: 5,
    waiting_user_count: 0,
    blocked_count: 0,
    readback_unknown_count: 5,
    worker_report_count: 2,
    capture_event_count: 1,
    observation_count: 2,
    next_step: "等待主管复核 K3 Level A evidence / handoff。",
    warnings: ["k3_level_a_read_model_from_workflow_audit_event"],
  };
  const snapshotWithProductCommands: WorkbenchSnapshot = {
    ...snapshot,
    real_execution_product_commands: activeReadModel,
    project_workflow_automation: activeAutomation,
  };

  return {
    activeReadModel,
    activeAutomation,
    snapshotWithProductCommands,
  };
}
