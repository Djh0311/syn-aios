// N0 native-workspace boundary: the optional external bridge must stay
// collapsed and must not be needed for Syn's Markdown workspace.
import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  ObsidianIntegrationPanel,
  subscribeToKnowledgeVaultSavedEvent,
} from "../src/views/KnowledgeBaseView";
import {
  knowledgeWorkspaceDraftNavigationDisposition,
  knowledgeWorkspaceDraftRefreshDisposition,
  knowledgeWorkspaceAsyncCommitDisposition,
} from "../src/lib/knowledgeWorkspace";
import { NativeKnowledgeWorkspace } from "../src/views/knowledge/NativeKnowledgeWorkspace";
import {
  KNOWLEDGE_WORKSPACE_TAURI_COMMANDS,
  createKnowledgeWorkspaceClient,
  type KnowledgeWorkspaceInvoke,
  type KnowledgeWorkspaceInvokeArgs,
  type KnowledgeWorkspaceInvokeName,
  type KnowledgeWorkspaceAttachment,
  type KnowledgeWorkspaceAttachmentImportResult,
  type KnowledgeWorkspaceGraphResponse,
  type KnowledgeWorkspaceCanvasDocument,
  type KnowledgeWorkspaceMarkdownDocument,
  type KnowledgeWorkspaceMutationResult,
  type KnowledgeWorkspaceRecoveryBackup,
  type KnowledgeWorkspaceRecoveryBackupSummary,
  type KnowledgeWorkspaceRecoveryRestoreResult,
  type KnowledgeWorkspaceSearchResponse,
  type KnowledgeWorkspaceSnapshot,
  type KnowledgeWorkspaceVaultManifest,
  type JsonCanvasObject,
} from "../src/lib/tauri";

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) throw new Error(`[native-knowledge-workspace] ${message}`);
}

const markup = renderToStaticMarkup(<ObsidianIntegrationPanel />);
const nativeWorkspaceMarkup = renderToStaticMarkup(<NativeKnowledgeWorkspace />);

assert(markup.includes('class="knowledge-compatibility-panel"'), "兼容入口必须是独立、收起的 details 面板");
assert(markup.includes("可选兼容与外部打开"), "兼容入口必须明确是可选外部打开");
assert(markup.includes("Syn 原生知识工作区无需安装 Obsidian"), "Syn 原生闭环不得以 Obsidian 安装为前置");
assert(markup.includes("Markdown / JSON Canvas 文件兼容"), "入口必须说明开放文件兼容边界");
assert(!markup.includes("正在检测官方 Obsidian 状态"), "收起的兼容入口不得成为初始状态中心");
assert(!markup.includes("伴随工作区") && !markup.includes("辅助功能权限"), "N0 不得暴露已停止的 companion/Accessibility 路线");
assert(nativeWorkspaceMarkup.includes("Markdown 与 JSON Canvas 均在 Syn 内原生接入"), "N4 完成后原生工作区头必须如实说明 Canvas 已接入");
assert(!nativeWorkspaceMarkup.includes("JSON Canvas 将在后续原生阶段接入"), "N4 完成后不得继续展示过时 Canvas 占位文案");

assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: false, externallyChanged: false }) === "replace"
  && knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: false, externallyChanged: true }) === "replace",
  "Markdown clean 草稿在 focus/manual/写后刷新时才可自动载入当前版本",
);
assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: true, externallyChanged: false }) === "preserve",
  "Markdown 脏草稿即使磁盘未变也必须在 focus/manual/写后刷新时保留",
);
assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: true, externallyChanged: true }) === "conflict",
  "Markdown 脏草稿与外部版本同时变化时必须进入冲突态而非静默覆盖",
);
assert(
  knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: true }) === "preserve"
  && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: false }) === "open",
  "Markdown 树、搜索、图谱、双链、新建和关闭当前标签只能在 clean 草稿时导航",
);
assert(
  knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 4,
    currentDraftRevision: 4,
    requestGeneration: 7,
    currentGeneration: 7,
    requestCurrentRelativePath: "research/plan.md",
    currentRelativePath: "research/plan.md",
  }) === "apply",
  "异步读取仅可在草稿 revision、编辑器 generation 和当前路径都未变时回填",
);
assert(
  knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 4,
    currentDraftRevision: 5,
    requestGeneration: 7,
    currentGeneration: 7,
    requestCurrentRelativePath: "research/plan.md",
    currentRelativePath: "research/plan.md",
  }) === "preserve"
  && knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 4,
    currentDraftRevision: 4,
    requestGeneration: 7,
    currentGeneration: 8,
    requestCurrentRelativePath: "research/plan.md",
    currentRelativePath: "research/plan.md",
  }) === "preserve"
  && knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 4,
    currentDraftRevision: 4,
    requestGeneration: 7,
    currentGeneration: 7,
    requestCurrentRelativePath: "research/plan.md",
    currentRelativePath: "research/next.md",
  }) === "preserve",
  "迟到的 Markdown 读取不得覆盖期间编辑、后续导航或新选中的条目",
);

