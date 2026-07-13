use super::supervisor_action_protocol::{
    parse_supervisor_action_proposal, SupervisorActionKind, SupervisorActionProposalV1,
    SupervisorFinalizeVerdict,
};
use crate::mcp::{McpRole, McpServerConfig, SupervisorQuotaLimits};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const STORE_SCHEMA_VERSION: &str = "supervisor_action_control.v1";
const SIDECAR_NAME: &str = "supervisor-action-control.v1.json";
const LOCK_NAME: &str = ".supervisor-action-control.v1.lock";
const LOCK_RETRY_COUNT: usize = 5;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(100);

pub(crate) fn supervisor_action_limit(limits: &SupervisorQuotaLimits) -> usize {
    limits
        .max_active_workers
        .saturating_mul(limits.max_follow_ups_per_worker.saturating_add(2))
        .saturating_add(4)
}

#[derive(Debug, Clone)]
pub(crate) struct SupervisorActionRuntime {
    pub(crate) run_id: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) authorization_id: String,
    pub(crate) workflow_state_path: PathBuf,
    pub(crate) quota_limits: SupervisorQuotaLimits,
    pub(crate) started_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SupervisorActionResultV1 {
    pub(crate) action_id: Option<String>,
    pub(crate) status: String,
    pub(crate) summary: String,
    pub(crate) worker_id: Option<String>,
    pub(crate) adapter_id: Option<String>,
    pub(crate) evidence_present: bool,
}

impl SupervisorActionResultV1 {
    pub(crate) fn should_continue(&self) -> bool {
        !matches!(
            self.status.as_str(),
            "waiting_user" | "protocol_invalid" | "report_invalid" | "quota_exceeded"
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AuthorizedSupervisorAction {
    pub(crate) runtime: SupervisorActionRuntime,
    pub(crate) proposal: SupervisorActionProposalV1,
    pub(crate) allowed_read_roots: Vec<String>,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) authorization_snapshot_hash: String,
    pub(crate) task_package_fingerprint: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SupervisorActionAdapterResult {
    pub(crate) status: String,
    pub(crate) summary: String,
    pub(crate) worker_id: Option<String>,
    pub(crate) adapter_id: String,
    pub(crate) evidence_present: bool,
    pub(crate) dispatch_ref: Option<String>,
    pub(crate) readback_ref: Option<String>,
    pub(crate) audit_ref: Option<String>,
}

pub(crate) trait SupervisorActionAdapter {
    fn supports(&self, action: &SupervisorActionKind) -> bool;

    fn execute(
        &self,
        action: &AuthorizedSupervisorAction,
    ) -> Result<SupervisorActionAdapterResult, String>;
}

pub(crate) struct WorkbenchSupervisorActionAdapter;

impl SupervisorActionAdapter for WorkbenchSupervisorActionAdapter {
    fn supports(&self, action: &SupervisorActionKind) -> bool {
        matches!(
            action,
            SupervisorActionKind::DispatchWorker { .. }
                | SupervisorActionKind::InspectWorker { .. }
                | SupervisorActionKind::FollowUpWorker { .. }
                | SupervisorActionKind::WaitWorker { .. }
                | SupervisorActionKind::Finalize { .. }
                | SupervisorActionKind::ReportUser { .. }
                | SupervisorActionKind::RequestUserDecision { .. }
        )
    }

    fn execute(
        &self,
        action: &AuthorizedSupervisorAction,
    ) -> Result<SupervisorActionAdapterResult, String> {
        let config = mcp_config(&action.runtime);
        let value = match &action.proposal.action {
            SupervisorActionKind::DispatchWorker { target } => {
                crate::mcp::supervisor_orchestrator::control_core_dispatch_worker(
                    &config,
                    &action.runtime.project_root,
                    &action.runtime.workflow_id,
                    &action.runtime.authorization_id,
                    &target.node_id,
                    &target.work_item_id,
                )?
            }
            SupervisorActionKind::InspectWorker { worker_id } => {
                crate::mcp::supervisor_orchestrator::control_core_read_worker_report(
                    &config, worker_id,
                )?
            }
            SupervisorActionKind::FollowUpWorker { worker_id, prompt } => {
                crate::mcp::supervisor_orchestrator::control_core_follow_up_worker(
                    &config, worker_id, prompt,
                )?
            }
            SupervisorActionKind::WaitWorker { worker_id } => {
                crate::mcp::supervisor_orchestrator::control_core_wait_for_worker(
                    &config, worker_id,
                )?
            }
            SupervisorActionKind::Finalize { verdict } => {
                crate::mcp::supervisor_orchestrator::control_core_finalize(
                    &config,
                    &action.runtime.project_root,
                    &action.runtime.workflow_id,
                    &action.runtime.authorization_id,
                    verdict.as_str(),
                    &action.proposal.reason,
                )?
            }
            SupervisorActionKind::ReportUser { message } => {
                crate::mcp::supervisor_orchestrator::control_core_report_user(
                    &config,
                    &action.runtime.project_root,
                    &action.runtime.workflow_id,
                    &action.runtime.authorization_id,
                    message,
                )?
            }
            SupervisorActionKind::RequestUserDecision { question } => {
                return Ok(SupervisorActionAdapterResult {
                    status: "waiting_user".to_string(),
                    summary: format!("主管请求用户决定：{question}"),
                    worker_id: None,
                    adapter_id: "syn-control-core".to_string(),
                    evidence_present: false,
                    dispatch_ref: None,
                    readback_ref: None,
                    audit_ref: None,
                });
            }
        };
        let worker_id = value
            .get("worker_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let state = value
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("completed");
        let (status, summary, evidence_present, readback_ref) = match &action.proposal.action {
            SupervisorActionKind::DispatchWorker { .. } => (
                "waiting_worker".to_string(),
                format!(
                    "worker 已派发；即使执行进程已退出，仍须 inspect_worker 读取合法结构化回程后才能进入验收。{}",
                    compact_value_summary(&value)
                ),
                false,
                None,
            ),
            SupervisorActionKind::InspectWorker { worker_id } => {
                worker_report_outcome(&value, worker_id)?
            }
            SupervisorActionKind::FollowUpWorker { .. } => (
                "waiting_worker".to_string(),
                format!(
                    "worker 已返回追问结果；必须重新 inspect_worker 读取并验证新报告后才能终标。{}",
                    compact_value_summary(&value)
                ),
                false,
                None,
            ),
            SupervisorActionKind::WaitWorker { .. } => match state {
                "blocked" => (
                    "waiting_user".to_string(),
                    format!("worker 已阻塞：{}", compact_value_summary(&value)),
                    false,
                    None,
                ),
                "completed" => (
                    "waiting_worker".to_string(),
                    "worker 执行进程已结束；仍须 inspect_worker 读取合法结构化回程。"
                        .to_string(),
                    false,
                    None,
                ),
                _ => (
                    "waiting_worker".to_string(),
                    compact_value_summary(&value),
                    false,
                    None,
                ),
            },
            _ => (
                if matches!(state, "running" | "queued") {
                    "waiting_worker".to_string()
                } else {
                    "completed".to_string()
                },
                compact_value_summary(&value),
                false,
                None,
            ),
        };
        let binding_ref = format!(
            "supervisor_orchestrator;run={};auth_snapshot={};task_package={};read_roots={};write_roots={}",
            crate::stable_id(&action.runtime.run_id),
            action.authorization_snapshot_hash,
            action.task_package_fingerprint,
            action.allowed_read_roots.len(),
            action.allowed_write_roots.len(),
        );
        Ok(SupervisorActionAdapterResult {
            status,
            summary,
            worker_id,
            adapter_id: "codex-local-authorized-dispatch".to_string(),
            evidence_present,
            dispatch_ref: value
                .get("dispatch_id")
                .and_then(Value::as_str)
                .map(str::to_string),
            readback_ref,
            audit_ref: Some(binding_ref),
        })
    }
}

const UNVERIFIED_WORKER_MESSAGE_EVIDENCE: &str = "（worker 未附证据，见 worker 最后消息 json 块）";

fn worker_report_outcome(
    value: &Value,
    expected_worker_id: &str,
) -> Result<(String, String, bool, Option<String>), String> {
    let worker_id = required_worker_report_string(value, "worker_id")?;
    if worker_id != expected_worker_id {
        return Err(format!(
            "report_invalid: 回程 worker_id 与当前动作不一致（expected {expected_worker_id}, actual {worker_id}）。"
        ));
    }
    let acceptance_status = required_worker_report_string(value, "acceptance_status")?;
    let summary = required_worker_report_string(value, "summary")?;
    match acceptance_status {
        "blocked" => Ok((
            "waiting_user".to_string(),
            format!("worker 回程明确 blocked：{summary}"),
            false,
            None,
        )),
        "reported_completed" => {
            let executed_what = required_worker_report_string(value, "executed_what")?;
            let changed_what = required_worker_report_string(value, "changed_what")?;
            let evidence_refs = value
                .get("evidence_refs")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    "report_invalid: reported_completed 回程缺少 evidence_refs 数组。".to_string()
                })?;
            if evidence_refs.is_empty()
                || evidence_refs.iter().any(|entry| {
                    entry.as_str().is_none_or(|reference| {
                        let reference = reference.trim();
                        reference.is_empty() || reference == UNVERIFIED_WORKER_MESSAGE_EVIDENCE
                    })
                })
            {
                return Err(
                    "report_invalid: reported_completed 回程必须给出已验证的非空 evidence_refs。"
                        .to_string(),
                );
            }
            // 把 evidence 与 findings 的**实际内容**带给主管 LM，而非只给一个 evidence_present 布尔。
            // 站 3b attempt-3/4 实证：控制核心桥只回 summary+evidence_present 时，主管看不到引用/行号/
            // 逐条判定，无法抽核 → 只能 follow_up（撞授权闸、停 waiting_user）。评审证据必须上桥面。
            let evidence_text = join_worker_report_strings(evidence_refs);
            let findings_text = value
                .get("findings")
                .and_then(Value::as_array)
                .map(|findings| join_worker_report_strings(findings))
                .unwrap_or_default();
            let mut detail = format!(
                "合法 worker 回程：{summary}；执行：{executed_what}；改动：{changed_what}；证据：{evidence_text}"
            );
            if !findings_text.is_empty() {
                detail.push_str(&format!("；结论逐条：{findings_text}"));
            }
            Ok((
                "completed".to_string(),
                detail,
                true,
                Some(format!("worker-report:{worker_id}")),
            ))
        }
        "reported_not_completed" | "needs_rework" => Ok((
            "waiting_user".to_string(),
            format!("worker 回程未满足验收（{acceptance_status}）：{summary}"),
            false,
            None,
        )),
        _ => Err(format!(
            "report_invalid: acceptance_status 不在冻结枚举内：{acceptance_status}。"
        )),
    }
}

/// 把 worker 回程数组（evidence_refs / findings）里的字符串条目拼成一段可读证据文本，
/// 供主管 LM 逐条抽核。非字符串或空白条目跳过；用「｜」分隔保留条目边界。
fn join_worker_report_strings(entries: &[Value]) -> String {
    entries
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>()
        .join("｜")
}

fn required_worker_report_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("report_invalid: worker 回程缺少非空字段 {field}。"))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SupervisorActionStore {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    revision: i64,
    #[serde(default)]
    updated_at_ms: i64,
    #[serde(default)]
    actions: Vec<SupervisorActionRecordV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SupervisorActionRecordV1 {
    action_id: String,
    idempotency_key: String,
    run_id: String,
    project_id: String,
    workflow_id: String,
    authorization_id: String,
    authorization_snapshot_hash: String,
    workflow_revision_before: i64,
    workflow_revision_after: Option<i64>,
    task_package_fingerprint: String,
    kind: String,
    target_node_id: Option<String>,
    target_work_item_id: Option<String>,
    worker_id: Option<String>,
    reason: String,
    expected_result: String,
    received_at_ms: i64,
    validation_result: String,
    execution_status: String,
    adapter_id: Option<String>,
    dispatch_ref: Option<String>,
    readback_ref: Option<String>,
    audit_ref: Option<String>,
    summary: String,
    evidence_present: bool,
}

pub(crate) fn execute_supervisor_action(
    runtime: &SupervisorActionRuntime,
    proposal: SupervisorActionProposalV1,
    adapter: &dyn SupervisorActionAdapter,
) -> Result<SupervisorActionResultV1, String> {
    let idempotency_key = action_idempotency_key(runtime, &proposal)?;
    if let Some(result) = prior_or_recover_result(runtime, &idempotency_key)? {
        return Ok(result);
    }
    let guard = match guard_action(runtime, &proposal) {
        Ok(guard) => guard,
        Err(error) => return record_guard_rejection(runtime, &proposal, &error),
    };
    if !adapter.supports(&proposal.action) {
        return record_rejected_action(
            runtime,
            &proposal,
            &guard,
            idempotency_key,
            "denied_scope",
            "当前受控 adapter 不支持该主管动作，已拒绝执行。",
        );
    }
    if action_count(runtime)? >= supervisor_action_limit(&runtime.quota_limits) {
        return record_rejected_action(
            runtime,
            &proposal,
            &guard,
            idempotency_key,
            "quota_exceeded",
            "主管动作数已达本次运行上限，等待用户决定。",
        );
    }
    if proposal.action.name() == "finalize"
        && matches!(
            &proposal.action,
            SupervisorActionKind::Finalize {
                verdict: SupervisorFinalizeVerdict::Pass
            }
        )
        && !has_prior_worker_evidence(runtime)?
    {
        return record_rejected_action(
            runtime,
            &proposal,
            &guard,
            idempotency_key,
            "denied_scope",
            "finalize: pass 缺少权威 worker report / readback 证据，已拒绝终标。",
        );
    }
    if let SupervisorActionKind::ReportUser { message } = &proposal.action {
        if let Err(error) = reject_impersonated_user_decision(message) {
            return record_rejected_action(
                runtime,
                &proposal,
                &guard,
                idempotency_key,
                "protocol_invalid",
                &error,
            );
        }
    }
    let action_id = format!(
        "supervisor-action:{}:{}",
        crate::stable_id(&runtime.run_id),
        crate::unix_timestamp_nanos()
    );
    reserve_action(runtime, &proposal, &guard, &action_id, &idempotency_key)?;
    let action = AuthorizedSupervisorAction {
        runtime: runtime.clone(),
        proposal: proposal.clone(),
        allowed_read_roots: guard.allowed_read_roots,
        allowed_write_roots: guard.allowed_write_roots,
        authorization_snapshot_hash: guard.authorization_snapshot_hash,
        task_package_fingerprint: guard.task_package_fingerprint,
    };
    let adapter_result = match adapter.execute(&action) {
        Ok(result) => result,
        Err(error) => SupervisorActionAdapterResult {
            status: classify_adapter_error(&error).to_string(),
            summary: crate::run_error_translation::humanize_error_for_display(&error),
            worker_id: match &proposal.action {
                SupervisorActionKind::InspectWorker { worker_id }
                | SupervisorActionKind::WaitWorker { worker_id }
                | SupervisorActionKind::FollowUpWorker { worker_id, .. } => Some(worker_id.clone()),
                _ => None,
            },
            adapter_id: "codex-local-authorized-dispatch".to_string(),
            evidence_present: false,
            dispatch_ref: None,
            readback_ref: None,
            audit_ref: None,
        },
    };
    let revision_after = workflow_revision(&runtime.workflow_state_path)?;
    complete_action(runtime, &action_id, revision_after, &adapter_result)
}

pub(crate) fn execute_supervisor_last_message(
    runtime: &SupervisorActionRuntime,
    last_message: &str,
    adapter: &dyn SupervisorActionAdapter,
) -> Result<SupervisorActionResultV1, String> {
    match parse_supervisor_action_proposal(last_message) {
        Ok(proposal) => execute_supervisor_action(runtime, proposal, adapter),
        Err(error) => record_supervisor_protocol_invalid(runtime, &error),
    }
}

pub(crate) fn record_supervisor_transport_failure(
    runtime: &SupervisorActionRuntime,
    summary: &str,
) -> Result<SupervisorActionResultV1, String> {
    let revision = workflow_revision(&runtime.workflow_state_path)?;
    let action_id = format!(
        "supervisor-action:{}:{}",
        crate::stable_id(&runtime.run_id),
        crate::unix_timestamp_nanos()
    );
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.clone()),
        status: "transport_failed".to_string(),
        summary: summary.to_string(),
        worker_id: None,
        adapter_id: None,
        evidence_present: false,
    };
    update_store(
        &runtime.workflow_state_path,
        "record-transport-failure",
        |store| {
            store.actions.push(SupervisorActionRecordV1 {
                action_id,
                idempotency_key: format!(
                    "transport:{}:{}",
                    runtime.run_id,
                    crate::unix_timestamp_nanos()
                ),
                run_id: runtime.run_id.clone(),
                project_id: crate::project_id(&runtime.project_root),
                workflow_id: runtime.workflow_id.clone(),
                authorization_id: runtime.authorization_id.clone(),
                authorization_snapshot_hash: String::new(),
                workflow_revision_before: revision,
                workflow_revision_after: Some(revision),
                task_package_fingerprint: String::new(),
                kind: "system_transport".to_string(),
                target_node_id: None,
                target_work_item_id: None,
                worker_id: None,
                reason: "主管进程未返回可执行动作。".to_string(),
                expected_result: "记录诚实失败状态。".to_string(),
                received_at_ms: crate::unix_timestamp_ms(),
                validation_result: "system_failure".to_string(),
                execution_status: result.status.clone(),
                adapter_id: None,
                dispatch_ref: None,
                readback_ref: None,
                audit_ref: None,
                summary: result.summary.clone(),
                evidence_present: false,
            });
            Ok(())
        },
    )?;
    Ok(result)
}

