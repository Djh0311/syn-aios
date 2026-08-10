//! M4C09 isolated ordinary-product acceptance runtime.
//!
//! The R4 profile remains the filesystem isolation authority.  This module
//! installs the ordinary M3 Secretary RoleSession and ordinary M4 repository
//! below that validated profile, seeds only two fixed structured synthetic
//! owners, and records a scrubbed restart receipt.  It has no real model,
//! provider, connector, account, Codex-message, or network adapter.

use crate::acceptance_runtime_profile::RuntimePaths;
use crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot;
use crate::m4_secretary_domain::{
    m4_internal_id, m4_primary_scope_ref, M4AttentionSignals, M4SourceLinkInput,
    M4WorkflowAttentionSourceInput, M4_SCRUBBED_SENSITIVITY, M4_WORKFLOW_ATTENTION_OBJECT_TYPE,
    M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
};
use crate::m4_secretary_read_model::{M4CoordinationSnapshot, M4SecretaryDailyReportEnvelope};
use crate::m4_secretary_repository::{
    M4OrdinarySecretaryRepositoryConfig, M4SecretarySqliteRepository,
    M4_ORDINARY_SECRETARY_RELATIVE_PATH,
};
use crate::m4_secretary_scheduler::M4SchedulerTrigger;
use crate::m4_secretary_service::{
    M4SecretaryApplicationService, M4SecretaryControlledModelEnhancementPort,
    M4SecretaryCoordinationSnapshotReadPort, M4SecretaryExplicitUserMessageTrigger,
    M4SecretaryHandoffPort, M4SecretaryHandoffPortRecord, M4SecretaryHandoffRequest,
    M4SecretaryHash, M4SecretaryModelEnhancementRequest, M4SecretaryModelEnhancementStatus,
    M4SecretaryModelPortOutcome, M4SecretaryOpaqueRef, M4SecretaryRoleSessionReadPort,
    M4SecretaryRoleSessionState, M4SecretaryServiceError, M4SecretaryServiceTrigger,
    M4SecretaryTypedRef,
};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub(crate) const M4C09_MODE_ENV: &str = "SYN_M4C09_ISOLATED_ACCEPTANCE";
pub(crate) const M4C09_MODE_VALUE: &str = "1";
pub(crate) const M4C09_RUNTIME_SCHEMA_VERSION: &str = "syn.m4c09.isolated-product-app-runtime.v1";
pub(crate) const M4C09_ISOLATED_IPC_BLOCKED: &str = "m4c09_isolated_acceptance_ipc_blocked";
pub(crate) const M4C09_MODE_CONFLICT: &str = "m4c09_isolated_acceptance_mode_conflict";

const M4C09_RECEIPT_FILE_NAME: &str = "m4c09-runtime-receipt.json";
const M4C09_PRODUCT_APP_DATA_DIR: &str = "local.codex.governance.workbench";
const M4C09_MAX_RECEIPT_BYTES: u64 = 32 * 1024;
const M4C09_MAX_LAUNCHES: u64 = 8;
const M4C09_FAKE_MODEL_FAILURE: &str = "M4C09_FAKE_MODEL_FAILURE";
const M4C09_FIXED_TEST_NOW: &str = "2026-08-10T16:20:00.000Z";

// The C09 child is the ordinary M4 product surface over an isolated profile.
// Startup reads and the fixed M4 read/coordination commands are admitted;
// all dispatch, provider, connector, project-write, and legacy product
// commands are rejected by the global invoke handler before deserialization.
const M4C09_ALLOWED_TAURI_COMMANDS: &[&str] = &[
    "load_workbench_snapshot",
    "query_workbench_page_read_model",
    "load_system_status_read_model",
    "load_workflow_state_snapshot",
    "load_blackboard_candidate_store",
    "load_plan_authorization_store",
    "load_project_consultation_proposal_store",
    "load_memory_capture_store",
    "load_observation_store",
    "load_memory_candidate_store",
    "load_formal_memory_store",
    "load_memory_lint_store",
    "load_memory_entity_relation_store",
    "load_memory_pattern_store",
    "load_global_supervisor_review_store",
    "load_secretary_home_context",
    "load_secretary_role_session_status",
    "load_secretary_legacy_read_compatibility_report",
    "load_secretary_daily_report",
    "run_secretary_explain",
    "operate_secretary_coordination",
    "load_m4c09_acceptance_status",
];

#[derive(Clone)]
pub(crate) struct M4C09InstalledRuntime {
    pub(crate) m3_read_runtime: M3RoleSessionReadRuntimeSlot,
    pub(crate) repository: M4SecretarySqliteRepository,
}

