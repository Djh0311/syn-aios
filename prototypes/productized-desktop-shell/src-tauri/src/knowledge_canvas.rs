//! N4 的 JSON Canvas 1.0 固定 vault 边界。
//!
//! Canvas 文件本身是唯一真相；本模块不落索引、数据库或布局 sidecar。它作为
//! `knowledge_vault` 的子模块，复用父模块已有的路径锁、原子替换与审计原语。

use super::{
    append_audit_event, ensure_vault_root, mtime_ms_of, resolve_existing_workspace_path,
    resolve_new_workspace_path, validate_workspace_relative_path,
    workspace_attachment_kind_for_relative_path, workspace_vault_root,
    workspace_workflow_state_path, write_workspace_temporary_file, ValidatedVaultRelativePath,
    MAX_ATTACHMENT_BYTES, MAX_CANVAS_BYTES,
};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const MAX_CANVAS_NODES: usize = 1_024;
const MAX_CANVAS_EDGES: usize = 2_048;
const MAX_CANVAS_DIAGNOSTICS: usize = 32;
const MAX_CANVAS_ID_CHARS: usize = 256;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceCanvasDiagnostic {
    code: &'static str,
    node_id: Option<String>,
    reference: Option<String>,
    message: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceCanvasDocument {
    relative_path: String,
    document: Value,
    mtime_ms: i64,
    content_hash: String,
    diagnostics: Vec<KnowledgeWorkspaceCanvasDiagnostic>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceCanvasMutationResult {
    operation: &'static str,
    relative_path: String,
    source_relative_path: Option<String>,
    mtime_ms: Option<i64>,
    content_hash: Option<String>,
    audit_event_id: String,
}

#[derive(Clone, Copy)]
enum CanvasReferenceMode {
    ReadWithMissingDiagnostics,
    RequireExisting,
}

#[derive(Clone, Copy)]
enum CanvasReferenceKind {
    FileNode,
    GroupBackground,
}

fn require_canvas_path(relative_path: &ValidatedVaultRelativePath) -> Result<(), String> {
    if !relative_path.file_name().ends_with(".canvas")
        || relative_path.file_name().len() <= ".canvas".len()
    {
        return Err(
            "knowledge_workspace_canvas_only: 此操作只允许固定 vault 内的 .canvas 文件。"
                .to_string(),
        );
    }
    Ok(())
}

fn canvas_document_as_compact_utf8(document: &Value) -> Result<String, String> {
    let body = serde_json::to_string(document).map_err(|_| {
        "knowledge_workspace_canvas_invalid_json: Canvas 无法序列化为 JSON。".to_string()
    })?;
    if body.len() as u64 > MAX_CANVAS_BYTES {
        return Err(
            "knowledge_workspace_canvas_too_large: JSON Canvas 超过 256 KiB 安全上限。".to_string(),
        );
    }
    Ok(body)
}

fn parse_canvas_document(body: &str) -> Result<Value, String> {
    serde_json::from_str(body).map_err(|_| {
        "knowledge_workspace_canvas_invalid_json: Canvas 必须是有效 UTF-8 JSON。".to_string()
    })
}

fn canvas_object<'a>(document: &'a Value) -> Result<&'a Map<String, Value>, String> {
    document.as_object().ok_or_else(|| {
        "knowledge_workspace_canvas_invalid_structure: JSON Canvas 根必须是对象。".to_string()
    })
}

fn optional_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a Vec<Value>>, String> {
    match object.get(field) {
        None => Ok(None),
        Some(value) => value.as_array().map(Some).ok_or_else(|| {
            format!(
                "knowledge_workspace_canvas_invalid_structure: JSON Canvas 的 {field} 必须是数组。"
            )
        }),
    }
}

fn required_string<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object.get(field).and_then(Value::as_str).ok_or_else(|| {
        format!("knowledge_workspace_canvas_invalid_structure: JSON Canvas 的 {field} 必须是文本。")
    })
}

fn optional_string<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<Option<&'a str>, String> {
    match object.get(field) {
        None => Ok(None),
        Some(value) => value.as_str().map(Some).ok_or_else(|| {
            format!(
                "knowledge_workspace_canvas_invalid_structure: JSON Canvas 的 {field} 必须是文本。"
            )
        }),
    }
}

