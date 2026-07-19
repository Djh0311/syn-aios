use std::collections::VecDeque;
use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const REAL_CLI_INVALID_RESUME_STDERR: &str = concat!(
    "Reading prompt from stdin...\n",
    "Error: thread/resume: thread/resume failed: no rollout found for thread id ",
    "019f75cb-783f-74a1-91fe-c910de94209e (code -32600)"
);

const S1B_LIVE_APPROVAL_HARNESS_CONFIRM_ENV: &str =
    "SYN_S1B_LIVE_MCP_TOOL_APPROVAL_HARNESS_CONFIRM";
const S1B_LIVE_APPROVAL_HARNESS_CONFIRM_VALUE: &str =
    "CONFIRMED_TEST_ONLY_SUBMIT_PROPOSAL_APPROVAL";
const S1B_LIVE_REAL_CODEX_ENV: &str = "SYN_S1B_LIVE_REAL_CODEX";
const S1B_LIVE_CODEX_APPROVAL_WRAPPER: &str = r#"#!/bin/sh
set -eu

if [ "$1" != "exec" ]; then
  echo "S1B live approval harness only permits codex exec" >&2
  exit 64
fi

shift
status=0
if [ "${5-}" = "resume" ]; then
  exec_cwd_flag=$1
  exec_cwd=$2
  sandbox_flag=$3
  sandbox_value=$4
  shift 5
  "$SYN_S1B_LIVE_REAL_CODEX" --strict-config \
    exec \
    "$exec_cwd_flag" "$exec_cwd" "$sandbox_flag" "$sandbox_value" \
    resume \
    -c 'mcp_servers.supervisor_orchestrator.enabled_tools=["submit_proposal"]' \
    -c 'mcp_servers.supervisor_orchestrator.tools.submit_proposal.approval_mode="approve"' \
    "$@" || status=$?
else
  "$SYN_S1B_LIVE_REAL_CODEX" --strict-config \
    -c 'mcp_servers.supervisor_orchestrator.enabled_tools=["submit_proposal"]' \
    -c 'mcp_servers.supervisor_orchestrator.tools.submit_proposal.approval_mode="approve"' \
    exec \
    "$@" || status=$?
fi
exit "$status"
"#;

struct S1bLiveApprovalHarnessGuard {
    original_path: OsString,
    wrapper_root: PathBuf,
}

impl Drop for S1bLiveApprovalHarnessGuard {
    fn drop(&mut self) {
        std::env::set_var("PATH", &self.original_path);
        let _ = fs::remove_dir_all(&self.wrapper_root);
    }
}

fn install_s1b_live_approval_harness() -> S1bLiveApprovalHarnessGuard {
    assert_eq!(
        std::env::var(S1B_LIVE_APPROVAL_HARNESS_CONFIRM_ENV).as_deref(),
        Ok(S1B_LIVE_APPROVAL_HARNESS_CONFIRM_VALUE),
        "set {S1B_LIVE_APPROVAL_HARNESS_CONFIRM_ENV}={S1B_LIVE_APPROVAL_HARNESS_CONFIRM_VALUE} after explicit user authorization"
    );
    let real_codex = PathBuf::from(
        std::env::var_os(S1B_LIVE_REAL_CODEX_ENV)
            .expect("set SYN_S1B_LIVE_REAL_CODEX to the absolute real codex executable"),
    );
    assert!(
        real_codex.is_absolute() && real_codex.is_file(),
        "SYN_S1B_LIVE_REAL_CODEX must be an absolute executable file"
    );
    #[cfg(unix)]
    assert_ne!(
        fs::metadata(&real_codex)
            .expect("inspect real codex executable")
            .permissions()
            .mode()
            & 0o111,
        0,
        "SYN_S1B_LIVE_REAL_CODEX must be executable"
    );

    let wrapper_root = std::env::temp_dir().join(format!(
        "s1b-live-codex-approval-harness-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir(&wrapper_root).expect("create S1B live approval wrapper root");
    let wrapper = wrapper_root.join("codex");
    fs::write(&wrapper, S1B_LIVE_CODEX_APPROVAL_WRAPPER).expect("write S1B live approval wrapper");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&wrapper)
            .expect("inspect S1B live approval wrapper")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&wrapper, permissions)
            .expect("make S1B live approval wrapper executable");
    }

    let original_path = std::env::var_os("PATH").expect("PATH required for live Codex harness");
    let mut search_paths = vec![wrapper_root.clone()];
    search_paths.extend(std::env::split_paths(&original_path));
    let wrapped_path = std::env::join_paths(search_paths).expect("join live Codex harness PATH");
    std::env::set_var("PATH", wrapped_path);
    S1bLiveApprovalHarnessGuard {
        original_path,
        wrapper_root,
    }
}

#[derive(Clone, Debug)]
enum MockOneShotOutcome {
    Turn {
        thread_id: String,
        content: String,
    },
    TurnWithDiagnostics {
        thread_id: String,
        content: String,
        recoverable_error_details: Vec<String>,
    },
    ResumeExitWithoutThreadStarted {
        exit_code: i32,
        stderr: String,
    },
    WatchdogSilence,
    WatchdogAfterThreadStarted {
        thread_id: String,
    },
    CleanupFailed,
    Protocol(String),
}

type MockAfterBind = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

struct MockOneShotRunner {
    outcomes: Mutex<VecDeque<MockOneShotOutcome>>,
    plans: Arc<Mutex<Vec<SupervisorResidentOneShotPlan>>>,
    pids: AtomicU32,
    after_bind: Option<MockAfterBind>,
    hook_calls: Arc<AtomicUsize>,
}

impl MockOneShotRunner {
    fn new(outcomes: Vec<MockOneShotOutcome>) -> Self {
        Self {
            outcomes: Mutex::new(outcomes.into_iter().collect()),
            plans: Arc::new(Mutex::new(Vec::new())),
            pids: AtomicU32::new(7000),
            after_bind: None,
            hook_calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn with_after_bind(mut self, after_bind: MockAfterBind) -> Self {
        self.after_bind = Some(after_bind);
        self
    }
}

impl SupervisorResidentOneShotRunner for MockOneShotRunner {
    fn run(
        &self,
        plan: &SupervisorResidentOneShotPlan,
        _home: &SupervisorResidentHome,
        on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
        self.plans
            .lock()
            .expect("record mock one-shot plan")
            .push(plan.clone());
        let outcome = self
            .outcomes
            .lock()
            .expect("take mock one-shot outcome")
            .pop_front()
            .expect("mock one-shot outcome configured");
        match outcome {
            MockOneShotOutcome::Turn { thread_id, content } => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                on_thread_started(&thread_id, pid)
                    .map_err(SupervisorResidentOneShotFailure::Protocol)?;
                if let Some(after_bind) = &self.after_bind {
                    after_bind().map_err(SupervisorResidentOneShotFailure::Protocol)?;
                    self.hook_calls.fetch_add(1, Ordering::SeqCst);
                }
                Ok(SupervisorResidentTurn {
                    thread_id,
                    content,
                    recoverable_error_details: vec![],
                })
            }
            MockOneShotOutcome::TurnWithDiagnostics {
                thread_id,
                content,
                recoverable_error_details,
            } => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                on_thread_started(&thread_id, pid)
                    .map_err(SupervisorResidentOneShotFailure::Protocol)?;
                if let Some(after_bind) = &self.after_bind {
                    after_bind().map_err(SupervisorResidentOneShotFailure::Protocol)?;
                    self.hook_calls.fetch_add(1, Ordering::SeqCst);
                }
                Ok(SupervisorResidentTurn {
                    thread_id,
                    content,
                    recoverable_error_details,
                })
            }
            MockOneShotOutcome::ResumeExitWithoutThreadStarted { exit_code, stderr } => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                // The real runner durably re-binds the expected thread before
                // it receives stdout.  Reproduce that pre-registration here,
                // but deliberately do not synthesize a stdout thread.started.
                if let Some(expected_thread_id) = plan.expected_thread_id.as_deref() {
                    on_thread_started(expected_thread_id, pid)
                        .map_err(SupervisorResidentOneShotFailure::Protocol)?;
                }
                Err(classify_resume_exit_without_thread_started(
                    plan,
                    exit_code,
                    false,
                    PathBuf::from("fixture-step-0.stderr.txt"),
                    stderr,
                )
                .expect("fixture is a nonzero resume without stdout thread.started"))
            }
            MockOneShotOutcome::WatchdogSilence => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                Err(SupervisorResidentOneShotFailure::WatchdogSilence)
            }
            MockOneShotOutcome::WatchdogAfterThreadStarted { thread_id } => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                on_thread_started(&thread_id, pid)
                    .map_err(SupervisorResidentOneShotFailure::Protocol)?;
                if let Some(after_bind) = &self.after_bind {
                    after_bind().map_err(SupervisorResidentOneShotFailure::Protocol)?;
                    self.hook_calls.fetch_add(1, Ordering::SeqCst);
                }
                Err(SupervisorResidentOneShotFailure::WatchdogSilence)
            }
            MockOneShotOutcome::CleanupFailed => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                Err(SupervisorResidentOneShotFailure::CleanupFailed(
                    "fixture process group remains live".to_string(),
                ))
            }
            MockOneShotOutcome::Protocol(detail) => {
                let pid = self.pids.fetch_add(1, Ordering::SeqCst);
                on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
                Err(SupervisorResidentOneShotFailure::Protocol(detail))
            }
        }
    }
}

