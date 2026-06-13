import { createElement } from "react";
import type { WorkflowRunCheck } from "../../lib/types";

export function stateLabel(state: string) {
  if (state === "empty") return "空态";
  if (state === "idle") return "空闲";
  if (state === "draft") return "草稿";
  if (state === "prepared") return "准备派发";
  if (state === "ready_to_dispatch") return "待派发";
  if (state === "running") return "执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "needs_review") return "待复核";
  if (state === "retry_pending") return "待重试";
  if (state === "failed") return "失败";
  if (state === "timed_out") return "已超时";
  if (state === "readback_unavailable") return "读回不可用";
  if (state === "cancelled") return "已取消";
  if (state === "ready_for_review") return "待回收";
  if (state === "accepted") return "已接受";
  if (state === "needs_changes") return "需修改";
  if (state === "paused") return "暂停";
  return state || "未知";
}

export function stateActionLabel(state: string) {
  if (state === "ready_to_dispatch") return "标记待派发";
  if (state === "running") return "标记执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "retry_pending") return "安排重试";
  if (state === "failed") return "标记失败";
  if (state === "timed_out") return "标记超时";
  if (state === "cancelled") return "请求取消";
  if (state === "ready_for_review") return "标记待回收";
  if (state === "accepted") return "接受";
  if (state === "needs_changes") return "要求修改";
  if (state === "paused") return "暂停";
  return stateLabel(state);
}

export function roleLabel(role?: string | null) {
  if (role === "codex-dev") return "Codex 开发线";
  if (role === "desktop-app") return "桌面应用线";
  if (role === "director") return "总指导";
  if (role === "review") return "回收评审";
  return role || "未指派";
}

export function runCheckTone(status?: WorkflowRunCheck["status"] | null): "candidate" | "warning" | "unknown" {
  if (status === "runnable") return "candidate";
  if (status === "warning" || status === "blocked") return "warning";
  return "unknown";
}

export function runCheckStatusLabel(status: WorkflowRunCheck["status"]) {
  if (status === "runnable") return "检查通过，可以进入后续人工确认";
  if (status === "warning") return "有警告，仍需人工判断";
  if (status === "blocked") return "有阻塞，不能运行或派发";
  return status || "未知状态";
}

export function runCheckItemStatusLabel(status: string) {
  if (status === "pass") return "通过";
  if (status === "warning") return "警告";
  if (status === "blocked") return "阻断";
  if (status === "not_ready") return "未就绪";
  if (status === "ready") return "就绪";
  return status || "未知";
}

export function listText(values: string[], emptyText: string) {
  return values.length ? values.join("；") : emptyText;
}

export function DetailLine({ label, value }: { label: string; value: string }) {
  return createElement(
    "div",
    { className: "detail-line" },
    createElement("span", null, label),
    createElement("strong", null, value),
  );
}

export function WorkflowNode({
  title,
  detail,
  meta,
  tone,
}: {
  title: string;
  detail: string;
  meta: string;
  tone: "project" | "codex" | "artifact" | "harness" | "gap";
}) {
  return createElement(
    "div",
    { className: `workflow-node ${tone}` },
    createElement("span", null, title),
    createElement("strong", null, detail),
    createElement("em", null, meta),
  );
}
