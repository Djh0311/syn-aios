// Direct read from codex's own sqlite (~/.codex/state_*.sqlite).
// Replaces the python build_index.py middleman so newly-created sessions show up immediately.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CodexThreadRow {
    pub thread_id: String,
    pub title: String,
    pub project_root: Option<String>,
    pub updated_at_ms: Option<i64>,
    pub archived: bool,
    pub rollout_exists: bool,
    pub rollout_path: Option<String>,
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub thread_source: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CodexThreadPage {
    pub rows: Vec<CodexThreadRow>,
    pub page_size: usize,
    pub offset: usize,
    pub has_more: bool,
    pub include_archived: bool,
    pub archived_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodexThreadPageOptions {
    pub page_size: usize,
    pub offset: usize,
    pub include_archived: bool,
    pub archived_only: bool,
    pub query: Option<String>,
}

impl Default for CodexThreadPageOptions {
    fn default() -> Self {
        Self {
            page_size: 100,
            offset: 0,
            include_archived: false,
            archived_only: false,
            query: None,
        }
    }
}

#[derive(Deserialize)]
struct SessionIndexRow {
    id: String,
    thread_name: Option<String>,
}

pub fn default_state_db_path() -> PathBuf {
    if let Some(paths) = crate::acceptance_runtime_profile::active_paths()
        .expect("acceptance runtime profile must resolve before Codex DB path use")
    {
        return paths.codex_db_path;
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    latest_state_db_path(&home.join(".codex"))
        .unwrap_or_else(|| home.join(".codex").join("state_5.sqlite"))
}

fn latest_state_db_path(codex_dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(codex_dir).ok()?;
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            let version = file_name
                .strip_prefix("state_")?
                .strip_suffix(".sqlite")?
                .parse::<u32>()
                .ok()?;
            Some((version, path))
        })
        .max_by_key(|(version, _)| *version)
        .map(|(_, path)| path)
}

/// Read all threads from codex's sqlite.
/// Skips threads where has_user_event=0 (codex desktop hides those too — they're empty placeholders).
pub fn read_threads(db_path: &Path) -> Result<Vec<CodexThreadRow>, String> {
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let page = read_threads_page(
            db_path,
            CodexThreadPageOptions {
                page_size: 250,
                offset,
                include_archived: true,
                archived_only: false,
                query: None,
            },
        )?;
        offset += page.rows.len();
        rows.extend(page.rows);
        if !page.has_more {
            return Ok(rows);
        }
    }
}

/// 按主键精确查一条 thread（**存在性语义**）。与 read_threads_page 的**列表显示过滤**
/// （has_user_event=1、过滤 subagent、归档开关）刻意不同：`codex exec` 产的会话
/// has_user_event=0，在会话列表里合理隐藏，但「按 id 校验这条会话是否真实存在」时必须
/// 找得到——2026-07-05 方案a 真跑逮到：交办新会话（relay new_session=codex exec）出生后
/// 绑定被「会话不在当前索引内」拒，根因就是存在性校验借用了列表查询。只读、不动列表查询本体。
pub fn find_thread_by_id(
    db_path: &Path,
    thread_id: &str,
) -> Result<Option<CodexThreadRow>, String> {
    if !db_path.exists() {
        return Err(format!("找不到 codex 状态库：{}", db_path.display()));
    }
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| format!("打开 sqlite 失败 {}：{e}", db_path.display()))?;
    let sql = r#"
        SELECT
            id,
            COALESCE(NULLIF(title, ''), '未命名会话') AS title,
            cwd,
            updated_at_ms,
            archived,
            rollout_path,
            model,
            reasoning_effort,
            thread_source
        FROM threads
        WHERE id = ?1
        LIMIT 1
    "#;
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("准备查询失败：{e}"))?;
    let mut rows = stmt
        .query_map(rusqlite::params![thread_id], decode_thread_row)
        .map_err(|e| format!("执行查询失败：{e}"))?;
    match rows.next() {
        None => Ok(None),
        Some(row) => {
            let mut thread = row.map_err(|e| format!("行解码失败：{e}"))?;
            let display_titles = read_session_index_titles(db_path);
            if let Some(title) = display_titles.get(&thread.thread_id) {
                thread.title = truncate_display_title(title);
            }
            Ok(Some(thread))
        }
    }
}

