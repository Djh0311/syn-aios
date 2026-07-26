//! N1 的可重建 Syn 原生知识工作区索引。
//!
//! 这里不写 `.index`、SQLite 或旁路 JSON。每次 snapshot/search/read 都从固定 vault
//! 的普通文件重新投影；Markdown/Canvas/附件文件本身才是唯一真相源。

use crate::knowledge_vault::{
    self, ValidatedVaultRelativePath, MAX_ATTACHMENT_BYTES, MAX_CANVAS_BYTES, MAX_MARKDOWN_BYTES,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const MAX_INDEX_ENTRIES: usize = 2_048;
const MAX_DIAGNOSTICS: usize = 32;
const MAX_SEARCH_QUERY_BYTES: usize = 256;
const MAX_SEARCH_RESULTS: usize = 100;
const MAX_FRONTMATTER_LINES: usize = 128;
const MAX_METADATA_TEXT_CHARS: usize = 256;
const MAX_METADATA_LIST_ITEMS: usize = 64;
const MAX_PROPERTIES: usize = 64;
const MAX_GRAPH_NODES: usize = 512;
const MAX_GRAPH_EDGES: usize = 1_024;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceSnapshot {
    entries: Vec<KnowledgeWorkspaceEntry>,
    tags: Vec<KnowledgeWorkspaceTag>,
    diagnostics: Vec<KnowledgeWorkspaceDiagnostic>,
}

/// N5 的 vault manifest 是即时索引投影：不落盘、不缓存，也不批量读取正文/附件字节。
/// 具体文件的 hash 仍只在对应 read/backup CAS 操作时计算。
#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceVaultManifest {
    pub(crate) entries: Vec<KnowledgeWorkspaceVaultManifestEntry>,
    pub(crate) diagnostics: Vec<KnowledgeWorkspaceDiagnostic>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceVaultManifestEntry {
    pub(crate) relative_path: String,
    pub(crate) kind: &'static str,
    pub(crate) mtime_ms: i64,
    pub(crate) size_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceEntry {
    relative_path: String,
    parent_path: Option<String>,
    kind: &'static str,
    title: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
    properties: BTreeMap<String, String>,
    mtime_ms: i64,
    size_bytes: u64,
    outlinks: Vec<String>,
    backlinks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceTag {
    tag: String,
    note_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceDiagnostic {
    code: &'static str,
    relative_path: Option<String>,
    message: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceSearchResponse {
    query: String,
    results: Vec<KnowledgeWorkspaceSearchResult>,
    diagnostics: Vec<KnowledgeWorkspaceDiagnostic>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceSearchResult {
    relative_path: String,
    title: String,
    snippet: String,
    tags: Vec<String>,
    mtime_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceMarkdownDocument {
    relative_path: String,
    title: String,
    body: String,
    tags: Vec<String>,
    aliases: Vec<String>,
    properties: BTreeMap<String, String>,
    outlinks: Vec<String>,
    backlinks: Vec<String>,
    mtime_ms: i64,
    content_hash: String,
}

/// N3 图谱范围只接受这两个精确值；不把未知值降级为全局图。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KnowledgeWorkspaceGraphScope {
    Global,
    Local,
}

impl KnowledgeWorkspaceGraphScope {
    fn from_raw(raw_scope: &str) -> Result<Self, String> {
        match raw_scope {
            "global" => Ok(Self::Global),
            "local" => Ok(Self::Local),
            _ => Err(
                "knowledge_workspace_invalid_graph_scope: 图谱范围只能是 global 或 local。"
                    .to_string(),
            ),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
        }
    }
}

/// 前端只通过固定 `knowledge_workspace_graph` 传入这些受限字段。这里不接受 vault
/// 根、任意文件系统目标、布局或写入意图；返回值始终可由 Markdown 重新投影。
#[derive(Clone, Debug)]
pub(crate) struct KnowledgeWorkspaceGraphRequest {
    scope: KnowledgeWorkspaceGraphScope,
    focus_relative_path: Option<ValidatedVaultRelativePath>,
    query: Option<String>,
    tag: Option<String>,
}

impl KnowledgeWorkspaceGraphRequest {
    fn from_raw(
        raw_scope: &str,
        raw_focus_relative_path: Option<String>,
        raw_query: Option<String>,
        raw_tag: Option<String>,
    ) -> Result<Self, String> {
        let scope = KnowledgeWorkspaceGraphScope::from_raw(raw_scope)?;
        let focus_relative_path = match scope {
            KnowledgeWorkspaceGraphScope::Global => {
                if raw_focus_relative_path.is_some() {
                    return Err(
                        "knowledge_workspace_invalid_graph_focus: 全局图不接受焦点路径。"
                            .to_string(),
                    );
                }
                None
            }
            KnowledgeWorkspaceGraphScope::Local => {
                let raw_focus_relative_path = raw_focus_relative_path.ok_or_else(|| {
                    "knowledge_workspace_invalid_graph_focus: 局部图必须提供 Markdown 焦点路径。"
                        .to_string()
                })?;
                let focus_relative_path =
                    ValidatedVaultRelativePath::parse(&raw_focus_relative_path)?;
                require_graph_markdown_path(&focus_relative_path)?;
                Some(focus_relative_path)
            }
        };
        let query = raw_query
            .as_deref()
            .map(validate_search_query)
            .transpose()?;
        let tag = raw_tag.as_deref().map(validate_graph_tag).transpose()?;
        Ok(Self {
            scope,
            focus_relative_path,
            query,
            tag,
        })
    }
}

/// `id` 和 `relative_path` 始终相同，且均由 `ValidatedVaultRelativePath` 从已验证
/// Markdown 条目派生。这正好可直接映射到 React Flow 的 node id，不能成为任意 URI。
#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceGraphNode {
    id: String,
    relative_path: String,
    title: String,
    tags: Vec<String>,
}

/// 两端都是已输出的 validated Markdown node id；图投影不会为无效链接虚构节点。
#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceGraphEdge {
    id: String,
    source: String,
    target: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct KnowledgeWorkspaceGraphResponse {
    scope: &'static str,
    focus_relative_path: Option<String>,
    query: Option<String>,
    tag: Option<String>,
    nodes: Vec<KnowledgeWorkspaceGraphNode>,
    edges: Vec<KnowledgeWorkspaceGraphEdge>,
    diagnostics: Vec<KnowledgeWorkspaceDiagnostic>,
    truncated: bool,
}

#[derive(Default)]
struct BuildState {
    entries: Vec<KnowledgeWorkspaceEntry>,
    markdown: Vec<MarkdownProjection>,
    diagnostics: Vec<KnowledgeWorkspaceDiagnostic>,
    entry_limit_reported: bool,
}

struct MarkdownProjection {
    entry_index: usize,
    searchable_body: String,
    raw_links: Vec<String>,
}

#[derive(Default)]
struct ParsedMarkdown {
    title: Option<String>,
    tags: Vec<String>,
    aliases: Vec<String>,
    properties: BTreeMap<String, String>,
    searchable_body: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrontmatterSection {
    Tags,
    Aliases,
    Properties,
    Other,
}

impl BuildState {
    fn diagnostic(
        &mut self,
        code: &'static str,
        relative_path: Option<&ValidatedVaultRelativePath>,
        message: &'static str,
    ) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics.push(KnowledgeWorkspaceDiagnostic {
                code,
                relative_path: relative_path.map(|path| path.as_str().to_string()),
                message,
            });
        }
    }

    fn can_add_entry(&mut self) -> bool {
        if self.entries.len() < MAX_INDEX_ENTRIES {
            return true;
        }
        if !self.entry_limit_reported {
            self.entry_limit_reported = true;
            self.diagnostic(
                "knowledge_workspace_index_entry_limit",
                None,
                "工作区条目超过本阶段可重建索引上限，剩余条目未加载。",
            );
        }
        false
    }
}

fn mtime_ms_of(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn parent_path(relative_path: &ValidatedVaultRelativePath) -> Option<String> {
    relative_path.parent().map(|path| path.as_str().to_string())
}

fn add_entry(
    state: &mut BuildState,
    relative_path: &ValidatedVaultRelativePath,
    kind: &'static str,
    metadata: &fs::Metadata,
) -> Option<usize> {
    if !state.can_add_entry() {
        return None;
    }
    let entry_index = state.entries.len();
    state.entries.push(KnowledgeWorkspaceEntry {
        relative_path: relative_path.as_str().to_string(),
        parent_path: parent_path(relative_path),
        kind,
        title: None,
        tags: Vec::new(),
        aliases: Vec::new(),
        properties: BTreeMap::new(),
        mtime_ms: mtime_ms_of(metadata),
        size_bytes: metadata.len(),
        outlinks: Vec::new(),
        backlinks: Vec::new(),
    });
    Some(entry_index)
}

fn classify_directory(
    directory: &Path,
    prefix: Option<&ValidatedVaultRelativePath>,
    in_attachments: bool,
    state: &mut BuildState,
) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|_| "knowledge_workspace_scan_failed: 无法读取固定 vault 目录。".to_string())?;
    let mut entries: Vec<_> = entries
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "knowledge_workspace_scan_failed: 无法枚举固定 vault 条目。".to_string())?;
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    for entry in entries {
        if !state.can_add_entry() {
            break;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            state.diagnostic(
                "knowledge_workspace_invalid_name",
                prefix,
                "含非 UTF-8 名称的条目未进入知识工作区索引。",
            );
            continue;
        };
        let raw_relative = match prefix {
            Some(prefix) => format!("{}/{}", prefix.as_str(), name),
            None => name.to_string(),
        };
        let relative_path = match ValidatedVaultRelativePath::parse(&raw_relative) {
            Ok(path) => path,
            Err(_) => {
                state.diagnostic(
                    "knowledge_workspace_invalid_path",
                    prefix,
                    "不符合受限相对路径合同的条目未进入索引。",
                );
                continue;
            }
        };
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|_| {
            "knowledge_workspace_scan_failed: 无法读取固定 vault 条目状态。".to_string()
        })?;
        if metadata.file_type().is_symlink() {
            state.diagnostic(
                "knowledge_workspace_symlink_skipped",
                Some(&relative_path),
                "符号链接不会进入 Syn 知识工作区索引。",
            );
            continue;
        }

        if metadata.is_dir() {
            let is_attachment_directory =
                in_attachments || (prefix.is_none() && relative_path.as_str() == "attachments");
            let _ = add_entry(state, &relative_path, "directory", &metadata);
            classify_directory(&path, Some(&relative_path), is_attachment_directory, state)?;
            continue;
        }
        if !metadata.is_file() {
            state.diagnostic(
                "knowledge_workspace_unsupported_entry",
                Some(&relative_path),
                "只有普通文件和普通目录可进入 Syn 知识工作区索引。",
            );
            continue;
        }

        if in_attachments {
            if knowledge_vault::workspace_attachment_kind_for_relative_path(&relative_path)
                .is_none()
            {
                state.diagnostic(
                    "knowledge_workspace_attachment_type_skipped",
                    Some(&relative_path),
                    "附件目录仅接受本阶段允许的受限文件类型。",
                );
                continue;
            }
            if metadata.len() > MAX_ATTACHMENT_BYTES {
                state.diagnostic(
                    "knowledge_workspace_attachment_too_large",
                    Some(&relative_path),
                    "附件超过本阶段 10 MiB 安全上限，未进入索引。",
                );
                continue;
            }
            let _ = add_entry(state, &relative_path, "attachment", &metadata);
            continue;
        }

        match path.extension().and_then(|extension| extension.to_str()) {
            Some("md") => {
                if metadata.len() > MAX_MARKDOWN_BYTES {
                    state.diagnostic(
                        "knowledge_workspace_markdown_too_large",
                        Some(&relative_path),
                        "Markdown 超过 64 KiB 安全上限，未进入索引。",
                    );
                    continue;
                }
                let bytes = fs::read(&path).map_err(|_| {
                    "knowledge_workspace_scan_failed: 无法读取固定 vault Markdown。".to_string()
                })?;
                let body = match String::from_utf8(bytes) {
                    Ok(body) => body,
                    Err(_) => {
                        state.diagnostic(
                            "knowledge_workspace_invalid_utf8",
                            Some(&relative_path),
                            "不是有效 UTF-8 的 Markdown 未进入索引。",
                        );
                        continue;
                    }
                };
                let parsed = match parse_markdown(&body) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        state.diagnostic(
                            "knowledge_workspace_invalid_frontmatter",
                            Some(&relative_path),
                            "Frontmatter 不符合本阶段安全子集，Markdown 未进入索引。",
                        );
                        continue;
                    }
                };
                let Some(entry_index) = add_entry(state, &relative_path, "markdown", &metadata)
                else {
                    continue;
                };
                let title = parsed.title.clone().unwrap_or_else(|| {
                    title_from_markdown(&relative_path, &parsed.searchable_body)
                });
                let raw_links = extract_wikilinks(&parsed.searchable_body);
                let entry = &mut state.entries[entry_index];
                entry.title = Some(title);
                entry.tags = parsed.tags;
                entry.aliases = parsed.aliases;
                entry.properties = parsed.properties;
                state.markdown.push(MarkdownProjection {
                    entry_index,
                    searchable_body: parsed.searchable_body,
                    raw_links,
                });
            }
            Some("canvas") => {
                if metadata.len() > MAX_CANVAS_BYTES {
                    state.diagnostic(
                        "knowledge_workspace_canvas_too_large",
                        Some(&relative_path),
                        "Canvas 超过 256 KiB 安全上限，未进入索引。",
                    );
                } else {
                    let _ = add_entry(state, &relative_path, "canvas", &metadata);
                }
            }
            _ => state.diagnostic(
                "knowledge_workspace_unsupported_file",
                Some(&relative_path),
                "不属于 Markdown、JSON Canvas 或附件目录的文件未进入索引。",
            ),
        }
    }
    Ok(())
}

