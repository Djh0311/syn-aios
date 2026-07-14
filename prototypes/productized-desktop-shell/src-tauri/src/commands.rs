// Tauri command wrappers split out during Task B conservative module split.
// This file is included at crate root to preserve command names and behavior.

#[tauri::command]
fn load_workbench_snapshot(state: tauri::State<'_, AppState>) -> Result<WorkbenchSnapshot, String> {
    let index = read_index(&state)?;
    let tasks_text = fs::read_to_string(&state.tasks_path).unwrap_or_default();
    Ok(build_snapshot(&state, &index, &tasks_text))
}

#[tauri::command]
fn query_workbench_page_read_model(
    request: page_read_model::PageReadModelQueryInput,
    state: tauri::State<'_, AppState>,
) -> Result<page_read_model::PageReadModelQueryResult, String> {
    let index = read_index(&state)?;
    let tasks_text = fs::read_to_string(&state.tasks_path).unwrap_or_default();
    let snapshot = build_snapshot(&state, &index, &tasks_text);
    let snapshot_value =
        serde_json::to_value(&snapshot).map_err(|error| format!("snapshot_serialize_failed:{error}"))?;
    let workflow_state_value = read_workflow_state_snapshot(&state.workflow_state_path)
        .ok()
        .map(|snapshot| {
            serde_json::to_value(snapshot)
                .map_err(|error| format!("workflow_state_serialize_failed:{error}"))
        })
        .transpose()?;
    let generated_at = optional_string(&index, "generated_at").unwrap_or_else(unix_timestamp_string);
    page_read_model::query_page_read_model_with_snapshot_value(
        &request,
        &generated_at,
        &snapshot_value,
        workflow_state_value.as_ref(),
    )
}

#[tauri::command]
fn record_operation_control_decision(
    request: operation_control::OperationControlDecisionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    operation_control::record_operation_control_decision_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn preview_manual_codex_relay(
    request: manual_relay::ManualRelayPreviewInput,
) -> Result<manual_relay::ManualRelayPreview, String> {
    Ok(manual_relay::preview_manual_relay(
        request,
        &unix_timestamp_string(),
    ))
}

#[tauri::command]
fn confirm_manual_codex_relay_once(
    request: manual_relay::ManualRelayConfirmInput,
) -> Result<manual_relay::ManualRelayConfirmation, String> {
    manual_relay::confirm_manual_relay_once(request, &unix_timestamp_string())
}

#[tauri::command]
fn run_manual_codex_relay_once(
    request: manual_relay::ManualRelayRunInput,
) -> Result<manual_relay::ManualRelayReceipt, String> {
    manual_relay::run_manual_relay_once(request, &unix_timestamp_string())
}

#[tauri::command]
fn run_manual_codex_relay_gui_direct(
    request: manual_relay::ManualRelayGuiDirectRunInput,
) -> Result<manual_relay::ManualRelayReceipt, String> {
    manual_relay::run_manual_relay_gui_direct_once(request, &unix_timestamp_string())
}

#[tauri::command]
fn run_manual_codex_relay_gui_direct_new_session(
    request: manual_relay::ManualRelayGuiDirectNewSessionInput,
) -> Result<manual_relay::ManualRelayReceipt, String> {
    manual_relay::run_manual_relay_gui_direct_new_session_once(request, &unix_timestamp_string())
}

#[tauri::command]
fn stop_manual_codex_relay_attempt(
    request: manual_relay::ManualRelayStopInput,
) -> Result<manual_relay::ManualRelayReceipt, String> {
    manual_relay::stop_manual_relay_attempt(request, &unix_timestamp_string())
}

#[tauri::command]
fn poll_manual_codex_relay_attempt(
    request: manual_relay::ManualRelayPollInput,
) -> Result<manual_relay::ManualRelayReceipt, String> {
    manual_relay::poll_manual_relay_attempt(request, &unix_timestamp_string())
}

#[tauri::command]
fn load_codex_session_transcript(
    thread_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<CodexTranscript, String> {
    load_codex_session_transcript_for_index(&state, &thread_id)
}

#[tauri::command]
fn load_codex_session_transcript_page(
    request: CodexTranscriptPageRequest,
    state: tauri::State<'_, AppState>,
) -> Result<CodexTranscript, String> {
    load_codex_session_transcript_page_for_index(&state, &request)
}

fn load_codex_session_transcript_page_for_index(
    state: &AppState,
    request: &CodexTranscriptPageRequest,
) -> Result<CodexTranscript, String> {
    let db_path = codex_db::default_state_db_path();
    match read_index(state) {
        Ok(index) => load_codex_session_transcript_page_with_catalog(&index, request, &db_path),
        Err(index_error) => load_codex_session_transcript_page_with_optional_catalog(
            None,
            request,
            &db_path,
            Some(index_error),
        ),
    }
}

fn load_codex_session_transcript_page_with_catalog(
    index: &Value,
    request: &CodexTranscriptPageRequest,
    db_path: &Path,
) -> Result<CodexTranscript, String> {
    load_codex_session_transcript_page_with_optional_catalog(Some(index), request, db_path, None)
}

fn load_codex_session_transcript_page_with_optional_catalog(
    index: Option<&Value>,
    request: &CodexTranscriptPageRequest,
    db_path: &Path,
    index_error: Option<String>,
) -> Result<CodexTranscript, String> {
    match codex_db::read_threads(db_path) {
        Ok(rows) => {
            if let Some(row) = rows
                .into_iter()
                .find(|row| row.thread_id == request.thread_id)
            {
                return load_codex_session_transcript_page_from_sqlite_row(&row, db_path, request);
            }
        }
        Err(err) => {
            if let Some(index) = index {
                if let Some(thread) = find_index_thread(index, &request.thread_id) {
                    return load_codex_session_transcript_page_from_index_thread(
                        index,
                        &thread,
                        "index_fallback_sqlite_unavailable",
                        request,
                    );
                }
            }
            if let Some(index_error) = index_error {
                return Err(format!(
                    "sqlite_unavailable:{err};index_unavailable:{index_error}"
                ));
            }
            return Err(format!("sqlite_unavailable:{err}"));
        }
    }

    if let Some(index) = index {
        if let Some(thread) = find_index_thread(index, &request.thread_id) {
            return load_codex_session_transcript_page_from_index_thread(
                index,
                &thread,
                "index_fallback_thread_missing_in_sqlite",
                request,
            );
        }
    }

    if let Some(codex_home) = db_path.parent() {
        if let Some(transcript) =
            load_codex_session_transcript_page_from_rollout_fallback(codex_home, request)?
        {
            return Ok(transcript);
        }
    }

    Err(format!("session_not_found:{}", request.thread_id))
}

fn load_codex_session_transcript_page_from_sqlite_row(
    row: &codex_db::CodexThreadRow,
    db_path: &Path,
    request: &CodexTranscriptPageRequest,
) -> Result<CodexTranscript, String> {
    let codex_home = db_path
        .parent()
        .ok_or_else(|| "unexpected_internal_error:sqlite_db_path_without_parent".to_string())?;
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: row.thread_id.clone(),
        rollout_path: row.rollout_path.clone(),
        project_root: row.project_root.clone(),
        title: Some(row.title.clone()),
        created_at_ms: None,
        updated_at_ms: row.updated_at_ms,
        catalog_source: "sqlite".to_string(),
        index_thread_count: None,
    };
    codex_transcript::read_transcript_page_from_rollout(
        metadata,
        codex_home,
        transcript_page_request(request),
    )
}

fn load_codex_session_transcript_page_from_index_thread(
    index: &Value,
    thread: &SessionRecord,
    catalog_source: &str,
    request: &CodexTranscriptPageRequest,
) -> Result<CodexTranscript, String> {
    let codex_home = codex_home_from_index(index)?;
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: thread.thread_id.clone(),
        rollout_path: thread.rollout_path.clone(),
        project_root: thread.project_root.clone(),
        title: Some(thread.title.clone()),
        created_at_ms: None,
        updated_at_ms: thread.updated_at_ms,
        catalog_source: catalog_source.to_string(),
        index_thread_count: index.get("threads").and_then(Value::as_array).map(Vec::len),
    };
    codex_transcript::read_transcript_page_from_rollout(
        metadata,
        &codex_home,
        transcript_page_request(request),
    )
}

fn transcript_page_request(
    request: &CodexTranscriptPageRequest,
) -> codex_transcript::TranscriptReadPageRequest {
    codex_transcript::TranscriptReadPageRequest {
        limit: request.limit.unwrap_or(80),
        before_line: request.before_line,
    }
}

#[tauri::command]
fn load_codex_session_page(
    request: CodexSessionPageRequest,
    state: tauri::State<'_, AppState>,
) -> Result<CodexSessionPage, String> {
    let db_path = codex_db::default_state_db_path();
    let page = codex_db::read_threads_page(
        &db_path,
        codex_db::CodexThreadPageOptions {
            page_size: request.page_size.unwrap_or(100),
            offset: request.offset.unwrap_or(0),
            include_archived: request.include_archived.unwrap_or(false),
            archived_only: request.archived_only.unwrap_or(false),
            query: request.query.clone(),
        },
    );
    match page {
        Ok(page) => {
            let mut sessions: Vec<SessionRecord> = page
                .rows
                .into_iter()
                .map(session_record_from_codex_thread)
                .collect();
            let mut warnings = Vec::new();
            let mut source = "sqlite_page".to_string();
            // 修显示 bug（2026-07-09）：工作台用 codex exec 建的会话 has_user_event=0，被 read_threads_page 的
            // 显示过滤藏掉（噪音过滤本体不动·codex_db.rs:90）。仅首页并上 store 绑过工作流节点的会话
            //（find_thread_by_id 绕过滤解析·标 workbench_bound）——后页这些已在首页显示过·避免重复与分页错乱
            //（offset 守卫在 helper 内）。软着陆:读 store 失败只出 warning、返回原列表（显示是增益不是闸）。
            let merge_warnings = merge_workbench_bound_sessions(
                &mut sessions,
                &state.workflow_state_path,
                &db_path,
                request.offset.unwrap_or(0),
                request.include_archived.unwrap_or(false),
                request.archived_only.unwrap_or(false),
            );
            warnings.extend(merge_warnings);
            if sessions.is_empty() {
                if let Some(query) = request.query.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
                    if let Some(codex_home) = db_path.parent() {
                        if let Some(fallback_session) =
                            find_rollout_session_by_thread_query(codex_home, query)
                        {
                            sessions.push(fallback_session);
                            warnings.push("sqlite_session_missing_rollout_filename_fallback".to_string());
                            source = "sqlite_page_rollout_filename_fallback".to_string();
                        }
                    }
                }
            }
            Ok(CodexSessionPage {
                sessions,
                page_size: page.page_size,
                offset: page.offset,
                has_more: page.has_more,
                include_archived: page.include_archived,
                archived_only: page.archived_only,
                warnings,
                source,
            })
        }
        Err(error) => {
            let index = read_index(&state)?;
            let include_archived = request.include_archived.unwrap_or(false);
            let archived_only = request.archived_only.unwrap_or(false);
            let page_size = request.page_size.unwrap_or(100).clamp(1, 250);
            let offset = request.offset.unwrap_or(0);
            let query = request.query.as_deref().map(str::trim).filter(|value| !value.is_empty());
            let mut sessions: Vec<SessionRecord> = parse_sessions(&index)
                .into_iter()
                .filter(|session| {
                    if archived_only {
                        session.archived
                    } else {
                        include_archived || !session.archived
                    }
                })
                .filter(|session| query.is_none_or(|query| session_matches_query(session, query)))
                .collect();
            sessions.sort_by(|a, b| {
                let at = a.updated_at_ms.unwrap_or(0);
                let bt = b.updated_at_ms.unwrap_or(0);
                bt.cmp(&at).then_with(|| b.thread_id.cmp(&a.thread_id))
            });
            if sessions.is_empty() {
                if let Some(query) = query {
                    if let Ok(codex_home) = codex_home_from_index(&index) {
                        if let Some(fallback_session) =
                            find_rollout_session_by_thread_query(&codex_home, query)
                        {
                            sessions.push(fallback_session);
                        }
                    }
                }
            }
            let has_more = sessions.len() > offset.saturating_add(page_size);
            let page_sessions = sessions.into_iter().skip(offset).take(page_size).collect();
            Ok(CodexSessionPage {
                sessions: page_sessions,
                page_size,
                offset,
                has_more,
                include_archived,
                archived_only,
                warnings: vec![format!("codex sqlite 分页读取失败，回落到旧索引分页：{error}")],
                source: "index_fallback_page".to_string(),
            })
        }
    }
}

