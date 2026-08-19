// L3 知识库第一片：工作台自管 vault（md 文件即真相）。
// 边界死锁 `~/Library/Application Support/CodexGovernanceWorkbench/knowledge-vault/`（lib.rs:1327 先例）：
// 不读不写用户任何既有目录；slug 组件锁 + `..`/绝对路径/符号链接三例拒绝（单元测试锁死）。
// AI 写入只经 `knowledge_vault_ai_write`（前端 PermissionDialog 用户允许那一下才调），
// actor_ref=`ai_proposed_user_confirmed`、source_summary 必填；无常驻授权、无自动沉淀、agent 零直写通道。
// 写操作落 workflow-state audit：knowledge_vault_note_created / knowledge_vault_note_user_edited /
// knowledge_vault_note_ai_written 三事件（前后端词表同步）。
use serde::Serialize;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "knowledge_canvas.rs"]
pub(crate) mod knowledge_canvas;

#[path = "knowledge_attachments.rs"]
pub(crate) mod knowledge_attachments;

#[path = "knowledge_recovery.rs"]
pub(crate) mod knowledge_recovery;

const VAULT_DIR_NAME: &str = "knowledge-vault";
const ATTACHMENTS_DIR_NAME: &str = "attachments";
const RECOVERY_BACKUPS_DIR_NAME: &str = "knowledge-workspace-recovery";
const WORKFLOW_STATE_RELATIVE: &str = "workflow-state/workflow-state.v0.json";
pub(crate) const MAX_MARKDOWN_BYTES: u64 = 64 * 1024;
pub(crate) const MAX_CANVAS_BYTES: u64 = 256 * 1024;
pub(crate) const MAX_ATTACHMENT_BYTES: u64 = 10 * 1024 * 1024;
const MAX_WORKSPACE_RELATIVE_PATH_CHARS: usize = 512;
const MAX_WORKSPACE_PATH_SEGMENTS: usize = 32;
const MAX_WORKSPACE_SEGMENT_CHARS: usize = 128;
static WORKSPACE_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
    content_hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeVaultWriteResult {
    slug: String,
    title: String,
    audit_event_id: String,
    created: bool,
}

/// N1 的嵌套路径宿主类型。只有这个类型可以进入工作区新命令的 fixed-vault resolver；
/// 旧的 `slug` 命令继续保持单层合同，绝不把嵌套路径塞回旧字段。
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ValidatedVaultRelativePath(String);

impl ValidatedVaultRelativePath {
    pub(crate) fn parse(raw: &str) -> Result<Self, String> {
        if raw.is_empty()
            || raw.len() > MAX_WORKSPACE_RELATIVE_PATH_CHARS
            || raw.contains('\\')
            || Path::new(raw).is_absolute()
        {
            return Err(
                "knowledge_workspace_invalid_path: 路径必须是固定 vault 内受限相对路径。"
                    .to_string(),
            );
        }

        let segments: Vec<&str> = raw.split('/').collect();
        if segments.is_empty() || segments.len() > MAX_WORKSPACE_PATH_SEGMENTS {
            return Err(
                "knowledge_workspace_invalid_path: 路径层级超过受限工作区上限。".to_string(),
            );
        }
        for segment in &segments {
            if !is_safe_workspace_path_segment(segment) {
                return Err("knowledge_workspace_invalid_path: 路径含有不安全组件。".to_string());
            }
        }
        Ok(Self(raw.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or_default()
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_string()))
    }

    fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

/// 供 N1 红/绿测试和后续 host command 共用的显式路径入口。
pub(crate) fn validate_workspace_relative_path(
    raw: &str,
) -> Result<ValidatedVaultRelativePath, String> {
    ValidatedVaultRelativePath::parse(raw)
}

/// N5 的附件类型是固定、最小的交集。导入、索引和 Canvas 都必须复用这一个枚举；
/// MIME 仅交叉校验浏览器 File 的声明，不能把任意类型放进固定 vault。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceAttachmentKind {
    Png,
    Jpg,
    Jpeg,
    Gif,
    Webp,
    Pdf,
    Txt,
    Csv,
}

impl WorkspaceAttachmentKind {
    fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "png" => Some(Self::Png),
            "jpg" => Some(Self::Jpg),
            "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::Webp),
            "pdf" => Some(Self::Pdf),
            "txt" => Some(Self::Txt),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }

    pub(crate) fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpg | Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Pdf => "application/pdf",
            Self::Txt => "text/plain",
            Self::Csv => "text/csv",
        }
    }

    pub(crate) fn is_raster(self) -> bool {
        matches!(
            self,
            Self::Png | Self::Jpg | Self::Jpeg | Self::Gif | Self::Webp
        )
    }
}

pub(crate) fn workspace_attachment_kind_for_relative_path(
    relative_path: &ValidatedVaultRelativePath,
) -> Option<WorkspaceAttachmentKind> {
    if !relative_path.as_str().starts_with("attachments/") {
        return None;
    }
    let (stem, extension) = relative_path.file_name().rsplit_once('.')?;
    (!stem.is_empty())
        .then(|| WorkspaceAttachmentKind::from_extension(extension))
        .flatten()
}

pub(crate) fn workspace_attachment_kind_for_import(
    display_name: &str,
    mime_type: &str,
) -> Result<(ValidatedVaultRelativePath, WorkspaceAttachmentKind), String> {
    if display_name.is_empty()
        || display_name.contains('/')
        || display_name.contains('\\')
        || !is_safe_workspace_path_segment(display_name)
    {
        return Err(
            "knowledge_workspace_attachment_invalid_display_name: 附件显示名必须是单个安全文件名。"
                .to_string(),
        );
    }
    let relative_path = validate_workspace_relative_path(&format!(
        "{ATTACHMENTS_DIR_NAME}/{display_name}"
    ))
    .map_err(|_| {
        "knowledge_workspace_attachment_invalid_display_name: 附件显示名必须是单个安全文件名。"
            .to_string()
    })?;
    let kind = workspace_attachment_kind_for_relative_path(&relative_path).ok_or_else(|| {
        "knowledge_workspace_attachment_type_not_allowed: 附件扩展名不在本阶段允许范围。"
            .to_string()
    })?;
    if mime_type != kind.mime_type() {
        return Err(
            "knowledge_workspace_attachment_invalid_mime_type: 附件 MIME 必须与允许扩展名精确匹配。"
                .to_string(),
        );
    }
    Ok((relative_path, kind))
}