struct FirstTurnBindingRaceRunner {
    config: McpServerConfig,
}

impl SupervisorResidentOneShotRunner for FirstTurnBindingRaceRunner {
    fn run(
        &self,
        _plan: &SupervisorResidentOneShotPlan,
        _home: &SupervisorResidentHome,
        on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
        let pid = 7441;
        on_turn_prepared(pid).map_err(SupervisorResidentOneShotFailure::Protocol)?;
        let config = self.config.clone();
        let (attempt_started, attempt_receiver) = std::sync::mpsc::channel();
        let tool_call = std::thread::spawn(move || -> Result<(), String> {
            attempt_started
                .send(())
                .map_err(|_| "first-turn race readiness channel closed".to_string())?;
            let response = supervisor_orchestrator::call_tool(
                &config,
                json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
            )?;
            let receipt: Value = serde_json::from_str(
                response["content"][0]["text"]
                    .as_str()
                    .ok_or_else(|| "race proposal receipt missing".to_string())?,
            )
            .map_err(|error| format!("race proposal receipt malformed:{error}"))?;
            if receipt["status"] != "proposal_created_pending_user_confirmation" {
                return Err("race proposal receipt was not a pending confirmation card".to_string());
            }
            Ok(())
        });
        attempt_receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(|error| {
                SupervisorResidentOneShotFailure::Protocol(format!(
                    "first-turn race tool did not start:{error}"
                ))
            })?;
        // The tool call is now in flight while the durable record remains
        // `resident_turn_starting`.  Its only valid outcome is to wait for this
        // callback, never to borrow a previous thread binding.
        std::thread::sleep(Duration::from_millis(30));
        on_thread_started("thread-s1b-race", pid)
            .map_err(SupervisorResidentOneShotFailure::Protocol)?;
        tool_call
            .join()
            .map_err(|_| {
                SupervisorResidentOneShotFailure::Protocol(
                    "first-turn race tool thread panicked".to_string(),
                )
            })?
            .map_err(SupervisorResidentOneShotFailure::Protocol)?;
        Ok(SupervisorResidentTurn {
            thread_id: "thread-s1b-race".to_string(),
            content: "首轮工具已在绑定完成后落卡。".to_string(),
            recoverable_error_details: vec![],
        })
    }
}

fn resident_fixture(label: &str) -> (PathBuf, String, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "s1b-supervisor-resident-{label}-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir_all(&root).expect("create resident fixture root");
    let state_path = root.join("workflow-state.v0.json");
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    crate::bootstrap_project_workflow_at(
        &state_path,
        &crate::ProjectRecord {
            project_root: project_root.to_string(),
            name: "S1B one-shot resident fixture".to_string(),
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
        },
    )
    .expect("bootstrap fixed resident workflow");
    (state_path, crate::default_workflow_id(project_root), root)
}

fn resident_fixture_manager(root: &Path) -> SupervisorResidentHomeManager {
    let auth_source = root.join("fixture-auth.json");
    fs::write(&auth_source, "{\"token\":\"fixture-only\"}").expect("write fake auth source");
    SupervisorResidentHomeManager {
        base: root.join("private-resident-home"),
        auth_source,
    }
}

fn resident_real_config_consuming_codex_script(root: &Path) -> PathBuf {
    let script_root = root.join("fake-codex");
    fs::create_dir_all(&script_root).expect("create fake codex script root");
    let script = script_root.join("codex");
    fs::write(
        &script,
        r#"#!/bin/sh
set -eu

last_message=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-last-message)
      last_message="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

test -n "$last_message"
test -n "${CODEX_HOME:-}"
cat >/dev/null
cp "$CODEX_HOME/config.toml" "${last_message}.h2-observed-config.toml"
printf '%s\n' '{"type":"thread.started","thread_id":"thread-h2-real-config"}'
printf '%s\n' '{"type":"item.completed","item":{"type":"agent_message","text":"fixture supervisor reply"}}'
printf '%s\n' '{"type":"turn.completed"}'
printf '%s' 'fixture supervisor reply' > "$last_message"
"#,
    )
    .expect("write fake codex script");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&script)
            .expect("inspect fake codex script")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&script, permissions).expect("make fake codex script executable");
    }
    script
}

fn assert_h2_resident_mcp_config(text: &str) {
    let config: toml::Value = toml::from_str(text).expect("parse resident MCP config");
    let servers = config
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .expect("resident MCP servers table");
    assert_eq!(
        servers.len(),
        1,
        "only the supervisor MCP server is present"
    );
    let server = servers
        .get("supervisor_orchestrator")
        .and_then(toml::Value::as_table)
        .expect("resident supervisor MCP server");
    let enabled_tools = server
        .get("enabled_tools")
        .and_then(toml::Value::as_array)
        .expect("resident enabled_tools list")
        .iter()
        .map(|value| value.as_str().expect("enabled tool name"))
        .collect::<Vec<_>>();
    assert_eq!(enabled_tools, vec!["submit_proposal"]);
    let tools = server
        .get("tools")
        .and_then(toml::Value::as_table)
        .expect("resident tool configuration");
    assert_eq!(
        tools.len(),
        1,
        "no other supervisor tool may be preapproved"
    );
    assert_eq!(
        tools
            .get("submit_proposal")
            .and_then(toml::Value::as_table)
            .and_then(|tool| tool.get("approval_mode"))
            .and_then(toml::Value::as_str),
        Some("approve"),
    );
    for forbidden in [
        "default_tools_approval_mode",
        "approval_policy",
        "approvals_reviewer",
        "sandbox",
        "full-auto",
        "dangerously-bypass",
        "*",
    ] {
        assert!(
            !text.contains(forbidden),
            "resident private config must not broaden authority with {forbidden}"
        );
    }
}

fn write_owner_only_test_config(path: &Path, text: &str) {
    fs::write(path, text).expect("write test resident config");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)
            .expect("inspect test resident config")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions).expect("make test resident config owner-only");
    }
}

fn resident_request(workflow_id: &str, message: &str) -> SubmitSupervisorResidentAnswerRequest {
    SubmitSupervisorResidentAnswerRequest {
        project_id: crate::project_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        workflow_id: workflow_id.to_string(),
        message_text: message.to_string(),
        client_request_id: None,
    }
}

fn resident_fixture_config(state_path: &Path) -> McpServerConfig {
    resident_supervisor_config(
        state_path,
        &resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
    )
}

fn resident_supervisor_audit_events(state_path: &Path) -> Vec<Value> {
    let sidecar = crate::utils::store_paths::sidecar_path(
        state_path,
        "supervisor-orchestrator.v1.json",
        "主管编排",
    )
    .expect("supervisor audit sidecar path");
    let value: Value =
        serde_json::from_slice(&fs::read(sidecar).expect("read supervisor audit sidecar"))
            .expect("parse supervisor audit sidecar");
    value["audit_events"]
        .as_array()
        .expect("supervisor audit events")
        .clone()
}

