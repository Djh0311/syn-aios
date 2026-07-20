// 人话工程①（2026-07-20）：前端错误翻译族单一真源。
// 10 个错误翻译函数自 7 个文件逐字原样迁入（人话输出串/判据/分支/兜底零变化），
// 原位置 import-back / re-export 保导入面（照 ProjectJiaobanPanel re-export 既有先例）。
// 后端单一真源 = src-tauri/src/run_error_translation.rs；本模块是其前端姊妹，不做大翻译层。

import type { CodexTranscriptEvent, ProjectWorkflowChainStatus } from "./types";

// ============================================================
// 交办 Panel：预拆 / 授权错误（原 ProjectJiaobanPanel.tsx）
// ============================================================

// 预拆偶发早退（flaky·后端已自动重试一次仍可能空）→ 人话，优雅降级：不影响批。
export function humanizePreviewError(error: unknown): string {
  const raw = error instanceof Error ? error.message : String(error ?? "");
  if (/找不到方案|proposal/i.test(raw)) return "这份方案暂时读不到，右侧预演画布的工序图没画出来（可重试）。";
  return "右侧预演画布的工序图没画出来（可重试）；不影响你批。";
}

// 判是不是合流命令对「已确认」方案的那一类干净拒（方案不是待用户确认状态）。
// 命中 → 授权本还活着、不用重批，卡住脸该给[接着跑,不用重批]（而非引导重新出方案）。
export function isAlreadyConfirmedRejection(e: unknown): boolean {
  const raw = e instanceof Error ? e.message : String(e);
  return (
    raw.includes("待用户确认") ||
    raw.includes("PendingUserConfirmation") ||
    raw.includes("不是「待") ||
    raw.includes("方案不是待")
  );
}