pub(crate) fn require_workspace_attachment_path(
    relative_path: &ValidatedVaultRelativePath,
) -> Result<WorkspaceAttachmentKind, String> {
    workspace_attachment_kind_for_relative_path(relative_path).ok_or_else(|| {
        "knowledge_workspace_attachment_only: 此操作只允许固定 vault attachments/ 内的允许附件。"
            .to_string()
    })
}

fn is_safe_workspace_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && !segment.starts_with('.')
        && !segment.starts_with('-')
        && !segment.contains("--")
        && segment.chars().count() <= MAX_WORKSPACE_SEGMENT_CHARS
        && !segment.chars().any(char::is_control)
        && !segment.contains([
            '/', '\\', ':', '*', '?', '[', ']', '{', '}', '\'', '"', '=', '|', '<', '>',
        ])
}

fn require_markdown_path(relative_path: &ValidatedVaultRelativePath) -> Result<(), String> {
    if !relative_path.file_name().ends_with(".md") || relative_path.file_name().len() <= ".md".len()
    {
        return Err(
            "knowledge_workspace_markdown_only: 此操作只允许固定 vault 内的 .md 文件。".to_string(),
        );
    }
    Ok(())
}

fn require_markdown_body_size(body: &str) -> Result<(), String> {
    if body.len() as u64 > MAX_MARKDOWN_BYTES {
        return Err(
            "knowledge_workspace_markdown_too_large: Markdown 超过 64 KiB 安全上限。".to_string(),
        );
    }
    Ok(())
}

fn app_data_root() -> PathBuf {
    if let Some(paths) = crate::acceptance_runtime_profile::active_paths()
        .expect("acceptance runtime profile must resolve before knowledge app-data use")
    {
        return paths.app_data_root;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/yoyi".to_string());
    PathBuf::from(home).join("Library/Application Support/CodexGovernanceWorkbench")
}

fn default_vault_root() -> PathBuf {
    app_data_root().join(VAULT_DIR_NAME)
}

// Obsidian/MCP 复用同一真相源时只拿到这个固定根；调用方不得接受外部 vault 路径。
pub(crate) fn syn_vault_root() -> PathBuf {
    default_vault_root()
}

// 给外部集成的只读路径锁：不创建目录、不遍历条目；根不存在或是符号链接时直接闭锁。
pub(crate) fn syn_vault_root_for_external_integration() -> Result<PathBuf, String> {
    syn_vault_root_for_external_integration_at(&app_data_root())
}

fn syn_vault_root_for_external_integration_at(app_data_root: &Path) -> Result<PathBuf, String> {
    let vault_root = app_data_root.join(VAULT_DIR_NAME);
    verify_existing_fixed_vault_root_at(app_data_root, &vault_root).map_err(|error| {
        if error.starts_with("knowledge_workspace_vault_invalid:") {
            "Syn 自管 vault 尚未初始化；请先在 Syn 新建一条笔记。".to_string()
        } else {
            "Syn 自管 vault 路径异常，已拒绝外部集成访问。".to_string()
        }
    })?;
    vault_root
        .canonicalize()
        .map_err(|_| "Syn 自管 vault 无法解析，已拒绝外部集成访问。".to_string())
}

// 外部 bridge 只接受现有知识库的单层 Markdown slug，额外拒绝 option-looking 名称和控制参数。
pub(crate) fn syn_note_relative_path(slug: &str) -> Result<String, String> {
    if !is_safe_slug(slug)
        || slug.starts_with('-')
        || slug.contains("--")
        || slug.contains('\n')
        || slug.contains('\r')
        || slug.contains('\0')
        || slug.contains('\'')
        || slug.contains('"')
        || slug.contains(['*', '?', '[', ']', '{', '}', '\\'])
        || slug.contains('=')
    {
        return Err("非法笔记名：只允许 Syn vault 内安全的单层 slug。".to_string());
    }
    Ok(format!("{slug}.md"))
}

/// 新工作区和索引都只从此固定根取路径；不向前端、MCP 或外部 bridge 接受根路径。
pub(crate) fn workspace_vault_root() -> PathBuf {
    default_vault_root()
}

pub(crate) fn workspace_workflow_state_path() -> PathBuf {
    default_workflow_state_path()
}

fn recovery_backups_root() -> PathBuf {
    app_data_root().join(RECOVERY_BACKUPS_DIR_NAME)
}

/// 备份目录与 vault 分离，但仍是本应用固定 app-data 下的受控目录；它不属于知识文件根。
/// 读取路径不创建任何目录，避免查询/刷新带来隐藏写入。
pub(crate) fn workspace_recovery_backups_root_for_read() -> Result<Option<PathBuf>, String> {
    let app_data = app_data_root();
    require_fixed_vault_ancestors(&app_data)?;
    match fs::symlink_metadata(&app_data) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(
                "knowledge_workspace_recovery_invalid: 固定 Syn app-data 目录不可读取。"
                    .to_string(),
            )
        }
        Ok(_) => require_fixed_directory(&app_data, "app-data 目录")?,
    }
    let recovery_root = recovery_backups_root();
    match fs::symlink_metadata(&recovery_root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => {
            Err("knowledge_workspace_recovery_invalid: 固定 Syn 恢复备份目录不可读取。".to_string())
        }
        Ok(_) => {
            require_fixed_directory(&recovery_root, "恢复备份目录")?;
            Ok(Some(recovery_root))
        }
    }
}

