// N5 的附件与恢复入口只描述固定 vault 内的用户动作。二进制、manifest、
// backup id 和 CAS 都留给 typed host contract；SSR 绝不读取 vault。

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  knowledgeWorkspace,
  type KnowledgeWorkspaceAttachment,
  type KnowledgeWorkspaceAttachmentMimeType,
  type KnowledgeWorkspaceClient,
  type KnowledgeWorkspaceRecoveryBackupSummary,
  type KnowledgeWorkspaceVaultManifest,
  type KnowledgeWorkspaceVaultManifestEntry,
} from "../../lib/tauri";
import {
  workspaceAttachmentCanvasReference,
  workspaceAttachmentMarkdownReference,
} from "../../lib/knowledgeWorkspace";

const MAX_ATTACHMENT_IMPORT_BYTES = 10 * 1024 * 1024;
const TEXT_ATTACHMENT_PREVIEW_CHARS = 8 * 1024;

type MaintenanceClient = Pick<
  KnowledgeWorkspaceClient,
  | "vaultManifest"
  | "importAttachment"
  | "readAttachment"
  | "readMarkdown"
  | "readCanvas"
  | "createRecoveryBackup"
  | "listRecoveryBackups"
  | "restoreRecoveryBackup"
>;

type MaintenanceBusyAction = "refresh" | "import" | "backup" | "restore" | null;
export type KnowledgeWorkspaceRecoveryTarget = Readonly<{
  relative_path: string;
  kind: "markdown" | "canvas" | "attachment";
}>;
export type KnowledgeWorkspaceRecoveryEntry = KnowledgeWorkspaceVaultManifestEntry & KnowledgeWorkspaceRecoveryTarget;
export type KnowledgeWorkspaceRecoveryRevision = Readonly<{
  relativePath: string;
  mtimeMs: number;
  contentHash: string;
}>;

// A manifest intentionally does not contain a content hash. Every backup and
// restore therefore re-reads its one selected fixed-vault file immediately
// before the CAS command; UI state and manifest rows cannot become a write key.
export async function loadKnowledgeWorkspaceRecoveryRevision(
  client: Pick<KnowledgeWorkspaceClient, "readMarkdown" | "readCanvas" | "readAttachment">,
  entry: KnowledgeWorkspaceRecoveryTarget,
): Promise<KnowledgeWorkspaceRecoveryRevision> {
  switch (entry.kind) {
    case "markdown": {
      const document = await client.readMarkdown(entry.relative_path);
      return {
        relativePath: document.relative_path,
        mtimeMs: document.mtime_ms,
        contentHash: document.content_hash,
      };
    }
    case "canvas": {
      const document = await client.readCanvas(entry.relative_path);
      return {
        relativePath: document.relative_path,
        mtimeMs: document.mtime_ms,
        contentHash: document.content_hash,
      };
    }
    case "attachment": {
      const attachment = await client.readAttachment(entry.relative_path);
      return {
        relativePath: attachment.relative_path,
        mtimeMs: attachment.mtime_ms,
        contentHash: attachment.content_hash,
      };
    }
  }
}

export function KnowledgeWorkspaceMaintenancePanel({
  client,
  refreshRequestId = 0,
  onWorkspaceMutation,
  onInsertMarkdownReference,
}: {
  client?: MaintenanceClient;
  refreshRequestId?: number;
  onWorkspaceMutation?: () => void;
  onInsertMarkdownReference?: (reference: string) => void;
}) {
  if (typeof window === "undefined") return <KnowledgeWorkspaceMaintenanceStaticShell />;
  return (
    <KnowledgeWorkspaceMaintenancePanelBrowser
      client={client ?? knowledgeWorkspace}
      refreshRequestId={refreshRequestId}
      onWorkspaceMutation={onWorkspaceMutation}
      onInsertMarkdownReference={onInsertMarkdownReference}
    />
  );
}

