use crate::utils::hash::{sha256_hex, short_hash};
use crate::utils::normalization::normalize_slash_lowercase as normalize;
use crate::{
    CodexLocalActiveAttempt, CodexLocalAuditRef, CodexLocalCommandPlan, CodexLocalExecutionAttempt,
    CodexLocalExecutionGuard, CodexLocalExecutionRequest, CodexLocalFailureReason,
    CodexLocalReadbackResult, CodexLocalRuntimeLogRef,
};
use std::fs;
use std::io::Write;
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

pub(crate) trait CodexLocalRunner {
    fn run_dry(
        &self,
        request: CodexLocalExecutionRequest,
        timestamp: &str,
    ) -> CodexLocalExecutionAttempt;
}

pub(crate) trait CodexLocalPhaseAProcessRunner {
    fn run_phase_a(
        &self,
        request: &CodexLocalExecutionRequest,
        command_plan: &CodexLocalCommandPlan,
    ) -> CodexLocalPhaseAProcessResult;
}

pub(crate) trait CodexLocalPhaseBProcessRunner {
    fn run_phase_b(
        &self,
        request: &CodexLocalExecutionRequest,
        command_plan: &CodexLocalCommandPlan,
        prompt_body: &str,
        last_message_path: &Path,
        timeout_ms: Option<i64>,
    ) -> CodexLocalPhaseBProcessResult;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CodexLocalPhaseAProcessResult {
    pub(crate) runner_kind: String,
    pub(crate) status: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) readback_status: String,
    pub(crate) readback_attempted: bool,
    pub(crate) readback_result_count: Option<i64>,
    pub(crate) failure_code: Option<String>,
    pub(crate) failure_message: Option<String>,
    pub(crate) retryable: bool,
    pub(crate) user_action_required: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CodexLocalPhaseBProcessResult {
    pub(crate) runner_kind: String,
    pub(crate) status: String,
    pub(crate) exit_code: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) prompt_sent: bool,
    pub(crate) real_codex_executed: bool,
    pub(crate) writes_codex_home: bool,
    pub(crate) writes_project_files: bool,
    pub(crate) readback_status: String,
    pub(crate) readback_attempted: bool,
    pub(crate) readback_result_count: Option<i64>,
    pub(crate) last_message_path: Option<String>,
    pub(crate) failure_code: Option<String>,
    pub(crate) failure_message: Option<String>,
    pub(crate) retryable: bool,
    pub(crate) user_action_required: bool,
    pub(crate) warnings: Vec<String>,
}

#[derive(Default)]
pub(crate) struct FakeCodexLocalRunner;

#[derive(Default)]
pub(crate) struct NoopCodexLocalPhaseAProcessRunner;

#[derive(Default)]
pub(crate) struct RealCodexLocalPhaseBProcessRunner;

impl CodexLocalRunner for FakeCodexLocalRunner {
    fn run_dry(
        &self,
        request: CodexLocalExecutionRequest,
        timestamp: &str,
    ) -> CodexLocalExecutionAttempt {
        let guard = inspect_codex_local_execution_guard(&request);
        dry_run_attempt(request, guard, timestamp)
    }
}

impl CodexLocalPhaseAProcessRunner for NoopCodexLocalPhaseAProcessRunner {
    fn run_phase_a(
        &self,
        _request: &CodexLocalExecutionRequest,
        command_plan: &CodexLocalCommandPlan,
    ) -> CodexLocalPhaseAProcessResult {
        CodexLocalPhaseAProcessResult {
            runner_kind: "codex_local_phase_a_noop_process_runner".to_string(),
            status: "readback_unavailable".to_string(),
            exit_code: None,
            timed_out: false,
            readback_status: "readback_unavailable".to_string(),
            readback_attempted: false,
            readback_result_count: None,
            failure_code: Some("phase_a_no_real_process".to_string()),
            failure_message: Some(format!(
                "H2.5 Phase A built structured command plan for {}, but did not spawn Codex.",
                command_plan.program
            )),
            retryable: false,
            user_action_required: true,
            warnings: vec![
                "phase_a_no_real_process_runner".to_string(),
                "prompt_not_sent".to_string(),
                "real_codex_executed_false".to_string(),
                "codex_home_not_touched".to_string(),
                "readback_unavailable_is_not_zero_results".to_string(),
            ],
        }
    }
}

impl CodexLocalPhaseBProcessRunner for RealCodexLocalPhaseBProcessRunner {
    fn run_phase_b(
        &self,
        request: &CodexLocalExecutionRequest,
        command_plan: &CodexLocalCommandPlan,
        prompt_body: &str,
        last_message_path: &Path,
        timeout_ms: Option<i64>,
    ) -> CodexLocalPhaseBProcessResult {
        run_real_codex_process(
            request,
            command_plan,
            prompt_body,
            last_message_path,
            timeout_ms,
        )
    }
}

/// 工作流单节点·真实 codex 执行适配器（中间版本闭环 worker 节点 / 收敛第一刀）。
///
/// 作用：把工作流侧的 `CodexResumeRunner` 接口，翻成 codex_local 侧请求，
/// **复用已验证的沙箱化 `command_plan_for` + `run_real_codex_process`**——
/// 不另造 spawn、不另拼沙箱参数、不注入任何审批绕过标。
///
/// 仅由 `execute_workflow_node_dispatch` 在「固定测试项目 + env 钥匙」双闸通过后构造；
/// 真实项目 / 没钥匙 时根本到不了这里（命令层直接 blocked）。
pub(crate) struct RealWorkflowNodeCodexRunner;

impl crate::CodexResumeRunner for RealWorkflowNodeCodexRunner {
    fn resume_with_options(
        &self,
        thread_id: &str,
        prompt: &str,
        last_message_path: &Path,
        options: &crate::CodexResumeRequestOptions,
    ) -> Result<(crate::CodexResumeRunResult, crate::WorkflowNodeDispatchExecutionOptions), String>
    {
        // safe_probe 是只读探针，不真跑 codex。
        if options.prompt_kind == "safe_probe" {
            return Err("safe_probe 不走真实 codex 执行".to_string());
        }
        // 沙箱与执行目录必须由上游指令给全，缺一律拒——不裸跑。
        let sandbox = options
            .sandbox_mode
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "缺少 sandbox_mode，已拒绝真实 codex 执行".to_string())?;
        let target_cwd = options
            .execution_cwd
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "缺少 execution_cwd，已拒绝真实 codex 执行".to_string())?;
        let allowed_write_roots: Vec<String> = options
            .allowed_write_roots
            .iter()
            .map(|root| root.to_string_lossy().to_string())
            .collect();
        // 节点已绑 codex 会话则 resume，否则新建会话（首跑场景）。
        let trimmed_thread = thread_id.trim();
        let (operation_id, session_id) = if trimmed_thread.is_empty() {
            ("new_session".to_string(), None)
        } else {
            ("resume".to_string(), Some(trimmed_thread.to_string()))
        };
        let request = CodexLocalExecutionRequest {
            request_version: 1,
            adapter_id: "codex-local".to_string(),
            operation_id,
            project_id: String::new(),
            project_root: target_cwd.clone(),
            workflow_id: String::new(),
            node_id: String::new(),
            session_id,
            work_item_id: None,
            continuation_id: None,
            target_cwd,
            allowed_write_roots,
            sandbox,
            prompt_source_kind: "workflow_node_prompt".to_string(),
            prompt_summary: String::new(),
            prompt_sha256: String::new(),
            prompt_ref: String::new(),
            readback_plan: crate::CodexLocalReadbackPlan {
                strategy: "required".to_string(),
                required: true,
                expected_sources: vec!["worker_report_candidate".to_string()],
                unavailable_behavior:
                    "readback_unavailable_or_failed_keeps_result_count_null".to_string(),
                trust_policy: "workbench_managed_refs_only_no_full_transcript_by_default"
                    .to_string(),
                warnings: vec![],
            },
            requested_by: "workflow_node_test_project_runner".to_string(),
            user_confirmation_state: "test_project_env_gated".to_string(),
            authorization_scope_id: None,
            runtime_log_refs: vec![],
            audit_refs: vec![],
            active_attempts: vec![],
            warnings: vec!["workflow_node_real_codex_test_project_only".to_string()],
        };
        let command_plan = command_plan_for(&request);
        let timeout_ms = options.timeout_seconds.map(|seconds| seconds * 1000);
        let result = run_real_codex_process(
            &request,
            &command_plan,
            prompt,
            last_message_path,
            timeout_ms,
        );
        if !result.real_codex_executed {
            return Err(result
                .failure_message
                .unwrap_or_else(|| "真实 codex 未执行".to_string()));
        }
        Ok((
            crate::CodexResumeRunResult {
                exit_code: result.exit_code.unwrap_or(-1),
                timed_out: result.timed_out,
                stderr_summary: result.failure_message,
            },
            // readback_stats 交回 None，由 execute_workflow_node_dispatch_at 走真实
            // dispatch_readback_stats 计算，不在这里伪造结果数。
            crate::WorkflowNodeDispatchExecutionOptions {
                readback_stats: None,
            },
        ))
    }
}

