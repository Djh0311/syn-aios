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

struct FixtureOutputInitFailureRunner {
    raw_marker: String,
    calls: AtomicUsize,
}

impl SupervisorResidentOneShotRunner for FixtureOutputInitFailureRunner {
    fn run(
        &self,
        plan: &SupervisorResidentOneShotPlan,
        home: &SupervisorResidentHome,
        on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let blocked_parent = home.root().join(&self.raw_marker);
        fs::write(&blocked_parent, "fixture output parent is a file").map_err(|error| {
            SupervisorResidentOneShotFailure::Protocol(format!(
                "fixture_output_init_setup_failed:{error}"
            ))
        })?;
        let mut failed_plan = plan.clone();
        failed_plan.command_plan.stderr_path = blocked_parent.join("step-0.stderr.txt");
        run_real_supervisor_resident_oneshot(
            &failed_plan,
            home,
            on_turn_prepared,
            on_thread_started,
        )
    }
}

struct FixturePreparedLifecycleFailureRunner {
    program: String,
    raw_marker: String,
    child_started: AtomicUsize,
}

impl SupervisorResidentOneShotRunner for FixturePreparedLifecycleFailureRunner {
    fn run(
        &self,
        plan: &SupervisorResidentOneShotPlan,
        home: &SupervisorResidentHome,
        _on_turn_prepared: &mut dyn FnMut(u32) -> Result<(), String>,
        _on_thread_started: &mut dyn FnMut(&str, u32) -> Result<(), String>,
    ) -> Result<SupervisorResidentTurn, SupervisorResidentOneShotFailure> {
        let mut failed_plan = plan.clone();
        failed_plan.command_plan.program = self.program.clone();
        failed_plan.command_plan.current_dir = failed_plan
            .workflow_state_path
            .parent()
            .expect("fixture workflow state parent")
            .display()
            .to_string();
        let mut fail_prepared_after_child_start = |pid: u32| {
            self.child_started.store(pid as usize, Ordering::SeqCst);
            Err(self.raw_marker.clone())
        };
        let mut no_binding_after_failed_prepare = |_thread_id: &str, _pid: u32| Ok(());
        run_real_supervisor_resident_oneshot(
            &failed_plan,
            home,
            &mut fail_prepared_after_child_start,
            &mut no_binding_after_failed_prepare,
        )
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
        fail_create_active_after_staging: false,
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

fn write_owner_only_test_file(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write test resident private file");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)
            .expect("inspect test resident private file")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions).expect("make test resident private file owner-only");
    }
}

#[cfg(unix)]
fn set_test_mode(path: &Path, mode: u32) {
    let mut permissions = fs::metadata(path)
        .expect("inspect fixture permissions")
        .permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).expect("set fixture permissions");
}

fn config_drift_quarantine_fixture(
    label: &str,
) -> (
    PathBuf,
    SupervisorResidentHomeManager,
    McpServerConfig,
    SupervisorCommandPlan,
    String,
) {
    let (state_path, _workflow_id, root) = resident_fixture(label);
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let executable = std::env::current_exe().expect("locate fixture workbench executable");
    let initial_plan = build_supervisor_resident_command_plan(
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &state_path,
        &resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        1,
        &executable,
        None,
    )
    .expect("build trusted first-generation command plan");
    let active = manager
        .ensure_active(&initial_plan, &config, 1)
        .expect("create trusted first-generation private home");
    let config_path = active.root().join(SUPERVISOR_TEMP_HOME_CONFIG);
    let drifted_config = format!(
        "{}\n[fixture_unknown_config_drift]\nvalue = true\n",
        fs::read_to_string(&config_path).expect("read trusted fixture config")
    );
    write_owner_only_test_config(&config_path, &drifted_config);
    let replacement_plan = build_supervisor_resident_command_plan(
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &state_path,
        &resident_run_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        2,
        &executable,
        None,
    )
    .expect("build replacement command plan");
    (root, manager, config, replacement_plan, drifted_config)
}

fn assert_config_drift_quarantine_rejected(
    manager: &SupervisorResidentHomeManager,
    plan: &SupervisorCommandPlan,
    config: &McpServerConfig,
) {
    let active = manager.active_path();
    let active_before = fs::symlink_metadata(&active).expect("inspect refused active home");
    assert!(
        manager
            .quarantine_config_drift_active(plan, config, 1, 2)
            .is_err(),
        "an untrusted home must not enter config-drift quarantine"
    );
    let active_after = fs::symlink_metadata(&active).expect("active home remains after refusal");
    assert_eq!(
        active_after.file_type().is_dir(),
        active_before.file_type().is_dir(),
        "refusal must leave the active home in place"
    );
    assert_eq!(
        active_after.file_type().is_symlink(),
        active_before.file_type().is_symlink(),
        "refusal must not replace the active home"
    );
    assert!(
        fs::symlink_metadata(manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)).is_err(),
        "a rejected home must not create an archive"
    );
    assert!(
        fs::read_dir(&manager.base)
            .expect("read fixture home base")
            .all(|entry| {
                !entry
                    .expect("read fixture home base entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".active-staging-")
            }),
        "a rejected home must not create staging"
    );
}