#[derive(Clone)]
struct M4C09AcceptanceRuntime {
    root: PathBuf,
    receipt: M4C09AcceptanceStatusDto,
    m3_read_runtime: M3RoleSessionReadRuntimeSlot,
    repository: M4SecretarySqliteRepository,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09OwnerStateDto {
    pub(crate) owner_code: String,
    pub(crate) object_code: String,
    pub(crate) inbox_status: String,
    pub(crate) open_loop_status: String,
    pub(crate) open_loop_revision: String,
    pub(crate) item_ref_hash: String,
    pub(crate) route_ref_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09IngestionProofDto {
    pub(crate) two_fixed_source_owners: bool,
    pub(crate) first_launch_fresh_admissions: bool,
    pub(crate) exact_duplicate_replayed: bool,
    pub(crate) restart_seed_replayed: bool,
    pub(crate) admitted_source_event_rows: u64,
    pub(crate) inbox_rows: u64,
    pub(crate) open_loop_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09DailyProofDto {
    pub(crate) empty_run_zero_material_events: bool,
    pub(crate) empty_run_zero_agent_turns: bool,
    pub(crate) empty_run_zero_model_invocations: bool,
    pub(crate) repeated_refresh_stable: bool,
    pub(crate) daily_report_ref_hash: String,
    pub(crate) report_version: String,
    pub(crate) daily_report_rows: u64,
    pub(crate) scheduler_run_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09LifecycleProofDto {
    pub(crate) acknowledged_state_recovered: bool,
    pub(crate) snoozed_state_recovered: bool,
    pub(crate) carried_over_receipt_recovered: bool,
    pub(crate) carried_over_receipt_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09ModelProofDto {
    pub(crate) fake_model_only: bool,
    pub(crate) zero_item_read_model_calls: bool,
    pub(crate) deterministic_brief_unchanged_after_failure: bool,
    pub(crate) first_failure_recorded: bool,
    pub(crate) exact_failure_replay_recorded: bool,
    pub(crate) terminal_failure_recovered_after_restart: bool,
    pub(crate) fake_adapter_calls_this_launch: u64,
    pub(crate) fake_adapter_calls_total: u64,
    pub(crate) durable_invocation_rows: u64,
    pub(crate) real_model_attempts: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09IsolationProofDto {
    pub(crate) validated_profile_required: bool,
    pub(crate) ordinary_product_runtime_used: bool,
    pub(crate) synthetic_fixture_only: bool,
    pub(crate) profile_fingerprint: String,
    pub(crate) real_provider_attempts: u64,
    pub(crate) external_connector_attempts: u64,
    pub(crate) external_network_writes: u64,
    pub(crate) real_codex_message_attempts: u64,
}

/// The same DTO is persisted and exposed to the fixed no-argument acceptance
/// command.  It contains only codes, hashes, booleans and counts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct M4C09AcceptanceStatusDto {
    pub(crate) schema_version: String,
    pub(crate) evidence_level: String,
    pub(crate) launch_count: u64,
    pub(crate) role_session_hash: String,
    pub(crate) same_role_session_recovered: bool,
    pub(crate) owners: Vec<M4C09OwnerStateDto>,
    pub(crate) ingestion: M4C09IngestionProofDto,
    pub(crate) lifecycle: M4C09LifecycleProofDto,
    pub(crate) daily: M4C09DailyProofDto,
    pub(crate) model: M4C09ModelProofDto,
    pub(crate) isolation: M4C09IsolationProofDto,
    pub(crate) evidence_limit: String,
}

impl M4C09AcceptanceRuntime {
    fn open(
        paths: &RuntimePaths,
    ) -> Result<
        (
            Self,
            M3RoleSessionReadRuntimeSlot,
            M4SecretarySqliteRepository,
        ),
        String,
    > {
        if !cfg!(debug_assertions) {
            return Err("m4c09_debug_build_required".to_string());
        }
        let root = canonical_profile_root(paths)?;
        let app_data_base = canonical_contained_directory(&root, &paths.app_data_root)?;
        let product_app_data_root = app_data_base.join(M4C09_PRODUCT_APP_DATA_DIR);
        fs::create_dir_all(&product_app_data_root)
            .map_err(|_| "m4c09_product_app_data_create_failed".to_string())?;
        let product_app_data_root = fs::canonicalize(&product_app_data_root)
            .map_err(|_| "m4c09_product_app_data_unavailable".to_string())?;
        if !product_app_data_root.starts_with(&root)
            || product_app_data_root
                .file_name()
                .and_then(|name| name.to_str())
                != Some(M4C09_PRODUCT_APP_DATA_DIR)
        {
            return Err("m4c09_product_app_data_outside_profile".to_string());
        }

        let receipt_parent = canonical_contained_directory(&root, &paths.app_log_dir)?;
        let receipt_path = receipt_parent.join(M4C09_RECEIPT_FILE_NAME);
        let previous = read_previous_receipt(&receipt_path)?;
        let profile_fingerprint = hash_text(&root.to_string_lossy());
        if previous.as_ref().is_some_and(|receipt| {
            receipt.isolation.profile_fingerprint != profile_fingerprint
                || receipt.launch_count >= M4C09_MAX_LAUNCHES
        }) {
            return Err("m4c09_previous_receipt_profile_or_launch_invalid".to_string());
        }

        let m3_read_runtime =
            crate::m4_secretary_domain::install_ordinary_product_secretary_runtime(
                &product_app_data_root,
            )?;
        let db_path = product_app_data_root.join(M4_ORDINARY_SECRETARY_RELATIVE_PATH);
        let repository = M4SecretarySqliteRepository::open_ordinary_product(
            &M4OrdinarySecretaryRepositoryConfig {
                app_data_root: product_app_data_root,
                db_path: db_path.clone(),
            },
        )
        .map_err(|error| error.code)?;
        #[cfg(test)]
        repository
            .set_test_server_utc_now(M4C09_FIXED_TEST_NOW)
            .map_err(|error| error.code)?;

        let receipt = exercise_runtime(
            &m3_read_runtime,
            &repository,
            &db_path,
            &profile_fingerprint,
            previous.as_ref(),
        )?;
        write_secure_receipt(&receipt_path, &receipt)?;
        Ok((
            Self {
                root,
                receipt,
                m3_read_runtime: m3_read_runtime.clone(),
                repository: repository.clone(),
            },
            m3_read_runtime,
            repository,
        ))
    }
}

pub(crate) fn install_for_validated_profile(
    paths: &RuntimePaths,
) -> Result<Option<M4C09InstalledRuntime>, String> {
    if !explicit_mode_enabled() {
        return Ok(None);
    }
    if crate::m3_acceptance::explicit_mode_enabled() || m2_reference_slice_environment_present() {
        return Err(M4C09_MODE_CONFLICT.to_string());
    }
    let (candidate, m3_read_runtime, repository) = M4C09AcceptanceRuntime::open(paths)?;
    let mut slot = process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?;
    match slot.as_ref() {
        Some(existing) if existing.root == candidate.root => Ok(Some(M4C09InstalledRuntime {
            m3_read_runtime,
            repository,
        })),
        Some(_) => Err("m4c09_runtime_profile_changed_in_process".to_string()),
        None => {
            *slot = Some(candidate);
            Ok(Some(M4C09InstalledRuntime {
                m3_read_runtime,
                repository,
            }))
        }
    }
}

pub(crate) fn load_acceptance_status() -> Result<M4C09AcceptanceStatusDto, String> {
    process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .as_ref()
        .map(|runtime| runtime.receipt.clone())
        .ok_or_else(|| "m4c09_acceptance_runtime_unavailable".to_string())
}

fn active_runtime() -> Result<M4C09AcceptanceRuntime, String> {
    process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .clone()
        .ok_or_else(|| "m4c09_acceptance_runtime_unavailable".to_string())
}

/// C09 swaps only the registered command wrapper.  Outside an installed C09
/// runtime it delegates byte-for-byte to the ordinary product bridge.
#[tauri::command(rename = "load_secretary_home_context")]
pub(crate) async fn m4c09_load_secretary_home_context(
    state: tauri::State<'_, crate::AppState>,
) -> Result<crate::secretary_agent::SecretaryHomeContextEnvelope, String> {
    if process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .is_none()
    {
        return crate::secretary_agent::load_secretary_home_context(state).await;
    }
    let runtime = active_runtime()?;
    Ok(tauri::async_runtime::spawn_blocking(move || {
        read_application_outcome(&runtime.m3_read_runtime, &runtime.repository)
            .map(
                |application_outcome| crate::secretary_agent::SecretaryHomeContextEnvelope {
                    status: "ready".to_string(),
                    application_outcome: Some(application_outcome),
                    reason: None,
                },
            )
            .unwrap_or_else(|_| crate::secretary_agent::SecretaryHomeContextEnvelope {
                status: "unavailable".to_string(),
                application_outcome: None,
                reason: Some("秘书上下文暂不可用，请稍后重试。".to_string()),
            })
    })
    .await
    .unwrap_or_else(|_| crate::secretary_agent::SecretaryHomeContextEnvelope {
        status: "unavailable".to_string(),
        application_outcome: None,
        reason: Some("秘书上下文暂不可用，请稍后重试。".to_string()),
    }))
}

#[tauri::command(rename = "run_secretary_explain")]
pub(crate) async fn m4c09_run_secretary_explain(
    state: tauri::State<'_, crate::AppState>,
) -> Result<crate::secretary_agent::SecretaryExplainOutcome, String> {
    if process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .is_none()
    {
        return crate::secretary_agent::run_secretary_explain(state).await;
    }
    let runtime = active_runtime()?;
    Ok(tauri::async_runtime::spawn_blocking(move || {
        read_application_outcome(&runtime.m3_read_runtime, &runtime.repository)
            .map(|application_outcome| {
                let brief = &application_outcome.deterministic_brief;
                let attention_count = brief.attention_items.len();
                let personal_action_count = brief.personal_actions.len();
                crate::secretary_agent::SecretaryExplainOutcome {
                    status: "ready".to_string(),
                    explanation: Some(if attention_count == 0 && personal_action_count == 0 {
                        "当前没有来源关注事项，也没有独立个人待办。".to_string()
                    } else {
                        format!(
                            "当前有 {attention_count} 项来源关注和 {personal_action_count} 项独立个人待办，已按服务端协调快照机械整理。"
                        )
                    }),
                    reason: None,
                    context_ref: Some(application_outcome.context.context_ref.as_str().to_string()),
                    brief_ref: Some(brief.brief_ref.as_str().to_string()),
                    scope_source_watermark: Some(
                        brief.scope_source_watermark.as_str().to_string(),
                    ),
                    attention_count,
                    personal_action_count,
                }
            })
            .unwrap_or_else(|_| unavailable_explain())
    })
    .await
    .unwrap_or_else(|_| unavailable_explain()))
}

#[tauri::command(rename = "operate_secretary_coordination")]
pub(crate) async fn m4c09_operate_secretary_coordination(
    state: tauri::State<'_, crate::AppState>,
    request: crate::secretary_agent::SecretaryCoordinationRequest,
) -> Result<crate::secretary_agent::SecretaryCoordinationReceipt, String> {
    if process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .is_none()
    {
        return crate::secretary_agent::operate_secretary_coordination(state, request).await;
    }
    let runtime = active_runtime()?;
    tauri::async_runtime::spawn_blocking(move || {
        read_application_outcome(&runtime.m3_read_runtime, &runtime.repository)
            .map_err(|_| "M4C06_COORDINATION_UNAVAILABLE".to_string())?;
        operate_coordination(&runtime.repository, request)
    })
    .await
    .map_err(|_| "M4C06_COORDINATION_UNAVAILABLE".to_string())?
}

fn read_application_outcome(
    role_runtime: &M3RoleSessionReadRuntimeSlot,
    repository: &M4SecretarySqliteRepository,
) -> Result<crate::m4_secretary_service::M4SecretaryApplicationOutcome, String> {
    let role_port = M4C09RoleSessionPort {
        runtime: role_runtime,
    };
    let coordination_port = M4C09CoordinationPort { repository };
    let handoff_port = M4C09UnavailableHandoffPort;
    let model_port = M4C09ReadOnlyModelPort;
    M4SecretaryApplicationService::new(
        &role_port,
        &coordination_port,
        &handoff_port,
        repository,
        &model_port,
    )
    .read_deterministic_brief()
    .map_err(|error| error.code)
}

fn unavailable_explain() -> crate::secretary_agent::SecretaryExplainOutcome {
    crate::secretary_agent::SecretaryExplainOutcome {
        status: "unavailable".to_string(),
        explanation: None,
        reason: Some("秘书上下文暂不可用，请稍后重试。".to_string()),
        context_ref: None,
        brief_ref: None,
        scope_source_watermark: None,
        attention_count: 0,
        personal_action_count: 0,
    }
}

fn operate_coordination(
    repository: &M4SecretarySqliteRepository,
    request: crate::secretary_agent::SecretaryCoordinationRequest,
) -> Result<crate::secretary_agent::SecretaryCoordinationReceipt, String> {
    use crate::secretary_agent::SecretaryCoordinationAction;

    let item_ref = M4SecretaryTypedRef::new(request.item_ref)
        .map_err(|_| "M4C06_COORDINATION_REQUEST_INVALID".to_string())?;
    let expected_revision = parse_canonical_revision(&request.expected_revision)?;
    let idempotency_key = M4SecretaryOpaqueRef::new(request.idempotency_key)
        .map_err(|_| "M4C06_COORDINATION_REQUEST_INVALID".to_string())?;
    let is_inbox = matches!(
        request.action,
        SecretaryCoordinationAction::InboxMarkRead | SecretaryCoordinationAction::InboxDismiss
    );
    if (is_inbox && !item_ref.as_str().starts_with("inbox:"))
        || (!is_inbox && !item_ref.as_str().starts_with("open-loop:"))
    {
        return Err("M4C06_COORDINATION_REQUEST_INVALID".to_string());
    }
    let outcome = match request.action {
        SecretaryCoordinationAction::InboxMarkRead => repository.mark_inbox_item_read(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::InboxDismiss => repository.dismiss_inbox_item(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopAcknowledge => repository.acknowledge_open_loop(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopSnooze => repository.snooze_open_loop(
            item_ref.as_str(),
            expected_revision,
            request
                .snoozed_until_utc
                .as_deref()
                .ok_or_else(|| "M4C06_COORDINATION_REQUEST_INVALID".to_string())?,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopClose => repository.close_open_loop(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopDismiss => repository.dismiss_open_loop(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopReopen => repository.reopen_open_loop(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
        SecretaryCoordinationAction::OpenLoopCarryOver => repository.carry_over_open_loop(
            item_ref.as_str(),
            expected_revision,
            idempotency_key.as_str(),
        ),
    }
    .map_err(|_| "M4C06_COORDINATION_OPERATION_FAILED".to_string())?;
    Ok(crate::secretary_agent::SecretaryCoordinationReceipt {
        command_receipt_ref: M4SecretaryOpaqueRef::new(outcome.command_receipt_id)
            .map_err(|_| "M4C06_COORDINATION_OPERATION_FAILED".to_string())?,
        coordination_event_ref: M4SecretaryOpaqueRef::new(outcome.coordination_event_id)
            .map_err(|_| "M4C06_COORDINATION_OPERATION_FAILED".to_string())?,
        aggregate_kind_code: outcome.aggregate_kind,
        item_ref: M4SecretaryTypedRef::new(outcome.aggregate_id)
            .map_err(|_| "M4C06_COORDINATION_OPERATION_FAILED".to_string())?,
        coordination_revision: outcome.aggregate_revision,
        outcome_code: outcome.outcome_code,
        replayed: outcome.replayed,
    })
}

fn parse_canonical_revision(value: &str) -> Result<u64, String> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("M4C06_COORDINATION_REQUEST_INVALID".to_string());
    }
    value
        .parse::<u64>()
        .map_err(|_| "M4C06_COORDINATION_REQUEST_INVALID".to_string())
}

pub(crate) fn explicit_mode_enabled() -> bool {
    matches!(
        std::env::var(M4C09_MODE_ENV).as_deref(),
        Ok(M4C09_MODE_VALUE)
    )
}

pub(crate) fn reject_unapproved_tauri_command(command: &str) -> Result<(), String> {
    let active = process_runtime_slot()
        .lock()
        .map_err(|_| "m4c09_runtime_lock_poisoned".to_string())?
        .is_some();
    reject_tauri_command_for_runtime(command, active)
}

fn reject_tauri_command_for_runtime(command: &str, active: bool) -> Result<(), String> {
    if !active || M4C09_ALLOWED_TAURI_COMMANDS.contains(&command) {
        Ok(())
    } else {
        Err(M4C09_ISOLATED_IPC_BLOCKED.to_string())
    }
}

fn process_runtime_slot() -> &'static Mutex<Option<M4C09AcceptanceRuntime>> {
    static SLOT: OnceLock<Mutex<Option<M4C09AcceptanceRuntime>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn exercise_runtime(
    role_runtime: &M3RoleSessionReadRuntimeSlot,
    repository: &M4SecretarySqliteRepository,
    db_path: &Path,
    profile_fingerprint: &str,
    previous: Option<&M4C09AcceptanceStatusDto>,
) -> Result<M4C09AcceptanceStatusDto, String> {
    let launch_count = previous.map_or(1, |receipt| receipt.launch_count + 1);
    let role_status = role_runtime.secretary_status()?;
    let role_session_hash = hash_texts(&[
        &role_status.role_session_id,
        &role_status.role_ref,
        &role_status.scope_ref,
        &role_status.current_object_ref,
        &role_status.permission_snapshot_ref,
        &role_status.owner_fingerprint,
    ]);
    let same_role_session_recovered =
        previous.is_some_and(|receipt| receipt.role_session_hash == role_session_hash);

    let fake_model = M4C09FailingModel::default();
    let role_port = M4C09RoleSessionPort {
        runtime: role_runtime,
    };
    let coordination_port = M4C09CoordinationPort { repository };
    let handoff_port = M4C09UnavailableHandoffPort;
    let service = M4SecretaryApplicationService::new(
        &role_port,
        &coordination_port,
        &handoff_port,
        repository,
        &fake_model,
    );

    let mut empty_run_zero_material_events =
        previous.is_some_and(|receipt| receipt.daily.empty_run_zero_material_events);
    let mut empty_run_zero_agent_turns =
        previous.is_some_and(|receipt| receipt.daily.empty_run_zero_agent_turns);
    let mut empty_run_zero_model_invocations =
        previous.is_some_and(|receipt| receipt.daily.empty_run_zero_model_invocations);
    let mut zero_item_read_model_calls =
        previous.is_some_and(|receipt| receipt.model.zero_item_read_model_calls);

    if previous.is_none() {
        let empty = service
            .process(&M4SecretaryServiceTrigger::ReadOnlyQuery)
            .map_err(|error| error.code)?;
        zero_item_read_model_calls = empty.deterministic_brief.attention_items.is_empty()
            && empty.deterministic_brief.personal_actions.is_empty()
            && fake_model.calls.get() == 0
            && count_table_rows(db_path, "m4_model_invocations")? == 0;
        repository
            .run_daily_scheduler_cycle(M4SchedulerTrigger::StartupRecovery)
            .map_err(|error| error.code)?;
        let empty_daily = repository
            .refresh_and_read_daily_report()
            .map_err(|error| error.code)?;
        let (_, _, last_run) = ready_daily_identity(&empty_daily)?;
        empty_run_zero_material_events = last_run.0 == 0;
        empty_run_zero_agent_turns = last_run.1 == 0;
        empty_run_zero_model_invocations = last_run.2 == 0;
    }
    if !(empty_run_zero_material_events
        && empty_run_zero_agent_turns
        && empty_run_zero_model_invocations
        && zero_item_read_model_calls)
    {
        return Err("m4c09_empty_run_boundary_failed".to_string());
    }

    let alpha = synthetic_source("synthetic_owner_alpha", "synthetic-work-item-alpha", 1)?;
    let beta = synthetic_source("synthetic_owner_beta", "synthetic-work-item-beta", 1)?;
    let alpha_first = repository
        .ingest_workflow_attention_source(&alpha)
        .map_err(|error| error.code)?;
    let alpha_duplicate = repository
        .ingest_workflow_attention_source(&alpha)
        .map_err(|error| error.code)?;
    let beta_first = repository
        .ingest_workflow_attention_source(&beta)
        .map_err(|error| error.code)?;
    let first_launch_fresh_admissions = if previous.is_none() {
        !alpha_first.replayed && !beta_first.replayed
    } else {
        previous.is_some_and(|receipt| receipt.ingestion.first_launch_fresh_admissions)
    };
    let restart_seed_replayed = previous.is_some() && alpha_first.replayed && beta_first.replayed;
    if !alpha_duplicate.replayed
        || (previous.is_none() && !first_launch_fresh_admissions)
        || (previous.is_some() && !restart_seed_replayed)
    {
        return Err("m4c09_ingestion_replay_boundary_failed".to_string());
    }

    let first_daily = repository
        .refresh_and_read_daily_report()
        .map_err(|error| error.code)?;
    let first_daily_counts = daily_table_counts(db_path)?;
    let second_daily = repository
        .refresh_and_read_daily_report()
        .map_err(|error| error.code)?;
    let second_daily_counts = daily_table_counts(db_path)?;
    let (daily_report_id, report_version, _) = ready_daily_identity(&first_daily)?;
    let (second_report_id, second_report_version, _) = ready_daily_identity(&second_daily)?;
    let repeated_refresh_stable = daily_report_id == second_report_id
        && report_version == second_report_version
        && first_daily_counts == second_daily_counts;
    if !repeated_refresh_stable {
        return Err("m4c09_daily_refresh_not_idempotent".to_string());
    }

    let mut deterministic_brief_unchanged_after_failure =
        previous.is_some_and(|receipt| receipt.model.deterministic_brief_unchanged_after_failure);
    let mut first_failure_recorded =
        previous.is_some_and(|receipt| receipt.model.first_failure_recorded);
    let mut exact_failure_replay_recorded =
        previous.is_some_and(|receipt| receipt.model.exact_failure_replay_recorded);
    let mut terminal_failure_recovered_after_restart = false;

    if previous.is_none() {
        let before = service
            .read_deterministic_brief()
            .map_err(|error| error.code)?;
        let trigger = synthetic_model_trigger()?;
        let failed = service
            .process(&M4SecretaryServiceTrigger::ExplicitUserMessage(
                trigger.clone(),
            ))
            .map_err(|error| error.code)?;
        let replayed = service
            .process(&M4SecretaryServiceTrigger::ExplicitUserMessage(trigger))
            .map_err(|error| error.code)?;
        let after = service
            .read_deterministic_brief()
            .map_err(|error| error.code)?;
        first_failure_recorded = failed.model_enhancement.as_ref().is_some_and(|outcome| {
            outcome.status == M4SecretaryModelEnhancementStatus::Failed
                && outcome.recovery_code.as_deref() == Some(M4C09_FAKE_MODEL_FAILURE)
        });
        exact_failure_replay_recorded = replayed
            .model_enhancement
            .as_ref()
            .is_some_and(|outcome| outcome.status == M4SecretaryModelEnhancementStatus::Replayed);
        deterministic_brief_unchanged_after_failure =
            before.deterministic_brief.brief_hash == after.deterministic_brief.brief_hash;
    } else {
        terminal_failure_recovered_after_restart =
            count_terminal_fake_failures(db_path)? == 1 && fake_model.calls.get() == 0;
    }
    let durable_invocation_rows = count_table_rows(db_path, "m4_model_invocations")?;
    let fake_adapter_calls_this_launch = fake_model.calls.get();
    let fake_adapter_calls_total = previous.map_or(fake_adapter_calls_this_launch, |receipt| {
        receipt.model.fake_adapter_calls_total + fake_adapter_calls_this_launch
    });
    if !first_failure_recorded
        || !exact_failure_replay_recorded
        || !deterministic_brief_unchanged_after_failure
        || durable_invocation_rows != 1
        || fake_adapter_calls_total != 1
        || (previous.is_some() && !terminal_failure_recovered_after_restart)
    {
        return Err("m4c09_fake_model_boundary_failed".to_string());
    }

    let snapshot = repository
        .read_coordination_snapshot(m4_primary_scope_ref())
        .map_err(|error| error.code)?;
    let owners = owner_states(&snapshot)?;
    let table_counts = acceptance_table_counts(db_path)?;
    if owners.len() != 2 || table_counts != (2, 2, 2) {
        return Err("m4c09_two_owner_projection_failed".to_string());
    }
    let carried_over_receipt_rows = count_carry_over_receipts(db_path)?;
    let lifecycle = M4C09LifecycleProofDto {
        acknowledged_state_recovered: previous.is_some()
            && owners.iter().any(|owner| {
                owner.owner_code == "OWNER_ALPHA" && owner.open_loop_status == "ACKNOWLEDGED"
            }),
        snoozed_state_recovered: previous.is_some()
            && owners.iter().any(|owner| {
                owner.owner_code == "OWNER_BETA" && owner.open_loop_status == "SNOOZED"
            }),
        carried_over_receipt_recovered: previous.is_some() && carried_over_receipt_rows == 1,
        carried_over_receipt_rows,
    };

    let receipt = M4C09AcceptanceStatusDto {
        schema_version: M4C09_RUNTIME_SCHEMA_VERSION.to_string(),
        evidence_level: "ISOLATED_PRODUCT_APP".to_string(),
        launch_count,
        role_session_hash,
        same_role_session_recovered,
        owners,
        ingestion: M4C09IngestionProofDto {
            two_fixed_source_owners: true,
            first_launch_fresh_admissions,
            exact_duplicate_replayed: true,
            restart_seed_replayed,
            admitted_source_event_rows: table_counts.0,
            inbox_rows: table_counts.1,
            open_loop_rows: table_counts.2,
        },
        lifecycle,
        daily: M4C09DailyProofDto {
            empty_run_zero_material_events,
            empty_run_zero_agent_turns,
            empty_run_zero_model_invocations,
            repeated_refresh_stable,
            daily_report_ref_hash: hash_text(&daily_report_id),
            report_version,
            daily_report_rows: second_daily_counts.0,
            scheduler_run_rows: second_daily_counts.1,
        },
        model: M4C09ModelProofDto {
            fake_model_only: true,
            zero_item_read_model_calls,
            deterministic_brief_unchanged_after_failure,
            first_failure_recorded,
            exact_failure_replay_recorded,
            terminal_failure_recovered_after_restart,
            fake_adapter_calls_this_launch,
            fake_adapter_calls_total,
            durable_invocation_rows,
            real_model_attempts: 0,
        },
        isolation: M4C09IsolationProofDto {
            validated_profile_required: true,
            ordinary_product_runtime_used: true,
            synthetic_fixture_only: true,
            profile_fingerprint: profile_fingerprint.to_string(),
            real_provider_attempts: 0,
            external_connector_attempts: 0,
            external_network_writes: 0,
            real_codex_message_attempts: 0,
        },
        evidence_limit: "MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE".to_string(),
    };
    validate_receipt(&receipt)?;
    Ok(receipt)
}

struct M4C09RoleSessionPort<'a> {
    runtime: &'a M3RoleSessionReadRuntimeSlot,
}

impl M4SecretaryRoleSessionReadPort for M4C09RoleSessionPort<'_> {
    fn read_personal_secretary_role_session(
        &self,
    ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
        let status = self
            .runtime
            .secretary_status()
            .map_err(|_| M4SecretaryServiceError::new("M4C09_ROLE_SESSION_UNAVAILABLE"))?;
        let identity = crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity()
            .map_err(|_| M4SecretaryServiceError::new("M4C09_ROLE_SESSION_INVALID"))?;
        let binding = identity
            .m3_server_resolved_binding()
            .map_err(|_| M4SecretaryServiceError::new("M4C09_ROLE_SESSION_INVALID"))?;
        if status.actor_id != binding.actor_id.as_str()
            || status.role_ref != binding.role_ref.as_str()
            || status.scope_ref != binding.scope_ref.as_str()
            || status.current_object_ref != binding.current_object_ref.as_str()
            || status.execution_channel != binding.execution_channel.as_str()
            || status.permission_snapshot_ref != binding.permission_snapshot_ref.as_str()
            || status.owner_fingerprint != binding.owner_fingerprint.as_str()
        {
            return Err(M4SecretaryServiceError::new(
                "M4C09_ROLE_SESSION_BINDING_MISMATCH",
            ));
        }
        let invalid = || M4SecretaryServiceError::new("M4C09_ROLE_SESSION_INVALID");
        Ok(M4SecretaryRoleSessionState {
            role_session_ref: M4SecretaryOpaqueRef::new(status.role_session_id)
                .map_err(|_| invalid())?,
            role_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID,
            )
            .map_err(|_| invalid())?,
            scope_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID,
            )
            .map_err(|_| invalid())?,
            current_object_ref: M4SecretaryTypedRef::new(
                crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID,
            )
            .map_err(|_| invalid())?,
            execution_channel_code: "DAILY".to_string(),
            session_state_code: status.session_state,
            permission_snapshot_ref: M4SecretaryOpaqueRef::new(status.permission_snapshot_ref)
                .map_err(|_| invalid())?,
            owner_fingerprint: M4SecretaryHash::new(status.owner_fingerprint)
                .map_err(|_| invalid())?,
        })
    }
}

struct M4C09CoordinationPort<'a> {
    repository: &'a M4SecretarySqliteRepository,
}

impl M4SecretaryCoordinationSnapshotReadPort for M4C09CoordinationPort<'_> {
    fn read_coordination_snapshot(
        &self,
        scope_ref: &M4SecretaryTypedRef,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
        self.repository
            .read_coordination_snapshot(scope_ref.as_str())
            .map_err(|_| M4SecretaryServiceError::new("M4C09_COORDINATION_UNAVAILABLE"))
    }
}

#[derive(Clone, Copy)]
struct M4C09UnavailableHandoffPort;

impl M4SecretaryHandoffPort for M4C09UnavailableHandoffPort {
    fn create_handoff(
        &self,
        _request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: "M4C09_HANDOFF_UNAVAILABLE".to_string(),
        })
    }

    fn read_handoff_receipt(
        &self,
        _handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: "M4C09_HANDOFF_UNAVAILABLE".to_string(),
        })
    }
}

