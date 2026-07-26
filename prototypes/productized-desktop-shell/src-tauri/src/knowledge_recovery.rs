//! N5 的即时 vault manifest 与显式单条恢复备份。
//!
//! 备份只是一条经用户请求创建的恢复副本；正常知识读取、搜索和索引永远不读取它，
//! 因而不会成为第二知识真相源。

use super::{
    append_audit_event, ensure_workspace_recovery_backups_root, mtime_ms_of,
    require_workspace_attachment_path, resolve_existing_workspace_path,
    validate_workspace_relative_path, workspace_recovery_backups_root_for_read,
    workspace_vault_root, workspace_workflow_state_path, write_workspace_temporary_bytes,
    ValidatedVaultRelativePath, MAX_ATTACHMENT_BYTES, MAX_CANVAS_BYTES, MAX_MARKDOWN_BYTES,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const BACKUP_ID_HEX_CHARS: usize = 32;
const MAX_RECOVERY_BACKUPS: usize = 32;
const MAX_RECOVERY_BACKUP_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_BACKUP_METADATA_BYTES: u64 = 4 * 1024;
static RECOVERY_BACKUP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecoverableWorkspaceKind {
    Markdown,
    Canvas,
    Attachment,
}

impl RecoverableWorkspaceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Canvas => "canvas",
            Self::Attachment => "attachment",
        }
    }

    fn max_bytes(self) -> u64 {
        match self {
            Self::Markdown => MAX_MARKDOWN_BYTES,
            Self::Canvas => MAX_CANVAS_BYTES,
            Self::Attachment => MAX_ATTACHMENT_BYTES,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RecoverableWorkspaceFile {
    relative_path: String,
    kind: RecoverableWorkspaceKind,
    bytes: Vec<u8>,
    mtime_ms: i64,
    content_hash: String,
    size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceRecoveryBackup {
    backup_id: String,
    relative_path: String,
    kind: &'static str,
    size_bytes: u64,
    content_hash: String,
    created_at_ms: i64,
    audit_event_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceRecoveryBackupSummary {
    backup_id: String,
    relative_path: String,
    kind: &'static str,
    size_bytes: u64,
    content_hash: String,
    created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceRecoveryRestoreResult {
    backup_id: String,
    relative_path: String,
    mtime_ms: i64,
    content_hash: String,
    audit_event_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RecoveryBackupMetadata {
    schema_version: String,
    backup_id: String,
    relative_path: String,
    kind: String,
    size_bytes: u64,
    content_hash: String,
    created_at_ms: i64,
}

fn recoverable_kind_for_path(
    relative_path: &ValidatedVaultRelativePath,
) -> Result<RecoverableWorkspaceKind, String> {
    // `attachments/` is a distinct bounded namespace. Do not reinterpret an externally placed
    // `attachments/foo.md` as a Markdown note when the rebuildable index would reject it.
    if relative_path.as_str().starts_with("attachments/") {
        require_workspace_attachment_path(relative_path)?;
        return Ok(RecoverableWorkspaceKind::Attachment);
    }
    if relative_path.file_name().ends_with(".md") && relative_path.file_name().len() > ".md".len() {
        return Ok(RecoverableWorkspaceKind::Markdown);
    }
    if relative_path.file_name().ends_with(".canvas")
        && relative_path.file_name().len() > ".canvas".len()
    {
        return Ok(RecoverableWorkspaceKind::Canvas);
    }
    Err(
        "knowledge_workspace_recovery_unsupported_entry: 备份和恢复只支持受限 Markdown、Canvas 或附件。"
            .to_string(),
    )
}

fn validate_recoverable_file_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
    kind: RecoverableWorkspaceKind,
) -> Result<(), String> {
    match kind {
        RecoverableWorkspaceKind::Markdown => {
            let _ = super::read_workspace_markdown_at(vault_root, relative_path)?;
        }
        RecoverableWorkspaceKind::Canvas => {
            let _ = super::knowledge_canvas::read_workspace_canvas_at(
                vault_root,
                relative_path.as_str(),
            )?;
        }
        RecoverableWorkspaceKind::Attachment => {
            let _ = super::knowledge_attachments::read_workspace_attachment_at(
                vault_root,
                relative_path.as_str(),
            )?;
        }
    }
    Ok(())
}

fn read_recoverable_file_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<RecoverableWorkspaceFile, String> {
    let kind = recoverable_kind_for_path(relative_path)?;
    validate_recoverable_file_at(vault_root, relative_path, kind)?;
    let path = resolve_existing_workspace_path(vault_root, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_recovery_unreadable: 无法读取受控恢复目标状态。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > kind.max_bytes()
    {
        return Err(
            "knowledge_workspace_recovery_unsupported_entry: 恢复目标不再是受限普通文件。"
                .to_string(),
        );
    }
    let bytes = fs::read(&path).map_err(|_| {
        "knowledge_workspace_recovery_unreadable: 无法读取受控恢复目标文件。".to_string()
    })?;
    if bytes.len() as u64 > kind.max_bytes() {
        return Err(
            "knowledge_workspace_recovery_unsupported_entry: 恢复目标超过受限文件上限。"
                .to_string(),
        );
    }
    Ok(RecoverableWorkspaceFile {
        relative_path: relative_path.as_str().to_string(),
        kind,
        content_hash: crate::utils::hash::sha256_hex_bytes(&bytes),
        size_bytes: bytes.len() as u64,
        bytes,
        mtime_ms: mtime_ms_of(&metadata),
    })
}

pub(crate) fn read_recoverable_workspace_file_at(
    vault_root: &Path,
    raw_relative_path: &str,
) -> Result<RecoverableWorkspaceFile, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    read_recoverable_file_at(vault_root, &relative_path)
}

fn backup_metadata_path(backup_root: &Path, backup_id: &str) -> PathBuf {
    backup_root.join(format!("{backup_id}.json"))
}

fn backup_payload_path(backup_root: &Path, backup_id: &str) -> PathBuf {
    backup_root.join(format!("{backup_id}.payload"))
}

fn validate_backup_id(backup_id: &str) -> Result<(), String> {
    if backup_id.len() != BACKUP_ID_HEX_CHARS
        || !backup_id
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(
            "knowledge_workspace_backup_invalid_id: 恢复备份 ID 必须是后端生成的固定十六进制标识。"
                .to_string(),
        );
    }
    Ok(())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

fn require_backup_root(backup_root: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(backup_root).map_err(|_| {
        "knowledge_workspace_recovery_invalid: 固定恢复备份目录不可读取。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(
            "knowledge_workspace_recovery_invalid: 固定恢复备份目录必须是普通目录。".to_string(),
        );
    }
    Ok(())
}

fn validate_backup_metadata(
    metadata: RecoveryBackupMetadata,
    expected_backup_id: &str,
) -> Result<RecoveryBackupMetadata, String> {
    if metadata.schema_version != "syn_knowledge_workspace_recovery_v1"
        || metadata.backup_id != expected_backup_id
        || !is_sha256_hex(&metadata.content_hash)
        || metadata.created_at_ms < 0
    {
        return Err(
            "knowledge_workspace_backup_corrupt: 恢复备份元数据不符合受控格式。".to_string(),
        );
    }
    let relative_path =
        validate_workspace_relative_path(&metadata.relative_path).map_err(|_| {
            "knowledge_workspace_backup_corrupt: 恢复备份路径不符合固定 vault 合同。".to_string()
        })?;
    let kind = recoverable_kind_for_path(&relative_path)?;
    if metadata.kind != kind.as_str() || metadata.size_bytes > kind.max_bytes() {
        return Err(
            "knowledge_workspace_backup_corrupt: 恢复备份类型或大小不符合受控格式。".to_string(),
        );
    }
    Ok(metadata)
}

fn read_backup_metadata_at(
    backup_root: &Path,
    backup_id: &str,
) -> Result<RecoveryBackupMetadata, String> {
    validate_backup_id(backup_id)?;
    require_backup_root(backup_root)?;
    let path = backup_metadata_path(backup_root, backup_id);
    let file_metadata = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "knowledge_workspace_backup_not_found: 指定恢复备份不存在。".to_string()
        } else {
            "knowledge_workspace_backup_corrupt: 无法读取恢复备份元数据。".to_string()
        }
    })?;
    if file_metadata.file_type().is_symlink()
        || !file_metadata.is_file()
        || file_metadata.len() > MAX_BACKUP_METADATA_BYTES
    {
        return Err(
            "knowledge_workspace_backup_corrupt: 恢复备份元数据必须是受限普通文件。".to_string(),
        );
    }
    let bytes = fs::read(&path)
        .map_err(|_| "knowledge_workspace_backup_corrupt: 无法读取恢复备份元数据。".to_string())?;
    let metadata = serde_json::from_slice::<RecoveryBackupMetadata>(&bytes).map_err(|_| {
        "knowledge_workspace_backup_corrupt: 恢复备份元数据不是有效 JSON。".to_string()
    })?;
    validate_backup_metadata(metadata, backup_id)
}

fn summary_from_metadata(
    metadata: &RecoveryBackupMetadata,
) -> KnowledgeWorkspaceRecoveryBackupSummary {
    KnowledgeWorkspaceRecoveryBackupSummary {
        backup_id: metadata.backup_id.clone(),
        relative_path: metadata.relative_path.clone(),
        kind: match metadata.kind.as_str() {
            "markdown" => "markdown",
            "canvas" => "canvas",
            "attachment" => "attachment",
            _ => "invalid",
        },
        size_bytes: metadata.size_bytes,
        content_hash: metadata.content_hash.clone(),
        created_at_ms: metadata.created_at_ms,
    }
}

pub(crate) fn list_workspace_recovery_backups_at(
    backup_root: &Path,
) -> Result<Vec<KnowledgeWorkspaceRecoveryBackupSummary>, String> {
    require_backup_root(backup_root)?;
    let entries = fs::read_dir(backup_root).map_err(|_| {
        "knowledge_workspace_recovery_invalid: 无法枚举固定恢复备份目录。".to_string()
    })?;
    let mut backup_ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| {
            "knowledge_workspace_recovery_invalid: 无法枚举固定恢复备份条目。".to_string()
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(
                "knowledge_workspace_recovery_invalid: 恢复备份目录含非 UTF-8 条目。".to_string(),
            );
        };
        let Some(backup_id) = name.strip_suffix(".json") else {
            continue;
        };
        validate_backup_id(backup_id)?;
        backup_ids.push(backup_id.to_string());
    }
    backup_ids.sort();
    if backup_ids.len() > MAX_RECOVERY_BACKUPS {
        return Err(
            "knowledge_workspace_backup_limit_reached: 恢复备份数量超过本阶段受控上限。"
                .to_string(),
        );
    }
    let mut summaries = Vec::with_capacity(backup_ids.len());
    let mut total_bytes = 0_u64;
    for backup_id in backup_ids {
        let metadata = read_backup_metadata_at(backup_root, &backup_id)?;
        total_bytes = total_bytes
            .checked_add(metadata.size_bytes)
            .ok_or_else(|| {
                "knowledge_workspace_backup_limit_reached: 恢复备份大小超过受控上限。".to_string()
            })?;
        if total_bytes > MAX_RECOVERY_BACKUP_TOTAL_BYTES {
            return Err(
                "knowledge_workspace_backup_limit_reached: 恢复备份大小超过本阶段受控上限。"
                    .to_string(),
            );
        }
        summaries.push(summary_from_metadata(&metadata));
    }
    summaries.sort_by(|left, right| {
        right
            .created_at_ms
            .cmp(&left.created_at_ms)
            .then_with(|| left.backup_id.cmp(&right.backup_id))
    });
    Ok(summaries)
}

fn next_backup_id(
    backup_root: &Path,
    relative_path: &str,
    content_hash: &str,
) -> Result<String, String> {
    for _ in 0..32 {
        let sequence = RECOVERY_BACKUP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate: String = crate::utils::hash::sha256_hex(&format!(
            "syn-knowledge-workspace-recovery:{}:{}:{}:{}",
            super::unix_timestamp_ms(),
            std::process::id(),
            sequence,
            relative_path
        ))
        .chars()
        .take(BACKUP_ID_HEX_CHARS)
        .collect();
        if candidate
            != content_hash
                .chars()
                .take(BACKUP_ID_HEX_CHARS)
                .collect::<String>()
            && !backup_metadata_path(backup_root, &candidate).exists()
            && !backup_payload_path(backup_root, &candidate).exists()
        {
            return Ok(candidate);
        }
    }
    Err("knowledge_workspace_backup_id_exhausted: 无法生成新的受控恢复备份 ID。".to_string())
}

fn atomically_create_backup_file(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = write_workspace_temporary_bytes(target, bytes)?;
    fs::rename(&temporary, target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_backup_write_failed: 无法原子写入恢复备份。".to_string()
    })
}