function KnowledgeWorkspaceMaintenanceStaticShell() {
  return (
    <section className="native-knowledge-maintenance" aria-label="Syn 知识工作区附件与恢复">
      <header className="native-maintenance-head">
        <div>
          <p className="eyebrow">Syn 原生维护</p>
          <h2>受限附件与恢复</h2>
        </div>
        <p>只操作固定 Syn vault；附件、备份与恢复不会读取任意外部路径。</p>
      </header>
      <div className="native-maintenance-grid">
        <section className="native-maintenance-section" aria-label="受限附件导入">
          <p className="eyebrow">附件</p>
          <strong>从浏览器选择一个文件</strong>
          <p>文件只会作为字节、显示名和 MIME 类型交给固定 host command；不会传递来源路径。</p>
          <label className="native-maintenance-file-field">
            <span>选择允许的附件</span>
            <input
              aria-label="选择允许的知识附件"
              type="file"
              accept=".png,.jpg,.jpeg,.gif,.webp,.pdf,.txt,.csv,image/png,image/jpeg,image/gif,image/webp,application/pdf,text/plain,text/csv"
            />
          </label>
          <button className="secondary-button" type="button" disabled>导入附件</button>
        </section>
        <section className="native-maintenance-section" aria-label="相对引用">
          <p className="eyebrow">相对引用</p>
          <strong>Markdown 与 Canvas 共用 vault 内路径</strong>
          <p>导入后可使用 <code>attachments/…</code> 相对引用；缺失附件会保持为可恢复错误，不会改写笔记或 Canvas。</p>
        </section>
        <section className="native-maintenance-section" aria-label="单条恢复">
          <p className="eyebrow">恢复</p>
          <strong>只创建备份并恢复单条</strong>
          <p>恢复会使用当前 revision 的冲突检查；不会整库覆盖、静默删除或清理知识文件。</p>
          <button className="secondary-button" type="button" disabled>创建可恢复备份</button>
          <button className="text-button" type="button" disabled>单条恢复</button>
        </section>
      </div>
    </section>
  );
}

