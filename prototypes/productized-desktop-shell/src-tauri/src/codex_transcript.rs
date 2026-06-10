use crate::{
    CodexTranscript, CodexTranscriptEvent, CodexTranscriptSummary, CodexTranscriptViewerBoundary,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct TranscriptThreadMetadata {
    pub thread_id: String,
    pub rollout_path: Option<String>,
    pub project_root: Option<String>,
    pub title: Option<String>,
    pub created_at_ms: Option<i64>,
    pub updated_at_ms: Option<i64>,
    pub catalog_source: String,
    pub index_thread_count: Option<usize>,
}

#[derive(Default)]
struct JsonlStats {
    line_count: usize,
    parsed_line_count: usize,
    bad_json_line_count: usize,
}

pub(crate) fn read_transcript_from_rollout(
    metadata: TranscriptThreadMetadata,
    codex_home: &Path,
) -> Result<CodexTranscript, String> {
    let rollout_path = validated_rollout_path(&metadata, codex_home)?;
    let (events, mut warnings, jsonl_stats) = read_jsonl_events(&rollout_path)?;
    let event_type_counts = count_events_by_type(&events);
    let raw_type_counts = count_metadata_value(&events, "raw_type");
    let payload_type_counts = count_metadata_value(&events, "payload_type");
    let event_warning_count = events
        .iter()
        .map(|event| event.warnings.len())
        .sum::<usize>();
    let encrypted_count = events
        .iter()
        .filter(|event| {
            event
                .warnings
                .iter()
                .any(|warning| warning == "encrypted_content_omitted")
        })
        .count();
    let sensitive_count = events
        .iter()
        .filter(|event| {
            event
                .warnings
                .iter()
                .any(|warning| warning == "sensitive_like_content")
        })
        .count();
    let unknown_count = *event_type_counts.get("unknown").unwrap_or(&0);

    if unknown_count > 0 {
        warnings.push(format!("unknown_event_count:{unknown_count}"));
    }
    if encrypted_count > 0 {
        warnings.push(format!("encrypted_content_event_count:{encrypted_count}"));
    }
    if sensitive_count > 0 {
        warnings.push(format!("sensitive_like_event_count:{sensitive_count}"));
    }

    Ok(CodexTranscript {
        thread_id: metadata.thread_id,
        rollout_path: rollout_path.display().to_string(),
        project_path: metadata.project_root,
        title: metadata.title,
        created_at_ms: metadata.created_at_ms,
        updated_at_ms: metadata.updated_at_ms,
        viewer_boundary: transcript_viewer_boundary(),
        events,
        summary: CodexTranscriptSummary {
            total_events: event_type_counts.values().sum(),
            event_type_counts,
            unknown_event_count: unknown_count,
            warning_count: warnings.len() + event_warning_count,
            encrypted_content_event_count: encrypted_count,
            sensitive_like_event_count: sensitive_count,
        },
        warnings,
        source_stats: json!({
            "catalog_source": metadata.catalog_source,
            "index_thread_count": metadata.index_thread_count,
            "jsonl": {
                "line_count": jsonl_stats.line_count,
                "parsed_line_count": jsonl_stats.parsed_line_count,
                "bad_json_line_count": jsonl_stats.bad_json_line_count,
            },
            "raw_type_counts": raw_type_counts,
            "payload_type_counts": payload_type_counts,
        }),
    })
}

pub(crate) fn transcript_viewer_boundary() -> CodexTranscriptViewerBoundary {
    CodexTranscriptViewerBoundary {
        view_kind: "session_history_viewer".to_string(),
        reads_session_history: true,
        is_execution_readback: false,
        real_execution_readback_performed: false,
        execution_readback_scope:
            "not_h_h5_execution_readback_use_runtime_log_attempt_readback_refs".to_string(),
        warnings: vec![
            "session_transcript_viewer_is_not_execution_readback".to_string(),
            "history_view_does_not_authorize_prompt_send_or_resume".to_string(),
        ],
    }
}

pub(crate) fn is_allowed_rollout_path(path: &Path, codex_home: &Path) -> bool {
    let path_exists = path.exists();
    let compare_path = comparable_path(path, path_exists);
    [
        codex_home.join("sessions"),
        codex_home.join("archived_sessions"),
    ]
    .iter()
    .map(|allowed| comparable_path(allowed, path_exists))
    .any(|allowed| compare_path.starts_with(allowed))
}

fn validated_rollout_path(
    metadata: &TranscriptThreadMetadata,
    codex_home: &Path,
) -> Result<PathBuf, String> {
    let raw_path = metadata
        .rollout_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "rollout_missing:missing_rollout_path:{}",
                metadata.thread_id
            )
        })?;
    let path = PathBuf::from(raw_path);
    if !is_allowed_rollout_path(&path, codex_home) {
        return Err(format!("rollout_outside_allowed_dirs:{}", path.display()));
    }
    if !path.exists() {
        return Err(format!("rollout_missing:{}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("rollout_missing:not_a_file:{}", path.display()));
    }
    Ok(path)
}

fn read_jsonl_events(
    path: &Path,
) -> Result<(Vec<CodexTranscriptEvent>, Vec<String>, JsonlStats), String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("filesystem_read_failed:{}:{error}", path.display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    let mut warnings = Vec::new();
    let mut stats = JsonlStats::default();

    for (index, line) in reader.lines().enumerate() {
        let line_number = index + 1;
        stats.line_count += 1;
        let line =
            line.map_err(|error| format!("filesystem_read_failed:{}:{error}", path.display()))?;
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(text) {
            Ok(item) => {
                stats.parsed_line_count += 1;
                events.push(parse_event(&item, line_number));
            }
            Err(_) => {
                stats.bad_json_line_count += 1;
                warnings.push(format!("invalid_json_line:{line_number}"));
            }
        }
    }

    Ok((events, warnings, stats))
}

fn parse_event(item: &Value, line_number: usize) -> CodexTranscriptEvent {
    let Some(object) = item.as_object() else {
        let mut event = empty_event(line_number, None);
        set_unknown_event(&mut event, item, None);
        add_warning(&mut event.warnings, "event_not_object");
        return event;
    };

    let mut event = empty_event(line_number, object.get("timestamp").and_then(Value::as_str));
    let raw_type = object.get("type").cloned().unwrap_or(Value::Null);
    let payload = object.get("payload");
    base_metadata(&mut event, &raw_type, payload);

    let Some(payload_object) = payload.and_then(Value::as_object) else {
        if matches!(
            raw_type.as_str(),
            Some("turn_context" | "session_meta" | "compacted")
        ) {
            event.event_type = raw_type.as_str().map(str::to_string);
            event.actor = Some("system".to_string());
            add_warning(&mut event.warnings, "payload_not_object");
            return event;
        }
        set_unknown_event(&mut event, item, payload);
        return event;
    };

    let payload_value = Value::Object(payload_object.clone());
    let payload_type = payload_object.get("type").and_then(Value::as_str);
    event.turn_id = payload_object
        .get("turn_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    event.call_id = payload_object
        .get("call_id")
        .and_then(Value::as_str)
        .map(str::to_string);

    match raw_type.as_str() {
        Some("turn_context") => {
            event.event_type = Some("turn_context".to_string());
            event.actor = Some("system".to_string());
            set_metadata_payload(&mut event, &payload_value);
        }
        Some("session_meta") => {
            event.event_type = Some("session_meta".to_string());
            event.actor = Some("system".to_string());
            set_metadata_payload(&mut event, &payload_value);
        }
        Some("compacted") => {
            event.event_type = Some("compacted".to_string());
            event.actor = Some("system".to_string());
            event.text = payload_text(payload_object);
            set_metadata_payload(&mut event, &payload_value);
        }
        Some("event_msg") => parse_event_msg(&mut event, item, payload_object, payload_type),
        Some("response_item") => {
            parse_response_item(&mut event, item, payload_object, payload_type)
        }
        _ => set_unknown_event(&mut event, item, payload),
    }

    if event_has_sensitive_like_content(&event) {
        add_warning(&mut event.warnings, "sensitive_like_content");
    }

    event
}

fn parse_event_msg(
    event: &mut CodexTranscriptEvent,
    item: &Value,
    payload: &Map<String, Value>,
    payload_type: Option<&str>,
) {
    match payload_type {
        Some("user_message") => {
            event.event_type = Some("user_message".to_string());
            event.actor = Some("user".to_string());
            event.role = Some("user".to_string());
            event.text = payload_text(payload);
        }
        Some("agent_message") => {
            event.event_type = Some("assistant_message".to_string());
            event.actor = Some("assistant".to_string());
            event.role = Some("assistant".to_string());
            event.text = payload_text(payload);
        }
        Some("patch_apply_end") => {
            event.event_type = Some("command_output".to_string());
            event.actor = Some("tool".to_string());
            event.tool_name = Some("apply_patch".to_string());
            apply_command_fields(event, payload, payload.get("output"));
            set_metadata_value(
                event,
                "status",
                payload.get("status").cloned().unwrap_or(Value::Null),
            );
            set_metadata_value(
                event,
                "success",
                payload.get("success").cloned().unwrap_or(Value::Null),
            );
        }
        Some("task_started" | "task_complete" | "token_count") => {
            event.event_type = Some("system_context".to_string());
            event.actor = Some("system".to_string());
            set_metadata_payload(event, &Value::Object(payload.clone()));
        }
        _ => set_unknown_event(event, item, Some(&Value::Object(payload.clone()))),
    }
}

fn parse_response_item(
    event: &mut CodexTranscriptEvent,
    item: &Value,
    payload: &Map<String, Value>,
    payload_type: Option<&str>,
) {
    match payload_type {
        Some("message") => {
            let role = payload.get("role").and_then(Value::as_str);
            event.role = role.map(str::to_string);
            event.actor = Some(if matches!(role, Some("user" | "assistant" | "system")) {
                role.unwrap_or("assistant").to_string()
            } else {
                "assistant".to_string()
            });
            event.event_type = Some(if role == Some("user") {
                "user_message".to_string()
            } else {
                "assistant_message".to_string()
            });
            event.text = payload_text(payload);
            if let Some(phase) = payload.get("phase") {
                set_metadata_value(event, "phase", phase.clone());
            }
        }
        Some("function_call" | "custom_tool_call") => {
            event.event_type = Some("tool_call".to_string());
            event.actor = Some("assistant".to_string());
            event.tool_name = payload
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string);
            event.arguments =
                parse_jsonish(payload.get("arguments").or_else(|| payload.get("input")));
            if let Some(status) = payload.get("status") {
                set_metadata_value(event, "status", status.clone());
            }
        }
        Some("function_call_output" | "custom_tool_call_output") => {
            let output = payload.get("output");
            if looks_like_command_result(payload, output) {
                event.event_type = Some("command_output".to_string());
                event.actor = Some("tool".to_string());
                apply_command_fields(event, payload, output);
            } else {
                event.event_type = Some("tool_result".to_string());
                event.actor = Some("tool".to_string());
                event.output = parse_jsonish(output);
            }
        }
        Some("reasoning") => {
            event.event_type = Some("system_context".to_string());
            event.actor = Some("assistant".to_string());
            event.text = payload_text(payload);
            set_metadata_payload(event, &Value::Object(payload.clone()));
        }
        _ => set_unknown_event(event, item, Some(&Value::Object(payload.clone()))),
    }
}

fn empty_event(line_number: usize, timestamp: Option<&str>) -> CodexTranscriptEvent {
    CodexTranscriptEvent {
        event_id: format!("line-{line_number:06}"),
        timestamp: timestamp.map(str::to_string),
        event_type: None,
        actor: None,
        role: None,
        turn_id: None,
        call_id: None,
        tool_name: None,
        text: None,
        arguments: Value::Null,
        output: Value::Null,
        stdout: None,
        stderr: None,
        exit_code: Value::Null,
        metadata: json!({ "line_number": line_number }),
        warnings: Vec::new(),
    }
}

fn base_metadata(event: &mut CodexTranscriptEvent, raw_type: &Value, payload: Option<&Value>) {
    set_metadata_value(event, "raw_type", raw_type.clone());
    if let Some(payload_object) = payload.and_then(Value::as_object) {
        set_metadata_value(
            event,
            "payload_type",
            payload_object.get("type").cloned().unwrap_or(Value::Null),
        );
        let mut keys = payload_object.keys().map(String::from).collect::<Vec<_>>();
        keys.sort();
        set_metadata_value(event, "payload_keys", json!(keys));
    } else {
        set_metadata_value(event, "payload_type", Value::Null);
        let value_type = payload.map(value_type_name).unwrap_or("null");
        set_metadata_value(event, "payload_value_type", json!(value_type));
    }
}

fn set_unknown_event(event: &mut CodexTranscriptEvent, item: &Value, payload: Option<&Value>) {
    event.event_type = Some("unknown".to_string());
    event.actor = Some("system".to_string());
    add_warning(&mut event.warnings, "unknown_event_type");
    let raw_event = strip_encrypted_content(item, &mut event.warnings);
    set_metadata_value(event, "raw_event", raw_event);
    if payload.is_some_and(|payload| !payload.is_object()) {
        add_warning(&mut event.warnings, "payload_not_object");
    }
}

fn set_metadata_payload(event: &mut CodexTranscriptEvent, payload: &Value) {
    let cleaned = strip_encrypted_content(payload, &mut event.warnings);
    set_metadata_value(event, "payload", cleaned);
}

fn set_metadata_value(event: &mut CodexTranscriptEvent, key: &str, value: Value) {
    if !event.metadata.is_object() {
        event.metadata = Value::Object(Map::new());
    }
    if let Some(metadata) = event.metadata.as_object_mut() {
        metadata.insert(key.to_string(), value);
    }
}

fn payload_text(payload: &Map<String, Value>) -> Option<String> {
    ["message", "text", "content", "summary"]
        .iter()
        .filter_map(|key| payload.get(*key))
        .filter_map(text_from_value)
        .next()
}

fn text_from_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let parts = items.iter().filter_map(text_from_value).collect::<Vec<_>>();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }
        Value::Object(object) => ["text", "message", "content", "summary"]
            .iter()
            .filter_map(|key| object.get(*key))
            .filter_map(text_from_value)
            .next(),
        _ => None,
    }
}

