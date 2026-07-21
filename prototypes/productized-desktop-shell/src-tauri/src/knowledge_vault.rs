// L3 知识库第一片：工作台自管 vault（md 文件即真相）。
// 边界死锁 `~/Library/Application Support/CodexGovernanceWorkbench/knowledge-vault/`（lib.rs:1327 先例）：
// 不读不写用户任何既有目录；slug 组件锁 + `..`/绝对路径/符号链接三例拒绝（单元测试锁死）。
// AI 写入只经 `knowledge_vault_ai_write`（前端 PermissionDialog 用户允许那一下才调），
// actor_ref=`ai_proposed_user_confirmed`、source_summary 必填；无常驻授权、无自动沉淀、agent 零直写通道。
// 写操作落 workflow-state audit：knowledge_vault_note_created / knowledge_vault_note_user_edited /
// knowledge_vault_note_ai_written 三事件（前后端词表同步）。
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Component, Path, PathBuf};

const VAULT_DIR_NAME: &str = "knowledge-vault";
const WORKFLOW_STATE_RELATIVE: &str = "workflow-state/workflow-state.v0.json";

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeVaultNoteSummary {
    slug: String,
    title: String,
    mtime_ms: i64,
    outlinks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeVaultNote {
    slug: String,
    title: String,
    body: String,
    mtime_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeVaultWriteResult {
    slug: String,
    title: String,
    audit_event_id: String,
    created: bool,
}

fn app_data_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/yoyi".to_string());
    PathBuf::from(home).join("Library/Application Support/CodexGovernanceWorkbench")
}

fn default_vault_root() -> PathBuf {
    app_data_root().join(VAULT_DIR_NAME)
}

fn default_workflow_state_path() -> PathBuf {
    app_data_root().join(WORKFLOW_STATE_RELATIVE)
}

// 文件名=标题 slug：CJK 保留·空白→`-`·剔 `/\:*?"<>|` 与控制字符·空则 `untitled`。
pub(crate) fn slugify_title(title: &str) -> String {
    let mut slug = String::new();
    for character in title.trim().chars() {
        if character.is_control() || matches!(character, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            continue;
        }
        if character.is_whitespace() {
            if !slug.is_empty() && !slug.ends_with('-') {
                slug.push('-');
            }
            continue;
        }
        slug.push(character);
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

// 路径锁：slug 必须是单个 Normal 组件（拒 `..`/绝对路径/分隔符），且非 `.` 开头隐藏名。
fn is_safe_slug(slug: &str) -> bool {
    if slug.is_empty() || slug.starts_with('.') {
        return false;
    }
    let mut components = Path::new(slug).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn note_path(vault_root: &Path, slug: &str) -> Result<PathBuf, String> {
    if !is_safe_slug(slug) {
        return Err(format!("非法笔记名「{slug}」：只允许 vault 内单层文件名"));
    }
    let path = vault_root.join(format!("{slug}.md"));
    // 符号链接逃逸锁：既有路径（含 vault 根与笔记文件）是 symlink 一律拒读写。
    if let Ok(metadata) = fs::symlink_metadata(&path) {
        if metadata.file_type().is_symlink() {
            return Err(format!("拒绝符号链接路径：{}", path.display()));
        }
    }
    Ok(path)
}

fn ensure_vault_root(vault_root: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(vault_root) {
        if metadata.file_type().is_symlink() {
            return Err(format!("拒绝符号链接 vault 根：{}", vault_root.display()));
        }
    }
    fs::create_dir_all(vault_root).map_err(|error| format!("创建 vault 目录失败：{error}"))
}

fn dedupe_slug(vault_root: &Path, title: &str) -> String {
    let base = slugify_title(title);
    if !vault_root.join(format!("{base}.md")).exists() {
        return base;
    }
    for suffix in 2.. {
        let candidate = format!("{base}-{suffix}");
        if !vault_root.join(format!("{candidate}.md")).exists() {
            return candidate;
        }
    }
    unreachable!()
}

fn title_from_body(slug: &str, body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let title = heading.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    slug.to_string()
}

fn extract_outlinks(body: &str) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("]]") {
            let title = rest[..end].trim();
            if !title.is_empty() && !links.iter().any(|item| item == title) {
                links.push(title.to_string());
            }
            rest = &rest[end + 2..];
        } else {
            break;
        }
    }
    links
}

fn mtime_ms_of(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn unix_timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn list_notes_at(vault_root: &Path) -> Result<Vec<KnowledgeVaultNoteSummary>, String> {
    ensure_vault_root(vault_root)?;
    let mut notes = Vec::new();
    let entries = fs::read_dir(vault_root).map_err(|error| format!("读取 vault 目录失败：{error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取 vault 条目失败：{error}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| format!("读取笔记状态失败：{error}"))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }
        let slug = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string();
        if !is_safe_slug(&slug) {
            continue;
        }
        let body = fs::read_to_string(&path).unwrap_or_default();
        notes.push(KnowledgeVaultNoteSummary {
            title: title_from_body(&slug, &body),
            slug,
            mtime_ms: mtime_ms_of(&metadata),
            outlinks: extract_outlinks(&body),
        });
    }
    notes.sort_by(|a, b| b.mtime_ms.cmp(&a.mtime_ms).then_with(|| a.slug.cmp(&b.slug)));
    Ok(notes)
}

fn read_note_at(vault_root: &Path, slug: &str) -> Result<KnowledgeVaultNote, String> {
    let path = note_path(vault_root, slug)?;
    let body = fs::read_to_string(&path).map_err(|error| format!("笔记「{slug}」读取失败：{error}"))?;
    let metadata = fs::metadata(&path).map_err(|error| format!("笔记「{slug}」状态读取失败：{error}"))?;
    Ok(KnowledgeVaultNote {
        title: title_from_body(slug, &body),
        slug: slug.to_string(),
        body,
        mtime_ms: mtime_ms_of(&metadata),
    })
}

fn append_audit_event(
    workflow_state_path: &Path,
    event_type: &str,
    target_ref: &str,
    actor_ref: &str,
    reason: &str,
) -> Result<String, String> {
    let timestamp_ms = unix_timestamp_ms();
    let audit_event_id =
        crate::workflow_audit::audit_event_identity("knowledge-vault", target_ref, timestamp_ms);
    let mut value = if workflow_state_path.exists() {
        crate::workflow_state_store::read_value(workflow_state_path)
            .unwrap_or_else(|_| json!({ "audit_events": [] }))
    } else {
        json!({ "audit_events": [] })
    };
    let events = value
        .get_mut("audit_events")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "workflow-state audit_events 形状异常".to_string())?;
    events.push(json!({
        "event_id": audit_event_id,
        "event_type": event_type,
        "target_ref": target_ref,
        "actor_ref": actor_ref,
        "source_kind": "knowledge_vault",
        "permission_level": "user_confirmed_write",
        "created_at": timestamp_ms.to_string(),
        "reason": reason,
    }));
    if let Some(parent) = workflow_state_path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建 workflow-state 目录失败：{error}"))?;
    }
    if workflow_state_path.exists() {
        let _ = crate::workflow_state_store::backup_file(workflow_state_path, &timestamp_ms.to_string());
    }
    crate::workflow_state_store::atomic_write(workflow_state_path, &value, &timestamp_ms.to_string())?;
    Ok(audit_event_id)
}

fn create_note_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    title: &str,
) -> Result<KnowledgeVaultWriteResult, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("笔记标题不能为空".to_string());
    }
    ensure_vault_root(vault_root)?;
    let slug = dedupe_slug(vault_root, title);
    let path = note_path(vault_root, &slug)?;
    fs::write(&path, format!("# {title}\n")).map_err(|error| format!("笔记写入失败：{error}"))?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_vault_note_created",
        &slug,
        "user_manual_edit",
        "用户手动新建知识库笔记。",
    )?;
    Ok(KnowledgeVaultWriteResult {
        slug,
        title: title.to_string(),
        audit_event_id,
        created: true,
    })
}

