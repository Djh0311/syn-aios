//! N5 的受限附件导入与读取边界。
//!
//! 这里的唯一写入入口只接受用户在浏览器 File 控件中读取出的 bytes、显示名和 MIME；
//! 它绝不接收源路径、URL、shell 参数或任意 vault 根。

use super::{
    append_audit_event, ensure_workspace_attachments_directory_at, mtime_ms_of,
    require_workspace_attachment_path, resolve_existing_workspace_path, resolve_new_workspace_path,
    validate_workspace_relative_path, workspace_attachment_kind_for_import, workspace_vault_root,
    workspace_workflow_state_path, write_workspace_temporary_bytes, ValidatedVaultRelativePath,
    WorkspaceAttachmentKind, MAX_ATTACHMENT_BYTES,
};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceAttachment {
    relative_path: String,
    mime_type: &'static str,
    bytes: Vec<u8>,
    mtime_ms: i64,
    content_hash: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceAttachmentImportResult {
    relative_path: String,
    mime_type: &'static str,
    mtime_ms: i64,
    content_hash: String,
    size_bytes: u64,
    audit_event_id: String,
}

fn attachment_too_large() -> String {
    "knowledge_workspace_attachment_too_large: 附件超过本阶段 10 MiB 安全上限。".to_string()
}

fn read_attachment_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<KnowledgeWorkspaceAttachment, String> {
    let kind = require_workspace_attachment_path(relative_path)?;
    let path = resolve_existing_workspace_path(vault_root, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_attachment_unreadable: 无法读取固定附件条目状态。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(
            "knowledge_workspace_attachment_only: 路径必须指向固定附件目录内的普通文件。"
                .to_string(),
        );
    }
    if metadata.len() > MAX_ATTACHMENT_BYTES {
        return Err(attachment_too_large());
    }
    let bytes = fs::read(&path).map_err(|_| {
        "knowledge_workspace_attachment_unreadable: 无法读取固定附件文件。".to_string()
    })?;
    if bytes.len() as u64 > MAX_ATTACHMENT_BYTES {
        return Err(attachment_too_large());
    }
    Ok(KnowledgeWorkspaceAttachment {
        relative_path: relative_path.as_str().to_string(),
        mime_type: kind.mime_type(),
        content_hash: crate::utils::hash::sha256_hex_bytes(&bytes),
        size_bytes: bytes.len() as u64,
        bytes,
        mtime_ms: mtime_ms_of(&metadata),
    })
}

pub(crate) fn read_workspace_attachment_at(
    vault_root: &Path,
    raw_relative_path: &str,
) -> Result<KnowledgeWorkspaceAttachment, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    read_attachment_at(vault_root, &relative_path)
}

pub(crate) fn workspace_import_attachment_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    bytes: &[u8],
    display_name: &str,
    mime_type: &str,
) -> Result<KnowledgeWorkspaceAttachmentImportResult, String> {
    if bytes.len() as u64 > MAX_ATTACHMENT_BYTES {
        return Err(attachment_too_large());
    }
    // 先在零写前完成显示名、扩展名和 MIME 的交叉检查；绝不以失败的导入创建 vault/附件目录。
    let (relative_path, kind) = workspace_attachment_kind_for_import(display_name, mime_type)?;
    let _attachments_directory = ensure_workspace_attachments_directory_at(vault_root)?;
    let target = resolve_new_workspace_path(vault_root, &relative_path)?;
    let temporary = write_workspace_temporary_bytes(&target, bytes)?;
    fs::rename(&temporary, &target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_create_failed: 无法原子创建固定附件。".to_string()
    })?;
    let attachment = read_attachment_at(vault_root, &relative_path)?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_attachment_imported",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区导入受限附件。",
    )?;
    Ok(attachment_import_result(&attachment, kind, audit_event_id))
}

fn attachment_import_result(
    attachment: &KnowledgeWorkspaceAttachment,
    kind: WorkspaceAttachmentKind,
    audit_event_id: String,
) -> KnowledgeWorkspaceAttachmentImportResult {
    KnowledgeWorkspaceAttachmentImportResult {
        relative_path: attachment.relative_path.clone(),
        mime_type: kind.mime_type(),
        mtime_ms: attachment.mtime_ms,
        content_hash: attachment.content_hash.clone(),
        size_bytes: attachment.size_bytes,
        audit_event_id,
    }
}

