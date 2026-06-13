use crate::utils::hash::sha256_hex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    array_mut, backup_workflow_state_file, codex_local_runner, default_workflow_id, find_work_item,
    i64_value, memory_capture_bus, node_exists, optional_string_from, project_id,
    read_workflow_state_value, real_execution_command,
    record_project_director_process_fact_decision_at, record_worker_structured_report_at,
    stable_id, validate_workflow_state, workflow_exists, write_validated_workflow_state,
    CaptureMemoryEventInput, CodexControlCommandInput, MemoryCaptureSourceRef, MemoryScope,
    ObservationSourceRef, PrepareRealExecutionProductCommandInput,
    PreviewRealExecutionProductCommandInput, ProcessFactCandidate,
    ProjectDirectorProcessFactDecisionInput, ProjectWorkflowAutomationInput,
    ProjectWorkflowAutomationJ2BB1Input, ProjectWorkflowAutomationJ2BB1Output,
    ProjectWorkflowAutomationJ2BB2Input, ProjectWorkflowAutomationJ2BB2Output,
    ProjectWorkflowAutomationK3BInput, ProjectWorkflowAutomationK3BOutput,
    ProjectWorkflowAutomationPlan, ProjectWorkflowAutomationReadModel,
    ProjectWorkflowAutomationResult, ProjectWorkflowRunUnit,
    RecordRealExecutionProductCommandDecisionInput,
    RunRealExecutionProductCommandNewSessionPhaseBInput, RunRealExecutionProductCommandPhaseAInput,
    RunRealExecutionProductCommandPhaseBInput, WorkerStructuredReportInput,
};

const J2_SCHEMA: &str = "project_workflow_automation.v1";
const J2_EVENT_TYPE: &str = "project_workflow_automation_phase_a_recorded";
const J2_SOURCE_KIND: &str = "stage_j_j2_project_workflow_automation";
const J2_B_B1_EVENT_TYPE: &str = "project_workflow_automation_j2_b_b1_phase_b_recorded";
const J2_B_B1_PROJECT_ROOT: &str = "/Users/yoyi/Documents/mario test";
const J2_B_B1_PROJECT_ID: &str = "project:users-yoyi-documents-mario-test";
const J2_B_B1_WORKFLOW_ID: &str = "workflow:users-yoyi-documents-mario-test:default";
const J2_B_B1_NODE_ID: &str = "workflow:users-yoyi-documents-mario-test:default:node:codex-dev";
const J2_B_B1_SESSION_ID: &str = "019e798a-ac37-7771-b982-e38084fcd22e";
const J2_B_B1_SANDBOX: &str = "read-only";
const J2_B_B1_PROMPT_SUMMARY: &str =
    "J2-B mario test developer run unit read-only real resume probe.";
const J2_B_B1_PROMPT_REF: &str =
    "workbench-managed:j2-b:mario-test:developer-run-unit:read-only:v1";
const J2_B_B1_PROMPT_HASH: &str =
    "31c8ceb071804168e46a1d5b3d3accbded1539037472479649766d676672caa0";
const J2_B_B1_READBACK_MARKER: &str = "J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09";
const J2_B_B1_CANONICAL_PROMPT: &str = "You are the codex-local developer run unit for Stage J / J2-B project workflow automation read-only closed-loop probe.\n\nScope:\n- Project: /Users/yoyi/Documents/mario test\n- Workflow: workflow:users-yoyi-documents-mario-test:default\n- Run unit: developer_execution\n- Operation: resume only\n- Sandbox: read-only\n- Marker: J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09\n\nRules:\n- Do not modify files.\n- Do not run commands.\n- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.\n- Reply only with the marker and a minimal structured worker report candidate.\n";
const J2_B_B2_EVENT_TYPE: &str = "project_workflow_automation_j2_b_b2_phase_b_recorded";
const J2_B_B2_PROJECT_ROOT: &str =
    "/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project";
const J2_B_B2_PROJECT_ID: &str = "project:stage-j-j2-b-isolated-project";
const J2_B_B2_WORKFLOW_ID: &str = "workflow:stage-j-j2-b-isolated-project:default";
const J2_B_B2_NODE_ID: &str = "workflow:stage-j-j2-b-isolated-project:default:node:codex-dev";
const J2_B_B2_SANDBOX: &str = "workspace-write";
const J2_B_B2_ALLOWED_WRITE_ROOT: &str =
    "/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b";
const J2_B_B2_ALLOWED_WRITE_PATH: &str = "/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md";
const J2_B_B2_PROMPT_SUMMARY: &str =
    "J2-B isolated project developer run unit workspace-write real probe.";
const J2_B_B2_PROMPT_REF: &str =
    "workbench-managed:j2-b:isolated-project:developer-run-unit:workspace-write:v1";
const J2_B_B2_PROMPT_HASH: &str =
    "a1e3eb2285a75b30d0104f5bd032e3b4fdfc51111ff52949597ce78de5878bb0";
const J2_B_B2_READBACK_MARKER: &str =
    "J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09";
const J2_B_B2_CANONICAL_PROMPT: &str = "You are the codex-local developer run unit for Stage J / J2-B project workflow automation workspace-write closed-loop probe.\n\nScope:\n- Project: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project\n- Workflow: workflow:stage-j-j2-b-isolated-project:default\n- Run unit: developer_execution\n- Operation: resume or new session only after task package authorization\n- Sandbox: workspace-write\n- Marker: J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09\n- Allowed write path: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md\n\nRules:\n- Only create or update the allowed write path.\n- Do not modify source, docs, task, evidence, handoff, or other project files.\n- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.\n- Reply only with the marker and a minimal structured worker report candidate listing the allowed file.\n";
const K3_B_B1_EVENT_TYPE: &str = "project_workflow_automation_k3_b1_phase_b_recorded";
const K3_B_B1_EXECUTION_POINT_ID: &str = "stage-k-k3-b1-mario-test-workflow-read-only";
const K3_B_B1_PROJECT_ROOT: &str = "/Users/yoyi/Documents/mario test";
const K3_B_B1_PROJECT_ID: &str = "project:users-yoyi-documents-mario-test";
const K3_B_B1_WORKFLOW_ID: &str = "workflow:users-yoyi-documents-mario-test:default";
const K3_B_B1_NODE_ID: &str = "workflow:users-yoyi-documents-mario-test:default:node:codex-dev";
const K3_B_B1_RUN_UNIT_ID: &str = "run-unit:stage-k:k3:b1:mario-test:developer_execution";
const K3_B_B1_WORK_ITEM_ID: &str = "work-item:stage-k:k3:b1:mario-test:developer-read-only";
const K3_B_B1_SESSION_ID: &str = "019e798a-ac37-7771-b982-e38084fcd22e";
const K3_B_B1_SANDBOX: &str = "read-only";
const K3_B_B1_PROMPT_SUMMARY: &str =
    "Stage K K3-B1 read-only project workflow probe for mario test codex-dev worker.";
const K3_B_B1_PROMPT_REF: &str = "prompt:stage-k:k3:b1:mario-test-workflow-read-only";
const K3_B_B1_PROMPT_HASH: &str =
    "ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039";
const K3_B_B1_TASK_PACKAGE_REF: &str = "task-package:stage-k:k3:b1:mario-test";
const K3_B_B1_MEMORY_PACKET_REF: &str = "memory-packet:stage-k:k3:b1:mario-test";
const K3_B_B1_PERMISSION_ENVELOPE_REF: &str = "permission:stage-k:k3:b1";
const K3_B_B1_READBACK_MARKER: &str = "K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10";
const K3_B_B2_EVENT_TYPE: &str = "project_workflow_automation_k3_b2_phase_b_recorded";
const K3_B_B2_EXECUTION_POINT_ID: &str = "stage-k-k3-b2-isolated-workflow-workspace-write";
const K3_B_B2_PROJECT_ROOT: &str =
    "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project";
const K3_B_B2_PROJECT_ID: &str =
    "project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project";
const K3_B_B2_WORKFLOW_ID: &str = "workflow:stage-k:k3:isolated";
const K3_B_B2_NODE_ID: &str = "node:stage-k:k3:b2:isolated:developer_execution";
const K3_B_B2_RUN_UNIT_ID: &str = "run-unit:stage-k:k3:b2:isolated:developer_execution";
const K3_B_B2_WORK_ITEM_ID: &str = "work-item:stage-k:k3:b2:isolated:developer-write";
const K3_B_B2_SANDBOX: &str = "workspace-write";
const K3_B_B2_ALLOWED_WRITE_ROOT: &str =
    "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/";
const K3_B_B2_ALLOWED_WRITE_PATH: &str = "/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k3/k3-b2-workspace-write-probe.md";
const K3_B_B2_PROMPT_SUMMARY: &str =
    "Stage K K3-B2 workspace-write isolated project workflow probe for codex-local worker.";
const K3_B_B2_PROMPT_REF: &str = "prompt:stage-k:k3:b2:isolated-workflow-write";
const K3_B_B2_PROMPT_HASH: &str =
    "9057c04f1bbd9ef5ff28b55e4f041fdc1a924c8ba5eeae18c564079ea80226c3";
const K3_B_B2_TASK_PACKAGE_REF: &str = "task-package:stage-k:k3:b2:isolated";
const K3_B_B2_MEMORY_PACKET_REF: &str = "memory-packet:stage-k:k3:b2:isolated";
const K3_B_B2_PERMISSION_ENVELOPE_REF: &str = "permission:stage-k:k3:b2";
const K3_B_B2_READBACK_MARKER: &str = "K3_B2_ISOLATED_WORKFLOW_WRITE_OK_2026_06_10";

pub(crate) fn run_project_workflow_automation_phase_a_at(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationInput,
    timestamp: &str,
    write_id: &str,
) -> Result<ProjectWorkflowAutomationResult, String> {
    validate_input(input)?;
    let workflow_id = input
        .workflow_id
        .clone()
        .unwrap_or_else(|| default_workflow_id(&input.project_root));
    let project_id_value = input
        .project_id
        .clone()
        .unwrap_or_else(|| project_id(&input.project_root));
    let mut value = read_workflow_state_value(workflow_state_path)?;
    if let Some(expected) = input.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, &workflow_id) {
        let mut plan = build_plan(
            input,
            &project_id_value,
            &workflow_id,
            None,
            None,
            timestamp,
        );
        for unit in &mut plan.run_units {
            unit.status = "blocked_by_guard".to_string();
            unit.blocked_reasons
                .push("workflow_missing_for_project_automation".to_string());
        }
        plan.current_phase = "blocked".to_string();
        plan.next_step = "先创建项目默认工作流，再生成自动编排计划。".to_string();
        plan.blocked_reasons
            .push("workflow_missing_for_project_automation".to_string());
        let read_model = read_model_from_plan(
            timestamp,
            Some("blocked".to_string()),
            Some(plan.clone()),
            vec!["k3_level_a_blocked_without_writing_sidecars".to_string()],
        );
        return Ok(ProjectWorkflowAutomationResult {
            status: "blocked".to_string(),
            plan,
            phase_a_output: None,
            worker_report_result: None,
            process_fact_result: None,
            read_model,
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_project_files: false,
            blocked_reasons: vec!["workflow_missing_for_project_automation".to_string()],
            warnings: vec!["k3_level_a_requires_existing_workflow_state".to_string()],
        });
    }
    let automation_id = automation_id_for(&workflow_id, &input.user_goal);
    let existing_read_model =
        load_project_workflow_automation_read_model(workflow_state_path, timestamp);
    if existing_read_model.latest_automation_id.as_deref() == Some(automation_id.as_str())
        && existing_read_model.latest_status.as_deref() == Some("phase_a_closed_loop_recorded")
    {
        let mut plan = existing_read_model.latest_plan.clone().unwrap_or_else(|| {
            build_plan(
                input,
                &project_id_value,
                &workflow_id,
                input.work_item_id.clone(),
                input.workflow_node_id.clone(),
                timestamp,
            )
        });
        plan.current_phase = "blocked".to_string();
        plan.next_step =
            "已有项目自动编排 Level A 闭环记录；如需再次执行，请生成新的用户目标或进入后续真实执行授权。"
                .to_string();
        plan.blocked_reasons
            .push("duplicate_project_workflow_automation_closed_loop".to_string());
        for unit in &mut plan.run_units {
            unit.status = "blocked_by_guard".to_string();
            unit.blocked_reasons
                .push("duplicate_project_workflow_automation_closed_loop".to_string());
        }
        let read_model = read_model_from_plan(
            timestamp,
            Some("blocked".to_string()),
            Some(plan.clone()),
            vec!["k3_level_a_duplicate_closed_loop_no_write".to_string()],
        );
        return Ok(ProjectWorkflowAutomationResult {
            status: "blocked".to_string(),
            plan,
            phase_a_output: None,
            worker_report_result: None,
            process_fact_result: None,
            read_model,
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_project_files: false,
            blocked_reasons: vec!["duplicate_project_workflow_automation_closed_loop".to_string()],
            warnings: vec!["k3_level_a_duplicate_closed_loop_no_write".to_string()],
        });
    }

    let work_item_id = ensure_work_item(
        workflow_state_path,
        &mut value,
        input,
        &project_id_value,
        &workflow_id,
        timestamp,
    )?;
    let developer_node_id = input
        .workflow_node_id
        .clone()
        .unwrap_or_else(|| format!("{workflow_id}:node:codex-dev"));
    if !node_exists(&value, &workflow_id, &developer_node_id) {
        return Err(format!(
            "K3 Level A 找不到开发线节点，无法绑定 run unit：{developer_node_id}"
        ));
    }
    if find_work_item(&value, &workflow_id, &work_item_id).is_none() {
        return Err(format!(
            "K3 Level A 找不到工作项，无法绑定 run unit：{work_item_id}"
        ));
    }

    let mut plan = build_plan(
        input,
        &project_id_value,
        &workflow_id,
        Some(work_item_id.clone()),
        Some(developer_node_id.clone()),
        timestamp,
    );
    let mut developer_product_command_id = None;
    for unit in &mut plan.run_units {
        let preview_input = PreviewRealExecutionProductCommandInput {
            source_kind: "codex_control".to_string(),
            h5_dispatch_preview: None,
            codex_control: Some(codex_control_for_unit(input, unit)),
            requested_by: input.requested_by.clone(),
            created_at: Some(timestamp.to_string()),
        };
        let preview = real_execution_command::preview_real_execution_product_command_at(
            workflow_state_path,
            &preview_input,
        )?;
        unit.product_command_preview_ref = Some(preview.request.product_command_id.clone());
        unit.blocked_reasons.extend(preview.blocked_reasons.clone());
        unit.warnings.extend(preview.warnings.clone());
        if !preview.blocked_reasons.is_empty() {
            unit.status = "blocked_by_guard".to_string();
            continue;
        }
        if unit.run_unit_kind == "developer_execution" {
            let prepare = real_execution_command::prepare_real_execution_product_command_at(
                workflow_state_path,
                &PrepareRealExecutionProductCommandInput {
                    source_kind: "codex_control".to_string(),
                    h5_dispatch_preview: None,
                    codex_control: preview_input.codex_control.clone(),
                    expected_store_revision: input.expected_product_command_store_revision,
                    requested_by: input.requested_by.clone(),
                    created_at: Some(timestamp.to_string()),
                },
            )?;
            unit.product_command_ref = prepare.product_command_id.clone();
            unit.warnings.extend(prepare.warnings.clone());
            if prepare.status != "prepared" {
                unit.status = if prepare.status == "store_conflict" {
                    "blocked_by_guard".to_string()
                } else {
                    "waiting_user".to_string()
                };
                unit.blocked_reasons.extend(prepare.blocked_reasons.clone());
                continue;
            }
            developer_product_command_id = prepare.product_command_id.clone();
            unit.status = "waiting_user".to_string();
        } else {
            unit.status = "planned".to_string();
        }
    }

    let Some(product_command_id) = developer_product_command_id.clone() else {
        let warnings = vec!["k3_level_a_developer_product_command_not_prepared".to_string()];
        plan.current_phase = "blocked".to_string();
        plan.next_step = "检查开发线 run unit 的准备态阻断原因。".to_string();
        let read_model = read_model_from_plan(
            timestamp,
            Some("blocked".to_string()),
            Some(plan.clone()),
            warnings.clone(),
        );
        return Ok(ProjectWorkflowAutomationResult {
            status: "blocked".to_string(),
            plan,
            phase_a_output: None,
            worker_report_result: None,
            process_fact_result: None,
            read_model,
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_project_files: false,
            blocked_reasons: vec!["developer_product_command_not_prepared".to_string()],
            warnings,
        });
    };

    let decision = real_execution_command::record_real_execution_product_command_decision_at(
        workflow_state_path,
        &RecordRealExecutionProductCommandDecisionInput {
            product_command_id: product_command_id.clone(),
            decision: "approved".to_string(),
            expected_store_revision: None,
            confirmed_by: input
                .confirmed_by
                .clone()
                .unwrap_or_else(|| "user".to_string()),
            risk_acknowledgement: input.risk_acknowledgement.clone().unwrap_or_else(|| {
                "K3 Level A 只记录 Phase A no-op；不发送 prompt、不执行真实 Codex。".to_string()
            }),
            allowed_once: true,
            reason: input
                .reason
                .clone()
                .unwrap_or_else(|| "用户目标进入项目自动编排 Level A 非真实闭环验证。".to_string()),
            requested_by: input.requested_by.clone(),
            confirmed_at: Some(timestamp.to_string()),
        },
    )?;
    if decision.status != "decision_recorded" {
        return Err(format!(
            "K3 Level A 用户确认未写入，无法进入 Phase A：{}",
            decision.status
        ));
    }

    let phase_a = real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &RunRealExecutionProductCommandPhaseAInput {
            product_command_id: product_command_id.clone(),
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: input
                .expected_session_continuation_store_revision,
            actor_role: "developer_execution".to_string(),
            execution_decision: Some("approved_for_phase_a".to_string()),
            timeout_ms: Some(30_000),
            requested_at: Some(timestamp.to_string()),
        },
        timestamp,
        &format!("{write_id}-product-command-phase-a"),
    )?;
    if phase_a.status != "phase_a_completed" {
        return Err(format!(
            "K3 Level A Phase A 未完成，无法生成 worker report fixture：{}",
            phase_a.status
        ));
    }

    apply_phase_a_to_developer_unit(&mut plan, &phase_a);
    mark_downstream_units(&mut plan, &phase_a);
    let worker_report_input = worker_report_input(
        input,
        &project_id_value,
        &workflow_id,
        &developer_node_id,
        &work_item_id,
        &product_command_id,
        &phase_a,
        timestamp,
    );
    let worker_report =
        record_worker_structured_report_at(workflow_state_path, &worker_report_input)?;
    apply_worker_report_to_plan(&mut plan, &worker_report.audit_event_id);
    let process_fact_input = process_fact_input(
        input,
        &project_id_value,
        &workflow_id,
        &worker_report.audit_event_id,
        &product_command_id,
        &phase_a,
        timestamp,
    );
    let process_fact =
        record_project_director_process_fact_decision_at(workflow_state_path, &process_fact_input)?;
    apply_process_fact_to_plan(&mut plan, &process_fact);
    let capture_result = capture_process_fact_event(
        workflow_state_path,
        input,
        &plan,
        &product_command_id,
        &phase_a,
        &worker_report.audit_event_id,
        &process_fact,
        timestamp,
        write_id,
    )?;
    apply_capture_event_to_plan(&mut plan, &capture_result);
    plan.current_phase = "collector_summary".to_string();
    plan.next_step =
        "等待主管复核 K3 Level A evidence / handoff；真实执行点留到 K3 Level B 单独授权。"
            .to_string();

    let audit_event_id = append_automation_audit_event(
        workflow_state_path,
        input,
        &plan,
        &phase_a,
        &worker_report.audit_event_id,
        &process_fact,
        timestamp,
    )?;
    if let Some(unit) = plan
        .run_units
        .iter_mut()
        .find(|unit| unit.run_unit_kind == "collector_summary")
    {
        unit.audit_refs.push(audit_event_id.clone());
    }
    let read_model = load_project_workflow_automation_read_model(workflow_state_path, timestamp);
    Ok(ProjectWorkflowAutomationResult {
        status: "phase_a_closed_loop_recorded".to_string(),
        plan,
        phase_a_output: Some(phase_a),
        worker_report_result: Some(worker_report),
        process_fact_result: Some(process_fact),
        read_model,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        blocked_reasons: Vec::new(),
        warnings: crate::dedupe_strings(
            [
                vec![
                    "k3_level_a_no_real_codex_execution".to_string(),
                    "k3_level_a_observation_is_not_formal_memory".to_string(),
                ],
                capture_result.warnings,
            ]
            .concat(),
        ),
    })
}