pub(crate) fn record_supervisor_action_quota_exceeded(
    runtime: &SupervisorActionRuntime,
    summary: &str,
) -> Result<SupervisorActionResultV1, String> {
    record_supervisor_system_result(runtime, "quota_exceeded", "system_quota", summary)
}

pub(crate) fn record_supervisor_protocol_invalid(
    runtime: &SupervisorActionRuntime,
    summary: &str,
) -> Result<SupervisorActionResultV1, String> {
    record_supervisor_system_result(
        runtime,
        "protocol_invalid",
        "system_protocol_invalid",
        summary,
    )
}

fn record_supervisor_system_result(
    runtime: &SupervisorActionRuntime,
    status: &str,
    kind: &str,
    summary: &str,
) -> Result<SupervisorActionResultV1, String> {
    let revision = workflow_revision(&runtime.workflow_state_path)?;
    let action_id = format!(
        "supervisor-action:{}:{}",
        crate::stable_id(&runtime.run_id),
        crate::unix_timestamp_nanos()
    );
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.clone()),
        status: status.to_string(),
        summary: summary.to_string(),
        worker_id: None,
        adapter_id: None,
        evidence_present: false,
    };
    update_store(
        &runtime.workflow_state_path,
        "record-system-result",
        |store| {
            store.actions.push(SupervisorActionRecordV1 {
                action_id,
                idempotency_key: format!(
                    "system:{}:{}:{}",
                    kind,
                    runtime.run_id,
                    crate::unix_timestamp_nanos()
                ),
                run_id: runtime.run_id.clone(),
                project_id: crate::project_id(&runtime.project_root),
                workflow_id: runtime.workflow_id.clone(),
                authorization_id: runtime.authorization_id.clone(),
                authorization_snapshot_hash: String::new(),
                workflow_revision_before: revision,
                workflow_revision_after: Some(revision),
                task_package_fingerprint: String::new(),
                kind: kind.to_string(),
                target_node_id: None,
                target_work_item_id: None,
                worker_id: None,
                reason: "控制核心停止主管动作循环。".to_string(),
                expected_result: "记录诚实系统状态。".to_string(),
                received_at_ms: crate::unix_timestamp_ms(),
                validation_result: status.to_string(),
                execution_status: status.to_string(),
                adapter_id: None,
                dispatch_ref: None,
                readback_ref: None,
                audit_ref: None,
                summary: result.summary.clone(),
                evidence_present: false,
            });
            Ok(())
        },
    )?;
    Ok(result)
}

struct ActionGuard {
    workflow_revision: i64,
    authorization_snapshot_hash: String,
    task_package_fingerprint: String,
    allowed_read_roots: Vec<String>,
    allowed_write_roots: Vec<String>,
}