#[derive(Default)]
struct M4C09FailingModel {
    calls: Cell<u64>,
}

impl M4SecretaryControlledModelEnhancementPort for M4C09FailingModel {
    fn enhance(
        &self,
        _request: &M4SecretaryModelEnhancementRequest,
    ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
        self.calls.set(self.calls.get() + 1);
        Ok(M4SecretaryModelPortOutcome::Failed {
            error_code: M4C09_FAKE_MODEL_FAILURE.to_string(),
        })
    }
}

#[derive(Clone, Copy)]
struct M4C09ReadOnlyModelPort;

impl M4SecretaryControlledModelEnhancementPort for M4C09ReadOnlyModelPort {
    fn enhance(
        &self,
        _request: &M4SecretaryModelEnhancementRequest,
    ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(
            "M4C09_READ_ONLY_MODEL_PORT_UNAVAILABLE",
        ))
    }
}

fn synthetic_source(
    owner: &str,
    object_id: &str,
    revision: u64,
) -> Result<M4WorkflowAttentionSourceInput, String> {
    let material = format!("{owner}:{object_id}:{revision}");
    let (status, signals) = if owner.ends_with("alpha") {
        (
            "WAITING_USER",
            M4AttentionSignals {
                requires_user_decision: true,
                attention_required: true,
                material_change: true,
                ..Default::default()
            },
        )
    } else {
        (
            "BLOCKED",
            M4AttentionSignals {
                source_blocked: true,
                attention_required: true,
                material_change: true,
                ..Default::default()
            },
        )
    };
    Ok(M4WorkflowAttentionSourceInput {
        source_owner_ref: owner.to_string(),
        scope_ref: m4_primary_scope_ref().to_string(),
        source_type: M4_WORKFLOW_ATTENTION_SOURCE_TYPE.to_string(),
        canonical_source_object_id: object_id.to_string(),
        source_revision: revision,
        source_event_id: opaque_ref("source-event", &material)?,
        source_owner_watermark: opaque_ref("owner-watermark", &material)?,
        occurred_at_utc: "2026-08-10T16:00:00Z".to_string(),
        source_link: M4SourceLinkInput {
            link_kind: "INTERNAL_ROUTE".to_string(),
            source_owner_ref: owner.to_string(),
            object_type: M4_WORKFLOW_ATTENTION_OBJECT_TYPE.to_string(),
            canonical_source_object_id: object_id.to_string(),
            expected_source_revision: revision,
            opaque_route_ref: opaque_ref("source-route", &material)?,
        },
        owner_status_code: status.to_string(),
        attention_signals: signals,
        due_at_utc: Some("2099-12-31T23:00:00Z".to_string()),
        sensitivity: M4_SCRUBBED_SENSITIVITY.to_string(),
        scrubbed_summary_ref: opaque_ref("source-summary", &material)?,
        payload_hash: hash_text(&format!("m4c09-payload:{material}:{status}")),
    })
}