fn config_drift_quarantine_refusal_case<F>(label: &str, mutate: F)
where
    F: FnOnce(&mut SupervisorResidentHomeManager, &McpServerConfig),
{
    let (root, mut manager, config, replacement_plan, _drifted_config) =
        config_drift_quarantine_fixture(label);
    mutate(&mut manager, &config);
    assert_config_drift_quarantine_rejected(&manager, &replacement_plan, &config);
    fixture_cleanup(&root);
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

fn resident_delivery_diagnostics_for_message(state_path: &Path, message_id: &str) -> Vec<Value> {
    resident_canonical_events(state_path)
        .expect("canonical resident events")
        .into_iter()
        .filter(|event| {
            event["event_type"] == "supervisor_resident_delivery_diagnostic_recorded"
                && event["message_id"] == message_id
        })
        .collect()
}

fn resident_recorded_message_id_for_client(
    state_path: &Path,
    client_request_id: Option<&str>,
) -> String {
    resident_canonical_events(state_path)
        .expect("canonical resident events")
        .into_iter()
        .find(|event| {
            event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT
                && match client_request_id {
                    Some(client_request_id) => event["client_request_id"] == client_request_id,
                    None => true,
                }
        })
        .and_then(|event| event["message_id"].as_str().map(str::to_string))
        .expect("recorded canonical user message id")
}

fn assert_sanitized_delivery_diagnostic(
    state_path: &Path,
    workflow_id: &str,
    message_id: &str,
    expected_stage: &str,
    expected_family: &str,
    raw_marker: &str,
) {
    let diagnostics = resident_delivery_diagnostics_for_message(state_path, message_id);
    assert_eq!(
        diagnostics.len(),
        1,
        "one failed recorded message has exactly one canonical diagnostic"
    );
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["stage"], expected_stage);
    assert_eq!(diagnostic["stable_error_family"], expected_family);
    assert_eq!(diagnostic["message_id"], message_id);
    assert_eq!(diagnostic["workflow_id"], workflow_id);
    assert!(diagnostic["run_id"].as_str().is_some());
    assert!(diagnostic["generation"].as_u64().is_some());
    assert!(diagnostic["thread_id"].is_null());
    assert!(diagnostic["event_id"].as_str().is_some());
    assert!(
        !diagnostic["target_ref"]
            .as_str()
            .expect("internal diagnostic target ref")
            .contains(workflow_id),
        "internal diagnostic must not be projected into the user workflow ledger"
    );
    let fields = diagnostic
        .as_object()
        .expect("diagnostic object")
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        fields,
        std::collections::BTreeSet::from([
            "created_at",
            "event_id",
            "event_type",
            "generation",
            "message_id",
            "project_id",
            "run_id",
            "stable_error_family",
            "stage",
            "target_ref",
            "workflow_id",
        ]),
        "diagnostic has only the safe canonical schema plus the required audit key"
    );
    let canonical_text = fs::read_to_string(state_path).expect("read canonical workflow state");
    assert!(
        !canonical_text.contains(raw_marker),
        "raw failure detail must not enter canonical JSON"
    );
    let canonical_events = resident_canonical_events(state_path).expect("canonical events");
    let ledger_entries =
        crate::derive_workflow_ledger_entries(workflow_id, &canonical_events, &[], &[], &[]);
    let diagnostic_event_id = diagnostic["event_id"]
        .as_str()
        .expect("diagnostic event id");
    assert!(
        ledger_entries
            .iter()
            .all(|entry| entry.ledger_entry_id != diagnostic_event_id),
        "internal diagnostic must not change the user workflow ledger"
    );
}

