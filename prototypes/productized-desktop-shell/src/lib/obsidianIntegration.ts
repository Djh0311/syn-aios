import { invoke } from "@tauri-apps/api/core";

// L3 Obsidian integration: this is deliberately a narrow client, not a CLI
// wrapper.  The backend owns the official binary, the fixed Syn vault, argv,
// timeout and output caps.  Callers can choose only named user actions below.

export const OBSIDIAN_TAURI_COMMANDS = {
  status: "obsidian_integration_status",
  open_vault: "obsidian_integration_open_vault",
  open_note: "obsidian_integration_open_note",
  open_search: "obsidian_integration_open_search",
  read_note: "obsidian_integration_read_note",
  search_notes: "obsidian_integration_search_notes",
} as const;

export type ObsidianIntegrationInvokeName = (typeof OBSIDIAN_TAURI_COMMANDS)[keyof typeof OBSIDIAN_TAURI_COMMANDS];

export type ObsidianIntegrationStatus =
  | "not_installed"
  | "installed"
  | "app_not_running"
  | "cli_not_enabled"
  | "ready"
  | "incompatible";

export const OBSIDIAN_INTEGRATION_ACTIONS = [
  "open_vault",
  "open_note",
  "open_search",
  "read_note",
  "search_notes",
] as const;

export type ObsidianIntegrationAction = (typeof OBSIDIAN_INTEGRATION_ACTIONS)[number];

export type ObsidianIntegrationStatusSnapshot = Readonly<{
  status: ObsidianIntegrationStatus;
  // User-safe backend copy.  It must not contain argv, absolute vault paths,
  // environment details, or arbitrary CLI output.
  message: string | null;
  app_version: string | null;
  cli_version: string | null;
  vault_label: string;
}>;

export type ObsidianIntegrationActionReceipt = Readonly<{
  action: ObsidianIntegrationAction;
  message: string;
  degraded: boolean;
}>;

export type ObsidianIntegrationNote = Readonly<{
  slug: string;
  title: string;
  body: string;
  mtime_ms: number;
}>;

export type ObsidianIntegrationSearchResult = Readonly<{
  slug: string;
  title: string;
  snippet: string;
  mtime_ms: number;
}>;

export type ObsidianIntegrationInvoke = <T>(
  command: ObsidianIntegrationInvokeName,
  args?: Record<string, unknown>,
) => Promise<T>;

export type ObsidianIntegrationClient = Readonly<{
  status: () => Promise<ObsidianIntegrationStatusSnapshot>;
  openVault: () => Promise<ObsidianIntegrationActionReceipt>;
  openNote: (slug: string) => Promise<ObsidianIntegrationActionReceipt>;
  openSearch: (query: string) => Promise<ObsidianIntegrationActionReceipt>;
  readNote: (slug: string) => Promise<ObsidianIntegrationNote>;
  searchNotes: (query: string) => Promise<ObsidianIntegrationSearchResult[]>;
}>;

const CLIENT_ERROR_INVALID_SLUG = "obsidian_integration_invalid_slug";
const CLIENT_ERROR_INVALID_QUERY = "obsidian_integration_invalid_query";
const MAX_SEARCH_QUERY_LENGTH = 240;

export function createObsidianIntegrationClient(
  invokeCommand: ObsidianIntegrationInvoke = invokeObsidianIntegration,
): ObsidianIntegrationClient {
  return Object.freeze({
    status: () => invokeCommand<ObsidianIntegrationStatusSnapshot>(OBSIDIAN_TAURI_COMMANDS.status),
    openVault: () => invokeCommand<ObsidianIntegrationActionReceipt>(OBSIDIAN_TAURI_COMMANDS.open_vault),
    openNote: async (slug) =>
      invokeCommand<ObsidianIntegrationActionReceipt>(OBSIDIAN_TAURI_COMMANDS.open_note, {
        slug: safeNoteSlug(slug),
      }),
    openSearch: async (query) =>
      invokeCommand<ObsidianIntegrationActionReceipt>(OBSIDIAN_TAURI_COMMANDS.open_search, {
        query: safeSearchQuery(query),
      }),
    readNote: async (slug) =>
      invokeCommand<ObsidianIntegrationNote>(OBSIDIAN_TAURI_COMMANDS.read_note, {
        slug: safeNoteSlug(slug),
      }),
    searchNotes: async (query) =>
      invokeCommand<ObsidianIntegrationSearchResult[]>(OBSIDIAN_TAURI_COMMANDS.search_notes, {
        query: safeSearchQuery(query),
      }),
  });
}

