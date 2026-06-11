import type {
  AutoDispatchGuardResult,
  PlanAuthorizationStoreV1,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  WorkflowRunCheck,
} from "../../src/lib/types";

interface AuthorizationWorkflowClusterFixtureInput {
  projectRoot: string;
  sessionThreadId: string;
  workflowProjectId: string;
  workflowId: string;
}

export function authorizationWorkflowClusterFixtures(input: AuthorizationWorkflowClusterFixtureInput) {
  const { projectRoot, sessionThreadId, workflowProjectId, workflowId } = input;

  const blockedWorkflowRunCheck: WorkflowRunCheck = {
    project_root: projectRoot,
    workflow_id: workflowId,
    status: "blocked",
    evidence_completeness: "missing",
    blocked_reasons: ["缺模型；系统不会自动选择模型。", "没有读范围；不能运行。", "会写文件但没有写范围；不能运行。"],
    warnings: ["节点没有声明工具；工具白名单为空。", "节点未要求 harness；harness 要求为空。"],
    checks: [
      {
        check_id: "missing_model",
        label: "模型",
        status: "blocked",
        severity: "blocked",
        reason: "缺模型；系统不会自动选择模型。",
        source_ref: "work-item:offline:001",
      },
      {
        check_id: "missing_read_scope",
        label: "读取范围",
        status: "blocked",
        severity: "blocked",
        reason: "没有读范围；不能运行。",
        source_ref: "work-item:offline:001",
      },
      {
        check_id: "missing_tool_whitelist",
        label: "工具白名单",
        status: "warning",
        severity: "warning",
        reason: "节点没有声明工具；工具白名单为空。",
        source_ref: "work-item:offline:001",
      },
    ],
  };

  const blockedAutoDispatchGuardResult: AutoDispatchGuardResult = {
    status: "blocked",
    authorization_id: "plan-auth:offline:active",
    reasons: ["写入范围超出方案授权"],
    required_user_confirmation: false,
    required_global_review: false,
    checked_at_ms: 1_764_000_004_000,
  };

  const planAuthorizationStore: PlanAuthorizationStoreV1 = {
    schema_version: "plan_authorization_store.v1",
    revision: 4,
    authorizations: [
      {
        authorization_id: "plan-auth:offline:active",
        schema_version: "plan_authorization.v1",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        source_proposal_id: "proposal:offline:001",
        title: "离线方案授权",
        goal_summary: "只允许离线夹具范围内的任务包检查。",
        status: "active",
        scope: {
          project_id: workflowProjectId,
          workflow_id: workflowId,
          allowed_role_ids: ["codex-dev"],
          allowed_agent_ids: [sessionThreadId],
          allowed_read_roots: [projectRoot],
          allowed_write_roots: ["/offline-fixture/projects/codex-workbench/src"],
          allowed_tools: ["read_file"],
          allowed_checks: [],
          allowed_task_package_kinds: ["task_package"],
          max_worker_dispatches: 1,
          max_runtime_minutes: 30,
          stop_conditions: [
            {
              condition_id: "requires-user-confirmation",
              kind: "requires_user_confirmation",
              summary: "需要用户确认时停止。",
              requires_user_confirmation: true,
            },
          ],
        },
        user_confirmation: {
          confirmed_by: "user",
          confirmed_at_ms: 1_764_000_001_000,
          confirmation_summary: "用户确认离线 fixture 方案授权范围。",
        },
        global_boundary_review: {
          reviewed_by: "global_director",
          reviewed_at_ms: 1_764_000_002_000,
          status: "approved",
          summary: "全局主管复核通过离线 fixture 边界。",
        },
        audit_refs: ["audit:plan-auth:offline:created", "audit:auto-dispatch-scope-checked:offline"],
        created_at_ms: 1_764_000_000_000,
        updated_at_ms: 1_764_000_002_000,
        expires_at_ms: null,
      },
    ],
    audit_events: [
      {
        audit_event_id: "audit:auto-dispatch-scope-checked:offline",
        event_type: "auto_dispatch_scope_checked",
        actor_id: "control_core",
        actor_role: "control_core",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        authorization_id: "plan-auth:offline:active",
        work_item_id: "work-item:offline:001",
        before_status: null,
        after_status: null,
        reason: "写入范围超出方案授权",
        guard_result: blockedAutoDispatchGuardResult,
        created_at_ms: 1_764_000_004_000,
      },
    ],
    updated_at_ms: 1_764_000_004_000,
    warnings: [],
  };

  const projectConsultationProposalStore: ProjectConsultationProposalStoreV1 = {
    schema_version: "project_consultation_proposal_store.v1",
    revision: 1,
    proposals: [
      {
        proposal_id: "proposal:offline:c2:pending",
        schema_version: "project_consultation_proposal.v1",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        title: "离线项目咨询方案草案",
        user_goal: "让用户先确认工作流自动推进的方案范围。",
        goal_summary: "确认任务包、角色、读写范围、工具和停止条件后，再进入全局复核。",
        proposed_steps: [
          "整理项目目标和当前任务包。",
          "确认允许角色、agent、读写范围和工具。",
          "用户确认后等待全局主管边界复核。",
        ],
        scope_draft: {
          allowed_role_ids: ["codex-dev", "project_director"],
          allowed_agent_ids: [sessionThreadId],
          allowed_read_roots: [projectRoot],
          allowed_write_roots: ["/offline-fixture/projects/codex-workbench/src"],
          allowed_tools: ["read_file"],
          allowed_checks: ["npm run typecheck"],
          allowed_task_package_kinds: ["task_package"],
          stop_conditions: ["超出读写范围或需要权限升级时必须停下。"],
          max_worker_dispatches: 3,
          max_runtime_minutes: 60,
        },
        risks: [
          {
            risk_id: "risk:offline:c2",
            severity: "warning",
            summary: "用户确认后仍不能自动派发。",
            mitigation: "等待全局主管复核。",
          },
        ],
        acceptance_criteria: ["确认后授权仍停在待全局复核。"],
        status: "pending_user_confirmation",
        plan_authorization_id: null,
        created_by_role: "project_consultant",
        created_at_ms: 1_764_000_005_000,
        updated_at_ms: 1_764_000_005_000,
      },
    ],
    decisions: [],
    audit_events: [
      {
        audit_event_id: "audit:project-consultation-proposal-created:offline",
        event_type: "project_consultation_proposal_created",
        actor_id: "project-consultation-fixture",
        actor_role: "project_consultant",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        proposal_id: "proposal:offline:c2:pending",
        plan_authorization_id: null,
        before_status: null,
        after_status: "pending_user_confirmation",
        reason: "创建离线项目咨询方案草案。",
        created_at_ms: 1_764_000_005_000,
      },
    ],
    updated_at_ms: 1_764_000_005_000,
    warnings: [],
  };

  const pendingGlobalBoundaryReviewAuthorization = {
    ...planAuthorizationStore.authorizations[0],
    authorization_id: "plan-auth:offline:pending-global",
    source_proposal_id: "proposal:offline:c3:confirmed",
    status: "pending_global_boundary_review" as const,
    user_confirmation: {
      confirmed_by: "user",
      confirmed_at_ms: 1_764_000_006_000,
      confirmation_summary: "用户确认离线 C3 方案范围。",
    },
    global_boundary_review: null,
    audit_refs: ["audit:plan-auth:offline:c3:created", "audit:plan-auth:offline:c3:user-confirmed"],
    updated_at_ms: 1_764_000_006_000,
  };

  const planAuthorizationStorePendingGlobal: PlanAuthorizationStoreV1 = {
    ...planAuthorizationStore,
    revision: 5,
    authorizations: [pendingGlobalBoundaryReviewAuthorization],
    audit_events: [],
    updated_at_ms: 1_764_000_006_000,
  };

  const projectConsultationProposalStoreConfirmed: ProjectConsultationProposalStoreV1 = {
    ...projectConsultationProposalStore,
    revision: 2,
    proposals: [
      {
        ...projectConsultationProposalStore.proposals[0],
        proposal_id: "proposal:offline:c3:confirmed",
        status: "user_confirmed",
        plan_authorization_id: "plan-auth:offline:pending-global",
        updated_at_ms: 1_764_000_006_000,
      },
    ],
    decisions: [
      {
        decision_id: "decision:offline:c3:user-confirmed",
        proposal_id: "proposal:offline:c3:confirmed",
        decided_by: "user",
        decision: "confirm",
        summary: "用户确认离线 C3 方案范围。",
        created_at_ms: 1_764_000_006_000,
      },
    ],
    updated_at_ms: 1_764_000_006_000,
  };

  const projectConsultationProposalStoreActive: ProjectConsultationProposalStoreV1 = {
    ...projectConsultationProposalStore,
    revision: 6,
    proposals: [
      {
        ...projectConsultationProposalStore.proposals[0],
        proposal_id: "proposal:offline:001",
        status: "user_confirmed",
        plan_authorization_id: "plan-auth:offline:active",
        updated_at_ms: 1_764_000_008_000,
      },
    ],
    decisions: [
      {
        decision_id: "decision:offline:c6:user-confirmed",
        proposal_id: "proposal:offline:001",
        decided_by: "user",
        decision: "confirm",
        summary: "用户确认离线 C6 方案范围。",
        created_at_ms: 1_764_000_008_000,
      },
    ],
    updated_at_ms: 1_764_000_008_000,
  };

  const projectDirectorTaskPlan: ProjectDirectorTaskPlan = {
    project_root: projectRoot,
    project_id: workflowProjectId,
    workflow_id: workflowId,
    proposal_id: "proposal:offline:c4:confirmed",
    authorization_id: "plan-auth:offline:active",
    actor_id: "project_director",
    planned_tasks: [
      {
        planned_task_id: "project-director-planned-task:offline:c4",
        title: "C4 准备态子任务",
        objective: "在授权范围内完成离线夹具检查。",
        scope: {
          project_id: workflowProjectId,
          workflow_id: workflowId,
          target_role: "codex-dev",
          task_package_kind: "task_package",
          allowed_read_scope: [projectRoot],
          allowed_write_scope: ["/offline-fixture/projects/codex-workbench/src"],
          callable_tool_capabilities: ["read_file"],
          required_checks: ["npm run typecheck"],
          stop_conditions: ["超出读写范围或需要权限升级时必须停下。"],
        },
        depends_on: [],
        acceptance_criteria: ["确认 prepared dispatch 仍然只是准备态。"],
        report_format: ["验证结果", "风险和下一步建议"],
        status: "authorized",
        guard_result: {
          status: "authorized",
          authorization_id: "plan-auth:offline:active",
          reasons: [],
          required_user_confirmation: false,
          required_global_review: false,
          checked_at_ms: 1_764_000_007_000,
        },
        work_item_id: "work-item:offline:c4",
        workflow_node_id: `${workflowId}:node:codex-dev`,
        task_package_id: "artifact:offline:task-package:c4",
        memory_packet_snapshot_id: "task-package-memory-packet-snapshot:v1:offline:c4",
        prepared_dispatch_id: null,
        blocked_reasons: [],
      },
    ],
    planned_task_count: 1,
    authorized_task_count: 1,
    prepared_dispatch_count: 0,
    blocked_count: 0,
    needs_binding_count: 0,
    blocked_reasons: [],
    memory_snapshot_summary: {
      snapshot_id: "task-package-memory-packet-snapshot:v1:offline:c4",
      included_count: 1,
      excluded_count: 0,
      review_material_count: 0,
      stale: false,
      stale_reasons: [],
      display_text: "任务包记忆快照：1 个 snapshot；使用了 1 条正式记忆；排除了 0 条候选 / 观察 / lint 阻断项；0 条待审查材料。",
      warnings: [],
    },
    display_text: "项目主管拆任务：计划 1 / 已授权 1 / 已准备 0 / 需绑定 0 / 阻断 0；准备派发仍未执行工作者。",
    warnings: ["prepared_dispatch_is_not_worker_execution"],
  };

  const runnableWorkflowRunCheck: WorkflowRunCheck = {
    project_root: projectRoot,
    workflow_id: workflowId,
    status: "runnable",
    evidence_completeness: "complete",
    blocked_reasons: [],
    warnings: [],
    checks: [
      {
        check_id: "missing_model",
        label: "模型",
        status: "pass",
        severity: "info",
        reason: "任务包已显式指定模型。",
        source_ref: "work-item:offline:001",
      },
      {
        check_id: "missing_memory_refs",
        label: "记忆引用",
        status: "pass",
        severity: "info",
        reason: "任务包没有声明需要记忆引用。",
        source_ref: "work-item:offline:001",
      },
    ],
  };

  return {
    blockedWorkflowRunCheck,
    planAuthorizationStore,
    projectConsultationProposalStore,
    planAuthorizationStorePendingGlobal,
    projectConsultationProposalStoreConfirmed,
    projectConsultationProposalStoreActive,
    projectDirectorTaskPlan,
    runnableWorkflowRunCheck,
  };
}
