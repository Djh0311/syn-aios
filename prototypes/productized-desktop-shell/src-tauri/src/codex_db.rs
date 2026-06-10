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

#[derive(Deserialize)]
struct SessionIndexRow {
    id: String,
    thread_name: Option<String>,
}

pub fn default_state_db_path() -> PathBuf {
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
        WHERE has_user_event = 1
        ORDER BY updated_at_ms DESC, id DESC
    "#;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("准备查询失败：{e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let thread_id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let cwd: String = row.get(2)?;
            let updated_at_ms: Option<i64> = row.get(3)?;
            let archived_int: i64 = row.get(4)?;
            let rollout_path: String = row.get(5)?;
            let model: Option<String> = row.get(6)?;
            let reasoning_effort: Option<String> = row.get(7)?;
            let thread_source: Option<String> = row.get(8)?;

            let project_root = if cwd.trim().is_empty() {
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
                title,
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
        })
        .map_err(|e| format!("执行查询失败：{e}"))?;

    let display_titles = read_session_index_titles(db_path);
    let mut out = Vec::new();
    for row in rows {
        let mut thread = row.map_err(|e| format!("行解码失败：{e}"))?;
        if let Some(title) = display_titles.get(&thread.thread_id) {
            thread.title = title.clone();
        }
        out.push(thread);
    }
    Ok(out)
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
}