fn write_note_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    slug: &str,
    body: &str,
) -> Result<KnowledgeVaultWriteResult, String> {
    let path = note_path(vault_root, slug)?;
    if !path.exists() {
        return Err(format!("笔记「{slug}」不存在"));
    }
    fs::write(&path, body).map_err(|error| format!("笔记写入失败：{error}"))?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_vault_note_user_edited",
        slug,
        "user_manual_edit",
        "用户手动编辑知识库笔记（整文回写）。",
    )?;
    Ok(KnowledgeVaultWriteResult {
        slug: slug.to_string(),
        title: title_from_body(slug, body),
        audit_event_id,
        created: false,
    })}

// AI 写入闸后端面：只允许「用户允许那一下」之后调用；actor/source 进审计，source_summary 必填。
fn ai_write_note_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    title: &str,
    body: &str,
    source_summary: &str,
) -> Result<KnowledgeVaultWriteResult, String> {
    let title = title.trim();
    let source_summary = source_summary.trim();
    if title.is_empty() {
        return Err("笔记标题不能为空".to_string());
    }
    if source_summary.is_empty() {
        return Err("AI 写入必须带来源说明（source_summary）".to_string());
    }
    ensure_vault_root(vault_root)?;
    let slug = dedupe_slug(vault_root, title);
    let path = note_path(vault_root, &slug)?;
    fs::write(&path, body).map_err(|error| format!("笔记写入失败：{error}"))?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_vault_note_ai_written",
        &slug,
        "ai_proposed_user_confirmed",
        &format!("AI 提议、用户确认后写入知识库笔记；来源：{source_summary}"),
    )?;
    Ok(KnowledgeVaultWriteResult {
        title: title_from_body(&slug, body),
        slug,
        audit_event_id,
        created: true,
    })
}

#[tauri::command]
pub(crate) fn knowledge_vault_list_notes() -> Result<Vec<KnowledgeVaultNoteSummary>, String> {
    list_notes_at(&default_vault_root())
}

