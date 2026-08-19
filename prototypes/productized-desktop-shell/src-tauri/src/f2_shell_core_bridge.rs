//! F2 core-side newline-delimited shell bridge.
//!
//! This module is a transport adapter over an exact five-method registry. It
//! does not own identity, permissions, facts, grants, completion or fallback
//! paths. Those remain in the existing core targets selected below.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::path::Path;

use crate::m3_role_session_read_model::{
    M3RoleSessionDetailRequest, M3RoleSessionDirectoryRequest, M3RoleSessionReadHost,
};
use crate::operation_control::OperationControlDecisionRequest;
use crate::AppState;

const REQUEST_SCHEMA: &str = "syn.f2.shell-core-bridge.request.v1";
const RESPONSE_SCHEMA: &str = "syn.f2.shell-core-bridge.response.v1";
const DEFAULT_MAX_REQUEST_TIMEOUT_MS: u64 = 30_000;
const ABSOLUTE_MAX_REQUEST_TIMEOUT_MS: u64 = 30_000;
const FIXED_READ_HOST: M3RoleSessionReadHost = M3RoleSessionReadHost::Jiaoban;

const CODE_OK: &str = "F2_OK";
const CODE_PARSE_ERROR: &str = "F2_PARSE_ERROR";
const CODE_INVALID_REQUEST: &str = "F2_INVALID_REQUEST";
const CODE_PROTOCOL_MISMATCH: &str = "F2_PROTOCOL_MISMATCH";
const CODE_FORBIDDEN_AUTHORITY_INPUT: &str = "F2_FORBIDDEN_AUTHORITY_INPUT";
const CODE_UNKNOWN_METHOD: &str = "F2_UNKNOWN_METHOD";
const CODE_DEADLINE_EXPIRED: &str = "F2_DEADLINE_EXPIRED";
const CODE_DEADLINE_TOO_FAR: &str = "F2_DEADLINE_TOO_FAR";
const CODE_INVALID_IDEMPOTENCY_KEY: &str = "F2_INVALID_IDEMPOTENCY_KEY";
const CODE_IDEMPOTENCY_CONFLICT: &str = "F2_IDEMPOTENCY_CONFLICT";
const CODE_CORE_REJECTED: &str = "F2_CORE_REJECTED";
const CODE_INTERNAL_PANIC: &str = "F2_INTERNAL_PANIC";
const CODE_STOP_ACKNOWLEDGED: &str = "F2_STOP_ACKNOWLEDGED";

const METHOD_SECRETARY_STATUS: &str = "role_session.secretary_status";
const METHOD_GLOBAL_SUPERVISOR_STATUS: &str = "role_session.global_supervisor_status";
const METHOD_ROLE_SESSION_DIRECTORY: &str = "role_session.directory";
const METHOD_ROLE_SESSION_DETAIL: &str = "role_session.detail";
const METHOD_OPERATION_CONTROL_DECISION: &str = "operation_control.record_decision";
const METHOD_STOP: &str = "bridge.stop";

#[derive(Clone, Copy)]
struct MethodDescriptor {
    method: &'static str,
    dispatch_target: &'static str,
    invocation_class: &'static str,
}

const METHOD_REGISTRY: [MethodDescriptor; 5] = [
    MethodDescriptor {
        method: METHOD_SECRETARY_STATUS,
        dispatch_target: "load_secretary_role_session_status_for_state",
        invocation_class: "CORE_LOCAL_NO_PROVIDER",
    },
    MethodDescriptor {
        method: METHOD_GLOBAL_SUPERVISOR_STATUS,
        dispatch_target: "load_global_supervisor_role_session_status_for_state",
        invocation_class: "CORE_LOCAL_NO_PROVIDER",
    },
    MethodDescriptor {
        method: METHOD_ROLE_SESSION_DIRECTORY,
        dispatch_target: "load_role_session_directory_for_host",
        invocation_class: "CORE_LOCAL_NO_PROVIDER",
    },
    MethodDescriptor {
        method: METHOD_ROLE_SESSION_DETAIL,
        dispatch_target: "load_role_session_detail_for_host",
        invocation_class: "CORE_LOCAL_NO_PROVIDER",
    },
    MethodDescriptor {
        method: METHOD_OPERATION_CONTROL_DECISION,
        dispatch_target: "operation_control::record_operation_control_decision_at",
        invocation_class: "CORE_LOCAL_NO_PROVIDER",
    },
];

