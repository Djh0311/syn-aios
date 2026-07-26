// L3 Obsidian typed client: offline API-shape and pure helper assertions.
// The injected recorder means this file does not start Syn, Obsidian, a CLI,
// a vault, or any real Tauri command.
import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  OBSIDIAN_INTEGRATION_ACTIONS,
  OBSIDIAN_TAURI_COMMANDS,
  createObsidianIntegrationClient,
  humanizeObsidianIntegrationError,
  obsidianActionLabel,
  obsidianStatusLabel,
  type ObsidianIntegrationInvoke,
  type ObsidianIntegrationInvokeName,
  type ObsidianIntegrationStatus,
} from "../src/lib/obsidianIntegration";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[obsidian-integration] ${message}`);
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

async function expectRejected(operation: Promise<unknown>, marker: string) {
  try {
    await operation;
  } catch (error) {
    assert(error instanceof Error && error.message.includes(marker), `拒绝应包含 ${marker}`);
    return;
  }
  throw new Error(`[obsidian-integration] 应拒绝 ${marker}`);
}

// 1) Contract names are frozen.  There is no raw command/path/binary entry.
assertDeep(
  OBSIDIAN_TAURI_COMMANDS,
  {
    status: "obsidian_integration_status",
    open_vault: "obsidian_integration_open_vault",
    open_note: "obsidian_integration_open_note",
    open_search: "obsidian_integration_open_search",
    read_note: "obsidian_integration_read_note",
    search_notes: "obsidian_integration_search_notes",
  },
  "Tauri command contract 漂移",
);

assertDeep(
  OBSIDIAN_INTEGRATION_ACTIONS,
  [
    "open_vault",
    "open_note",
    "open_search",
    "read_note",
    "search_notes",
  ],
  "allowed action union drifted",
);

// 2) Named methods invoke only their fixed contracts and transmit no root,
// binary, argv or generic command parameter.
const calls: Array<Readonly<{ command: ObsidianIntegrationInvokeName; args: Record<string, unknown> | undefined }>> = [];
const fakeInvoke: ObsidianIntegrationInvoke = async function <T>(
  command: ObsidianIntegrationInvokeName,
  args?: Record<string, unknown>,
): Promise<T> {
  calls.push({ command, args });
  const response: unknown =
    command === OBSIDIAN_TAURI_COMMANDS.status
      ? {
          status: "ready",
          message: "已连接 Syn 知识库",
          app_version: "1.12.7",
          cli_version: "1.12.7",
          vault_label: "Syn 知识库",
        }
      : command === OBSIDIAN_TAURI_COMMANDS.read_note
        ? { slug: "daily-note", title: "Daily note", body: "# Daily note", mtime_ms: 1 }
        : command === OBSIDIAN_TAURI_COMMANDS.search_notes
          ? [{ slug: "daily-note", title: "Daily note", snippet: "Daily", mtime_ms: 1 }]
          : { action: actionFor(command), message: "已提交", degraded: false };
  return response as T;
};
const client = createObsidianIntegrationClient(fakeInvoke);

assertDeep(
  Object.keys(client).sort(),
  [
    "openNote",
    "openSearch",
    "openVault",
    "readNote",
    "searchNotes",
    "status",
  ],
  "client must expose named typed actions only",
);

const status = await client.status();
assert(status.status === "ready" && status.vault_label === "Syn 知识库", "status payload shape should remain typed and path-free");
await client.openVault();
await client.openNote("daily-note");
await client.openSearch("release notes");
const note = await client.readNote("daily-note");
assert(note.body === "# Daily note", "read_note result shape should remain typed");
const hits = await client.searchNotes("release notes");
assert(hits[0]?.slug === "daily-note", "search_notes result shape should remain typed");

assertDeep(
  calls,
  [
    { command: "obsidian_integration_status", args: undefined },
    { command: "obsidian_integration_open_vault", args: undefined },
    { command: "obsidian_integration_open_note", args: { slug: "daily-note" } },
    { command: "obsidian_integration_open_search", args: { query: "release notes" } },
    { command: "obsidian_integration_read_note", args: { slug: "daily-note" } },
    { command: "obsidian_integration_search_notes", args: { query: "release notes" } },
  ],
  "every named client method should use one fixed Tauri invoke signature",
);

// 3) Runtime guards back up the TypeScript unions; user supplied paths and CLI
// flags do not reach the injected invoker.
const callCountBeforeRejections = calls.length;
await expectRejected(client.openNote("../other-vault"), "obsidian_integration_invalid_slug");
await expectRejected(client.openSearch("--help"), "obsidian_integration_invalid_query");
assert(calls.length === callCountBeforeRejections, "unsafe values must fail before Tauri invoke");

// 4) Six readiness states remain a path-free, optional compatibility signal.
const statuses: ObsidianIntegrationStatus[] = [
  "not_installed",
  "installed",
  "app_not_running",
  "cli_not_enabled",
  "ready",
  "incompatible",
];
assertDeep(
  statuses.map(obsidianStatusLabel),
  [
    "尚未安装官方 Obsidian",
    "已安装 Obsidian，正在检测连接",
    "Obsidian 未运行",
    "Obsidian CLI 未启用",
    "已连接 Obsidian",
    "Obsidian 版本不兼容",
  ],
  "six status labels drifted",
);
const ssrMarkup = renderToStaticMarkup(
  <span>{`${obsidianStatusLabel("ready")} · ${obsidianActionLabel("open_vault")}`}</span>,
);
assert(ssrMarkup.includes("已连接 Obsidian") && !ssrMarkup.includes("伴随"), "SSR helper output must not expose the stopped companion route");

// 5) Known error codes gain actionable copy.  Unknown backend text is not
// echoed, preventing an accidental path/argv leak to the UI.
assert(
  humanizeObsidianIntegrationError(new Error("obsidian_integration_cli_not_enabled: details")).includes("CLI"),
  "CLI error should be humanized",
);
assert(
  humanizeObsidianIntegrationError(new Error("/private/secret --argv")).includes("/private") === false,
  "unknown diagnostic text must not be rendered verbatim",
);

console.log("obsidian integration typed client test passed");

function actionFor(command: ObsidianIntegrationInvokeName) {
  switch (command) {
    case "obsidian_integration_open_vault":
      return "open_vault" as const;
    case "obsidian_integration_open_note":
      return "open_note" as const;
    case "obsidian_integration_open_search":
      return "open_search" as const;
    default:
      throw new Error(`unexpected receipt command ${command}`);
  }
}