fn require_canvas_identifier(raw: &str, kind: &str) -> Result<(), String> {
    if raw.is_empty()
        || raw.chars().count() > MAX_CANVAS_ID_CHARS
        || raw.chars().any(char::is_control)
    {
        return Err(format!(
            "knowledge_workspace_canvas_invalid_{kind}_id: JSON Canvas 的 {kind} id 必须是受限文本。"
        ));
    }
    Ok(())
}

fn required_integer(object: &Map<String, Value>, field: &str) -> Result<i64, String> {
    object.get(field).and_then(Value::as_i64).ok_or_else(|| {
        format!(
            "knowledge_workspace_canvas_invalid_coordinate: JSON Canvas 的 {field} 必须是整数。"
        )
    })
}

fn validate_node_geometry(node: &Map<String, Value>) -> Result<(), String> {
    let _ = required_integer(node, "x")?;
    let _ = required_integer(node, "y")?;
    let width = required_integer(node, "width")?;
    let height = required_integer(node, "height")?;
    if width <= 0 || height <= 0 {
        return Err(
            "knowledge_workspace_canvas_invalid_coordinate: Canvas 节点宽高必须为正整数。"
                .to_string(),
        );
    }
    Ok(())
}

fn push_missing_reference_diagnostic(
    diagnostics: &mut Vec<KnowledgeWorkspaceCanvasDiagnostic>,
    node_id: &str,
    reference: &str,
) {
    if diagnostics.len() < MAX_CANVAS_DIAGNOSTICS {
        diagnostics.push(KnowledgeWorkspaceCanvasDiagnostic {
            code: "knowledge_workspace_canvas_missing_reference",
            node_id: Some(node_id.to_string()),
            reference: Some(reference.to_string()),
            message: "Canvas 引用的 vault 文件暂时缺失；可恢复后重新载入。",
        });
    }
}

fn canvas_reference_extension(relative_path: &ValidatedVaultRelativePath) -> Option<&str> {
    let (stem, extension) = relative_path.file_name().rsplit_once('.')?;
    (!stem.is_empty()).then_some(extension)
}

fn is_attachment_reference(relative_path: &ValidatedVaultRelativePath) -> bool {
    relative_path.as_str().starts_with("attachments/")
}

fn validate_canvas_reference_policy(
    relative_path: &ValidatedVaultRelativePath,
    kind: CanvasReferenceKind,
) -> Result<bool, String> {
    if is_attachment_reference(relative_path) {
        let attachment_kind = workspace_attachment_kind_for_relative_path(relative_path).ok_or_else(|| {
            "knowledge_workspace_canvas_invalid_reference: Canvas attachments/ 引用的扩展名不在本阶段允许范围。"
                .to_string()
        })?;
        if matches!(kind, CanvasReferenceKind::GroupBackground) && !attachment_kind.is_raster() {
            return Err(
                "knowledge_workspace_canvas_invalid_reference: Canvas 分组背景只能引用 attachments/ 下的栅格附件。"
                    .to_string(),
            );
        }
        return Ok(true);
    }

    if matches!(kind, CanvasReferenceKind::FileNode)
        && matches!(
            canvas_reference_extension(relative_path),
            Some("md" | "canvas")
        )
    {
        return Ok(false);
    }

    Err(
        "knowledge_workspace_canvas_invalid_reference: Canvas 文件节点只能引用 .md/.canvas，分组背景只能引用 attachments/ 下的栅格附件。"
            .to_string(),
    )
}