fn fixture_cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn valid_submit_proposal_arguments() -> Value {
    json!({
        "user_goal": "为固定测试项目整理并确认 S1B 实施方案。",
        "goal_summary": "一次一发主管会话仍只落待确认方案卡。",
        "scope_note": "用户确认前不启动工作流。",
        "reasoning": ["同回合 thread.started 已先持久化绑定。"],
        "risks": [{
            "severity": "warning",
            "summary": "未经确认不得执行。",
            "mitigation": "保持 PendingUserConfirmation，等待既有确认路径。"
        }],
        "must_stop_points": ["未获用户确认不得启动工作流。"],
        "next_steps": ["用户在右侧方案卡确认后再走既有批准路径。"],
        "worker_acceptance_criteria": ["worker 回交可验证结果。"],
        "control_core_acceptance_criteria": ["控制面只从已批准卡启动。"],
        "supervisor_acceptance_criteria": ["主管依据审计事实汇报。"],
        "execution_scope": {
            "requires_write": true,
            "write_roots": [crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT],
            "target_files": ["src/s1b-proof.txt"],
            "tools": ["shell(读写·写域由沙箱锁定)"],
            "checks": ["cargo test --lib"]
        },
        "suggest_workflow": true,
        "tasks": [{
            "title": "补齐 S1B 证明",
            "task_goal": "在固定测试项目内补齐 S1B 验收所需的最小证明。",
            "target_role": "codex-dev",
            "depends_on": [],
            "acceptance_criteria": ["证明可由指定检查回读。"],
            "report_format": ["说明改动和验证结果。"]
        }]
    })
}

#[test]
fn s1b_h1_live_wrapper_preapproves_only_submit_proposal() {
    assert!(S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("--strict-config"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("\nexec \"$SYN_S1B_LIVE_REAL_CODEX\""));
    assert!(
        S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains(
            "resume \\\n    -c 'mcp_servers.supervisor_orchestrator.enabled_tools=[\"submit_proposal\"]'"
        ),
        "resume must receive its own subcommand-level MCP override"
    );
    assert_eq!(
        S1B_LIVE_CODEX_APPROVAL_WRAPPER
            .matches("enabled_tools=[\"submit_proposal\"]")
            .count(),
        2,
        "initial and resume invocations must each receive the same narrow allowlist"
    );
    assert!(
        S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("tools.submit_proposal.approval_mode=\"approve\"")
    );
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("default_tools_approval_mode"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("approval_policy"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("approvals_reviewer"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("--sandbox"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("full-auto"));
    assert!(!S1B_LIVE_CODEX_APPROVAL_WRAPPER.contains("dangerously-bypass"));
}

#[test]
fn s1b_public_resident_user_message_path_requires_the_canonical_answer_command() {
    let error = consult_supervisor_resident_turn(
        Path::new("/unused-for-user-message-guard"),
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow-unused",
        "this must not bypass canonical user-message persistence",
        "user_message",
    )
    .expect_err("the public resident API must not inject user messages directly");
    assert_eq!(
        error,
        "supervisor_resident_user_message_requires_answer_command"
    );
}

fn finalize_fixture_stdout_events(
    lines: &[&str],
    status_success: bool,
    exit_code: i32,
    prebound_thread: Option<&str>,
) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
    let mut bound_thread = prebound_thread.map(str::to_string);
    let mut assistant_message = None;
    let mut turn_completed = false;
    let mut turn_failed = None;
    let mut recoverable_error_details = Vec::new();
    let mut thread_started_event_seen = false;
    let mut on_thread_started = |_thread_id: &str, _pid: u32| Ok(());
    for line in lines {
        apply_supervisor_resident_json_event(
            line,
            7171,
            &mut bound_thread,
            &mut assistant_message,
            &mut turn_completed,
            &mut turn_failed,
            &mut recoverable_error_details,
            &mut thread_started_event_seen,
            &mut on_thread_started,
        )?;
    }
    finalize_supervisor_resident_turn(
        status_success,
        exit_code,
        turn_failed,
        turn_completed,
        thread_started_event_seen,
        bound_thread,
        None,
        assistant_message,
        recoverable_error_details,
    )
}

#[test]
fn s1b_h2_transient_error_then_completed_turn_keeps_the_conversation() {
    let turn = finalize_fixture_stdout_events(
        &[
            r#"{"type":"thread.started","thread_id":"thread-h2-recoverable"}"#,
            r#"{"type":"error","message":"user cancelled MCP tool call"}"#,
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"主管仍然完成了本轮答复。"}}"#,
            r#"{"type":"turn.completed"}"#,
        ],
        true,
        0,
        None,
    )
    .expect("a recovered error must not erase a completed conversation");
    assert_eq!(turn.thread_id, "thread-h2-recoverable");
    assert_eq!(turn.content, "主管仍然完成了本轮答复。");
    assert_eq!(
        turn.recoverable_error_details,
        vec!["user cancelled MCP tool call"]
    );
}

#[test]
fn s1b_h2_turn_failed_remains_hard_failure_after_a_message() {
    let error = finalize_fixture_stdout_events(
        &[
            r#"{"type":"thread.started","thread_id":"thread-h2-failed"}"#,
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"这条消息不能替代失败。"}}"#,
            r#"{"type":"turn.failed","message":"fixture hard failure"}"#,
            r#"{"type":"turn.completed"}"#,
        ],
        true,
        0,
        None,
    )
    .expect_err("turn.failed must remain terminal even after an agent message");
    assert!(matches!(
        error,
        SupervisorResidentOneShotFailure::Protocol(detail)
            if detail == "supervisor_resident_turn_failed:fixture hard failure"
    ));
}

#[test]
fn s1b_h2_error_without_completion_remains_hard_failure() {
    let error = finalize_fixture_stdout_events(
        &[
            r#"{"type":"thread.started","thread_id":"thread-h2-no-completion"}"#,
            r#"{"type":"error","message":"user cancelled MCP tool call"}"#,
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"没有 completion 的消息不能算完成。"}}"#,
        ],
        true,
        0,
        None,
    )
    .expect_err("error without turn.completed must not become success");
    assert!(matches!(
        error,
        SupervisorResidentOneShotFailure::Protocol(detail)
            if detail == "supervisor_resident_turn_completed_event_missing"
    ));
}

#[test]
fn s1b_h2_resume_prebinding_cannot_replace_a_real_stdout_thread_started() {
    let error = finalize_fixture_stdout_events(
        &[
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"预绑定不能冒充事件。"}}"#,
            r#"{"type":"turn.completed"}"#,
        ],
        true,
        0,
        Some("thread-h2-preregistered"),
    )
    .expect_err("a pre-registered resume thread is not stdout completion evidence");
    assert!(matches!(
        error,
        SupervisorResidentOneShotFailure::Protocol(detail)
            if detail == "supervisor_resident_thread_started_event_missing"
    ));
}