// ===== S3 咨询第一刀·只读 confinement（结构性硬钉只读）=====
// 复用现成 command_plan_for / 沙箱（**字节不改**）+ RealCodexLocalPhaseBProcessRunner；只在这里「喂一个
// 只能只读的请求」。死线：sandbox="read-only" + allowed_write_roots=[] **写死在构造里、不收权限参数**，
// 调用方永远拿不到改成可写/可执行的机会 → 咨询 codex 结构性只读、不走 worker 执行闸也写不了/跑不了命令。

/// 构造一个**只读** codex 请求（read-only 沙箱·写盘根为空·cwd=被咨询项目根）。纯函数、可单测断言只读。
pub(crate) fn build_readonly_consult_request(
    project_root: &str,
    prompt: &str,
) -> CodexLocalExecutionRequest {
    CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: "codex-local".to_string(),
        operation_id: "new_session".to_string(),
        project_id: format!("consult:{}", crate::utils::hash::short_hash(project_root)),
        project_root: project_root.to_string(),
        workflow_id: String::new(),
        node_id: String::new(),
        session_id: None,
        work_item_id: None,
        continuation_id: None,
        target_cwd: project_root.to_string(),
        allowed_write_roots: vec![], // 写死空：read-only → 无 --add-dir、不能写
        sandbox: "read-only".to_string(), // 写死只读
        prompt_source_kind: "consultant_readonly".to_string(),
        prompt_summary: prompt.chars().take(160).collect(),
        prompt_sha256: crate::utils::hash::sha256_hex(prompt),
        prompt_ref: format!(
            "consult-readonly:{}",
            crate::utils::hash::short_hash(prompt)
        ),
        readback_plan: crate::CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec!["consultation_proposal".to_string()],
            unavailable_behavior: "readback_unavailable_or_failed_keeps_result_count_null"
                .to_string(),
            trust_policy: "workbench_managed_refs_only_no_full_transcript_by_default".to_string(),
            warnings: vec![],
        },
        requested_by: "consultant_agent_readonly".to_string(),
        user_confirmation_state: "none_required_readonly".to_string(),
        authorization_scope_id: None,
        runtime_log_refs: vec![],
        audit_refs: vec![],
        active_attempts: vec![],
        warnings: vec!["consultant_readonly_enforced_in_constructor".to_string()],
    }
}

/// 一次性只读起 codex 读项目、抠出最后消息文本。只读 confinement 见 build_readonly_consult_request。
pub(crate) fn readonly_codex_consult(
    project_root: &str,
    prompt: &str,
    timeout_ms: Option<i64>,
) -> Result<String, String> {
    let request = build_readonly_consult_request(project_root, prompt);
    // 只读咨询**不是 workflow 节点执行**：guard 的「执行身份」(work_item/node/workflow IDs) 与「执行授权」
    // (用户确认/授权范围/审计 ref) 这 6 道不适用——只读已由 sandbox=read-only + 写盘根空**结构性**保证，
    // 不写不跑命令、无执行可授权。故只对**读相关安全** reason 拦（adapter/路径越界/密钥 deny/prompt 边界/
    // command_plan 仍照拦）；执行身份/授权 reason 豁免。同 S1 canvas_node 排除 3 道授权 reason 的思路、不碰 guard 本体。
    const CONSULT_READONLY_EXEMPT_GUARD_REASONS: [&str; 6] = [
        "user_confirmation_required",
        "authorization_scope_missing",
        "audit_ref_missing",
        "new_session_requires_work_item_id",
        "node_id_missing",
        "workflow_id_missing",
    ];
    let guard = inspect_codex_local_execution_guard(&request);
    let blocking: Vec<String> = guard
        .reasons
        .iter()
        .filter(|reason| !CONSULT_READONLY_EXEMPT_GUARD_REASONS.contains(&reason.as_str()))
        .cloned()
        .collect();
    if !blocking.is_empty() {
        return Err(format!("consultant_readonly_blocked:{}", blocking.join(",")));
    }
    let command_plan = command_plan_for(&request);
    let last_message_path = std::env::temp_dir().join(format!(
        "consult-last-{}.txt",
        crate::utils::hash::short_hash(prompt)
    ));
    let runner = RealCodexLocalPhaseBProcessRunner;
    let result = runner.run_phase_b(
        &request,
        &command_plan,
        prompt,
        &last_message_path,
        timeout_ms,
    );
    if !result.real_codex_executed {
        return Err(result
            .failure_message
            .unwrap_or_else(|| "真实 codex 未执行".to_string()));
    }
    std::fs::read_to_string(&last_message_path)
        .map_err(|error| format!("consult_last_message_read_failed:{error}"))
}

pub(crate) fn inspect_codex_local_execution_guard(
    request: &CodexLocalExecutionRequest,
) -> CodexLocalExecutionGuard {
    let mut reasons = Vec::new();
    let mut required_fixes = Vec::new();
    let mut warnings = h1_warnings();

    if request.adapter_id != "codex-local" {
        reasons.push("adapter_not_codex_local".to_string());
        required_fixes.push("H1 只允许 adapter_id=codex-local。".to_string());
    }
    if !matches!(
        request.operation_id.as_str(),
        "new_session" | "send_message" | "resume"
    ) {
        reasons.push("operation_not_contractual".to_string());
        required_fixes
            .push("operation_id 只能是 new_session、send_message 或 resume。".to_string());
    }
    if request.operation_id == "resume" && blank_opt(request.session_id.as_deref()) {
        reasons.push("resume_requires_session_id".to_string());
        required_fixes.push("resume 必须绑定 session_id。".to_string());
    }
    if request.operation_id == "new_session" && blank_opt(request.work_item_id.as_deref()) {
        reasons.push("new_session_requires_work_item_id".to_string());
        required_fixes.push("new_session 必须绑定 work_item_id，不能创建自由会话。".to_string());
    }
    check_required_binding(request, &mut reasons, &mut required_fixes);
    check_paths(request, &mut reasons, &mut required_fixes);
    check_secret_deny_list(request, &mut reasons, &mut required_fixes);
    check_prompt_boundary(request, &mut reasons, &mut required_fixes);
    check_readback_plan(request, &mut reasons, &mut required_fixes);

    let duplicate_running_attempt = has_duplicate_running_attempt(&request.active_attempts);
    if duplicate_running_attempt {
        reasons.push("duplicate_running_attempt".to_string());
        required_fixes.push("已有 running/queued attempt 时不得创建新的 H1 attempt。".to_string());
    }
    let requires_user_confirmation = request.user_confirmation_state != "confirmed";
    if requires_user_confirmation {
        reasons.push("user_confirmation_required".to_string());
        required_fixes.push("H1 dry-run 也必须引用已确认的工作台授权/确认记录。".to_string());
    }
    if request
        .authorization_scope_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        reasons.push("authorization_scope_missing".to_string());
        required_fixes.push("必须绑定 authorization_scope_id，避免越过授权范围。".to_string());
    }
    if request.audit_refs.is_empty() {
        reasons.push("audit_ref_missing".to_string());
        required_fixes.push("必须引用确认/权限 audit ref。".to_string());
    }
    if request.runtime_log_refs.is_empty() {
        warnings.push("runtime_log_ref_missing_for_initial_h1_attempt".to_string());
    }

    let command_plan = if reasons
        .iter()
        .any(|reason| reason == "operation_not_contractual")
    {
        None
    } else {
        Some(command_plan_for(request))
    };

    let command_safe = command_plan
        .as_ref()
        .map(|plan| {
            !plan.shell_invocation
                && !plan.prompt_in_command
                && plan.program == "codex"
                && !plan
                    .argv
                    .iter()
                    .any(|arg| arg.contains(&request.prompt_sha256))
        })
        .unwrap_or(false);
    if !command_safe {
        reasons.push("command_plan_not_structured_safe".to_string());
        required_fixes.push(
            "CLI 计划必须是 program+argv+stdin prompt ref，禁止 shell 字符串拼接。".to_string(),
        );
    }

    reasons.sort();
    reasons.dedup();
    required_fixes.sort();
    required_fixes.dedup();
    warnings.sort();
    warnings.dedup();

    let blocks_execution = !reasons.is_empty();
    CodexLocalExecutionGuard {
        guard_version: 1,
        status: if blocks_execution {
            "blocked".to_string()
        } else {
            "dry_run_allowed".to_string()
        },
        severity: if blocks_execution {
            "blocking".to_string()
        } else {
            "info".to_string()
        },
        blocks_execution,
        allows_dry_run: !blocks_execution,
        requires_user_confirmation,
        duplicate_running_attempt,
        command_plan,
        reasons,
        required_fixes,
        warnings,
    }
}