pub(crate) fn ensure_workspace_recovery_backups_root() -> Result<PathBuf, String> {
    let app_data = app_data_root();
    require_fixed_vault_ancestors(&app_data)?;
    let application_support = app_data.parent().ok_or_else(|| {
        "knowledge_workspace_recovery_invalid: 固定 Syn app-data 父目录缺失。".to_string()
    })?;
    ensure_fixed_directory_child(application_support, &app_data, "app-data 目录")?;
    let recovery_root = recovery_backups_root();
    ensure_fixed_directory_child(&app_data, &recovery_root, "恢复备份目录")?;
    Ok(recovery_root)
}

fn require_existing_vault_root(vault_root: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(vault_root).map_err(|_| {
        "knowledge_workspace_vault_uninitialized: Syn 自管 vault 尚未初始化。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(
            "knowledge_workspace_vault_invalid: Syn 自管 vault 根必须是普通目录。".to_string(),
        );
    }
    Ok(())
}

/// 指定目录中只接受目录枚举所得的精确字面条目。macOS 的大小写折叠不能替调用方
/// 静默改写授权字符串；发现 ASCII case variant 时直接闭锁。
fn exact_child_path(parent: &Path, expected_name: &str) -> Result<PathBuf, String> {
    let mut case_variant_found = false;
    let entries = fs::read_dir(parent).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取固定 vault 目录。".to_string()
    })?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            "knowledge_workspace_path_unreadable: 无法枚举固定 vault 目录条目。".to_string()
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name == expected_name {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|_| {
                "knowledge_workspace_path_unreadable: 无法读取固定 vault 条目状态。".to_string()
            })?;
            if metadata.file_type().is_symlink() {
                return Err(
                    "knowledge_workspace_symlink_rejected: 固定 vault 中不允许符号链接。"
                        .to_string(),
                );
            }
            return Ok(path);
        }
        if name.to_lowercase() == expected_name.to_lowercase() {
            case_variant_found = true;
        }
    }
    if case_variant_found {
        return Err(
            "knowledge_workspace_case_mismatch: 路径大小写必须与 vault 条目完全一致。".to_string(),
        );
    }
    Err("knowledge_workspace_entry_not_found: 固定 vault 中不存在该条目。".to_string())
}

/// 每一段都由 `read_dir` 精确解析，同时拒绝根、祖先与目标的 symlink。
/// 这提供每次 host 操作前的 fail-closed 边界；恶意并发替换不属于 N1 的平台级
/// `openat/O_NOFOLLOW` 防御范围，因此调用方不得把它表述为 TOCTOU 全面防御。
pub(crate) fn resolve_existing_workspace_path(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<PathBuf, String> {
    require_existing_vault_root(vault_root)?;
    let mut current = vault_root.to_path_buf();
    let segment_count = relative_path.segments().count();
    for (index, segment) in relative_path.segments().enumerate() {
        let next = exact_child_path(&current, segment)?;
        let metadata = fs::symlink_metadata(&next).map_err(|_| {
            "knowledge_workspace_path_unreadable: 无法读取固定 vault 条目状态。".to_string()
        })?;
        if metadata.file_type().is_symlink() {
            return Err(
                "knowledge_workspace_symlink_rejected: 固定 vault 中不允许符号链接。".to_string(),
            );
        }
        if index + 1 < segment_count && !metadata.is_dir() {
            return Err(
                "knowledge_workspace_invalid_ancestor: 路径祖先必须是普通目录。".to_string(),
            );
        }
        current = next;
    }
    Ok(current)
}

/// 创建目标必须已有受检验的父目录，且 leaf 不得存在（含大小写变体）。不做隐式
/// `create_dir_all`，避免以“新建笔记”为由扩张固定 vault 以外的写面。
pub(crate) fn resolve_new_workspace_path(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<PathBuf, String> {
    require_existing_vault_root(vault_root)?;
    let parent = match relative_path.parent() {
        Some(parent) => {
            let parent_path = resolve_existing_workspace_path(vault_root, &parent)?;
            let metadata = fs::symlink_metadata(&parent_path).map_err(|_| {
                "knowledge_workspace_path_unreadable: 无法读取新建目标父目录状态。".to_string()
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(
                    "knowledge_workspace_invalid_ancestor: 新建目标父级必须是普通目录。"
                        .to_string(),
                );
            }
            parent_path
        }
        None => vault_root.to_path_buf(),
    };

    match exact_child_path(&parent, relative_path.file_name()) {
        Ok(_) => Err(
            "knowledge_workspace_target_exists: 新建、移动或重命名目标不得覆盖现有条目。"
                .to_string(),
        ),
        Err(error) if error.starts_with("knowledge_workspace_entry_not_found:") => {
            Ok(parent.join(relative_path.file_name()))
        }
        Err(error) => Err(error),
    }
}

/// 受限附件目录只允许是 vault 顶层精确的 `attachments`；导入不根据用户文本递归创建目录。
pub(crate) fn ensure_workspace_attachments_directory_at(
    vault_root: &Path,
) -> Result<PathBuf, String> {
    ensure_vault_root(vault_root)?;
    match exact_child_path(vault_root, ATTACHMENTS_DIR_NAME) {
        Ok(path) => {
            let metadata = fs::symlink_metadata(&path).map_err(|_| {
                "knowledge_workspace_attachment_directory_invalid: 无法读取固定附件目录。"
                    .to_string()
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(
                    "knowledge_workspace_attachment_directory_invalid: 固定附件目录必须是普通目录。"
                        .to_string(),
                );
            }
            Ok(path)
        }
        Err(error) if error.starts_with("knowledge_workspace_entry_not_found:") => {
            let directory = vault_root.join(ATTACHMENTS_DIR_NAME);
            fs::create_dir(&directory).map_err(|_| {
                "knowledge_workspace_attachment_directory_create_failed: 无法创建固定附件目录。"
                    .to_string()
            })?;
            exact_child_path(vault_root, ATTACHMENTS_DIR_NAME)
        }
        Err(error) => Err(error),
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WorkspaceMarkdownFile {
    relative_path: String,
    body: String,
    mtime_ms: i64,
    content_hash: String,
}

impl WorkspaceMarkdownFile {
    pub(crate) fn body(&self) -> &str {
        &self.body
    }

    pub(crate) fn mtime_ms(&self) -> i64 {
        self.mtime_ms
    }

    pub(crate) fn content_hash(&self) -> &str {
        &self.content_hash
    }
}

pub(crate) fn read_workspace_markdown_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
) -> Result<WorkspaceMarkdownFile, String> {
    require_markdown_path(relative_path)?;
    let path = resolve_existing_workspace_path(vault_root, relative_path)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取 Markdown 条目状态。".to_string()
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(
            "knowledge_workspace_markdown_only: 路径必须指向普通 Markdown 文件。".to_string(),
        );
    }
    if metadata.len() > MAX_MARKDOWN_BYTES {
        return Err(
            "knowledge_workspace_markdown_too_large: Markdown 超过 64 KiB 安全上限。".to_string(),
        );
    }
    let bytes = fs::read(&path).map_err(|_| {
        "knowledge_workspace_path_unreadable: 无法读取固定 vault Markdown。".to_string()
    })?;
    let body = String::from_utf8(bytes).map_err(|_| {
        "knowledge_workspace_invalid_utf8: Markdown 不是有效 UTF-8，已拒绝读取。".to_string()
    })?;
    Ok(WorkspaceMarkdownFile {
        relative_path: relative_path.as_str().to_string(),
        content_hash: crate::utils::hash::sha256_hex(&body),
        body,
        mtime_ms: mtime_ms_of(&metadata),
    })
}

fn temporary_workspace_path(target: &Path) -> Result<PathBuf, String> {
    let parent = target.parent().ok_or_else(|| {
        "knowledge_workspace_invalid_path: 固定 vault 目标缺少父目录。".to_string()
    })?;
    let leaf = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            "knowledge_workspace_invalid_path: 固定 vault 目标文件名异常。".to_string()
        })?;
    let sequence = WORKSPACE_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(
        ".{leaf}.syn-workspace-{}-{sequence}.tmp",
        std::process::id()
    )))
}

