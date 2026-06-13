use crate::utils::hash::{sha256_hex, short_hash12 as short_hash};
use crate::{
    codex_local_runner, h5_project_dispatch_bridge, runtime_log_store, session_continuation_store,
    CodexControlCommandInput, ConfirmControlledSessionContinuationInput,
    ConfirmRealExecutionProductCommandInput, ContinuationAuditImpact, ContinuationFailureBoundary,
    H2RealResumeAuthorizationMatrix, H3RealNewSessionAuthorizationMatrix,
    H5ProjectWorkflowDispatchPreview, H5ProjectWorkflowDispatchPreviewInput,
    PrepareRealExecutionProductCommandInput, PreviewRealExecutionProductCommandInput,
    ReadbackExpectation, RealExecutionProductCommandAttempt,
    RealExecutionProductCommandAuditPreview, RealExecutionProductCommandDecision,
    RealExecutionProductCommandDecisionOutput, RealExecutionProductCommandDiagnosticsSummary,
    RealExecutionProductCommandDuplicateScope, RealExecutionProductCommandFailureStopRetryItem,
    RealExecutionProductCommandFailureStopRetrySummary, RealExecutionProductCommandGuardPreview,
    RealExecutionProductCommandPermissionEnvelope, RealExecutionProductCommandPhaseAOutput,
    RealExecutionProductCommandPhaseBOutput, RealExecutionProductCommandPrepareOutput,
    RealExecutionProductCommandPreview, RealExecutionProductCommandReadModel,
    RealExecutionProductCommandReadbackBoundary, RealExecutionProductCommandReadiness,
    RealExecutionProductCommandRequest, RealExecutionProductCommandRuntimeLogPreview,
    RealExecutionProductCommandStore, RecordRealExecutionProductCommandDecisionInput,
    RunControlledSessionContinuationRealNewSessionH3BInput,
    RunControlledSessionContinuationRealResumePhaseAInput,
    RunControlledSessionContinuationRealResumePhaseBInput,
    RunRealExecutionProductCommandNewSessionPhaseBInput, RunRealExecutionProductCommandPhaseAInput,
    RunRealExecutionProductCommandPhaseBInput, SessionContinuationAttempt,
    SessionContinuationGuardResult, SessionContinuationPreview, SessionContinuationRequest,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const PRODUCT_COMMAND_STORE_SCHEMA_VERSION: &str = "real_execution_product_commands.v1";
const PRODUCT_COMMAND_SIDECAR_NAME: &str = "real-execution-product-commands.v1.json";
const K2_R1_EXECUTION_POINT_ID: &str = "stage-k-k2-r1-mario-test-resume-read-only";
const K2_R2_EXECUTION_POINT_ID: &str = "stage-k-k2-r2-mario-test-resume-workspace-write";
const K2_N1_EXECUTION_POINT_ID: &str = "stage-k-k2-n1-isolated-new-session-read-only";
const K2_N2_EXECUTION_POINT_ID: &str = "stage-k-k2-n2-isolated-new-session-workspace-write";
const K2_MARIO_PROJECT_ROOT: &str = "/Users/yoyi/Documents/mario test";
const K2_MARIO_PROJECT_ID: &str = "project:users-yoyi-documents-mario-test";
const K2_MARIO_WORKFLOW_ID: &str = "workflow:stage-k:k2:mario-test";
const K2_ISOLATED_PROJECT_ROOT: &str =
    "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project";
const K2_ISOLATED_PROJECT_ID: &str =
    "project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project";
const K2_ISOLATED_WORKFLOW_ID: &str = "workflow:stage-k:k2:isolated";
const K2_R1_SESSION_ID: &str = "019e798a-6ce5-76c3-b8ee-33bd0fda841f";
const K2_R2_SESSION_ID: &str = "019e798a-ac37-7771-b982-e38084fcd22e";
const K2_R2_ALLOWED_WRITE_ROOT: &str = "/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/";
const K2_R2_ALLOWED_WRITE_PATH: &str =
    "/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md";
const K2_N2_ALLOWED_WRITE_ROOT: &str = "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/";
const K2_N2_ALLOWED_WRITE_PATH: &str = "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md";
const K2_R1_PROMPT_HASH: &str = "2dc6d059fe5373ba547da91bd2b28296ab0ec15450cb7264f26243a3bff86e1d";
const K2_R2_PROMPT_HASH: &str = "03091a7bfc9e8a9b86bcc79f421f8b0ab982cd513cca7e1b8346afc709205c49";
const K2_N1_PROMPT_HASH: &str = "b19d41bf5e37cd41af5630cd71241f729e576ed4409574b10594c56e2d359833";
const K2_N2_PROMPT_HASH: &str = "3ff79f634ab4eaaf341878e62e0d8542d39b4c1d4f9cee67d69ba823849ebead";
const K2_R1_CANONICAL_PROMPT: &str = include_str!(
    "../../../../tmp/stage-k-k2-real-execution-prompts/k2-r1-mario-test-resume-read-only.txt"
);
const K2_R2_CANONICAL_PROMPT: &str = include_str!(
    "../../../../tmp/stage-k-k2-real-execution-prompts/k2-r2-mario-test-resume-workspace-write.txt"
);
const K2_N1_CANONICAL_PROMPT: &str = include_str!(
    "../../../../tmp/stage-k-k2-real-execution-prompts/k2-n1-isolated-new-session-read-only.txt"
);
const K2_N2_CANONICAL_PROMPT: &str = include_str!(
    "../../../../tmp/stage-k-k2-real-execution-prompts/k2-n2-isolated-new-session-workspace-write.txt"
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct K2ExecutionPointConfig {
    pub(crate) execution_point_id: String,
    pub(crate) operation: String,
    pub(crate) adapter_id: String,
    pub(crate) project_root: String,
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    pub(crate) run_unit_id: String,
    pub(crate) node_id: String,
    pub(crate) target_session_id: Option<String>,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) allowed_write_path: Option<String>,
    pub(crate) denied_paths: Vec<String>,
    pub(crate) prompt_summary: String,
    pub(crate) prompt_ref: String,
    pub(crate) prompt_hash: String,
    pub(crate) canonical_prompt: String,
    pub(crate) task_memory_packet_ref: String,
    pub(crate) permission_envelope_ref: String,
    pub(crate) readback_marker: String,
    pub(crate) readback_plan: String,
    pub(crate) runtime_log_policy: String,
    pub(crate) audit_policy: String,
    pub(crate) baseline_hashes: Vec<String>,
    pub(crate) codex_scope: String,
    pub(crate) dirty_worktree_policy: String,
    pub(crate) rollback_policy: String,
    pub(crate) user_confirmation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductCommandBoundarySpec {
    pub(crate) boundary_version: i64,
    pub(crate) command_name: String,
    pub(crate) command_family: String,
    pub(crate) boundary_kind: String,
    pub(crate) h5_unified_product_command: bool,
    pub(crate) deprecated: bool,
    pub(crate) product_routing_allows_real_execution: bool,
    pub(crate) legacy_path_may_have_real_side_effects: bool,
    pub(crate) replacement_command: Option<String>,
    pub(crate) reason: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RealExecutionCommandGateInput<'a> {
    pub(crate) command_name: &'a str,
    pub(crate) command_family: &'a str,
    pub(crate) operation_id: &'a str,
    pub(crate) h5_unified_product_command: bool,
    pub(crate) authorization_complete: bool,
    pub(crate) user_rejected: bool,
    pub(crate) duplicate_blocked: bool,
    pub(crate) guard_blocked: bool,
    pub(crate) diagnostics_blocked: bool,
    pub(crate) stale_memory_blocked: bool,
    pub(crate) readback_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RealExecutionCommandDecision {
    pub(crate) command_name: String,
    pub(crate) command_family: String,
    pub(crate) operation_id: String,
    pub(crate) status: String,
    pub(crate) runner_call_allowed: bool,
    pub(crate) product_routing_allows_real_execution: bool,
    pub(crate) reason: String,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn legacy_product_command_boundary_spec(
    command_name: &str,
) -> ProductCommandBoundarySpec {
    ProductCommandBoundarySpec {
        boundary_version: 1,
        command_name: command_name.to_string(),
        command_family: "legacy_workflow_real_execution".to_string(),
        boundary_kind: "deprecated_guarded_boundary".to_string(),
        h5_unified_product_command: false,
        deprecated: true,
        product_routing_allows_real_execution: false,
        legacy_path_may_have_real_side_effects: true,
        replacement_command: Some(
            "preview_h5_project_workflow_dispatch + controlled_session_continuation".to_string(),
        ),
        reason:
            "旧 workflow dispatch / workflow machine 不是 H5 unified product command；普通产品路径不得绕过 permission、continuation、runtime log、audit 和 readback 契约。"
                .to_string(),
        warnings: vec![
            "legacy_command_deprecated_not_h5_unified_product_command".to_string(),
            "real_execution_blocked_at_product_wrapper_level".to_string(),
            "use_h5_preview_and_continuation_routing_for_product_path".to_string(),
            "real_execution_command_gate_v1".to_string(),
        ],
    }
}

pub(crate) fn legacy_product_command_blocked_message(command_name: &str) -> String {
    let boundary = legacy_product_command_boundary_spec(command_name);
    format!(
        "legacy_product_command_blocked:{}:{}",
        boundary.command_name, boundary.reason
    )
}

pub(crate) fn mcp_canvas_real_execution_blocked_message(command_name: &str) -> String {
    format!(
        "mcp_canvas_real_execution_blocked:{command_name}:legacy experiment canvas run is sealed; use the H-stage unified product command boundary before any real Codex execution"
    )
}

pub(crate) fn decide_real_execution_command(
    input: RealExecutionCommandGateInput<'_>,
) -> RealExecutionCommandDecision {
    let (status, reason, runner_call_allowed) = if input.user_rejected {
        (
            "user_rejected",
            "user rejected explicit real execution decision",
            false,
        )
    } else if input.duplicate_blocked {
        (
            "duplicate_blocked",
            "duplicate active attempt blocks real runner call",
            false,
        )
    } else if input.diagnostics_blocked {
        (
            "blocked_by_diagnostics",
            "diagnostics degraded state blocks real runner call",
            false,
        )
    } else if input.stale_memory_blocked {
        (
            "blocked_stale_memory",
            "stale task memory packet blocks real runner call",
            false,
        )
    } else if input.guard_blocked {
        (
            "blocked_by_guard",
            "codex-local execution guard blocks real runner call",
            false,
        )
    } else if !input.readback_required {
        (
            "blocked_waiting_readback_plan",
            "readback plan is required before any real runner call",
            false,
        )
    } else if !input.authorization_complete {
        (
            "blocked_waiting_authorization",
            "permission envelope or authorization matrix is incomplete",
            false,
        )
    } else {
        (
            "authorized_for_real_runner",
            "unified product command gate allows the explicit real runner call",
            true,
        )
    };

    let mut warnings = vec![
        "real_execution_command_gate_v1".to_string(),
        format!("product_command:{}", input.command_name),
        format!("command_family:{}", input.command_family),
        format!("operation_id:{}", input.operation_id),
    ];
    if input.h5_unified_product_command {
        warnings.push("h5_unified_product_command_boundary".to_string());
    } else {
        warnings.push("not_h5_unified_product_command".to_string());
    }
    if input.readback_required {
        warnings.push("readback_required_by_product_gate".to_string());
    } else {
        warnings.push("readback_plan_missing_blocks_real_runner".to_string());
    }
    if runner_call_allowed {
        warnings.push("runner_call_allowed_after_unified_product_gate".to_string());
    } else {
        warnings.push("runner_call_blocked_by_unified_product_gate".to_string());
    }
    warnings.push(format!("product_gate_status:{status}"));
    warnings.sort();
    warnings.dedup();

    RealExecutionCommandDecision {
        command_name: input.command_name.to_string(),
        command_family: input.command_family.to_string(),
        operation_id: input.operation_id.to_string(),
        status: status.to_string(),
        runner_call_allowed,
        product_routing_allows_real_execution: runner_call_allowed,
        reason: reason.to_string(),
        warnings,
    }
}

pub(crate) fn k2_execution_point_config(
    execution_point_id: &str,
) -> Result<K2ExecutionPointConfig, String> {
    let config = match execution_point_id {
        K2_R1_EXECUTION_POINT_ID => K2ExecutionPointConfig {
            execution_point_id: K2_R1_EXECUTION_POINT_ID.to_string(),
            operation: "resume".to_string(),
            adapter_id: "codex-local".to_string(),
            project_root: K2_MARIO_PROJECT_ROOT.to_string(),
            project_id: K2_MARIO_PROJECT_ID.to_string(),
            workflow_id: K2_MARIO_WORKFLOW_ID.to_string(),
            run_unit_id: "run-unit:stage-k:k2:r1".to_string(),
            node_id: "node:stage-k:k2:r1".to_string(),
            target_session_id: Some(K2_R1_SESSION_ID.to_string()),
            sandbox: "read-only".to_string(),
            allowed_write_roots: Vec::new(),
            allowed_write_path: None,
            denied_paths: k2_denied_paths("unauthorized project files"),
            prompt_summary: "Stage K K2 resume/read-only health probe for mario test.".to_string(),
            prompt_ref: "prompt:stage-k:k2:r1:mario-test-resume-read-only".to_string(),
            prompt_hash: K2_R1_PROMPT_HASH.to_string(),
            canonical_prompt: K2_R1_CANONICAL_PROMPT.to_string(),
            task_memory_packet_ref: "memory-packet:stage-k:k2:r1:mario-test".to_string(),
            permission_envelope_ref: "permission:stage-k:k2:r1".to_string(),
            readback_marker: "K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10".to_string(),
            readback_plan:
                "Expect marker K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10; unavailable, failed, or timed_out readback keeps result_count=null."
                    .to_string(),
            runtime_log_policy: "write K2 runtime log summary; never persist prompt body".to_string(),
            audit_policy: "write preview/decision/attempt/readback audit refs".to_string(),
            baseline_hashes: k2_mario_baseline_hash_fields(),
            codex_scope: "runner may write required Codex native state; readback reads only the target session last message and marker".to_string(),
            dirty_worktree_policy: "do not revert user changes; compare core baseline file hashes".to_string(),
            rollback_policy:
                "read-only authorizes no project writes; any project file change blocks acceptance"
                    .to_string(),
            user_confirmation: "confirmed_by:user".to_string(),
        },
        K2_R2_EXECUTION_POINT_ID => K2ExecutionPointConfig {
            execution_point_id: K2_R2_EXECUTION_POINT_ID.to_string(),
            operation: "resume".to_string(),
            adapter_id: "codex-local".to_string(),
            project_root: K2_MARIO_PROJECT_ROOT.to_string(),
            project_id: K2_MARIO_PROJECT_ID.to_string(),
            workflow_id: K2_MARIO_WORKFLOW_ID.to_string(),
            run_unit_id: "run-unit:stage-k:k2:r2".to_string(),
            node_id: "node:stage-k:k2:r2".to_string(),
            target_session_id: Some(K2_R2_SESSION_ID.to_string()),
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![K2_R2_ALLOWED_WRITE_ROOT.to_string()],
            allowed_write_path: Some(K2_R2_ALLOWED_WRITE_PATH.to_string()),
            denied_paths: k2_denied_paths("project files outside the allowed write path"),
            prompt_summary:
                "Stage K K2 resume/workspace-write probe writing only the allowed marker file."
                    .to_string(),
            prompt_ref: "prompt:stage-k:k2:r2:mario-test-resume-write".to_string(),
            prompt_hash: K2_R2_PROMPT_HASH.to_string(),
            canonical_prompt: K2_R2_CANONICAL_PROMPT.to_string(),
            task_memory_packet_ref: "memory-packet:stage-k:k2:r2:mario-test".to_string(),
            permission_envelope_ref: "permission:stage-k:k2:r2".to_string(),
            readback_marker: "K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10".to_string(),
            readback_plan:
                "Expect marker K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10 in last message and allowed file; unavailable, failed, or timed_out readback keeps result_count=null."
                    .to_string(),
            runtime_log_policy: "write K2 runtime log summary; never persist prompt body".to_string(),
            audit_policy: "write preview/decision/attempt/readback audit refs".to_string(),
            baseline_hashes: k2_mario_baseline_hash_fields(),
            codex_scope: "runner may write required Codex native state; readback reads only the target session last message and marker".to_string(),
            dirty_worktree_policy:
                "do not revert user changes; only allowed write path may change outside core hash checks"
                    .to_string(),
            rollback_policy:
                "allowed file may be removed or retained as evidence by the task package owner"
                    .to_string(),
            user_confirmation: "confirmed_by:user".to_string(),
        },
        K2_N1_EXECUTION_POINT_ID => K2ExecutionPointConfig {
            execution_point_id: K2_N1_EXECUTION_POINT_ID.to_string(),
            operation: "new_session".to_string(),
            adapter_id: "codex-local".to_string(),
            project_root: K2_ISOLATED_PROJECT_ROOT.to_string(),
            project_id: K2_ISOLATED_PROJECT_ID.to_string(),
            workflow_id: K2_ISOLATED_WORKFLOW_ID.to_string(),
            run_unit_id: "run-unit:stage-k:k2:n1".to_string(),
            node_id: "node:stage-k:k2:n1".to_string(),
            target_session_id: None,
            sandbox: "read-only".to_string(),
            allowed_write_roots: Vec::new(),
            allowed_write_path: None,
            denied_paths: k2_denied_paths("product-line paths outside the fixture"),
            prompt_summary:
                "Stage K K2 new-session/read-only health probe in isolated project.".to_string(),
            prompt_ref: "prompt:stage-k:k2:n1:isolated-new-session-read-only".to_string(),
            prompt_hash: K2_N1_PROMPT_HASH.to_string(),
            canonical_prompt: K2_N1_CANONICAL_PROMPT.to_string(),
            task_memory_packet_ref: "memory-packet:stage-k:k2:n1:isolated".to_string(),
            permission_envelope_ref: "permission:stage-k:k2:n1".to_string(),
            readback_marker: "K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10".to_string(),
            readback_plan:
                "Expect marker K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10; unavailable, failed, or timed_out readback keeps result_count=null."
                    .to_string(),
            runtime_log_policy: "write K2 runtime log summary; never persist prompt body".to_string(),
            audit_policy: "write preview/decision/attempt/readback audit refs".to_string(),
            baseline_hashes: vec!["fixture_directory_state".to_string()],
            codex_scope: "runner may create required Codex native session state; readback reads only the new session last message and marker".to_string(),
            dirty_worktree_policy:
                "fixture directory must stay isolated; do not modify product source".to_string(),
            rollback_policy: "read-only authorizes no project writes; fixture empty dirs may be cleaned"
                .to_string(),
            user_confirmation: "confirmed_by:user".to_string(),
        },
        K2_N2_EXECUTION_POINT_ID => K2ExecutionPointConfig {
            execution_point_id: K2_N2_EXECUTION_POINT_ID.to_string(),
            operation: "new_session".to_string(),
            adapter_id: "codex-local".to_string(),
            project_root: K2_ISOLATED_PROJECT_ROOT.to_string(),
            project_id: K2_ISOLATED_PROJECT_ID.to_string(),
            workflow_id: K2_ISOLATED_WORKFLOW_ID.to_string(),
            run_unit_id: "run-unit:stage-k:k2:n2".to_string(),
            node_id: "node:stage-k:k2:n2".to_string(),
            target_session_id: None,
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![K2_N2_ALLOWED_WRITE_ROOT.to_string()],
            allowed_write_path: Some(K2_N2_ALLOWED_WRITE_PATH.to_string()),
            denied_paths: k2_denied_paths(
                "product-line paths outside the fixture and fixture files outside the allowed path",
            ),
            prompt_summary:
                "Stage K K2 new-session/workspace-write probe writing only the allowed marker file."
                    .to_string(),
            prompt_ref: "prompt:stage-k:k2:n2:isolated-new-session-write".to_string(),
            prompt_hash: K2_N2_PROMPT_HASH.to_string(),
            canonical_prompt: K2_N2_CANONICAL_PROMPT.to_string(),
            task_memory_packet_ref: "memory-packet:stage-k:k2:n2:isolated".to_string(),
            permission_envelope_ref: "permission:stage-k:k2:n2".to_string(),
            readback_marker: "K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10".to_string(),
            readback_plan:
                "Expect marker K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10 in last message and allowed file; unavailable, failed, or timed_out readback keeps result_count=null."
                    .to_string(),
            runtime_log_policy: "write K2 runtime log summary; never persist prompt body".to_string(),
            audit_policy: "write preview/decision/attempt/readback audit refs".to_string(),
            baseline_hashes: vec![
                "fixture_directory_state".to_string(),
                "allowed_write_file_hash".to_string(),
            ],
            codex_scope: "runner may create required Codex native session state; readback reads only the new session last message and marker".to_string(),
            dirty_worktree_policy:
                "fixture directory must stay isolated; do not modify product source".to_string(),
            rollback_policy: format!(
                "only {K2_N2_ALLOWED_WRITE_PATH} may be created or updated"
            ),
            user_confirmation: "confirmed_by:user".to_string(),
        },
        other => return Err(format!("unsupported_k2_execution_point:{other}")),
    };
    validate_k2_execution_point_config(&config)?;
    Ok(config)
}

pub(crate) fn validate_k2_execution_point_config(
    config: &K2ExecutionPointConfig,
) -> Result<(), String> {
    if config.adapter_id != "codex-local" {
        return Err("k2_execution_point_adapter_must_be_codex_local".to_string());
    }
    if !matches!(config.operation.as_str(), "resume" | "new_session") {
        return Err("k2_execution_point_operation_unsupported".to_string());
    }
    if config.operation == "resume" && config.target_session_id.as_deref().unwrap_or("").is_empty()
    {
        return Err("k2_resume_requires_target_session".to_string());
    }
    if config.operation == "new_session" && config.target_session_id.is_some() {
        return Err("k2_new_session_must_not_bind_target_session".to_string());
    }
    if config.sandbox == "read-only" && !config.allowed_write_roots.is_empty() {
        return Err("k2_read_only_allowed_write_roots_must_be_empty".to_string());
    }
    if config.sandbox == "workspace-write" {
        let Some(allowed_write_path) = config.allowed_write_path.as_ref() else {
            return Err("k2_workspace_write_requires_allowed_write_path".to_string());
        };
        if config.allowed_write_roots.len() != 1 {
            return Err("k2_workspace_write_requires_one_allowed_write_root".to_string());
        }
        if !allowed_write_path.starts_with(&config.allowed_write_roots[0]) {
            return Err("k2_allowed_write_path_must_be_inside_allowed_root".to_string());
        }
    }
    if sha256_hex(&config.canonical_prompt) != config.prompt_hash {
        return Err(format!(
            "k2_canonical_prompt_hash_mismatch:{}",
            config.execution_point_id
        ));
    }
    if !codex_control_denied_paths_cover_sensitive_boundary(&config.denied_paths) {
        return Err("k2_sensitive_denied_paths_missing".to_string());
    }
    if !config.readback_plan.contains(&config.readback_marker) {
        return Err("k2_readback_plan_must_include_marker".to_string());
    }
    Ok(())
}

pub(crate) fn k2_prepare_input(
    config: &K2ExecutionPointConfig,
    expected_store_revision: Option<i64>,
    created_at: Option<String>,
) -> Result<PrepareRealExecutionProductCommandInput, String> {
    validate_k2_execution_point_config(config)?;
    Ok(PrepareRealExecutionProductCommandInput {
        source_kind: "codex_control".to_string(),
        h5_dispatch_preview: None,
        codex_control: Some(k2_codex_control_input(config)?),
        expected_store_revision,
        requested_by: Some("user".to_string()),
        created_at,
    })
}

pub(crate) fn k2_decision_input(
    product_command_id: &str,
    expected_store_revision: i64,
    confirmed_at: Option<String>,
) -> RecordRealExecutionProductCommandDecisionInput {
    RecordRealExecutionProductCommandDecisionInput {
        product_command_id: product_command_id.to_string(),
        decision: "approved".to_string(),
        expected_store_revision: Some(expected_store_revision),
        confirmed_by: "user".to_string(),
        risk_acknowledgement:
            "User authorized this K2 execution point once through Product Command.".to_string(),
        allowed_once: true,
        reason: "K2 execution point follows prepare -> user decision -> Phase A -> Phase B."
            .to_string(),
        requested_by: Some("user".to_string()),
        confirmed_at,
    }
}

pub(crate) fn k2_phase_a_input(
    product_command_id: &str,
    expected_product_command_store_revision: i64,
    requested_at: Option<String>,
) -> RunRealExecutionProductCommandPhaseAInput {
    RunRealExecutionProductCommandPhaseAInput {
        product_command_id: product_command_id.to_string(),
        expected_product_command_store_revision: Some(expected_product_command_store_revision),
        expected_session_continuation_store_revision: None,
        actor_role: "project_director".to_string(),
        execution_decision: Some("phase_a_noop".to_string()),
        timeout_ms: Some(120_000),
        requested_at,
    }
}

pub(crate) fn k2_resume_phase_b_input(
    workflow_state_path: &Path,
    config: &K2ExecutionPointConfig,
    product_command_id: &str,
    expected_product_revision: i64,
    expected_continuation_revision: i64,
    requested_at: Option<String>,
) -> Result<RunRealExecutionProductCommandPhaseBInput, String> {
    validate_k2_execution_point_config(config)?;
    if config.operation != "resume" {
        return Err("k2_resume_phase_b_requires_resume_config".to_string());
    }
    let requested_at_value = requested_at
        .clone()
        .unwrap_or_else(crate::unix_timestamp_string);
    let (store, _, _) =
        load_real_execution_product_command_store(workflow_state_path, &requested_at_value)?;
    let request = store
        .commands
        .iter()
        .find(|command| command.product_command_id == product_command_id)
        .ok_or_else(|| "k2_product_command_not_prepared".to_string())?;
    let continuation_id = pcr9a_phase_b_continuation_id(&store, product_command_id)
        .ok_or_else(|| "k2_phase_b_requires_phase_a_continuation".to_string())?;
    let continuation_store =
        session_continuation_store::load_store(workflow_state_path, &requested_at_value)?;
    let continuation = continuation_store
        .continuations
        .iter()
        .find(|continuation| continuation.continuation_id == continuation_id)
        .ok_or_else(|| "k2_continuation_not_found".to_string())?;

    Ok(RunRealExecutionProductCommandPhaseBInput {
        product_command_id: product_command_id.to_string(),
        expected_product_command_store_revision: Some(expected_product_revision),
        expected_session_continuation_store_revision: Some(expected_continuation_revision),
        actor_role: "project_director".to_string(),
        execution_decision: Some("approved_for_phase_b".to_string()),
        authorization: k2_resume_authorization_from_request_and_continuation(
            workflow_state_path,
            config,
            request,
            continuation,
        ),
        prompt_body: config.canonical_prompt.clone(),
        requested_at,
    })
}

pub(crate) fn k2_new_session_phase_b_input(
    workflow_state_path: &Path,
    config: &K2ExecutionPointConfig,
    product_command_id: &str,
    expected_product_revision: i64,
    expected_continuation_revision: i64,
    requested_at: Option<String>,
) -> Result<RunRealExecutionProductCommandNewSessionPhaseBInput, String> {
    validate_k2_execution_point_config(config)?;
    if config.operation != "new_session" {
        return Err("k2_new_session_phase_b_requires_new_session_config".to_string());
    }
    let requested_at_value = requested_at
        .clone()
        .unwrap_or_else(crate::unix_timestamp_string);
    let (store, _, _) =
        load_real_execution_product_command_store(workflow_state_path, &requested_at_value)?;
    let request = store
        .commands
        .iter()
        .find(|command| command.product_command_id == product_command_id)
        .ok_or_else(|| "k2_product_command_not_prepared".to_string())?;
    let continuation_id = pcr9a_phase_b_continuation_id(&store, product_command_id)
        .ok_or_else(|| "k2_phase_b_requires_phase_a_continuation".to_string())?;
    let continuation_store =
        session_continuation_store::load_store(workflow_state_path, &requested_at_value)?;
    let continuation = continuation_store
        .continuations
        .iter()
        .find(|continuation| continuation.continuation_id == continuation_id)
        .ok_or_else(|| "k2_continuation_not_found".to_string())?;

    Ok(RunRealExecutionProductCommandNewSessionPhaseBInput {
        product_command_id: product_command_id.to_string(),
        expected_product_command_store_revision: Some(expected_product_revision),
        expected_session_continuation_store_revision: Some(expected_continuation_revision),
        actor_role: "project_director".to_string(),
        execution_decision: Some("approved_for_h3_b".to_string()),
        authorization: k2_new_session_authorization_from_request_and_continuation(
            workflow_state_path,
            config,
            request,
            continuation,
        ),
        prompt_body: config.canonical_prompt.clone(),
        requested_at,
    })
}

fn k2_codex_control_input(
    config: &K2ExecutionPointConfig,
) -> Result<CodexControlCommandInput, String> {
    validate_k2_execution_point_config(config)?;
    Ok(CodexControlCommandInput {
        project_id: Some(config.project_id.clone()),
        project_root: config.project_root.clone(),
        workflow_id: Some(config.workflow_id.clone()),
        node_id: Some(config.node_id.clone()),
        work_item_id: Some(config.run_unit_id.clone()),
        task_package_ref: Some(format!("task-package:{}", config.execution_point_id)),
        memory_packet_ref: Some(config.task_memory_packet_ref.clone()),
        adapter_id: config.adapter_id.clone(),
        operation_id: config.operation.clone(),
        session_mode: if config.operation == "new_session" {
            "new_session_execution_point".to_string()
        } else {
            "resume_existing_session".to_string()
        },
        target_session_id: config.target_session_id.clone(),
        sandbox: config.sandbox.clone(),
        prompt_summary: config.prompt_summary.clone(),
        prompt_ref: config.prompt_ref.clone(),
        prompt_hash: config.prompt_hash.clone(),
        allowed_write_roots: config.allowed_write_roots.clone(),
        denied_paths: config.denied_paths.clone(),
        readback_plan: config.readback_plan.clone(),
        timeout_ms: Some(120_000),
        requested_by: Some("user".to_string()),
    })
}

fn k2_resume_authorization_from_request_and_continuation(
    workflow_state_path: &Path,
    config: &K2ExecutionPointConfig,
    request: &RealExecutionProductCommandRequest,
    continuation: &crate::ControlledSessionContinuation,
) -> H2RealResumeAuthorizationMatrix {
    H2RealResumeAuthorizationMatrix {
        operation_type: "resume".to_string(),
        test_project: format!("K2 {}", config.execution_point_id),
        project_root: continuation.project_root.clone(),
        target_cwd: continuation.target_cwd.clone(),
        target_session: continuation.session_id.clone(),
        prompt_summary: request.prompt_summary.clone(),
        prompt_sha256: config.prompt_hash.clone(),
        prompt_ref: request.prompt_ref.clone(),
        allowed_write_roots: request.allowed_write_roots.clone(),
        codex_home_scope: config.codex_scope.clone(),
        sandbox: continuation.sandbox.clone(),
        timeout_ms: request.timeout_ms.or(Some(120_000)),
        readback_plan: config.readback_plan.clone(),
        evidence_path: k2_evidence_path(workflow_state_path, config),
        rollback_plan: config.rollback_policy.clone(),
        user_confirmed_real_resume: true,
        global_supervisor_confirmed: true,
    }
}

fn k2_new_session_authorization_from_request_and_continuation(
    workflow_state_path: &Path,
    config: &K2ExecutionPointConfig,
    request: &RealExecutionProductCommandRequest,
    continuation: &crate::ControlledSessionContinuation,
) -> H3RealNewSessionAuthorizationMatrix {
    H3RealNewSessionAuthorizationMatrix {
        operation_type: "new_session".to_string(),
        test_project: format!("K2 {}", config.execution_point_id),
        project_root: continuation.project_root.clone(),
        target_cwd: continuation.target_cwd.clone(),
        work_item_id: continuation.work_item_id.clone().unwrap_or_default(),
        prompt_summary: request.prompt_summary.clone(),
        prompt_sha256: config.prompt_hash.clone(),
        prompt_ref: request.prompt_ref.clone(),
        allowed_write_roots: request.allowed_write_roots.clone(),
        codex_home_scope: config.codex_scope.clone(),
        sandbox: continuation.sandbox.clone(),
        timeout_ms: request.timeout_ms.or(Some(120_000)),
        readback_plan: config.readback_plan.clone(),
        evidence_path: k2_evidence_path(workflow_state_path, config),
        rollback_plan: config.rollback_policy.clone(),
        user_confirmed_real_new_session: true,
        global_supervisor_confirmed: true,
    }
}

fn k2_evidence_path(workflow_state_path: &Path, config: &K2ExecutionPointConfig) -> String {
    workflow_state_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}-evidence-ref.json", config.execution_point_id))
        .display()
        .to_string()
}

fn k2_denied_paths(extra: &str) -> Vec<String> {
    [
        "secret",
        "token",
        ".env",
        "auth",
        "keychain",
        "OAuth",
        "provider credential",
        "full transcript",
        "rollout",
        extra,
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn k2_mario_baseline_hash_fields() -> Vec<String> {
    ["index.html", "styles.css", "game.js", "README.md"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) fn real_execution_product_command_sidecar_path(
    workflow_state_path: &Path,
) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state path has no parent; cannot derive product command sidecar: {}",
                workflow_state_path.display()
            )
        })?
        .join(PRODUCT_COMMAND_SIDECAR_NAME))
}

pub(crate) fn load_real_execution_product_command_read_model(
    workflow_state_path: &Path,
    generated_at: &str,
) -> RealExecutionProductCommandReadModel {
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)
        .unwrap_or_else(|_| workflow_state_path.with_file_name(PRODUCT_COMMAND_SIDECAR_NAME));
    match load_real_execution_product_command_store(workflow_state_path, generated_at) {
        Ok((store, store_available, sidecar_path)) => {
            read_model_from_store(&store, store_available, Some(sidecar_path), vec![])
        }
        Err(error) => {
            let store = empty_real_execution_product_command_store(generated_at);
            read_model_from_store(&store, false, Some(sidecar), vec![error])
        }
    }
}

pub(crate) fn load_real_execution_product_command_store(
    workflow_state_path: &Path,
    generated_at: &str,
) -> Result<(RealExecutionProductCommandStore, bool, PathBuf), String> {
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok((
            empty_real_execution_product_command_store(generated_at),
            false,
            sidecar,
        ));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "read real execution product command sidecar failed {}: {error}",
            sidecar.display()
        )
    })?;
    let store: RealExecutionProductCommandStore = serde_json::from_str(&text).map_err(|error| {
        format!(
            "parse real execution product command sidecar failed {}: {error}",
            sidecar.display()
        )
    })?;
    validate_real_execution_product_command_store(&store)?;
    Ok((store, true, sidecar))
}

pub(crate) fn empty_real_execution_product_command_store(
    generated_at: &str,
) -> RealExecutionProductCommandStore {
    RealExecutionProductCommandStore {
        schema_version: PRODUCT_COMMAND_STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        created_at: generated_at.to_string(),
        updated_at: generated_at.to_string(),
        last_write_id: None,
        commands: Vec::new(),
        previews: Vec::new(),
        decisions: Vec::new(),
        attempts: Vec::new(),
        audit_refs: Vec::new(),
        warnings: vec![
            "pcr1_store_skeleton_only".to_string(),
            "no_real_execution_product_command_sidecar_written_by_pcr1".to_string(),
        ],
    }
}

pub(crate) fn validate_real_execution_product_command_store(
    store: &RealExecutionProductCommandStore,
) -> Result<(), String> {
    if store.schema_version != PRODUCT_COMMAND_STORE_SCHEMA_VERSION {
        return Err(format!(
            "real_execution_product_command_store_schema_mismatch:{}",
            store.schema_version
        ));
    }

    let command_ids = store
        .commands
        .iter()
        .map(|command| command.product_command_id.as_str())
        .collect::<BTreeSet<_>>();
    for preview in &store.previews {
        if !command_ids.contains(preview.request.product_command_id.as_str()) {
            return Err(format!(
                "real_execution_product_command_preview_unknown_command:{}",
                preview.request.product_command_id
            ));
        }
        validate_pcr1_preview_safety(preview)?;
    }
    for attempt in &store.attempts {
        if !command_ids.contains(attempt.product_command_id.as_str()) {
            return Err(format!(
                "real_execution_product_command_attempt_unknown_command:{}",
                attempt.product_command_id
            ));
        }
        validate_real_execution_product_command_attempt(attempt)?;
    }
    for decision in &store.decisions {
        let Some(request) = store
            .commands
            .iter()
            .find(|command| command.product_command_id == decision.product_command_id)
        else {
            return Err(format!(
                "real_execution_product_command_decision_unknown_command:{}",
                decision.product_command_id
            ));
        };
        validate_real_execution_product_command_decision(request, decision)?;
    }
    Ok(())
}