/// 在触及恢复目录之前完成路径、类型和 CAS 校验。command handler 先调用它，确保非法
/// 请求或已 stale 的请求不会以“检查备份”为由创建任何新目录；实际写入函数会再校验一次，
/// 覆盖两次检查之间的外部变更。
fn current_recovery_backup_source_at(
    vault_root: &Path,
    raw_relative_path: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<RecoverableWorkspaceFile, String> {
    let source = read_recoverable_workspace_file_at(vault_root, raw_relative_path)?;
    if source.mtime_ms != expected_mtime_ms || source.content_hash != expected_content_hash {
        return Err(
            "knowledge_vault_conflict: 这条知识文件已被外部来源或另一窗口修改，请先重新读取后再备份。"
                .to_string(),
        );
    }
    Ok(source)
}

pub(crate) fn create_workspace_recovery_backup_at(
    vault_root: &Path,
    backup_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceRecoveryBackup, String> {
    let source = current_recovery_backup_source_at(
        vault_root,
        raw_relative_path,
        expected_mtime_ms,
        expected_content_hash,
    )?;
    require_backup_root(backup_root)?;
    let existing = list_workspace_recovery_backups_at(backup_root)?;
    if existing.len() >= MAX_RECOVERY_BACKUPS
        || existing
            .iter()
            .try_fold(0_u64, |total, item| total.checked_add(item.size_bytes))
            .ok_or_else(|| {
                "knowledge_workspace_backup_limit_reached: 恢复备份大小超过受控上限。".to_string()
            })?
            .checked_add(source.size_bytes)
            .filter(|total| *total <= MAX_RECOVERY_BACKUP_TOTAL_BYTES)
            .is_none()
    {
        return Err(
            "knowledge_workspace_backup_limit_reached: 恢复备份已达到受控数量或大小上限。"
                .to_string(),
        );
    }
    let backup_id = next_backup_id(backup_root, &source.relative_path, &source.content_hash)?;
    let metadata = RecoveryBackupMetadata {
        schema_version: "syn_knowledge_workspace_recovery_v1".to_string(),
        backup_id: backup_id.clone(),
        relative_path: source.relative_path.clone(),
        kind: source.kind.as_str().to_string(),
        size_bytes: source.size_bytes,
        content_hash: source.content_hash.clone(),
        created_at_ms: super::unix_timestamp_ms(),
    };
    let payload_path = backup_payload_path(backup_root, &backup_id);
    let metadata_path = backup_metadata_path(backup_root, &backup_id);
    atomically_create_backup_file(&payload_path, &source.bytes)?;
    let metadata_bytes = serde_json::to_vec(&metadata).map_err(|_| {
        "knowledge_workspace_backup_write_failed: 无法序列化恢复备份元数据。".to_string()
    })?;
    if let Err(error) = atomically_create_backup_file(&metadata_path, &metadata_bytes) {
        let _ = fs::remove_file(&payload_path);
        return Err(error);
    }
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_recovery_backup_created",
        &source.relative_path,
        "user_manual_edit",
        "用户在 Syn 原生知识工作区创建单条恢复备份。",
    )?;
    Ok(KnowledgeWorkspaceRecoveryBackup {
        audit_event_id,
        backup_id,
        relative_path: source.relative_path,
        kind: source.kind.as_str(),
        size_bytes: source.size_bytes,
        content_hash: source.content_hash,
        created_at_ms: metadata.created_at_ms,
    })
}

fn read_backup_payload_at(
    backup_root: &Path,
    metadata: &RecoveryBackupMetadata,
) -> Result<Vec<u8>, String> {
    let path = backup_payload_path(backup_root, &metadata.backup_id);
    let file_metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_backup_corrupt: 恢复备份载荷不存在或不可读取。".to_string()
    })?;
    if file_metadata.file_type().is_symlink()
        || !file_metadata.is_file()
        || file_metadata.len() != metadata.size_bytes
    {
        return Err(
            "knowledge_workspace_backup_corrupt: 恢复备份载荷必须是大小一致的普通文件。"
                .to_string(),
        );
    }
    let bytes = fs::read(&path)
        .map_err(|_| "knowledge_workspace_backup_corrupt: 无法读取恢复备份载荷。".to_string())?;
    if bytes.len() as u64 != metadata.size_bytes
        || crate::utils::hash::sha256_hex_bytes(&bytes) != metadata.content_hash
    {
        return Err("knowledge_workspace_backup_corrupt: 恢复备份载荷校验失败。".to_string());
    }
    Ok(bytes)
}