fn assert_no_message_injection_or_reply(state_path: &Path, message_id: &str) {
    let events = resident_canonical_events(state_path).expect("canonical resident events");
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT
                    && event["message_id"] == message_id
            })
            .count(),
        0
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT
                    && event["reply_to_message_id"] == message_id
            })
            .count(),
        0
    );
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
            "checks": ["cargo test --lib"],
            "max_worker_dispatches": 1,
            "max_runtime_minutes": 30
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
        SupervisorResidentOneShotFailure::Classified {
            diagnostic_stage: SupervisorResidentDeliveryDiagnosticStage::RunnerTerminal,
            detail,
        } if detail == "supervisor_resident_turn_failed:fixture hard failure"
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
        SupervisorResidentOneShotFailure::Classified {
            diagnostic_stage: SupervisorResidentDeliveryDiagnosticStage::RunnerTerminal,
            detail,
        } if detail == "supervisor_resident_turn_completed_event_missing"
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
        SupervisorResidentOneShotFailure::Classified {
            diagnostic_stage: SupervisorResidentDeliveryDiagnosticStage::ThreadBinding,
            detail,
        } if detail == "supervisor_resident_thread_started_event_missing"
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
    assert!(
        fs::symlink_metadata(manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)).is_err(),
        "expected and exact legacy configs must never enter config-drift quarantine"
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
fn s1b_h2_config_drift_quarantines_trusted_active_and_restarts_generation() {
    let (state_path, workflow_id, root) = resident_fixture("config-drift-quarantine");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let hook_config = config.clone();
    let hook_turn = Arc::new(AtomicUsize::new(0));
    let hook_turn_for_closure = Arc::clone(&hook_turn);
    let hook = Arc::new(move || {
        // The first binding is the old generation. Only the replacement
        // initial turn may exercise the preserved H2 submit_proposal grant.
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
                .ok_or_else(|| "config-drift proposal receipt missing".to_string())?,
        )
        .map_err(|error| format!("config-drift proposal receipt malformed:{error}"))?;
        if receipt["status"] != "proposal_created_pending_user_confirmation" {
            return Err("config-drift proposal was not a pending card".to_string());
        }
        Ok(())
    });
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-config-drift-old".to_string(),
            content: "保留 CONFIG_DRIFT_FACT。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-config-drift-new".to_string(),
            content: "隔离后的新 generation 首轮已完成。".to_string(),
        },
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-config-drift-new".to_string(),
            content: "新 generation 的后续回合已续接。".to_string(),
        },
    ])
    .with_after_bind(hook);

    let initial = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立将被隔离的旧 generation。"),
        &config,
    )
    .expect("first generation");
    assert_eq!(
        initial.thread_id.as_deref(),
        Some("thread-s1b-config-drift-old")
    );

    let active = manager.active_path();
    let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
    let private_marker = "R1_CONFIG_DRIFT_PRIVATE_SENTINEL";
    let drifted_config = format!(
        "{}\n[fixture_unknown_config_drift]\nmarker = \"{private_marker}\"\n",
        fs::read_to_string(&config_path).expect("read current resident config")
    );
    write_owner_only_test_config(&config_path, &drifted_config);
    let old_plan = runner
        .plans
        .lock()
        .expect("old generation command plan")
        .first()
        .expect("one old generation plan")
        .command_plan
        .clone();
    assert!(matches!(
        manager.ensure_active_or_config_drift(&old_plan, &config, 1),
        Err(SupervisorResidentEnsureActiveError::ConfigDrift)
    ));

    let replacement = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "当前消息必须从隔离后的新 generation 开始。"),
        &config,
    )
    .expect("config drift quarantines the trusted old generation");
    let follow_up = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请续接隔离后的新 generation。"),
        &config,
    )
    .expect("follow-up resumes the replacement generation");

    assert_eq!(
        replacement.thread_id.as_deref(),
        Some("thread-s1b-config-drift-new")
    );
    assert_eq!(follow_up.thread_id, replacement.thread_id);
    let plans = runner.plans.lock().expect("recorded plans");
    assert_eq!(plans.len(), 3);
    assert!(
        !plans[1].command_plan.argv.iter().any(|arg| arg == "resume"),
        "the config-drift message must never attempt the old resume"
    );
    assert!(plans[1].prompt.contains("换代/首轮核心事实"));
    assert!(plans[1].prompt.contains("CONFIG_DRIFT_FACT"));
    assert!(
        plans[2].command_plan.argv.iter().any(|arg| arg == "resume")
            && plans[2].expected_thread_id.as_deref() == Some("thread-s1b-config-drift-new"),
        "the following message must resume only the replacement thread"
    );
    let replacement_plan = plans[1].command_plan.clone();
    drop(plans);

    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    let archived = fs::read_dir(&archive_root)
        .expect("old active is quarantined")
        .next()
        .expect("one quarantined generation")
        .expect("read quarantined generation")
        .path();
    assert_eq!(
        fs::read_to_string(archived.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read quarantined unknown config"),
        drifted_config,
        "quarantine must preserve unknown config bytes without migration"
    );
    assert_eq!(
        fs::read_to_string(active.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read replacement config"),
        supervisor_resident_mcp_config_toml(&replacement_plan)
            .expect("render exact replacement config")
    );
    let metadata: SupervisorResidentHomeMetadata = serde_json::from_slice(
        &fs::read(active.join(SUPERVISOR_RESIDENT_HOME_METADATA))
            .expect("read replacement metadata"),
    )
    .expect("parse replacement metadata");
    assert_eq!(metadata.generation, 2);
    let session = supervisor_orchestrator::load_resident_session(&config)
        .expect("load replacement session")
        .expect("replacement session");
    assert_eq!(session.thread_id, "thread-s1b-config-drift-new");
    assert_eq!(session.generation, 2);
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 3);
    assert_eq!(hook_turn.load(Ordering::SeqCst), 3);
    let proposal_store = crate::project_consultation_proposal_store::load_store(
        &state_path,
        crate::unix_timestamp_ms(),
    )
    .expect("config-drift proposal store");
    assert_eq!(proposal_store.proposals.len(), 1);
    assert_eq!(
        proposal_store.proposals[0].status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    let workflow_after: Value =
        serde_json::from_slice(&fs::read(&state_path).expect("workflow after replacement"))
            .expect("workflow JSON after replacement");
    assert_eq!(workflow_after["workflow_chain_runs"], chain_before);
    let ordinary_read_model = serde_json::to_string(
        &resident_canonical_events(&state_path).expect("canonical resident events"),
    )
    .expect("serialize ordinary resident read model");
    let audit_read_model = serde_json::to_string(&resident_supervisor_audit_events(&state_path))
        .expect("serialize resident audit read model");
    assert!(!ordinary_read_model.contains(private_marker));
    assert!(!audit_read_model.contains(private_marker));
    fixture_cleanup(&root);
}

#[cfg(unix)]
#[test]
fn s1b_h2_config_drift_quarantine_refuses_unsafe_active_or_private_files() {
    config_drift_quarantine_refusal_case("config-drift-malformed", |manager, _config| {
        write_owner_only_test_config(
            &manager.active_path().join(SUPERVISOR_TEMP_HOME_CONFIG),
            "fixture = [",
        );
    });
    config_drift_quarantine_refusal_case("config-drift-missing", |manager, _config| {
        fs::remove_file(manager.active_path().join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("remove fixture config");
    });
    config_drift_quarantine_refusal_case("config-drift-config-symlink", |manager, _config| {
        let active = manager.active_path();
        let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
        let target = manager.base.join("fixture-config-target");
        write_owner_only_test_file(&target, b"fixture config target");
        fs::remove_file(&config_path).expect("remove fixture config before symlink");
        std::os::unix::fs::symlink(&target, &config_path).expect("create fixture config symlink");
    });
    config_drift_quarantine_refusal_case("config-drift-metadata-symlink", |manager, _config| {
        let active = manager.active_path();
        let metadata_path = active.join(SUPERVISOR_RESIDENT_HOME_METADATA);
        let target = manager.base.join("fixture-metadata-target");
        write_owner_only_test_file(&target, b"{}");
        fs::remove_file(&metadata_path).expect("remove fixture metadata before symlink");
        std::os::unix::fs::symlink(&target, &metadata_path)
            .expect("create fixture metadata symlink");
    });
    config_drift_quarantine_refusal_case("config-drift-config-permissions", |manager, _config| {
        set_test_mode(
            &manager.active_path().join(SUPERVISOR_TEMP_HOME_CONFIG),
            0o640,
        );
    });
    config_drift_quarantine_refusal_case(
        "config-drift-metadata-permissions",
        |manager, _config| {
            set_test_mode(
                &manager
                    .active_path()
                    .join(SUPERVISOR_RESIDENT_HOME_METADATA),
                0o640,
            );
        },
    );
    config_drift_quarantine_refusal_case("config-drift-active-permissions", |manager, _config| {
        set_test_mode(&manager.active_path(), 0o755);
    });
    config_drift_quarantine_refusal_case("config-drift-config-unreadable", |manager, _config| {
        set_test_mode(
            &manager.active_path().join(SUPERVISOR_TEMP_HOME_CONFIG),
            0o000,
        );
    });
    config_drift_quarantine_refusal_case("config-drift-active-symlink", |manager, _config| {
        let active = manager.active_path();
        let parked = manager.base.join("fixture-active-parked");
        fs::rename(&active, &parked).expect("park fixture active home");
        std::os::unix::fs::symlink(&parked, &active).expect("create fixture active symlink");
    });
}

#[cfg(unix)]
#[test]
fn s1b_h2_config_drift_quarantine_refuses_metadata_or_auth_mismatch() {
    config_drift_quarantine_refusal_case("config-drift-metadata-run", |manager, _config| {
        let metadata_path = manager
            .active_path()
            .join(SUPERVISOR_RESIDENT_HOME_METADATA);
        let mut metadata: SupervisorResidentHomeMetadata =
            serde_json::from_slice(&fs::read(&metadata_path).expect("read fixture metadata"))
                .expect("parse fixture metadata");
        metadata.run_id = "fixture-wrong-run".to_string();
        write_owner_only_test_file(
            &metadata_path,
            &serde_json::to_vec(&metadata).expect("serialize mismatched run metadata"),
        );
    });
    config_drift_quarantine_refusal_case("config-drift-metadata-workflow", |manager, _config| {
        let metadata_path = manager
            .active_path()
            .join(SUPERVISOR_RESIDENT_HOME_METADATA);
        let mut metadata: SupervisorResidentHomeMetadata =
            serde_json::from_slice(&fs::read(&metadata_path).expect("read fixture metadata"))
                .expect("parse fixture metadata");
        metadata.workflow_state_path = PathBuf::from("fixture-wrong-workflow-state");
        write_owner_only_test_file(
            &metadata_path,
            &serde_json::to_vec(&metadata).expect("serialize mismatched workflow metadata"),
        );
    });
    config_drift_quarantine_refusal_case("config-drift-metadata-generation", |manager, _config| {
        let metadata_path = manager
            .active_path()
            .join(SUPERVISOR_RESIDENT_HOME_METADATA);
        let mut metadata: SupervisorResidentHomeMetadata =
            serde_json::from_slice(&fs::read(&metadata_path).expect("read fixture metadata"))
                .expect("parse fixture metadata");
        metadata.generation = 2;
        write_owner_only_test_file(
            &metadata_path,
            &serde_json::to_vec(&metadata).expect("serialize mismatched generation metadata"),
        );
    });
    config_drift_quarantine_refusal_case("config-drift-auth-regular", |manager, _config| {
        let auth_path = manager.active_path().join(SUPERVISOR_TEMP_HOME_AUTH);
        fs::remove_file(&auth_path).expect("remove fixture auth symlink");
        write_owner_only_test_file(&auth_path, b"fixture auth copy");
    });
    config_drift_quarantine_refusal_case("config-drift-auth-mismatch", |manager, _config| {
        let auth_path = manager.active_path().join(SUPERVISOR_TEMP_HOME_AUTH);
        let target = manager.base.join("fixture-wrong-auth-source");
        write_owner_only_test_file(&target, b"fixture wrong auth source");
        fs::remove_file(&auth_path).expect("remove fixture auth symlink");
        std::os::unix::fs::symlink(&target, &auth_path).expect("create mismatched auth symlink");
    });
    config_drift_quarantine_refusal_case("config-drift-auth-source-symlink", |manager, _config| {
        let target = manager.base.join("fixture-auth-source-target");
        fs::rename(&manager.auth_source, &target).expect("park fixture auth source");
        std::os::unix::fs::symlink(&target, &manager.auth_source)
            .expect("create fixture auth-source symlink");
    });
}

#[cfg(unix)]
#[test]
fn s1b_h2_config_drift_quarantine_refuses_untrusted_archive_root() {
    let (root, manager, config, replacement_plan, drifted_config) =
        config_drift_quarantine_fixture("config-drift-archive-permissions");
    let active = manager.active_path();
    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    fs::create_dir(&archive_root).expect("create fixture archive root");
    set_test_mode(&archive_root, 0o755);
    assert!(
        manager
            .quarantine_config_drift_active(&replacement_plan, &config, 1, 2)
            .is_err(),
        "a non-owner-only archive root must reject config-drift quarantine"
    );
    assert!(active.is_dir());
    assert_eq!(
        fs::read_to_string(active.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read active config after archive-root refusal"),
        drifted_config
    );
    assert!(fs::read_dir(&archive_root)
        .expect("read refused archive root")
        .next()
        .is_none());
    fixture_cleanup(&root);

    let (root, manager, config, replacement_plan, drifted_config) =
        config_drift_quarantine_fixture("config-drift-archive-symlink");
    let active = manager.active_path();
    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    let archive_target = manager.base.join("fixture-archive-target");
    fs::create_dir(&archive_target).expect("create fixture archive target");
    std::os::unix::fs::symlink(&archive_target, &archive_root)
        .expect("create fixture archive symlink");
    assert!(
        manager
            .quarantine_config_drift_active(&replacement_plan, &config, 1, 2)
            .is_err(),
        "a symlink archive root must reject config-drift quarantine"
    );
    assert!(active.is_dir());
    assert_eq!(
        fs::read_to_string(active.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read active config after archive symlink refusal"),
        drifted_config
    );
    assert!(fs::read_dir(&archive_target)
        .expect("read fixture archive target")
        .next()
        .is_none());
    fixture_cleanup(&root);
}

#[cfg(unix)]
#[test]
fn s1b_h2_config_drift_base_symlink_is_rejected_before_permission_mutation() {
    let (root, manager, config, replacement_plan, _drifted_config) =
        config_drift_quarantine_fixture("config-drift-base-symlink");
    let base = manager.base.clone();
    let parked = root.join("fixture-base-parked");
    fs::rename(&base, &parked).expect("park fixture home base");
    set_test_mode(&parked, 0o755);
    let mode_before = fs::metadata(&parked)
        .expect("inspect parked base permissions")
        .permissions()
        .mode()
        & 0o777;
    std::os::unix::fs::symlink(&parked, &base).expect("create fixture base symlink");

    assert!(matches!(
        manager.ensure_active_or_config_drift(&replacement_plan, &config, 1),
        Err(SupervisorResidentEnsureActiveError::Other(_))
    ));
    let mode_after = fs::metadata(&parked)
        .expect("inspect parked base after refusal")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode_after, mode_before);
    assert!(fs::symlink_metadata(&base)
        .expect("inspect refused base")
        .file_type()
        .is_symlink());
    assert!(
        fs::symlink_metadata(parked.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)).is_err(),
        "a rejected base must not create an archive"
    );
    fixture_cleanup(&root);

    let (root, manager, config, replacement_plan, _drifted_config) =
        config_drift_quarantine_fixture("config-drift-base-permissions");
    set_test_mode(&manager.base, 0o755);
    let mode_before = fs::metadata(&manager.base)
        .expect("inspect non-owner-only base permissions")
        .permissions()
        .mode()
        & 0o777;
    assert!(matches!(
        manager.ensure_active_or_config_drift(&replacement_plan, &config, 1),
        Err(SupervisorResidentEnsureActiveError::Other(_))
    ));
    let mode_after = fs::metadata(&manager.base)
        .expect("inspect non-owner-only base after refusal")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode_after, mode_before);
    assert!(
        fs::symlink_metadata(manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)).is_err(),
        "a non-owner-only base must not create an archive"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_config_drift_quarantine_restores_old_active_when_fresh_create_fails() {
    let (root, mut manager, config, replacement_plan, drifted_config) =
        config_drift_quarantine_fixture("config-drift-create-rollback");
    manager.fail_create_active_after_staging = true;
    let active = manager.active_path();

    assert!(
        manager
            .quarantine_config_drift_active(&replacement_plan, &config, 1, 2)
            .is_err(),
        "a fresh-home create failure must stop the config-drift replacement"
    );
    assert!(
        active.is_dir(),
        "the original active home is restored after fresh-home create failure"
    );
    assert_eq!(
        fs::read_to_string(active.join(SUPERVISOR_TEMP_HOME_CONFIG))
            .expect("read restored unknown config"),
        drifted_config,
        "rollback must preserve unknown config bytes exactly"
    );
    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    assert!(
        fs::read_dir(&archive_root)
            .expect("read empty rollback archive")
            .next()
            .is_none(),
        "rollback must not leave a second active generation in archive"
    );
    assert!(
        fs::read_dir(&manager.base)
            .expect("read fixture home base after rollback")
            .all(|entry| {
                !entry
                    .expect("read fixture home base entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".active-staging-")
            }),
        "rollback fixture must not leave staging behind"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_config_drift_untrusted_preflight_never_archives_or_runs_the_old_resume() {
    let (state_path, workflow_id, root) = resident_fixture("config-drift-untrusted-preflight");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-s1b-config-drift-untrusted".to_string(),
        content: "旧 generation 已建立。".to_string(),
    }]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立可拒绝的旧 generation。"),
        &config,
    )
    .expect("trusted first generation");
    let active = manager.active_path();
    let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
    let private_marker = "R1_CONFIG_DRIFT_UNTRUSTED_PRIVATE_SENTINEL";
    write_owner_only_test_config(&config_path, &format!("fixture = [\"{private_marker}\""));
    let mut request = resident_request(&workflow_id, "这个不可信 home 必须保持关闭。 ");
    request.client_request_id = Some("c6d5b4a3-9210-4f8e-b7d6-a5c4b3d2e1f0".to_string());

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &request,
        &config,
    )
    .expect("untrusted preflight returns a sanitized user outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert!(!outcome.message.contains(private_marker));
    assert_eq!(runner.plans.lock().expect("recorded plans").len(), 1);
    assert!(active.is_dir());
    assert_eq!(
        fs::read_to_string(&config_path).expect("read rejected config bytes"),
        format!("fixture = [\"{private_marker}\"")
    );
    assert!(
        fs::symlink_metadata(manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE)).is_err(),
        "unsafe preflight must not archive the active home"
    );
    let message_id =
        resident_recorded_message_id_for_client(&state_path, request.client_request_id.as_deref());
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "preflight",
        "preflight_home",
        private_marker,
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_config_drift_initial_runner_failure_is_not_retried_or_injected() {
    let (state_path, workflow_id, root) = resident_fixture("config-drift-initial-runner-failure");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let private_marker = "R1_CONFIG_DRIFT_INITIAL_PRIVATE_FAILURE";
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-config-drift-runner-old".to_string(),
            content: "旧 generation 已建立。".to_string(),
        },
        MockOneShotOutcome::Protocol(private_marker.to_string()),
    ]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立会被隔离的旧 generation。"),
        &config,
    )
    .expect("trusted first generation");
    let active = manager.active_path();
    let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
    let drifted_config = format!(
        "{}\n[fixture_unknown_config_drift]\nvalue = true\n",
        fs::read_to_string(&config_path).expect("read trusted config")
    );
    write_owner_only_test_config(&config_path, &drifted_config);
    let mut request = resident_request(&workflow_id, "隔离后的 initial 失败不得重试或伪造回复。 ");
    request.client_request_id = Some("f0e1d2c3-b4a5-4698-87f6-e5d4c3b2a190".to_string());

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &request,
        &config,
    )
    .expect("initial runner failure returns a sanitized user outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert!(!outcome.message.contains(private_marker));
    let plans = runner.plans.lock().expect("recorded plans");
    assert_eq!(plans.len(), 2, "failed replacement initial is not retried");
    assert!(
        !plans[1].command_plan.argv.iter().any(|arg| arg == "resume"),
        "the failed replacement remains one initial invocation"
    );
    drop(plans);
    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    assert_eq!(
        fs::read_dir(&archive_root)
            .expect("read config-drift archive")
            .count(),
        1,
        "the old home is quarantined once even when the new initial runner fails"
    );
    let message_id =
        resident_recorded_message_id_for_client(&state_path, request.client_request_id.as_deref());
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "unknown",
        "unknown",
        private_marker,
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_config_drift_watchdog_silence_runs_the_replacement_initial_once() {
    let (state_path, workflow_id, root) = resident_fixture("config-drift-watchdog-once");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Turn {
            thread_id: "thread-s1b-config-drift-watchdog-old".to_string(),
            content: "旧 generation 已建立。".to_string(),
        },
        MockOneShotOutcome::WatchdogSilence,
    ]);
    submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "先建立会被隔离的旧 generation。"),
        &config,
    )
    .expect("trusted first generation");
    let active = manager.active_path();
    let config_path = active.join(SUPERVISOR_TEMP_HOME_CONFIG);
    let drifted_config = format!(
        "{}\n[fixture_unknown_config_drift]\nvalue = true\n",
        fs::read_to_string(&config_path).expect("read trusted config")
    );
    write_owner_only_test_config(&config_path, &drifted_config);
    let mut request = resident_request(&workflow_id, "隔离后的 initial 只能执行一次。 ");
    request.client_request_id = Some("f2e1d0c9-b8a7-46f5-94e3-d2c1b0a99887".to_string());

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &request,
        &config,
    )
    .expect("watchdog silence returns a sanitized user outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    let plans = runner.plans.lock().expect("recorded plans");
    assert_eq!(
        plans.len(),
        2,
        "replacement initial must not be watchdog-retried"
    );
    assert!(
        !plans[1].command_plan.argv.iter().any(|arg| arg == "resume"),
        "the sole replacement invocation remains initial"
    );
    drop(plans);
    let archive_root = manager.base.join(SUPERVISOR_RESIDENT_HOME_ARCHIVE);
    assert_eq!(
        fs::read_dir(&archive_root)
            .expect("read config-drift archive")
            .count(),
        1
    );
    let message_id =
        resident_recorded_message_id_for_client(&state_path, request.client_request_id.as_deref());
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "runner",
        "runner_terminal",
        "R1_CONFIG_DRIFT_WATCHDOG_PRIVATE_SENTINEL",
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_config_drift_direct_preflight_home_error_is_sanitized() {
    let private_marker = "R1_CONFIG_DRIFT_DIRECT_PRIVATE_SENTINEL";
    let error = supervisor_resident_direct_consult_error(SupervisorResidentConsultFailure {
        diagnostic_stage: SupervisorResidentDeliveryDiagnosticStage::PreflightHome,
        raw_detail: format!("fixture direct preflight detail {private_marker}"),
        generation: Some(1),
        thread_id: Some("fixture-private-thread-identity".to_string()),
    });

    assert_eq!(error, SUPERVISOR_RESIDENT_HUMAN_RETRY_MESSAGE);
    assert!(!error.contains(private_marker));
    assert!(!error.contains("fixture-private-thread-identity"));
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

#[test]
fn s1b_h2_delivery_diagnostic_preflight_home_failure_is_sanitized_and_message_scoped() {
    let (state_path, workflow_id, root) = resident_fixture("r3b-preflight-home");
    let raw_marker = "R3B_PREFLIGHT_HOME_PRIVATE_PATH_SENTINEL";
    let mut manager = resident_fixture_manager(&root);
    manager.auth_source = root.join(raw_marker);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Protocol(
        "runner must not start after home preflight failure".to_string(),
    )]);

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请继续自然说明当前方案。"),
        &config,
    )
    .expect("recorded user message keeps the existing incomplete outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(
        outcome.message,
        "消息已送到主管，但主管这次没回上来——可以再发一次。"
    );
    assert_eq!(
        runner.plans.lock().expect("mock plans").len(),
        0,
        "home preflight failure must not start the runner"
    );
    let message_id = resident_recorded_message_id_for_client(&state_path, None);
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "preflight",
        "preflight_home",
        raw_marker,
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_delivery_diagnostic_output_init_failure_has_no_spawn_or_registry_fact() {
    let (state_path, workflow_id, root) = resident_fixture("r3b-output-init");
    let raw_marker = "R3B_OUTPUT_INIT_PRIVATE_PATH_SENTINEL";
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = FixtureOutputInitFailureRunner {
        raw_marker: raw_marker.to_string(),
        calls: AtomicUsize::new(0),
    };

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请继续这一段自然对话。"),
        &config,
    )
    .expect("recorded user message keeps the existing incomplete outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(runner.calls.load(Ordering::SeqCst), 1);
    let message_id = resident_recorded_message_id_for_client(&state_path, None);
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "runner",
        "runner_output_init",
        raw_marker,
    );
    let registry_path = state_path
        .parent()
        .expect("fixture state parent")
        .join("exec-process-registry.v1.json");
    assert!(
        !registry_path.exists(),
        "output initialization failure must not create a process registry fact"
    );
    let audit_events = resident_supervisor_audit_events(&state_path);
    assert!(audit_events.iter().all(|event| {
        event["event_type"] != "supervisor_resident_turn_prepared"
            && event["event_type"] != "supervisor_resident_session_created"
    }));
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_delivery_diagnostic_prepared_lifecycle_failure_has_no_false_binding_or_registry() {
    let (state_path, workflow_id, root) = resident_fixture("r3b-prepared-lifecycle");
    let raw_marker = "R3B_PREPARED_LIFECYCLE_PRIVATE_ERROR_SENTINEL";
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let fake_codex = resident_real_config_consuming_codex_script(&root);
    let runner = FixturePreparedLifecycleFailureRunner {
        program: fake_codex.display().to_string(),
        raw_marker: raw_marker.to_string(),
        child_started: AtomicUsize::new(0),
    };

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请继续这一段主管自然对话。"),
        &config,
    )
    .expect("recorded user message keeps the existing incomplete outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_ne!(
        runner.child_started.load(Ordering::SeqCst),
        0,
        "the child must have started before the prepared lifecycle callback failed"
    );
    let message_id = resident_recorded_message_id_for_client(&state_path, None);
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert_sanitized_delivery_diagnostic(
        &state_path,
        &workflow_id,
        &message_id,
        "runner",
        "prepared_lifecycle_write",
        raw_marker,
    );
    let registry_path = state_path
        .parent()
        .expect("fixture state parent")
        .join("exec-process-registry.v1.json");
    assert!(
        !registry_path.exists(),
        "a rejected prepared lifecycle write must not register the child"
    );
    let audit_events = resident_supervisor_audit_events(&state_path);
    assert!(audit_events.iter().all(|event| {
        event["event_type"] != "supervisor_resident_turn_prepared"
            && event["event_type"] != "supervisor_resident_session_created"
            && event["event_type"] != "supervisor_resident_session_reused"
            && event["event_type"] != "supervisor_resident_session_replaced"
    }));
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_delivery_diagnostic_same_client_replay_does_not_duplicate_and_new_clients_remain_distinct(
) {
    let (state_path, workflow_id, root) = resident_fixture("r3b-client-idempotency");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![
        MockOneShotOutcome::Protocol("R3B_CLIENT_ONE_PRIVATE_ERROR".to_string()),
        MockOneShotOutcome::Protocol("R3B_CLIENT_TWO_PRIVATE_ERROR".to_string()),
        MockOneShotOutcome::Protocol("R3B_CLIENT_THREE_PRIVATE_ERROR".to_string()),
    ]);
    let mut first = resident_request(&workflow_id, "请继续同一段自然对话。");
    first.client_request_id = Some("0a1b2c3d-4e5f-4a6b-8c9d-0e1f2a3b4c5d".to_string());

    let first_outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &first,
        &config,
    )
    .expect("first recorded message is incomplete");
    let replay_outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &first,
        &config,
    )
    .expect("same client request is reconciled without another turn");
    assert_eq!(
        first_outcome.status,
        "message_recorded_supervisor_incomplete"
    );
    assert_eq!(
        replay_outcome.status,
        "message_recorded_supervisor_incomplete"
    );

    for (client_request_id, message) in [
        (
            "1a2b3c4d-5e6f-4a7b-8c9d-0e1f2a3b4c5d",
            "这是第二个独立用户动作。",
        ),
        (
            "2a3b4c5d-6e7f-4a8b-8c9d-0e1f2a3b4c5d",
            "这是第三个独立用户动作。",
        ),
    ] {
        let mut request = resident_request(&workflow_id, message);
        request.client_request_id = Some(client_request_id.to_string());
        let outcome = submit_supervisor_resident_answer_with_parts(
            &runner,
            &manager,
            &state_path,
            &request,
            &config,
        )
        .expect("each independent recorded user action remains incomplete");
        assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    }

    let events = resident_canonical_events(&state_path).expect("canonical resident events");
    let recorded = events
        .iter()
        .filter(|event| {
            event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT
                && event["client_request_id"].is_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        recorded.len(),
        3,
        "three distinct client actions stay distinct"
    );
    let diagnostic_count = events
        .iter()
        .filter(|event| event["event_type"] == "supervisor_resident_delivery_diagnostic_recorded")
        .count();
    assert_eq!(
        diagnostic_count, 3,
        "same-client replay does not duplicate its diagnostic, while new clients receive one each"
    );
    for (client_request_id, raw_marker) in [
        (
            "0a1b2c3d-4e5f-4a6b-8c9d-0e1f2a3b4c5d",
            "R3B_CLIENT_ONE_PRIVATE_ERROR",
        ),
        (
            "1a2b3c4d-5e6f-4a7b-8c9d-0e1f2a3b4c5d",
            "R3B_CLIENT_TWO_PRIVATE_ERROR",
        ),
        (
            "2a3b4c5d-6e7f-4a8b-8c9d-0e1f2a3b4c5d",
            "R3B_CLIENT_THREE_PRIVATE_ERROR",
        ),
    ] {
        let message_id =
            resident_recorded_message_id_for_client(&state_path, Some(client_request_id));
        assert_sanitized_delivery_diagnostic(
            &state_path,
            &workflow_id,
            &message_id,
            "unknown",
            "unknown",
            raw_marker,
        );
    }
    assert_eq!(
        runner.plans.lock().expect("mock plans").len(),
        3,
        "same-client replay must not rerun consult"
    );
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_delivery_diagnostic_batch2_append_failure_preserves_recorded_business_outcome() {
    let (state_path, workflow_id, root) = resident_fixture("r3b-diagnostic-append-failure");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Protocol(
        "R3B_DIAGNOSTIC_SOURCE_FAILURE_SENTINEL".to_string(),
    )]);
    let append_failure_marker = "supervisor_resident_test_diagnostic_batch2_failure";
    let append_failure_guard = force_supervisor_resident_diagnostic_batch2_failure();

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请保留这条已记录的自然对话。"),
        &config,
    )
    .expect("diagnostic write failure must preserve the existing incomplete outcome");

    assert_eq!(outcome.status, "message_recorded_supervisor_incomplete");
    assert_eq!(
        supervisor_resident_diagnostic_batch2_attempts(),
        1,
        "diagnostic append failure must not retry, reread, or rebase"
    );
    let message_id = resident_recorded_message_id_for_client(&state_path, None);
    assert_no_message_injection_or_reply(&state_path, &message_id);
    assert!(
        resident_delivery_diagnostics_for_message(&state_path, &message_id).is_empty(),
        "a failed best-effort append must not leave a partial diagnostic"
    );
    let canonical_text = fs::read_to_string(&state_path).expect("read canonical workflow state");
    assert!(
        !canonical_text.contains(append_failure_marker),
        "diagnostic append failure detail must not enter canonical JSON"
    );
    assert_eq!(
        runner.plans.lock().expect("mock plans").len(),
        1,
        "diagnostic append failure must not rerun consult"
    );
    drop(append_failure_guard);
    fixture_cleanup(&root);
}