pub(crate) fn preview_real_execution_product_command_at(
    workflow_state_path: &Path,
    input: &PreviewRealExecutionProductCommandInput,
) -> Result<RealExecutionProductCommandPreview, String> {
    let created_at = input
        .created_at
        .clone()
        .unwrap_or_else(crate::unix_timestamp_string);
    match input.source_kind.as_str() {
        "h5_project_workflow_dispatch" => {
            let h5_input = h5_preview_input_for_source(
                &input.source_kind,
                input.h5_dispatch_preview.as_ref(),
            )?;
            let h5_preview = h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at(
                workflow_state_path,
                h5_input,
            )?;
            Ok(real_execution_product_command_preview_from_h5(
                h5_input,
                &h5_preview,
                input.requested_by.as_deref(),
                &created_at,
            ))
        }
        "codex_control" => {
            let control = input.codex_control.as_ref().ok_or_else(|| {
                "codex_control_input_required_for_product_command_source".to_string()
            })?;
            Ok(real_execution_product_command_preview_from_codex_control(
                control,
                input.requested_by.as_deref(),
                &created_at,
            ))
        }
        other => Err(format!("unsupported_product_command_source:{other}")),
    }
}

pub(crate) fn prepare_real_execution_product_command_at(
    workflow_state_path: &Path,
    input: &PrepareRealExecutionProductCommandInput,
) -> Result<RealExecutionProductCommandPrepareOutput, String> {
    let preview_input = PreviewRealExecutionProductCommandInput {
        source_kind: input.source_kind.clone(),
        h5_dispatch_preview: input.h5_dispatch_preview.clone(),
        codex_control: input.codex_control.clone(),
        requested_by: input.requested_by.clone(),
        created_at: input.created_at.clone(),
    };
    let preview = preview_real_execution_product_command_at(workflow_state_path, &preview_input)?;
    let generated_at = input
        .created_at
        .clone()
        .unwrap_or_else(crate::unix_timestamp_string);
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    let (mut store, _, sidecar_path) =
        load_real_execution_product_command_store(workflow_state_path, &generated_at)?;

    if !preview.blocked_reasons.is_empty() {
        let blocked_reasons = preview.blocked_reasons.clone();
        let read_model =
            read_model_from_store(&store, sidecar.exists(), Some(sidecar_path), vec![]);
        return Ok(RealExecutionProductCommandPrepareOutput {
            status: "blocked_not_prepared".to_string(),
            product_command_id: Some(preview.request.product_command_id.clone()),
            store_revision: read_model.store_revision,
            sidecar_path: read_model.sidecar_path.clone(),
            preview,
            read_model,
            writes_product_command_sidecar: false,
            blocked_reasons,
            warnings: vec!["blocked_preview_not_written_to_product_command_sidecar".to_string()],
        });
    }

    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            let read_model = read_model_from_store(
                &store,
                sidecar.exists(),
                Some(sidecar_path),
                vec![format!(
                    "product_command_store_revision_conflict:expected={expected}:actual={}",
                    store.revision
                )],
            );
            return Ok(RealExecutionProductCommandPrepareOutput {
                status: "store_conflict".to_string(),
                product_command_id: Some(preview.request.product_command_id.clone()),
                store_revision: read_model.store_revision,
                sidecar_path: read_model.sidecar_path.clone(),
                preview,
                read_model,
                writes_product_command_sidecar: false,
                blocked_reasons: Vec::new(),
                warnings: vec!["product_command_store_revision_conflict_no_write".to_string()],
            });
        }
    }

    let product_command_id = preview.request.product_command_id.clone();
    store.commands.push(preview.request.clone());
    store.previews.push(preview.clone());
    store.revision += 1;
    store.updated_at = generated_at.clone();
    store.last_write_id = Some(format!(
        "pcr2_prepare:{product_command_id}:{}",
        store.revision
    ));
    store
        .warnings
        .push("pcr2_prepare_wrote_command_and_preview_only".to_string());
    store.warnings = crate::dedupe_strings(store.warnings);
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar, &store, &generated_at)?;
    let read_model =
        load_real_execution_product_command_read_model(workflow_state_path, &generated_at);

    Ok(RealExecutionProductCommandPrepareOutput {
        status: "prepared".to_string(),
        product_command_id: Some(product_command_id),
        store_revision: read_model.store_revision,
        sidecar_path: read_model.sidecar_path.clone(),
        preview,
        read_model,
        writes_product_command_sidecar: true,
        blocked_reasons: Vec::new(),
        warnings: vec!["pcr2_prepare_no_decision_no_attempt_no_runner".to_string()],
    })
}

pub(crate) fn record_real_execution_product_command_decision_at(
    workflow_state_path: &Path,
    input: &RecordRealExecutionProductCommandDecisionInput,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    let confirmed_at = input
        .confirmed_at
        .clone()
        .unwrap_or_else(crate::unix_timestamp_string);
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        let blocked_reasons = vec!["product_command_sidecar_missing_for_decision".to_string()];
        let store = empty_real_execution_product_command_store(&confirmed_at);
        let read_model =
            read_model_from_store(&store, false, Some(sidecar), blocked_reasons.clone());
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            vec!["pcr3_decision_requires_existing_product_command_sidecar".to_string()],
        ));
    }

    let (mut store, store_available, sidecar_path) =
        load_real_execution_product_command_store(workflow_state_path, &confirmed_at)?;
    let sidecar_path_for_read_model = Some(sidecar_path.clone());

    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            let blocked_reasons = vec!["product_command_store_revision_conflict".to_string()];
            let read_model = read_model_from_store(
                &store,
                store_available,
                sidecar_path_for_read_model,
                vec![format!(
                    "product_command_store_revision_conflict:expected={expected}:actual={}",
                    store.revision
                )],
            );
            return Ok(real_execution_product_command_decision_output(
                "store_conflict",
                None,
                read_model,
                None,
                false,
                blocked_reasons,
                vec!["pcr3_decision_revision_conflict_no_write".to_string()],
            ));
        }
    }

    if !pcr3_is_terminal_decision(&input.decision) {
        let blocked_reasons = vec!["unsupported_product_command_decision".to_string()];
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_unsupported_decision_no_write"),
        ));
    }

    let Some(request) = store
        .commands
        .iter()
        .find(|command| command.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_not_prepared".to_string()];
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_unknown_command_no_write"),
        ));
    };

    let Some(preview) = store
        .previews
        .iter()
        .find(|preview| preview.request.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_preview_missing".to_string()];
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_preview_missing_no_write"),
        ));
    };

    if store.decisions.iter().any(|decision| {
        decision.product_command_id == input.product_command_id
            && pcr3_is_terminal_decision(&decision.decision)
    }) {
        let blocked_reasons = vec!["product_command_decision_already_recorded".to_string()];
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_duplicate_terminal_decision_no_write"),
        ));
    }

    if input.decision == "approved" && !pcr3_preview_ready_for_approval(&preview) {
        let blocked_reasons = vec!["product_command_preview_not_ready_for_approval".to_string()];
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_blocked_preview_approval_no_write"),
        ));
    }

    let next_revision = store.revision + 1;
    let decision = RealExecutionProductCommandDecision {
        decision_id: format!(
            "real-exec-decision:{}:{next_revision}",
            input.product_command_id
        ),
        product_command_id: input.product_command_id.clone(),
        decision: input.decision.clone(),
        confirmed_by: input.confirmed_by.clone(),
        confirmed_at: confirmed_at.clone(),
        store_revision: next_revision,
        risk_acknowledgement: input.risk_acknowledgement.clone(),
        allowed_once: input.allowed_once,
        reason: input.reason.clone(),
    };
    let blocked_reasons = pcr3_decision_validation_blocked_reasons(&request, &decision);
    if !blocked_reasons.is_empty() {
        let read_model = read_model_from_store(
            &store,
            store_available,
            sidecar_path_for_read_model,
            blocked_reasons.clone(),
        );
        return Ok(real_execution_product_command_decision_output(
            "blocked",
            None,
            read_model,
            None,
            false,
            blocked_reasons,
            pcr3_decision_warnings(input, "pcr3_decision_validation_failed_no_write"),
        ));
    }

    let audit_ref = format!(
        "real-exec-command-audit:{}:{}",
        input.product_command_id, decision.decision_id
    );
    store.decisions.push(decision.clone());
    store.audit_refs.push(audit_ref.clone());
    store.revision = next_revision;
    store.updated_at = confirmed_at.clone();
    store.last_write_id = Some(format!(
        "pcr3_decision:{}:{}",
        input.product_command_id, store.revision
    ));
    store
        .warnings
        .push("pcr3_decision_recorded_no_runner_no_attempt".to_string());
    store.warnings = crate::dedupe_strings(store.warnings);
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar_path, &store, &confirmed_at)?;
    let read_model =
        load_real_execution_product_command_read_model(workflow_state_path, &confirmed_at);

    Ok(real_execution_product_command_decision_output(
        "decision_recorded",
        Some(decision),
        read_model,
        Some(audit_ref),
        true,
        Vec::new(),
        pcr3_decision_warnings(input, "pcr3_decision_recorded_permission_only_no_runner"),
    ))
}

pub(crate) fn confirm_real_execution_product_command_at(
    workflow_state_path: &Path,
    input: &ConfirmRealExecutionProductCommandInput,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    let request = RecordRealExecutionProductCommandDecisionInput {
        product_command_id: input.product_command_id.clone(),
        decision: "approved".to_string(),
        expected_store_revision: input.expected_store_revision,
        confirmed_by: input.confirmed_by.clone(),
        risk_acknowledgement: input.risk_acknowledgement.clone(),
        allowed_once: input.allowed_once,
        reason: input.reason.clone(),
        requested_by: input.requested_by.clone(),
        confirmed_at: input.confirmed_at.clone(),
    };
    let mut output =
        record_real_execution_product_command_decision_at(workflow_state_path, &request)?;
    output
        .warnings
        .push("confirm_real_execution_product_command_records_approved_decision_only".to_string());
    output.warnings = crate::dedupe_strings(output.warnings);
    Ok(output)
}

pub(crate) fn run_real_execution_product_command_phase_a_at(
    workflow_state_path: &Path,
    input: &RunRealExecutionProductCommandPhaseAInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RealExecutionProductCommandPhaseAOutput, String> {
    let requested_at = input
        .requested_at
        .clone()
        .unwrap_or_else(|| timestamp.to_string());
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        let blocked_reasons = vec!["product_command_sidecar_missing_for_phase_a".to_string()];
        let store = empty_real_execution_product_command_store(&requested_at);
        let read_model =
            read_model_from_store(&store, false, Some(sidecar), blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_a_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr4_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr4_phase_a_requires_existing_product_command_sidecar".to_string()],
        ));
    }

    let (mut store, store_available, sidecar_path) =
        load_real_execution_product_command_store(workflow_state_path, &requested_at)?;
    let sidecar_path_for_read_model = Some(sidecar_path.clone());
    let read_model_for_block = |store: &RealExecutionProductCommandStore, warnings: Vec<String>| {
        read_model_from_store(
            store,
            store_available,
            sidecar_path_for_read_model.clone(),
            warnings,
        )
    };

    if let Some(expected) = input.expected_product_command_store_revision {
        if expected != store.revision {
            let blocked_reasons = vec!["product_command_store_revision_conflict".to_string()];
            let read_model = read_model_for_block(
                &store,
                vec![format!(
                    "product_command_store_revision_conflict:expected={expected}:actual={}",
                    store.revision
                )],
            );
            return Ok(real_execution_product_command_phase_a_output(
                "store_conflict",
                input.product_command_id.clone(),
                None,
                read_model,
                None,
                None,
                None,
                None,
                Vec::new(),
                pcr4_readback_boundary("readback_unavailable", None),
                false,
                false,
                false,
                blocked_reasons,
                vec!["pcr4_phase_a_revision_conflict_no_write".to_string()],
            ));
        }
    }

    let Some(request) = store
        .commands
        .iter()
        .find(|command| command.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_not_prepared".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_a_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr4_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr4_unknown_command_no_write".to_string()],
        ));
    };

    let Some(preview) = store
        .previews
        .iter()
        .find(|preview| preview.request.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_preview_missing".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_a_output(
            "blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr4_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr4_preview_missing_no_write".to_string()],
        ));
    };

    let blocked_reasons = pcr4_phase_a_blocked_reasons(&store, &request, &preview, input);
    if !blocked_reasons.is_empty() {
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_a_output(
            "phase_a_blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr4_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr4_phase_a_blocked_before_noop_runner_no_write".to_string()],
        ));
    }

    runtime_log_store::ensure_appendable(workflow_state_path)?;
    let continuation_preview = pcr4_session_continuation_preview(&request, &preview);
    let confirm = session_continuation_store::confirm_continuation(
        workflow_state_path,
        &ConfirmControlledSessionContinuationInput {
            preview: continuation_preview,
            confirmed_by: "user".to_string(),
            confirmation_reason: "PCR4 Phase A no-op follows approved product command decision."
                .to_string(),
            expected_store_revision: input.expected_session_continuation_store_revision,
        },
        timestamp,
        &format!("{write_id}-confirm-continuation"),
    )?;
    if request.operation_id == "new_session" {
        let next_revision = store.revision + 1;
        let mut audit_refs = vec![confirm.audit_event.event_id.clone()];
        audit_refs.extend(confirm.continuation.audit_refs.clone());
        audit_refs.sort();
        audit_refs.dedup();
        let readback_summary = pcr4_readback_boundary("readback_unavailable", None);
        let product_attempt = RealExecutionProductCommandAttempt {
            attempt_id: format!(
                "real-exec-command-attempt:phase-a:{}:{next_revision}",
                request.product_command_id
            ),
            product_command_id: request.product_command_id.clone(),
            continuation_id: Some(confirm.continuation.continuation_id.clone()),
            adapter_id: request.adapter_id.clone(),
            operation_id: request.operation_id.clone(),
            status: "phase_a_noop_completed".to_string(),
            started_at: timestamp.to_string(),
            completed_at: Some(timestamp.to_string()),
            runner_call_allowed: false,
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_project_files: false,
            runtime_log_ref: None,
            audit_refs: audit_refs.clone(),
            readback_summary: readback_summary.clone(),
            failure_reason: None,
            warnings: crate::dedupe_strings(vec![
                "pcr4_phase_a_noop_completed_for_new_session".to_string(),
                "runner_call_allowed_false_phase_a_noop_only".to_string(),
                "prompt_not_sent".to_string(),
                "real_codex_executed_false".to_string(),
                "codex_home_not_touched".to_string(),
                "project_files_not_written".to_string(),
                "readback_unavailable_is_not_zero_results".to_string(),
            ]),
        };
        store.attempts.push(product_attempt.clone());
        store.audit_refs.extend(audit_refs.clone());
        store.audit_refs.sort();
        store.audit_refs.dedup();
        store.revision = next_revision;
        store.updated_at = timestamp.to_string();
        store.last_write_id = Some(format!(
            "pcr4_phase_a_new_session:{}:{}",
            request.product_command_id, store.revision
        ));
        store
            .warnings
            .push("pcr4_phase_a_new_session_noop_attempt_recorded_no_real_codex".to_string());
        store.warnings = crate::dedupe_strings(store.warnings);
        validate_real_execution_product_command_store(&store)?;
        write_real_execution_product_command_store_atomic(&sidecar_path, &store, timestamp)?;
        let read_model =
            load_real_execution_product_command_read_model(workflow_state_path, timestamp);

        return Ok(real_execution_product_command_phase_a_output(
            "phase_a_completed",
            request.product_command_id,
            Some(product_attempt),
            read_model,
            Some(confirm.continuation.continuation_id),
            None,
            Some(confirm.store_revision),
            None,
            audit_refs,
            readback_summary,
            false,
            true,
            false,
            Vec::new(),
            crate::dedupe_strings(vec![
                "pcr4_phase_a_new_session_noop_did_not_execute_codex".to_string(),
                "product_command_new_session_phase_b_required_for_real_codex_execution".to_string(),
            ]),
        ));
    }
    let authorization = pcr4_phase_a_authorization(&request, &confirm.continuation, input);
    let phase_a = session_continuation_store::run_real_resume_phase_a(
        workflow_state_path,
        &RunControlledSessionContinuationRealResumePhaseAInput {
            continuation_id: confirm.continuation.continuation_id.clone(),
            actor_role: input.actor_role.trim().to_string(),
            expected_store_revision: Some(confirm.store_revision),
            authorization,
            execution_decision: Some("approved_for_phase_a".to_string()),
        },
        timestamp,
        &format!("{write_id}-phase-a-noop"),
    )?;

    let next_revision = store.revision + 1;
    let runtime_log_ref = format!(
        "runtime-log:dispatch-attempt:{}",
        phase_a.attempt.attempt_id
    );
    let mut audit_refs = phase_a
        .audit_events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    audit_refs.extend(phase_a.attempt.audit_refs.clone());
    audit_refs.sort();
    audit_refs.dedup();
    let readback_summary = pcr4_readback_from_session_attempt(&phase_a.attempt);
    let product_attempt = RealExecutionProductCommandAttempt {
        attempt_id: format!(
            "real-exec-command-attempt:phase-a:{}:{next_revision}",
            request.product_command_id
        ),
        product_command_id: request.product_command_id.clone(),
        continuation_id: Some(phase_a.continuation.continuation_id.clone()),
        adapter_id: request.adapter_id.clone(),
        operation_id: request.operation_id.clone(),
        status: "phase_a_noop_completed".to_string(),
        started_at: timestamp.to_string(),
        completed_at: Some(timestamp.to_string()),
        runner_call_allowed: false,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        runtime_log_ref: Some(runtime_log_ref.clone()),
        audit_refs: audit_refs.clone(),
        readback_summary: readback_summary.clone(),
        failure_reason: phase_a.attempt.failure_reason.clone(),
        warnings: crate::dedupe_strings(vec![
            "pcr4_phase_a_noop_completed".to_string(),
            "runner_call_allowed_false_phase_a_noop_only".to_string(),
            "prompt_not_sent".to_string(),
            "real_codex_executed_false".to_string(),
            "codex_home_not_touched".to_string(),
            "project_files_not_written".to_string(),
            "readback_unavailable_is_not_zero_results".to_string(),
        ]),
    };
    store.attempts.push(product_attempt.clone());
    store.audit_refs.extend(audit_refs.clone());
    store.audit_refs.sort();
    store.audit_refs.dedup();
    store.revision = next_revision;
    store.updated_at = timestamp.to_string();
    store.last_write_id = Some(format!(
        "pcr4_phase_a:{}:{}",
        request.product_command_id, store.revision
    ));
    store
        .warnings
        .push("pcr4_phase_a_noop_attempt_recorded_no_real_codex".to_string());
    store.warnings = crate::dedupe_strings(store.warnings);
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar_path, &store, timestamp)?;
    let read_model = load_real_execution_product_command_read_model(workflow_state_path, timestamp);

    Ok(real_execution_product_command_phase_a_output(
        "phase_a_completed",
        request.product_command_id,
        Some(product_attempt),
        read_model,
        Some(phase_a.continuation.continuation_id),
        Some(phase_a.attempt.attempt_id),
        Some(phase_a.store_revision),
        Some(runtime_log_ref),
        audit_refs,
        readback_summary,
        false,
        true,
        true,
        Vec::new(),
        crate::dedupe_strings(vec![
            "pcr4_phase_a_noop_did_not_execute_codex".to_string(),
            "pcr9_level_b_required_for_real_codex_execution".to_string(),
        ]),
    ))
}

pub(crate) fn run_real_execution_product_command_phase_b_at(
    workflow_state_path: &Path,
    input: &RunRealExecutionProductCommandPhaseBInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path =
        pcr9a_phase_b_last_message_path(workflow_state_path, &input.product_command_id, timestamp)?;
    run_real_execution_product_command_phase_b_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_real_execution_product_command_new_session_phase_b_at(
    workflow_state_path: &Path,
    input: &RunRealExecutionProductCommandNewSessionPhaseBInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path =
        pcr9a_phase_b_last_message_path(workflow_state_path, &input.product_command_id, timestamp)?;
    run_real_execution_product_command_new_session_phase_b_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_real_execution_product_command_phase_b_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &RunRealExecutionProductCommandPhaseBInput,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    let requested_at = input
        .requested_at
        .clone()
        .unwrap_or_else(|| timestamp.to_string());
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        let blocked_reasons = vec!["product_command_sidecar_missing_for_phase_b".to_string()];
        let store = empty_real_execution_product_command_store(&requested_at);
        let read_model =
            read_model_from_store(&store, false, Some(sidecar), blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr9a_phase_b_requires_existing_product_command_sidecar".to_string()],
        ));
    }

    let (mut store, store_available, sidecar_path) =
        load_real_execution_product_command_store(workflow_state_path, &requested_at)?;
    let sidecar_path_for_read_model = Some(sidecar_path.clone());
    let read_model_for_block = |store: &RealExecutionProductCommandStore, warnings: Vec<String>| {
        read_model_from_store(
            store,
            store_available,
            sidecar_path_for_read_model.clone(),
            warnings,
        )
    };

    if let Some(expected) = input.expected_product_command_store_revision {
        if expected != store.revision {
            let blocked_reasons = vec!["product_command_store_revision_conflict".to_string()];
            let read_model = read_model_for_block(
                &store,
                vec![format!(
                    "product_command_store_revision_conflict:expected={expected}:actual={}",
                    store.revision
                )],
            );
            return Ok(real_execution_product_command_phase_b_output(
                "store_conflict",
                input.product_command_id.clone(),
                None,
                read_model,
                None,
                None,
                None,
                None,
                Vec::new(),
                pcr9a_readback_boundary("readback_unavailable", None),
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                blocked_reasons,
                vec!["pcr9a_phase_b_revision_conflict_no_write".to_string()],
            ));
        }
    }

    let Some(request) = store
        .commands
        .iter()
        .find(|command| command.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_not_prepared".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr9a_unknown_command_no_write".to_string()],
        ));
    };

    let Some(preview) = store
        .previews
        .iter()
        .find(|preview| preview.request.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_preview_missing".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr9a_preview_missing_no_write".to_string()],
        ));
    };

    let (blocked_reasons, continuation_id) = pcr9a_phase_b_blocked_reasons(
        workflow_state_path,
        &store,
        &request,
        &preview,
        input,
        timestamp,
    )?;
    if !blocked_reasons.is_empty() {
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "phase_b_blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            continuation_id,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["pcr9a_phase_b_blocked_before_continuation_runner_no_write".to_string()],
        ));
    }
    let continuation_id =
        continuation_id.ok_or_else(|| "pcr9a_phase_b_continuation_id_missing".to_string())?;

    let phase_b = session_continuation_store::run_real_resume_phase_b_with_runner(
        workflow_state_path,
        &RunControlledSessionContinuationRealResumePhaseBInput {
            continuation_id: continuation_id.clone(),
            actor_role: input.actor_role.trim().to_string(),
            expected_store_revision: input.expected_session_continuation_store_revision,
            authorization: input.authorization.clone(),
            execution_decision: Some(
                input
                    .execution_decision
                    .clone()
                    .unwrap_or_else(|| "approved_for_phase_b".to_string()),
            ),
            prompt_body: input.prompt_body.clone(),
        },
        timestamp,
        &format!("{write_id}-continuation-phase-b"),
        last_message_path,
        runner,
    )?;

    let next_revision = store.revision + 1;
    let runtime_log_ref = format!(
        "runtime-log:dispatch-attempt:{}",
        phase_b.attempt.attempt_id
    );
    let mut audit_refs = phase_b
        .audit_events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    audit_refs.extend(phase_b.attempt.audit_refs.clone());
    if let Some(audit_ref) = phase_b
        .codex_local_attempt
        .as_ref()
        .and_then(|attempt| attempt.audit_ref.as_ref())
    {
        audit_refs.push(audit_ref.ref_id.clone());
    }
    audit_refs.sort();
    audit_refs.dedup();
    let readback_summary =
        pcr9a_readback_from_phase_b(&phase_b.attempt, phase_b.codex_local_attempt.as_ref());
    let runner_call_allowed = phase_b.authorization_status == "phase_b_real_resume_executed";
    let prompt_sent = phase_b.attempt.prompt_sent
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.prompt_sent);
    let real_codex_executed = phase_b.attempt.real_codex_executed
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.real_codex_executed);
    let writes_codex_home = phase_b.attempt.writes_codex_home
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.writes_codex_home);
    let writes_project_files = phase_b
        .codex_local_attempt
        .as_ref()
        .is_some_and(|attempt| attempt.writes_project_files);
    let failure_reason = phase_b
        .codex_local_attempt
        .as_ref()
        .and_then(|attempt| attempt.failure_reason.as_ref())
        .map(|failure| failure.message.clone())
        .or_else(|| phase_b.attempt.failure_reason.clone());
    let product_attempt = RealExecutionProductCommandAttempt {
        attempt_id: format!(
            "real-exec-command-attempt:phase-b:{}:{next_revision}",
            request.product_command_id
        ),
        product_command_id: request.product_command_id.clone(),
        continuation_id: Some(phase_b.continuation.continuation_id.clone()),
        adapter_id: request.adapter_id.clone(),
        operation_id: request.operation_id.clone(),
        status: pcr9a_phase_b_product_attempt_status(
            &phase_b.authorization_status,
            &phase_b.attempt,
        ),
        started_at: timestamp.to_string(),
        completed_at: Some(timestamp.to_string()),
        runner_call_allowed,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        runtime_log_ref: Some(runtime_log_ref.clone()),
        audit_refs: audit_refs.clone(),
        readback_summary: readback_summary.clone(),
        failure_reason,
        warnings: pcr9a_phase_b_product_attempt_warnings(
            &phase_b.authorization_status,
            &phase_b.attempt,
            phase_b.codex_local_attempt.as_ref(),
        ),
    };
    store.attempts.push(product_attempt.clone());
    store.audit_refs.extend(audit_refs.clone());
    store.audit_refs.sort();
    store.audit_refs.dedup();
    store.revision = next_revision;
    store.updated_at = timestamp.to_string();
    store.last_write_id = Some(format!(
        "pcr9a_phase_b:{}:{}",
        request.product_command_id, store.revision
    ));
    store
        .warnings
        .push("pcr9a_phase_b_attempt_recorded_from_product_command_bridge".to_string());
    store.warnings = crate::dedupe_strings(store.warnings);
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar_path, &store, timestamp)?;
    let read_model = load_real_execution_product_command_read_model(workflow_state_path, timestamp);

    Ok(real_execution_product_command_phase_b_output(
        if runner_call_allowed {
            "phase_b_completed"
        } else {
            "phase_b_blocked"
        },
        request.product_command_id,
        Some(product_attempt),
        read_model,
        Some(phase_b.continuation.continuation_id),
        Some(phase_b.attempt.attempt_id),
        Some(phase_b.store_revision),
        Some(runtime_log_ref),
        audit_refs,
        readback_summary,
        runner_call_allowed,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        true,
        true,
        true,
        phase_b.missing_or_invalid_items.clone(),
        pcr9a_phase_b_output_warnings(&phase_b.authorization_status, phase_b.warnings),
    ))
}

pub(crate) fn run_real_execution_product_command_new_session_phase_b_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &RunRealExecutionProductCommandNewSessionPhaseBInput,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    let requested_at = input
        .requested_at
        .clone()
        .unwrap_or_else(|| timestamp.to_string());
    let sidecar = real_execution_product_command_sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        let blocked_reasons =
            vec!["product_command_sidecar_missing_for_new_session_phase_b".to_string()];
        let store = empty_real_execution_product_command_store(&requested_at);
        let read_model =
            read_model_from_store(&store, false, Some(sidecar), blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["product_command_new_session_phase_b_requires_sidecar".to_string()],
        ));
    }

    let (mut store, store_available, sidecar_path) =
        load_real_execution_product_command_store(workflow_state_path, &requested_at)?;
    let sidecar_path_for_read_model = Some(sidecar_path.clone());
    let read_model_for_block = |store: &RealExecutionProductCommandStore, warnings: Vec<String>| {
        read_model_from_store(
            store,
            store_available,
            sidecar_path_for_read_model.clone(),
            warnings,
        )
    };

    if let Some(expected) = input.expected_product_command_store_revision {
        if expected != store.revision {
            let blocked_reasons = vec!["product_command_store_revision_conflict".to_string()];
            let read_model = read_model_for_block(
                &store,
                vec![format!(
                    "product_command_store_revision_conflict:expected={expected}:actual={}",
                    store.revision
                )],
            );
            return Ok(real_execution_product_command_phase_b_output(
                "store_conflict",
                input.product_command_id.clone(),
                None,
                read_model,
                None,
                None,
                None,
                None,
                Vec::new(),
                pcr9a_readback_boundary("readback_unavailable", None),
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                blocked_reasons,
                vec!["product_command_new_session_phase_b_revision_conflict_no_write".to_string()],
            ));
        }
    }

    let Some(request) = store
        .commands
        .iter()
        .find(|command| command.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_not_prepared".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            input.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["product_command_new_session_unknown_command_no_write".to_string()],
        ));
    };

    let Some(preview) = store
        .previews
        .iter()
        .find(|preview| preview.request.product_command_id == input.product_command_id)
        .cloned()
    else {
        let blocked_reasons = vec!["product_command_preview_missing".to_string()];
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            None,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["product_command_new_session_preview_missing_no_write".to_string()],
        ));
    };

    let (blocked_reasons, continuation_id) = product_new_session_phase_b_blocked_reasons(
        workflow_state_path,
        &store,
        &request,
        &preview,
        input,
        timestamp,
    )?;
    if !blocked_reasons.is_empty() {
        let read_model = read_model_for_block(&store, blocked_reasons.clone());
        return Ok(real_execution_product_command_phase_b_output(
            "phase_b_blocked",
            request.product_command_id.clone(),
            None,
            read_model,
            continuation_id,
            None,
            None,
            None,
            Vec::new(),
            pcr9a_readback_boundary("readback_unavailable", None),
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            blocked_reasons,
            vec!["product_command_new_session_phase_b_blocked_before_runner_no_write".to_string()],
        ));
    }
    let continuation_id = continuation_id
        .ok_or_else(|| "product_new_session_phase_b_continuation_id_missing".to_string())?;

    let phase_b = session_continuation_store::run_real_new_session_h3_b_with_runner(
        workflow_state_path,
        &RunControlledSessionContinuationRealNewSessionH3BInput {
            continuation_id: continuation_id.clone(),
            actor_role: input.actor_role.trim().to_string(),
            expected_store_revision: input.expected_session_continuation_store_revision,
            authorization: input.authorization.clone(),
            execution_decision: Some(
                input
                    .execution_decision
                    .clone()
                    .unwrap_or_else(|| "approved_for_h3_b".to_string()),
            ),
            prompt_body: input.prompt_body.clone(),
        },
        timestamp,
        &format!("{write_id}-continuation-new-session-phase-b"),
        last_message_path,
        runner,
    )?;

    let next_revision = store.revision + 1;
    let runtime_log_ref = format!(
        "runtime-log:dispatch-attempt:{}",
        phase_b.attempt.attempt_id
    );
    let mut audit_refs = phase_b
        .audit_events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    audit_refs.extend(phase_b.attempt.audit_refs.clone());
    if let Some(audit_ref) = phase_b
        .codex_local_attempt
        .as_ref()
        .and_then(|attempt| attempt.audit_ref.as_ref())
    {
        audit_refs.push(audit_ref.ref_id.clone());
    }
    audit_refs.sort();
    audit_refs.dedup();
    let readback_summary =
        pcr9a_readback_from_phase_b(&phase_b.attempt, phase_b.codex_local_attempt.as_ref());
    let runner_call_allowed = phase_b.authorization_status == "h3_b_real_new_session_executed";
    let prompt_sent = phase_b.attempt.prompt_sent
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.prompt_sent);
    let real_codex_executed = phase_b.attempt.real_codex_executed
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.real_codex_executed);
    let writes_codex_home = phase_b.attempt.writes_codex_home
        || phase_b
            .codex_local_attempt
            .as_ref()
            .is_some_and(|attempt| attempt.writes_codex_home);
    let writes_project_files = phase_b
        .codex_local_attempt
        .as_ref()
        .is_some_and(|attempt| attempt.writes_project_files);
    let failure_reason = phase_b
        .codex_local_attempt
        .as_ref()
        .and_then(|attempt| attempt.failure_reason.as_ref())
        .map(|failure| failure.message.clone())
        .or_else(|| phase_b.attempt.failure_reason.clone());
    let product_attempt = RealExecutionProductCommandAttempt {
        attempt_id: format!(
            "real-exec-command-attempt:phase-b-new-session:{}:{next_revision}",
            request.product_command_id
        ),
        product_command_id: request.product_command_id.clone(),
        continuation_id: Some(phase_b.continuation.continuation_id.clone()),
        adapter_id: request.adapter_id.clone(),
        operation_id: request.operation_id.clone(),
        status: product_new_session_phase_b_product_attempt_status(
            &phase_b.authorization_status,
            &phase_b.attempt,
        ),
        started_at: timestamp.to_string(),
        completed_at: Some(timestamp.to_string()),
        runner_call_allowed,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        runtime_log_ref: Some(runtime_log_ref.clone()),
        audit_refs: audit_refs.clone(),
        readback_summary: readback_summary.clone(),
        failure_reason,
        warnings: pcr9a_phase_b_product_attempt_warnings(
            &phase_b.authorization_status,
            &phase_b.attempt,
            phase_b.codex_local_attempt.as_ref(),
        ),
    };
    store.attempts.push(product_attempt.clone());
    store.audit_refs.extend(audit_refs.clone());
    store.audit_refs.sort();
    store.audit_refs.dedup();
    store.revision = next_revision;
    store.updated_at = timestamp.to_string();
    store.last_write_id = Some(format!(
        "product_new_session_phase_b:{}:{}",
        request.product_command_id, store.revision
    ));
    store.warnings.push(
        "product_new_session_phase_b_attempt_recorded_from_product_command_bridge".to_string(),
    );
    store.warnings = crate::dedupe_strings(store.warnings);
    validate_real_execution_product_command_store(&store)?;
    write_real_execution_product_command_store_atomic(&sidecar_path, &store, timestamp)?;
    let read_model = load_real_execution_product_command_read_model(workflow_state_path, timestamp);

    Ok(real_execution_product_command_phase_b_output(
        if runner_call_allowed {
            "phase_b_completed"
        } else {
            "phase_b_blocked"
        },
        request.product_command_id,
        Some(product_attempt),
        read_model,
        Some(phase_b.continuation.continuation_id),
        Some(phase_b.attempt.attempt_id),
        Some(phase_b.store_revision),
        Some(runtime_log_ref),
        audit_refs,
        readback_summary,
        runner_call_allowed,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        true,
        true,
        true,
        phase_b.missing_or_invalid_items.clone(),
        pcr9a_phase_b_output_warnings(&phase_b.authorization_status, phase_b.warnings),
    ))
}

