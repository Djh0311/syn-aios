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