#[derive(Clone, Debug)]
struct BridgeConfig {
    app_data_root: PathBuf,
    index_seed: PathBuf,
    tasks_seed: PathBuf,
    role_session_project_locator: String,
    max_request_timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BridgeRequest {
    schema_version: String,
    request_id: String,
    method: String,
    deadline_unix_ms: Option<u64>,
    #[serde(default = "empty_params")]
    params: Value,
    #[serde(default)]
    external_refs: Vec<ExternalRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExternalRef {
    kind: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyParams {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirectoryParams {
    cursor: Option<String>,
    limit: Option<u32>,
    request_nonce: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DetailParams {
    selection: String,
    request_nonce: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationControlParams {
    idempotency_key: String,
    decision: OperationControlDecisionRequest,
}

#[derive(Serialize)]
#[serde(tag = "result_kind", content = "payload", rename_all = "snake_case")]
enum BridgeResult {
    SecretaryRoleSessionStatus(crate::m3_role_session_read_model::M3SecretaryRoleSessionStatusDto),
    GlobalSupervisorRoleSessionStatus(
        crate::m6_org_global_role_session::M6OrgGlobalRoleSessionStatusDto,
    ),
    RoleSessionDirectory(crate::m3_role_session_read_model::M3RoleSessionDirectoryDto),
    RoleSessionDetail(crate::m3_role_session_read_model::M3RoleSessionDetailDto),
    OperationControlReceipt(OperationControlBridgeReceipt),
    Stop(StopResult),
}

#[derive(Serialize)]
struct OperationControlBridgeReceipt {
    recovery: &'static str,
    audit_event_id: String,
    real_operation_executed: bool,
    confirmed_recorded_is_success: bool,
    core_receipt: Option<crate::WorkflowStateMutationResult>,
    recovered_snapshot: Option<crate::WorkflowStateSnapshot>,
}

#[derive(Serialize)]
struct StopResult {
    process_stop: &'static str,
    cancels_in_flight_core_call: bool,
    stops_agent_or_runtime: bool,
}

#[derive(Serialize)]
struct BridgeResponse {
    schema_version: &'static str,
    request_id: Option<String>,
    method: Option<String>,
    ok: bool,
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<BridgeResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<BridgeError>,
    receipt: BridgeTransportReceipt,
}

#[derive(Serialize)]
struct BridgeError {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    core_code: Option<String>,
    message: String,
}

#[derive(Serialize)]
struct BridgeTransportReceipt {
    idempotency_key: Option<String>,
    replayed: bool,
    external_refs: Vec<ExternalRef>,
}

struct DispatchOutcome {
    result: BridgeResult,
    idempotency_key: Option<String>,
    replayed: bool,
}

#[derive(Debug)]
struct DispatchFailure {
    code: &'static str,
    core_code: Option<String>,
    message: String,
    idempotency_key: Option<String>,
}

struct LineOutcome {
    payload: String,
    stop_after_response: bool,
}

pub(crate) fn run_cli(args: Vec<String>) -> Result<(), String> {
    let config = parse_cli_args(&args)?;
    let state = AppState::try_new_with_tauri_ordinary_product_seeds(
        &config.app_data_root,
        &config.index_seed,
        &config.tasks_seed,
    )?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_loop(&state, &config, stdin.lock(), stdout.lock())
}

fn run_loop<R: BufRead, W: Write>(
    state: &AppState,
    config: &BridgeConfig,
    reader: R,
    writer: W,
) -> Result<(), String> {
    let mut writer = BufWriter::new(writer);
    for line in reader.lines() {
        let line = line.map_err(|error| format!("F2_STDIN_READ_FAILED:{error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let outcome = handle_line_at(state, config, &line, current_unix_ms());
        writeln!(writer, "{}", outcome.payload)
            .map_err(|error| format!("F2_STDOUT_WRITE_FAILED:{error}"))?;
        writer
            .flush()
            .map_err(|error| format!("F2_STDOUT_FLUSH_FAILED:{error}"))?;
        if outcome.stop_after_response {
            break;
        }
    }
    Ok(())
}

fn parse_cli_args(args: &[String]) -> Result<BridgeConfig, String> {
    let mut app_data_root = None;
    let mut index_seed = None;
    let mut tasks_seed = None;
    let mut role_session_project_locator = None;
    let mut max_request_timeout_ms = DEFAULT_MAX_REQUEST_TIMEOUT_MS;
    let mut saw_timeout = false;
    let mut index = 0;
    while index < args.len() {
        let key = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("F2_CLI_MISSING_VALUE:{key}"))?;
        match key {
            "--app-data-root" => set_once(&mut app_data_root, value, key)?,
            "--index-seed" => set_once(&mut index_seed, value, key)?,
            "--tasks-seed" => set_once(&mut tasks_seed, value, key)?,
            "--role-session-project-locator" => {
                set_once(&mut role_session_project_locator, value, key)?
            }
            "--max-request-timeout-ms" => {
                if saw_timeout {
                    return Err(format!("F2_CLI_DUPLICATE_ARGUMENT:{key}"));
                }
                saw_timeout = true;
                max_request_timeout_ms = value
                    .parse::<u64>()
                    .map_err(|_| "F2_CLI_INVALID_MAX_REQUEST_TIMEOUT".to_string())?;
                if max_request_timeout_ms == 0
                    || max_request_timeout_ms > ABSOLUTE_MAX_REQUEST_TIMEOUT_MS
                {
                    return Err("F2_CLI_INVALID_MAX_REQUEST_TIMEOUT".to_string());
                }
            }
            _ => return Err(format!("F2_CLI_UNKNOWN_ARGUMENT:{key}")),
        }
        index += 2;
    }

    let app_data_root = canonical_explicit_path(
        app_data_root
            .as_deref()
            .ok_or_else(|| "F2_CLI_APP_DATA_ROOT_REQUIRED".to_string())?,
        true,
        "APP_DATA_ROOT",
    )?;
    let index_seed = canonical_explicit_path(
        index_seed
            .as_deref()
            .ok_or_else(|| "F2_CLI_INDEX_SEED_REQUIRED".to_string())?,
        false,
        "INDEX_SEED",
    )?;
    let tasks_seed = canonical_explicit_path(
        tasks_seed
            .as_deref()
            .ok_or_else(|| "F2_CLI_TASKS_SEED_REQUIRED".to_string())?,
        false,
        "TASKS_SEED",
    )?;
    let role_session_project_locator = role_session_project_locator
        .ok_or_else(|| "F2_CLI_ROLE_SESSION_PROJECT_LOCATOR_REQUIRED".to_string())?;
    validate_bounded_opaque(&role_session_project_locator, 1024)
        .map_err(|_| "F2_CLI_ROLE_SESSION_PROJECT_LOCATOR_INVALID".to_string())?;

    Ok(BridgeConfig {
        app_data_root,
        index_seed,
        tasks_seed,
        role_session_project_locator,
        max_request_timeout_ms,
    })
}

fn set_once(slot: &mut Option<String>, value: &str, key: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("F2_CLI_DUPLICATE_ARGUMENT:{key}"));
    }
    *slot = Some(value.to_string());
    Ok(())
}

fn canonical_explicit_path(raw: &str, directory: bool, label: &str) -> Result<PathBuf, String> {
    let supplied = PathBuf::from(raw);
    if !supplied.is_absolute() {
        return Err(format!("F2_CLI_{label}_MUST_BE_ABSOLUTE"));
    }
    let canonical =
        std::fs::canonicalize(&supplied).map_err(|_| format!("F2_CLI_{label}_UNAVAILABLE"))?;
    if canonical != supplied {
        return Err(format!("F2_CLI_{label}_MUST_BE_CANONICAL"));
    }
    if (directory && !canonical.is_dir()) || (!directory && !canonical.is_file()) {
        return Err(format!("F2_CLI_{label}_TYPE_MISMATCH"));
    }
    Ok(canonical)
}

fn handle_line_at(state: &AppState, config: &BridgeConfig, line: &str, now_ms: u64) -> LineOutcome {
    let raw = match serde_json::from_str::<Value>(line) {
        Ok(raw) => raw,
        Err(error) => {
            return line_error(
                None,
                None,
                CODE_PARSE_ERROR,
                None,
                format!("invalid JSON: {error}"),
                Vec::new(),
                None,
            )
        }
    };
    let request_id = raw
        .get("request_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let method = raw
        .get("method")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(forbidden) = find_forbidden_authority_key(&raw) {
        return line_error(
            request_id,
            method,
            CODE_FORBIDDEN_AUTHORITY_INPUT,
            None,
            format!("authority-bearing input key is forbidden: {forbidden}"),
            Vec::new(),
            None,
        );
    }
    let request = match serde_json::from_value::<BridgeRequest>(raw) {
        Ok(request) => request,
        Err(error) => {
            return line_error(
                request_id,
                method,
                CODE_INVALID_REQUEST,
                None,
                format!("invalid request envelope: {error}"),
                Vec::new(),
                None,
            )
        }
    };
    if request.schema_version != REQUEST_SCHEMA {
        return request_error(
            &request,
            CODE_PROTOCOL_MISMATCH,
            None,
            "request schema_version does not match v1".to_string(),
            None,
        );
    }
    if let Err(message) = validate_request_identity_and_refs(&request) {
        return request_error(&request, CODE_INVALID_REQUEST, None, message, None);
    }
    if request.method != METHOD_STOP
        && !METHOD_REGISTRY.iter().any(|entry| {
            entry.method == request.method
                && !entry.dispatch_target.is_empty()
                && entry.invocation_class == "CORE_LOCAL_NO_PROVIDER"
        })
    {
        return request_error(
            &request,
            CODE_UNKNOWN_METHOD,
            None,
            "method is not registered in F2 v1".to_string(),
            None,
        );
    }
    if request.method == METHOD_STOP {
        if serde_json::from_value::<EmptyParams>(request.params.clone()).is_err() {
            return request_error(
                &request,
                CODE_INVALID_REQUEST,
                None,
                "bridge.stop requires empty params".to_string(),
                None,
            );
        }
        return success_line(
            &request,
            DispatchOutcome {
                result: BridgeResult::Stop(StopResult {
                    process_stop: "acknowledged_at_request_boundary",
                    cancels_in_flight_core_call: false,
                    stops_agent_or_runtime: false,
                }),
                idempotency_key: None,
                replayed: false,
            },
            CODE_STOP_ACKNOWLEDGED,
            true,
        );
    }
    let deadline = match request.deadline_unix_ms {
        Some(deadline) => deadline,
        None => {
            return request_error(
                &request,
                CODE_INVALID_REQUEST,
                None,
                "deadline_unix_ms is required for domain methods".to_string(),
                None,
            )
        }
    };
    if deadline <= now_ms {
        return request_error(
            &request,
            CODE_DEADLINE_EXPIRED,
            None,
            "request deadline expired before dispatch".to_string(),
            None,
        );
    }
    if deadline - now_ms > config.max_request_timeout_ms {
        return request_error(
            &request,
            CODE_DEADLINE_TOO_FAR,
            None,
            "request deadline exceeds configured maximum".to_string(),
            None,
        );
    }

    match catch_dispatch(|| dispatch_request(state, config, &request)) {
        Ok(outcome) => success_line(&request, outcome, CODE_OK, false),
        Err(failure) => request_error(
            &request,
            failure.code,
            failure.core_code,
            failure.message,
            failure.idempotency_key,
        ),
    }
}

fn catch_dispatch<F>(dispatch: F) -> Result<DispatchOutcome, DispatchFailure>
where
    F: FnOnce() -> Result<DispatchOutcome, DispatchFailure>,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(dispatch)) {
        Ok(result) => result,
        Err(_) => Err(DispatchFailure {
            code: CODE_INTERNAL_PANIC,
            core_code: None,
            message: "request dispatch panicked; outcome is unknown".to_string(),
            idempotency_key: None,
        }),
    }
}

fn dispatch_request(
    state: &AppState,
    config: &BridgeConfig,
    request: &BridgeRequest,
) -> Result<DispatchOutcome, DispatchFailure> {
    let result = match request.method.as_str() {
        METHOD_SECRETARY_STATUS => {
            parse_empty_params(&request.params)?;
            BridgeResult::SecretaryRoleSessionStatus(
                crate::load_secretary_role_session_status_for_state(state).map_err(core_failure)?,
            )
        }
        METHOD_GLOBAL_SUPERVISOR_STATUS => {
            parse_empty_params(&request.params)?;
            BridgeResult::GlobalSupervisorRoleSessionStatus(
                crate::load_global_supervisor_role_session_status_for_state(state)
                    .map_err(core_failure)?,
            )
        }
        METHOD_ROLE_SESSION_DIRECTORY => {
            let params: DirectoryParams = parse_params(request.params.clone())?;
            let core_request = M3RoleSessionDirectoryRequest {
                project_locator: config.role_session_project_locator.clone(),
                cursor: params.cursor,
                limit: params.limit,
                request_nonce: params.request_nonce,
            };
            BridgeResult::RoleSessionDirectory(
                crate::load_role_session_directory_for_host(state, FIXED_READ_HOST, &core_request)
                    .map_err(core_failure)?,
            )
        }
        METHOD_ROLE_SESSION_DETAIL => {
            let params: DetailParams = parse_params(request.params.clone())?;
            let core_request = M3RoleSessionDetailRequest {
                project_locator: config.role_session_project_locator.clone(),
                selection: params.selection,
                request_nonce: params.request_nonce,
            };
            BridgeResult::RoleSessionDetail(
                crate::load_role_session_detail_for_host(state, FIXED_READ_HOST, &core_request)
                    .map_err(core_failure)?,
            )
        }
        METHOD_OPERATION_CONTROL_DECISION => {
            return dispatch_operation_control(state, request.params.clone())
        }
        _ => {
            return Err(DispatchFailure {
                code: CODE_UNKNOWN_METHOD,
                core_code: None,
                message: "method is not registered in F2 v1".to_string(),
                idempotency_key: None,
            })
        }
    };
    Ok(DispatchOutcome {
        result,
        idempotency_key: None,
        replayed: false,
    })
}

fn dispatch_operation_control(
    state: &AppState,
    raw_params: Value,
) -> Result<DispatchOutcome, DispatchFailure> {
    let params: OperationControlParams = parse_params(raw_params)?;
    let expected_key = format!("operation-control:{}", params.decision.operation_id);
    if params.idempotency_key != expected_key {
        return Err(DispatchFailure {
            code: CODE_INVALID_IDEMPOTENCY_KEY,
            core_code: None,
            message: "idempotency key must be operation-control:<operation_id>".to_string(),
            idempotency_key: Some(params.idempotency_key),
        });
    }
    if params.decision.readback_status != "not_attempted_l3_decision_only"
        || params.decision.runtime_status_after_confirmation
            != "operation_decision_recorded_pending_real_authorization"
    {
        return Err(DispatchFailure {
            code: CODE_CORE_REJECTED,
            core_code: Some("operation_control_bridge_status_contract_mismatch".to_string()),
            message: "operation-control status contract mismatch".to_string(),
            idempotency_key: Some(params.idempotency_key),
        });
    }

    let timestamp = crate::unix_timestamp_string();
    match crate::operation_control::record_operation_control_decision_at(
        &state.workflow_state_path,
        &params.decision,
        &timestamp,
    ) {
        Ok(core_receipt) => {
            let audit_event_id = core_receipt.audit_event_id.clone();
            Ok(DispatchOutcome {
                result: BridgeResult::OperationControlReceipt(OperationControlBridgeReceipt {
                    recovery: "first_commit",
                    audit_event_id,
                    real_operation_executed: false,
                    confirmed_recorded_is_success: false,
                    core_receipt: Some(core_receipt),
                    recovered_snapshot: None,
                }),
                idempotency_key: Some(params.idempotency_key),
                replayed: false,
            })
        }
        Err(error) if error.starts_with("operation_control_duplicate:") => {
            recover_operation_control_replay(state, params)
        }
        Err(error) => {
            let mut failure = core_failure(error);
            failure.idempotency_key = Some(params.idempotency_key);
            Err(failure)
        }
    }
}

fn recover_operation_control_replay(
    state: &AppState,
    params: OperationControlParams,
) -> Result<DispatchOutcome, DispatchFailure> {
    let workflow_state =
        crate::read_workflow_state_value(&state.workflow_state_path).map_err(core_failure)?;
    let event = workflow_state
        .get("audit_events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().find(|event| {
                event.get("event_type").and_then(Value::as_str)
                    == Some("operation_decision_recorded")
                    && event.get("operation_id").and_then(Value::as_str)
                        == Some(params.decision.operation_id.as_str())
                    && event.get("after_state").and_then(Value::as_str)
                        == Some("confirmed_recorded")
            })
        })
        .ok_or_else(|| DispatchFailure {
            code: CODE_CORE_REJECTED,
            core_code: Some("operation_control_duplicate_without_authoritative_audit".to_string()),
            message: "duplicate decision has no recoverable authoritative audit".to_string(),
            idempotency_key: Some(params.idempotency_key.clone()),
        })?;

    if !operation_audit_matches(event, &params.decision) {
        return Err(DispatchFailure {
            code: CODE_IDEMPOTENCY_CONFLICT,
            core_code: None,
            message: "persisted operation decision differs from replay".to_string(),
            idempotency_key: Some(params.idempotency_key),
        });
    }
    let audit_event_id = event
        .get("event_id")
        .and_then(Value::as_str)
        .ok_or_else(|| DispatchFailure {
            code: CODE_CORE_REJECTED,
            core_code: Some("operation_control_recovery_audit_id_missing".to_string()),
            message: "recoverable audit is missing event_id".to_string(),
            idempotency_key: Some(params.idempotency_key.clone()),
        })?
        .to_string();
    let recovered_snapshot =
        crate::read_workflow_state_snapshot(&state.workflow_state_path).map_err(core_failure)?;

    Ok(DispatchOutcome {
        result: BridgeResult::OperationControlReceipt(OperationControlBridgeReceipt {
            recovery: "authoritative_audit_replay",
            audit_event_id,
            real_operation_executed: false,
            confirmed_recorded_is_success: false,
            core_receipt: None,
            recovered_snapshot: Some(recovered_snapshot),
        }),
        idempotency_key: Some(params.idempotency_key),
        replayed: true,
    })
}

fn operation_audit_matches(event: &Value, request: &OperationControlDecisionRequest) -> bool {
    string_field_matches(event, "event_type", "operation_decision_recorded")
        && string_field_matches(event, "operation_id", &request.operation_id)
        && string_field_matches(event, "before_state", &request.current_status)
        && string_field_matches(event, "after_state", &request.status_after_confirmation)
        && string_field_matches(event, "current_gate", &request.current_gate)
        && string_field_matches(event, "label", &request.label)
        && string_field_matches(event, "would_write_if_real", &request.would_write_if_real)
        && string_field_matches(event, "risk_disclosure", &request.risk_disclosure)
        && string_field_matches(event, "readback_status", &request.readback_status)
        && event.get("readback_result_count") == Some(&Value::Null)
        && string_field_matches(
            event,
            "runtime_status_after_confirmation",
            &request.runtime_status_after_confirmation,
        )
        && bool_field_matches(event, "does_execute_in_l3", request.does_execute_in_l3)
        && bool_field_matches(
            event,
            "requires_separate_authorized_window",
            request.requires_separate_authorized_window,
        )
        && bool_field_matches(event, "blocks_k3_b2", request.blocks_k3_b2)
}

fn string_field_matches(event: &Value, key: &str, expected: &str) -> bool {
    event.get(key).and_then(Value::as_str) == Some(expected)
}

fn bool_field_matches(event: &Value, key: &str, expected: bool) -> bool {
    event.get(key).and_then(Value::as_bool) == Some(expected)
}

fn parse_empty_params(value: &Value) -> Result<(), DispatchFailure> {
    serde_json::from_value::<EmptyParams>(value.clone())
        .map(|_| ())
        .map_err(invalid_params)
}

fn parse_params<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, DispatchFailure> {
    serde_json::from_value(value).map_err(invalid_params)
}

fn invalid_params(error: serde_json::Error) -> DispatchFailure {
    DispatchFailure {
        code: CODE_INVALID_REQUEST,
        core_code: None,
        message: format!("invalid method params: {error}"),
        idempotency_key: None,
    }
}

fn core_failure(error: String) -> DispatchFailure {
    DispatchFailure {
        code: CODE_CORE_REJECTED,
        core_code: Some(stable_core_code(&error)),
        message: error,
        idempotency_key: None,
    }
}

fn stable_core_code(error: &str) -> String {
    let candidate = error
        .split(|character: char| character == ':' || character.is_whitespace())
        .next()
        .unwrap_or_default();
    if !candidate.is_empty()
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        candidate.to_string()
    } else {
        "core_rejected_unclassified".to_string()
    }
}