pub(crate) fn run_h2_phase_a_with_runner<R: CodexLocalPhaseAProcessRunner>(
    request: CodexLocalExecutionRequest,
    timestamp: &str,
    runner: &R,
) -> CodexLocalExecutionAttempt {
    let guard = inspect_codex_local_execution_guard(&request);
    if guard.blocks_execution {
        return h2_phase_a_attempt(
            request,
            guard,
            timestamp,
            "blocked_by_guard",
            "codex_local_phase_a_guard",
            "readback_unavailable",
            false,
            None,
            Some(CodexLocalFailureReason {
                code: "guard_blocked".to_string(),
                message: "CodexLocal guard blocked H2.5 Phase A runner path.".to_string(),
                retryable: true,
                user_action_required: true,
            }),
            vec!["phase_a_runner_not_called_guard_blocked".to_string()],
        );
    }

    let command_plan = guard.command_plan.clone();
    let Some(plan) = command_plan.as_ref() else {
        return h2_phase_a_attempt(
            request,
            guard,
            timestamp,
            "blocked_by_guard",
            "codex_local_phase_a_guard",
            "readback_unavailable",
            false,
            None,
            Some(CodexLocalFailureReason {
                code: "command_plan_missing".to_string(),
                message: "Structured command plan missing; runner path refused to proceed."
                    .to_string(),
                retryable: true,
                user_action_required: true,
            }),
            vec!["phase_a_runner_not_called_command_plan_missing".to_string()],
        );
    };

    let process_result = runner.run_phase_a(&request, plan);
    let status = classify_phase_a_status(&process_result);
    let failure_reason = failure_reason_for_phase_a_status(&status, &process_result);
    let mut warnings = process_result.warnings.clone();
    warnings.push(format!(
        "phase_a_exit_code_redacted:{:?}",
        process_result.exit_code
    ));
    if process_result.timed_out {
        warnings.push("phase_a_timeout_classified_without_real_codex".to_string());
    }
    h2_phase_a_attempt(
        request,
        guard,
        timestamp,
        &status,
        &process_result.runner_kind,
        &process_result.readback_status,
        process_result.readback_attempted,
        process_result.readback_result_count,
        failure_reason,
        warnings,
    )
}

pub(crate) fn run_h2_phase_b_with_runner<R: CodexLocalPhaseBProcessRunner>(
    request: CodexLocalExecutionRequest,
    timestamp: &str,
    prompt_body: &str,
    last_message_path: &Path,
    runner: &R,
) -> CodexLocalExecutionAttempt {
    let guard = inspect_codex_local_execution_guard(&request);
    if guard.blocks_execution {
        return h2_phase_b_attempt(
            request,
            guard,
            timestamp,
            "blocked_by_guard",
            "codex_local_phase_b_guard",
            false,
            false,
            false,
            false,
            "readback_unavailable",
            false,
            None,
            Some(CodexLocalFailureReason {
                code: "guard_blocked".to_string(),
                message: "CodexLocal guard blocked H2 Phase B real runner path.".to_string(),
                retryable: true,
                user_action_required: true,
            }),
            vec!["phase_b_runner_not_called_guard_blocked".to_string()],
        );
    }

    let command_plan = guard.command_plan.clone();
    let Some(plan) = command_plan.as_ref() else {
        return h2_phase_b_attempt(
            request,
            guard,
            timestamp,
            "blocked_by_guard",
            "codex_local_phase_b_guard",
            false,
            false,
            false,
            false,
            "readback_unavailable",
            false,
            None,
            Some(CodexLocalFailureReason {
                code: "command_plan_missing".to_string(),
                message: "Structured command plan missing; Phase B runner refused to proceed."
                    .to_string(),
                retryable: true,
                user_action_required: true,
            }),
            vec!["phase_b_runner_not_called_command_plan_missing".to_string()],
        );
    };

    let process_result = runner.run_phase_b(
        &request,
        plan,
        prompt_body,
        last_message_path,
        request_timeout_ms(&request),
    );
    let status = classify_phase_b_status(&process_result);
    let failure_reason =
        failure_reason_for_phase_b_status(&status, &process_result, &request.operation_id);
    let mut warnings = process_result.warnings.clone();
    warnings.push(format!(
        "phase_b_exit_code_redacted:{:?}",
        process_result.exit_code
    ));
    if process_result.timed_out {
        warnings.push("phase_b_timeout_classified".to_string());
    }
    if let Some(path) = &process_result.last_message_path {
        warnings.push(format!("workbench_managed_last_message_ref:{path}"));
    }
    h2_phase_b_attempt(
        request,
        guard,
        timestamp,
        &status,
        &process_result.runner_kind,
        process_result.prompt_sent,
        process_result.real_codex_executed,
        process_result.writes_codex_home,
        process_result.writes_project_files,
        &process_result.readback_status,
        process_result.readback_attempted,
        process_result.readback_result_count,
        failure_reason,
        warnings,
    )
}

fn dry_run_attempt(
    request: CodexLocalExecutionRequest,
    guard: CodexLocalExecutionGuard,
    timestamp: &str,
) -> CodexLocalExecutionAttempt {
    let request_id = stable_request_id(&request);
    let attempt_id = format!(
        "codex-local-attempt:h1:{}:{}",
        timestamp,
        short_hash(&request_id)
    );
    let status = if guard.allows_dry_run {
        "dry_run_succeeded"
    } else {
        "dry_run_blocked"
    };
    let failure_reason = if guard.allows_dry_run {
        None
    } else {
        Some(CodexLocalFailureReason {
            code: "guard_blocked".to_string(),
            message: guard.reasons.join(","),
            retryable: true,
            user_action_required: true,
        })
    };
    let runtime_log_ref = Some(CodexLocalRuntimeLogRef {
        ref_id: format!("runtime-log:codex-local:h1:{attempt_id}"),
        category: "dispatch_attempt".to_string(),
        status: status.to_string(),
        redaction_status: "redacted_safe_summary".to_string(),
    });
    let audit_ref = request.audit_refs.first().cloned();
    let mut warnings = h1_warnings();
    warnings.extend(guard.warnings.clone());
    warnings.push("fake_runner_only".to_string());
    warnings.push("stdin_prompt_ref_only".to_string());
    warnings.push("readback_unavailable_is_not_zero_results".to_string());
    warnings.sort();
    warnings.dedup();

    CodexLocalExecutionAttempt {
        attempt_version: 1,
        attempt_id,
        request_id,
        runner_kind: "fake_dry_run".to_string(),
        execution_level: "h1_contract_only_no_real_execution".to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        command_plan: guard.command_plan.clone(),
        request,
        guard,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
        runtime_log_ref,
        audit_ref,
        readback_result: CodexLocalReadbackResult {
            status: "readback_unavailable".to_string(),
            attempted: false,
            real_readback_performed: false,
            result_count: None,
            confidence: "none_h1_dry_run".to_string(),
            unavailable_reason: Some("h1_dry_run_does_not_read_transcript".to_string()),
            source_refs: vec![],
            warnings: vec![
                "readback_unavailable_is_not_zero_results".to_string(),
                "no_real_transcript_read_in_h1".to_string(),
            ],
        },
        failure_reason,
        warnings,
    }
}

fn h2_phase_a_attempt(
    request: CodexLocalExecutionRequest,
    guard: CodexLocalExecutionGuard,
    timestamp: &str,
    status: &str,
    runner_kind: &str,
    readback_status: &str,
    readback_attempted: bool,
    readback_result_count: Option<i64>,
    failure_reason: Option<CodexLocalFailureReason>,
    mut warnings: Vec<String>,
) -> CodexLocalExecutionAttempt {
    let request_id = stable_request_id(&request);
    let attempt_id = format!(
        "codex-local-attempt:h2-phase-a:{}:{}",
        timestamp,
        short_hash(&request_id)
    );
    let runtime_log_ref = Some(CodexLocalRuntimeLogRef {
        ref_id: format!("runtime-log:codex-local:h2-phase-a:{attempt_id}"),
        category: "dispatch_attempt".to_string(),
        status: status.to_string(),
        redaction_status: "redacted_safe_summary".to_string(),
    });
    let audit_ref = request.audit_refs.first().cloned();
    warnings.extend([
        "h2_phase_a_runner_path_no_real_codex".to_string(),
        "argv_only_no_shell".to_string(),
        "prompt_via_stdin_ref_only".to_string(),
        "prompt_sent_false".to_string(),
        "real_codex_executed_false".to_string(),
        "writes_codex_home_false".to_string(),
        "readback_unavailable_or_failed_is_not_zero_results".to_string(),
    ]);
    warnings.extend(guard.warnings.clone());
    warnings.sort();
    warnings.dedup();
    let result_count = crate::h4_execution_boundary::h4_result_count(
        status,
        readback_status,
        readback_result_count,
    );

    CodexLocalExecutionAttempt {
        attempt_version: 1,
        attempt_id,
        request_id,
        runner_kind: runner_kind.to_string(),
        execution_level: "h2_phase_a_runner_path_no_real_codex".to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        command_plan: guard.command_plan.clone(),
        request,
        guard,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
        runtime_log_ref,
        audit_ref,
        readback_result: CodexLocalReadbackResult {
            status: readback_status.to_string(),
            attempted: readback_attempted,
            real_readback_performed: false,
            result_count,
            confidence: "phase_a_fake_or_unavailable".to_string(),
            unavailable_reason: if result_count.is_none() {
                Some(
                    "H2.5 Phase A does not read raw transcript; unavailable/failed/timed_out is not zero results."
                        .to_string(),
                )
            } else {
                None
            },
            source_refs: vec![],
            warnings: vec![
                "no_raw_transcript_read_in_phase_a".to_string(),
                "readback_unavailable_or_failed_is_not_zero_results".to_string(),
            ],
        },
        failure_reason,
        warnings,
    }
}