#[test]
fn s1b_h2_r4e_tool_diagnostic_batch2_failure_preserves_recorded_injected_reply_and_human_result() {
    let (state_path, workflow_id, root) = resident_fixture("r4e-tool-diagnostic-append-failure");
    let manager = resident_fixture_manager(&root);
    let config = resident_fixture_config(&state_path);
    let chain_before: Value =
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"]
            .clone();
    let hook_config = config.clone();
    let hook = Arc::new(move || {
        let toolface = supervisor_orchestrator::list_tools(&hook_config);
        let submit_visible = toolface["tools"]
            .as_array()
            .is_some_and(|tools| tools.iter().any(|tool| tool["name"] == "submit_proposal"));
        submit_visible
            .then_some(())
            .ok_or_else(|| "resident tools/list lost submit_proposal".to_string())
    });
    let runner = MockOneShotRunner::new(vec![MockOneShotOutcome::Turn {
        thread_id: "thread-r4e-tool-diagnostic-failure".to_string(),
        content: "主管已继续自然对话。".to_string(),
    }])
    .with_after_bind(hook);
    let failure_guard = force_supervisor_resident_diagnostic_batch2_failure();

    let outcome = submit_supervisor_resident_answer_with_parts(
        &runner,
        &manager,
        &state_path,
        &resident_request(&workflow_id, "请继续这段已经建立的自然对话。"),
        &config,
    )
    .expect("best-effort R4E diagnostic failure must preserve the completed conversation");

    assert_eq!(outcome.status, "message_sent");
    assert_eq!(
        outcome.message,
        "用户消息已同 thread 注入；主管回复已写入项目对话。"
    );
    assert_eq!(runner.hook_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        supervisor_resident_diagnostic_batch2_attempts(),
        1,
        "failed R4E diagnostic append must not retry, rebase, or open another write path"
    );
    let message_id = resident_recorded_message_id_for_client(&state_path, None);
    let events = resident_canonical_events(&state_path).expect("canonical resident events");
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT
                    && event["message_id"] == message_id
            })
            .count(),
        1,
        "existing injected fact survives a diagnostic append failure"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT
                    && event["reply_to_message_id"] == message_id
            })
            .count(),
        1,
        "existing natural reply survives a diagnostic append failure"
    );
    assert!(
        events.iter().all(|event| {
            event["event_type"] != "supervisor_resident_tool_invocation_diagnostic_recorded"
        }),
        "a failed best-effort append must not leave a partial R4E diagnostic"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&state_path).expect("workflow state"))
            .expect("workflow JSON")["workflow_chain_runs"],
        chain_before,
        "R4E diagnostic failure must not advance the chain"
    );
    assert_eq!(
        crate::project_consultation_proposal_store::load_store(
            &state_path,
            crate::unix_timestamp_ms()
        )
        .expect("proposal store")
        .proposals
        .len(),
        0,
        "R4E tools/list diagnostic failure must not create a proposal card"
    );
    assert_eq!(
        runner.plans.lock().expect("mock plans").len(),
        1,
        "R4E diagnostic failure must not rerun the resident turn"
    );
    drop(failure_guard);
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