/// 把「工作台绑过工作流节点的会话」合并进列表（修显示 bug：C1 任务会话看不见）。
///
/// 背景：工作台经 codex exec 建的会话 `has_user_event=0`，被 `read_threads_page` 的显示过滤
/// （codex_db.rs:90「Skips threads where has_user_event=0」）合理藏掉——那过滤对 codex 一堆空占位
/// 噪音有用、**本体不动**。这里只**定向补上**工作台真在用的：判据 = store
/// `workflow_node_session_bindings[].native_thread_id` 这个硬信号（**不靠标题字符串猜**），用只读、
/// **绕显示过滤**的 `find_thread_by_id`（codex_db.rs:118）按主键解析，标 `workbench_bound`，
/// 按 `updated_at_ms DESC, id DESC` 重排（与 sqlite `ORDER BY` 一致·自然按时间交错）。
///
/// 软着陆：读 store 失败 / 单条解析失败 → 只出 warning、保留原列表，**不 Err 断列表**
/// （显示是增益不是闸）。只补不在列表里的（按 thread_id 去重），归档按当前视图口径过滤。
fn merge_workbench_bound_sessions(
    sessions: &mut Vec<SessionRecord>,
    workflow_state_path: &Path,
    db_path: &Path,
    offset: usize,
    include_archived: bool,
    archived_only: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();
    // 仅首页并入（后页这些已在首页显示过·避免重复与分页错乱）。
    if offset != 0 {
        return warnings;
    }
    let state = match read_workflow_state_value(workflow_state_path) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!(
                "workbench_bound_sessions_skipped_state_unreadable:{error}"
            ));
            return warnings;
        }
    };
    // store 绑定的 native_thread_id：非空 · 排除已在列表里的 · 去重。
    let existing: std::collections::HashSet<String> =
        sessions.iter().map(|s| s.thread_id.clone()).collect();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let bound_thread_ids: Vec<String> = state
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .map(|bindings| {
            bindings
                .iter()
                .filter_map(|binding| optional_string_from(binding, "native_thread_id"))
                .filter(|thread_id| !thread_id.is_empty())
                .filter(|thread_id| !existing.contains(thread_id))
                .filter(|thread_id| seen.insert(thread_id.clone()))
                .collect()
        })
        .unwrap_or_default();
    let mut added = 0usize;
    for thread_id in bound_thread_ids {
        match codex_db::find_thread_by_id(db_path, &thread_id) {
            Ok(Some(row)) => {
                // 归档按当前视图口径（与 read_threads_page 显示过滤一致）：archived_only 只收归档的；
                // 否则非归档收、归档看 include_archived。
                let visible = if archived_only {
                    row.archived
                } else {
                    include_archived || !row.archived
                };
                if !visible {
                    continue;
                }
                let mut record = session_record_from_codex_thread(row);
                record.workbench_bound = true;
                sessions.push(record);
                added += 1;
            }
            // 会话不在 codex 侧（已删/尚未落库）——跳过，不算错。
            Ok(None) => {}
            Err(error) => {
                warnings.push(format!(
                    "workbench_bound_session_resolve_failed:{thread_id}:{error}"
                ));
            }
        }
    }
    if added > 0 {
        // 与 read_threads_page 的 `ORDER BY updated_at_ms DESC, id DESC` 一致·并入后自然按时间交错。
        sessions.sort_by(|a, b| {
            let at = a.updated_at_ms.unwrap_or(0);
            let bt = b.updated_at_ms.unwrap_or(0);
            bt.cmp(&at).then_with(|| b.thread_id.cmp(&a.thread_id))
        });
    }
    warnings
}

fn session_matches_query(session: &SessionRecord, query: &str) -> bool {
    let normalized = query.to_lowercase();
    [
        Some(session.thread_id.as_str()),
        Some(session.title.as_str()),
        session.project_root.as_deref(),
        session.rollout_path.as_deref(),
        session.model.as_deref(),
        session.reasoning_effort.as_deref(),
        session.thread_source.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| value.to_lowercase().contains(&normalized))
}

fn load_codex_session_transcript_page_from_rollout_fallback(
    codex_home: &Path,
    request: &CodexTranscriptPageRequest,
) -> Result<Option<CodexTranscript>, String> {
    let Some(session) = find_rollout_session_by_thread_query(codex_home, &request.thread_id) else {
        return Ok(None);
    };
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: session.thread_id,
        rollout_path: session.rollout_path,
        project_root: session.project_root,
        title: Some(session.title),
        created_at_ms: None,
        updated_at_ms: session.updated_at_ms,
        catalog_source: "rollout_filename_fallback".to_string(),
        index_thread_count: None,
    };
    codex_transcript::read_transcript_page_from_rollout(
        metadata,
        codex_home,
        transcript_page_request(request),
    )
    .map(Some)
}

fn load_codex_session_transcript_from_rollout_fallback(
    codex_home: &Path,
    thread_id: &str,
) -> Result<Option<CodexTranscript>, String> {
    let Some(session) = find_rollout_session_by_thread_query(codex_home, thread_id) else {
        return Ok(None);
    };
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: session.thread_id,
        rollout_path: session.rollout_path,
        project_root: session.project_root,
        title: Some(session.title),
        created_at_ms: None,
        updated_at_ms: session.updated_at_ms,
        catalog_source: "rollout_filename_fallback".to_string(),
        index_thread_count: None,
    };
    codex_transcript::read_transcript_from_rollout(metadata, codex_home).map(Some)
}

fn find_rollout_session_by_thread_query(codex_home: &Path, query: &str) -> Option<SessionRecord> {
    let query = query.trim().to_lowercase();
    if query.len() < 8 {
        return None;
    }
    let mut candidates = Vec::new();
    for root_name in ["sessions", "archived_sessions"] {
        collect_rollout_matches(
            &codex_home.join(root_name),
            &query,
            root_name == "archived_sessions",
            &mut candidates,
        );
    }
    candidates
        .into_iter()
        .max_by_key(|session| session.updated_at_ms.unwrap_or(0))
}

fn collect_rollout_matches(
    root: &Path,
    query: &str,
    archived: bool,
    out: &mut Vec<SessionRecord>,
) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rollout_matches(&path, query, archived, out);
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(thread_id) = thread_id_from_rollout_file_name(file_name) else {
            continue;
        };
        if !thread_id.to_lowercase().contains(query) && !file_name.to_lowercase().contains(query) {
            continue;
        }
        out.push(session_record_from_rollout_path(path, thread_id, archived));
    }
}

fn thread_id_from_rollout_file_name(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".jsonl")?;
    let body = stem.strip_prefix("rollout-")?;
    if body.len() < 36 {
        return None;
    }
    let thread_id = &body[body.len() - 36..];
    let bytes = thread_id.as_bytes();
    let uuid_shape = bytes.len() == 36
        && bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| [8, 13, 18, 23].contains(&index) || byte.is_ascii_hexdigit());
    if uuid_shape {
        Some(thread_id.to_string())
    } else {
        None
    }
}

fn session_record_from_rollout_path(path: PathBuf, thread_id: String, archived: bool) -> SessionRecord {
    let (project_root, model, reasoning_effort) = rollout_session_meta(&path);
    let updated_at_ms = fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64);
    SessionRecord {
        thread_id: thread_id.clone(),
        title: format!("新建 Codex 对话 {}", thread_id.chars().take(8).collect::<String>()),
        project_root,
        updated_at_ms,
        archived,
        rollout_exists: true,
        rollout_path: Some(path.display().to_string()),
        model,
        reasoning_effort,
        thread_source: Some("codex".to_string()),
        warnings: vec!["session_index_pending_rollout_filename_fallback".to_string()],
        workbench_bound: false,
    }
}