fn parse_markdown(body: &str) -> Result<ParsedMarkdown, String> {
    let lines: Vec<&str> = body.lines().collect();
    if lines.first().map(|line| line.trim_end_matches('\r')) != Some("---") {
        return Ok(ParsedMarkdown {
            searchable_body: body.to_string(),
            ..ParsedMarkdown::default()
        });
    }
    let closing_index = lines
        .iter()
        .enumerate()
        .skip(1)
        .take(MAX_FRONTMATTER_LINES)
        .find_map(|(index, line)| (line.trim_end_matches('\r') == "---").then_some(index))
        .ok_or_else(|| "frontmatter delimiter missing".to_string())?;
    let mut parsed = parse_frontmatter_lines(&lines[1..closing_index])?;
    parsed.searchable_body = lines
        .get(closing_index + 1..)
        .unwrap_or_default()
        .join("\n");
    Ok(parsed)
}

fn parse_frontmatter_lines(lines: &[&str]) -> Result<ParsedMarkdown, String> {
    let mut parsed = ParsedMarkdown::default();
    let mut section: Option<FrontmatterSection> = None;
    for raw_line in lines {
        let line = raw_line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        if let Some(item) = line.strip_prefix("  - ") {
            match section {
                Some(FrontmatterSection::Tags) => push_bounded_scalar(&mut parsed.tags, item)?,
                Some(FrontmatterSection::Aliases) => {
                    push_bounded_scalar(&mut parsed.aliases, item)?
                }
                _ => return Err("list item outside safe list".to_string()),
            }
            continue;
        }
        if let Some(property_line) = line.strip_prefix("  ") {
            if section == Some(FrontmatterSection::Properties) {
                let (key, value) = property_line
                    .split_once(':')
                    .ok_or_else(|| "property delimiter missing".to_string())?;
                let key = safe_property_key(key)?;
                let value = safe_scalar(value)?;
                if parsed.properties.len() >= MAX_PROPERTIES
                    && !parsed.properties.contains_key(&key)
                {
                    return Err("properties exceed limit".to_string());
                }
                if parsed.properties.insert(key, value).is_some() {
                    return Err("duplicate property".to_string());
                }
                continue;
            }
            if section == Some(FrontmatterSection::Other) {
                continue;
            }
            return Err("unexpected indentation".to_string());
        }
        if line.starts_with(char::is_whitespace) {
            return Err("unsupported indentation".to_string());
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| "frontmatter key delimiter missing".to_string())?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "title" => {
                if parsed.title.is_some() {
                    return Err("duplicate title".to_string());
                }
                parsed.title = Some(safe_scalar(value)?);
                section = None;
            }
            "tags" => {
                parse_scalar_or_list(value, &mut parsed.tags)?;
                section = Some(FrontmatterSection::Tags);
            }
            "aliases" => {
                parse_scalar_or_list(value, &mut parsed.aliases)?;
                section = Some(FrontmatterSection::Aliases);
            }
            "properties" => {
                if !value.is_empty() {
                    return Err("properties must be a one-level map".to_string());
                }
                section = Some(FrontmatterSection::Properties);
            }
            _ => section = Some(FrontmatterSection::Other),
        }
    }
    Ok(parsed)
}