// ── N2R 可观测性改造 A9 ────────────────────────────────────────────────
// 病灶：首句失败时五个完全不同的分支对外只有三种签名，`:2950`（首句没接上）
// 与 `:2889`（重放找不到记录）逐字同形。下面锁住"每支有自己的名字，且两两不等"。
// 这些断言在加 `failure_family` 之前无法编译（字段不存在），红证据见 evidence。

#[test]
fn resident_failure_families_are_fixed_constants_and_pairwise_distinct() {
    use super::resident_failure_family as family;

    let branch_families = [
        family::REPLY_MISSING_AFTER_INJECTION,
        family::REPLAY_RECORDED_WITHOUT_INJECTION,
        family::INJECTED_EVENT_APPEND_FAILED,
        family::SUPERVISOR_REPLY_APPEND_FAILED,
    ];
    // 两两不等——这是"外部能不能分辨"的唯一依据，不能靠肉眼看常量表。
    for (left_index, left) in branch_families.iter().enumerate() {
        assert!(
            !left.trim().is_empty(),
            "family 不得为空串：index {left_index}"
        );
        for (right_index, right) in branch_families.iter().enumerate() {
            if left_index == right_index {
                continue;
            }
            assert_ne!(
                left, right,
                "family 必须两两不等：index {left_index} 与 {right_index} 相同"
            );
        }
    }
}