#[tauri::command]
pub(crate) fn knowledge_vault_read_note(slug: String) -> Result<KnowledgeVaultNote, String> {
    read_note_at(&default_vault_root(), &slug)
}

#[tauri::command]
pub(crate) fn knowledge_vault_create_note(title: String) -> Result<KnowledgeVaultWriteResult, String> {
    create_note_at(&default_vault_root(), &default_workflow_state_path(), &title)
}

#[tauri::command]
pub(crate) fn knowledge_vault_write_note(slug: String, body: String) -> Result<KnowledgeVaultWriteResult, String> {
    write_note_at(&default_vault_root(), &default_workflow_state_path(), &slug, &body)
}

#[tauri::command]
pub(crate) fn knowledge_vault_ai_write(
    note_title: String,
    body: String,
    source_summary: String,
) -> Result<KnowledgeVaultWriteResult, String> {
    ai_write_note_at(
        &default_vault_root(),
        &default_workflow_state_path(),
        &note_title,
        &body,
        &source_summary,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "knowledge-vault-test-{tag}-{}-{}",
            std::process::id(),
            unix_timestamp_ms()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn slugify_keeps_cjk_and_strips_forbidden() {
        assert_eq!(slugify_title("我的 第一条 笔记"), "我的-第一条-笔记");
        assert_eq!(slugify_title("a/b\\c:d*e?f\"g<h>i|j"), "abcdefghij");
        assert_eq!(slugify_title("   "), "untitled");
        assert_eq!(slugify_title(""), "untitled");
    }

    #[test]
    fn path_lock_rejects_parent_dir() {
        let root = temp_root("parent");
        let result = write_note_at(&root, &root.join("state.json"), "../escape", "x");
        assert!(result.is_err(), ".. 组件必须拒绝");
        assert!(!root.join("escape.md").exists());
    }

    #[test]
    fn path_lock_rejects_absolute_path() {
        let root = temp_root("absolute");
        let result = read_note_at(&root, "/etc/passwd");
        assert!(result.is_err(), "绝对路径必须拒绝");
    }

    #[test]
    fn path_lock_rejects_symlink_escape() {
        let root = temp_root("symlink");
        let outside = temp_root("symlink-outside");
        let outside_file = outside.join("secret.md");
        fs::write(&outside_file, "secret").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside_file, root.join("link.md")).unwrap();
            let result = read_note_at(&root, "link");
            assert!(result.is_err(), "符号链接必须拒绝");
            let write_result = write_note_at(&root, &root.join("state.json"), "link", "x");
            assert!(write_result.is_err(), "符号链接写入必须拒绝");
        }
    }

    #[test]
    fn create_list_read_edit_roundtrip() {
        let root = temp_root("roundtrip");
        let state = root.join("state.json");
        let created = create_note_at(&root, &state, "我的 笔记").unwrap();
        assert_eq!(created.slug, "我的-笔记");
        let duplicate = create_note_at(&root, &state, "我的 笔记").unwrap();
        assert_eq!(duplicate.slug, "我的-笔记-2", "重名追加 -2");

        let notes = list_notes_at(&root).unwrap();
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().all(|note| note.title == "我的 笔记"));

        let edited = write_note_at(&root, &state, "我的-笔记", "# 我的 笔记\n\n正文 [[另一条]]。\n").unwrap();
        assert_eq!(edited.title, "我的 笔记");
        let note = read_note_at(&root, "我的-笔记").unwrap();
        assert!(note.body.contains("正文"));
        assert_eq!(note.mtime_ms > 0, true);
        let listed = list_notes_at(&root).unwrap();
        assert_eq!(listed[0].outlinks, vec!["另一条".to_string()]);

        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        let event_types: Vec<&str> = events.iter().filter_map(|event| event["event_type"].as_str()).collect();
        assert_eq!(
            event_types,
            vec![
                "knowledge_vault_note_created",
                "knowledge_vault_note_created",
                "knowledge_vault_note_user_edited"
            ]
        );
        assert!(events.iter().all(|event| event["actor_ref"] == "user_manual_edit"));
    }

    #[test]
    fn ai_write_requires_source_and_audits_actor() {
        let root = temp_root("ai-write");
        let state = root.join("state.json");
        let rejected = ai_write_note_at(&root, &state, "标题", "正文", "  ");
        assert!(rejected.is_err(), "source_summary 空必须拒绝");
        assert!(!root.join("标题.md").exists(), "拒绝时不得落文件");

        let written = ai_write_note_at(&root, &state, "AI 笔记", "# AI 笔记\n\n内容", "记忆候选 memcand:v1:x").unwrap();
        assert_eq!(written.slug, "AI-笔记");
        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "knowledge_vault_note_ai_written");
        assert_eq!(events[0]["actor_ref"], "ai_proposed_user_confirmed");
        assert!(events[0]["reason"].as_str().unwrap().contains("记忆候选 memcand:v1:x"));
    }
}