fn h2_phase_b_attempt(
    request: CodexLocalExecutionRequest,
    guard: CodexLocalExecutionGuard,
    timestamp: &str,
    status: &str,
    runner_kind: &str,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    readback_status: &str,
    readback_attempted: bool,
    readback_result_count: Option<i64>,
    failure_reason: Option<CodexLocalFailureReason>,
    mut warnings: Vec<String>,
) -> CodexLocalExecutionAttempt {
    let request_id = stable_request_id(&request);
    let is_new_session = request.operation_id == "new_session";
    let phase_label = if is_new_session { "h3-b" } else { "h2-phase-b" };
    let attempt_id = format!(
        "codex-local-attempt:{phase_label}:{}:{}",
        timestamp,
        short_hash(&request_id)
    );
    let runtime_log_ref = Some(CodexLocalRuntimeLogRef {
        ref_id: format!("runtime-log:codex-local:{phase_label}:{attempt_id}"),
        category: "dispatch_attempt".to_string(),
        status: status.to_string(),
        redaction_status: "redacted_safe_summary".to_string(),
    });
    let audit_ref = request.audit_refs.first().cloned();
    warnings.extend([
        if is_new_session {
            "h3_b_real_new_session_runner_path".to_string()
        } else {
            "h2_phase_b_real_runner_path".to_string()
        },
        "argv_only_no_shell".to_string(),
        "prompt_via_stdin_only".to_string(),
        "prompt_body_not_persisted_by_runner".to_string(),
        "readback_unavailable_or_failed_is_not_zero_results".to_string(),
        "project_file_diff_must_be_verified_by_hash".to_string(),
    ]);
    warnings.extend(phase_b_safe_guard_warnings(&guard.warnings));
    warnings.sort();
    warnings.dedup();
    let result_count = crate::h4_execution_boundary::h4_result_count(
        status,
        readback_status,
        readback_result_count,
    );

    CodexLocalExecutionAttempt {
        attempt_version: 1,
        attempt_id,
        request_id,
        runner_kind: runner_kind.to_string(),
        execution_level: if is_new_session {
            "h3_b_real_codex_new_session".to_string()
        } else {
            "h2_phase_b_real_codex_resume".to_string()
        },
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        command_plan: guard.command_plan.clone(),
        request,
        guard,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files,
        writes_workbench_state: true,
        runtime_log_ref,
        audit_ref,
        readback_result: CodexLocalReadbackResult {
            status: readback_status.to_string(),
            attempted: readback_attempted,
            real_readback_performed: readback_attempted && result_count.is_some(),
            result_count,
            confidence: if result_count.is_some() {
                "workbench_managed_last_message".to_string()
            } else if is_new_session {
                "none_or_failed_h3_b".to_string()
            } else {
                "none_or_failed_h2_phase_b".to_string()
            },
            unavailable_reason: if result_count.is_none() {
                Some(if is_new_session {
                    "H3-B readback unavailable/failed/timed_out is not zero results.".to_string()
                } else {
                    "H2 Phase B readback unavailable/failed/timed_out is not zero results."
                        .to_string()
                })
            } else {
                None
            },
            source_refs: vec!["workbench_managed_last_message".to_string()],
            warnings: vec![
                "readback_unavailable_or_failed_is_not_zero_results".to_string(),
                if is_new_session {
                    "raw_transcript_not_read_in_h3_b".to_string()
                } else {
                    "raw_transcript_not_read_in_h2_phase_b".to_string()
                },
            ],
        },
        failure_reason,
        warnings,
    }
}

fn classify_phase_a_status(result: &CodexLocalPhaseAProcessResult) -> String {
    if result.timed_out {
        return "timed_out".to_string();
    }
    match result.status.as_str() {
        "succeeded"
        | "failed"
        | "readback_unavailable"
        | "readback_failed"
        | "readback_timed_out"
        | "blocked_by_guard" => result.status.clone(),
        _ => "failed".to_string(),
    }
}

fn classify_phase_b_status(result: &CodexLocalPhaseBProcessResult) -> String {
    if result.timed_out {
        return "timed_out".to_string();
    }
    if phase_b_mentions_codex_state_error(result) {
        return "codex_state_error".to_string();
    }
    match result.status.as_str() {
        "succeeded"
        | "failed"
        | "readback_unavailable"
        | "readback_failed"
        | "readback_timed_out"
        | "blocked_by_guard" => result.status.clone(),
        _ => "failed".to_string(),
    }
}

