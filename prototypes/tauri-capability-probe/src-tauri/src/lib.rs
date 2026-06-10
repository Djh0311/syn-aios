use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone)]
struct ProbeState {
  index_path: PathBuf,
}

#[derive(Serialize)]
struct ProbeSummary {
  index_path: String,
  generated_at: Option<String>,
  project_count: usize,
  thread_count: usize,
  skill_count: usize,
  plugin_count: usize,
  project_action_count: usize,
  rollout_action_count: usize,
  first_project_path: Option<String>,
  first_rollout_path: Option<String>,
  warnings: Vec<String>,
}

#[derive(Default)]
struct AllowedPaths {
  projects: BTreeSet<String>,
  rollouts: BTreeSet<String>,
}

impl ProbeState {
  fn new() -> Self {
    Self {
      index_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../index-kernel/codex-index.json"),
    }
  }
}

#[tauri::command]
fn load_probe_summary(state: tauri::State<'_, ProbeState>) -> Result<ProbeSummary, String> {
  let index = read_index(&state)?;
  Ok(build_summary(&state.index_path, &index))
}

#[tauri::command]
fn copy_indexed_path(path: String, state: tauri::State<'_, ProbeState>) -> Result<String, String> {
  let allowed = allowed_paths(&read_index(&state)?);
  if !allowed.can_copy(&path) {
    return Err("路径不在索引白名单内，已拒绝复制".to_string());
  }
  copy_to_clipboard(&path)?;
  Ok(format!("已复制：{path}"))
}

#[tauri::command]
fn open_indexed_project(path: String, state: tauri::State<'_, ProbeState>) -> Result<String, String> {
  let allowed = allowed_paths(&read_index(&state)?);
  if !allowed.projects.contains(&path) {
    return Err("路径不是索引内项目根目录，已拒绝打开".to_string());
  }
  let path_buf = PathBuf::from(&path);
  if !path_buf.is_dir() {
    return Err("索引路径当前不是可打开目录".to_string());
  }
  run_open(&[path.as_str()])?;
  Ok(format!("已请求打开项目目录：{path}"))
}

#[tauri::command]
fn reveal_indexed_rollout(path: String, state: tauri::State<'_, ProbeState>) -> Result<String, String> {
  let allowed = allowed_paths(&read_index(&state)?);
  if !allowed.rollouts.contains(&path) {
    return Err("路径不是索引内 rollout 文件，已拒绝定位".to_string());
  }
  let path_buf = PathBuf::from(&path);
  if !path_buf.is_file() {
    return Err("索引 rollout 路径当前不是文件".to_string());
  }
  run_open(&["-R", path.as_str()])?;
  Ok(format!("已请求定位 rollout 文件：{path}"))
}

fn read_index(state: &ProbeState) -> Result<Value, String> {
  let text = fs::read_to_string(&state.index_path)
    .map_err(|error| format!("无法读取索引文件 {}：{error}", state.index_path.display()))?;
  serde_json::from_str(&text)
    .map_err(|error| format!("索引 JSON 解析失败 {}：{error}", state.index_path.display()))
}

fn build_summary(index_path: &PathBuf, index: &Value) -> ProbeSummary {
  let allowed = allowed_paths(index);
  ProbeSummary {
    index_path: index_path.display().to_string(),
    generated_at: index
      .get("generated_at")
      .and_then(Value::as_str)
      .map(str::to_string),
    project_count: array_len(index, "projects"),
    thread_count: array_len(index, "threads"),
    skill_count: array_len(index, "skills"),
    plugin_count: array_len(index, "plugins"),
    project_action_count: allowed.projects.len(),
    rollout_action_count: allowed.rollouts.len(),
    first_project_path: allowed.projects.iter().next().cloned(),
    first_rollout_path: allowed.rollouts.iter().next().cloned(),
    warnings: collect_warning_labels(index),
  }
}

