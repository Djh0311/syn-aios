import type {
  PendingAction,
  TaskPackageDispatchReadiness,
  TaskPackageFields,
} from "../../src/lib/types";

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

export function taskDraftFormValues(): Map<string, string> {
  return new Map<string, string>([
    ["task-title", "登记任务包草稿"],
    ["task-objective", "写入 work_items 和 artifacts"],
  ]);
}

export function taskDraftFormDataFixture(values: Map<string, string>): typeof FormData {
  return class {
    get(name: string) {
      return values.get(name) ?? null;
    }
  } as unknown as typeof FormData;
}

export function buildCreateTaskDraftAction(projectRoot: string): PendingAction {
  return {
    kind: "create-task-draft",
    label: "创建任务包草稿",
    path: projectRoot,
    source: "索引内项目路径",
    boundary: "只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行。",
    taskDraft: {
      projectRoot,
      title: "登记任务包草稿",
      objective: "写入 work_items 和 artifacts",
      assignedRole: "codex-dev",
    },
  };
}

export function buildCopyTaskPreviewAction(projectRoot: string, workItemId: string): PendingAction {
  return {
    kind: "copy-task-preview",
    label: "复制任务包 Markdown 预览",
    path: projectRoot,
    source: "索引内项目路径",
    boundary: "只复制预览文本到剪贴板；不写真实任务文件、不派发真实 Codex 会话。",
    taskPreview: {
      projectRoot,
      workItemId,
    },
  };
}

export function buildGenerateTaskFileAction(projectRoot: string, workItemId: string): PendingAction {
  return {
    kind: "generate-task-file",
    label: "生成任务包文件",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "写入 /Users/yoyi/workspace/product-line/tasks/ 下的新 Markdown 文件，并更新工作台自己的 workflow-state.v0.json；不覆盖已有任务包、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
    taskFileGeneration: {
      project_root: projectRoot,
      work_item_id: workItemId,
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

export function expectedUpdateTaskFieldsAction(projectRoot: string, workItemId: string): PendingAction {
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
        task_name: "字段编辑任务",
        assigned_line: "桌面应用线",
        background: ["来自结构化字段。"],
        goals: ["完成字段编辑。"],
        allowed_read: ["/tmp/indexed-project"],
        allowed_write: ["工作台状态文件"],
        forbidden_actions: ["不生成真实任务文件。"],
        acceptance_criteria: ["预览使用新字段。"],
        required_return: ["做了什么"],
        review_focus: ["确认结构化字段。"],
      },
    },
  };
}

export function expectedCorrectDispatchFieldsAction(
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

export function taskFieldCorrectionFixtures(projectRoot: string): {
  correctionFields: TaskPackageFields;
  missingPreviewFields: TaskPackageFields;
  fieldValues: Map<string, string>;
} {
  const correctionFields: TaskPackageFields = {
    task_name: "派发准备字段修正",
    assigned_line: "桌面应用线",
    background: ["用户提供真实背景。"],
    goals: ["用户提供真实目标。"],
    allowed_read: [projectRoot],
    allowed_write: ["product-line/prototypes/productized-desktop-shell/src/"],
    forbidden_actions: ["不派发真实 Codex 会话。", "不运行运行器。"],
    acceptance_criteria: ["字段保存后可复检 readiness。"],
    required_return: ["做了什么", "改了哪些文件", "验证命令和结果", "风险和下一步建议"],
    review_focus: ["确认没有编造业务目标。"],
  };
  const missingPreviewFields: TaskPackageFields = { ...correctionFields, goals: [], allowed_write: [] };
  const fieldValues = new Map<string, string>([
    ["task_name", "字段编辑任务"],
    ["assigned_line", "桌面应用线"],
    ["background", "来自结构化字段。"],
    ["goals", "完成字段编辑。"],
    ["allowed_read", "/tmp/indexed-project"],
    ["allowed_write", "工作台状态文件"],
    ["forbidden_actions", "不生成真实任务文件。"],
    ["acceptance_criteria", "预览使用新字段。"],
    ["required_return", "做了什么"],
    ["review_focus", "确认结构化字段。"],
  ]);

  return { correctionFields, missingPreviewFields, fieldValues };
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