pub(crate) fn restore_workspace_recovery_backup_at(
    vault_root: &Path,
    backup_root: &Path,
    workflow_state_path: &Path,
    backup_id: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceRecoveryRestoreResult, String> {
    let metadata = read_backup_metadata_at(backup_root, backup_id)?;
    let relative_path =
        validate_workspace_relative_path(&metadata.relative_path).map_err(|_| {
            "knowledge_workspace_backup_corrupt: 恢复备份路径不符合固定 vault 合同。".to_string()
        })?;
    let current = read_recoverable_file_at(vault_root, &relative_path)?;
    if current.kind.as_str() != metadata.kind
        || current.mtime_ms != expected_mtime_ms
        || current.content_hash != expected_content_hash
    {
        return Err(
            "knowledge_vault_conflict: 这条知识文件已被外部来源或另一窗口修改，请先重新读取后再恢复。"
                .to_string(),
        );
    }
    let payload = read_backup_payload_at(backup_root, &metadata)?;
    let target = resolve_existing_workspace_path(vault_root, &relative_path)?;
    let temporary = write_workspace_temporary_bytes(&target, &payload)?;
    fs::rename(&temporary, &target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_replace_failed: 无法原子恢复固定知识文件。".to_string()
    })?;
    let restored = read_recoverable_file_at(vault_root, &relative_path)?;
    if restored.content_hash != metadata.content_hash || restored.size_bytes != metadata.size_bytes
    {
        return Err(
            "knowledge_workspace_backup_corrupt: 恢复后的文件与受控备份校验不一致。".to_string(),
        );
    }
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_recovery_backup_restored",
        &restored.relative_path,
        "user_manual_edit",
        "用户在 Syn 原生知识工作区按 CAS 恢复单条备份。",
    )?;
    Ok(KnowledgeWorkspaceRecoveryRestoreResult {
        backup_id: metadata.backup_id,
        relative_path: restored.relative_path,
        mtime_ms: restored.mtime_ms,
        content_hash: restored.content_hash,
        audit_event_id,
    })
}

