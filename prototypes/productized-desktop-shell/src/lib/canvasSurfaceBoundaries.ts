export type CanvasSurfaceBoundaryKind = "experiment_canvas" | "project_workflow_canvas";

export type CanvasSurfaceBoundaryItem = {
  item_id: string;
  label: string;
  value: string;
};

export type CanvasSurfaceBoundary = {
  boundary_id: string;
  context_kind: CanvasSurfaceBoundaryKind;
  eyebrow: string;
  title: string;
  summary: string;
  badges: string[];
  items: CanvasSurfaceBoundaryItem[];
  warnings: string[];
};

export const experimentCanvasBoundary: CanvasSurfaceBoundary = {
  boundary_id: "experiment-canvas-boundary:v1",
  context_kind: "experiment_canvas",
  eyebrow: "experiment / template / canvas library",
  title: "实验 / 模板画布",
  summary: "用于实验、模板、草图和后置 canvas library；不是项目 workflow 事实源。",
  badges: ["不会写项目事实", "不会写正式记忆", "MCP canvas run 非默认项目工作流"],
  items: [
    { item_id: "project-facts", label: "项目事实", value: "不会写项目事实" },
    { item_id: "formal-memory", label: "正式记忆", value: "不会写正式记忆" },
    { item_id: "workflow-state", label: "工作流状态", value: "不会写项目工作流状态" },
    { item_id: "authority", label: "事实源", value: "不是项目 workflow 事实源" },
    {
      item_id: "experiment-run",
      label: "实验运行",
      value: "只属于实验画布语境，不会自动写正式项目事实、正式记忆或项目 workflow",
    },
    {
      item_id: "project-run",
      label: "正式项目运行",
      value: "请回项目工作流，并经过控制核心 / 权限 / 审计",
    },
  ],
  warnings: [
    "experiment_canvas_is_not_project_workflow_authority",
    "experiment_run_does_not_write_formal_project_facts",
    "mcp_canvas_run_is_not_default_project_workflow",
  ],
};

export const projectWorkflowCanvasBoundary: CanvasSurfaceBoundary = {
  boundary_id: "project-workflow-canvas-boundary:v1",
  context_kind: "project_workflow_canvas",
  eyebrow: "project / workflow / authorization",
  title: "项目工作流画布",
  summary: "服务项目、工作流、授权和控制核心；事实源来自工作流状态派生读模型。",
  badges: ["工作流状态派生读模型", "方案授权 / 控制核心 / 权限 / 审计", "React Flow 仅负责渲染"],
  items: [
    { item_id: "authority", label: "事实源", value: "工作流状态派生读模型" },
    { item_id: "control-core", label: "控制边界", value: "方案授权 / 控制核心 / 权限 / 审计" },
    { item_id: "renderer", label: "渲染层", value: "React Flow 仅负责渲染" },
    {
      item_id: "project-runtime",
      label: "运行和变更",
      value: "任务包、权限、记忆包、读回、审计和结果回收都在项目工作流边界内",
    },
    { item_id: "experiment-canvas", label: "实验画布", value: "实验画布不会写入本项目事实" },
  ],
  warnings: [
    "project_workflow_canvas_uses_workflow_state_read_model",
    "project_runtime_requires_control_core_permission_and_audit",
    "react_flow_is_renderer_not_authority",
  ],
};

export const canvasBoundaryForbiddenPhrases = [
  "实验运行已写项目状态",
  "MCP canvas run 已成为正式 workflow",
  "实验画布已并入项目",
  "独立 CanvasDefinition 是项目事实源",
  "已写正式记忆",
  "已派发项目工作者",
  "工作者已执行",
  "Codex 已收到任务",
  "自动派发已开始",
  "自动重试已完成",
  "运行日志已完成",
  "阶段 G 已验收",
];