#[test]
fn s1b_three_oneshot_turns_resume_same_thread_and_keep_project_private_home() {
    let (state_path, workflow_id, root) = resident_fixture("three-turns");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-one".to_string(),
            content: "记住 ALPHA。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-one".to_string(),
            content: "ALPHA 仍在上下文。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-one".to_string(),
            content: "第三轮仍是同一 thread。".to_string(),
        },
    ]);

    let first = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请记住 ALPHA。"),
        &config,
    )
    .expect("first one-shot turn");
    let second = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请引用刚才的标记。"),
        &config,
    )
    .expect("second one-shot resume");
    let third = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请再次确认上下文。"),
        &config,
    )
    .expect("third one-shot resume");

    assert_eq!(first.thread_id.as_deref(), Some("thread-s1b-one"));
    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(third.thread_id, first.thread_id);
    let plans = runner.plans.lock().expect("mock plans");
    assert_eq!(
        plans.len(),
        3,
        "each user message spawns one finite process"
    );
    assert!(!plans[0].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(plans[1].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(plans[2].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(plans.iter().all(|plan| {
        plan.command_plan
            .argv
            .iter()
            .filter(|arg| arg.as_str() == "--strict-config")
            .count()
            == 1
    }));
    assert!(plans
        .iter()
        .all(|plan| !plan.command_plan.argv.iter().any(|arg| arg == "mcp-server")));
    let active = manager.active_path();
    assert!(
        active.is_dir(),
        "normal one-shot turns must not clean the home"
    );
    assert!(fs::symlink_metadata(active.join(SUPERVISOR_TEMP_HOME_AUTH))
        .expect("auth link metadata")
        .file_type()
        .is_symlink());
    let active_config_text = fs::read_to_string(active.join(SUPERVISOR_TEMP_HOME_CONFIG))
        .expect("read active private MCP config");
    let active_config: toml::Value =
        toml::from_str(&active_config_text).expect("parse active private MCP config");
    let mcp_servers = active_config
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .expect("private MCP servers table");
    assert_eq!(
        mcp_servers.len(),
        1,
        "private home has one MCP allowlist entry"
    );
    let supervisor_server = mcp_servers
        .get("supervisor_orchestrator")
        .and_then(toml::Value::as_table)
        .expect("private supervisor MCP server");
    let server_args = supervisor_server
        .get("args")
        .and_then(toml::Value::as_array)
        .expect("private supervisor MCP args")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert!(server_args
        .windows(2)
        .any(|pair| { pair[0] == "--role" && pair[1] == "supervisor_orchestrator" }));
    let session = supervisor_orchestrator::load_resident_session(&config)
        .expect("load session")
        .expect("session exists");
    assert_eq!(session.thread_id, "thread-s1b-one");
    assert_eq!(session.host_pid, 0, "completed turn leaves no resident pid");
    assert_eq!(session.launch_status, "resident_exited");
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config() {
    let (state_path, _workflow_id, root) = resident_fixture("h2-real-config-consumption");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let fake_codex = resident_real_config_consuming_codex_script(&root);
    let workbench_executable = std::env::current_exe().expect("locate test workbench executable");
    let run_id = resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT);
    let mut initial_command_plan = build_supervisor_resident_command_plan(
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &state_path,
        &run_id,
        1,
        &workbench_executable,
        None,
    )
    .expect("build initial resident command plan");
    initial_command_plan.program = fake_codex.display().to_string();
    let home = manager
        .ensure_active(&initial_command_plan, &config, 1)
        .expect("create resident private home");
    let initial_plan = SupervisorResidentOneShotPlan {
        command_plan: initial_command_plan,
        prompt: "fixture initial prompt".to_string(),
        expected_thread_id: None,
        workflow_state_path: state_path.clone(),
        run_id: run_id.clone(),
    };
    let mut prepared = |_pid: u32| Ok(());
    let mut bound = |_thread_id: &str, _pid: u32| Ok(());
    let initial =
        run_real_supervisor_resident_oneshot(&initial_plan, &home, &mut prepared, &mut bound)
            .expect("initial real one-shot fixture");
    assert_eq!(initial.thread_id, "thread-h2-real-config");

    let mut resume_command_plan = build_supervisor_resident_command_plan(
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &state_path,
        &run_id,
        1,
        &workbench_executable,
        Some("thread-h2-real-config"),
    )
    .expect("build resume resident command plan");
    resume_command_plan.program = fake_codex.display().to_string();
    let resumed_home = manager
        .ensure_active(&resume_command_plan, &config, 1)
        .expect("reuse resident private home for resume");
    let resume_plan = SupervisorResidentOneShotPlan {
        command_plan: resume_command_plan,
        prompt: "fixture resume prompt".to_string(),
        expected_thread_id: Some("thread-h2-real-config".to_string()),
        workflow_state_path: state_path.clone(),
        run_id,
    };
    let mut resume_prepared = |_pid: u32| Ok(());
    let mut resume_bound = |_thread_id: &str, _pid: u32| Ok(());
    let resumed = run_real_supervisor_resident_oneshot(
        &resume_plan,
        &resumed_home,
        &mut resume_prepared,
        &mut resume_bound,
    )
    .expect("resume real one-shot fixture");
    assert_eq!(resumed.thread_id, "thread-h2-real-config");

    let initial_observed = fs::read_to_string(format!(
        "{}.h2-observed-config.toml",
        initial_plan.command_plan.last_message_path.display()
    ))
    .expect("initial fake Codex observed CODEX_HOME config");
    let resume_observed = fs::read_to_string(format!(
        "{}.h2-observed-config.toml",
        resume_plan.command_plan.last_message_path.display()
    ))
    .expect("resume fake Codex observed CODEX_HOME config");
    assert_h2_resident_mcp_config(&initial_observed);
    assert_h2_resident_mcp_config(&resume_observed);
    assert_eq!(initial_observed, resume_observed);
    assert_eq!(
        initial_observed,
        supervisor_resident_mcp_config_toml(&initial_plan.command_plan)
            .expect("render exact resident initial config"),
        "the child must read the config written to its actual CODEX_HOME"
    );
    assert!(
        !supervisor_mcp_config_toml(&initial_plan.command_plan)
            .expect("render legacy shared config")
            .contains("enabled_tools"),
        "the H2 preapproval remains resident-only and does not broaden the shared pilot config"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_only_an_exact_legacy_private_config_migrates_to_the_single_tool_config() {
    let (state_path, _workflow_id, root) = resident_fixture("h2-config-migration");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let workbench_executable = std::env::current_exe().expect("locate test workbench executable");
    let plan = build_supervisor_resident_command_plan(
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &state_path,
        &resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        1,
        &workbench_executable,
        None,
    )
    .expect("build resident command plan");
    let home = manager
        .ensure_active(&plan, &config, 1)
        .expect("create private resident home");
    let config_path = home.root().join(SUPERVISOR_TEMP_HOME_CONFIG);
    let legacy = supervisor_mcp_config_toml(&plan).expect("render exact legacy config");
    write_owner_only_test_config(&config_path, &legacy);

    manager
        .ensure_active(&plan, &config, 1)
        .expect("only the exact legacy config may receive the H2 migration");
    let expected = supervisor_resident_mcp_config_toml(&plan).expect("render H2 resident config");
    assert_eq!(
        fs::read_to_string(&config_path).expect("read migrated resident config"),
        expected
    );
    assert_h2_resident_mcp_config(&expected);

    let drifted = format!("{expected}\n[h2_unrecognized_drift]\nvalue = true\n");
    write_owner_only_test_config(&config_path, &drifted);
    let error = manager
        .ensure_active(&plan, &config, 1)
        .expect_err("unknown config drift must fail closed instead of being rewritten");
    assert!(error.contains("MCP 白名单与当前受控计划不一致"));
    assert_eq!(
        fs::read_to_string(&config_path).expect("read rejected drift bytes"),
        drifted,
        "fail-closed validation must not overwrite an unrecognized private config"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_first_thread_started_is_bound_before_real_submit_proposal_card_write() {
    let (state_path, workflow_id, root) = resident_fixture("first-tool");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let hook_config = config.clone();
    let hook = Arc::new(move || {
        let response = supervisor_orchestrator::call_tool(
            &hook_config,
            json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
        )?;
        let receipt: Value = serde_json::from_str(
            response["content"][0]["text"]
                .as_str()
                .ok_or_else(|| "mock proposal receipt missing".to_string())?,
        )
        .map_err(|error| format!("mock proposal receipt malformed:{error}"))?;
        if receipt["status"] != "proposal_created_pending_user_confirmation" {
            return Err("mock proposal receipt was not a pending confirmation card".to_string());
        }
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-s1b-card".to_string(),
        content: "方案已按工具落到右侧待确认卡。".to_string(),
    }])
    .with_after_bind(hook);

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请给出终版方案并落一张待确认卡。"),
        &config,
    )
    .expect("first turn proposal tool is allowed after synchronous bind");
    assert_eq!(outcome.thread_id.as_deref(), Some("thread-s1b-card"));
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 1);
    let store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("proposal store");
    assert_eq!(store.proposals.len(), 1);
    assert_eq!(
        store.proposals[0].status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    let workflow: Value = serde_json::from_slice(&fs::read(&state_path).expect("workflow state"))
        .expect("workflow JSON");
    assert_eq!(workflow["workflow_chain_runs"], chain_before);
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_same_resident_message_replays_reuse_one_card_but_a_new_message_can_create_another() {
    let (state_path, workflow_id, root) = resident_fixture("h2-proposal-idempotency");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let receipt_ids = Arc::new(Mutex::new(Vec::new()));
    let hook_config = config.clone();
    let hook_receipt_ids = Arc::clone(&receipt_ids);
    let hook = Arc::new(move || {
        for _ in 0..2 {
            let response = supervisor_orchestrator::call_tool(
                &hook_config,
                json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
            )?;
            let receipt: Value = serde_json::from_str(
                response["content"][0]["text"]
                    .as_str()
                    .ok_or_else(|| "idempotency proposal receipt missing".to_string())?,
            )
            .map_err(|error| format!("idempotency proposal receipt malformed:{error}"))?;
            let proposal_id = receipt["proposal_id"]
                .as_str()
                .ok_or_else(|| "idempotency proposal id missing".to_string())?;
            hook_receipt_ids
                .lock()
                .expect("record proposal receipt id")
                .push(proposal_id.to_string());
        }
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::WatchdogAfterThreadStarted {
            thread_id: "thread-h2-idempotency".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-h2-idempotency".to_string(),
            content: "技术重试后的同一回合已完成。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-h2-idempotency".to_string(),
            content: "新的明确出方案请求已完成。".to_string(),
        },
    ])
    .with_after_bind(hook);

    let first = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请出方案，并允许技术重试保持同一张待确认卡。"),
        &config,
    )
    .expect("technical retry should complete the original user message");
    let second = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "我再次明确说一次：请出方案。"),
        &config,
    )
    .expect("a later explicit user message is a new proposal intent");
    assert_eq!(first.status, "message_sent_proposal_materialized");
    assert_eq!(second.status, "message_sent_proposal_materialized");
    assert_eq!(first.thread_id, second.thread_id);
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 3);

    let receipt_ids = receipt_ids.lock().expect("read proposal receipt ids");
    assert_eq!(
        receipt_ids.len(),
        6,
        "each fixture turn makes two tool calls"
    );
    assert!(
        receipt_ids[..4]
            .iter()
            .all(|proposal_id| proposal_id == &receipt_ids[0]),
        "same canonical message, same-turn double call, and watchdog retry must reuse one card"
    );
    assert!(
        receipt_ids[4..]
            .iter()
            .all(|proposal_id| proposal_id == &receipt_ids[4])
            && receipt_ids[4] != receipt_ids[0],
        "a later explicitly submitted canonical message may create its own card"
    );
    let store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("proposal store after technical replay");
    assert_eq!(store.proposals.len(), 2);
    assert!(store.proposals.iter().all(|proposal| {
        proposal.status == crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    }));
    assert_eq!(
        store
            .audit_events
            .iter()
            .filter(|event| event.event_type == "project_consultation_proposal_created")
            .count(),
        2,
        "technical replays do not append a second proposal audit or DB/JSON projection"
    );
    let workflow_after: Value =
        serde_json::from_slice(&fs::read(&state_path).expect("workflow state after replay"))
            .expect("workflow JSON after replay");
    assert_eq!(workflow_after["workflow_chain_runs"], chain_before);
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_transport_response_loss_reuses_the_canonical_turn_and_pending_card() {
    let (state_path, workflow_id, root) = resident_fixture("h2-transport-response-loss");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let hook_config = config.clone();
    let hook = Arc::new(move || {
        supervisor_orchestrator::call_tool(
            &hook_config,
            json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
        )?;
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-h2-transport-response-loss".to_string(),
        content: "同一条用户消息已经落为待确认方案卡。".to_string(),
    }])
    .with_after_bind(hook);
    let mut request = resident_request(&workflow_id, "请形成终版方案并只落一张待确认卡。");
    let client_request_id = "f0a5e8c1-9b42-4d67-a321-5c7e8f901234";
    request.client_request_id = Some(client_request_id.to_string());

    let first = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &request,
        &config,
    )
    .expect("the first turn completes before its transport response is lost");
    let replay = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &request,
        &config,
    )
    .expect("the same client request id must reconcile rather than start another turn");

    assert_eq!(first.status, "message_sent_proposal_materialized");
    assert_eq!(replay.status, "message_sent_proposal_materialized");
    assert_eq!(replay.thread_id, first.thread_id);
    assert_eq!(replay.supervisor_reply, first.supervisor_reply);
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        runner.plans.lock().expect("mock plans").len(),
        1,
        "a response-loss retry must not execute a second supervisor turn"
    );
    let canonical_events = resident_canonical_events(&state_path).expect("canonical events");
    assert_eq!(
        canonical_events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT
                    && event["client_request_id"] == client_request_id
            })
            .count(),
        1,
        "the UI retry must reuse one server-generated canonical user message"
    );
    let store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("proposal store after response-loss replay");
    assert_eq!(store.proposals.len(), 1);
    assert_eq!(
        store.proposals[0].status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    let workflow_after: Value =
        serde_json::from_slice(&fs::read(&state_path).expect("workflow after replay"))
            .expect("workflow JSON after replay");
    assert_eq!(workflow_after["workflow_chain_runs"], chain_before);
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_recoverable_non_tool_diagnostic_keeps_canonical_conversation_without_a_card() {
    let (state_path, workflow_id, root) = resident_fixture("h2-recoverable-non-tool-error");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let raw_diagnostic = "transient websocket reset";
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::TurnWithDiagnostics {
        thread_id: "thread-h2-recoverable-non-tool-error".to_string(),
        content: "我已经完成这次自然对话说明。".to_string(),
        recoverable_error_details: vec![raw_diagnostic.to_string()],
    }]);

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请先自然说明当前情况。"),
        &config,
    )
    .expect("a recovered non-tool diagnostic must not fail the conversation");
    assert_eq!(outcome.status, "message_sent");
    assert_eq!(
        outcome.message,
        "用户消息已同 thread 注入；主管回复已写入项目对话。"
    );
    assert_eq!(
        outcome.supervisor_reply.as_deref(),
        Some("我已经完成这次自然对话说明。")
    );
    assert!(!outcome.message.contains(raw_diagnostic));
    assert!(
        crate::project_consultation_proposal_store::load_store(
            &state_path,
            crate::unix_timestamp_ms(),
        )
        .expect("proposal store after recovered tool diagnostic")
        .proposals
        .is_empty(),
        "diagnostic-only failure must not fabricate a Pending card"
    );
    let canonical_events = crate::read_workflow_state_value(&state_path)
        .expect("canonical workflow events")
        .get("audit_events")
        .and_then(Value::as_array)
        .expect("canonical audit events")
        .clone();
    assert!(canonical_events
        .iter()
        .any(|event| { event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT }));
    assert!(canonical_events.iter().any(|event| {
        event["event_type"] == SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT
            && event["message_text"] == "我已经完成这次自然对话说明。"
    }));
    let audit_events = resident_supervisor_audit_events(&state_path);
    assert!(audit_events.iter().any(|event| {
        event["event_type"] == "supervisor_resident_recoverable_diagnostic"
            && event["parameter_summary"]
                .as_str()
                .is_some_and(|detail| detail.contains(raw_diagnostic))
    }));
    let pilot_read_model = supervisor_orchestrator::load_pilot_read_model(&config)
        .expect("ordinary supervisor read model");
    assert!(pilot_read_model
        .audit_events
        .iter()
        .all(|event| !event.result_summary.contains(raw_diagnostic)));
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_private_tool_arguments_do_not_enter_the_ordinary_supervisor_read_model() {
    let (state_path, workflow_id, root) = resident_fixture("h2-private-tool-arguments");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let private_marker = "H2_PRIVATE_MCP_ARGUMENT_DO_NOT_PROJECT";
    let hook_config = config.clone();
    let hook = Arc::new(move || {
        let error = supervisor_orchestrator::call_tool(
            &hook_config,
            json!({
                "name": "submit_proposal",
                "arguments": {"unexpected_h2_private_marker": private_marker}
            }),
        )
        .expect_err("bad tool arguments must not create a card");
        assert_eq!(error, "方案卡没有生成；详细诊断已保留。");
        assert!(!error.contains(private_marker));
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-h2-private-tool-arguments".to_string(),
        content: "方案参数没有通过；我会用人话继续说明。".to_string(),
    }])
    .with_after_bind(hook);

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请出方案；若参数不对也保留主管答复。"),
        &config,
    )
    .expect("tool handler rejection must not swallow a completed conversation");
    assert_eq!(outcome.status, "message_sent_proposal_tool_failed");
    assert!(!outcome.message.contains(private_marker));
    assert!(crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("proposal store after bad private arguments")
    .proposals
    .is_empty());
    let private_audits = resident_supervisor_audit_events(&state_path);
    assert!(private_audits.iter().any(|event| {
        event["tool"] == "submit_proposal"
            && event["parameter_summary"]
                .as_str()
                .is_some_and(|detail| detail.contains(private_marker))
    }));
    let pilot_read_model = supervisor_orchestrator::load_pilot_read_model(&config)
        .expect("ordinary supervisor read model");
    assert!(pilot_read_model.audit_events.iter().all(|event| {
        !event.result_summary.contains(private_marker)
            && !event
                .result_summary
                .contains("unexpected_h2_private_marker")
    }));
    fixture_cleanup(&root);
}