fn synthetic_model_trigger() -> Result<M4SecretaryExplicitUserMessageTrigger, String> {
    Ok(M4SecretaryExplicitUserMessageTrigger {
        trigger_ref: M4SecretaryOpaqueRef::new(opaque_ref("trigger", "fake-failure")?)
            .map_err(|error| error.code)?,
        user_message_ref: M4SecretaryOpaqueRef::new(opaque_ref("message", "fake-failure")?)
            .map_err(|error| error.code)?,
        user_message_hash: M4SecretaryHash::new(hash_text("m4c09-synthetic-message"))
            .map_err(|error| error.code)?,
        idempotency_key_ref: M4SecretaryOpaqueRef::new(opaque_ref("idempotency", "fake-failure")?)
            .map_err(|error| error.code)?,
        purpose_code: "M4C09_FAKE_FAILURE".to_string(),
    })
}

fn owner_states(snapshot: &M4CoordinationSnapshot) -> Result<Vec<M4C09OwnerStateDto>, String> {
    let mut owners = Vec::new();
    for (owner_ref, owner_code, object_id, object_code) in [
        (
            "synthetic_owner_alpha",
            "OWNER_ALPHA",
            "synthetic-work-item-alpha",
            "OBJECT_ALPHA",
        ),
        (
            "synthetic_owner_beta",
            "OWNER_BETA",
            "synthetic-work-item-beta",
            "OBJECT_BETA",
        ),
    ] {
        let inbox = snapshot
            .inbox_items
            .iter()
            .find(|item| {
                item.source_owner_ref == owner_ref
                    && item.source_link.canonical_source_object_id == object_id
            })
            .ok_or_else(|| "m4c09_owner_inbox_missing".to_string())?;
        let open_loop = snapshot
            .open_loops
            .iter()
            .find(|item| {
                item.source_owner_ref == owner_ref
                    && item.source_link.canonical_source_object_id == object_id
            })
            .ok_or_else(|| "m4c09_owner_open_loop_missing".to_string())?;
        owners.push(M4C09OwnerStateDto {
            owner_code: owner_code.to_string(),
            object_code: object_code.to_string(),
            inbox_status: inbox.status.clone(),
            open_loop_status: open_loop.status.clone(),
            open_loop_revision: open_loop.revision.to_string(),
            item_ref_hash: hash_text(&open_loop.open_loop_id),
            route_ref_hash: hash_text(&open_loop.source_link.opaque_route_ref),
        });
    }
    Ok(owners)
}