const vaultSaveTarget = new EventTarget();
let nativeRefreshCount = 0;
const stopListeningForVaultSave = subscribeToKnowledgeVaultSavedEvent(vaultSaveTarget, () => {
  nativeRefreshCount += 1;
});
vaultSaveTarget.dispatchEvent(new Event("syn-knowledge-vault-saved"));
assert(nativeRefreshCount === 1, "legacy 或 AI 成功写入必须请求 Syn 原生工作区刷新");
stopListeningForVaultSave();
vaultSaveTarget.dispatchEvent(new Event("syn-knowledge-vault-saved"));
assert(nativeRefreshCount === 1, "卸载后不得留下全局 vault 写入监听器");

// N1/N2 red-first contract: the fixed workspace client and its later write
// action are introduced only after these assertions. The injected recorder
// cannot start Tauri, touch a vault, or introduce a raw command surface.
assertDeep(
  KNOWLEDGE_WORKSPACE_TAURI_COMMANDS,
  {
    snapshot: "knowledge_workspace_snapshot",
    vault_manifest: "knowledge_workspace_vault_manifest",
    search: "knowledge_workspace_search",
    graph: "knowledge_workspace_graph",
    read_markdown: "knowledge_workspace_read_markdown",
    read_canvas: "knowledge_workspace_read_canvas",
    create_directory: "knowledge_workspace_create_directory",
    create_markdown: "knowledge_workspace_create_markdown",
    write_markdown: "knowledge_workspace_write_markdown",
    create_canvas: "knowledge_workspace_create_canvas",
    write_canvas: "knowledge_workspace_write_canvas",
    import_attachment: "knowledge_workspace_import_attachment",
    read_attachment: "knowledge_workspace_read_attachment",
    create_recovery_backup: "knowledge_workspace_create_recovery_backup",
    list_recovery_backups: "knowledge_workspace_list_recovery_backups",
    restore_recovery_backup: "knowledge_workspace_restore_recovery_backup",
    move_entry: "knowledge_workspace_move_entry",
    rename_entry: "knowledge_workspace_rename_entry",
    delete_entry: "knowledge_workspace_delete_entry",
  },
  "N1 fixed Tauri command contract drifted",
);