#[tauri::command]
pub(crate) fn knowledge_workspace_create_recovery_backup(
    relative_path: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceRecoveryBackup, String> {
    // Validate before ensure_*: rejected inputs must not initialize a recovery directory.
    let _ = current_recovery_backup_source_at(
        &workspace_vault_root(),
        &relative_path,
        expected_mtime_ms,
        &expected_content_hash,
    )?;
    let backup_root = ensure_workspace_recovery_backups_root()?;
    create_workspace_recovery_backup_at(
        &workspace_vault_root(),
        &backup_root,
        &workspace_workflow_state_path(),
        &relative_path,
        expected_mtime_ms,
        &expected_content_hash,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_list_recovery_backups(
) -> Result<Vec<KnowledgeWorkspaceRecoveryBackupSummary>, String> {
    match workspace_recovery_backups_root_for_read()? {
        Some(backup_root) => list_workspace_recovery_backups_at(&backup_root),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub(crate) fn knowledge_workspace_restore_recovery_backup(
    backup_id: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceRecoveryRestoreResult, String> {
    let backup_root = workspace_recovery_backups_root_for_read()?
        .ok_or_else(|| "knowledge_workspace_backup_not_found: 指定恢复备份不存在。".to_string())?;
    restore_workspace_recovery_backup_at(
        &workspace_vault_root(),
        &backup_root,
        &workspace_workflow_state_path(),
        &backup_id,
        expected_mtime_ms,
        &expected_content_hash,
    )
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
            "syn-knowledge-recovery-{tag}-{}-{sequence}",
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
    fn manifest_is_rebuilt_from_the_vault_and_backup_restore_is_single_entry_cas() {
        let root = temp_root("restore-red");
        let backups = temp_root("backup-red");
        let state = workflow_state_path(&root);
        fs::write(root.join("note.md"), "# Original\n").unwrap();

        let manifest = crate::knowledge_index::workspace_vault_manifest_at(&root).unwrap();
        assert!(manifest
            .entries
            .iter()
            .any(|entry| entry.relative_path == "note.md"));

        let source = read_recoverable_workspace_file_at(&root, "note.md").unwrap();
        let backup = create_workspace_recovery_backup_at(
            &root,
            &backups,
            &state,
            "note.md",
            source.mtime_ms,
            &source.content_hash,
        )
        .expect("N5 must create an opaque controlled backup for one current entry");
        assert!(!backup.backup_id.contains('/') && !backup.backup_id.contains("note.md"));

        fs::write(root.join("note.md"), "# Changed\n").unwrap();
        let changed = read_recoverable_workspace_file_at(&root, "note.md").unwrap();
        let restored = restore_workspace_recovery_backup_at(
            &root,
            &backups,
            &state,
            &backup.backup_id,
            changed.mtime_ms,
            &changed.content_hash,
        )
        .expect("explicit restore with the current CAS must only replace its original entry");
        assert_eq!(restored.relative_path, "note.md");
        assert_eq!(
            fs::read_to_string(root.join("note.md")).unwrap(),
            "# Original\n"
        );

        let now = read_recoverable_workspace_file_at(&root, "note.md").unwrap();
        assert!(restore_workspace_recovery_backup_at(
            &root,
            &backups,
            &state,
            &backup.backup_id,
            now.mtime_ms.saturating_add(1),
            &now.content_hash,
        )
        .unwrap_err()
        .starts_with("knowledge_vault_conflict:"));
        assert!(restore_workspace_recovery_backup_at(
            &root,
            &backups,
            &state,
            "../../outside",
            now.mtime_ms,
            &now.content_hash,
        )
        .unwrap_err()
        .starts_with("knowledge_workspace_backup_invalid_id:"));
    }
}