fn guard_action(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
) -> Result<ActionGuard, String> {
    let config = mcp_config(runtime);
    crate::mcp::supervisor_orchestrator::ensure_control_core_run_binding(
        &config,
        &runtime.project_root,
        &runtime.workflow_id,
        &runtime.authorization_id,
    )?;
    let now = crate::unix_timestamp_ms();
    if now.saturating_sub(runtime.started_at_ms)
        > runtime
            .quota_limits
            .max_runtime_minutes
            .saturating_mul(60_000)
    {
        return Err("quota_exceeded: 主管总时长已耗尽，等待用户决定。".to_string());
    }
    let authorization_store =
        crate::plan_authorization_store::load_store(&runtime.workflow_state_path, now)?;
    let project_id = crate::project_id(&runtime.project_root);
    let authorization = authorization_store
        .authorizations
        .iter()
        .find(|authorization| {
            authorization.authorization_id == runtime.authorization_id
                && authorization.project_id == project_id
                && authorization.workflow_id == runtime.workflow_id
        })
        .ok_or_else(|| "authorization_stale: 当前 run 找不到原授权段。".to_string())?;
    if authorization.status != crate::PlanAuthorizationStatus::Active
        || authorization.user_confirmation.is_none()
        || authorization
            .expires_at_ms
            .is_some_and(|expires_at_ms| expires_at_ms < now)
    {
        return Err("authorization_stale: 授权段已撤销、过期或不再 active。".to_string());
    }
    let value = crate::read_workflow_state_value(&runtime.workflow_state_path)?;
    let workflow_revision = value
        .get("revision")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    if let Some(expected_revision) = latest_workflow_revision(runtime)? {
        if expected_revision != workflow_revision {
            return Err(format!(
                "authorization_stale: workflow revision 已漂移（expected {expected_revision}, actual {workflow_revision}）。"
            ));
        }
    }
    let (target_node_id, target_work_item_id) = match &proposal.action {
        SupervisorActionKind::DispatchWorker { target } => {
            ensure_unique_prepared_dispatch(
                &value,
                &runtime.project_root,
                &runtime.workflow_id,
                &runtime.authorization_id,
                &target.node_id,
                &target.work_item_id,
            )?;
            (
                Some(target.node_id.clone()),
                Some(target.work_item_id.clone()),
            )
        }
        _ => (None, None),
    };
    let authorization_material = serde_json::to_string(authorization)
        .map_err(|error| format!("authorization_stale: 授权快照无法序列化：{error}"))?;
    let authorization_snapshot_hash = crate::utils::hash::sha256_hex(&authorization_material);
    let prior_identity = prior_run_identity(runtime)?;
    if let Some(prior_hash) = prior_identity.authorization_snapshot_hash.as_deref() {
        if prior_hash != authorization_snapshot_hash {
            return Err(
                "authorization_stale: 当前 run 的授权快照已漂移，拒绝继续执行。".to_string(),
            );
        }
    }
    let task_package_fingerprint = if let Some(work_item_id) = target_work_item_id.as_deref() {
        task_package_fingerprint(&value, &runtime.workflow_id, work_item_id)?
    } else {
        prior_identity.task_package_fingerprint.unwrap_or_default()
    };
    let _ = target_node_id;
    Ok(ActionGuard {
        workflow_revision,
        authorization_snapshot_hash,
        task_package_fingerprint,
        allowed_read_roots: authorization.scope.allowed_read_roots.clone(),
        allowed_write_roots: authorization.scope.allowed_write_roots.clone(),
    })
}

fn ensure_unique_prepared_dispatch(
    value: &Value,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    node_id: &str,
    work_item_id: &str,
) -> Result<(), String> {
    let expected_project_id = crate::project_id(project_root);
    let count = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|dispatch| {
            crate::optional_string_from(dispatch, "state").as_deref() == Some("prepared")
                && crate::optional_string_from(dispatch, "prompt_kind").as_deref()
                    == Some("authorized_prepared_auto_dispatch")
                && crate::optional_string_from(dispatch, "project_id").as_deref()
                    == Some(expected_project_id.as_str())
                && crate::optional_string_from(dispatch, "workflow_id").as_deref()
                    == Some(workflow_id)
                && crate::optional_string_from(dispatch, "plan_authorization_id").as_deref()
                    == Some(authorization_id)
                && crate::optional_string_from(dispatch, "node_id").as_deref() == Some(node_id)
                && crate::optional_string_from(dispatch, "work_item_id").as_deref()
                    == Some(work_item_id)
        })
        .count();
    if count != 1 {
        return Err(format!(
            "denied_scope: 当前授权下目标 prepared dispatch 数量为 {count}，拒绝派发。"
        ));
    }
    Ok(())
}

fn task_package_fingerprint(
    value: &Value,
    workflow_id: &str,
    work_item_id: &str,
) -> Result<String, String> {
    let work_item = crate::find_work_item(value, workflow_id, work_item_id)
        .ok_or_else(|| "denied_scope: 目标 work item 不存在，无法冻结任务包指纹。".to_string())?;
    let artifact = crate::find_task_package_artifact_by_id(value, work_item_id);
    let material = serde_json::to_string(&(work_item, artifact))
        .map_err(|error| format!("denied_scope: 任务包无法序列化：{error}"))?;
    Ok(crate::utils::hash::sha256_hex(&material))
}

#[derive(Default)]
struct PriorRunIdentity {
    authorization_snapshot_hash: Option<String>,
    task_package_fingerprint: Option<String>,
}

fn prior_run_identity(runtime: &SupervisorActionRuntime) -> Result<PriorRunIdentity, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    let mut identity = PriorRunIdentity::default();
    for record in store.actions.iter().filter(|record| {
        record.run_id == runtime.run_id
            && record.authorization_id == runtime.authorization_id
            && record.validation_result == "accepted"
    }) {
        if identity.authorization_snapshot_hash.is_none()
            && !record.authorization_snapshot_hash.trim().is_empty()
        {
            identity.authorization_snapshot_hash = Some(record.authorization_snapshot_hash.clone());
        }
        if record.kind == "dispatch_worker"
            && identity.task_package_fingerprint.is_none()
            && !record.task_package_fingerprint.trim().is_empty()
        {
            identity.task_package_fingerprint = Some(record.task_package_fingerprint.clone());
        }
    }
    Ok(identity)
}

fn reserve_action(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
    guard: &ActionGuard,
    action_id: &str,
    idempotency_key: &str,
) -> Result<(), String> {
    if let Some(repository) = crate::workbench_sqlite_storage_mode::primary_repository_for_write(
        &runtime.workflow_state_path,
    )? {
        return reserve_action_db_primary(
            runtime,
            proposal,
            guard,
            action_id,
            idempotency_key,
            &repository,
        );
    }
    let (target_node_id, target_work_item_id) = match &proposal.action {
        SupervisorActionKind::DispatchWorker { target } => (
            Some(target.node_id.clone()),
            Some(target.work_item_id.clone()),
        ),
        _ => (None, None),
    };
    update_store(&runtime.workflow_state_path, "reserve-action", |store| {
        if let (Some(node_id), Some(work_item_id)) =
            (target_node_id.as_deref(), target_work_item_id.as_deref())
        {
            let duplicate = store.actions.iter().any(|record| {
                record.kind == "dispatch_worker"
                    && record.validation_result == "accepted"
                    && record.authorization_id == runtime.authorization_id
                    && record.target_node_id.as_deref() == Some(node_id)
                    && record.target_work_item_id.as_deref() == Some(work_item_id)
            });
            if duplicate {
                return Err(format!(
                    "denied_scope: 同一 authorization + work item 已有派发 reservation 或结果，拒绝跨主管 run 重复启动 worker：{work_item_id}"
                ));
            }
        }
        store.actions.push(SupervisorActionRecordV1 {
            action_id: action_id.to_string(),
            idempotency_key: idempotency_key.to_string(),
            run_id: runtime.run_id.clone(),
            project_id: crate::project_id(&runtime.project_root),
            workflow_id: runtime.workflow_id.clone(),
            authorization_id: runtime.authorization_id.clone(),
            authorization_snapshot_hash: guard.authorization_snapshot_hash.clone(),
            workflow_revision_before: guard.workflow_revision,
            workflow_revision_after: None,
            task_package_fingerprint: guard.task_package_fingerprint.clone(),
            kind: proposal.action.name().to_string(),
            target_node_id,
            target_work_item_id,
            worker_id: None,
            reason: proposal.reason.clone(),
            expected_result: proposal.expected_result.clone(),
            received_at_ms: crate::unix_timestamp_ms(),
            validation_result: "accepted".to_string(),
            execution_status: "reserved".to_string(),
            adapter_id: None,
            dispatch_ref: None,
            readback_ref: None,
            audit_ref: None,
            summary: "控制核心已接受动作，等待受控 adapter 回传。".to_string(),
            evidence_present: false,
        });
        Ok(())
    })
}

fn reserve_action_db_primary(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
    guard: &ActionGuard,
    action_id: &str,
    idempotency_key: &str,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<(), String> {
    let (target_node_id, target_work_item_id) = match &proposal.action {
        SupervisorActionKind::DispatchWorker { target } => (
            Some(target.node_id.clone()),
            Some(target.work_item_id.clone()),
        ),
        _ => (None, None),
    };
    let existing_store = load_store(&runtime.workflow_state_path)?;
    if let (Some(node_id), Some(work_item_id)) =
        (target_node_id.as_deref(), target_work_item_id.as_deref())
    {
        let duplicate = existing_store.actions.iter().any(|record| {
            record.kind == "dispatch_worker"
                && record.validation_result == "accepted"
                && record.authorization_id == runtime.authorization_id
                && record.target_node_id.as_deref() == Some(node_id)
                && record.target_work_item_id.as_deref() == Some(work_item_id)
        });
        if duplicate {
            return Err(format!(
                "denied_scope: 同一 authorization + work item 已有派发 reservation 或结果，拒绝跨主管 run 重复启动 worker：{work_item_id}"
            ));
        }
    }
    let record = SupervisorActionRecordV1 {
        action_id: action_id.to_string(),
        idempotency_key: idempotency_key.to_string(),
        run_id: runtime.run_id.clone(),
        project_id: crate::project_id(&runtime.project_root),
        workflow_id: runtime.workflow_id.clone(),
        authorization_id: runtime.authorization_id.clone(),
        authorization_snapshot_hash: guard.authorization_snapshot_hash.clone(),
        workflow_revision_before: guard.workflow_revision,
        workflow_revision_after: None,
        task_package_fingerprint: guard.task_package_fingerprint.clone(),
        kind: proposal.action.name().to_string(),
        target_node_id,
        target_work_item_id,
        worker_id: None,
        reason: proposal.reason.clone(),
        expected_result: proposal.expected_result.clone(),
        received_at_ms: crate::unix_timestamp_ms(),
        validation_result: "accepted".to_string(),
        execution_status: "reserved".to_string(),
        adapter_id: None,
        dispatch_ref: None,
        readback_ref: None,
        audit_ref: None,
        summary: "控制核心已接受动作，等待受控 adapter 回传。".to_string(),
        evidence_present: false,
    };
    let record_value = serde_json::to_value(&record)
        .map_err(|error| format!("序列化主管动作 DB 主写记录失败：{error}"))?;
    let audit_timestamp = crate::unix_timestamp_string();
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "supervisor-action-reserved",
        action_id,
        &audit_timestamp,
    );
    let audit_event = json!({
        "event_id": audit_event_id.clone(),
        "event_type": "supervisor_action_reserved",
        "target_ref": action_id,
        "actor_ref": "supervisor_action_controller",
        "source_kind": "workspace_state",
        "permission_level": "authorized_supervisor_execution",
        "before_state": "none",
        "after_state": "reserved",
        "created_at": audit_timestamp,
        "reason": "主管动作已由控制核心保留，尚未调用 adapter。"
    });
    let reservation = repository.reserve_supervisor_action_with_audit(
        &record_value,
        &crate::workbench_sqlite_repository::RepositoryAuditEntry {
            event_id: audit_event_id,
            target_kind: "supervisor_action".to_string(),
            target_id: action_id.to_string(),
            payload: audit_event,
        },
        None,
    )?;
    if reservation.already_reserved {
        crate::workbench_sqlite_storage_mode::block_db_primary_writes(
            &runtime.workflow_state_path,
            "supervisor_action_reservation",
            format!("already_reserved:{}", reservation.action_id),
        );
        return Err(format!(
            "db_primary_supervisor_action_already_reserved:{}: refusing adapter replay before startup reconciliation",
            reservation.action_id
        ));
    }
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        &runtime.workflow_state_path,
        "supervisor_action_reservation",
        || {
            update_store(&runtime.workflow_state_path, "reserve-action", |store| {
                if let (Some(node_id), Some(work_item_id)) = (
                    record.target_node_id.as_deref(),
                    record.target_work_item_id.as_deref(),
                ) {
                    let duplicate = store.actions.iter().any(|existing| {
                        existing.kind == "dispatch_worker"
                            && existing.validation_result == "accepted"
                            && existing.authorization_id == runtime.authorization_id
                            && existing.target_node_id.as_deref() == Some(node_id)
                            && existing.target_work_item_id.as_deref() == Some(work_item_id)
                    });
                    if duplicate {
                        return Err(format!(
                            "denied_scope: 同一 authorization + work item 已有派发 reservation 或结果，拒绝跨主管 run 重复启动 worker：{work_item_id}"
                        ));
                    }
                }
                store.actions.push(record.clone());
                Ok(())
            })
        },
    )
}

