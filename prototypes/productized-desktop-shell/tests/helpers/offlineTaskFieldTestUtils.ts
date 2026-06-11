import type { PendingAction, TaskPackageDispatchReadiness, TaskPackageFields } from "../../src/lib/types";

export function buildNotReadyDispatchReadiness(projectRoot: string): TaskPackageDispatchReadiness {
  return {
    project_root: projectRoot,
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    work_item_id: "work-item:offline:001",
    artifact_id: "artifact:offline:001",
    artifact_path: "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
    status: "not_ready",
    blocking_reasons: [
      "任务名为空、待补充或仍像测试草稿。",
      "禁止事项仍包含和当前生成行为冲突的历史禁令。",
    ],
    warnings: [],
    can_generate_next_version: false,
    memory_injection_summary: {
      snapshot_id: null,
      included_count: 0,
      excluded_count: 0,
      review_material_count: 0,
      stale: true,
      stale_reasons: ["task_memory_packet_snapshot_missing"],
      display_text:
        "任务包记忆注入摘要：尚未生成任务包记忆快照。仅活跃正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。",
      warnings: ["task_memory_packet_snapshot_missing"],
    },
  };
}

export function buildUpdateTaskFieldsAction(
  projectRoot: string,
  workItemId: string,
  values: Map<string, string>,
): PendingAction {
  return {
    kind: "update-task-fields",
    label: "保存任务包字段",
    path: projectRoot,
    source: "索引内项目路径",
    boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
    taskFields: {
      project_root: projectRoot,
      work_item_id: workItemId,
      fields: {
        task_name: values.get("task_name") ?? "",
        assigned_line: values.get("assigned_line") ?? "",
        background: listValue(values.get("background")),
        goals: listValue(values.get("goals")),
        allowed_read: listValue(values.get("allowed_read")),
        allowed_write: listValue(values.get("allowed_write")),
        forbidden_actions: listValue(values.get("forbidden_actions")),
        acceptance_criteria: listValue(values.get("acceptance_criteria")),
        required_return: listValue(values.get("required_return")),
        review_focus: listValue(values.get("review_focus")),
      },
    },
  };
}

export function buildCorrectDispatchFieldsAction(
  projectRoot: string,
  workItemId: string,
  fields: TaskPackageFields,
): PendingAction {
  return {
    kind: "correct-dispatch-fields",
    label: "保存派发字段修正",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
    dispatchFields: {
      project_root: projectRoot,
      work_item_id: workItemId,
      fields,
    },
  };
}

function listValue(value: string | undefined): string[] {
  return (value ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}