function KnowledgeWorkspaceMaintenancePanelBrowser({
  client,
  refreshRequestId,
  onWorkspaceMutation,
  onInsertMarkdownReference,
}: {
  client: MaintenanceClient;
  refreshRequestId: number;
  onWorkspaceMutation?: () => void;
  onInsertMarkdownReference?: (reference: string) => void;
}) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const refreshSequence = useRef(0);
  const lastRefreshRequestId = useRef(refreshRequestId);
  const [manifest, setManifest] = useState<KnowledgeWorkspaceVaultManifest | null>(null);
  const [backups, setBackups] = useState<ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary>>([]);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [selectedAttachmentPath, setSelectedAttachmentPath] = useState<string | null>(null);
  const [attachment, setAttachment] = useState<KnowledgeWorkspaceAttachment | null>(null);
  const [attachmentLoadState, setAttachmentLoadState] = useState<"idle" | "loading" | "ready" | "unavailable">("idle");
  const [attachmentPreviewUrl, setAttachmentPreviewUrl] = useState<string | null>(null);
  const [selectedRecoveryPath, setSelectedRecoveryPath] = useState<string | null>(null);
  const [selectedBackupId, setSelectedBackupId] = useState<string | null>(null);
  const [busy, setBusy] = useState<MaintenanceBusyAction>("refresh");
  const [notice, setNotice] = useState<string | null>(null);

  const attachmentEntries = useMemo(
    () => (manifest?.entries ?? []).filter((entry) => entry.kind === "attachment"),
    [manifest],
  );
  const recoveryEntries = useMemo(
    () => (manifest?.entries ?? []).filter(isRecoveryManifestEntry),
    [manifest],
  );
  const selectedRecoveryEntry = recoveryEntries.find((entry) => entry.relative_path === selectedRecoveryPath) ?? null;
  const selectedBackup = backups.find((backup) => backup.backup_id === selectedBackupId) ?? null;

  const refreshMaintenance = useCallback(async (successNotice?: string) => {
    const request = refreshSequence.current + 1;
    refreshSequence.current = request;
    setBusy("refresh");
    try {
      const [nextManifest, nextBackups] = await Promise.all([
        client.vaultManifest(),
        client.listRecoveryBackups(),
      ]);
      if (request !== refreshSequence.current) return;
      setManifest(nextManifest);
      setBackups(nextBackups);
      if (successNotice) setNotice(successNotice);
    } catch {
      if (request !== refreshSequence.current) return;
      setNotice("固定 Syn vault 的附件或恢复目录暂时读不到；当前选择和本地草稿没有被改写。");
    } finally {
      if (request === refreshSequence.current) setBusy(null);
    }
  }, [client]);

  useEffect(() => {
    void refreshMaintenance();
  }, [refreshMaintenance]);

  useEffect(() => {
    if (refreshRequestId === 0 || refreshRequestId === lastRefreshRequestId.current) return;
    lastRefreshRequestId.current = refreshRequestId;
    void refreshMaintenance();
  }, [refreshMaintenance, refreshRequestId]);

  useEffect(() => {
    setSelectedAttachmentPath((current) => (
      current && attachmentEntries.some((entry) => entry.relative_path === current)
        ? current
        : attachmentEntries[0]?.relative_path ?? null
    ));
  }, [attachmentEntries]);

  useEffect(() => {
    setSelectedRecoveryPath((current) => (
      current && recoveryEntries.some((entry) => entry.relative_path === current)
        ? current
        : recoveryEntries[0]?.relative_path ?? null
    ));
  }, [recoveryEntries]);

  useEffect(() => {
    setSelectedBackupId((current) => (
      current && backups.some((backup) => backup.backup_id === current)
        ? current
        : backups[0]?.backup_id ?? null
    ));
  }, [backups]);

  useEffect(() => {
    let cancelled = false;
    if (!selectedAttachmentPath) {
      setAttachment(null);
      setAttachmentLoadState("idle");
      return undefined;
    }
    setAttachment(null);
    setAttachmentLoadState("loading");
    void client.readAttachment(selectedAttachmentPath).then(
      (nextAttachment) => {
        if (cancelled) return;
        setAttachment(nextAttachment);
        setAttachmentLoadState("ready");
      },
      () => {
        if (cancelled) return;
        setAttachmentLoadState("unavailable");
        setNotice("这项附件已不存在、不可读取或不再符合固定 vault 合同；笔记与 Canvas 引用没有被改写。");
      },
    );
    return () => {
      cancelled = true;
    };
  }, [client, selectedAttachmentPath]);

  useEffect(() => {
    setAttachmentPreviewUrl(null);
    if (!attachment || !isImageAttachment(attachment.mime_type)) return undefined;
    const previewUrl = URL.createObjectURL(new Blob([new Uint8Array(attachment.bytes)], { type: attachment.mime_type }));
    setAttachmentPreviewUrl(previewUrl);
    return () => URL.revokeObjectURL(previewUrl);
  }, [attachment]);

  const selectedAttachmentMarkdownReference = attachment
    ? workspaceAttachmentMarkdownReference(attachment.relative_path, displayNameFromRelativePath(attachment.relative_path))
    : null;
  const selectedAttachmentCanvasReference = attachment
    ? workspaceAttachmentCanvasReference(attachment.relative_path)
    : null;

  const importAttachment = useCallback(async () => {
    if (!selectedFile) {
      setNotice("先在浏览器文件控件中选择一项允许的附件；不会读取任何来源路径。");
      return;
    }
    const mimeType = attachmentMimeTypeForDisplayName(selectedFile.name);
    if (!mimeType) {
      setNotice("附件扩展名不在允许范围；没有读取或写入任何文件。");
      return;
    }
    if (selectedFile.size > MAX_ATTACHMENT_IMPORT_BYTES) {
      setNotice("附件超过 10 MiB 受限上限；没有读取或写入任何文件。");
      return;
    }
    setBusy("import");
    try {
      // The browser File picker is the only source. Its local path never
      // enters state or the host command; only bounded bytes/name/fixed MIME do.
      const bytes = new Uint8Array(await selectedFile.arrayBuffer());
      const imported = await client.importAttachment(bytes, selectedFile.name, mimeType);
      setSelectedFile(null);
      if (inputRef.current) inputRef.current.value = "";
      setSelectedAttachmentPath(imported.relative_path);
      await refreshMaintenance("附件已导入固定 Syn vault；可把相对引用加入当前 Markdown 草稿，或在 Canvas 文件节点中选择它。");
      onWorkspaceMutation?.();
    } catch {
      setNotice("附件导入没有完成。类型、大小、文件名和固定 vault 边界没有被放宽。");
    } finally {
      setBusy(null);
    }
  }, [client, onWorkspaceMutation, refreshMaintenance, selectedFile]);

  const insertMarkdownReference = useCallback(() => {
    if (!selectedAttachmentMarkdownReference) return;
    onInsertMarkdownReference?.(selectedAttachmentMarkdownReference);
    setNotice("已请求把受限附件相对引用加入当前 Markdown 草稿；点击笔记保存才会写入固定 vault。");
  }, [onInsertMarkdownReference, selectedAttachmentMarkdownReference]);

  const createRecoveryBackup = useCallback(async () => {
    if (!selectedRecoveryEntry) {
      setNotice("先从当前 vault manifest 选择一条 Markdown、Canvas 或附件；不会猜测恢复目标。");
      return;
    }
    setBusy("backup");
    try {
      const revision = await loadKnowledgeWorkspaceRecoveryRevision(client, selectedRecoveryEntry);
      await client.createRecoveryBackup(revision.relativePath, revision.mtimeMs, revision.contentHash);
      await refreshMaintenance("已为这一条固定 vault 文件创建可恢复备份；没有执行整库复制或删除。");
      onWorkspaceMutation?.();
    } catch (error) {
      setNotice(recoveryFailureNotice(error, "创建备份"));
    } finally {
      setBusy(null);
    }
  }, [client, onWorkspaceMutation, refreshMaintenance, selectedRecoveryEntry]);

  const restoreRecoveryBackup = useCallback(async () => {
    if (!selectedBackup) {
      setNotice("先选择一条已列出的恢复备份；不会接受手工输入的备份 ID。");
      return;
    }
    const currentEntry = recoveryEntries.find(
      (entry) => entry.relative_path === selectedBackup.relative_path && entry.kind === selectedBackup.kind,
    );
    if (!currentEntry) {
      setNotice("备份对应的当前条目不存在或类型已变化，恢复已安全阻止；没有创建新文件或整库覆盖。");
      return;
    }
    setBusy("restore");
    try {
      const revision = await loadKnowledgeWorkspaceRecoveryRevision(client, currentEntry);
      await client.restoreRecoveryBackup(selectedBackup.backup_id, revision.mtimeMs, revision.contentHash);
      await refreshMaintenance("已按当前 CAS 恢复这一条固定 vault 文件；其余知识文件没有被改写。");
      onWorkspaceMutation?.();
    } catch (error) {
      setNotice(recoveryFailureNotice(error, "单条恢复"));
    } finally {
      setBusy(null);
    }
  }, [client, onWorkspaceMutation, recoveryEntries, refreshMaintenance, selectedBackup]);

  const textPreview = useMemo(
    () => (attachment && isTextAttachment(attachment.mime_type) ? textPreviewForAttachment(attachment) : null),
    [attachment],
  );

  return (
    <section className="native-knowledge-maintenance" aria-label="Syn 知识工作区附件与恢复">
      <header className="native-maintenance-head">
        <div>
          <p className="eyebrow">Syn 原生维护</p>
          <h2>受限附件与恢复</h2>
        </div>
        <div className="native-maintenance-head-actions">
          <p>只操作固定 Syn vault；附件、备份与恢复不会读取任意外部路径。</p>
          <button className="text-button" type="button" onClick={() => void refreshMaintenance()} disabled={busy !== null}>
            {busy === "refresh" ? "正在刷新…" : "刷新 manifest"}
          </button>
        </div>
      </header>
      <div className="native-maintenance-grid">
        <section className="native-maintenance-section" aria-label="受限附件导入与显示">
          <p className="eyebrow">附件</p>
          <strong>从浏览器选择一个文件</strong>
          <p>只把字节、显示名和由扩展名固定的 MIME 类型交给受限 host command；来源路径不会进入 Syn。</p>
          <label className="native-maintenance-file-field">
            <span>选择允许的附件</span>
            <input
              ref={inputRef}
              aria-label="选择允许的知识附件"
              type="file"
              accept=".png,.jpg,.jpeg,.gif,.webp,.pdf,.txt,.csv,image/png,image/jpeg,image/gif,image/webp,application/pdf,text/plain,text/csv"
              disabled={busy !== null}
              onChange={(event) => setSelectedFile(event.currentTarget.files?.item(0) ?? null)}
            />
          </label>
          {selectedFile ? <p className="muted small-note">已选择 {selectedFile.name}（{formatBytes(selectedFile.size)}）；导入前仍会做固定类型和大小校验。</p> : null}
          <button className="secondary-button" type="button" onClick={() => void importAttachment()} disabled={!selectedFile || busy !== null}>
            {busy === "import" ? "正在导入…" : "导入附件"}
          </button>
          <label className="native-maintenance-select-field">
            <span>固定 vault 内的附件</span>
            <select
              aria-label="固定 vault 内的附件"
              value={selectedAttachmentPath ?? ""}
              onChange={(event) => setSelectedAttachmentPath(event.target.value || null)}
              disabled={!attachmentEntries.length || busy !== null}
            >
              {!attachmentEntries.length ? <option value="">暂无允许附件</option> : null}
              {attachmentEntries.map((entry) => <option value={entry.relative_path} key={entry.relative_path}>{entry.relative_path}</option>)}
            </select>
          </label>
          {attachmentLoadState === "loading" ? <p className="muted small-note">正在从固定 vault 读取受限附件…</p> : null}
          {attachmentLoadState === "unavailable" ? <p className="state-warning">附件不可读取；现有引用仍会保留为可恢复错误。</p> : null}
          {attachment ? (
            <div className="native-maintenance-attachment-preview">
              <p><code>{attachment.relative_path}</code> · {attachment.mime_type} · {formatBytes(attachment.size_bytes)}</p>
              {attachmentPreviewUrl ? <img src={attachmentPreviewUrl} alt={displayNameFromRelativePath(attachment.relative_path)} /> : null}
              {textPreview ? <pre>{textPreview}</pre> : null}
              {attachment.mime_type === "application/pdf" ? <p className="muted small-note">PDF 已按受限 bytes 读取；本阶段不启用外部预览或外部打开。</p> : null}
            </div>
          ) : null}
        </section>

        <section className="native-maintenance-section" aria-label="附件相对引用">
          <p className="eyebrow">相对引用</p>
          <strong>Markdown 与 Canvas 共用 vault 内路径</strong>
          <p>引用只使用 <code>attachments/…</code> 相对路径；不会把来源路径、URI 或外部打开动作写进 Markdown / Canvas。</p>
          {selectedAttachmentMarkdownReference ? (
            <div className="native-maintenance-reference">
              <p>Markdown</p>
              <code>{selectedAttachmentMarkdownReference}</code>
              <button className="secondary-button" type="button" onClick={insertMarkdownReference} disabled={busy !== null}>
                加入当前 Markdown 草稿
              </button>
              <p>Canvas 文件节点</p>
              <code>{selectedAttachmentCanvasReference}</code>
              <p className="muted small-note">导入后会刷新 Canvas 的受限文件选项；你仍需在 Canvas 中显式选择并保存。</p>
            </div>
          ) : (
            <p className="muted small-note">选择一项固定 vault 内的附件后，才会显示可插入的相对引用。</p>
          )}
        </section>

        <section className="native-maintenance-section" aria-label="单条恢复">
          <p className="eyebrow">恢复</p>
          <strong>只创建备份并恢复单条</strong>
          <p>每次操作都会先重新读取当前 revision；不会整库覆盖、静默删除或清理知识文件。</p>
          <label className="native-maintenance-select-field">
            <span>创建备份的当前条目</span>
            <select
              aria-label="创建备份的当前条目"
              value={selectedRecoveryPath ?? ""}
              onChange={(event) => setSelectedRecoveryPath(event.target.value || null)}
              disabled={!recoveryEntries.length || busy !== null}
            >
              {!recoveryEntries.length ? <option value="">manifest 中暂无可恢复条目</option> : null}
              {recoveryEntries.map((entry) => <option value={entry.relative_path} key={entry.relative_path}>{entry.relative_path}</option>)}
            </select>
          </label>
          <button className="secondary-button" type="button" onClick={() => void createRecoveryBackup()} disabled={!selectedRecoveryEntry || busy !== null}>
            {busy === "backup" ? "正在创建备份…" : "创建可恢复备份"}
          </button>
          <label className="native-maintenance-select-field">
            <span>按备份恢复单条</span>
            <select
              aria-label="按备份恢复单条"
              value={selectedBackupId ?? ""}
              onChange={(event) => setSelectedBackupId(event.target.value || null)}
              disabled={!backups.length || busy !== null}
            >
              {!backups.length ? <option value="">暂无已列出的恢复备份</option> : null}
              {backups.map((backup) => (
                <option value={backup.backup_id} key={backup.backup_id}>
                  {backup.relative_path} · {formatBackupMoment(backup.created_at_ms)}
                </option>
              ))}
            </select>
          </label>
          {selectedBackup ? <p className="muted small-note">将按当前 CAS 恢复 <code>{selectedBackup.relative_path}</code>；不创建新路径，不影响其他条目。</p> : null}
          <button className="text-button" type="button" onClick={() => void restoreRecoveryBackup()} disabled={!selectedBackup || busy !== null}>
            {busy === "restore" ? "正在单条恢复…" : "单条恢复"}
          </button>
        </section>
      </div>
      {manifest?.diagnostics.length ? <p className="muted small-note native-maintenance-diagnostic">manifest 返回 {manifest.diagnostics.length} 条受限诊断；未把诊断变成额外文件扫描。</p> : null}
      {notice ? <p className="native-maintenance-notice" role="status">{notice}</p> : null}
    </section>
  );
}