const hashA = "a".repeat(64);
const workspaceSnapshot: KnowledgeWorkspaceSnapshot = {
  entries: [
    {
      relative_path: "research/plan.md",
      parent_path: "research",
      kind: "markdown",
      title: "Plan",
      tags: ["work"],
      aliases: ["Roadmap"],
      properties: { owner: "Syn" },
      mtime_ms: 123,
      size_bytes: 42,
      outlinks: ["research/next.md"],
      backlinks: [],
    },
  ],
  tags: [{ tag: "work", note_count: 1 }],
  diagnostics: [],
};
const workspaceSearch: KnowledgeWorkspaceSearchResponse = {
  query: "plan",
  results: [
    {
      relative_path: "research/plan.md",
      title: "Plan",
      snippet: "# Plan",
      tags: ["work"],
      mtime_ms: 123,
    },
  ],
  diagnostics: [],
};
const workspaceManifest: KnowledgeWorkspaceVaultManifest = {
  entries: [
    { relative_path: "research/plan.md", kind: "markdown", mtime_ms: 123, size_bytes: 42 },
    { relative_path: "research/board.canvas", kind: "canvas", mtime_ms: 123, size_bytes: 88 },
    { relative_path: "attachments/figure.png", kind: "attachment", mtime_ms: 125, size_bytes: 3 },
  ],
  diagnostics: [],
};
const workspaceGraph: KnowledgeWorkspaceGraphResponse = {
  scope: "local",
  focus_relative_path: "research/plan.md",
  query: "plan",
  tag: "work",
  nodes: [
    { id: "research/plan.md", relative_path: "research/plan.md", title: "Plan", tags: ["work"] },
    { id: "research/next.md", relative_path: "research/next.md", title: "Next", tags: [] },
  ],
  edges: [{ id: "research/plan.md->research/next.md", source: "research/plan.md", target: "research/next.md" }],
  diagnostics: [],
  truncated: false,
};
const canvasPayload: JsonCanvasObject = {
  nodes: [
    { id: "note", type: "text", text: "Plan", x: 0, y: 0, width: 240, height: 80 },
  ],
  edges: [],
  future_root_field: { preserved: true },
};
const workspaceCanvas: KnowledgeWorkspaceCanvasDocument = {
  relative_path: "research/board.canvas",
  document: canvasPayload,
  mtime_ms: 123,
  content_hash: hashA,
  diagnostics: [],
};
const workspaceDocument: KnowledgeWorkspaceMarkdownDocument = {
  relative_path: "research/plan.md",
  title: "Plan",
  body: "# Plan\n",
  tags: ["work"],
  aliases: ["Roadmap"],
  properties: { owner: "Syn" },
  outlinks: ["research/next.md"],
  backlinks: [],
  mtime_ms: 123,
  content_hash: hashA,
};
const mutationResult: KnowledgeWorkspaceMutationResult = {
  operation: "markdown_created",
  relative_path: "research/plan.md",
  source_relative_path: null,
  mtime_ms: 123,
  content_hash: hashA,
  audit_event_id: "audit-1",
};
const writeMutationResult: KnowledgeWorkspaceMutationResult = {
  operation: "markdown_updated",
  relative_path: "research/plan.md",
  source_relative_path: null,
  mtime_ms: 124,
  content_hash: hashA,
  audit_event_id: "audit-2",
};
const workspaceAttachment: KnowledgeWorkspaceAttachment = {
  relative_path: "attachments/figure.png",
  mime_type: "image/png",
  bytes: [137, 80, 78],
  mtime_ms: 125,
  content_hash: "c".repeat(64),
  size_bytes: 3,
};
const attachmentImportResult: KnowledgeWorkspaceAttachmentImportResult = {
  relative_path: "attachments/figure.png",
  mime_type: "image/png",
  mtime_ms: 125,
  content_hash: "c".repeat(64),
  size_bytes: 3,
  audit_event_id: "audit-attachment",
};
const recoveryBackup: KnowledgeWorkspaceRecoveryBackup = {
  backup_id: "d".repeat(32),
  relative_path: "research/plan.md",
  kind: "markdown",
  size_bytes: 42,
  content_hash: hashA,
  created_at_ms: 456,
  audit_event_id: "audit-backup",
};
const recoveryBackups: ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary> = [{
  backup_id: recoveryBackup.backup_id,
  relative_path: recoveryBackup.relative_path,
  kind: recoveryBackup.kind,
  size_bytes: recoveryBackup.size_bytes,
  content_hash: recoveryBackup.content_hash,
  created_at_ms: recoveryBackup.created_at_ms,
}];
const recoveryRestoreResult: KnowledgeWorkspaceRecoveryRestoreResult = {
  backup_id: recoveryBackup.backup_id,
  relative_path: recoveryBackup.relative_path,
  mtime_ms: 457,
  content_hash: hashA,
  audit_event_id: "audit-restore",
};
const workspaceCalls: Array<Readonly<{
  command: KnowledgeWorkspaceInvokeName;
  args: KnowledgeWorkspaceInvokeArgs | undefined;
}>> = [];
const fakeWorkspaceInvoke: KnowledgeWorkspaceInvoke = async function <T>(
  command: KnowledgeWorkspaceInvokeName,
  args?: KnowledgeWorkspaceInvokeArgs,
): Promise<T> {
  workspaceCalls.push({ command, args });
  const response = workspaceResponseFor(command);
  return response as T;
};
const workspaceClient = createKnowledgeWorkspaceClient(fakeWorkspaceInvoke);

assertDeep(
  Object.keys(workspaceClient).sort(),
  [
    "createCanvas",
    "createDirectory",
    "createMarkdown",
    "createRecoveryBackup",
    "deleteEntry",
    "graph",
    "importAttachment",
    "listRecoveryBackups",
    "moveEntry",
    "readAttachment",
    "readCanvas",
    "readMarkdown",
    "renameEntry",
    "restoreRecoveryBackup",
    "search",
    "snapshot",
    "vaultManifest",
    "writeCanvas",
    "writeMarkdown",
  ],
  "workspace client must expose named actions only",
);
assert(("invoke" in workspaceClient) === false && ("root" in workspaceClient) === false, "client must not expose raw invoke or vault root");