fn ready_daily_identity(
    envelope: &M4SecretaryDailyReportEnvelope,
) -> Result<(String, String, (u64, u64, u64)), String> {
    match envelope {
        M4SecretaryDailyReportEnvelope::Ready {
            daily_report,
            last_run,
            ..
        } => Ok((
            daily_report.daily_report_id.clone(),
            daily_report.report_version.clone(),
            (
                last_run.admitted_material_event_count,
                last_run.agent_turn_count,
                last_run.model_invocation_count,
            ),
        )),
        M4SecretaryDailyReportEnvelope::Unavailable { reason, .. }
        | M4SecretaryDailyReportEnvelope::Disabled { reason, .. } => {
            Err(format!("m4c09_daily_not_ready:{reason}"))
        }
    }
}

fn acceptance_table_counts(db_path: &Path) -> Result<(u64, u64, u64), String> {
    Ok((
        count_table_rows(db_path, "m4_admitted_source_events")?,
        count_table_rows(db_path, "m4_inbox_items")?,
        count_table_rows(db_path, "m4_open_loops")?,
    ))
}

fn daily_table_counts(db_path: &Path) -> Result<(u64, u64), String> {
    Ok((
        count_table_rows(db_path, "m4_daily_reports")?,
        count_table_rows(db_path, "m4_scheduler_runs")?,
    ))
}