fn phase_b_mentions_codex_state_error(result: &CodexLocalPhaseBProcessResult) -> bool {
    let haystack = result
        .warnings
        .iter()
        .chain(result.failure_code.iter())
        .chain(result.failure_message.iter())
        .map(|text| text.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    (haystack.contains("state db") || haystack.contains("state_"))
        && (haystack.contains("readonly")
            || haystack.contains("read-only")
            || haystack.contains("permission denied")
            || haystack.contains("attempt to write a readonly database"))
}

fn failure_reason_for_phase_a_status(
    status: &str,
    result: &CodexLocalPhaseAProcessResult,
) -> Option<CodexLocalFailureReason> {
    if status == "succeeded" {
        return None;
    }
    Some(CodexLocalFailureReason {
        code: result
            .failure_code
            .clone()
            .unwrap_or_else(|| status.to_string()),
        message: result
            .failure_message
            .clone()
            .unwrap_or_else(|| format!("H2.5 Phase A classified attempt as {status}.")),
        retryable: result.retryable,
        user_action_required: result.user_action_required,
    })
}

fn failure_reason_for_phase_b_status(
    status: &str,
    result: &CodexLocalPhaseBProcessResult,
    operation_id: &str,
) -> Option<CodexLocalFailureReason> {
    if status == "succeeded" {
        return None;
    }
    let mut message = result
        .failure_message
        .clone()
        .unwrap_or_else(|| format!("H2 Phase B classified attempt as {status}."));
    if operation_id == "new_session" {
        message = message.replace("H2 Phase B", "H3-B");
    }
    Some(CodexLocalFailureReason {
        code: if status == "codex_state_error" {
            status.to_string()
        } else {
            result
                .failure_code
                .clone()
                .unwrap_or_else(|| status.to_string())
        },
        message,
        retryable: result.retryable,
        user_action_required: result.user_action_required,
    })
}

fn run_real_codex_process(
    request: &CodexLocalExecutionRequest,
    command_plan: &CodexLocalCommandPlan,
    prompt_body: &str,
    last_message_path: &Path,
    timeout_ms: Option<i64>,
) -> CodexLocalPhaseBProcessResult {
    let Some(output_dir) = last_message_path.parent() else {
        return phase_b_process_failure(
            "phase_b_last_message_parent_missing",
            "last message path has no parent directory",
            false,
            false,
            false,
            vec![],
        );
    };
    if let Err(error) = fs::create_dir_all(output_dir) {
        return phase_b_process_failure(
            "phase_b_last_message_dir_create_failed",
            &format!("failed to create last message directory: {error}"),
            false,
            false,
            false,
            vec![],
        );
    }
    let stderr_path = last_message_path.with_extension("stderr.txt");
    let stderr_file = match fs::File::create(&stderr_path) {
        Ok(file) => file,
        Err(error) => {
            return phase_b_process_failure(
                "phase_b_stderr_create_failed",
                &format!("failed to create stderr file: {error}"),
                false,
                false,
                false,
                vec![],
            );
        }
    };

    let mut command = Command::new(&command_plan.program);
    for arg in &command_plan.argv {
        if arg == "<workbench-managed-last-message>" {
            command.arg(last_message_path);
        } else {
            command.arg(arg);
        }
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return phase_b_process_failure(
                "phase_b_spawn_failed",
                &format!("failed to spawn codex process: {error}"),
                false,
                false,
                false,
                vec![stderr_summary_warning(&stderr_path)],
            );
        }
    };
    let prompt_sent;
    if let Some(mut stdin) = child.stdin.take() {
        match stdin.write_all(prompt_body.as_bytes()) {
            Ok(()) => {
                prompt_sent = true;
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return phase_b_process_failure(
                    "phase_b_stdin_write_failed",
                    &format!("failed to write prompt to codex stdin: {error}"),
                    false,
                    true,
                    true,
                    vec![stderr_summary_warning(&stderr_path)],
                );
            }
        }
    } else {
        let _ = child.kill();
        let _ = child.wait();
        return phase_b_process_failure(
            "phase_b_stdin_unavailable",
            "codex stdin was unavailable",
            false,
            true,
            true,
            vec![stderr_summary_warning(&stderr_path)],
        );
    }

    let mut timed_out = false;
    let status = if let Some(timeout_ms) = timeout_ms {
        let timeout = Duration::from_millis(timeout_ms.max(1) as u64);
        let started = SystemTime::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if started.elapsed().unwrap_or_default() >= timeout {
                        timed_out = true;
                        let _ = child.kill();
                        match child.wait() {
                            Ok(status) => break status,
                            Err(error) => {
                                return phase_b_process_failure(
                                    "phase_b_timeout_wait_failed",
                                    &format!("failed to wait for timed out codex process: {error}"),
                                    prompt_sent,
                                    true,
                                    true,
                                    vec![stderr_summary_warning(&stderr_path)],
                                );
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => {
                    return phase_b_process_failure(
                        "phase_b_wait_failed",
                        &format!("failed to wait for codex process: {error}"),
                        prompt_sent,
                        true,
                        true,
                        vec![stderr_summary_warning(&stderr_path)],
                    );
                }
            }
        }
    } else {
        match child.wait() {
            Ok(status) => status,
            Err(error) => {
                return phase_b_process_failure(
                    "phase_b_wait_failed",
                    &format!("failed to wait for codex process: {error}"),
                    prompt_sent,
                    true,
                    true,
                    vec![stderr_summary_warning(&stderr_path)],
                );
            }
        }
    };

    let exit_code = status.code().unwrap_or(-1);
    let last_message = fs::read_to_string(last_message_path).unwrap_or_default();
    let readback_available = !last_message.trim().is_empty();
    let stderr_warning = stderr_summary_warning(&stderr_path);
    let _ = fs::remove_file(&stderr_path);
    let mut warnings = vec![
        "phase_b_real_codex_process_spawned".to_string(),
        "prompt_body_not_persisted_by_runner".to_string(),
        "raw_stdout_stderr_not_persisted".to_string(),
        "raw_transcript_not_read".to_string(),
    ];
    if !stderr_warning.is_empty() {
        warnings.push(stderr_warning);
    }
    if prompt_sent && request.sandbox != "read-only" {
        warnings.push("writes_project_files_unverified_requires_hash_manifest".to_string());
    }
    let status = if timed_out {
        "timed_out"
    } else if exit_code == 0 && readback_available {
        "succeeded"
    } else if exit_code == 0 {
        "readback_unavailable"
    } else {
        "failed"
    };
    let readback_status = if timed_out {
        "readback_timed_out"
    } else if readback_available {
        "succeeded"
    } else if exit_code == 0 {
        "readback_unavailable"
    } else {
        "readback_failed"
    };
    let codex_state_error =
        classify_phase_b_stderr_for_codex_state_error(&warnings, exit_code, status);
    let failure_code = if status == "succeeded" {
        None
    } else if codex_state_error {
        Some("codex_state_error".to_string())
    } else {
        Some(status.to_string())
    };
    let failure_message = if status == "succeeded" {
        None
    } else if codex_state_error {
        Some(format!(
            "H2 Phase B codex process could not write Codex native state; exit_code={exit_code}. Retry requires a writable Codex environment."
        ))
    } else {
        Some(format!(
            "H2 Phase B codex process ended as {status}; exit_code={exit_code}."
        ))
    };
    if codex_state_error {
        warnings.push("codex_state_readonly_or_permission_denied".to_string());
    }
    CodexLocalPhaseBProcessResult {
        runner_kind: "codex_local_phase_b_real_process_runner".to_string(),
        status: status.to_string(),
        exit_code: Some(exit_code),
        timed_out,
        prompt_sent,
        real_codex_executed: true,
        writes_codex_home: true,
        writes_project_files: false,
        readback_status: readback_status.to_string(),
        readback_attempted: true,
        readback_result_count: if readback_available { Some(1) } else { None },
        last_message_path: Some(last_message_path.display().to_string()),
        failure_code,
        failure_message,
        retryable: status != "succeeded" && !timed_out,
        user_action_required: status != "succeeded",
        warnings,
    }
}

fn classify_phase_b_stderr_for_codex_state_error(
    warnings: &[String],
    exit_code: i32,
    status: &str,
) -> bool {
    if exit_code == 0 || status == "succeeded" {
        return false;
    }
    let haystack = warnings
        .iter()
        .map(|warning| warning.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    (haystack.contains("state db") || haystack.contains("state_"))
        && (haystack.contains("readonly")
            || haystack.contains("read-only")
            || haystack.contains("permission denied")
            || haystack.contains("attempt to write a readonly database"))
}

fn phase_b_process_failure(
    code: &str,
    message: &str,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    warnings: Vec<String>,
) -> CodexLocalPhaseBProcessResult {
    CodexLocalPhaseBProcessResult {
        runner_kind: "codex_local_phase_b_real_process_runner".to_string(),
        status: "failed".to_string(),
        exit_code: None,
        timed_out: false,
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_project_files: false,
        readback_status: "readback_unavailable".to_string(),
        readback_attempted: false,
        readback_result_count: None,
        last_message_path: None,
        failure_code: Some(code.to_string()),
        failure_message: Some(message.to_string()),
        retryable: true,
        user_action_required: true,
        warnings,
    }
}

fn stderr_summary_warning(stderr_path: &Path) -> String {
    let text = fs::read_to_string(stderr_path).unwrap_or_default();
    let summary = compact_summary(&text);
    if summary.is_empty() {
        String::new()
    } else {
        format!("stderr_summary:{summary}")
    }
}

fn compact_summary(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(500).collect()
}

fn request_timeout_ms(request: &CodexLocalExecutionRequest) -> Option<i64> {
    request
        .warnings
        .iter()
        .find_map(|warning| warning.strip_prefix("timeout_ms:"))
        .and_then(|raw| raw.parse::<i64>().ok())
}

fn phase_b_safe_guard_warnings(warnings: &[String]) -> Vec<String> {
    warnings
        .iter()
        .filter(|warning| {
            !matches!(
                warning.as_str(),
                "no_real_codex_exec"
                    | "no_real_codex_exec_resume"
                    | "prompt_not_sent"
                    | "codex_home_not_touched"
                    | "h1_contract_only"
                    | "h1_preview_not_executed"
                    | "codex_local_guard_only_no_runner_call"
            )
        })
        .cloned()
        .collect()
}

fn check_required_binding(
    request: &CodexLocalExecutionRequest,
    reasons: &mut Vec<String>,
    required_fixes: &mut Vec<String>,
) {
    for (field, value) in [
        ("project_id", request.project_id.as_str()),
        ("project_root", request.project_root.as_str()),
        ("workflow_id", request.workflow_id.as_str()),
        ("node_id", request.node_id.as_str()),
        ("target_cwd", request.target_cwd.as_str()),
        ("prompt_source_kind", request.prompt_source_kind.as_str()),
        ("requested_by", request.requested_by.as_str()),
    ] {
        if value.trim().is_empty() {
            reasons.push(format!("{field}_missing"));
            required_fixes.push(format!("缺少 {field}。"));
        }
    }
    if request.operation_id != "new_session"
        && blank_opt(request.session_id.as_deref())
        && blank_opt(request.work_item_id.as_deref())
    {
        reasons.push("session_or_work_item_binding_missing".to_string());
        required_fixes.push("必须绑定 session_id 或 work_item_id。".to_string());
    }
}

fn check_paths(
    request: &CodexLocalExecutionRequest,
    reasons: &mut Vec<String>,
    required_fixes: &mut Vec<String>,
) {
    if request.allowed_write_roots.is_empty() && request.sandbox != "read-only" {
        reasons.push("allowed_write_roots_missing".to_string());
        required_fixes.push("必须声明 allowed_write_roots。".to_string());
    }
    for (field, value) in [
        ("project_root", request.project_root.as_str()),
        ("target_cwd", request.target_cwd.as_str()),
    ] {
        if !safe_absolute_path(value) {
            reasons.push(format!("{field}_unsafe_path"));
            required_fixes.push(format!("{field} 必须是绝对路径，且不能包含 .. 逃逸。"));
        }
    }
    if request
        .allowed_write_roots
        .iter()
        .any(|root| !safe_absolute_path(root))
    {
        reasons.push("allowed_write_root_unsafe_path".to_string());
        required_fixes.push("allowed_write_roots 必须是绝对路径，且不能包含 .. 逃逸。".to_string());
    }
    if !path_within_scope(&request.target_cwd, &request.project_root)
        && !request
            .allowed_write_roots
            .iter()
            .any(|root| path_within_scope(&request.target_cwd, root))
    {
        reasons.push("target_cwd_out_of_scope".to_string());
        required_fixes
            .push("target_cwd 必须落在 project_root 或 allowed_write_roots 内。".to_string());
    }
    if request
        .allowed_write_roots
        .iter()
        .any(|root| !path_within_scope(root, &request.project_root))
    {
        reasons.push("allowed_write_root_out_of_project_scope".to_string());
        required_fixes.push("allowed_write_roots 必须落在 project_root 内。".to_string());
    }
}

fn check_secret_deny_list(
    request: &CodexLocalExecutionRequest,
    reasons: &mut Vec<String>,
    required_fixes: &mut Vec<String>,
) {
    let values = [
        request.project_root.as_str(),
        request.target_cwd.as_str(),
        request.prompt_ref.as_str(),
        request.prompt_source_kind.as_str(),
        request.prompt_summary.as_str(),
        request.requested_by.as_str(),
        request.readback_plan.strategy.as_str(),
        request.readback_plan.unavailable_behavior.as_str(),
        request.readback_plan.trust_policy.as_str(),
    ];
    let sensitive_hit = values
        .iter()
        .any(|value| contains_sensitive_fragment(value))
        || request
            .session_id
            .as_deref()
            .is_some_and(contains_sensitive_fragment)
        || request
            .work_item_id
            .as_deref()
            .is_some_and(contains_sensitive_fragment)
        || request
            .continuation_id
            .as_deref()
            .is_some_and(contains_sensitive_fragment)
        || request
            .authorization_scope_id
            .as_deref()
            .is_some_and(contains_sensitive_fragment)
        || request
            .allowed_write_roots
            .iter()
            .any(|root| contains_sensitive_fragment(root))
        || request
            .readback_plan
            .expected_sources
            .iter()
            .any(|source| contains_sensitive_fragment(source))
        || request
            .readback_plan
            .warnings
            .iter()
            .any(|warning| contains_sensitive_fragment(warning));
    if sensitive_hit {
        reasons.push("secret_deny_list_hit".to_string());
        required_fixes
            .push("请求字段命中 secret/auth/token/.env/.codex 等 deny list。".to_string());
    }
}

fn check_prompt_boundary(
    request: &CodexLocalExecutionRequest,
    reasons: &mut Vec<String>,
    required_fixes: &mut Vec<String>,
) {
    if request.prompt_summary.trim().is_empty() {
        reasons.push("prompt_summary_missing".to_string());
        required_fixes.push("必须提供脱敏 prompt_summary。".to_string());
    }
    if request.prompt_sha256.len() != 64
        || !request
            .prompt_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        reasons.push("prompt_sha256_invalid".to_string());
        required_fixes.push("必须提供 64 位 hex prompt_sha256，而不是 prompt 正文。".to_string());
    }
    if request.prompt_ref.trim().is_empty() {
        reasons.push("prompt_ref_missing".to_string());
        required_fixes.push("必须提供工作台管理的 prompt_ref。".to_string());
    }
}

fn check_readback_plan(
    request: &CodexLocalExecutionRequest,
    reasons: &mut Vec<String>,
    required_fixes: &mut Vec<String>,
) {
    let plan = &request.readback_plan;
    if plan.strategy != "required" || !plan.required {
        reasons.push("readback_plan_not_required".to_string());
        required_fixes
            .push("H1 契约必须要求 readback plan，且 unavailable/failed 不能当作 0。".to_string());
    }
    if plan.expected_sources.is_empty() {
        reasons.push("readback_expected_sources_missing".to_string());
        required_fixes.push("readback_plan.expected_sources 不能为空。".to_string());
    }
    if plan.unavailable_behavior.trim().is_empty() {
        reasons.push("readback_unavailable_behavior_missing".to_string());
        required_fixes.push("必须说明 readback unavailable 的处理方式。".to_string());
    }
}

fn command_plan_for(request: &CodexLocalExecutionRequest) -> CodexLocalCommandPlan {
    let mut argv = vec!["exec".to_string()];
    argv.push("-C".to_string());
    argv.push(request.target_cwd.clone());
    argv.push("--sandbox".to_string());
    argv.push(request.sandbox.clone());
    for root in &request.allowed_write_roots {
        argv.push("--add-dir".to_string());
        argv.push(root.clone());
    }
    if request.operation_id == "resume" {
        argv.push("resume".to_string());
        argv.push("--skip-git-repo-check".to_string());
        argv.push("--json".to_string());
        argv.push("--output-last-message".to_string());
        argv.push("<workbench-managed-last-message>".to_string());
        argv.push(
            request
                .session_id
                .clone()
                .unwrap_or_else(|| "<missing-session>".to_string()),
        );
    } else {
        if request.operation_id == "new_session" {
            argv.push("--skip-git-repo-check".to_string());
        }
        argv.push("--json".to_string());
        argv.push("--output-last-message".to_string());
        argv.push("<workbench-managed-last-message>".to_string());
    }
    CodexLocalCommandPlan {
        program: "codex".to_string(),
        argv,
        stdin_prompt_ref: request.prompt_ref.clone(),
        stdin_prompt_sha256: request.prompt_sha256.clone(),
        prompt_in_command: false,
        shell_invocation: false,
        redacted_preview: format!(
            "codex {} <stdin:{}> # workbench-managed prompt",
            match request.operation_id.as_str() {
                "resume" => "exec resume",
                "new_session" => "exec new-session",
                _ => "exec",
            },
            short_hash(&request.prompt_sha256)
        ),
        sensitive_omissions: vec!["prompt_body".to_string(), "raw_runner_output".to_string()],
        warnings: vec![
            "argv_only_no_shell".to_string(),
            "prompt_via_stdin_ref_only".to_string(),
            "workbench_managed_command_plan".to_string(),
        ],
    }
}

fn has_duplicate_running_attempt(active_attempts: &[CodexLocalActiveAttempt]) -> bool {
    active_attempts
        .iter()
        .any(|attempt| crate::h4_execution_boundary::is_h4_active_attempt_status(&attempt.status))
}

fn h1_warnings() -> Vec<String> {
    vec![
        "h1_contract_only".to_string(),
        "no_real_codex_exec".to_string(),
        "no_real_codex_exec_resume".to_string(),
        "prompt_not_sent".to_string(),
        "codex_home_not_touched".to_string(),
        "planned_adapters_not_connected".to_string(),
    ]
}

fn stable_request_id(request: &CodexLocalExecutionRequest) -> String {
    format!(
        "codex-local-request:h1:{}",
        sha256_hex(
            &[
                normalize(&request.adapter_id),
                normalize(&request.operation_id),
                normalize(&request.project_id),
                normalize(&request.workflow_id),
                normalize(&request.node_id),
                normalize(request.session_id.as_deref().unwrap_or_default()),
                normalize(request.work_item_id.as_deref().unwrap_or_default()),
                normalize(&request.prompt_sha256),
            ]
            .join("\0")
        )
    )
}

fn blank_opt(value: Option<&str>) -> bool {
    value.unwrap_or("").trim().is_empty()
}

fn path_within_scope(path: &str, root: &str) -> bool {
    if !safe_absolute_path(path) || !safe_absolute_path(root) {
        return false;
    }
    let path = Path::new(path);
    let root = Path::new(root);
    path == root || path.starts_with(root)
}

fn safe_absolute_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn contains_sensitive_fragment(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("/.codex")
        || normalized.contains("\\.codex")
        || normalized.ends_with(".codex")
        || normalized.contains(".env")
        || normalized.contains("keychain")
        || normalized.contains("oauth")
        || normalized.contains("provider credential")
        || normalized.contains("credential")
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("/auth")
        || normalized.contains("\\auth")
        || normalized.contains("full transcript")
        || normalized.contains("完整 transcript")
}

#[cfg(test)]
mod tests {
    use super::*;

    // 引擎解封·第一刀的真跑验证(高危#1)。默认 #[ignore],只在显式
    // `cargo test --lib real_run_workflow_node_adapter -- --ignored --nocapture` 时起真 codex。
    // 直接验适配器:在固定测试项目里真起一次 codex、新建会话、让它写一个证明文件。
    #[test]
    #[ignore = "spawns real codex in the fixed test project"]
    fn real_run_workflow_node_adapter() {
        let runner = RealWorkflowNodeCodexRunner;
        let last_message_path =
            std::env::temp_dir().join("workflow-node-real-run-last-message.txt");
        let test_root = "/Users/yoyi/codex-workflow-mario-test";
        let options = crate::CodexResumeRequestOptions {
            prompt_kind: "workflow_node_business".to_string(),
            execution_cwd: Some(std::path::PathBuf::from(test_root)),
            sandbox_mode: Some("workspace-write".to_string()),
            allowed_write_roots: vec![std::path::PathBuf::from(test_root)],
            timeout_seconds: Some(180),
        };
        let result = crate::CodexResumeRunner::resume_with_options(
            &runner,
            "",
            "在当前目录创建文件 workflow-real-run-proof.txt，写入一行：workflow engine real run ok。完成后用一句话说明你做了什么。",
            &last_message_path,
            &options,
        );
        println!("[REAL_RUN] last_message_path = {}", last_message_path.display());
        println!("[REAL_RUN] result = {result:?}");
        let (run, _opts) = result.expect("adapter real run should succeed");
        println!("[REAL_RUN] exit_code={} timed_out={}", run.exit_code, run.timed_out);
        assert_eq!(run.exit_code, 0, "codex exit code should be 0");
    }
    use crate::CodexLocalReadbackPlan;

    #[test]
    fn codex_local_guard_allows_confirmed_structured_dry_run_only() {
        let request = safe_request();
        let runner = FakeCodexLocalRunner;
        let attempt = runner.run_dry(request, "2026-06-07T12:00:00Z");
        assert_eq!(attempt.status, "dry_run_succeeded");
        assert_eq!(attempt.runner_kind, "fake_dry_run");
        assert_eq!(
            attempt.execution_level,
            "h1_contract_only_no_real_execution"
        );
        assert!(!attempt.prompt_sent);
        assert!(!attempt.real_codex_executed);
        assert!(!attempt.writes_codex_home);
        assert!(!attempt.writes_project_files);
        assert!(!attempt.writes_workbench_state);
        assert_eq!(attempt.readback_result.status, "readback_unavailable");
        assert_eq!(attempt.readback_result.result_count, None);
        let command_plan = attempt.command_plan.expect("dry run command plan");
        assert_eq!(command_plan.program, "codex");
        assert!(!command_plan.shell_invocation);
        assert!(!command_plan.prompt_in_command);
        assert!(command_plan.argv.iter().any(|arg| arg == "resume"));
        assert!(command_plan
            .warnings
            .contains(&"argv_only_no_shell".to_string()));
    }

    #[test]
    fn codex_local_guard_allows_new_session_noop_without_existing_session() {
        let mut request = safe_request();
        request.operation_id = "new_session".to_string();
        request.session_id = None;
        request.prompt_source_kind = "h3_new_session_task_package".to_string();
        request.prompt_ref = "workbench-managed-prompt:h3-new-session".to_string();

        let attempt = FakeCodexLocalRunner.run_dry(request, "2026-06-07T13:00:00Z");

        assert_eq!(attempt.status, "dry_run_succeeded");
        assert!(!attempt.prompt_sent);
        assert!(!attempt.real_codex_executed);
        assert!(!attempt.writes_codex_home);
        assert_eq!(attempt.readback_result.status, "readback_unavailable");
        assert_eq!(attempt.readback_result.result_count, None);
        let command_plan = attempt.command_plan.expect("new session command plan");
        assert_eq!(command_plan.program, "codex");
        assert!(!command_plan.argv.iter().any(|arg| arg == "resume"));
        assert!(!command_plan
            .argv
            .iter()
            .any(|arg| arg.contains("thread:h1")));
        assert!(command_plan
            .argv
            .iter()
            .any(|arg| arg == "--skip-git-repo-check"));
        assert!(!command_plan.shell_invocation);
        assert!(!command_plan.prompt_in_command);
        assert!(command_plan.redacted_preview.contains("exec new-session"));
    }

    #[test]
    fn codex_local_guard_blocks_new_session_without_work_item_binding() {
        let mut request = safe_request();
        request.operation_id = "new_session".to_string();
        request.session_id = None;
        request.work_item_id = None;

        let guard = inspect_codex_local_execution_guard(&request);

        assert!(guard.blocks_execution);
        assert!(guard
            .reasons
            .contains(&"new_session_requires_work_item_id".to_string()));
    }

    #[test]
    fn codex_local_guard_blocks_secret_paths_and_prompt_hash_gap() {
        let mut request = safe_request();
        request.target_cwd = "/tmp/h1-project/.codex".to_string();
        request.prompt_sha256 = "not-a-hash".to_string();
        let guard = inspect_codex_local_execution_guard(&request);
        assert!(guard.blocks_execution);
        assert!(!guard.allows_dry_run);
        assert!(guard.reasons.contains(&"secret_deny_list_hit".to_string()));
        assert!(guard.reasons.contains(&"prompt_sha256_invalid".to_string()));
    }

    #[test]
    fn codex_local_guard_blocks_planned_adapter_duplicate_and_missing_confirmation() {
        let mut request = safe_request();
        request.adapter_id = "claude-code".to_string();
        request.user_confirmation_state = "missing".to_string();
        request.active_attempts = vec![CodexLocalActiveAttempt {
            attempt_id: "attempt-running".to_string(),
            status: "running".to_string(),
            continuation_id: Some("continuation-1".to_string()),
        }];
        let attempt = FakeCodexLocalRunner.run_dry(request, "2026-06-07T12:01:00Z");
        assert_eq!(attempt.status, "dry_run_blocked");
        assert!(attempt.failure_reason.is_some());
        assert!(attempt.guard.duplicate_running_attempt);
        assert!(attempt.guard.requires_user_confirmation);
        assert!(attempt
            .guard
            .reasons
            .contains(&"adapter_not_codex_local".to_string()));
        assert!(attempt
            .guard
            .reasons
            .contains(&"duplicate_running_attempt".to_string()));
        assert!(attempt
            .guard
            .reasons
            .contains(&"user_confirmation_required".to_string()));
        assert!(!attempt.real_codex_executed);
        assert_eq!(attempt.readback_result.result_count, None);
    }

    #[test]
    fn codex_local_guard_blocks_path_escape_and_sensitive_readback_refs() {
        let mut request = safe_request();
        request.target_cwd = "/tmp/h1-project/../escape".to_string();
        request
            .readback_plan
            .expected_sources
            .push("/Users/yoyi/.codex/sessions/raw.jsonl".to_string());

        let guard = inspect_codex_local_execution_guard(&request);

        assert!(guard.blocks_execution);
        assert!(guard
            .reasons
            .contains(&"target_cwd_unsafe_path".to_string()));
        assert!(guard
            .reasons
            .contains(&"target_cwd_out_of_scope".to_string()));
        assert!(guard.reasons.contains(&"secret_deny_list_hit".to_string()));
    }

    #[test]
    fn h2_phase_a_noop_runner_records_no_real_execution() {
        let request = safe_request();
        let attempt = run_h2_phase_a_with_runner(
            request,
            "2026-06-07T12:02:00Z",
            &NoopCodexLocalPhaseAProcessRunner,
        );

        assert_eq!(attempt.status, "readback_unavailable");
        assert_eq!(
            attempt.execution_level,
            "h2_phase_a_runner_path_no_real_codex"
        );
        assert_eq!(
            attempt.runner_kind,
            "codex_local_phase_a_noop_process_runner"
        );
        assert!(!attempt.prompt_sent);
        assert!(!attempt.real_codex_executed);
        assert!(!attempt.writes_codex_home);
        assert!(!attempt.writes_project_files);
        assert_eq!(attempt.readback_result.result_count, None);
        let plan = attempt.command_plan.expect("phase A command plan");
        assert_eq!(plan.program, "codex");
        assert!(!plan.shell_invocation);
        assert!(!plan.prompt_in_command);
        assert!(plan.argv.iter().any(|arg| arg == "resume"));
        assert!(attempt
            .warnings
            .contains(&"h2_phase_a_runner_path_no_real_codex".to_string()));
    }

    #[test]
    fn h2_phase_a_runner_classifies_timeout_and_readback_failed_without_zero_results() {
        struct FakePhaseARunner {
            status: &'static str,
            timed_out: bool,
            readback_status: &'static str,
        }

        impl CodexLocalPhaseAProcessRunner for FakePhaseARunner {
            fn run_phase_a(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
            ) -> CodexLocalPhaseAProcessResult {
                CodexLocalPhaseAProcessResult {
                    runner_kind: "fake_phase_a_process".to_string(),
                    status: self.status.to_string(),
                    exit_code: Some(1),
                    timed_out: self.timed_out,
                    readback_status: self.readback_status.to_string(),
                    readback_attempted: true,
                    readback_result_count: Some(0),
                    failure_code: Some(self.status.to_string()),
                    failure_message: Some("fake phase A result".to_string()),
                    retryable: true,
                    user_action_required: false,
                    warnings: vec!["fake_phase_a_runner".to_string()],
                }
            }
        }

        let timeout = run_h2_phase_a_with_runner(
            safe_request(),
            "2026-06-07T12:03:00Z",
            &FakePhaseARunner {
                status: "failed",
                timed_out: true,
                readback_status: "timed_out",
            },
        );
        assert_eq!(timeout.status, "timed_out");
        assert_eq!(timeout.readback_result.result_count, None);
        assert!(!timeout.real_codex_executed);

        let readback_failed = run_h2_phase_a_with_runner(
            safe_request(),
            "2026-06-07T12:04:00Z",
            &FakePhaseARunner {
                status: "readback_failed",
                timed_out: false,
                readback_status: "readback_failed",
            },
        );
        assert_eq!(readback_failed.status, "readback_failed");
        assert_eq!(readback_failed.readback_result.result_count, None);
        assert!(!readback_failed.prompt_sent);
    }

    #[test]
    fn h4_unknown_result_statuses_keep_result_count_null() {
        for status in [
            "readback_unavailable",
            "readback_failed",
            "readback_timed_out",
            "timed_out",
            "not_attempted",
            "blocked_by_guard",
            "duplicate_blocked",
            "user_rejected",
            "cancel_requested",
            "stale_cancelled",
        ] {
            assert_eq!(
                crate::h4_execution_boundary::h4_result_count(status, status, Some(0)),
                None,
                "{status} must not look like a trusted zero-result readback"
            );
        }
        assert_eq!(
            crate::h4_execution_boundary::h4_result_count("succeeded", "succeeded", Some(0)),
            Some(0)
        );
    }

    #[test]
    fn h2_phase_b_fake_runner_records_real_execution_flags_and_readback() {
        struct FakePhaseBRunner;

        impl CodexLocalPhaseBProcessRunner for FakePhaseBRunner {
            fn run_phase_b(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                last_message_path: &Path,
                _timeout_ms: Option<i64>,
            ) -> CodexLocalPhaseBProcessResult {
                CodexLocalPhaseBProcessResult {
                    runner_kind: "fake_phase_b_process".to_string(),
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
                    warnings: vec!["fake_phase_b_runner".to_string()],
                }
            }
        }

        let attempt = run_h2_phase_b_with_runner(
            safe_request(),
            "2026-06-07T12:05:00Z",
            "safe prompt",
            Path::new("/tmp/h2-phase-b-last-message.txt"),
            &FakePhaseBRunner,
        );

        assert_eq!(attempt.status, "succeeded");
        assert_eq!(attempt.runner_kind, "fake_phase_b_process");
        assert_eq!(attempt.execution_level, "h2_phase_b_real_codex_resume");
        assert!(attempt.prompt_sent);
        assert!(attempt.real_codex_executed);
        assert!(attempt.writes_codex_home);
        assert!(attempt.writes_workbench_state);
        assert_eq!(attempt.readback_result.result_count, Some(1));
        assert!(!attempt
            .warnings
            .contains(&"no_real_codex_exec_resume".to_string()));
        let plan = attempt.command_plan.expect("phase B command plan");
        assert_eq!(plan.program, "codex");
        assert!(!plan.shell_invocation);
        assert!(!plan.prompt_in_command);
        assert!(plan.argv.iter().any(|arg| arg == "resume"));
    }

    #[test]
    fn h2_phase_b_fake_runner_keeps_failed_readback_count_unknown() {
        struct FakeReadbackFailedRunner;

        impl CodexLocalPhaseBProcessRunner for FakeReadbackFailedRunner {
            fn run_phase_b(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                _last_message_path: &Path,
                _timeout_ms: Option<i64>,
            ) -> CodexLocalPhaseBProcessResult {
                CodexLocalPhaseBProcessResult {
                    runner_kind: "fake_phase_b_process".to_string(),
                    status: "readback_failed".to_string(),
                    exit_code: Some(0),
                    timed_out: false,
                    prompt_sent: true,
                    real_codex_executed: true,
                    writes_codex_home: true,
                    writes_project_files: false,
                    readback_status: "readback_failed".to_string(),
                    readback_attempted: true,
                    readback_result_count: Some(0),
                    last_message_path: None,
                    failure_code: Some("readback_failed".to_string()),
                    failure_message: Some("fake readback failed".to_string()),
                    retryable: true,
                    user_action_required: true,
                    warnings: vec!["fake_phase_b_runner".to_string()],
                }
            }
        }

        let attempt = run_h2_phase_b_with_runner(
            safe_request(),
            "2026-06-07T12:06:00Z",
            "safe prompt",
            Path::new("/tmp/h2-phase-b-last-message.txt"),
            &FakeReadbackFailedRunner,
        );

        assert_eq!(attempt.status, "readback_failed");
        assert_eq!(attempt.readback_result.result_count, None);
        assert!(attempt.prompt_sent);
        assert!(attempt.real_codex_executed);
        assert!(attempt.failure_reason.is_some());
    }

    #[test]
    fn h2_phase_b_classifies_codex_state_readonly_without_zero_results() {
        struct FakeCodexStateReadonlyRunner;

        impl CodexLocalPhaseBProcessRunner for FakeCodexStateReadonlyRunner {
            fn run_phase_b(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                _last_message_path: &Path,
                _timeout_ms: Option<i64>,
            ) -> CodexLocalPhaseBProcessResult {
                CodexLocalPhaseBProcessResult {
                    runner_kind: "fake_phase_b_process".to_string(),
                    status: "failed".to_string(),
                    exit_code: Some(1),
                    timed_out: false,
                    prompt_sent: true,
                    real_codex_executed: true,
                    writes_codex_home: true,
                    writes_project_files: false,
                    readback_status: "readback_failed".to_string(),
                    readback_attempted: true,
                    readback_result_count: Some(0),
                    last_message_path: None,
                    failure_code: Some("failed".to_string()),
                    failure_message: Some(
                        "failed to open state db at /Users/yoyi/.codex/state_5.sqlite".to_string(),
                    ),
                    retryable: true,
                    user_action_required: true,
                    warnings: vec![
                        "stderr_summary:attempt to write a readonly database".to_string()
                    ],
                }
            }
        }

        let attempt = run_h2_phase_b_with_runner(
            safe_request(),
            "2026-06-10T12:06:00Z",
            "safe prompt",
            Path::new("/tmp/h2-phase-b-last-message.txt"),
            &FakeCodexStateReadonlyRunner,
        );

        assert_eq!(attempt.status, "codex_state_error");
        assert_eq!(attempt.readback_result.status, "readback_failed");
        assert_eq!(attempt.readback_result.result_count, None);
        assert!(attempt.prompt_sent);
        assert!(attempt.real_codex_executed);
        assert!(attempt.writes_codex_home);
        let failure = attempt.failure_reason.expect("failure reason");
        assert_eq!(failure.code, "codex_state_error");
        assert!(failure.message.contains("state_5.sqlite"));
    }

    fn safe_request() -> CodexLocalExecutionRequest {
        CodexLocalExecutionRequest {
            request_version: 1,
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            project_id: "project:h1".to_string(),
            project_root: "/tmp/h1-project".to_string(),
            workflow_id: "workflow:h1".to_string(),
            node_id: "node:h1".to_string(),
            session_id: Some("thread:h1".to_string()),
            work_item_id: Some("work:h1".to_string()),
            continuation_id: Some("continuation:h1".to_string()),
            target_cwd: "/tmp/h1-project".to_string(),
            allowed_write_roots: vec!["/tmp/h1-project".to_string()],
            sandbox: "workspace-write".to_string(),
            prompt_source_kind: "task_package_summary".to_string(),
            prompt_summary: "脱敏 H1 prompt 摘要".to_string(),
            prompt_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            prompt_ref: "workbench-managed-prompt:h1".to_string(),
            readback_plan: CodexLocalReadbackPlan {
                strategy: "required".to_string(),
                required: true,
                expected_sources: vec!["runtime_log_ref".to_string(), "audit_ref".to_string()],
                unavailable_behavior: "readback unavailable 不等于 0 条结果".to_string(),
                trust_policy: "must_be_explicit_readback_result".to_string(),
                warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
            },
            requested_by: "h1-test".to_string(),
            user_confirmation_state: "confirmed".to_string(),
            authorization_scope_id: Some("authorization:h1".to_string()),
            runtime_log_refs: vec![CodexLocalRuntimeLogRef {
                ref_id: "runtime-log:h1:existing".to_string(),
                category: "permission_wait".to_string(),
                status: "confirmed".to_string(),
                redaction_status: "redacted_safe_summary".to_string(),
            }],
            audit_refs: vec![CodexLocalAuditRef {
                ref_id: "audit:h1:confirmed".to_string(),
                event_type: "codex_local_execution_confirmed".to_string(),
                actor_role: "user".to_string(),
                decision: "confirmed".to_string(),
            }],
            active_attempts: vec![],
            warnings: vec![],
        }
    }
}