fn validate_request_identity_and_refs(request: &BridgeRequest) -> Result<(), String> {
    validate_bounded_opaque(&request.request_id, 160)
        .map_err(|_| "request_id must be non-empty, bounded and control-free".to_string())?;
    if request.external_refs.len() > 8 {
        return Err("external_refs exceeds eight entries".to_string());
    }
    for external_ref in &request.external_refs {
        if !matches!(
            external_ref.kind.as_str(),
            "thread_id" | "desktop_id" | "pairing_id"
        ) {
            return Err("external_refs kind is not allowed".to_string());
        }
        validate_bounded_opaque(&external_ref.value, 512)
            .map_err(|_| "external_refs value is invalid".to_string())?;
    }
    Ok(())
}

fn validate_bounded_opaque(value: &str, max_bytes: usize) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.chars().any(|character| character.is_control())
    {
        return Err(());
    }
    Ok(())
}

fn find_forbidden_authority_key(value: &Value) -> Option<&str> {
    const FORBIDDEN: [&str; 23] = [
        "actor_id",
        "owner_id",
        "owner_ref",
        "role",
        "role_ref",
        "scope",
        "scope_ref",
        "permission",
        "permission_snapshot_ref",
        "provider",
        "provider_handle",
        "model",
        "project_path",
        "project_root",
        "project_locator",
        "role_session_id",
        "session_id",
        "workflow_state_path",
        "index_path",
        "tasks_path",
        "app_data_root",
        "host",
        "timestamp",
    ];
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if FORBIDDEN.contains(&key.as_str()) {
                    return Some(key.as_str());
                }
                if let Some(found) = find_forbidden_authority_key(child) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(values) => values.iter().find_map(find_forbidden_authority_key),
        _ => None,
    }
}