const snapshot = await workspaceClient.snapshot();
const manifest = await workspaceClient.vaultManifest();
const search = await workspaceClient.search("plan");
const graph = await workspaceClient.graph({
  scope: "local",
  focusRelativePath: "research/plan.md",
  query: "plan",
  tag: "work",
});
const document = await workspaceClient.readMarkdown("research/plan.md");
const canvas = await workspaceClient.readCanvas("research/board.canvas");
const importedAttachment = await workspaceClient.importAttachment(
  new Uint8Array([137, 80, 78]),
  "figure.png",
  "image/png",
);
const attachment = await workspaceClient.readAttachment("attachments/figure.png");
const backup = await workspaceClient.createRecoveryBackup("research/plan.md", 123, hashA);
const listedBackups = await workspaceClient.listRecoveryBackups();
const restored = await workspaceClient.restoreRecoveryBackup("d".repeat(32), 123, hashA);
await workspaceClient.createDirectory("research");
await workspaceClient.createMarkdown("research/plan.md", "# Plan\n");
const written = await workspaceClient.writeMarkdown("research/plan.md", "# Plan\nUpdated\n", 123, hashA);
const canvasCreated = await workspaceClient.createCanvas("research/board.canvas", canvasPayload);
const canvasWritten = await workspaceClient.writeCanvas("research/board.canvas", canvasPayload, 123, hashA);
await workspaceClient.moveEntry("research/plan.md", "archive/plan.md", 123, hashA);
await workspaceClient.renameEntry("archive/plan.md", "archive/final-plan.md", 123, hashA);
await workspaceClient.deleteEntry("archive/final-plan.md", 123, hashA);

assert(snapshot.entries[0]?.relative_path === "research/plan.md", "snapshot response must keep serde snake_case fields");
assert(manifest.entries[2]?.relative_path === "attachments/figure.png", "manifest must only expose fixed-vault relative entries");
assert(search.results[0]?.snippet === "# Plan", "search response must keep serde snake_case fields");
assert(graph.nodes[0]?.relative_path === "research/plan.md", "graph node selection must stay on validated relative paths");
assert(document.content_hash === hashA, "read response must expose CAS content hash");
assert(canvas.document.future_root_field === canvasPayload.future_root_field, "canvas response must preserve unknown JSON fields");
assert(written.operation === "markdown_updated", "write response must expose the markdown_updated mutation");
assert(canvasCreated.operation === "canvas_created" && canvasWritten.operation === "canvas_updated", "canvas mutations must remain typed and explicit");
assert(importedAttachment.relative_path === "attachments/figure.png" && attachment.bytes.length === 3, "attachment import/read must remain typed and byte-bounded");
assert(backup.backup_id.length === 32 && listedBackups.length === 1 && restored.relative_path === "research/plan.md", "recovery must keep single-entry backup IDs and CAS results typed");
assertDeep(
  workspaceCalls,
  [
    { command: "knowledge_workspace_snapshot", args: undefined },
    { command: "knowledge_workspace_vault_manifest", args: undefined },
    { command: "knowledge_workspace_search", args: { query: "plan" } },
    {
      command: "knowledge_workspace_graph",
      args: { scope: "local", focusRelativePath: "research/plan.md", query: "plan", tag: "work" },
    },
    { command: "knowledge_workspace_read_markdown", args: { relativePath: "research/plan.md" } },
    { command: "knowledge_workspace_read_canvas", args: { relativePath: "research/board.canvas" } },
    {
      command: "knowledge_workspace_import_attachment",
      args: { bytes: [137, 80, 78], displayName: "figure.png", mimeType: "image/png" },
    },
    { command: "knowledge_workspace_read_attachment", args: { relativePath: "attachments/figure.png" } },
    {
      command: "knowledge_workspace_create_recovery_backup",
      args: { relativePath: "research/plan.md", expectedMtimeMs: 123, expectedContentHash: hashA },
    },
    { command: "knowledge_workspace_list_recovery_backups", args: undefined },
    {
      command: "knowledge_workspace_restore_recovery_backup",
      args: { backupId: "d".repeat(32), expectedMtimeMs: 123, expectedContentHash: hashA },
    },
    { command: "knowledge_workspace_create_directory", args: { relativePath: "research" } },
    { command: "knowledge_workspace_create_markdown", args: { relativePath: "research/plan.md", body: "# Plan\n" } },
    {
      command: "knowledge_workspace_write_markdown",
      args: {
        relativePath: "research/plan.md",
        body: "# Plan\nUpdated\n",
        expectedMtimeMs: 123,
        expectedContentHash: hashA,
      },
    },
    {
      command: "knowledge_workspace_create_canvas",
      args: { relativePath: "research/board.canvas", document: canvasPayload },
    },
    {
      command: "knowledge_workspace_write_canvas",
      args: {
        relativePath: "research/board.canvas",
        document: canvasPayload,
        expectedMtimeMs: 123,
        expectedContentHash: hashA,
      },
    },
    {
      command: "knowledge_workspace_move_entry",
      args: { from: "research/plan.md", to: "archive/plan.md", expectedMtimeMs: 123, expectedContentHash: hashA },
    },
    {
      command: "knowledge_workspace_rename_entry",
      args: { from: "archive/plan.md", to: "archive/final-plan.md", expectedMtimeMs: 123, expectedContentHash: hashA },
    },
    {
      command: "knowledge_workspace_delete_entry",
      args: { relativePath: "archive/final-plan.md", expectedMtimeMs: 123, expectedContentHash: hashA },
    },
  ],
  "every workspace method must use one fixed Tauri command and lower-camel payload",
);