fn safe_scalar(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty()
        || value.chars().count() > MAX_METADATA_TEXT_CHARS
        || value.chars().any(char::is_control)
        || value.contains(['[', ']', '{', '}', '|', '>'])
    {
        return Err("unsafe scalar".to_string());
    }
    Ok(value.to_string())
}

fn safe_property_key(raw: &str) -> Result<String, String> {
    let key = raw.trim();
    if key.is_empty()
        || key.chars().count() > 64
        || key.chars().any(char::is_control)
        || !key
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err("unsafe property key".to_string());
    }
    Ok(key.to_string())
}

fn push_bounded_scalar(values: &mut Vec<String>, raw: &str) -> Result<(), String> {
    let value = safe_scalar(raw)?;
    if !values.iter().any(|existing| existing == &value) {
        if values.len() == MAX_METADATA_LIST_ITEMS {
            return Err("metadata list exceeds limit".to_string());
        }
        values.push(value);
    }
    Ok(())
}

fn parse_scalar_or_list(value: &str, values: &mut Vec<String>) -> Result<(), String> {
    if value.is_empty() {
        return Ok(());
    }
    if value.starts_with('[') || value.ends_with(']') {
        if !(value.starts_with('[') && value.ends_with(']')) {
            return Err("unbalanced inline list".to_string());
        }
        let inner = &value[1..value.len() - 1];
        if inner.trim().is_empty() {
            return Ok(());
        }
        for item in inner.split(',') {
            push_bounded_scalar(values, item)?;
        }
        return Ok(());
    }
    push_bounded_scalar(values, value)
}

