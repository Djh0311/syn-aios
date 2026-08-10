use tauri::Manager;

const ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE: i32 = 78;
const ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE: i32 = 79;
const ACCEPTANCE_RUNTIME_PROFILE_FINALIZATION_EXIT_CODE: i32 = 80;

fn exit_acceptance_startup_failure(exit_code: i32) -> ! {
    eprintln!("验收 runtime profile 启动失败");
    std::process::exit(exit_code);
}

fn software_key_of_session(session: &SessionRecord) -> String {
    let key = session
        .thread_source
        .as_deref()
        .unwrap_or("codex")
        .trim()
        .to_ascii_lowercase();
    if key.is_empty() {
        "codex".to_string()
    } else {
        key
    }
}
fn load_sessions(index: &Value, mode: SessionSourceMode) -> (Vec<SessionRecord>, Vec<String>) {
    match mode {
        SessionSourceMode::RealWithSqliteFallback => load_sessions_from_sqlite_or_index(index),
        SessionSourceMode::IndexOnly => (parse_sessions(index), Vec::new()),
    }
}

fn load_sessions_from_sqlite_or_index(index: &Value) -> (Vec<SessionRecord>, Vec<String>) {
    let db_path = codex_db::default_state_db_path();
    match codex_db::read_threads(&db_path) {
        Ok(rows) => {
            let sessions = rows
                .into_iter()
                .map(session_record_from_codex_thread)
                .collect();
            (sessions, Vec::new())
        }
        Err(err) => (
            parse_sessions(index),
            vec![format!("codex sqlite 读取失败，回落到旧索引：{err}")],
        ),
    }
}

fn session_record_from_codex_thread(r: codex_db::CodexThreadRow) -> SessionRecord {
    SessionRecord {
        thread_id: r.thread_id,
        title: r.title,
        project_root: r.project_root,
        updated_at_ms: r.updated_at_ms,
        archived: r.archived,
        rollout_exists: r.rollout_exists,
        rollout_path: r.rollout_path,
        model: r.model,
        reasoning_effort: r.reasoning_effort,
        thread_source: r.thread_source,
        warnings: r.warnings,
        // 基础映射默认非工作台绑定；合并 store 绑定会话那步（commands.rs）才置 true。
        workbench_bound: false,
    }
}

fn overlay_project_thread_counts(projects: &mut Vec<ProjectRecord>, sessions: &[SessionRecord]) {
    use std::collections::HashMap;
    struct Acc {
        total: usize,
        active: usize,
        archived: usize,
        latest: Option<i64>,
    }
    let mut by_root: HashMap<String, Acc> = HashMap::new();
    for s in sessions {
        let Some(root) = s.project_root.as_ref() else {
            continue;
        };
        let entry = by_root.entry(root.clone()).or_insert(Acc {
            total: 0,
            active: 0,
            archived: 0,
            latest: None,
        });
        entry.total += 1;
        if s.archived {
            entry.archived += 1;
        } else {
            entry.active += 1;
        }
        if let Some(ms) = s.updated_at_ms {
            entry.latest = Some(entry.latest.map_or(ms, |c| c.max(ms)));
        }
    }
    for project in projects.iter_mut() {
        if let Some(acc) = by_root.get(&project.project_root) {
            project.thread_count = acc.total;
            project.active_thread_count = acc.active;
            project.archived_thread_count = acc.archived;
            project.latest_updated_at_ms = acc.latest;
        }
    }
}