fn count_table_rows(db_path: &Path, table: &str) -> Result<u64, String> {
    const ALLOWED: &[&str] = &[
        "m4_admitted_source_events",
        "m4_inbox_items",
        "m4_open_loops",
        "m4_daily_reports",
        "m4_scheduler_runs",
        "m4_model_invocations",
    ];
    if !ALLOWED.contains(&table) {
        return Err("m4c09_table_count_not_allowlisted".to_string());
    }
    let connection = open_read_only_database(db_path)?;
    let count: i64 = connection
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .map_err(|_| "m4c09_table_count_failed".to_string())?;
    u64::try_from(count).map_err(|_| "m4c09_table_count_invalid".to_string())
}

fn count_terminal_fake_failures(db_path: &Path) -> Result<u64, String> {
    let connection = open_read_only_database(db_path)?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_model_invocations
             WHERE status = 'FAILED' AND outcome_code = ?1",
            [M4C09_FAKE_MODEL_FAILURE],
            |row| row.get(0),
        )
        .map_err(|_| "m4c09_terminal_model_count_failed".to_string())?;
    u64::try_from(count).map_err(|_| "m4c09_terminal_model_count_invalid".to_string())
}

fn count_carry_over_receipts(db_path: &Path) -> Result<u64, String> {
    let connection = open_read_only_database(db_path)?;
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM m4_coordination_command_receipts
             WHERE command_kind = 'OPEN_LOOP_CARRY_OVER' AND outcome_code = 'CARRIED_OVER'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "m4c09_carry_over_receipt_count_failed".to_string())?;
    u64::try_from(count).map_err(|_| "m4c09_carry_over_receipt_count_invalid".to_string())
}