// fix8：供给类错误识别（codex 额度 / 订阅 / 登录 / 服务不可用）。姊妹后端包会带 codex_provider_unavailable:
// 前缀 + 人话，直接取其人话；后端未落地前用兜底关键词匹配。返回 null = 不是供给类（交给别的 humanize）。
export function humanizeProviderUnavailable(e: unknown): string | null {
  const raw = e instanceof Error ? e.message : String(e ?? "");
  const marker = "codex_provider_unavailable";
  if (raw.includes(marker)) {
    const after = raw.split(marker)[1]?.replace(/^["'\s]*[:：]?["'\s]*/, "").trim();
    return after && after.length > 0 ? after : "codex 额度 / 订阅 / 登录不可用——处理后点重试。";
  }
  if (/\b403\b|SUBSCRIPTION|quota|usage limit|\b401\b|unauthorized|consult_last_message_read_failed/i.test(raw)) {
    return "codex 服务不可用（常见：额度用完 / 订阅过期 / 登录失效）——处理后点重试；若是网络抽风，重试一次通常就过。";
  }
  return null;
}

// 合流命令的报错翻人话。最要紧的一类：对「已确认」方案后端会拒（方案不是待用户确认状态）——
// 那不是系统坏了，是这份方案已经批过、授权还活着，引导用户点[接着跑,不用重批]而非重批。
export function humanizeAuthorizeError(e: unknown): string {
  // fix8：合流 / 接着跑撞供给死时，同用供给类人话（否则裸抛英文栈让人以为系统坏了）。
  const provider = humanizeProviderUnavailable(e);
  if (provider) return provider;
  const raw = e instanceof Error ? e.message : String(e);
  // 后端拒词：ProjectConsultationProposalStatus 不是 PendingUserConfirmation 时的那句。
  // 这份已批过 → 不裸抛原始错误，翻成「已经批过了，点下面接着跑」。
  if (isAlreadyConfirmedRejection(e)) {
    return "这份方案已经批过了——不用重批，点下面「接着跑」，会从拆任务接着往下推进。";
  }
  if (raw.includes("找不到方案")) {
    return "找不到这份方案了（可能已被新方案取代）。点「重新出方案」重新说一遍目标。";
  }
  return raw;
}

// ============================================================
// 会话中心：转录读取错误（原 AgentConversationShell.tsx）
// ============================================================

export type TranscriptErrorCategory = "data_missing" | "filesystem" | "parse" | "safety" | "system";

export type TranscriptErrorInfo = {
  code: string;
  category: TranscriptErrorCategory;
  title: string;
  message: string;
};

export function normalizeTranscriptError(rawError: string): TranscriptErrorInfo {
  const code = rawError.split(":")[0] || "unexpected_internal_error";
  if (code === "session_not_found") {
    return {
      code,
      category: "data_missing",
      title: "会话不在当前目录中",
      message: "sqlite 和兼容索引都没有找到该 thread，无法读取正文。",
    };
  }
  if (code === "rollout_missing") {
    return {
      code,
      category: "data_missing",
      title: "没有可读回放记录",
      message: "该会话目录存在，但对应的回放记录文件缺失或不是文件。",
    };
  }
  if (code === "rollout_outside_allowed_dirs") {
    return {
      code,
      category: "safety",
      title: "路径被安全边界拒绝",
      message: "回放记录路径不在 Codex 主目录的 sessions 或 archived_sessions 目录下。",
    };
  }
  if (code === "filesystem_read_failed") {
    return {
      code,
      category: "filesystem",
      title: "文件读取失败",
      message: "系统无法读取回放记录文件；请检查文件是否仍存在以及权限是否可读。",
    };
  }
  if (code === "jsonl_parse_failed") {
    return {
      code,
      category: "parse",
      title: "回放记录格式无法解析",
      message: "会话正文格式异常，当前无法安全展示。",
    };
  }
  if (code === "sqlite_unavailable") {
    return {
      code,
      category: "system",
      title: "会话目录暂不可用",
      message: "Codex sqlite 目录不可读，且没有可用的兼容索引条目。",
    };
  }
  if (code === "transcript_reader_unavailable") {
    return {
      code,
      category: "system",
      title: "历史读取器不可用",
      message: "旧会话记录读取器不可用；会话中心主路径不应依赖它。",
    };
  }
  return {
    code,
    category: "system",
    title: "读取失败",
    message: "会话正文暂时无法读取。底层错误已归类为系统错误。",
  };
}

// ============================================================
// 转录 live 标题 / 详情（原 TranscriptViews.tsx）；
// commandFromArguments / firstLine 是 friendlyLiveDetail 的私有依赖，逐字随迁。
// ============================================================

export function friendlyLiveTitle(title: string): string {
  const labels: Record<string, string> = {
    "Codex 开始处理": "开始处理",
    "Codex 正在回复": "正在回复",
    "Codex 回复完成": "回复完成",
    "Codex 完成": "完成",
    "Codex 失败": "失败",
    "对话已创建": "已创建对话",
    "思考中": "正在思考",
    "思考完成": "思考完成",
    "正在运行命令": "正在运行命令",
    "命令完成": "命令完成",
    "正在调用工具": "正在调用工具",
    "工具完成": "工具完成",
    "工具输出": "工具输出",
  };
  return labels[title] ?? title;
}

export function friendlyLiveDetail(event: CodexTranscriptEvent, liveStatus: string | null, liveEventType: string | null): string {
  const command = commandFromArguments(event.arguments);
  if (command) return command;
  const stdout = event.stdout?.trim();
  if (stdout) return firstLine(stdout);
  const liveStatusLabel = liveStatus === "running" ? "运行中" : liveStatus === "completed" ? "已完成" : liveStatus === "failed" ? "失败" : "";
  const eventTypeLabel = liveEventType ? liveEventType.replace("item.", "").replace("turn.", "") : "";
  return [liveStatusLabel, eventTypeLabel].filter(Boolean).join(" · ");
}

export function commandFromArguments(value: unknown): string {
  if (!value || typeof value !== "object" || Array.isArray(value)) return "";
  const record = value as Record<string, unknown>;
  const cmd = record.cmd;
  if (typeof cmd === "string") return cmd;
  if (Array.isArray(cmd)) return cmd.map(String).join(" ");
  return "";
}

export function firstLine(text: string): string {
  return text.split(/\r?\n/)[0]?.trim() ?? "";
}

// ============================================================
// 历史 / 授权 / 进度（原 JiaobanHistory / JiaobanAuthorizeStates / JiaobanRunningStates）
// ============================================================

// A·错误族 → 人话短标（前端映射·不露 family 机器键；未知族兜底「运行错误」）。
export function historyErrorFamilyLabel(family: string): string {
  switch (family) {
    case "provider_unavailable":
      return "供给不可用";
    case "network":
      return "网络抽风";
    case "timeout":
      return "超时";
    case "sandbox_denied":
      return "权限/沙箱";
    case "command_failed":
      return "命令失败";
    case "codex_subsystem":
      return "codex 子系统";
    case "readback_failed":
      return "口供没读回";
    default:
      return "运行错误";
  }
}

// verdict 英文枚举 → 人话（词表死线）。
export function humanizeVerdict(v: string): string {
  switch (v) {
    case "pass":
      return "通过";
    case "needs_rework":
      return "要返工";
    case "needs_human_check":
    case "human_verify":
      return "建议你亲验";
    case "looks_ok":
      return "看着没问题";
    case "mismatch":
      return "对不上目标";
    case "caution":
      return "留个心";
    default:
      return v;
  }
}

// 人话优先(07-17 用户拍):界面默认只说「哪个目录」,完整路径收进「工程详情」。
export function humanizeWriteRoots(roots: string[]): string {
  const names = roots
    .map((root) => root.replace(/\/+$/, "").split("/").pop() || root)
    .filter(Boolean);
  return names.length ? `就在「${names.join("、")}」目录里` : "";
}

// 链状态 → 「正在…第 x/y 步」。链事件还没出现的阶段（拿不到节点）= 主管还在拆任务，据实说清。
export function humanizeChainProgress(
  chainStatus: ProjectWorkflowChainStatus | null,
  directorPlanningElapsedMinutes: number,
): string {
  if (!chainStatus || chainStatus.nodes.length === 0) {
    return `主管正在拆任务 · 已 ${Math.max(0, directorPlanningElapsedMinutes)} 分钟`;
  }
  const total = chainStatus.nodes.length;
  const done = countDoneNodes(chainStatus);
  const current = Math.min(done + 1, total);
  return `正在做第 ${current}/${total} 步…`;
}

function countDoneNodes(chainStatus: ProjectWorkflowChainStatus | null): number {
  if (!chainStatus) return 0;
  return chainStatus.nodes.filter((node) => /(finished|completed|done|succeeded|accepted)/i.test(node.state)).length;
}

// ============================================================
// App notice 薄委托（人话工程①②新增）：语义照后端 run_error_translation::humanize_error_for_display——
// 命中已知族 → 人话；未命中（unknown）→ 原文逐字回退，不硬塞人话盖原文。
// ============================================================
export function humanizeNoticeMessage(raw: string): string {
  const provider = humanizeProviderUnavailable(raw);
  if (provider) return provider;
  return raw;
}
