import type { ProjectDirectorTaskPlan } from "./types";

export const projectDirectorPlannedTaskStatusLabels: Record<string, string> = {
  draft: "草稿",
  authorized: "授权检查通过",
  blocked: "越界任务已阻断",
  needs_binding: "等待会话绑定",
  prepared: "已准备；仍未执行工作者",
};

export function summarizeProjectDirectorTaskPlan(plan: ProjectDirectorTaskPlan | null) {
  if (!plan) {
    return {
      status_label: "未生成",
      display_text: "尚未生成项目主管拆任务草案",
      planned_task_count: 0,
      prepared_dispatch_count: 0,
      blocked_count: 0,
      needs_binding_count: 0,
      active_authorization_id: null as string | null,
      memory_text: "任务包记忆快照未生成",
      blocked_reasons: [] as string[],
    };
  }

  const statusLabel =
    plan.prepared_dispatch_count > 0
      ? "已准备；仍未执行工作者"
      : plan.blocked_count > 0
        ? "存在阻断"
        : plan.needs_binding_count > 0
          ? "等待会话绑定"
          : "可准备";

  return {
    status_label: statusLabel,
    display_text: plan.display_text,
    planned_task_count: plan.planned_task_count,
    prepared_dispatch_count: plan.prepared_dispatch_count,
    blocked_count: plan.blocked_count,
    needs_binding_count: plan.needs_binding_count,
    active_authorization_id: plan.authorization_id,
    memory_text: plan.memory_snapshot_summary.display_text,
    blocked_reasons: plan.blocked_reasons.slice(0, 3),
  };
}