fn parse_projects(index: &Value) -> Vec<ProjectRecord> {
    index
        .get("projects")
        .and_then(Value::as_array)
        .map(|projects| {
            projects
                .iter()
                .filter_map(|project| {
                    let project_root = optional_string_from(project, "project_root")?;
                    Some(ProjectRecord {
                        name: path_name(&project_root),
                        project_root,
                        active_hint: bool_value(project, "active_hint"),
                        thread_count: usize_value(project, "thread_count"),
                        active_thread_count: usize_value(project, "active_thread_count"),
                        archived_thread_count: usize_value(project, "archived_thread_count"),
                        latest_updated_at_ms: i64_value(project, "latest_updated_at_ms"),
                        authority_files: parse_file_candidates(project, "authority_files"),
                        handoff_files: parse_file_candidates(project, "handoff_files"),
                        evidence_files: parse_file_candidates(project, "evidence_files"),
                        harness_candidates: parse_harness_candidates(project),
                        harness_resources: parse_harness_resources(project),
                        context_warnings: string_array(project, "context_warnings"),
                        warnings: string_array(project, "warnings"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_sessions(index: &Value) -> Vec<SessionRecord> {
    index
        .get("threads")
        .and_then(Value::as_array)
        .map(|threads| {
            threads
                .iter()
                .map(|thread| SessionRecord {
                    thread_id: optional_string_from(thread, "thread_id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    title: optional_string_from(thread, "title")
                        .unwrap_or_else(|| "未知标题".to_string()),
                    project_root: optional_string_from(thread, "project_root"),
                    updated_at_ms: i64_value(thread, "updated_at_ms"),
                    archived: bool_value(thread, "archived"),
                    rollout_exists: bool_value(thread, "rollout_exists"),
                    rollout_path: optional_string_from(thread, "rollout_path"),
                    model: optional_string_from(thread, "model"),
                    reasoning_effort: optional_string_from(thread, "reasoning_effort"),
                    thread_source: optional_string_from(thread, "thread_source"),
                    warnings: string_array(thread, "warnings"),
                    workbench_bound: false,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_codex_transcript(value: &Value) -> Result<CodexTranscript, String> {
    let summary = value
        .get("summary")
        .ok_or_else(|| "transcript 缺少 summary".to_string())?;
    let events = value
        .get("events")
        .and_then(Value::as_array)
        .ok_or_else(|| "transcript 缺少 events 数组".to_string())?
        .iter()
        .map(parse_codex_transcript_event)
        .collect::<Vec<_>>();
    let event_count = events.len();

    Ok(CodexTranscript {
        thread_id: optional_string_from(value, "thread_id")
            .ok_or_else(|| "transcript 缺少 thread_id".to_string())?,
        rollout_path: optional_string_from(value, "rollout_path")
            .ok_or_else(|| "transcript 缺少 rollout_path".to_string())?,
        project_path: optional_string_from(value, "project_path"),
        title: optional_string_from(value, "title"),
        created_at_ms: i64_value(value, "created_at_ms"),
        updated_at_ms: i64_value(value, "updated_at_ms"),
        viewer_boundary: codex_transcript::transcript_viewer_boundary(),
        events,
        summary: CodexTranscriptSummary {
            total_events: usize_value(summary, "total_events"),
            event_type_counts: usize_map(summary.get("event_type_counts")),
            unknown_event_count: usize_value(summary, "unknown_event_count"),
            warning_count: usize_value(summary, "warning_count"),
            encrypted_content_event_count: usize_value(summary, "encrypted_content_event_count"),
            sensitive_like_event_count: usize_value(summary, "sensitive_like_event_count"),
        },
        pagination: CodexTranscriptPagination {
            mode: "full".to_string(),
            page_size: event_count,
            returned_events: event_count,
            total_line_count: event_count,
            selected_line_count: event_count,
            has_older: false,
            older_before_line: None,
        },
        warnings: string_array(value, "warnings"),
        source_stats: value.get("source_stats").cloned().unwrap_or(Value::Null),
    })
}

fn parse_codex_transcript_event(value: &Value) -> CodexTranscriptEvent {
    CodexTranscriptEvent {
        event_id: optional_string_from(value, "event_id").unwrap_or_else(|| "unknown".to_string()),
        timestamp: optional_string_from(value, "timestamp"),
        event_type: optional_string_from(value, "event_type"),
        actor: optional_string_from(value, "actor"),
        role: optional_string_from(value, "role"),
        turn_id: optional_string_from(value, "turn_id"),
        call_id: optional_string_from(value, "call_id"),
        tool_name: optional_string_from(value, "tool_name"),
        text: optional_string_from(value, "text"),
        arguments: value.get("arguments").cloned().unwrap_or(Value::Null),
        output: value.get("output").cloned().unwrap_or(Value::Null),
        stdout: optional_string_from(value, "stdout"),
        stderr: optional_string_from(value, "stderr"),
        exit_code: value.get("exit_code").cloned().unwrap_or(Value::Null),
        metadata: value.get("metadata").cloned().unwrap_or(Value::Null),
        warnings: string_array(value, "warnings"),
    }
}

fn parse_skills(index: &Value) -> Vec<SkillRecord> {
    index
        .get("skills")
        .and_then(Value::as_array)
        .map(|skills| {
            skills
                .iter()
                .map(|skill| SkillRecord {
                    skill_id: optional_string_from(skill, "skill_id")
                        .unwrap_or_else(|| "unknown".to_string()),
                    title: optional_string_from(skill, "title")
                        .unwrap_or_else(|| "未知 Skill".to_string()),
                    description: optional_string_from(skill, "description"),
                    path: optional_string_from(skill, "path")
                        .unwrap_or_else(|| "未知路径".to_string()),
                    source_type: optional_string_from(skill, "source_type")
                        .unwrap_or_else(|| "unknown".to_string()),
                    plugin_name: optional_string_from(skill, "plugin_name"),
                    plugin_version: optional_string_from(skill, "plugin_version"),
                    warnings: string_array(skill, "warnings"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_plugins(index: &Value) -> Vec<PluginRecord> {
    index
        .get("plugins")
        .and_then(Value::as_array)
        .map(|plugins| {
            plugins
                .iter()
                .map(|plugin| PluginRecord {
                    plugin_name: optional_string_from(plugin, "plugin_name")
                        .unwrap_or_else(|| "unknown".to_string()),
                    plugin_version: optional_string_from(plugin, "plugin_version")
                        .unwrap_or_else(|| "unknown".to_string()),
                    homepage: optional_string_from(plugin, "homepage"),
                    skill_count: plugin
                        .get("skill_paths")
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len),
                    has_apps: bool_value(plugin, "has_apps"),
                    has_mcp_servers: bool_value(plugin, "has_mcp_servers"),
                    warnings: string_array(plugin, "warnings"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_file_candidates(parent: &Value, key: &str) -> Vec<FileCandidate> {
    parent
        .get(key)
        .and_then(Value::as_array)
        .map(|files| {
            files
                .iter()
                .filter_map(|file| {
                    let path = optional_string_from(file, "path")?;
                    Some(FileCandidate {
                        kind: optional_string_from(file, "kind"),
                        name: optional_string_from(file, "name"),
                        path,
                        warnings: string_array(file, "warnings"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_harness_candidates(project: &Value) -> Vec<HarnessCandidate> {
    project
        .get("harness_candidates")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let path = optional_string_from(item, "path")?;
                    Some(HarnessCandidate {
                        entry_type: optional_string_from(item, "entry_type"),
                        name: optional_string_from(item, "name"),
                        path,
                        source: optional_string_from(item, "source"),
                        size_bytes: i64_value(item, "size_bytes"),
                        updated_at_ms: i64_value(item, "updated_at_ms"),
                        warnings: string_array(item, "warnings"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_harness_resources(project: &Value) -> Vec<HarnessResource> {
    project
        .get("harness_resources")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let root_path = optional_string_from(item, "root_path")?;
                    Some(HarnessResource {
                        root_path,
                        display_name: optional_string_from(item, "display_name"),
                        harness_kind: optional_string_from(item, "harness_kind"),
                        agent_type: optional_string_from(item, "agent_type"),
                        adapter_id: optional_string_from(item, "adapter_id"),
                        source_kind: optional_string_from(item, "source_kind"),
                        capabilities: string_array(item, "capabilities"),
                        manifest_path: optional_string_from(item, "manifest_path"),
                        readme_path: optional_string_from(item, "readme_path"),
                        version: optional_string_from(item, "version"),
                        entrypoints: parse_harness_entrypoints(item),
                        permission_level: optional_string_from(item, "permission_level"),
                        size_bytes: i64_value(item, "size_bytes"),
                        updated_at_ms: i64_value(item, "updated_at_ms"),
                        warnings: string_array(item, "warnings"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_harness_entrypoints(resource: &Value) -> Vec<HarnessEntrypoint> {
    resource
        .get("entrypoints")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let path = optional_string_from(item, "path")?;
                    Some(HarnessEntrypoint {
                        entry_type: optional_string_from(item, "entry_type"),
                        name: optional_string_from(item, "name"),
                        path,
                        source_kind: optional_string_from(item, "source_kind"),
                        size_bytes: i64_value(item, "size_bytes"),
                        updated_at_ms: i64_value(item, "updated_at_ms"),
                        warnings: string_array(item, "warnings"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_tasks(markdown: &str) -> Vec<TaskEntry> {
    let mut section = String::new();
    let section_map = BTreeMap::from([
        ("待派发", "待派发"),
        ("进行中", "进行中"),
        ("已回收", "已回收"),
        ("暂停", "暂停"),
    ]);
    let mut tasks = Vec::new();

    for line in markdown.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            section = section_map
                .get(title.trim())
                .map(|value| (*value).to_string())
                .unwrap_or_default();
            continue;
        }

        if section.is_empty() {
            continue;
        }

        if let Some(raw_item) = line.strip_prefix("- ") {
            let title = raw_item
                .split('：')
                .next()
                .unwrap_or(raw_item)
                .replace('`', "")
                .trim()
                .to_string();
            tasks.push(TaskEntry {
                status: section.clone(),
                title,
            });
        }
    }

    tasks
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

fn allowed_paths_with_sessions(index: &Value, sessions: &[SessionRecord]) -> AllowedPaths {
    let mut allowed = allowed_paths(index);
    for session in sessions {
        if let Some(path) = session.rollout_path.as_ref() {
            allowed.rollouts.insert(path.clone());
        }
    }
    allowed
}

fn extend_allowed_rollouts_from_sqlite(allowed: &mut AllowedPaths) {
    let db_path = codex_db::default_state_db_path();
    let Ok(rows) = codex_db::read_threads(&db_path) else {
        return;
    };
    let Some(codex_home) = db_path.parent() else {
        return;
    };
    for row in rows {
        let Some(path) = row.rollout_path else {
            continue;
        };
        let rollout_path = PathBuf::from(&path);
        if codex_transcript::is_allowed_rollout_path(&rollout_path, codex_home) {
            allowed.rollouts.insert(path);
        }
    }
}

impl AllowedPaths {
    fn can_copy(&self, path: &str) -> bool {
        self.projects.contains(path) || self.rollouts.contains(path)
    }
}

fn array_len(index: &Value, key: &str) -> usize {
    index.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn optional_string(index: &Value, key: &str) -> Option<String> {
    index.get(key).and_then(Value::as_str).map(str::to_string)
}

fn optional_string_from(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn optional_i64_from(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn usize_value(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map_or(0, |raw| raw as usize)
}

fn i64_value(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn usize_map(value: Option<&Value>) -> BTreeMap<String, usize> {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, raw)| raw.as_u64().map(|count| (key.clone(), count as usize)))
                .collect()
        })
        .unwrap_or_default()
}

fn bool_value(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn path_name(path: &str) -> String {
    path.split('/')
        .filter(|part| !part.is_empty())
        .last()
        .unwrap_or(path)
        .to_string()
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
    Err("当前一期只验证 macOS 剪贴板能力".to_string())
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
    Err("当前一期只验证 macOS 打开和定位能力".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let acceptance_profile_requested = std::env::var_os("SYN_R4_ACCEPTANCE_PROFILE").is_some();
    if let Err(error) = crate::acceptance_runtime_profile::initialize_from_env() {
        if acceptance_profile_requested {
            exit_acceptance_startup_failure(ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE);
        }
        eprintln!("验收 runtime profile 初始化失败：{error}");
        return;
    }
    let acceptance_state = if acceptance_profile_requested {
        Some(match AppState::try_new() {
            Ok(state) => state,
            Err(_) => {
                exit_acceptance_startup_failure(ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE);
            }
        })
    } else {
        None
    };
    let m2_r4_reference_slice_requested = match crate::m2_r4_reference_slice_driver::requested() {
        Ok(requested) => requested,
        Err(error) => {
            if acceptance_profile_requested {
                eprintln!("M2 R4 reference-slice runner 请求无效：{error}");
                exit_acceptance_startup_failure(ACCEPTANCE_RUNTIME_PROFILE_FINALIZATION_EXIT_CODE);
            }
            return;
        }
    };
    if let Some(state) = acceptance_state.as_ref() {
        if m2_r4_reference_slice_requested {
            if let Err(error) = crate::m2_r4_reference_slice_driver::prepare_before_startup(state) {
                eprintln!("M2 R4 reference-slice runner 预置失败：{error}");
                exit_acceptance_startup_failure(
                    ACCEPTANCE_RUNTIME_PROFILE_FINALIZATION_EXIT_CODE,
                );
            }
        }
        if let Err(error) = crate::migrate_legacy_workflow_node_session_binding_ids_at(
            &state.workflow_state_path,
        ) {
            eprintln!("工作流 binding_id 迁移未完成：{error}");
        }
        if let Err(error) =
            crate::exec_process_registry::reap_registered_orphans(&state.workflow_state_path)
        {
            eprintln!("执行进程遗留回收未完成：{error}");
        }
        if let Err(error) =
            crate::supervisor_session_launcher::reap_supervisor_resident_stale_sessions_at(
                &state.workflow_state_path,
            )
        {
            eprintln!("主管一次一发会话陈账对账未完成：{error}");
        }
        if let Err(error) = crate::workbench_sqlite_storage_mode::initialize_for_startup(
            &state.workflow_state_path,
        ) {
            if m2_r4_reference_slice_requested {
                eprintln!("M2 R4 reference-slice DB-primary startup failure:{error}");
            }
            exit_acceptance_startup_failure(
                ACCEPTANCE_RUNTIME_PROFILE_FINALIZATION_EXIT_CODE,
            );
        }
        if let Err(error) = crate::acceptance_runtime_profile::finalize_first_r4_initialization() {
            eprintln!("验收 runtime profile 首次初始化标记失败：{error}");
            exit_acceptance_startup_failure(ACCEPTANCE_RUNTIME_PROFILE_FINALIZATION_EXIT_CODE);
        }
    }
    let app = tauri::Builder::default()
        .manage(mcp::orchestrator::OrchestratorState::new())
        .manage(crate::knowledge_open_relay::KnowledgeOpenRelayState::new())
        .invoke_handler(workbench_command_handler!())
        .setup(move |app| {
            let isolated_profile_active = crate::acceptance_runtime_profile::active_paths()
                .map_err(std::io::Error::other)?
                .is_some();
            let state = if isolated_profile_active {
                acceptance_state.ok_or_else(|| {
                    "acceptance_runtime_profile_state_missing_after_initialization".to_string()
                })
            } else {
                app.path()
                    .app_data_dir()
                    .map_err(|_| "m4_secretary_tauri_app_data_root_unavailable".to_string())
                    .and_then(|app_data_root| {
                        AppState::try_new_with_tauri_app_data_root(&app_data_root)
                    })
            };
            let state = match state {
                Ok(state) => state,
                Err(error) => {
                    if acceptance_profile_requested {
                        exit_acceptance_startup_failure(
                            ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE,
                        );
                    }
                    return Err(std::io::Error::other(format!(
                        "AppState 启动装配失败：{error}"
                    ))
                    .into());
                }
            };
            if !isolated_profile_active {
                if m2_r4_reference_slice_requested {
                    if let Err(error) =
                        crate::m2_r4_reference_slice_driver::prepare_before_startup(&state)
                    {
                        return Err(std::io::Error::other(format!(
                            "M2 R4 reference-slice runner 预置失败：{error}"
                        ))
                        .into());
                    }
                }
                if let Err(error) = crate::migrate_legacy_workflow_node_session_binding_ids_at(
                    &state.workflow_state_path,
                ) {
                    eprintln!("工作流 binding_id 迁移未完成：{error}");
                }
                if let Err(error) = crate::exec_process_registry::reap_registered_orphans(
                    &state.workflow_state_path,
                ) {
                    eprintln!("执行进程遗留回收未完成：{error}");
                }
                if let Err(error) =
                    crate::supervisor_session_launcher::reap_supervisor_resident_stale_sessions_at(
                        &state.workflow_state_path,
                    )
                {
                    eprintln!("主管一次一发会话陈账对账未完成：{error}");
                }
                if let Err(error) = crate::workbench_sqlite_storage_mode::initialize_for_startup(
                    &state.workflow_state_path,
                ) {
                    eprintln!("DB 主写模式启动对账未通过：{error}");
                }
            }
            if !app.manage(state) {
                return Err(std::io::Error::other("AppState 重复注册").into());
            }
            if !isolated_profile_active {
                app.state::<crate::knowledge_open_relay::KnowledgeOpenRelayState>()
                    .start(app.handle().clone())
                    .map_err(std::io::Error::other)?;
            }
            if cfg!(debug_assertions) {
                let log_plugin = match crate::acceptance_runtime_profile::isolated_log_dir()
                    .map_err(std::io::Error::other)?
                {
                    Some(path) => tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .clear_targets()
                        .target(tauri_plugin_log::Target::new(
                            tauri_plugin_log::TargetKind::Folder {
                                path,
                                file_name: Some("syn-r4-isolated".into()),
                            },
                        ))
                        .build(),
                    None => tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                };
                app.handle().plugin(log_plugin)?;
            }
            if m2_r4_reference_slice_requested {
                crate::m2_r4_reference_slice_driver::install_after_runtime_ready(app)
                    .map_err(std::io::Error::other)?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|app_handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            app_handle
                .state::<crate::knowledge_open_relay::KnowledgeOpenRelayState>()
                .shutdown();
            if let Err(error) = crate::manual_relay::stop_all_active_manual_relay_attempts() {
                eprintln!("手动中转进程退出清理未完成：{error}");
            }
        }
    });
}