pub(crate) fn write_workspace_temporary_bytes(
    target: &Path,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    let temporary = temporary_workspace_path(target)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|_| {
            "knowledge_workspace_temporary_create_failed: 无法创建固定 vault 同目录临时文件。"
                .to_string()
        })?;
    let result = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| {
            "knowledge_workspace_temporary_write_failed: 无法写入固定 vault 同目录临时文件。"
                .to_string()
        });
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(temporary)
}

fn write_workspace_temporary_file(target: &Path, body: &str) -> Result<PathBuf, String> {
    write_workspace_temporary_bytes(target, body.as_bytes())
}

fn atomically_create_workspace_markdown_at(target: &Path, body: &str) -> Result<(), String> {
    let temporary = write_workspace_temporary_file(target, body)?;
    fs::rename(&temporary, target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_create_failed: 无法原子创建 Markdown。".to_string()
    })
}

fn atomically_replace_workspace_markdown_at(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
    body: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<WorkspaceMarkdownFile, String> {
    require_markdown_body_size(body)?;
    let current = read_workspace_markdown_at(vault_root, relative_path)?;
    if current.mtime_ms() != expected_mtime_ms || current.content_hash() != expected_content_hash {
        return Err(
            "knowledge_vault_conflict: 这条笔记已被外部来源或另一窗口修改，请先重新读取后再保存。"
                .to_string(),
        );
    }
    let target = resolve_existing_workspace_path(vault_root, relative_path)?;
    let temporary = write_workspace_temporary_file(&target, body)?;
    fs::rename(&temporary, &target).map_err(|_| {
        let _ = fs::remove_file(&temporary);
        "knowledge_workspace_atomic_replace_failed: 无法原子替换 Markdown。".to_string()
    })?;
    read_workspace_markdown_at(vault_root, relative_path)
}

fn default_workflow_state_path() -> PathBuf {
    if let Some(paths) = crate::acceptance_runtime_profile::active_paths()
        .expect("acceptance runtime profile must resolve before knowledge workflow path use")
    {
        return paths.workflow_state_path;
    }
    app_data_root().join(WORKFLOW_STATE_RELATIVE)
}

// 文件名=标题 slug：CJK 保留·空白→`-`·剔 `/\:*?"<>|` 与控制字符·空则 `untitled`。
pub(crate) fn slugify_title(title: &str) -> String {
    let mut slug = String::new();
    for character in title.trim().chars() {
        if character.is_control()
            || matches!(
                character,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            )
        {
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

fn require_fixed_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        format!("knowledge_workspace_vault_invalid: 固定 Syn {label} 不存在或不可读取。")
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "knowledge_workspace_symlink_rejected: 固定 Syn {label} 必须是普通目录。"
        ));
    }
    Ok(())
}

fn ensure_fixed_directory_child(parent: &Path, child: &Path, label: &str) -> Result<(), String> {
    require_fixed_directory(parent, "app-data 父目录")?;
    match fs::symlink_metadata(child) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(format!(
            "knowledge_workspace_symlink_rejected: 固定 Syn {label} 必须是普通目录。"
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(child).map_err(|_| {
                format!("knowledge_workspace_vault_create_failed: 无法创建固定 Syn {label}。")
            })?;
            require_fixed_directory(child, label)
        }
        Err(_) => Err(format!(
            "knowledge_workspace_vault_invalid: 固定 Syn {label} 不可读取。"
        )),
    }
}

fn validate_fixed_vault_root_contract(
    app_data_root: &Path,
    vault_root: &Path,
) -> Result<(), String> {
    if vault_root != app_data_root.join(VAULT_DIR_NAME) {
        return Err("knowledge_workspace_vault_invalid: 固定 Syn vault 根合同不匹配。".to_string());
    }
    Ok(())
}

fn require_fixed_vault_ancestors(app_data_root: &Path) -> Result<(), String> {
    let application_support = app_data_root.parent().ok_or_else(|| {
        "knowledge_workspace_vault_invalid: 固定 Syn app-data 父目录缺失。".to_string()
    })?;
    let library = application_support.parent().ok_or_else(|| {
        "knowledge_workspace_vault_invalid: 固定 Syn Library 祖先缺失。".to_string()
    })?;
    let home = library
        .parent()
        .ok_or_else(|| "knowledge_workspace_vault_invalid: 固定 Syn home 祖先缺失。".to_string())?;
    require_fixed_directory(home, "home 祖先")?;
    require_fixed_directory(library, "Library 祖先")?;
    require_fixed_directory(application_support, "Application Support 祖先")
}

