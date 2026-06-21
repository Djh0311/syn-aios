// Canvas surface config (workflow-canvas two-surfaces-one-engine plan §3).
// One editable React Flow engine (WorkflowCanvasEngine), called with a per-
// surface config: capabilities / boundary / run target / authority are injected,
// not hard-coded. P0 shipped the `experiment` config; P1 adds the per-project
// `project` config (this file) so the project surface drives the same engine.

import {
  experimentCanvasBoundary,
  projectWorkflowCanvasBoundary,
  type CanvasSurfaceBoundary,
} from "./canvasSurfaceBoundaries";

export type CanvasSurfaceCapabilities = {
  edit: boolean;
  connect: boolean;
  createNode: boolean;
  saveTemplate: boolean;
  manualOrchestrate: boolean;
};

export type CanvasSurfaceConfig = {
  kind: "experiment" | "project";
  boundary: CanvasSurfaceBoundary;
  projectRoot?: string;
  capabilities: CanvasSurfaceCapabilities;
  views: ("plan" | "run_state")[];
  realRunTarget: "fixed_test_project" | "bound_project";
  authority: "free_canvas_def" | "workflow_state_read_model";
  showProjectRuleBar: boolean;
  // When true the engine skips its own <header> chrome — the host surface (e.g.
  // the project view) provides its own head / rule status bar / view toggle.
  embedded?: boolean;
};

// Experiment surface (sandbox): full free authoring, not a project fact source,
// real run only ever targets the fixed test project (current C double gate).
// Mirrors exactly what CanvasView rendered before the engine extraction.
export const experimentCanvasSurfaceConfig: CanvasSurfaceConfig = {
  kind: "experiment",
  boundary: experimentCanvasBoundary,
  capabilities: {
    edit: true,
    connect: true,
    createNode: true,
    saveTemplate: true,
    manualOrchestrate: true,
  },
  views: ["plan"],
  realRunTarget: "fixed_test_project",
  authority: "free_canvas_def",
  showProjectRuleBar: false,
};

// Project surface (real, controlled): same engine, bound to a real project.
// Authority is the workflow-state derived read model (the project fact source).
// Default surface is the read-only 运行状态 view; editing is an ACTION (草案 →
// 提交 → 通过), NOT a persistent view — the 方案/运行 toggle was removed per the
// 2026-06-21 真机反馈. Real run targets the bound project (P3, control core +
// double gate). Built per project so it carries the project root. `embedded`:
// the project view renders the head / rule bar / edit actions, so the engine
// drops its own header chrome.
export function projectCanvasSurfaceConfig(projectRoot: string): CanvasSurfaceConfig {
  return {
    kind: "project",
    boundary: projectWorkflowCanvasBoundary,
    projectRoot,
    capabilities: {
      edit: true,
      connect: true,
      createNode: true,
      saveTemplate: true,
      manualOrchestrate: true,
    },
    // 编辑是动作不是视图（真机后删切换）：默认就是只读运行状态。
    views: ["run_state"],
    realRunTarget: "bound_project",
    authority: "workflow_state_read_model",
    showProjectRuleBar: true,
    embedded: true,
  };
}