const callCountBeforeRejections = workspaceCalls.length;
await expectWorkspaceRejected(workspaceClient.readMarkdown("../other-vault.md"), "knowledge_workspace_invalid_path");
await expectWorkspaceRejected(workspaceClient.createDirectory("/outside"), "knowledge_workspace_invalid_path");
await expectWorkspaceRejected(workspaceClient.createMarkdown("research/plan.txt", "plain"), "knowledge_workspace_markdown_only");
await expectWorkspaceRejected(workspaceClient.writeMarkdown("../other-vault.md", "plain", 123, hashA), "knowledge_workspace_invalid_path");
await expectWorkspaceRejected(
  workspaceClient.writeMarkdown("research/plan.md", "x".repeat(64 * 1024 + 1), 123, hashA),
  "knowledge_workspace_markdown_too_large",
);
await expectWorkspaceRejected(workspaceClient.search("\u0000"), "knowledge_workspace_invalid_search_query");
await expectWorkspaceRejected(
  workspaceClient.graph({ scope: "global", focusRelativePath: "research/plan.md" }),
  "knowledge_workspace_invalid_graph_focus",
);
await expectWorkspaceRejected(
  workspaceClient.graph({ scope: "local", focusRelativePath: "../other-vault.md" }),
  "knowledge_workspace_invalid_path",
);
await expectWorkspaceRejected(workspaceClient.graph({ scope: "global", tag: "\u0000" }), "knowledge_workspace_invalid_graph_tag");
await expectWorkspaceRejected(workspaceClient.readCanvas("../other-vault.canvas"), "knowledge_workspace_invalid_path");
await expectWorkspaceRejected(
  workspaceClient.importAttachment(new Uint8Array([1]), "../outside.png", "image/png"),
  "knowledge_workspace_attachment_invalid_display_name",
);
await expectWorkspaceRejected(
  workspaceClient.importAttachment(new Uint8Array([1]), "figure.png", "image/jpeg"),
  "knowledge_workspace_attachment_invalid_mime_type",
);
await expectWorkspaceRejected(
  workspaceClient.importAttachment(new Uint8Array(10 * 1024 * 1024 + 1), "figure.png", "image/png"),
  "knowledge_workspace_attachment_too_large",
);
await expectWorkspaceRejected(workspaceClient.readAttachment("research/figure.png"), "knowledge_workspace_attachment_only");
await expectWorkspaceRejected(
  workspaceClient.createRecoveryBackup("research/plan.txt", 123, hashA),
  "knowledge_workspace_recovery_unsupported_entry",
);
await expectWorkspaceRejected(
  workspaceClient.restoreRecoveryBackup("not-a-backup-id", 123, hashA),
  "knowledge_workspace_backup_invalid_id",
);
await expectWorkspaceRejected(
  workspaceClient.createCanvas("research/board.md", canvasPayload),
  "knowledge_workspace_canvas_only",
);
await expectWorkspaceRejected(workspaceClient.deleteEntry("research/plan.md", Number.NaN, hashA), "knowledge_workspace_invalid_mtime");
await expectWorkspaceRejected(workspaceClient.deleteEntry("research/plan.md", 123, "not-a-sha256"), "knowledge_workspace_invalid_content_hash");
await expectWorkspaceRejected(workspaceClient.writeMarkdown("research/plan.md", "plain", Number.NaN, hashA), "knowledge_workspace_invalid_mtime");
await expectWorkspaceRejected(workspaceClient.writeMarkdown("research/plan.md", "plain", 123, "not-a-sha256"), "knowledge_workspace_invalid_content_hash");
assert(workspaceCalls.length === callCountBeforeRejections, "unsafe values must fail before the fixed Tauri invoker");