/// Read one page of non-archived threads by default.
/// This keeps session shell loading bounded while preserving the older read_threads() helper.
pub fn read_threads_page(
    db_path: &Path,
    options: CodexThreadPageOptions,
) -> Result<CodexThreadPage, String> {
    if !db_path.exists() {
        return Err(format!("找不到 codex 状态库：{}", db_path.display()));
    }
    let page_size = options.page_size.clamp(1, 250);
    let offset = options.offset;

    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| format!("打开 sqlite 失败 {}：{e}", db_path.display()))?;

    let archived_clause = if options.archived_only {
        "AND archived = 1"
    } else if options.include_archived {
        ""
    } else {
        "AND archived = 0"
    };
    let query = options
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", value.to_lowercase()));
    let query_clause = if query.is_some() {
        r#"
        AND (
            lower(id) LIKE ?1
            OR lower(COALESCE(title, '')) LIKE ?1
            OR lower(COALESCE(cwd, '')) LIKE ?1
            OR lower(COALESCE(rollout_path, '')) LIKE ?1
            OR lower(COALESCE(model, '')) LIKE ?1
            OR lower(COALESCE(reasoning_effort, '')) LIKE ?1
            OR lower(COALESCE(thread_source, '')) LIKE ?1
        )
        "#
    } else {
        ""
    };
    let (limit_param, offset_param) = if query.is_some() {
        ("?2", "?3")
    } else {
        ("?1", "?2")
    };
    let sql = format!(
        r#"
        SELECT
            id,
            COALESCE(NULLIF(title, ''), '未命名会话') AS title,
            cwd,
            updated_at_ms,
            archived,
            rollout_path,
            model,
            reasoning_effort,
            thread_source
        FROM threads
        WHERE has_user_event = 1
        AND source NOT LIKE '%subagent%'
        {archived_clause}
        {query_clause}
        ORDER BY updated_at_ms DESC, id DESC
        LIMIT {limit_param} OFFSET {offset_param}
    "#
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("准备查询失败：{e}"))?;

    let display_titles = read_session_index_titles(db_path);
    let mut out = Vec::new();
    if let Some(query) = query {
        let rows = stmt
            .query_map(
                rusqlite::params![query, page_size as i64 + 1, offset as i64],
                decode_thread_row,
            )
            .map_err(|e| format!("执行查询失败：{e}"))?;
        for row in rows {
            let mut thread = row.map_err(|e| format!("行解码失败：{e}"))?;
            if let Some(title) = display_titles.get(&thread.thread_id) {
                thread.title = truncate_display_title(title);
            }
            out.push(thread);
        }
    } else {
        let rows = stmt
            .query_map(
                rusqlite::params![page_size as i64 + 1, offset as i64],
                decode_thread_row,
            )
            .map_err(|e| format!("执行查询失败：{e}"))?;
        for row in rows {
            let mut thread = row.map_err(|e| format!("行解码失败：{e}"))?;
            if let Some(title) = display_titles.get(&thread.thread_id) {
                thread.title = truncate_display_title(title);
            }
            out.push(thread);
        }
    }
    let has_more = out.len() > page_size;
    out.truncate(page_size);
    Ok(CodexThreadPage {
        rows: out,
        page_size,
        offset,
        has_more,
        include_archived: options.include_archived,
        archived_only: options.archived_only,
    })
}