#[test]
fn s1b_first_turn_tool_waits_for_durable_binding_instead_of_racing_a_stale_thread() {
    let (state_path, workflow_id, root) = resident_fixture("first-tool-race");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = FirstTurnBindingRaceRunner {
        config: config.clone(),
    };
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请在首轮形成完整方案并落卡。"),
        &config,
    )
    .expect("first-turn tool waits for the durable binding");
    assert_eq!(outcome.thread_id.as_deref(), Some("thread-s1b-race"));
    let store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("proposal store after race proof");
    assert_eq!(store.proposals.len(), 1);
    assert_eq!(
        store.proposals[0].status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_watchdog_retries_once_and_second_silence_returns_human_message() {
    let (state_path, workflow_id, root) = resident_fixture("watchdog");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::WatchdogSilence,
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-watchdog".to_string(),
            content: "第二次尝试成功。".to_string(),
        },
    ]);
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请重试这一句。"),
        &config,
    )
    .expect("first watchdog retry succeeds");
    assert_eq!(outcome.thread_id.as_deref(), Some("thread-s1b-watchdog"));
    assert_eq!(runner.plans.lock().expect("plans").len(), 2);
    assert!(
        manager.active_path().is_dir(),
        "watchdog must not delete private home"
    );

    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::WatchdogSilence,
        MockOneShotOutcome::WatchdogSilence,
    ]);
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "这次应在两次静默后停止。"),
        &config,
    )
    .expect("the canonical user message survives a supervisor retry failure");
    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(
        outcome.message,
        "消息已送到主管，但主管这次没回上来——可以再发一次。"
    );
    assert_eq!(runner.plans.lock().expect("plans").len(), 2);
    fixture_cleanup(&root);
}

