// Canvas surface config (workflow-canvas two-surfaces-one-engine plan §3).
// One editable React Flow engine (WorkflowCanvasEngine), called with a per-
// surface config: capabilities / boundary / run target / authority are injected,
// not hard-coded. P0 only ships the `experiment` config — it must reproduce the
// current CanvasView behaviour exactly. `project` config is plan P1.

import { experimentCanvasBoundary, type CanvasSurfaceBoundary } from "./canvasSurfaceBoundaries";

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