pub(crate) fn run_project_workflow_automation_j2_b_b1_at(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationJ2BB1Input,
    timestamp: &str,
    write_id: &str,
) -> Result<ProjectWorkflowAutomationJ2BB1Output, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path = workflow_state_path
        .parent()
        .ok_or_else(|| "J2-B B1 workflow_state_path 缺少 parent".to_string())?
        .join(format!(
            "j2-b-b1-last-message-{}.json",
            stable_id(timestamp)
        ));
    run_project_workflow_automation_j2_b_b1_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_project_workflow_automation_j2_b_b1_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationJ2BB1Input,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<ProjectWorkflowAutomationJ2BB1Output, String> {
    validate_j2_b_b1_input(input)?;
    let mut value = read_workflow_state_value(workflow_state_path)?;
    if let Some(expected) = input.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "J2-B B1 workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, J2_B_B1_WORKFLOW_ID) {
        return Err("J2-B B1 找不到冻结 workflow，拒绝准备真实执行点".to_string());
    }
    if !node_exists(&value, J2_B_B1_WORKFLOW_ID, J2_B_B1_NODE_ID) {
        return Err("J2-B B1 找不到冻结 codex-dev node，拒绝准备真实执行点".to_string());
    }

    let automation_input = j2_b_b1_automation_input(input);
    let work_item_id = ensure_work_item(
        workflow_state_path,
        &mut value,
        &automation_input,
        J2_B_B1_PROJECT_ID,
        J2_B_B1_WORKFLOW_ID,
        timestamp,
    )?;
    if find_work_item(&value, J2_B_B1_WORKFLOW_ID, &work_item_id).is_none() {
        return Err(format!(
            "J2-B B1 找不到工作项，无法绑定 run unit：{work_item_id}"
        ));
    }

    let mut plan = build_plan(
        &automation_input,
        J2_B_B1_PROJECT_ID,
        J2_B_B1_WORKFLOW_ID,
        Some(work_item_id.clone()),
        Some(J2_B_B1_NODE_ID.to_string()),
        timestamp,
    );
    plan.current_phase = "developer_execution".to_string();
    plan.next_step = "J2-B B1 开发线通过统一 Product Command Phase B 受控执行。".to_string();
    plan.warnings = crate::dedupe_strings(vec![
        "j2_b_b1_read_only_real_resume_execution_point".to_string(),
        "prompt_body_runtime_only_not_persisted".to_string(),
        "allowed_write_roots_empty_for_read_only".to_string(),
    ]);
    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution")
        .cloned()
        .ok_or_else(|| "J2-B B1 developer run unit missing".to_string())?;
    let codex_control = j2_b_b1_codex_control(&developer_unit);
    let preview_input = PreviewRealExecutionProductCommandInput {
        source_kind: "codex_control".to_string(),
        h5_dispatch_preview: None,
        codex_control: Some(codex_control.clone()),
        requested_by: input
            .requested_by
            .clone()
            .or_else(|| Some("user".to_string())),
        created_at: Some(timestamp.to_string()),
    };
    let preview = real_execution_command::preview_real_execution_product_command_at(
        workflow_state_path,
        &preview_input,
    )?;
    if !preview.blocked_reasons.is_empty() {
        return Err(format!(
            "J2-B B1 preview 被 guard 阻断：{}",
            preview.blocked_reasons.join(", ")
        ));
    }
    let prepare = real_execution_command::prepare_real_execution_product_command_at(
        workflow_state_path,
        &PrepareRealExecutionProductCommandInput {
            source_kind: "codex_control".to_string(),
            h5_dispatch_preview: None,
            codex_control: Some(codex_control.clone()),
            expected_store_revision: input.expected_product_command_store_revision,
            requested_by: input
                .requested_by
                .clone()
                .or_else(|| Some("user".to_string())),
            created_at: Some(timestamp.to_string()),
        },
    )?;
    if prepare.status != "prepared" {
        return Err(format!(
            "J2-B B1 prepare 未进入 prepared：{}",
            prepare.status
        ));
    }
    let product_command_id = prepare
        .product_command_id
        .clone()
        .ok_or_else(|| "J2-B B1 prepare 缺少 product_command_id".to_string())?;
    let decision = real_execution_command::record_real_execution_product_command_decision_at(
        workflow_state_path,
        &RecordRealExecutionProductCommandDecisionInput {
            product_command_id: product_command_id.clone(),
            decision: "approved".to_string(),
            expected_store_revision: Some(prepare.store_revision),
            confirmed_by: "user".to_string(),
            risk_acknowledgement: input.risk_acknowledgement.clone().unwrap_or_else(|| {
                "确认 J2-B B1 read-only resume 只能通过统一 Product Command Phase B，prompt body 不持久化。"
                    .to_string()
            }),
            allowed_once: true,
            reason: input.reason.clone().unwrap_or_else(|| {
                "J2-B B1 冻结 mario test 开发线 read-only run unit 真实执行点。".to_string()
            }),
            requested_by: input.requested_by.clone().or_else(|| Some("user".to_string())),
            confirmed_at: Some(timestamp.to_string()),
        },
    )?;
    if decision.status != "decision_recorded" {
        return Err(format!(
            "J2-B B1 用户确认未写入，无法进入 Phase A：{}",
            decision.status
        ));
    }

    let phase_a = real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &RunRealExecutionProductCommandPhaseAInput {
            product_command_id: product_command_id.clone(),
            expected_product_command_store_revision: Some(decision.store_revision),
            expected_session_continuation_store_revision: input
                .expected_session_continuation_store_revision,
            actor_role: "developer_execution".to_string(),
            execution_decision: Some("approved_for_phase_a".to_string()),
            timeout_ms: Some(120_000),
            requested_at: Some(timestamp.to_string()),
        },
        timestamp,
        &format!("{write_id}-phase-a"),
    )?;
    if phase_a.status != "phase_a_completed" {
        return Err(format!("J2-B B1 Phase A 未完成：{}", phase_a.status));
    }

    let phase_b = real_execution_command::run_real_execution_product_command_phase_b_with_runner(
        workflow_state_path,
        &RunRealExecutionProductCommandPhaseBInput {
            product_command_id: product_command_id.clone(),
            expected_product_command_store_revision: Some(phase_a.product_command_store_revision),
            expected_session_continuation_store_revision: phase_a
                .session_continuation_store_revision,
            actor_role: "developer_execution".to_string(),
            execution_decision: Some("approved_for_phase_b".to_string()),
            authorization: j2_b_b1_authorization(workflow_state_path, &product_command_id),
            prompt_body: J2_B_B1_CANONICAL_PROMPT.to_string(),
            requested_at: Some(timestamp.to_string()),
        },
        timestamp,
        &format!("{write_id}-phase-b"),
        last_message_path,
        runner,
    )?;

    apply_j2_b_b1_outputs_to_plan(&mut plan, &phase_a, &phase_b);
    let audit_event_id =
        append_j2_b_b1_audit_event(workflow_state_path, &plan, &phase_a, &phase_b, timestamp)?;
    if let Some(unit) = plan
        .run_units
        .iter_mut()
        .find(|unit| unit.run_unit_kind == "developer_execution")
    {
        unit.audit_refs.push(audit_event_id.clone());
    }
    let runtime_log_refs = [
        phase_a.runtime_log_ref.clone(),
        phase_b.runtime_log_ref.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let audit_refs = crate::dedupe_strings(
        phase_a
            .audit_refs
            .iter()
            .chain(phase_b.audit_refs.iter())
            .chain(std::iter::once(&audit_event_id))
            .cloned()
            .collect(),
    );
    let readback_ref = phase_b
        .product_command_attempt
        .as_ref()
        .map(|attempt| format!("readback:{}", attempt.attempt_id));
    Ok(ProjectWorkflowAutomationJ2BB1Output {
        status: phase_b.status.clone(),
        plan,
        product_command_id,
        preview,
        prepare_output: prepare,
        decision_output: decision,
        phase_a_output: phase_a,
        phase_b_output: phase_b.clone(),
        prompt_body_persisted: false,
        allowed_project_write_roots: Vec::new(),
        runtime_log_refs,
        audit_refs,
        readback_ref,
        blocked_reasons: phase_b.blocked_reasons.clone(),
        warnings: crate::dedupe_strings(vec![
            "j2_b_b1_bridge_used_real_execution_product_command_family".to_string(),
            "prompt_body_runtime_only_not_persisted".to_string(),
            "read_only_allowed_write_roots_empty".to_string(),
        ]),
    })
}

pub(crate) fn run_project_workflow_automation_j2_b_b2_at(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationJ2BB2Input,
    timestamp: &str,
    write_id: &str,
) -> Result<ProjectWorkflowAutomationJ2BB2Output, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path = workflow_state_path
        .parent()
        .ok_or_else(|| "J2-B B2 workflow_state_path 缺少 parent".to_string())?
        .join(format!(
            "j2-b-b2-last-message-{}.json",
            stable_id(timestamp)
        ));
    run_project_workflow_automation_j2_b_b2_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_project_workflow_automation_j2_b_b2_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationJ2BB2Input,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<ProjectWorkflowAutomationJ2BB2Output, String> {
    validate_j2_b_b2_input(input)?;
    let mut value = read_workflow_state_value(workflow_state_path)?;
    if let Some(expected) = input.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "J2-B B2 workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, J2_B_B2_WORKFLOW_ID) {
        return Err("J2-B B2 找不到冻结 workflow，拒绝准备真实执行点".to_string());
    }
    if !node_exists(&value, J2_B_B2_WORKFLOW_ID, J2_B_B2_NODE_ID) {
        return Err("J2-B B2 找不到冻结 codex-dev node，拒绝准备真实执行点".to_string());
    }

    let automation_input = j2_b_b2_automation_input(input);
    let work_item_id = ensure_work_item(
        workflow_state_path,
        &mut value,
        &automation_input,
        J2_B_B2_PROJECT_ID,
        J2_B_B2_WORKFLOW_ID,
        timestamp,
    )?;
    if find_work_item(&value, J2_B_B2_WORKFLOW_ID, &work_item_id).is_none() {
        return Err(format!(
            "J2-B B2 找不到工作项，无法绑定 run unit：{work_item_id}"
        ));
    }

    let mut plan = build_plan(
        &automation_input,
        J2_B_B2_PROJECT_ID,
        J2_B_B2_WORKFLOW_ID,
        Some(work_item_id.clone()),
        Some(J2_B_B2_NODE_ID.to_string()),
        timestamp,
    );
    plan.current_phase = "developer_execution".to_string();
    plan.next_step =
        "J2-B B2 开发线通过统一 Product Command Phase B 受控创建新 session。".to_string();
    plan.warnings = crate::dedupe_strings(vec![
        "j2_b_b2_workspace_write_real_new_session_execution_point".to_string(),
        "prompt_body_runtime_only_not_persisted".to_string(),
        "allowed_write_path_must_be_verified_by_hash".to_string(),
    ]);
    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution")
        .cloned()
        .ok_or_else(|| "J2-B B2 developer run unit missing".to_string())?;
    let codex_control = j2_b_b2_codex_control(&developer_unit);
    let preview_input = PreviewRealExecutionProductCommandInput {
        source_kind: "codex_control".to_string(),
        h5_dispatch_preview: None,
        codex_control: Some(codex_control.clone()),
        requested_by: input
            .requested_by
            .clone()
            .or_else(|| Some("user".to_string())),
        created_at: Some(timestamp.to_string()),
    };
    let preview = real_execution_command::preview_real_execution_product_command_at(
        workflow_state_path,
        &preview_input,
    )?;
    if !preview.blocked_reasons.is_empty() {
        return Err(format!(
            "J2-B B2 preview 被 guard 阻断：{}",
            preview.blocked_reasons.join(", ")
        ));
    }
    let prepare = real_execution_command::prepare_real_execution_product_command_at(
        workflow_state_path,
        &PrepareRealExecutionProductCommandInput {
            source_kind: "codex_control".to_string(),
            h5_dispatch_preview: None,
            codex_control: Some(codex_control.clone()),
            expected_store_revision: input.expected_product_command_store_revision,
            requested_by: input
                .requested_by
                .clone()
                .or_else(|| Some("user".to_string())),
            created_at: Some(timestamp.to_string()),
        },
    )?;
    if prepare.status != "prepared" {
        return Err(format!(
            "J2-B B2 prepare 未进入 prepared：{}",
            prepare.status
        ));
    }
    let product_command_id = prepare
        .product_command_id
        .clone()
        .ok_or_else(|| "J2-B B2 prepare 缺少 product_command_id".to_string())?;
    let decision = real_execution_command::record_real_execution_product_command_decision_at(
        workflow_state_path,
        &RecordRealExecutionProductCommandDecisionInput {
            product_command_id: product_command_id.clone(),
            decision: "approved".to_string(),
            expected_store_revision: Some(prepare.store_revision),
            confirmed_by: "user".to_string(),
            risk_acknowledgement: input.risk_acknowledgement.clone().unwrap_or_else(|| {
                "确认 J2-B B2 只能在隔离项目内通过统一 Product Command Phase B 创建新 session，prompt body 不持久化。"
                    .to_string()
            }),
            allowed_once: true,
            reason: input.reason.clone().unwrap_or_else(|| {
                "J2-B B2 冻结隔离项目 workspace-write run unit 真实执行点。".to_string()
            }),
            requested_by: input.requested_by.clone().or_else(|| Some("user".to_string())),
            confirmed_at: Some(timestamp.to_string()),
        },
    )?;
    if decision.status != "decision_recorded" {
        return Err(format!(
            "J2-B B2 用户确认未写入，无法进入 Phase A：{}",
            decision.status
        ));
    }

    let phase_a = real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &RunRealExecutionProductCommandPhaseAInput {
            product_command_id: product_command_id.clone(),
            expected_product_command_store_revision: Some(decision.store_revision),
            expected_session_continuation_store_revision: input
                .expected_session_continuation_store_revision,
            actor_role: "developer_execution".to_string(),
            execution_decision: Some("approved_for_phase_a".to_string()),
            timeout_ms: Some(120_000),
            requested_at: Some(timestamp.to_string()),
        },
        timestamp,
        &format!("{write_id}-phase-a"),
    )?;
    if phase_a.status != "phase_a_completed" {
        return Err(format!(
            "J2-B B2 Phase A 未完成：{}；{}",
            phase_a.status,
            phase_a.blocked_reasons.join(",")
        ));
    }

    let phase_b =
        real_execution_command::run_real_execution_product_command_new_session_phase_b_with_runner(
            workflow_state_path,
            &RunRealExecutionProductCommandNewSessionPhaseBInput {
                product_command_id: product_command_id.clone(),
                expected_product_command_store_revision: Some(
                    phase_a.product_command_store_revision,
                ),
                expected_session_continuation_store_revision: phase_a
                    .session_continuation_store_revision,
                actor_role: "developer_execution".to_string(),
                execution_decision: Some("approved_for_h3_b".to_string()),
                authorization: j2_b_b2_authorization(workflow_state_path),
                prompt_body: J2_B_B2_CANONICAL_PROMPT.to_string(),
                requested_at: Some(timestamp.to_string()),
            },
            timestamp,
            &format!("{write_id}-phase-b"),
            last_message_path,
            runner,
        )?;

    apply_j2_b_b1_outputs_to_plan(&mut plan, &phase_a, &phase_b);
    let audit_event_id =
        append_j2_b_b2_audit_event(workflow_state_path, &plan, &phase_a, &phase_b, timestamp)?;
    if let Some(unit) = plan
        .run_units
        .iter_mut()
        .find(|unit| unit.run_unit_kind == "developer_execution")
    {
        unit.audit_refs.push(audit_event_id.clone());
    }
    let runtime_log_refs = [
        phase_a.runtime_log_ref.clone(),
        phase_b.runtime_log_ref.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let audit_refs = crate::dedupe_strings(
        phase_a
            .audit_refs
            .iter()
            .chain(phase_b.audit_refs.iter())
            .chain(std::iter::once(&audit_event_id))
            .cloned()
            .collect(),
    );
    let readback_ref = phase_b
        .product_command_attempt
        .as_ref()
        .map(|attempt| format!("readback:{}", attempt.attempt_id));
    Ok(ProjectWorkflowAutomationJ2BB2Output {
        status: phase_b.status.clone(),
        plan,
        product_command_id,
        preview,
        prepare_output: prepare,
        decision_output: decision,
        phase_a_output: phase_a,
        phase_b_output: phase_b.clone(),
        prompt_body_persisted: false,
        allowed_project_write_roots: vec![J2_B_B2_ALLOWED_WRITE_ROOT.to_string()],
        allowed_project_write_path: J2_B_B2_ALLOWED_WRITE_PATH.to_string(),
        runtime_log_refs,
        audit_refs,
        readback_ref,
        blocked_reasons: phase_b.blocked_reasons.clone(),
        warnings: crate::dedupe_strings(vec![
            "j2_b_b2_bridge_used_real_execution_product_command_family".to_string(),
            "prompt_body_runtime_only_not_persisted".to_string(),
            "workspace_write_limited_by_fixture_hash_and_allowed_path".to_string(),
        ]),
    })
}

pub(crate) fn run_project_workflow_automation_k3_b_at(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationK3BInput,
    timestamp: &str,
    write_id: &str,
) -> Result<ProjectWorkflowAutomationK3BOutput, String> {
    let config = k3_b_config(&input.execution_point_id)?;
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path = workflow_state_path
        .parent()
        .ok_or_else(|| "K3-B workflow_state_path 缺少 parent".to_string())?
        .join(format!(
            "{}-last-message-{}.json",
            config.execution_point_id.replace("stage-k-", ""),
            stable_id(timestamp)
        ));
    run_project_workflow_automation_k3_b_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_project_workflow_automation_k3_b_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationK3BInput,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<ProjectWorkflowAutomationK3BOutput, String> {
    let config = k3_b_config(&input.execution_point_id)?;
    validate_k3_b_input(input, &config)?;
    let mut value = read_workflow_state_value(workflow_state_path)?;
    if let Some(expected) = input.expected_workflow_revision {
        let current = i64_value(&value, "workflow_version").unwrap_or_default();
        if current != expected {
            return Err(format!(
                "K3-B workflow revision 不匹配：expected {expected}, actual {current}"
            ));
        }
    }
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    if !workflow_exists(&value, config.workflow_id) {
        return Err(format!(
            "K3-B 找不到冻结 workflow，拒绝准备真实执行点：{}",
            config.workflow_id
        ));
    }
    if !node_exists(&value, config.workflow_id, config.node_id) {
        return Err(format!(
            "K3-B 找不到冻结开发线 node，拒绝准备真实执行点：{}",
            config.node_id
        ));
    }

    let automation_input = k3_b_automation_input(input, &config);
    ensure_k3_b_work_item(workflow_state_path, &mut value, &config, input, timestamp)?;
    let mut plan = build_plan(
        &automation_input,
        config.project_id,
        config.workflow_id,
        Some(config.work_item_id.to_string()),
        Some(config.node_id.to_string()),
        timestamp,
    );
    apply_k3_b_frozen_refs_to_plan(&mut plan, &config);

    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution")
        .cloned()
        .ok_or_else(|| "K3-B developer run unit missing".to_string())?;
    let codex_control = k3_b_codex_control(&config, &developer_unit);
    let requested_by = input
        .requested_by
        .clone()
        .or_else(|| Some("user".to_string()));
    let preview_input = PreviewRealExecutionProductCommandInput {
        source_kind: "codex_control".to_string(),
        h5_dispatch_preview: None,
        codex_control: Some(codex_control.clone()),
        requested_by: requested_by.clone(),
        created_at: Some(timestamp.to_string()),
    };
    let preview = real_execution_command::preview_real_execution_product_command_at(
        workflow_state_path,
        &preview_input,
    )?;
    if !preview.blocked_reasons.is_empty() {
        return Err(format!(
            "K3-B preview 被 guard 阻断：{}",
            preview.blocked_reasons.join(", ")
        ));
    }
    let prepare = real_execution_command::prepare_real_execution_product_command_at(
        workflow_state_path,
        &PrepareRealExecutionProductCommandInput {
            source_kind: "codex_control".to_string(),
            h5_dispatch_preview: None,
            codex_control: Some(codex_control.clone()),
            expected_store_revision: input.expected_product_command_store_revision,
            requested_by: requested_by.clone(),
            created_at: Some(timestamp.to_string()),
        },
    )?;
    if prepare.status != "prepared" {
        return Err(format!("K3-B prepare 未进入 prepared：{}", prepare.status));
    }
    let product_command_id = prepare
        .product_command_id
        .clone()
        .ok_or_else(|| "K3-B prepare 缺少 product_command_id".to_string())?;
    let decision = real_execution_command::record_real_execution_product_command_decision_at(
        workflow_state_path,
        &RecordRealExecutionProductCommandDecisionInput {
            product_command_id: product_command_id.clone(),
            decision: "approved".to_string(),
            expected_store_revision: Some(prepare.store_revision),
            confirmed_by: "user".to_string(),
            risk_acknowledgement: input.risk_acknowledgement.clone().unwrap_or_else(|| {
                format!(
                    "确认 {} 只通过 K3-B 专用 bridge 和统一 Product Command Phase B；prompt body 仅作为 runtime input。",
                    config.execution_point_id
                )
            }),
            allowed_once: true,
            reason: input.reason.clone().unwrap_or_else(|| {
                format!(
                    "{} 冻结 K3 developer_execution run unit 真实执行前置 harness。",
                    config.execution_point_id
                )
            }),
            requested_by,
            confirmed_at: Some(timestamp.to_string()),
        },
    )?;
    if decision.status != "decision_recorded" {
        return Err(format!(
            "K3-B 用户确认未写入，无法进入 Phase A：{}；blocked={}; warnings={}",
            decision.status,
            decision.blocked_reasons.join(","),
            decision.warnings.join(",")
        ));
    }

    let phase_a = real_execution_command::run_real_execution_product_command_phase_a_at(
        workflow_state_path,
        &RunRealExecutionProductCommandPhaseAInput {
            product_command_id: product_command_id.clone(),
            expected_product_command_store_revision: Some(decision.store_revision),
            expected_session_continuation_store_revision: input
                .expected_session_continuation_store_revision,
            actor_role: "developer_execution".to_string(),
            execution_decision: Some("approved_for_phase_a".to_string()),
            timeout_ms: Some(120_000),
            requested_at: Some(timestamp.to_string()),
        },
        timestamp,
        &format!("{write_id}-phase-a"),
    )?;
    if phase_a.status != "phase_a_completed" {
        return Err(format!(
            "K3-B Phase A 未完成：{}；{}",
            phase_a.status,
            phase_a.blocked_reasons.join(",")
        ));
    }

    let runtime_prompt_body = input.runtime_prompt_body.clone().unwrap_or_default();
    let phase_b = if config.operation_id == "resume" {
        real_execution_command::run_real_execution_product_command_phase_b_with_runner(
            workflow_state_path,
            &RunRealExecutionProductCommandPhaseBInput {
                product_command_id: product_command_id.clone(),
                expected_product_command_store_revision: Some(
                    phase_a.product_command_store_revision,
                ),
                expected_session_continuation_store_revision: phase_a
                    .session_continuation_store_revision,
                actor_role: "developer_execution".to_string(),
                execution_decision: Some("approved_for_phase_b".to_string()),
                authorization: k3_b_resume_authorization(workflow_state_path, &config),
                prompt_body: runtime_prompt_body,
                requested_at: Some(timestamp.to_string()),
            },
            timestamp,
            &format!("{write_id}-phase-b"),
            last_message_path,
            runner,
        )?
    } else {
        real_execution_command::run_real_execution_product_command_new_session_phase_b_with_runner(
            workflow_state_path,
            &RunRealExecutionProductCommandNewSessionPhaseBInput {
                product_command_id: product_command_id.clone(),
                expected_product_command_store_revision: Some(
                    phase_a.product_command_store_revision,
                ),
                expected_session_continuation_store_revision: phase_a
                    .session_continuation_store_revision,
                actor_role: "developer_execution".to_string(),
                execution_decision: Some("approved_for_h3_b".to_string()),
                authorization: k3_b_new_session_authorization(workflow_state_path, &config),
                prompt_body: runtime_prompt_body,
                requested_at: Some(timestamp.to_string()),
            },
            timestamp,
            &format!("{write_id}-phase-b"),
            last_message_path,
            runner,
        )?
    };

    apply_k3_b_outputs_to_plan(&mut plan, &phase_a, &phase_b, &config);
    let audit_event_id = append_k3_b_audit_event(
        workflow_state_path,
        &plan,
        &phase_a,
        &phase_b,
        &config,
        timestamp,
    )?;
    if let Some(unit) = plan
        .run_units
        .iter_mut()
        .find(|unit| unit.run_unit_kind == "developer_execution")
    {
        unit.audit_refs.push(audit_event_id.clone());
    }
    let runtime_log_refs = [
        phase_a.runtime_log_ref.clone(),
        phase_b.runtime_log_ref.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let audit_refs = crate::dedupe_strings(
        phase_a
            .audit_refs
            .iter()
            .chain(phase_b.audit_refs.iter())
            .chain(std::iter::once(&audit_event_id))
            .cloned()
            .collect(),
    );
    let readback_ref = phase_b
        .product_command_attempt
        .as_ref()
        .map(|attempt| format!("readback:{}", attempt.attempt_id));
    Ok(ProjectWorkflowAutomationK3BOutput {
        status: phase_b.status.clone(),
        execution_point_id: config.execution_point_id.to_string(),
        run_unit_id: config.run_unit_id.to_string(),
        workflow_id: config.workflow_id.to_string(),
        work_item_id: config.work_item_id.to_string(),
        task_memory_packet_ref: config.memory_packet_ref.to_string(),
        permission_envelope_ref: config.permission_envelope_ref.to_string(),
        readback_marker: config.readback_marker.to_string(),
        plan,
        product_command_id,
        preview,
        prepare_output: prepare,
        decision_output: decision,
        phase_a_output: phase_a,
        phase_b_output: phase_b.clone(),
        prompt_body_persisted: false,
        allowed_project_write_roots: config.allowed_write_roots(),
        allowed_project_write_path: config.allowed_write_path.map(str::to_string),
        baseline_refs: config.baseline_refs(),
        manifest_requirements: config.manifest_requirements(),
        runtime_log_refs,
        audit_refs,
        readback_ref,
        blocked_reasons: phase_b.blocked_reasons.clone(),
        warnings: k3_b_output_warnings(&config, input.runtime_prompt_body.is_some(), &phase_b),
    })
}

pub(crate) fn load_project_workflow_automation_read_model(
    workflow_state_path: &Path,
    generated_at: &str,
) -> ProjectWorkflowAutomationReadModel {
    let Ok(value) = read_workflow_state_value(workflow_state_path) else {
        return read_model_from_plan(
            generated_at,
            None,
            None,
            vec!["workflow_state_unavailable_for_project_workflow_automation".to_string()],
        );
    };
    let latest = value
        .get("audit_events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().rev().find(|event| {
                optional_string_from(event, "event_type").as_deref() == Some(J2_EVENT_TYPE)
            })
        });
    let Some(event) = latest else {
        return read_model_from_plan(
            generated_at,
            None,
            None,
            vec!["project_workflow_automation_not_recorded".to_string()],
        );
    };
    let plan = event
        .get("plan")
        .cloned()
        .and_then(|value| serde_json::from_value::<ProjectWorkflowAutomationPlan>(value).ok());
    read_model_from_plan(
        generated_at,
        optional_string_from(event, "result_status"),
        plan,
        vec!["k3_level_a_read_model_from_workflow_audit_event".to_string()],
    )
}