#[test]
fn s1b_watchdog_after_initial_thread_binding_retries_with_same_thread_resume() {
    let (state_path, workflow_id, root) = resident_fixture("watchdog-initial-resume");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::WatchdogAfterThreadStarted {
            thread_id: "thread-s1b-initial-watchdog".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-initial-watchdog".to_string(),
            content: "初始 thread 的一次重试已续接。".to_string(),
        },
    ]);
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "首轮静默后也必须续接同一 thread。"),
        &config,
    )
    .expect("watchdog retry resumes the first bound thread");
    assert_eq!(
        outcome.thread_id.as_deref(),
        Some("thread-s1b-initial-watchdog")
    );
    let plans = runner.plans.lock().expect("recorded plans");
    assert_eq!(plans.len(), 2);
    assert!(
        !plans[0].command_plan.argv.iter().any(|arg| arg == "resume"),
        "the first attempt is the initial exec"
    );
    assert!(
        plans[1].command_plan.argv.iter().any(|arg| arg == "resume")
            && plans[1].expected_thread_id.as_deref() == Some("thread-s1b-initial-watchdog"),
        "the retry must resume the thread already bound before the first watchdog"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_cleanup_failure_keeps_pid_visible_until_process_group_reconciliation() {
    let (state_path, workflow_id, root) = resident_fixture("cleanup-pending");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::CleanupFailed]);
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "模拟无法确认进程组清理。"),
        &config,
    )
    .expect("the canonical user message survives a cleanup failure");
    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(
        outcome.message,
        "消息已送到主管，但主管这次没回上来——可以再发一次。"
    );
    assert!(!outcome
        .message
        .contains("fixture process group remains live"));
    let pending = supervisor_orchestrator::load_resident_turn_for_reconciliation(&config)
        .expect("read cleanup-pending lifecycle")
        .expect("cleanup failure keeps a lifecycle record");
    assert_eq!(pending.host_pid, 7000);
    assert_eq!(pending.launch_status, "resident_turn_cleanup_failed");
    assert!(
        pending.thread_id.is_empty(),
        "first turn did not bind a thread"
    );

    let reaped = reap_supervisor_resident_stale_sessions_with(
        &state_path,
        &|pid| {
            assert_eq!(pid, 7000);
            Ok(false)
        },
        &config,
    )
    .expect("dead cleanup-pending group can be reconciled");
    assert!(reaped);
    let reconciled = supervisor_orchestrator::load_resident_turn_for_reconciliation(&config)
        .expect("read reconciled lifecycle")
        .expect("sidecar remains for audit after reconciliation");
    assert_eq!(reconciled.host_pid, 0);
    assert_eq!(reconciled.launch_status, "resident_exited");
    fixture_cleanup(&root);
}

#[test]
fn s1b_startup_reaps_dead_virgin_prepared_turn_without_thread_binding() {
    let (state_path, workflow_id, root) = resident_fixture("stale-virgin-prepared");
    let config = resident_fixture_config(&state_path);
    supervisor_orchestrator::record_resident_turn_prepared(
        &config,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        9393,
        1,
        None,
    )
    .expect("record a first exec that dies before thread.started");
    let prepared = supervisor_orchestrator::load_resident_turn_for_reconciliation(&config)
        .expect("read virgin prepared turn")
        .expect("prepared turn is visible without a thread");
    assert!(prepared.thread_id.is_empty());
    assert_eq!(prepared.launch_status, "resident_turn_starting");
    let reaped = reap_supervisor_resident_stale_sessions_with(
        &state_path,
        &|pid| {
            assert_eq!(pid, 9393);
            Ok(false)
        },
        &config,
    )
    .expect("startup reconciles a dead virgin prepared group");
    assert!(reaped);
    let reconciled = supervisor_orchestrator::load_resident_turn_for_reconciliation(&config)
        .expect("read reconciled virgin prepared turn")
        .expect("sidecar remains for audit after reconciliation");
    assert!(reconciled.thread_id.is_empty());
    assert_eq!(reconciled.host_pid, 0);
    assert_eq!(reconciled.launch_status, "resident_exited");
    fixture_cleanup(&root);
}