fn validate_canvas_reference(
    vault_root: &Path,
    raw_reference: &str,
    node_id: &str,
    diagnostics: &mut Vec<KnowledgeWorkspaceCanvasDiagnostic>,
    mode: CanvasReferenceMode,
    kind: CanvasReferenceKind,
) -> Result<(), String> {
    let relative_path = validate_workspace_relative_path(raw_reference).map_err(|_| {
        "knowledge_workspace_canvas_invalid_reference: Canvas 引用必须是固定 vault 内的受限相对路径。"
            .to_string()
    })?;
    let is_attachment = validate_canvas_reference_policy(&relative_path, kind)?;
    let resolved = resolve_existing_workspace_path(vault_root, &relative_path);
    let path = match resolved {
        Ok(path) => path,
        Err(error)
            if error.starts_with("knowledge_workspace_entry_not_found:")
                && matches!(mode, CanvasReferenceMode::ReadWithMissingDiagnostics) =>
        {
            push_missing_reference_diagnostic(diagnostics, node_id, raw_reference);
            return Ok(());
        }
        Err(error) => {
            return Err(format!(
                "knowledge_workspace_canvas_invalid_reference: Canvas 引用不满足固定 vault 路径锁（{error}）。"
            ));
        }
    };
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_canvas_invalid_reference: 无法读取 Canvas 引用的 vault 文件。"
            .to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(
            "knowledge_workspace_canvas_invalid_reference: Canvas 引用必须是固定 vault 内的普通文件。"
                .to_string(),
        );
    }
    if is_attachment && metadata.len() > MAX_ATTACHMENT_BYTES {
        return Err(
            "knowledge_workspace_canvas_invalid_reference: Canvas 引用的附件超过 10 MiB 安全上限。"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_canvas_node(
    vault_root: &Path,
    node: &Map<String, Value>,
    diagnostics: &mut Vec<KnowledgeWorkspaceCanvasDiagnostic>,
    mode: CanvasReferenceMode,
) -> Result<String, String> {
    let id = required_string(node, "id")?;
    require_canvas_identifier(id, "node")?;
    let node_type = required_string(node, "type")?;
    validate_node_geometry(node)?;
    match node_type {
        "text" => {
            let _ = required_string(node, "text")?;
        }
        "file" => {
            let reference = required_string(node, "file")?;
            validate_canvas_reference(
                vault_root,
                reference,
                id,
                diagnostics,
                mode,
                CanvasReferenceKind::FileNode,
            )?;
            if let Some(subpath) = optional_string(node, "subpath")? {
                if !subpath.starts_with('#') {
                    return Err(
                        "knowledge_workspace_canvas_invalid_subpath: Canvas 文件节点 subpath 必须以 # 开头。"
                            .to_string(),
                    );
                }
            }
        }
        "link" => {
            // URL 仅作为 JSON 文本保存和返回；本边界绝不解析、执行或打开它。
            let _ = required_string(node, "url")?;
        }
        "group" => {
            let _ = optional_string(node, "label")?;
            if let Some(background) = optional_string(node, "background")? {
                validate_canvas_reference(
                    vault_root,
                    background,
                    id,
                    diagnostics,
                    mode,
                    CanvasReferenceKind::GroupBackground,
                )?;
            }
            if let Some(background_style) = optional_string(node, "backgroundStyle")? {
                if !matches!(background_style, "cover" | "ratio" | "repeat") {
                    return Err(
                        "knowledge_workspace_canvas_invalid_background_style: Canvas 分组 backgroundStyle 只能是 cover/ratio/repeat。"
                            .to_string(),
                    );
                }
            }
        }
        _ => {
            return Err(
                "knowledge_workspace_canvas_invalid_node_type: Canvas 节点只能是 text/file/link/group。"
                    .to_string(),
            );
        }
    }
    Ok(id.to_string())
}

fn validate_edge_side(object: &Map<String, Value>, field: &str) -> Result<(), String> {
    let Some(value) = optional_string(object, field)? else {
        return Ok(());
    };
    if matches!(value, "top" | "right" | "bottom" | "left") {
        Ok(())
    } else {
        Err(format!(
            "knowledge_workspace_canvas_invalid_edge_side: Canvas 的 {field} 只能是 top/right/bottom/left。"
        ))
    }
}

fn validate_edge_end(object: &Map<String, Value>, field: &str) -> Result<(), String> {
    let Some(value) = optional_string(object, field)? else {
        return Ok(());
    };
    if matches!(value, "none" | "arrow") {
        Ok(())
    } else {
        Err(format!(
            "knowledge_workspace_canvas_invalid_edge_end: Canvas 的 {field} 只能是 none/arrow。"
        ))
    }
}

fn validate_canvas_document(
    vault_root: &Path,
    document: &Value,
    mode: CanvasReferenceMode,
) -> Result<Vec<KnowledgeWorkspaceCanvasDiagnostic>, String> {
    canvas_document_as_compact_utf8(document)?;
    let root = canvas_object(document)?;
    let nodes = optional_array(root, "nodes")?
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let edges = optional_array(root, "edges")?
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if nodes.len() > MAX_CANVAS_NODES || edges.len() > MAX_CANVAS_EDGES {
        return Err(
            "knowledge_workspace_canvas_too_complex: JSON Canvas 超过本阶段节点或连线安全上限。"
                .to_string(),
        );
    }

    let mut diagnostics = Vec::new();
    let mut node_ids = BTreeSet::new();
    for value in nodes {
        let node = value.as_object().ok_or_else(|| {
            "knowledge_workspace_canvas_invalid_structure: Canvas 的每个节点必须是对象。"
                .to_string()
        })?;
        let node_id = validate_canvas_node(vault_root, node, &mut diagnostics, mode)?;
        if !node_ids.insert(node_id) {
            return Err(
                "knowledge_workspace_canvas_duplicate_node_id: Canvas 节点 id 不能重复。"
                    .to_string(),
            );
        }
    }

    let mut edge_ids = BTreeSet::new();
    for value in edges {
        let edge = value.as_object().ok_or_else(|| {
            "knowledge_workspace_canvas_invalid_structure: Canvas 的每条连线必须是对象。"
                .to_string()
        })?;
        let edge_id = required_string(edge, "id")?;
        require_canvas_identifier(edge_id, "edge")?;
        if !edge_ids.insert(edge_id.to_string()) {
            return Err(
                "knowledge_workspace_canvas_duplicate_edge_id: Canvas 连线 id 不能重复。"
                    .to_string(),
            );
        }
        let from_node = required_string(edge, "fromNode")?;
        let to_node = required_string(edge, "toNode")?;
        if !node_ids.contains(from_node) || !node_ids.contains(to_node) {
            return Err(
                "knowledge_workspace_canvas_dangling_edge: Canvas 连线必须连接现有节点。"
                    .to_string(),
            );
        }
        validate_edge_side(edge, "fromSide")?;
        validate_edge_side(edge, "toSide")?;
        validate_edge_end(edge, "fromEnd")?;
        validate_edge_end(edge, "toEnd")?;
    }
    Ok(diagnostics)
}

fn canvas_path_and_body_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<(std::path::PathBuf, String), String> {
    require_canvas_path(relative_path)?;
    let path = resolve_existing_workspace_path(vault_root, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取 JSON Canvas 条目状态。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(
            "knowledge_workspace_canvas_only: 路径必须指向普通 JSON Canvas 文件。".to_string(),
        );
    }
    if metadata.len() > MAX_CANVAS_BYTES {
        return Err(
            "knowledge_workspace_canvas_too_large: JSON Canvas 超过 256 KiB 安全上限。".to_string(),
        );
    }
    let bytes = fs::read(&path).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取固定 vault JSON Canvas。".to_string()
    })?;
    let body = String::from_utf8(bytes).map_err(|_| {
        "knowledge_workspace_invalid_utf8: JSON Canvas 不是有效 UTF-8，已拒绝读取。".to_string()
    })?;
    Ok((path, body))
}