fn real_execution_product_command_phase_a_output(
    status: &str,
    product_command_id: String,
    product_command_attempt: Option<RealExecutionProductCommandAttempt>,
    read_model: RealExecutionProductCommandReadModel,
    continuation_id: Option<String>,
    continuation_attempt_id: Option<String>,
    session_continuation_store_revision: Option<i64>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_summary: RealExecutionProductCommandReadbackBoundary,
    runner_call_allowed: bool,
    writes_continuation_sidecar: bool,
    writes_runtime_log: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
) -> RealExecutionProductCommandPhaseAOutput {
    RealExecutionProductCommandPhaseAOutput {
        status: status.to_string(),
        product_command_id,
        product_command_attempt,
        product_command_store_revision: read_model.store_revision,
        product_command_sidecar_path: read_model.sidecar_path.clone(),
        read_model,
        continuation_id,
        continuation_attempt_id,
        session_continuation_store_revision,
        runtime_log_ref,
        audit_refs: crate::dedupe_strings(audit_refs),
        readback_summary,
        runner_call_allowed,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_product_command_sidecar: status == "phase_a_completed",
        writes_continuation_sidecar,
        writes_runtime_log,
        blocked_reasons: crate::dedupe_strings(blocked_reasons),
        warnings: crate::dedupe_strings(warnings),
    }
}

fn real_execution_product_command_phase_b_output(
    status: &str,
    product_command_id: String,
    product_command_attempt: Option<RealExecutionProductCommandAttempt>,
    read_model: RealExecutionProductCommandReadModel,
    continuation_id: Option<String>,
    continuation_attempt_id: Option<String>,
    session_continuation_store_revision: Option<i64>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_summary: RealExecutionProductCommandReadbackBoundary,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_product_command_sidecar: bool,
    writes_continuation_sidecar: bool,
    writes_runtime_log: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
) -> RealExecutionProductCommandPhaseBOutput {
    RealExecutionProductCommandPhaseBOutput {
        status: status.to_string(),
        product_command_id,
        product_command_attempt,
        product_command_store_revision: read_model.store_revision,
        product_command_sidecar_path: read_model.sidecar_path.clone(),
        read_model,
        continuation_id,
        continuation_attempt_id,
        session_continuation_store_revision,
        runtime_log_ref,
        audit_refs: crate::dedupe_strings(audit_refs),
        readback_summary,
        runner_call_allowed,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        writes_product_command_sidecar,
        writes_continuation_sidecar,
        writes_runtime_log,
        blocked_reasons: crate::dedupe_strings(blocked_reasons),
        warnings: crate::dedupe_strings(warnings),
    }
}

fn pcr9a_phase_b_blocked_reasons(
    workflow_state_path: &Path,
    store: &RealExecutionProductCommandStore,
    request: &RealExecutionProductCommandRequest,
    preview: &RealExecutionProductCommandPreview,
    input: &RunRealExecutionProductCommandPhaseBInput,
    timestamp: &str,
) -> Result<(Vec<String>, Option<String>), String> {
    let mut reasons = Vec::new();
    if input.actor_role.trim().is_empty() {
        reasons.push("phase_b_actor_role_missing".to_string());
    }
    if matches!(input.execution_decision.as_deref(), Some("rejected")) {
        reasons.push("phase_b_execution_decision_rejected".to_string());
    }
    if !matches!(
        input.execution_decision.as_deref(),
        None | Some("approved_for_phase_b") | Some("rejected")
    ) {
        reasons.push("phase_b_execution_decision_unsupported".to_string());
    }
    if request.adapter_id != "codex-local" {
        reasons.push("phase_b_only_supports_codex_local_adapter".to_string());
    }
    if request.operation_id != "resume" {
        reasons.push("phase_b_only_supports_resume".to_string());
    }
    if request
        .target_session_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("phase_b_resume_requires_target_session".to_string());
    }
    if request
        .project_root
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("phase_b_project_root_missing".to_string());
    }
    if !pcr3_preview_ready_for_approval(preview) {
        reasons.push("phase_b_preview_not_ready".to_string());
    }
    if preview.diagnostics_summary.blocks_real_execution {
        reasons.push("phase_b_blocked_by_diagnostics".to_string());
    }
    if preview.duplicate_scope.duplicate_blocked {
        reasons.push("phase_b_duplicate_scope_blocked".to_string());
    }
    if preview_has_blocked_reason(
        preview,
        &[
            "memory_packet_missing",
            "memory_packet_stale",
            "task_memory_packet_snapshot_missing",
            "task_memory_packet_stale",
        ],
    ) {
        reasons.push("phase_b_blocked_stale_memory".to_string());
    }
    let approved_decision = store.decisions.iter().rev().find(|decision| {
        decision.product_command_id == request.product_command_id && decision.decision == "approved"
    });
    match approved_decision {
        Some(decision) if decision.confirmed_by == "user" && decision.allowed_once => {}
        Some(_) => reasons.push("phase_b_approved_decision_must_be_user_once".to_string()),
        None => reasons.push("phase_b_requires_user_approved_decision".to_string()),
    }
    if store.attempts.iter().any(|attempt| {
        attempt.product_command_id == request.product_command_id
            && pcr9a_is_phase_b_product_attempt(attempt)
    }) {
        reasons.push("phase_b_duplicate_attempt_blocked".to_string());
    }
    if input.prompt_body.trim().is_empty() {
        reasons.push("phase_b_runtime_prompt_missing".to_string());
    } else if sha256_hex(&input.prompt_body) != input.authorization.prompt_sha256
        || input.authorization.prompt_sha256 != request.prompt_hash
    {
        reasons.push("phase_b_prompt_hash_mismatch".to_string());
    }

    let continuation_id = pcr9a_phase_b_continuation_id(store, &request.product_command_id);
    if continuation_id.is_none() {
        reasons.push("phase_b_requires_phase_a_continuation".to_string());
    }
    if let Some(continuation_id) = continuation_id.as_ref() {
        let continuation_store =
            session_continuation_store::load_store(workflow_state_path, timestamp)?;
        if let Some(expected) = input.expected_session_continuation_store_revision {
            if expected != continuation_store.revision {
                reasons.push("session_continuation_store_revision_conflict".to_string());
            }
        }
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|continuation| continuation.continuation_id == *continuation_id);
        match continuation {
            Some(continuation) => {
                reasons.extend(pcr9a_phase_b_authorization_binding_reasons(
                    request,
                    continuation,
                    &input.authorization,
                ));
            }
            None => reasons.push("phase_b_continuation_not_found".to_string()),
        }
    }

    reasons.sort();
    reasons.dedup();
    Ok((reasons, continuation_id))
}

fn pcr9a_phase_b_authorization_binding_reasons(
    request: &RealExecutionProductCommandRequest,
    continuation: &crate::ControlledSessionContinuation,
    authorization: &H2RealResumeAuthorizationMatrix,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if authorization.operation_type != "resume" {
        reasons.push("phase_b_authorization_operation_mismatch".to_string());
    }
    if request
        .target_session_id
        .as_deref()
        .is_some_and(|session| session != continuation.session_id)
    {
        reasons.push("phase_b_continuation_session_mismatch_request".to_string());
    }
    if request
        .project_root
        .as_deref()
        .is_some_and(|project_root| project_root != continuation.project_root)
    {
        reasons.push("phase_b_continuation_project_root_mismatch_request".to_string());
    }
    if authorization.project_root != continuation.project_root {
        reasons.push("phase_b_authorization_project_root_mismatch".to_string());
    }
    if authorization.target_cwd != continuation.target_cwd {
        reasons.push("phase_b_authorization_target_cwd_mismatch".to_string());
    }
    if authorization.target_session != continuation.session_id {
        reasons.push("phase_b_authorization_target_session_mismatch".to_string());
    }
    if authorization.sandbox != continuation.sandbox {
        reasons.push("phase_b_authorization_sandbox_mismatch_continuation".to_string());
    }
    let request_sandbox = pcr9a_product_command_sandbox(request);
    if authorization.sandbox != request_sandbox {
        reasons.push("phase_b_authorization_sandbox_mismatch_request".to_string());
    }
    if authorization.prompt_summary != request.prompt_summary {
        reasons.push("phase_b_authorization_prompt_summary_mismatch".to_string());
    }
    if authorization.prompt_ref != request.prompt_ref {
        reasons.push("phase_b_authorization_prompt_ref_mismatch".to_string());
    }
    if authorization.prompt_sha256 != request.prompt_hash {
        reasons.push("phase_b_authorization_prompt_hash_mismatch".to_string());
    }
    if !same_string_set(
        &authorization.allowed_write_roots,
        &request.allowed_write_roots,
    ) {
        reasons.push("phase_b_authorization_allowed_write_roots_mismatch_request".to_string());
    }
    if !same_string_set(
        &authorization.allowed_write_roots,
        &continuation.allowed_write_roots,
    ) {
        reasons.push("phase_b_authorization_allowed_write_roots_mismatch_continuation".to_string());
    }
    reasons
}

fn product_new_session_phase_b_blocked_reasons(
    workflow_state_path: &Path,
    store: &RealExecutionProductCommandStore,
    request: &RealExecutionProductCommandRequest,
    preview: &RealExecutionProductCommandPreview,
    input: &RunRealExecutionProductCommandNewSessionPhaseBInput,
    timestamp: &str,
) -> Result<(Vec<String>, Option<String>), String> {
    let mut reasons = Vec::new();
    if input.actor_role.trim().is_empty() {
        reasons.push("phase_b_actor_role_missing".to_string());
    }
    if matches!(input.execution_decision.as_deref(), Some("rejected")) {
        reasons.push("phase_b_execution_decision_rejected".to_string());
    }
    if !matches!(
        input.execution_decision.as_deref(),
        None | Some("approved_for_phase_b") | Some("approved_for_h3_b") | Some("rejected")
    ) {
        reasons.push("phase_b_execution_decision_unsupported".to_string());
    }
    if request.adapter_id != "codex-local" {
        reasons.push("phase_b_only_supports_codex_local_adapter".to_string());
    }
    if request.operation_id != "new_session" {
        reasons.push("phase_b_only_supports_new_session".to_string());
    }
    if !request
        .target_session_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("phase_b_new_session_must_not_bind_target_session".to_string());
    }
    if request
        .project_root
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("phase_b_project_root_missing".to_string());
    }
    if !pcr3_preview_ready_for_approval(preview) {
        reasons.push("phase_b_preview_not_ready".to_string());
    }
    if preview.diagnostics_summary.blocks_real_execution {
        reasons.push("phase_b_blocked_by_diagnostics".to_string());
    }
    if preview.duplicate_scope.duplicate_blocked {
        reasons.push("phase_b_duplicate_scope_blocked".to_string());
    }
    let approved_decision = store.decisions.iter().rev().find(|decision| {
        decision.product_command_id == request.product_command_id && decision.decision == "approved"
    });
    match approved_decision {
        Some(decision) if decision.confirmed_by == "user" && decision.allowed_once => {}
        Some(_) => reasons.push("phase_b_approved_decision_must_be_user_once".to_string()),
        None => reasons.push("phase_b_requires_user_approved_decision".to_string()),
    }
    if store.attempts.iter().any(|attempt| {
        attempt.product_command_id == request.product_command_id
            && pcr9a_is_phase_b_product_attempt(attempt)
    }) {
        reasons.push("phase_b_duplicate_attempt_blocked".to_string());
    }
    if input.prompt_body.trim().is_empty() {
        reasons.push("phase_b_runtime_prompt_missing".to_string());
    } else if sha256_hex(&input.prompt_body) != input.authorization.prompt_sha256
        || input.authorization.prompt_sha256 != request.prompt_hash
    {
        reasons.push("phase_b_prompt_hash_mismatch".to_string());
    }

    let continuation_id = pcr9a_phase_b_continuation_id(store, &request.product_command_id);
    if continuation_id.is_none() {
        reasons.push("phase_b_requires_phase_a_continuation".to_string());
    }
    if let Some(continuation_id) = continuation_id.as_ref() {
        let continuation_store =
            session_continuation_store::load_store(workflow_state_path, timestamp)?;
        if let Some(expected) = input.expected_session_continuation_store_revision {
            if expected != continuation_store.revision {
                reasons.push("session_continuation_store_revision_conflict".to_string());
            }
        }
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|continuation| continuation.continuation_id == *continuation_id);
        match continuation {
            Some(continuation) => {
                reasons.extend(product_new_session_authorization_binding_reasons(
                    request,
                    continuation,
                    &input.authorization,
                ));
            }
            None => reasons.push("phase_b_continuation_not_found".to_string()),
        }
    }

    reasons.sort();
    reasons.dedup();
    Ok((reasons, continuation_id))
}

fn product_new_session_authorization_binding_reasons(
    request: &RealExecutionProductCommandRequest,
    continuation: &crate::ControlledSessionContinuation,
    authorization: &H3RealNewSessionAuthorizationMatrix,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if authorization.operation_type != "new_session" {
        reasons.push("phase_b_authorization_operation_mismatch".to_string());
    }
    if continuation.operation_id != "new_session" {
        reasons.push("phase_b_continuation_operation_mismatch".to_string());
    }
    if !continuation.session_id.trim().is_empty() {
        reasons.push("phase_b_new_session_continuation_must_not_bind_session".to_string());
    }
    if request
        .project_root
        .as_deref()
        .is_some_and(|project_root| project_root != continuation.project_root)
    {
        reasons.push("phase_b_continuation_project_root_mismatch_request".to_string());
    }
    if authorization.project_root != continuation.project_root {
        reasons.push("phase_b_authorization_project_root_mismatch".to_string());
    }
    if authorization.target_cwd != continuation.target_cwd {
        reasons.push("phase_b_authorization_target_cwd_mismatch".to_string());
    }
    if Some(authorization.work_item_id.as_str()) != continuation.work_item_id.as_deref() {
        reasons.push("phase_b_authorization_work_item_mismatch".to_string());
    }
    if authorization.sandbox != continuation.sandbox {
        reasons.push("phase_b_authorization_sandbox_mismatch_continuation".to_string());
    }
    let request_sandbox = pcr9a_product_command_sandbox(request);
    if authorization.sandbox != request_sandbox {
        reasons.push("phase_b_authorization_sandbox_mismatch_request".to_string());
    }
    if authorization.prompt_summary != request.prompt_summary {
        reasons.push("phase_b_authorization_prompt_summary_mismatch".to_string());
    }
    if authorization.prompt_ref != request.prompt_ref {
        reasons.push("phase_b_authorization_prompt_ref_mismatch".to_string());
    }
    if authorization.prompt_sha256 != request.prompt_hash {
        reasons.push("phase_b_authorization_prompt_hash_mismatch".to_string());
    }
    if !same_string_set(
        &authorization.allowed_write_roots,
        &request.allowed_write_roots,
    ) {
        reasons.push("phase_b_authorization_allowed_write_roots_mismatch_request".to_string());
    }
    if !same_string_set(
        &authorization.allowed_write_roots,
        &continuation.allowed_write_roots,
    ) {
        reasons.push("phase_b_authorization_allowed_write_roots_mismatch_continuation".to_string());
    }
    reasons
}

fn pcr9a_phase_b_continuation_id(
    store: &RealExecutionProductCommandStore,
    product_command_id: &str,
) -> Option<String> {
    store
        .attempts
        .iter()
        .rev()
        .find(|attempt| {
            attempt.product_command_id == product_command_id
                && attempt.status == "phase_a_noop_completed"
                && attempt.continuation_id.is_some()
        })
        .and_then(|attempt| attempt.continuation_id.clone())
}

fn product_new_session_phase_b_product_attempt_status(
    authorization_status: &str,
    attempt: &SessionContinuationAttempt,
) -> String {
    if authorization_status != "h3_b_real_new_session_executed" {
        return authorization_status.to_string();
    }
    match attempt.status.as_str() {
        "succeeded" => "phase_b_real_new_session_executed".to_string(),
        "failed" => "runner_failed".to_string(),
        "timed_out"
        | "readback_unavailable"
        | "readback_failed"
        | "readback_timed_out"
        | "codex_state_error" => attempt.status.clone(),
        other => format!("phase_b_runner_status:{other}"),
    }
}

fn pcr9a_is_phase_b_product_attempt(attempt: &RealExecutionProductCommandAttempt) -> bool {
    attempt.attempt_id.contains(":phase-b:")
        || attempt.status.starts_with("phase_b")
        || attempt.runner_call_allowed
        || matches!(
            attempt.status.as_str(),
            "runner_failed" | "timed_out" | "codex_state_error"
        )
}

fn pcr9a_phase_b_product_attempt_status(
    authorization_status: &str,
    attempt: &SessionContinuationAttempt,
) -> String {
    if authorization_status != "phase_b_real_resume_executed" {
        return authorization_status.to_string();
    }
    match attempt.status.as_str() {
        "succeeded" => "phase_b_real_resume_executed".to_string(),
        "failed" => "runner_failed".to_string(),
        "timed_out"
        | "readback_unavailable"
        | "readback_failed"
        | "readback_timed_out"
        | "codex_state_error" => attempt.status.clone(),
        other => format!("phase_b_runner_status:{other}"),
    }
}

fn pcr9a_readback_from_phase_b(
    attempt: &SessionContinuationAttempt,
    codex_attempt: Option<&crate::CodexLocalExecutionAttempt>,
) -> RealExecutionProductCommandReadbackBoundary {
    if let Some(codex_attempt) = codex_attempt {
        return RealExecutionProductCommandReadbackBoundary {
            status: codex_attempt.readback_result.status.clone(),
            attempted: codex_attempt.readback_result.attempted,
            real_readback_performed: codex_attempt.readback_result.real_readback_performed,
            result_count: pcr2_readback_result_count(
                &codex_attempt.readback_result.status,
                codex_attempt.readback_result.result_count,
            ),
            unavailable_reason: codex_attempt.readback_result.unavailable_reason.clone(),
            warnings: crate::dedupe_strings(codex_attempt.readback_result.warnings.clone()),
        };
    }
    RealExecutionProductCommandReadbackBoundary {
        status: attempt.readback_summary.status.clone(),
        attempted: attempt.readback_summary.status != "not_attempted",
        real_readback_performed: false,
        result_count: pcr2_readback_result_count(
            &attempt.readback_summary.status,
            attempt.readback_summary.result_count,
        ),
        unavailable_reason: attempt.readback_summary.unavailable_reason.clone(),
        warnings: crate::dedupe_strings(attempt.readback_summary.warnings.clone()),
    }
}

fn pcr9a_readback_boundary(
    status: &str,
    proposed_result_count: Option<i64>,
) -> RealExecutionProductCommandReadbackBoundary {
    RealExecutionProductCommandReadbackBoundary {
        status: status.to_string(),
        attempted: false,
        real_readback_performed: false,
        result_count: pcr2_readback_result_count(status, proposed_result_count),
        unavailable_reason: Some("pcr9a_phase_b_no_readback_before_runner".to_string()),
        warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
    }
}

fn pcr9a_phase_b_product_attempt_warnings(
    authorization_status: &str,
    attempt: &SessionContinuationAttempt,
    codex_attempt: Option<&crate::CodexLocalExecutionAttempt>,
) -> Vec<String> {
    let mut warnings = vec![
        "pcr9a_product_command_phase_b_bridge".to_string(),
        "stdin_prompt_runtime_only_not_persisted".to_string(),
        format!("continuation_phase_b_status:{}", attempt.status),
        format!("authorization_status:{authorization_status}"),
    ];
    warnings.extend(attempt.warnings.clone());
    if let Some(codex_attempt) = codex_attempt {
        warnings.extend(codex_attempt.warnings.clone());
        warnings.push(format!(
            "codex_local_attempt_ref:{}",
            codex_attempt.attempt_id
        ));
    }
    crate::dedupe_strings(warnings)
}

fn pcr9a_phase_b_output_warnings(authorization_status: &str, warnings: Vec<String>) -> Vec<String> {
    let mut output_warnings = warnings;
    output_warnings.extend([
        "pcr9a_product_command_phase_b_bridge_recorded_product_attempt".to_string(),
        "stdin_prompt_runtime_only_not_persisted".to_string(),
        format!("authorization_status:{authorization_status}"),
    ]);
    crate::dedupe_strings(output_warnings)
}

fn pcr9a_phase_b_last_message_path(
    workflow_state_path: &Path,
    product_command_id: &str,
    timestamp: &str,
) -> Result<PathBuf, String> {
    let parent = workflow_state_path.parent().ok_or_else(|| {
        format!(
            "workflow state path has no parent; cannot derive product command Phase B last message: {}",
            workflow_state_path.display()
        )
    })?;
    Ok(parent
        .join("runtime")
        .join("product-command-phase-b")
        .join(format!(
            "{}.{}.last-message.txt",
            timestamp,
            short_hash(product_command_id)
        )))
}

fn pcr9a_product_command_sandbox(request: &RealExecutionProductCommandRequest) -> String {
    if request.sandbox.trim().is_empty() {
        "workspace-write".to_string()
    } else {
        request.sandbox.clone()
    }
}

fn same_string_set(left: &[String], right: &[String]) -> bool {
    left.iter().map(String::as_str).collect::<BTreeSet<_>>()
        == right.iter().map(String::as_str).collect::<BTreeSet<_>>()
}

fn pcr4_phase_a_blocked_reasons(
    store: &RealExecutionProductCommandStore,
    request: &RealExecutionProductCommandRequest,
    preview: &RealExecutionProductCommandPreview,
    input: &RunRealExecutionProductCommandPhaseAInput,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if input.actor_role.trim().is_empty() {
        reasons.push("phase_a_actor_role_missing".to_string());
    }
    if matches!(input.execution_decision.as_deref(), Some("rejected")) {
        reasons.push("phase_a_execution_decision_rejected".to_string());
    }
    if !matches!(
        input.execution_decision.as_deref(),
        None | Some("phase_a_noop") | Some("approved_for_phase_a")
    ) {
        reasons.push("phase_a_execution_decision_unsupported".to_string());
    }
    if !matches!(request.operation_id.as_str(), "resume" | "new_session") {
        reasons.push("phase_a_only_supports_resume_or_new_session_in_pcr4".to_string());
    }
    if request.operation_id == "resume"
        && request
            .target_session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        reasons.push("phase_a_resume_requires_target_session".to_string());
    }
    if request.operation_id == "new_session"
        && !request
            .target_session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        reasons.push("phase_a_new_session_must_not_bind_target_session".to_string());
    }
    if request
        .project_root
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("phase_a_project_root_missing".to_string());
    }
    if request.prompt_hash.len() != 64
        || !request
            .prompt_hash
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        reasons.push("phase_a_prompt_hash_invalid".to_string());
    }
    if !pcr3_preview_ready_for_approval(preview) {
        reasons.push("phase_a_preview_not_ready".to_string());
    }
    let approved_decision = store.decisions.iter().find(|decision| {
        decision.product_command_id == request.product_command_id && decision.decision == "approved"
    });
    match approved_decision {
        Some(decision) if decision.confirmed_by == "user" && decision.allowed_once => {}
        Some(_) => reasons.push("phase_a_approved_decision_must_be_user_once".to_string()),
        None => reasons.push("phase_a_requires_user_approved_decision".to_string()),
    }
    if store.attempts.iter().any(|attempt| {
        attempt.product_command_id == request.product_command_id
            && (attempt.status.contains("running")
                || attempt.status == "phase_a_noop_completed"
                || (attempt.runner_call_allowed && attempt.status.contains("phase_a")))
    }) {
        reasons.push("phase_a_duplicate_running_or_completed_attempt".to_string());
    }
    crate::dedupe_strings(reasons)
}

fn pcr4_session_continuation_preview(
    request: &RealExecutionProductCommandRequest,
    preview: &RealExecutionProductCommandPreview,
) -> SessionContinuationPreview {
    let project_root = request.project_root.clone();
    let target_cwd = request.project_root.clone();
    let readback_plan = if request.readback_plan.trim().is_empty() {
        "readback_unavailable_is_not_zero_results".to_string()
    } else {
        request.readback_plan.clone()
    };
    let sandbox = pcr9a_product_command_sandbox(request);
    SessionContinuationPreview {
        preview_id: format!(
            "session-continuation-preview:pcr4:{}",
            request.product_command_id
        ),
        adapter_id: request.adapter_id.clone(),
        operation_id: request.operation_id.clone(),
        target_session_id: request.target_session_id.clone(),
        target_session_title: request.target_session_id.as_ref().map(|session| {
            format!(
                "Product command session {}",
                session.chars().take(12).collect::<String>()
            )
        }),
        project_id: request.project_id.clone(),
        project_root: project_root.clone(),
        workflow_id: request.workflow_id.clone(),
        node_id: request.node_id.clone(),
        binding_id: Some(format!("binding:pcr4:{}", request.product_command_id)),
        work_item_id: request.work_item_id.clone(),
        target_cwd,
        allowed_write_roots_summary: request.allowed_write_roots.clone(),
        sandbox_summary: sandbox.clone(),
        prompt_source_kind: "product_command_prompt_ref".to_string(),
        prompt_summary: request.prompt_summary.clone(),
        readback_expectation: ReadbackExpectation {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![
                "session_continuation_attempt".to_string(),
                "runtime_log_ref".to_string(),
                "product_command_attempt".to_string(),
            ],
            unavailable_behavior: readback_plan.clone(),
            warnings: vec!["pcr4_phase_a_no_real_transcript_read".to_string()],
        },
        failure_handling: ContinuationFailureBoundary {
            timeout_policy: "phase_a_timeout_records_failure_without_retry".to_string(),
            retry_policy: "no_auto_retry".to_string(),
            failure_record: "product_command_attempt_and_session_continuation_attempt".to_string(),
            user_visible_behavior: "show_phase_a_boundary_and_readback_unavailable".to_string(),
            warnings: vec!["pcr4_no_auto_retry".to_string()],
        },
        audit_impact: ContinuationAuditImpact {
            impact_kind: "pcr4_phase_a_noop".to_string(),
            writes_attempt_in_e4: true,
            writes_dispatch_in_e4: false,
            writes_readback_in_e4: true,
            future_audit_requirement: "PCR9 Level B requires separate authorization".to_string(),
            warnings: vec!["product_command_audit_ref_links_to_continuation".to_string()],
        },
        provider_availability_summary: None,
        guard_result: SessionContinuationGuardResult {
            status: "ready_for_phase_a_noop".to_string(),
            severity: "medium".to_string(),
            blocks_execution: false,
            allows_preview: true,
            requires_user_confirmation: false,
            reasons: Vec::new(),
            required_fixes: Vec::new(),
            warnings: preview.guard_preview.warnings.clone(),
        },
        request: SessionContinuationRequest {
            adapter_id: request.adapter_id.clone(),
            operation_id: request.operation_id.clone(),
            project_id: request.project_id.clone(),
            project_root: request.project_root.clone(),
            workflow_id: request.workflow_id.clone(),
            node_id: request.node_id.clone(),
            session_id: request.target_session_id.clone(),
            work_item_id: request.work_item_id.clone(),
            target_cwd: request.project_root.clone(),
            allowed_write_roots: request.allowed_write_roots.clone(),
            sandbox,
            prompt_source_kind: "product_command_prompt_ref".to_string(),
            prompt_summary: request.prompt_summary.clone(),
            readback_strategy: "required".to_string(),
            requested_by: request.requested_by.clone(),
            user_confirmation_state: "confirmed".to_string(),
        },
        user_visible_warnings: crate::dedupe_strings(vec![
            "pcr4_phase_a_noop_only".to_string(),
            "prompt_not_sent".to_string(),
            "real_codex_executed_false".to_string(),
            "readback_unavailable_is_not_zero_results".to_string(),
        ]),
    }
}

fn pcr4_phase_a_authorization(
    request: &RealExecutionProductCommandRequest,
    continuation: &crate::ControlledSessionContinuation,
    input: &RunRealExecutionProductCommandPhaseAInput,
) -> H2RealResumeAuthorizationMatrix {
    H2RealResumeAuthorizationMatrix {
        operation_type: "resume".to_string(),
        test_project: request
            .project_id
            .clone()
            .unwrap_or_else(|| "pcr4-product-command".to_string()),
        project_root: continuation.project_root.clone(),
        target_cwd: continuation.target_cwd.clone(),
        target_session: continuation.session_id.clone(),
        prompt_summary: request.prompt_summary.clone(),
        prompt_sha256: request.prompt_hash.clone(),
        prompt_ref: request.prompt_ref.clone(),
        allowed_write_roots: request.allowed_write_roots.clone(),
        codex_home_scope: "not_accessed_in_pcr4_phase_a_noop".to_string(),
        sandbox: continuation.sandbox.clone(),
        timeout_ms: Some(input.timeout_ms.unwrap_or(120_000).max(1)),
        readback_plan: if request.readback_plan.trim().is_empty() {
            "readback_unavailable_is_not_zero_results".to_string()
        } else {
            request.readback_plan.clone()
        },
        evidence_path: "product-command-sidecar-managed-by-workbench".to_string(),
        rollback_plan: "PCR4 Phase A is no-op; product files and codex home are not written."
            .to_string(),
        user_confirmed_real_resume: true,
        global_supervisor_confirmed: true,
    }
}

fn pcr4_readback_from_session_attempt(
    attempt: &crate::SessionContinuationAttempt,
) -> RealExecutionProductCommandReadbackBoundary {
    RealExecutionProductCommandReadbackBoundary {
        status: attempt.readback_summary.status.clone(),
        attempted: attempt.readback_summary.status != "not_attempted",
        real_readback_performed: false,
        result_count: pcr2_readback_result_count(
            &attempt.readback_summary.status,
            attempt.readback_summary.result_count,
        ),
        unavailable_reason: attempt
            .readback_summary
            .unavailable_reason
            .clone()
            .or_else(|| Some("PCR4 Phase A does not read real transcript.".to_string())),
        warnings: crate::dedupe_strings(vec![
            "pcr4_phase_a_no_real_readback".to_string(),
            "readback_unavailable_is_not_zero_results".to_string(),
        ]),
    }
}

fn pcr4_readback_boundary(
    status: &str,
    proposed_result_count: Option<i64>,
) -> RealExecutionProductCommandReadbackBoundary {
    RealExecutionProductCommandReadbackBoundary {
        status: status.to_string(),
        attempted: false,
        real_readback_performed: false,
        result_count: pcr2_readback_result_count(status, proposed_result_count),
        unavailable_reason: Some("pcr4_phase_a_no_real_readback".to_string()),
        warnings: vec![
            "pcr4_phase_a_no_real_readback".to_string(),
            "readback_unavailable_is_not_zero_results".to_string(),
        ],
    }
}

fn real_execution_product_command_decision_output(
    status: &str,
    decision: Option<RealExecutionProductCommandDecision>,
    read_model: RealExecutionProductCommandReadModel,
    audit_ref: Option<String>,
    writes_product_command_sidecar: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
) -> RealExecutionProductCommandDecisionOutput {
    let store_revision = read_model.store_revision;
    let sidecar_path = read_model.sidecar_path.clone();
    RealExecutionProductCommandDecisionOutput {
        status: status.to_string(),
        decision,
        read_model,
        store_revision,
        sidecar_path,
        audit_ref,
        runner_call_allowed: false,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_product_command_sidecar,
        blocked_reasons: crate::dedupe_strings(blocked_reasons),
        warnings: crate::dedupe_strings(warnings),
    }
}

fn pcr3_is_terminal_decision(decision: &str) -> bool {
    matches!(decision, "approved" | "rejected" | "request_changes")
}

fn pcr3_preview_ready_for_approval(preview: &RealExecutionProductCommandPreview) -> bool {
    preview.blocked_reasons.is_empty()
        && preview.readiness.blocked_reasons.is_empty()
        && !preview.guard_preview.blocks_execution
        && preview.readiness.status == "ready_for_pcr3_decision_preview_only"
}

fn pcr3_decision_validation_blocked_reasons(
    request: &RealExecutionProductCommandRequest,
    decision: &RealExecutionProductCommandDecision,
) -> Vec<String> {
    let mut blocked_reasons = Vec::new();
    if decision.reason.trim().is_empty() {
        blocked_reasons.push("real_execution_product_command_decision_missing_reason".to_string());
    }
    if let Err(error) = validate_real_execution_product_command_decision(request, decision) {
        blocked_reasons.push(error);
    }
    crate::dedupe_strings(blocked_reasons)
}

fn pcr3_decision_warnings(
    input: &RecordRealExecutionProductCommandDecisionInput,
    warning: &str,
) -> Vec<String> {
    let mut warnings = vec![
        warning.to_string(),
        "pcr3_decision_never_calls_runner_or_codex".to_string(),
        format!("decision:{}", input.decision),
    ];
    if let Some(requested_by) = input.requested_by.as_ref() {
        if !requested_by.trim().is_empty() {
            warnings.push(format!("requested_by:{requested_by}"));
        }
    }
    crate::dedupe_strings(warnings)
}

fn h5_preview_input_for_source<'a>(
    source_kind: &str,
    h5_input: Option<&'a H5ProjectWorkflowDispatchPreviewInput>,
) -> Result<&'a H5ProjectWorkflowDispatchPreviewInput, String> {
    if source_kind != "h5_project_workflow_dispatch" {
        return Err(format!("unsupported_product_command_source:{source_kind}"));
    }
    h5_input.ok_or_else(|| "h5_dispatch_preview_required_for_product_command_source".to_string())
}