/// cwd prefixes codex uses as scratch / "open a chat without a project" roots.
/// Threads rooted here are direct chats, not project work, so they collapse into
/// the None project bucket (codex's own unified "direct chat" list). Kept as a
/// named list — user-specified for 2026-06-20 (codex's default new-chat area
/// under Documents/Codex); add prefixes here or swap for a more general signal
/// later without touching the derivation logic.
const NO_PROJECT_PATH_PREFIXES: &[&str] = &["/Users/yoyi/Documents/Codex"];

/// A cwd is "no project" when it equals one of the scratch roots or sits anywhere
/// beneath it. Everything else (named project dirs, including ~/workspace) keeps
/// its own project_root.
fn is_no_project_cwd(cwd: &str) -> bool {
    let trimmed = cwd.trim();
    NO_PROJECT_PATH_PREFIXES
        .iter()
        .any(|prefix| trimmed == *prefix || trimmed.starts_with(&format!("{prefix}/")))
}

/// codex stores the entire first user message in `threads.title` (observed mean
/// ~11k chars, max ~76k). Collapse it to a single display line capped at 120
/// chars so the read model payload stays small and the UI never has to render a
/// multi-kilobyte title. Search still runs against the full title column in SQL,
/// so this only trims what is shown, not what is matched.
fn truncate_display_title(raw: &str) -> String {
    const MAX_CHARS: usize = 120;
    let trimmed = raw.trim();
    let first_line = trimmed.lines().next().unwrap_or(trimmed).trim();
    let base = if first_line.is_empty() {
        trimmed
    } else {
        first_line
    };
    let is_single_line = base.chars().count() == trimmed.chars().count();
    if is_single_line && base.chars().count() <= MAX_CHARS {
        return base.to_string();
    }
    let truncated: String = base.chars().take(MAX_CHARS).collect();
    format!("{truncated}…")
}

fn decode_thread_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CodexThreadRow> {
    let thread_id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let cwd: String = row.get(2)?;
    let updated_at_ms: Option<i64> = row.get(3)?;
    let archived_int: i64 = row.get(4)?;
    let rollout_path: String = row.get(5)?;
    let model: Option<String> = row.get(6)?;
    let reasoning_effort: Option<String> = row.get(7)?;
    let thread_source: Option<String> = row.get(8)?;

    let project_root = if cwd.trim().is_empty() || is_no_project_cwd(&cwd) {
        None
    } else {
        Some(cwd)
    };
    let rollout_exists = !rollout_path.is_empty() && Path::new(&rollout_path).exists();

    let mut warnings = Vec::new();
    if rollout_path.is_empty() {
        warnings.push("rollout_path_empty".to_string());
    } else if !rollout_exists {
        warnings.push("rollout_missing_on_disk".to_string());
    }

    Ok(CodexThreadRow {
        thread_id,
        title: truncate_display_title(&title),
        project_root,
        updated_at_ms,
        archived: archived_int != 0,
        rollout_exists,
        rollout_path: if rollout_path.is_empty() {
            None
        } else {
            Some(rollout_path)
        },
        model,
        reasoning_effort,
        thread_source,
        warnings,
    })
}

pub fn has_parent_session_id_column(db_path: &Path) -> Result<bool, String> {
    if !db_path.exists() {
        return Err(format!("找不到 codex 状态库：{}", db_path.display()));
    }
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("打开 sqlite 失败 {}：{e}", db_path.display()))?;
    let mut stmt = conn
        .prepare("PRAGMA table_info(threads)")
        .map_err(|e| format!("准备 threads schema 查询失败：{e}"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("读取 threads schema 失败：{e}"))?;
    for row in rows {
        if row.map_err(|e| format!("读取 threads schema row 失败：{e}"))? == "parent_session_id"
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn read_session_index_titles(db_path: &Path) -> HashMap<String, String> {
    let Some(codex_dir) = db_path.parent() else {
        return HashMap::new();
    };
    let path = codex_dir.join("session_index.jsonl");
    let Ok(text) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str::<SessionIndexRow>(line).ok())
        .filter_map(|row| {
            let title = row.thread_name?.trim().to_string();
            if title.is_empty() {
                return None;
            }
            Some((row.id, title))
        })
        .collect()
}

/// Group threads by project_root. Threads with no cwd land in the None bucket.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ProjectThreadGroup {
    pub project_root: Option<String>,
    pub thread_count: usize,
    pub active_thread_count: usize,
    pub archived_thread_count: usize,
    pub latest_updated_at_ms: Option<i64>,
}