fn validate_input(input: &ProjectWorkflowAutomationInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("K3 Level A project_root 不能为空".to_string());
    }
    if input.user_goal.trim().is_empty() {
        return Err("K3 Level A user_goal 不能为空".to_string());
    }
    if input.confirmed_by.as_deref().unwrap_or("user") != "user" {
        return Err("K3 Level A Phase A no-op 仍要求 confirmed_by=user".to_string());
    }
    Ok(())
}

fn validate_j2_b_b1_input(input: &ProjectWorkflowAutomationJ2BB1Input) -> Result<(), String> {
    let canonical_hash = sha256_hex(J2_B_B1_CANONICAL_PROMPT);
    if canonical_hash != J2_B_B1_PROMPT_HASH {
        return Err(format!(
            "J2-B B1 canonical prompt hash mismatch in code: expected {J2_B_B1_PROMPT_HASH}, actual {canonical_hash}"
        ));
    }
    for (field, actual, expected) in [
        (
            "project_root",
            input
                .project_root
                .as_deref()
                .unwrap_or(J2_B_B1_PROJECT_ROOT),
            J2_B_B1_PROJECT_ROOT,
        ),
        (
            "project_id",
            input.project_id.as_deref().unwrap_or(J2_B_B1_PROJECT_ID),
            J2_B_B1_PROJECT_ID,
        ),
        (
            "workflow_id",
            input.workflow_id.as_deref().unwrap_or(J2_B_B1_WORKFLOW_ID),
            J2_B_B1_WORKFLOW_ID,
        ),
        (
            "workflow_node_id",
            input.workflow_node_id.as_deref().unwrap_or(J2_B_B1_NODE_ID),
            J2_B_B1_NODE_ID,
        ),
        (
            "target_session_id",
            input
                .target_session_id
                .as_deref()
                .unwrap_or(J2_B_B1_SESSION_ID),
            J2_B_B1_SESSION_ID,
        ),
        (
            "sandbox",
            input.sandbox.as_deref().unwrap_or(J2_B_B1_SANDBOX),
            J2_B_B1_SANDBOX,
        ),
        (
            "prompt_summary",
            input
                .prompt_summary
                .as_deref()
                .unwrap_or(J2_B_B1_PROMPT_SUMMARY),
            J2_B_B1_PROMPT_SUMMARY,
        ),
        (
            "prompt_ref",
            input.prompt_ref.as_deref().unwrap_or(J2_B_B1_PROMPT_REF),
            J2_B_B1_PROMPT_REF,
        ),
        (
            "prompt_hash",
            input.prompt_hash.as_deref().unwrap_or(J2_B_B1_PROMPT_HASH),
            J2_B_B1_PROMPT_HASH,
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "J2-B B1 冻结字段不匹配：{field} expected={expected} actual={actual}"
            ));
        }
    }
    if input.confirmed_by.as_deref() != Some("user") {
        return Err("J2-B B1 真实 Phase B bridge 要求 confirmed_by=user".to_string());
    }
    Ok(())
}

fn validate_j2_b_b2_input(input: &ProjectWorkflowAutomationJ2BB2Input) -> Result<(), String> {
    let canonical_hash = sha256_hex(J2_B_B2_CANONICAL_PROMPT);
    if canonical_hash != J2_B_B2_PROMPT_HASH {
        return Err(format!(
            "J2-B B2 canonical prompt hash mismatch in code: expected {J2_B_B2_PROMPT_HASH}, actual {canonical_hash}"
        ));
    }
    for (field, actual, expected) in [
        (
            "project_root",
            input
                .project_root
                .as_deref()
                .unwrap_or(J2_B_B2_PROJECT_ROOT),
            J2_B_B2_PROJECT_ROOT,
        ),
        (
            "project_id",
            input.project_id.as_deref().unwrap_or(J2_B_B2_PROJECT_ID),
            J2_B_B2_PROJECT_ID,
        ),
        (
            "workflow_id",
            input.workflow_id.as_deref().unwrap_or(J2_B_B2_WORKFLOW_ID),
            J2_B_B2_WORKFLOW_ID,
        ),
        (
            "workflow_node_id",
            input.workflow_node_id.as_deref().unwrap_or(J2_B_B2_NODE_ID),
            J2_B_B2_NODE_ID,
        ),
        (
            "sandbox",
            input.sandbox.as_deref().unwrap_or(J2_B_B2_SANDBOX),
            J2_B_B2_SANDBOX,
        ),
        (
            "allowed_write_path",
            input
                .allowed_write_path
                .as_deref()
                .unwrap_or(J2_B_B2_ALLOWED_WRITE_PATH),
            J2_B_B2_ALLOWED_WRITE_PATH,
        ),
        (
            "prompt_summary",
            input
                .prompt_summary
                .as_deref()
                .unwrap_or(J2_B_B2_PROMPT_SUMMARY),
            J2_B_B2_PROMPT_SUMMARY,
        ),
        (
            "prompt_ref",
            input.prompt_ref.as_deref().unwrap_or(J2_B_B2_PROMPT_REF),
            J2_B_B2_PROMPT_REF,
        ),
        (
            "prompt_hash",
            input.prompt_hash.as_deref().unwrap_or(J2_B_B2_PROMPT_HASH),
            J2_B_B2_PROMPT_HASH,
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "J2-B B2 冻结字段不匹配：{field} expected={expected} actual={actual}"
            ));
        }
    }
    if input.confirmed_by.as_deref() != Some("user") {
        return Err("J2-B B2 真实 Phase B bridge 要求 confirmed_by=user".to_string());
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct K3BExecutionPointConfig {
    execution_point_id: &'static str,
    event_type: &'static str,
    operation_id: &'static str,
    session_mode: &'static str,
    project_root: &'static str,
    project_id: &'static str,
    workflow_id: &'static str,
    node_id: &'static str,
    run_unit_id: &'static str,
    work_item_id: &'static str,
    target_session_id: Option<&'static str>,
    sandbox: &'static str,
    allowed_write_root: Option<&'static str>,
    allowed_write_path: Option<&'static str>,
    prompt_summary: &'static str,
    prompt_ref: &'static str,
    prompt_hash: &'static str,
    task_package_ref: &'static str,
    memory_packet_ref: &'static str,
    permission_envelope_ref: &'static str,
    readback_marker: &'static str,
}

impl K3BExecutionPointConfig {
    fn allowed_write_roots(&self) -> Vec<String> {
        self.allowed_write_root
            .map(|root| vec![root.to_string()])
            .unwrap_or_default()
    }

    fn baseline_refs(&self) -> Vec<String> {
        match self.execution_point_id {
            K3_B_B1_EXECUTION_POINT_ID => vec![
                "mario:index.html:f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf".to_string(),
                "mario:styles.css:6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f".to_string(),
                "mario:game.js:814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd".to_string(),
                "mario:README.md:02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5".to_string(),
            ],
            K3_B_B2_EXECUTION_POINT_ID => vec![
                "isolated:README.md:cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c".to_string(),
                "isolated:.workbench/stage-k/k2/new-session-write-probe.md:603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07".to_string(),
            ],
            _ => Vec::new(),
        }
    }

    fn manifest_requirements(&self) -> Vec<String> {
        match self.execution_point_id {
            K3_B_B1_EXECUTION_POINT_ID => vec![
                "read_only_core_hashes_must_match_before_after".to_string(),
                "allowed_write_roots_must_be_empty".to_string(),
            ],
            K3_B_B2_EXECUTION_POINT_ID => vec![
                "workspace_write_must_compare_full_manifest_before_after".to_string(),
                format!(
                    "only_allowed_write_path_may_change:{}",
                    K3_B_B2_ALLOWED_WRITE_PATH
                ),
                "writes_project_files_must_be_proven_by_manifest_not_sandbox".to_string(),
            ],
            _ => Vec::new(),
        }
    }
}

fn k3_b_config(execution_point_id: &str) -> Result<K3BExecutionPointConfig, String> {
    match execution_point_id {
        K3_B_B1_EXECUTION_POINT_ID => Ok(K3BExecutionPointConfig {
            execution_point_id: K3_B_B1_EXECUTION_POINT_ID,
            event_type: K3_B_B1_EVENT_TYPE,
            operation_id: "resume",
            session_mode: "resume_existing_session",
            project_root: K3_B_B1_PROJECT_ROOT,
            project_id: K3_B_B1_PROJECT_ID,
            workflow_id: K3_B_B1_WORKFLOW_ID,
            node_id: K3_B_B1_NODE_ID,
            run_unit_id: K3_B_B1_RUN_UNIT_ID,
            work_item_id: K3_B_B1_WORK_ITEM_ID,
            target_session_id: Some(K3_B_B1_SESSION_ID),
            sandbox: K3_B_B1_SANDBOX,
            allowed_write_root: None,
            allowed_write_path: None,
            prompt_summary: K3_B_B1_PROMPT_SUMMARY,
            prompt_ref: K3_B_B1_PROMPT_REF,
            prompt_hash: K3_B_B1_PROMPT_HASH,
            task_package_ref: K3_B_B1_TASK_PACKAGE_REF,
            memory_packet_ref: K3_B_B1_MEMORY_PACKET_REF,
            permission_envelope_ref: K3_B_B1_PERMISSION_ENVELOPE_REF,
            readback_marker: K3_B_B1_READBACK_MARKER,
        }),
        K3_B_B2_EXECUTION_POINT_ID => Ok(K3BExecutionPointConfig {
            execution_point_id: K3_B_B2_EXECUTION_POINT_ID,
            event_type: K3_B_B2_EVENT_TYPE,
            operation_id: "new_session",
            session_mode: "new_session_execution_point",
            project_root: K3_B_B2_PROJECT_ROOT,
            project_id: K3_B_B2_PROJECT_ID,
            workflow_id: K3_B_B2_WORKFLOW_ID,
            node_id: K3_B_B2_NODE_ID,
            run_unit_id: K3_B_B2_RUN_UNIT_ID,
            work_item_id: K3_B_B2_WORK_ITEM_ID,
            target_session_id: None,
            sandbox: K3_B_B2_SANDBOX,
            allowed_write_root: Some(K3_B_B2_ALLOWED_WRITE_ROOT),
            allowed_write_path: Some(K3_B_B2_ALLOWED_WRITE_PATH),
            prompt_summary: K3_B_B2_PROMPT_SUMMARY,
            prompt_ref: K3_B_B2_PROMPT_REF,
            prompt_hash: K3_B_B2_PROMPT_HASH,
            task_package_ref: K3_B_B2_TASK_PACKAGE_REF,
            memory_packet_ref: K3_B_B2_MEMORY_PACKET_REF,
            permission_envelope_ref: K3_B_B2_PERMISSION_ENVELOPE_REF,
            readback_marker: K3_B_B2_READBACK_MARKER,
        }),
        other => Err(format!("unsupported_k3_b_execution_point:{other}")),
    }
}

fn validate_k3_b_input(
    input: &ProjectWorkflowAutomationK3BInput,
    config: &K3BExecutionPointConfig,
) -> Result<(), String> {
    for (field, actual, expected) in [
        (
            "project_root",
            input.project_root.as_deref().unwrap_or(config.project_root),
            config.project_root,
        ),
        (
            "project_id",
            input.project_id.as_deref().unwrap_or(config.project_id),
            config.project_id,
        ),
        (
            "workflow_id",
            input.workflow_id.as_deref().unwrap_or(config.workflow_id),
            config.workflow_id,
        ),
        (
            "workflow_node_id",
            input.workflow_node_id.as_deref().unwrap_or(config.node_id),
            config.node_id,
        ),
        (
            "run_unit_id",
            input.run_unit_id.as_deref().unwrap_or(config.run_unit_id),
            config.run_unit_id,
        ),
        (
            "work_item_id",
            input.work_item_id.as_deref().unwrap_or(config.work_item_id),
            config.work_item_id,
        ),
        (
            "task_memory_packet_ref",
            input
                .task_memory_packet_ref
                .as_deref()
                .unwrap_or(config.memory_packet_ref),
            config.memory_packet_ref,
        ),
        (
            "permission_envelope_ref",
            input
                .permission_envelope_ref
                .as_deref()
                .unwrap_or(config.permission_envelope_ref),
            config.permission_envelope_ref,
        ),
        (
            "readback_marker",
            input
                .readback_marker
                .as_deref()
                .unwrap_or(config.readback_marker),
            config.readback_marker,
        ),
        (
            "sandbox",
            input.sandbox.as_deref().unwrap_or(config.sandbox),
            config.sandbox,
        ),
        (
            "prompt_summary",
            input
                .prompt_summary
                .as_deref()
                .unwrap_or(config.prompt_summary),
            config.prompt_summary,
        ),
        (
            "prompt_ref",
            input.prompt_ref.as_deref().unwrap_or(config.prompt_ref),
            config.prompt_ref,
        ),
        (
            "prompt_hash",
            input.prompt_hash.as_deref().unwrap_or(config.prompt_hash),
            config.prompt_hash,
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "K3-B 冻结字段不匹配：{field} expected={expected} actual={actual}"
            ));
        }
    }
    if let Some(expected_session) = config.target_session_id {
        if input
            .target_session_id
            .as_deref()
            .unwrap_or(expected_session)
            != expected_session
        {
            return Err("K3-B target_session_id 冻结字段不匹配".to_string());
        }
    } else if input
        .target_session_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
        == false
    {
        return Err("K3-B new_session 执行点不得预绑定 target_session_id".to_string());
    }
    if let Some(expected_path) = config.allowed_write_path {
        if input.allowed_write_path.as_deref().unwrap_or(expected_path) != expected_path {
            return Err("K3-B allowed_write_path 冻结字段不匹配".to_string());
        }
    }
    if input.confirmed_by.as_deref() != Some("user") {
        return Err("K3-B Phase B bridge 要求 confirmed_by=user".to_string());
    }
    if let Some(prompt_body) = input.runtime_prompt_body.as_ref() {
        let actual = sha256_hex(prompt_body);
        if actual != config.prompt_hash {
            return Err(format!(
                "K3-B runtime prompt hash mismatch：expected={} actual={actual}",
                config.prompt_hash
            ));
        }
    }
    Ok(())
}

fn j2_b_b1_automation_input(
    input: &ProjectWorkflowAutomationJ2BB1Input,
) -> ProjectWorkflowAutomationInput {
    ProjectWorkflowAutomationInput {
        project_root: J2_B_B1_PROJECT_ROOT.to_string(),
        project_id: Some(J2_B_B1_PROJECT_ID.to_string()),
        workflow_id: Some(J2_B_B1_WORKFLOW_ID.to_string()),
        workflow_node_id: Some(J2_B_B1_NODE_ID.to_string()),
        work_item_id: input.work_item_id.clone(),
        user_goal: J2_B_B1_PROMPT_SUMMARY.to_string(),
        task_package_ref: input.task_package_ref.clone().or_else(|| {
            Some("task-package:j2-b:b1:mario-test:developer-run-unit:read-only:v1".to_string())
        }),
        memory_packet_ref: input.memory_packet_ref.clone().or_else(|| {
            Some("memory-packet:j2-b:b1:mario-test:developer-run-unit:read-only:v1".to_string())
        }),
        target_session_id: Some(J2_B_B1_SESSION_ID.to_string()),
        sandbox: Some(J2_B_B1_SANDBOX.to_string()),
        requested_by: input
            .requested_by
            .clone()
            .or_else(|| Some("user".to_string())),
        confirmed_by: input.confirmed_by.clone(),
        risk_acknowledgement: input.risk_acknowledgement.clone(),
        reason: input.reason.clone(),
        expected_workflow_revision: input.expected_workflow_revision,
        expected_product_command_store_revision: input.expected_product_command_store_revision,
        expected_session_continuation_store_revision: input
            .expected_session_continuation_store_revision,
    }
}

fn j2_b_b2_automation_input(
    input: &ProjectWorkflowAutomationJ2BB2Input,
) -> ProjectWorkflowAutomationInput {
    ProjectWorkflowAutomationInput {
        project_root: J2_B_B2_PROJECT_ROOT.to_string(),
        project_id: Some(J2_B_B2_PROJECT_ID.to_string()),
        workflow_id: Some(J2_B_B2_WORKFLOW_ID.to_string()),
        workflow_node_id: Some(J2_B_B2_NODE_ID.to_string()),
        work_item_id: input.work_item_id.clone(),
        user_goal: J2_B_B2_PROMPT_SUMMARY.to_string(),
        task_package_ref: input.task_package_ref.clone().or_else(|| {
            Some(
                "task-package:j2-b:b2:isolated-project:developer-run-unit:workspace-write:v1"
                    .to_string(),
            )
        }),
        memory_packet_ref: input.memory_packet_ref.clone().or_else(|| {
            Some(
                "memory-packet:j2-b:b2:isolated-project:developer-run-unit:workspace-write:v1"
                    .to_string(),
            )
        }),
        target_session_id: None,
        sandbox: Some(J2_B_B2_SANDBOX.to_string()),
        requested_by: input
            .requested_by
            .clone()
            .or_else(|| Some("user".to_string())),
        confirmed_by: input.confirmed_by.clone(),
        risk_acknowledgement: input.risk_acknowledgement.clone(),
        reason: input.reason.clone(),
        expected_workflow_revision: input.expected_workflow_revision,
        expected_product_command_store_revision: input.expected_product_command_store_revision,
        expected_session_continuation_store_revision: input
            .expected_session_continuation_store_revision,
    }
}

fn k3_b_automation_input(
    input: &ProjectWorkflowAutomationK3BInput,
    config: &K3BExecutionPointConfig,
) -> ProjectWorkflowAutomationInput {
    ProjectWorkflowAutomationInput {
        project_root: config.project_root.to_string(),
        project_id: Some(config.project_id.to_string()),
        workflow_id: Some(config.workflow_id.to_string()),
        workflow_node_id: Some(config.node_id.to_string()),
        work_item_id: Some(config.work_item_id.to_string()),
        user_goal: config.prompt_summary.to_string(),
        task_package_ref: input
            .task_package_ref
            .clone()
            .or_else(|| Some(config.task_package_ref.to_string())),
        memory_packet_ref: input
            .task_memory_packet_ref
            .clone()
            .or_else(|| Some(config.memory_packet_ref.to_string())),
        target_session_id: config.target_session_id.map(str::to_string),
        sandbox: Some(config.sandbox.to_string()),
        requested_by: input
            .requested_by
            .clone()
            .or_else(|| Some("user".to_string())),
        confirmed_by: input.confirmed_by.clone(),
        risk_acknowledgement: input.risk_acknowledgement.clone(),
        reason: input.reason.clone(),
        expected_workflow_revision: input.expected_workflow_revision,
        expected_product_command_store_revision: input.expected_product_command_store_revision,
        expected_session_continuation_store_revision: input
            .expected_session_continuation_store_revision,
    }
}