fn array_len(index: &Value, key: &str) -> usize {
  index.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn allowed_paths(index: &Value) -> AllowedPaths {
  let mut allowed = AllowedPaths::default();

  if let Some(projects) = index.get("projects").and_then(Value::as_array) {
    for project in projects {
      if let Some(path) = project.get("project_root").and_then(Value::as_str) {
        allowed.projects.insert(path.to_string());
      }
    }
  }

  if let Some(threads) = index.get("threads").and_then(Value::as_array) {
    for thread in threads {
      if let Some(path) = thread.get("rollout_path").and_then(Value::as_str) {
        allowed.rollouts.insert(path.to_string());
      }
    }
  }

  allowed
}

fn collect_warning_labels(index: &Value) -> Vec<String> {
  index
    .get("warnings")
    .and_then(Value::as_array)
    .map(|warnings| {
      warnings
        .iter()
        .filter_map(|warning| {
          warning
            .get("code")
            .or_else(|| warning.get("label"))
            .and_then(Value::as_str)
            .map(str::to_string)
        })
        .collect()
    })
    .unwrap_or_default()
}

impl AllowedPaths {
  fn can_copy(&self, path: &str) -> bool {
    self.projects.contains(path) || self.rollouts.contains(path)
  }
}

#[cfg(target_os = "macos")]
fn copy_to_clipboard(text: &str) -> Result<(), String> {
  let mut child = Command::new("pbcopy")
    .stdin(Stdio::piped())
    .spawn()
    .map_err(|error| format!("启动 pbcopy 失败：{error}"))?;

  let stdin = child
    .stdin
    .as_mut()
    .ok_or_else(|| "无法写入 pbcopy stdin".to_string())?;
  stdin
    .write_all(text.as_bytes())
    .map_err(|error| format!("写入剪贴板失败：{error}"))?;

  let status = child
    .wait()
    .map_err(|error| format!("等待 pbcopy 结束失败：{error}"))?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("pbcopy 退出失败：{status}"))
  }
}

#[cfg(not(target_os = "macos"))]
fn copy_to_clipboard(_text: &str) -> Result<(), String> {
  Err("当前探针只验证 macOS 剪贴板能力".to_string())
}

#[cfg(target_os = "macos")]
fn run_open(args: &[&str]) -> Result<(), String> {
  let status = Command::new("open")
    .args(args)
    .status()
    .map_err(|error| format!("启动 open 失败：{error}"))?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("open 退出失败：{status}"))
  }
}

#[cfg(not(target_os = "macos"))]
fn run_open(_args: &[&str]) -> Result<(), String> {
  Err("当前探针只验证 macOS 打开和定位能力".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(ProbeState::new())
    .invoke_handler(tauri::generate_handler![
      load_probe_summary,
      copy_indexed_path,
      open_indexed_project,
      reveal_indexed_rollout
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn extracts_only_indexed_project_and_rollout_paths() {
    let index = json!({
      "projects": [
        { "project_root": "/Users/yoyi/workspace" },
        { "project_root": null }
      ],
      "threads": [
        { "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl" },
        { "rollout_path": 12 }
      ]
    });

    let allowed = allowed_paths(&index);

    assert!(allowed.projects.contains("/Users/yoyi/workspace"));
    assert!(allowed.rollouts.contains("/Users/yoyi/.codex/sessions/sample.jsonl"));
    assert!(!allowed.can_copy("/Users/yoyi/.codex/auth.json"));
  }

  #[test]
  fn builds_summary_without_reading_session_body() {
    let index = json!({
      "generated_at": "2026-05-27T10:23:52Z",
      "projects": [{ "project_root": "/Users/yoyi/workspace" }],
      "threads": [{ "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl" }],
      "skills": [{ "name": "one" }],
      "plugins": [{ "name": "plugin" }],
      "warnings": [{ "code": "title_truncated" }]
    });

    let summary = build_summary(&PathBuf::from("/tmp/codex-index.json"), &index);

    assert_eq!(summary.project_count, 1);
    assert_eq!(summary.thread_count, 1);
    assert_eq!(summary.project_action_count, 1);
    assert_eq!(summary.rollout_action_count, 1);
    assert_eq!(summary.warnings, vec!["title_truncated".to_string()]);
  }

  #[test]
  fn reads_real_static_index_summary() {
    let state = ProbeState::new();
    let index = read_index(&state).expect("static index should be readable");
    let summary = build_summary(&state.index_path, &index);

    assert!(summary.project_count > 0);
    assert!(summary.thread_count > 0);
    assert!(summary.project_action_count > 0);
    assert!(summary.rollout_action_count > 0);
  }
}