fn parse_jsonish(value: Option<&Value>) -> Value {
    let Some(value) = value else {
        return Value::Null;
    };
    let Some(text) = value.as_str() else {
        return value.clone();
    };
    let trimmed = text.trim();
    if trimmed.is_empty() || !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return value.clone();
    }
    serde_json::from_str::<Value>(trimmed).unwrap_or_else(|_| value.clone())
}

fn apply_command_fields(
    event: &mut CodexTranscriptEvent,
    payload: &Map<String, Value>,
    output: Option<&Value>,
) {
    let parsed_output = parse_jsonish(output.or_else(|| payload.get("output")));
    event.stdout = payload
        .get("stdout")
        .and_then(Value::as_str)
        .map(str::to_string);
    event.stderr = payload
        .get("stderr")
        .and_then(Value::as_str)
        .map(str::to_string);
    event.exit_code = payload.get("exit_code").cloned().unwrap_or(Value::Null);
    event.output = parsed_output.clone();
    if let Some(parsed) = parsed_output.as_object() {
        if event.stdout.is_none() {
            event.stdout = parsed
                .get("stdout")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        if event.stderr.is_none() {
            event.stderr = parsed
                .get("stderr")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        if event.exit_code.is_null() {
            event.exit_code = parsed.get("exit_code").cloned().unwrap_or(Value::Null);
        }
    }
}

fn looks_like_command_result(payload: &Map<String, Value>, output: Option<&Value>) -> bool {
    if ["stdout", "stderr", "exit_code"]
        .iter()
        .any(|key| payload.contains_key(*key))
    {
        return true;
    }
    parse_jsonish(output).as_object().is_some_and(|parsed| {
        ["stdout", "stderr", "exit_code"]
            .iter()
            .any(|key| parsed.contains_key(*key))
    })
}

fn strip_encrypted_content(value: &Value, warnings: &mut Vec<String>) -> Value {
    match value {
        Value::Object(object) => {
            let mut cleaned = Map::new();
            for (key, item) in object {
                if key == "encrypted_content" {
                    add_warning(warnings, "encrypted_content_omitted");
                    cleaned.insert(
                        key.clone(),
                        json!({ "present": !item.is_null(), "omitted": true }),
                    );
                } else {
                    cleaned.insert(key.clone(), strip_encrypted_content(item, warnings));
                }
            }
            Value::Object(cleaned)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| strip_encrypted_content(item, warnings))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn event_has_sensitive_like_content(event: &CodexTranscriptEvent) -> bool {
    event
        .text
        .as_ref()
        .is_some_and(|value| has_sensitive_like_text(value))
        || event
            .stdout
            .as_ref()
            .is_some_and(|value| has_sensitive_like_text(value))
        || event
            .stderr
            .as_ref()
            .is_some_and(|value| has_sensitive_like_text(value))
        || has_sensitive_like_value(&event.arguments)
        || has_sensitive_like_value(&event.output)
        || has_sensitive_like_value(&event.metadata)
}

fn has_sensitive_like_value(value: &Value) -> bool {
    match value {
        Value::String(text) => has_sensitive_like_text(text),
        Value::Array(items) => items.iter().any(has_sensitive_like_value),
        Value::Object(object) => object.values().any(has_sensitive_like_value),
        _ => false,
    }
}

fn has_sensitive_like_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    (lower.contains("authorization") && lower.contains("bearer "))
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("api-key")
        || lower.contains("secret=")
        || lower.contains("secret:")
        || lower.contains("token=")
        || lower.contains("token:")
        || text.contains("sk-")
        || text.contains("ghp_")
        || text.contains("AKIA")
        || lower.contains("xoxb-")
        || lower.contains("xoxa-")
        || lower.contains("xoxp-")
        || lower.contains("xoxr-")
        || lower.contains("xoxs-")
}

fn add_warning(warnings: &mut Vec<String>, warning: &str) {
    if !warnings.iter().any(|current| current == warning) {
        warnings.push(warning.to_string());
    }
}

fn count_events_by_type(events: &[CodexTranscriptEvent]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for event in events {
        let label = event.event_type.as_deref().unwrap_or("null").to_string();
        *counts.entry(label).or_insert(0) += 1;
    }
    counts
}

fn count_metadata_value(events: &[CodexTranscriptEvent], key: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for event in events {
        let label = event
            .metadata
            .get(key)
            .map(value_label)
            .unwrap_or_else(|| "null".to_string());
        *counts.entry(label).or_insert(0) += 1;
    }
    counts
}

fn value_label(value: &Value) -> String {
    value.as_str().map(str::to_string).unwrap_or_else(|| {
        if value.is_null() {
            "null".to_string()
        } else {
            value.to_string()
        }
    })
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "NoneType",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "str",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn canonical_or_normalized(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| normalized_path(path))
}

fn comparable_path(path: &Path, canonicalize_existing: bool) -> PathBuf {
    if canonicalize_existing {
        canonical_or_normalized(path)
    } else {
        normalized_path(path)
    }
}

fn normalized_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => out.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(part) => out.push(part),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_user_assistant_tool_call_and_command_output() {
        let fixture = fixture("codex-transcript-basic");
        let rollout_path = fixture.sessions_dir.join("thread-ok.jsonl");
        write_jsonl(
            &rollout_path,
            &[
                json!({"timestamp":"2026-05-29T00:00:00Z","type":"session_meta","payload":{"id":"thread-ok","cwd":"/tmp/project","model_provider":"ai"}}),
                json!({"timestamp":"2026-05-29T00:00:01Z","type":"turn_context","payload":{"turn_id":"turn-1","cwd":"/tmp/project","model":"fixture-model"}}),
                json!({"timestamp":"2026-05-29T00:00:02Z","type":"event_msg","payload":{"type":"user_message","message":"User asks for a fixture."}}),
                json!({"timestamp":"2026-05-29T00:00:03Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Assistant answers fixture."}]}}),
                json!({"timestamp":"2026-05-29T00:00:04Z","type":"response_item","payload":{"type":"function_call","call_id":"call-1","name":"functions.exec_command","arguments":"{\"cmd\":\"pwd\"}"}}),
                json!({"timestamp":"2026-05-29T00:00:05Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call-1","output":"{\"stdout\":\"/tmp/project\\n\",\"stderr\":\"\",\"exit_code\":0}"}}),
                json!({"timestamp":"2026-05-29T00:00:06Z","type":"compacted","payload":{"summary":"Compacted fixture summary."}}),
            ],
        );

        let transcript = read_transcript_from_rollout(metadata(&rollout_path), &fixture.codex_home)
            .expect("transcript");

        assert_eq!(transcript.thread_id, "thread-ok");
        assert_eq!(
            transcript.viewer_boundary.view_kind,
            "session_history_viewer"
        );
        assert!(transcript.viewer_boundary.reads_session_history);
        assert!(!transcript.viewer_boundary.is_execution_readback);
        assert!(!transcript.viewer_boundary.real_execution_readback_performed);
        assert!(transcript
            .viewer_boundary
            .warnings
            .contains(&"session_transcript_viewer_is_not_execution_readback".to_string()));
        assert_eq!(
            transcript.summary.event_type_counts.get("session_meta"),
            Some(&1)
        );
        assert_eq!(
            transcript.summary.event_type_counts.get("turn_context"),
            Some(&1)
        );
        assert_eq!(
            transcript.summary.event_type_counts.get("user_message"),
            Some(&1)
        );
        assert_eq!(
            transcript
                .summary
                .event_type_counts
                .get("assistant_message"),
            Some(&1)
        );
        assert_eq!(
            transcript.summary.event_type_counts.get("tool_call"),
            Some(&1)
        );
        assert_eq!(
            transcript.summary.event_type_counts.get("command_output"),
            Some(&1)
        );
        let tool_call = transcript
            .events
            .iter()
            .find(|event| event.event_type.as_deref() == Some("tool_call"))
            .expect("tool call");
        assert_eq!(
            tool_call.tool_name.as_deref(),
            Some("functions.exec_command")
        );
        assert_eq!(tool_call.arguments, json!({"cmd": "pwd"}));
        let command_output = transcript
            .events
            .iter()
            .find(|event| event.event_type.as_deref() == Some("command_output"))
            .expect("command output");
        assert_eq!(command_output.stdout.as_deref(), Some("/tmp/project\n"));
        assert_eq!(command_output.stderr.as_deref(), Some(""));
        assert_eq!(command_output.exit_code, json!(0));
    }

    #[test]
    fn bad_jsonl_line_records_warning_and_keeps_other_events() {
        let fixture = fixture("codex-transcript-bad-line");
        let rollout_path = fixture.sessions_dir.join("thread-ok.jsonl");
        fs::write(
            &rollout_path,
            format!(
                "{}\n{{bad json\n{}\n",
                json!({"type":"event_msg","payload":{"type":"user_message","message":"hello"}}),
                json!({"type":"event_msg","payload":{"type":"agent_message","message":"hi"}}),
            ),
        )
        .expect("write jsonl");

        let transcript = read_transcript_from_rollout(metadata(&rollout_path), &fixture.codex_home)
            .expect("transcript");

        assert_eq!(transcript.events.len(), 2);
        assert!(transcript
            .warnings
            .contains(&"invalid_json_line:2".to_string()));
        assert_eq!(
            transcript.source_stats["jsonl"]["bad_json_line_count"],
            json!(1)
        );
    }

    #[test]
    fn encrypted_content_is_marked_and_not_output() {
        let fixture = fixture("codex-transcript-encrypted");
        let rollout_path = fixture.sessions_dir.join("thread-ok.jsonl");
        write_jsonl(
            &rollout_path,
            &[
                json!({"type":"response_item","payload":{"type":"reasoning","summary":[],"encrypted_content":"ENCRYPTED_PAYLOAD_SHOULD_NOT_APPEAR"}}),
            ],
        );

        let transcript = read_transcript_from_rollout(metadata(&rollout_path), &fixture.codex_home)
            .expect("transcript");
        let serialized = serde_json::to_string(&transcript).expect("serialize transcript");

        assert!(!serialized.contains("ENCRYPTED_PAYLOAD_SHOULD_NOT_APPEAR"));
        assert!(transcript.events[0]
            .warnings
            .contains(&"encrypted_content_omitted".to_string()));
        assert_eq!(transcript.summary.encrypted_content_event_count, 1);
    }

    #[test]
    fn sensitive_like_content_gets_warning() {
        let fixture = fixture("codex-transcript-sensitive");
        let rollout_path = fixture.sessions_dir.join("thread-ok.jsonl");
        write_jsonl(
            &rollout_path,
            &[
                json!({"type":"event_msg","payload":{"type":"user_message","message":"Authorization: Bearer abcdefghijklmnopqrstuvwxyz123456"}}),
            ],
        );

        let transcript = read_transcript_from_rollout(metadata(&rollout_path), &fixture.codex_home)
            .expect("transcript");

        assert!(transcript.events[0]
            .warnings
            .contains(&"sensitive_like_content".to_string()));
        assert!(transcript
            .warnings
            .contains(&"sensitive_like_event_count:1".to_string()));
    }

    #[test]
    fn unknown_event_preserves_diagnostic_metadata() {
        let fixture = fixture("codex-transcript-unknown");
        let rollout_path = fixture.sessions_dir.join("thread-ok.jsonl");
        write_jsonl(
            &rollout_path,
            &[
                json!({"type":"new_future_event","payload":{"type":"future_payload","shape":{"value":1}}}),
            ],
        );

        let transcript = read_transcript_from_rollout(metadata(&rollout_path), &fixture.codex_home)
            .expect("transcript");
        let event = &transcript.events[0];

        assert_eq!(event.event_type.as_deref(), Some("unknown"));
        assert!(event.warnings.contains(&"unknown_event_type".to_string()));
        assert_eq!(event.metadata["raw_type"], json!("new_future_event"));
        assert_eq!(event.metadata["payload_type"], json!("future_payload"));
        assert!(event.metadata.get("raw_event").is_some());
        assert!(transcript
            .warnings
            .contains(&"unknown_event_count:1".to_string()));
    }

    #[test]
    fn rollout_outside_allowed_dirs_is_rejected() {
        let fixture = fixture("codex-transcript-outside");
        let outside = fixture.root.join("outside.jsonl");
        fs::write(&outside, "").expect("write outside");

        let error = read_transcript_from_rollout(metadata(&outside), &fixture.codex_home)
            .expect_err("outside path should be rejected");

        assert!(error.starts_with("rollout_outside_allowed_dirs:"));
    }

    #[test]
    fn missing_rollout_is_classified() {
        let fixture = fixture("codex-transcript-missing");
        let missing = fixture.sessions_dir.join("missing.jsonl");

        let error = read_transcript_from_rollout(metadata(&missing), &fixture.codex_home)
            .expect_err("missing rollout should be rejected");

        assert!(error.starts_with("rollout_missing:"));
    }

    struct Fixture {
        root: PathBuf,
        codex_home: PathBuf,
        sessions_dir: PathBuf,
    }

    fn fixture(prefix: &str) -> Fixture {
        let root = temp_dir(prefix);
        let codex_home = root.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions dir");
        fs::create_dir_all(codex_home.join("archived_sessions"))
            .expect("create archived sessions dir");
        Fixture {
            root,
            codex_home,
            sessions_dir,
        }
    }

    fn metadata(rollout_path: &Path) -> TranscriptThreadMetadata {
        TranscriptThreadMetadata {
            thread_id: "thread-ok".to_string(),
            rollout_path: Some(rollout_path.display().to_string()),
            project_root: Some("/tmp/project".to_string()),
            title: Some("Fixture thread".to_string()),
            created_at_ms: Some(1),
            updated_at_ms: Some(2),
            catalog_source: "test".to_string(),
            index_thread_count: Some(1),
        }
    }

    fn write_jsonl(path: &Path, rows: &[Value]) {
        let mut text = rows
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        text.push('\n');
        fs::write(path, text).expect("write jsonl");
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }
}