/// 外部 bridge 的非创建校验：固定的每一级祖先、app-data 目录及 vault 根都必须已经是
/// 普通目录，避免 canonicalize 经由一个既有父级符号链接落到外部位置。
fn verify_existing_fixed_vault_root_at(
    app_data_root: &Path,
    vault_root: &Path,
) -> Result<(), String> {
    validate_fixed_vault_root_contract(app_data_root, vault_root)?;
    require_fixed_vault_ancestors(app_data_root)?;
    require_fixed_directory(app_data_root, "app-data 目录")?;
    require_fixed_directory(vault_root, "vault 根")
}

/// 只给生产默认根使用的冷启动链：不使用 `create_dir_all`，先检查 home/Library/
/// Application Support 三个既有祖先，再逐段创建本应用自己的目录和 vault 叶子。
/// 泛型测试夹具仍走下方普通根分支，避免把此检查误用为任意路径权限机制。
fn ensure_fixed_vault_root_at(app_data_root: &Path, vault_root: &Path) -> Result<(), String> {
    validate_fixed_vault_root_contract(app_data_root, vault_root)?;
    require_fixed_vault_ancestors(app_data_root)?;
    let application_support = app_data_root.parent().ok_or_else(|| {
        "knowledge_workspace_vault_invalid: 固定 Syn app-data 父目录缺失。".to_string()
    })?;
    ensure_fixed_directory_child(application_support, app_data_root, "app-data 目录")?;
    ensure_fixed_directory_child(app_data_root, vault_root, "vault 根")
}

fn ensure_vault_root(vault_root: &Path) -> Result<(), String> {
    if vault_root == default_vault_root() {
        return ensure_fixed_vault_root_at(&app_data_root(), vault_root);
    }
    if let Ok(metadata) = fs::symlink_metadata(vault_root) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "拒绝符号链接或非目录 vault 根：{}",
                vault_root.display()
            ));
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
    let entries =
        fs::read_dir(vault_root).map_err(|error| format!("读取 vault 目录失败：{error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取 vault 条目失败：{error}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("读取笔记状态失败：{error}"))?;
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
    notes.sort_by(|a, b| {
        b.mtime_ms
            .cmp(&a.mtime_ms)
            .then_with(|| a.slug.cmp(&b.slug))
    });
    Ok(notes)
}

fn read_note_at(vault_root: &Path, slug: &str) -> Result<KnowledgeVaultNote, String> {
    let path = note_path(vault_root, slug)?;
    let body =
        fs::read_to_string(&path).map_err(|error| format!("笔记「{slug}」读取失败：{error}"))?;
    let metadata =
        fs::metadata(&path).map_err(|error| format!("笔记「{slug}」状态读取失败：{error}"))?;
    Ok(KnowledgeVaultNote {
        title: title_from_body(slug, &body),
        slug: slug.to_string(),
        content_hash: crate::utils::hash::sha256_hex(&body),
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
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 workflow-state 目录失败：{error}"))?;
    }
    if workflow_state_path.exists() {
        let _ = crate::workflow_state_store::backup_file(
            workflow_state_path,
            &timestamp_ms.to_string(),
        );
    }
    crate::write_m5b_batch2_workflow_state(workflow_state_path, "knowledge_vault_audit", &value)?;
    Ok(audit_event_id)
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceMutationResult {
    operation: &'static str,
    relative_path: String,
    source_relative_path: Option<String>,
    mtime_ms: Option<i64>,
    content_hash: Option<String>,
    audit_event_id: String,
}

fn workspace_mutation_result(
    operation: &'static str,
    relative_path: &ValidatedVaultRelativePath,
    source_relative_path: Option<&ValidatedVaultRelativePath>,
    file: Option<&WorkspaceMarkdownFile>,
    audit_event_id: String,
) -> KnowledgeWorkspaceMutationResult {
    KnowledgeWorkspaceMutationResult {
        operation,
        relative_path: relative_path.as_str().to_string(),
        source_relative_path: source_relative_path.map(|path| path.as_str().to_string()),
        mtime_ms: file.map(WorkspaceMarkdownFile::mtime_ms),
        content_hash: file.map(|item| item.content_hash().to_string()),
        audit_event_id,
    }
}

pub(crate) fn workspace_create_directory_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    ensure_vault_root(vault_root)?;
    let target = resolve_new_workspace_path(vault_root, &relative_path)?;
    fs::create_dir(&target).map_err(|_| {
        "knowledge_workspace_directory_create_failed: 无法创建固定 vault 目录。".to_string()
    })?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_directory_created",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区新建目录。",
    )?;
    Ok(workspace_mutation_result(
        "directory_created",
        &relative_path,
        None,
        None,
        audit_event_id,
    ))
}

pub(crate) fn workspace_create_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    body: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    require_markdown_path(&relative_path)?;
    require_markdown_body_size(body)?;
    ensure_vault_root(vault_root)?;
    let target = resolve_new_workspace_path(vault_root, &relative_path)?;
    atomically_create_workspace_markdown_at(&target, body)?;
    let file = read_workspace_markdown_at(vault_root, &relative_path)?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_markdown_created",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区新建 Markdown。",
    )?;
    Ok(workspace_mutation_result(
        "markdown_created",
        &relative_path,
        None,
        Some(&file),
        audit_event_id,
    ))
}

pub(crate) fn workspace_write_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    body: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    require_markdown_path(&relative_path)?;
    let written = atomically_replace_workspace_markdown_at(
        vault_root,
        &relative_path,
        body,
        expected_mtime_ms,
        expected_content_hash,
    )?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_markdown_updated",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区更新 Markdown。",
    )?;
    Ok(workspace_mutation_result(
        "markdown_updated",
        &relative_path,
        None,
        Some(&written),
        audit_event_id,
    ))
}