fn success_line(
    request: &BridgeRequest,
    outcome: DispatchOutcome,
    code: &'static str,
    stop_after_response: bool,
) -> LineOutcome {
    serialize_response(
        BridgeResponse {
            schema_version: RESPONSE_SCHEMA,
            request_id: Some(request.request_id.clone()),
            method: Some(request.method.clone()),
            ok: true,
            code,
            result: Some(outcome.result),
            error: None,
            receipt: BridgeTransportReceipt {
                idempotency_key: outcome.idempotency_key,
                replayed: outcome.replayed,
                external_refs: request.external_refs.clone(),
            },
        },
        stop_after_response,
    )
}

fn request_error(
    request: &BridgeRequest,
    code: &'static str,
    core_code: Option<String>,
    message: String,
    idempotency_key: Option<String>,
) -> LineOutcome {
    line_error(
        Some(request.request_id.clone()),
        Some(request.method.clone()),
        code,
        core_code,
        message,
        request.external_refs.clone(),
        idempotency_key,
    )
}

fn line_error(
    request_id: Option<String>,
    method: Option<String>,
    code: &'static str,
    core_code: Option<String>,
    message: String,
    external_refs: Vec<ExternalRef>,
    idempotency_key: Option<String>,
) -> LineOutcome {
    serialize_response(
        BridgeResponse {
            schema_version: RESPONSE_SCHEMA,
            request_id,
            method,
            ok: false,
            code,
            result: None,
            error: Some(BridgeError {
                code,
                core_code,
                message: bounded_message(&message),
            }),
            receipt: BridgeTransportReceipt {
                idempotency_key,
                replayed: false,
                external_refs,
            },
        },
        false,
    )
}