fn real_execution_product_command_preview_from_h5(
    h5_input: &H5ProjectWorkflowDispatchPreviewInput,
    h5_preview: &H5ProjectWorkflowDispatchPreview,
    requested_by: Option<&str>,
    created_at: &str,
) -> RealExecutionProductCommandPreview {
    let blocked_reasons = pcr2_effective_blocked_reasons(&h5_preview.blocked_reasons);
    let is_blocked = !blocked_reasons.is_empty();
    let product_command_id = format!("real-exec-command:{}", h5_preview.dispatch_id);
    let request = RealExecutionProductCommandRequest {
        product_command_id: product_command_id.clone(),
        command_family: "real_execution_product_command".to_string(),
        operation_id: h5_preview.operation_id.clone(),
        project_id: Some(h5_preview.project_id.clone()),
        project_root: Some(h5_input.project_root.clone()),
        workflow_id: Some(h5_preview.workflow_id.clone()),
        node_id: Some(h5_preview.workflow_node_id.clone()),
        work_item_id: Some(h5_preview.work_item_id.clone()),
        task_package_ref: h5_preview.task_package_id.clone(),
        memory_packet_ref: h5_preview
            .memory_packet
            .snapshot_id
            .clone()
            .or_else(|| h5_preview.memory_packet.fingerprint.clone()),
        adapter_id: h5_preview.permission_envelope.adapter_id.clone(),
        session_mode: if h5_preview.target_session_id.is_some() {
            "resume_existing_session".to_string()
        } else {
            "new_session_preview_only".to_string()
        },
        target_session_id: h5_preview.target_session_id.clone(),
        sandbox: h5_input
            .sandbox
            .clone()
            .unwrap_or_else(|| "workspace-write".to_string()),
        prompt_summary: h5_preview.permission_envelope.prompt_summary.clone(),
        prompt_ref: h5_preview.permission_envelope.prompt_ref.clone(),
        prompt_hash: h5_preview.permission_envelope.prompt_sha256.clone(),
        allowed_write_roots: h5_preview.permission_envelope.allowed_write_roots.clone(),
        denied_paths: h5_preview.permission_envelope.denied_paths.clone(),
        readback_plan: h5_preview.readback_boundary.unavailable_behavior.clone(),
        timeout_ms: None,
        requested_by: requested_by
            .map(str::to_string)
            .unwrap_or_else(|| h5_input.actor_id.clone()),
        created_at: created_at.to_string(),
    };
    let guard_warnings = h5_preview
        .codex_local_guard
        .as_ref()
        .map(|guard| {
            guard
                .reasons
                .iter()
                .map(|reason| format!("h1_guard_preview:{reason}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let runtime_log_refs = h5_preview
        .runtime_audit_preview
        .runtime_log_refs
        .iter()
        .map(|item| item.ref_id.clone())
        .collect::<Vec<_>>();
    let audit_refs = h5_preview
        .runtime_audit_preview
        .audit_refs
        .iter()
        .map(|item| item.ref_id.clone())
        .collect::<Vec<_>>();
    let mut warnings = h5_preview.warnings.clone();
    warnings.extend(h5_preview.permission_envelope.warnings.clone());
    warnings.extend(h5_preview.runtime_audit_preview.warnings.clone());
    warnings.extend(guard_warnings);
    warnings.push("pcr2_preview_mapped_from_h5_no_real_execution".to_string());
    warnings.push("pcr3_user_decision_required_before_execute".to_string());

    RealExecutionProductCommandPreview {
        preview_id: format!("real-exec-preview:{}", h5_preview.preview_id),
        request,
        permission_envelope: RealExecutionProductCommandPermissionEnvelope {
            envelope_id: format!("permission-envelope:{product_command_id}"),
            product_command_id,
            status: h5_preview.permission_envelope.status.clone(),
            explicit_user_confirmation_required: h5_preview
                .permission_envelope
                .explicit_approval_required,
            approved_for_real_execution: false,
            confirmed_by: None,
            allowed_write_roots: h5_preview.permission_envelope.allowed_write_roots.clone(),
            denied_paths: h5_preview.permission_envelope.denied_paths.clone(),
            risk_summary:
                "PCR2 preview only; PCR3 user decision and later execute gate are required."
                    .to_string(),
            warnings: h5_preview.permission_envelope.warnings.clone(),
        },
        readiness: RealExecutionProductCommandReadiness {
            status: if is_blocked {
                "blocked_pcr2_preview".to_string()
            } else {
                "ready_for_pcr3_decision_preview_only".to_string()
            },
            runner_call_allowed: false,
            level_b_authorization_required: true,
            blocked_reasons: blocked_reasons.clone(),
            warnings: vec![
                "pcr2_prepare_does_not_execute".to_string(),
                "pcr3_decision_required_before_any_runner".to_string(),
            ],
        },
        guard_preview: RealExecutionProductCommandGuardPreview {
            status: if is_blocked {
                "blocked_pcr2_preview".to_string()
            } else {
                "ready_for_pcr3_decision_no_runner".to_string()
            },
            runner_call_allowed: false,
            blocks_execution: is_blocked,
            reasons: blocked_reasons.clone(),
            required_fixes: h5_preview
                .codex_local_guard
                .as_ref()
                .map(|guard| guard.required_fixes.clone())
                .unwrap_or_default(),
            warnings: vec![
                "guard_inspected_only_no_phase_a_or_phase_b_call".to_string(),
                "user_confirmation_is_deferred_to_pcr3".to_string(),
            ],
        },
        diagnostics_summary: RealExecutionProductCommandDiagnosticsSummary {
            status: h5_preview.runtime_audit_preview.diagnostic_status.clone(),
            blocks_real_execution: h5_preview
                .blocked_reasons
                .iter()
                .any(|reason| reason == "diagnostics_blocking_degraded"),
            degraded_reasons: h5_preview.runtime_audit_preview.diagnostic_blockers.clone(),
            warnings: vec!["diagnostics_summary_mapped_from_h5_preview".to_string()],
        },
        duplicate_scope: RealExecutionProductCommandDuplicateScope {
            scope_id: format!(
                "h5:{}:{}:{}",
                h5_preview.workflow_node_id, h5_preview.work_item_id, h5_preview.dispatch_id
            ),
            active_attempt_count: h5_preview
                .codex_local_request
                .as_ref()
                .map(|request| request.active_attempts.len())
                .unwrap_or(0),
            duplicate_blocked: h5_preview
                .blocked_reasons
                .iter()
                .any(|reason| reason == "duplicate_dispatch_blocked"),
            warnings: vec!["duplicate_scope_mapped_from_h5_active_attempts".to_string()],
        },
        runtime_log_preview: RealExecutionProductCommandRuntimeLogPreview {
            status: "preview_only_not_written".to_string(),
            runtime_log_refs,
            redaction_status: "redacted_safe_summary_only".to_string(),
            warnings: vec!["runtime_log_refs_are_preview_only_not_written".to_string()],
        },
        audit_preview: RealExecutionProductCommandAuditPreview {
            status: "preview_only_not_written".to_string(),
            audit_refs,
            warnings: vec!["audit_refs_are_preview_only_not_written".to_string()],
        },
        readback_boundary: RealExecutionProductCommandReadbackBoundary {
            status: h5_preview.readback_boundary.status.clone(),
            attempted: false,
            real_readback_performed: false,
            result_count: pcr2_readback_result_count(
                &h5_preview.readback_boundary.status,
                h5_preview.readback_boundary.result_count,
            ),
            unavailable_reason: Some("pcr2_preview_no_real_readback".to_string()),
            warnings: h5_preview.readback_boundary.warnings.clone(),
        },
        warnings: crate::dedupe_strings(warnings),
        blocked_reasons,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
    }
}

fn real_execution_product_command_preview_from_codex_control(
    input: &CodexControlCommandInput,
    top_level_requested_by: Option<&str>,
    created_at: &str,
) -> RealExecutionProductCommandPreview {
    let blocked_reasons = codex_control_blocked_reasons(input);
    let is_blocked = !blocked_reasons.is_empty();
    let requested_by = top_level_requested_by
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| {
            input
                .requested_by
                .as_ref()
                .filter(|value| !value.trim().is_empty())
                .cloned()
        })
        .unwrap_or_else(|| "user".to_string());
    let normalized_operation = input.operation_id.trim().to_string();
    let operation_id = if normalized_operation.is_empty() {
        "resume".to_string()
    } else {
        normalized_operation
    };
    let normalized_sandbox = if input.sandbox.trim().is_empty() {
        "read-only".to_string()
    } else {
        input.sandbox.trim().to_string()
    };
    let product_command_id = format!(
        "real-exec-command:codex-control:{}",
        short_hash(&format!(
            "{}:{}:{}:{}:{}",
            created_at,
            input.project_root,
            operation_id,
            input.target_session_id.as_deref().unwrap_or("new-session"),
            input.prompt_hash
        ))
    );
    let request = RealExecutionProductCommandRequest {
        product_command_id: product_command_id.clone(),
        command_family: "real_execution_product_command".to_string(),
        operation_id: operation_id.clone(),
        project_id: input.project_id.clone(),
        project_root: Some(input.project_root.trim().to_string()),
        workflow_id: input.workflow_id.clone(),
        node_id: input.node_id.clone(),
        work_item_id: input.work_item_id.clone(),
        task_package_ref: input.task_package_ref.clone(),
        memory_packet_ref: input.memory_packet_ref.clone(),
        adapter_id: input.adapter_id.trim().to_string(),
        session_mode: codex_control_session_mode(input),
        target_session_id: input.target_session_id.clone(),
        sandbox: normalized_sandbox.clone(),
        prompt_summary: input.prompt_summary.trim().to_string(),
        prompt_ref: input.prompt_ref.trim().to_string(),
        prompt_hash: input.prompt_hash.trim().to_ascii_lowercase(),
        allowed_write_roots: crate::dedupe_strings(input.allowed_write_roots.clone()),
        denied_paths: crate::dedupe_strings(input.denied_paths.clone()),
        readback_plan: input.readback_plan.trim().to_string(),
        timeout_ms: input.timeout_ms,
        requested_by,
        created_at: created_at.to_string(),
    };
    let mut warnings = vec![
        "j1_codex_control_preview_not_h5_dispatch".to_string(),
        "prompt_body_runtime_only_not_persisted".to_string(),
        "j1a_no_real_codex_execution".to_string(),
        "phase_b_requires_separate_execution_point_authorization".to_string(),
        "memory_capture_observation_candidate_only_not_formal_memory".to_string(),
    ];
    if operation_id == "new_session" && input.session_mode.trim() != "new_session_execution_point" {
        warnings.push("new_session_deferred_in_j1a".to_string());
    } else if operation_id == "new_session" {
        warnings.push("new_session_execution_point_requires_phase_b_authorization".to_string());
    }
    let preview_status = if is_blocked {
        "blocked_or_deferred_codex_control_preview"
    } else {
        "ready_for_pcr3_decision_preview_only"
    };
    let guard_status = if is_blocked {
        "blocked_or_deferred_codex_control_preview"
    } else {
        "ready_for_pcr3_decision_no_runner"
    };

    RealExecutionProductCommandPreview {
        preview_id: format!("real-exec-preview:codex-control:{}", short_hash(&product_command_id)),
        permission_envelope: RealExecutionProductCommandPermissionEnvelope {
            envelope_id: format!("permission-envelope:{product_command_id}"),
            product_command_id: product_command_id.clone(),
            status: if is_blocked {
                "blocked_or_deferred".to_string()
            } else {
                "awaiting_user_confirmation".to_string()
            },
            explicit_user_confirmation_required: true,
            approved_for_real_execution: false,
            confirmed_by: None,
            allowed_write_roots: request.allowed_write_roots.clone(),
            denied_paths: request.denied_paths.clone(),
            risk_summary:
                "J1-A preview only; user decision, Phase A no-op, and later Level B authorization are required."
                    .to_string(),
            warnings: vec![
                "confirmed_by_user_required_for_high_impact_operation".to_string(),
                "prompt_body_not_stored_in_product_command".to_string(),
            ],
        },
        readiness: RealExecutionProductCommandReadiness {
            status: preview_status.to_string(),
            runner_call_allowed: false,
            level_b_authorization_required: true,
            blocked_reasons: blocked_reasons.clone(),
            warnings: vec![
                "j1a_prepare_does_not_execute".to_string(),
                "j1b_required_for_real_codex_resume".to_string(),
            ],
        },
        guard_preview: RealExecutionProductCommandGuardPreview {
            status: guard_status.to_string(),
            runner_call_allowed: false,
            blocks_execution: is_blocked,
            reasons: blocked_reasons.clone(),
            required_fixes: codex_control_required_fixes(&blocked_reasons),
            warnings: vec![
                "codex_control_guard_preview_only".to_string(),
                "no_cli_command_exposed_to_ordinary_ui".to_string(),
            ],
        },
        diagnostics_summary: RealExecutionProductCommandDiagnosticsSummary {
            status: "not_evaluated_j1a_preview".to_string(),
            blocks_real_execution: false,
            degraded_reasons: Vec::new(),
            warnings: vec!["diagnostics_not_probed_in_j1a_preview".to_string()],
        },
        duplicate_scope: RealExecutionProductCommandDuplicateScope {
            scope_id: format!(
                "codex-control:{}:{}:{}",
                operation_id,
                short_hash(&input.project_root),
                input
                    .target_session_id
                    .as_deref()
                    .map(short_hash)
                    .unwrap_or_else(|| "new-session".to_string())
            ),
            active_attempt_count: 0,
            duplicate_blocked: false,
            warnings: vec!["duplicate_scope_preview_only_checked_again_in_phase_a".to_string()],
        },
        runtime_log_preview: RealExecutionProductCommandRuntimeLogPreview {
            status: "preview_only_not_written".to_string(),
            runtime_log_refs: Vec::new(),
            redaction_status: "prompt_body_redacted_summary_ref_hash_only".to_string(),
            warnings: vec!["runtime_log_will_store_summary_ref_hash_only".to_string()],
        },
        audit_preview: RealExecutionProductCommandAuditPreview {
            status: "preview_only_not_written".to_string(),
            audit_refs: Vec::new(),
            warnings: vec!["audit_will_store_decision_and_refs_not_prompt_body".to_string()],
        },
        readback_boundary: RealExecutionProductCommandReadbackBoundary {
            status: "readback_unavailable".to_string(),
            attempted: false,
            real_readback_performed: false,
            result_count: None,
            unavailable_reason: Some("j1a_no_real_readback_before_phase_b".to_string()),
            warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
        },
        request,
        warnings: crate::dedupe_strings(warnings),
        blocked_reasons,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
    }
}

fn codex_control_blocked_reasons(input: &CodexControlCommandInput) -> Vec<String> {
    let mut reasons = Vec::new();
    let operation_id = input.operation_id.trim();
    if input.adapter_id.trim() != "codex-local" {
        reasons.push("codex_control_only_supports_codex_local_adapter".to_string());
    }
    if operation_id.is_empty() {
        reasons.push("codex_control_operation_missing".to_string());
    } else if operation_id == "new_session" {
        if input.session_mode.trim() != "new_session_execution_point" {
            reasons.push("codex_control_new_session_deferred_in_j1a".to_string());
        }
    } else if operation_id != "resume" {
        reasons.push("codex_control_operation_unsupported".to_string());
    }
    if input.project_root.trim().is_empty() {
        reasons.push("codex_control_project_root_missing".to_string());
    }
    if operation_id == "resume"
        && input
            .target_session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        reasons.push("codex_control_resume_requires_target_session".to_string());
    }
    if operation_id == "new_session"
        && !input
            .target_session_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        reasons.push("codex_control_new_session_must_not_bind_target_session".to_string());
    }
    if input.prompt_summary.trim().is_empty() {
        reasons.push("codex_control_prompt_summary_missing".to_string());
    }
    if input.prompt_ref.trim().is_empty() {
        reasons.push("codex_control_prompt_ref_missing".to_string());
    }
    if !valid_sha256_hex(input.prompt_hash.trim()) {
        reasons.push("codex_control_prompt_hash_invalid".to_string());
    }
    if input.readback_plan.trim().is_empty() {
        reasons.push("codex_control_readback_plan_missing".to_string());
    }
    if input.allowed_write_roots.is_empty() && input.sandbox.trim() != "read-only" {
        reasons.push("codex_control_allowed_write_roots_boundary_missing".to_string());
    } else if input.sandbox.trim() != "read-only" {
        // Non-read-only sandboxes must make the writable boundary explicit before later Level B.
        if input
            .allowed_write_roots
            .iter()
            .all(|root| root.trim().is_empty())
        {
            reasons.push("codex_control_workspace_write_requires_allowed_write_root".to_string());
        }
    }
    if !codex_control_denied_paths_cover_sensitive_boundary(&input.denied_paths) {
        reasons.push("codex_control_sensitive_denied_paths_missing".to_string());
    }
    if input.timeout_ms.is_some_and(|timeout| timeout <= 0) {
        reasons.push("codex_control_timeout_must_be_positive".to_string());
    }
    crate::dedupe_strings(reasons)
}

fn codex_control_session_mode(input: &CodexControlCommandInput) -> String {
    let explicit = input.session_mode.trim();
    if !explicit.is_empty() {
        return explicit.to_string();
    }
    if input.operation_id.trim() == "new_session" {
        "new_session_preview_only".to_string()
    } else {
        "resume_existing_session".to_string()
    }
}

fn codex_control_required_fixes(blocked_reasons: &[String]) -> Vec<String> {
    blocked_reasons
        .iter()
        .map(|reason| match reason.as_str() {
            "codex_control_new_session_deferred_in_j1a" => {
                "wait_for_j1b_or_later_new_session_task".to_string()
            }
            "codex_control_sensitive_denied_paths_missing" => {
                "include_secret_token_env_keychain_oauth_credential_transcript_rollout_denials"
                    .to_string()
            }
            "codex_control_prompt_hash_invalid" => {
                "provide_sha256_prompt_hash_without_persisting_prompt_body".to_string()
            }
            "codex_control_workspace_write_requires_allowed_write_root" => {
                "use_read_only_or_list_allowed_write_roots".to_string()
            }
            "codex_control_allowed_write_roots_boundary_missing" => {
                "provide_project_root_as_execution_boundary_even_for_read_only".to_string()
            }
            other => format!("fix:{other}"),
        })
        .collect()
}

fn valid_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn codex_control_denied_paths_cover_sensitive_boundary(denied_paths: &[String]) -> bool {
    let joined = denied_paths.join("\n").to_ascii_lowercase();
    [
        "secret",
        "token",
        ".env",
        "keychain",
        "oauth",
        "credential",
        "transcript",
        "rollout",
    ]
    .iter()
    .all(|needle| joined.contains(needle))
}

fn pcr2_effective_blocked_reasons(h5_reasons: &[String]) -> Vec<String> {
    let mut reasons = Vec::new();
    for reason in h5_reasons {
        if pcr2_h5_reason_deferred_to_decision(reason) {
            continue;
        }
        reasons.push(reason.clone());
        if let Some(normalized) = pcr2_normalized_blocked_reason(reason) {
            reasons.push(normalized.to_string());
        }
    }
    crate::dedupe_strings(reasons)
}

fn pcr2_h5_reason_deferred_to_decision(reason: &str) -> bool {
    matches!(reason, "h1_guard:user_confirmation_required")
}

fn pcr2_normalized_blocked_reason(reason: &str) -> Option<&'static str> {
    match reason {
        "task_memory_packet_snapshot_missing" => Some("memory_packet_missing"),
        "task_memory_packet_stale" => Some("memory_packet_stale"),
        "diagnostics_blocking_degraded" => Some("diagnostics_degraded"),
        "duplicate_dispatch_blocked" => Some("duplicate_active"),
        _ => None,
    }
}

pub(crate) fn pcr2_readback_result_count(
    status: &str,
    proposed_result_count: Option<i64>,
) -> Option<i64> {
    match status {
        "readback_unavailable"
        | "readback_failed"
        | "readback_timed_out"
        | "timed_out"
        | "not_attempted" => None,
        _ => proposed_result_count,
    }
}

fn write_real_execution_product_command_store_atomic(
    sidecar: &Path,
    store: &RealExecutionProductCommandStore,
    timestamp: &str,
) -> Result<(), String> {
    let parent = sidecar.parent().ok_or_else(|| {
        format!(
            "product command sidecar has no parent: {}",
            sidecar.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create product command sidecar directory failed {}: {error}",
            parent.display()
        )
    })?;
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("serialize product command sidecar failed: {error}"))?;
    let temp_path = sidecar.with_file_name(format!(
        ".{}.{}.tmp",
        PRODUCT_COMMAND_SIDECAR_NAME,
        timestamp.replace([':', '.'], "-")
    ));
    fs::write(&temp_path, text).map_err(|error| {
        format!(
            "write product command sidecar temp failed {}: {error}",
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "replace product command sidecar failed {}: {error}",
            sidecar.display()
        )
    })
}

pub(crate) fn pcr1_contract_preview(
    request: RealExecutionProductCommandRequest,
    preview_id: &str,
) -> RealExecutionProductCommandPreview {
    let blocked_reasons = vec![
        "pcr1_contract_only_no_execute_service".to_string(),
        "level_b_authorization_required".to_string(),
    ];
    let warnings = vec![
        "pcr1_preview_contract_only".to_string(),
        "prompt_body_not_stored".to_string(),
        "runner_call_not_allowed".to_string(),
    ];
    RealExecutionProductCommandPreview {
        preview_id: preview_id.to_string(),
        permission_envelope: RealExecutionProductCommandPermissionEnvelope {
            envelope_id: format!("permission-envelope:{preview_id}"),
            product_command_id: request.product_command_id.clone(),
            status: "awaiting_pcr2_preview_service".to_string(),
            explicit_user_confirmation_required: true,
            approved_for_real_execution: false,
            confirmed_by: None,
            allowed_write_roots: request.allowed_write_roots.clone(),
            denied_paths: request.denied_paths.clone(),
            risk_summary: "PCR1 defines the permission envelope only; no runner can be called."
                .to_string(),
            warnings: vec!["confirmed_by_user_required_for_level_b".to_string()],
        },
        readiness: RealExecutionProductCommandReadiness {
            status: "blocked_pcr1_contract_only".to_string(),
            runner_call_allowed: false,
            level_b_authorization_required: true,
            blocked_reasons: blocked_reasons.clone(),
            warnings: vec!["pcr2_prepare_preview_required".to_string()],
        },
        guard_preview: RealExecutionProductCommandGuardPreview {
            status: "blocked_contract_only".to_string(),
            runner_call_allowed: false,
            blocks_execution: true,
            reasons: blocked_reasons.clone(),
            required_fixes: vec!["complete_pcr2_pcr3_pcr4_before_any_execute_path".to_string()],
            warnings: vec!["old_runner_entry_not_opened".to_string()],
        },
        diagnostics_summary: RealExecutionProductCommandDiagnosticsSummary {
            status: "not_evaluated_in_pcr1".to_string(),
            blocks_real_execution: true,
            degraded_reasons: vec!["pcr1_does_not_run_diagnostics_probe".to_string()],
            warnings: vec!["pcr2_must_supply_diagnostics_summary".to_string()],
        },
        duplicate_scope: RealExecutionProductCommandDuplicateScope {
            scope_id: format!("duplicate-scope:{}", request.product_command_id),
            active_attempt_count: 0,
            duplicate_blocked: false,
            warnings: vec!["duplicate_scope_contract_only".to_string()],
        },
        runtime_log_preview: RealExecutionProductCommandRuntimeLogPreview {
            status: "preview_only_not_written".to_string(),
            runtime_log_refs: Vec::new(),
            redaction_status: "redacted_safe_summary_only".to_string(),
            warnings: vec!["runtime_log_not_written_in_pcr1".to_string()],
        },
        audit_preview: RealExecutionProductCommandAuditPreview {
            status: "preview_only_not_written".to_string(),
            audit_refs: Vec::new(),
            warnings: vec!["audit_not_written_in_pcr1".to_string()],
        },
        readback_boundary: pcr1_readback_boundary("readback_unavailable", None),
        request,
        warnings,
        blocked_reasons,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
    }
}

pub(crate) fn pcr1_blocked_attempt(
    request: &RealExecutionProductCommandRequest,
    attempt_id: &str,
    status: &str,
) -> RealExecutionProductCommandAttempt {
    RealExecutionProductCommandAttempt {
        attempt_id: attempt_id.to_string(),
        product_command_id: request.product_command_id.clone(),
        continuation_id: None,
        adapter_id: request.adapter_id.clone(),
        operation_id: request.operation_id.clone(),
        status: status.to_string(),
        started_at: request.created_at.clone(),
        completed_at: Some(request.created_at.clone()),
        runner_call_allowed: false,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        runtime_log_ref: None,
        audit_refs: Vec::new(),
        readback_summary: pcr1_readback_boundary("readback_unavailable", None),
        failure_reason: Some("pcr1_contract_only_runner_call_blocked".to_string()),
        warnings: vec![
            "pcr1_blocked_attempt_contract_fixture".to_string(),
            "runner_call_allowed_false".to_string(),
        ],
    }
}

pub(crate) fn pcr1_readback_boundary(
    status: &str,
    proposed_result_count: Option<i64>,
) -> RealExecutionProductCommandReadbackBoundary {
    RealExecutionProductCommandReadbackBoundary {
        status: status.to_string(),
        attempted: false,
        real_readback_performed: false,
        result_count: pcr1_readback_result_count(status, proposed_result_count),
        unavailable_reason: Some("pcr1_no_real_readback".to_string()),
        warnings: vec!["unknown_readback_result_count_must_remain_null".to_string()],
    }
}

pub(crate) fn pcr1_readback_result_count(
    status: &str,
    proposed_result_count: Option<i64>,
) -> Option<i64> {
    match status {
        "readback_unavailable" | "readback_failed" | "readback_timed_out" | "timed_out" => None,
        _ => proposed_result_count,
    }
}

pub(crate) fn validate_real_execution_product_command_decision(
    request: &RealExecutionProductCommandRequest,
    decision: &RealExecutionProductCommandDecision,
) -> Result<(), String> {
    if decision.product_command_id != request.product_command_id {
        return Err("real_execution_product_command_decision_command_mismatch".to_string());
    }
    if decision.decision.trim().is_empty() {
        return Err("real_execution_product_command_decision_missing_decision".to_string());
    }
    if decision.confirmed_by.trim().is_empty() {
        return Err("real_execution_product_command_decision_missing_confirmed_by".to_string());
    }
    if decision.decision == "approved" && high_impact_real_execution_request(request) {
        if decision.confirmed_by != "user" {
            return Err(
                "high_impact_real_execution_requires_confirmed_by_user_not_project_director"
                    .to_string(),
            );
        }
        if !decision.allowed_once {
            return Err("high_impact_real_execution_requires_allowed_once".to_string());
        }
        if decision.risk_acknowledgement.trim().is_empty() {
            return Err("high_impact_real_execution_requires_risk_acknowledgement".to_string());
        }
    }
    Ok(())
}

fn validate_pcr1_preview_safety(
    preview: &RealExecutionProductCommandPreview,
) -> Result<(), String> {
    if preview.prompt_sent
        || preview.real_codex_executed
        || preview.writes_codex_home
        || preview.writes_project_files
        || preview.writes_workbench_state
        || preview.readiness.runner_call_allowed
        || preview.guard_preview.runner_call_allowed
    {
        return Err(format!(
            "pcr1_preview_must_not_allow_real_execution:{}",
            preview.preview_id
        ));
    }
    validate_readback_boundary(&preview.readback_boundary)
}

fn validate_pcr1_attempt_safety(
    attempt: &RealExecutionProductCommandAttempt,
) -> Result<(), String> {
    if attempt.runner_call_allowed
        || attempt.prompt_sent
        || attempt.real_codex_executed
        || attempt.writes_codex_home
        || attempt.writes_project_files
    {
        return Err(format!(
            "pcr1_attempt_must_not_call_runner_or_codex:{}",
            attempt.attempt_id
        ));
    }
    validate_readback_boundary(&attempt.readback_summary)
}

fn validate_real_execution_product_command_attempt(
    attempt: &RealExecutionProductCommandAttempt,
) -> Result<(), String> {
    let has_real_phase_b_flags = attempt.runner_call_allowed
        || attempt.prompt_sent
        || attempt.real_codex_executed
        || attempt.writes_codex_home
        || attempt.writes_project_files;
    if !has_real_phase_b_flags {
        return validate_pcr1_attempt_safety(attempt);
    }
    if !pcr9a_is_phase_b_product_attempt(attempt) {
        return Err(format!(
            "real_execution_product_command_real_flags_require_phase_b_attempt:{}",
            attempt.attempt_id
        ));
    }
    if !attempt.runner_call_allowed {
        return Err(format!(
            "real_execution_product_command_phase_b_real_flags_require_runner_allowed:{}",
            attempt.attempt_id
        ));
    }
    if attempt
        .continuation_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(format!(
            "real_execution_product_command_phase_b_requires_continuation_id:{}",
            attempt.attempt_id
        ));
    }
    if attempt
        .runtime_log_ref
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(format!(
            "real_execution_product_command_phase_b_requires_runtime_log_ref:{}",
            attempt.attempt_id
        ));
    }
    if attempt.audit_refs.is_empty() {
        return Err(format!(
            "real_execution_product_command_phase_b_requires_audit_refs:{}",
            attempt.attempt_id
        ));
    }
    validate_readback_boundary(&attempt.readback_summary)
}

fn validate_readback_boundary(
    boundary: &RealExecutionProductCommandReadbackBoundary,
) -> Result<(), String> {
    if matches!(
        boundary.status.as_str(),
        "readback_unavailable" | "readback_failed" | "readback_timed_out" | "timed_out"
    ) && boundary.result_count.is_some()
    {
        return Err(format!(
            "unknown_readback_result_count_must_be_null:{}",
            boundary.status
        ));
    }
    Ok(())
}

fn high_impact_real_execution_request(request: &RealExecutionProductCommandRequest) -> bool {
    request.command_family.contains("real_execution")
        || request
            .command_family
            .contains("controlled_session_continuation")
        || matches!(
            request.operation_id.as_str(),
            "execute" | "resume" | "new_session" | "send_message"
        )
}

fn failure_stop_retry_summary_from_store(
    store: &RealExecutionProductCommandStore,
) -> RealExecutionProductCommandFailureStopRetrySummary {
    let mut items = Vec::<RealExecutionProductCommandFailureStopRetryItem>::new();
    let mut retry_requires_new_user_confirmation = false;

    for decision in &store.decisions {
        if matches!(decision.decision.as_str(), "rejected" | "request_changes") {
            push_failure_stop_retry_item(
                &mut items,
                "user_rejected",
                1,
                Some(format!("decision:{}", decision.decision_id)),
                None,
                vec![format!("decision:{}", decision.decision)],
            );
            retry_requires_new_user_confirmation = true;
        }
        if text_mentions_manual_stop(&decision.reason) {
            push_failure_stop_retry_item(
                &mut items,
                "manual_stop_requested",
                1,
                Some(format!("decision:{}", decision.decision_id)),
                None,
                vec!["manual_stop_requested_from_decision_reason".to_string()],
            );
        }
    }

    for preview in &store.previews {
        let special_blocked = preview_has_blocked_reason(
            preview,
            &[
                "memory_packet_missing",
                "memory_packet_stale",
                "task_memory_packet_snapshot_missing",
                "task_memory_packet_stale",
            ],
        );
        if special_blocked {
            push_failure_stop_retry_item(
                &mut items,
                "blocked_stale_memory",
                1,
                Some(format!("preview:{}", preview.preview_id)),
                None,
                preview.blocked_reasons.clone(),
            );
        }

        let diagnostics_blocked = preview.diagnostics_summary.blocks_real_execution
            || preview_has_blocked_reason(
                preview,
                &["diagnostics_degraded", "diagnostics_blocking_degraded"],
            );
        if diagnostics_blocked {
            let mut warnings = preview.diagnostics_summary.degraded_reasons.clone();
            warnings.extend(preview.diagnostics_summary.warnings.clone());
            push_failure_stop_retry_item(
                &mut items,
                "blocked_by_diagnostics",
                1,
                Some(format!("preview:{}", preview.preview_id)),
                None,
                warnings,
            );
        }

        let duplicate_blocked = preview.duplicate_scope.duplicate_blocked
            || preview_has_blocked_reason(
                preview,
                &["duplicate_active", "duplicate_dispatch_blocked"],
            );
        if duplicate_blocked {
            push_failure_stop_retry_item(
                &mut items,
                "duplicate_blocked",
                1,
                Some(format!("preview:{}", preview.preview_id)),
                None,
                preview.duplicate_scope.warnings.clone(),
            );
        }

        let preview_blocked = !preview.blocked_reasons.is_empty()
            || !preview.readiness.blocked_reasons.is_empty()
            || preview.guard_preview.blocks_execution
            || !preview.guard_preview.reasons.is_empty();
        if preview_blocked && !(special_blocked || diagnostics_blocked || duplicate_blocked) {
            let mut warnings = preview.blocked_reasons.clone();
            warnings.extend(preview.readiness.blocked_reasons.clone());
            warnings.extend(preview.guard_preview.reasons.clone());
            push_failure_stop_retry_item(
                &mut items,
                "blocked_by_guard",
                1,
                Some(format!("preview:{}", preview.preview_id)),
                None,
                warnings,
            );
        }
    }

    for attempt in &store.attempts {
        let readback_status = attempt.readback_summary.status.as_str();
        if attempt.status == "timed_out"
            || matches!(readback_status, "timed_out" | "readback_timed_out")
        {
            push_failure_stop_retry_item(
                &mut items,
                "timed_out",
                1,
                Some(format!("attempt:{}", attempt.attempt_id)),
                None,
                attempt.warnings.clone(),
            );
            retry_requires_new_user_confirmation = true;
        }
        if readback_status == "readback_unavailable" {
            push_failure_stop_retry_item(
                &mut items,
                "readback_unavailable",
                1,
                Some(format!("attempt:{}", attempt.attempt_id)),
                None,
                attempt.readback_summary.warnings.clone(),
            );
        }
        if readback_status == "readback_failed" {
            push_failure_stop_retry_item(
                &mut items,
                "readback_failed",
                1,
                Some(format!("attempt:{}", attempt.attempt_id)),
                None,
                attempt.readback_summary.warnings.clone(),
            );
            retry_requires_new_user_confirmation = true;
        }
        if attempt.status == "failed"
            || attempt.status == "failed_stub"
            || attempt.status == "runner_failed"
            || attempt.status == "codex_state_error"
            || attempt
                .failure_reason
                .as_deref()
                .is_some_and(non_blocking_failure_reason)
        {
            let mut warnings = attempt.warnings.clone();
            if let Some(reason) = attempt.failure_reason.as_ref() {
                warnings.push(format!("failure_reason:{reason}"));
            }
            let kind = if attempt.status == "codex_state_error"
                || warnings
                    .iter()
                    .any(|warning| warning.contains("codex_state"))
            {
                "codex_state_error"
            } else {
                "runner_failed"
            };
            push_failure_stop_retry_item(
                &mut items,
                kind,
                1,
                Some(format!("attempt:{}", attempt.attempt_id)),
                None,
                warnings,
            );
            retry_requires_new_user_confirmation = true;
        }
        if attempt
            .warnings
            .iter()
            .any(|warning| text_mentions_manual_stop(warning))
            || attempt
                .failure_reason
                .as_deref()
                .is_some_and(text_mentions_manual_stop)
        {
            push_failure_stop_retry_item(
                &mut items,
                "manual_stop_requested",
                1,
                Some(format!("attempt:{}", attempt.attempt_id)),
                None,
                attempt.warnings.clone(),
            );
        }
    }

    if retry_requires_new_user_confirmation {
        push_failure_stop_retry_item(
            &mut items,
            "retry_requires_new_user_confirmation",
            1,
            Some("product_command_retry_boundary".to_string()),
            None,
            vec!["pcr7_no_auto_retry_requires_new_user_confirmation".to_string()],
        );
    }

    let failure_count = item_count_for_kinds(
        &items,
        &[
            "user_rejected",
            "timed_out",
            "readback_failed",
            "runner_failed",
            "codex_state_error",
        ],
    );
    let blocked_count = item_count_for_kinds(
        &items,
        &[
            "blocked_by_guard",
            "blocked_by_diagnostics",
            "duplicate_blocked",
            "blocked_stale_memory",
        ],
    );
    let readback_issue_count = item_count_for_kinds(
        &items,
        &["timed_out", "readback_unavailable", "readback_failed"],
    );
    let manual_stop_requested_count = item_count_for_kinds(&items, &["manual_stop_requested"]);
    let mut warnings = vec!["pcr7_failure_stop_retry_summary_is_read_model_only".to_string()];
    if retry_requires_new_user_confirmation {
        warnings.push("retry_requires_new_user_confirmation_no_auto_retry".to_string());
    }
    warnings = crate::dedupe_strings(warnings);

    RealExecutionProductCommandFailureStopRetrySummary {
        schema_version: "real_execution_product_command_failure_stop_retry.v1".to_string(),
        item_count: items.len(),
        failure_count,
        blocked_count,
        readback_issue_count,
        manual_stop_requested_count,
        retry_requires_new_user_confirmation,
        items,
        warnings,
    }
}

fn push_failure_stop_retry_item(
    items: &mut Vec<RealExecutionProductCommandFailureStopRetryItem>,
    kind: &str,
    count: usize,
    source_ref: Option<String>,
    result_count: Option<i64>,
    warnings: Vec<String>,
) {
    let (title, summary, severity, requires_new_user_confirmation) =
        failure_stop_retry_product_copy(kind);
    if let Some(existing) = items.iter_mut().find(|item| item.kind == kind) {
        existing.count += count;
        existing.requires_new_user_confirmation |= requires_new_user_confirmation;
        existing.result_count = merge_result_count(existing.result_count, result_count);
        if severity_rank(severity) > severity_rank(&existing.severity) {
            existing.severity = severity.to_string();
        }
        if let Some(source_ref) = source_ref {
            existing.source_refs.push(source_ref);
        }
        existing.warnings.extend(warnings);
        existing.source_refs = crate::dedupe_strings(existing.source_refs.clone());
        existing.warnings = crate::dedupe_strings(existing.warnings.clone());
        return;
    }

    let source_refs = source_ref.into_iter().collect::<Vec<_>>();
    items.push(RealExecutionProductCommandFailureStopRetryItem {
        kind: kind.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        count,
        severity: severity.to_string(),
        requires_new_user_confirmation,
        result_count,
        source_refs,
        warnings: crate::dedupe_strings(warnings),
    });
}

fn failure_stop_retry_product_copy(kind: &str) -> (&'static str, &'static str, &'static str, bool) {
    match kind {
        "user_rejected" => (
            "用户已拒绝",
            "用户拒绝或要求修改，当前不能继续执行。",
            "high",
            true,
        ),
        "blocked_by_guard" => (
            "被安全边界阻断",
            "安全边界或准备状态阻断了统一执行链路。",
            "high",
            false,
        ),
        "blocked_by_diagnostics" => (
            "被诊断阻断",
            "诊断降级或阻断状态要求先查看诊断。",
            "high",
            false,
        ),
        "duplicate_blocked" => (
            "重复执行已阻断",
            "已有重复命令或运行记录，不能并行继续。",
            "medium",
            false,
        ),
        "blocked_stale_memory" => (
            "记忆包缺失或过期",
            "任务记忆包缺失或过期，重新确认前需要先检查。",
            "medium",
            false,
        ),
        "timed_out" => (
            "执行超时",
            "执行或读回超时，不能解释为已经完成停止。",
            "high",
            true,
        ),
        "readback_unavailable" => (
            "读回不可用",
            "没有可用读回来源，结果数未知。",
            "medium",
            false,
        ),
        "readback_failed" => (
            "读回失败",
            "读回尝试失败或不可信，结果数未知。",
            "high",
            true,
        ),
        "runner_failed" => ("运行失败", "运行记录失败，不能自动重新执行。", "high", true),
        "codex_state_error" => (
            "Codex 状态不可写",
            "Codex 原生状态库不可写或权限不足，需要在可写 Codex 环境中重新确认后重试。",
            "high",
            true,
        ),
        "manual_stop_requested" => (
            "停止请求需受控处理",
            "用户请求停止仅作为产品状态，本任务不会停止真实进程。",
            "medium",
            false,
        ),
        "retry_requires_new_user_confirmation" => (
            "需要重新确认",
            "再次执行前需要新的用户确认；不会自动重试。",
            "high",
            true,
        ),
        _ => ("需要查看", "统一执行链路需要人工查看。", "medium", false),
    }
}