fn require_current_workspace_markdown(
    vault_root: &Path,
    relative_path: &ValidatedVaultRelativePath,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<WorkspaceMarkdownFile, String> {
    let current = read_workspace_markdown_at(vault_root, relative_path)?;
    if current.mtime_ms() != expected_mtime_ms || current.content_hash() != expected_content_hash {
        return Err(
            "knowledge_vault_conflict: 这条笔记已被外部来源或另一窗口修改，请先重新读取后再保存。"
                .to_string(),
        );
    }
    Ok(current)
}

fn workspace_move_or_rename_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_from: &str,
    raw_to: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
    operation: &'static str,
    event_type: &'static str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    let from = validate_workspace_relative_path(raw_from)?;
    let to = validate_workspace_relative_path(raw_to)?;
    require_markdown_path(&from)?;
    require_markdown_path(&to)?;
    let _current = require_current_workspace_markdown(
        vault_root,
        &from,
        expected_mtime_ms,
        expected_content_hash,
    )?;
    let source = resolve_existing_workspace_path(vault_root, &from)?;
    let target = resolve_new_workspace_path(vault_root, &to)?;
    fs::rename(&source, &target).map_err(|_| {
        "knowledge_workspace_move_failed: 无法移动固定 vault Markdown。".to_string()
    })?;
    let moved = read_workspace_markdown_at(vault_root, &to)?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        event_type,
        to.as_str(),
        "user_manual_edit",
        &format!(
            "用户在 Syn 原生知识工作区移动 Markdown：{} -> {}。",
            from.as_str(),
            to.as_str()
        ),
    )?;
    Ok(workspace_mutation_result(
        operation,
        &to,
        Some(&from),
        Some(&moved),
        audit_event_id,
    ))
}

pub(crate) fn workspace_move_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_from: &str,
    raw_to: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_move_or_rename_markdown_at(
        vault_root,
        workflow_state_path,
        raw_from,
        raw_to,
        expected_mtime_ms,
        expected_content_hash,
        "markdown_moved",
        "knowledge_workspace_markdown_moved",
    )
}

pub(crate) fn workspace_rename_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_from: &str,
    raw_to: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_move_or_rename_markdown_at(
        vault_root,
        workflow_state_path,
        raw_from,
        raw_to,
        expected_mtime_ms,
        expected_content_hash,
        "markdown_renamed",
        "knowledge_workspace_markdown_renamed",
    )
}

pub(crate) fn workspace_delete_markdown_at(
    vault_root: &Path,
    workflow_state_path: &Path,
    raw_relative_path: &str,
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    let relative_path = validate_workspace_relative_path(raw_relative_path)?;
    require_markdown_path(&relative_path)?;
    let _current = require_current_workspace_markdown(
        vault_root,
        &relative_path,
        expected_mtime_ms,
        expected_content_hash,
    )?;
    let target = resolve_existing_workspace_path(vault_root, &relative_path)?;
    fs::remove_file(&target).map_err(|_| {
        "knowledge_workspace_delete_failed: 无法删除固定 vault Markdown。".to_string()
    })?;
    let audit_event_id = append_audit_event(
        workflow_state_path,
        "knowledge_workspace_markdown_deleted",
        relative_path.as_str(),
        "user_manual_edit",
        "用户在 Syn 原生知识工作区删除 Markdown。",
    )?;
    Ok(workspace_mutation_result(
        "markdown_deleted",
        &relative_path,
        None,
        None,
        audit_event_id,
    ))
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
    expected_mtime_ms: i64,
    expected_content_hash: &str,
) -> Result<KnowledgeVaultWriteResult, String> {
    let relative_path = ValidatedVaultRelativePath::parse(&format!("{slug}.md"))
        .map_err(|_| format!("非法笔记名「{slug}」：只允许 vault 内单层文件名"))?;
    let _written = atomically_replace_workspace_markdown_at(
        vault_root,
        &relative_path,
        body,
        expected_mtime_ms,
        expected_content_hash,
    )?;
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
    })
}

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
pub(crate) fn knowledge_vault_create_note(
    title: String,
) -> Result<KnowledgeVaultWriteResult, String> {
    create_note_at(
        &default_vault_root(),
        &default_workflow_state_path(),
        &title,
    )
}