fn serialize_response(response: BridgeResponse, stop_after_response: bool) -> LineOutcome {
    let payload = serde_json::to_string(&response).unwrap_or_else(|_| {
        format!(
            "{{\"schema_version\":\"{RESPONSE_SCHEMA}\",\"request_id\":null,\"method\":null,\"ok\":false,\"code\":\"{CODE_INTERNAL_PANIC}\",\"error\":{{\"code\":\"{CODE_INTERNAL_PANIC}\",\"message\":\"response serialization failed\"}},\"receipt\":{{\"idempotency_key\":null,\"replayed\":false,\"external_refs\":[]}}}}"
        )
    });
    LineOutcome {
        payload,
        stop_after_response,
    }
}

fn bounded_message(message: &str) -> String {
    message.chars().take(512).collect()
}

fn empty_params() -> Value {
    Value::Object(Default::default())
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{
        CorrelationId, OpaqueRef, RequestIdempotencyKey, RoleSessionId, ServerResolvedBinding,
        Sha256Digest,
    };
    use crate::m3_role_session_read_model::{
        M3C07IsolatedReadBinding, M3RoleSessionReadRuntimeSlot,
    };
    use crate::m3_role_session_repository::{
        CreateRoleSessionCommand, M3CommandMetadata, M3RoleSessionSqliteRepository,
    };
    use serde_json::json;
    use std::fs;

    const NOW_MS: u64 = 1_787_126_400_000;
    const FIXTURE: &str =
        include_str!("../../../../docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json");

    fn temp_dir(tag: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "syn-f2-bridge-{tag}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).expect("create F2 fixture directory");
        fs::canonicalize(path).expect("canonicalize F2 fixture directory")
    }

    fn config(root: &Path, project_locator: &str) -> BridgeConfig {
        let index_seed = root.join("index-seed.json");
        let tasks_seed = root.join("tasks-seed.md");
        fs::write(&index_seed, r#"{"projects":[]}"#).expect("write index seed");
        fs::write(&tasks_seed, "# F2 fixture tasks\n").expect("write tasks seed");
        BridgeConfig {
            app_data_root: root.to_path_buf(),
            index_seed,
            tasks_seed,
            role_session_project_locator: project_locator.to_string(),
            max_request_timeout_ms: 30_000,
        }
    }

    fn uninstalled_state(root: &Path) -> AppState {
        let workflow_state_path = root.join("workflow-state.v0.json");
        let initial = crate::initial_workflow_state_json(
            "2026-08-19T00:00:00Z",
            "audit:init:f2-bridge",
            false,
            &workflow_state_path,
        );
        crate::atomic_write_json(&workflow_state_path, &initial).expect("write workflow fixture");
        AppState {
            index_path: root.join("index.json"),
            tasks_path: root.join("tasks.md"),
            workflow_state_path,
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
            m6_org_global_role_session: Default::default(),
        }
    }

    fn ordinary_state(tag: &str) -> (PathBuf, AppState, BridgeConfig) {
        let parent = temp_dir(tag);
        let app_data_root = parent.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        fs::create_dir_all(&app_data_root).expect("create ordinary app-data");
        let app_data_root = fs::canonicalize(app_data_root).expect("canonical app-data");
        let bridge_config = config(&app_data_root, "project:f2-core-provisioned");
        let state = AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &bridge_config.index_seed,
            &bridge_config.tasks_seed,
        )
        .expect("construct ordinary F2 AppState under cfg(test)");
        (parent, state, bridge_config)
    }

    fn request(method: &str, params: Value) -> Value {
        json!({
            "schema_version": REQUEST_SCHEMA,
            "request_id": format!("request:{method}"),
            "method": method,
            "deadline_unix_ms": NOW_MS + 1_000,
            "params": params,
            "external_refs": []
        })
    }

    fn response(state: &AppState, config: &BridgeConfig, request: Value) -> (Value, bool) {
        let outcome = handle_line_at(state, config, &request.to_string(), NOW_MS);
        (
            serde_json::from_str(&outcome.payload).expect("parse bridge response"),
            outcome.stop_after_response,
        )
    }

    fn assert_code(response: &Value, code: &str) {
        assert_eq!(response["code"], code, "response was {response:#}");
    }

    fn sealed(namespace: &str, value: &str) -> String {
        format!(
            "{namespace}:sha256:{}",
            Sha256Digest::of_bytes(value.as_bytes()).as_str()
        )
    }

    fn opaque(namespace: &str, value: &str) -> OpaqueRef {
        OpaqueRef::try_from_canonical(sealed(namespace, value)).expect("opaque fixture ref")
    }

    fn metadata(tag: &str) -> M3CommandMetadata {
        M3CommandMetadata {
            receipt_id: opaque("receipt", tag),
            event_id: opaque("event", tag),
            audit_id: opaque("audit", tag),
            correlation_id: CorrelationId::try_from_canonical(sealed("correlation", tag))
                .expect("fixture correlation"),
            request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed(
                "request", tag,
            ))
            .expect("fixture idempotency"),
            occurred_at: "2026-08-19T00:00:00Z".to_string(),
        }
    }

    fn directory_state(tag: &str) -> (PathBuf, AppState, BridgeConfig) {
        let root = temp_dir(tag);
        let repository =
            M3RoleSessionSqliteRepository::open_rehearsal(&root.join("role-session.sqlite"))
                .expect("open M3 rehearsal repository");
        let binding = ServerResolvedBinding::from_server_canonical(
            sealed("actor", tag),
            sealed("role", "project-supervisor"),
            sealed("scope", tag),
            sealed("object", tag),
            sealed("channel", "jiaoban"),
            sealed("permission", tag),
        )
        .expect("server fixture binding");
        let role_session_id =
            RoleSessionId::try_from_canonical(sealed("session", tag)).expect("session id");
        repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id,
                binding: binding.clone(),
                metadata: metadata(tag),
            })
            .expect("seed role session");
        let project_locator = "project:f2-core-provisioned";
        let read_runtime = M3RoleSessionReadRuntimeSlot::from_m3c07_isolated_acceptance(vec![
            M3C07IsolatedReadBinding {
                host: M3RoleSessionReadHost::Jiaoban,
                project_locator: project_locator.to_string(),
                repository,
                binding,
            },
        ]);
        let mut state = uninstalled_state(&root);
        state.m3_role_session_read_runtime = read_runtime;
        let bridge_config = config(&root, project_locator);
        (root, state, bridge_config)
    }

    fn decision(operation_id: &str) -> Value {
        json!({
            "operation_id": operation_id,
            "label": "恢复",
            "current_status": "available",
            "status_after_confirmation": "confirmed_recorded",
            "current_gate": "gated_real_resume_mario_test_only",
            "would_write_if_real": "codex_home_and_workbench_state",
            "risk_disclosure": "确认后只记录恢复决策；不会进入 real-resume phase B。",
            "readback_status": "not_attempted_l3_decision_only",
            "readback_result_count": null,
            "audit_event_type": "operation_decision_recorded",
            "runtime_status_after_confirmation": "operation_decision_recorded_pending_real_authorization",
            "does_execute_in_l3": false,
            "requires_separate_authorized_window": true,
            "blocks_k3_b2": true
        })
    }

    #[test]
    fn f2c01_fixture_registry_and_all_case_shapes_are_machine_checked() {
        let fixture: Value = serde_json::from_str(FIXTURE).expect("parse F2 contract fixture");
        let registry = fixture["method_registry"]
            .as_array()
            .expect("registry array");
        assert_eq!(registry.len(), METHOD_REGISTRY.len());
        for entry in METHOD_REGISTRY {
            let fixture_entry = registry
                .iter()
                .find(|candidate| candidate["method"] == entry.method)
                .expect("exact method fixture");
            assert_eq!(fixture_entry["dispatch_target"], entry.dispatch_target);
            assert_eq!(fixture_entry["invocation_class"], entry.invocation_class);
            assert_eq!(entry.invocation_class, "CORE_LOCAL_NO_PROVIDER");
        }
        let required = fixture["required_keys"]
            .as_array()
            .expect("required keys")
            .iter()
            .map(|value| value.as_str().expect("required key"))
            .collect::<Vec<_>>();
        let cases = fixture["cases"].as_array().expect("cases");
        assert!(cases.len() >= 25);
        for case in cases {
            for key in &required {
                assert!(case.get(*key).is_some(), "case missing {key}: {case:#}");
            }
        }
        for code in [
            CODE_PARSE_ERROR,
            CODE_INVALID_REQUEST,
            CODE_PROTOCOL_MISMATCH,
            CODE_FORBIDDEN_AUTHORITY_INPUT,
            CODE_UNKNOWN_METHOD,
            CODE_DEADLINE_EXPIRED,
            CODE_DEADLINE_TOO_FAR,
            CODE_INVALID_IDEMPOTENCY_KEY,
            CODE_IDEMPOTENCY_CONFLICT,
            CODE_CORE_REJECTED,
            CODE_INTERNAL_PANIC,
        ] {
            assert!(fixture["stable_error_codes"]
                .as_array()
                .expect("error codes")
                .iter()
                .any(|candidate| candidate == code));
        }
    }

    #[test]
    fn f2c01_no_model_invocation_is_exact_registry_and_source_constraint() {
        let source = include_str!("f2_shell_core_bridge.rs");
        let dispatch_start = source
            .find("fn dispatch_request(")
            .expect("dispatch source start");
        let dispatch_end = source
            .find("#[cfg(test)]\nmod tests")
            .expect("production source end");
        let production = &source[..dispatch_end];
        let dispatch = &source[dispatch_start..dispatch_end];
        for target in METHOD_REGISTRY.map(|entry| entry.dispatch_target) {
            assert!(dispatch.contains(target), "missing exact target {target}");
        }
        assert!(dispatch.contains("FIXED_READ_HOST"));
        assert!(source.contains(
            "const FIXED_READ_HOST: M3RoleSessionReadHost = M3RoleSessionReadHost::Jiaoban"
        ));
        for forbidden in [
            "spawn_blocking",
            "send_secretary_message",
            "resolve_secretary_source_route",
            "ProviderPort",
            "ModelPort",
        ] {
            assert!(
                !dispatch.contains(forbidden),
                "dispatch contains forbidden invocation marker {forbidden}"
            );
        }
        assert!(production.contains("try_new_with_tauri_ordinary_product_seeds"));
        assert!(!production.contains("AppState::try_new_with_tauri_app_data_root"));
    }

    #[test]
    fn f2c01_status_methods_cover_ready_and_fail_closed_unavailable_states() {
        let (parent, ordinary, ordinary_config) = ordinary_state("status-ready");
        let (secretary, _) = response(
            &ordinary,
            &ordinary_config,
            request(METHOD_SECRETARY_STATUS, json!({})),
        );
        assert_code(&secretary, CODE_OK);
        assert_eq!(
            secretary["result"]["result_kind"],
            "secretary_role_session_status"
        );
        let (global, _) = response(
            &ordinary,
            &ordinary_config,
            request(METHOD_GLOBAL_SUPERVISOR_STATUS, json!({})),
        );
        assert_code(&global, CODE_OK);
        assert_eq!(global["result"]["payload"]["availability"], "ready");
        drop(ordinary);
        let _ = fs::remove_dir_all(parent);

        let root = temp_dir("status-unavailable");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let (secretary, _) = response(
            &state,
            &bridge_config,
            request(METHOD_SECRETARY_STATUS, json!({})),
        );
        assert_code(&secretary, CODE_CORE_REJECTED);
        assert_eq!(secretary["error"]["core_code"], "M3_BINDING_UNAVAILABLE");
        let (global, _) = response(
            &state,
            &bridge_config,
            request(METHOD_GLOBAL_SUPERVISOR_STATUS, json!({})),
        );
        assert_code(&global, CODE_OK);
        assert_eq!(global["result"]["payload"]["availability"], "unavailable");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_directory_and_detail_use_fixed_host_and_opaque_selection() {
        let (root, state, bridge_config) = directory_state("directory-detail");
        let (directory, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_ROLE_SESSION_DIRECTORY,
                json!({"cursor": null, "limit": 20, "request_nonce": "directory-1"}),
            ),
        );
        assert_code(&directory, CODE_OK);
        let selection = directory["result"]["payload"]["entries"][0]["selection"]
            .as_str()
            .expect("opaque selection")
            .to_string();
        assert!(!selection.contains("session:sha256:"));
        let (detail, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_ROLE_SESSION_DETAIL,
                json!({"selection": selection, "request_nonce": "detail-1"}),
            ),
        );
        assert_code(&detail, CODE_OK);

        let (stale, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_ROLE_SESSION_DETAIL,
                json!({"selection": "m3rs:unknown:1", "request_nonce": "detail-stale"}),
            ),
        );
        assert_code(&stale, CODE_CORE_REJECTED);
        assert_eq!(
            stale["error"]["core_code"],
            "m3_role_session_selector_unknown"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_directory_unavailable_does_not_fallback() {
        let root = temp_dir("directory-unavailable");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let (response, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_ROLE_SESSION_DIRECTORY,
                json!({"cursor": null, "limit": 20, "request_nonce": "directory-unavailable"}),
            ),
        );
        assert_code(&response, CODE_CORE_REJECTED);
        assert_eq!(response["error"]["core_code"], "M3_BINDING_UNAVAILABLE");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_operation_receipt_exact_replay_and_divergence_are_fail_closed() {
        let root = temp_dir("operation-replay");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let mut first = request(
            METHOD_OPERATION_CONTROL_DECISION,
            json!({
                "idempotency_key": "operation-control:resume",
                "decision": decision("resume")
            }),
        );
        first["external_refs"] = json!([
            {"kind": "thread_id", "value": "shell-thread-opaque"},
            {"kind": "desktop_id", "value": "shell-desktop-opaque"},
            {"kind": "pairing_id", "value": "shell-pairing-opaque"}
        ]);
        let (first_response, _) = response(&state, &bridge_config, first.clone());
        assert_code(&first_response, CODE_OK);
        assert_eq!(first_response["receipt"]["replayed"], false);
        assert_eq!(
            first_response["result"]["payload"]["real_operation_executed"],
            false
        );
        assert_eq!(
            first_response["result"]["payload"]["confirmed_recorded_is_success"],
            false
        );
        assert_eq!(
            first_response["receipt"]["external_refs"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert!(!first_response["result"]
            .to_string()
            .contains("shell-thread-opaque"));

        let (replay, _) = response(&state, &bridge_config, first.clone());
        assert_code(&replay, CODE_OK);
        assert_eq!(replay["receipt"]["replayed"], true);
        assert_eq!(
            replay["result"]["payload"]["recovery"],
            "authoritative_audit_replay"
        );

        let mut divergent = first;
        divergent["params"]["decision"]["label"] = json!("不同标签");
        let (conflict, _) = response(&state, &bridge_config, divergent);
        assert_code(&conflict, CODE_IDEMPOTENCY_CONFLICT);
        let workflow = crate::read_workflow_state_value(&state.workflow_state_path)
            .expect("read authoritative workflow");
        assert_eq!(
            workflow["audit_events"]
                .as_array()
                .expect("audit array")
                .iter()
                .filter(|event| event["event_type"] == "operation_decision_recorded")
                .count(),
            1
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_operation_invalid_key_and_execution_claim_cover_core_errors_without_write() {
        let root = temp_dir("operation-errors");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let (invalid_key, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_OPERATION_CONTROL_DECISION,
                json!({
                    "idempotency_key": "random-shell-key",
                    "decision": decision("stop")
                }),
            ),
        );
        assert_code(&invalid_key, CODE_INVALID_IDEMPOTENCY_KEY);

        let mut execution_claim = decision("retry");
        execution_claim["does_execute_in_l3"] = json!(true);
        let (rejected, _) = response(
            &state,
            &bridge_config,
            request(
                METHOD_OPERATION_CONTROL_DECISION,
                json!({
                    "idempotency_key": "operation-control:retry",
                    "decision": execution_claim
                }),
            ),
        );
        assert_code(&rejected, CODE_CORE_REJECTED);
        assert_eq!(
            rejected["error"]["core_code"],
            "operation_control_decision_must_not_execute_in_l3"
        );
        let workflow = crate::read_workflow_state_value(&state.workflow_state_path)
            .expect("read unchanged workflow");
        assert_eq!(workflow["audit_events"].as_array().unwrap().len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_protocol_errors_deadlines_stop_and_panic_have_stable_codes() {
        let root = temp_dir("protocol-errors");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");

        let parse = handle_line_at(&state, &bridge_config, "{not-json", NOW_MS);
        assert_code(
            &serde_json::from_str(&parse.payload).expect("parse parse-error response"),
            CODE_PARSE_ERROR,
        );

        let mut unknown_field = request(METHOD_SECRETARY_STATUS, json!({}));
        unknown_field["unknown_field"] = json!(true);
        let (invalid, _) = response(&state, &bridge_config, unknown_field);
        assert_code(&invalid, CODE_INVALID_REQUEST);

        let mut mismatch = request(METHOD_SECRETARY_STATUS, json!({}));
        mismatch["schema_version"] = json!("syn.f2.shell-core-bridge.request.v0");
        let (mismatch, _) = response(&state, &bridge_config, mismatch);
        assert_code(&mismatch, CODE_PROTOCOL_MISMATCH);

        let (unknown, _) = response(
            &state,
            &bridge_config,
            request("secretary.send_message", json!({})),
        );
        assert_code(&unknown, CODE_UNKNOWN_METHOD);

        let mut forbidden = request(
            METHOD_ROLE_SESSION_DETAIL,
            json!({"selection": "opaque", "request_nonce": "forbidden"}),
        );
        forbidden["params"]["role_session_id"] = json!("shell-claim");
        let (forbidden, _) = response(&state, &bridge_config, forbidden);
        assert_code(&forbidden, CODE_FORBIDDEN_AUTHORITY_INPUT);

        let mut expired = request(METHOD_SECRETARY_STATUS, json!({}));
        expired["deadline_unix_ms"] = json!(NOW_MS);
        let (expired, _) = response(&state, &bridge_config, expired);
        assert_code(&expired, CODE_DEADLINE_EXPIRED);

        let mut too_far = request(METHOD_SECRETARY_STATUS, json!({}));
        too_far["deadline_unix_ms"] = json!(NOW_MS + 30_001);
        let (too_far, _) = response(&state, &bridge_config, too_far);
        assert_code(&too_far, CODE_DEADLINE_TOO_FAR);

        let mut stop = request(METHOD_STOP, json!({}));
        stop.as_object_mut().unwrap().remove("deadline_unix_ms");
        let (stop, stop_after_response) = response(&state, &bridge_config, stop);
        assert_code(&stop, CODE_STOP_ACKNOWLEDGED);
        assert!(stop_after_response);
        assert_eq!(stop["result"]["payload"]["stops_agent_or_runtime"], false);

        let panic = match catch_dispatch(|| panic!("F2 fixture panic")) {
            Ok(_) => panic!("panic must map to stable error"),
            Err(error) => error,
        };
        assert_eq!(panic.code, CODE_INTERNAL_PANIC);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_cli_requires_explicit_canonical_paths_and_has_no_path_fallback() {
        let root = temp_dir("cli-paths");
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let args = vec![
            "--app-data-root".to_string(),
            root.display().to_string(),
            "--index-seed".to_string(),
            bridge_config.index_seed.display().to_string(),
            "--tasks-seed".to_string(),
            bridge_config.tasks_seed.display().to_string(),
            "--role-session-project-locator".to_string(),
            "project:f2-core-provisioned".to_string(),
        ];
        let parsed = parse_cli_args(&args).expect("explicit CLI config");
        assert_eq!(parsed.app_data_root, root);
        assert_eq!(parsed.max_request_timeout_ms, 30_000);

        let missing = parse_cli_args(&args[2..]).expect_err("app-data root is required");
        assert_eq!(missing, "F2_CLI_APP_DATA_ROOT_REQUIRED");
        let relative = vec![
            "--app-data-root".to_string(),
            "relative".to_string(),
            "--index-seed".to_string(),
            bridge_config.index_seed.display().to_string(),
            "--tasks-seed".to_string(),
            bridge_config.tasks_seed.display().to_string(),
            "--role-session-project-locator".to_string(),
            "project:f2-core-provisioned".to_string(),
        ];
        assert_eq!(
            parse_cli_args(&relative).expect_err("relative root must fail"),
            "F2_CLI_APP_DATA_ROOT_MUST_BE_ABSOLUTE"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn f2c01_run_loop_is_line_delimited_and_stops_after_ack() {
        let root = temp_dir("run-loop");
        let state = uninstalled_state(&root);
        let bridge_config = config(&root, "project:f2-core-provisioned");
        let mut first = request(METHOD_GLOBAL_SUPERVISOR_STATUS, json!({}));
        first["request_id"] = json!("loop:first");
        let mut stop = request(METHOD_STOP, json!({}));
        stop["request_id"] = json!("loop:stop");
        stop.as_object_mut().unwrap().remove("deadline_unix_ms");
        let ignored_after_stop = request(METHOD_GLOBAL_SUPERVISOR_STATUS, json!({}));
        let input = format!("\n{}\n{}\n{}\n", first, stop, ignored_after_stop);
        let mut output = Vec::new();
        run_loop(&state, &bridge_config, input.as_bytes(), &mut output).expect("run bridge loop");
        let lines = String::from_utf8(output)
            .expect("UTF-8 bridge output")
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("typed response line"))
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["request_id"], "loop:first");
        assert_eq!(lines[1]["code"], CODE_STOP_ACKNOWLEDGED);
        let _ = fs::remove_dir_all(root);
    }
}