fn ensure_k3_b_work_item(
    workflow_state_path: &Path,
    value: &mut Value,
    config: &K3BExecutionPointConfig,
    input: &ProjectWorkflowAutomationK3BInput,
    timestamp: &str,
) -> Result<(), String> {
    if find_work_item(value, config.workflow_id, config.work_item_id).is_some() {
        return Ok(());
    }
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    array_mut(value, "work_items")?.push(json!({
      "work_item_id": config.work_item_id,
      "project_id": config.project_id,
      "workflow_id": config.workflow_id,
      "title": format!("K3-B 真实执行前置：{}", config.execution_point_id),
      "state": "ready_to_dispatch",
      "source_kind": "stage_k_k3_b_real_workflow_execution_bridge",
      "source_ref": input.task_package_ref.as_deref().unwrap_or(config.task_package_ref),
      "assigned_role_id": "developer_execution",
      "current_node_id": config.node_id,
      "agent_type": "codex",
      "adapter_id": "codex-local",
      "permission_level": "real_execution_product_command",
      "created_at": timestamp,
      "updated_at": timestamp
    }));
    array_mut(value, "artifacts")?.push(json!({
      "artifact_id": config.task_package_ref,
      "artifact_type": "task_package",
      "project_id": config.project_id,
      "path": Value::Null,
      "title": format!("K3-B 字段冻结任务包：{}", config.execution_point_id),
      "brief": config.prompt_summary,
      "source_kind": "stage_k_k3_b_real_workflow_execution_bridge",
      "source_ref": config.work_item_id,
      "permission_level": "real_execution_product_command",
      "version": 1,
      "stale": false,
      "stale_reasons": [],
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": [
        "k3_b_task_package_summary_no_raw_prompt",
        "real_execution_requires_separate_user_authorization"
      ]
    }));
    array_mut(value, "audit_events")?.push(json!({
      "event_id": format!("audit:k3-b-work-item:{}:{timestamp}", stable_id(config.work_item_id)),
      "event_type": "project_workflow_automation_k3_b_work_item_created",
      "target_ref": config.work_item_id,
      "execution_point_id": config.execution_point_id,
      "project_id": config.project_id,
      "workflow_id": config.workflow_id,
      "run_unit_id": config.run_unit_id,
      "actor_ref": input.requested_by.as_deref().unwrap_or("user"),
      "source_kind": "stage_k_k3_b_real_workflow_execution_bridge",
      "permission_level": "real_execution_product_command",
      "after_state": "ready_to_dispatch",
      "created_at": timestamp,
      "reason": "K3-B0 创建 K3-B 专用 work item；本事件不代表真实 Codex 已执行。",
      "backup_ref": backup.display().to_string()
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, value)?;
    let mut refreshed = read_workflow_state_value(workflow_state_path)?;
    std::mem::swap(value, &mut refreshed);
    Ok(())
}

fn ensure_work_item(
    workflow_state_path: &Path,
    value: &mut Value,
    input: &ProjectWorkflowAutomationInput,
    project_id_value: &str,
    workflow_id: &str,
    timestamp: &str,
) -> Result<String, String> {
    if let Some(work_item_id) = input.work_item_id.as_ref() {
        return Ok(work_item_id.clone());
    }
    if let Some(existing) = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(item, "source_kind").as_deref() == Some(J2_SOURCE_KIND)
            })
        })
        .and_then(|item| optional_string_from(item, "work_item_id"))
    {
        return Ok(existing);
    }
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    let is_j2_b_probe =
        input.user_goal == J2_B_B1_PROMPT_SUMMARY || input.user_goal == J2_B_B2_PROMPT_SUMMARY;
    let work_item_prefix = if is_j2_b_probe {
        "work-item:j2"
    } else {
        "work-item:automation"
    };
    let artifact_prefix = if is_j2_b_probe {
        "artifact:j2"
    } else {
        "artifact:automation"
    };
    let work_item_id = format!(
        "{work_item_prefix}:{}:{}",
        stable_id(workflow_id),
        stable_id(&input.user_goal)
    );
    let artifact_id = format!(
        "{artifact_prefix}:{}:task-package",
        stable_id(&work_item_id)
    );
    let task_package_ref = input
        .task_package_ref
        .clone()
        .unwrap_or_else(|| artifact_id.clone());
    array_mut(value, "work_items")?.push(json!({
      "work_item_id": work_item_id,
      "project_id": project_id_value,
      "workflow_id": workflow_id,
      "title": format!("项目自动编排：{}", compact_goal(&input.user_goal)),
      "state": "ready_to_dispatch",
      "source_kind": J2_SOURCE_KIND,
      "source_ref": task_package_ref,
      "assigned_role_id": "developer_execution",
      "current_node_id": format!("{workflow_id}:node:codex-dev"),
      "agent_type": "codex",
      "adapter_id": "codex-local",
      "permission_level": "workflow_event_record",
      "created_at": timestamp,
      "updated_at": timestamp
    }));
    array_mut(value, "artifacts")?.push(json!({
      "artifact_id": artifact_id,
      "artifact_type": "task_package",
      "project_id": project_id_value,
      "path": Value::Null,
      "title": "项目自动编排任务包摘要",
      "brief": compact_goal(&input.user_goal),
      "source_kind": J2_SOURCE_KIND,
      "source_ref": work_item_id,
      "permission_level": "workflow_event_record",
      "version": 1,
      "stale": false,
      "stale_reasons": [],
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": ["k3_level_a_task_package_summary_no_raw_prompt"]
    }));
    array_mut(value, "audit_events")?.push(json!({
      "event_id": format!("audit:k3-work-item:{}:{timestamp}", stable_id(&work_item_id)),
      "event_type": "project_workflow_automation_work_item_created",
      "target_ref": work_item_id,
      "project_id": project_id_value,
      "workflow_id": workflow_id,
      "actor_ref": input.requested_by.as_deref().unwrap_or("user"),
      "source_kind": J2_SOURCE_KIND,
      "permission_level": "workflow_event_record",
      "after_state": "ready_to_dispatch",
      "created_at": timestamp,
      "reason": "K3 Level A 为用户目标生成项目自动编排工作项；不代表真实 Codex 已执行。"
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, value)?;
    let mut refreshed = read_workflow_state_value(workflow_state_path)?;
    std::mem::swap(value, &mut refreshed);
    let _ = backup;
    Ok(work_item_id)
}

fn build_plan(
    input: &ProjectWorkflowAutomationInput,
    project_id_value: &str,
    workflow_id: &str,
    work_item_id: Option<String>,
    node_id: Option<String>,
    timestamp: &str,
) -> ProjectWorkflowAutomationPlan {
    let work_item = work_item_id.unwrap_or_else(|| "work-item:automation:unresolved".to_string());
    let node = node_id.unwrap_or_else(|| format!("{workflow_id}:node:codex-dev"));
    let automation_id = automation_id_for(workflow_id, &input.user_goal);
    let units = [
        (
            "director_plan",
            "project_director",
            "计划已生成，等待开发线准备受控命令。",
        ),
        (
            "developer_execution",
            "developer_execution",
            "开发线走统一 Product Command Phase A no-op。",
        ),
        (
            "verifier_check",
            "verifier_check",
            "验证线检查 no-op 结果和边界。",
        ),
        (
            "collector_summary",
            "collector_summary",
            "回收线整理 worker report 和过程事实观察。",
        ),
        (
            "director_final_review",
            "project_director",
            "主管复核 K3 Level A evidence / handoff。",
        ),
    ];
    ProjectWorkflowAutomationPlan {
        schema_version: J2_SCHEMA.to_string(),
        automation_id: automation_id.clone(),
        project_id: project_id_value.to_string(),
        project_root: input.project_root.clone(),
        workflow_id: workflow_id.to_string(),
        user_goal: input.user_goal.trim().to_string(),
        current_phase: "director_plan".to_string(),
        next_step: "准备开发线统一 Product Command 预览。".to_string(),
        run_units: units
            .into_iter()
            .map(|(kind, role, summary)| ProjectWorkflowRunUnit {
                run_unit_id: format!("run-unit:{automation_id}:{kind}"),
                run_unit_kind: kind.to_string(),
                role: role.to_string(),
                status: "planned".to_string(),
                project_id: project_id_value.to_string(),
                project_root: input.project_root.clone(),
                workflow_id: workflow_id.to_string(),
                workflow_node_id: node.clone(),
                work_item_id: work_item.clone(),
                task_package_ref: input
                    .task_package_ref
                    .clone()
                    .or_else(|| Some(format!("task-package:{automation_id}"))),
                memory_packet_ref: input
                    .memory_packet_ref
                    .clone()
                    .or_else(|| Some(format!("memory-packet:{automation_id}"))),
                product_command_preview_ref: None,
                product_command_ref: None,
                runtime_log_refs: Vec::new(),
                audit_refs: Vec::new(),
                readback_ref: None,
                readback_status: "readback_unavailable".to_string(),
                readback_result_count: None,
                worker_report_ref: None,
                capture_event_refs: Vec::new(),
                observation_refs: Vec::new(),
                memory_candidate_refs: Vec::new(),
                runner_call_allowed: false,
                prompt_sent: false,
                real_codex_executed: false,
                writes_codex_home: false,
                writes_project_files: false,
                summary: summary.to_string(),
                next_step: "等待前序 run unit 结果。".to_string(),
                blocked_reasons: Vec::new(),
                warnings: vec!["k3_level_a_no_real_codex_execution".to_string()],
            })
            .collect(),
        blocked_reasons: Vec::new(),
        warnings: vec![
            "k3_level_a_phase_a_only".to_string(),
            format!("k3_level_a_plan_generated_at:{timestamp}"),
        ],
    }
}

fn codex_control_for_unit(
    input: &ProjectWorkflowAutomationInput,
    unit: &ProjectWorkflowRunUnit,
) -> CodexControlCommandInput {
    CodexControlCommandInput {
        project_id: Some(unit.project_id.clone()),
        project_root: unit.project_root.clone(),
        workflow_id: Some(unit.workflow_id.clone()),
        node_id: Some(unit.workflow_node_id.clone()),
        work_item_id: Some(unit.work_item_id.clone()),
        task_package_ref: unit.task_package_ref.clone(),
        memory_packet_ref: unit.memory_packet_ref.clone(),
        adapter_id: "codex-local".to_string(),
        operation_id: "resume".to_string(),
        session_mode: "resume_existing_session".to_string(),
        target_session_id: input.target_session_id.clone(),
        sandbox: input
            .sandbox
            .clone()
            .unwrap_or_else(|| "read-only".to_string()),
        prompt_summary: format!(
            "K3 Level A {}：{}",
            unit.run_unit_kind,
            compact_goal(&input.user_goal)
        ),
        prompt_ref: format!(
            "task-package-ref:{}",
            unit.task_package_ref.as_deref().unwrap_or("k3-level-a")
        ),
        prompt_hash: sha256_hex(&format!("{}:{}", unit.run_unit_id, input.user_goal)),
        allowed_write_roots: vec![input.project_root.clone()],
        denied_paths: sensitive_denied_paths(),
        readback_plan: "K3 Level A Phase A no-op 只记录 readback_unavailable，结果数保持未知。"
            .to_string(),
        timeout_ms: Some(30_000),
        requested_by: input.requested_by.clone(),
    }
}

fn j2_b_b1_codex_control(unit: &ProjectWorkflowRunUnit) -> CodexControlCommandInput {
    CodexControlCommandInput {
        project_id: Some(J2_B_B1_PROJECT_ID.to_string()),
        project_root: J2_B_B1_PROJECT_ROOT.to_string(),
        workflow_id: Some(J2_B_B1_WORKFLOW_ID.to_string()),
        node_id: Some(J2_B_B1_NODE_ID.to_string()),
        work_item_id: Some(unit.work_item_id.clone()),
        task_package_ref: unit.task_package_ref.clone(),
        memory_packet_ref: unit.memory_packet_ref.clone(),
        adapter_id: "codex-local".to_string(),
        operation_id: "resume".to_string(),
        session_mode: "resume_existing_session".to_string(),
        target_session_id: Some(J2_B_B1_SESSION_ID.to_string()),
        sandbox: J2_B_B1_SANDBOX.to_string(),
        prompt_summary: J2_B_B1_PROMPT_SUMMARY.to_string(),
        prompt_ref: J2_B_B1_PROMPT_REF.to_string(),
        prompt_hash: J2_B_B1_PROMPT_HASH.to_string(),
        allowed_write_roots: Vec::new(),
        denied_paths: sensitive_denied_paths(),
        readback_plan: format!(
            "Read only the workbench-managed latest message for marker {J2_B_B1_READBACK_MARKER}; no broader session history."
        ),
        timeout_ms: Some(120_000),
        requested_by: Some("user".to_string()),
    }
}

fn j2_b_b2_codex_control(unit: &ProjectWorkflowRunUnit) -> CodexControlCommandInput {
    CodexControlCommandInput {
        project_id: Some(J2_B_B2_PROJECT_ID.to_string()),
        project_root: J2_B_B2_PROJECT_ROOT.to_string(),
        workflow_id: Some(J2_B_B2_WORKFLOW_ID.to_string()),
        node_id: Some(J2_B_B2_NODE_ID.to_string()),
        work_item_id: Some(unit.work_item_id.clone()),
        task_package_ref: unit.task_package_ref.clone(),
        memory_packet_ref: unit.memory_packet_ref.clone(),
        adapter_id: "codex-local".to_string(),
        operation_id: "new_session".to_string(),
        session_mode: "new_session_execution_point".to_string(),
        target_session_id: None,
        sandbox: J2_B_B2_SANDBOX.to_string(),
        prompt_summary: J2_B_B2_PROMPT_SUMMARY.to_string(),
        prompt_ref: J2_B_B2_PROMPT_REF.to_string(),
        prompt_hash: J2_B_B2_PROMPT_HASH.to_string(),
        allowed_write_roots: vec![J2_B_B2_ALLOWED_WRITE_ROOT.to_string()],
        denied_paths: sensitive_denied_paths(),
        readback_plan: format!(
            "Read only the workbench-managed latest message for marker {J2_B_B2_READBACK_MARKER}; no broader session history."
        ),
        timeout_ms: Some(120_000),
        requested_by: Some("user".to_string()),
    }
}

fn k3_b_codex_control(
    config: &K3BExecutionPointConfig,
    unit: &ProjectWorkflowRunUnit,
) -> CodexControlCommandInput {
    CodexControlCommandInput {
        project_id: Some(config.project_id.to_string()),
        project_root: config.project_root.to_string(),
        workflow_id: Some(config.workflow_id.to_string()),
        node_id: Some(config.node_id.to_string()),
        work_item_id: Some(config.work_item_id.to_string()),
        task_package_ref: Some(config.task_package_ref.to_string()),
        memory_packet_ref: Some(config.memory_packet_ref.to_string()),
        adapter_id: "codex-local".to_string(),
        operation_id: config.operation_id.to_string(),
        session_mode: config.session_mode.to_string(),
        target_session_id: config.target_session_id.map(str::to_string),
        sandbox: config.sandbox.to_string(),
        prompt_summary: config.prompt_summary.to_string(),
        prompt_ref: config.prompt_ref.to_string(),
        prompt_hash: config.prompt_hash.to_string(),
        allowed_write_roots: config.allowed_write_roots(),
        denied_paths: sensitive_denied_paths(),
        readback_plan: format!(
            "Read only the workbench-managed latest message for marker {}; unavailable, failed, or timed_out keeps result_count=null; no full transcript.",
            config.readback_marker
        ),
        timeout_ms: Some(120_000),
        requested_by: Some(unit.role.clone()),
    }
}

fn j2_b_b1_authorization(
    workflow_state_path: &Path,
    product_command_id: &str,
) -> crate::H2RealResumeAuthorizationMatrix {
    crate::H2RealResumeAuthorizationMatrix {
        operation_type: "resume".to_string(),
        test_project: "mario test J2-B B1 project workflow automation read-only probe".to_string(),
        project_root: J2_B_B1_PROJECT_ROOT.to_string(),
        target_cwd: J2_B_B1_PROJECT_ROOT.to_string(),
        target_session: J2_B_B1_SESSION_ID.to_string(),
        prompt_summary: J2_B_B1_PROMPT_SUMMARY.to_string(),
        prompt_sha256: J2_B_B1_PROMPT_HASH.to_string(),
        prompt_ref: J2_B_B1_PROMPT_REF.to_string(),
        allowed_write_roots: Vec::new(),
        codex_home_scope:
            "Codex CLI minimum native session state for one authorized J2-B B1 real resume; no credential material requested."
                .to_string(),
        sandbox: J2_B_B1_SANDBOX.to_string(),
        timeout_ms: Some(120_000),
        readback_plan: format!(
            "Workbench-managed latest message for product command {product_command_id}; unavailable is not zero and broader session history is out of scope."
        ),
        evidence_path: workflow_state_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("j2-b-b1-real-probe-evidence-ref.json")
            .display()
            .to_string(),
        rollback_plan:
            "J2-B B1 read-only probe authorizes no project write roots; mario test files must remain unchanged."
                .to_string(),
        user_confirmed_real_resume: true,
        global_supervisor_confirmed: true,
    }
}

fn k3_b_resume_authorization(
    workflow_state_path: &Path,
    config: &K3BExecutionPointConfig,
) -> crate::H2RealResumeAuthorizationMatrix {
    crate::H2RealResumeAuthorizationMatrix {
        operation_type: "resume".to_string(),
        test_project: "Stage K K3-B1 mario test workflow read-only probe".to_string(),
        project_root: config.project_root.to_string(),
        target_cwd: config.project_root.to_string(),
        target_session: config
            .target_session_id
            .unwrap_or_default()
            .to_string(),
        prompt_summary: config.prompt_summary.to_string(),
        prompt_sha256: config.prompt_hash.to_string(),
        prompt_ref: config.prompt_ref.to_string(),
        allowed_write_roots: config.allowed_write_roots(),
        codex_home_scope:
            "Codex CLI minimum native session state for one authorized K3-B1 real resume; no credential material requested."
                .to_string(),
        sandbox: config.sandbox.to_string(),
        timeout_ms: Some(120_000),
        readback_plan: format!(
            "Workbench-managed latest message for marker {}; unavailable is not zero and broader session history is out of scope.",
            config.readback_marker
        ),
        evidence_path: workflow_state_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("k3-b1-real-probe-evidence-ref.json")
            .display()
            .to_string(),
        rollback_plan:
            "K3-B1 read-only probe authorizes no project write roots; mario test files must remain unchanged."
                .to_string(),
        user_confirmed_real_resume: true,
        global_supervisor_confirmed: true,
    }
}

fn k3_b_new_session_authorization(
    workflow_state_path: &Path,
    config: &K3BExecutionPointConfig,
) -> crate::H3RealNewSessionAuthorizationMatrix {
    crate::H3RealNewSessionAuthorizationMatrix {
        operation_type: "new_session".to_string(),
        test_project: "Stage K K3-B2 isolated project workspace-write workflow probe".to_string(),
        project_root: config.project_root.to_string(),
        target_cwd: config.project_root.to_string(),
        work_item_id: config.work_item_id.to_string(),
        prompt_summary: config.prompt_summary.to_string(),
        prompt_sha256: config.prompt_hash.to_string(),
        prompt_ref: config.prompt_ref.to_string(),
        allowed_write_roots: config.allowed_write_roots(),
        codex_home_scope:
            "Codex CLI minimum native session state for one authorized K3-B2 real new_session; no credential material requested."
                .to_string(),
        sandbox: config.sandbox.to_string(),
        timeout_ms: Some(120_000),
        readback_plan: format!(
            "Workbench-managed latest message for marker {}; unavailable is not zero and broader session history is out of scope.",
            config.readback_marker
        ),
        evidence_path: workflow_state_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("k3-b2-real-probe-evidence-ref.json")
            .display()
            .to_string(),
        rollback_plan: format!(
            "K3-B2 only accepts writes under {}; baseline manifest must remain unchanged outside the allowed path.",
            config.allowed_write_path.unwrap_or(K3_B_B2_ALLOWED_WRITE_PATH)
        ),
        user_confirmed_real_new_session: true,
        global_supervisor_confirmed: true,
    }
}

fn j2_b_b2_authorization(workflow_state_path: &Path) -> crate::H3RealNewSessionAuthorizationMatrix {
    crate::H3RealNewSessionAuthorizationMatrix {
        operation_type: "new_session".to_string(),
        test_project: "stage-j-j2-b isolated project workspace-write probe".to_string(),
        project_root: J2_B_B2_PROJECT_ROOT.to_string(),
        target_cwd: J2_B_B2_PROJECT_ROOT.to_string(),
        work_item_id: format!(
            "work-item:j2:{}:{}",
            stable_id(J2_B_B2_WORKFLOW_ID),
            stable_id(J2_B_B2_PROMPT_SUMMARY)
        ),
        prompt_summary: J2_B_B2_PROMPT_SUMMARY.to_string(),
        prompt_sha256: J2_B_B2_PROMPT_HASH.to_string(),
        prompt_ref: J2_B_B2_PROMPT_REF.to_string(),
        allowed_write_roots: vec![J2_B_B2_ALLOWED_WRITE_ROOT.to_string()],
        codex_home_scope:
            "Codex CLI minimum native session state for one authorized J2-B B2 real new_session; no credential material requested."
                .to_string(),
        sandbox: J2_B_B2_SANDBOX.to_string(),
        timeout_ms: Some(120_000),
        readback_plan: format!(
            "Workbench-managed latest message for marker {J2_B_B2_READBACK_MARKER}; unavailable is not zero and broader session history is out of scope."
        ),
        evidence_path: workflow_state_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("j2-b-b2-real-probe-evidence-ref.json")
            .display()
            .to_string(),
        rollback_plan: format!(
            "J2-B B2 only accepts writes under {J2_B_B2_ALLOWED_WRITE_PATH}; baseline README/project-notes hashes must remain unchanged."
        ),
        user_confirmed_real_new_session: true,
        global_supervisor_confirmed: true,
    }
}

fn apply_j2_b_b1_outputs_to_plan(
    plan: &mut ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
) {
    for unit in &mut plan.run_units {
        if unit.run_unit_kind == "developer_execution" {
            unit.status = phase_b.status.clone();
            unit.product_command_ref = Some(phase_b.product_command_id.clone());
            unit.runtime_log_refs = [
                phase_a.runtime_log_ref.clone(),
                phase_b.runtime_log_ref.clone(),
            ]
            .into_iter()
            .flatten()
            .collect();
            unit.audit_refs = crate::dedupe_strings(
                phase_a
                    .audit_refs
                    .iter()
                    .chain(phase_b.audit_refs.iter())
                    .cloned()
                    .collect(),
            );
            unit.readback_ref = phase_b
                .product_command_attempt
                .as_ref()
                .map(|attempt| format!("readback:{}", attempt.attempt_id));
            unit.readback_status = phase_b.readback_summary.status.clone();
            unit.readback_result_count = phase_b.readback_summary.result_count;
            unit.runner_call_allowed = phase_b.runner_call_allowed;
            unit.prompt_sent = phase_b.prompt_sent;
            unit.real_codex_executed = phase_b.real_codex_executed;
            unit.writes_codex_home = phase_b.writes_codex_home;
            unit.writes_project_files = phase_b.writes_project_files;
            unit.next_step =
                "回收线只读取本次 attempt 摘要/readback refs，不读取 full transcript。".to_string();
            unit.blocked_reasons.extend(phase_b.blocked_reasons.clone());
            unit.warnings.extend(phase_b.warnings.clone());
        } else if phase_b.status == "phase_b_completed" && unit.run_unit_kind == "collector_summary"
        {
            unit.status = "needs_review".to_string();
            unit.product_command_ref = Some(phase_b.product_command_id.clone());
            unit.readback_status = phase_b.readback_summary.status.clone();
            unit.readback_result_count = phase_b.readback_summary.result_count;
            unit.next_step = "等待后续 C5 worker report / process fact 回收路径。".to_string();
        }
    }
    plan.current_phase = "collector_summary".to_string();
    plan.next_step = if phase_b.status == "phase_b_completed" {
        "J2-B B1 Phase B 已记录，等待后续回收路径处理 worker report candidate。".to_string()
    } else {
        "J2-B B1 Phase B 被阻断，检查 blocked_reasons。".to_string()
    };
    plan.blocked_reasons.extend(phase_b.blocked_reasons.clone());
}

fn apply_k3_b_frozen_refs_to_plan(
    plan: &mut ProjectWorkflowAutomationPlan,
    config: &K3BExecutionPointConfig,
) {
    plan.current_phase = "developer_execution".to_string();
    plan.next_step = format!(
        "{} 已进入 K3-B0 bridge；真实执行仍需单独授权和 runtime prompt。",
        config.execution_point_id
    );
    plan.warnings = crate::dedupe_strings(vec![
        "k3_b0_bridge_harness_only".to_string(),
        "prompt_body_runtime_only_not_persisted".to_string(),
        "real_codex_execution_requires_ignored_env_gated_entry".to_string(),
    ]);
    for unit in &mut plan.run_units {
        if unit.run_unit_kind == "developer_execution" {
            unit.run_unit_id = config.run_unit_id.to_string();
            unit.workflow_node_id = config.node_id.to_string();
            unit.work_item_id = config.work_item_id.to_string();
            unit.task_package_ref = Some(config.task_package_ref.to_string());
            unit.memory_packet_ref = Some(config.memory_packet_ref.to_string());
            unit.summary = format!("K3-B developer run unit：{}", config.prompt_summary);
            unit.next_step =
                "等待 Product Command Phase B gate；缺 runtime prompt 时必须阻断。".to_string();
            unit.warnings.push("k3_b_frozen_run_unit_ref".to_string());
        }
    }
}

fn apply_k3_b_outputs_to_plan(
    plan: &mut ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
    config: &K3BExecutionPointConfig,
) {
    for unit in &mut plan.run_units {
        if unit.run_unit_kind == "developer_execution" {
            unit.status = phase_b.status.clone();
            unit.product_command_ref = Some(phase_b.product_command_id.clone());
            unit.runtime_log_refs = [
                phase_a.runtime_log_ref.clone(),
                phase_b.runtime_log_ref.clone(),
            ]
            .into_iter()
            .flatten()
            .collect();
            unit.audit_refs = crate::dedupe_strings(
                phase_a
                    .audit_refs
                    .iter()
                    .chain(phase_b.audit_refs.iter())
                    .cloned()
                    .collect(),
            );
            unit.readback_ref = phase_b
                .product_command_attempt
                .as_ref()
                .map(|attempt| format!("readback:{}", attempt.attempt_id));
            unit.readback_status = phase_b.readback_summary.status.clone();
            unit.readback_result_count = phase_b.readback_summary.result_count;
            unit.runner_call_allowed = phase_b.runner_call_allowed;
            unit.prompt_sent = phase_b.prompt_sent;
            unit.real_codex_executed = phase_b.real_codex_executed;
            unit.writes_codex_home = phase_b.writes_codex_home;
            unit.writes_project_files = phase_b.writes_project_files;
            unit.next_step = if phase_b.status == "phase_b_completed" {
                "等待 worker report / process fact / capture source 回收；不得自动写 FormalMemory。"
                    .to_string()
            } else {
                "Phase B 未执行或被 guard 阻断；检查 blocked_reasons，不能包装为成功。".to_string()
            };
            unit.blocked_reasons.extend(phase_b.blocked_reasons.clone());
            unit.warnings.extend(phase_b.warnings.clone());
            unit.warnings.push(format!(
                "k3_b_permission_envelope_ref:{}",
                config.permission_envelope_ref
            ));
        } else if phase_b.status == "phase_b_completed" && unit.run_unit_kind == "collector_summary"
        {
            unit.status = "needs_review".to_string();
            unit.product_command_ref = Some(phase_b.product_command_id.clone());
            unit.readback_status = phase_b.readback_summary.status.clone();
            unit.readback_result_count = phase_b.readback_summary.result_count;
            unit.next_step =
                "回收线只处理摘要、worker report 和过程事实候选，不读取 full transcript。"
                    .to_string();
        }
    }
    plan.current_phase = if phase_b.status == "phase_b_completed" {
        "collector_summary".to_string()
    } else {
        "blocked".to_string()
    };
    plan.next_step = if phase_b.status == "phase_b_completed" {
        format!(
            "{} Phase B 已记录，等待主管线回收 worker report/process fact/capture refs。",
            config.execution_point_id
        )
    } else {
        format!(
            "{} Phase B 未真实执行或被阻断，不能声明 B1/B2 完成。",
            config.execution_point_id
        )
    };
    plan.blocked_reasons.extend(phase_b.blocked_reasons.clone());
    plan.blocked_reasons = crate::dedupe_strings(plan.blocked_reasons.clone());
}

fn apply_phase_a_to_developer_unit(
    plan: &mut ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
) {
    if let Some(unit) = plan
        .run_units
        .iter_mut()
        .find(|unit| unit.run_unit_kind == "developer_execution")
    {
        unit.status = "readback_unavailable".to_string();
        unit.product_command_ref = Some(phase_a.product_command_id.clone());
        if let Some(runtime_log_ref) = phase_a.runtime_log_ref.clone() {
            unit.runtime_log_refs.push(runtime_log_ref);
        }
        unit.audit_refs.extend(phase_a.audit_refs.clone());
        unit.readback_ref = phase_a
            .product_command_attempt
            .as_ref()
            .map(|attempt| format!("readback:{}", attempt.attempt_id));
        unit.readback_status = phase_a.readback_summary.status.clone();
        unit.readback_result_count = phase_a.readback_summary.result_count;
        unit.runner_call_allowed = phase_a.runner_call_allowed;
        unit.prompt_sent = phase_a.prompt_sent;
        unit.real_codex_executed = phase_a.real_codex_executed;
        unit.writes_codex_home = phase_a.writes_codex_home;
        unit.writes_project_files = phase_a.writes_project_files;
        unit.next_step = "交给验证线检查 readback unknown 和 no-op 边界。".to_string();
        unit.warnings.extend(phase_a.warnings.clone());
    }
}

fn mark_downstream_units(
    plan: &mut ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
) {
    for unit in &mut plan.run_units {
        if unit.run_unit_kind == "verifier_check" {
            unit.status = "completed".to_string();
            unit.product_command_ref = Some(phase_a.product_command_id.clone());
            unit.readback_status = phase_a.readback_summary.status.clone();
            unit.readback_result_count = phase_a.readback_summary.result_count;
            unit.next_step = "readback unknown 已保持为未知，进入回收摘要。".to_string();
        }
        if unit.run_unit_kind == "collector_summary" {
            unit.status = "needs_review".to_string();
            unit.product_command_ref = Some(phase_a.product_command_id.clone());
            unit.readback_status = phase_a.readback_summary.status.clone();
            unit.readback_result_count = phase_a.readback_summary.result_count;
        }
    }
}

fn worker_report_input(
    input: &ProjectWorkflowAutomationInput,
    project_id_value: &str,
    workflow_id: &str,
    node_id: &str,
    work_item_id: &str,
    product_command_id: &str,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    timestamp: &str,
) -> WorkerStructuredReportInput {
    WorkerStructuredReportInput {
        project_root: input.project_root.clone(),
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_node_id: node_id.to_string(),
        work_item_id: work_item_id.to_string(),
        dispatch_id: None,
        actor_role: "developer_execution".to_string(),
        executed_what:
            "记录 K3 Level A Product Command Phase A no-op；未发送 prompt，未执行真实 Codex。"
                .to_string(),
        changed_what: "写入工作台自有 product command / continuation / runtime / audit 边界记录。"
            .to_string(),
        summary: "K3 Level A 开发线回收：Phase A no-op 已记录，readback 结果数未知。".to_string(),
        evidence_refs: vec![
            product_command_id.to_string(),
            phase_a
                .product_command_attempt
                .as_ref()
                .map(|attempt| attempt.attempt_id.clone())
                .unwrap_or_else(|| "phase-a-attempt:unavailable".to_string()),
        ],
        open_issues: vec!["真实执行点需要 K3 Level B 单独授权。".to_string()],
        permission_requests: Vec::new(),
        direction_risks: vec!["readback unavailable 不能解释为 0 条结果或真实完成。".to_string()],
        follow_up_suggestions: vec!["主管线 fresh verify 后再决定是否进入 K3 Level B。".to_string()],
        acceptance_status: "reported_completed".to_string(),
        source_refs: vec![ObservationSourceRef {
            source_ref_id: format!("source:k3-level-a:{}", stable_id(product_command_id)),
            source_kind: "evidence".to_string(),
            source_id: product_command_id.to_string(),
            project_id: Some(project_id_value.to_string()),
            workflow_id: Some(workflow_id.to_string()),
            session_id: phase_a.continuation_id.clone(),
            file_path: None,
            evidence_ref: Some(format!("work-item:{work_item_id}")),
            summary: "K3 Level A Phase A no-op Product Command 证据。".to_string(),
            sensitive_level: "internal".to_string(),
            created_at: timestamp.to_string(),
        }],
        expected_workflow_revision: None,
    }
}

fn process_fact_input(
    input: &ProjectWorkflowAutomationInput,
    project_id_value: &str,
    workflow_id: &str,
    report_id: &str,
    product_command_id: &str,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    timestamp: &str,
) -> ProjectDirectorProcessFactDecisionInput {
    let process_fact_id = format!("process-fact:k3-level-a:{}", stable_id(report_id));
    ProjectDirectorProcessFactDecisionInput {
        project_root: input.project_root.clone(),
        project_id: project_id_value.to_string(),
        workflow_id: workflow_id.to_string(),
        report_id: report_id.to_string(),
        actor_id: "project-director-k3-level-a".to_string(),
        actor_role: "project_director".to_string(),
        decision: "confirm_process_fact".to_string(),
        accepted_facts: vec![ProcessFactCandidate {
            process_fact_id,
            summary:
                "K3 Level A 低风险本项目过程事实：项目自动编排 Phase A no-op 已完成并回收为 worker report。"
                    .to_string(),
            source_report_id: report_id.to_string(),
            source_dispatch_id: None,
            evidence_refs: vec![
                product_command_id.to_string(),
                phase_a
                    .product_command_attempt
                    .as_ref()
                    .map(|attempt| attempt.attempt_id.clone())
                    .unwrap_or_else(|| "phase-a-attempt:unavailable".to_string()),
            ],
            source_refs: vec![ObservationSourceRef {
                source_ref_id: format!("source:k3-level-a-report:{}", stable_id(report_id)),
                source_kind: "worker_report".to_string(),
                source_id: report_id.to_string(),
                project_id: Some(project_id_value.to_string()),
                workflow_id: Some(workflow_id.to_string()),
                session_id: phase_a.continuation_id.clone(),
                file_path: None,
                evidence_ref: Some(product_command_id.to_string()),
                summary: "K3 Level A worker report fixture。".to_string(),
                sensitive_level: "internal".to_string(),
                created_at: timestamp.to_string(),
            }],
            scope: MemoryScope {
                scope_id: format!("scope:k3-level-a:{}", stable_id(report_id)),
                scope_type: "workflow".to_string(),
                user_id: None,
                project_id: Some(project_id_value.to_string()),
                workflow_id: Some(workflow_id.to_string()),
                session_id: None,
                role_ids: vec![
                    "project_director".to_string(),
                    "developer_execution".to_string(),
                    "verifier_check".to_string(),
                ],
                document_refs: Vec::new(),
                permission_policy_ref: None,
                model_export_policy: "local_only".to_string(),
                valid_from: timestamp.to_string(),
                valid_until: None,
            },
            risk_level: "low".to_string(),
            sensitive_level: "internal".to_string(),
            proposed_observation_type: "process_fact".to_string(),
        }],
        rejected_fact_ids: Vec::new(),
        summary: "项目主管确认 K3 Level A 过程事实；只写 observation，不生成正式记忆。".to_string(),
        expected_workflow_revision: None,
        expected_observation_store_revision: None,
    }
}

fn capture_process_fact_event(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationInput,
    plan: &ProjectWorkflowAutomationPlan,
    product_command_id: &str,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    report_id: &str,
    process_fact: &crate::ProjectDirectorProcessFactDecisionResult,
    timestamp: &str,
    write_id: &str,
) -> Result<crate::CaptureMemoryEventOutput, String> {
    let collector_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "collector_summary")
        .or_else(|| {
            plan.run_units
                .iter()
                .find(|unit| unit.run_unit_kind == "developer_execution")
        });
    let product_attempt_id = phase_a
        .product_command_attempt
        .as_ref()
        .map(|attempt| attempt.attempt_id.clone());
    let readback_ref = product_attempt_id
        .as_ref()
        .map(|attempt_id| format!("readback:{attempt_id}"));
    let mut audit_refs = phase_a.audit_refs.clone();
    audit_refs.push(report_id.to_string());
    audit_refs.push(process_fact.audit_event_id.clone());
    let audit_refs = crate::dedupe_strings(audit_refs);
    let source_ref_id = format!(
        "source:k3-level-a-process-fact:{}",
        stable_id(&process_fact.decision_record_id)
    );
    let capture_input = CaptureMemoryEventInput {
        project_root: input.project_root.clone(),
        project_id: Some(plan.project_id.clone()),
        workflow_id: Some(plan.workflow_id.clone()),
        workflow_node_id: collector_unit.map(|unit| unit.workflow_node_id.clone()),
        run_unit_id: collector_unit.map(|unit| unit.run_unit_id.clone()),
        product_command_id: Some(product_command_id.to_string()),
        product_attempt_id: product_attempt_id.clone(),
        runtime_log_ref: phase_a.runtime_log_ref.clone(),
        audit_refs: audit_refs.clone(),
        readback_ref: readback_ref.clone(),
        task_package_ref: collector_unit.and_then(|unit| unit.task_package_ref.clone()),
        memory_packet_ref: collector_unit.and_then(|unit| unit.memory_packet_ref.clone()),
        scope: MemoryScope {
            scope_id: format!(
                "scope:k3-level-a:{}",
                stable_id(&process_fact.decision_record_id)
            ),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(plan.project_id.clone()),
            workflow_id: Some(plan.workflow_id.clone()),
            session_id: phase_a.continuation_id.clone(),
            role_ids: vec![
                "project_director".to_string(),
                "developer_execution".to_string(),
                "verifier_check".to_string(),
                "collector_summary".to_string(),
            ],
            document_refs: Vec::new(),
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: timestamp.to_string(),
            valid_until: None,
        },
        source_type: "process_fact_decision".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id,
            source_type: "process_fact_decision".to_string(),
            source_id: process_fact.decision_record_id.clone(),
            project_id: Some(plan.project_id.clone()),
            workflow_id: Some(plan.workflow_id.clone()),
            workflow_node_id: collector_unit.map(|unit| unit.workflow_node_id.clone()),
            run_unit_id: collector_unit.map(|unit| unit.run_unit_id.clone()),
            product_command_id: Some(product_command_id.to_string()),
            product_attempt_id,
            runtime_log_ref: phase_a.runtime_log_ref.clone(),
            audit_ref_id: Some(process_fact.audit_event_id.clone()),
            readback_ref,
            task_package_ref: collector_unit.and_then(|unit| unit.task_package_ref.clone()),
            memory_packet_ref: collector_unit.and_then(|unit| unit.memory_packet_ref.clone()),
            evidence_ref: Some(report_id.to_string()),
            summary: "K3 Level A 过程事实确认和 worker report 回收来源。".to_string(),
            sensitive_level: "internal".to_string(),
            created_at: timestamp.to_string(),
        }],
        summary: "K3 Level A 项目自动编排过程事实已形成 capture source；不是正式记忆。".to_string(),
        evidence_summary:
            "来源包含 Product Command Phase A no-op、worker report 和项目主管过程事实确认。"
                .to_string(),
        sensitivity: "internal".to_string(),
        candidate_policy: "audit_only".to_string(),
        generated_by_role: "project_director".to_string(),
        actor_id: "project-director-k3-level-a".to_string(),
        risk_level: "low".to_string(),
        reason: "K3 Level A 只记录 capture source，不生成候选或 FormalMemory。".to_string(),
        candidate: None,
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    };
    let capture_write_id = format!("{write_id}-k3-memory-capture");
    let observation_write_id = format!("{write_id}-k3-memory-capture-observation");
    let candidate_write_id = format!("{write_id}-k3-memory-capture-candidate");
    memory_capture_bus::capture_event(
        workflow_state_path,
        &capture_input,
        timestamp,
        &capture_write_id,
        &observation_write_id,
        &candidate_write_id,
    )
}