#[test]
fn resident_consult_failure_families_cover_every_stage_without_collision() {
    use super::{
        resident_consult_failure_family_for_test, resident_delivery_diagnostic_stages_for_test,
    };

    let stages = resident_delivery_diagnostic_stages_for_test();
    assert_eq!(stages.len(), 13, "诊断相位共 13 个，覆盖必须齐全");

    let families: Vec<&'static str> = stages
        .iter()
        .copied()
        .map(resident_consult_failure_family_for_test)
        .collect();

    for (left_index, left) in families.iter().enumerate() {
        assert!(
            left.starts_with("consult_failed_"),
            "consult 家族必须带稳定前缀：{left}"
        );
        for (right_index, right) in families.iter().enumerate() {
            if left_index == right_index {
                continue;
            }
            assert_ne!(
                left, right,
                "consult 家族必须两两不等：{left_index} 与 {right_index}"
            );
        }
    }

    // 与分支家族也不得撞名——否则外部仍然分不清是哪一类失败。
    use super::resident_failure_family as family;
    for branch in [
        family::REPLY_MISSING_AFTER_INJECTION,
        family::REPLAY_RECORDED_WITHOUT_INJECTION,
        family::INJECTED_EVENT_APPEND_FAILED,
        family::SUPERVISOR_REPLY_APPEND_FAILED,
    ] {
        assert!(
            !families.contains(&branch),
            "分支家族 {branch} 与 consult 家族撞名"
        );
    }
}