pub(crate) fn read_workspace_canvas_at(
    vault_root: &Path,
    raw_relative_path: &str,
) -> Result<KnowledgeWorkspaceCanvasDocument, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    read_canvas_at(vault_root, &relative_path)
}

fn read_canvas_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<KnowledgeWorkspaceCanvasDocument, String> {
    let (path, body) = canvas_path_and_body_at(vault_root, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取 JSON Canvas 条目状态。".to_string()
    })?;
    let document = parse_canvas_document(&body)?;
    let diagnostics = validate_canvas_document(
        vault_root,
        &document,
        CanvasReferenceMode::ReadWithMissingDiagnostics,
    )?;
    Ok(KnowledgeWorkspaceCanvasDocument {
        relative_path: relative_path.as_str().to_string(),
        document,
        mtime_ms: mtime_ms_of(&metadata),
        content_hash: crate::utils::hash::sha256_hex(&body),
        diagnostics,
    })
}

fn canvas_mutation_result(
    operation: &'static str,
    relative_path: &ValidatedVaultRelativePath,
    file: &KnowledgeWorkspaceCanvasDocument,
    audit_event_id: String,
) -> KnowledgeWorkspaceCanvasMutationResult {
    KnowledgeWorkspaceCanvasMutationResult {
        operation,
        relative_path: relative_path.as_str().to_string(),
        source_relative_path: None,
        mtime_ms: Some(file.mtime_ms),
        content_hash: Some(file.content_hash.clone()),
        audit_event_id,
    }
}