console.log("native knowledge workspace N0 optional compatibility and N1/N2/N3/N4/N5 fixed client contract tests passed");

function assertDeep(actual: object, expected: object, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

type WorkspaceResponse =
  | KnowledgeWorkspaceSnapshot
  | KnowledgeWorkspaceSearchResponse
  | KnowledgeWorkspaceGraphResponse
  | KnowledgeWorkspaceCanvasDocument
  | KnowledgeWorkspaceMarkdownDocument
  | KnowledgeWorkspaceMutationResult
  | KnowledgeWorkspaceVaultManifest
  | KnowledgeWorkspaceAttachment
  | KnowledgeWorkspaceAttachmentImportResult
  | KnowledgeWorkspaceRecoveryBackup
  | ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary>
  | KnowledgeWorkspaceRecoveryRestoreResult;

async function expectWorkspaceRejected(operation: Promise<WorkspaceResponse>, marker: string) {
  try {
    await operation;
  } catch (error) {
    assert(error instanceof Error && error.message.includes(marker), `拒绝应包含 ${marker}`);
    return;
  }
  throw new Error(`[native-knowledge-workspace] 应拒绝 ${marker}`);
}

function workspaceResponseFor(
  command: KnowledgeWorkspaceInvokeName,
):
  | KnowledgeWorkspaceSnapshot
  | KnowledgeWorkspaceSearchResponse
  | KnowledgeWorkspaceGraphResponse
  | KnowledgeWorkspaceCanvasDocument
  | KnowledgeWorkspaceMarkdownDocument
  | KnowledgeWorkspaceMutationResult
  | KnowledgeWorkspaceVaultManifest
  | KnowledgeWorkspaceAttachment
  | KnowledgeWorkspaceAttachmentImportResult
  | KnowledgeWorkspaceRecoveryBackup
  | ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary>
  | KnowledgeWorkspaceRecoveryRestoreResult {
  switch (command) {
    case "knowledge_workspace_snapshot":
      return workspaceSnapshot;
    case "knowledge_workspace_vault_manifest":
      return workspaceManifest;
    case "knowledge_workspace_search":
      return workspaceSearch;
    case "knowledge_workspace_graph":
      return workspaceGraph;
    case "knowledge_workspace_read_canvas":
      return workspaceCanvas;
    case "knowledge_workspace_read_markdown":
      return workspaceDocument;
    case "knowledge_workspace_import_attachment":
      return attachmentImportResult;
    case "knowledge_workspace_read_attachment":
      return workspaceAttachment;
    case "knowledge_workspace_create_recovery_backup":
      return recoveryBackup;
    case "knowledge_workspace_list_recovery_backups":
      return recoveryBackups;
    case "knowledge_workspace_restore_recovery_backup":
      return recoveryRestoreResult;
    case "knowledge_workspace_create_directory":
      return { ...mutationResult, operation: "directory_created", relative_path: "research", mtime_ms: null, content_hash: null };
    case "knowledge_workspace_create_markdown":
      return mutationResult;
    case "knowledge_workspace_write_markdown":
      return writeMutationResult;
    case "knowledge_workspace_create_canvas":
      return { ...mutationResult, operation: "canvas_created", relative_path: "research/board.canvas" };
    case "knowledge_workspace_write_canvas":
      return { ...mutationResult, operation: "canvas_updated", relative_path: "research/board.canvas" };
    case "knowledge_workspace_move_entry":
      return { ...mutationResult, operation: "markdown_moved", relative_path: "archive/plan.md", source_relative_path: "research/plan.md" };
    case "knowledge_workspace_rename_entry":
      return { ...mutationResult, operation: "markdown_renamed", relative_path: "archive/final-plan.md", source_relative_path: "archive/plan.md" };
    case "knowledge_workspace_delete_entry":
      return { ...mutationResult, operation: "markdown_deleted", relative_path: "archive/final-plan.md", mtime_ms: null, content_hash: null };
  }
  throw new Error(`Unexpected fixed workspace command: ${command}`);
}