#[tauri::command]
pub(crate) fn knowledge_workspace_import_attachment(
    bytes: Vec<u8>,
    display_name: String,
    mime_type: String,
) -> Result<KnowledgeWorkspaceAttachmentImportResult, String> {
    workspace_import_attachment_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &bytes,
        &display_name,
        &mime_type,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_read_attachment(
    relative_path: String,
) -> Result<KnowledgeWorkspaceAttachment, String> {
    read_workspace_attachment_at(&workspace_vault_root(), &relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_root(tag: &str) -> PathBuf {
        let sequence = TEST_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-knowledge-attachment-{tag}-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn workflow_state_path(root: &Path) -> PathBuf {
        let path = root.join("workflow-state.json");
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "schema_version": "workflow_state_v0",
                "workflow_version": 1,
                "revision": 0,
                "projects": [],
                "agent_adapters": [],
                "workflows": [],
                "nodes": [],
                "edges": [],
                "work_items": [],
                "artifacts": [],
                "reviews": [],
                "workflow_node_session_bindings": [],
                "workflow_node_dispatches": [],
                "audit_events": [],
                "capabilities": [],
                "harness_resources": []
            }))
            .unwrap(),
        )
        .unwrap();
        path
    }

    #[test]
    fn import_is_file_bytes_only_and_rejects_unsafe_name_mime_size_or_overwrite_before_audit() {
        let root = temp_root("import-red");
        let state = workflow_state_path(&root);

        let imported = workspace_import_attachment_at(
            &root,
            &state,
            b"png bytes",
            "paper.png",
            "image/png",
        )
        .expect(
            "N5 must import a permitted browser File payload into the fixed attachment directory",
        );
        assert_eq!(imported.relative_path, "attachments/paper.png");
        assert_eq!(imported.mime_type, "image/png");
        assert_eq!(
            fs::read(root.join("attachments/paper.png")).unwrap(),
            b"png bytes"
        );

        for (display_name, mime_type, bytes) in [
            ("../outside.png", "image/png", b"x".as_slice()),
            ("folder/paper.png", "image/png", b"x".as_slice()),
            ("paper.png", "application/pdf", b"x".as_slice()),
            ("paper.exe", "application/octet-stream", b"x".as_slice()),
        ] {
            let error =
                workspace_import_attachment_at(&root, &state, bytes, display_name, mime_type)
                    .expect_err("unsafe source-shaped input must fail closed");
            assert!(
                error.starts_with("knowledge_workspace_attachment_"),
                "unexpected attachment rejection: {error}"
            );
        }
        let oversized = vec![0_u8; super::super::MAX_ATTACHMENT_BYTES as usize + 1];
        assert!(workspace_import_attachment_at(
            &root,
            &state,
            &oversized,
            "too-large.png",
            "image/png",
        )
        .unwrap_err()
        .starts_with("knowledge_workspace_attachment_too_large:"));
        assert!(
            workspace_import_attachment_at(&root, &state, b"second", "paper.png", "image/png",)
                .unwrap_err()
                .starts_with("knowledge_workspace_target_exists:")
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(&state).unwrap()).unwrap()
                ["audit_events"]
                .as_array()
                .unwrap()
                .len(),
            1,
            "all rejected import inputs must leave the audit trail untouched"
        );
    }

    #[test]
    fn attachment_read_never_accepts_a_non_attachment_path_or_leaks_a_source_path() {
        let root = temp_root("read-red");
        let state = workflow_state_path(&root);
        workspace_import_attachment_at(&root, &state, b"hello", "readme.txt", "text/plain")
            .unwrap();
        fs::write(root.join("note.md"), "# note\n").unwrap();

        let attachment = read_workspace_attachment_at(&root, "attachments/readme.txt").unwrap();
        assert_eq!(attachment.relative_path, "attachments/readme.txt");
        assert_eq!(attachment.mime_type, "text/plain");
        assert_eq!(attachment.bytes, b"hello");
        assert!(read_workspace_attachment_at(&root, "note.md")
            .unwrap_err()
            .starts_with("knowledge_workspace_attachment_only:"));
        assert!(read_workspace_attachment_at(&root, "../outside.txt")
            .unwrap_err()
            .starts_with("knowledge_workspace_invalid_path:"));
    }
}