fn apply_worker_report_to_plan(plan: &mut ProjectWorkflowAutomationPlan, report_id: &str) {
    for kind in ["developer_execution", "collector_summary"] {
        if let Some(unit) = plan
            .run_units
            .iter_mut()
            .find(|unit| unit.run_unit_kind == kind)
        {
            unit.worker_report_ref = Some(report_id.to_string());
            unit.audit_refs.push(report_id.to_string());
        }
    }
}

fn apply_capture_event_to_plan(
    plan: &mut ProjectWorkflowAutomationPlan,
    output: &crate::CaptureMemoryEventOutput,
) {
    let capture_event_id = output.capture_event.capture_event_id.clone();
    let observation_ref = output
        .observation
        .as_ref()
        .map(|observation| observation.observation_id.clone());
    let candidate_ref = output
        .candidate
        .as_ref()
        .map(|candidate| candidate.candidate_key.clone());
    for unit in &mut plan.run_units {
        unit.capture_event_refs.push(capture_event_id.clone());
        if let Some(observation_ref) = observation_ref.clone() {
            unit.observation_refs.push(observation_ref);
        }
        if let Some(candidate_ref) = candidate_ref.clone() {
            unit.memory_candidate_refs.push(candidate_ref);
        }
        unit.warnings.extend(output.warnings.clone());
        unit.capture_event_refs = crate::dedupe_strings(unit.capture_event_refs.clone());
        unit.observation_refs = crate::dedupe_strings(unit.observation_refs.clone());
        unit.memory_candidate_refs = crate::dedupe_strings(unit.memory_candidate_refs.clone());
        unit.warnings = crate::dedupe_strings(unit.warnings.clone());
    }
    plan.warnings.extend(output.warnings.clone());
    plan.warnings = crate::dedupe_strings(plan.warnings.clone());
}

fn apply_process_fact_to_plan(
    plan: &mut ProjectWorkflowAutomationPlan,
    result: &crate::ProjectDirectorProcessFactDecisionResult,
) {
    let observation_refs = result
        .observations
        .iter()
        .map(|observation| observation.observation_id.clone())
        .collect::<Vec<_>>();
    for unit in &mut plan.run_units {
        if unit.run_unit_kind == "collector_summary" {
            unit.status = "completed".to_string();
            unit.observation_refs.extend(observation_refs.clone());
            unit.audit_refs.push(result.audit_event_id.clone());
            unit.next_step = "过程事实已进入 observation；等待主管最终复核。".to_string();
        }
        if unit.run_unit_kind == "director_final_review" {
            unit.status = "needs_review".to_string();
            unit.observation_refs.extend(observation_refs.clone());
            unit.next_step =
                "主管线复核 K3 Level A 证据后决定是否接受或进入 K3 Level B。".to_string();
        }
    }
}