fn open_read_only_database(db_path: &Path) -> Result<Connection, String> {
    let metadata = fs::symlink_metadata(db_path)
        .map_err(|_| "m4c09_database_metadata_unavailable".to_string())?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.nlink() != 1
    {
        return Err("m4c09_database_path_not_regular".to_string());
    }
    Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(|_| "m4c09_database_read_open_failed".to_string())
}

fn validate_receipt(receipt: &M4C09AcceptanceStatusDto) -> Result<(), String> {
    if receipt.schema_version != M4C09_RUNTIME_SCHEMA_VERSION
        || receipt.evidence_level != "ISOLATED_PRODUCT_APP"
        || receipt.launch_count == 0
        || receipt.launch_count > M4C09_MAX_LAUNCHES
        || receipt.role_session_hash.len() != 64
        || receipt.owners.len() != 2
        || !receipt.ingestion.two_fixed_source_owners
        || !receipt.ingestion.first_launch_fresh_admissions
        || !receipt.ingestion.exact_duplicate_replayed
        || receipt.ingestion.admitted_source_event_rows != 2
        || receipt.ingestion.inbox_rows != 2
        || receipt.ingestion.open_loop_rows != 2
        || !receipt.daily.empty_run_zero_material_events
        || !receipt.daily.empty_run_zero_agent_turns
        || !receipt.daily.empty_run_zero_model_invocations
        || !receipt.daily.repeated_refresh_stable
        || !receipt.model.fake_model_only
        || !receipt.model.zero_item_read_model_calls
        || !receipt.model.deterministic_brief_unchanged_after_failure
        || !receipt.model.first_failure_recorded
        || !receipt.model.exact_failure_replay_recorded
        || receipt.model.fake_adapter_calls_total != 1
        || receipt.model.durable_invocation_rows != 1
        || receipt.model.real_model_attempts != 0
        || !receipt.isolation.validated_profile_required
        || !receipt.isolation.ordinary_product_runtime_used
        || !receipt.isolation.synthetic_fixture_only
        || receipt.isolation.profile_fingerprint.len() != 64
        || receipt.isolation.real_provider_attempts != 0
        || receipt.isolation.external_connector_attempts != 0
        || receipt.isolation.external_network_writes != 0
        || receipt.isolation.real_codex_message_attempts != 0
        || receipt.evidence_limit != "MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE"
        || (receipt.launch_count == 1 && receipt.same_role_session_recovered)
        || (receipt.launch_count > 1
            && (!receipt.same_role_session_recovered
                || !receipt.ingestion.restart_seed_replayed
                || !receipt.lifecycle.acknowledged_state_recovered
                || !receipt.lifecycle.snoozed_state_recovered
                || !receipt.lifecycle.carried_over_receipt_recovered
                || receipt.lifecycle.carried_over_receipt_rows != 1
                || !receipt.model.terminal_failure_recovered_after_restart
                || receipt.model.fake_adapter_calls_this_launch != 0))
    {
        return Err("m4c09_receipt_semantic_validation_failed".to_string());
    }
    Ok(())
}

fn read_previous_receipt(path: &Path) -> Result<Option<M4C09AcceptanceStatusDto>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "m4c09_receipt_metadata_unavailable".to_string())?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.nlink() != 1
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.len() > M4C09_MAX_RECEIPT_BYTES
    {
        return Err("m4c09_receipt_file_invalid".to_string());
    }
    let bytes = fs::read(path).map_err(|_| "m4c09_receipt_read_failed".to_string())?;
    let receipt: M4C09AcceptanceStatusDto =
        serde_json::from_slice(&bytes).map_err(|_| "m4c09_receipt_decode_failed".to_string())?;
    validate_receipt(&receipt)?;
    Ok(Some(receipt))
}

fn write_secure_receipt(path: &Path, receipt: &M4C09AcceptanceStatusDto) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|_| "m4c09_receipt_encode_failed".to_string())?;
    if bytes.len() as u64 > M4C09_MAX_RECEIPT_BYTES {
        return Err("m4c09_receipt_too_large".to_string());
    }
    let temp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp_path)
        .map_err(|_| "m4c09_receipt_temp_create_failed".to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "m4c09_receipt_temp_write_failed".to_string())?;
    drop(file);
    fs::rename(&temp_path, path).map_err(|_| "m4c09_receipt_commit_failed".to_string())?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| "m4c09_receipt_permission_failed".to_string())?;
    Ok(())
}

