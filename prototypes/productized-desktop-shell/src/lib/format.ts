export function formatDate(value?: number | string | null): string {
  if (!value) return "未知";
  const date = typeof value === "number" ? new Date(value) : new Date(value);
  if (Number.isNaN(date.getTime())) return "未知";
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function shortId(value: string): string {
  return value.length > 10 ? value.slice(0, 8) : value;
}

export function relativeTime(value?: number | string | null, now: number = Date.now()): string {
  if (!value) return "时间未知";
  const ms = typeof value === "number" ? value : new Date(value).getTime();
  if (Number.isNaN(ms)) return "时间未知";
  const diff = now - ms;
  if (diff < 0) return formatDate(ms);
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return "刚刚";
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`;
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`;
  if (diff < 7 * day) return `${Math.floor(diff / day)} 天前`;
  return formatDate(ms);
}

export function pathTail(path?: string | null): string {
  if (!path) return "";
  const parts = path.split("/").filter(Boolean);
  return parts.at(-1) || path;
}

export function warningText(warnings: string[]): string {
  return warnings.length ? warnings.join(", ") : "无";
}

export function projectName(path: string): string {
  const parts = path.split("/").filter(Boolean);
  return parts.at(-1) || path || "未知项目";
}

// 状态机 raw 值 → 人话（宪法 §四.3 禁机器内部术语上脸）。
// 04 施工：原本只长在 RightDetailPanel 里；审计账本页也要同一套说法，故上提到共享层，
// 避免再造第五套翻译器（既有重复翻译器已被逮过）。缺映射时原样回落，不假装认识。
export function displayStatus(value: string | null | undefined): string {
  if (!value) return "未记录";
  const labels: Record<string, string> = {
    active: "活跃",
    approved: "已批准",
    archived: "已归档",
    blocked: "阻断",
    completed: "已完成",
    degraded: "降级",
    degraded_readonly: "只读降级",
    done: "已完成",
    err: "错误",
    error: "错误",
    failed: "失败",
    healthy: "健康",
    info: "信息",
    missing: "缺失",
    neutral: "中性",
    ok: "正常",
    open: "打开",
    pending: "待处理",
    ready_for_review: "待复核",
    ready_to_dispatch: "待派发",
    retry_pending: "待重试",
    run: "运行",
    running: "运行中",
    state_未登记: "状态未登记",
    succeeded: "成功",
    timed_out: "超时",
    unknown: "未知",
    waiting_for_permission: "等待权限",
    warning: "警告",
    readback_unavailable: "读回不可用",
    readback_failed: "读回失败",
    blocked_by_guard: "被边界阻断",
    needs_user: "需要用户处理",
    needs_user_confirmation: "需要用户确认",
    needs_review: "需要复核",
  };
  return labels[value] ?? value;
}

// 运行日志类别 → 人话。与 displayStatus 同因上提（右栏抽屉 + 审计账本页共用）。
export function runtimeLogCategoryLabel(category: string): string {
  if (category === "app_session") return "应用会话";
  if (category === "workflow_run") return "工作流运行";
  if (category === "dispatch_attempt") return "派发尝试";
  if (category === "readback") return "读回";
  if (category === "permission_wait") return "权限等待";
  if (category === "diagnostic_event") return "诊断事件";
  return category;
}

// 列表行时间（三元素定式的第三元素）。真拿不到就回 null，由调用方决定留白——不编时间。
export function listRowTimeLabel(value?: string | null): string | null {
  if (!value) return null;
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return null;
  const pad = (input: number) => `${input}`.padStart(2, "0");
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
