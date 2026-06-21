// Experiment canvas surface = a thin shell over the shared WorkflowCanvasEngine.
// (workflow-canvas two-surfaces-one-engine plan §6 P0). All editable React Flow
// logic now lives in WorkflowCanvasEngine; this just wires the engine with the
// experiment surface config inside a ReactFlowProvider. Behaviour is unchanged.
import { ReactFlowProvider } from "@xyflow/react";

import { experimentCanvasSurfaceConfig } from "../lib/canvasSurfaceConfig";
import type { SessionRecord } from "../lib/types";
import { WorkflowCanvasEngine } from "./WorkflowCanvasEngine";

type CanvasViewProps = {
  canvasId: string;
  sessions: SessionRecord[];
  onNotice: (msg: string) => void;
};

export function CanvasViewWithProvider(props: CanvasViewProps) {
  return (
    <ReactFlowProvider>
      <WorkflowCanvasEngine config={experimentCanvasSurfaceConfig} {...props} />
    </ReactFlowProvider>
  );
}