fn complete_action(
    runtime: &SupervisorActionRuntime,
    action_id: &str,
    workflow_revision_after: i64,
    adapter_result: &SupervisorActionAdapterResult,
) -> Result<SupervisorActionResultV1, String> {
    if let Some(repository) = crate::workbench_sqlite_storage_mode::primary_repository_for_write(
        &runtime.workflow_state_path,
    )? {
        return complete_action_db_primary(
            runtime,
            action_id,
            workflow_revision_after,
            adapter_result,
            &repository,
        );
    }
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.to_string()),
        status: adapter_result.status.clone(),
        summary: adapter_result.summary.clone(),
        worker_id: adapter_result.worker_id.clone(),
        adapter_id: Some(adapter_result.adapter_id.clone()),
        evidence_present: adapter_result.evidence_present,
    };
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        &runtime.workflow_state_path,
        "supervisor_action_completion",
        || {
            update_store(&runtime.workflow_state_path, "complete-action", |store| {
                let record = store
                    .actions
                    .iter_mut()
                    .find(|record| record.action_id == action_id)
                    .ok_or_else(|| {
                        "主管动作 reservation 丢失，拒绝写回 adapter 结果。".to_string()
                    })?;
                record.workflow_revision_after = Some(workflow_revision_after);
                record.execution_status = result.status.clone();
                record.summary = result.summary.clone();
                record.worker_id = result.worker_id.clone();
                record.adapter_id = result.adapter_id.clone();
                record.dispatch_ref = adapter_result.dispatch_ref.clone();
                record.readback_ref = adapter_result.readback_ref.clone();
                record.audit_ref = adapter_result.audit_ref.clone();
                record.evidence_present = result.evidence_present;
                Ok(())
            })
        },
    )?;
    Ok(result)
}

fn complete_action_db_primary(
    runtime: &SupervisorActionRuntime,
    action_id: &str,
    workflow_revision_after: i64,
    adapter_result: &SupervisorActionAdapterResult,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<SupervisorActionResultV1, String> {
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.to_string()),
        status: adapter_result.status.clone(),
        summary: adapter_result.summary.clone(),
        worker_id: adapter_result.worker_id.clone(),
        adapter_id: Some(adapter_result.adapter_id.clone()),
        evidence_present: adapter_result.evidence_present,
    };
    let audit_timestamp = crate::unix_timestamp_string();
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "supervisor-action-completed",
        action_id,
        &audit_timestamp,
    );
    let audit_event = json!({
        "event_id": audit_event_id.clone(),
        "event_type": "supervisor_action_completed",
        "target_ref": action_id,
        "actor_ref": "supervisor_action_controller",
        "source_kind": "workspace_state",
        "permission_level": "authorized_supervisor_execution",
        "before_state": "reserved",
        "after_state": result.status,
        "created_at": audit_timestamp,
        "reason": result.summary
    });
    let db_result = json!({
        "status": result.status,
        "summary": result.summary,
        "worker_id": result.worker_id,
        "adapter_id": result.adapter_id,
        "evidence_present": result.evidence_present,
        "dispatch_ref": adapter_result.dispatch_ref,
        "readback_ref": adapter_result.readback_ref,
        "audit_ref": adapter_result.audit_ref,
        "workflow_revision_after": workflow_revision_after,
    });
    repository.complete_supervisor_action_with_audit(
        action_id,
        &db_result,
        &crate::workbench_sqlite_repository::RepositoryAuditEntry {
            event_id: audit_event_id,
            target_kind: "supervisor_action".to_string(),
            target_id: action_id.to_string(),
            payload: audit_event,
        },
        None,
    )?;
    update_store(&runtime.workflow_state_path, "complete-action", |store| {
        let record = store
            .actions
            .iter_mut()
            .find(|record| record.action_id == action_id)
            .ok_or_else(|| "主管动作 reservation 丢失，拒绝写回 adapter 结果。".to_string())?;
        record.workflow_revision_after = Some(workflow_revision_after);
        record.execution_status = result.status.clone();
        record.summary = result.summary.clone();
        record.worker_id = result.worker_id.clone();
        record.adapter_id = result.adapter_id.clone();
        record.dispatch_ref = adapter_result.dispatch_ref.clone();
        record.readback_ref = adapter_result.readback_ref.clone();
        record.audit_ref = adapter_result.audit_ref.clone();
        record.evidence_present = result.evidence_present;
        Ok(())
    })?;
    Ok(result)
}

fn record_rejected_action(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
    guard: &ActionGuard,
    idempotency_key: String,
    status: &str,
    summary: &str,
) -> Result<SupervisorActionResultV1, String> {
    let action_id = format!(
        "supervisor-action:{}:{}",
        crate::stable_id(&runtime.run_id),
        crate::unix_timestamp_nanos()
    );
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.clone()),
        status: status.to_string(),
        summary: summary.to_string(),
        worker_id: None,
        adapter_id: None,
        evidence_present: false,
    };
    update_store(&runtime.workflow_state_path, "reject-action", |store| {
        store.actions.push(SupervisorActionRecordV1 {
            action_id,
            idempotency_key,
            run_id: runtime.run_id.clone(),
            project_id: crate::project_id(&runtime.project_root),
            workflow_id: runtime.workflow_id.clone(),
            authorization_id: runtime.authorization_id.clone(),
            authorization_snapshot_hash: guard.authorization_snapshot_hash.clone(),
            workflow_revision_before: guard.workflow_revision,
            workflow_revision_after: Some(guard.workflow_revision),
            task_package_fingerprint: guard.task_package_fingerprint.clone(),
            kind: proposal.action.name().to_string(),
            target_node_id: None,
            target_work_item_id: None,
            worker_id: None,
            reason: proposal.reason.clone(),
            expected_result: proposal.expected_result.clone(),
            received_at_ms: crate::unix_timestamp_ms(),
            validation_result: status.to_string(),
            execution_status: status.to_string(),
            adapter_id: None,
            dispatch_ref: None,
            readback_ref: None,
            audit_ref: None,
            summary: summary.to_string(),
            evidence_present: false,
        });
        Ok(())
    })?;
    Ok(result)
}

fn record_guard_rejection(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
    error: &str,
) -> Result<SupervisorActionResultV1, String> {
    let revision = workflow_revision(&runtime.workflow_state_path).unwrap_or_default();
    let status = if error.contains("quota_exceeded") {
        "quota_exceeded"
    } else if error.contains("authorization_stale") {
        "authorization_stale"
    } else {
        "denied_scope"
    };
    let action_id = format!(
        "supervisor-action:{}:{}",
        crate::stable_id(&runtime.run_id),
        crate::unix_timestamp_nanos()
    );
    let result = SupervisorActionResultV1 {
        action_id: Some(action_id.clone()),
        status: status.to_string(),
        summary: error.to_string(),
        worker_id: None,
        adapter_id: None,
        evidence_present: false,
    };
    update_store(
        &runtime.workflow_state_path,
        "reject-guard-action",
        |store| {
            store.actions.push(SupervisorActionRecordV1 {
                action_id,
                idempotency_key: format!(
                    "guard:{}:{}",
                    runtime.run_id,
                    crate::unix_timestamp_nanos()
                ),
                run_id: runtime.run_id.clone(),
                project_id: crate::project_id(&runtime.project_root),
                workflow_id: runtime.workflow_id.clone(),
                authorization_id: runtime.authorization_id.clone(),
                authorization_snapshot_hash: String::new(),
                workflow_revision_before: revision,
                workflow_revision_after: Some(revision),
                task_package_fingerprint: String::new(),
                kind: proposal.action.name().to_string(),
                target_node_id: None,
                target_work_item_id: None,
                worker_id: None,
                reason: proposal.reason.clone(),
                expected_result: proposal.expected_result.clone(),
                received_at_ms: crate::unix_timestamp_ms(),
                validation_result: status.to_string(),
                execution_status: status.to_string(),
                adapter_id: None,
                dispatch_ref: None,
                readback_ref: None,
                audit_ref: None,
                summary: error.to_string(),
                evidence_present: false,
            });
            Ok(())
        },
    )?;
    Ok(result)
}

fn prior_or_recover_result(
    runtime: &SupervisorActionRuntime,
    idempotency_key: &str,
) -> Result<Option<SupervisorActionResultV1>, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    let Some(record) = store
        .actions
        .iter()
        .find(|record| record.run_id == runtime.run_id && record.idempotency_key == idempotency_key)
        .cloned()
    else {
        return Ok(None);
    };
    if record.execution_status != "reserved" {
        return Ok(Some(result_from_record(&record)));
    }

    // A prior process may have reached the external adapter before crashing. Never replay a
    // reservation whose completion was not durably recorded: the worker might already exist.
    let revision_after = workflow_revision(&runtime.workflow_state_path).ok();
    let recovered = update_store(
        &runtime.workflow_state_path,
        "recover-inflight-action",
        |store| {
            let record = store
                .actions
                .iter_mut()
                .find(|record| {
                    record.run_id == runtime.run_id && record.idempotency_key == idempotency_key
                })
                .ok_or_else(|| "主管动作 reservation 在恢复前丢失，拒绝重放。".to_string())?;
            if record.execution_status == "reserved" {
                record.execution_status = "waiting_user".to_string();
                record.summary = "检测到同一主管动作已 reservation 但未完成回写；为避免重复启动 worker，已停止自动重放，等待用户核对现有 worker。".to_string();
                if record.workflow_revision_after.is_none() {
                    record.workflow_revision_after = revision_after;
                }
            }
            Ok(record.clone())
        },
    )?;
    Ok(Some(result_from_record(&recovered)))
}

fn result_from_record(record: &SupervisorActionRecordV1) -> SupervisorActionResultV1 {
    SupervisorActionResultV1 {
        action_id: Some(record.action_id.clone()),
        status: record.execution_status.clone(),
        summary: record.summary.clone(),
        worker_id: record.worker_id.clone(),
        adapter_id: record.adapter_id.clone(),
        evidence_present: record.evidence_present,
    }
}

fn action_count(runtime: &SupervisorActionRuntime) -> Result<usize, String> {
    Ok(load_store(&runtime.workflow_state_path)?
        .actions
        .iter()
        .filter(|record| record.run_id == runtime.run_id)
        .count())
}