// ── A9b：补上 A9 的覆盖缺口 ───────────────────────────────────────────
// A9 只断言了 13 个**基础** consult 家族两两不等，没有覆盖 A7 追加
// `__diagnostic_append_failed` 后缀之后的另外 13 个。真机取证要靠这 26 个
// 取值区分「consult 失败」与「consult 失败且诊断自己也没写进去」，
// 少一半覆盖就等于这层区分没有被任何断言锁住。
#[test]
fn resident_consult_families_including_diagnostic_append_suffix_are_pairwise_distinct() {
    use super::{
        resident_consult_failure_family_for_test, resident_delivery_diagnostic_stages_for_test,
        RESIDENT_DIAGNOSTIC_APPEND_FAILED_SUFFIX,
    };

    let stages = resident_delivery_diagnostic_stages_for_test();
    assert_eq!(stages.len(), 13, "诊断相位共 13 个，覆盖必须齐全");

    // 后缀本身不得为空——否则带后缀与不带后缀的两组会逐字相同，
    // 「诊断写入自己失败了」这件事就再也无法从 family 上看出来。
    assert!(
        !RESIDENT_DIAGNOSTIC_APPEND_FAILED_SUFFIX.trim().is_empty(),
        "诊断追加失败后缀不得为空串"
    );

    let mut families: Vec<String> = Vec::with_capacity(26);
    for stage in stages {
        let base = resident_consult_failure_family_for_test(stage);
        families.push(base.to_string());
        families.push(format!("{base}{RESIDENT_DIAGNOSTIC_APPEND_FAILED_SUFFIX}"));
    }
    assert_eq!(families.len(), 26, "13 个相位 × 带/不带后缀 = 26 个取值");

    for (left_index, left) in families.iter().enumerate() {
        assert!(
            !left.trim().is_empty(),
            "family 不得为空串：index {left_index}"
        );
        for (right_index, right) in families.iter().enumerate() {
            if left_index == right_index {
                continue;
            }
            assert_ne!(
                left, right,
                "26 个 consult 家族必须两两不等：{left_index} 与 {right_index} 相同"
            );
        }
    }
}

