import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  KnowledgeWorkspaceMaintenancePanel,
  loadKnowledgeWorkspaceRecoveryRevision,
} from "../src/views/knowledge/KnowledgeWorkspaceMaintenancePanel";
import { workspaceAttachmentReferenceStatus } from "../src/views/knowledge/NativeKnowledgeWorkspace";
import { parseMarkdown } from "../src/lib/knowledgeVault";
import {
  DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES,
  isKnowledgeWorkspaceAttachmentRelativePath,
  loadKnowledgeWorkspaceUiPreferences,
  saveKnowledgeWorkspaceUiPreferences,
  workspaceAttachmentCanvasReference,
  workspaceAttachmentMarkdownReference,
} from "../src/lib/knowledgeWorkspace";
import type { KnowledgeWorkspaceClient } from "../src/lib/tauri";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[knowledge-attachment-recovery] ${message}`);
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

class MemoryStorage {
  private readonly values = new Map<string, string>();

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

const staticMarkup = renderToStaticMarkup(<KnowledgeWorkspaceMaintenancePanel />);
assert(staticMarkup.includes("受限附件与恢复"), "N5 静态入口必须说明附件与恢复仍在 Syn 内");
assert(staticMarkup.includes('type="file"'), "附件选择只能使用浏览器 File input");
assert(staticMarkup.includes("固定 Syn vault"), "静态入口必须说明固定 vault 边界");
assert(staticMarkup.includes("单条恢复"), "恢复入口必须明确只恢复单个条目");
assert(!staticMarkup.includes("文件系统路径") && !staticMarkup.includes("Shell"), "附件入口不得暴露任意文件系统或 shell");

const emptyStorage = new MemoryStorage();
assertDeep(
  loadKnowledgeWorkspaceUiPreferences(emptyStorage),
  DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES,
  "缺少偏好必须回到安全默认布局",
);

emptyStorage.setItem("syn-native-knowledge-workspace-ui-v1", "{not-json");
assertDeep(
  loadKnowledgeWorkspaceUiPreferences(emptyStorage),
  DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES,
  "损坏偏好不得阻止安全默认布局",
);

saveKnowledgeWorkspaceUiPreferences(emptyStorage, {
  version: 1,
  tabs: ["research/plan.md", "research/plan.md", "../outside.md", "canvas/board.canvas"],
  selectedRelativePath: "research/plan.md",
  viewMode: "split",
});
assertDeep(
  loadKnowledgeWorkspaceUiPreferences(emptyStorage),
  {
    version: 1,
    tabs: ["research/plan.md"],
    selectedRelativePath: "research/plan.md",
    viewMode: "split",
  },
  "偏好只能保留受限 Markdown 标签页与选中项",
);
const persistedPreference = emptyStorage.getItem("syn-native-knowledge-workspace-ui-v1") ?? "";
assert(!persistedPreference.includes("body") && !persistedPreference.includes("content_hash") && !persistedPreference.includes("bytes"), "localStorage 不得保存知识内容、revision 或附件字节");

assert(isKnowledgeWorkspaceAttachmentRelativePath("attachments/figure.png"), "固定附件相对路径必须可用于引用");
assert(!isKnowledgeWorkspaceAttachmentRelativePath("/tmp/figure.png"), "绝对路径不得进入附件引用");
assert(!isKnowledgeWorkspaceAttachmentRelativePath("attachments/../figure.png"), "父级穿越不得进入附件引用");
assert(!isKnowledgeWorkspaceAttachmentRelativePath("research/figure.png"), "非 attachments 目录不得伪装成附件引用");
assert(!isKnowledgeWorkspaceAttachmentRelativePath("attachments/figure.exe"), "不允许的附件扩展名不得进入引用");
assert(
  workspaceAttachmentMarkdownReference("attachments/figure.png", "现场图") === "![现场图](attachments/figure.png)",
  "Markdown 必须只生成 vault 内相对附件引用",
);
assert(
  workspaceAttachmentCanvasReference("attachments/figure.png") === "attachments/figure.png",
  "Canvas 必须只使用 vault 内相对附件引用",
);

const recoveryReadCalls: string[] = [];
const recoveryClient = {
  readMarkdown: async (relativePath: string) => {
    recoveryReadCalls.push(`markdown:${relativePath}`);
    return { relative_path: relativePath, mtime_ms: 11, content_hash: "a".repeat(64) };
  },
  readCanvas: async (relativePath: string) => {
    recoveryReadCalls.push(`canvas:${relativePath}`);
    return { relative_path: relativePath, mtime_ms: 12, content_hash: "b".repeat(64) };
  },
  readAttachment: async (relativePath: string) => {
    recoveryReadCalls.push(`attachment:${relativePath}`);
    return { relative_path: relativePath, mtime_ms: 13, content_hash: "c".repeat(64) };
  },
} as Pick<KnowledgeWorkspaceClient, "readMarkdown" | "readCanvas" | "readAttachment">;

assertDeep(
  await loadKnowledgeWorkspaceRecoveryRevision(recoveryClient, { relative_path: "research/plan.md", kind: "markdown" }),
  { relativePath: "research/plan.md", mtimeMs: 11, contentHash: "a".repeat(64) },
  "备份只能从当前受限 Markdown revision 取得 CAS",
);
assertDeep(
  await loadKnowledgeWorkspaceRecoveryRevision(recoveryClient, { relative_path: "research/board.canvas", kind: "canvas" }),
  { relativePath: "research/board.canvas", mtimeMs: 12, contentHash: "b".repeat(64) },
  "备份只能从当前受限 Canvas revision 取得 CAS",
);
assertDeep(
  await loadKnowledgeWorkspaceRecoveryRevision(recoveryClient, { relative_path: "attachments/figure.png", kind: "attachment" }),
  { relativePath: "attachments/figure.png", mtimeMs: 13, contentHash: "c".repeat(64) },
  "备份只能从当前受限附件 revision 取得 CAS",
);
assertDeep(
  recoveryReadCalls,
  ["markdown:research/plan.md", "canvas:research/board.canvas", "attachment:attachments/figure.png"],
  "恢复前必须重新读取当前条目的 revision，不能从 manifest 或 UI 偏好猜测 CAS",
);

const attachmentMarkdown = "![现场图](attachments/figure.png)";
assert(
  !JSON.stringify(parseMarkdown(attachmentMarkdown)).includes('"kind":"link"'),
  "固定 vault 附件相对引用不得被 Markdown 预览识别为外部链接",
);
assertDeep(
  workspaceAttachmentReferenceStatus(attachmentMarkdown, [
    { relative_path: "attachments/figure.png", kind: "attachment" },
  ]),
  { references: ["attachments/figure.png"], missing: [] },
  "存在的固定附件只能保留为本地相对引用状态",
);
assertDeep(
  workspaceAttachmentReferenceStatus(attachmentMarkdown, []),
  { references: ["attachments/figure.png"], missing: ["attachments/figure.png"] },
  "缺失附件必须显示为可恢复状态，不得静默改写笔记",
);
assertDeep(
  workspaceAttachmentReferenceStatus("![外部](file:///tmp/figure.png)", []),
  { references: [], missing: [] },
  "外部 file URI 不得成为原生附件引用",
);

console.log("knowledge attachment/recovery static boundary and UI-preference tests passed");