fn latest_workflow_revision(runtime: &SupervisorActionRuntime) -> Result<Option<i64>, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    Ok(store
        .actions
        .iter()
        .rev()
        .find(|record| record.run_id == runtime.run_id)
        .and_then(|record| record.workflow_revision_after))
}

fn has_prior_worker_evidence(runtime: &SupervisorActionRuntime) -> Result<bool, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    let mut evidence_is_fresh = BTreeMap::<String, bool>::new();
    for record in store
        .actions
        .iter()
        .filter(|record| record.run_id == runtime.run_id)
    {
        let Some(worker_id) = record.worker_id.as_ref() else {
            continue;
        };
        match record.kind.as_str() {
            "dispatch_worker" if record.validation_result == "accepted" => {
                evidence_is_fresh.insert(worker_id.clone(), false);
            }
            // 追问一旦交给 adapter，worker 会话就可能已经产生了新事实；无论回程最终成功、
            // 失败还是只完成 reservation，旧 inspect 都不能继续支撑 PASS。
            "follow_up_worker" if record.validation_result == "accepted" => {
                evidence_is_fresh.insert(worker_id.clone(), false);
            }
            "inspect_worker"
                if record.execution_status == "completed" && record.evidence_present =>
            {
                evidence_is_fresh.insert(worker_id.clone(), true);
            }
            _ => {}
        }
    }
    Ok(!evidence_is_fresh.is_empty() && evidence_is_fresh.values().all(|fresh| *fresh))
}

fn action_idempotency_key(
    runtime: &SupervisorActionRuntime,
    proposal: &SupervisorActionProposalV1,
) -> Result<String, String> {
    let target = match &proposal.action {
        SupervisorActionKind::DispatchWorker { target } => json!({
            "node_id": target.node_id,
            "work_item_id": target.work_item_id,
        }),
        SupervisorActionKind::InspectWorker { worker_id } => json!({
            "worker_id": worker_id,
            "worker_generation": latest_worker_generation(runtime, worker_id)?,
        }),
        SupervisorActionKind::WaitWorker { worker_id } => json!({"worker_id": worker_id}),
        SupervisorActionKind::FollowUpWorker { worker_id, prompt } => json!({
            "worker_id": worker_id,
            "prompt_sha256": crate::utils::hash::sha256_hex(prompt),
        }),
        SupervisorActionKind::Finalize { verdict } => json!({
            "verdict": verdict.as_str(),
            "evidence_generation": latest_worker_evidence_generation(runtime)?,
        }),
        SupervisorActionKind::ReportUser { message } => {
            json!({"message_sha256": crate::utils::hash::sha256_hex(message)})
        }
        SupervisorActionKind::RequestUserDecision { question } => {
            json!({"question_sha256": crate::utils::hash::sha256_hex(question)})
        }
    };
    let material = serde_json::to_string(&json!({
        "run_id": runtime.run_id,
        "project_root": runtime.project_root,
        "workflow_id": runtime.workflow_id,
        "authorization_id": runtime.authorization_id,
        "kind": proposal.action.name(),
        "target": target,
    }))
    .expect("supervisor action idempotency identity is serializable");
    Ok(format!(
        "supervisor-action:{}",
        crate::utils::hash::sha256_hex(&material)
    ))
}

fn latest_worker_generation(
    runtime: &SupervisorActionRuntime,
    worker_id: &str,
) -> Result<String, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    Ok(store
        .actions
        .iter()
        .rev()
        .find(|record| {
            record.run_id == runtime.run_id
                && record.worker_id.as_deref() == Some(worker_id)
                && record.validation_result == "accepted"
                && matches!(record.kind.as_str(), "dispatch_worker" | "follow_up_worker")
        })
        .map(|record| record.action_id.clone())
        .unwrap_or_else(|| "worker-generation:none".to_string()))
}

fn latest_worker_evidence_generation(runtime: &SupervisorActionRuntime) -> Result<String, String> {
    let store = load_store(&runtime.workflow_state_path)?;
    Ok(store
        .actions
        .iter()
        .rev()
        .find(|record| {
            record.run_id == runtime.run_id
                && record.kind == "inspect_worker"
                && record.execution_status == "completed"
                && record.evidence_present
        })
        .map(|record| record.action_id.clone())
        .unwrap_or_else(|| "worker-evidence:none".to_string()))
}

fn reject_impersonated_user_decision(message: &str) -> Result<(), String> {
    let lower = message.to_ascii_lowercase();
    if [
        "用户已取消",
        "用户确认",
        "用户决定",
        "user cancelled",
        "user confirmed",
    ]
    .iter()
    .any(|marker| lower.contains(&marker.to_ascii_lowercase()))
    {
        return Err("protocol_invalid: report_user 不得冒充用户决定、确认或取消。".to_string());
    }
    Ok(())
}

fn classify_adapter_error(error: &str) -> &'static str {
    if error.contains("report_invalid") {
        "report_invalid"
    } else if error.contains("授权")
        || error.contains("authorization")
        || error.contains("revision")
    {
        "authorization_stale"
    } else if error.contains("配额") || error.contains("quota") {
        "quota_exceeded"
    } else if error.contains("transport") || error.contains("cancel") || error.contains("timeout") {
        "transport_failed"
    } else {
        "adapter_failed"
    }
}

fn compact_value_summary(value: &Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "<unserializable>".to_string())
        .chars()
        .take(2_000)
        .collect()
}

fn mcp_config(runtime: &SupervisorActionRuntime) -> McpServerConfig {
    McpServerConfig {
        role: McpRole::SupervisorOrchestrator,
        run_id: runtime.run_id.clone(),
        node_id: None,
        supervisor_workflow_state_path: Some(runtime.workflow_state_path.clone()),
        supervisor_quota_limits: Some(runtime.quota_limits),
    }
}

fn workflow_revision(path: &Path) -> Result<i64, String> {
    Ok(crate::read_workflow_state_value(path)?
        .get("revision")
        .and_then(Value::as_i64)
        .unwrap_or_default())
}

fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    crate::utils::store_paths::sidecar_path(
        workflow_state_path,
        SIDECAR_NAME,
        "supervisor action control",
    )
}

fn load_store(workflow_state_path: &Path) -> Result<SupervisorActionStore, String> {
    let path = sidecar_path(workflow_state_path)?;
    if !path.exists() {
        return Ok(SupervisorActionStore {
            schema_version: STORE_SCHEMA_VERSION.to_string(),
            revision: 0,
            updated_at_ms: crate::unix_timestamp_ms(),
            actions: vec![],
        });
    }
    let store: SupervisorActionStore = serde_json::from_slice(
        &fs::read(&path)
            .map_err(|error| format!("读取主管动作账本失败 {}：{error}", path.display()))?,
    )
    .map_err(|error| format!("主管动作账本损坏，拒绝覆盖 {}：{error}", path.display()))?;
    if store.schema_version != STORE_SCHEMA_VERSION || store.revision < 0 {
        return Err("主管动作账本 schema 或 revision 非法，拒绝覆盖。".to_string());
    }
    Ok(store)
}

pub(crate) fn db_primary_projection_records(
    workflow_state_path: &Path,
) -> Result<Vec<Value>, String> {
    load_store(workflow_state_path)?
        .actions
        .into_iter()
        .map(|record| {
            serde_json::to_value(record)
                .map_err(|error| format!("主管动作账本投影序列化失败：{error}"))
        })
        .collect()
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    actions: &[Value],
    replace_db_primary_leading: bool,
    write_id: &str,
) -> Result<usize, String> {
    if actions.is_empty() {
        return Ok(0);
    }
    let path = sidecar_path(workflow_state_path)?;
    let parent = path
        .parent()
        .ok_or_else(|| "主管动作账本缺父目录。".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建主管动作账本目录失败 {}：{error}", parent.display()))?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let mut store = load_store(workflow_state_path)?;
    let mut changes = 0_i64;

    for value in actions {
        let action: SupervisorActionRecordV1 = serde_json::from_value(value.clone())
            .map_err(|error| format!("DB 主管动作投影记录无法解析：{error}"))?;
        if let Some(index) = store
            .actions
            .iter()
            .position(|existing| existing.action_id == action.action_id)
        {
            if serde_json::to_value(&store.actions[index])
                .map_err(|error| format!("主管动作账本投影序列化失败：{error}"))?
                != value.clone()
            {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:supervisor_actions:{}",
                        action.action_id
                    ));
                }
                store.actions[index] = action;
                changes += 1;
            }
        } else {
            store.actions.push(action);
            changes += 1;
        }
    }

    if changes == 0 {
        return Ok(0);
    }
    store.revision = store
        .revision
        .checked_add(changes)
        .ok_or_else(|| "主管动作账本 revision 已到上限。".to_string())?;
    store.updated_at_ms = crate::unix_timestamp_ms();
    let temporary = parent.join(format!(
        ".{SIDECAR_NAME}.{}.tmp",
        crate::stable_id(write_id)
    ));
    let serialized = serde_json::to_vec_pretty(&store)
        .map_err(|error| format!("序列化主管动作账本失败：{error}"))?;
    let mut file = fs::File::create(&temporary)
        .map_err(|error| format!("创建主管动作账本临时文件失败：{error}"))?;
    file.write_all(&serialized)
        .map_err(|error| format!("写入主管动作账本临时文件失败：{error}"))?;
    file.sync_all()
        .map_err(|error| format!("同步主管动作账本临时文件失败：{error}"))?;
    fs::rename(&temporary, &path).map_err(|error| format!("原子替换主管动作账本失败：{error}"))?;
    Ok(changes as usize)
}