fn title_from_markdown(relative_path: &ValidatedVaultRelativePath, body: &str) -> String {
    body.lines()
        .find_map(|line| {
            line.trim_start()
                .strip_prefix("# ")
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(|title| title.chars().take(MAX_METADATA_TEXT_CHARS).collect())
        })
        .unwrap_or_else(|| {
            relative_path
                .file_name()
                .strip_suffix(".md")
                .unwrap_or(relative_path.file_name())
                .to_string()
        })
}

fn extract_wikilinks(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find("]]") else {
            break;
        };
        let raw_target = rest[..end]
            .split('|')
            .next()
            .unwrap_or_default()
            .split('#')
            .next()
            .unwrap_or_default()
            .trim();
        if !raw_target.is_empty()
            && raw_target.chars().count() <= MAX_METADATA_TEXT_CHARS
            && !raw_target.chars().any(char::is_control)
            && !links.iter().any(|existing| existing == raw_target)
        {
            links.push(raw_target.to_string());
        }
        rest = &rest[end + 2..];
    }
    links
}

fn add_lookup_key(lookup: &mut BTreeMap<String, BTreeSet<usize>>, key: &str, index: usize) {
    if !key.is_empty() {
        lookup.entry(key.to_string()).or_default().insert(index);
    }
}

fn build_links_and_backlinks(state: &mut BuildState) {
    let mut lookup = BTreeMap::<String, BTreeSet<usize>>::new();
    for projection in &state.markdown {
        let entry = &state.entries[projection.entry_index];
        add_lookup_key(&mut lookup, &entry.relative_path, projection.entry_index);
        if let Some(stem) = entry.relative_path.strip_suffix(".md") {
            add_lookup_key(&mut lookup, stem, projection.entry_index);
        }
        if let Some(title) = &entry.title {
            add_lookup_key(&mut lookup, title, projection.entry_index);
        }
        for alias in &entry.aliases {
            add_lookup_key(&mut lookup, alias, projection.entry_index);
        }
    }

    let link_jobs: Vec<(usize, Vec<String>)> = state
        .markdown
        .iter()
        .map(|projection| (projection.entry_index, projection.raw_links.clone()))
        .collect();
    for (source_index, raw_links) in link_jobs {
        let source_path = state.entries[source_index].relative_path.clone();
        for raw_link in &raw_links {
            let Some(candidates) = lookup.get(raw_link) else {
                continue;
            };
            if candidates.len() != 1 {
                let source_relative = ValidatedVaultRelativePath::parse(&source_path).ok();
                state.diagnostic(
                    "knowledge_workspace_ambiguous_wikilink",
                    source_relative.as_ref(),
                    "双链匹配到多个笔记，未任意选择目标。",
                );
                continue;
            }
            let target_index = *candidates.iter().next().expect("one candidate");
            let target_path = state.entries[target_index].relative_path.clone();
            if !state.entries[source_index]
                .outlinks
                .iter()
                .any(|existing| existing == &target_path)
            {
                state.entries[source_index]
                    .outlinks
                    .push(target_path.clone());
            }
            if !state.entries[target_index]
                .backlinks
                .iter()
                .any(|existing| existing == &source_path)
            {
                state.entries[target_index]
                    .backlinks
                    .push(source_path.clone());
            }
        }
    }
    for entry in &mut state.entries {
        entry.outlinks.sort();
        entry.backlinks.sort();
    }
}

fn tags_from_entries(entries: &[KnowledgeWorkspaceEntry]) -> Vec<KnowledgeWorkspaceTag> {
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in entries.iter().filter(|entry| entry.kind == "markdown") {
        for tag in &entry.tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(tag, note_count)| KnowledgeWorkspaceTag { tag, note_count })
        .collect()
}

fn build_index_at(vault_root: &Path) -> Result<BuildState, String> {
    match fs::symlink_metadata(vault_root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BuildState::default())
        }
        Err(_) => {
            return Err(
                "knowledge_workspace_vault_invalid: 无法读取 Syn 自管 vault 根。".to_string(),
            )
        }
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(
                "knowledge_workspace_vault_invalid: Syn 自管 vault 根必须是普通目录。".to_string(),
            )
        }
        Ok(_) => {}
    }
    let mut state = BuildState::default();
    classify_directory(vault_root, None, false, &mut state)?;
    build_links_and_backlinks(&mut state);
    Ok(state)
}

fn snapshot_from_state(state: &BuildState) -> KnowledgeWorkspaceSnapshot {
    KnowledgeWorkspaceSnapshot {
        entries: state.entries.clone(),
        tags: tags_from_entries(&state.entries),
        diagnostics: state.diagnostics.clone(),
    }
}

fn manifest_from_state(state: &BuildState) -> KnowledgeWorkspaceVaultManifest {
    KnowledgeWorkspaceVaultManifest {
        entries: state
            .entries
            .iter()
            .map(|entry| KnowledgeWorkspaceVaultManifestEntry {
                relative_path: entry.relative_path.clone(),
                kind: entry.kind,
                mtime_ms: entry.mtime_ms,
                size_bytes: entry.size_bytes,
            })
            .collect(),
        diagnostics: state.diagnostics.clone(),
    }
}

pub(crate) fn workspace_snapshot_at(
    vault_root: &Path,
) -> Result<KnowledgeWorkspaceSnapshot, String> {
    let state = build_index_at(vault_root)?;
    Ok(snapshot_from_state(&state))
}

pub(crate) fn workspace_vault_manifest_at(
    vault_root: &Path,
) -> Result<KnowledgeWorkspaceVaultManifest, String> {
    let state = build_index_at(vault_root)?;
    Ok(manifest_from_state(&state))
}

fn validate_search_query(query: &str) -> Result<String, String> {
    let query = query.trim();
    if query.is_empty()
        || query.len() > MAX_SEARCH_QUERY_BYTES
        || query.chars().any(char::is_control)
    {
        return Err(
            "knowledge_workspace_invalid_search_query: 搜索词必须是受限的普通文本。".to_string(),
        );
    }
    Ok(query.to_string())
}