// Production client.  Tests inject a local recorder instead, so they never
// start Syn, Obsidian, a CLI process, or a real Tauri command.
export const obsidianIntegration = createObsidianIntegrationClient();

export function obsidianStatusLabel(status: ObsidianIntegrationStatus): string {
  switch (status) {
    case "not_installed":
      return "尚未安装官方 Obsidian";
    case "installed":
      return "已安装 Obsidian，正在检测连接";
    case "app_not_running":
      return "Obsidian 未运行";
    case "cli_not_enabled":
      return "Obsidian CLI 未启用";
    case "ready":
      return "已连接 Obsidian";
    case "incompatible":
      return "Obsidian 版本不兼容";
  }
}

export function obsidianActionLabel(action: ObsidianIntegrationAction): string {
  switch (action) {
    case "open_vault":
      return "打开 Obsidian 知识库";
    case "open_note":
      return "打开笔记";
    case "open_search":
      return "在 Obsidian 中搜索";
    case "read_note":
      return "读取笔记";
    case "search_notes":
      return "搜索笔记";
  }
}

// Backend error text is treated as untrusted diagnostic material.  Known,
// stable codes receive user-safe Chinese guidance; unknown text is never shown
// verbatim because it may contain a path, argv or platform detail.
export function humanizeObsidianIntegrationError(error: unknown): string {
  const raw = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  if (raw.includes("obsidian_integration_not_installed")) return "尚未安装官方 Obsidian；知识库原生阅读和编辑仍可使用。";
  if (raw.includes("obsidian_integration_app_not_running")) return "Obsidian 未运行；请先正常打开应用后重试。";
  if (raw.includes("obsidian_integration_cli_not_enabled")) return "Obsidian CLI 尚未启用；请在 Obsidian 设置中按官方流程启用后重试。";
  if (raw.includes("obsidian_integration_incompatible")) return "当前 Obsidian 版本不兼容；请使用受支持的官方版本。";
  if (raw.includes("obsidian_integration_invalid_slug")) return "笔记定位不符合当前知识库边界。";
  if (raw.includes("obsidian_integration_invalid_query")) return "搜索内容不符合当前知识库边界。";
  if (raw.includes("obsidian_integration_timeout")) return "Obsidian 操作超时；请确认应用仍在运行后重试。";
  if (raw.includes("obsidian_integration_output_limit")) return "结果过大，已被安全限制拒绝。请缩小搜索范围后重试。";
  return "Obsidian 集成操作未完成；请检查连接状态后重试。";
}

function invokeObsidianIntegration<T>(
  command: ObsidianIntegrationInvokeName,
  args?: Record<string, unknown>,
): Promise<T> {
  return invoke<T>(command, args);
}

function safeNoteSlug(slug: string): string {
  if (
    typeof slug !== "string"
    || !slug.trim()
    || slug !== slug.trim()
    || slug.startsWith(".")
    || /[\\/\u0000-\u001f\u007f]/.test(slug)
  ) {
    throw new Error(CLIENT_ERROR_INVALID_SLUG);
  }
  return slug;
}

function safeSearchQuery(query: string): string {
  if (typeof query !== "string") throw new Error(CLIENT_ERROR_INVALID_QUERY);
  const normalized = query.trim();
  if (
    !normalized
    || normalized.length > MAX_SEARCH_QUERY_LENGTH
    || normalized.startsWith("-")
    || normalized.startsWith("/")
    || /[\\\u0000-\u001f\u007f]/.test(normalized)
  ) {
    throw new Error(CLIENT_ERROR_INVALID_QUERY);
  }
  return normalized;
}