#[test]
fn s1b_invalid_resume_rotates_home_and_rebuilds_facts() {
    let (state_path, workflow_id, root) = resident_fixture("resume-rejected-replacement");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    assert!(
        !resident_thread_invalid(REAL_CLI_INVALID_RESUME_STDERR),
        "the incident fixture must prove this branch is structural, not an expanded string list"
    );
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let hook_config = config.clone();
    let hook_turn = Arc::new(AtomicUsize::new(0));
    let hook_turn_for_closure = Arc::clone(&hook_turn);
    let hook = Arc::new(move || {
        // The first binding is the old generation.  The replacement initial
        // turn is the only one that must prove submit_proposal can land a card.
        if hook_turn_for_closure.fetch_add(1, Ordering::SeqCst) != 1 {
            return Ok(());
        }
        let response = supervisor_orchestrator::call_tool(
            &hook_config,
            json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
        )?;
        let receipt: Value = serde_json::from_str(
            response["content"][0]["text"]
                .as_str()
                .ok_or_else(|| "replacement proposal receipt missing".to_string())?,
        )
        .map_err(|error| format!("replacement proposal receipt malformed:{error}"))?;
        if receipt["status"] != "proposal_created_pending_user_confirmation" {
            return Err(
                "replacement proposal receipt was not a pending confirmation card".to_string(),
            );
        }
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-old".to_string(),
            content: "首轮已建立。".to_string(),
        },
        MockOneShotOutcome::ResumeExitWithoutThreadStarted {
            exit_code: 1,
            stderr: REAL_CLI_INVALID_RESUME_STDERR.to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-new".to_string(),
            content: "换代后的首轮已落待确认卡。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-new".to_string(),
            content: "换代后第二轮仍在同一 thread。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-new".to_string(),
            content: "换代后第三轮仍在同一 thread。".to_string(),
        },
    ])
    .with_after_bind(hook);
    let initial = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "保留 FIRST_FACT。"),
        &config,
    )
    .expect("first generation");
    let replacement = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "thread 无效时请重建。"),
        &config,
    )
    .expect("invalid resume creates a new generation");
    let second_after_replacement = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请确认仍在换代后的同一 thread。"),
        &config,
    )
    .expect("second new-generation turn resumes the replacement thread");
    let third_after_replacement = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请再次确认仍在同一 thread。"),
        &config,
    )
    .expect("third new-generation turn resumes the replacement thread");
    assert_eq!(initial.thread_id.as_deref(), Some("thread-s1b-old"));
    assert_eq!(replacement.thread_id.as_deref(), Some("thread-s1b-new"));
    assert_eq!(second_after_replacement.thread_id, replacement.thread_id);
    assert_eq!(third_after_replacement.thread_id, replacement.thread_id);
    let plans = runner.plans.lock().expect("plans");
    assert_eq!(plans.len(), 5);
    assert!(plans[1].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(!plans[2].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(plans[3].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert!(plans[4].command_plan.argv.iter().any(|arg| arg == "resume"));
    assert_eq!(
        plans[1].expected_thread_id.as_deref(),
        Some("thread-s1b-old")
    );
    assert_eq!(
        plans[3].expected_thread_id.as_deref(),
        Some("thread-s1b-new")
    );
    assert_eq!(
        plans[4].expected_thread_id.as_deref(),
        Some("thread-s1b-new")
    );
    assert!(plans[2].prompt.contains("换代/首轮核心事实"));
    assert!(
        plans[2].prompt.contains("FIRST_FACT"),
        "replacement must inject the prior canonical conversation fact through the project blackboard"
    );
    assert!(manager
        .base
        .join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)
        .read_dir()
        .expect("archive dir")
        .next()
        .is_some());
    assert!(manager.active_path().is_dir());
    let session = supervisor_orchestrator::load_resident_session(&config)
        .expect("load replacement session")
        .expect("replacement session");
    assert_eq!(session.thread_id, "thread-s1b-new");
    assert_eq!(session.generation, 2);
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 4);
    assert_eq!(hook_turn.load(Ordering::SeqCst), 4);
    let proposal_store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("replacement proposal store");
    assert_eq!(proposal_store.proposals.len(), 1);
    assert_eq!(
        proposal_store.proposals[0].status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    let workflow_after: Value =
        serde_json::from_slice(&fs::read(&state_path).expect("workflow state after replacement"))
            .expect("workflow JSON after replacement");
    assert_eq!(workflow_after["workflow_chain_runs"], chain_before);
    let incident_audit = resident_supervisor_audit_events(&state_path)
        .into_iter()
        .find(|event| event["event_type"] == "supervisor_resident_invalid_resume_detected")
        .expect("invalid-resume audit event");
    let audit_detail = incident_audit["parameter_summary"]
        .as_str()
        .expect("private invalid-resume audit detail");
    assert!(audit_detail.contains("resume_exit_without_thread_started"));
    assert!(audit_detail.contains("no rollout found"));
    assert!(audit_detail.contains("(code -32600)"));
    assert!(
        !incident_audit["result_summary"]
            .as_str()
            .expect("human invalid-resume result summary")
            .contains("no rollout found"),
        "raw CLI stderr must stay out of the user-facing audit summary"
    );
    let pilot_read_model = supervisor_orchestrator::load_pilot_read_model(&config)
        .expect("pilot read model after invalid resume");
    assert!(pilot_read_model.audit_events.iter().all(|event| {
        !event.result_summary.contains("no rollout found")
            && !event.result_summary.contains("(code -32600)")
    }));
    fixture_cleanup(&root);
}

#[test]
fn s1b_invalid_resume_recovery_never_returns_cli_raw_error_to_user() {
    let (state_path, workflow_id, root) = resident_fixture("resume-rejected-human-fallback");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-old".to_string(),
            content: "首轮已建立。".to_string(),
        },
        MockOneShotOutcome::ResumeExitWithoutThreadStarted {
            exit_code: 1,
            stderr: REAL_CLI_INVALID_RESUME_STDERR.to_string(),
        },
        MockOneShotOutcome::Protocol("supervisor_resident_oneshot_exit_failed:1".to_string()),
    ]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立旧 thread。"),
        &config,
    )
    .expect("first generation");
    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "触发作废 resume。"),
        &config,
    )
    .expect("the canonical user message survives a failed replacement initial turn");
    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(
        outcome.message,
        "消息已送到主管，但主管这次没回上来——可以再发一次。"
    );
    assert!(!outcome.message.contains("no rollout found"));
    assert!(!outcome.message.contains("(code -32600)"));
    assert!(!outcome
        .message
        .contains("supervisor_resident_oneshot_exit_failed"));
    let incident_audit = resident_supervisor_audit_events(&state_path)
        .into_iter()
        .find(|event| event["event_type"] == "supervisor_resident_invalid_resume_detected")
        .expect("invalid-resume audit event remains available after fallback failure");
    assert!(incident_audit["parameter_summary"]
        .as_str()
        .expect("private invalid-resume audit detail")
        .contains(REAL_CLI_INVALID_RESUME_STDERR));
    assert_eq!(runner.plans.lock().expect("plans").len(), 3);
    fixture_cleanup(&root);
}

#[test]
fn s1b_failed_home_replacement_restores_the_prior_active_generation() {
    let (state_path, workflow_id, root) = resident_fixture("replacement-rollback");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-s1b-rollback".to_string(),
        content: "首轮已建立。".to_string(),
    }]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立可恢复的私有 home。"),
        &config,
    )
    .expect("first generation");
    let command_plan = runner
        .plans
        .lock()
        .expect("first command plan")
        .first()
        .expect("one mock plan")
        .command_plan
        .clone();
    let active = manager.active_path();
    let original_config =
        fs::read(active.join(SUPERVISOR_TEMP_HOME_CONFIG)).expect("read original private config");
    fs::remove_file(&manager.auth_source).expect("make replacement auth source unavailable");
    let error = manager
        .replace_active(&command_plan, &config, 2)
        .expect_err("replacement must fail before promoting a home without auth");
    assert!(error.contains("auth.json"));
    assert!(
        active.is_dir(),
        "prior active home is restored after failure"
    );
    assert_eq!(
        fs::read(active.join(SUPERVISOR_TEMP_HOME_CONFIG)).expect("read restored private config"),
        original_config
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_refuses_to_reuse_an_active_home_from_a_different_generation() {
    let (state_path, workflow_id, root) = resident_fixture("generation-mismatch");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-s1b-generation-one".to_string(),
        content: "首轮已建立。".to_string(),
    }]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "建立第一代私有家。"),
        &config,
    )
    .expect("first generation");
    let first_plan = runner
        .plans
        .lock()
        .expect("plans")
        .first()
        .expect("one mock plan")
        .command_plan
        .clone();
    let error = manager
        .ensure_active(&first_plan, &config, 2)
        .expect_err("a different durable generation must not reuse the active home");
    assert!(error.contains("元数据与当前项目身份不一致"));
    fixture_cleanup(&root);
}