fn rollout_session_meta(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    let Ok(file) = fs::File::open(path) else {
        return (None, None, None);
    };
    let reader = std::io::BufReader::new(file);
    for line in std::io::BufRead::lines(reader).take(32).flatten() {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("session_meta") {
            continue;
        }
        let payload = value.get("payload").and_then(Value::as_object);
        let project_root = payload
            .and_then(|payload| payload.get("cwd"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        let model = payload
            .and_then(|payload| payload.get("model"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        let reasoning_effort = payload
            .and_then(|payload| payload.get("reasoning_effort"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        return (project_root, model, reasoning_effort);
    }
    (None, None, None)
}

#[cfg(test)]
mod command_rollout_fallback_tests {
    use super::*;

    #[test]
    fn rollout_filename_fallback_finds_new_thread_before_sqlite_index_catches_up() {
        let codex_home = temp_codex_home("rollout-filename-fallback");
        let thread_id = "019ede51-6ca4-78a2-b658-6c3ef465ea14";
        let project_root = "/tmp/stage-k-isolated-project";
        let rollout_dir = codex_home.join("sessions/2026/06/19");
        fs::create_dir_all(&rollout_dir).expect("create rollout dir");
        let rollout_path = rollout_dir.join(format!("rollout-2026-06-19T13-18-58-{thread_id}.jsonl"));
        fs::write(
            &rollout_path,
            format!(
                "{}\n{}\n",
                json!({
                    "timestamp": "2026-06-19T13:18:58Z",
                    "type": "session_meta",
                    "payload": {
                        "id": thread_id,
                        "cwd": project_root,
                        "model": "gpt-test"
                    }
                }),
                json!({
                    "timestamp": "2026-06-19T13:19:00Z",
                    "type": "event_msg",
                    "payload": {
                        "type": "user_message",
                        "message": "fallback fixture"
                    }
                })
            ),
        )
        .expect("write rollout");

        let session = find_rollout_session_by_thread_query(&codex_home, "019ede51")
            .expect("fallback session");
        assert_eq!(session.thread_id, thread_id);
        assert_eq!(session.project_root.as_deref(), Some(project_root));
        assert_eq!(session.rollout_path.as_deref(), Some(rollout_path.to_str().expect("path")));
        assert!(session.rollout_exists);
        assert!(session
            .warnings
            .contains(&"session_index_pending_rollout_filename_fallback".to_string()));

        let transcript = load_codex_session_transcript_page_from_rollout_fallback(
            &codex_home,
            &CodexTranscriptPageRequest {
                thread_id: thread_id.to_string(),
                limit: Some(80),
                before_line: None,
            },
        )
        .expect("fallback transcript")
        .expect("fallback transcript present");
        assert_eq!(transcript.thread_id, thread_id);
        assert_eq!(transcript.project_path.as_deref(), Some(project_root));
        assert_eq!(
            transcript.source_stats["catalog_source"].as_str(),
            Some("rollout_filename_fallback")
        );
    }

    fn temp_codex_home(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("codex-workbench-{label}-{stamp}"))
    }

    // P3-A default-safe proof: the canvas "运行此节点" button calls
    // execute_workflow_node_dispatch, whose first line is the test-project
    // path-lock gate. 2026-06-22 下放: the fixed test project unseals on path
    // ALONE (env belt removed); every NON-test real project stays sealed by the
    // path-lock, so a run into a real project is blocked before any codex spawns.
    #[test]
    fn workflow_engine_gate_seals_non_test_project_regardless_of_env() {
        // 非测试真实项目:只凭 project_root 即 false(path-lock)，与 env 无关、任何环境都成立。
        assert!(!workflow_engine_test_project_unsealed("/Users/yoyi/workspace/product-line"));
        assert!(!workflow_engine_test_project_unsealed("/tmp/some-other-project"));
        assert!(!workflow_engine_test_project_unsealed(""));
        // P3-A:固定测试项目现在只凭 path 解封(不再需要 env 钥匙)——这是本次授权的松闸。
        assert!(workflow_engine_test_project_unsealed("/Users/yoyi/codex-workflow-mario-test"));
        // path-lock 那把锁是唯一解封键,常量固定、无隐式放开路径。
        assert_eq!(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "/Users/yoyi/codex-workflow-mario-test");
    }

    // 站 3b 小闸案发测试（2026-07-12 拍板）：只认「3b 项目根精确相等 ∧ 写根为空」。
    #[test]
    fn station3b_gate_only_unseals_exact_root_with_zero_write_roots() {
        let root = STATION_3B_READONLY_PROJECT_ROOT;
        assert_eq!(root, "/Users/yoyi/Documents/mario test");
        // 合法：3b 根 + 零写根。
        assert!(station3b_readonly_project_unsealed(root, &[]));
        // 案发：任何写根都不解封——包括写根就是 3b 根自己。
        assert!(!station3b_readonly_project_unsealed(root, &[root.to_string()]));
        assert!(!station3b_readonly_project_unsealed(
            root,
            &[WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()]
        ));
        // 案发：子目录 / 前缀 / 尾斜杠 / 其它项目一律拒（精确相等，无路径规范化魔法）。
        assert!(!station3b_readonly_project_unsealed("/Users/yoyi/Documents/mario test/subdir", &[]));
        assert!(!station3b_readonly_project_unsealed("/Users/yoyi/Documents/mario test/", &[]));
        assert!(!station3b_readonly_project_unsealed("/Users/yoyi/Documents", &[]));
        assert!(!station3b_readonly_project_unsealed("/Users/yoyi/gameai/crazytown", &[]));
        assert!(!station3b_readonly_project_unsealed("", &[]));
        // 固定测试项目不走 3b 小闸（它走 S1 原闸；两闸互不越界）。
        assert!(!station3b_readonly_project_unsealed(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, &[]));
    }

    // 站 3b 首发实拦复盘（2026-07-12 真机）：S1 执行层合一闸原为 authorization_complete=仅 path-lock，
    // 主管授权的 3b 只读派发被 blocked_waiting_authorization 安全拦下。修正后的判定必须保持这个刻度：
    // 只多认「主管授权 ∧ 3b 项目 ∧ 零写根」这一种新真值，其余一律照旧拒。
    #[test]
    fn station3b_real_execution_authorization_complete_scoping() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let root = STATION_3B_READONLY_PROJECT_ROOT;
        // 测试项目：path-lock 命中即真（与主管、写根无关——原语义零变化）。
        assert!(real_execution_authorization_complete(test_root, &[], false));
        assert!(real_execution_authorization_complete(
            test_root,
            &[test_root.to_string()],
            true
        ));
        // 3b：仅「主管授权 + 零写根」为真。
        assert!(real_execution_authorization_complete(root, &[], true));
        // 经典线（无主管授权）喂 3b → 拒。
        assert!(!real_execution_authorization_complete(root, &[], false));
        // 主管授权但带写根 → 拒。
        assert!(!real_execution_authorization_complete(root, &[root.to_string()], true));
        // 其它真实项目：主管授权也不行。
        assert!(!real_execution_authorization_complete(
            "/Users/yoyi/gameai/crazytown",
            &[],
            true
        ));
    }

    // 站 3b 不放宽 S1：同一个 mario test 目录喂 S1 原闸/legacy 封条仍然全拒——
    // 尤其 j2_b_b1 旧桩写死的就是这个目录，3b 解封绝不能让它复活。
    #[test]
    fn station3b_does_not_widen_s1_gate_or_legacy_seals() {
        assert!(!workflow_engine_test_project_unsealed(STATION_3B_READONLY_PROJECT_ROOT));
        assert!(require_test_project_path_lock(STATION_3B_READONLY_PROJECT_ROOT, "x").is_err());
        assert_eq!(
            project_workflow_automation::J2_B_B1_PROJECT_ROOT,
            STATION_3B_READONLY_PROJECT_ROOT,
            "j2_b_b1 写死目录与 3b 项目是同一个——这条相等就是「必须并列小闸、不得改 S1 本体」的铁证"
        );
        assert!(require_test_project_path_lock(
            project_workflow_automation::J2_B_B1_PROJECT_ROOT,
            "run_project_workflow_automation_j2_b_b1"
        )
        .is_err());
    }

    // ===== 修显示 bug：工作台绑定会话在智能体页可见（2026-07-09）=====
    // 建一个 codex sqlite（含 read_threads_page 用的 threads 表 + has_user_event 列）。
    fn create_codex_threads_db(path: &Path) {
        let conn = rusqlite::Connection::open(path).expect("open sqlite");
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
            "#,
        )
        .expect("create threads table");
    }
    fn insert_codex_thread(
        path: &Path,
        thread_id: &str,
        title: &str,
        updated_at_ms: i64,
        archived: i64,
        has_user_event: i64,
    ) {
        let conn = rusqlite::Connection::open(path).expect("open sqlite");
        conn.execute(
            "INSERT INTO threads (id, title, cwd, updated_at_ms, archived, rollout_path, \
             model, reasoning_effort, thread_source, has_user_event) \
             VALUES (?1, ?2, '/tmp/project', ?3, ?4, '', 'gpt-test', 'medium', 'codex', ?5)",
            rusqlite::params![thread_id, title, updated_at_ms, archived, has_user_event],
        )
        .expect("insert thread");
    }
    fn read_first_page(db_path: &Path) -> Vec<SessionRecord> {
        codex_db::read_threads_page(
            db_path,
            codex_db::CodexThreadPageOptions {
                page_size: 100,
                offset: 0,
                include_archived: false,
                archived_only: false,
                query: None,
            },
        )
        .expect("read page")
        .rows
        .into_iter()
        .map(session_record_from_codex_thread)
        .collect()
    }

    // 两侧都测：store 绑过的 has_user_event=0 会话应出现并标 workbench_bound；没绑的同类噪音仍藏。
    #[test]
    fn workbench_bound_session_surfaces_but_unbound_noise_stays_hidden() {
        let dir = temp_codex_home("workbench-bound-merge");
        fs::create_dir_all(&dir).expect("mkdir");
        let db_path = dir.join("state.sqlite");
        create_codex_threads_db(&db_path);
        // 普通会话（有用户事件·正常显示）；工作台会话 + 噪音会话都 has_user_event=0（被显示过滤藏）。
        insert_codex_thread(&db_path, "thread-normal", "普通会话", 3_000, 0, 1);
        insert_codex_thread(
            &db_path,
            "thread-workbench",
            "交办任务专用会话",
            5_000,
            0,
            0,
        );
        insert_codex_thread(&db_path, "thread-noise", "codex 空占位噪音", 4_000, 0, 0);
        // store 只绑 thread-workbench 到某工作流节点（native_thread_id 硬信号）。
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(
            &workflow_state_path,
            json!({
                "workflow_node_session_bindings": [
                    { "native_thread_id": "thread-workbench", "node_id": "wf:node:codex-dev" }
                ]
            })
            .to_string(),
        )
        .expect("write state");

        // 起点：显示过滤下只见 thread-normal，两条 has_user_event=0 都藏着。
        let mut sessions = read_first_page(&db_path);
        let ids0: std::collections::HashSet<&str> =
            sessions.iter().map(|s| s.thread_id.as_str()).collect();
        assert!(ids0.contains("thread-normal"), "普通会话应可见");
        assert!(
            !ids0.contains("thread-workbench") && !ids0.contains("thread-noise"),
            "起点：两条 has_user_event=0 会话被显示过滤藏着"
        );

        let warnings = merge_workbench_bound_sessions(
            &mut sessions,
            &workflow_state_path,
            &db_path,
            0,
            false,
            false,
        );
        assert!(warnings.is_empty(), "正常路径无 warning：{warnings:?}");

        // 绑定的出现且标 workbench_bound=true。
        let bound = sessions
            .iter()
            .find(|s| s.thread_id == "thread-workbench")
            .expect("工作台绑定会话应并进列表");
        assert!(bound.workbench_bound, "应标 workbench_bound=true");
        // 没绑定的 has_user_event=0 噪音仍不出现（只补工作台的、没把噪音放出来）。
        assert!(
            !sessions.iter().any(|s| s.thread_id == "thread-noise"),
            "没绑定的噪音会话仍藏（证过滤本体没松）"
        );
        // 普通会话不被误标。
        let normal = sessions
            .iter()
            .find(|s| s.thread_id == "thread-normal")
            .expect("普通会话在");
        assert!(!normal.workbench_bound, "普通会话不该被标 workbench_bound");
        // 排序：updated_at_ms DESC → 工作台(5000) 在普通(3000) 前。
        let pos_wb = sessions
            .iter()
            .position(|s| s.thread_id == "thread-workbench")
            .expect("wb");
        let pos_n = sessions
            .iter()
            .position(|s| s.thread_id == "thread-normal")
            .expect("n");
        assert!(
            pos_wb < pos_n,
            "按 updated_at_ms 倒序·工作台(5000) 应在普通(3000) 前"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 软着陆：读 store 失败（路径不存在）→ 返回原列表不变 + warning、不 Err 断列表。
    #[test]
    fn workbench_bound_merge_soft_lands_when_state_unreadable() {
        let dir = temp_codex_home("workbench-bound-softland");
        fs::create_dir_all(&dir).expect("mkdir");
        let db_path = dir.join("state.sqlite");
        create_codex_threads_db(&db_path);
        insert_codex_thread(&db_path, "thread-normal", "普通会话", 3_000, 0, 1);
        let mut sessions = read_first_page(&db_path);
        let before = sessions.clone();
        let missing_state = dir.join("nonexistent-workflow-state.json");
        let warnings = merge_workbench_bound_sessions(
            &mut sessions,
            &missing_state,
            &db_path,
            0,
            false,
            false,
        );
        assert_eq!(sessions, before, "软着陆：读 store 失败返回原列表不变");
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("workbench_bound_sessions_skipped_state_unreadable")),
            "应出软着陆 warning：{warnings:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 后页（offset>0）不重复注入：工作台会话只在首页并一次。
    #[test]
    fn workbench_bound_merge_skips_later_pages() {
        let dir = temp_codex_home("workbench-bound-laterpage");
        fs::create_dir_all(&dir).expect("mkdir");
        let db_path = dir.join("state.sqlite");
        create_codex_threads_db(&db_path);
        insert_codex_thread(&db_path, "thread-normal", "普通会话", 3_000, 0, 1);
        insert_codex_thread(
            &db_path,
            "thread-workbench",
            "交办任务专用会话",
            5_000,
            0,
            0,
        );
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(
            &workflow_state_path,
            json!({
                "workflow_node_session_bindings": [
                    { "native_thread_id": "thread-workbench", "node_id": "wf:node:codex-dev" }
                ]
            })
            .to_string(),
        )
        .expect("write state");
        let mut sessions = read_first_page(&db_path);
        let before = sessions.clone();
        let warnings = merge_workbench_bound_sessions(
            &mut sessions,
            &workflow_state_path,
            &db_path,
            100,
            false,
            false,
        );
        assert_eq!(sessions, before, "后页不注入工作台会话");
        assert!(warnings.is_empty(), "后页跳过·无 warning：{warnings:?}");
        let _ = fs::remove_dir_all(dir);
    }
}

#[tauri::command]
fn load_workflow_state_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateSnapshot, String> {
    read_workflow_state_snapshot(&state.workflow_state_path)
}

#[tauri::command]
fn load_plan_authorization_store(
    state: tauri::State<'_, AppState>,
) -> Result<PlanAuthorizationStoreV1, String> {
    plan_authorization_store::load_store(&state.workflow_state_path, unix_timestamp_ms())
}

#[tauri::command]
fn create_plan_authorization(
    request: CreatePlanAuthorizationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreatePlanAuthorizationOutput, String> {
    plan_authorization_store::create_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_plan_authorization_user_confirmation(
    request: RecordPlanAuthorizationUserConfirmationInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    let result = plan_authorization_store::record_user_confirmation(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-user-{}", unix_timestamp_nanos()),
    )?;
    let captured_at = unix_timestamp_string();
    let project_id_value = project_id(&request.project_root);
    let workflow_id_value = default_workflow_id(&request.project_root);
    let ctx = memory_daily_loop::MemoryDailyLoopContext {
        project_root: &request.project_root,
        project_id: &project_id_value,
        workflow_id: &workflow_id_value,
        workflow_node_id: None,
        run_unit_id: None,
        actor_id: &request.actor_id,
        created_at: &captured_at,
    };
    l5_capture_governance_best_effort(
        &state.workflow_state_path,
        memory_daily_loop::plan_authorization_capture_input(&ctx, &request),
        &captured_at,
        "plan-auth",
    );
    Ok(result)
}

#[tauri::command]
fn record_plan_authorization_global_boundary_review(
    request: RecordPlanAuthorizationGlobalBoundaryReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    plan_authorization_store::record_global_boundary_review(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-plan-authorization-boundary-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn record_global_boundary_review(
    request: RecordGlobalBoundaryReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordGlobalBoundaryReviewOutput, String> {
    plan_authorization_store::record_global_boundary_review_with_proposal(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-global-boundary-review-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn revoke_plan_authorization(
    request: RevokePlanAuthorizationInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    plan_authorization_store::revoke_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-revoke-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn inspect_auto_dispatch_authorization(
    request: AutoDispatchGuardInput,
    state: tauri::State<'_, AppState>,
) -> Result<AutoDispatchGuardResult, String> {
    plan_authorization_store::inspect_auto_dispatch_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-auto-dispatch-scope-check-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_project_director_task_plan(
    request: PreviewProjectDirectorTaskPlanInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectDirectorTaskPlan, String> {
    let index = read_index(&state)?;
    preview_project_director_task_plan_for_index_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn prepare_authorized_auto_dispatch(
    request: PrepareAuthorizedAutoDispatchInput,
    state: tauri::State<'_, AppState>,
) -> Result<AuthorizedPreparedDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_authorized_auto_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn preview_h5_project_workflow_dispatch(
    request: H5ProjectWorkflowDispatchPreviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<H5ProjectWorkflowDispatchPreview, String> {
    h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn preview_real_execution_product_command(
    request: PreviewRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPreview, String> {
    real_execution_command::preview_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn prepare_real_execution_product_command(
    request: PrepareRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPrepareOutput, String> {
    real_execution_command::prepare_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn record_real_execution_product_command_decision(
    request: RecordRealExecutionProductCommandDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    real_execution_command::record_real_execution_product_command_decision_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn confirm_real_execution_product_command(
    request: ConfirmRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    real_execution_command::confirm_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn run_real_execution_product_command_phase_a(
    request: RunRealExecutionProductCommandPhaseAInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseAOutput, String> {
    real_execution_command::run_real_execution_product_command_phase_a_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-real-exec-command-phase-a-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_real_execution_product_command_phase_b(
    request: RunRealExecutionProductCommandPhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    real_execution_command::run_real_execution_product_command_phase_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-real-exec-command-phase-b-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_real_execution_product_command_new_session_phase_b(
    request: RunRealExecutionProductCommandNewSessionPhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    real_execution_command::run_real_execution_product_command_new_session_phase_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-real-exec-command-new-session-phase-b-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_project_workflow_automation_phase_a(
    request: ProjectWorkflowAutomationInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationResult, String> {
    project_workflow_automation::run_project_workflow_automation_phase_a_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-project-workflow-automation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_j2_b_b1(
    request: ProjectWorkflowAutomationJ2BB1Input,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationJ2BB1Output, String> {
    // 旁路封堵：j2_b_b1 真跑写死 J2_B_B1_PROJECT_ROOT（非测试 Documents/mario test）→ 此 gate 永远拦、封死真跑。
    require_test_project_path_lock(
        project_workflow_automation::J2_B_B1_PROJECT_ROOT,
        "run_project_workflow_automation_j2_b_b1",
    )?;
    project_workflow_automation::run_project_workflow_automation_j2_b_b1_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-b-b1-project-workflow-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_j2_b_b2(
    request: ProjectWorkflowAutomationJ2BB2Input,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationJ2BB2Output, String> {
    // 旁路封堵：j2_b_b2 真跑写死 J2_B_B2_PROJECT_ROOT（非测试 product-line/tmp 隔离项目、workspace-write）→ 此 gate 永远拦、封死真跑。
    require_test_project_path_lock(
        project_workflow_automation::J2_B_B2_PROJECT_ROOT,
        "run_project_workflow_automation_j2_b_b2",
    )?;
    project_workflow_automation::run_project_workflow_automation_j2_b_b2_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-b-b2-project-workflow-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_k3_b(
    request: ProjectWorkflowAutomationK3BInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationK3BOutput, String> {
    ensure_k3_b_tauri_no_real_harness_request(&request)?;
    // 旁路封堵：k3_b 真跑 root = request.project_root（兜底 config 也是非测试）→ 非测试一律拦。
    require_test_project_path_lock(
        request.project_root.as_deref().unwrap_or_default(),
        "run_project_workflow_automation_k3_b",
    )?;
    project_workflow_automation::run_project_workflow_automation_k3_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-k3-b-project-workflow-{}", unix_timestamp_nanos()),
    )
}

fn ensure_k3_b_tauri_no_real_harness_request(
    request: &ProjectWorkflowAutomationK3BInput,
) -> Result<(), String> {
    if request
        .runtime_prompt_body
        .as_deref()
        .is_some_and(|body| !body.is_empty())
    {
        return Err(
            "k3_b_real_execution_requires_dedicated_level_b_authorization".to_string(),
        );
    }
    Ok(())
}

// S3 L5 best-effort 采集挂钩：治理命令记录成功后调它把事件采成待确认候选；**失败只吞成 warning、绝不改主返回**
// （采集是旁路、不能拖垮治理命令）。挂在 #[tauri::command] wrapper（不挂 _at）→ 既有 _at 测试 0-diff、不破回归。
fn l5_capture_governance_best_effort(
    path: &std::path::Path,
    mapped: Result<CaptureMemoryEventInput, String>,
    captured_at: &str,
    tag: &str,
) {
    let input = match mapped {
        Ok(input) => input,
        Err(_) => return,
    };
    let nanos = unix_timestamp_nanos();
    let cap = format!("write-l5-{tag}-cap-{nanos}");
    let obs = format!("write-l5-{tag}-obs-{nanos}");
    let cand = format!("write-l5-{tag}-cand-{nanos}");
    let write_ids = memory_daily_loop::MemoryDailyLoopWriteIds {
        capture_write_id: &cap,
        observation_write_id: &obs,
        candidate_write_id: &cand,
    };
    let _ =
        memory_daily_loop::capture_governance_event_best_effort(path, &input, captured_at, &write_ids);
}

#[tauri::command]
fn record_worker_structured_report(
    request: WorkerStructuredReportInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let result = record_worker_structured_report_at(&state.workflow_state_path, &request)?;
    let captured_at = unix_timestamp_string();
    let ctx = memory_daily_loop::MemoryDailyLoopContext {
        project_root: &request.project_root,
        project_id: &request.project_id,
        workflow_id: &request.workflow_id,
        workflow_node_id: Some(&request.workflow_node_id),
        run_unit_id: None,
        actor_id: &request.actor_role,
        created_at: &captured_at,
    };
    l5_capture_governance_best_effort(
        &state.workflow_state_path,
        memory_daily_loop::worker_report_capture_input(&ctx, &request),
        &captured_at,
        "wr",
    );
    Ok(result)
}

#[tauri::command]
fn record_project_director_process_fact_decision(
    request: ProjectDirectorProcessFactDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectDirectorProcessFactDecisionResult, String> {
    record_project_director_process_fact_decision_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn record_global_final_result_review(
    request: GlobalFinalResultReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let result = record_global_final_result_review_at(&state.workflow_state_path, &request)?;
    let captured_at = unix_timestamp_string();
    let ctx = memory_daily_loop::MemoryDailyLoopContext {
        project_root: &request.project_root,
        project_id: &request.project_id,
        workflow_id: &request.workflow_id,
        workflow_node_id: None,
        run_unit_id: None,
        actor_id: &request.actor_id,
        created_at: &captured_at,
    };
    l5_capture_governance_best_effort(
        &state.workflow_state_path,
        memory_daily_loop::final_review_capture_input(&ctx, &request),
        &captured_at,
        "final-review",
    );
    Ok(result)
}

#[tauri::command]
fn record_user_result_decision(
    request: UserResultDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    record_user_result_decision_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn generate_stage_c_acceptance_summary(
    request: GenerateStageCAcceptanceSummaryInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    generate_stage_c_acceptance_summary_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn load_project_consultation_proposal_store(
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposalStoreV1, String> {
    project_consultation_proposal_store::load_store(&state.workflow_state_path, unix_timestamp_ms())
}

#[tauri::command]
fn create_project_consultation_proposal(
    request: CreateProjectConsultationProposalInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateProjectConsultationProposalOutput, String> {
    project_consultation_proposal_store::create_proposal(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-project-consultation-proposal-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn render_project_consultation_proposal_markdown(
    request: RenderProjectConsultationProposalMarkdownInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposalMarkdown, String> {
    project_consultation_proposal_store::render_markdown(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
    )
}

#[tauri::command]
fn record_project_consultation_proposal_decision(
    request: RecordProjectConsultationProposalDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordProjectConsultationProposalDecisionOutput, String> {
    project_consultation_proposal_store::record_decision(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-project-consultation-proposal-decision-{}",
            unix_timestamp_nanos()
        ),
        &format!(
            "write-project-consultation-plan-authorization-{}",
            unix_timestamp_nanos()
        ),
        &format!(
            "write-project-consultation-plan-confirm-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn load_session_continuation_store(
    state: tauri::State<'_, AppState>,
) -> Result<SessionContinuationStoreV1, String> {
    session_continuation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn confirm_controlled_session_continuation(
    request: ConfirmControlledSessionContinuationInput,
    state: tauri::State<'_, AppState>,
) -> Result<ConfirmControlledSessionContinuationOutput, String> {
    session_continuation_store::confirm_continuation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-session-continuation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_stub(
    request: RunControlledSessionContinuationStubInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationStubOutput, String> {
    session_continuation_store::run_stub(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-session-continuation-stub-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn inspect_controlled_session_continuation_real_resume_authorization(
    request: InspectControlledSessionContinuationRealResumeInput,
    state: tauri::State<'_, AppState>,
) -> Result<InspectControlledSessionContinuationRealResumeOutput, String> {
    session_continuation_store::inspect_real_resume_authorization(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-preflight-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_real_resume_phase_a(
    request: RunControlledSessionContinuationRealResumePhaseAInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationRealResumePhaseAOutput, String> {
    session_continuation_store::run_real_resume_phase_a(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-phase-a-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_real_resume_phase_b(
    request: RunControlledSessionContinuationRealResumePhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationRealResumePhaseBOutput, String> {
    // 旁路封堵：H5 直连真 resume 此前无 path-lock（A 线 store 不动，只在命令包装层补闸）。
    require_test_project_path_lock(
        &request.authorization.project_root,
        "run_controlled_session_continuation_real_resume_phase_b",
    )?;
    session_continuation_store::run_real_resume_phase_b(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-phase-b-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn load_blackboard_candidate_store(
    state: tauri::State<'_, AppState>,
) -> Result<BlackboardCandidateStoreV1, String> {
    blackboard_candidate_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn record_blackboard_candidate_decision(
    request: RecordBlackboardCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordBlackboardCandidateDecisionOutput, String> {
    control_core::validate_blackboard_candidate_decision(
        blackboard_entry_kind_name(request.entry_kind),
        blackboard_target_kind_name(request.target_kind),
        blackboard_state_name(request.requested_state),
    )?;
    blackboard_candidate_store::record_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_observation_store(state: tauri::State<'_, AppState>) -> Result<ObservationStoreV1, String> {
    observation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_capture_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryCaptureStoreV1, String> {
    memory_capture_bus::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn capture_memory_event(
    request: CaptureMemoryEventInput,
    state: tauri::State<'_, AppState>,
) -> Result<CaptureMemoryEventOutput, String> {
    let timestamp = unix_timestamp_string();
    memory_capture_bus::capture_event(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-memory-capture-{}", unix_timestamp_nanos()),
        &format!("write-memory-capture-observation-{}", unix_timestamp_nanos()),
        &format!("write-memory-capture-candidate-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn create_observation(
    request: CreateObservationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateObservationOutput, String> {
    create_observation_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-observation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn create_memory_candidate_from_observation(
    request: CreateMemoryCandidateFromObservationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateMemoryCandidateFromObservationOutput, String> {
    let timestamp = unix_timestamp_string();
    create_memory_candidate_from_observation_at(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-observation-candidate-{}", unix_timestamp_nanos()),
        &format!(
            "write-memory-candidate-from-observation-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn preview_task_memory_packet(
    request: TaskMemoryPacketBuildInput,
    state: tauri::State<'_, AppState>,
) -> Result<TaskMemoryPacketBuildOutput, String> {
    preview_task_memory_packet_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn load_memory_lint_store(state: tauri::State<'_, AppState>) -> Result<MemoryLintStoreV1, String> {
    memory_lint_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_entity_relation_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryEntityRelationStoreV1, String> {
    memory_entity_relation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_pattern_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryPatternStoreV1, String> {
    mature_pattern_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn preview_mature_patterns(
    request: PreviewMaturePatternsInput,
    state: tauri::State<'_, AppState>,
) -> Result<MaturePatternPreviewOutput, String> {
    mature_pattern_governance::preview_mature_patterns(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_mature_pattern_decision(
    request: RecordMaturePatternDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMaturePatternDecisionOutput, String> {
    mature_pattern_governance::record_mature_pattern_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-pattern-{}", unix_timestamp_nanos()),
        &format!("write-formal-memory-pattern-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_memory_entity_relation_candidates(
    request: PreviewMemoryEntityRelationCandidatesInput,
    state: tauri::State<'_, AppState>,
) -> Result<MemoryEntityRelationPreviewOutput, String> {
    memory_entity_relation_governance::preview_candidates(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_memory_entity_alias_decision(
    request: RecordMemoryEntityAliasDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryEntityAliasDecisionOutput, String> {
    memory_entity_relation_governance::record_alias_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-entity-alias-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_entity_merge_decision(
    request: RecordMemoryEntityMergeDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryEntityMergeDecisionOutput, String> {
    memory_entity_relation_governance::record_merge_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-entity-merge-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_relation_candidate_decision(
    request: RecordMemoryRelationCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryRelationCandidateDecisionOutput, String> {
    memory_entity_relation_governance::record_relation_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-relation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_memory_lint(
    request: MemoryLintRunInput,
    state: tauri::State<'_, AppState>,
) -> Result<MemoryLintRunOutput, String> {
    run_memory_lint_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-lint-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_memory_candidate_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryCandidateStoreV1, String> {
    memory_candidate_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn create_memory_candidate(
    request: CreateMemoryCandidateInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateMemoryCandidateOutput, String> {
    memory_candidate_store::create_candidate(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_candidate_decision(
    request: RecordMemoryCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryCandidateDecisionOutput, String> {
    memory_candidate_store::record_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn adopt_memory_candidate_to_formal_memory(
    request: AdoptMemoryCandidateInput,
    state: tauri::State<'_, AppState>,
) -> Result<AdoptMemoryCandidateOutput, String> {
    let timestamp = unix_timestamp_string();
    adopt_memory_candidate_to_formal_memory_at(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-memory-candidate-adoption-{}", unix_timestamp_nanos()),
        &format!("write-formal-memory-adoption-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_formal_memory_store(
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryStoreV1, String> {
    formal_memory_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn create_formal_memory_record(
    request: CreateFormalMemoryRecordInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateFormalMemoryRecordOutput, String> {
    create_formal_memory_record_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_formal_memory_lifecycle_operation(
    request: FormalMemoryLifecyclePreviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryLifecyclePreview, String> {
    formal_memory_lifecycle::preview_operation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_formal_memory_lifecycle_operation(
    request: FormalMemoryLifecycleInput,
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryLifecycleOutput, String> {
    formal_memory_lifecycle::record_operation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-formal-memory-lifecycle-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn initialize_workflow_state(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    initialize_workflow_state_at(&state.workflow_state_path)
}

#[tauri::command]
fn bootstrap_project_workflow(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    let project = find_index_project(&index, &request.path)
        .ok_or_else(|| "项目不在当前索引内，已拒绝创建本地工作流草稿".to_string())?;
    bootstrap_project_workflow_at(&state.workflow_state_path, &project)
}

#[tauri::command]
fn create_task_draft(
    request: TaskDraftRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    create_task_draft_for_index_project_at(&state.workflow_state_path, &index, &request)
}

fn create_task_draft_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskDraftRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(&index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝登记任务包草稿".to_string());
    }
    create_task_draft_at(path, request)
}

#[tauri::command]
fn render_task_package_preview(
    request: TaskPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackagePreview, String> {
    let index = read_index(&state)?;
    render_task_package_preview_for_index_project_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn copy_task_package_preview(
    request: TaskPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let preview = render_task_package_preview_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )?;
    copy_to_clipboard(&preview.markdown)?;
    Ok("已复制任务包 Markdown 预览文本；没有写入真实任务包文件。".to_string())
}

fn render_task_package_preview_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackagePreviewRequest,
) -> Result<TaskPackagePreview, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝渲染任务包预览".to_string())?;
    render_task_package_preview_at(path, &project, request)
}

#[tauri::command]
fn update_task_package_draft_fields(
    request: TaskPackageFieldsUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    update_task_package_draft_fields_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn update_task_package_draft_fields_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageFieldsUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝更新任务包字段".to_string());
    }
    update_task_package_draft_fields_at(path, request)
}

#[tauri::command]
fn correct_task_package_dispatch_fields(
    request: TaskPackageDispatchFieldsCorrectionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    correct_task_package_dispatch_fields_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn correct_task_package_dispatch_fields_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageDispatchFieldsCorrectionRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝修正派发字段".to_string());
    }
    let update_request = TaskPackageFieldsUpdateRequest {
        project_root: request.project_root.clone(),
        work_item_id: request.work_item_id.clone(),
        fields: request.fields.clone(),
    };
    update_task_package_fields_at(
        path,
        &update_request,
        TaskPackageFieldWriteMode::DispatchCorrection,
    )
}

#[tauri::command]
fn generate_task_package_file(
    request: TaskPackageFileGenerationRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackageFileGenerationResult, String> {
    let index = read_index(&state)?;
    generate_task_package_file_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
        &default_task_package_output_dir(),
    )
}

fn generate_task_package_file_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageFileGenerationRequest,
    tasks_dir: &Path,
) -> Result<TaskPackageFileGenerationResult, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝生成真实任务包文件".to_string())?;
    generate_task_package_file_at(path, &project, request, tasks_dir)
}

#[tauri::command]
fn inspect_task_package_dispatch_readiness(
    request: TaskPackageDispatchReadinessRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackageDispatchReadiness, String> {
    let index = read_index(&state)?;
    inspect_task_package_dispatch_readiness_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn inspect_task_package_dispatch_readiness_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageDispatchReadinessRequest,
) -> Result<TaskPackageDispatchReadiness, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝检查任务包派发准备状态".to_string())?;
    inspect_task_package_dispatch_readiness_at(path, &project, request)
}

#[tauri::command]
fn inspect_workflow_run_check(
    request: WorkflowRunCheckRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowRunCheck, String> {
    let index = read_index(&state)?;
    inspect_workflow_run_check_for_index_at(&state.workflow_state_path, &index, &request)
}

fn inspect_workflow_run_check_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowRunCheckRequest,
) -> Result<WorkflowRunCheck, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝检查工作流运行性".to_string())?;
    inspect_workflow_run_check_at(path, &project, request)
}

#[tauri::command]
fn update_work_item_state(
    request: WorkItemStateUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    update_work_item_state_for_index_project_at(&state.workflow_state_path, &index, &request)
}

fn update_work_item_state_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &WorkItemStateUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝推进工作项状态".to_string());
    }
    update_work_item_state_at(path, request)
}

#[tauri::command]
fn bind_workflow_node_codex_session(
    request: WorkflowNodeSessionBindRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    bind_workflow_node_codex_session_for_index_at(&state.workflow_state_path, &index, &request)
}

fn bind_workflow_node_codex_session_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeSessionBindRequest,
) -> Result<WorkflowStateMutationResult, String> {
    bind_workflow_node_codex_session_for_index_with_provenance_at(
        path,
        index,
        request,
        &WorkflowNodeSessionBindingProvenance::user_selected_existing(),
    )
}

fn bind_workflow_node_codex_session_for_index_with_provenance_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeSessionBindRequest,
    provenance: &WorkflowNodeSessionBindingProvenance,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝绑定节点会话".to_string());
    }
    // 路A：静态快照找不到 → 回退实时 sqlite（用户能绑近期/新会话，不被 5/31 快照卡）。
    let session = find_index_thread_or_sqlite(index, &request.thread_id)
        .ok_or_else(|| "会话不在当前索引内（含实时 sqlite），已拒绝绑定节点会话".to_string())?;
    bind_workflow_node_codex_session_with_provenance_at(path, request, &session, provenance)
}

#[tauri::command]
fn unbind_workflow_node_codex_session(
    request: WorkflowNodeSessionUnbindRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    unbind_workflow_node_codex_session_for_index_at(&state.workflow_state_path, &index, &request)
}

fn unbind_workflow_node_codex_session_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeSessionUnbindRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝解绑节点会话".to_string());
    }
    unbind_workflow_node_codex_session_at(path, request)
}

#[tauri::command]
fn prepare_workflow_node_dispatch(
    request: WorkflowNodeDispatchPrepareRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_workflow_node_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

fn prepare_workflow_node_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝准备节点派发".to_string());
    }
    prepare_workflow_node_dispatch_at(path, index, request)
}

/// 工作流引擎解封·固定测试项目 path-lock 闸。决策见
/// `decisions/2026-06-22-p3-test-project-real-run-light-tier-v1.md`（在
/// `...2026-06-21-next-step-unseal...` 之上，把测试项目真跑下放为轻档）。
/// 2026-06-22 P3-A：去掉原 env-CONFIRM「你确定」belt（重档遗留、现多余），
/// **只保留 path-lock**——只有「目标 == 固定测试项目」才放行真实执行；其余任何
/// 真实项目 → 维持 legacy blocked，**非测试真实项目行为零变化、仍 sealed**。
/// 沙箱仍由 `command_plan_for` 强制（codex 关在测试目录），本次一个字节不动。
const WORKFLOW_ENGINE_TEST_PROJECT_ROOT: &str = "/Users/yoyi/codex-workflow-mario-test";

fn workflow_engine_test_project_unsealed(project_root: &str) -> bool {
    project_root == WORKFLOW_ENGINE_TEST_PROJECT_ROOT
}

// 旁路封堵（2026-06-24·用户授权破例封 deprecated 旧桩旁路）：旧桩 automation(j2_b_b1/k3_b) 与 H5 直连
// (controlled_session_continuation) 的真 runner 产品入口此前**无 path-lock**，可绕过 S1 闸真跑 codex 进
// 非测试项目。在这些 #[tauri::command] 产品入口补 path-lock：只放行固定测试项目，非测试 → legacy blocked、
// 不起 runner。与 execute_workflow_node_dispatch 同款；gate 在命令包装层（不被单测调）→ A 线 store / 沙箱 /
// 旧桩内层 _at/_with_runner 一字不动、既有测试零影响。**gate 在「真跑实际用的那个 root」**（j2_b_b1 写死
// J2_B_B1_PROJECT_ROOT，故 gate 它 → 永远拦＝封死该 deprecated 入口的真跑）。
fn require_test_project_path_lock(project_root: &str, command_name: &str) -> Result<(), String> {
    if workflow_engine_test_project_unsealed(project_root) {
        Ok(())
    } else {
        Err(legacy_product_command_blocked_message(command_name))
    }
}

// 站 3b（2026-07-12 用户拍板；任务包 tasks/2026-07-12-orchestrator-station3b-readonly-real-project-
// mario-test-v1.md）：唯一获批的「真实项目只读解封」，与 S1 闸**并列**——不放宽
// workflow_engine_test_project_unsealed / require_test_project_path_lock 本体，legacy 封条（含写死同一
// 目录的 j2_b_b1）继续 blocked。只挂主管编排链路（发射器 + 主管派发/追问适配器 + 前端开关），不挂经典
// 管线、legacy 旧桩、自动连环。判定 = 项目根**精确相等** ∧ 写根为空；任何写根、任何其它项目 → 不解封。
const STATION_3B_READONLY_PROJECT_ROOT: &str = "/Users/yoyi/Documents/mario test";

// 根等值判定：仅用于「授权段写根不在手上」的次级闸（发射器命令面/argv 终验——入口闸已先按
// 「根+零写根」全判过）。会拿到写根的地方一律用 station3b_readonly_project_unsealed 全判。
fn station3b_readonly_project_root(project_root: &str) -> bool {
    project_root == STATION_3B_READONLY_PROJECT_ROOT
}

fn station3b_readonly_project_unsealed(project_root: &str, allowed_write_roots: &[String]) -> bool {
    station3b_readonly_project_root(project_root) && allowed_write_roots.is_empty()
}

// S1 执行层合一闸的 authorization_complete 判定（站 3b 扩展；判决体 decide_real_execution_command
// 一字不动）：测试项目 path-lock 命中，或「主管授权派发（带已核 prepared dispatch/授权段）∧ 3b 项目
// ∧ 零写根」。经典画布(B 线)喂 3b 项目时 supervisor_authorized=false → 此处照拒（其命令入口的
// S1 闸也已先拦，双层兜底）。
fn real_execution_authorization_complete(
    project_root: &str,
    write_roots: &[String],
    supervisor_authorized: bool,
) -> bool {
    workflow_engine_test_project_unsealed(project_root)
        || (supervisor_authorized && station3b_readonly_project_unsealed(project_root, write_roots))
}

#[tauri::command]
fn execute_workflow_node_dispatch(
    request: WorkflowNodeDispatchExecuteRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    // path-lock 闸:仅固定测试项目放行(2026-06-22 P3-A 去 env belt)。非测试真实项目
    // → 维持原 blocked、零变化。沙箱仍由 command_plan_for 强制(测试目录,字节未动)。
    if !workflow_engine_test_project_unsealed(&request.project_root) {
        return Err(legacy_product_command_blocked_message(
            "execute_workflow_node_dispatch",
        ));
    }
    // 已通过双闸:读索引、取 readback 库路径、用复用真 spawn 的适配器执行单节点。
    // 真实现自带 find_index_project 二次校验;沙箱由 command_plan_for 强制(见适配器)。
    let index = read_index(&state)?;
    let readback_db_path = codex_db::default_state_db_path();
    let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
    execute_workflow_node_dispatch_for_index_at(
        &state.workflow_state_path,
        &index,
        &readback_db_path,
        &runner,
        &request,
    )
}

fn execute_workflow_node_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝执行节点派发".to_string());
    }
    execute_workflow_node_dispatch_at(path, index, readback_db_path, runner, request)
}

// P3 实验面真跑（架构方案 §9 的 A 映射）。实验节点 id 是自由生成的、对不上 workflow-state，
// 所以不能直接走 execute_workflow_node_dispatch；这条命令在固定测试项目里**自动建一个临时
// work_item + 绑会话**，再走与项目面同一条已验派发路径真跑（= 复刻 real_run_full_dispatch_resume
// 配方，由实验节点的会话/prompt 驱动）。
//
// 安全：目标**恒为固定测试项目**（前端传不进 project_root），path-lock 与
// execute_workflow_node_dispatch 同一道闸；沙箱由 command_plan_for 强制（字节未动）。
// 会话策略：resume 用给定 thread_id；new 不启用（resume-only，2026-06-22 用户拍板 C）——唯一
// mint+拿 id 的路径 manual_relay new_session 后面卡在场 env 闸，启用要么加摩擦要么动安全闸，
// 故本期只做 resume；选 new 明确报错不假跑。
#[tauri::command]
fn execute_experiment_node_dispatch(
    request: ExperimentNodeDispatchExecuteRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    let readback_db_path = codex_db::default_state_db_path();
    let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
    execute_experiment_node_dispatch_at(
        &state.workflow_state_path,
        &index,
        &readback_db_path,
        &runner,
        &request,
    )
}

fn execute_experiment_node_dispatch_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &ExperimentNodeDispatchExecuteRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    // 目标恒为固定测试项目（path-lock：与 execute_workflow_node_dispatch 同一道闸）。
    // 前端无法传入 project_root，所以不会被重定向到真实项目。
    let project_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    if !workflow_engine_test_project_unsealed(project_root) {
        return Err(legacy_product_command_blocked_message(
            "execute_experiment_node_dispatch",
        ));
    }

    // C · 会话解析：resume 用给定 thread_id；new 不启用（resume-only，用户拍板 C）。
    let thread_id = match request.session_mode.trim() {
        "resume" => request
            .thread_id
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| "实验节点续已有会话需要 thread_id".to_string())?
            .to_string(),
        "new" => {
            // 决策（2026-06-22）：本期只做 resume-only，不启用「开新会话」。开新会话的唯一
            // mint+拿 id 路径（manual_relay new_session）后面卡着在场 env 闸，启用它要么加摩擦
            // 要么动安全闸；用户拍板暂不做。请选「续已有会话」并给一条已存在的 codex 会话。
            return Err(
                "实验面本期只支持「续已有会话」（resume-only，用户拍板）；「开新会话」未启用。请选续已有会话并给一条已存在的 thread。"
                    .to_string(),
            )
        }
        other => return Err(format!("未知会话策略：{other}")),
    };

    // 会话必须能找到且有 rollout（bind/dispatch 前置）。路A 后：静态快照找不到则回退实时 sqlite，
    // 所以近期/新 mint 的会话也能认。
    let session = find_index_thread_or_sqlite(index, &thread_id)
        .ok_or_else(|| "会话不在当前索引内（含实时 sqlite），已拒绝实验真跑".to_string())?;
    if !session.rollout_exists {
        return Err("会话在索引中缺少 rollout，已拒绝实验真跑".to_string());
    }
    let project = find_index_project(index, project_root)
        .ok_or_else(|| "固定测试项目不在当前索引内，已拒绝实验真跑".to_string())?;

    // prompt / sandbox 校验（validate_user_reviewed_instruction 的子集，提前给清楚错）。
    let objective = request.objective.trim();
    if objective.is_empty() {
        return Err("实验节点缺少 prompt（objective），无法真跑".to_string());
    }
    let sandbox_mode = request.sandbox_mode.trim();
    if !matches!(sandbox_mode, "read-only" | "workspace-write") {
        return Err("sandbox 只允许 read-only 或 workspace-write".to_string());
    }
    let summary = {
        let trimmed = request.summary.trim();
        if trimmed.is_empty() {
            "实验节点真跑".to_string()
        } else {
            trimmed.to_string()
        }
    };

    let workflow_id = default_workflow_id(project_root);
    // 工作流骨架缺时才 bootstrap（已存在则不重建，避免每次真跑都写一遍）。
    let needs_bootstrap = match read_workflow_state_value(path) {
        Ok(value) => !workflow_exists(&value, &workflow_id),
        Err(_) => true,
    };
    if needs_bootstrap {
        bootstrap_project_workflow_at(path, &project)?;
    }

    // A · 自动建临时 work_item（唯一标题避免去重 no-op），推进到 ready_to_dispatch。
    let stamp = unix_timestamp_string();
    let title = format!("experiment-temp-{stamp}");
    create_task_draft_at(
        path,
        &TaskDraftRequest {
            project_root: project_root.to_string(),
            title: title.clone(),
            objective: objective.to_string(),
            assigned_role: Some("codex-dev".to_string()),
        },
    )?;
    let work_item_id = {
        let value = read_workflow_state_value(path)?;
        value
            .get("work_items")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().rev().find(|item| {
                    optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id.as_str())
                        && optional_string_from(item, "title").as_deref() == Some(title.as_str())
                })
            })
            .and_then(|item| optional_string_from(item, "work_item_id"))
            .ok_or_else(|| "刚建的临时 work_item 找不回，已中止实验真跑".to_string())?
    };
    update_work_item_state_at(
        path,
        &WorkItemStateUpdateRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.clone(),
            next_state: "ready_to_dispatch".to_string(),
        },
    )?;

    // C · 把会话绑到 codex-dev 节点 + 这个临时 work_item（resume 路径）。
    let node_id = format!("{workflow_id}:node:codex-dev");
    bind_workflow_node_codex_session_at(
        path,
        &WorkflowNodeSessionBindRequest {
            project_root: project_root.to_string(),
            node_id: node_id.clone(),
            work_item_id: Some(work_item_id.clone()),
            thread_id: thread_id.clone(),
        },
        &session,
    )?;

    // D · 走与项目面同一条已验派发真跑路径。后端构造完整 instruction（前端那条 builder 发空
    // allowed_reads/forbidden_actions/required_return，过不了 validate_user_reviewed_instruction）。
    let exec_request = WorkflowNodeDispatchExecuteRequest {
        project_root: project_root.to_string(),
        node_id,
        work_item_id,
        prompt_kind: "user_reviewed_instruction".to_string(),
        user_reviewed_instruction: Some(UserReviewedInstructionInput {
            instruction_id: format!("instruction:experiment:{stamp}"),
            summary,
            objective: objective.to_string(),
            execution_cwd: project_root.to_string(),
            sandbox_mode: sandbox_mode.to_string(),
            allowed_write_roots: if sandbox_mode == "workspace-write" {
                vec![project_root.to_string()]
            } else {
                vec![]
            },
            allowed_reads: vec![project_root.to_string()],
            allowed_writes: if sandbox_mode == "workspace-write" {
                vec![project_root.to_string()]
            } else {
                vec![]
            },
            forbidden_actions: vec![
                "不读取 auth.json、.env、密钥、token 或授权文件。".to_string(),
                "不读取完整 transcript。".to_string(),
                "不运行 harness。".to_string(),
            ],
            timeout_seconds: request.timeout_seconds.unwrap_or(600).max(1),
            max_retries: 0,
            required_return: vec!["本步做了什么".to_string(), "改了哪些文件".to_string()],
            prompt_preview: Some(objective.to_string()),
        }),
    };
    execute_workflow_node_dispatch_for_index_at(path, index, readback_db_path, runner, &exec_request)
}

// P3 项目面真跑（架构方案 §9 的 C 映射）。项目画布只读运行态的节点 = workflow-state 的 work_item
// 本体（无手绑）。「▶ 运行此节点」= 派发那个已存在的 work_item：从它的任务包构造派发指令，走与
// 实验面同一条已验派发路径。忠实于 kickoff「跑节点=派发那个 work_item」——work_item 必须已是
// ready_to_dispatch 且节点已绑会话（resume-only），否则派发自身会清楚报错（不自动推进状态）。
//
// 安全：path-lock 同 execute_workflow_node_dispatch（前端可传 project_root，但非固定测试项目→blocked）。
// 沙箱由 command_plan_for 强制（字节未动）。
#[tauri::command]
fn execute_project_workflow_node(
    request: ProjectWorkflowNodeRunRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    if !workflow_engine_test_project_unsealed(&request.project_root) {
        return Err(legacy_product_command_blocked_message(
            "execute_project_workflow_node",
        ));
    }
    let index = read_index(&state)?;
    let readback_db_path = codex_db::default_state_db_path();
    let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
    execute_project_workflow_node_at(
        &state.workflow_state_path,
        &index,
        &readback_db_path,
        &runner,
        &request,
    )
}

fn execute_project_workflow_node_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectWorkflowNodeRunRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    execute_project_workflow_node_with_authorization_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        None,
    )
}

fn execute_authorized_project_workflow_node_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectWorkflowNodeRunRequest,
    authorization_id: &str,
    allowed_write: &[String],
) -> Result<WorkflowNodeDispatchResult, String> {
    execute_project_workflow_node_with_authorization_at(
        path,
        index,
        readback_db_path,
        runner,
        request,
        Some((authorization_id, allowed_write)),
    )
}

fn execute_project_workflow_node_with_authorization_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &ProjectWorkflowNodeRunRequest,
    supervisor_authorization: Option<(&str, &[String])>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝运行项目节点".to_string())?;
    // 后置C defect#1：workflow_id 不写死 default——优先用请求传入的，否则从 node_id
    // （`{workflow_id}:node:…`）解析，再退回 default。让非默认工作流的节点也能定位/派发。
    let workflow_id = request
        .workflow_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            request
                .node_id
                .split_once(":node:")
                .map(|(prefix, _)| prefix.to_string())
                .filter(|prefix| !prefix.is_empty())
        })
        .unwrap_or_else(|| default_workflow_id(&request.project_root));
    let prepared_authorization = supervisor_authorization
        .map(|(authorization_id, allowed_write)| {
            authorized_prepared_dispatch_for_execution(
                path,
                &request.project_root,
                &workflow_id,
                &request.node_id,
                &request.work_item_id,
                authorization_id,
                allowed_write,
            )
        })
        .transpose()?;
    let mut value = read_workflow_state_value(path)?;
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目下找不到该 workflow；无法运行项目节点".to_string());
    }
    if !node_exists(&value, &workflow_id, &request.node_id) {
        return Err("当前 workflow 下找不到该 node；无法运行项目节点".to_string());
    }
    let std_forbidden = || {
        vec![
            "不读取 auth.json、.env、密钥、token 或授权文件。".to_string(),
            "不读取完整 transcript。".to_string(),
            "不运行 harness。".to_string(),
        ]
    };
    let std_required = || vec!["本步做了什么".to_string(), "改了哪些文件".to_string()];

    // 节点画布载荷（canvas_payload.data）：画布建的节点真跑的 prompt/sandbox/会话策略来源。
    let node_payload = value
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| {
            nodes.iter().find(|n| {
                optional_string_from(n, "workflow_id").as_deref() == Some(workflow_id.as_str())
                    && optional_string_from(n, "node_id").as_deref() == Some(request.node_id.as_str())
            })
        })
        .and_then(|n| n.get("canvas_payload"))
        .cloned();
    let payload_data = node_payload.as_ref().and_then(|p| p.get("data")).cloned();

    // 后置C#2 · 会话 thread：有明确 work item 时优先精确绑定；普通路径才允许回退 node 绑定或画布 resume。
    // 主管授权派发禁止 node fallback，避免 guard 检查旧 thread、实际 dispatch 却使用任务新 thread。
    let exact_work_item_thread = (!request.work_item_id.trim().is_empty())
        .then(|| {
            value
                .get("workflow_node_session_bindings")
                .and_then(Value::as_array)
                .and_then(|bindings| {
                    bindings.iter().find(|binding| {
                        optional_string_from(binding, "workflow_id").as_deref()
                            == Some(workflow_id.as_str())
                            && optional_string_from(binding, "node_id").as_deref()
                                == Some(request.node_id.as_str())
                            && optional_string_from(binding, "work_item_id").as_deref()
                                == Some(request.work_item_id.as_str())
                            && optional_string_from(binding, "lifecycle").as_deref()
                                == Some("active")
                    })
                })
                .and_then(|binding| optional_string_from(binding, "native_thread_id"))
        })
        .flatten();
    let node_thread = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| {
            bindings.iter().find(|b| {
                optional_string_from(b, "workflow_id").as_deref() == Some(workflow_id.as_str())
                    && optional_string_from(b, "node_id").as_deref() == Some(request.node_id.as_str())
                    && optional_string_from(b, "lifecycle").as_deref() == Some("active")
            })
        })
        .and_then(|b| optional_string_from(b, "native_thread_id"));
    let payload_thread = payload_data
        .as_ref()
        .and_then(|d| d.get("session"))
        .filter(|s| optional_string_from(s, "mode").as_deref() == Some("resume"))
        .and_then(|s| optional_string_from(s, "thread_id"))
        .filter(|t| !t.trim().is_empty());
    let thread_id = if supervisor_authorization.is_some() {
        exact_work_item_thread.ok_or_else(|| {
            "主管授权派发缺少当前 work item 的精确 active 会话绑定；拒绝回退旧 node 会话"
                .to_string()
        })?
    } else {
        exact_work_item_thread
            .or(node_thread)
            .or(payload_thread)
            .ok_or_else(|| {
                "该节点没有可用会话（无既有绑定、画布也没设 resume 会话）；resume-only：请先给节点绑一条已有 codex 会话".to_string()
            })?
    };

    // 后置C#2 · work_item + 指令：① 请求给了且存在 → 用它（指令取任务包）；② 否则 → 自动建临时
    // work_item（§9「节点即 work_item」；指令取画布载荷 prompt/sandbox），让画布建的工作流也能真跑。
    let timestamp = unix_timestamp_string();
    let existing_wi = if request.work_item_id.trim().is_empty() {
        None
    } else {
        find_work_item(&value, &workflow_id, &request.work_item_id).cloned()
    };
    let (work_item_id, summary, objective, sandbox_mode, forbidden_actions, required_return) =
        if let Some(work_item) = existing_wi.as_ref() {
            let empty_artifact = json!({});
            let artifact = find_task_package_artifact(&value, &request.work_item_id, work_item)
                .unwrap_or(&empty_artifact);
            let fields = task_package_fields_from(
                work_item,
                artifact,
                &project,
                &workflow_id,
                &request.work_item_id,
            );
            // 原始任务包缺或空 allowed_write 都是只读：绝不采用渲染层的占位文案来扩大写权限。
            let task_package_is_read_only = string_array(artifact, "allowed_write").is_empty();
            let objective = {
                let joined = fields.goals.join("\n");
                if joined.trim().is_empty() {
                    fields.task_name.clone()
                } else {
                    joined
                }
            };
            (
                request.work_item_id.clone(),
                if fields.task_name.trim().is_empty() {
                    "项目节点真跑".to_string()
                } else {
                    fields.task_name.clone()
                },
                objective,
                if task_package_is_read_only {
                    "read-only".to_string()
                } else {
                    "workspace-write".to_string()
                },
                if fields.forbidden_actions.is_empty() {
                    std_forbidden()
                } else {
                    fields.forbidden_actions.clone()
                },
                if fields.required_return.is_empty() {
                    std_required()
                } else {
                    fields.required_return.clone()
                },
            )
        } else {
            let prompt = payload_data
                .as_ref()
                .and_then(|d| optional_string_from(d, "prompt"))
                .unwrap_or_default();
            let label = node_payload
                .as_ref()
                .and_then(|p| optional_string_from(p, "label"))
                .filter(|l| !l.trim().is_empty())
                .unwrap_or_else(|| "项目画布节点".to_string());
            let sandbox = payload_data
                .as_ref()
                .and_then(|d| optional_string_from(d, "sandbox"))
                .filter(|s| matches!(s.as_str(), "read-only" | "workspace-write"))
                .unwrap_or_else(|| "workspace-write".to_string());
            (
                format!("work-item:{workflow_id}:canvas-run:{timestamp}"),
                label,
                prompt,
                sandbox,
                std_forbidden(),
                std_required(),
            )
        };
    if objective.trim().is_empty() {
        return Err("该节点缺少 prompt/目标，无法运行项目节点".to_string());
    }

    // 自动建的临时 work_item 先落盘（bind/dispatch 都重读文件）。
    if existing_wi.is_none() {
        // 积压清理：每跑一次链/节点都自动建一个 canvas_run 临时 work_item（外加一条会话绑定），跑多了在
        // 状态文件里越堆越多（run_check 已剔除它们、不破坏功能，纯属臃肿）。同一 (workflow, node) 的旧
        // canvas_run 件连同其会话绑定一起剔掉，只留这次新建的 → 封顶每节点 1 个临时件 + 1 条绑定。
        // 历史 dispatch 是审计留痕（按 dispatch_id），保留不动。
        let stale_ids: Vec<String> = value
            .get("work_items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter(|wi| {
                        optional_string_from(wi, "source_kind").as_deref() == Some("canvas_run")
                            && optional_string_from(wi, "workflow_id").as_deref()
                                == Some(workflow_id.as_str())
                            && optional_string_from(wi, "origin_node_id").as_deref()
                                == Some(request.node_id.as_str())
                    })
                    .filter_map(|wi| optional_string_from(wi, "work_item_id"))
                    .collect()
            })
            .unwrap_or_default();
        if !stale_ids.is_empty() {
            if let Some(items) = value.get_mut("work_items").and_then(Value::as_array_mut) {
                items.retain(|wi| {
                    optional_string_from(wi, "work_item_id")
                        .map(|id| !stale_ids.contains(&id))
                        .unwrap_or(true)
                });
            }
            if let Some(binds) = value
                .get_mut("workflow_node_session_bindings")
                .and_then(Value::as_array_mut)
            {
                binds.retain(|b| {
                    optional_string_from(b, "work_item_id")
                        .map(|id| !stale_ids.contains(&id))
                        .unwrap_or(true)
                });
            }
        }
        ensure_array_mut(&mut value, "work_items")?.push(json!({
          "work_item_id": work_item_id,
          "project_id": project_id(&request.project_root),
          "workflow_id": workflow_id,
          "title": summary,
          "state": "ready_to_dispatch",
          "source_kind": "canvas_run",
          "assigned_role_id": "codex-dev",
          "current_node_id": request.node_id,
          // 稳定的"出生节点"——current_node_id 会随 dispatch 漂移，积压清理按这个剔才准。
          "origin_node_id": request.node_id,
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": []
        }));
        backup_workflow_state_file(path, &timestamp)?;
        write_m5b_batch1_workflow_state(path, "canvas_run_work_item_created", &value)?;
    }

    // 绑定：若该 (node, work_item) 还没 active 绑定，用解析到的会话现绑（bind 已 workflow 感知）。
    let need_bind = {
        let current = read_workflow_state_value(path)?;
        workflow_node_session_binding_index(&current, &workflow_id, &request.node_id, Some(&work_item_id))
            .is_none()
    };
    if need_bind {
        let session = find_index_thread_or_sqlite(index, &thread_id)
            .ok_or_else(|| "节点会话不在当前索引内（含实时 sqlite），已拒绝运行".to_string())?;
        bind_workflow_node_codex_session_at(
            path,
            &WorkflowNodeSessionBindRequest {
                project_root: request.project_root.clone(),
                node_id: request.node_id.clone(),
                work_item_id: Some(work_item_id.clone()),
                thread_id: thread_id.clone(),
            },
            &session,
        )?;
    }

    // 沙箱：限定该项目根（codex 仍被 command_plan_for 关在项目目录；path-lock 已限测试项目）。
    let write_roots = if sandbox_mode == "workspace-write" {
        vec![request.project_root.clone()]
    } else {
        vec![]
    };

    // S1 执行层合一（option A）：真起 runner 前，B 派发过 A 的统一强闸 decide_real_execution_command。
    //   authorization_complete = path-lock 命中（铁律：authorized ⟹ 此 ⟹ path-lock；非测试项目即拦）；
    //   duplicate_blocked = 查在飞派发（B 净新）；
    //   guard_blocked = 过 A 的执行安全 guard，但只计「执行安全」reason、排除 A 的 3 道授权 reason（B 授权=path-lock）；
    //   diagnostics/stale_memory/user_rejected 取 false（见 evidence：B 无诊断摘要输入·不走任务记忆包·无逐次审批=S2）；
    //   readback_required = true（B 走 readback_db 回读）。
    // 沙箱 command_plan_for / 判决体 decide_real_execution_command / A 线路径 均一字未改。
    {
        // 站 3b：authorization_complete 认两种真授权——测试项目 path-lock，或主管授权的 3b 只读派发
        // （supervisor_authorization 携带已核 prepared dispatch；write_roots 由 read-only 沙箱推出=空）。
        let path_lock_hit = real_execution_authorization_complete(
            &request.project_root,
            &write_roots,
            supervisor_authorization.is_some(),
        );
        let gate_state = read_workflow_state_value(path)?;
        let duplicate_blocked = has_inflight_dispatch(&gate_state, &workflow_id, &request.node_id);
        let guard = codex_local_runner::inspect_codex_local_execution_guard(
            &build_canvas_node_codex_local_request(
                &request.project_root,
                &project_id(&request.project_root),
                &workflow_id,
                &request.node_id,
                &thread_id,
                &work_item_id,
                &objective,
                &sandbox_mode,
                &write_roots,
            ),
        );
        let guard_blocked = canvas_node_guard_blocked(&guard);
        let gate = real_execution_command::decide_real_execution_command(
            real_execution_command::RealExecutionCommandGateInput {
                command_name: "execute_project_workflow_node",
                command_family: "workflow_real_execution",
                operation_id: "resume",
                h5_unified_product_command: true,
                authorization_complete: path_lock_hit,
                user_rejected: false,
                duplicate_blocked,
                guard_blocked,
                diagnostics_blocked: false,
                stale_memory_blocked: false,
                readback_required: true,
            },
        );
        if !gate.runner_call_allowed {
            return Err(format!(
                "real_execution_gate_blocked:{}:{}（guard_reasons: {}）",
                gate.status,
                gate.reason,
                guard.reasons.join(",")
            ));
        }
    }

    let exec_request = WorkflowNodeDispatchExecuteRequest {
        project_root: request.project_root.clone(),
        node_id: request.node_id.clone(),
        work_item_id,
        prompt_kind: "user_reviewed_instruction".to_string(),
        user_reviewed_instruction: Some(UserReviewedInstructionInput {
            instruction_id: format!("instruction:project:{timestamp}"),
            summary,
            objective: objective.clone(),
            execution_cwd: request.project_root.clone(),
            sandbox_mode,
            allowed_write_roots: write_roots.clone(),
            allowed_reads: vec![request.project_root.clone()],
            allowed_writes: write_roots,
            forbidden_actions,
            timeout_seconds: 600,
            max_retries: 0,
            required_return,
            prompt_preview: Some(objective),
        }),
    };
    if let Some(prepared_authorization) = prepared_authorization.as_ref() {
        execute_workflow_node_dispatch_with_authorization_at(
            path,
            index,
            readback_db_path,
            runner,
            &exec_request,
            Some(prepared_authorization),
        )
    } else {
        execute_workflow_node_dispatch_for_index_at(
            path,
            index,
            readback_db_path,
            runner,
            &exec_request,
        )
    }
}

fn authorized_prepared_dispatch_for_execution(
    path: &Path,
    project_root: &str,
    workflow_id: &str,
    node_id: &str,
    work_item_id: &str,
    authorization_id: &str,
    allowed_write: &[String],
) -> Result<PreparedDispatchAuthorization, String> {
    let value = read_workflow_state_value(path)?;
    let work_item = find_work_item(&value, workflow_id, work_item_id)
        .ok_or_else(|| "主管请求的 work item 不存在，已拒绝启动 worker".to_string())?;
    let artifact = find_task_package_artifact(&value, work_item_id, work_item)
        .ok_or_else(|| "主管请求的 work item 缺任务包，已拒绝启动 worker".to_string())?;
    let package_write_roots = string_array(artifact, "allowed_write");
    let requested_write_roots = allowed_write.iter().cloned().collect::<BTreeSet<_>>();
    let package_write_roots_set = package_write_roots.iter().cloned().collect::<BTreeSet<_>>();
    if requested_write_roots != package_write_roots_set
        || requested_write_roots.len() != allowed_write.len()
        || package_write_roots_set.len() != package_write_roots.len()
    {
        return Err(
            "主管请求 allowed_write 与已批准任务包不一致，已拒绝启动 worker".to_string(),
        );
    }

    let expected_project_id = project_id(project_root);
    let dispatch = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .and_then(|dispatches| {
            dispatches.iter().rev().find(|dispatch| {
                optional_string_from(dispatch, "state").as_deref() == Some("prepared")
                    && optional_string_from(dispatch, "prompt_kind").as_deref()
                        == Some("authorized_prepared_auto_dispatch")
                    && optional_string_from(dispatch, "project_id").as_deref()
                        == Some(expected_project_id.as_str())
                    && optional_string_from(dispatch, "workflow_id").as_deref()
                        == Some(workflow_id)
                    && optional_string_from(dispatch, "node_id").as_deref() == Some(node_id)
                    && optional_string_from(dispatch, "work_item_id").as_deref()
                        == Some(work_item_id)
                    && optional_string_from(dispatch, "plan_authorization_id").as_deref()
                        == Some(authorization_id)
            })
        })
        .ok_or_else(|| {
            "找不到与主管请求完全匹配的已授权 prepared dispatch，已拒绝启动 worker"
                .to_string()
        })?;
    let authorization_check: AutoDispatchGuardResult = serde_json::from_value(
        dispatch
            .get("authorization_check")
            .cloned()
            .ok_or_else(|| "已授权 prepared dispatch 缺授权检查，已拒绝启动 worker".to_string())?,
    )
    .map_err(|error| format!("已授权 prepared dispatch 的授权检查损坏，已拒绝启动 worker：{error}"))?;
    if authorization_check.status != "authorized"
        || authorization_check.authorization_id.as_deref() != Some(authorization_id)
        || authorization_check.required_user_confirmation
        || authorization_check.required_global_review
    {
        return Err("prepared dispatch 的授权检查未完整放行，已拒绝启动 worker".to_string());
    }
    Ok(PreparedDispatchAuthorization {
        authorization_id: authorization_id.to_string(),
        authorization_check,
    })
}

// ===== S1 执行层合一：B 画布派发过 A 强闸的辅助件 =====

// S1：查某 (workflow, node) 是否已有「在飞」（state=="running"）派发——给 duplicate_blocked。
// 只数 "running"，不数 "prepared"：execute_workflow_node_dispatch_at 每次派发都先 write_prepared_dispatch
// 留一条 orphan "prepared" 记录（永不推进，真正执行是另一条 started→completed），所以 "prepared" 每次残留、
// 不是可靠在飞信号——数它会误拦同节点的合法重跑。"running" 是真正执行中的窗口（同一次调用里推进到
// completed/failed，不残留），才是「当前正在跑」的准信号。
fn has_inflight_dispatch(value: &Value, workflow_id: &str, node_id: &str) -> bool {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .map(|dispatches| {
            dispatches.iter().any(|d| {
                optional_string_from(d, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(d, "node_id").as_deref() == Some(node_id)
                    && optional_string_from(d, "state").as_deref() == Some("running")
            })
        })
        .unwrap_or(false)
}

// S1（option A）：B 的授权走 path-lock，不是 A 的「确认/范围/审计」那套。故 A 强 guard 的这 3 道**授权**
// reason 不计入 B 的 guard_blocked；其余**执行安全** reason（adapter/operation/路径/密钥/prompt 边界/
// readback/duplicate/command_plan…）照计。= 只加严（B 拿到执行安全检查）、不伪造授权产物。
const CANVAS_NODE_GUARD_AUTHORIZATION_REASONS: [&str; 3] = [
    "user_confirmation_required",
    "authorization_scope_missing",
    "audit_ref_missing",
];

fn canvas_node_guard_blocked(guard: &CodexLocalExecutionGuard) -> bool {
    guard
        .reasons
        .iter()
        .any(|reason| !CANVAS_NODE_GUARD_AUTHORIZATION_REASONS.contains(&reason.as_str()))
}

// S1：从画布节点派发上下文构造 CodexLocalExecutionRequest，仅供过 A 的**执行安全** guard。
// 安全字段据实填（路径锁项目根、prompt 真算 sha256、readback 计划齐）；A 的授权产物（确认/范围/审计）
// B 没有、**不伪造**，留空——它们触发的 3 道授权 reason 在 canvas_node_guard_blocked 里被排除（option A）。
#[allow(clippy::too_many_arguments)]
fn build_canvas_node_codex_local_request(
    project_root: &str,
    project_id_value: &str,
    workflow_id: &str,
    node_id: &str,
    thread_id: &str,
    work_item_id: &str,
    objective: &str,
    sandbox_mode: &str,
    write_roots: &[String],
) -> CodexLocalExecutionRequest {
    CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: "codex-local".to_string(),
        operation_id: "resume".to_string(),
        project_id: project_id_value.to_string(),
        project_root: project_root.to_string(),
        workflow_id: workflow_id.to_string(),
        node_id: node_id.to_string(),
        session_id: Some(thread_id.to_string()),
        work_item_id: Some(work_item_id.to_string()),
        continuation_id: None,
        target_cwd: project_root.to_string(),
        allowed_write_roots: write_roots.to_vec(),
        sandbox: sandbox_mode.to_string(),
        prompt_source_kind: "user_reviewed_instruction".to_string(),
        prompt_summary: objective.chars().take(160).collect(),
        prompt_sha256: crate::utils::hash::sha256_hex(objective),
        prompt_ref: format!("prompt-ref:canvas-node:{}", stable_id(node_id)),
        readback_plan: CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![
                "workbench_managed_last_message".to_string(),
                "workflow_node_dispatch".to_string(),
            ],
            unavailable_behavior: "readback_unavailable_or_failed_keeps_result_count_null"
                .to_string(),
            trust_policy: "must_be_explicit_readback_result_not_raw_transcript".to_string(),
            warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
        },
        requested_by: "workflow_node_canvas_dispatch".to_string(),
        // option A：授权走 path-lock（在 authorization_complete 算）；A 的授权产物 B 没有、不伪造，留空。
        user_confirmation_state: "path_lock_only".to_string(),
        authorization_scope_id: None,
        runtime_log_refs: vec![],
        audit_refs: vec![],
        active_attempts: vec![],
        warnings: vec!["canvas_node_guard_safety_subset_only".to_string()],
    }
}

// P3 E · 多工作流底座（架构 §12）：列出某项目的所有工作流（看有哪些、选一个查看/编辑/新建）。
#[tauri::command]
fn list_project_workflows(
    project_root: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Value>, String> {
    if !state.workflow_state_path.exists() {
        return Ok(vec![]);
    }
    let value = read_workflow_state_value(&state.workflow_state_path)?;
    let pid = project_id(&project_root);
    let slug = stable_id(&project_root);
    let default_id = default_workflow_id(&project_root);
    let nodes = value.get("nodes").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for wf in value.get("workflows").and_then(Value::as_array).cloned().unwrap_or_default() {
        let Some(wid) = optional_string_from(&wf, "workflow_id") else {
            continue;
        };
        // 归属该项目：workflow 带 project_id 命中，或 workflow_id 含项目 slug（兼容老记录）。
        let belongs = optional_string_from(&wf, "project_id").as_deref() == Some(pid.as_str())
            || wid.contains(&slug);
        if !belongs {
            continue;
        }
        let node_count = nodes
            .iter()
            .filter(|n| optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str()))
            .count();
        out.push(json!({
          "workflow_id": wid,
          "title": optional_string_from(&wf, "title").unwrap_or_default(),
          "state": optional_string_from(&wf, "state").unwrap_or_default(),
          "node_count": node_count,
          "is_default": wid == default_id,
        }));
    }
    Ok(out)
}

// P3 E · 把项目画布草案写回 workflow-state（架构 §12）。workflow_id 空=新建一个工作流（不覆盖谁）、
// 非空=更新那一个（替换它的 nodes/edges）。提交闸：①仅固定测试项目（轻档）②运行性检查不 blocked
// （§12「通过」；新草案需含 director 节点）③记审计 + 备份 + 原子写。会话方案/运行中工作项不在此动。
#[tauri::command]
fn submit_project_workflow_draft(
    request: SubmitProjectWorkflowDraftRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    submit_project_workflow_draft_at(&state.workflow_state_path, &request)
}

fn submit_project_workflow_draft_at(
    path: &Path,
    request: &SubmitProjectWorkflowDraftRequest,
) -> Result<WorkflowStateMutationResult, String> {
    // E 测试项目·轻档：写回当前仅限固定测试项目；非测试项目的工作流定义仍不碰。
    if !workflow_engine_test_project_unsealed(&request.project_root) {
        return Err("提交工作流草案当前仅限固定测试项目（轻档）；非测试项目仍锁".to_string());
    }
    let title = request.title.trim();
    if title.is_empty() {
        return Err("工作流标题不能为空".to_string());
    }
    if request.nodes.is_empty() {
        return Err("草案没有节点；不写回空工作流".to_string());
    }
    if !path.exists() {
        return Err("工作流状态文件不存在；无法写回草案".to_string());
    }
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let is_new = request
        .workflow_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_none();
    let workflow_id = if is_new {
        format!("workflow:{}:{timestamp}", stable_id(&request.project_root))
    } else {
        request.workflow_id.clone().unwrap_or_default()
    };
    if !is_new && !workflow_exists(&value, &workflow_id) {
        return Err("要更新的工作流不存在；无法写回".to_string());
    }

    // 草案节点 → workflow-state 节点（结构字段供读模型/显示 + canvas_payload 原样存供往返）。
    let mut id_map: Vec<(String, String)> = Vec::new();
    let mut built_nodes: Vec<Value> = Vec::new();
    for (i, dn) in request.nodes.iter().enumerate() {
        let canvas_id = optional_string_from(dn, "id").unwrap_or_else(|| format!("n{i}"));
        let kind = optional_string_from(dn, "kind")
            .or_else(|| optional_string_from(dn, "role"))
            .filter(|k| !k.trim().is_empty())
            .unwrap_or_else(|| "custom".to_string());
        let node_title = optional_string_from(dn, "label")
            .filter(|l| !l.trim().is_empty())
            .unwrap_or_else(|| kind.clone());
        // 架构债·根治 B：node_id 后缀用节点稳定 id（引擎建节点即定、随 canvas_payload 跨编辑往返），
        // 不再用位置式 {i}-{kind} —— 删/重排节点后 node_id 不变 → 会话绑定不悬空/不静默重挂。
        // 前缀 {wf}:node: 保留，所有 :node: 解析（派发/run-check/bind）照常不断。
        let node_id = format!("{workflow_id}:node:{}", stable_id(&canvas_id));
        id_map.push((canvas_id, node_id.clone()));
        built_nodes.push(json!({
          "node_id": node_id,
          "workflow_id": workflow_id,
          "node_type": kind,
          "title": node_title,
          "state": "draft",
          "source_kind": "canvas_submitted",
          "source_ref": Value::Null,
          "agent_type": Value::Null,
          "adapter_id": Value::Null,
          "artifact_type": Value::Null,
          "permission_level": "user_confirmed_write",
          "position": dn.get("position").cloned().unwrap_or_else(|| json!({"x":120,"y":120})),
          "canvas_payload": dn.clone(),
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": []
        }));
    }
    let mut built_edges: Vec<Value> = Vec::new();
    for (i, de) in request.edges.iter().enumerate() {
        let from_node = optional_string_from(de, "from")
            .and_then(|c| id_map.iter().find(|(cid, _)| *cid == c).map(|(_, n)| n.clone()));
        let to_node = optional_string_from(de, "to")
            .and_then(|c| id_map.iter().find(|(cid, _)| *cid == c).map(|(_, n)| n.clone()));
        if let (Some(f), Some(t)) = (from_node, to_node) {
            built_edges.push(json!({
              "edge_id": format!("{workflow_id}:edge:{i}"),
              "workflow_id": workflow_id,
              "from_node_id": f,
              "to_node_id": t,
              "edge_type": "canvas_link",
              "state": "draft",
              "source_kind": "canvas_submitted",
              "permission_level": "user_confirmed_write",
              "created_at": timestamp,
              "updated_at": timestamp,
              "warnings": []
            }));
        }
    }

    // 候选状态：更新=先删该 workflow 旧 nodes/edges 再加；新建=加 workflow 记录。
    // 后置B：新节点集的 node_id（用于 prune 失效绑定）。在 built_nodes 被 move 前抓。
    let new_node_ids: Vec<String> = built_nodes
        .iter()
        .filter_map(|n| optional_string_from(n, "node_id"))
        .collect();
    {
        let nodes = ensure_array_mut(&mut value, "nodes")?;
        nodes.retain(|n| optional_string_from(n, "workflow_id").as_deref() != Some(workflow_id.as_str()));
        nodes.extend(built_nodes);
    }
    {
        let edges = ensure_array_mut(&mut value, "edges")?;
        edges.retain(|e| optional_string_from(e, "workflow_id").as_deref() != Some(workflow_id.as_str()));
        edges.extend(built_edges);
    }
    // 后置B：prune 本工作流里 node_id 已不在新节点集的会话绑定——防位置式 node_id 下「删/重排节点后
    // 旧绑定悬空」或「旧会话静默重挂到占了同位置 id 的新节点」。残留边：同位置+同种类节点 id 不变 →
    // 绑定仍跟随（彻底解需稳定 uuid，随 A 引擎统一一起做）。
    {
        let bindings = ensure_array_mut(&mut value, "workflow_node_session_bindings")?;
        bindings.retain(|b| {
            optional_string_from(b, "workflow_id").as_deref() != Some(workflow_id.as_str())
                || optional_string_from(b, "node_id")
                    .map(|nid| new_node_ids.contains(&nid))
                    .unwrap_or(false)
        });
    }
    if is_new {
        ensure_array_mut(&mut value, "workflows")?.push(json!({
          "workflow_id": workflow_id,
          "project_id": project_id(&request.project_root),
          "title": title,
          "state": "draft",
          "source_kind": "canvas_submitted",
          "permission_level": "user_confirmed_write",
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": []
        }));
    } else if let Some(wf) = array_mut(&mut value, "workflows")?
        .iter_mut()
        .find(|w| optional_string_from(w, "workflow_id").as_deref() == Some(workflow_id.as_str()))
    {
        wf["title"] = Value::String(title.to_string());
        wf["updated_at"] = Value::String(timestamp.clone());
    }

    // 运行性检查（§12「通过」= 不 blocked）。在候选状态上跑。
    let workflow_record = value
        .get("workflows")
        .and_then(Value::as_array)
        .and_then(|ws| {
            ws.iter()
                .find(|w| optional_string_from(w, "workflow_id").as_deref() == Some(workflow_id.as_str()))
        })
        .cloned()
        .unwrap_or_else(|| json!({}));
    let nodes_all = value.get("nodes").and_then(Value::as_array).cloned().unwrap_or_default();
    let work_items_all = value.get("work_items").and_then(Value::as_array).cloned().unwrap_or_default();
    // 提交是保存「定义」，不该被运行时自动建的临时 work_item 卡住：跑链/跑节点会给每次执行建一个
    // canvas_run 临时 work_item（无任务包 artifact），它们会累积、在运行性检查里全 blocked（缺模型/
    // 读写范围/验收/会话绑定）→ 跑过一次后就再也存不了草案。存草案只看定义结构（director 等），故剔除
    // canvas_run 临时件再做运行性检查；真任务包 work_item 仍照查（旧模型不受影响）。
    let work_items_for_check: Vec<Value> = work_items_all
        .iter()
        .filter(|wi| optional_string_from(wi, "source_kind").as_deref() != Some("canvas_run"))
        .cloned()
        .collect();
    let artifacts_all = value.get("artifacts").and_then(Value::as_array).cloned().unwrap_or_default();
    let bindings_all = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let run_check = inspect_workflow_run_check_from_value(
        &request.project_root,
        Some(&workflow_id),
        &workflow_record,
        &nodes_all,
        &work_items_for_check,
        &artifacts_all,
        &bindings_all,
    );
    if run_check.status == "blocked" {
        return Err(format!(
            "运行性检查未通过（blocked）：{}",
            run_check.blocked_reasons.join("；")
        ));
    }

    // 控制核心 / 审计：记审计事件；备份 + schema 校验后原子写。
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-submit",
        &workflow_id,
        &timestamp,
    );
    ensure_array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": if is_new { "project_workflow_created_from_canvas" } else { "project_workflow_updated_from_canvas" },
      "target_ref": workflow_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "created_at": timestamp,
      "reason": "用户在项目画布提交草案为项目工作流（经运行性检查通过 + 审计；测试项目·轻档）。"
    }));

    let backup = backup_workflow_state_file(path, &timestamp)?;
    write_m5b_batch1_workflow_state(path, "project_workflow_draft_submitted", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: format!(
            "{}项目工作流「{}」（运行性 {}）",
            if is_new { "已新建" } else { "已更新" },
            title,
            run_check.status
        ),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

// P3 E · 取某工作流的画布节点/边，供「编辑工作流」把现有 nodes 加载进草案（避免空白覆盖，§12）。
// 返回 { nodes: [画布节点], edges: [{id,from,to} 画布 id] }：canvas-submitted 节点用其 canvas_payload；
// 老 bootstrap 节点无 payload → 用结构字段合成一个，使默认治理工作流也能被编辑回填。
#[tauri::command]
fn get_project_workflow_nodes(
    project_root: String,
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Value, String> {
    let _ = project_root;
    if !state.workflow_state_path.exists() {
        return Ok(json!({ "nodes": [], "edges": [] }));
    }
    let value = read_workflow_state_value(&state.workflow_state_path)?;
    let mut node_to_canvas: Vec<(String, String)> = Vec::new();
    let mut nodes_out: Vec<Value> = Vec::new();
    for n in value.get("nodes").and_then(Value::as_array).cloned().unwrap_or_default() {
        if optional_string_from(&n, "workflow_id").as_deref() != Some(workflow_id.as_str()) {
            continue;
        }
        let node_id = optional_string_from(&n, "node_id").unwrap_or_default();
        let payload = n.get("canvas_payload").cloned().unwrap_or_else(|| {
            // 后置A 止血：老 bootstrap 节点无 canvas_payload → 合成时角色/种类按 node_type 派生
            // （别全写 subagent，对齐读模型真角色：director/审查/执行…）。CanvasNodeRole 只有
            // director|subagent；kind 开放（用于显示色/标签）。
            let node_type = optional_string_from(&n, "node_type").unwrap_or_else(|| "custom".to_string());
            let (role, kind) = match node_type.as_str() {
                "director" => ("director", "director"),
                "actor" => ("subagent", "subagent"),
                "validation" | "review" => ("subagent", "reviewer"),
                other => ("subagent", other),
            };
            json!({
              "id": node_id,
              "role": role,
              "kind": kind,
              "label": optional_string_from(&n, "title").unwrap_or_default(),
              "position": n.get("position").cloned().unwrap_or_else(|| json!({"x":120,"y":120})),
              "warnings": []
            })
        });
        let canvas_id = optional_string_from(&payload, "id").unwrap_or_else(|| node_id.clone());
        node_to_canvas.push((node_id, canvas_id));
        nodes_out.push(payload);
    }
    let mut edges_out: Vec<Value> = Vec::new();
    for (i, e) in value
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        if optional_string_from(e, "workflow_id").as_deref() != Some(workflow_id.as_str()) {
            continue;
        }
        let from_c = optional_string_from(e, "from_node_id")
            .and_then(|nid| node_to_canvas.iter().find(|(n, _)| *n == nid).map(|(_, c)| c.clone()));
        let to_c = optional_string_from(e, "to_node_id")
            .and_then(|nid| node_to_canvas.iter().find(|(n, _)| *n == nid).map(|(_, c)| c.clone()));
        if let (Some(f), Some(t)) = (from_c, to_c) {
            edges_out.push(json!({ "id": format!("seed-edge-{i}"), "from": f, "to": t }));
        }
    }
    Ok(json!({ "nodes": nodes_out, "edges": edges_out }))
}

#[tauri::command]
fn read_workflow_node_dispatch_result(
    _request: WorkflowNodeDispatchReadbackRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let _ = state;
    Err(legacy_product_command_blocked_message(
        "read_workflow_node_dispatch_result",
    ))
}

#[tauri::command]
fn record_workflow_dispatch_director_review(
    request: WorkflowDispatchDirectorReviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_workflow_dispatch_director_review_for_index_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn record_workflow_dispatch_director_review_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowDispatchDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录总指导回收意见".to_string());
    }
    record_workflow_dispatch_director_review_at(path, request)
}

#[tauri::command]
fn record_workflow_permission_decision(
    request: WorkflowPermissionDecisionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_workflow_permission_decision_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_workflow_permission_decision_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowPermissionDecisionRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录权限结论".to_string());
    }
    record_workflow_permission_decision_at(path, request)
}

#[tauri::command]
fn prepare_offline_role_dispatch(
    request: OfflineRoleDispatchRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_offline_role_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

fn prepare_offline_role_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineRoleDispatchRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线角色派发".to_string());
    }
    prepare_offline_role_dispatch_at(path, request)
}

#[tauri::command]
fn record_offline_role_result_handoff(
    request: OfflineRoleResultHandoffRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    record_offline_role_result_handoff_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_offline_role_result_handoff_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineRoleResultHandoffRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线角色回传".to_string());
    }
    record_offline_role_result_handoff_at(path, request)
}

#[tauri::command]
fn record_offline_director_review(
    request: OfflineDirectorReviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_offline_director_review_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_offline_director_review_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线总指导回收".to_string());
    }
    record_offline_director_review_at(path, request)
}

#[tauri::command]
fn run_workflow_machine(
    _request: WorkflowMachineRunRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowMachineRunResult, String> {
    let _ = state;
    Err(legacy_product_command_blocked_message(
        "run_workflow_machine",
    ))
}

fn run_workflow_machine_for_index_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowMachineRunRequest,
) -> Result<WorkflowMachineRunResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝运行工作流机器".to_string());
    }
    run_workflow_machine_at(path, index, readback_db_path, runner, request)
}

#[tauri::command]
fn copy_indexed_path(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let allowed = allowed_paths(&index);
    if !allowed.can_copy(&request.path) {
        return Err("路径不在索引白名单内，已拒绝复制".to_string());
    }
    copy_to_clipboard(&request.path)?;
    Ok(format!("已复制索引内路径：{}", request.path))
}

#[tauri::command]
fn open_indexed_project(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let allowed = allowed_paths(&index);
    if !allowed.projects.contains(&request.path) {
        return Err("路径不是索引内项目根目录，已拒绝打开".to_string());
    }
    let path = PathBuf::from(&request.path);
    if !path.is_dir() {
        return Err("索引项目路径当前不是可打开目录".to_string());
    }
    run_open(&[request.path.as_str()])?;
    Ok(format!("已请求打开项目目录：{}", request.path))
}

#[tauri::command]
fn reveal_indexed_rollout(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let mut allowed = allowed_paths(&index);
    extend_allowed_rollouts_from_sqlite(&mut allowed);
    if !allowed.rollouts.contains(&request.path) {
        return Err(
            "rollout_outside_allowed_dirs:路径不是允许的 rollout 文件，已拒绝定位".to_string(),
        );
    }
    let path = PathBuf::from(&request.path);
    if !path.is_file() {
        return Err("rollout_missing:允许的 rollout 路径当前不是文件".to_string());
    }
    run_open(&["-R", request.path.as_str()])?;
    Ok(format!("已请求定位 rollout 文件：{}", request.path))
}