fn update_store<R>(
    workflow_state_path: &Path,
    write_id: &str,
    update: impl FnOnce(&mut SupervisorActionStore) -> Result<R, String>,
) -> Result<R, String> {
    let path = sidecar_path(workflow_state_path)?;
    let parent = path
        .parent()
        .ok_or_else(|| "主管动作账本缺父目录。".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建主管动作账本目录失败 {}：{error}", parent.display()))?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let mut store = load_store(workflow_state_path)?;
    let result = update(&mut store)?;
    store.revision += 1;
    store.updated_at_ms = crate::unix_timestamp_ms();
    let temporary = parent.join(format!(
        ".{SIDECAR_NAME}.{}.tmp",
        crate::stable_id(write_id)
    ));
    let serialized = serde_json::to_vec_pretty(&store)
        .map_err(|error| format!("序列化主管动作账本失败：{error}"))?;
    let mut file = fs::File::create(&temporary)
        .map_err(|error| format!("创建主管动作账本临时文件失败：{error}"))?;
    file.write_all(&serialized)
        .map_err(|error| format!("写入主管动作账本临时文件失败：{error}"))?;
    file.sync_all()
        .map_err(|error| format!("同步主管动作账本临时文件失败：{error}"))?;
    fs::rename(&temporary, &path).map_err(|error| format!("原子替换主管动作账本失败：{error}"))?;
    Ok(result)
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        for retry in 0..=LOCK_RETRY_COUNT {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(mut file) => {
                    file.write_all(write_id.as_bytes())
                        .map_err(|error| format!("写入主管动作账本锁失败：{error}"))?;
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::AlreadyExists
                        && retry < LOCK_RETRY_COUNT =>
                {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err("supervisor_action_control_store_locked: 请稍后重试。".to_string());
                }
                Err(error) => return Err(format!("创建主管动作账本锁失败：{error}")),
            }
        }
        unreachable!("finite retry loop returns on its final attempt")
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::Cell;
    use std::sync::atomic::{AtomicU64, Ordering};

    const PROJECT: &str = "/Users/yoyi/codex-workflow-mario-test";
    const WORKFLOW: &str = "workflow:station3a";
    const AUTH: &str = "authorization:station3a";
    const NODE: &str = "workflow:station3a:node:worker";
    const WORK_ITEM: &str = "work-item:station3a";
    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        path: PathBuf,
        runtime: SupervisorActionRuntime,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "station3a-action-controller-{}-{}",
                crate::unix_timestamp_nanos(),
                FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&root).expect("fixture root");
            let path = root.join("workflow-state.json");
            fs::write(
                &path,
                json!({
                    "revision": 1,
                    "workflow_node_dispatches": [{
                        "state":"prepared", "prompt_kind":"authorized_prepared_auto_dispatch",
                        "project_id": crate::project_id(PROJECT), "workflow_id": WORKFLOW,
                        "plan_authorization_id": AUTH, "node_id": NODE, "work_item_id": WORK_ITEM
                    }],
                    "work_items": [{"workflow_id": WORKFLOW, "work_item_id": WORK_ITEM}],
                    "artifacts": []
                })
                .to_string(),
            )
            .expect("fixture state");
            let authorization = crate::PlanAuthorization {
                authorization_id: AUTH.to_string(),
                schema_version: "plan_authorization.v1".to_string(),
                project_id: crate::project_id(PROJECT),
                workflow_id: WORKFLOW.to_string(),
                source_proposal_id: None,
                title: "station3a".to_string(),
                goal_summary: "station3a".to_string(),
                status: crate::PlanAuthorizationStatus::Active,
                scope: crate::AuthorizedExecutionScope {
                    project_id: crate::project_id(PROJECT),
                    workflow_id: WORKFLOW.to_string(),
                    allowed_role_ids: vec!["worker".to_string()],
                    allowed_agent_ids: vec![],
                    allowed_read_roots: vec![PROJECT.to_string()],
                    allowed_write_roots: vec![PROJECT.to_string()],
                    allowed_tools: vec![
                        "read_file".to_string(),
                        "write_file".to_string(),
                        "apply_patch".to_string(),
                    ],
                    allowed_checks: vec![],
                    allowed_task_package_kinds: vec![],
                    max_worker_dispatches: Some(2),
                    max_runtime_minutes: Some(30),
                    stop_conditions: vec![],
                },
                user_confirmation: Some(crate::PlanAuthorizationUserConfirmation {
                    confirmed_by: "user".to_string(),
                    confirmed_at_ms: crate::unix_timestamp_ms(),
                    confirmation_summary: "station3a fixture confirmation".to_string(),
                }),
                global_boundary_review: None,
                audit_refs: vec![],
                created_at_ms: crate::unix_timestamp_ms(),
                updated_at_ms: crate::unix_timestamp_ms(),
                expires_at_ms: None,
            };
            let store = crate::PlanAuthorizationStoreV1 {
                schema_version: "plan_authorization_store.v1".to_string(),
                revision: 1,
                authorizations: vec![authorization],
                audit_events: vec![],
                updated_at_ms: crate::unix_timestamp_ms(),
                warnings: vec![],
            };
            fs::write(
                crate::plan_authorization_store::sidecar_path(&path).expect("auth path"),
                serde_json::to_vec(&store).expect("auth store"),
            )
            .expect("write auth store");
            let runtime = SupervisorActionRuntime {
                run_id: "supervisor:station3a:test".to_string(),
                project_root: PROJECT.to_string(),
                workflow_id: WORKFLOW.to_string(),
                authorization_id: AUTH.to_string(),
                workflow_state_path: path.clone(),
                quota_limits: SupervisorQuotaLimits {
                    max_active_workers: 2,
                    max_follow_ups_per_worker: 2,
                    max_runtime_minutes: 30,
                },
                started_at_ms: crate::unix_timestamp_ms(),
            };
            crate::mcp::supervisor_orchestrator::record_pilot_session_started(
                &mcp_config(&runtime),
                &crate::mcp::supervisor_orchestrator::SupervisorPilotSessionLaunch {
                    project_root: PROJECT.to_string(),
                    workflow_id: WORKFLOW.to_string(),
                    authorization_id: AUTH.to_string(),
                    model_id: "test".to_string(),
                    reasoning_effort: "medium".to_string(),
                    workbench_executable_path: "/tmp/codex-governance-workbench-test".to_string(),
                    workbench_build_id: "test-build".to_string(),
                    supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
                    supervisor_contract_sha256: "test-supervisor-contract-hash".to_string(),
                    worker_report_contract_sha256: "test-worker-contract-hash".to_string(),
                },
            )
            .expect("record session");
            Self {
                root,
                path,
                runtime,
            }
        }

        fn proposal(kind: &str) -> SupervisorActionProposalV1 {
            let value = match kind {
                "dispatch" => {
                    json!({"schema_version":"supervisor_action_proposal.v1","kind":"dispatch_worker","target":{"node_id":NODE,"work_item_id":WORK_ITEM},"reason":"准备完成","expected_result":"worker"})
                }
                "inspect" => {
                    json!({"schema_version":"supervisor_action_proposal.v1","kind":"inspect_worker","worker_id":"worker-1","reason":"读取口供","expected_result":"evidence"})
                }
                "follow_up" => {
                    json!({"schema_version":"supervisor_action_proposal.v1","kind":"follow_up_worker","worker_id":"worker-1","prompt":"补充证据","reason":"证据不足","expected_result":"fresh report"})
                }
                "finalize" => {
                    json!({"schema_version":"supervisor_action_proposal.v1","kind":"finalize","verdict":"pass","reason":"证据充分","expected_result":"advisory"})
                }
                "report" => {
                    json!({"schema_version":"supervisor_action_proposal.v1","kind":"report_user","message":"任务已完成，证据已回读。","reason":"收尾","expected_result":"用户报告"})
                }
                _ => unreachable!(),
            };
            parse_supervisor_action_proposal(&value.to_string()).expect("proposal")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct FakeAdapter {
        dispatches: Cell<usize>,
    }

    impl SupervisorActionAdapter for FakeAdapter {
        fn supports(&self, _action: &SupervisorActionKind) -> bool {
            true
        }

        fn execute(
            &self,
            action: &AuthorizedSupervisorAction,
        ) -> Result<SupervisorActionAdapterResult, String> {
            if matches!(
                &action.proposal.action,
                SupervisorActionKind::DispatchWorker { .. }
            ) {
                self.dispatches.set(self.dispatches.get() + 1);
            }
            let evidence_present = matches!(
                &action.proposal.action,
                SupervisorActionKind::InspectWorker { .. }
            );
            Ok(SupervisorActionAdapterResult {
                status: "completed".to_string(),
                summary: format!("fake {}", action.proposal.action.name()),
                worker_id: Some("worker-1".to_string()),
                adapter_id: "fake-adapter".to_string(),
                evidence_present,
                dispatch_ref: Some("dispatch:fake".to_string()),
                readback_ref: evidence_present.then(|| "readback:fake".to_string()),
                audit_ref: Some("audit:fake".to_string()),
            })
        }
    }

    struct RevisionAdvancingAdapter {
        dispatches: Cell<usize>,
    }

    impl SupervisorActionAdapter for RevisionAdvancingAdapter {
        fn supports(&self, action: &SupervisorActionKind) -> bool {
            matches!(action, SupervisorActionKind::DispatchWorker { .. })
        }

        fn execute(
            &self,
            action: &AuthorizedSupervisorAction,
        ) -> Result<SupervisorActionAdapterResult, String> {
            assert!(matches!(
                action.proposal.action,
                SupervisorActionKind::DispatchWorker { .. }
            ));
            self.dispatches.set(self.dispatches.get() + 1);
            let mut workflow: Value = serde_json::from_slice(
                &fs::read(&action.runtime.workflow_state_path).expect("workflow state"),
            )
            .expect("workflow json");
            let revision = workflow["revision"].as_i64().unwrap_or_default();
            workflow["revision"] = json!(revision + 1);
            fs::write(
                &action.runtime.workflow_state_path,
                serde_json::to_vec(&workflow).expect("workflow json"),
            )
            .expect("advance workflow revision");
            Ok(SupervisorActionAdapterResult {
                status: "waiting_worker".to_string(),
                summary: "fake worker launched".to_string(),
                worker_id: Some("worker-1".to_string()),
                adapter_id: "revision-advancing-adapter".to_string(),
                evidence_present: false,
                dispatch_ref: Some("dispatch:fake".to_string()),
                readback_ref: None,
                audit_ref: Some("audit:fake".to_string()),
            })
        }
    }

    #[test]
    fn station3a_fake_control_loop_records_each_authoritative_step_once() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        for kind in ["dispatch", "inspect", "finalize", "report"] {
            let fake_supervisor_last_message = serde_json::to_string(&Fixture::proposal(kind))
                .expect("serialize fake supervisor proposal");
            let result = execute_supervisor_last_message(
                &fixture.runtime,
                &fake_supervisor_last_message,
                &adapter,
            )
            .expect("control action");
            assert_eq!(result.status, "completed");
        }
        assert_eq!(adapter.dispatches.get(), 1);
        let store = load_store(&fixture.path).expect("action store");
        assert_eq!(store.actions.len(), 4);
        assert!(store
            .actions
            .iter()
            .all(|action| action.validation_result == "accepted"));
        assert!(store
            .actions
            .iter()
            .all(|action| action.adapter_id.is_some()));
        assert!(store
            .actions
            .iter()
            .all(|action| action.audit_ref.is_some()));
        let authorization_hash = &store.actions[0].authorization_snapshot_hash;
        let task_fingerprint = &store.actions[0].task_package_fingerprint;
        assert_eq!(authorization_hash.len(), 64);
        assert_eq!(task_fingerprint.len(), 64);
        assert!(store
            .actions
            .iter()
            .all(|action| &action.authorization_snapshot_hash == authorization_hash));
        assert!(store
            .actions
            .iter()
            .all(|action| &action.task_package_fingerprint == task_fingerprint));
    }

    #[test]
    fn station3b_follow_up_invalidates_old_inspect_until_new_report_is_inspected() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        for kind in ["dispatch", "inspect", "follow_up"] {
            execute_supervisor_action(&fixture.runtime, Fixture::proposal(kind), &adapter)
                .expect("control action");
        }
        assert!(!has_prior_worker_evidence(&fixture.runtime).expect("freshness check"));

        let rejected =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("finalize"), &adapter)
                .expect("old evidence must be rejected without adapter failure");
        assert_eq!(rejected.status, "denied_scope");

        let refreshed =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("inspect"), &adapter)
                .expect("new generation inspect");
        assert!(refreshed.evidence_present);
        assert!(has_prior_worker_evidence(&fixture.runtime).expect("freshness check"));

        let finalized =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("finalize"), &adapter)
                .expect("finalize after fresh inspect");
        assert_eq!(finalized.status, "completed");
    }

    #[test]
    fn station3a_rejects_authorization_snapshot_drift_after_dispatch() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        execute_supervisor_action(&fixture.runtime, Fixture::proposal("dispatch"), &adapter)
            .expect("dispatch");
        let auth_path = crate::plan_authorization_store::sidecar_path(&fixture.path)
            .expect("authorization path");
        let mut store =
            crate::plan_authorization_store::load_store(&fixture.path, crate::unix_timestamp_ms())
                .expect("authorization store");
        store.authorizations[0]
            .scope
            .allowed_checks
            .push("new-check".to_string());
        store.revision += 1;
        fs::write(
            auth_path,
            serde_json::to_vec(&store).expect("serialize authorization store"),
        )
        .expect("write changed authorization");

        let result =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("inspect"), &adapter)
                .expect("drift is recorded as a controlled result");

        assert_eq!(result.status, "authorization_stale");
        assert_eq!(adapter.dispatches.get(), 1);
    }

    #[test]
    fn station3a_replay_is_idempotent_and_protocol_errors_are_diagnosed() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        let proposal = Fixture::proposal("dispatch");
        let first =
            execute_supervisor_action(&fixture.runtime, proposal.clone(), &adapter).expect("first");
        let replay =
            execute_supervisor_action(&fixture.runtime, proposal, &adapter).expect("replay");
        assert_eq!(first.action_id, replay.action_id);
        assert_eq!(adapter.dispatches.get(), 1);
        let invalid = execute_supervisor_last_message(&fixture.runtime, "not json", &adapter)
            .expect("invalid result");
        assert_eq!(invalid.status, "protocol_invalid");
        let store = load_store(&fixture.path).expect("store");
        assert_eq!(store.actions.len(), 2);
        assert_eq!(store.actions[1].kind, "system_protocol_invalid");
        assert_eq!(store.actions[1].execution_status, "protocol_invalid");
    }

    #[test]
    fn station3a_dispatch_is_single_flight_across_supervisor_runs() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        execute_supervisor_action(&fixture.runtime, Fixture::proposal("dispatch"), &adapter)
            .expect("first supervisor run may dispatch");

        let mut second_runtime = fixture.runtime.clone();
        second_runtime.run_id = "supervisor:station3a:second-run".to_string();
        crate::mcp::supervisor_orchestrator::record_pilot_session_started(
            &mcp_config(&second_runtime),
            &crate::mcp::supervisor_orchestrator::SupervisorPilotSessionLaunch {
                project_root: PROJECT.to_string(),
                workflow_id: WORKFLOW.to_string(),
                authorization_id: AUTH.to_string(),
                model_id: "test".to_string(),
                reasoning_effort: "medium".to_string(),
                workbench_executable_path: "/tmp/codex-governance-workbench-test".to_string(),
                workbench_build_id: "test-build".to_string(),
                supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
                supervisor_contract_sha256: "test-supervisor-contract-hash".to_string(),
                worker_report_contract_sha256: "test-worker-contract-hash".to_string(),
            },
        )
        .expect("record second supervisor run");
        let error =
            execute_supervisor_action(&second_runtime, Fixture::proposal("dispatch"), &adapter)
                .expect_err("same authorization and work item must be single-flight across runs");
        assert!(error.contains("跨主管 run 重复启动 worker"), "{error}");
        assert_eq!(adapter.dispatches.get(), 1);
    }

    #[test]
    fn station3a_top_level_dispatch_fields_are_diagnosed_then_nested_target_dispatches_once() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        let wrong_top_level = format!(
            r#"{{"schema_version":"supervisor_action_proposal.v1","kind":"dispatch_worker","node_id":"{NODE}","work_item_id":"{WORK_ITEM}","reason":"准备完成","expected_result":"worker"}}"#
        );
        let invalid = execute_supervisor_last_message(&fixture.runtime, &wrong_top_level, &adapter)
            .expect("protocol invalid result");
        assert_eq!(invalid.status, "protocol_invalid");
        assert_eq!(adapter.dispatches.get(), 0);
        let store = load_store(&fixture.path).expect("protocol diagnostic store");
        assert_eq!(store.actions.len(), 1);
        assert_eq!(store.actions[0].kind, "system_protocol_invalid");
        assert_eq!(store.actions[0].execution_status, "protocol_invalid");

        let dispatch = serde_json::to_string(&Fixture::proposal("dispatch"))
            .expect("serialize nested target dispatch");
        let first = execute_supervisor_last_message(&fixture.runtime, &dispatch, &adapter)
            .expect("nested target dispatch");
        let replay = execute_supervisor_last_message(&fixture.runtime, &dispatch, &adapter)
            .expect("dispatch replay");
        assert_eq!(first.status, "completed");
        assert_eq!(first.action_id, replay.action_id);
        assert_eq!(adapter.dispatches.get(), 1);

        let inspect = serde_json::to_string(&Fixture::proposal("inspect"))
            .expect("serialize inspect proposal");
        let inspected = execute_supervisor_last_message(&fixture.runtime, &inspect, &adapter)
            .expect("inspect after dispatch");
        assert_eq!(inspected.status, "completed");
        assert_eq!(adapter.dispatches.get(), 1);
    }

    #[test]
    fn station3a_v3_dispatch_invalid_inspect_corrected_inspect_starts_one_worker() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        let dispatch = serde_json::to_string(&Fixture::proposal("dispatch"))
            .expect("serialize dispatch proposal");
        execute_supervisor_last_message(&fixture.runtime, &dispatch, &adapter)
            .expect("dispatch worker");
        assert_eq!(adapter.dispatches.get(), 1);

        let invalid_inspect = r#"{"schema_version":"supervisor_action_proposal.v1","kind":"inspect_worker","target":{"worker_id":"worker-1"},"reason":"读取回程","expected_result":"获得证据"}"#;
        let invalid = execute_supervisor_last_message(&fixture.runtime, invalid_inspect, &adapter)
            .expect("record invalid inspect");
        assert_eq!(invalid.status, "protocol_invalid");
        assert_eq!(adapter.dispatches.get(), 1);

        let inspect = serde_json::to_string(&Fixture::proposal("inspect"))
            .expect("serialize corrected inspect");
        let corrected = execute_supervisor_last_message(&fixture.runtime, &inspect, &adapter)
            .expect("corrected inspect");
        assert_eq!(corrected.status, "completed");
        assert_eq!(adapter.dispatches.get(), 1);
        let store = load_store(&fixture.path).expect("action store");
        assert_eq!(store.actions.len(), 3);
        assert_eq!(store.actions[1].kind, "system_protocol_invalid");
    }

    #[test]
    fn station3a_v3_worker_report_requires_legal_evidence_and_blocks_conservatively() {
        let blocked = worker_report_outcome(
            &json!({
                "worker_id": "worker-1",
                "acceptance_status": "blocked",
                "summary": "缺少用户确认"
            }),
            "worker-1",
        )
        .expect("blocked report is a valid waiting signal");
        assert_eq!(blocked.0, "waiting_user");
        assert!(!blocked.2);

        let invalid = worker_report_outcome(
            &json!({
                "worker_id": "worker-1",
                "acceptance_status": "reported_completed",
                "summary": "已完成",
                "executed_what": "写入文件",
                "changed_what": "proof.txt",
                "evidence_refs": []
            }),
            "worker-1",
        )
        .expect_err("empty evidence must not enter acceptance");
        assert!(invalid.contains("report_invalid"));

        let unverified = worker_report_outcome(
            &json!({
                "worker_id": "worker-1",
                "acceptance_status": "reported_completed",
                "summary": "已完成",
                "executed_what": "写入文件",
                "changed_what": "proof.txt",
                "evidence_refs": [UNVERIFIED_WORKER_MESSAGE_EVIDENCE]
            }),
            "worker-1",
        )
        .expect_err("a generic worker-message placeholder is not legal evidence");
        assert!(unverified.contains("report_invalid"));

        let invalid_result = SupervisorActionResultV1 {
            action_id: None,
            status: classify_adapter_error("report_invalid: bad JSON").to_string(),
            summary: "bad JSON".to_string(),
            worker_id: Some("worker-1".to_string()),
            adapter_id: None,
            evidence_present: false,
        };
        assert_eq!(invalid_result.status, "report_invalid");
        assert!(!invalid_result.should_continue());
        assert_eq!(
            classify_adapter_error("transport_timeout: 初始化超过 120 秒，进程组已回收"),
            "transport_failed"
        );
    }

    // 站 3b attempt-4 回归：reported_completed 的权威结果必须把 evidence 与 findings 的**实际内容**
    // 带给主管（塞进 summary），而不是只回一个 evidence_present 布尔。否则主管看不到引用/行号/逐条判定、
    // 无法抽核，只能 follow_up 撞授权闸、停 waiting_user。
    #[test]
    fn station3b_reported_completed_carries_evidence_and_findings_to_supervisor() {
        let (status, summary, evidence_present, readback) = worker_report_outcome(
            &json!({
                "worker_id": "worker-1",
                "acceptance_status": "reported_completed",
                "summary": "只读盘点完成",
                "executed_what": "对照 README 逐条核验",
                "changed_what": "worker 未列出产出文件",
                "evidence_refs": ["node --check game.js 退出码 0", "逐行回读 README.md 与源码"],
                "findings": [
                    "承诺移动已实现 README.md:11 原文，game.js:119 原文 const left = ...",
                    "P0 game.js:137 未按 delta 缩放，原文 player.x += player.vx;"
                ]
            }),
            "worker-1",
        )
        .expect("reported_completed with evidence+findings is valid");
        assert_eq!(status, "completed");
        assert!(evidence_present);
        assert_eq!(readback.as_deref(), Some("worker-report:worker-1"));
        // 证据内容进 summary（主管可抽核，不只是 present 布尔）。
        assert!(
            summary.contains("node --check game.js 退出码 0"),
            "evidence 内容必须上桥面：{summary}"
        );
        // findings 逐条进 summary。
        assert!(
            summary.contains("P0 game.js:137 未按 delta 缩放"),
            "findings 必须上桥面：{summary}"
        );
        assert!(
            summary.contains("game.js:119"),
            "逐条判定引用必须上桥面：{summary}"
        );
        assert!(
            summary.contains("结论逐条"),
            "findings 段必须带标签：{summary}"
        );
    }

    // 写单（无 findings）：summary 带 evidence、不带「结论逐条」段——findings 空不留空标签。
    #[test]
    fn station3b_write_order_without_findings_omits_findings_segment() {
        let (_status, summary, _present, _readback) = worker_report_outcome(
            &json!({
                "worker_id": "worker-1",
                "acceptance_status": "reported_completed",
                "summary": "创建文件完成",
                "executed_what": "写入 proof.txt",
                "changed_what": "/p/proof.txt",
                "evidence_refs": ["回读字节校验通过"]
            }),
            "worker-1",
        )
        .expect("write-order reported_completed is valid");
        assert!(summary.contains("回读字节校验通过"));
        assert!(
            !summary.contains("结论逐条"),
            "无 findings 不应出现结论段：{summary}"
        );
    }

    #[test]
    fn station3a_dispatch_uses_active_authorization_not_runner_internal_tool_name() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        let result =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("dispatch"), &adapter)
                .expect("dispatch result");
        assert_eq!(result.status, "completed");
        assert_eq!(adapter.dispatches.get(), 1);
    }

    #[test]
    fn station3a_crash_after_revision_advancing_adapter_does_not_replay_worker() {
        let fixture = Fixture::new();
        let adapter = RevisionAdvancingAdapter {
            dispatches: Cell::new(0),
        };
        let proposal = Fixture::proposal("dispatch");
        let guard = guard_action(&fixture.runtime, &proposal).expect("dispatch guard");
        let idempotency_key =
            action_idempotency_key(&fixture.runtime, &proposal).expect("idempotency key");
        let action_id = "supervisor-action:crash-window";
        reserve_action(
            &fixture.runtime,
            &proposal,
            &guard,
            action_id,
            &idempotency_key,
        )
        .expect("reserve action");
        let action = AuthorizedSupervisorAction {
            runtime: fixture.runtime.clone(),
            proposal: proposal.clone(),
            allowed_read_roots: guard.allowed_read_roots.clone(),
            allowed_write_roots: guard.allowed_write_roots.clone(),
            authorization_snapshot_hash: guard.authorization_snapshot_hash.clone(),
            task_package_fingerprint: guard.task_package_fingerprint.clone(),
        };

        // Simulate a crash after the adapter launched the worker, but before complete_action.
        adapter.execute(&action).expect("adapter launch");
        assert_eq!(workflow_revision(&fixture.path).expect("revision"), 2);

        let recovered = execute_supervisor_action(&fixture.runtime, proposal, &adapter)
            .expect("recovery result");
        assert_eq!(recovered.action_id.as_deref(), Some(action_id));
        assert_eq!(recovered.status, "waiting_user");
        assert_eq!(adapter.dispatches.get(), 1);
        let store = load_store(&fixture.path).expect("action store");
        assert_eq!(store.actions.len(), 1);
        assert_eq!(store.actions[0].execution_status, "waiting_user");
        assert!(store.actions[0].summary.contains("停止自动重放"));
    }

    #[test]
    fn m5a_db_primary_projects_supervisor_reservation_and_completion() {
        let _serial = crate::workbench_sqlite_storage_mode::storage_mode_test_lock()
            .lock()
            .expect("storage mode test lock");
        let root = std::env::temp_dir().join(format!(
            "m5a-supervisor-action-{}-{}",
            crate::unix_timestamp_nanos(),
            FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture root");
        let root = fs::canonicalize(&root).expect("canonical fixture root");
        let state_path = root.join("workflow-state").join("workflow-state.v0.json");
        fs::create_dir_all(state_path.parent().expect("state parent")).expect("state parent");
        let timestamp = crate::unix_timestamp_string();
        let bootstrap_event_id = crate::workflow_audit::audit_event_identity(
            "m5a-supervisor-bootstrap",
            &state_path.display().to_string(),
            &timestamp,
        );
        let initial_state =
            crate::initial_workflow_state_json(&timestamp, &bootstrap_event_id, false, &state_path);
        crate::write_validated_workflow_state(&state_path, &initial_state)
            .expect("write valid fixture state");
        let state_path = fs::canonicalize(&state_path).expect("canonical state path");
        let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(&state_path)
            .expect("storage mode config path");
        fs::create_dir_all(config_path.parent().expect("runtime artifacts parent"))
            .expect("runtime artifacts parent");
        let runtime_artifacts = fs::canonicalize(config_path.parent().expect("runtime artifacts"))
            .expect("canonical runtime artifacts");
        let config = crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
            workflow_state_path: state_path.clone(),
            confirmed_workflow_state_path: state_path.clone(),
            db_path: runtime_artifacts.join("workbench.sqlite"),
            confirmed_db_path: runtime_artifacts.join("workbench.sqlite"),
            denied_path_markers: vec![],
        };
        fs::write(
            &config_path,
            serde_json::to_vec(&json!({
                "schema_version": crate::workbench_sqlite_storage_mode::STORAGE_MODE_SCHEMA_VERSION,
                "mode": "db_primary_json_projection",
                "workflow_state_path": config.workflow_state_path,
                "confirmed_workflow_state_path": config.confirmed_workflow_state_path,
                "db_path": config.db_path,
                "confirmed_db_path": config.confirmed_db_path,
            }))
            .expect("serialize DB primary config"),
        )
        .expect("write DB primary config");

        let repository =
            crate::workbench_sqlite_repository::WorkbenchSqliteRepository::open_confirmed(
                &crate::workbench_sqlite_repository::ConfirmedWorkbenchSqliteRepositoryConfig {
                    db_path: config.db_path.clone(),
                    confirmed_db_path: config.confirmed_db_path.clone(),
                    denied_path_markers: vec![],
                },
            )
            .expect("initialize confirmed fixture DB");
        let bootstrap_audit = initial_state["audit_events"][0].clone();
        repository
            .append_audit(
                &crate::workbench_sqlite_repository::RepositoryAuditEntry {
                    event_id: bootstrap_event_id,
                    target_kind: "workflow_state".to_string(),
                    target_id: bootstrap_audit["target_ref"]
                        .as_str()
                        .expect("bootstrap target ref")
                        .to_string(),
                    payload: bootstrap_audit,
                },
                None,
            )
            .expect("seed bootstrap audit projection");
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
        crate::workbench_sqlite_storage_mode::initialize_for_startup(&state_path)
            .expect("DB primary startup reconciliation");

        let runtime = SupervisorActionRuntime {
            run_id: "supervisor:m5a:test".to_string(),
            project_root: root.join("fixture-project").display().to_string(),
            workflow_id: "workflow:m5a".to_string(),
            authorization_id: "authorization:m5a".to_string(),
            workflow_state_path: state_path.clone(),
            quota_limits: SupervisorQuotaLimits {
                max_active_workers: 1,
                max_follow_ups_per_worker: 0,
                max_runtime_minutes: 1,
            },
            started_at_ms: crate::unix_timestamp_ms(),
        };
        let proposal = parse_supervisor_action_proposal(
            r#"{"schema_version":"supervisor_action_proposal.v1","kind":"dispatch_worker","target":{"node_id":"node:m5a","work_item_id":"work-item:m5a"},"reason":"fixture reserve","expected_result":"fixture worker"}"#,
        )
        .expect("dispatch proposal");
        let guard = ActionGuard {
            workflow_revision: 0,
            authorization_snapshot_hash: "m5a-auth-snapshot".to_string(),
            task_package_fingerprint: "m5a-task-package".to_string(),
            allowed_read_roots: vec![],
            allowed_write_roots: vec![],
        };
        reserve_action(
            &runtime,
            &proposal,
            &guard,
            "supervisor-action:m5a",
            "idempotency:m5a",
        )
        .expect("DB-primary supervisor reservation");
        let completed = complete_action(
            &runtime,
            "supervisor-action:m5a",
            1,
            &SupervisorActionAdapterResult {
                status: "completed".to_string(),
                summary: "fixture completed".to_string(),
                worker_id: Some("worker:m5a".to_string()),
                adapter_id: "m5a-fixture-adapter".to_string(),
                evidence_present: true,
                dispatch_ref: Some("dispatch:m5a".to_string()),
                readback_ref: Some("readback:m5a".to_string()),
                audit_ref: Some("audit:m5a".to_string()),
            },
        )
        .expect("DB-primary supervisor completion");
        assert_eq!(completed.status, "completed");
        let store = load_store(&state_path).expect("supervisor action projection");
        assert_eq!(store.actions.len(), 1);
        assert_eq!(store.actions[0].execution_status, "completed");
        let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(&config)
            .expect("reconcile supervisor projection");
        assert!(
            report.is_green(),
            "reconciliation must be green: {report:?}"
        );
        let supervisor_actions = report
            .tables
            .iter()
            .find(|table| table.table_name == "supervisor_actions")
            .expect("supervisor actions table");
        assert_eq!(supervisor_actions.db_count, 1);
        assert_eq!(supervisor_actions.matched_count, 1);
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_tests();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn station3a_revision_drift_is_recorded_before_adapter_execution() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        execute_supervisor_action(&fixture.runtime, Fixture::proposal("dispatch"), &adapter)
            .expect("first action");
        let mut workflow: Value =
            serde_json::from_slice(&fs::read(&fixture.path).expect("state")).expect("workflow");
        workflow["revision"] = json!(2);
        fs::write(
            &fixture.path,
            serde_json::to_vec(&workflow).expect("workflow json"),
        )
        .expect("write drifted workflow");
        let result =
            execute_supervisor_action(&fixture.runtime, Fixture::proposal("inspect"), &adapter)
                .expect("rejection result");
        assert_eq!(result.status, "authorization_stale");
        assert_eq!(adapter.dispatches.get(), 1);
        assert_eq!(
            load_store(&fixture.path)
                .expect("action store")
                .actions
                .last()
                .expect("guard rejection")
                .execution_status,
            "authorization_stale"
        );
    }

    #[test]
    fn station3a_report_cannot_impersonate_user_cancellation() {
        let fixture = Fixture::new();
        let adapter = FakeAdapter {
            dispatches: Cell::new(0),
        };
        let proposal = parse_supervisor_action_proposal(
            r#"{"schema_version":"supervisor_action_proposal.v1","kind":"report_user","message":"用户已取消本单","reason":"x","expected_result":"y"}"#,
        )
        .expect("proposal");
        let result = execute_supervisor_action(&fixture.runtime, proposal, &adapter)
            .expect("protocol rejection result");
        assert_eq!(result.status, "protocol_invalid");
        assert!(result.summary.contains("不得冒充"));
        let store = load_store(&fixture.path).expect("store");
        assert_eq!(store.actions.len(), 1);
        assert_eq!(store.actions[0].execution_status, "protocol_invalid");
    }

    #[test]
    fn station3a_request_user_decision_is_waiting_without_user_cancel_attribution() {
        let fixture = Fixture::new();
        let proposal = parse_supervisor_action_proposal(
            r#"{"schema_version":"supervisor_action_proposal.v1","kind":"request_user_decision","question":"是否扩大范围？","reason":"当前任务包不覆盖该写入","expected_result":"等待用户决定"}"#,
        )
        .expect("proposal");
        let result = execute_supervisor_action(
            &fixture.runtime,
            proposal,
            &WorkbenchSupervisorActionAdapter,
        )
        .expect("waiting result");
        assert_eq!(result.status, "waiting_user");
        assert!(!result.summary.contains("user_cancelled"));
        assert_eq!(
            load_store(&fixture.path)
                .expect("action store")
                .actions
                .last()
                .expect("waiting record")
                .execution_status,
            "waiting_user"
        );
    }
}