fn append_automation_audit_event(
    workflow_state_path: &Path,
    input: &ProjectWorkflowAutomationInput,
    plan: &ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    report_id: &str,
    process_fact: &crate::ProjectDirectorProcessFactDecisionResult,
    timestamp: &str,
) -> Result<String, String> {
    let mut value = read_workflow_state_value(workflow_state_path)?;
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    let audit_event_id = format!(
        "audit:k3-project-workflow-automation:{}:{timestamp}",
        stable_id(&plan.automation_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": J2_EVENT_TYPE,
      "target_ref": plan.automation_id,
      "project_id": plan.project_id,
      "workflow_id": plan.workflow_id,
      "work_item_id": plan.run_units.first().map(|unit| unit.work_item_id.clone()),
      "actor_ref": input.requested_by.as_deref().unwrap_or("user"),
      "source_kind": J2_SOURCE_KIND,
      "permission_level": "workflow_event_record",
      "result_status": "phase_a_closed_loop_recorded",
      "product_command_id": phase_a.product_command_id,
      "worker_report_ref": report_id,
      "capture_event_refs": crate::dedupe_strings(plan.run_units.iter().flat_map(|unit| unit.capture_event_refs.clone()).collect::<Vec<_>>()),
      "observation_refs": process_fact.observations.iter().map(|observation| observation.observation_id.clone()).collect::<Vec<_>>(),
      "plan": plan,
      "prompt_sent": false,
      "real_codex_executed": false,
      "writes_codex_home": false,
      "writes_project_files": false,
      "created_at": timestamp,
      "reason": "K3 Level A 记录项目自动编排非真实闭环；真实执行点留到 K3 Level B 单独授权。",
      "warnings": [
        "k3_level_a_phase_a_noop_only",
        "readback_unavailable_is_not_zero_results",
        "observation_is_not_formal_memory",
        "memory_capture_source_is_not_formal_memory"
      ],
      "backup_ref": backup.display().to_string()
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, &value)?;
    Ok(audit_event_id)
}

fn append_j2_b_b1_audit_event(
    workflow_state_path: &Path,
    plan: &ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
    timestamp: &str,
) -> Result<String, String> {
    let mut value = read_workflow_state_value(workflow_state_path)?;
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution");
    let audit_event_id = format!(
        "audit:j2-b-b1-project-workflow-automation:{}:{timestamp}",
        stable_id(&phase_b.product_command_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": J2_B_B1_EVENT_TYPE,
      "target_ref": plan.automation_id,
      "project_id": J2_B_B1_PROJECT_ID,
      "workflow_id": J2_B_B1_WORKFLOW_ID,
      "workflow_node_id": J2_B_B1_NODE_ID,
      "work_item_id": developer_unit.map(|unit| unit.work_item_id.clone()),
      "run_unit_id": developer_unit.map(|unit| unit.run_unit_id.clone()),
      "actor_ref": "user",
      "source_kind": "stage_j_j2_b_b1_project_workflow_automation_execution_bridge",
      "permission_level": "real_execution_product_command",
      "result_status": phase_b.status,
      "product_command_id": phase_b.product_command_id,
      "phase_a_attempt_ref": phase_a.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "phase_b_attempt_ref": phase_b.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "continuation_id": phase_b.continuation_id,
      "runtime_log_ref": phase_b.runtime_log_ref,
      "readback_status": phase_b.readback_summary.status,
      "readback_result_count": phase_b.readback_summary.result_count,
      "task_package_ref": developer_unit.and_then(|unit| unit.task_package_ref.clone()),
      "memory_packet_ref": developer_unit.and_then(|unit| unit.memory_packet_ref.clone()),
      "prompt_summary": J2_B_B1_PROMPT_SUMMARY,
      "prompt_ref": J2_B_B1_PROMPT_REF,
      "prompt_hash": J2_B_B1_PROMPT_HASH,
      "prompt_body_persisted": false,
      "allowed_write_roots": [],
      "denied_paths": sensitive_denied_paths(),
      "sandbox": J2_B_B1_SANDBOX,
      "created_at": timestamp,
      "warnings": [
        "j2_b_b1_bridge_uses_unified_product_command_phase_b",
        "prompt_body_runtime_only_not_persisted",
        "readback_full_transcript_forbidden"
      ],
      "backup_ref": backup.display().to_string()
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, &value)?;
    Ok(audit_event_id)
}

fn append_j2_b_b2_audit_event(
    workflow_state_path: &Path,
    plan: &ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
    timestamp: &str,
) -> Result<String, String> {
    let mut value = read_workflow_state_value(workflow_state_path)?;
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution");
    let audit_event_id = format!(
        "audit:j2-b-b2-project-workflow-automation:{}:{timestamp}",
        stable_id(&phase_b.product_command_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": J2_B_B2_EVENT_TYPE,
      "target_ref": plan.automation_id,
      "project_id": J2_B_B2_PROJECT_ID,
      "workflow_id": J2_B_B2_WORKFLOW_ID,
      "workflow_node_id": J2_B_B2_NODE_ID,
      "work_item_id": developer_unit.map(|unit| unit.work_item_id.clone()),
      "run_unit_id": developer_unit.map(|unit| unit.run_unit_id.clone()),
      "actor_ref": "user",
      "source_kind": "stage_j_j2_b_b2_project_workflow_automation_new_session_bridge",
      "permission_level": "real_execution_product_command",
      "operation_id": "new_session",
      "result_status": phase_b.status,
      "product_command_id": phase_b.product_command_id,
      "phase_a_attempt_ref": phase_a.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "phase_b_attempt_ref": phase_b.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "continuation_id": phase_b.continuation_id,
      "runtime_log_ref": phase_b.runtime_log_ref,
      "readback_status": phase_b.readback_summary.status,
      "readback_result_count": phase_b.readback_summary.result_count,
      "task_package_ref": developer_unit.and_then(|unit| unit.task_package_ref.clone()),
      "memory_packet_ref": developer_unit.and_then(|unit| unit.memory_packet_ref.clone()),
      "prompt_summary": J2_B_B2_PROMPT_SUMMARY,
      "prompt_ref": J2_B_B2_PROMPT_REF,
      "prompt_hash": J2_B_B2_PROMPT_HASH,
      "prompt_body_persisted": false,
      "allowed_write_roots": [J2_B_B2_ALLOWED_WRITE_ROOT],
      "allowed_write_path": J2_B_B2_ALLOWED_WRITE_PATH,
      "denied_paths": sensitive_denied_paths(),
      "sandbox": J2_B_B2_SANDBOX,
      "created_at": timestamp,
      "warnings": [
        "j2_b_b2_bridge_uses_unified_product_command_new_session_phase_b",
        "prompt_body_runtime_only_not_persisted",
        "readback_full_transcript_forbidden",
        "baseline_hashes_must_verify_only_allowed_write_path_changed"
      ],
      "backup_ref": backup.display().to_string()
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, &value)?;
    Ok(audit_event_id)
}

fn append_k3_b_audit_event(
    workflow_state_path: &Path,
    plan: &ProjectWorkflowAutomationPlan,
    phase_a: &crate::RealExecutionProductCommandPhaseAOutput,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
    config: &K3BExecutionPointConfig,
    timestamp: &str,
) -> Result<String, String> {
    let mut value = read_workflow_state_value(workflow_state_path)?;
    let backup = backup_workflow_state_file(workflow_state_path, timestamp)?;
    let developer_unit = plan
        .run_units
        .iter()
        .find(|unit| unit.run_unit_kind == "developer_execution");
    let audit_event_id = format!(
        "audit:k3-b-project-workflow-automation:{}:{timestamp}",
        stable_id(&phase_b.product_command_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": config.event_type,
      "target_ref": plan.automation_id,
      "execution_point_id": config.execution_point_id,
      "project_id": config.project_id,
      "workflow_id": config.workflow_id,
      "workflow_node_id": config.node_id,
      "work_item_id": config.work_item_id,
      "run_unit_id": config.run_unit_id,
      "actor_ref": "user",
      "source_kind": "stage_k_k3_b_real_workflow_execution_bridge",
      "permission_level": "real_execution_product_command",
      "operation_id": config.operation_id,
      "result_status": phase_b.status,
      "product_command_id": phase_b.product_command_id,
      "phase_a_attempt_ref": phase_a.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "phase_b_attempt_ref": phase_b.product_command_attempt.as_ref().map(|attempt| attempt.attempt_id.clone()),
      "continuation_id": phase_b.continuation_id,
      "runtime_log_ref": phase_b.runtime_log_ref,
      "readback_status": phase_b.readback_summary.status,
      "readback_result_count": phase_b.readback_summary.result_count,
      "task_package_ref": developer_unit.and_then(|unit| unit.task_package_ref.clone()).unwrap_or_else(|| config.task_package_ref.to_string()),
      "task_memory_packet_ref": config.memory_packet_ref,
      "permission_envelope_ref": config.permission_envelope_ref,
      "readback_marker": config.readback_marker,
      "prompt_summary": config.prompt_summary,
      "prompt_ref": config.prompt_ref,
      "prompt_hash": config.prompt_hash,
      "prompt_body_persisted": false,
      "allowed_write_roots": config.allowed_write_roots(),
      "allowed_write_path": config.allowed_write_path,
      "baseline_refs": config.baseline_refs(),
      "manifest_requirements": config.manifest_requirements(),
      "denied_paths": sensitive_denied_paths(),
      "sandbox": config.sandbox,
      "created_at": timestamp,
      "warnings": [
        "k3_b_bridge_uses_unified_product_command_phase_b",
        "prompt_body_runtime_only_not_persisted",
        "readback_full_transcript_forbidden",
        "readback_unavailable_is_not_zero_results",
        "worker_report_capture_followup_not_formal_memory"
      ],
      "backup_ref": backup.display().to_string()
    }));
    value["updated_at"] = Value::String(timestamp.to_string());
    write_validated_workflow_state(workflow_state_path, &value)?;
    Ok(audit_event_id)
}

fn k3_b_output_warnings(
    config: &K3BExecutionPointConfig,
    has_runtime_prompt: bool,
    phase_b: &crate::RealExecutionProductCommandPhaseBOutput,
) -> Vec<String> {
    let mut warnings = vec![
        "k3_b_bridge_used_real_execution_product_command_family".to_string(),
        "prompt_body_runtime_only_not_persisted".to_string(),
        "worker_report_process_fact_capture_followup_only".to_string(),
    ];
    if !has_runtime_prompt {
        warnings.push("k3_b0_no_runtime_prompt_body_runner_not_called".to_string());
    }
    if config.execution_point_id == K3_B_B2_EXECUTION_POINT_ID {
        warnings
            .push("k3_b2_new_session_product_command_phase_b_path_available_env_gated".to_string());
        warnings.push("workspace_write_requires_manifest_diff_not_sandbox_flag".to_string());
    }
    warnings.extend(phase_b.warnings.clone());
    crate::dedupe_strings(warnings)
}

fn read_model_from_plan(
    generated_at: &str,
    status: Option<String>,
    plan: Option<ProjectWorkflowAutomationPlan>,
    warnings: Vec<String>,
) -> ProjectWorkflowAutomationReadModel {
    let run_units = plan
        .as_ref()
        .map(|plan| plan.run_units.clone())
        .unwrap_or_default();
    ProjectWorkflowAutomationReadModel {
        schema_version: J2_SCHEMA.to_string(),
        available: plan.is_some(),
        generated_at: generated_at.to_string(),
        latest_automation_id: plan.as_ref().map(|plan| plan.automation_id.clone()),
        latest_status: status,
        latest_plan: plan.clone(),
        run_unit_count: run_units.len(),
        waiting_user_count: run_units
            .iter()
            .filter(|unit| unit.status == "waiting_user")
            .count(),
        blocked_count: run_units
            .iter()
            .filter(|unit| unit.status == "blocked_by_guard")
            .count(),
        readback_unknown_count: run_units
            .iter()
            .filter(|unit| unit.readback_result_count.is_none())
            .count(),
        worker_report_count: run_units
            .iter()
            .filter(|unit| unit.worker_report_ref.is_some())
            .count(),
        capture_event_count: crate::dedupe_strings(
            run_units
                .iter()
                .flat_map(|unit| unit.capture_event_refs.clone())
                .collect::<Vec<_>>(),
        )
        .len(),
        observation_count: run_units
            .iter()
            .map(|unit| unit.observation_refs.len())
            .sum(),
        next_step: plan.as_ref().map(|plan| plan.next_step.clone()),
        warnings: crate::dedupe_strings(warnings),
    }
}

fn compact_goal(goal: &str) -> String {
    let trimmed = goal.trim();
    if trimmed.chars().count() <= 48 {
        trimmed.to_string()
    } else {
        trimmed.chars().take(48).collect::<String>()
    }
}

fn automation_id_for(workflow_id: &str, user_goal: &str) -> String {
    format!(
        "project-workflow-automation:{}:{}",
        stable_id(workflow_id),
        stable_id(user_goal)
    )
}

fn sensitive_denied_paths() -> Vec<String> {
    vec![
        "/Users/yoyi/.codex".to_string(),
        "secret material".to_string(),
        "token material".to_string(),
        ".env files".to_string(),
        "keychain material".to_string(),
        "OAuth material".to_string(),
        format!("{} {}", "provider", "credential"),
        "full transcript material".to_string(),
        "rollout material".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bootstrap_project_workflow_at, formal_memory_store, observation_store, ProjectRecord,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        std::env::temp_dir().join(format!("product-line-j2a-{name}-{nanos}"))
    }

    fn fixture_project(project_root: &str) -> ProjectRecord {
        ProjectRecord {
            project_root: project_root.to_string(),
            name: "J2A Fixture".to_string(),
            active_hint: true,
            thread_count: 0,
            active_thread_count: 0,
            archived_thread_count: 0,
            latest_updated_at_ms: None,
            authority_files: vec![],
            handoff_files: vec![],
            evidence_files: vec![],
            harness_candidates: vec![],
            harness_resources: vec![],
            context_warnings: vec![],
            warnings: vec![],
        }
    }

    fn fixture_input(project_root: &str) -> ProjectWorkflowAutomationInput {
        ProjectWorkflowAutomationInput {
            project_root: project_root.to_string(),
            project_id: None,
            workflow_id: None,
            workflow_node_id: None,
            work_item_id: None,
            user_goal: "把 K3 用户目标转成项目自动编排 Level A 闭环。".to_string(),
            task_package_ref: None,
            memory_packet_ref: None,
            target_session_id: Some("thread-j2a-fixture".to_string()),
            sandbox: Some("read-only".to_string()),
            requested_by: Some("user".to_string()),
            confirmed_by: Some("user".to_string()),
            risk_acknowledgement: Some("确认 K3 Level A 只做 Phase A no-op。".to_string()),
            reason: Some("测试 K3 Level A 非真实闭环。".to_string()),
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    fn b1_input() -> ProjectWorkflowAutomationJ2BB1Input {
        ProjectWorkflowAutomationJ2BB1Input {
            project_root: Some(J2_B_B1_PROJECT_ROOT.to_string()),
            project_id: Some(J2_B_B1_PROJECT_ID.to_string()),
            workflow_id: Some(J2_B_B1_WORKFLOW_ID.to_string()),
            workflow_node_id: Some(J2_B_B1_NODE_ID.to_string()),
            work_item_id: None,
            task_package_ref: Some(
                "task-package:j2-b:b1:mario-test:developer-run-unit:read-only:v1".to_string(),
            ),
            memory_packet_ref: Some(
                "memory-packet:j2-b:b1:mario-test:developer-run-unit:read-only:v1".to_string(),
            ),
            target_session_id: Some(J2_B_B1_SESSION_ID.to_string()),
            sandbox: Some(J2_B_B1_SANDBOX.to_string()),
            prompt_summary: Some(J2_B_B1_PROMPT_SUMMARY.to_string()),
            prompt_ref: Some(J2_B_B1_PROMPT_REF.to_string()),
            prompt_hash: Some(J2_B_B1_PROMPT_HASH.to_string()),
            requested_by: Some("user".to_string()),
            confirmed_by: Some("user".to_string()),
            risk_acknowledgement: Some("J2-B B1 fake-runner test authorization.".to_string()),
            reason: Some("Test J2-B B1 bridge without spawning Codex.".to_string()),
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    fn b2_input() -> ProjectWorkflowAutomationJ2BB2Input {
        ProjectWorkflowAutomationJ2BB2Input {
            project_root: Some(J2_B_B2_PROJECT_ROOT.to_string()),
            project_id: Some(J2_B_B2_PROJECT_ID.to_string()),
            workflow_id: Some(J2_B_B2_WORKFLOW_ID.to_string()),
            workflow_node_id: Some(J2_B_B2_NODE_ID.to_string()),
            work_item_id: None,
            task_package_ref: Some(
                "task-package:j2-b:b2:isolated-project:developer-run-unit:workspace-write:v1"
                    .to_string(),
            ),
            memory_packet_ref: Some(
                "memory-packet:j2-b:b2:isolated-project:developer-run-unit:workspace-write:v1"
                    .to_string(),
            ),
            sandbox: Some(J2_B_B2_SANDBOX.to_string()),
            allowed_write_path: Some(J2_B_B2_ALLOWED_WRITE_PATH.to_string()),
            prompt_summary: Some(J2_B_B2_PROMPT_SUMMARY.to_string()),
            prompt_ref: Some(J2_B_B2_PROMPT_REF.to_string()),
            prompt_hash: Some(J2_B_B2_PROMPT_HASH.to_string()),
            requested_by: Some("user".to_string()),
            confirmed_by: Some("user".to_string()),
            risk_acknowledgement: Some("J2-B B2 fake-runner test authorization.".to_string()),
            reason: Some("Test J2-B B2 bridge without spawning Codex.".to_string()),
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    fn k3_b1_input(runtime_prompt_body: Option<String>) -> ProjectWorkflowAutomationK3BInput {
        ProjectWorkflowAutomationK3BInput {
            execution_point_id: K3_B_B1_EXECUTION_POINT_ID.to_string(),
            project_root: Some(K3_B_B1_PROJECT_ROOT.to_string()),
            project_id: Some(K3_B_B1_PROJECT_ID.to_string()),
            workflow_id: Some(K3_B_B1_WORKFLOW_ID.to_string()),
            workflow_node_id: Some(K3_B_B1_NODE_ID.to_string()),
            run_unit_id: Some(K3_B_B1_RUN_UNIT_ID.to_string()),
            work_item_id: Some(K3_B_B1_WORK_ITEM_ID.to_string()),
            task_package_ref: Some(K3_B_B1_TASK_PACKAGE_REF.to_string()),
            task_memory_packet_ref: Some(K3_B_B1_MEMORY_PACKET_REF.to_string()),
            permission_envelope_ref: Some(K3_B_B1_PERMISSION_ENVELOPE_REF.to_string()),
            readback_marker: Some(K3_B_B1_READBACK_MARKER.to_string()),
            target_session_id: Some(K3_B_B1_SESSION_ID.to_string()),
            sandbox: Some(K3_B_B1_SANDBOX.to_string()),
            allowed_write_path: None,
            prompt_summary: Some(K3_B_B1_PROMPT_SUMMARY.to_string()),
            prompt_ref: Some(K3_B_B1_PROMPT_REF.to_string()),
            prompt_hash: Some(K3_B_B1_PROMPT_HASH.to_string()),
            runtime_prompt_body,
            requested_by: Some("user".to_string()),
            confirmed_by: Some("user".to_string()),
            risk_acknowledgement: Some("K3-B1 fake/no-op harness authorization.".to_string()),
            reason: Some("Test K3-B1 bridge without spawning Codex.".to_string()),
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    fn k3_b2_input(runtime_prompt_body: Option<String>) -> ProjectWorkflowAutomationK3BInput {
        ProjectWorkflowAutomationK3BInput {
            execution_point_id: K3_B_B2_EXECUTION_POINT_ID.to_string(),
            project_root: Some(K3_B_B2_PROJECT_ROOT.to_string()),
            project_id: Some(K3_B_B2_PROJECT_ID.to_string()),
            workflow_id: Some(K3_B_B2_WORKFLOW_ID.to_string()),
            workflow_node_id: Some(K3_B_B2_NODE_ID.to_string()),
            run_unit_id: Some(K3_B_B2_RUN_UNIT_ID.to_string()),
            work_item_id: Some(K3_B_B2_WORK_ITEM_ID.to_string()),
            task_package_ref: Some(K3_B_B2_TASK_PACKAGE_REF.to_string()),
            task_memory_packet_ref: Some(K3_B_B2_MEMORY_PACKET_REF.to_string()),
            permission_envelope_ref: Some(K3_B_B2_PERMISSION_ENVELOPE_REF.to_string()),
            readback_marker: Some(K3_B_B2_READBACK_MARKER.to_string()),
            target_session_id: None,
            sandbox: Some(K3_B_B2_SANDBOX.to_string()),
            allowed_write_path: Some(K3_B_B2_ALLOWED_WRITE_PATH.to_string()),
            prompt_summary: Some(K3_B_B2_PROMPT_SUMMARY.to_string()),
            prompt_ref: Some(K3_B_B2_PROMPT_REF.to_string()),
            prompt_hash: Some(K3_B_B2_PROMPT_HASH.to_string()),
            runtime_prompt_body,
            requested_by: Some("user".to_string()),
            confirmed_by: Some("user".to_string()),
            risk_acknowledgement: Some("K3-B2 fake/no-op harness authorization.".to_string()),
            reason: Some("Test K3-B2 bridge without spawning Codex.".to_string()),
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    struct J2BB1FakePhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for J2BB1FakePhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            prompt_body: &str,
            last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            assert_eq!(prompt_body, J2_B_B1_CANONICAL_PROMPT);
            codex_local_runner::CodexLocalPhaseBProcessResult {
                runner_kind: "j2_b_b1_fake_phase_b_runner".to_string(),
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
                warnings: vec!["j2_b_b1_fake_runner_no_real_process_spawned".to_string()],
            }
        }
    }

    struct J2BB2FakePhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for J2BB2FakePhaseBRunner {
        fn run_phase_b(
            &self,
            request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            prompt_body: &str,
            last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            assert_eq!(request.operation_id, "new_session");
            assert_eq!(request.project_root, J2_B_B2_PROJECT_ROOT);
            assert_eq!(request.sandbox, J2_B_B2_SANDBOX);
            assert_eq!(prompt_body, J2_B_B2_CANONICAL_PROMPT);
            codex_local_runner::CodexLocalPhaseBProcessResult {
                runner_kind: "j2_b_b2_fake_phase_b_runner".to_string(),
                status: "succeeded".to_string(),
                exit_code: Some(0),
                timed_out: false,
                prompt_sent: true,
                real_codex_executed: true,
                writes_codex_home: true,
                writes_project_files: true,
                readback_status: "succeeded".to_string(),
                readback_attempted: true,
                readback_result_count: Some(1),
                last_message_path: Some(last_message_path.display().to_string()),
                failure_code: None,
                failure_message: None,
                retryable: false,
                user_action_required: false,
                warnings: vec!["j2_b_b2_fake_runner_no_real_process_spawned".to_string()],
            }
        }
    }

    struct K3BNoopPhaseBRunner;

    impl codex_local_runner::CodexLocalPhaseBProcessRunner for K3BNoopPhaseBRunner {
        fn run_phase_b(
            &self,
            _request: &crate::CodexLocalExecutionRequest,
            _command_plan: &crate::CodexLocalCommandPlan,
            _prompt_body: &str,
            _last_message_path: &Path,
            _timeout_ms: Option<i64>,
        ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
            panic!("K3-B0 no-op tests must be blocked before the runner is called")
        }
    }

    fn bootstrap_b1_workflow(path: &Path) {
        let project = fixture_project(J2_B_B1_PROJECT_ROOT);
        bootstrap_project_workflow_at(path, &project).expect("B1 workflow should bootstrap");
    }

    fn bootstrap_b2_workflow(path: &Path) {
        let project = fixture_project(J2_B_B2_PROJECT_ROOT);
        bootstrap_project_workflow_at(path, &project).expect("B2 workflow should bootstrap");
        let default_project_id = project_id(J2_B_B2_PROJECT_ROOT);
        let default_workflow_id = default_workflow_id(J2_B_B2_PROJECT_ROOT);
        let text = fs::read_to_string(path).expect("B2 workflow should read");
        let normalized = text
            .replace(&default_project_id, J2_B_B2_PROJECT_ID)
            .replace(&default_workflow_id, J2_B_B2_WORKFLOW_ID);
        fs::write(path, normalized).expect("B2 workflow ids should normalize");
    }

    fn bootstrap_k3_b1_workflow(path: &Path) {
        let project = fixture_project(K3_B_B1_PROJECT_ROOT);
        bootstrap_project_workflow_at(path, &project).expect("K3-B1 workflow should bootstrap");
    }

    fn bootstrap_k3_b2_workflow(path: &Path) {
        let project = fixture_project(K3_B_B2_PROJECT_ROOT);
        bootstrap_project_workflow_at(path, &project).expect("K3-B2 workflow should bootstrap");
        let default_workflow = default_workflow_id(K3_B_B2_PROJECT_ROOT);
        let text = fs::read_to_string(path).expect("K3-B2 workflow should read");
        let mut value: Value = serde_json::from_str(&text).expect("K3-B2 workflow json");
        let normalized = serde_json::to_string_pretty(&value)
            .expect("json serialize")
            .replace(&default_workflow, K3_B_B2_WORKFLOW_ID);
        value = serde_json::from_str(&normalized).expect("K3-B2 normalized json");
        array_mut(&mut value, "nodes")
            .expect("nodes array")
            .push(json!({
              "node_id": K3_B_B2_NODE_ID,
              "workflow_id": K3_B_B2_WORKFLOW_ID,
              "node_type": "dev_line",
              "title": "K3-B2 开发线",
              "state": "ready"
            }));
        fs::write(
            path,
            serde_json::to_string_pretty(&value).expect("K3-B2 normalized write"),
        )
        .expect("K3-B2 normalized workflow ids should write");
    }

    fn sha256_file(path: &Path) -> String {
        let bytes = fs::read(path).expect("hash target should read");
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

    fn isolated_core_file_hashes(project_root: &Path) -> Vec<(String, String)> {
        ["README.md", "project-notes.md"]
            .iter()
            .map(|file| {
                let path = project_root.join(file);
                ((*file).to_string(), sha256_file(&path))
            })
            .collect()
    }

    fn isolated_project_file_manifest(
        project_root: &Path,
        allowed_write_path: &Path,
    ) -> BTreeMap<String, String> {
        let mut manifest = BTreeMap::new();
        collect_project_file_hashes(
            project_root,
            project_root,
            allowed_write_path,
            &mut manifest,
        );
        manifest
    }

    fn collect_project_file_hashes(
        project_root: &Path,
        current: &Path,
        allowed_write_path: &Path,
        manifest: &mut BTreeMap<String, String>,
    ) {
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        let mut paths = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let Ok(file_type) = fs::symlink_metadata(&path).map(|metadata| metadata.file_type())
            else {
                continue;
            };
            if file_type.is_dir() {
                collect_project_file_hashes(project_root, &path, allowed_write_path, manifest);
            } else if file_type.is_file() && path != allowed_write_path {
                let relative = path
                    .strip_prefix(project_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                manifest.insert(relative, sha256_file(&path));
            }
        }
    }

    fn assert_path_does_not_persist_prompt(path: &Path, prompt_body: &str) {
        let text = fs::read_to_string(path).unwrap_or_default();
        assert!(
            !text.contains(prompt_body),
            "prompt body must not be persisted in {}",
            path.display()
        );
    }

    #[test]
    fn k3_level_a_generates_five_run_units_and_records_no_real_phase_a_closed_loop() {
        let dir = temp_path("closed-loop");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/j2a-closed-loop");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should bootstrap");

        let result = run_project_workflow_automation_phase_a_at(
            &path,
            &fixture_input(&project.project_root),
            "2026-06-09T00:00:00Z",
            "write-j2a-test",
        )
        .expect("J2-A should record closed loop");

        assert_eq!(result.status, "phase_a_closed_loop_recorded");
        assert_eq!(result.plan.run_units.len(), 5);
        for expected in [
            "director_plan",
            "developer_execution",
            "verifier_check",
            "collector_summary",
            "director_final_review",
        ] {
            assert!(result
                .plan
                .run_units
                .iter()
                .any(|unit| unit.run_unit_kind == expected));
        }
        let developer = result
            .plan
            .run_units
            .iter()
            .find(|unit| unit.run_unit_kind == "developer_execution")
            .expect("developer run unit should exist");
        assert!(developer.product_command_ref.is_some());
        assert!(developer.worker_report_ref.is_some());
        assert!(!developer.capture_event_refs.is_empty());
        assert_eq!(developer.readback_result_count, None);
        assert!(!result.prompt_sent);
        assert!(!result.real_codex_executed);
        assert!(!result.writes_codex_home);
        assert!(!result.writes_project_files);
        let phase_a = result.phase_a_output.as_ref().expect("phase A output");
        assert!(!phase_a.prompt_sent);
        assert!(!phase_a.real_codex_executed);
        assert!(!phase_a.writes_codex_home);
        assert!(!phase_a.writes_project_files);
        assert_eq!(phase_a.readback_summary.result_count, None);
        assert_eq!(
            result
                .worker_report_result
                .as_ref()
                .unwrap()
                .first_initialize,
            false
        );
        assert_eq!(
            result
                .process_fact_result
                .as_ref()
                .expect("process fact output")
                .observations
                .len(),
            1
        );
        assert_eq!(result.read_model.run_unit_count, 5);
        assert_eq!(result.read_model.capture_event_count, 1);
        assert!(result.read_model.observation_count >= 1);
        let capture_store = memory_capture_bus::load_store(&path, "2026-06-09T00:00:00Z")
            .expect("capture store should load");
        assert_eq!(capture_store.events.len(), 1);
        assert_eq!(capture_store.events[0].candidate_policy, "audit_only");
        assert_eq!(capture_store.events[0].candidate_key, None);
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal sidecar path")
                .exists(),
            "K3 Level A must not generate formal memory"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2a_rejects_non_user_confirmation_before_writing_sidecars() {
        let dir = temp_path("non-user");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/j2a-non-user");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should bootstrap");
        let mut input = fixture_input(&project.project_root);
        input.confirmed_by = Some("project_director".to_string());

        let err = run_project_workflow_automation_phase_a_at(
            &path,
            &input,
            "2026-06-09T00:00:00Z",
            "write-j2a-non-user",
        )
        .expect_err("non-user confirmation should reject");

        assert!(err.contains("confirmed_by=user"), "{err}");
        assert!(
            !observation_store::sidecar_path(&path)
                .expect("observation sidecar path")
                .exists(),
            "rejected input must not write observation"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2a_duplicate_closed_loop_is_blocked_without_second_phase_a() {
        let dir = temp_path("duplicate");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/j2a-duplicate");
        let input = fixture_input(&project.project_root);
        bootstrap_project_workflow_at(&path, &project).expect("workflow should bootstrap");
        run_project_workflow_automation_phase_a_at(
            &path,
            &input,
            "2026-06-09T00:00:00Z",
            "write-j2a-duplicate-first",
        )
        .expect("first J2-A should write");

        let duplicate = run_project_workflow_automation_phase_a_at(
            &path,
            &input,
            "2026-06-09T00:01:00Z",
            "write-j2a-duplicate-second",
        )
        .expect("duplicate should return blocked output");

        assert_eq!(duplicate.status, "blocked");
        assert!(duplicate.phase_a_output.is_none());
        assert!(duplicate
            .blocked_reasons
            .contains(&"duplicate_project_workflow_automation_closed_loop".to_string()));
        assert!(!duplicate.prompt_sent);
        assert!(!duplicate.real_codex_executed);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2a_missing_workflow_returns_blocked_plan_without_prompt_or_real_execution() {
        let dir = temp_path("missing-workflow");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
              "schema_version": "workflow_state_v0",
              "workflow_version": 1,
              "workspace_id": "workspace:test",
              "updated_at": "2026-06-09T00:00:00Z",
              "projects": [],
              "agent_adapters": [],
              "workflows": [],
              "nodes": [],
              "edges": [],
              "work_items": [],
              "artifacts": [],
              "reviews": [],
              "workflow_node_session_bindings": [],
              "workflow_node_dispatches": [],
              "audit_events": [],
              "capabilities": [],
              "harness_resources": []
            }))
            .expect("json should serialize"),
        )
        .expect("fixture should write");

        let result = run_project_workflow_automation_phase_a_at(
            &path,
            &fixture_input("/tmp/j2a-missing-workflow"),
            "2026-06-09T00:00:00Z",
            "write-j2a-missing-workflow",
        )
        .expect("missing workflow should return blocked result");

        assert_eq!(result.status, "blocked");
        assert_eq!(result.plan.run_units.len(), 5);
        assert!(result
            .plan
            .run_units
            .iter()
            .all(|unit| unit.status == "blocked_by_guard"));
        assert!(!result.prompt_sent);
        assert!(!result.real_codex_executed);
        assert!(!result.writes_codex_home);
        assert!(!result.writes_project_files);
        assert_eq!(result.read_model.blocked_count, 5);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b1_bridge_records_product_command_phase_b_with_fake_runner() {
        let dir = temp_path("j2-b-b1-success");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b1_workflow(&path);
        let last_message_path = dir.join("j2-b-b1-last-message.json");

        let result = run_project_workflow_automation_j2_b_b1_with_runner(
            &path,
            &b1_input(),
            "2026-06-09T02:00:00Z",
            "write-j2-b-b1-success",
            &last_message_path,
            &J2BB1FakePhaseBRunner,
        )
        .expect("B1 bridge should complete with fake runner");

        assert_eq!(result.status, "phase_b_completed");
        assert_eq!(
            result.preview.request.command_family,
            "real_execution_product_command"
        );
        assert_eq!(
            result.preview.request.project_root.as_deref(),
            Some(J2_B_B1_PROJECT_ROOT)
        );
        assert_eq!(
            result.preview.request.project_id.as_deref(),
            Some(J2_B_B1_PROJECT_ID)
        );
        assert_eq!(
            result.preview.request.workflow_id.as_deref(),
            Some(J2_B_B1_WORKFLOW_ID)
        );
        assert_eq!(
            result.preview.request.node_id.as_deref(),
            Some(J2_B_B1_NODE_ID)
        );
        assert_eq!(result.preview.request.prompt_hash, J2_B_B1_PROMPT_HASH);
        assert_eq!(
            result.preview.request.allowed_write_roots,
            Vec::<String>::new()
        );
        assert!(result.allowed_project_write_roots.is_empty());
        assert!(result.prompt_body_persisted == false);
        assert!(result.phase_b_output.runner_call_allowed);
        assert!(result.phase_b_output.prompt_sent);
        assert!(result.phase_b_output.real_codex_executed);
        assert!(result.phase_b_output.writes_codex_home);
        assert!(!result.phase_b_output.writes_project_files);
        assert_eq!(result.phase_b_output.readback_summary.result_count, Some(1));
        assert!(result.phase_a_output.product_command_attempt.is_some());
        assert!(result.phase_b_output.product_command_attempt.is_some());
        assert!(result.phase_b_output.continuation_attempt_id.is_some());
        assert!(!result.runtime_log_refs.is_empty());
        assert!(!result.audit_refs.is_empty());
        assert!(result.readback_ref.is_some());
        let developer = result
            .plan
            .run_units
            .iter()
            .find(|unit| unit.run_unit_kind == "developer_execution")
            .expect("developer run unit");
        assert_eq!(
            developer.product_command_ref.as_deref(),
            Some(result.product_command_id.as_str())
        );
        assert_eq!(developer.readback_result_count, Some(1));
        assert!(developer.runtime_log_refs.len() >= 2);
        assert!(developer
            .audit_refs
            .iter()
            .any(|audit| audit.contains("j2-b-b1")));

        let sidecar = real_execution_command::real_execution_product_command_sidecar_path(&path)
            .expect("product command sidecar path");
        let sidecar_text = fs::read_to_string(sidecar).expect("sidecar should read");
        assert!(sidecar_text.contains(J2_B_B1_PROMPT_HASH));
        assert!(!sidecar_text.contains(J2_B_B1_CANONICAL_PROMPT));
        assert!(!sidecar_text.contains("You are the codex-local developer run unit"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b2_bridge_records_product_command_new_session_phase_b_with_fake_runner() {
        let dir = temp_path("j2-b-b2-success");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b2_workflow(&path);
        let last_message_path = dir.join("j2-b-b2-last-message.json");

        let result = run_project_workflow_automation_j2_b_b2_with_runner(
            &path,
            &b2_input(),
            "2026-06-09T03:00:00Z",
            "write-j2-b-b2-success",
            &last_message_path,
            &J2BB2FakePhaseBRunner,
        )
        .expect("B2 bridge should complete with fake runner");

        assert_eq!(result.status, "phase_b_completed");
        assert_eq!(
            result.preview.request.command_family,
            "real_execution_product_command"
        );
        assert_eq!(result.preview.request.operation_id, "new_session");
        assert_eq!(result.preview.request.target_session_id, None);
        assert_eq!(
            result.preview.request.project_root.as_deref(),
            Some(J2_B_B2_PROJECT_ROOT)
        );
        assert_eq!(
            result.preview.request.project_id.as_deref(),
            Some(J2_B_B2_PROJECT_ID)
        );
        assert_eq!(
            result.preview.request.workflow_id.as_deref(),
            Some(J2_B_B2_WORKFLOW_ID)
        );
        assert_eq!(
            result.preview.request.node_id.as_deref(),
            Some(J2_B_B2_NODE_ID)
        );
        assert_eq!(result.preview.request.prompt_hash, J2_B_B2_PROMPT_HASH);
        assert_eq!(
            result.preview.request.allowed_write_roots,
            vec![J2_B_B2_ALLOWED_WRITE_ROOT.to_string()]
        );
        assert_eq!(
            result.allowed_project_write_path,
            J2_B_B2_ALLOWED_WRITE_PATH
        );
        assert!(!result.prompt_body_persisted);
        assert!(result.phase_b_output.runner_call_allowed);
        assert!(result.phase_b_output.prompt_sent);
        assert!(result.phase_b_output.real_codex_executed);
        assert!(result.phase_b_output.writes_codex_home);
        assert!(result.phase_b_output.writes_project_files);
        assert_eq!(result.phase_b_output.readback_summary.result_count, Some(1));
        assert!(result.phase_a_output.product_command_attempt.is_some());
        assert!(result.phase_b_output.product_command_attempt.is_some());
        assert!(result.phase_b_output.continuation_attempt_id.is_some());
        assert!(!result.runtime_log_refs.is_empty());
        assert!(!result.audit_refs.is_empty());
        assert!(result.readback_ref.is_some());
        let developer = result
            .plan
            .run_units
            .iter()
            .find(|unit| unit.run_unit_kind == "developer_execution")
            .expect("developer run unit");
        assert_eq!(
            developer.product_command_ref.as_deref(),
            Some(result.product_command_id.as_str())
        );
        assert_eq!(developer.readback_result_count, Some(1));
        assert!(developer
            .audit_refs
            .iter()
            .any(|audit| audit.contains("j2-b-b2")));

        let sidecar = real_execution_command::real_execution_product_command_sidecar_path(&path)
            .expect("product command sidecar path");
        let sidecar_text = fs::read_to_string(sidecar).expect("sidecar should read");
        assert!(sidecar_text.contains(J2_B_B2_PROMPT_HASH));
        assert!(!sidecar_text.contains(J2_B_B2_CANONICAL_PROMPT));
        assert!(!sidecar_text.contains("You are the codex-local developer run unit"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b2_bridge_rejects_wrong_prompt_hash_before_sidecars() {
        let dir = temp_path("j2-b-b2-wrong-hash");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b2_workflow(&path);
        let mut input = b2_input();
        input.prompt_hash =
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

        let err = run_project_workflow_automation_j2_b_b2_with_runner(
            &path,
            &input,
            "2026-06-09T03:00:00Z",
            "write-j2-b-b2-wrong-hash",
            &dir.join("last-message.json"),
            &J2BB2FakePhaseBRunner,
        )
        .expect_err("wrong prompt hash should reject");

        assert!(err.contains("prompt_hash"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&path)
                .expect("product command sidecar path")
                .exists(),
            "wrong hash must not write product command sidecar"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b2_bridge_rejects_non_user_confirmation_before_sidecars() {
        let dir = temp_path("j2-b-b2-non-user");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b2_workflow(&path);
        let mut input = b2_input();
        input.confirmed_by = Some("project_director".to_string());

        let err = run_project_workflow_automation_j2_b_b2_with_runner(
            &path,
            &input,
            "2026-06-09T03:00:00Z",
            "write-j2-b-b2-non-user",
            &dir.join("last-message.json"),
            &J2BB2FakePhaseBRunner,
        )
        .expect_err("non-user confirmation should reject");

        assert!(err.contains("confirmed_by=user"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&path)
                .expect("product command sidecar path")
                .exists(),
            "non-user confirmation must not write product command sidecar"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b1_bridge_rejects_wrong_prompt_hash_before_sidecars() {
        let dir = temp_path("j2-b-b1-wrong-hash");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b1_workflow(&path);
        let mut input = b1_input();
        input.prompt_hash =
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

        let err = run_project_workflow_automation_j2_b_b1_with_runner(
            &path,
            &input,
            "2026-06-09T02:00:00Z",
            "write-j2-b-b1-wrong-hash",
            &dir.join("last-message.json"),
            &J2BB1FakePhaseBRunner,
        )
        .expect_err("wrong prompt hash should reject");

        assert!(err.contains("prompt_hash"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&path)
                .expect("product command sidecar path")
                .exists(),
            "wrong hash must not write product command sidecar"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b1_bridge_rejects_non_user_confirmation_before_sidecars() {
        let dir = temp_path("j2-b-b1-non-user");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b1_workflow(&path);
        let mut input = b1_input();
        input.confirmed_by = Some("project_director".to_string());

        let err = run_project_workflow_automation_j2_b_b1_with_runner(
            &path,
            &input,
            "2026-06-09T02:00:00Z",
            "write-j2-b-b1-non-user",
            &dir.join("last-message.json"),
            &J2BB1FakePhaseBRunner,
        )
        .expect_err("non-user confirmation should reject");

        assert!(err.contains("confirmed_by=user"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&path)
                .expect("product command sidecar path")
                .exists(),
            "non-user confirmation must not write product command sidecar"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn j2_b_b1_bridge_blocks_duplicate_phase_b_for_same_product_command() {
        let dir = temp_path("j2-b-b1-duplicate");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_b1_workflow(&path);
        let input = b1_input();

        let first = run_project_workflow_automation_j2_b_b1_with_runner(
            &path,
            &input,
            "2026-06-09T02:00:00Z",
            "write-j2-b-b1-duplicate-first",
            &dir.join("first-last-message.json"),
            &J2BB1FakePhaseBRunner,
        )
        .expect("first B1 bridge should complete");
        let (product_store, _, _) =
            real_execution_command::load_real_execution_product_command_store(
                &path,
                "2026-06-09T02:00:01Z",
            )
            .expect("product command store should load");
        let continuation_store =
            crate::session_continuation_store::load_store(&path, "2026-06-09T02:00:01Z")
                .expect("continuation store should load");
        let duplicate =
            real_execution_command::run_real_execution_product_command_phase_b_with_runner(
                &path,
                &RunRealExecutionProductCommandPhaseBInput {
                    product_command_id: first.product_command_id.clone(),
                    expected_product_command_store_revision: Some(product_store.revision),
                    expected_session_continuation_store_revision: Some(continuation_store.revision),
                    actor_role: "developer_execution".to_string(),
                    execution_decision: Some("approved_for_phase_b".to_string()),
                    authorization: j2_b_b1_authorization(&path, &first.product_command_id),
                    prompt_body: J2_B_B1_CANONICAL_PROMPT.to_string(),
                    requested_at: Some("2026-06-09T02:00:01Z".to_string()),
                },
                "2026-06-09T02:00:01Z",
                "write-j2-b-b1-duplicate-second-phase-b",
                &dir.join("second-last-message.json"),
                &J2BB1FakePhaseBRunner,
            )
            .expect("duplicate Phase B should return blocked output");

        assert_eq!(duplicate.status, "phase_b_blocked");
        assert!(duplicate
            .blocked_reasons
            .contains(&"phase_b_duplicate_attempt_blocked".to_string()));
        assert!(duplicate.product_command_attempt.is_none());
        assert!(!duplicate.prompt_sent);
        assert!(!duplicate.real_codex_executed);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn k3_b1_bridge_builds_frozen_request_and_blocks_without_runtime_prompt() {
        let dir = temp_path("k3-b1-noop");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_k3_b1_workflow(&path);
        let result = run_project_workflow_automation_k3_b_with_runner(
            &path,
            &k3_b1_input(None),
            "2026-06-10T02:00:00Z",
            "write-k3-b1-noop",
            &dir.join("k3-b1.last-message.json"),
            &K3BNoopPhaseBRunner,
        )
        .expect("K3-B1 no-op bridge should stop at Phase B guard");

        assert_eq!(result.status, "phase_b_blocked");
        assert_eq!(result.execution_point_id, K3_B_B1_EXECUTION_POINT_ID);
        assert_eq!(result.run_unit_id, K3_B_B1_RUN_UNIT_ID);
        assert_eq!(result.workflow_id, K3_B_B1_WORKFLOW_ID);
        assert_eq!(result.work_item_id, K3_B_B1_WORK_ITEM_ID);
        assert_eq!(result.task_memory_packet_ref, K3_B_B1_MEMORY_PACKET_REF);
        assert_eq!(
            result.permission_envelope_ref,
            K3_B_B1_PERMISSION_ENVELOPE_REF
        );
        assert_eq!(result.readback_marker, K3_B_B1_READBACK_MARKER);
        assert_eq!(
            result.preview.request.command_family,
            "real_execution_product_command"
        );
        assert_eq!(
            result.preview.request.workflow_id.as_deref(),
            Some(K3_B_B1_WORKFLOW_ID)
        );
        assert_eq!(
            result.preview.request.node_id.as_deref(),
            Some(K3_B_B1_NODE_ID)
        );
        assert_eq!(
            result.preview.request.work_item_id.as_deref(),
            Some(K3_B_B1_WORK_ITEM_ID)
        );
        assert_eq!(
            result.preview.request.memory_packet_ref.as_deref(),
            Some(K3_B_B1_MEMORY_PACKET_REF)
        );
        assert_eq!(result.preview.request.prompt_hash, K3_B_B1_PROMPT_HASH);
        assert!(result.preview.request.allowed_write_roots.is_empty());
        assert_eq!(result.phase_a_output.status, "phase_a_completed");
        assert_eq!(result.phase_b_output.status, "phase_b_blocked");
        assert!(result
            .phase_b_output
            .blocked_reasons
            .contains(&"phase_b_runtime_prompt_missing".to_string()));
        assert!(!result.phase_b_output.runner_call_allowed);
        assert!(!result.phase_b_output.prompt_sent);
        assert!(!result.phase_b_output.real_codex_executed);
        assert!(!result.phase_b_output.writes_codex_home);
        assert!(!result.phase_b_output.writes_project_files);
        assert_eq!(result.phase_b_output.readback_summary.result_count, None);
        assert!(result.allowed_project_write_roots.is_empty());
        assert!(result.allowed_project_write_path.is_none());
        assert!(result
            .baseline_refs
            .iter()
            .any(|item| item.contains("mario:index.html")));
        assert!(result
            .manifest_requirements
            .contains(&"read_only_core_hashes_must_match_before_after".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn k3_b2_bridge_builds_new_session_manifest_guard_without_runner() {
        let dir = temp_path("k3-b2-noop");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_k3_b2_workflow(&path);
        let result = run_project_workflow_automation_k3_b_with_runner(
            &path,
            &k3_b2_input(None),
            "2026-06-10T02:01:00Z",
            "write-k3-b2-noop",
            &dir.join("k3-b2.last-message.json"),
            &K3BNoopPhaseBRunner,
        )
        .expect("K3-B2 no-op bridge should stop at Phase B guard");

        assert_eq!(result.status, "phase_b_blocked");
        assert_eq!(result.execution_point_id, K3_B_B2_EXECUTION_POINT_ID);
        assert_eq!(result.run_unit_id, K3_B_B2_RUN_UNIT_ID);
        assert_eq!(result.workflow_id, K3_B_B2_WORKFLOW_ID);
        assert_eq!(result.work_item_id, K3_B_B2_WORK_ITEM_ID);
        assert_eq!(result.task_memory_packet_ref, K3_B_B2_MEMORY_PACKET_REF);
        assert_eq!(
            result.permission_envelope_ref,
            K3_B_B2_PERMISSION_ENVELOPE_REF
        );
        assert_eq!(result.readback_marker, K3_B_B2_READBACK_MARKER);
        assert_eq!(result.preview.request.operation_id, "new_session");
        assert_eq!(
            result.preview.request.session_mode,
            "new_session_execution_point"
        );
        assert_eq!(
            result.preview.request.workflow_id.as_deref(),
            Some(K3_B_B2_WORKFLOW_ID)
        );
        assert_eq!(
            result.preview.request.node_id.as_deref(),
            Some(K3_B_B2_NODE_ID)
        );
        assert_eq!(result.preview.request.target_session_id, None);
        assert_eq!(
            result.preview.request.allowed_write_roots,
            vec![K3_B_B2_ALLOWED_WRITE_ROOT.to_string()]
        );
        assert_eq!(
            result.allowed_project_write_path.as_deref(),
            Some(K3_B_B2_ALLOWED_WRITE_PATH)
        );
        assert!(result
            .phase_b_output
            .blocked_reasons
            .contains(&"phase_b_runtime_prompt_missing".to_string()));
        assert!(!result.phase_b_output.prompt_sent);
        assert!(!result.phase_b_output.real_codex_executed);
        assert!(!result.phase_b_output.writes_codex_home);
        assert!(!result.phase_b_output.writes_project_files);
        assert_eq!(result.phase_b_output.readback_summary.result_count, None);
        assert!(result
            .manifest_requirements
            .iter()
            .any(|item| item.contains("only_allowed_write_path_may_change")));
        assert!(result.warnings.contains(
            &"k3_b2_new_session_product_command_phase_b_path_available_env_gated".to_string()
        ));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn k3_b_bridge_rejects_wrong_hash_and_non_user_before_sidecars() {
        let dir = temp_path("k3-b-rejects");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let wrong_hash_path = dir.join("workflow-state-wrong-hash.v0.json");
        bootstrap_k3_b1_workflow(&wrong_hash_path);
        let mut wrong_hash = k3_b1_input(None);
        wrong_hash.prompt_hash =
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());
        let err = run_project_workflow_automation_k3_b_with_runner(
            &wrong_hash_path,
            &wrong_hash,
            "2026-06-10T02:02:00Z",
            "write-k3-b-wrong-hash",
            &dir.join("wrong-hash.last-message.json"),
            &K3BNoopPhaseBRunner,
        )
        .expect_err("wrong hash should reject before sidecars");
        assert!(err.contains("prompt_hash"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&wrong_hash_path)
                .expect("product command sidecar path")
                .exists(),
            "wrong hash must not write product command sidecar"
        );

        let non_user_path = dir.join("workflow-state-non-user.v0.json");
        bootstrap_k3_b2_workflow(&non_user_path);
        let mut non_user = k3_b2_input(None);
        non_user.confirmed_by = Some("project_director".to_string());
        let err = run_project_workflow_automation_k3_b_with_runner(
            &non_user_path,
            &non_user,
            "2026-06-10T02:02:00Z",
            "write-k3-b-non-user",
            &dir.join("non-user.last-message.json"),
            &K3BNoopPhaseBRunner,
        )
        .expect_err("non-user confirmation should reject before sidecars");
        assert!(err.contains("confirmed_by=user"), "{err}");
        assert!(
            !real_execution_command::real_execution_product_command_sidecar_path(&non_user_path)
                .expect("product command sidecar path")
                .exists(),
            "non-user confirmation must not write product command sidecar"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn k3_b_phase_a_duplicate_guard_blocks_same_product_command_without_runner() {
        let dir = temp_path("k3-b-duplicate");
        fs::create_dir_all(&dir).expect("temp dir should create");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_k3_b1_workflow(&path);
        let first = run_project_workflow_automation_k3_b_with_runner(
            &path,
            &k3_b1_input(None),
            "2026-06-10T02:03:00Z",
            "write-k3-b-duplicate-first",
            &dir.join("first.last-message.json"),
            &K3BNoopPhaseBRunner,
        )
        .expect("first K3-B bridge should block at missing prompt");
        assert_eq!(first.status, "phase_b_blocked");
        let (product_store, _, _) =
            real_execution_command::load_real_execution_product_command_store(
                &path,
                "2026-06-10T02:03:01Z",
            )
            .expect("product command store should load");
        let continuation_store =
            crate::session_continuation_store::load_store(&path, "2026-06-10T02:03:01Z")
                .expect("continuation store should load");
        let duplicate = real_execution_command::run_real_execution_product_command_phase_a_at(
            &path,
            &RunRealExecutionProductCommandPhaseAInput {
                product_command_id: first.product_command_id.clone(),
                expected_product_command_store_revision: Some(product_store.revision),
                expected_session_continuation_store_revision: Some(continuation_store.revision),
                actor_role: "developer_execution".to_string(),
                execution_decision: Some("approved_for_phase_a".to_string()),
                timeout_ms: Some(120_000),
                requested_at: Some("2026-06-10T02:03:01Z".to_string()),
            },
            "2026-06-10T02:03:01Z",
            "write-k3-b-duplicate-second-phase-a",
        )
        .expect("duplicate Phase A should return blocked output");

        assert_eq!(duplicate.status, "phase_a_blocked");
        assert!(duplicate
            .blocked_reasons
            .contains(&"phase_a_duplicate_running_or_completed_attempt".to_string()));
        assert!(!duplicate.prompt_sent);
        assert!(!duplicate.real_codex_executed);

        let duplicate_b =
            real_execution_command::run_real_execution_product_command_phase_b_with_runner(
                &path,
                &RunRealExecutionProductCommandPhaseBInput {
                    product_command_id: first.product_command_id.clone(),
                    expected_product_command_store_revision: Some(product_store.revision),
                    expected_session_continuation_store_revision: Some(continuation_store.revision),
                    actor_role: "developer_execution".to_string(),
                    execution_decision: Some("approved_for_phase_b".to_string()),
                    authorization: k3_b_resume_authorization(
                        &path,
                        &k3_b_config(K3_B_B1_EXECUTION_POINT_ID).unwrap(),
                    ),
                    prompt_body: String::new(),
                    requested_at: Some("2026-06-10T02:03:01Z".to_string()),
                },
                "2026-06-10T02:03:01Z",
                "write-k3-b-duplicate-second",
                &dir.join("second.last-message.json"),
                &K3BNoopPhaseBRunner,
            )
            .expect("Phase B without runtime prompt should return blocked output");

        assert_eq!(duplicate_b.status, "phase_b_blocked");
        assert!(duplicate_b
            .blocked_reasons
            .contains(&"phase_b_runtime_prompt_missing".to_string()));
        assert!(!duplicate_b.prompt_sent);
        assert!(!duplicate_b.real_codex_executed);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    #[ignore = "requires explicit K3-B1 mario test workflow real resume authorization"]
    fn k3_b1_real_mario_test_workflow_resume_requires_env_authorization() {
        assert_eq!(
            env::var("K3_B1_REAL_EXECUTION_AUTHORIZED")
                .expect("K3_B1_REAL_EXECUTION_AUTHORIZED is required"),
            K3_B_B1_EXECUTION_POINT_ID
        );
        assert_eq!(
            env::var("K3_B1_PROJECT_ROOT").expect("K3_B1_PROJECT_ROOT is required"),
            K3_B_B1_PROJECT_ROOT
        );
        assert_eq!(
            env::var("K3_B1_SESSION_ID").expect("K3_B1_SESSION_ID is required"),
            K3_B_B1_SESSION_ID
        );
        assert_eq!(
            env::var("K3_B1_EXPECTED_MARKER").expect("K3_B1_EXPECTED_MARKER is required"),
            K3_B_B1_READBACK_MARKER
        );
        let workflow_state_parent = PathBuf::from(
            env::var("K3_B1_WORKFLOW_STATE_PARENT")
                .expect("K3_B1_WORKFLOW_STATE_PARENT is required"),
        );
        assert!(
            workflow_state_parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs"
            ),
            "K3-B1 workflow state parent must be inside product-line tmp/k3-b-real-workflow-automation/runs"
        );
        let prompt_path =
            PathBuf::from(env::var("K3_B1_PROMPT_PATH").expect("K3_B1_PROMPT_PATH is required"));
        assert!(
            prompt_path.starts_with("/Users/yoyi/workspace/product-line/tmp/"),
            "K3-B1 prompt path must be a runtime-only product-line tmp file"
        );
        let prompt_body = fs::read_to_string(&prompt_path).expect("K3-B1 prompt should read");
        assert_eq!(sha256_hex(&prompt_body), K3_B_B1_PROMPT_HASH);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        let run_dir = workflow_state_parent.join(format!("k3-b1-run-{unique}"));
        fs::create_dir_all(&run_dir).expect("run dir should create");
        let workflow_state_path = run_dir.join("workflow-state.v0.json");
        bootstrap_k3_b1_workflow(&workflow_state_path);

        let project_root_path = PathBuf::from(K3_B_B1_PROJECT_ROOT);
        let before_core_hashes = mario_core_file_hashes(&project_root_path);
        let output = run_project_workflow_automation_k3_b_at(
            &workflow_state_path,
            &k3_b1_input(Some(prompt_body.clone())),
            "2026-06-10T03:00:03Z",
            "write-k3-b1-real-resume",
        )
        .expect("K3-B1 real project workflow bridge should complete");
        let after_core_hashes = mario_core_file_hashes(&project_root_path);
        let last_message_path = run_dir.join(format!(
            "{}-last-message-{}.json",
            K3_B_B1_EXECUTION_POINT_ID.replace("stage-k-", ""),
            stable_id("2026-06-10T03:00:03Z")
        ));
        let last_message =
            fs::read_to_string(&last_message_path).expect("K3-B1 last message should read");
        let product_sidecar_path =
            real_execution_command::real_execution_product_command_sidecar_path(
                &workflow_state_path,
            )
            .expect("product sidecar path");
        let continuation_sidecar_path =
            crate::session_continuation_store::sidecar_path(&workflow_state_path)
                .expect("continuation sidecar path");
        let runtime_log_path =
            crate::runtime_log_store::sidecar_path(&workflow_state_path).expect("runtime log path");

        println!(
            "K3_B1_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "K3_B1_PRODUCT_COMMAND_SIDECAR_PATH={}",
            product_sidecar_path.display()
        );
        println!(
            "K3_B1_SESSION_CONTINUATION_STORE_PATH={}",
            continuation_sidecar_path.display()
        );
        println!("K3_B1_RUNTIME_LOG_PATH={}", runtime_log_path.display());
        println!("K3_B1_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("K3_B1_PRODUCT_COMMAND_ID={}", output.product_command_id);
        println!("K3_B1_AUDIT_REFS={}", output.audit_refs.join(","));
        println!("K3_B1_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("K3_B1_CORE_HASHES_AFTER={:?}", after_core_hashes);

        assert_eq!(output.status, "phase_b_completed");
        assert_eq!(
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .expect("Phase B product attempt")
                .status,
            "phase_b_real_resume_executed"
        );
        assert!(output.phase_b_output.runner_call_allowed);
        assert!(output.phase_b_output.prompt_sent);
        assert!(output.phase_b_output.real_codex_executed);
        assert!(output.phase_b_output.writes_codex_home);
        assert!(!output.phase_b_output.writes_project_files);
        assert_eq!(output.phase_b_output.readback_summary.status, "succeeded");
        assert_eq!(output.phase_b_output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(K3_B_B1_READBACK_MARKER));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert_path_does_not_persist_prompt(&product_sidecar_path, &prompt_body);
        assert_path_does_not_persist_prompt(&continuation_sidecar_path, &prompt_body);
        assert_path_does_not_persist_prompt(&runtime_log_path, &prompt_body);
    }

    #[test]
    #[ignore = "requires explicit K3-B2 isolated workflow real new-session authorization"]
    fn k3_b2_real_isolated_workflow_new_session_requires_env_authorization() {
        assert_eq!(
            env::var("K3_B2_REAL_EXECUTION_AUTHORIZED")
                .expect("K3_B2_REAL_EXECUTION_AUTHORIZED is required"),
            K3_B_B2_EXECUTION_POINT_ID
        );
        assert_eq!(
            env::var("K3_B2_PROJECT_ROOT").expect("K3_B2_PROJECT_ROOT is required"),
            K3_B_B2_PROJECT_ROOT
        );
        assert_eq!(
            env::var("K3_B2_ALLOWED_WRITE_PATH").expect("K3_B2_ALLOWED_WRITE_PATH is required"),
            K3_B_B2_ALLOWED_WRITE_PATH
        );
        assert_eq!(
            env::var("K3_B2_EXPECTED_MARKER").expect("K3_B2_EXPECTED_MARKER is required"),
            K3_B_B2_READBACK_MARKER
        );
        let workflow_state_parent = PathBuf::from(
            env::var("K3_B2_WORKFLOW_STATE_PARENT")
                .expect("K3_B2_WORKFLOW_STATE_PARENT is required"),
        );
        assert!(
            workflow_state_parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/runs"
            ),
            "K3-B2 workflow state parent must be inside product-line tmp/k3-b-real-workflow-automation/runs"
        );
        let prompt_path =
            PathBuf::from(env::var("K3_B2_PROMPT_PATH").expect("K3_B2_PROMPT_PATH is required"));
        assert!(
            prompt_path.starts_with("/Users/yoyi/workspace/product-line/tmp/"),
            "K3-B2 prompt path must be a runtime-only product-line tmp file"
        );
        let prompt_body = fs::read_to_string(&prompt_path).expect("K3-B2 prompt should read");
        assert_eq!(sha256_hex(&prompt_body), K3_B_B2_PROMPT_HASH);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        let run_dir = workflow_state_parent.join(format!("k3-b2-run-{unique}"));
        fs::create_dir_all(&run_dir).expect("run dir should create");
        let workflow_state_path = run_dir.join("workflow-state.v0.json");
        bootstrap_k3_b2_workflow(&workflow_state_path);

        let project_root_path = PathBuf::from(K3_B_B2_PROJECT_ROOT);
        let allowed_write_path = PathBuf::from(K3_B_B2_ALLOWED_WRITE_PATH);
        fs::create_dir_all(K3_B_B2_ALLOWED_WRITE_ROOT)
            .expect("K3-B2 allowed write root should exist before Codex add-dir");
        let before_project_manifest =
            isolated_project_file_manifest(&project_root_path, &allowed_write_path);
        let output = run_project_workflow_automation_k3_b_at(
            &workflow_state_path,
            &k3_b2_input(Some(prompt_body.clone())),
            "2026-06-10T03:01:03Z",
            "write-k3-b2-real-new-session",
        )
        .expect("K3-B2 real project workflow bridge should complete");
        let after_project_manifest =
            isolated_project_file_manifest(&project_root_path, &allowed_write_path);
        let last_message_path = run_dir.join(format!(
            "{}-last-message-{}.json",
            K3_B_B2_EXECUTION_POINT_ID.replace("stage-k-", ""),
            stable_id("2026-06-10T03:01:03Z")
        ));
        let last_message =
            fs::read_to_string(&last_message_path).expect("K3-B2 last message should read");
        let product_sidecar_path =
            real_execution_command::real_execution_product_command_sidecar_path(
                &workflow_state_path,
            )
            .expect("product sidecar path");
        let continuation_sidecar_path =
            crate::session_continuation_store::sidecar_path(&workflow_state_path)
                .expect("continuation sidecar path");
        let runtime_log_path =
            crate::runtime_log_store::sidecar_path(&workflow_state_path).expect("runtime log path");

        println!(
            "K3_B2_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "K3_B2_PRODUCT_COMMAND_SIDECAR_PATH={}",
            product_sidecar_path.display()
        );
        println!(
            "K3_B2_SESSION_CONTINUATION_STORE_PATH={}",
            continuation_sidecar_path.display()
        );
        println!("K3_B2_RUNTIME_LOG_PATH={}", runtime_log_path.display());
        println!("K3_B2_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("K3_B2_PRODUCT_COMMAND_ID={}", output.product_command_id);
        println!("K3_B2_AUDIT_REFS={}", output.audit_refs.join(","));
        println!(
            "K3_B2_PROJECT_MANIFEST_BEFORE={:?}",
            before_project_manifest
        );
        println!("K3_B2_PROJECT_MANIFEST_AFTER={:?}", after_project_manifest);
        if allowed_write_path.exists() {
            println!(
                "K3_B2_ALLOWED_WRITE_PATH_HASH={}",
                sha256_file(&allowed_write_path)
            );
        }

        assert_eq!(output.status, "phase_b_completed");
        assert_eq!(
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .expect("Phase B product attempt")
                .status,
            "phase_b_real_new_session_executed"
        );
        assert!(output.phase_b_output.runner_call_allowed);
        assert!(output.phase_b_output.prompt_sent);
        assert!(output.phase_b_output.real_codex_executed);
        assert!(output.phase_b_output.writes_codex_home);
        assert!(output.phase_b_output.writes_project_files);
        assert_eq!(output.phase_b_output.readback_summary.status, "succeeded");
        assert_eq!(output.phase_b_output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(K3_B_B2_READBACK_MARKER));
        assert_eq!(
            before_project_manifest, after_project_manifest,
            "K3-B2 real probe must not add or modify files outside the allowed write path"
        );
        assert!(allowed_write_path.exists());
        assert_path_does_not_persist_prompt(&product_sidecar_path, &prompt_body);
        assert_path_does_not_persist_prompt(&continuation_sidecar_path, &prompt_body);
        assert_path_does_not_persist_prompt(&runtime_log_path, &prompt_body);
    }

    #[test]
    #[ignore = "requires explicit J2-B B1 mario test project workflow real resume authorization"]
    fn j2_b_b1_real_mario_test_project_workflow_resume_probe_requires_env_authorization() {
        assert_eq!(
            env::var("J2_B_B1_REAL_EXECUTION_AUTHORIZED")
                .expect("J2_B_B1_REAL_EXECUTION_AUTHORIZED is required"),
            "1"
        );
        assert_eq!(
            env::var("J2_B_B1_PROJECT_ROOT").expect("J2_B_B1_PROJECT_ROOT is required"),
            J2_B_B1_PROJECT_ROOT
        );
        assert_eq!(
            env::var("J2_B_B1_SESSION_ID").expect("J2_B_B1_SESSION_ID is required"),
            J2_B_B1_SESSION_ID
        );
        assert_eq!(
            env::var("J2_B_B1_EXPECTED_MARKER").expect("J2_B_B1_EXPECTED_MARKER is required"),
            J2_B_B1_READBACK_MARKER
        );
        let workflow_state_parent = PathBuf::from(
            env::var("J2_B_B1_WORKFLOW_STATE_PARENT")
                .expect("J2_B_B1_WORKFLOW_STATE_PARENT is required"),
        );
        assert!(
            workflow_state_parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs"
            ),
            "J2-B B1 workflow state parent must be inside product-line tmp/j2-b-real-workflow-automation/runs"
        );
        assert_eq!(sha256_hex(J2_B_B1_CANONICAL_PROMPT), J2_B_B1_PROMPT_HASH);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        let run_dir = workflow_state_parent.join(format!("j2-b-b1-run-{unique}"));
        fs::create_dir_all(&run_dir).expect("run dir should create");
        let workflow_state_path = run_dir.join("workflow-state.v0.json");
        bootstrap_b1_workflow(&workflow_state_path);

        let project_root_path = PathBuf::from(J2_B_B1_PROJECT_ROOT);
        let before_core_hashes = mario_core_file_hashes(&project_root_path);
        let output = run_project_workflow_automation_j2_b_b1_at(
            &workflow_state_path,
            &b1_input(),
            "2026-06-09T11:00:03Z",
            "write-j2-b-b1-real-resume",
        )
        .expect("J2-B B1 real project workflow bridge should complete");
        let after_core_hashes = mario_core_file_hashes(&project_root_path);
        let last_message_path = run_dir.join(format!(
            "j2-b-b1-last-message-{}.json",
            stable_id("2026-06-09T11:00:03Z")
        ));
        let last_message =
            fs::read_to_string(&last_message_path).expect("J2-B B1 last message should read");
        let product_sidecar_path =
            real_execution_command::real_execution_product_command_sidecar_path(
                &workflow_state_path,
            )
            .expect("product sidecar path");
        let continuation_sidecar_path =
            crate::session_continuation_store::sidecar_path(&workflow_state_path)
                .expect("continuation sidecar path");
        let runtime_log_path =
            crate::runtime_log_store::sidecar_path(&workflow_state_path).expect("runtime log path");

        println!(
            "J2_B_B1_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "J2_B_B1_PRODUCT_COMMAND_SIDECAR_PATH={}",
            product_sidecar_path.display()
        );
        println!(
            "J2_B_B1_SESSION_CONTINUATION_STORE_PATH={}",
            continuation_sidecar_path.display()
        );
        println!("J2_B_B1_RUNTIME_LOG_PATH={}", runtime_log_path.display());
        println!("J2_B_B1_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("J2_B_B1_PRODUCT_COMMAND_ID={}", output.product_command_id);
        println!(
            "J2_B_B1_PRODUCT_ATTEMPT_ID={}",
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .map(|attempt| attempt.attempt_id.as_str())
                .unwrap_or("")
        );
        println!(
            "J2_B_B1_CONTINUATION_ID={}",
            output
                .phase_b_output
                .continuation_id
                .as_deref()
                .unwrap_or("")
        );
        println!(
            "J2_B_B1_CONTINUATION_ATTEMPT_ID={}",
            output
                .phase_b_output
                .continuation_attempt_id
                .as_deref()
                .unwrap_or("")
        );
        println!(
            "J2_B_B1_RUNTIME_LOG_REF={}",
            output
                .phase_b_output
                .runtime_log_ref
                .as_deref()
                .unwrap_or("")
        );
        println!("J2_B_B1_AUDIT_REFS={}", output.audit_refs.join(","));
        println!("J2_B_B1_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("J2_B_B1_CORE_HASHES_AFTER={:?}", after_core_hashes);

        assert_eq!(output.status, "phase_b_completed");
        assert_eq!(output.phase_b_output.status, "phase_b_completed");
        assert_eq!(
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .expect("Phase B product attempt")
                .status,
            "phase_b_real_resume_executed"
        );
        assert!(output.phase_b_output.runner_call_allowed);
        assert!(output.phase_b_output.prompt_sent);
        assert!(output.phase_b_output.real_codex_executed);
        assert!(output.phase_b_output.writes_codex_home);
        assert!(!output.phase_b_output.writes_project_files);
        assert_eq!(output.phase_b_output.readback_summary.status, "succeeded");
        assert_eq!(output.phase_b_output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(J2_B_B1_READBACK_MARKER));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert!(output.allowed_project_write_roots.is_empty());
        assert_path_does_not_persist_prompt(&product_sidecar_path, J2_B_B1_CANONICAL_PROMPT);
        assert_path_does_not_persist_prompt(&continuation_sidecar_path, J2_B_B1_CANONICAL_PROMPT);
        assert_path_does_not_persist_prompt(&runtime_log_path, J2_B_B1_CANONICAL_PROMPT);
    }

    #[test]
    #[ignore = "requires explicit J2-B B2 isolated project workflow real new-session authorization"]
    fn j2_b_b2_real_isolated_project_workflow_new_session_probe_requires_env_authorization() {
        assert_eq!(
            env::var("J2_B_B2_REAL_EXECUTION_AUTHORIZED")
                .expect("J2_B_B2_REAL_EXECUTION_AUTHORIZED is required"),
            "1"
        );
        assert_eq!(
            env::var("J2_B_B2_PROJECT_ROOT").expect("J2_B_B2_PROJECT_ROOT is required"),
            J2_B_B2_PROJECT_ROOT
        );
        assert_eq!(
            env::var("J2_B_B2_ALLOWED_WRITE_PATH").expect("J2_B_B2_ALLOWED_WRITE_PATH is required"),
            J2_B_B2_ALLOWED_WRITE_PATH
        );
        assert_eq!(
            env::var("J2_B_B2_EXPECTED_MARKER").expect("J2_B_B2_EXPECTED_MARKER is required"),
            J2_B_B2_READBACK_MARKER
        );
        let workflow_state_parent = PathBuf::from(
            env::var("J2_B_B2_WORKFLOW_STATE_PARENT")
                .expect("J2_B_B2_WORKFLOW_STATE_PARENT is required"),
        );
        assert!(
            workflow_state_parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs"
            ),
            "J2-B B2 workflow state parent must be inside product-line tmp/j2-b-real-workflow-automation/runs"
        );
        assert_eq!(sha256_hex(J2_B_B2_CANONICAL_PROMPT), J2_B_B2_PROMPT_HASH);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should work")
            .as_nanos();
        let run_dir = workflow_state_parent.join(format!("j2-b-b2-run-{unique}"));
        fs::create_dir_all(&run_dir).expect("run dir should create");
        let workflow_state_path = run_dir.join("workflow-state.v0.json");
        bootstrap_b2_workflow(&workflow_state_path);

        let project_root_path = PathBuf::from(J2_B_B2_PROJECT_ROOT);
        let allowed_write_path = PathBuf::from(J2_B_B2_ALLOWED_WRITE_PATH);
        fs::create_dir_all(J2_B_B2_ALLOWED_WRITE_ROOT)
            .expect("J2-B B2 allowed write root should exist before Codex add-dir");
        let before_core_hashes = isolated_core_file_hashes(&project_root_path);
        let before_project_manifest =
            isolated_project_file_manifest(&project_root_path, &allowed_write_path);
        let output = run_project_workflow_automation_j2_b_b2_at(
            &workflow_state_path,
            &b2_input(),
            "2026-06-09T12:00:03Z",
            "write-j2-b-b2-real-new-session",
        )
        .expect("J2-B B2 real project workflow bridge should complete");
        let after_core_hashes = isolated_core_file_hashes(&project_root_path);
        let after_project_manifest =
            isolated_project_file_manifest(&project_root_path, &allowed_write_path);
        let last_message_path = run_dir.join(format!(
            "j2-b-b2-last-message-{}.json",
            stable_id("2026-06-09T12:00:03Z")
        ));
        let last_message =
            fs::read_to_string(&last_message_path).expect("J2-B B2 last message should read");
        let product_sidecar_path =
            real_execution_command::real_execution_product_command_sidecar_path(
                &workflow_state_path,
            )
            .expect("product sidecar path");
        let continuation_sidecar_path =
            crate::session_continuation_store::sidecar_path(&workflow_state_path)
                .expect("continuation sidecar path");
        let runtime_log_path =
            crate::runtime_log_store::sidecar_path(&workflow_state_path).expect("runtime log path");

        println!(
            "J2_B_B2_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "J2_B_B2_PRODUCT_COMMAND_SIDECAR_PATH={}",
            product_sidecar_path.display()
        );
        println!(
            "J2_B_B2_SESSION_CONTINUATION_STORE_PATH={}",
            continuation_sidecar_path.display()
        );
        println!("J2_B_B2_RUNTIME_LOG_PATH={}", runtime_log_path.display());
        println!("J2_B_B2_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("J2_B_B2_PRODUCT_COMMAND_ID={}", output.product_command_id);
        println!(
            "J2_B_B2_PRODUCT_ATTEMPT_ID={}",
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .map(|attempt| attempt.attempt_id.as_str())
                .unwrap_or("")
        );
        println!(
            "J2_B_B2_CONTINUATION_ID={}",
            output
                .phase_b_output
                .continuation_id
                .as_deref()
                .unwrap_or("")
        );
        println!(
            "J2_B_B2_CONTINUATION_ATTEMPT_ID={}",
            output
                .phase_b_output
                .continuation_attempt_id
                .as_deref()
                .unwrap_or("")
        );
        println!(
            "J2_B_B2_RUNTIME_LOG_REF={}",
            output
                .phase_b_output
                .runtime_log_ref
                .as_deref()
                .unwrap_or("")
        );
        println!("J2_B_B2_AUDIT_REFS={}", output.audit_refs.join(","));
        println!("J2_B_B2_CORE_HASHES_BEFORE={:?}", before_core_hashes);
        println!("J2_B_B2_CORE_HASHES_AFTER={:?}", after_core_hashes);
        println!(
            "J2_B_B2_PROJECT_MANIFEST_BEFORE={:?}",
            before_project_manifest
        );
        println!(
            "J2_B_B2_PROJECT_MANIFEST_AFTER={:?}",
            after_project_manifest
        );
        println!(
            "J2_B_B2_ALLOWED_WRITE_PATH_EXISTS={}",
            allowed_write_path.exists()
        );
        if allowed_write_path.exists() {
            println!(
                "J2_B_B2_ALLOWED_WRITE_PATH_HASH={}",
                sha256_file(&allowed_write_path)
            );
        }

        assert_eq!(output.status, "phase_b_completed");
        assert_eq!(output.phase_b_output.status, "phase_b_completed");
        assert_eq!(
            output
                .phase_b_output
                .product_command_attempt
                .as_ref()
                .expect("Phase B product attempt")
                .status,
            "phase_b_real_new_session_executed"
        );
        assert!(output.phase_b_output.runner_call_allowed);
        assert!(output.phase_b_output.prompt_sent);
        assert!(output.phase_b_output.real_codex_executed);
        assert!(output.phase_b_output.writes_codex_home);
        assert!(output.phase_b_output.writes_project_files);
        assert_eq!(output.phase_b_output.readback_summary.status, "succeeded");
        assert_eq!(output.phase_b_output.readback_summary.result_count, Some(1));
        assert!(last_message.contains(J2_B_B2_READBACK_MARKER));
        assert_eq!(before_core_hashes, after_core_hashes);
        assert_eq!(
            before_project_manifest, after_project_manifest,
            "B2 real probe must not add or modify files outside the allowed write path"
        );
        assert!(allowed_write_path.exists());
        assert_path_does_not_persist_prompt(&product_sidecar_path, J2_B_B2_CANONICAL_PROMPT);
        assert_path_does_not_persist_prompt(&continuation_sidecar_path, J2_B_B2_CANONICAL_PROMPT);
        assert_path_does_not_persist_prompt(&runtime_log_path, J2_B_B2_CANONICAL_PROMPT);
    }
}