#[test]
fn s1b_startup_marks_dead_running_turn_exited_without_destroying_binding() {
    let (state_path, workflow_id, root) = resident_fixture("stale");
    let config = resident_fixture_config(&state_path);
    supervisor_orchestrator::record_resident_session_created(
        &config,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        "thread-s1b-stale",
        9191,
        1,
    )
    .expect("record stale running session");
    let reaped = reap_supervisor_resident_stale_sessions_with(
        &state_path,
        &|pid| {
            assert_eq!(pid, 9191);
            Ok(false)
        },
        &config,
    )
    .expect("stale session scan");
    assert!(reaped);
    let session = supervisor_orchestrator::load_resident_session(&config)
        .expect("load stale session")
        .expect("stale session remains bound");
    assert_eq!(session.thread_id, "thread-s1b-stale");
    assert_eq!(session.host_pid, 0);
    assert_eq!(session.launch_status, "resident_exited");
    fixture_cleanup(&root);
}

#[test]
fn s1b_startup_marks_dead_prepared_turn_exited_without_destroying_binding() {
    let (state_path, workflow_id, root) = resident_fixture("stale-prepared");
    let config = resident_fixture_config(&state_path);
    supervisor_orchestrator::record_resident_session_created(
        &config,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        "thread-s1b-stale-prepared",
        9292,
        1,
    )
    .expect("record durable binding before a new turn starts");
    supervisor_orchestrator::record_resident_turn_prepared(
        &config,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        9292,
        1,
        None,
    )
    .expect("record a turn that dies before thread.started");
    let reaped = reap_supervisor_resident_stale_sessions_with(
        &state_path,
        &|pid| {
            assert_eq!(pid, 9292);
            Ok(false)
        },
        &config,
    )
    .expect("stale prepared-turn scan");
    assert!(reaped);
    let session = supervisor_orchestrator::load_resident_session(&config)
        .expect("load stale prepared session")
        .expect("stale prepared session remains bound");
    assert_eq!(session.thread_id, "thread-s1b-stale-prepared");
    assert_eq!(session.host_pid, 0);
    assert_eq!(session.launch_status, "resident_exited");
    fixture_cleanup(&root);
}

#[cfg(unix)]
#[test]
fn s1b_process_group_cleanup_reaps_a_term_ignoring_descendant() {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    // The parent and its background descendant both ignore TERM.  The helper
    // must therefore reach its group KILL sweep rather than merely reaping the
    // outer shell and leaving a bridge-like descendant behind.
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("trap '' TERM; /bin/sh -c 'trap \"\" TERM; while :; do sleep 1; done' & wait")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.process_group(0);
    let mut child = command.spawn().expect("spawn isolated cleanup fixture");
    let pid = child.id();
    std::thread::sleep(Duration::from_millis(30));

    if let Err(error) = stop_supervisor_resident_process_group(&mut child) {
        let _ = Command::new("/bin/kill")
            .arg("-KILL")
            .arg(format!("-{pid}"))
            .status();
        let _ = child.wait();
        panic!("one-shot process-group cleanup failed: {error}");
    }
    let group_alive = Command::new("/bin/kill")
        .arg("-0")
        .arg(format!("-{pid}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("probe isolated process group");
    assert!(
        !group_alive.success(),
        "TERM-ignoring descendant must not survive the one-shot group cleanup"
    );
}

#[test]
#[ignore = "requires explicit user authorization, a real Codex account, and a prepared fixed-project workflow state"]
fn s1b_live_resume_tool_card_and_replacement_require_explicit_harness_authorization() {
    let state_path = PathBuf::from(
        std::env::var("SYN_S1B_LIVE_WORKFLOW_STATE_PATH")
            .expect("set SYN_S1B_LIVE_WORKFLOW_STATE_PATH after user authorization"),
    );
    let workflow_id = std::env::var("SYN_S1B_LIVE_WORKFLOW_ID")
        .expect("set SYN_S1B_LIVE_WORKFLOW_ID after user authorization");
    let _approval_harness = install_s1b_live_approval_harness();
    let project_id = crate::project_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT);
    let proposal_count_before = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("read live proposal store before the tool turn")
    .proposals
    .len();
    let chain_before: Value = serde_json::from_slice::<Value>(
        &fs::read(&state_path).expect("read live workflow before the tool turn"),
    )
    .expect("parse live workflow before the tool turn")["workflow_chain_runs"]
        .clone();
    let fact_marker = "S1B_LIVE_FACT_MARIO_20260719";
    let card_marker = "S1B_LIVE_CARD_MARIO_20260719";
    let send = |message: &str| {
        submit_supervisor_resident_message(
            &state_path,
            &SubmitSupervisorResidentAnswerRequest {
                project_id: project_id.clone(),
                workflow_id: workflow_id.clone(),
                message_text: message.to_string(),
                client_request_id: None,
            },
        )
        .expect("authorized real one-shot supervisor turn")
    };

    let first = send(&format!(
        "请记住这条唯一工作台事实标记：{fact_marker}。现在请为“改标题成小马里奥”形成完整终版方案，并只通过 submit_proposal 工具在右侧创建一张待用户确认的方案卡；工具参数 user_goal 和 goal_summary 都必须逐字包含唯一卡片标记 {card_marker}；不要批准、不要起链。"
    ));
    let second = send(&format!(
        "请逐字引用刚才的唯一事实标记 {fact_marker}，并说明你仍在同一段主管对话里。"
    ));
    let third = send(&format!(
        "请再次逐字引用唯一事实标记 {fact_marker}，确认仍在同一 thread；不要新建方案卡、不要批准、不要起链。"
    ));
    let thread_id = first.thread_id.expect("first real thread id");
    assert_eq!(second.thread_id.as_deref(), Some(thread_id.as_str()));
    assert_eq!(third.thread_id.as_deref(), Some(thread_id.as_str()));
    assert!(
        second
            .supervisor_reply
            .as_deref()
            .is_some_and(|reply| reply.contains(fact_marker)),
        "second real turn must semantically continue the first-turn fact"
    );
    assert!(
        third
            .supervisor_reply
            .as_deref()
            .is_some_and(|reply| reply.contains(fact_marker)),
        "third real turn must continue the fact after the first exec already wrote its card"
    );
    let proposal_store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("read live proposal store after the tool turn");
    assert!(proposal_store.proposals.len() > proposal_count_before);
    assert!(proposal_store
        .proposals
        .iter()
        .skip(proposal_count_before)
        .any(|proposal| {
            proposal.status == crate::ProjectConsultationProposalStatus::PendingUserConfirmation
                && proposal.user_goal.contains(card_marker)
                && proposal.goal_summary.contains(card_marker)
        }));
    let workflow_after_tool: Value = serde_json::from_slice(
        &fs::read(&state_path).expect("read live workflow after the tool turn"),
    )
    .expect("parse live workflow after the tool turn");
    assert_eq!(workflow_after_tool["workflow_chain_runs"], chain_before);

    // Force the same recovery branch that a deleted/invalid real thread takes.
    // The next real turn must create a new thread and use the existing rebuild
    // facts rather than silently falling back to a persistent host.
    let config = resident_fixture_config(&state_path);
    let before_replacement = supervisor_orchestrator::load_resident_session(&config)
        .expect("load live resident binding")
        .expect("live resident binding exists");
    let mut invalid_thread_id_chars = before_replacement.thread_id.chars().collect::<Vec<_>>();
    let last = invalid_thread_id_chars
        .pop()
        .expect("live thread id is nonempty");
    invalid_thread_id_chars.push(if last == '0' { '1' } else { '0' });
    let invalid_thread_id = invalid_thread_id_chars.into_iter().collect::<String>();
    assert_ne!(invalid_thread_id, before_replacement.thread_id);
    supervisor_orchestrator::record_resident_session_reused(
        &config,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        &invalid_thread_id,
        std::process::id(),
        before_replacement.generation,
    )
    .expect("seed the authorized invalid-resume probe");
    let replacement = send(
        "请从工作台已有事实找出此前那条唯一事实标记并逐字回引；目标仍是把标题改成小马里奥；不要执行或批准。",
    );
    assert_ne!(
        replacement.thread_id.as_deref(),
        Some(invalid_thread_id.as_str())
    );
    let after_replacement = supervisor_orchestrator::load_resident_session(&config)
        .expect("load replacement binding")
        .expect("replacement binding exists");
    assert!(after_replacement.generation > before_replacement.generation);
    assert!(
        replacement
            .supervisor_reply
            .as_deref()
            .is_some_and(|reply| reply.contains(fact_marker)),
        "replacement first turn must rebuild from the durable project facts, not an old transcript"
    );
}