pub fn group_by_project(threads: &[CodexThreadRow]) -> Vec<ProjectThreadGroup> {
    let mut by_root: std::collections::BTreeMap<Option<String>, ProjectThreadGroup> =
        std::collections::BTreeMap::new();
    for t in threads {
        let entry = by_root
            .entry(t.project_root.clone())
            .or_insert_with(|| ProjectThreadGroup {
                project_root: t.project_root.clone(),
                thread_count: 0,
                active_thread_count: 0,
                archived_thread_count: 0,
                latest_updated_at_ms: None,
            });
        entry.thread_count += 1;
        if t.archived {
            entry.archived_thread_count += 1;
        } else {
            entry.active_thread_count += 1;
        }
        if let Some(ms) = t.updated_at_ms {
            entry.latest_updated_at_ms = Some(entry.latest_updated_at_ms.map_or(ms, |c| c.max(ms)));
        }
    }
    let mut groups: Vec<_> = by_root.into_values().collect();
    groups.sort_by(|a, b| {
        b.latest_updated_at_ms
            .unwrap_or(0)
            .cmp(&a.latest_updated_at_ms.unwrap_or(0))
    });
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn read_threads_prefers_session_index_thread_name() {
        let dir = temp_dir("codex-session-title");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        fs::write(
            dir.join("session_index.jsonl"),
            r#"{"id":"thread-1","thread_name":"用户重命名标题","updated_at":"2026-06-02T00:00:00Z"}"#,
        )
        .expect("write session index");

        let rows = read_threads(&db_path).expect("read threads");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].thread_id, "thread-1");
        assert_eq!(rows[0].title, "用户重命名标题");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_falls_back_to_sqlite_title_without_session_index() {
        let dir = temp_dir("codex-session-title-fallback");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);

        let rows = read_threads(&db_path).expect("read threads");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "第一句话标题");
        let _ = fs::remove_dir_all(dir);
    }

    // 方案a 根因回归（2026-07-05）：`codex exec` 产的会话 has_user_event=0 → 列表查询
    // （read_threads·显示过滤）合理看不见；但按 id 精确查（find_thread_by_id·存在性语义）必须找得到。
    #[test]
    fn find_thread_by_id_sees_exec_thread_hidden_from_list() {
        let dir = temp_dir("codex-find-by-id");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        insert_thread(&db_path, "thread-exec", "exec 会话", 5_000, 0, 0);
        let listed = read_threads(&db_path).expect("read threads");
        assert!(
            listed.iter().all(|row| row.thread_id != "thread-exec"),
            "列表查询应看不见 has_user_event=0 的 exec 会话（显示过滤原样）"
        );
        let found = find_thread_by_id(&db_path, "thread-exec")
            .expect("query ok")
            .expect("按 id 应找到 exec 会话");
        assert_eq!(found.thread_id, "thread-exec");
        assert!(
            find_thread_by_id(&db_path, "thread-none")
                .expect("query ok")
                .is_none(),
            "不存在的 id 返回 None"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_page_filters_archived_and_limits_rows() {
        let dir = temp_dir("codex-session-page");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        insert_thread(&db_path, "thread-2", "第二条", 2_000, 0, 1);
        insert_thread(&db_path, "thread-3", "第三条归档", 3_000, 1, 1);
        insert_thread(&db_path, "thread-4", "第四条占位", 4_000, 0, 0);

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 1,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: None,
            },
        )
        .expect("read page");

        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].thread_id, "thread-2");
        assert!(!page.rows[0].archived);
        assert!(page.has_more);
        assert!(!page.include_archived);

        let next_page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 10,
                offset: 1,
                include_archived: false,
                archived_only: false,
                query: None,
            },
        )
        .expect("read next page");
        assert_eq!(
            next_page
                .rows
                .iter()
                .map(|row| row.thread_id.as_str())
                .collect::<Vec<_>>(),
            vec!["thread-1"]
        );
        assert!(!next_page.has_more);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_page_can_include_archived_for_explicit_archive_view() {
        let dir = temp_dir("codex-session-page-archived");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        insert_thread(&db_path, "thread-archived", "归档条目", 2_000, 1, 1);

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 10,
                offset: 0,
                include_archived: true,
                archived_only: false,
                query: None,
            },
        )
        .expect("read archived page");

        assert_eq!(
            page.rows
                .iter()
                .map(|row| (row.thread_id.as_str(), row.archived))
                .collect::<Vec<_>>(),
            vec![("thread-archived", true), ("thread-1", false)]
        );
        assert!(page.include_archived);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_page_can_target_archived_only() {
        let dir = temp_dir("codex-session-page-archived-only");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        insert_thread(&db_path, "thread-archived", "归档条目", 2_000, 1, 1);

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 10,
                offset: 0,
                include_archived: false,
                archived_only: true,
                query: None,
            },
        )
        .expect("read archived-only page");

        assert_eq!(
            page.rows
                .iter()
                .map(|row| (row.thread_id.as_str(), row.archived))
                .collect::<Vec<_>>(),
            vec![("thread-archived", true)]
        );
        assert!(page.archived_only);
        assert!(!page.include_archived);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_page_searches_visible_threads_by_id_title_and_project() {
        let dir = temp_dir("codex-session-page-query");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        insert_thread_with_cwd(
            &db_path,
            "older-visible-thread",
            "Stage K visible",
            "/tmp/stage-k",
            2_000,
            0,
            1,
        );
        insert_thread_with_cwd(
            &db_path,
            "hidden-exec-thread",
            "Stage K hidden",
            "/tmp/stage-k",
            3_000,
            0,
            0,
        );

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 10,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: Some("older-visible".to_string()),
            },
        )
        .expect("read queried page");
        assert_eq!(
            page.rows
                .iter()
                .map(|row| row.thread_id.as_str())
                .collect::<Vec<_>>(),
            vec!["older-visible-thread"]
        );

        let project_page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 10,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: Some("stage-k".to_string()),
            },
        )
        .expect("read project query page");
        assert!(
            project_page
                .rows
                .iter()
                .all(|row| row.thread_id != "hidden-exec-thread"),
            "query must not surface has_user_event=0 placeholder threads"
        );
        assert!(
            project_page
                .rows
                .iter()
                .any(|row| row.thread_id == "older-visible-thread"),
            "project query should find visible older sessions without loading prior pages"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_parent_session_id_column_from_fixture_schema_only() {
        let dir = temp_dir("codex-parent-session-column");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        assert!(
            !has_parent_session_id_column(&db_path).expect("detect missing parent column"),
            "baseline fixture should not pretend subagent folding is available"
        );

        let conn = Connection::open(&db_path).expect("open sqlite");
        conn.execute("ALTER TABLE threads ADD COLUMN parent_session_id TEXT", [])
            .expect("add parent_session_id");
        assert!(
            has_parent_session_id_column(&db_path).expect("detect parent column"),
            "schema discovery should notice parent_session_id when present"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_threads_legacy_helper_still_reads_all_pages() {
        let dir = temp_dir("codex-read-threads-all-pages");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        for index in 0..260 {
            insert_thread(
                &db_path,
                &format!("bulk-thread-{index}"),
                &format!("Bulk thread {index}"),
                10_000 + index,
                0,
                1,
            );
        }

        let rows = read_threads(&db_path).expect("read all rows");
        assert_eq!(rows.len(), 261);
        assert!(
            rows.iter().any(|row| row.thread_id == "bulk-thread-259"),
            "legacy helper must not be capped at the first page"
        );
        assert!(
            rows.iter().any(|row| row.thread_id == "thread-1"),
            "legacy helper should retain older rows beyond the first page"
        );
        let _ = fs::remove_dir_all(dir);
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    fn create_threads_db(path: &Path) {
        let conn = Connection::open(path).expect("open sqlite");
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                cwd TEXT NOT NULL,
                updated_at_ms INTEGER,
                archived INTEGER NOT NULL,
                rollout_path TEXT NOT NULL,
                model TEXT,
                reasoning_effort TEXT,
                thread_source TEXT,
                source TEXT NOT NULL DEFAULT 'cli',
                has_user_event INTEGER NOT NULL
            );
            INSERT INTO threads (
                id,
                title,
                cwd,
                updated_at_ms,
                archived,
                rollout_path,
                model,
                reasoning_effort,
                thread_source,
                has_user_event
            ) VALUES (
                'thread-1',
                '第一句话标题',
                '/tmp/project',
                1000,
                0,
                '',
                'gpt-test',
                'medium',
                'codex',
                1
            );
            "#,
        )
        .expect("create threads table");
    }

    fn insert_thread(
        db_path: &Path,
        thread_id: &str,
        title: &str,
        updated_at_ms: i64,
        archived: i64,
        has_user_event: i64,
    ) {
        insert_thread_with_cwd(
            db_path,
            thread_id,
            title,
            "/tmp/project",
            updated_at_ms,
            archived,
            has_user_event,
        );
    }

    fn insert_thread_with_cwd(
        db_path: &Path,
        thread_id: &str,
        title: &str,
        cwd: &str,
        updated_at_ms: i64,
        archived: i64,
        has_user_event: i64,
    ) {
        let conn = Connection::open(db_path).expect("open sqlite");
        conn.execute(
            r#"
            INSERT INTO threads (
                id,
                title,
                cwd,
                updated_at_ms,
                archived,
                rollout_path,
                model,
                reasoning_effort,
                thread_source,
                has_user_event
            ) VALUES (?1, ?2, ?3, ?4, ?5, '', 'gpt-test', 'medium', 'codex', ?6)
            "#,
            (
                thread_id,
                title,
                cwd,
                updated_at_ms,
                archived,
                has_user_event,
            ),
        )
        .expect("insert thread");
    }

    fn insert_thread_with_source(
        db_path: &Path,
        thread_id: &str,
        title: &str,
        source: &str,
        updated_at_ms: i64,
        has_user_event: i64,
    ) {
        let conn = Connection::open(db_path).expect("open sqlite");
        conn.execute(
            r#"
            INSERT INTO threads (
                id,
                title,
                cwd,
                updated_at_ms,
                archived,
                rollout_path,
                model,
                reasoning_effort,
                thread_source,
                source,
                has_user_event
            ) VALUES (?1, ?2, '/tmp/project', ?3, 0, '', 'gpt-test', 'medium', 'codex', ?4, ?5)
            "#,
            (thread_id, title, updated_at_ms, source, has_user_event),
        )
        .expect("insert thread with source");
    }

    #[test]
    fn read_threads_hides_subagent_source() {
        let dir = temp_dir("codex-session-subagent-filter");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path);
        // thread-1 (from fixture) has source 'cli'. Add a real top-level session
        // and several subagent child threads — the subagents must be hidden.
        insert_thread_with_source(&db_path, "real-vscode", "真实顶层会话", "vscode", 5_000, 1);
        insert_thread_with_source(
            &db_path,
            "subagent-guardian",
            "guardian subagent noise",
            r#"{"subagent":{"other":"guardian"}}"#,
            6_000,
            1,
        );
        insert_thread_with_source(
            &db_path,
            "subagent-worker",
            "worker subagent noise",
            r#"{"subagent":{"thread_spawn":{"role":"worker"}}}"#,
            7_000,
            1,
        );

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 100,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: None,
            },
        )
        .expect("read page");

        let ids: Vec<&str> = page.rows.iter().map(|row| row.thread_id.as_str()).collect();
        assert!(
            ids.contains(&"real-vscode"),
            "top-level vscode session must stay"
        );
        assert!(ids.contains(&"thread-1"), "fixture cli session must stay");
        assert!(
            !ids.iter().any(|id| id.starts_with("subagent-")),
            "subagent child threads must be hidden from the session list: {ids:?}",
        );
        assert_eq!(
            page.rows.len(),
            2,
            "only the two non-subagent sessions remain"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn truncate_display_title_collapses_long_first_message() {
        let short = "第一句话标题";
        assert_eq!(
            truncate_display_title(short),
            short,
            "short titles pass through unchanged"
        );
        let renamed = "用户重命名标题";
        assert_eq!(
            truncate_display_title(renamed),
            renamed,
            "deliberate short renames untouched"
        );

        let multiline = "首行摘要\n第二行不应进入标题";
        assert_eq!(
            truncate_display_title(multiline),
            "首行摘要…",
            "multi-line collapses to first line"
        );

        let huge: String = "话".repeat(76_000);
        let truncated = truncate_display_title(&huge);
        assert!(truncated.ends_with('…'), "huge title is ellipsised");
        assert!(
            truncated.chars().count() <= 121,
            "huge title capped at 120 chars + ellipsis"
        );
    }

    #[test]
    fn is_no_project_cwd_matches_only_scratch_root_and_descendants() {
        assert!(is_no_project_cwd("/Users/yoyi/Documents/Codex"));
        assert!(is_no_project_cwd(
            "/Users/yoyi/Documents/Codex/2026-05-09/ai"
        ));
        assert!(is_no_project_cwd(
            "  /Users/yoyi/Documents/Codex/new-chat  "
        ));
        // A sibling dir that merely shares the prefix string is NOT under it.
        assert!(!is_no_project_cwd("/Users/yoyi/Documents/CodexProjects"));
        assert!(!is_no_project_cwd("/Users/yoyi/workspace/product-line"));
        assert!(!is_no_project_cwd(""));
    }

    #[test]
    fn read_threads_collapses_documents_codex_into_no_project_bucket() {
        let dir = temp_dir("codex-session-no-project");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("state_5.sqlite");
        create_threads_db(&db_path); // thread-1 cwd '/tmp/project' → real project
        insert_thread_with_cwd(
            &db_path,
            "scratch-root",
            "直接聊天根",
            "/Users/yoyi/Documents/Codex",
            5_000,
            0,
            1,
        );
        insert_thread_with_cwd(
            &db_path,
            "scratch-dated",
            "日期戳暂存",
            "/Users/yoyi/Documents/Codex/2026-05-09/ai",
            6_000,
            0,
            1,
        );
        insert_thread_with_cwd(
            &db_path,
            "real-workspace",
            "真实项目",
            "/Users/yoyi/workspace/product-line",
            7_000,
            0,
            1,
        );

        let page = read_threads_page(
            &db_path,
            CodexThreadPageOptions {
                page_size: 100,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: None,
            },
        )
        .expect("read page");
        let row = |id: &str| {
            page.rows
                .iter()
                .find(|r| r.thread_id == id)
                .unwrap_or_else(|| panic!("row {id} missing"))
        };

        assert_eq!(
            row("scratch-root").project_root,
            None,
            "Documents/Codex root is no-project"
        );
        assert_eq!(
            row("scratch-dated").project_root,
            None,
            "below Documents/Codex is no-project"
        );
        assert_eq!(
            row("real-workspace").project_root,
            Some("/Users/yoyi/workspace/product-line".to_string()),
            "workspace must stay a real project, not collapse into no-project",
        );
        assert_eq!(
            row("thread-1").project_root,
            Some("/tmp/project".to_string()),
            "ordinary project dir keeps its root",
        );

        let _ = fs::remove_dir_all(dir);
    }
}