fn canonical_profile_root(paths: &RuntimePaths) -> Result<PathBuf, String> {
    let root =
        fs::canonicalize(&paths.root).map_err(|_| "m4c09_profile_root_unavailable".to_string())?;
    let project_root = fs::canonicalize(&paths.project_root)
        .map_err(|_| "m4c09_profile_project_unavailable".to_string())?;
    if !project_root.starts_with(&root) {
        return Err("m4c09_profile_project_outside_root".to_string());
    }
    Ok(root)
}

fn canonical_contained_directory(root: &Path, path: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(path).map_err(|_| "m4c09_profile_directory_create_failed".to_string())?;
    let canonical =
        fs::canonicalize(path).map_err(|_| "m4c09_profile_directory_unavailable".to_string())?;
    if !canonical.starts_with(root) || canonical == root {
        return Err("m4c09_profile_directory_outside_root".to_string());
    }
    Ok(canonical)
}

fn m2_reference_slice_environment_present() -> bool {
    [
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_DRIVER_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_ATTEMPT_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_PHASE_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_NONCE_ENV,
        crate::m2_r4_reference_slice_driver::M2_R4_REFERENCE_SLICE_EXTERNAL_EFFECT_ENV,
    ]
    .into_iter()
    .any(|name| std::env::var_os(name).is_some())
}

fn opaque_ref(namespace: &str, material: &str) -> Result<String, String> {
    m4_internal_id(
        &format!("{namespace}:sha256:"),
        "syn.m4c09.isolated-fixture/v1",
        &[material],
    )
}

fn hash_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn hash_texts(values: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"syn.m4c09.receipt-hash/v1");
    for value in values {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        paths: RuntimePaths,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "syn-m4-acceptance-{label}-{}-{sequence}",
                std::process::id()
            ));
            let project_root = root.join("fixture/project");
            let app_data_root = root.join("app-data");
            let app_log_dir = root.join("logs");
            for path in [&project_root, &app_data_root, &app_log_dir] {
                fs::create_dir_all(path).expect("create M4C09 fixture directory");
            }
            let paths = RuntimePaths {
                root: root.clone(),
                index_path: root.join("fixture/codex-index.json"),
                tasks_path: root.join("fixture/tasks.md"),
                project_root,
                workflow_state_path: root.join("workflow-state/workflow-state.v0.json"),
                app_data_root,
                vault_root: root.join("vault"),
                recovery_backups_root: root.join("recovery"),
                canvas_root: root.join("canvas"),
                codex_db_path: root.join("codex-db/state.sqlite"),
                app_log_dir,
            };
            Self { root, paths }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn m4c09_first_launch_lifecycle_restart_and_fake_failure_are_durable() {
        let fixture = Fixture::new("restart");
        let (first, _, repository) =
            M4C09AcceptanceRuntime::open(&fixture.paths).expect("first C09 launch");
        assert_eq!(first.receipt.launch_count, 1);
        assert_eq!(first.receipt.model.fake_adapter_calls_total, 1);
        assert!(first
            .receipt
            .owners
            .iter()
            .all(|owner| owner.open_loop_status == "OPEN"));

        let snapshot = repository
            .read_coordination_snapshot(m4_primary_scope_ref())
            .expect("read first-launch C09 snapshot");
        let alpha = snapshot
            .open_loops
            .iter()
            .find(|item| item.source_owner_ref == "synthetic_owner_alpha")
            .expect("alpha open loop");
        let acknowledged = repository
            .acknowledge_open_loop(
                &alpha.open_loop_id,
                alpha.revision as u64,
                &opaque_ref("coordination", "alpha-ack").expect("alpha ack key"),
            )
            .expect("acknowledge alpha");
        repository
            .carry_over_open_loop(
                &alpha.open_loop_id,
                acknowledged
                    .aggregate_revision
                    .parse()
                    .expect("ack revision"),
                &opaque_ref("coordination", "alpha-carry").expect("alpha carry key"),
            )
            .expect("carry alpha");
        let beta = snapshot
            .open_loops
            .iter()
            .find(|item| item.source_owner_ref == "synthetic_owner_beta")
            .expect("beta open loop");
        repository
            .snooze_open_loop(
                &beta.open_loop_id,
                beta.revision as u64,
                "2099-12-31T22:00:00Z",
                &opaque_ref("coordination", "beta-snooze").expect("beta snooze key"),
            )
            .expect("snooze beta");
        drop(repository);
        drop(first);

        let (second, _, _) =
            M4C09AcceptanceRuntime::open(&fixture.paths).expect("second C09 launch");
        assert_eq!(second.receipt.launch_count, 2);
        assert!(second.receipt.same_role_session_recovered);
        assert!(second.receipt.ingestion.restart_seed_replayed);
        assert!(
            second
                .receipt
                .model
                .terminal_failure_recovered_after_restart
        );
        assert_eq!(second.receipt.model.fake_adapter_calls_this_launch, 0);
        assert_eq!(second.receipt.model.fake_adapter_calls_total, 1);
        assert_eq!(
            second
                .receipt
                .owners
                .iter()
                .find(|owner| owner.owner_code == "OWNER_ALPHA")
                .expect("alpha receipt")
                .open_loop_status,
            "ACKNOWLEDGED"
        );
        assert!(second.receipt.lifecycle.carried_over_receipt_recovered);
        assert_eq!(
            second
                .receipt
                .owners
                .iter()
                .find(|owner| owner.owner_code == "OWNER_BETA")
                .expect("beta receipt")
                .open_loop_status,
            "SNOOZED"
        );
    }

    #[test]
    fn m4c09_global_gate_is_fail_closed_only_when_runtime_is_active() {
        for allowed in M4C09_ALLOWED_TAURI_COMMANDS {
            assert!(reject_tauri_command_for_runtime(allowed, true).is_ok());
        }
        for blocked in [
            "start_agent_conversation_transport",
            "run_manual_codex_relay_once",
            "execute_workflow_node_dispatch",
            "recover_secretary_daily_catch_up",
            "knowledge_vault_write_note",
        ] {
            assert_eq!(
                reject_tauri_command_for_runtime(blocked, true)
                    .expect_err("C09 must reject effectful command"),
                M4C09_ISOLATED_IPC_BLOCKED
            );
            assert!(reject_tauri_command_for_runtime(blocked, false).is_ok());
        }
    }

    #[test]
    fn m4c09_receipt_is_scrubbed_and_contains_no_raw_fixture_identity_or_path() {
        let fixture = Fixture::new("redaction");
        let (runtime, _, _) =
            M4C09AcceptanceRuntime::open(&fixture.paths).expect("open redaction fixture");
        let json = serde_json::to_string(&runtime.receipt).expect("serialize C09 receipt");
        for forbidden in [
            fixture.root.to_string_lossy().as_ref(),
            "synthetic_owner_alpha",
            "synthetic_owner_beta",
            "synthetic-work-item-alpha",
            "synthetic-work-item-beta",
            "raw_prompt",
            "provider_body",
            "credential",
        ] {
            assert!(!json.contains(forbidden), "receipt leaked {forbidden}");
        }
    }
}