pub(crate) fn workspace_create_canvas_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    document: Value,
) -> Result<KnowledgeWorkspaceCanvasMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    require_canvas_path(&relative_path)?;
    ensure_vault_root(vault_root)?;
    validate_canvas_document(vault_root, &document, CanvasReferenceMode::RequireExisting)?;
    let body = canvas_document_as_compact_utf8(&document)?;
    let target = resolve_new_workspace_path(vault_root, &relative_path)?;
    let temporary = write_workspace_temporary_file(&target, &body)?;
    fs::rename(&temporary, &target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_create_failed: 无法原子创建 JSON Canvas。".to_string()
    })?;
    let file = read_canvas_at(vault_root, &relative_path)?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_canvas_created",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区新建 JSON Canvas。",
    )?;
    Ok(canvas_mutation_result(
        "canvas_created",
        &relative_path,
        &file,
        audit_event_id,
    ))
}

pub(crate) fn workspace_write_canvas_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    document: Value,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceCanvasMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    require_canvas_path(&relative_path)?;
    validate_canvas_document(vault_root, &document, CanvasReferenceMode::RequireExisting)?;
    let body = canvas_document_as_compact_utf8(&document)?;
    let current = read_canvas_at(vault_root, &relative_path)?;
    if current.mtime_ms != expected_mtime_ms || current.content_hash != expected_content_hash {
        return Err(
            "knowledge_vault_conflict: 这个 Canvas 已被外部来源或另一窗口修改，请先重新读取后再保存。"
                .to_string(),
        );
    }
    let target = resolve_existing_workspace_path(vault_root, &relative_path)?;
    let temporary = write_workspace_temporary_file(&target, &body)?;
    fs::rename(&temporary, &target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_replace_failed: 无法原子替换 JSON Canvas。".to_string()
    })?;
    let file = read_canvas_at(vault_root, &relative_path)?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_canvas_updated",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区更新 JSON Canvas。",
    )?;
    Ok(canvas_mutation_result(
        "canvas_updated",
        &relative_path,
        &file,
        audit_event_id,
    ))
}

#[tauri::command]
pub(crate) fn knowledge_workspace_read_canvas(
    relative_path: String,
) -> Result<KnowledgeWorkspaceCanvasDocument, String> {
    read_workspace_canvas_at(&workspace_vault_root(), &relative_path)
}