function isRecoveryManifestEntry(entry: KnowledgeWorkspaceVaultManifestEntry): entry is KnowledgeWorkspaceRecoveryEntry {
  return entry.kind === "markdown" || entry.kind === "canvas" || entry.kind === "attachment";
}

function attachmentMimeTypeForDisplayName(displayName: string): KnowledgeWorkspaceAttachmentMimeType | null {
  const extension = displayName.split(".").at(-1);
  switch (extension) {
    case "png": return "image/png";
    case "jpg":
    case "jpeg": return "image/jpeg";
    case "gif": return "image/gif";
    case "webp": return "image/webp";
    case "pdf": return "application/pdf";
    case "txt": return "text/plain";
    case "csv": return "text/csv";
    default: return null;
  }
}

function isImageAttachment(mimeType: KnowledgeWorkspaceAttachmentMimeType): boolean {
  return mimeType === "image/png" || mimeType === "image/jpeg" || mimeType === "image/gif" || mimeType === "image/webp";
}

function isTextAttachment(mimeType: KnowledgeWorkspaceAttachmentMimeType): boolean {
  return mimeType === "text/plain" || mimeType === "text/csv";
}

function textPreviewForAttachment(attachment: KnowledgeWorkspaceAttachment): string {
  try {
    const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(attachment.bytes));
    return text.length > TEXT_ATTACHMENT_PREVIEW_CHARS ? `${text.slice(0, TEXT_ATTACHMENT_PREVIEW_CHARS)}\n…` : text;
  } catch {
    return "（文本附件无法安全预览；原始文件未被改写。）";
  }
}

function displayNameFromRelativePath(relativePath: string): string {
  return relativePath.split("/").at(-1) ?? "附件";
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "大小未知";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KiB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
}

function formatBackupMoment(createdAtMs: number): string {
  if (!Number.isFinite(createdAtMs) || createdAtMs < 0) return "时间未知";
  return new Date(createdAtMs).toLocaleString("zh-CN", { hour12: false });
}

function recoveryFailureNotice(error: unknown, action: string): string {
  const message = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  if (message.includes("knowledge_vault_conflict")) {
    return `${action}被当前 revision 冲突保护阻止；请刷新后明确重试，现有文件没有被覆盖。`;
  }
  return `${action}没有完成；固定 vault、单条路径和备份 ID 边界没有被放宽。`;
}