fn snippet_for(body: &str, normalized_query: &str) -> String {
    let source = body
        .lines()
        .find(|line| line.to_lowercase().contains(normalized_query))
        .unwrap_or(body);
    let snippet: String = source.trim().chars().take(280).collect();
    if snippet.is_empty() {
        "（空笔记）".to_string()
    } else {
        snippet
    }
}

pub(crate) fn workspace_search_at(
    vault_root: &Path,
    query: &str,
) -> Result<KnowledgeWorkspaceSearchResponse, String> {
    let query = validate_search_query(query)?;
    let normalized_query = query.to_lowercase();
    let state = build_index_at(vault_root)?;
    let mut results = Vec::new();
    for projection in &state.markdown {
        let entry = &state.entries[projection.entry_index];
        let title = entry.title.as_deref().unwrap_or_default();
        let metadata_matches = title.to_lowercase().contains(&normalized_query)
            || entry
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&normalized_query));
        if !metadata_matches
            && !projection
                .searchable_body
                .to_lowercase()
                .contains(&normalized_query)
        {
            continue;
        }
        results.push(KnowledgeWorkspaceSearchResult {
            relative_path: entry.relative_path.clone(),
            title: title.to_string(),
            snippet: snippet_for(&projection.searchable_body, &normalized_query),
            tags: entry.tags.clone(),
            mtime_ms: entry.mtime_ms,
        });
        if results.len() == MAX_SEARCH_RESULTS {
            break;
        }
    }
    results.sort_by(|left, right| {
        left.title
            .cmp(&right.title)
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(KnowledgeWorkspaceSearchResponse {
        query,
        results,
        diagnostics: state.diagnostics,
    })
}

fn require_graph_markdown_path(relative_path: &ValidatedVaultRelativePath) -> Result<(), String> {
    if !relative_path.file_name().ends_with(".md") || relative_path.file_name().len() <= ".md".len()
    {
        return Err(
            "knowledge_workspace_graph_markdown_only: 图谱焦点必须是固定 vault 内的 .md 文件。"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_graph_tag(raw_tag: &str) -> Result<String, String> {
    let tag = safe_scalar(raw_tag).map_err(|_| {
        "knowledge_workspace_invalid_graph_tag: 标签必须是受限的精确普通文本。".to_string()
    })?;
    if tag.len() > MAX_SEARCH_QUERY_BYTES {
        return Err("knowledge_workspace_invalid_graph_tag: 标签超过本阶段受限长度。".to_string());
    }
    Ok(tag)
}

fn markdown_matches_graph_filters(
    entry: &KnowledgeWorkspaceEntry,
    projection: &MarkdownProjection,
    normalized_query: Option<&str>,
    tag: Option<&str>,
) -> bool {
    if let Some(tag) = tag {
        if !entry.tags.iter().any(|entry_tag| entry_tag == tag) {
            return false;
        }
    }
    let Some(normalized_query) = normalized_query else {
        return true;
    };
    entry
        .title
        .as_deref()
        .unwrap_or_default()
        .to_lowercase()
        .contains(normalized_query)
        || entry
            .tags
            .iter()
            .any(|entry_tag| entry_tag.to_lowercase().contains(normalized_query))
        || projection
            .searchable_body
            .to_lowercase()
            .contains(normalized_query)
}

fn graph_node_from_entry(
    entry: &KnowledgeWorkspaceEntry,
) -> Result<KnowledgeWorkspaceGraphNode, String> {
    if entry.kind != "markdown" {
        return Err(
            "knowledge_workspace_graph_invalid_node: 图谱节点必须来自已验证 Markdown。".to_string(),
        );
    }
    let relative_path = ValidatedVaultRelativePath::parse(&entry.relative_path)?;
    require_graph_markdown_path(&relative_path)?;
    let title = entry.title.clone().ok_or_else(|| {
        "knowledge_workspace_graph_invalid_node: 已验证 Markdown 缺少标题投影。".to_string()
    })?;
    Ok(KnowledgeWorkspaceGraphNode {
        id: relative_path.as_str().to_string(),
        relative_path: relative_path.as_str().to_string(),
        title,
        tags: entry.tags.clone(),
    })
}

fn append_graph_diagnostic(
    diagnostics: &mut Vec<KnowledgeWorkspaceDiagnostic>,
    code: &'static str,
    message: &'static str,
) {
    if diagnostics.len() < MAX_DIAGNOSTICS {
        diagnostics.push(KnowledgeWorkspaceDiagnostic {
            code,
            relative_path: None,
            message,
        });
    }
}

pub(crate) fn workspace_graph_at(
    vault_root: &Path,
    request: &KnowledgeWorkspaceGraphRequest,
) -> Result<KnowledgeWorkspaceGraphResponse, String> {
    if let Some(focus_relative_path) = &request.focus_relative_path {
        // 索引扫描之外再走一次 exact resolver：局部焦点绝不靠大小写折叠、符号链接或
        // 已删除条目的路径字符串打开。
        let _ = knowledge_vault::read_workspace_markdown_at(vault_root, focus_relative_path)?;
    }

    let state = build_index_at(vault_root)?;
    let normalized_query = request.query.as_deref().map(str::to_lowercase);
    let markdown_by_path: BTreeMap<String, usize> = state
        .markdown
        .iter()
        .map(|projection| {
            (
                state.entries[projection.entry_index].relative_path.clone(),
                projection.entry_index,
            )
        })
        .collect();
    let filtered_indices: BTreeSet<usize> = state
        .markdown
        .iter()
        .filter_map(|projection| {
            let entry = &state.entries[projection.entry_index];
            markdown_matches_graph_filters(
                entry,
                projection,
                normalized_query.as_deref(),
                request.tag.as_deref(),
            )
            .then_some(projection.entry_index)
        })
        .collect();

    let selected_indices = match request.scope {
        KnowledgeWorkspaceGraphScope::Global => filtered_indices,
        KnowledgeWorkspaceGraphScope::Local => {
            let focus_relative_path = request
                .focus_relative_path
                .as_ref()
                .expect("local graph request validates its focus path");
            let focus_index = markdown_by_path
                .get(focus_relative_path.as_str())
                .copied()
                .ok_or_else(|| {
                    "knowledge_workspace_graph_focus_not_indexable: 焦点 Markdown 未通过受限索引校验。"
                        .to_string()
                })?;
            if !filtered_indices.contains(&focus_index) {
                return Err(
                    "knowledge_workspace_graph_focus_filtered: 局部图焦点不满足当前搜索或标签筛选。"
                        .to_string(),
                );
            }
            let mut local_indices = BTreeSet::from([focus_index]);
            let focus_entry = &state.entries[focus_index];
            for neighbor_path in focus_entry.outlinks.iter().chain(&focus_entry.backlinks) {
                let Some(neighbor_index) = markdown_by_path.get(neighbor_path).copied() else {
                    continue;
                };
                if filtered_indices.contains(&neighbor_index) {
                    local_indices.insert(neighbor_index);
                }
            }
            local_indices
        }
    };

    let mut sorted_indices: Vec<usize> = selected_indices.into_iter().collect();
    sorted_indices.sort_by(|left, right| {
        state.entries[*left]
            .relative_path
            .cmp(&state.entries[*right].relative_path)
    });

    let mut diagnostics = state.diagnostics.clone();
    let mut truncated = state.entry_limit_reported;
    let mut nodes = Vec::new();
    let mut included_paths = BTreeSet::new();
    for entry_index in sorted_indices {
        if nodes.len() == MAX_GRAPH_NODES {
            truncated = true;
            append_graph_diagnostic(
                &mut diagnostics,
                "knowledge_workspace_graph_node_limit",
                "图谱节点超过本阶段上限，剩余节点和相关边未显示。",
            );
            break;
        }
        let node = graph_node_from_entry(&state.entries[entry_index])?;
        included_paths.insert(node.id.clone());
        nodes.push(node);
    }

    let mut edges = Vec::new();
    'edge_sources: for source_path in &included_paths {
        let source_index = markdown_by_path.get(source_path).copied().ok_or_else(|| {
            "knowledge_workspace_graph_invalid_node: 图谱节点不在受限 Markdown 投影中。".to_string()
        })?;
        for target_path in &state.entries[source_index].outlinks {
            if !included_paths.contains(target_path) {
                continue;
            }
            if edges.len() == MAX_GRAPH_EDGES {
                truncated = true;
                append_graph_diagnostic(
                    &mut diagnostics,
                    "knowledge_workspace_graph_edge_limit",
                    "图谱关系超过本阶段上限，剩余关系未显示。",
                );
                break 'edge_sources;
            }
            edges.push(KnowledgeWorkspaceGraphEdge {
                id: format!("{source_path}->{target_path}"),
                source: source_path.clone(),
                target: target_path.clone(),
            });
        }
    }

    Ok(KnowledgeWorkspaceGraphResponse {
        scope: request.scope.as_str(),
        focus_relative_path: request
            .focus_relative_path
            .as_ref()
            .map(|path| path.as_str().to_string()),
        query: request.query.clone(),
        tag: request.tag.clone(),
        nodes,
        edges,
        diagnostics,
        truncated,
    })
}

pub(crate) fn workspace_read_markdown_at(
    vault_root: &Path,
    raw_relative_path: &str,
) -> Result<KnowledgeWorkspaceMarkdownDocument, String> {
    let relative_path = ValidatedVaultRelativePath::parse(raw_relative_path)?;
    let file = knowledge_vault::read_workspace_markdown_at(vault_root, &relative_path)?;
    let parsed = parse_markdown(file.body()).map_err(|_| {
        "knowledge_workspace_invalid_frontmatter: Frontmatter 不符合本阶段安全子集。".to_string()
    })?;
    let state = build_index_at(vault_root)?;
    let entry = state
        .entries
        .iter()
        .find(|entry| entry.relative_path == relative_path.as_str() && entry.kind == "markdown")
        .ok_or_else(|| {
            "knowledge_workspace_markdown_not_indexable: Markdown 未通过受限索引校验。".to_string()
        })?;
    Ok(KnowledgeWorkspaceMarkdownDocument {
        relative_path: relative_path.as_str().to_string(),
        title: entry
            .title
            .clone()
            .unwrap_or_else(|| title_from_markdown(&relative_path, &parsed.searchable_body)),
        body: file.body().to_string(),
        tags: entry.tags.clone(),
        aliases: entry.aliases.clone(),
        properties: entry.properties.clone(),
        outlinks: entry.outlinks.clone(),
        backlinks: entry.backlinks.clone(),
        mtime_ms: file.mtime_ms(),
        content_hash: file.content_hash().to_string(),
    })
}

#[tauri::command]
pub(crate) fn knowledge_workspace_snapshot() -> Result<KnowledgeWorkspaceSnapshot, String> {
    workspace_snapshot_at(&knowledge_vault::workspace_vault_root())
}

#[tauri::command]
pub(crate) fn knowledge_workspace_vault_manifest() -> Result<KnowledgeWorkspaceVaultManifest, String>
{
    workspace_vault_manifest_at(&knowledge_vault::workspace_vault_root())
}

#[tauri::command]
pub(crate) fn knowledge_workspace_search(
    query: String,
) -> Result<KnowledgeWorkspaceSearchResponse, String> {
    workspace_search_at(&knowledge_vault::workspace_vault_root(), &query)
}

#[tauri::command]
pub(crate) fn knowledge_workspace_graph(
    scope: String,
    focus_relative_path: Option<String>,
    query: Option<String>,
    tag: Option<String>,
) -> Result<KnowledgeWorkspaceGraphResponse, String> {
    let request =
        KnowledgeWorkspaceGraphRequest::from_raw(&scope, focus_relative_path, query, tag)?;
    workspace_graph_at(&knowledge_vault::workspace_vault_root(), &request)
}

#[tauri::command]
pub(crate) fn knowledge_workspace_read_markdown(
    relative_path: String,
) -> Result<KnowledgeWorkspaceMarkdownDocument, String> {
    workspace_read_markdown_at(&knowledge_vault::workspace_vault_root(), &relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temp_root(tag: &str) -> PathBuf {
        let sequence = TEST_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-knowledge-index-{tag}-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_markdown(root: &Path, relative: &str, body: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    #[test]
    fn n1_nested_relative_path_contract_starts_red() {
        assert!(
            crate::knowledge_vault::validate_workspace_relative_path("research/plan.md").is_ok()
        );
    }

    #[test]
    fn nested_paths_reject_escape_hidden_options_and_wildcards() {
        for invalid in [
            "",
            "/etc/passwd",
            "research//plan.md",
            "research/../plan.md",
            "research/.draft.md",
            "research/-flag.md",
            "research/plan?.md",
            "research/plan=1.md",
            "research\\plan.md",
            "research/plan\n.md",
        ] {
            assert!(
                ValidatedVaultRelativePath::parse(invalid).is_err(),
                "{invalid}"
            );
        }
        assert_eq!(
            ValidatedVaultRelativePath::parse("research/plan.md")
                .unwrap()
                .as_str(),
            "research/plan.md"
        );
    }

    #[test]
    fn scan_rebuilds_frontmatter_links_backlinks_and_search() {
        let root = temp_root("rebuild");
        write_markdown(
            &root,
            "research/plan.md",
            "---\ntitle: Plan\ntags: [roadmap, syn]\naliases:\n  - Plan Alias\nproperties:\n  status: active\n---\n# Plan\n\nBuild the native workspace.\n",
        );
        write_markdown(
            &root,
            "home.md",
            "# Home\n\nSee [[Plan Alias]] for the workspace plan.\n",
        );
        let snapshot = workspace_snapshot_at(&root).unwrap();
        let plan = snapshot
            .entries
            .iter()
            .find(|entry| entry.relative_path == "research/plan.md")
            .unwrap();
        assert_eq!(plan.title.as_deref(), Some("Plan"));
        assert_eq!(plan.tags, vec!["roadmap", "syn"]);
        assert_eq!(plan.properties.get("status"), Some(&"active".to_string()));
        assert_eq!(plan.backlinks, vec!["home.md"]);
        let home = snapshot
            .entries
            .iter()
            .find(|entry| entry.relative_path == "home.md")
            .unwrap();
        assert_eq!(home.outlinks, vec!["research/plan.md"]);
        assert!(snapshot
            .tags
            .iter()
            .any(|tag| tag.tag == "roadmap" && tag.note_count == 1));

        let search = workspace_search_at(&root, "native workspace").unwrap();
        assert_eq!(search.results.len(), 1);
        assert_eq!(search.results[0].relative_path, "research/plan.md");
        let document = workspace_read_markdown_at(&root, "research/plan.md").unwrap();
        assert_eq!(document.backlinks, vec!["home.md"]);
    }

    #[test]
    fn n3_graph_projection_starts_red_with_validated_links_and_isolated_markdown() {
        let root = temp_root("graph-red");
        write_markdown(
            &root,
            "research/plan.md",
            "---\ntags: [roadmap]\n---\n# Plan\n\nSee [[home]].\n",
        );
        write_markdown(&root, "home.md", "# Home\n\n");
        write_markdown(
            &root,
            "isolated.md",
            "---\ntags: [archive]\n---\n# Isolated\n",
        );

        let graph = workspace_graph_at(
            &root,
            &KnowledgeWorkspaceGraphRequest {
                scope: KnowledgeWorkspaceGraphScope::Global,
                focus_relative_path: None,
                query: None,
                tag: None,
            },
        )
        .unwrap();

        assert!(!graph.truncated);
        assert_eq!(
            graph
                .nodes
                .iter()
                .map(|node| node.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["home.md", "isolated.md", "research/plan.md"]
        );
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].source, "research/plan.md");
        assert_eq!(graph.edges[0].target, "home.md");
    }

    #[test]
    fn graph_projection_keeps_local_neighbors_and_rejects_scope_filter_and_focus_drift() {
        let root = temp_root("graph-local");
        write_markdown(
            &root,
            "research/plan.md",
            "---\ntags: [roadmap]\n---\n# Plan\n\nSee [[home]].\n",
        );
        write_markdown(&root, "home.md", "# Home\n\nSee [[isolated]].\n");
        write_markdown(
            &root,
            "isolated.md",
            "---\ntags: [archive]\n---\n# Isolated\n",
        );

        let local = KnowledgeWorkspaceGraphRequest::from_raw(
            "local",
            Some("research/plan.md".to_string()),
            None,
            None,
        )
        .unwrap();
        let local_graph = workspace_graph_at(&root, &local).unwrap();
        assert_eq!(local_graph.scope, "local");
        assert_eq!(
            local_graph
                .nodes
                .iter()
                .map(|node| node.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["home.md", "research/plan.md"]
        );
        assert_eq!(local_graph.edges.len(), 1);
        assert_eq!(local_graph.edges[0].source, "research/plan.md");

        let exact_tag = KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            None,
            None,
            Some("roadmap".to_string()),
        )
        .unwrap();
        assert_eq!(
            workspace_graph_at(&root, &exact_tag).unwrap().nodes.len(),
            1
        );
        let case_variant_tag = KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            None,
            None,
            Some("Roadmap".to_string()),
        )
        .unwrap();
        assert!(workspace_graph_at(&root, &case_variant_tag)
            .unwrap()
            .nodes
            .is_empty());
        let query = KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            None,
            Some("Home".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(
            workspace_graph_at(&root, &query).unwrap().nodes[0].relative_path,
            "home.md"
        );

        assert!(KnowledgeWorkspaceGraphRequest::from_raw("nearby", None, None, None).is_err());
        assert!(KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            Some("research/plan.md".to_string()),
            None,
            None,
        )
        .is_err());
        assert!(KnowledgeWorkspaceGraphRequest::from_raw("local", None, None, None).is_err());
        assert!(KnowledgeWorkspaceGraphRequest::from_raw(
            "local",
            Some("research/plan.canvas".to_string()),
            None,
            None,
        )
        .is_err());
        assert!(KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            None,
            None,
            Some("[not-a-tag]".to_string()),
        )
        .is_err());
        let case_drift = KnowledgeWorkspaceGraphRequest::from_raw(
            "local",
            Some("research/PLAN.md".to_string()),
            None,
            None,
        )
        .unwrap();
        assert!(workspace_graph_at(&root, &case_drift)
            .unwrap_err()
            .starts_with("knowledge_workspace_case_mismatch:"));
        assert!(KnowledgeWorkspaceGraphRequest::from_raw(
            "global",
            None,
            Some("\n".to_string()),
            None,
        )
        .is_err());
    }

    #[test]
    fn graph_projection_has_deterministic_node_and_edge_caps() {
        let node_root = temp_root("graph-node-cap");
        for index in 0..=MAX_GRAPH_NODES {
            write_markdown(
                &node_root,
                &format!("plain-{index:04}.md"),
                &format!("# Plain {index}\n"),
            );
        }
        let global = KnowledgeWorkspaceGraphRequest::from_raw("global", None, None, None).unwrap();
        let node_capped = workspace_graph_at(&node_root, &global).unwrap();
        assert_eq!(node_capped.nodes.len(), MAX_GRAPH_NODES);
        assert!(node_capped.truncated);
        assert!(node_capped
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "knowledge_workspace_graph_node_limit" }));

        let edge_root = temp_root("graph-edge-cap");
        for index in 0..MAX_GRAPH_NODES {
            let first = (index + 1) % MAX_GRAPH_NODES;
            let second = (index + 2) % MAX_GRAPH_NODES;
            let third = (index + 3) % MAX_GRAPH_NODES;
            write_markdown(
                &edge_root,
                &format!("edge-{index:04}.md"),
                &format!(
                    "# Edge {index}\n\n[[edge-{first:04}]] [[edge-{second:04}]] [[edge-{third:04}]]\n"
                ),
            );
        }
        let edge_capped = workspace_graph_at(&edge_root, &global).unwrap();
        assert_eq!(edge_capped.nodes.len(), MAX_GRAPH_NODES);
        assert_eq!(edge_capped.edges.len(), MAX_GRAPH_EDGES);
        assert!(edge_capped.truncated);
        assert!(edge_capped
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "knowledge_workspace_graph_edge_limit" }));
    }

    #[test]
    fn scan_skips_bad_utf8_frontmatter_and_oversize_files_with_bounded_diagnostics() {
        let root = temp_root("invalid");
        fs::write(root.join("bad-utf8.md"), [0xff_u8, 0xfe_u8]).unwrap();
        write_markdown(
            &root,
            "bad-frontmatter.md",
            "---\ntags: [broken\n---\ntext\n",
        );
        fs::write(
            root.join("too-large.md"),
            vec![b'x'; MAX_MARKDOWN_BYTES as usize + 1],
        )
        .unwrap();
        let snapshot = workspace_snapshot_at(&root).unwrap();
        assert!(snapshot.entries.is_empty());
        let codes: BTreeSet<_> = snapshot
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert!(codes.contains("knowledge_workspace_invalid_utf8"));
        assert!(codes.contains("knowledge_workspace_invalid_frontmatter"));
        assert!(codes.contains("knowledge_workspace_markdown_too_large"));
    }

    #[test]
    fn scan_classifies_canvas_and_bounded_attachments_without_parsing_them() {
        let root = temp_root("opaque");
        fs::write(root.join("map.canvas"), "{not parsed in N1}").unwrap();
        fs::create_dir(root.join("attachments")).unwrap();
        fs::write(root.join("attachments/photo.png"), b"png").unwrap();
        fs::write(root.join("attachments/nope.exe"), b"no").unwrap();
        let snapshot = workspace_snapshot_at(&root).unwrap();
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.relative_path == "map.canvas" && entry.kind == "canvas"));
        assert!(snapshot.entries.iter().any(|entry| {
            entry.relative_path == "attachments/photo.png" && entry.kind == "attachment"
        }));
        assert!(snapshot.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "knowledge_workspace_attachment_type_skipped"
        }));
    }

    #[cfg(unix)]
    #[test]
    fn scan_rejects_a_symlink_root_and_skips_symlink_entries_with_diagnostics() {
        use std::os::unix::fs::symlink;

        let outside = temp_root("root-target");
        write_markdown(&outside, "safe.md", "# Safe\n");
        let parent = temp_root("root-link-parent");
        let root_link = parent.join("vault-link");
        symlink(&outside, &root_link).unwrap();
        assert!(workspace_snapshot_at(&root_link)
            .unwrap_err()
            .starts_with("knowledge_workspace_vault_invalid:"));

        let root = temp_root("entry-link");
        symlink(outside.join("safe.md"), root.join("linked.md")).unwrap();
        let snapshot = workspace_snapshot_at(&root).unwrap();
        assert!(snapshot.entries.is_empty());
        assert!(snapshot
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "knowledge_workspace_symlink_skipped" }));
    }

    #[test]
    fn legacy_slug_is_not_a_nested_path_migration() {
        assert!(crate::knowledge_vault::syn_note_relative_path("top-level").is_ok());
        assert!(crate::knowledge_vault::syn_note_relative_path("research/plan").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn resolver_rejects_ancestor_leaf_symlink_and_case_drift() {
        use std::os::unix::fs::symlink;

        let root = temp_root("symlink");
        fs::create_dir(root.join("research")).unwrap();
        fs::write(root.join("research/Plan.md"), "# Plan").unwrap();
        let case_variant = ValidatedVaultRelativePath::parse("research/plan.md").unwrap();
        assert!(resolve_workspace_path_for_test(&root, &case_variant)
            .unwrap_err()
            .starts_with("knowledge_workspace_case_mismatch:"));

        let outside = temp_root("symlink-outside");
        fs::write(outside.join("secret.md"), "secret").unwrap();
        symlink(&outside, root.join("linked-dir")).unwrap();
        let linked_child = ValidatedVaultRelativePath::parse("linked-dir/secret.md").unwrap();
        assert!(resolve_workspace_path_for_test(&root, &linked_child)
            .unwrap_err()
            .starts_with("knowledge_workspace_symlink_rejected:"));
        symlink(outside.join("secret.md"), root.join("leaf.md")).unwrap();
        let leaf = ValidatedVaultRelativePath::parse("leaf.md").unwrap();
        assert!(resolve_workspace_path_for_test(&root, &leaf)
            .unwrap_err()
            .starts_with("knowledge_workspace_symlink_rejected:"));
    }

    fn resolve_workspace_path_for_test(
        root: &Path,
        relative_path: &ValidatedVaultRelativePath,
    ) -> Result<PathBuf, String> {
        crate::knowledge_vault::resolve_existing_workspace_path(root, relative_path)
    }
}