#[tauri::command]
pub(crate) fn knowledge_workspace_create_canvas(
    relative_path: String,
    document: Value,
) -> Result<KnowledgeWorkspaceCanvasMutationResult, String> {
    workspace_create_canvas_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
        document,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_write_canvas(
    relative_path: String,
    document: Value,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceCanvasMutationResult, String> {
    workspace_write_canvas_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
        document,
        expected_mtime_ms,
        &expected_content_hash,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_root(tag: &str) -> PathBuf {
        let sequence = TEST_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-knowledge-canvas-{tag}-{}-{sequence}",
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
            serde_json::to_vec(&json!({
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

    fn standard_canvas() -> Value {
        json!({
            "nodes": [
                {
                    "id": "note",
                    "type": "text",
                    "text": "A",
                    "x": 0,
                    "y": 0,
                    "width": 240,
                    "height": 80,
                    "future_text_field": { "preserve": true }
                },
                {
                    "id": "file",
                    "type": "file",
                    "file": "notes/plan.md",
                    "subpath": "#Plan",
                    "x": 280,
                    "y": 0,
                    "width": 240,
                    "height": 80
                },
                {
                    "id": "link",
                    "type": "link",
                    "url": "https://example.test/reference",
                    "x": 0,
                    "y": 120,
                    "width": 240,
                    "height": 80
                },
                {
                    "id": "group",
                    "type": "group",
                    "label": "Context",
                    "background": "attachments/paper.png",
                    "backgroundStyle": "cover",
                    "x": -16,
                    "y": -16,
                    "width": 560,
                    "height": 260
                }
            ],
            "edges": [
                {
                    "id": "note-to-file",
                    "fromNode": "note",
                    "toNode": "file",
                    "fromSide": "right",
                    "toSide": "left",
                    "toEnd": "arrow"
                }
            ],
            "future_root_field": ["preserve", 1]
        })
    }

    fn prepare_reference_files(root: &Path) {
        fs::create_dir_all(root.join("notes")).unwrap();
        fs::create_dir_all(root.join("attachments")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("notes/plan.md"), "# Plan\n").unwrap();
        fs::write(root.join("notes/linked.canvas"), "{}\n").unwrap();
        fs::write(root.join("attachments/paper.png"), [1_u8, 2, 3]).unwrap();
        fs::write(root.join("attachments/notes.txt"), "reference notes\n").unwrap();
        fs::write(root.join("attachments/nope.exe"), [4_u8, 5, 6]).unwrap();
        fs::write(root.join("assets/paper.png"), [7_u8, 8, 9]).unwrap();
    }

    #[test]
    fn n4_standard_json_canvas_fixture_starts_red() {
        let root = temp_root("fixture");
        prepare_reference_files(&root);
        fs::create_dir_all(root.join("research")).unwrap();
        fs::write(
            root.join("research/board.canvas"),
            serde_json::to_vec(&standard_canvas()).unwrap(),
        )
        .unwrap();

        let canvas = read_workspace_canvas_at(&root, "research/board.canvas")
            .expect("N4 validates a standard JSON Canvas fixture from the fixed vault");
        assert_eq!(canvas.relative_path, "research/board.canvas");
        assert_eq!(canvas.document["nodes"].as_array().unwrap().len(), 4);
        assert_eq!(canvas.document["future_root_field"][0], "preserve");
        assert_eq!(
            canvas.document["nodes"][0]["future_text_field"]["preserve"],
            true
        );
        assert!(canvas.diagnostics.is_empty());
    }

    #[test]
    fn canvas_rejects_invalid_structure_ids_edges_coordinates_and_reference_escapes() {
        let root = temp_root("invalid");
        prepare_reference_files(&root);
        let cases = [
            (
                "duplicate-node",
                json!({
                    "nodes": [
                        {"id":"a","type":"text","text":"A","x":0,"y":0,"width":1,"height":1},
                        {"id":"a","type":"text","text":"B","x":0,"y":0,"width":1,"height":1}
                    ],
                    "edges": []
                }),
            ),
            (
                "duplicate-edge",
                json!({
                    "nodes": [
                        {"id":"a","type":"text","text":"A","x":0,"y":0,"width":1,"height":1},
                        {"id":"b","type":"text","text":"B","x":0,"y":0,"width":1,"height":1}
                    ],
                    "edges": [
                        {"id":"same","fromNode":"a","toNode":"b"},
                        {"id":"same","fromNode":"a","toNode":"b"}
                    ]
                }),
            ),
            (
                "dangling",
                json!({
                    "nodes": [{"id":"a","type":"text","text":"A","x":0,"y":0,"width":1,"height":1}],
                    "edges": [{"id":"edge","fromNode":"a","toNode":"missing"}]
                }),
            ),
            (
                "decimal-coordinate",
                json!({
                    "nodes": [{"id":"a","type":"text","text":"A","x":0.5,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "bad-side-end",
                json!({
                    "nodes": [
                        {"id":"a","type":"text","text":"A","x":0,"y":0,"width":1,"height":1},
                        {"id":"b","type":"text","text":"B","x":0,"y":0,"width":1,"height":1}
                    ],
                    "edges": [{"id":"edge","fromNode":"a","toNode":"b","fromSide":"north","toEnd":"run"}]
                }),
            ),
            (
                "outside-reference",
                json!({
                    "nodes": [{"id":"a","type":"file","file":"../secret.md","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "bad-file-subpath",
                json!({
                    "nodes": [{"id":"a","type":"file","file":"notes/plan.md","subpath":"heading","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "bad-group-background-style",
                json!({
                    "nodes": [{"id":"a","type":"group","backgroundStyle":"stretch","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "disallowed-executable-attachment",
                json!({
                    "nodes": [{"id":"a","type":"file","file":"attachments/nope.exe","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "outside-attachment-background",
                json!({
                    "nodes": [{"id":"a","type":"group","background":"assets/paper.png","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
            (
                "non-raster-group-background",
                json!({
                    "nodes": [{"id":"a","type":"group","background":"attachments/notes.txt","x":0,"y":0,"width":1,"height":1}],
                    "edges": []
                }),
            ),
        ];
        for (name, document) in cases {
            let error =
                validate_canvas_document(&root, &document, CanvasReferenceMode::RequireExisting)
                    .unwrap_err();
            assert!(
                error.starts_with("knowledge_workspace_canvas_"),
                "{name}: {error}"
            );
        }
    }

    #[test]
    fn canvas_allows_omitted_top_level_nodes_and_edges_but_rejects_non_arrays() {
        let root = temp_root("optional-arrays");
        let state = workflow_state_path(&root);
        for document in [json!({}), json!({"nodes": []}), json!({"edges": []})] {
            assert!(validate_canvas_document(
                &root,
                &document,
                CanvasReferenceMode::RequireExisting,
            )
            .is_ok());
        }
        let created = workspace_create_canvas_at(&root, &state, "empty.canvas", json!({})).unwrap();
        assert_eq!(created.operation, "canvas_created");
        let loaded = read_workspace_canvas_at(&root, "empty.canvas").unwrap();
        assert_eq!(loaded.document, json!({}));
        assert!(loaded.diagnostics.is_empty());

        for document in [json!({"nodes": {}}), json!({"edges": "not-an-array"})] {
            let error =
                validate_canvas_document(&root, &document, CanvasReferenceMode::RequireExisting)
                    .unwrap_err();
            assert!(error.starts_with("knowledge_workspace_canvas_invalid_structure:"));
        }
    }

    #[test]
    fn canvas_rejects_oversize_invalid_utf8_and_bad_json_before_any_write() {
        let root = temp_root("file-bounds");
        fs::write(
            root.join("too-large.canvas"),
            vec![b' '; MAX_CANVAS_BYTES as usize + 1],
        )
        .unwrap();
        assert!(read_workspace_canvas_at(&root, "too-large.canvas")
            .unwrap_err()
            .starts_with("knowledge_workspace_canvas_too_large:"));

        fs::write(root.join("invalid-utf8.canvas"), [0xff_u8]).unwrap();
        assert!(read_workspace_canvas_at(&root, "invalid-utf8.canvas")
            .unwrap_err()
            .starts_with("knowledge_workspace_invalid_utf8:"));

        fs::write(root.join("invalid-json.canvas"), "{").unwrap();
        assert!(read_workspace_canvas_at(&root, "invalid-json.canvas")
            .unwrap_err()
            .starts_with("knowledge_workspace_canvas_invalid_json:"));
    }

    #[test]
    fn canvas_reference_policy_allows_only_supported_files_and_bounded_attachments() {
        let root = temp_root("reference-policy");
        prepare_reference_files(&root);
        for reference in [
            "notes/plan.md",
            "notes/linked.canvas",
            "attachments/paper.png",
            "attachments/notes.txt",
        ] {
            let document = json!({
                "nodes": [{"id":"file","type":"file","file":reference,"x":0,"y":0,"width":1,"height":1}],
                "edges": []
            });
            assert!(validate_canvas_document(
                &root,
                &document,
                CanvasReferenceMode::RequireExisting,
            )
            .is_ok());
        }

        let oversized = root.join("attachments/oversized.png");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_ATTACHMENT_BYTES + 1)
            .unwrap();
        let document = json!({
            "nodes": [{"id":"file","type":"file","file":"attachments/oversized.png","x":0,"y":0,"width":1,"height":1}],
            "edges": []
        });
        assert!(
            validate_canvas_document(&root, &document, CanvasReferenceMode::RequireExisting,)
                .unwrap_err()
                .starts_with("knowledge_workspace_canvas_invalid_reference:")
        );
    }

    #[test]
    fn canvas_read_reports_missing_references_but_write_requires_them() {
        let root = temp_root("missing-reference");
        fs::create_dir_all(root.join("research")).unwrap();
        let state = workflow_state_path(&root);
        let document = json!({
            "nodes": [{
                "id":"file",
                "type":"file",
                "file":"attachments/future.png",
                "x":0,
                "y":0,
                "width":10,
                "height":10
            }],
            "edges": []
        });
        fs::write(
            root.join("research/missing.canvas"),
            serde_json::to_vec(&document).unwrap(),
        )
        .unwrap();

        let loaded = read_workspace_canvas_at(&root, "research/missing.canvas").unwrap();
        assert_eq!(loaded.diagnostics.len(), 1);
        assert_eq!(
            loaded.diagnostics[0].code,
            "knowledge_workspace_canvas_missing_reference"
        );
        let error = workspace_write_canvas_at(
            &root,
            &state,
            "research/missing.canvas",
            document,
            loaded.mtime_ms,
            &loaded.content_hash,
        )
        .unwrap_err();
        assert!(error.starts_with("knowledge_workspace_canvas_invalid_reference:"));
    }

    #[test]
    fn canvas_create_write_are_atomic_cas_checked_audited_and_preserve_unknown_fields() {
        let root = temp_root("write");
        prepare_reference_files(&root);
        let state = workflow_state_path(&root);
        let created =
            workspace_create_canvas_at(&root, &state, "board.canvas", standard_canvas()).unwrap();
        assert_eq!(created.operation, "canvas_created");
        assert_eq!(created.relative_path, "board.canvas");
        assert!(created.content_hash.as_deref().is_some());
        assert!(
            workspace_create_canvas_at(&root, &state, "board.canvas", standard_canvas()).is_err()
        );

        let before = read_workspace_canvas_at(&root, "board.canvas").unwrap();
        let mut edited = before.document.clone();
        edited["nodes"][0]["text"] = Value::String("Edited".to_string());
        edited["nodes"][0]["future_text_field"] = json!({"preserve": "still-here"});
        let updated = workspace_write_canvas_at(
            &root,
            &state,
            "board.canvas",
            edited,
            before.mtime_ms,
            &before.content_hash,
        )
        .unwrap();
        assert_eq!(updated.operation, "canvas_updated");
        let after = read_workspace_canvas_at(&root, "board.canvas").unwrap();
        assert_eq!(after.document["nodes"][0]["text"], "Edited");
        assert_eq!(
            after.document["nodes"][0]["future_text_field"]["preserve"],
            "still-here"
        );

        fs::write(
            root.join("board.canvas"),
            serde_json::to_vec(&standard_canvas()).unwrap(),
        )
        .unwrap();
        let stale = workspace_write_canvas_at(
            &root,
            &state,
            "board.canvas",
            standard_canvas(),
            after.mtime_ms,
            &after.content_hash,
        )
        .unwrap_err();
        assert!(stale.starts_with("knowledge_vault_conflict:"));
        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        assert_eq!(events.len(), 2, "stale Canvas write must not append audit");
        assert_eq!(
            events[0]["event_type"],
            "knowledge_workspace_canvas_created"
        );
        assert_eq!(
            events[1]["event_type"],
            "knowledge_workspace_canvas_updated"
        );
    }

    #[cfg(unix)]
    #[test]
    fn canvas_rejects_file_and_group_references_to_symlinks_or_directories() {
        let root = temp_root("symlink-directory");
        let outside = temp_root("outside");
        fs::create_dir_all(root.join("attachments")).unwrap();
        fs::create_dir_all(root.join("attachments/directory-ref.png")).unwrap();
        fs::write(outside.join("secret.txt"), "secret").unwrap();
        std::os::unix::fs::symlink(
            outside.join("secret.txt"),
            root.join("attachments/link.png"),
        )
        .unwrap();
        let symlink_document = json!({
            "nodes": [{"id":"file","type":"file","file":"attachments/link.png","x":0,"y":0,"width":1,"height":1}],
            "edges": []
        });
        let directory_document = json!({
            "nodes": [{"id":"group","type":"group","background":"attachments/directory-ref.png","x":0,"y":0,"width":1,"height":1}],
            "edges": []
        });
        for document in [symlink_document, directory_document] {
            let error = validate_canvas_document(
                &root,
                &document,
                CanvasReferenceMode::ReadWithMissingDiagnostics,
            )
            .unwrap_err();
            assert!(error.starts_with("knowledge_workspace_canvas_invalid_reference:"));
        }
    }
}