// ── A8*：三处 `delivery_unknown` 的子 family ──────────────────────────
// 这三处返回的是 `Err(String)` 而不是 outcome，所以拿不到 `failure_family`
// 那条通道。能做的只有一件事：让这三个错误取值本身分得开，且仍以既有取值
// 为前缀（旧的 `starts_with` 消费方不受影响）。错误类型、返回形状、控制流
// 一律未动——这是 A8* 能落地的全部边界。
#[test]
fn resident_delivery_unknown_subfamilies_are_distinct_and_keep_the_legacy_prefix() {
    use super::resident_delivery_unknown_families_for_test;

    let [base, replay_lookup, replay_reply_outcome, record_recheck] =
        resident_delivery_unknown_families_for_test();

    // 既有取值仍是三个子 family 的前缀——旧消费方按前缀仍然命中。
    for subfamily in [replay_lookup, replay_reply_outcome, record_recheck] {
        assert!(
            subfamily.starts_with(base),
            "子 family 必须保留既有取值作前缀：{subfamily}"
        );
        assert_ne!(subfamily, base, "子 family 必须比既有取值更具体：{subfamily}");
    }

    // 三处两两不等——否则这次拆分等于没做。
    let subfamilies = [replay_lookup, replay_reply_outcome, record_recheck];
    for (left_index, left) in subfamilies.iter().enumerate() {
        for (right_index, right) in subfamilies.iter().enumerate() {
            if left_index == right_index {
                continue;
            }
            assert_ne!(
                left, right,
                "delivery_unknown 子 family 必须两两不等：{left_index} 与 {right_index}"
            );
        }
    }
}

#[test]
fn resident_outcome_omits_failure_family_when_absent() {
    use super::SupervisorResidentAnswerOutcome;

    // 可选性：成功路径不带该字段，序列化后 JSON 里根本没有这个键，
    // 既有消费者与既有存量记录形状不受影响。
    let outcome = SupervisorResidentAnswerOutcome {
        status: "message_sent".to_string(),
        reply_injected: true,
        thread_id: Some("thread:x".to_string()),
        supervisor_reply: Some("ok".to_string()),
        message: "用户消息已同 thread 注入；主管回复已写入项目对话。".to_string(),
        failure_family: None,
    };
    let json = serde_json::to_value(&outcome).expect("outcome 可序列化");
    assert!(
        json.get("failure_family").is_none(),
        "failure_family 为 None 时必须整键省略，实际={json}"
    );

    let with_family = SupervisorResidentAnswerOutcome {
        failure_family: Some("resident_reply_missing_after_injection".to_string()),
        ..outcome
    };
    let json = serde_json::to_value(&with_family).expect("outcome 可序列化");
    assert_eq!(
        json.get("failure_family").and_then(serde_json::Value::as_str),
        Some("resident_reply_missing_after_injection")
    );
}
