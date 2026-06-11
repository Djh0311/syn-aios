import type { WorkflowStateSnapshot } from "../../src/lib/types";

export function workflowStateReadyForReviewFixture(baseWorkflowState: WorkflowStateSnapshot): WorkflowStateSnapshot {
  return {
    ...baseWorkflowState,
    counts: {
      ...baseWorkflowState.counts,
      reviews: 0,
    },
    project_workflows: [
      {
        ...baseWorkflowState.project_workflows[0],
        task_drafts: [
          {
            ...baseWorkflowState.project_workflows[0].task_drafts[0],
            state: "ready_for_review",
            current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
            next_states: ["accepted", "needs_changes", "paused"],
            next_action_label: "下一步：接受或要求修改",
          },
          baseWorkflowState.project_workflows[0].task_drafts[1],
        ],
      },
    ],
  };
}

export function workflowStateWithPreparedOfflineDispatchFixture(
  baseWorkflowState: WorkflowStateSnapshot,
  projectRoot: string,
): WorkflowStateSnapshot {
  return {
    ...baseWorkflowState,
    project_workflows: [
      {
        ...baseWorkflowState.project_workflows[0],
        node_dispatches: [
          {
            dispatch_id: "offline-dispatch:fixture:prepared",
            project_id: "project:offline-fixture-projects-codex-workbench",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            work_item_id: "work-item:offline:001",
            binding_id: "offline-role-binding:codex-dev",
            native_thread_id: "offline-role:codex-dev",
            prompt_preview: "派发给：开发线\n任务名：已落账离线派发\n目标：验证回传使用已落账派发块。",
            prompt_kind: "offline_role_dispatch",
            offline_role_dispatch: {
              project_root: projectRoot,
              work_item_id: "work-item:offline:001",
              target_role_id: "codex-dev",
              target_role_label: "开发线",
              task_title: "已落账离线派发",
              objective: "验证回传使用已落账派发块。",
              execution_cwd: projectRoot,
              allowed_reads: [projectRoot],
              allowed_writes: [`${projectRoot}/README.md`],
              forbidden_actions: ["不执行 codex exec resume"],
              acceptance_criteria: ["角色回传摘要包含已落账任务名"],
              timeout_seconds: 600,
              required_return: ["薄弱点", "验证结果"],
              raw_block: "派发给：开发线\n任务名：已落账离线派发\n目标：验证回传使用已落账派发块。",
            },
            state: "prepared",
            started_at_ms: null,
            ended_at_ms: null,
            exit_code: null,
            last_message_path: null,
            last_message_summary: null,
            transcript_event_count: null,
            transcript_target_hits: null,
            warnings: ["offline_only_no_codex_resume"],
          },
          ...baseWorkflowState.project_workflows[0].node_dispatches,
        ],
      },
    ],
  };
}

export function workflowStateWithCompletedOfflineDispatchFixture(
  baseWorkflowState: WorkflowStateSnapshot,
  projectRoot: string,
): WorkflowStateSnapshot {
  return {
    ...baseWorkflowState,
    project_workflows: [
      {
        ...baseWorkflowState.project_workflows[0],
        task_drafts: [
          {
            ...baseWorkflowState.project_workflows[0].task_drafts[0],
            state: "ready_for_review",
            current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
            next_states: ["accepted", "needs_changes", "paused"],
            next_action_label: "下一步：接受或要求修改",
          },
          baseWorkflowState.project_workflows[0].task_drafts[1],
        ],
        node_dispatches: [
          {
            dispatch_id: "offline-dispatch:fixture:completed",
            project_id: "project:offline-fixture-projects-codex-workbench",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            work_item_id: "work-item:offline:001",
            binding_id: "offline-role-binding:codex-dev",
            native_thread_id: "offline-role:codex-dev",
            prompt_preview: "派发给：开发线\n任务名：已完成离线派发\n目标：验证总指导回收。",
            prompt_kind: "offline_role_dispatch",
            offline_role_dispatch: {
              project_root: projectRoot,
              work_item_id: "work-item:offline:001",
              target_role_id: "codex-dev",
              target_role_label: "开发线",
              task_title: "已完成离线派发",
              objective: "验证总指导回收。",
              execution_cwd: projectRoot,
              allowed_reads: [projectRoot],
              allowed_writes: [`${projectRoot}/README.md`],
              forbidden_actions: ["不执行 codex exec resume"],
              acceptance_criteria: ["总指导回收按钮可用"],
              timeout_seconds: 600,
              required_return: ["薄弱点", "验证结果"],
              raw_block: "派发给：开发线\n任务名：已完成离线派发\n目标：验证总指导回收。",
            },
            state: "completed",
            started_at_ms: null,
            ended_at_ms: 1_764_000_004_000,
            exit_code: 0,
            last_message_path: null,
            last_message_summary: "离线桩结果：已接收任务，没有执行真实 Codex 会话。",
            transcript_event_count: 0,
            transcript_target_hits: 0,
            warnings: ["offline_only_no_codex_resume"],
          },
          ...baseWorkflowState.project_workflows[0].node_dispatches,
        ],
      },
    ],
  };
}

export function workflowStateWithGeneratedTaskFileFixture(baseWorkflowState: WorkflowStateSnapshot): WorkflowStateSnapshot {
  return {
    ...baseWorkflowState,
    project_workflows: [
      {
        ...baseWorkflowState.project_workflows[0],
        task_drafts: [
          {
            ...baseWorkflowState.project_workflows[0].task_drafts[0],
            artifact_path: "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
          },
          baseWorkflowState.project_workflows[0].task_drafts[1],
        ],
      },
    ],
  };
}