#[tauri::command]
pub(crate) fn knowledge_vault_write_note(
    slug: String,
    body: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeVaultWriteResult, String> {
    write_note_at(
        &default_vault_root(),
        &default_workflow_state_path(),
        &slug,
        &body,
        expected_mtime_ms,
        &expected_content_hash,
    )
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

#[tauri::command]
pub(crate) fn knowledge_workspace_create_directory(
    relative_path: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_create_directory_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_create_markdown(
    relative_path: String,
    body: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_create_markdown_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
        &body,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_write_markdown(
    relative_path: String,
    body: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_write_markdown_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
        &body,
        expected_mtime_ms,
        &expected_content_hash,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_move_entry(
    from: String,
    to: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_move_markdown_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &from,
        &to,
        expected_mtime_ms,
        &expected_content_hash,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_rename_entry(
    from: String,
    to: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_rename_markdown_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &from,
        &to,
        expected_mtime_ms,
        &expected_content_hash,
    )
}

#[tauri::command]
pub(crate) fn knowledge_workspace_delete_entry(
    relative_path: String,
    expected_mtime_ms: i64,
    expected_content_hash: String,
) -> Result<KnowledgeWorkspaceMutationResult, String> {
    workspace_delete_markdown_at(
        &workspace_vault_root(),
        &workspace_workflow_state_path(),
        &relative_path,
        expected_mtime_ms,
        &expected_content_hash,
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

    fn write_workflow_state_fixture(path: &Path) {
        let state = json!({
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
        });
        fs::write(path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
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
        let result = write_note_at(&root, &root.join("state.json"), "../escape", "x", 0, "");
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
            let write_result = write_note_at(&root, &root.join("state.json"), "link", "x", 0, "");
            assert!(write_result.is_err(), "符号链接写入必须拒绝");
        }
    }

    #[test]
    fn create_list_read_edit_roundtrip() {
        let root = temp_root("roundtrip");
        let state = root.join("state.json");
        write_workflow_state_fixture(&state);
        let created = create_note_at(&root, &state, "我的 笔记").unwrap();
        assert_eq!(created.slug, "我的-笔记");
        let duplicate = create_note_at(&root, &state, "我的 笔记").unwrap();
        assert_eq!(duplicate.slug, "我的-笔记-2", "重名追加 -2");

        let notes = list_notes_at(&root).unwrap();
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().all(|note| note.title == "我的 笔记"));

        let before_edit = read_note_at(&root, "我的-笔记").unwrap();
        let edited = write_note_at(
            &root,
            &state,
            "我的-笔记",
            "# 我的 笔记\n\n正文 [[另一条]]。\n",
            before_edit.mtime_ms,
            &before_edit.content_hash,
        )
        .unwrap();
        assert_eq!(edited.title, "我的 笔记");
        let note = read_note_at(&root, "我的-笔记").unwrap();
        assert!(note.body.contains("正文"));
        assert_eq!(note.mtime_ms > 0, true);
        let listed = list_notes_at(&root).unwrap();
        assert_eq!(listed[0].outlinks, vec!["另一条".to_string()]);
        assert_eq!(
            note.content_hash,
            crate::utils::hash::sha256_hex(&note.body)
        );

        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        let event_types: Vec<&str> = events
            .iter()
            .filter_map(|event| event["event_type"].as_str())
            .collect();
        assert_eq!(
            event_types,
            vec![
                "knowledge_vault_note_created",
                "knowledge_vault_note_created",
                "knowledge_vault_note_user_edited"
            ]
        );
        assert!(events
            .iter()
            .all(|event| event["actor_ref"] == "user_manual_edit"));
        assert_eq!(
            state_value["revision"], 3,
            "three audits consume three revisions"
        );
    }

    #[test]
    fn write_rejects_external_change_without_overwrite_or_audit() {
        let root = temp_root("conflict");
        let state = root.join("state.json");
        write_workflow_state_fixture(&state);
        let created = create_note_at(&root, &state, "冲突笔记").unwrap();
        let before = read_note_at(&root, &created.slug).unwrap();

        fs::write(
            root.join(format!("{}.md", created.slug)),
            "# 冲突笔记\n\nObsidian 外部改动\n",
        )
        .unwrap();
        let rejected = write_note_at(
            &root,
            &state,
            &created.slug,
            "# 冲突笔记\n\nSyn 覆盖尝试\n",
            before.mtime_ms,
            &before.content_hash,
        )
        .unwrap_err();

        assert!(rejected.starts_with("knowledge_vault_conflict:"));
        let after = fs::read_to_string(root.join(format!("{}.md", created.slug))).unwrap();
        assert!(
            after.contains("Obsidian 外部改动"),
            "冲突时不得覆盖外部正文"
        );
        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        assert_eq!(
            state_value["audit_events"].as_array().unwrap().len(),
            1,
            "冲突不能新增审计写入"
        );
    }

    #[test]
    fn ai_write_requires_source_and_audits_actor() {
        let root = temp_root("ai-write");
        let state = root.join("state.json");
        write_workflow_state_fixture(&state);
        let rejected = ai_write_note_at(&root, &state, "标题", "正文", "  ");
        assert!(rejected.is_err(), "source_summary 空必须拒绝");
        assert!(!root.join("标题.md").exists(), "拒绝时不得落文件");

        let written = ai_write_note_at(
            &root,
            &state,
            "AI 笔记",
            "# AI 笔记\n\n内容",
            "记忆候选 memcand:v1:x",
        )
        .unwrap();
        assert_eq!(written.slug, "AI-笔记");
        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "knowledge_vault_note_ai_written");
        assert_eq!(events[0]["actor_ref"], "ai_proposed_user_confirmed");
        assert!(events[0]["reason"]
            .as_str()
            .unwrap()
            .contains("记忆候选 memcand:v1:x"));
        assert_eq!(
            state_value["revision"], 1,
            "one audit consumes one revision"
        );
    }

    #[test]
    fn workspace_nested_create_move_rename_delete_are_conflict_checked_and_audited() {
        let root = temp_root("workspace-mutations");
        let state = root.join("state.json");
        write_workflow_state_fixture(&state);

        let missing_parent =
            workspace_create_markdown_at(&root, &state, "research/plan.md", "# Plan\n")
                .unwrap_err();
        assert!(missing_parent.starts_with("knowledge_workspace_entry_not_found:"));

        let directory = workspace_create_directory_at(&root, &state, "research").unwrap();
        assert_eq!(directory.operation, "directory_created");
        let created =
            workspace_create_markdown_at(&root, &state, "research/plan.md", "# Plan\n\nfirst\n")
                .unwrap();
        assert_eq!(created.operation, "markdown_created");
        assert!(root.join("research/plan.md").is_file());
        assert!(
            workspace_create_markdown_at(&root, &state, "research/plan.md", "# replacement\n",)
                .unwrap_err()
                .starts_with("knowledge_workspace_target_exists:")
        );

        let stale = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/plan.md").unwrap(),
        )
        .unwrap();
        fs::write(root.join("research/plan.md"), "# Plan\n\nexternal\n").unwrap();
        let rejected = workspace_move_markdown_at(
            &root,
            &state,
            "research/plan.md",
            "research/moved.md",
            stale.mtime_ms(),
            stale.content_hash(),
        )
        .unwrap_err();
        assert!(rejected.starts_with("knowledge_vault_conflict:"));
        assert!(!root.join("research/moved.md").exists());
        assert!(fs::read_to_string(root.join("research/plan.md"))
            .unwrap()
            .contains("external"));

        let fresh = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/plan.md").unwrap(),
        )
        .unwrap();
        let moved = workspace_move_markdown_at(
            &root,
            &state,
            "research/plan.md",
            "research/moved.md",
            fresh.mtime_ms(),
            fresh.content_hash(),
        )
        .unwrap();
        assert_eq!(moved.operation, "markdown_moved");
        assert!(!root.join("research/plan.md").exists());
        assert!(root.join("research/moved.md").is_file());

        let before_rename = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/moved.md").unwrap(),
        )
        .unwrap();
        let renamed = workspace_rename_markdown_at(
            &root,
            &state,
            "research/moved.md",
            "research/renamed.md",
            before_rename.mtime_ms(),
            before_rename.content_hash(),
        )
        .unwrap();
        assert_eq!(renamed.operation, "markdown_renamed");

        let before_delete = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/renamed.md").unwrap(),
        )
        .unwrap();
        let deleted = workspace_delete_markdown_at(
            &root,
            &state,
            "research/renamed.md",
            before_delete.mtime_ms(),
            before_delete.content_hash(),
        )
        .unwrap();
        assert_eq!(deleted.operation, "markdown_deleted");
        assert!(!root.join("research/renamed.md").exists());

        let state_value = crate::workflow_state_store::read_value(&state).unwrap();
        let events = state_value["audit_events"].as_array().unwrap();
        assert_eq!(events.len(), 5, "stale move must not append an audit event");
        assert_eq!(
            events[0]["event_type"],
            "knowledge_workspace_directory_created"
        );
        assert_eq!(
            events[1]["event_type"],
            "knowledge_workspace_markdown_created"
        );
        assert_eq!(
            events[2]["event_type"],
            "knowledge_workspace_markdown_moved"
        );
        assert_eq!(
            events[3]["event_type"],
            "knowledge_workspace_markdown_renamed"
        );
        assert_eq!(
            events[4]["event_type"],
            "knowledge_workspace_markdown_deleted"
        );
    }

    #[test]
    fn workspace_write_markdown_updates_only_an_existing_nested_markdown_with_cas() {
        let root = temp_root("workspace-write-markdown");
        let state = root.join("state.json");
        write_workflow_state_fixture(&state);
        workspace_create_directory_at(&root, &state, "research").unwrap();
        workspace_create_markdown_at(&root, &state, "research/plan.md", "# Plan\n\nfirst\n")
            .unwrap();
        fs::write(root.join("board.canvas"), "{}\n").unwrap();
        fs::create_dir(root.join("attachments")).unwrap();
        fs::write(root.join("attachments/item.txt"), "attachment\n").unwrap();

        let before = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/plan.md").unwrap(),
        )
        .unwrap();
        let updated = workspace_write_markdown_at(
            &root,
            &state,
            "research/plan.md",
            "# Plan\n\nupdated\n",
            before.mtime_ms(),
            before.content_hash(),
        )
        .unwrap();
        assert_eq!(updated.operation, "markdown_updated");
        assert_eq!(updated.relative_path, "research/plan.md");
        assert_eq!(
            fs::read_to_string(root.join("research/plan.md")).unwrap(),
            "# Plan\n\nupdated\n"
        );

        let stale = read_workspace_markdown_at(
            &root,
            &ValidatedVaultRelativePath::parse("research/plan.md").unwrap(),
        )
        .unwrap();
        fs::write(root.join("research/plan.md"), "# Plan\n\nexternal\n").unwrap();
        let stale_error = workspace_write_markdown_at(
            &root,
            &state,
            "research/plan.md",
            "# Plan\n\nshould-not-write\n",
            stale.mtime_ms(),
            stale.content_hash(),
        )
        .unwrap_err();
        assert!(stale_error.starts_with("knowledge_vault_conflict:"));
        assert!(fs::read_to_string(root.join("research/plan.md"))
            .unwrap()
            .contains("external"));

        for rejected_path in [
            "research",
            "board.canvas",
            "attachments/item.txt",
            "../escape.md",
            "research/missing.md",
        ] {
            assert!(
                workspace_write_markdown_at(
                    &root,
                    &state,
                    rejected_path,
                    "# rejected\n",
                    updated.mtime_ms.unwrap(),
                    updated.content_hash.as_deref().unwrap(),
                )
                .is_err(),
                "{rejected_path} 必须拒绝"
            );
        }

        let events = crate::workflow_state_store::read_value(&state).unwrap()["audit_events"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(events.len(), 3, "失败写入不得追加审计");
        assert_eq!(
            events[2]["event_type"],
            "knowledge_workspace_markdown_updated"
        );
    }

    #[test]
    fn workspace_cold_start_can_create_only_the_fixed_root_and_a_root_markdown() {
        let parent = temp_root("workspace-cold-start");
        let root = parent.join("fixed-vault");
        let state = parent.join("state.json");
        write_workflow_state_fixture(&state);
        assert!(!root.exists());

        let created = workspace_create_markdown_at(&root, &state, "first.md", "# First\n")
            .expect("fixed root can initialize for the first Syn Markdown");
        assert_eq!(created.relative_path, "first.md");
        assert!(root.join("first.md").is_file());
        assert!(
            workspace_create_markdown_at(&root, &state, "nested/second.md", "# No\n")
                .unwrap_err()
                .starts_with("knowledge_workspace_entry_not_found:")
        );
        assert!(!root.join("nested").exists(), "不得隐式创建嵌套父目录");
    }

    #[cfg(unix)]
    #[test]
    fn fixed_root_cold_start_rejects_a_symlink_parent_before_creating_any_vault_child() {
        use std::os::unix::fs::symlink;

        let home = temp_root("fixed-root-home");
        let library = home.join("Library");
        fs::create_dir(&library).unwrap();
        let outside = temp_root("fixed-root-outside");
        symlink(&outside, library.join("Application Support")).unwrap();
        let app_data = library
            .join("Application Support")
            .join("CodexGovernanceWorkbench");
        let vault = app_data.join(VAULT_DIR_NAME);

        let rejected = ensure_fixed_vault_root_at(&app_data, &vault).unwrap_err();
        assert!(rejected.starts_with("knowledge_workspace_symlink_rejected:"));
        let external_rejected = syn_vault_root_for_external_integration_at(&app_data).unwrap_err();
        assert_eq!(
            external_rejected,
            "Syn 自管 vault 路径异常，已拒绝外部集成访问。"
        );
        assert!(
            !outside.join("CodexGovernanceWorkbench").exists(),
            "父级异常时不得创建任何 vault 子目录"
        );
    }
}