fn preview_has_blocked_reason(
    preview: &RealExecutionProductCommandPreview,
    needles: &[&str],
) -> bool {
    preview
        .blocked_reasons
        .iter()
        .chain(preview.readiness.blocked_reasons.iter())
        .chain(preview.guard_preview.reasons.iter())
        .any(|reason| {
            needles
                .iter()
                .any(|needle| reason == needle || reason.contains(needle))
        })
}

fn non_blocking_failure_reason(reason: &str) -> bool {
    let normalized = reason.to_ascii_lowercase();
    ![
        "blocked",
        "guard",
        "diagnostic",
        "duplicate",
        "memory_packet",
        "manual_stop",
        "stop request",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn text_mentions_manual_stop(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    normalized.contains("manual_stop")
        || normalized.contains("manual stop")
        || normalized.contains("stop_requested")
        || normalized.contains("stop request")
        || text.contains("停止")
}

fn item_count_for_kinds(
    items: &[RealExecutionProductCommandFailureStopRetryItem],
    kinds: &[&str],
) -> usize {
    items
        .iter()
        .filter(|item| kinds.contains(&item.kind.as_str()))
        .map(|item| item.count)
        .sum()
}

fn merge_result_count(current: Option<i64>, next: Option<i64>) -> Option<i64> {
    match (current, next) {
        (Some(current), Some(next)) => Some(current + next),
        (None, None) => None,
        _ => None,
    }
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

fn read_model_from_store(
    store: &RealExecutionProductCommandStore,
    store_available: bool,
    sidecar_path: Option<PathBuf>,
    extra_warnings: Vec<String>,
) -> RealExecutionProductCommandReadModel {
    let completed_decisions = store
        .decisions
        .iter()
        .filter(|decision| decision.decision != "pending")
        .map(|decision| decision.product_command_id.as_str())
        .collect::<BTreeSet<_>>();
    let pending_decision_count = store
        .commands
        .iter()
        .filter(|command| !completed_decisions.contains(command.product_command_id.as_str()))
        .count()
        + store
            .decisions
            .iter()
            .filter(|decision| decision.decision == "pending")
            .count();
    let running_attempt_count = store
        .attempts
        .iter()
        .filter(|attempt| matches!(attempt.status.as_str(), "queued" | "running"))
        .count();
    let blocked_attempt_count = store
        .attempts
        .iter()
        .filter(|attempt| attempt.status.contains("blocked") || !attempt.runner_call_allowed)
        .count();
    let mut warnings = store.warnings.clone();
    warnings.extend(extra_warnings);
    warnings.push("ordinary_product_entry_is_readiness_only_in_pcr1".to_string());
    warnings.push("legacy_entries_remain_blocked".to_string());
    warnings.push("runner_entries_require_pcr9_level_b_authorization".to_string());
    warnings.sort();
    warnings.dedup();
    let failure_stop_retry_summary = failure_stop_retry_summary_from_store(store);

    RealExecutionProductCommandReadModel {
        schema_version: PRODUCT_COMMAND_STORE_SCHEMA_VERSION.to_string(),
        sidecar_name: PRODUCT_COMMAND_SIDECAR_NAME.to_string(),
        sidecar_path: sidecar_path.map(|path| path.display().to_string()),
        store_available,
        store_revision: store.revision,
        command_count: store.commands.len(),
        pending_decision_count,
        running_attempt_count,
        blocked_attempt_count,
        last_attempt_status: store.attempts.last().map(|attempt| attempt.status.clone()),
        failure_stop_retry_summary,
        ordinary_product_entry_status: "readiness_only_pcr1_no_execute".to_string(),
        legacy_entry_status: "legacy_sealed_blocked_not_product_command".to_string(),
        runner_entry_status: "internal_runner_blocked_until_unified_execute_and_level_b"
            .to_string(),
        level_b_authorization_required: true,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const PCR9A_FAKE_PROMPT_BODY: &str =
        "PCR9A fake runtime prompt for product command Phase B tests.\n";
    const PCR9A_WRONG_PROMPT_BODY: &str =
        "PCR9A mismatched runtime prompt that must not reach the runner.\n";

    #[test]
    fn legacy_and_mcp_boundaries_do_not_claim_h5_unified_execution() {
        let legacy = legacy_product_command_boundary_spec("run_workflow_machine");
        assert!(!legacy.h5_unified_product_command);
        assert!(!legacy.product_routing_allows_real_execution);
        assert!(legacy.deprecated);
        assert!(legacy
            .warnings
            .contains(&"real_execution_command_gate_v1".to_string()));

        let mcp = mcp_canvas_real_execution_blocked_message("canvas_start_run");
        assert!(mcp.contains("mcp_canvas_real_execution_blocked:canvas_start_run"));
        assert!(mcp.contains("unified product command boundary"));
    }

    #[test]
    fn real_execution_gate_blocks_before_runner_for_level_a_cases() {
        for (name, input, expected_status) in [
            (
                "user rejected",
                RealExecutionCommandGateInput {
                    command_name: "run_controlled_session_continuation_real_resume_phase_b",
                    command_family: "controlled_session_continuation",
                    operation_id: "resume",
                    h5_unified_product_command: true,
                    authorization_complete: true,
                    user_rejected: true,
                    duplicate_blocked: false,
                    guard_blocked: false,
                    diagnostics_blocked: false,
                    stale_memory_blocked: false,
                    readback_required: true,
                },
                "user_rejected",
            ),
            (
                "duplicate",
                RealExecutionCommandGateInput {
                    command_name: "run_controlled_session_continuation_real_resume_phase_b",
                    command_family: "controlled_session_continuation",
                    operation_id: "resume",
                    h5_unified_product_command: true,
                    authorization_complete: true,
                    user_rejected: false,
                    duplicate_blocked: true,
                    guard_blocked: false,
                    diagnostics_blocked: false,
                    stale_memory_blocked: false,
                    readback_required: true,
                },
                "duplicate_blocked",
            ),
            (
                "guard",
                RealExecutionCommandGateInput {
                    command_name: "run_controlled_session_continuation_real_resume_phase_b",
                    command_family: "controlled_session_continuation",
                    operation_id: "resume",
                    h5_unified_product_command: true,
                    authorization_complete: true,
                    user_rejected: false,
                    duplicate_blocked: false,
                    guard_blocked: true,
                    diagnostics_blocked: false,
                    stale_memory_blocked: false,
                    readback_required: true,
                },
                "blocked_by_guard",
            ),
            (
                "diagnostics",
                RealExecutionCommandGateInput {
                    command_name: "preview_h5_project_workflow_dispatch",
                    command_family: "h5_project_workflow_dispatch",
                    operation_id: "resume",
                    h5_unified_product_command: true,
                    authorization_complete: true,
                    user_rejected: false,
                    duplicate_blocked: false,
                    guard_blocked: false,
                    diagnostics_blocked: true,
                    stale_memory_blocked: false,
                    readback_required: true,
                },
                "blocked_by_diagnostics",
            ),
            (
                "readback missing",
                RealExecutionCommandGateInput {
                    command_name: "run_controlled_session_continuation_real_resume_phase_b",
                    command_family: "controlled_session_continuation",
                    operation_id: "resume",
                    h5_unified_product_command: true,
                    authorization_complete: true,
                    user_rejected: false,
                    duplicate_blocked: false,
                    guard_blocked: false,
                    diagnostics_blocked: false,
                    stale_memory_blocked: false,
                    readback_required: false,
                },
                "blocked_waiting_readback_plan",
            ),
        ] {
            let decision = decide_real_execution_command(input);
            assert_eq!(decision.status, expected_status, "{name}");
            assert!(!decision.runner_call_allowed, "{name}");
            assert!(!decision.product_routing_allows_real_execution, "{name}");
            assert!(decision
                .warnings
                .contains(&"runner_call_blocked_by_unified_product_gate".to_string()));
        }
    }

    #[test]
    fn real_execution_gate_allows_runner_only_after_unified_boundary_passes() {
        let decision = decide_real_execution_command(RealExecutionCommandGateInput {
            command_name: "run_controlled_session_continuation_real_resume_phase_b",
            command_family: "controlled_session_continuation",
            operation_id: "resume",
            h5_unified_product_command: true,
            authorization_complete: true,
            user_rejected: false,
            duplicate_blocked: false,
            guard_blocked: false,
            diagnostics_blocked: false,
            stale_memory_blocked: false,
            readback_required: true,
        });

        assert_eq!(decision.status, "authorized_for_real_runner");
        assert!(decision.runner_call_allowed);
        assert!(decision.product_routing_allows_real_execution);
        assert!(decision
            .warnings
            .contains(&"runner_call_allowed_after_unified_product_gate".to_string()));
    }

    #[test]
    fn pcr1_preview_and_attempt_defaults_do_not_call_runner_or_codex() {
        let request = sample_product_command_request();
        let preview = pcr1_contract_preview(request.clone(), "preview:pcr1");
        assert!(!preview.prompt_sent);
        assert!(!preview.real_codex_executed);
        assert!(!preview.writes_codex_home);
        assert!(!preview.writes_project_files);
        assert!(!preview.writes_workbench_state);
        assert!(!preview.readiness.runner_call_allowed);
        assert!(!preview.guard_preview.runner_call_allowed);
        assert_eq!(preview.readback_boundary.result_count, None);
        validate_pcr1_preview_safety(&preview).expect("preview should be PCR1-safe");

        let attempt = pcr1_blocked_attempt(&request, "attempt:pcr1", "blocked_pcr1_contract_only");
        assert!(!attempt.runner_call_allowed);
        assert!(!attempt.prompt_sent);
        assert!(!attempt.real_codex_executed);
        assert!(!attempt.writes_codex_home);
        assert!(!attempt.writes_project_files);
        assert_eq!(attempt.readback_summary.result_count, None);
        validate_pcr1_attempt_safety(&attempt).expect("attempt should be PCR1-safe");

        let mut project_write_attempt = attempt.clone();
        project_write_attempt.writes_project_files = true;
        let error = validate_pcr1_attempt_safety(&project_write_attempt)
            .expect_err("PCR1 attempt must not claim project file writes");
        assert!(error.contains("pcr1_attempt_must_not_call_runner_or_codex"));
    }

    #[test]
    fn pcr1_readback_unknown_statuses_keep_null_result_count() {
        for status in [
            "readback_unavailable",
            "readback_failed",
            "readback_timed_out",
            "timed_out",
        ] {
            assert_eq!(
                pcr1_readback_result_count(status, Some(0)),
                None,
                "{status}"
            );
            assert_eq!(
                pcr1_readback_boundary(status, Some(0)).result_count,
                None,
                "{status}"
            );
        }
        assert_eq!(
            pcr1_readback_result_count("readback_succeeded", Some(1)),
            Some(1)
        );
    }

    #[test]
    fn high_impact_decision_requires_user_confirmation() {
        let request = sample_product_command_request();
        let mut decision = sample_product_command_decision(&request);
        decision.confirmed_by = "project_director".to_string();
        let error = validate_real_execution_product_command_decision(&request, &decision)
            .expect_err("project director cannot approve high-impact execution");
        assert!(error.contains("confirmed_by_user"));

        decision.confirmed_by = "user".to_string();
        decision.allowed_once = false;
        let error = validate_real_execution_product_command_decision(&request, &decision)
            .expect_err("high-impact execution must be allowed once");
        assert!(error.contains("allowed_once"));

        decision.allowed_once = true;
        validate_real_execution_product_command_decision(&request, &decision)
            .expect("user one-shot decision should be valid");
    }

    #[test]
    fn product_command_store_summary_keeps_legacy_and_runner_blocked() {
        let request = sample_product_command_request();
        let preview = pcr1_contract_preview(request.clone(), "preview:pcr1");
        let attempt = pcr1_blocked_attempt(&request, "attempt:pcr1", "blocked_pcr1_contract_only");
        let store = RealExecutionProductCommandStore {
            schema_version: PRODUCT_COMMAND_STORE_SCHEMA_VERSION.to_string(),
            revision: 1,
            created_at: "2026-06-09T00:00:00Z".to_string(),
            updated_at: "2026-06-09T00:00:00Z".to_string(),
            last_write_id: None,
            commands: vec![request],
            previews: vec![preview],
            decisions: Vec::new(),
            attempts: vec![attempt],
            audit_refs: Vec::new(),
            warnings: Vec::new(),
        };
        validate_real_execution_product_command_store(&store).expect("store should validate");

        let summary = read_model_from_store(&store, true, None, Vec::new());
        assert_eq!(summary.command_count, 1);
        assert_eq!(summary.pending_decision_count, 1);
        assert_eq!(summary.blocked_attempt_count, 1);
        assert_eq!(
            summary.legacy_entry_status,
            "legacy_sealed_blocked_not_product_command"
        );
        assert!(summary.level_b_authorization_required);
    }

    #[test]
    fn pcr2_preview_maps_h5_preview_without_real_execution() {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let input = PreviewRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(dir.display().to_string(), None)),
            codex_control: None,
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();
        let preview = preview_real_execution_product_command_at(&path, &input).unwrap();

        assert_eq!(
            preview.request.command_family,
            "real_execution_product_command"
        );
        assert_eq!(preview.request.operation_id, "resume");
        assert!(!preview.prompt_sent);
        assert!(!preview.real_codex_executed);
        assert!(!preview.writes_codex_home);
        assert!(!preview.writes_project_files);
        assert!(!preview.writes_workbench_state);
        assert_eq!(preview.readback_boundary.result_count, None);
        assert!(
            preview.blocked_reasons.is_empty(),
            "{:?}",
            preview.blocked_reasons
        );
        assert!(
            !sidecar.exists(),
            "preview must not write product command sidecar"
        );
    }

    #[test]
    fn pcr2_prepare_writes_product_command_sidecar_only_when_ready() {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let input = PrepareRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(dir.display().to_string(), None)),
            codex_control: None,
            expected_store_revision: Some(0),
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let output = prepare_real_execution_product_command_at(&path, &input).unwrap();
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();
        let (store, available, _) =
            load_real_execution_product_command_store(&path, "2026-06-09T00:00:01Z").unwrap();

        assert_eq!(output.status, "prepared");
        assert!(output.writes_product_command_sidecar);
        assert!(sidecar.exists());
        assert!(available);
        assert_eq!(store.revision, 1);
        assert_eq!(store.commands.len(), 1);
        assert_eq!(store.previews.len(), 1);
        assert!(store.decisions.is_empty());
        assert!(store.attempts.is_empty());
        assert!(store.audit_refs.is_empty());
        assert!(!dir.join("runtime-log.v1.json").exists());
        assert_eq!(output.read_model.store_revision, 1);
    }

    #[test]
    fn j1_codex_control_resume_preview_prepare_confirm_and_phase_a_stay_no_real_execution() {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let prompt_body = "J1 runtime-only prompt body must not be persisted.";
        let preview_input = PreviewRealExecutionProductCommandInput {
            source_kind: "codex_control".to_string(),
            h5_dispatch_preview: None,
            codex_control: Some(j1_codex_control_input(
                dir.display().to_string(),
                "resume",
                Some("session:j1"),
                prompt_body,
            )),
            requested_by: Some("user".to_string()),
            created_at: Some("2026-06-09T10:00:00Z".to_string()),
        };
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();
        let preview = preview_real_execution_product_command_at(&path, &preview_input).unwrap();

        assert_eq!(
            preview.request.command_family,
            "real_execution_product_command"
        );
        assert!(preview.request.product_command_id.contains("codex-control"));
        assert_eq!(preview.request.operation_id, "resume");
        assert_eq!(preview.request.adapter_id, "codex-local");
        assert_eq!(preview.request.prompt_hash, sha256_hex(prompt_body));
        assert_eq!(
            preview.readiness.status,
            "ready_for_pcr3_decision_preview_only"
        );
        assert!(
            preview.blocked_reasons.is_empty(),
            "{:?}",
            preview.blocked_reasons
        );
        assert!(preview
            .warnings
            .contains(&"j1_codex_control_preview_not_h5_dispatch".to_string()));
        assert!(!preview.prompt_sent);
        assert!(!preview.real_codex_executed);
        assert!(!preview.writes_codex_home);
        assert!(!preview.writes_project_files);
        assert!(!sidecar.exists(), "preview must not write sidecar");

        let prepare = prepare_real_execution_product_command_at(
            &path,
            &PrepareRealExecutionProductCommandInput {
                source_kind: "codex_control".to_string(),
                h5_dispatch_preview: None,
                codex_control: Some(j1_codex_control_input(
                    dir.display().to_string(),
                    "resume",
                    Some("session:j1"),
                    prompt_body,
                )),
                expected_store_revision: Some(0),
                requested_by: Some("user".to_string()),
                created_at: Some("2026-06-09T10:00:00Z".to_string()),
            },
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(prepare.writes_product_command_sidecar);
        assert!(
            sidecar.exists(),
            "prepare should write product command sidecar"
        );

        let command_id = prepare.product_command_id.clone().unwrap();
        let decision = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input(&command_id, "approved", prepare.store_revision),
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");
        assert_eq!(decision.decision.as_ref().unwrap().confirmed_by, "user");
        assert!(!decision.runner_call_allowed);

        let phase_a = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, decision.store_revision),
            "2026-06-09T10:00:04Z",
            "j1-codex-control-phase-a",
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");
        assert!(phase_a.writes_product_command_sidecar);
        assert!(phase_a.writes_continuation_sidecar);
        assert!(phase_a.writes_runtime_log);
        assert!(!phase_a.runner_call_allowed);
        assert!(!phase_a.prompt_sent);
        assert!(!phase_a.real_codex_executed);
        assert!(!phase_a.writes_codex_home);
        assert!(!phase_a.writes_project_files);
        assert_eq!(phase_a.readback_summary.result_count, None);

        let product_text = std::fs::read_to_string(sidecar).unwrap();
        let continuation_sidecar = crate::session_continuation_store::sidecar_path(&path).unwrap();
        let continuation_text = std::fs::read_to_string(continuation_sidecar).unwrap();
        let runtime_sidecar = crate::runtime_log_store::sidecar_path(&path).unwrap();
        let runtime_text = std::fs::read_to_string(runtime_sidecar).unwrap();
        assert!(!product_text.contains(prompt_body));
        assert!(!continuation_text.contains(prompt_body));
        assert!(!runtime_text.contains(prompt_body));
        assert!(product_text.contains("\"command_family\": \"real_execution_product_command\""));
        assert!(!product_text.contains("\"command_family\": \"codex_control\""));
    }

    #[test]
    fn j1_codex_control_new_session_is_deferred_and_not_prepared() {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let prompt_body = "J1 new session preview only.";
        let prepare = prepare_real_execution_product_command_at(
            &path,
            &PrepareRealExecutionProductCommandInput {
                source_kind: "codex_control".to_string(),
                h5_dispatch_preview: None,
                codex_control: Some(j1_codex_control_input(
                    dir.display().to_string(),
                    "new_session",
                    None,
                    prompt_body,
                )),
                expected_store_revision: Some(0),
                requested_by: Some("user".to_string()),
                created_at: Some("2026-06-09T10:01:00Z".to_string()),
            },
        )
        .unwrap();
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();

        assert_eq!(prepare.status, "blocked_not_prepared");
        assert!(!prepare.writes_product_command_sidecar);
        assert!(prepare
            .blocked_reasons
            .contains(&"codex_control_new_session_deferred_in_j1a".to_string()));
        assert_eq!(prepare.preview.request.operation_id, "new_session");
        assert!(
            !sidecar.exists(),
            "deferred new_session must not write sidecar"
        );
    }

    #[test]
    fn pcr2_preview_blocks_missing_memory_packet() {
        pcr2_blocked_preview_case(
            Pcr2MemoryFixture::Missing,
            false,
            None,
            &[
                "task_memory_packet_snapshot_missing",
                "memory_packet_missing",
            ],
        );
    }

    #[test]
    fn pcr2_preview_blocks_stale_memory_packet() {
        pcr2_blocked_preview_case(
            Pcr2MemoryFixture::Stale,
            false,
            None,
            &["task_memory_packet_stale", "memory_packet_stale"],
        );
    }

    #[test]
    fn pcr2_preview_blocks_diagnostics_degraded() {
        pcr2_blocked_preview_case(
            Pcr2MemoryFixture::Fresh,
            false,
            Some(crate::H5DiagnosticSummaryInput {
                overall_severity: "blocked".to_string(),
                blocked_count: 1,
                degraded_states: vec![crate::H5DiagnosticDegradedStateInput {
                    kind: "diagnostics:blocking_fixture".to_string(),
                    blocks_real_execution: true,
                }],
            }),
            &["diagnostics_blocking_degraded", "diagnostics_degraded"],
        );
    }

    #[test]
    fn pcr2_preview_blocks_duplicate_active() {
        pcr2_blocked_preview_case(
            Pcr2MemoryFixture::Fresh,
            true,
            None,
            &["duplicate_dispatch_blocked", "duplicate_active"],
        );
    }

    #[test]
    fn pcr2_prepare_rejects_store_revision_conflict() {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let input = PrepareRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(dir.display().to_string(), None)),
            codex_control: None,
            expected_store_revision: Some(99),
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let output = prepare_real_execution_product_command_at(&path, &input).unwrap();
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();

        assert_eq!(output.status, "store_conflict");
        assert!(!output.writes_product_command_sidecar);
        assert!(!sidecar.exists(), "conflict must not create sidecar");
    }

    #[test]
    fn pcr2_readback_unknown_keeps_result_count_null() {
        for status in [
            "readback_unavailable",
            "readback_failed",
            "readback_timed_out",
            "timed_out",
            "not_attempted",
        ] {
            assert_eq!(
                pcr2_readback_result_count(status, Some(0)),
                None,
                "{status}"
            );
        }
    }

    #[test]
    fn pcr7_failure_stop_retry_summary_covers_product_states_without_runner() {
        let request = sample_product_command_request();
        let mut user_rejected = sample_product_command_decision(&request);
        user_rejected.decision_id = "decision:pcr7:user_rejected".to_string();
        user_rejected.decision = "rejected".to_string();
        user_rejected.allowed_once = false;
        user_rejected.risk_acknowledgement.clear();

        let mut manual_stop = sample_product_command_decision(&request);
        manual_stop.decision_id = "decision:pcr7:manual_stop".to_string();
        manual_stop.reason = "manual_stop request from user".to_string();

        let mut guard_preview = sample_pcr7_preview(&request, "preview:pcr7:guard");
        guard_preview.blocked_reasons = vec!["guard_policy_blocked".to_string()];
        guard_preview.readiness.blocked_reasons = guard_preview.blocked_reasons.clone();
        guard_preview.guard_preview.blocks_execution = true;
        guard_preview.guard_preview.reasons = guard_preview.blocked_reasons.clone();

        let mut diagnostics_preview = sample_pcr7_preview(&request, "preview:pcr7:diagnostics");
        diagnostics_preview.blocked_reasons = vec!["diagnostics_degraded".to_string()];
        diagnostics_preview
            .diagnostics_summary
            .blocks_real_execution = true;
        diagnostics_preview.diagnostics_summary.degraded_reasons =
            vec!["diagnostics:blocking_fixture".to_string()];

        let mut duplicate_preview = sample_pcr7_preview(&request, "preview:pcr7:duplicate");
        duplicate_preview.blocked_reasons = vec!["duplicate_active".to_string()];
        duplicate_preview.duplicate_scope.duplicate_blocked = true;
        duplicate_preview.duplicate_scope.active_attempt_count = 1;

        let mut stale_memory_preview = sample_pcr7_preview(&request, "preview:pcr7:memory");
        stale_memory_preview.blocked_reasons = vec!["memory_packet_stale".to_string()];
        stale_memory_preview.readiness.blocked_reasons =
            stale_memory_preview.blocked_reasons.clone();

        let mut timed_out = pcr1_blocked_attempt(&request, "attempt:pcr7:timed_out", "timed_out");
        timed_out.readback_summary = pcr1_readback_boundary("readback_timed_out", Some(0));

        let mut readback_unavailable =
            pcr1_blocked_attempt(&request, "attempt:pcr7:readback_unavailable", "blocked");
        readback_unavailable.readback_summary =
            pcr1_readback_boundary("readback_unavailable", Some(0));

        let mut readback_failed =
            pcr1_blocked_attempt(&request, "attempt:pcr7:readback_failed", "blocked");
        readback_failed.readback_summary = pcr1_readback_boundary("readback_failed", Some(0));

        let mut runner_failed =
            pcr1_blocked_attempt(&request, "attempt:pcr7:runner_failed", "failed_stub");
        runner_failed.failure_reason = Some("runner_failed_fixture".to_string());

        let mut codex_state_error = pcr1_blocked_attempt(
            &request,
            "attempt:pcr7:codex_state_error",
            "codex_state_error",
        );
        codex_state_error.failure_reason = Some(
            "Codex native state db at /Users/yoyi/.codex/state_5.sqlite is readonly".to_string(),
        );
        codex_state_error
            .warnings
            .push("codex_state_readonly_or_permission_denied".to_string());

        let store = RealExecutionProductCommandStore {
            schema_version: PRODUCT_COMMAND_STORE_SCHEMA_VERSION.to_string(),
            revision: 7,
            created_at: "2026-06-09T00:00:00Z".to_string(),
            updated_at: "2026-06-09T00:00:00Z".to_string(),
            last_write_id: Some("pcr7-read-model-fixture".to_string()),
            commands: vec![request],
            previews: vec![
                guard_preview,
                diagnostics_preview,
                duplicate_preview,
                stale_memory_preview,
            ],
            decisions: vec![user_rejected, manual_stop],
            attempts: vec![
                timed_out,
                readback_unavailable,
                readback_failed,
                runner_failed,
                codex_state_error,
            ],
            audit_refs: Vec::new(),
            warnings: Vec::new(),
        };
        let read_model = read_model_from_store(&store, true, None, Vec::new());
        let summary = read_model.failure_stop_retry_summary;
        let kinds = summary
            .items
            .iter()
            .map(|item| item.kind.as_str())
            .collect::<BTreeSet<_>>();

        for expected in [
            "user_rejected",
            "blocked_by_guard",
            "blocked_by_diagnostics",
            "duplicate_blocked",
            "blocked_stale_memory",
            "timed_out",
            "readback_unavailable",
            "readback_failed",
            "runner_failed",
            "codex_state_error",
            "manual_stop_requested",
            "retry_requires_new_user_confirmation",
        ] {
            assert!(kinds.contains(expected), "missing {expected}: {kinds:?}");
        }
        assert_eq!(summary.item_count, 12);
        assert!(summary.retry_requires_new_user_confirmation);
        assert_eq!(summary.manual_stop_requested_count, 1);
        assert_eq!(
            store.attempts.len(),
            5,
            "read model must not write retry attempt"
        );

        for item in summary.items.iter().filter(|item| {
            matches!(
                item.kind.as_str(),
                "readback_unavailable" | "readback_failed" | "timed_out"
            )
        }) {
            assert_eq!(item.result_count, None, "{}", item.kind);
        }
        let retry_item = summary
            .items
            .iter()
            .find(|item| item.kind == "retry_requires_new_user_confirmation")
            .unwrap();
        assert!(retry_item.requires_new_user_confirmation);
        assert!(retry_item.summary.contains("不会自动重试"));
    }

    #[test]
    fn pcr3_record_decision_writes_approved_decision_and_audit_ref() {
        let (dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let input = pcr3_decision_input(&command_id, "approved", revision);
        let output = record_real_execution_product_command_decision_at(&path, &input).unwrap();
        let (store, _, _) =
            load_real_execution_product_command_store(&path, "2026-06-09T00:00:03Z").unwrap();

        assert_eq!(output.status, "decision_recorded");
        assert_eq!(output.store_revision, revision + 1);
        assert_eq!(output.read_model.pending_decision_count, 0);
        assert!(!output.runner_call_allowed);
        assert!(!output.prompt_sent);
        assert!(!output.real_codex_executed);
        assert!(!output.writes_codex_home);
        assert!(!output.writes_project_files);
        assert!(output.writes_product_command_sidecar);
        assert_eq!(store.commands.len(), 1);
        assert_eq!(store.previews.len(), 1);
        assert_eq!(store.decisions.len(), 1);
        assert!(store.attempts.is_empty());
        assert_eq!(store.audit_refs.len(), 1);
        assert_eq!(output.audit_ref, Some(store.audit_refs[0].clone()));
        assert!(!dir.join("runtime-log.v1.json").exists());
    }

    #[test]
    fn pcr3_confirm_requires_user_allowed_once_and_risk_acknowledgement() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let input = ConfirmRealExecutionProductCommandInput {
            product_command_id: command_id,
            expected_store_revision: Some(revision),
            confirmed_by: "user".to_string(),
            risk_acknowledgement: "I understand this is a one-shot Level B permission record."
                .to_string(),
            allowed_once: true,
            reason: "User approved this prepared command only.".to_string(),
            requested_by: Some("project_director".to_string()),
            confirmed_at: Some("2026-06-09T00:00:03Z".to_string()),
        };
        let output = confirm_real_execution_product_command_at(&path, &input).unwrap();

        assert_eq!(output.status, "decision_recorded");
        assert_eq!(output.decision.as_ref().unwrap().decision, "approved");
        assert_eq!(output.decision.as_ref().unwrap().confirmed_by, "user");
        assert!(!output.runner_call_allowed);
        assert!(!output.real_codex_executed);
    }

    #[test]
    fn pcr3_high_impact_approved_rejects_project_director() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let before = pcr3_sidecar_text(&path);
        let mut input = pcr3_decision_input(&command_id, "approved", revision);
        input.confirmed_by = "project_director".to_string();
        let output = record_real_execution_product_command_decision_at(&path, &input).unwrap();

        assert_eq!(output.status, "blocked");
        assert!(output
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("confirmed_by_user")));
        assert!(!output.writes_product_command_sidecar);
        assert_eq!(pcr3_sidecar_text(&path), before);
    }

    #[test]
    fn pcr3_high_impact_approved_rejects_allowed_once_false() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let before = pcr3_sidecar_text(&path);
        let mut input = pcr3_decision_input(&command_id, "approved", revision);
        input.allowed_once = false;
        let output = record_real_execution_product_command_decision_at(&path, &input).unwrap();

        assert_eq!(output.status, "blocked");
        assert!(output
            .blocked_reasons
            .contains(&"high_impact_real_execution_requires_allowed_once".to_string()));
        assert_eq!(pcr3_sidecar_text(&path), before);
    }

    #[test]
    fn pcr3_high_impact_approved_rejects_empty_risk_acknowledgement() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let before = pcr3_sidecar_text(&path);
        let mut input = pcr3_decision_input(&command_id, "approved", revision);
        input.risk_acknowledgement = " ".to_string();
        let output = record_real_execution_product_command_decision_at(&path, &input).unwrap();

        assert_eq!(output.status, "blocked");
        assert!(output
            .blocked_reasons
            .contains(&"high_impact_real_execution_requires_risk_acknowledgement".to_string()));
        assert_eq!(pcr3_sidecar_text(&path), before);
    }

    #[test]
    fn pcr3_rejected_and_request_changes_write_decision_without_runner_or_attempt() {
        for decision in ["rejected", "request_changes"] {
            let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
            let mut input = pcr3_decision_input(&command_id, decision, revision);
            input.confirmed_by = "project_director".to_string();
            input.allowed_once = false;
            input.risk_acknowledgement = String::new();
            let output = record_real_execution_product_command_decision_at(&path, &input).unwrap();
            let (store, _, _) =
                load_real_execution_product_command_store(&path, "2026-06-09T00:00:03Z").unwrap();

            assert_eq!(output.status, "decision_recorded", "{decision}");
            assert_eq!(store.decisions[0].decision, decision);
            assert_eq!(store.audit_refs.len(), 1);
            assert!(store.attempts.is_empty());
            assert!(!output.runner_call_allowed);
            assert!(!output.prompt_sent);
            assert!(!output.real_codex_executed);
        }
    }

    #[test]
    fn pcr3_revision_conflict_unknown_command_and_damaged_json_do_not_write() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let before = pcr3_sidecar_text(&path);
        let output = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input(&command_id, "approved", revision + 9),
        )
        .unwrap();
        assert_eq!(output.status, "store_conflict");
        assert_eq!(pcr3_sidecar_text(&path), before);

        let output = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input("real-exec-command:unknown", "approved", revision),
        )
        .unwrap();
        assert_eq!(output.status, "blocked");
        assert!(output
            .blocked_reasons
            .contains(&"product_command_not_prepared".to_string()));
        assert_eq!(pcr3_sidecar_text(&path), before);

        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();
        std::fs::write(&sidecar, "{not json").unwrap();
        let err = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input(&command_id, "approved", revision),
        )
        .expect_err("damaged sidecar must return parse error");
        assert!(err.contains("parse real execution product command sidecar failed"));
        assert_eq!(std::fs::read_to_string(sidecar).unwrap(), "{not json");
    }

    #[test]
    fn pcr3_duplicate_terminal_decision_and_blocked_preview_approval_do_not_write() {
        let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let input = pcr3_decision_input(&command_id, "approved", revision);
        record_real_execution_product_command_decision_at(&path, &input).unwrap();
        let before = pcr3_sidecar_text(&path);
        let duplicate = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input(&command_id, "rejected", revision + 1),
        )
        .unwrap();
        assert_eq!(duplicate.status, "blocked");
        assert!(duplicate
            .blocked_reasons
            .contains(&"product_command_decision_already_recorded".to_string()));
        assert_eq!(pcr3_sidecar_text(&path), before);

        let (_dir, blocked_path) = pcr2_fixture_state(Pcr2MemoryFixture::Missing, false);
        let preview_input = PreviewRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(
                blocked_path.parent().unwrap().display().to_string(),
                None,
            )),
            codex_control: None,
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let blocked_preview =
            preview_real_execution_product_command_at(&blocked_path, &preview_input).unwrap();
        let sidecar = real_execution_product_command_sidecar_path(&blocked_path).unwrap();
        let store = RealExecutionProductCommandStore {
            schema_version: PRODUCT_COMMAND_STORE_SCHEMA_VERSION.to_string(),
            revision: 1,
            created_at: "2026-06-09T00:00:00Z".to_string(),
            updated_at: "2026-06-09T00:00:00Z".to_string(),
            last_write_id: None,
            commands: vec![blocked_preview.request.clone()],
            previews: vec![blocked_preview.clone()],
            decisions: Vec::new(),
            attempts: Vec::new(),
            audit_refs: Vec::new(),
            warnings: Vec::new(),
        };
        std::fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap()).unwrap();
        let before_blocked = std::fs::read_to_string(&sidecar).unwrap();
        let output = record_real_execution_product_command_decision_at(
            &blocked_path,
            &pcr3_decision_input(&blocked_preview.request.product_command_id, "approved", 1),
        )
        .unwrap();
        assert_eq!(output.status, "blocked");
        assert!(output
            .blocked_reasons
            .contains(&"product_command_preview_not_ready_for_approval".to_string()));
        assert_eq!(std::fs::read_to_string(sidecar).unwrap(), before_blocked);
    }

    #[test]
    fn pcr4_phase_a_noop_writes_trace_refs_without_real_codex() {
        let (_dir, path, command_id, revision) = pcr4_approved_command_fixture();
        let output = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, revision),
            "2026-06-09T00:00:04Z",
            "pcr4-phase-a-success",
        )
        .unwrap();
        let (product_store, _, _) =
            load_real_execution_product_command_store(&path, "2026-06-09T00:00:05Z").unwrap();
        let continuation_store =
            crate::session_continuation_store::load_store(&path, "2026-06-09T00:00:05Z").unwrap();
        let runtime_store = crate::runtime_log_store::load_store(&path).unwrap();

        assert_eq!(output.status, "phase_a_completed");
        assert!(output.writes_product_command_sidecar);
        assert!(output.writes_continuation_sidecar);
        assert!(output.writes_runtime_log);
        assert!(!output.runner_call_allowed);
        assert!(!output.prompt_sent);
        assert!(!output.real_codex_executed);
        assert!(!output.writes_codex_home);
        assert!(!output.writes_project_files);
        assert_eq!(output.readback_summary.result_count, None);

        assert_eq!(product_store.attempts.len(), 1);
        let product_attempt = &product_store.attempts[0];
        assert_eq!(product_attempt.status, "phase_a_noop_completed");
        assert!(!product_attempt.runner_call_allowed);
        assert!(!product_attempt.prompt_sent);
        assert!(!product_attempt.real_codex_executed);
        assert!(!product_attempt.writes_codex_home);
        assert!(!product_attempt.writes_project_files);
        assert_eq!(product_attempt.readback_summary.result_count, None);
        assert_eq!(
            product_attempt.continuation_id.as_deref(),
            output.continuation_id.as_deref()
        );
        assert_eq!(
            product_attempt.runtime_log_ref.as_deref(),
            output.runtime_log_ref.as_deref()
        );

        assert_eq!(continuation_store.continuations.len(), 1);
        assert_eq!(continuation_store.attempts.len(), 1);
        let continuation_attempt = &continuation_store.attempts[0];
        assert_eq!(
            output.continuation_attempt_id.as_deref(),
            Some(continuation_attempt.attempt_id.as_str())
        );
        assert!(!continuation_attempt.prompt_sent);
        assert!(!continuation_attempt.real_codex_executed);
        assert!(!continuation_attempt.writes_codex_home);
        assert!(continuation_attempt.writes_workbench_state);
        assert_eq!(continuation_attempt.readback_summary.result_count, None);

        let expected_dispatch_ref = format!(
            "runtime-log:dispatch-attempt:{}",
            continuation_attempt.attempt_id
        );
        assert_eq!(
            output.runtime_log_ref.as_deref(),
            Some(expected_dispatch_ref.as_str())
        );
        assert!(runtime_store
            .entries
            .iter()
            .any(|entry| entry.entry_id == expected_dispatch_ref));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.entry_id == format!("runtime-log:readback:{}", continuation_attempt.attempt_id)
        }));
    }

    #[test]
    fn pcr9a_phase_b_fake_runner_writes_product_attempt_without_persisting_prompt_body() {
        let (_dir, path, command_id, revision) = pcr4_approved_command_fixture();
        let phase_a = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, revision),
            "2026-06-09T00:00:04Z",
            "pcr9a-phase-a-before-b",
        )
        .unwrap();
        let input = pcr9a_phase_b_input(
            &path,
            &command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            PCR9A_FAKE_PROMPT_BODY,
        );
        let last_message_path = path
            .parent()
            .unwrap()
            .join("runtime")
            .join("pcr9a-test.last-message.txt");
        let output = run_real_execution_product_command_phase_b_with_runner(
            &path,
            &input,
            "2026-06-09T00:00:06Z",
            "pcr9a-phase-b-success",
            &last_message_path,
            &Pcr9aFakePhaseBRunner,
        )
        .unwrap();
        let (product_store, _, product_sidecar) =
            load_real_execution_product_command_store(&path, "2026-06-09T00:00:07Z").unwrap();
        let continuation_sidecar = crate::session_continuation_store::sidecar_path(&path).unwrap();
        let runtime_sidecar = crate::runtime_log_store::sidecar_path(&path).unwrap();

        assert_eq!(output.status, "phase_b_completed");
        assert!(output.writes_product_command_sidecar);
        assert!(output.writes_continuation_sidecar);
        assert!(output.writes_runtime_log);
        assert!(output.runner_call_allowed);
        assert!(output.prompt_sent);
        assert!(output.real_codex_executed);
        assert!(output.writes_codex_home);
        assert!(!output.writes_project_files);
        assert_eq!(output.readback_summary.result_count, Some(1));

        assert_eq!(product_store.attempts.len(), 2);
        let product_attempt = product_store.attempts.last().unwrap();
        assert_eq!(product_attempt.status, "phase_b_real_resume_executed");
        assert!(product_attempt.runner_call_allowed);
        assert!(product_attempt.prompt_sent);
        assert!(product_attempt.real_codex_executed);
        assert!(product_attempt.writes_codex_home);
        assert!(!product_attempt.writes_project_files);
        assert_eq!(
            product_attempt.continuation_id.as_deref(),
            output.continuation_id.as_deref()
        );
        assert_eq!(
            product_attempt.runtime_log_ref.as_deref(),
            output.runtime_log_ref.as_deref()
        );
        assert!(!product_attempt.audit_refs.is_empty());

        let continuation_store =
            crate::session_continuation_store::load_store(&path, "2026-06-09T00:00:07Z").unwrap();
        assert_eq!(continuation_store.attempts.len(), 2);
        let continuation_attempt = continuation_store.attempts.last().unwrap();
        assert_eq!(
            output.continuation_attempt_id.as_deref(),
            Some(continuation_attempt.attempt_id.as_str())
        );
        assert!(continuation_attempt.prompt_sent);
        assert!(continuation_attempt.real_codex_executed);
        assert!(continuation_attempt.writes_codex_home);

        let product_text = std::fs::read_to_string(product_sidecar).unwrap();
        let continuation_text = std::fs::read_to_string(continuation_sidecar).unwrap();
        let runtime_text = std::fs::read_to_string(runtime_sidecar).unwrap();
        assert!(!product_text.contains(PCR9A_FAKE_PROMPT_BODY));
        assert!(!continuation_text.contains(PCR9A_FAKE_PROMPT_BODY));
        assert!(!runtime_text.contains(PCR9A_FAKE_PROMPT_BODY));
    }

    #[test]
    fn pcr9a_phase_b_blocks_product_gate_cases_without_runner_or_write() {
        let cases = [
            ("missing_decision", Pcr9aBlockedCase::MissingDecision),
            ("prompt_hash_mismatch", Pcr9aBlockedCase::PromptHashMismatch),
            ("planned_adapter", Pcr9aBlockedCase::PlannedAdapter),
            (
                "unsupported_operation",
                Pcr9aBlockedCase::UnsupportedOperation,
            ),
        ];
        for (name, case) in cases {
            let (_dir, path, command_id, revision) = match case {
                Pcr9aBlockedCase::MissingDecision => pcr3_prepared_command_fixture(),
                _ => pcr4_approved_command_fixture(),
            };
            if !matches!(case, Pcr9aBlockedCase::MissingDecision) {
                let phase_a = run_real_execution_product_command_phase_a_at(
                    &path,
                    &pcr4_phase_a_input(&command_id, revision),
                    "2026-06-09T00:00:04Z",
                    &format!("pcr9a-phase-a-{name}"),
                )
                .unwrap();
                mutate_pcr9a_blocked_case(&path, case);
                let before = pcr3_sidecar_text(&path);
                let input = pcr9a_phase_b_input(
                    &path,
                    &command_id,
                    phase_a.product_command_store_revision,
                    phase_a.session_continuation_store_revision.unwrap(),
                    if matches!(case, Pcr9aBlockedCase::PromptHashMismatch) {
                        PCR9A_WRONG_PROMPT_BODY
                    } else {
                        PCR9A_FAKE_PROMPT_BODY
                    },
                );
                let output = run_real_execution_product_command_phase_b_with_runner(
                    &path,
                    &input,
                    "2026-06-09T00:00:06Z",
                    &format!("pcr9a-blocked-{name}"),
                    &path
                        .parent()
                        .unwrap()
                        .join(format!("{name}.last-message.txt")),
                    &Pcr9aPanicPhaseBRunner,
                )
                .unwrap();
                assert_eq!(output.status, "phase_b_blocked", "{name}");
                assert!(!output.writes_product_command_sidecar, "{name}");
                assert!(!output.writes_continuation_sidecar, "{name}");
                assert!(!output.writes_runtime_log, "{name}");
                assert_eq!(pcr3_sidecar_text(&path), before, "{name}");
                assert_pcr9a_blocked_reason(&output.blocked_reasons, case);
            } else {
                let before = pcr3_sidecar_text(&path);
                let input = pcr9a_phase_b_input_without_continuation(
                    &path,
                    &command_id,
                    revision,
                    PCR9A_FAKE_PROMPT_BODY,
                );
                let output = run_real_execution_product_command_phase_b_with_runner(
                    &path,
                    &input,
                    "2026-06-09T00:00:06Z",
                    &format!("pcr9a-blocked-{name}"),
                    &path
                        .parent()
                        .unwrap()
                        .join(format!("{name}.last-message.txt")),
                    &Pcr9aPanicPhaseBRunner,
                )
                .unwrap();
                assert_eq!(output.status, "phase_b_blocked", "{name}");
                assert!(!output.writes_product_command_sidecar, "{name}");
                assert_eq!(pcr3_sidecar_text(&path), before, "{name}");
                assert!(output
                    .blocked_reasons
                    .contains(&"phase_b_requires_user_approved_decision".to_string()));
            }
        }
    }

    #[test]
    fn pcr9a_phase_b_records_continuation_block_and_blocks_duplicate_attempt() {
        let (_dir, path, command_id, revision) = pcr4_approved_command_fixture();
        let phase_a = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, revision),
            "2026-06-09T00:00:04Z",
            "pcr9a-phase-a-before-continuation-block",
        )
        .unwrap();
        let mut blocked_input = pcr9a_phase_b_input(
            &path,
            &command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            PCR9A_FAKE_PROMPT_BODY,
        );
        blocked_input.authorization.user_confirmed_real_resume = false;
        let output = run_real_execution_product_command_phase_b_with_runner(
            &path,
            &blocked_input,
            "2026-06-09T00:00:06Z",
            "pcr9a-phase-b-continuation-blocked",
            &path
                .parent()
                .unwrap()
                .join("continuation-blocked.last-message.txt"),
            &Pcr9aPanicPhaseBRunner,
        )
        .unwrap();

        assert_eq!(output.status, "phase_b_blocked");
        assert!(output.writes_product_command_sidecar);
        assert!(output.writes_continuation_sidecar);
        assert!(output.writes_runtime_log);
        assert!(!output.runner_call_allowed);
        assert_eq!(
            output.product_command_attempt.as_ref().unwrap().status,
            "blocked_waiting_authorization"
        );
        assert!(output
            .blocked_reasons
            .contains(&"user_confirmed_real_resume_missing".to_string()));

        let before_duplicate = pcr3_sidecar_text(&path);
        let duplicate_input = pcr9a_phase_b_input(
            &path,
            &command_id,
            output.product_command_store_revision,
            output.session_continuation_store_revision.unwrap(),
            PCR9A_FAKE_PROMPT_BODY,
        );
        let duplicate = run_real_execution_product_command_phase_b_with_runner(
            &path,
            &duplicate_input,
            "2026-06-09T00:00:07Z",
            "pcr9a-phase-b-duplicate",
            &path.parent().unwrap().join("duplicate.last-message.txt"),
            &Pcr9aPanicPhaseBRunner,
        )
        .unwrap();
        assert_eq!(duplicate.status, "phase_b_blocked");
        assert!(duplicate
            .blocked_reasons
            .contains(&"phase_b_duplicate_attempt_blocked".to_string()));
        assert!(!duplicate.writes_product_command_sidecar);
        assert_eq!(pcr3_sidecar_text(&path), before_duplicate);
    }

    #[test]
    fn k2_execution_point_configs_freeze_task_package_fields_and_prompt_hashes() {
        let r1 = k2_execution_point_config(K2_R1_EXECUTION_POINT_ID).unwrap();
        let r2 = k2_execution_point_config(K2_R2_EXECUTION_POINT_ID).unwrap();
        let n1 = k2_execution_point_config(K2_N1_EXECUTION_POINT_ID).unwrap();
        let n2 = k2_execution_point_config(K2_N2_EXECUTION_POINT_ID).unwrap();

        for config in [&r1, &r2, &n1, &n2] {
            validate_k2_execution_point_config(config).unwrap();
            assert_eq!(sha256_hex(&config.canonical_prompt), config.prompt_hash);
            assert_eq!(config.adapter_id, "codex-local");
            assert_eq!(config.user_confirmation, "confirmed_by:user");
            assert!(config.readback_plan.contains(&config.readback_marker));
            assert!(codex_control_denied_paths_cover_sensitive_boundary(
                &config.denied_paths
            ));
        }

        assert_eq!(r1.operation, "resume");
        assert_eq!(r1.project_root, K2_MARIO_PROJECT_ROOT);
        assert_eq!(r1.target_session_id.as_deref(), Some(K2_R1_SESSION_ID));
        assert_eq!(r1.sandbox, "read-only");
        assert!(r1.allowed_write_roots.is_empty());

        assert_eq!(r2.operation, "resume");
        assert_eq!(r2.target_session_id.as_deref(), Some(K2_R2_SESSION_ID));
        assert_eq!(r2.sandbox, "workspace-write");
        assert_eq!(r2.allowed_write_roots, vec![K2_R2_ALLOWED_WRITE_ROOT]);
        assert_eq!(
            r2.allowed_write_path.as_deref(),
            Some(K2_R2_ALLOWED_WRITE_PATH)
        );

        assert_eq!(n1.operation, "new_session");
        assert_eq!(n1.project_root, K2_ISOLATED_PROJECT_ROOT);
        assert!(n1.target_session_id.is_none());
        assert_eq!(n1.sandbox, "read-only");
        assert!(n1.allowed_write_roots.is_empty());

        assert_eq!(n2.operation, "new_session");
        assert!(n2.target_session_id.is_none());
        assert_eq!(n2.sandbox, "workspace-write");
        assert_eq!(n2.allowed_write_roots, vec![K2_N2_ALLOWED_WRITE_ROOT]);
        assert_eq!(
            n2.allowed_write_path.as_deref(),
            Some(K2_N2_ALLOWED_WRITE_PATH)
        );
    }

    #[test]
    fn k2_r1_resume_fake_runner_uses_product_command_chain_and_blocks_duplicate() {
        let (_dir, workflow_state_path) = k2_fixture_state("r1");
        let config = k2_execution_point_config(K2_R1_EXECUTION_POINT_ID).unwrap();
        let (phase_b, product_sidecar, continuation_sidecar, runtime_sidecar) =
            run_k2_resume_fake_chain(
                &workflow_state_path,
                &config,
                K2FakePhaseBRunner {
                    expected_prompt_hash: config.prompt_hash.clone(),
                    marker: config.readback_marker.clone(),
                    writes_project_files: false,
                    readback_status: "succeeded".to_string(),
                    readback_result_count: Some(1),
                },
            );

        assert_eq!(phase_b.status, "phase_b_completed");
        assert_eq!(
            phase_b.product_command_attempt.as_ref().unwrap().status,
            "phase_b_real_resume_executed"
        );
        assert!(phase_b.runner_call_allowed);
        assert!(phase_b.prompt_sent);
        assert!(phase_b.real_codex_executed);
        assert!(phase_b.writes_codex_home);
        assert!(!phase_b.writes_project_files);
        assert_eq!(phase_b.readback_summary.result_count, Some(1));
        assert_k2_sidecars_do_not_persist_prompt(
            [&product_sidecar, &continuation_sidecar, &runtime_sidecar],
            &config.canonical_prompt,
        );

        let duplicate_input = k2_resume_phase_b_input(
            &workflow_state_path,
            &config,
            &phase_b.product_command_id,
            phase_b.product_command_store_revision,
            phase_b.session_continuation_store_revision.unwrap(),
            Some("2026-06-10T00:00:07Z".to_string()),
        )
        .unwrap();
        let before_duplicate = fs::read_to_string(&product_sidecar).unwrap();
        let duplicate = run_real_execution_product_command_phase_b_with_runner(
            &workflow_state_path,
            &duplicate_input,
            "2026-06-10T00:00:07Z",
            "k2-r1-duplicate-phase-b",
            &workflow_state_path
                .parent()
                .unwrap()
                .join("k2-r1-duplicate.last-message.txt"),
            &K2PanicPhaseBRunner,
        )
        .unwrap();

        assert_eq!(duplicate.status, "phase_b_blocked");
        assert!(duplicate
            .blocked_reasons
            .contains(&"phase_b_duplicate_attempt_blocked".to_string()));
        assert!(!duplicate.writes_product_command_sidecar);
        assert_eq!(
            fs::read_to_string(product_sidecar).unwrap(),
            before_duplicate
        );
    }

    #[test]
    fn k2_new_session_fake_runner_covers_read_only_empty_roots_and_write_root_boundary() {
        let (_n1_dir, n1_workflow_state_path) = k2_fixture_state("n1");
        let n1 = k2_execution_point_config(K2_N1_EXECUTION_POINT_ID).unwrap();
        let (n1_phase_b, _, _, _) = run_k2_new_session_fake_chain(
            &n1_workflow_state_path,
            &n1,
            K2FakePhaseBRunner {
                expected_prompt_hash: n1.prompt_hash.clone(),
                marker: n1.readback_marker.clone(),
                writes_project_files: false,
                readback_status: "succeeded".to_string(),
                readback_result_count: Some(1),
            },
        );
        assert_eq!(n1_phase_b.status, "phase_b_completed");
        assert!(n1_phase_b.runner_call_allowed);
        assert!(!n1_phase_b.writes_project_files);
        assert_eq!(n1_phase_b.readback_summary.result_count, Some(1));
        let (n1_store, _, _) = load_real_execution_product_command_store(
            &n1_workflow_state_path,
            "2026-06-10T00:00:08Z",
        )
        .unwrap();
        assert!(n1_store.commands[0].allowed_write_roots.is_empty());

        let (_n2_dir, n2_workflow_state_path) = k2_fixture_state("n2");
        let n2 = k2_execution_point_config(K2_N2_EXECUTION_POINT_ID).unwrap();
        let (n2_phase_b, _, _, _) = run_k2_new_session_fake_chain(
            &n2_workflow_state_path,
            &n2,
            K2FakePhaseBRunner {
                expected_prompt_hash: n2.prompt_hash.clone(),
                marker: n2.readback_marker.clone(),
                writes_project_files: true,
                readback_status: "readback_unavailable".to_string(),
                readback_result_count: Some(0),
            },
        );
        assert_eq!(n2_phase_b.status, "phase_b_completed");
        assert!(n2_phase_b.runner_call_allowed);
        assert!(n2_phase_b.writes_project_files);
        assert_eq!(n2_phase_b.readback_summary.status, "readback_unavailable");
        assert_eq!(n2_phase_b.readback_summary.result_count, None);
        let (n2_store, _, _) = load_real_execution_product_command_store(
            &n2_workflow_state_path,
            "2026-06-10T00:00:08Z",
        )
        .unwrap();
        assert_eq!(
            n2_store.commands[0].allowed_write_roots,
            vec![K2_N2_ALLOWED_WRITE_ROOT]
        );
    }

    #[test]
    #[ignore = "requires explicit K2-R1 real resume/read-only authorization"]
    fn k2_r1_real_mario_test_resume_read_only_requires_env_authorization() {
        let config = k2_execution_point_config(K2_R1_EXECUTION_POINT_ID).unwrap();
        k2_require_real_confirmation("K2_R1", &config.execution_point_id);
        assert_eq!(config.operation, "resume");
        assert_eq!(config.sandbox, "read-only");
        assert!(config.allowed_write_roots.is_empty());
        assert_eq!(
            config.readback_marker,
            "K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10"
        );

        let project_root = PathBuf::from(&config.project_root);
        let before_core_hashes = mario_core_file_hashes(&project_root);
        let run = run_k2_real_resume_probe("K2_R1", &config);
        let after_core_hashes = mario_core_file_hashes(&project_root);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();

        println!(
            "K2_R1_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "K2_R1_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "K2_R1_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!("K2_R1_RUNTIME_LOG_PATH={}", run.runtime_log_path.display());
        println!(
            "K2_R1_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("K2_R1_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("K2_R1_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("K2_R1_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "K2_R1_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("K2_R1_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("K2_R1_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("K2_R1_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("K2_R1_CORE_HASHES_AFTER={:?}", after_core_hashes);

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(!run.output.writes_project_files);
        assert!(last_message.contains(&config.readback_marker));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert_k2_sidecars_do_not_persist_prompt(
            [
                &run.product_sidecar_path,
                &run.continuation_sidecar_path,
                &run.runtime_log_path,
            ],
            &config.canonical_prompt,
        );
    }

    #[test]
    #[ignore = "requires explicit K2-R2 real resume/workspace-write authorization"]
    fn k2_r2_real_mario_test_resume_workspace_write_requires_env_authorization() {
        let config = k2_execution_point_config(K2_R2_EXECUTION_POINT_ID).unwrap();
        k2_require_real_confirmation("K2_R2", &config.execution_point_id);
        assert_eq!(config.operation, "resume");
        assert_eq!(config.sandbox, "workspace-write");
        assert_eq!(
            config.allowed_write_path.as_deref(),
            Some(K2_R2_ALLOWED_WRITE_PATH)
        );

        let project_root = PathBuf::from(&config.project_root);
        let before_core_hashes = mario_core_file_hashes(&project_root);
        let before_project_hashes = project_file_hashes(&project_root);
        let run = run_k2_real_resume_probe("K2_R2", &config);
        let after_core_hashes = mario_core_file_hashes(&project_root);
        let after_project_hashes = project_file_hashes(&project_root);
        let changed_files = changed_project_files(&before_project_hashes, &after_project_hashes);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();
        let allowed_file = PathBuf::from(config.allowed_write_path.as_ref().unwrap());
        let allowed_body = fs::read_to_string(&allowed_file).unwrap();

        println!(
            "K2_R2_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "K2_R2_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "K2_R2_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!("K2_R2_RUNTIME_LOG_PATH={}", run.runtime_log_path.display());
        println!(
            "K2_R2_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("K2_R2_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("K2_R2_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("K2_R2_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "K2_R2_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("K2_R2_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("K2_R2_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("K2_R2_CHANGED_PROJECT_FILES={}", changed_files.join(","));
        println!("K2_R2_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("K2_R2_CORE_HASHES_AFTER={:?}", after_core_hashes);

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(run.output.writes_project_files);
        assert!(last_message.contains(&config.readback_marker));
        assert!(allowed_body.contains(&config.readback_marker));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert!(
            changed_files
                .iter()
                .all(|path| path == ".workbench/stage-k/k2/resume-workspace-write-probe.md"),
            "{changed_files:?}"
        );
        assert_k2_sidecars_do_not_persist_prompt(
            [
                &run.product_sidecar_path,
                &run.continuation_sidecar_path,
                &run.runtime_log_path,
            ],
            &config.canonical_prompt,
        );
    }

    #[test]
    #[ignore = "requires explicit K2-N1 real new-session/read-only authorization"]
    fn k2_n1_real_isolated_new_session_read_only_requires_env_authorization() {
        let config = k2_execution_point_config(K2_N1_EXECUTION_POINT_ID).unwrap();
        k2_require_real_confirmation("K2_N1", &config.execution_point_id);
        assert_eq!(config.operation, "new_session");
        assert_eq!(config.sandbox, "read-only");
        assert!(config.allowed_write_roots.is_empty());

        let project_root = PathBuf::from(&config.project_root);
        let before_project_hashes = project_file_hashes(&project_root);
        let run = run_k2_real_new_session_probe("K2_N1", &config);
        let after_project_hashes = project_file_hashes(&project_root);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();

        println!(
            "K2_N1_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "K2_N1_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "K2_N1_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!("K2_N1_RUNTIME_LOG_PATH={}", run.runtime_log_path.display());
        println!(
            "K2_N1_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("K2_N1_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("K2_N1_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("K2_N1_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "K2_N1_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("K2_N1_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("K2_N1_AUDIT_REFS={}", run.audit_refs.join(","));

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(!run.output.writes_project_files);
        assert!(last_message.contains(&config.readback_marker));
        assert_eq!(before_project_hashes, after_project_hashes);
        assert_k2_sidecars_do_not_persist_prompt(
            [
                &run.product_sidecar_path,
                &run.continuation_sidecar_path,
                &run.runtime_log_path,
            ],
            &config.canonical_prompt,
        );
    }

    #[test]
    #[ignore = "requires explicit K2-N2 real new-session/workspace-write authorization"]
    fn k2_n2_real_isolated_new_session_workspace_write_requires_env_authorization() {
        let config = k2_execution_point_config(K2_N2_EXECUTION_POINT_ID).unwrap();
        k2_require_real_confirmation("K2_N2", &config.execution_point_id);
        assert_eq!(config.operation, "new_session");
        assert_eq!(config.sandbox, "workspace-write");
        assert_eq!(
            config.allowed_write_path.as_deref(),
            Some(K2_N2_ALLOWED_WRITE_PATH)
        );

        let project_root = PathBuf::from(&config.project_root);
        let before_project_hashes = project_file_hashes(&project_root);
        let run = run_k2_real_new_session_probe("K2_N2", &config);
        let after_project_hashes = project_file_hashes(&project_root);
        let changed_files = changed_project_files(&before_project_hashes, &after_project_hashes);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();
        let allowed_file = PathBuf::from(config.allowed_write_path.as_ref().unwrap());
        let allowed_body = fs::read_to_string(&allowed_file).unwrap();

        println!(
            "K2_N2_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "K2_N2_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "K2_N2_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!("K2_N2_RUNTIME_LOG_PATH={}", run.runtime_log_path.display());
        println!(
            "K2_N2_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("K2_N2_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("K2_N2_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("K2_N2_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "K2_N2_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("K2_N2_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("K2_N2_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("K2_N2_CHANGED_PROJECT_FILES={}", changed_files.join(","));

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(run.output.writes_project_files);
        assert!(last_message.contains(&config.readback_marker));
        assert!(allowed_body.contains(&config.readback_marker));
        assert!(
            changed_files
                .iter()
                .all(|path| path == ".workbench/stage-k/k2/new-session-write-probe.md"),
            "{changed_files:?}"
        );
        assert_k2_sidecars_do_not_persist_prompt(
            [
                &run.product_sidecar_path,
                &run.continuation_sidecar_path,
                &run.runtime_log_path,
            ],
            &config.canonical_prompt,
        );
    }

    #[test]
    #[ignore = "requires explicit PCR9 B1 real product command read-only authorization"]
    fn pcr9_b1_real_mario_test_product_command_read_only_probe_requires_env_authorization() {
        let config = pcr9_real_probe_config("B1");
        assert_eq!(config.project_root, "/Users/yoyi/Documents/mario test");
        assert_eq!(config.session_id, "019e798a-ac37-7771-b982-e38084fcd22e");
        assert_eq!(
            config.expected_marker,
            "PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_READ_ONLY_OK_2026_06_09"
        );
        assert_eq!(
            config.prompt_hash,
            "99f65e9f986272da4b1dfda91261b0bed32621b963b515e08296384443d650cc"
        );
        assert_eq!(config.sandbox, "read-only");

        let project_root_path = PathBuf::from(&config.project_root);
        let pre_core_hashes = mario_core_file_hashes(&project_root_path);
        let pre_project_files = project_file_set(&project_root_path);
        let pre_project_hashes = project_file_hashes(&project_root_path);
        let run = run_pcr9_real_product_command_probe(&config);
        let post_core_hashes = mario_core_file_hashes(&project_root_path);
        let post_project_files = project_file_set(&project_root_path);
        let post_project_hashes = project_file_hashes(&project_root_path);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();

        println!(
            "PCR9_B1_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "PCR9_B1_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "PCR9_B1_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!(
            "PCR9_B1_RUNTIME_LOG_PATH={}",
            run.runtime_log_path.display()
        );
        println!(
            "PCR9_B1_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("PCR9_B1_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("PCR9_B1_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("PCR9_B1_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "PCR9_B1_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("PCR9_B1_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("PCR9_B1_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("PCR9_B1_CORE_HASHES_BEFORE={:?}", pre_core_hashes);
        println!("PCR9_B1_CORE_HASHES_AFTER={:?}", post_core_hashes);

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(!run.output.writes_project_files);
        assert_eq!(run.output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(&config.expected_marker));
        assert_eq!(pre_core_hashes, post_core_hashes);
        assert_eq!(pre_project_files, post_project_files);
        assert_eq!(pre_project_hashes, post_project_hashes);
        assert_product_command_store_does_not_persist_prompt(
            &run.product_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.continuation_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.runtime_log_path,
            &config.prompt_body,
        );
    }

    #[test]
    #[ignore = "requires explicit PCR9 B2 real product command workspace-write authorization"]
    fn pcr9_b2_real_mario_test_product_command_write_probe_requires_env_authorization() {
        let config = pcr9_real_probe_config("B2");
        assert_eq!(config.project_root, "/Users/yoyi/Documents/mario test");
        assert_eq!(config.session_id, "019e798a-ac37-7771-b982-e38084fcd22e");
        assert_eq!(
            config.expected_marker,
            "PCR9_UNIFIED_PRODUCT_COMMAND_MARIO_TEST_WRITE_PROBE_OK_2026_06_09"
        );
        assert_eq!(
            config.prompt_hash,
            "00a85874146fc1f5928486de85e7ed1c55c8fe5ea29fefcbab56973b4f71a48c"
        );
        assert_eq!(config.sandbox, "workspace-write");
        assert_eq!(
            config.allowed_write_roots,
            vec!["/Users/yoyi/Documents/mario test/.workbench/pcr9".to_string()]
        );

        let project_root_path = PathBuf::from(&config.project_root);
        let probe_path =
            project_root_path.join(".workbench/pcr9/real-product-command-write-probe.md");
        fs::create_dir_all(probe_path.parent().unwrap()).unwrap();
        let probe_pre_state = if probe_path.exists() {
            format!("exists:{}", sha256_file(&probe_path))
        } else {
            "missing".to_string()
        };
        let pre_core_hashes = mario_core_file_hashes(&project_root_path);
        let pre_project_files = project_file_set(&project_root_path);
        let pre_project_hashes = project_file_hashes(&project_root_path);
        let run = run_pcr9_real_product_command_probe(&config);
        let post_core_hashes = mario_core_file_hashes(&project_root_path);
        let post_project_files = project_file_set(&project_root_path);
        let post_project_hashes = project_file_hashes(&project_root_path);
        let new_project_files = post_project_files
            .difference(&pre_project_files)
            .cloned()
            .collect::<Vec<_>>();
        let changed_project_files =
            changed_project_files(&pre_project_hashes, &post_project_hashes);
        let probe_body = fs::read_to_string(&probe_path).expect("read PCR9 B2 probe file");
        let probe_hash = sha256_file(&probe_path);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();

        println!(
            "PCR9_B2_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "PCR9_B2_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "PCR9_B2_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!(
            "PCR9_B2_RUNTIME_LOG_PATH={}",
            run.runtime_log_path.display()
        );
        println!(
            "PCR9_B2_LAST_MESSAGE_PATH={}",
            run.last_message_path.display()
        );
        println!("PCR9_B2_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("PCR9_B2_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("PCR9_B2_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "PCR9_B2_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("PCR9_B2_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("PCR9_B2_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("PCR9_B2_PROBE_PATH={}", probe_path.display());
        println!("PCR9_B2_PROBE_PRE_STATE={probe_pre_state}");
        println!("PCR9_B2_PROBE_SHA256={probe_hash}");
        println!("PCR9_B2_NEW_PROJECT_FILES={}", new_project_files.join(","));
        println!(
            "PCR9_B2_CHANGED_PROJECT_FILES={}",
            changed_project_files.join(",")
        );
        println!("PCR9_B2_CORE_HASHES_BEFORE={:?}", pre_core_hashes);
        println!("PCR9_B2_CORE_HASHES_AFTER={:?}", post_core_hashes);

        assert_eq!(run.output.status, "phase_b_completed");
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(run.output.writes_project_files);
        assert_eq!(run.output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(&config.expected_marker));
        assert!(probe_body.contains(&config.expected_marker));
        assert!(probe_body.contains(".workbench/pcr9/real-product-command-write-probe.md"));
        assert_eq!(pre_core_hashes, post_core_hashes);
        assert!(new_project_files
            .iter()
            .all(|path| path.starts_with(".workbench/pcr9/")));
        assert!(
            changed_project_files
                .iter()
                .all(|path| path == ".workbench/pcr9/real-product-command-write-probe.md"),
            "{changed_project_files:?}"
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.product_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.continuation_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.runtime_log_path,
            &config.prompt_body,
        );
    }

    #[test]
    #[ignore = "requires explicit J1-B mario test Codex Control real resume authorization"]
    fn j1_b_real_mario_test_codex_control_resume_probe_requires_env_authorization() {
        let config = j1_b_real_probe_config();
        assert_eq!(
            config.expected_marker,
            "J1_B_MARIO_TEST_CODEX_CONTROL_RESUME_OK_2026_06_09"
        );
        assert_eq!(config.sandbox, "read-only");
        assert_eq!(
            config.allowed_write_roots,
            vec!["/Users/yoyi/Documents/mario test".to_string()]
        );
        assert_eq!(
            sha256_hex(&config.prompt_body),
            "2547d65c4e86e6357906a7a55b5923f806f719b952658606e2a6ff9d3797755b"
        );

        let project_root_path = PathBuf::from(&config.project_root);
        let before_core_hashes = mario_core_file_hashes(&project_root_path);
        let run = run_j1_b_real_codex_control_probe(&config);
        let after_core_hashes = mario_core_file_hashes(&project_root_path);
        let last_message = fs::read_to_string(&run.last_message_path).unwrap();

        println!(
            "J1_B_WORKFLOW_STATE_PATH={}",
            run.workflow_state_path.display()
        );
        println!(
            "J1_B_PRODUCT_COMMAND_SIDECAR_PATH={}",
            run.product_sidecar_path.display()
        );
        println!(
            "J1_B_SESSION_CONTINUATION_STORE_PATH={}",
            run.continuation_sidecar_path.display()
        );
        println!("J1_B_RUNTIME_LOG_PATH={}", run.runtime_log_path.display());
        println!("J1_B_LAST_MESSAGE_PATH={}", run.last_message_path.display());
        println!("J1_B_PRODUCT_COMMAND_ID={}", run.product_command_id);
        println!("J1_B_PRODUCT_ATTEMPT_ID={}", run.product_attempt_id);
        println!("J1_B_CONTINUATION_ID={}", run.continuation_id);
        println!(
            "J1_B_CONTINUATION_ATTEMPT_ID={}",
            run.continuation_attempt_id
        );
        println!("J1_B_RUNTIME_LOG_REF={}", run.runtime_log_ref);
        println!("J1_B_AUDIT_REFS={}", run.audit_refs.join(","));
        println!("J1_B_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("J1_B_CORE_HASHES_AFTER={:?}", after_core_hashes);

        assert_eq!(run.output.status, "phase_b_completed");
        assert_eq!(
            run.output.product_command_attempt.as_ref().unwrap().status,
            "phase_b_real_resume_executed"
        );
        assert!(run.output.runner_call_allowed);
        assert!(run.output.prompt_sent);
        assert!(run.output.real_codex_executed);
        assert!(run.output.writes_codex_home);
        assert!(!run.output.writes_project_files);
        assert!(run.output.writes_product_command_sidecar);
        assert!(run.output.writes_continuation_sidecar);
        assert!(run.output.writes_runtime_log);
        assert_eq!(run.output.readback_summary.status, "succeeded");
        assert_eq!(run.output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(&config.expected_marker));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert_product_command_store_does_not_persist_prompt(
            &run.product_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.continuation_sidecar_path,
            &config.prompt_body,
        );
        assert_product_command_store_does_not_persist_prompt(
            &run.runtime_log_path,
            &config.prompt_body,
        );
    }

    struct Pcr9RealProbeConfig {
        level: String,
        project_root: String,
        session_id: String,
        prompt_body: String,
        prompt_hash: String,
        expected_marker: String,
        workflow_state_parent: PathBuf,
        sandbox: String,
        prompt_summary: String,
        prompt_ref: String,
        dispatch_id: String,
        work_item_id: String,
        artifact_id: String,
        snapshot_id: String,
        memory_fingerprint: String,
        allowed_write_roots: Vec<String>,
    }

    struct Pcr9RealProbeRun {
        output: RealExecutionProductCommandPhaseBOutput,
        workflow_state_path: PathBuf,
        product_sidecar_path: PathBuf,
        continuation_sidecar_path: PathBuf,
        runtime_log_path: PathBuf,
        last_message_path: PathBuf,
        product_command_id: String,
        product_attempt_id: String,
        continuation_id: String,
        continuation_attempt_id: String,
        runtime_log_ref: String,
        audit_refs: Vec<String>,
    }

    struct J1BRealProbeConfig {
        project_root: String,
        session_id: String,
        prompt_body: String,
        expected_marker: String,
        workflow_state_parent: PathBuf,
        sandbox: String,
        prompt_summary: String,
        prompt_ref: String,
        prompt_hash: String,
        work_item_id: String,
        artifact_id: String,
        snapshot_id: String,
        memory_fingerprint: String,
        allowed_write_roots: Vec<String>,
    }

    struct J1BRealProbeRun {
        output: RealExecutionProductCommandPhaseBOutput,
        workflow_state_path: PathBuf,
        product_sidecar_path: PathBuf,
        continuation_sidecar_path: PathBuf,
        runtime_log_path: PathBuf,
        last_message_path: PathBuf,
        product_command_id: String,
        product_attempt_id: String,
        continuation_id: String,
        continuation_attempt_id: String,
        runtime_log_ref: String,
        audit_refs: Vec<String>,
    }

    fn pcr9_real_probe_config(level: &str) -> Pcr9RealProbeConfig {
        let prefix = format!("PCR9_{level}_");
        let project_root = env::var(format!("{prefix}PROJECT_ROOT"))
            .unwrap_or_else(|_| panic!("{prefix}PROJECT_ROOT is required for real probe"));
        let session_id = env::var(format!("{prefix}SESSION_ID"))
            .unwrap_or_else(|_| panic!("{prefix}SESSION_ID is required for real probe"));
        let prompt_path = PathBuf::from(
            env::var(format!("{prefix}PROMPT_PATH"))
                .unwrap_or_else(|_| panic!("{prefix}PROMPT_PATH is required for real probe")),
        );
        let expected_marker = env::var(format!("{prefix}EXPECTED_MARKER"))
            .unwrap_or_else(|_| panic!("{prefix}EXPECTED_MARKER is required for real probe"));
        let workflow_state_parent = PathBuf::from(
            env::var(format!("{prefix}WORKFLOW_STATE_PARENT")).unwrap_or_else(|_| {
                panic!("{prefix}WORKFLOW_STATE_PARENT is required for real probe")
            }),
        );
        let prompt_body = fs::read_to_string(&prompt_path)
            .expect("read PCR9 prompt ref")
            .trim_end_matches(['\r', '\n'])
            .to_string();
        let prompt_hash = sha256_hex(&prompt_body);
        assert!(
            prompt_path
                .starts_with("/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/"),
            "PCR9 prompt ref must be inside product-line tmp/pcr9-real-product-command"
        );
        assert!(
            workflow_state_parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/pcr9-real-product-command/runs"
            ),
            "PCR9 workflow state parent must be inside product-line tmp/pcr9-real-product-command/runs"
        );

        let is_b2 = level == "B2";
        let prompt_summary = if is_b2 {
            "PCR9 Level B workspace-write unified product command resume probe for mario test codex-dev worker."
        } else {
            "PCR9 Level B read-only unified product command resume probe for mario test codex-dev worker."
        }
        .to_string();
        let prompt_ref = if is_b2 {
            "workbench-managed:pcr9:mario-test:codex-dev:workspace-write-unified-product-command-probe:v1"
        } else {
            "workbench-managed:pcr9:mario-test:codex-dev:read-only-unified-product-command-probe:v1"
        }
        .to_string();
        let dispatch_id = if is_b2 {
            "dispatch:pcr9-b2:mario-test:codex-dev:write-probe:v1"
        } else {
            "dispatch:pcr9-b1:mario-test:codex-dev:read-only-probe:v1"
        }
        .to_string();
        let work_item_id = if is_b2 {
            "work-item:pcr9-b2:mario-test:codex-dev:write-probe:v1"
        } else {
            "work-item:pcr9-b1:mario-test:codex-dev:read-only-probe:v1"
        }
        .to_string();
        let artifact_id = if is_b2 {
            "artifact:pcr9-b2:mario-test:codex-dev:write-probe:v1"
        } else {
            "artifact:pcr9-b1:mario-test:codex-dev:read-only-probe:v1"
        }
        .to_string();
        let snapshot_id = if is_b2 {
            "task-memory-packet-snapshot:pcr9-b2:mario-test:codex-dev:2026-06-09"
        } else {
            "task-memory-packet-snapshot:pcr9-b1:mario-test:codex-dev:2026-06-09"
        }
        .to_string();
        let memory_fingerprint = if is_b2 {
            "pcr9-b2-memory-fingerprint-mario-test-codex-dev-2026-06-09"
        } else {
            "pcr9-b1-memory-fingerprint-mario-test-codex-dev-2026-06-09"
        }
        .to_string();
        let allowed_write_roots = if is_b2 {
            vec![PathBuf::from(&project_root)
                .join(".workbench/pcr9")
                .display()
                .to_string()]
        } else {
            vec![project_root.clone()]
        };

        Pcr9RealProbeConfig {
            level: level.to_string(),
            project_root,
            session_id,
            prompt_body,
            prompt_hash,
            expected_marker,
            workflow_state_parent,
            sandbox: if is_b2 {
                "workspace-write".to_string()
            } else {
                "read-only".to_string()
            },
            prompt_summary,
            prompt_ref,
            dispatch_id,
            work_item_id,
            artifact_id,
            snapshot_id,
            memory_fingerprint,
            allowed_write_roots,
        }
    }

    fn run_pcr9_real_product_command_probe(config: &Pcr9RealProbeConfig) -> Pcr9RealProbeRun {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = config.workflow_state_parent.join(format!(
            "{}-run-{unique}",
            config.level.to_ascii_lowercase()
        ));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        write_pcr9_workflow_state(&workflow_state_path, config);

        let prepare = prepare_real_execution_product_command_at(
            &workflow_state_path,
            &PrepareRealExecutionProductCommandInput {
                source_kind: "h5_project_workflow_dispatch".to_string(),
                h5_dispatch_preview: Some(pcr9_h5_preview_input(config)),
                codex_control: None,
                expected_store_revision: Some(0),
                requested_by: Some("project_director".to_string()),
                created_at: Some("2026-06-09T09:00:00Z".to_string()),
            },
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(
            prepare.blocked_reasons.is_empty(),
            "{:?}",
            prepare.blocked_reasons
        );
        let product_command_id = prepare.product_command_id.unwrap();

        let decision = record_real_execution_product_command_decision_at(
            &workflow_state_path,
            &RecordRealExecutionProductCommandDecisionInput {
                product_command_id: product_command_id.clone(),
                decision: "approved".to_string(),
                expected_store_revision: Some(prepare.store_revision),
                confirmed_by: "user".to_string(),
                risk_acknowledgement:
                    "User authorized PCR9 Level B unified product command real resume once."
                        .to_string(),
                allowed_once: true,
                reason: "PCR9 B1/B2 real probe authorization from user and global supervisor."
                    .to_string(),
                requested_by: Some("global_supervisor".to_string()),
                confirmed_at: Some("2026-06-09T09:00:01Z".to_string()),
            },
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");

        let phase_a = run_real_execution_product_command_phase_a_at(
            &workflow_state_path,
            &RunRealExecutionProductCommandPhaseAInput {
                product_command_id: product_command_id.clone(),
                expected_product_command_store_revision: Some(decision.store_revision),
                expected_session_continuation_store_revision: None,
                actor_role: "project_director".to_string(),
                execution_decision: Some("phase_a_noop".to_string()),
                timeout_ms: Some(120_000),
                requested_at: Some("2026-06-09T09:00:02Z".to_string()),
            },
            "2026-06-09T09:00:02Z",
            &format!("pcr9-{}-phase-a", config.level.to_ascii_lowercase()),
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");

        let input = pcr9_phase_b_input(
            &workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
        );
        let phase_b_timestamp = "2026-06-09T09:00:03Z";
        let output = run_real_execution_product_command_phase_b_at(
            &workflow_state_path,
            &input,
            phase_b_timestamp,
            &format!("pcr9-{}-phase-b", config.level.to_ascii_lowercase()),
        )
        .unwrap();
        let last_message_path = pcr9a_phase_b_last_message_path(
            &workflow_state_path,
            &product_command_id,
            phase_b_timestamp,
        )
        .unwrap();
        let product_attempt = output
            .product_command_attempt
            .as_ref()
            .expect("PCR9 product command Phase B attempt");

        Pcr9RealProbeRun {
            product_sidecar_path: real_execution_product_command_sidecar_path(&workflow_state_path)
                .unwrap(),
            continuation_sidecar_path: crate::session_continuation_store::sidecar_path(
                &workflow_state_path,
            )
            .unwrap(),
            runtime_log_path: crate::runtime_log_store::sidecar_path(&workflow_state_path).unwrap(),
            workflow_state_path,
            last_message_path,
            product_command_id,
            product_attempt_id: product_attempt.attempt_id.clone(),
            continuation_id: output.continuation_id.clone().unwrap(),
            continuation_attempt_id: output.continuation_attempt_id.clone().unwrap(),
            runtime_log_ref: output.runtime_log_ref.clone().unwrap(),
            audit_refs: output.audit_refs.clone(),
            output,
        }
    }

    fn j1_b_real_probe_config() -> J1BRealProbeConfig {
        let project_root =
            env::var("J1_B_PROJECT_ROOT").expect("J1_B_PROJECT_ROOT is required for real probe");
        let session_id =
            env::var("J1_B_SESSION_ID").expect("J1_B_SESSION_ID is required for real probe");
        let prompt_body =
            env::var("J1_B_PROMPT_BODY").expect("J1_B_PROMPT_BODY is required for real probe");
        let expected_marker = env::var("J1_B_EXPECTED_MARKER")
            .expect("J1_B_EXPECTED_MARKER is required for real probe");
        let workflow_state_parent = PathBuf::from(
            env::var("J1_B_WORKFLOW_STATE_PARENT")
                .expect("J1_B_WORKFLOW_STATE_PARENT is required for real probe"),
        );
        assert!(
            workflow_state_parent
                .starts_with("/Users/yoyi/workspace/product-line/tmp/j1-b-codex-control/runs"),
            "J1-B workflow state parent must be inside product-line tmp/j1-b-codex-control/runs"
        );

        J1BRealProbeConfig {
            project_root,
            session_id,
            prompt_body,
            expected_marker,
            workflow_state_parent,
            sandbox: "read-only".to_string(),
            prompt_summary: "J1-B mario test Codex Control real resume marker probe".to_string(),
            prompt_ref: "workbench-runtime-prompt:j1-b:mario-test:2547d65c4e86".to_string(),
            prompt_hash: "2547d65c4e86e6357906a7a55b5923f806f719b952658606e2a6ff9d3797755b"
                .to_string(),
            work_item_id: "work-item:j1-b:mario-test:codex-control:real-resume-probe:v1"
                .to_string(),
            artifact_id: "artifact:j1-b:mario-test:codex-control:real-resume-probe:v1".to_string(),
            snapshot_id: "task-memory-packet-snapshot:j1-b:mario-test:codex-control:2026-06-09"
                .to_string(),
            memory_fingerprint: "j1-b-memory-fingerprint-mario-test-codex-control-2026-06-09"
                .to_string(),
            allowed_write_roots: vec!["/Users/yoyi/Documents/mario test".to_string()],
        }
    }

    fn run_j1_b_real_codex_control_probe(config: &J1BRealProbeConfig) -> J1BRealProbeRun {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = config
            .workflow_state_parent
            .join(format!("j1-b-run-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        write_j1_b_workflow_state(&workflow_state_path, config);

        let prepare = prepare_real_execution_product_command_at(
            &workflow_state_path,
            &PrepareRealExecutionProductCommandInput {
                source_kind: "codex_control".to_string(),
                h5_dispatch_preview: None,
                codex_control: Some(j1_b_codex_control_input(config)),
                expected_store_revision: Some(0),
                requested_by: Some("user".to_string()),
                created_at: Some("2026-06-09T10:00:00Z".to_string()),
            },
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(
            prepare.blocked_reasons.is_empty(),
            "{:?}",
            prepare.blocked_reasons
        );
        let product_command_id = prepare.product_command_id.unwrap();
        let (prepared_store, _, _) =
            load_real_execution_product_command_store(&workflow_state_path, "2026-06-09T10:00:00Z")
                .unwrap();
        let request = prepared_store
            .commands
            .iter()
            .find(|command| command.product_command_id == product_command_id)
            .unwrap();
        assert_eq!(request.command_family, "real_execution_product_command");
        assert_eq!(request.operation_id, "resume");
        assert_eq!(request.adapter_id, "codex-local");
        assert_eq!(
            request.workflow_id.as_deref(),
            Some("workflow:users-yoyi-documents-mario-test:default")
        );
        assert_eq!(
            request.memory_packet_ref.as_deref(),
            Some(config.snapshot_id.as_str())
        );

        let decision = record_real_execution_product_command_decision_at(
            &workflow_state_path,
            &RecordRealExecutionProductCommandDecisionInput {
                product_command_id: product_command_id.clone(),
                decision: "approved".to_string(),
                expected_store_revision: Some(prepare.store_revision),
                confirmed_by: "user".to_string(),
                risk_acknowledgement:
                    "User authorized J1-B mario test Codex Control real resume once.".to_string(),
                allowed_once: true,
                reason:
                    "J1-B real resume execution point authorized by user and global supervisor."
                        .to_string(),
                requested_by: Some("global_supervisor".to_string()),
                confirmed_at: Some("2026-06-09T10:00:01Z".to_string()),
            },
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");

        let phase_a = run_real_execution_product_command_phase_a_at(
            &workflow_state_path,
            &RunRealExecutionProductCommandPhaseAInput {
                product_command_id: product_command_id.clone(),
                expected_product_command_store_revision: Some(decision.store_revision),
                expected_session_continuation_store_revision: None,
                actor_role: "project_director".to_string(),
                execution_decision: Some("phase_a_noop".to_string()),
                timeout_ms: Some(120_000),
                requested_at: Some("2026-06-09T10:00:02Z".to_string()),
            },
            "2026-06-09T10:00:02Z",
            "j1-b-codex-control-phase-a",
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");

        let input = j1_b_phase_b_input(
            &workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
        );
        let phase_b_timestamp = "2026-06-09T10:00:03Z";
        let output = run_real_execution_product_command_phase_b_at(
            &workflow_state_path,
            &input,
            phase_b_timestamp,
            "j1-b-codex-control-phase-b",
        )
        .unwrap();
        let last_message_path = pcr9a_phase_b_last_message_path(
            &workflow_state_path,
            &product_command_id,
            phase_b_timestamp,
        )
        .unwrap();
        let product_attempt = output
            .product_command_attempt
            .as_ref()
            .expect("J1-B product command Phase B attempt");

        J1BRealProbeRun {
            product_sidecar_path: real_execution_product_command_sidecar_path(&workflow_state_path)
                .unwrap(),
            continuation_sidecar_path: crate::session_continuation_store::sidecar_path(
                &workflow_state_path,
            )
            .unwrap(),
            runtime_log_path: crate::runtime_log_store::sidecar_path(&workflow_state_path).unwrap(),
            workflow_state_path,
            last_message_path,
            product_command_id,
            product_attempt_id: product_attempt.attempt_id.clone(),
            continuation_id: output.continuation_id.clone().unwrap(),
            continuation_attempt_id: output.continuation_attempt_id.clone().unwrap(),
            runtime_log_ref: output.runtime_log_ref.clone().unwrap(),
            audit_refs: output.audit_refs.clone(),
            output,
        }
    }

    fn j1_b_phase_b_input(
        workflow_state_path: &Path,
        config: &J1BRealProbeConfig,
        product_command_id: &str,
        expected_product_revision: i64,
        expected_continuation_revision: i64,
    ) -> RunRealExecutionProductCommandPhaseBInput {
        let (store, _, _) =
            load_real_execution_product_command_store(workflow_state_path, "2026-06-09T10:00:03Z")
                .unwrap();
        let request = store
            .commands
            .iter()
            .find(|command| command.product_command_id == product_command_id)
            .unwrap();
        let continuation_id = pcr9a_phase_b_continuation_id(&store, product_command_id).unwrap();
        let continuation_store = crate::session_continuation_store::load_store(
            workflow_state_path,
            "2026-06-09T10:00:03Z",
        )
        .unwrap();
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|continuation| continuation.continuation_id == continuation_id)
            .unwrap();

        RunRealExecutionProductCommandPhaseBInput {
            product_command_id: product_command_id.to_string(),
            expected_product_command_store_revision: Some(expected_product_revision),
            expected_session_continuation_store_revision: Some(expected_continuation_revision),
            actor_role: "project_director".to_string(),
            execution_decision: Some("approved_for_phase_b".to_string()),
            authorization: j1_b_authorization_from_request_and_continuation(
                workflow_state_path,
                config,
                request,
                continuation,
            ),
            prompt_body: config.prompt_body.clone(),
            requested_at: Some("2026-06-09T10:00:03Z".to_string()),
        }
    }

    fn j1_b_authorization_from_request_and_continuation(
        workflow_state_path: &Path,
        config: &J1BRealProbeConfig,
        request: &RealExecutionProductCommandRequest,
        continuation: &crate::ControlledSessionContinuation,
    ) -> H2RealResumeAuthorizationMatrix {
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "mario test J1-B Codex Control real resume probe".to_string(),
            project_root: continuation.project_root.clone(),
            target_cwd: continuation.target_cwd.clone(),
            target_session: continuation.session_id.clone(),
            prompt_summary: request.prompt_summary.clone(),
            prompt_sha256: config.prompt_hash.clone(),
            prompt_ref: request.prompt_ref.clone(),
            allowed_write_roots: request.allowed_write_roots.clone(),
            codex_home_scope:
                "Codex CLI minimum native session state for one authorized J1-B real resume; no credential material requested."
                    .to_string(),
            sandbox: continuation.sandbox.clone(),
            timeout_ms: Some(120_000),
            readback_plan:
                "workbench-managed last message plus product command/continuation/runtime/audit refs; unavailable is not zero"
                    .to_string(),
            evidence_path: workflow_state_path
                .parent()
                .unwrap()
                .join("j1-b-real-probe-evidence-ref.json")
                .display()
                .to_string(),
            rollback_plan:
                "J1-B read-only probe requires no project writes; mario core hashes must remain unchanged."
                    .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        }
    }

    fn j1_b_codex_control_input(config: &J1BRealProbeConfig) -> CodexControlCommandInput {
        CodexControlCommandInput {
            project_id: Some("project:users-yoyi-documents-mario-test".to_string()),
            project_root: config.project_root.clone(),
            workflow_id: Some("workflow:users-yoyi-documents-mario-test:default".to_string()),
            node_id: Some(
                "node:j1-b:mario-test:codex-control:real-resume-probe:v1".to_string(),
            ),
            work_item_id: Some(config.work_item_id.clone()),
            task_package_ref: Some(config.artifact_id.clone()),
            memory_packet_ref: Some(config.snapshot_id.clone()),
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            session_mode: "resume".to_string(),
            target_session_id: Some(config.session_id.clone()),
            sandbox: config.sandbox.clone(),
            prompt_summary: config.prompt_summary.clone(),
            prompt_ref: config.prompt_ref.clone(),
            prompt_hash: config.prompt_hash.clone(),
            allowed_write_roots: config.allowed_write_roots.clone(),
            denied_paths: vec![
                "secret".to_string(),
                "token".to_string(),
                ".env".to_string(),
                "keychain".to_string(),
                "OAuth".to_string(),
                "provider credential".to_string(),
                "full transcript".to_string(),
                "rollout".to_string(),
            ],
            readback_plan: "Read only the workbench-managed last message for this attempt and verify the J1-B marker; do not read full transcript."
                .to_string(),
            timeout_ms: Some(120_000),
            requested_by: Some("user".to_string()),
        }
    }

    fn pcr9_phase_b_input(
        workflow_state_path: &Path,
        config: &Pcr9RealProbeConfig,
        product_command_id: &str,
        expected_product_revision: i64,
        expected_continuation_revision: i64,
    ) -> RunRealExecutionProductCommandPhaseBInput {
        let (store, _, _) =
            load_real_execution_product_command_store(workflow_state_path, "2026-06-09T09:00:03Z")
                .unwrap();
        let request = store
            .commands
            .iter()
            .find(|command| command.product_command_id == product_command_id)
            .unwrap();
        let continuation_id = pcr9a_phase_b_continuation_id(&store, product_command_id).unwrap();
        let continuation_store = crate::session_continuation_store::load_store(
            workflow_state_path,
            "2026-06-09T09:00:03Z",
        )
        .unwrap();
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|continuation| continuation.continuation_id == continuation_id)
            .unwrap();

        RunRealExecutionProductCommandPhaseBInput {
            product_command_id: product_command_id.to_string(),
            expected_product_command_store_revision: Some(expected_product_revision),
            expected_session_continuation_store_revision: Some(expected_continuation_revision),
            actor_role: "project_director".to_string(),
            execution_decision: Some("approved_for_phase_b".to_string()),
            authorization: pcr9_authorization_from_request_and_continuation(
                workflow_state_path,
                config,
                request,
                continuation,
            ),
            prompt_body: config.prompt_body.clone(),
            requested_at: Some("2026-06-09T09:00:03Z".to_string()),
        }
    }

    fn pcr9_authorization_from_request_and_continuation(
        workflow_state_path: &Path,
        config: &Pcr9RealProbeConfig,
        request: &RealExecutionProductCommandRequest,
        continuation: &crate::ControlledSessionContinuation,
    ) -> H2RealResumeAuthorizationMatrix {
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: format!(
                "mario test PCR9 {} unified product command real probe",
                config.level
            ),
            project_root: continuation.project_root.clone(),
            target_cwd: continuation.target_cwd.clone(),
            target_session: continuation.session_id.clone(),
            prompt_summary: request.prompt_summary.clone(),
            prompt_sha256: config.prompt_hash.clone(),
            prompt_ref: request.prompt_ref.clone(),
            allowed_write_roots: request.allowed_write_roots.clone(),
            codex_home_scope:
                "Codex CLI minimum native session state for one authorized PCR9 real resume; no credential material requested."
                    .to_string(),
            sandbox: continuation.sandbox.clone(),
            timeout_ms: Some(120_000),
            readback_plan:
                "workbench-managed last message plus product command/continuation/runtime/audit refs; unavailable is not zero"
                    .to_string(),
            evidence_path: workflow_state_path
                .parent()
                .unwrap()
                .join("pcr9-real-probe-evidence-ref.json")
                .display()
                .to_string(),
            rollback_plan:
                "B1 requires no project writes; B2 is limited to .workbench/pcr9 probe file and verified by hashes."
                    .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        }
    }

    fn pcr9_h5_preview_input(
        config: &Pcr9RealProbeConfig,
    ) -> crate::H5ProjectWorkflowDispatchPreviewInput {
        crate::H5ProjectWorkflowDispatchPreviewInput {
            project_root: config.project_root.clone(),
            project_id: "project:users-yoyi-documents-mario-test".to_string(),
            workflow_id: "workflow:users-yoyi-documents-mario-test:default".to_string(),
            dispatch_id: config.dispatch_id.clone(),
            actor_id: "project_director".to_string(),
            operation_id: Some("resume".to_string()),
            session_id: Some(config.session_id.clone()),
            target_cwd: Some(config.project_root.clone()),
            sandbox: Some(config.sandbox.clone()),
            prompt_summary: config.prompt_summary.clone(),
            prompt_ref: config.prompt_ref.clone(),
            prompt_sha256: config.prompt_hash.clone(),
            h3_b_level_b_authorized: false,
            expected_workflow_revision: Some(1),
            diagnostic_summary: Some(crate::H5DiagnosticSummaryInput {
                overall_severity: "ok".to_string(),
                blocked_count: 0,
                degraded_states: vec![],
            }),
        }
    }

    fn write_pcr9_workflow_state(workflow_state_path: &Path, config: &Pcr9RealProbeConfig) {
        let snapshot = serde_json::json!({
            "snapshot_id": config.snapshot_id,
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": format!("task-memory-packet:{}:mario-test:codex-dev:2026-06-09", config.level.to_ascii_lowercase()),
            "project_id": "project:users-yoyi-documents-mario-test",
            "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
            "work_item_id": config.work_item_id,
            "task_package_artifact_id": config.artifact_id,
            "role_id": "codex-dev",
            "retrieval_intent": "worker_task",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": config.memory_fingerprint,
            "generated_at": "2026-06-09T09:00:00Z",
            "stale": false,
            "stale_reasons": [],
            "warnings": []
        });
        let value = serde_json::json!({
            "revision": 1,
            "workflow_node_dispatches": [{
                "dispatch_id": config.dispatch_id,
                "project_id": "project:users-yoyi-documents-mario-test",
                "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
                "node_id": "workflow:users-yoyi-documents-mario-test:default:node:codex-dev",
                "work_item_id": config.work_item_id,
                "native_thread_id": config.session_id,
                "prompt_preview": "redacted prompt preview; body is workbench-managed and sent via stdin only",
                "prompt_kind": format!("pcr9_{}_unified_product_command_real_probe", config.level.to_ascii_lowercase()),
                "memory_packet_snapshot_id": config.snapshot_id,
                "memory_packet_fingerprint": config.memory_fingerprint,
                "plan_authorization_id": format!("authorization:pcr9-{}:mario-test:codex-dev:v1", config.level.to_ascii_lowercase()),
                "task_package_id": config.artifact_id,
                "authorization_check": {"status": "authorized"},
                "state": "prepared"
            }],
            "artifacts": [{
                "artifact_id": config.artifact_id,
                "artifact_type": "task_package",
                "source_ref": config.work_item_id,
                "allowed_write": config.allowed_write_roots,
                "memory_packet_snapshot": snapshot
            }],
            "projects": [{
                "project_id": "project:users-yoyi-documents-mario-test",
                "project_root": config.project_root
            }]
        });
        fs::write(
            workflow_state_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn write_j1_b_workflow_state(workflow_state_path: &Path, config: &J1BRealProbeConfig) {
        let snapshot = serde_json::json!({
            "snapshot_id": config.snapshot_id,
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": "task-memory-packet:j1-b:mario-test:codex-control:2026-06-09",
            "project_id": "project:users-yoyi-documents-mario-test",
            "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
            "work_item_id": config.work_item_id,
            "task_package_artifact_id": config.artifact_id,
            "role_id": "project_director",
            "retrieval_intent": "codex_control_resume_probe",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": config.memory_fingerprint,
            "generated_at": "2026-06-09T10:00:00Z",
            "stale": false,
            "stale_reasons": [],
            "warnings": []
        });
        let value = serde_json::json!({
            "revision": 1,
            "workflow_node_dispatches": [{
                "dispatch_id": "dispatch:j1-b:mario-test:codex-control:real-resume-probe:v1",
                "project_id": "project:users-yoyi-documents-mario-test",
                "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
                "node_id": "node:j1-b:mario-test:codex-control:real-resume-probe:v1",
                "work_item_id": config.work_item_id,
                "native_thread_id": config.session_id,
                "prompt_preview": "redacted prompt preview; body is runtime stdin only",
                "prompt_kind": "j1_b_codex_control_real_resume_probe",
                "memory_packet_snapshot_id": config.snapshot_id,
                "memory_packet_fingerprint": config.memory_fingerprint,
                "plan_authorization_id": "authorization:j1-b:mario-test:codex-control:v1",
                "task_package_id": config.artifact_id,
                "authorization_check": {"status": "authorized"},
                "state": "prepared"
            }],
            "artifacts": [{
                "artifact_id": config.artifact_id,
                "artifact_type": "task_package",
                "source_ref": config.work_item_id,
                "allowed_write": config.allowed_write_roots,
                "memory_packet_snapshot": snapshot
            }],
            "projects": [{
                "project_id": "project:users-yoyi-documents-mario-test",
                "project_root": config.project_root
            }]
        });
        fs::write(
            workflow_state_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn sha256_file(path: &Path) -> String {
        let bytes = fs::read(path).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    }

    fn mario_core_file_hashes(project_root: &Path) -> Vec<(String, String)> {
        ["index.html", "styles.css", "game.js", "README.md"]
            .iter()
            .map(|file| {
                let path = project_root.join(file);
                ((*file).to_string(), sha256_file(&path))
            })
            .collect()
    }

    fn project_file_set(project_root: &Path) -> BTreeSet<String> {
        project_file_hashes(project_root).into_keys().collect()
    }

    fn project_file_hashes(project_root: &Path) -> BTreeMap<String, String> {
        fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<String, String>) {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(_) => return,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap_or(&path);
                let relative_text = relative.to_string_lossy().replace('\\', "/");
                if relative_text.starts_with(".git/") || relative_text == ".git" {
                    continue;
                }
                if path.is_dir() {
                    visit(root, &path, files);
                } else if path.is_file() {
                    files.insert(relative_text, sha256_file(&path));
                }
            }
        }

        let mut files = BTreeMap::new();
        visit(project_root, project_root, &mut files);
        files
    }

    fn changed_project_files(
        before: &BTreeMap<String, String>,
        after: &BTreeMap<String, String>,
    ) -> Vec<String> {
        let mut changed = after
            .iter()
            .filter_map(|(path, hash)| {
                if before.get(path) != Some(hash) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        changed.extend(
            before
                .keys()
                .filter(|path| !after.contains_key(*path))
                .map(|path| format!("deleted:{path}")),
        );
        changed.sort();
        changed.dedup();
        changed
    }

    fn assert_product_command_store_does_not_persist_prompt(path: &Path, prompt_body: &str) {
        let text = fs::read_to_string(path).unwrap_or_default();
        assert!(
            !text.contains(prompt_body),
            "prompt body must not be persisted in {}",
            path.display()
        );
    }

    struct K2RealProbeRun {
        output: RealExecutionProductCommandPhaseBOutput,
        workflow_state_path: PathBuf,
        product_sidecar_path: PathBuf,
        continuation_sidecar_path: PathBuf,
        runtime_log_path: PathBuf,
        last_message_path: PathBuf,
        product_command_id: String,
        product_attempt_id: String,
        continuation_id: String,
        continuation_attempt_id: String,
        runtime_log_ref: String,
        audit_refs: Vec<String>,
    }

    fn k2_require_real_confirmation(prefix: &str, execution_point_id: &str) {
        let key = format!("{prefix}_USER_CONFIRMED");
        let value = env::var(&key).unwrap_or_else(|_| {
            panic!("{key} must equal {execution_point_id} for real K2 execution")
        });
        assert_eq!(
            value, execution_point_id,
            "{key} must exactly match the K2 execution point id"
        );
    }

    fn k2_real_workflow_state_path(prefix: &str, config: &K2ExecutionPointConfig) -> PathBuf {
        let key = format!("{prefix}_WORKFLOW_STATE_PARENT");
        let parent = PathBuf::from(
            env::var(&key).unwrap_or_else(|_| panic!("{key} is required for real K2 execution")),
        );
        assert!(
            parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/stage-k-k2-real-execution/runs"
            ),
            "{key} must be inside product-line/tmp/stage-k-k2-real-execution/runs"
        );
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = parent.join(format!(
            "{}-{}-{unique}",
            config.execution_point_id,
            config.prompt_hash.chars().take(12).collect::<String>()
        ));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        write_k2_workflow_state(&workflow_state_path, config);
        workflow_state_path
    }

    fn write_k2_workflow_state(workflow_state_path: &Path, config: &K2ExecutionPointConfig) {
        let value = serde_json::json!({
            "schema_version": "workflow-state.v0",
            "revision": 1,
            "stage": "stage-k-k2-real-execution-probe",
            "projects": [{
                "project_id": config.project_id,
                "project_root": config.project_root
            }],
            "warnings": [
                "stage_k_k2_real_execution_probe_state",
                "prompt_body_not_persisted_in_product_command_sidecar"
            ]
        });
        fs::write(
            workflow_state_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn run_k2_real_resume_probe(prefix: &str, config: &K2ExecutionPointConfig) -> K2RealProbeRun {
        let workflow_state_path = k2_real_workflow_state_path(prefix, config);
        let prepare = prepare_real_execution_product_command_at(
            &workflow_state_path,
            &k2_prepare_input(config, Some(0), Some("2026-06-10T01:00:00Z".to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(prepare.blocked_reasons.is_empty());
        let product_command_id = prepare.product_command_id.clone().unwrap();
        let decision = record_real_execution_product_command_decision_at(
            &workflow_state_path,
            &k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some("2026-06-10T01:00:01Z".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");
        let phase_a = run_real_execution_product_command_phase_a_at(
            &workflow_state_path,
            &k2_phase_a_input(
                &product_command_id,
                decision.store_revision,
                Some("2026-06-10T01:00:02Z".to_string()),
            ),
            "2026-06-10T01:00:02Z",
            &format!("{prefix}-phase-a"),
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");
        let phase_b_input = k2_resume_phase_b_input(
            &workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            Some("2026-06-10T01:00:03Z".to_string()),
        )
        .unwrap();
        let output = run_real_execution_product_command_phase_b_at(
            &workflow_state_path,
            &phase_b_input,
            "2026-06-10T01:00:03Z",
            &format!("{prefix}-phase-b"),
        )
        .unwrap();
        k2_real_probe_run_from_output(workflow_state_path, product_command_id, output)
    }

    fn run_k2_real_new_session_probe(
        prefix: &str,
        config: &K2ExecutionPointConfig,
    ) -> K2RealProbeRun {
        let workflow_state_path = k2_real_workflow_state_path(prefix, config);
        let prepare = prepare_real_execution_product_command_at(
            &workflow_state_path,
            &k2_prepare_input(config, Some(0), Some("2026-06-10T01:00:00Z".to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(prepare.blocked_reasons.is_empty());
        let product_command_id = prepare.product_command_id.clone().unwrap();
        let decision = record_real_execution_product_command_decision_at(
            &workflow_state_path,
            &k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some("2026-06-10T01:00:01Z".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");
        let phase_a = run_real_execution_product_command_phase_a_at(
            &workflow_state_path,
            &k2_phase_a_input(
                &product_command_id,
                decision.store_revision,
                Some("2026-06-10T01:00:02Z".to_string()),
            ),
            "2026-06-10T01:00:02Z",
            &format!("{prefix}-phase-a"),
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");
        let phase_b_input = k2_new_session_phase_b_input(
            &workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            Some("2026-06-10T01:00:03Z".to_string()),
        )
        .unwrap();
        let output = run_real_execution_product_command_new_session_phase_b_at(
            &workflow_state_path,
            &phase_b_input,
            "2026-06-10T01:00:03Z",
            &format!("{prefix}-phase-b"),
        )
        .unwrap();
        k2_real_probe_run_from_output(workflow_state_path, product_command_id, output)
    }

    fn k2_real_probe_run_from_output(
        workflow_state_path: PathBuf,
        product_command_id: String,
        output: RealExecutionProductCommandPhaseBOutput,
    ) -> K2RealProbeRun {
        let product_attempt = output
            .product_command_attempt
            .as_ref()
            .expect("K2 product command Phase B attempt");
        let last_message_path = pcr9a_phase_b_last_message_path(
            &workflow_state_path,
            &product_command_id,
            "2026-06-10T01:00:03Z",
        )
        .unwrap();
        K2RealProbeRun {
            product_sidecar_path: real_execution_product_command_sidecar_path(&workflow_state_path)
                .unwrap(),
            continuation_sidecar_path: crate::session_continuation_store::sidecar_path(
                &workflow_state_path,
            )
            .unwrap(),
            runtime_log_path: crate::runtime_log_store::sidecar_path(&workflow_state_path).unwrap(),
            workflow_state_path,
            last_message_path,
            product_command_id,
            product_attempt_id: product_attempt.attempt_id.clone(),
            continuation_id: output.continuation_id.clone().unwrap(),
            continuation_attempt_id: output.continuation_attempt_id.clone().unwrap(),
            runtime_log_ref: output.runtime_log_ref.clone().unwrap(),
            audit_refs: output.audit_refs.clone(),
            output,
        }
    }

    fn k2_fixture_state(label: &str) -> (PathBuf, PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};

        static K2_FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = env::temp_dir().join(format!(
            "k2-product-command-{}-{}-{label}",
            crate::unix_timestamp_nanos(),
            K2_FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(
            &workflow_state_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "revision": 1,
                "stage": "k2-real-execution-harness-test",
                "warnings": ["fixture_only_no_real_codex"]
            }))
            .unwrap(),
        )
        .unwrap();
        (dir, workflow_state_path)
    }

    fn run_k2_resume_fake_chain(
        workflow_state_path: &Path,
        config: &K2ExecutionPointConfig,
        runner: K2FakePhaseBRunner,
    ) -> (
        RealExecutionProductCommandPhaseBOutput,
        PathBuf,
        PathBuf,
        PathBuf,
    ) {
        let prepare = prepare_real_execution_product_command_at(
            workflow_state_path,
            &k2_prepare_input(config, Some(0), Some("2026-06-10T00:00:00Z".to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(prepare.blocked_reasons.is_empty());
        let product_command_id = prepare.product_command_id.clone().unwrap();
        let decision = record_real_execution_product_command_decision_at(
            workflow_state_path,
            &k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some("2026-06-10T00:00:01Z".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");
        let phase_a = run_real_execution_product_command_phase_a_at(
            workflow_state_path,
            &k2_phase_a_input(
                &product_command_id,
                decision.store_revision,
                Some("2026-06-10T00:00:02Z".to_string()),
            ),
            "2026-06-10T00:00:02Z",
            "k2-resume-phase-a",
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");
        assert!(!phase_a.runner_call_allowed);
        assert!(!phase_a.prompt_sent);
        let phase_b_input = k2_resume_phase_b_input(
            workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            Some("2026-06-10T00:00:03Z".to_string()),
        )
        .unwrap();
        let last_message_path = workflow_state_path
            .parent()
            .unwrap()
            .join("k2-resume.last-message.txt");
        let phase_b = run_real_execution_product_command_phase_b_with_runner(
            workflow_state_path,
            &phase_b_input,
            "2026-06-10T00:00:03Z",
            "k2-resume-phase-b",
            &last_message_path,
            &runner,
        )
        .unwrap();

        (
            phase_b,
            real_execution_product_command_sidecar_path(workflow_state_path).unwrap(),
            crate::session_continuation_store::sidecar_path(workflow_state_path).unwrap(),
            crate::runtime_log_store::sidecar_path(workflow_state_path).unwrap(),
        )
    }

    fn run_k2_new_session_fake_chain(
        workflow_state_path: &Path,
        config: &K2ExecutionPointConfig,
        runner: K2FakePhaseBRunner,
    ) -> (
        RealExecutionProductCommandPhaseBOutput,
        PathBuf,
        PathBuf,
        PathBuf,
    ) {
        let prepare = prepare_real_execution_product_command_at(
            workflow_state_path,
            &k2_prepare_input(config, Some(0), Some("2026-06-10T00:00:00Z".to_string())).unwrap(),
        )
        .unwrap();
        assert_eq!(prepare.status, "prepared");
        assert!(prepare.blocked_reasons.is_empty());
        let product_command_id = prepare.product_command_id.clone().unwrap();
        let decision = record_real_execution_product_command_decision_at(
            workflow_state_path,
            &k2_decision_input(
                &product_command_id,
                prepare.store_revision,
                Some("2026-06-10T00:00:01Z".to_string()),
            ),
        )
        .unwrap();
        assert_eq!(decision.status, "decision_recorded");
        let phase_a = run_real_execution_product_command_phase_a_at(
            workflow_state_path,
            &k2_phase_a_input(
                &product_command_id,
                decision.store_revision,
                Some("2026-06-10T00:00:02Z".to_string()),
            ),
            "2026-06-10T00:00:02Z",
            "k2-new-session-phase-a",
        )
        .unwrap();
        assert_eq!(phase_a.status, "phase_a_completed");
        assert!(!phase_a.runner_call_allowed);
        assert!(!phase_a.prompt_sent);
        let phase_b_input = k2_new_session_phase_b_input(
            workflow_state_path,
            config,
            &product_command_id,
            phase_a.product_command_store_revision,
            phase_a.session_continuation_store_revision.unwrap(),
            Some("2026-06-10T00:00:03Z".to_string()),
        )
        .unwrap();
        let last_message_path = workflow_state_path
            .parent()
            .unwrap()
            .join("k2-new-session.last-message.txt");
        let phase_b = run_real_execution_product_command_new_session_phase_b_with_runner(
            workflow_state_path,
            &phase_b_input,
            "2026-06-10T00:00:03Z",
            "k2-new-session-phase-b",
            &last_message_path,
            &runner,
        )
        .unwrap();

        (
            phase_b,
            real_execution_product_command_sidecar_path(workflow_state_path).unwrap(),
            crate::session_continuation_store::sidecar_path(workflow_state_path).unwrap(),
            crate::runtime_log_store::sidecar_path(workflow_state_path).unwrap(),
        )
    }

    fn assert_k2_sidecars_do_not_persist_prompt<const N: usize>(
        paths: [&PathBuf; N],
        prompt_body: &str,
    ) {
        for path in paths {
            assert_product_command_store_does_not_persist_prompt(path, prompt_body);
        }
    }

    struct K2FakePhaseBRunner {
        expected_prompt_hash: String,
        marker: String,
        writes_project_files: bool,
        readback_status: String,
        readback_result_count: Option<i64>,
    }

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for K2FakePhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            prompt_body: &str,
            last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            assert_eq!(sha256_hex(prompt_body), self.expected_prompt_hash);
            if let Some(parent) = last_message_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(
                last_message_path,
                format!("{}\nK2 fake runner status only.\n", self.marker),
            )
            .unwrap();
            codex_local_runner::CodexLocalPhaseBProcessResult {
                runner_kind: "k2_fake_phase_b_runner_no_real_process".to_string(),
                status: "succeeded".to_string(),
                exit_code: Some(0),
                timed_out: false,
                prompt_sent: true,
                real_codex_executed: true,
                writes_codex_home: true,
                writes_project_files: self.writes_project_files,
                readback_status: self.readback_status.clone(),
                readback_attempted: true,
                readback_result_count: self.readback_result_count,
                last_message_path: Some(last_message_path.display().to_string()),
                failure_code: None,
                failure_message: None,
                retryable: false,
                user_action_required: false,
                warnings: vec![
                    "k2_fake_runner_no_real_process_spawned".to_string(),
                    "default_test_does_not_execute_codex".to_string(),
                ],
            }
        }
    }

    struct K2PanicPhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for K2PanicPhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            _prompt_body: &str,
            _last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            panic!("K2 blocked test must not call Phase B runner");
        }
    }

    #[test]
    fn pcr4_phase_a_blocks_without_approved_user_decision_and_does_not_write() {
        for (name, decision) in [
            ("missing", None),
            ("rejected", Some("rejected")),
            ("request_changes", Some("request_changes")),
        ] {
            let (_dir, path, command_id, revision) = pcr3_prepared_command_fixture();
            let expected_revision = if let Some(decision) = decision {
                let mut input = pcr3_decision_input(&command_id, decision, revision);
                input.confirmed_by = "project_director".to_string();
                input.allowed_once = false;
                input.risk_acknowledgement = String::new();
                record_real_execution_product_command_decision_at(&path, &input)
                    .unwrap()
                    .store_revision
            } else {
                revision
            };
            let before = pcr3_sidecar_text(&path);
            let output = run_real_execution_product_command_phase_a_at(
                &path,
                &pcr4_phase_a_input(&command_id, expected_revision),
                "2026-06-09T00:00:04Z",
                &format!("pcr4-phase-a-blocked-{name}"),
            )
            .unwrap();

            assert_eq!(output.status, "phase_a_blocked", "{name}");
            assert!(!output.writes_product_command_sidecar, "{name}");
            assert!(!output.writes_continuation_sidecar, "{name}");
            assert!(!output.writes_runtime_log, "{name}");
            assert!(!output.runner_call_allowed, "{name}");
            assert_eq!(pcr3_sidecar_text(&path), before, "{name}");
            assert!(
                !crate::session_continuation_store::sidecar_path(&path)
                    .unwrap()
                    .exists(),
                "{name}"
            );
            assert!(
                !crate::runtime_log_store::sidecar_path(&path)
                    .unwrap()
                    .exists(),
                "{name}"
            );
        }
    }

    #[test]
    fn pcr4_phase_a_revision_conflict_blocked_preview_duplicate_and_corrupt_json_do_not_write() {
        let (_dir, path, command_id, revision) = pcr4_approved_command_fixture();
        let before = pcr3_sidecar_text(&path);
        let conflict = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, revision + 7),
            "2026-06-09T00:00:04Z",
            "pcr4-conflict",
        )
        .unwrap();
        assert_eq!(conflict.status, "store_conflict");
        assert_eq!(pcr3_sidecar_text(&path), before);
        assert!(!crate::session_continuation_store::sidecar_path(&path)
            .unwrap()
            .exists());

        let (_dir, blocked_path, blocked_command_id, blocked_revision) =
            pcr4_approved_command_fixture();
        let sidecar = real_execution_product_command_sidecar_path(&blocked_path).unwrap();
        let (mut store, _, _) =
            load_real_execution_product_command_store(&blocked_path, "2026-06-09T00:00:04Z")
                .unwrap();
        store.previews[0]
            .blocked_reasons
            .push("forced_blocked_preview".to_string());
        store.previews[0].readiness.status = "blocked_for_test".to_string();
        store.previews[0]
            .readiness
            .blocked_reasons
            .push("forced_blocked_preview".to_string());
        std::fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap()).unwrap();
        let before_blocked = std::fs::read_to_string(&sidecar).unwrap();
        let blocked = run_real_execution_product_command_phase_a_at(
            &blocked_path,
            &pcr4_phase_a_input(&blocked_command_id, blocked_revision),
            "2026-06-09T00:00:04Z",
            "pcr4-blocked-preview",
        )
        .unwrap();
        assert_eq!(blocked.status, "phase_a_blocked");
        assert!(blocked
            .blocked_reasons
            .contains(&"phase_a_preview_not_ready".to_string()));
        assert_eq!(std::fs::read_to_string(&sidecar).unwrap(), before_blocked);

        let (_dir, duplicate_path, duplicate_command_id, duplicate_revision) =
            pcr4_approved_command_fixture();
        let duplicate = run_real_execution_product_command_phase_a_at(
            &duplicate_path,
            &pcr4_phase_a_input(&duplicate_command_id, duplicate_revision),
            "2026-06-09T00:00:04Z",
            "pcr4-duplicate-first",
        )
        .unwrap();
        assert_eq!(duplicate.status, "phase_a_completed");
        let before_duplicate = pcr3_sidecar_text(&duplicate_path);
        let duplicate_again = run_real_execution_product_command_phase_a_at(
            &duplicate_path,
            &pcr4_phase_a_input(
                &duplicate_command_id,
                duplicate.product_command_store_revision,
            ),
            "2026-06-09T00:00:05Z",
            "pcr4-duplicate-second",
        )
        .unwrap();
        assert_eq!(duplicate_again.status, "phase_a_blocked");
        assert!(duplicate_again
            .blocked_reasons
            .contains(&"phase_a_duplicate_running_or_completed_attempt".to_string()));
        assert_eq!(pcr3_sidecar_text(&duplicate_path), before_duplicate);

        let (_dir, corrupt_path, corrupt_command_id, corrupt_revision) =
            pcr4_approved_command_fixture();
        let corrupt_sidecar = real_execution_product_command_sidecar_path(&corrupt_path).unwrap();
        std::fs::write(&corrupt_sidecar, "{not json").unwrap();
        let err = run_real_execution_product_command_phase_a_at(
            &corrupt_path,
            &pcr4_phase_a_input(&corrupt_command_id, corrupt_revision),
            "2026-06-09T00:00:04Z",
            "pcr4-corrupt-json",
        )
        .expect_err("damaged product command sidecar must not be overwritten");
        assert!(err.contains("parse real execution product command sidecar failed"));
        assert_eq!(
            std::fs::read_to_string(corrupt_sidecar).unwrap(),
            "{not json"
        );
    }

    #[test]
    fn pcr4_phase_a_corrupt_runtime_log_preflight_does_not_write_partial_continuation() {
        let (_dir, path, command_id, revision) = pcr4_approved_command_fixture();
        let runtime_sidecar = crate::runtime_log_store::sidecar_path(&path).unwrap();
        std::fs::write(&runtime_sidecar, "{not json").unwrap();
        let before_product = pcr3_sidecar_text(&path);
        let err = run_real_execution_product_command_phase_a_at(
            &path,
            &pcr4_phase_a_input(&command_id, revision),
            "2026-06-09T00:00:04Z",
            "pcr4-corrupt-runtime",
        )
        .expect_err("corrupt runtime log must block before continuation write");

        assert!(err.contains("runtime_log_sidecar_unreadable_refuse_h2_attempt"));
        assert_eq!(pcr3_sidecar_text(&path), before_product);
        assert!(!crate::session_continuation_store::sidecar_path(&path)
            .unwrap()
            .exists());
        assert_eq!(
            std::fs::read_to_string(runtime_sidecar).unwrap(),
            "{not json"
        );
    }

    fn pcr2_blocked_preview_case(
        memory_fixture: Pcr2MemoryFixture,
        duplicate_attempt: bool,
        diagnostic_summary: Option<crate::H5DiagnosticSummaryInput>,
        expected_reasons: &[&str],
    ) {
        let (dir, path) = pcr2_fixture_state(memory_fixture, duplicate_attempt);
        let input = PreviewRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(
                dir.display().to_string(),
                diagnostic_summary,
            )),
            codex_control: None,
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let preview = preview_real_execution_product_command_at(&path, &input).unwrap();
        let sidecar = real_execution_product_command_sidecar_path(&path).unwrap();

        assert_eq!(preview.readiness.status, "blocked_pcr2_preview");
        for reason in expected_reasons {
            assert!(
                preview.blocked_reasons.contains(&reason.to_string()),
                "{reason}: {:?}",
                preview.blocked_reasons
            );
        }
        assert!(!preview.prompt_sent);
        assert!(!preview.real_codex_executed);
        assert!(!preview.writes_codex_home);
        assert!(!preview.writes_project_files);
        assert!(!sidecar.exists(), "blocked preview must remain read-only");
    }

    fn pcr3_prepared_command_fixture() -> (std::path::PathBuf, std::path::PathBuf, String, i64) {
        let (dir, path) = pcr2_fixture_state(Pcr2MemoryFixture::Fresh, false);
        let input = PrepareRealExecutionProductCommandInput {
            source_kind: "h5_project_workflow_dispatch".to_string(),
            h5_dispatch_preview: Some(pcr2_preview_request(dir.display().to_string(), None)),
            codex_control: None,
            expected_store_revision: Some(0),
            requested_by: Some("project_director".to_string()),
            created_at: Some("2026-06-09T00:00:00Z".to_string()),
        };
        let output = prepare_real_execution_product_command_at(&path, &input).unwrap();
        (
            dir,
            path,
            output.product_command_id.unwrap(),
            output.store_revision,
        )
    }

    fn j1_codex_control_input(
        project_root: String,
        operation_id: &str,
        target_session_id: Option<&str>,
        prompt_body: &str,
    ) -> CodexControlCommandInput {
        CodexControlCommandInput {
            project_id: Some("project:j1".to_string()),
            project_root: project_root.clone(),
            workflow_id: Some("workflow:j1".to_string()),
            node_id: Some("node:j1".to_string()),
            work_item_id: Some("work-item:j1".to_string()),
            task_package_ref: Some("task-package:j1".to_string()),
            memory_packet_ref: Some("memory-packet:j1".to_string()),
            adapter_id: "codex-local".to_string(),
            operation_id: operation_id.to_string(),
            session_mode: if operation_id == "new_session" {
                "new_session_preview_only".to_string()
            } else {
                "resume_existing_session".to_string()
            },
            target_session_id: target_session_id.map(str::to_string),
            sandbox: "read-only".to_string(),
            prompt_summary: "J1 Codex Control task summary only".to_string(),
            prompt_ref: "workbench-runtime-prompt:j1:summary-ref-only".to_string(),
            prompt_hash: sha256_hex(prompt_body),
            allowed_write_roots: vec![project_root],
            denied_paths: vec![
                "secret".to_string(),
                "token".to_string(),
                ".env".to_string(),
                "keychain".to_string(),
                "OAuth".to_string(),
                "provider credential".to_string(),
                "full transcript".to_string(),
                "rollout".to_string(),
            ],
            readback_plan: "readback_unavailable_is_not_zero_results".to_string(),
            timeout_ms: Some(120_000),
            requested_by: Some("user".to_string()),
        }
    }

    fn pcr3_decision_input(
        product_command_id: &str,
        decision: &str,
        expected_store_revision: i64,
    ) -> RecordRealExecutionProductCommandDecisionInput {
        RecordRealExecutionProductCommandDecisionInput {
            product_command_id: product_command_id.to_string(),
            decision: decision.to_string(),
            expected_store_revision: Some(expected_store_revision),
            confirmed_by: "user".to_string(),
            risk_acknowledgement: "I understand this is a one-shot Level B permission record."
                .to_string(),
            allowed_once: true,
            reason: "User approved this prepared command only.".to_string(),
            requested_by: Some("project_director".to_string()),
            confirmed_at: Some("2026-06-09T00:00:03Z".to_string()),
        }
    }

    fn pcr3_sidecar_text(workflow_state_path: &Path) -> String {
        let sidecar = real_execution_product_command_sidecar_path(workflow_state_path).unwrap();
        std::fs::read_to_string(sidecar).unwrap()
    }

    fn pcr4_approved_command_fixture() -> (std::path::PathBuf, std::path::PathBuf, String, i64) {
        let (dir, path, command_id, revision) = pcr3_prepared_command_fixture();
        let decision = record_real_execution_product_command_decision_at(
            &path,
            &pcr3_decision_input(&command_id, "approved", revision),
        )
        .unwrap();
        (dir, path, command_id, decision.store_revision)
    }

    fn pcr4_phase_a_input(
        product_command_id: &str,
        expected_revision: i64,
    ) -> RunRealExecutionProductCommandPhaseAInput {
        RunRealExecutionProductCommandPhaseAInput {
            product_command_id: product_command_id.to_string(),
            expected_product_command_store_revision: Some(expected_revision),
            expected_session_continuation_store_revision: None,
            actor_role: "project_director".to_string(),
            execution_decision: Some("phase_a_noop".to_string()),
            timeout_ms: Some(1_000),
            requested_at: Some("2026-06-09T00:00:04Z".to_string()),
        }
    }

    fn pcr9a_phase_b_input(
        workflow_state_path: &Path,
        product_command_id: &str,
        expected_product_revision: i64,
        expected_continuation_revision: i64,
        prompt_body: &str,
    ) -> RunRealExecutionProductCommandPhaseBInput {
        let (store, _, _) =
            load_real_execution_product_command_store(workflow_state_path, "2026-06-09T00:00:06Z")
                .unwrap();
        let request = store
            .commands
            .iter()
            .find(|command| command.product_command_id == product_command_id)
            .unwrap();
        let continuation_id = pcr9a_phase_b_continuation_id(&store, product_command_id).unwrap();
        let continuation_store = crate::session_continuation_store::load_store(
            workflow_state_path,
            "2026-06-09T00:00:06Z",
        )
        .unwrap();
        let continuation = continuation_store
            .continuations
            .iter()
            .find(|continuation| continuation.continuation_id == continuation_id)
            .unwrap();
        RunRealExecutionProductCommandPhaseBInput {
            product_command_id: product_command_id.to_string(),
            expected_product_command_store_revision: Some(expected_product_revision),
            expected_session_continuation_store_revision: Some(expected_continuation_revision),
            actor_role: "project_director".to_string(),
            execution_decision: Some("approved_for_phase_b".to_string()),
            authorization: pcr9a_authorization_from_request_and_continuation(
                workflow_state_path,
                request,
                continuation,
                prompt_body,
            ),
            prompt_body: prompt_body.to_string(),
            requested_at: Some("2026-06-09T00:00:06Z".to_string()),
        }
    }

    fn pcr9a_phase_b_input_without_continuation(
        workflow_state_path: &Path,
        product_command_id: &str,
        expected_product_revision: i64,
        prompt_body: &str,
    ) -> RunRealExecutionProductCommandPhaseBInput {
        let (store, _, _) =
            load_real_execution_product_command_store(workflow_state_path, "2026-06-09T00:00:06Z")
                .unwrap();
        let request = store
            .commands
            .iter()
            .find(|command| command.product_command_id == product_command_id)
            .unwrap();
        RunRealExecutionProductCommandPhaseBInput {
            product_command_id: product_command_id.to_string(),
            expected_product_command_store_revision: Some(expected_product_revision),
            expected_session_continuation_store_revision: None,
            actor_role: "project_director".to_string(),
            execution_decision: Some("approved_for_phase_b".to_string()),
            authorization: pcr9a_authorization_from_request(
                workflow_state_path,
                request,
                prompt_body,
            ),
            prompt_body: prompt_body.to_string(),
            requested_at: Some("2026-06-09T00:00:06Z".to_string()),
        }
    }

    fn pcr9a_authorization_from_request_and_continuation(
        workflow_state_path: &Path,
        request: &RealExecutionProductCommandRequest,
        continuation: &crate::ControlledSessionContinuation,
        prompt_body: &str,
    ) -> H2RealResumeAuthorizationMatrix {
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: request
                .project_id
                .clone()
                .unwrap_or_else(|| "pcr9a-product-command".to_string()),
            project_root: continuation.project_root.clone(),
            target_cwd: continuation.target_cwd.clone(),
            target_session: continuation.session_id.clone(),
            prompt_summary: request.prompt_summary.clone(),
            prompt_sha256: sha256_hex(prompt_body),
            prompt_ref: request.prompt_ref.clone(),
            allowed_write_roots: request.allowed_write_roots.clone(),
            codex_home_scope: "user_authorized_codex_home_for_pcr9a_phase_b".to_string(),
            sandbox: continuation.sandbox.clone(),
            timeout_ms: request.timeout_ms.or(Some(1_000)),
            readback_plan: if request.readback_plan.trim().is_empty() {
                "readback_unavailable_is_not_zero_results".to_string()
            } else {
                request.readback_plan.clone()
            },
            evidence_path: workflow_state_path
                .parent()
                .unwrap()
                .join("evidence")
                .join("pcr9a-product-command-phase-b.json")
                .display()
                .to_string(),
            rollback_plan: "PCR9A test bridge uses fake runner; no real process rollback required."
                .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        }
    }

    fn pcr9a_authorization_from_request(
        workflow_state_path: &Path,
        request: &RealExecutionProductCommandRequest,
        prompt_body: &str,
    ) -> H2RealResumeAuthorizationMatrix {
        let project_root = request
            .project_root
            .clone()
            .unwrap_or_else(|| workflow_state_path.parent().unwrap().display().to_string());
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: request
                .project_id
                .clone()
                .unwrap_or_else(|| "pcr9a-product-command".to_string()),
            project_root: project_root.clone(),
            target_cwd: project_root.clone(),
            target_session: request.target_session_id.clone().unwrap_or_default(),
            prompt_summary: request.prompt_summary.clone(),
            prompt_sha256: sha256_hex(prompt_body),
            prompt_ref: request.prompt_ref.clone(),
            allowed_write_roots: request.allowed_write_roots.clone(),
            codex_home_scope: "user_authorized_codex_home_for_pcr9a_phase_b".to_string(),
            sandbox: pcr9a_product_command_sandbox(request),
            timeout_ms: request.timeout_ms.or(Some(1_000)),
            readback_plan: request.readback_plan.clone(),
            evidence_path: workflow_state_path
                .parent()
                .unwrap()
                .join("evidence")
                .join("pcr9a-product-command-phase-b.json")
                .display()
                .to_string(),
            rollback_plan: "PCR9A product gate blocked before runner.".to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Pcr9aBlockedCase {
        MissingDecision,
        PromptHashMismatch,
        PlannedAdapter,
        UnsupportedOperation,
    }

    fn mutate_pcr9a_blocked_case(workflow_state_path: &Path, case: Pcr9aBlockedCase) {
        if matches!(
            case,
            Pcr9aBlockedCase::PromptHashMismatch | Pcr9aBlockedCase::MissingDecision
        ) {
            return;
        }
        let sidecar = real_execution_product_command_sidecar_path(workflow_state_path).unwrap();
        let (mut store, _, _) =
            load_real_execution_product_command_store(workflow_state_path, "2026-06-09T00:00:05Z")
                .unwrap();
        match case {
            Pcr9aBlockedCase::PlannedAdapter => {
                store.commands[0].adapter_id = "planned-adapter".to_string();
                store.previews[0].request.adapter_id = "planned-adapter".to_string();
            }
            Pcr9aBlockedCase::UnsupportedOperation => {
                store.commands[0].operation_id = "send_message".to_string();
                store.previews[0].request.operation_id = "send_message".to_string();
            }
            Pcr9aBlockedCase::MissingDecision | Pcr9aBlockedCase::PromptHashMismatch => {}
        }
        std::fs::write(&sidecar, serde_json::to_string_pretty(&store).unwrap()).unwrap();
    }

    fn assert_pcr9a_blocked_reason(reasons: &[String], case: Pcr9aBlockedCase) {
        let expected = match case {
            Pcr9aBlockedCase::MissingDecision => "phase_b_requires_user_approved_decision",
            Pcr9aBlockedCase::PromptHashMismatch => "phase_b_prompt_hash_mismatch",
            Pcr9aBlockedCase::PlannedAdapter => "phase_b_only_supports_codex_local_adapter",
            Pcr9aBlockedCase::UnsupportedOperation => "phase_b_only_supports_resume",
        };
        assert!(
            reasons.contains(&expected.to_string()),
            "{expected}: {reasons:?}"
        );
    }

    struct Pcr9aFakePhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for Pcr9aFakePhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            _prompt_body: &str,
            last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            codex_local_runner::CodexLocalPhaseBProcessResult {
                runner_kind: "pcr9a_fake_phase_b_runner".to_string(),
                status: "succeeded".to_string(),
                exit_code: Some(0),
                timed_out: false,
                prompt_sent: true,
                real_codex_executed: true,
                writes_codex_home: true,
                writes_project_files: false,
                readback_status: "succeeded".to_string(),
                readback_attempted: true,
                readback_result_count: Some(1),
                last_message_path: Some(last_message_path.display().to_string()),
                failure_code: None,
                failure_message: None,
                retryable: false,
                user_action_required: false,
                warnings: vec!["pcr9a_fake_runner_no_real_process_spawned".to_string()],
            }
        }
    }

    struct Pcr9aPanicPhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for Pcr9aPanicPhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            _prompt_body: &str,
            _last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            panic!("PCR9A blocked test must not call Phase B runner");
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Pcr2MemoryFixture {
        Fresh,
        Missing,
        Stale,
    }

    fn pcr2_fixture_state(
        memory_fixture: Pcr2MemoryFixture,
        duplicate_attempt: bool,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        use serde_json::json;
        use std::fs;
        use std::sync::atomic::{AtomicU64, Ordering};

        static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "pcr2-product-command-{}-{}",
            crate::unix_timestamp_nanos(),
            FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("workflow-state.v0.json");
        let stale_memory = memory_fixture == Pcr2MemoryFixture::Stale;
        let snapshot = json!({
            "snapshot_id": "snapshot-1",
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": "packet-1",
            "project_id": "project:h5",
            "workflow_id": "workflow:h5",
            "work_item_id": "work-item:h5",
            "task_package_artifact_id": "artifact:h5",
            "role_id": "codex_worker",
            "retrieval_intent": "worker_task",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": "memory-fingerprint-1",
            "generated_at": "2026-06-08T00:00:00Z",
            "stale": stale_memory,
            "stale_reasons": if stale_memory { vec!["fixture_stale"] } else { Vec::<&str>::new() },
            "warnings": []
        });
        let mut artifacts = Vec::new();
        if memory_fixture != Pcr2MemoryFixture::Missing {
            artifacts.push(json!({
                "artifact_id": "artifact:h5",
                "artifact_type": "task_package",
                "source_ref": "work-item:h5",
                "allowed_write": [dir.display().to_string()],
                "memory_packet_snapshot": snapshot
            }));
        }
        let value = json!({
            "revision": 7,
            "workflow_node_dispatches": [{
                "dispatch_id": "dispatch:h5",
                "project_id": "project:h5",
                "workflow_id": "workflow:h5",
                "node_id": "node:h5",
                "work_item_id": "work-item:h5",
                "native_thread_id": "session:h5",
                "prompt_preview": "redacted prompt preview",
                "prompt_kind": "authorized_prepared_auto_dispatch",
                "memory_packet_snapshot_id": "snapshot-1",
                "memory_packet_fingerprint": "memory-fingerprint-1",
                "plan_authorization_id": "authorization:h5",
                "authorization_check": {"status": "authorized"},
                "task_package_id": "artifact:h5",
                "state": "prepared"
            }],
            "artifacts": artifacts
        });
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        if duplicate_attempt {
            let continuation = json!({
                "record_version": 1,
                "continuation_id": "continuation:h5",
                "preview_id": "preview:h5",
                "adapter_id": "codex-local",
                "operation_id": "resume",
                "project_id": "project:h5",
                "project_root": dir.display().to_string(),
                "workflow_id": "workflow:h5",
                "node_id": "node:h5",
                "session_id": "session:h5",
                "work_item_id": "work-item:h5",
                "target_cwd": dir.display().to_string(),
                "allowed_write_roots": [dir.display().to_string()],
                "sandbox": "workspace-write",
                "prompt_source_kind": "task_package_prompt_ref",
                "prompt_summary": "summary",
                "command_preview": "redacted",
                "readback_strategy": "required",
                "status": "queued",
                "execution_level": "h5_fixture",
                "runner_kind": "fake",
                "user_confirmation_state": "confirmed",
                "guard_status": "allowed",
                "requested_by": "project_director",
                "confirmed_by": "user",
                "confirmation_reason": "fixture",
                "created_at": "2026-06-08T00:00:00Z",
                "updated_at": "2026-06-08T00:00:00Z",
                "audit_refs": [],
                "warnings": []
            });
            let attempt = json!({
                "attempt_version": 1,
                "attempt_id": "attempt:h5",
                "continuation_id": "continuation:h5",
                "runner_kind": "fake",
                "execution_level": "h5_fixture",
                "status": "running",
                "started_at": "2026-06-08T00:00:00Z",
                "finished_at": null,
                "timeout_ms": null,
                "command_preview": "redacted",
                "prompt_sent": false,
                "real_codex_executed": false,
                "writes_codex_home": false,
                "writes_workbench_state": true,
                "readback_summary": {
                    "status": "not_attempted",
                    "source_kind": "fixture",
                    "result_count": null,
                    "unavailable_reason": "running",
                    "warnings": []
                },
                "failure_reason": null,
                "audit_refs": [],
                "warnings": []
            });
            let store = json!({
                "schema_version": "session_continuation_store.v1",
                "store_version": 1,
                "storage_kind": "sidecar_json_v0",
                "scope": {
                    "scope_kind": "workflow_state_sidecar",
                    "workflow_state_path": path.display().to_string(),
                    "sidecar_path": dir.join("session-continuations.v1.json").display().to_string(),
                    "project_roots": [dir.display().to_string()]
                },
                "revision": 1,
                "last_write_id": null,
                "generated_by": "fixture",
                "created_at": "2026-06-08T00:00:00Z",
                "updated_at": "2026-06-08T00:00:00Z",
                "continuations": [continuation],
                "attempts": [attempt],
                "audit_events": [],
                "warnings": []
            });
            fs::write(
                dir.join("session-continuations.v1.json"),
                serde_json::to_string_pretty(&store).unwrap(),
            )
            .unwrap();
        }

        (dir, path)
    }

    fn pcr2_preview_request(
        project_root: String,
        diagnostic_summary: Option<crate::H5DiagnosticSummaryInput>,
    ) -> crate::H5ProjectWorkflowDispatchPreviewInput {
        crate::H5ProjectWorkflowDispatchPreviewInput {
            project_root: project_root.clone(),
            project_id: "project:h5".to_string(),
            workflow_id: "workflow:h5".to_string(),
            dispatch_id: "dispatch:h5".to_string(),
            actor_id: "project_director".to_string(),
            operation_id: Some("resume".to_string()),
            session_id: Some("session:h5".to_string()),
            target_cwd: Some(project_root),
            sandbox: Some("workspace-write".to_string()),
            prompt_summary: "H5 Level A safe preview".to_string(),
            prompt_ref: "workbench-managed:h5-preview:v1".to_string(),
            prompt_sha256: sha256_hex(PCR9A_FAKE_PROMPT_BODY),
            h3_b_level_b_authorized: false,
            expected_workflow_revision: Some(7),
            diagnostic_summary,
        }
    }

    fn sample_product_command_request() -> RealExecutionProductCommandRequest {
        RealExecutionProductCommandRequest {
            product_command_id: "product-command:pcr1".to_string(),
            command_family: "controlled_session_continuation".to_string(),
            operation_id: "resume".to_string(),
            project_id: Some("project:pcr1".to_string()),
            project_root: Some("/tmp/pcr1-project".to_string()),
            workflow_id: Some("workflow:pcr1".to_string()),
            node_id: Some("node:pcr1".to_string()),
            work_item_id: Some("work:pcr1".to_string()),
            task_package_ref: Some("task-package:pcr1".to_string()),
            memory_packet_ref: Some("memory-packet:pcr1".to_string()),
            adapter_id: "codex-local".to_string(),
            session_mode: "resume_existing".to_string(),
            target_session_id: Some("thread:pcr1".to_string()),
            sandbox: "workspace-write".to_string(),
            prompt_summary: "PCR1 fixture prompt summary only".to_string(),
            prompt_ref: "prompt-ref:pcr1".to_string(),
            prompt_hash: "sha256:pcr1".to_string(),
            allowed_write_roots: vec!["/tmp/pcr1-project".to_string()],
            denied_paths: vec![
                "/Users/yoyi/.codex".to_string(),
                "secret/token/.env/keychain/OAuth/provider credential".to_string(),
            ],
            readback_plan: "required_without_full_transcript".to_string(),
            timeout_ms: Some(1_000),
            requested_by: "pcr1-test".to_string(),
            created_at: "2026-06-09T00:00:00Z".to_string(),
        }
    }

    fn sample_product_command_decision(
        request: &RealExecutionProductCommandRequest,
    ) -> RealExecutionProductCommandDecision {
        RealExecutionProductCommandDecision {
            decision_id: "decision:pcr1".to_string(),
            product_command_id: request.product_command_id.clone(),
            decision: "approved".to_string(),
            confirmed_by: "user".to_string(),
            confirmed_at: "2026-06-09T00:00:01Z".to_string(),
            store_revision: 1,
            risk_acknowledgement: "I understand this would be high-impact Level B.".to_string(),
            allowed_once: true,
            reason: "test".to_string(),
        }
    }

    fn sample_pcr7_preview(
        request: &RealExecutionProductCommandRequest,
        preview_id: &str,
    ) -> RealExecutionProductCommandPreview {
        let mut preview = pcr1_contract_preview(request.clone(), preview_id);
        preview.blocked_reasons.clear();
        preview.readiness.status = "ready_for_pcr7_read_model_fixture".to_string();
        preview.readiness.blocked_reasons.clear();
        preview.guard_preview.status = "ready_for_pcr7_read_model_fixture".to_string();
        preview.guard_preview.blocks_execution = false;
        preview.guard_preview.reasons.clear();
        preview.diagnostics_summary.status = "ready_for_pcr7_read_model_fixture".to_string();
        preview.diagnostics_summary.blocks_real_execution = false;
        preview.diagnostics_summary.degraded_reasons.clear();
        preview.duplicate_scope.duplicate_blocked = false;
        preview
    }
}
