import type { ProjectCanvasDetailItem } from "../../lib/projectCanvas";

export function ProjectCanvasDetailLine({ item }: { item: ProjectCanvasDetailItem }) {
  return (
    <div className={`project-canvas-detail-line ${item.value_kind ?? "text"}`}>
      <span>{item.label}</span>
      <strong>{item.value}</strong>
    </div>
  );
}
