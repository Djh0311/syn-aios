use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct K3B1RecoveryReadModel {
    pub(crate) schema_version: String,
    pub(crate) execution_point_id: String,
    pub(crate) current_state: String,
    pub(crate) k3_b2_gate: K3B2GateStatus,
    pub(crate) recovery_options: Vec<K3B1RecoveryOption>,
    pub(crate) manual_exact_command: ManualExactCommandContract,
    pub(crate) manual_submission_contract: ManualRecoverySubmissionContract,
    pub(crate) renewed_risk_approval: RenewedRiskApprovalContract,
    pub(crate) narrow_bridge_design: NarrowBridgeDesignContract,
    pub(crate) runtime_boundary: RecoveryRuntimeBoundary,
    pub(crate) audit_boundary: RecoveryAuditBoundary,
    pub(crate) readback_boundary: RecoveryReadbackBoundary,
    pub(crate) memory_capture_boundary: RecoveryMemoryCaptureBoundary,
    pub(crate) user_summary: Vec<String>,
    pub(crate) developer_details: Vec<RecoveryDeveloperDetail>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct K3B2GateStatus {
    pub(crate) blocked: bool,
    pub(crate) status: String,
    pub(crate) reason: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct K3B1RecoveryOption {
    pub(crate) option_id: String,
    pub(crate) label: String,
    pub(crate) status_after_selection: String,
    pub(crate) user_visible_description: String,
    pub(crate) does_execute_codex: bool,
    pub(crate) requires_separate_task_package: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualExactCommandContract {
    pub(crate) source_task_ref: String,
    pub(crate) source_evidence_ref: String,
    pub(crate) working_directory: String,
    pub(crate) command_lines: Vec<String>,
    pub(crate) prompt_ref: String,
    pub(crate) prompt_hash: String,
    pub(crate) prompt_path_ref: String,
    pub(crate) prompt_body_included: bool,
    pub(crate) user_execution_required: bool,
    pub(crate) workbench_executes_in_l1: bool,
    pub(crate) runs_real_codex_if_user_executes: bool,
    pub(crate) writes_codex_home_if_user_executes: bool,
    pub(crate) boundary: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRecoverySubmissionContract {
    pub(crate) status_after_submit: String,
    pub(crate) auto_accepts_success: bool,
    pub(crate) required_fields: Vec<String>,
    pub(crate) sensitive_material_policy: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenewedRiskApprovalContract {
    pub(crate) status_after_request: String,
    pub(crate) inherited_authorization_allowed: bool,
    pub(crate) warning: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct NarrowBridgeDesignContract {
    pub(crate) status_after_selection: String,
    pub(crate) implementation_allowed_in_l1: bool,
    pub(crate) minimum_future_fields: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecoveryRuntimeBoundary {
    pub(crate) records_blocked_state: bool,
    pub(crate) records_recovery_choice: bool,
    pub(crate) stores_prompt_body: bool,
    pub(crate) stores_codex_home_content: bool,
    pub(crate) allowed_summary: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecoveryAuditBoundary {
    pub(crate) event_type: String,
    pub(crate) records_actor: bool,
    pub(crate) records_choice: bool,
    pub(crate) records_supervisor_review: bool,
    pub(crate) stores_sensitive_material: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecoveryReadbackBoundary {
    pub(crate) status: String,
    pub(crate) result_count: Option<i64>,
    pub(crate) unavailable_reason: String,
    pub(crate) user_submitted_evidence_only: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecoveryMemoryCaptureBoundary {
    pub(crate) capture_event_allowed: bool,
    pub(crate) observation_allowed: bool,
    pub(crate) candidate_allowed: bool,
    pub(crate) formal_memory_auto_write: bool,
    pub(crate) suggested_candidate_text: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecoveryDeveloperDetail {
    pub(crate) label: String,
    pub(crate) value: String,
}

pub(crate) const K3_B1_RECOVERY_STATES: [&str; 9] = [
    "blocked_by_safety_review_again",
    "manual_recovery_available",
    "manual_recovery_submitted",
    "manual_recovery_needs_review",
    "manual_recovery_rejected",
    "manual_recovery_accepted",
    "pending_renewed_risk_approval",
    "renewed_execution_request_rejected",
    "narrow_bridge_design_required",
];
pub(crate) const WARNING_COUNT: usize = 4;

pub(crate) fn derive_k3_b1_recovery_read_model() -> K3B1RecoveryReadModel {
    K3B1RecoveryReadModel {
        schema_version: "k3_b1_recovery_read_model.v1".to_string(),
        execution_point_id: "stage-k-k3-b1-mario-test-workflow-read-only".to_string(),
        current_state: "blocked_by_safety_review_again".to_string(),
        k3_b2_gate: K3B2GateStatus {
            blocked: true,
            status: "blocked_waiting_k3_b1_recovery_acceptance".to_string(),
            reason: "K3-B1 仍未被主管线接受为成功或等价恢复；K3-B2 继续阻断。".to_string(),
        },
        recovery_options: vec![
            K3B1RecoveryOption {
                option_id: "manual_exact_command_submission".to_string(),
                label: "用户手动运行 exact command 并回交".to_string(),
                status_after_selection: "manual_recovery_available".to_string(),
                user_visible_description: "用户在自己明确控制的终端环境中运行冻结命令，再回交脱敏摘要、退出码、运行目录、引用和 hash；主管线复核前不改变成功状态。".to_string(),
                does_execute_codex: false,
                requires_separate_task_package: false,
            },
            K3B1RecoveryOption {
                option_id: "renewed_risk_approval_request".to_string(),
                label: "重新明确批准风险后另行申请真实执行".to_string(),
                status_after_selection: "pending_renewed_risk_approval".to_string(),
                user_visible_description: "该路径会向外部服务发送项目/session 派生 prompt，并写入 Codex 本地状态；L1 只进入待重新授权/待安全审查。".to_string(),
                does_execute_codex: false,
                requires_separate_task_package: true,
            },
            K3B1RecoveryOption {
                option_id: "narrow_local_bridge_design".to_string(),
                label: "设计更窄的本地执行桥".to_string(),
                status_after_selection: "narrow_bridge_design_required".to_string(),
                user_visible_description: "只作为后续设计候选，用于降低外发和本地状态写入范围；不能绕过安全审查。".to_string(),
                does_execute_codex: false,
                requires_separate_task_package: true,
            },
        ],
        manual_exact_command: ManualExactCommandContract {
            source_task_ref: "tasks/2026-06-10-stage-k-k3-b1-mario-test-workflow-read-only-real-resume-run-v1.md#6".to_string(),
            source_evidence_ref: "evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md#申请执行的冻结命令".to_string(),
            working_directory: "/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri".to_string(),
            command_lines: vec![
                "K3_B1_REAL_EXECUTION_AUTHORIZED=stage-k-k3-b1-mario-test-workflow-read-only".to_string(),
                "K3_B1_PROJECT_ROOT=/Users/yoyi/Documents/mario test".to_string(),
                "K3_B1_SESSION_ID=019e798a-ac37-7771-b982-e38084fcd22e".to_string(),
                "K3_B1_EXPECTED_MARKER=K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10".to_string(),
                "K3_B1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs".to_string(),
                "K3_B1_PROMPT_PATH=/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt".to_string(),
                "cargo test --lib project_workflow_automation::tests::k3_b1_real_mario_test_workflow_resume_requires_env_authorization -- --ignored --exact --nocapture".to_string(),
            ],
            prompt_ref: "prompt:stage-k:k3:b1:mario-test-workflow-read-only".to_string(),
            prompt_hash: "ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039".to_string(),
            prompt_path_ref: "tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt".to_string(),
            prompt_body_included: false,
            user_execution_required: true,
            workbench_executes_in_l1: false,
            runs_real_codex_if_user_executes: true,
            writes_codex_home_if_user_executes: true,
            boundary: "L1 只展示冻结 exact command，供用户在自己明确控制的终端环境手动执行并回交；工作台不执行、不发送 prompt、不读取或写入 .codex。".to_string(),
        },
        manual_submission_contract: ManualRecoverySubmissionContract {
            status_after_submit: "manual_recovery_needs_review".to_string(),
            auto_accepts_success: false,
            required_fields: vec![
                "stdout_summary",
                "stderr_summary",
                "exit_code",
                "run_dir",
                "last_message",
                "sidecar_refs",
                "runtime_log_refs",
                "audit_refs",
                "readback_status",
                "result_count",
                "project_file_hashes_before_after",
                "user_statement",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            sensitive_material_policy: "如回交材料包含 secret/token/完整 transcript/.codex 原始内容，必须阻断或先脱敏；exit_code=0 仍不自动等于成功。".to_string(),
        },
        renewed_risk_approval: RenewedRiskApprovalContract {
            status_after_request: "pending_renewed_risk_approval".to_string(),
            inherited_authorization_allowed: false,
            warning: "重新申请必须让用户明确看到 prompt 外发和 /Users/yoyi/.codex 写入风险；L1 不继承旧授权、不执行。".to_string(),
        },
        narrow_bridge_design: NarrowBridgeDesignContract {
            status_after_selection: "narrow_bridge_design_required".to_string(),
            implementation_allowed_in_l1: false,
            minimum_future_fields: vec![
                "allowed_roots",
                "denied_paths",
                "prompt_hash",
                "readback",
                "audit",
                "rollback",
                "user_confirmation",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        },
        runtime_boundary: RecoveryRuntimeBoundary {
            records_blocked_state: true,
            records_recovery_choice: true,
            stores_prompt_body: false,
            stores_codex_home_content: false,
            allowed_summary: "只记录 blocked 状态、恢复选择、是否等待复核和引用；不记录 prompt body、secret、完整 transcript 或 .codex 原文。".to_string(),
        },
        audit_boundary: RecoveryAuditBoundary {
            event_type: "k3_b1_recovery_decision_recorded".to_string(),
            records_actor: true,
            records_choice: true,
            records_supervisor_review: true,
            stores_sensitive_material: false,
        },
        readback_boundary: RecoveryReadbackBoundary {
            status: "not_attempted_l1_recovery_path_only".to_string(),
            result_count: None,
            unavailable_reason: "L1 未执行真实 Codex；读回结果未知/不可用，不能显示为 0 条。".to_string(),
            user_submitted_evidence_only: true,
        },
        memory_capture_boundary: RecoveryMemoryCaptureBoundary {
            capture_event_allowed: true,
            observation_allowed: true,
            candidate_allowed: true,
            formal_memory_auto_write: false,
            suggested_candidate_text: "K3-B1 retry 曾因真实 Codex resume 的外发和 .codex 写入风险被安全审查再次阻断；后续恢复需要用户手动回交、重新授权申请或更窄本地执行桥。".to_string(),
        },
        user_summary: vec![
            "K3-B1 被安全审查再次阻断。".to_string(),
            "阻断原因：真实 Codex resume 会向外部服务发送项目/session 派生 prompt，并写入 Codex 本地状态。".to_string(),
            "合法恢复：手动运行并回交、重新授权申请、或等待更窄本地执行桥设计。".to_string(),
            "当前不能自动重试，不能进入 K3-B2，也不能把手动回交自动当成功。".to_string(),
        ],
        developer_details: vec![
            detail("execution_point_id", "stage-k-k3-b1-mario-test-workflow-read-only"),
            detail(
                "exact_command_ref",
                "evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md#申请执行的冻结命令",
            ),
            detail(
                "prompt_hash",
                "ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039",
            ),
            detail("target_project_root", "/Users/yoyi/Documents/mario test"),
            detail("target_session_id", "019e798a-ac37-7771-b982-e38084fcd22e"),
            detail(
                "expected_marker",
                "K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10",
            ),
            detail(
                "rejected_safety_reason",
                "真实 Codex resume 会发送项目/session 派生 prompt 到外部服务并写入 Codex 本地状态。",
            ),
        ],
        warnings: vec![
            "k3_b1_still_blocked".to_string(),
            "k3_b2_gate_remains_blocked".to_string(),
            "manual_submission_requires_supervisor_review".to_string(),
            "l1_does_not_execute_codex".to_string(),
        ],
    }
}

pub(crate) fn recovery_state_blocks_k3_b2(state: &str) -> bool {
    state != "manual_recovery_accepted"
}

pub(crate) fn manual_submission_state_after_submit() -> &'static str {
    "manual_recovery_needs_review"
}

pub(crate) fn recovery_material_contains_forbidden_content(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    [
        "secret",
        "token",
        ".env",
        "keychain",
        "oauth",
        "provider credential",
        "full transcript",
        "完整 transcript",
        "rollout",
        "prompt body",
        "/users/yoyi/.codex",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn detail(label: &str, value: &str) -> RecoveryDeveloperDetail {
    RecoveryDeveloperDetail {
        label: label.to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l1_recovery_model_covers_all_states_and_keeps_k3_b2_blocked_until_acceptance() {
        let model = derive_k3_b1_recovery_read_model();

        assert_eq!(K3_B1_RECOVERY_STATES.len(), 9);
        assert_eq!(model.current_state, "blocked_by_safety_review_again");
        assert_eq!(model.warnings.len(), WARNING_COUNT);
        assert!(model.k3_b2_gate.blocked);
        for state in K3_B1_RECOVERY_STATES {
            let blocks = recovery_state_blocks_k3_b2(state);
            if state == "manual_recovery_accepted" {
                assert!(
                    !blocks,
                    "accepted manual recovery is the only L1 state that can prepare L2"
                );
            } else {
                assert!(blocks, "{state} must not unlock K3-B2");
            }
        }
    }

    #[test]
    fn l1_manual_submission_stays_pending_review_with_unknown_readback() {
        let model = derive_k3_b1_recovery_read_model();

        assert_eq!(
            manual_submission_state_after_submit(),
            "manual_recovery_needs_review"
        );
        assert_eq!(
            model.manual_submission_contract.status_after_submit,
            "manual_recovery_needs_review"
        );
        assert!(!model.manual_submission_contract.auto_accepts_success);
        assert_eq!(model.readback_boundary.result_count, None);
        assert_eq!(
            model.readback_boundary.status,
            "not_attempted_l1_recovery_path_only"
        );
    }

    #[test]
    fn l1_manual_exact_command_is_visible_without_prompt_body_or_workbench_execution() {
        let model = derive_k3_b1_recovery_read_model();
        let command = model.manual_exact_command.command_lines.join("\n");

        assert_eq!(
            model.manual_exact_command.working_directory,
            "/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri"
        );
        assert!(command.contains(
            "K3_B1_REAL_EXECUTION_AUTHORIZED=stage-k-k3-b1-mario-test-workflow-read-only"
        ));
        assert!(command.contains(
            "cargo test --lib project_workflow_automation::tests::k3_b1_real_mario_test_workflow_resume_requires_env_authorization -- --ignored --exact --nocapture"
        ));
        assert_eq!(
            model.manual_exact_command.prompt_hash,
            "ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039"
        );
        assert!(!model.manual_exact_command.prompt_body_included);
        assert!(model.manual_exact_command.user_execution_required);
        assert!(!model.manual_exact_command.workbench_executes_in_l1);
        assert!(model.manual_exact_command.runs_real_codex_if_user_executes);
        assert!(
            model
                .manual_exact_command
                .writes_codex_home_if_user_executes
        );
        assert!(!command.contains("prompt body"));
    }

    #[test]
    fn l1_recovery_boundaries_reject_sensitive_material_and_never_store_prompt_or_codex_home() {
        let model = derive_k3_b1_recovery_read_model();

        assert!(recovery_material_contains_forbidden_content(
            "contains auth token and /Users/yoyi/.codex transcript"
        ));
        assert!(!recovery_material_contains_forbidden_content(
            "exit_code=0; last_message marker present; hashes unchanged"
        ));
        assert!(!model.runtime_boundary.stores_prompt_body);
        assert!(!model.runtime_boundary.stores_codex_home_content);
        assert!(!model.audit_boundary.stores_sensitive_material);
        assert!(!model.memory_capture_boundary.formal_memory_auto_write);
    }
}
