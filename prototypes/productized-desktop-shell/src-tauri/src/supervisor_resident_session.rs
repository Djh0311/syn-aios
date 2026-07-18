// P1-A project-scoped resident supervisor session.
//
// This is intentionally included by `supervisor_session_launcher.rs` so it
// shares the established temporary-home, process-registration and supervisor
// command-plan machinery without creating another configuration path.

use std::collections::BTreeSet;
use std::io::{BufRead, BufReader};
use std::process::ChildStdin;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

// ===== P1-A · project-scoped resident supervisor session =====================
//
// The old pilot below remains an action-proposal loop. P1-A adds the separate
// conversational host used by project consultation/planning: its only process
// substitution is `codex exec` -> `codex mcp-server`; the private home, auth
// symlink, MCP whitelist and process registry are all reused without alteration.

const SUPERVISOR_RESIDENT_PROTOCOL_VERSION: &str = "2025-06-18";
const SUPERVISOR_RESIDENT_TURN_TIMEOUT: Duration = Duration::from_secs(420);
const SUPERVISOR_RESIDENT_DEVELOPER_INSTRUCTIONS: &str = "你处于项目主管常驻会话。当前会话只能只读：不得修改文件、不得运行会改变项目状态的命令、不得扩大权限或绕过确认。请像正常对话一样回应用户；不要输出或要求固定 JSON 回合协议，也不要冒充用户。只有在你已形成完整终版方案时才调用私有 submit_proposal 工具落卡；纯文字方案、建议或问题都不会落卡、更不会推进工作流。批准卡之外不得启动链、派发 worker 或改变执行状态。";

type SupervisorResidentHostMap = BTreeMap<String, Arc<Mutex<SupervisorResidentHostSlot>>>;

// This lock protects only the project -> per-host lock map. A model turn never
// holds it, so independent project supervisors can make their own MCP calls in
// parallel while a single project's thread remains strictly ordered.
static SUPERVISOR_RESIDENT_HOSTS: OnceLock<Mutex<SupervisorResidentHostMap>> = OnceLock::new();
// A user reply is a one-shot, user-originated fact.  Serializing its durable
// check/write/injection sequence prevents concurrent duplicate commands from
// silently injecting the same answer twice.
// One fixed test project has one resident conversation.  Serializing the
// command that can open a question and the command that can answer it keeps a
// pre-issued question id from racing with a second opening turn.
static SUPERVISOR_RESIDENT_CONVERSATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SupervisorResidentTurn {
    pub(crate) thread_id: String,
    pub(crate) content: String,
}

struct SupervisorResidentHostSlot {
    // `None` is a short-lived creation placeholder. It prevents two first
    // messages for one project from spawning separate mcp-server hosts.
    host: Option<Box<dyn SupervisorResidentMcpHost>>,
    thread_id: String,
    generation: u64,
}

trait SupervisorResidentMcpHost: Send {
    fn pid(&self) -> u32;
    fn is_alive(&mut self) -> Result<bool, String>;
    fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<SupervisorResidentTurn, String>;
    fn terminate(&mut self);
}

trait SupervisorResidentMcpHostSpawner {
    fn spawn(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        workflow_state_path: &Path,
    ) -> Result<Box<dyn SupervisorResidentMcpHost>, String>;
}

struct CodexSupervisorResidentMcpHost {
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Receiver<Result<String, String>>,
    temporary_home: Arc<SupervisorTemporaryHome>,
    registration: Option<Box<dyn SupervisorProcessRegistration>>,
    next_request_id: u64,
}

impl CodexSupervisorResidentMcpHost {
    fn initialize_and_validate(&mut self) -> Result<(), String> {
        let (initialized, _) = self.request(
            "initialize",
            json!({
                "protocolVersion": SUPERVISOR_RESIDENT_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "syn-supervisor-resident-session",
                    "version": "p1-a"
                }
            }),
            false,
        )?;
        ensure_resident_rpc_success(&initialized)?;
        self.notify("notifications/initialized", json!({}))?;
        let (listed, _) = self.request("tools/list", json!({}), false)?;
        ensure_resident_rpc_success(&listed)?;
        let tool_names = listed
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .map(|tools| {
                tools
                    .iter()
                    .filter_map(|tool| tool.get("name").and_then(Value::as_str))
                    .collect::<BTreeSet<_>>()
            })
            .ok_or_else(|| "supervisor_resident_mcp_tools_list_invalid".to_string())?;
        for expected in ["codex", "codex-reply"] {
            if !tool_names.contains(expected) {
                return Err(format!(
                    "supervisor_resident_mcp_required_tool_missing:{expected}"
                ));
            }
        }
        Ok(())
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        let encoded = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
        .map_err(|error| format!("主管常驻 MCP 通知编码失败：{error}"))?;
        self.stdin
            .write_all(&encoded)
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("主管常驻 MCP 通知写入失败：{error}"))
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
        wait_for_task_complete: bool,
    ) -> Result<(Value, Option<Value>), String> {
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or_else(|| "supervisor_resident_mcp_request_id_exhausted".to_string())?;
        let encoded = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        }))
        .map_err(|error| format!("主管常驻 MCP 请求编码失败：{error}"))?;
        self.stdin
            .write_all(&encoded)
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("主管常驻 MCP 请求写入失败：{error}"))?;

        let started = Instant::now();
        let mut response = None;
        let mut terminal_event = None;
        loop {
            if response.is_some() && (!wait_for_task_complete || terminal_event.is_some()) {
                return Ok((response.expect("response checked above"), terminal_event));
            }
            let remaining = SUPERVISOR_RESIDENT_TURN_TIMEOUT
                .checked_sub(started.elapsed())
                .ok_or_else(|| format!("supervisor_resident_mcp_timeout:{method}"))?;
            let line = match self.stdout_lines.recv_timeout(remaining) {
                Ok(Ok(line)) => line,
                Ok(Err(error)) => return Err(error),
                Err(RecvTimeoutError::Timeout) => {
                    return Err(format!("supervisor_resident_mcp_timeout:{method}"));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("supervisor_resident_host_exited:stdout_closed".to_string());
                }
            };
            let message: Value = serde_json::from_str(&line).map_err(|error| {
                // A tier-1 event that cannot be parsed is not recoverable by guessing.
                format!("supervisor_resident_mcp_event_parse_failed:{error}")
            })?;
            if resident_value_as_u64(message.get("id")) == Some(request_id) {
                response = Some(message.clone());
            }
            if !wait_for_task_complete
                || message.get("method").and_then(Value::as_str) != Some("codex/event")
            {
                continue;
            }
            let event_request_id =
                resident_value_as_u64(message.pointer("/params/_meta/requestId"));
            if event_request_id != Some(request_id) {
                continue;
            }
            let event_type = message.pointer("/params/msg/type").and_then(Value::as_str);
            match event_type {
                Some("task_complete") => terminal_event = Some(message),
                Some("error") => {
                    let detail = message
                        .pointer("/params/msg/message")
                        .and_then(Value::as_str)
                        .unwrap_or("未提供原因");
                    return Err(format!("supervisor_resident_mcp_terminal_error:{detail}"));
                }
                _ => {}
            }
        }
    }

    fn cleanup_after_exit(&mut self, trigger: &str) {
        if let Some(registration) = self.registration.take() {
            registration.unregister();
        }
        let _ = self.temporary_home.cleanup(trigger);
    }
}

impl SupervisorResidentMcpHost for CodexSupervisorResidentMcpHost {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn is_alive(&mut self) -> Result<bool, String> {
        match self
            .child
            .try_wait()
            .map_err(|error| format!("supervisor_resident_host_process_check_failed:{error}"))?
        {
            Some(_) => {
                self.cleanup_after_exit("resident_process_exited");
                Ok(false)
            }
            None => Ok(true),
        }
    }

    fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<SupervisorResidentTurn, String> {
        let (response, terminal_event) = self.request(
            "tools/call",
            json!({"name": tool_name, "arguments": arguments}),
            true,
        )?;
        ensure_resident_rpc_success(&response)?;
        let thread_id = resident_find_string(&response, &["threadId", "thread_id"])
            .ok_or_else(|| "supervisor_resident_mcp_missing_thread_id".to_string())?;
        let content = resident_response_content(&response)
            .or_else(|| {
                terminal_event
                    .as_ref()
                    .and_then(|event| event.pointer("/params/msg/last_agent_message"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| "supervisor_resident_mcp_empty_content".to_string())?;
        Ok(SupervisorResidentTurn { thread_id, content })
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        self.cleanup_after_exit("resident_process_terminated");
    }
}

impl Drop for CodexSupervisorResidentMcpHost {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct RealSupervisorResidentMcpHostSpawner;

impl SupervisorResidentMcpHostSpawner for RealSupervisorResidentMcpHostSpawner {
    fn spawn(
        &self,
        plan: &SupervisorCommandPlan,
        config: &McpServerConfig,
        workflow_state_path: &Path,
    ) -> Result<Box<dyn SupervisorResidentMcpHost>, String> {
        let temporary_home = SupervisorTemporaryHome::create(plan, config)?;
        let output_dir = plan
            .stderr_path
            .parent()
            .ok_or_else(|| "主管常驻 stderr 路径缺父目录，拒绝发射".to_string())?;
        fs::create_dir_all(output_dir)
            .map_err(|error| format!("创建主管常驻 stderr 目录失败：{error}"))?;
        restrict_private_dir(output_dir)?;
        let stderr_file = fs::File::create(&plan.stderr_path)
            .map_err(|error| format!("创建主管常驻 stderr 尸检文件失败：{error}"))?;
        restrict_private_file(&plan.stderr_path)?;
        let mut child = match Command::new(&plan.program)
            .args(&plan.argv)
            .current_dir(&plan.current_dir)
            .env("CODEX_HOME", temporary_home.root())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(stderr_file))
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                let _ = temporary_home.cleanup("resident_process_spawn_failed");
                return Err(format!("启动主管 codex mcp-server 失败：{error}"));
            }
        };
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = temporary_home.cleanup("resident_process_stdin_missing");
                return Err("主管常驻 codex stdin 不可用".to_string());
            }
        };
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = temporary_home.cleanup("resident_process_stdout_missing");
                return Err("主管常驻 codex stdout 不可用".to_string());
            }
        };
        let (line_sender, stdout_lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let next = line.map_err(|error| {
                    format!("supervisor_resident_host_exited:stdout_read_failed:{error}")
                });
                if line_sender.send(next).is_err() {
                    break;
                }
            }
        });
        let registration: Box<dyn SupervisorProcessRegistration> =
            match crate::exec_process_registry::register_supervisor_resident_process(
                workflow_state_path,
                &config.run_id,
                child.id(),
            ) {
                Ok(registration) => Box::new(WorkbenchSupervisorProcessRegistration {
                    registration: Some(registration),
                }),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = temporary_home.cleanup("resident_process_registration_failed");
                    return Err(format!(
                        "supervisor_resident_process_registration_failed:{error}"
                    ));
                }
            };
        let mut host = CodexSupervisorResidentMcpHost {
            child,
            stdin,
            stdout_lines,
            temporary_home,
            registration: Some(registration),
            next_request_id: 1,
        };
        if let Err(error) = host.initialize_and_validate() {
            host.terminate();
            return Err(error);
        }
        Ok(Box::new(host))
    }
}

fn ensure_resident_rpc_success(response: &Value) -> Result<(), String> {
    if let Some(error) = response.get("error") {
        return Err(format!("supervisor_resident_mcp_rpc_error:{error}"));
    }
    if response.pointer("/result/isError").and_then(Value::as_bool) == Some(true) {
        return Err(format!(
            "supervisor_resident_mcp_tool_error:{}",
            resident_response_content(response).unwrap_or_else(|| "未提供原因".to_string())
        ));
    }
    Ok(())
}

fn resident_value_as_u64(value: Option<&Value>) -> Option<u64> {
    value.and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|value| value.parse::<u64>().ok()))
    })
}

fn resident_find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(values) => {
            for key in keys {
                if let Some(value) = values.get(*key).and_then(Value::as_str) {
                    return Some(value.to_string());
                }
            }
            values
                .values()
                .find_map(|value| resident_find_string(value, keys))
        }
        Value::Array(values) => values
            .iter()
            .find_map(|value| resident_find_string(value, keys)),
        _ => None,
    }
}

fn resident_response_content(response: &Value) -> Option<String> {
    response
        .pointer("/result/structuredContent/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            response
                .pointer("/result/content")
                .and_then(Value::as_array)
                .and_then(|content| {
                    content.iter().find_map(|item| {
                        item.get("text").and_then(Value::as_str).map(str::to_string)
                    })
                })
        })
}

fn resident_hosts() -> &'static Mutex<SupervisorResidentHostMap> {
    SUPERVISOR_RESIDENT_HOSTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn resident_host_slot(
    hosts: &Mutex<SupervisorResidentHostMap>,
    host_key: &str,
) -> Result<(Arc<Mutex<SupervisorResidentHostSlot>>, bool), String> {
    let mut hosts = hosts
        .lock()
        .map_err(|_| "supervisor_resident_host_map_lock_poisoned".to_string())?;
    if let Some(slot) = hosts.get(host_key) {
        return Ok((slot.clone(), false));
    }
    let slot = Arc::new(Mutex::new(SupervisorResidentHostSlot {
        host: None,
        thread_id: String::new(),
        generation: 0,
    }));
    hosts.insert(host_key.to_string(), slot.clone());
    Ok((slot, true))
}

fn remove_resident_host_slot(
    hosts: &Mutex<SupervisorResidentHostMap>,
    host_key: &str,
    expected_slot: &Arc<Mutex<SupervisorResidentHostSlot>>,
) -> Result<(), String> {
    let mut hosts = hosts
        .lock()
        .map_err(|_| "supervisor_resident_host_map_lock_poisoned".to_string())?;
    if hosts
        .get(host_key)
        .is_some_and(|current| Arc::ptr_eq(current, expected_slot))
    {
        hosts.remove(host_key);
    }
    Ok(())
}

pub(crate) fn consult_supervisor_resident(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<String, String> {
    Ok(consult_supervisor_resident_turn(
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
    )?
    .content)
}

pub(crate) fn consult_supervisor_resident_turn(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<SupervisorResidentTurn, String> {
    // `user_message` is an internal transport kind.  Public callers must go
    // through submit_supervisor_resident_answer so the original user text is
    // canonicalized before any host call or rebuild can happen.
    if prompt_kind == "user_message" {
        return Err("supervisor_resident_user_message_requires_answer_command".to_string());
    }
    reap_supervisor_temporary_homes_once()?;
    consult_supervisor_resident_with(
        &RealSupervisorResidentMcpHostSpawner,
        resident_hosts(),
        workflow_state_path,
        project_root,
        workflow_id,
        prompt,
        prompt_kind,
    )
}

fn consult_supervisor_resident_with(
    spawner: &dyn SupervisorResidentMcpHostSpawner,
    hosts: &Mutex<SupervisorResidentHostMap>,
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<SupervisorResidentTurn, String> {
    validate_resident_request(project_root, prompt, prompt_kind)?;
    let run_id = resident_run_id(project_root);
    let config = supervisor_config(
        workflow_state_path,
        &run_id,
        SupervisorQuotaLimits {
            max_active_workers: DEFAULT_MAX_ACTIVE_WORKERS,
            max_follow_ups_per_worker: DEFAULT_MAX_FOLLOW_UPS_PER_WORKER,
            max_runtime_minutes: DEFAULT_MAX_RUNTIME_MINUTES,
        },
    );
    let host_key = resident_host_key(workflow_state_path, project_root);
    let persisted = supervisor_orchestrator::load_resident_session(&config)?;

    // A previous desktop process cannot safely reattach its stdio pipes. Start a
    // new host and rebuild its facts instead of pretending the old transcript is live.
    let mut replacement_reason = persisted
        .is_some()
        .then(|| "host_not_resident_in_current_process".to_string());

    loop {
        let (slot_handle, created) = resident_host_slot(hosts, &host_key)?;
        let mut slot = slot_handle
            .lock()
            .map_err(|_| "supervisor_resident_host_slot_lock_poisoned".to_string())?;

        if created {
            let generation = persisted
                .as_ref()
                .map(|state| state.generation.saturating_add(1))
                .unwrap_or(1);
            let executable = match resident_workbench_executable() {
                Ok(executable) => executable,
                Err(error) => {
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err(error);
                }
            };
            let plan = match build_supervisor_resident_command_plan(
                project_root,
                workflow_state_path,
                &run_id,
                generation,
                &executable,
            ) {
                Ok(plan) => plan,
                Err(error) => {
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err(error);
                }
            };
            let facts = match resident_rebuild_facts(workflow_state_path, project_root, workflow_id)
            {
                Ok(facts) => facts,
                Err(error) => {
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err(error);
                }
            };
            let opening_prompt =
                format!("{facts}\n\n===== 当前项目主管请求（{prompt_kind}）=====\n{prompt}");
            let mut host = match spawner.spawn(&plan, &config, workflow_state_path) {
                Ok(host) => host,
                Err(error) => {
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err(error);
                }
            };
            let turn = match host.call_tool(
                "codex",
                &resident_initial_tool_arguments(project_root, &opening_prompt),
            ) {
                Ok(turn)
                    if !turn.thread_id.trim().is_empty() && !turn.content.trim().is_empty() =>
                {
                    turn
                }
                Ok(_) => {
                    host.terminate();
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err("supervisor_resident_mcp_thread_or_content_invalid".to_string());
                }
                Err(error) => {
                    host.terminate();
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    return Err(error);
                }
            };
            let host_pid = host.pid();
            let record_result = match replacement_reason.as_deref() {
                Some(reason) => supervisor_orchestrator::record_resident_session_replaced(
                    &config,
                    project_root,
                    workflow_id,
                    &turn.thread_id,
                    host_pid,
                    generation,
                    reason,
                ),
                None => supervisor_orchestrator::record_resident_session_created(
                    &config,
                    project_root,
                    workflow_id,
                    &turn.thread_id,
                    host_pid,
                    generation,
                ),
            };
            if let Err(error) = record_result.and_then(|_| {
                supervisor_orchestrator::record_resident_consult_merged(
                    &config,
                    project_root,
                    workflow_id,
                    prompt_kind,
                )
            }) {
                host.terminate();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                return Err(format!("supervisor_resident_audit_failed:{error}"));
            }
            slot.host = Some(host);
            slot.thread_id = turn.thread_id.clone();
            slot.generation = generation;
            return Ok(turn);
        }

        let Some(host) = slot.host.as_mut() else {
            // The creator failed before publishing a host. Do not guess at any
            // half-read event stream; remove only this slot and let this request
            // establish a clean generation.
            drop(slot);
            remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
            replacement_reason = Some("host_initialization_unavailable".to_string());
            continue;
        };
        let alive = match host.is_alive() {
            Ok(alive) => alive,
            Err(error) => {
                let stale = slot.host.take();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                if let Some(mut stale) = stale {
                    stale.terminate();
                }
                return Err(error);
            }
        };
        if !alive {
            let stale = slot.host.take();
            drop(slot);
            remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
            if let Some(mut stale) = stale {
                stale.terminate();
            }
            replacement_reason = Some("host_process_exited".to_string());
            continue;
        }
        let expected_thread_id = slot.thread_id.clone();
        let turn = slot.host.as_mut().expect("host checked above").call_tool(
            "codex-reply",
            &json!({"threadId": expected_thread_id, "prompt": prompt}),
        );
        match turn {
            Ok(turn) if turn.thread_id == slot.thread_id && !turn.content.trim().is_empty() => {
                let host_pid = slot.host.as_ref().expect("host checked above").pid();
                let generation = slot.generation;
                if let Err(error) = supervisor_orchestrator::record_resident_session_reused(
                    &config,
                    project_root,
                    workflow_id,
                    &turn.thread_id,
                    host_pid,
                    generation,
                )
                .and_then(|_| {
                    supervisor_orchestrator::record_resident_consult_merged(
                        &config,
                        project_root,
                        workflow_id,
                        prompt_kind,
                    )
                }) {
                    let stale = slot.host.take();
                    drop(slot);
                    remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                    if let Some(mut stale) = stale {
                        stale.terminate();
                    }
                    return Err(format!("supervisor_resident_audit_failed:{error}"));
                }
                return Ok(turn);
            }
            Ok(_) => {
                let stale = slot.host.take();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                if let Some(mut stale) = stale {
                    stale.terminate();
                }
                return Err("supervisor_resident_mcp_thread_or_content_invalid".to_string());
            }
            Err(error) if resident_thread_invalid(&error) => {
                let stale = slot.host.take();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                if let Some(mut stale) = stale {
                    stale.terminate();
                }
                replacement_reason = Some("thread_invalid".to_string());
            }
            Err(error) if resident_host_exited(&error) => {
                let stale = slot.host.take();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                if let Some(mut stale) = stale {
                    stale.terminate();
                }
                replacement_reason = Some("host_exited_during_reply".to_string());
            }
            // Event parse errors, timeouts and all other protocol uncertainty are
            // a conservative stop: retire this host and return the original error.
            Err(error) => {
                let stale = slot.host.take();
                drop(slot);
                remove_resident_host_slot(hosts, &host_key, &slot_handle)?;
                if let Some(mut stale) = stale {
                    stale.terminate();
                }
                return Err(error);
            }
        }
    }
}

fn validate_resident_request(
    project_root: &str,
    prompt: &str,
    prompt_kind: &str,
) -> Result<(), String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    if prompt.trim().is_empty() {
        return Err("supervisor_resident_prompt_empty".to_string());
    }
    if !matches!(
        prompt_kind,
        "project_consult" | "director_plan" | "director_plan_preview" | "user_message"
    ) {
        return Err("supervisor_resident_prompt_kind_not_allowed".to_string());
    }
    Ok(())
}

const SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT: &str =
    "supervisor_resident_user_message_recorded";
const SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT: &str =
    "supervisor_resident_user_message_injected";
const SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT: &str =
    "supervisor_resident_supervisor_message_recorded";

#[derive(serde::Deserialize)]
pub(crate) struct SubmitSupervisorResidentAnswerRequest {
    pub(crate) project_id: String,
    pub(crate) workflow_id: String,
    // Historical command name is retained for the public entrypoint, but this
    // is now one ordinary user message—not an answer to a generated question.
    pub(crate) message_text: String,
}

#[derive(serde::Serialize)]
pub(crate) struct SupervisorResidentAnswerOutcome {
    pub(crate) status: String,
    pub(crate) reply_injected: bool,
    pub(crate) thread_id: Option<String>,
    pub(crate) supervisor_reply: Option<String>,
    pub(crate) message: String,
}

pub(crate) fn resident_conversation_lock() -> &'static Mutex<()> {
    SUPERVISOR_RESIDENT_CONVERSATION_LOCK.get_or_init(|| Mutex::new(()))
}

fn resident_event_string<'a>(event: &'a Value, field: &str) -> Option<&'a str> {
    event.get(field).and_then(Value::as_str)
}

fn require_resident_workflow_binding(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<String, String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    if workflow_id.trim().is_empty() {
        return Err("supervisor_resident_workflow_id_required".to_string());
    }
    let project_id = crate::project_id(project_root);
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;
    if snapshot.project_workflows.iter().any(|workflow| {
        workflow.project_id == project_id
            && workflow.project_root == project_root
            && workflow.workflow_id == workflow_id
    }) {
        Ok(project_id)
    } else {
        Err("supervisor_resident_project_workflow_binding_not_found".to_string())
    }
}


fn resident_message_target_ref(workflow_id: &str, message_id: &str) -> String {
    format!("{workflow_id}:resident-message:{message_id}")
}

fn append_resident_message_canonical_event(
    workflow_state_path: &Path,
    event: Value,
    phase: &str,
) -> Result<(), String> {
    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    crate::array_mut(&mut value, "audit_events")?.push(event);
    crate::write_m5b_batch2_workflow_state(workflow_state_path, phase, &value)
}

fn append_resident_user_message_recorded(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    message_id: &str,
    message_text: &str,
) -> Result<(), String> {
    let created_at = crate::unix_timestamp_string();
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:user:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "message_text": message_text,
            "actor_ref": "user",
            "source_kind": "supervisor_resident_user_message",
            "permission_level": "read_only_conversation",
            "created_at": created_at,
            // The generic read model turns the canonical reason into the
            // blackboard summary; keep it exactly user-originated.
            "reason": message_text,
        }),
        "supervisor_resident_user_message_recorded",
    )
}

fn append_resident_user_message_injected(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    message_id: &str,
    thread_id: &str,
) -> Result<(), String> {
    let created_at = crate::unix_timestamp_string();
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:injected:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "thread_id": thread_id,
            "actor_ref": "supervisor_resident",
            "source_kind": "supervisor_resident_user_message_injection",
            "permission_level": "read_only_conversation",
            "created_at": created_at,
            "reason": "用户消息已通过同一常驻主管会话注入；未伪造用户输入。",
        }),
        "supervisor_resident_user_message_injected",
    )
}

fn append_resident_supervisor_message_recorded(
    workflow_state_path: &Path,
    project_id: &str,
    workflow_id: &str,
    reply_to_message_id: &str,
    thread_id: &str,
    message_text: &str,
) -> Result<(), String> {
    let created_at = crate::unix_timestamp_string();
    let message_id = format!("supervisor:{}", crate::unix_timestamp_nanos());
    append_resident_message_canonical_event(
        workflow_state_path,
        json!({
            "event_id": format!("supervisor-resident-message:supervisor:{}", crate::unix_timestamp_nanos()),
            "event_type": SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT,
            "target_ref": resident_message_target_ref(workflow_id, &message_id),
            "project_id": project_id,
            "workflow_id": workflow_id,
            "message_id": message_id,
            "reply_to_message_id": reply_to_message_id,
            "thread_id": thread_id,
            "message_text": message_text,
            "actor_ref": "supervisor_resident",
            "source_kind": "supervisor_resident_supervisor_message",
            "permission_level": "read_only_conversation",
            "created_at": created_at,
            "reason": message_text,
        }),
        "supervisor_resident_supervisor_message_recorded",
    )
}

fn resident_user_message_prompt(message_text: &str) -> String {
    format!(
        "===== 用户通过工作台提交的原文（唯一用户输入）=====\n{message_text}\n===== 用户原文结束 =====\n\n请像正常对话一样直接继续同一 thread。可以提问、澄清、讨论或给建议；不要把自己的文字当作用户决定，也不要生成固定 JSON 回合。只有完整终版方案才调用 submit_proposal；工具成功前不要声称右侧已有方案卡。聊天本身绝不批准、起链或派发 worker。"
    )
}

fn resolve_resident_message_scope(
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<String, String> {
    if request.project_id.trim().is_empty() || request.workflow_id.trim().is_empty() {
        return Err("supervisor_resident_message_identity_incomplete".to_string());
    }
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    let expected_project_id =
        require_resident_workflow_binding(workflow_state_path, project_root, &request.workflow_id)?;
    if request.project_id != expected_project_id {
        return Err("supervisor_resident_message_project_id_mismatch".to_string());
    }
    Ok(project_root.to_string())
}

fn submit_supervisor_resident_answer_with(
    spawner: &dyn SupervisorResidentMcpHostSpawner,
    hosts: &Mutex<SupervisorResidentHostMap>,
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    if request.message_text.trim().is_empty() {
        return Err("supervisor_resident_message_text_empty".to_string());
    }
    let project_root = resolve_resident_message_scope(workflow_state_path, request)?;
    let message_id = format!("user:{}", crate::unix_timestamp_nanos());
    // Persist before the MCP call.  A host replacement rebuilds from the
    // canonical blackboard, so a failed/expired host cannot erase user speech.
    append_resident_user_message_recorded(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &request.message_text,
    )?;
    let turn = consult_supervisor_resident_with(
        spawner,
        hosts,
        workflow_state_path,
        &project_root,
        &request.workflow_id,
        &resident_user_message_prompt(&request.message_text),
        "user_message",
    )?;
    append_resident_user_message_injected(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &turn.thread_id,
    )?;
    append_resident_supervisor_message_recorded(
        workflow_state_path,
        &request.project_id,
        &request.workflow_id,
        &message_id,
        &turn.thread_id,
        &turn.content,
    )?;
    Ok(SupervisorResidentAnswerOutcome {
        status: "message_sent".to_string(),
        reply_injected: true,
        thread_id: Some(turn.thread_id),
        supervisor_reply: Some(turn.content),
        message: "用户消息已同 thread 注入；主管回复已写入项目对话。".to_string(),
    })
}

pub(crate) fn supervisor_resident_latest_user_message_snapshot(
    config: &McpServerConfig,
    workflow_id: &str,
) -> Result<String, String> {
    let session = supervisor_orchestrator::load_resident_session(config)?
        .ok_or_else(|| "supervisor_resident_session_binding_missing".to_string())?;
    if session.project_id.trim().is_empty()
        || session.project_root.trim().is_empty()
        || session.workflow_id.trim().is_empty()
        || session.thread_id.trim().is_empty()
        || session.workflow_id != workflow_id
        || session.project_id != crate::project_id(&session.project_root)
    {
        return Err("supervisor_resident_session_binding_invalid".to_string());
    }
    let workflow_state_path = config
        .supervisor_workflow_state_path
        .as_deref()
        .ok_or_else(|| "supervisor_resident_workflow_state_path_missing".to_string())?;
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    value
        .get("audit_events")
        .and_then(Value::as_array)
        .ok_or_else(|| "supervisor_resident_audit_events_missing".to_string())?
        .iter()
        .rev()
        .find_map(|event| {
            (resident_event_string(event, "event_type")
                == Some(SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT)
                && resident_event_string(event, "project_id") == Some(session.project_id.as_str())
                && resident_event_string(event, "workflow_id") == Some(workflow_id)
                && resident_event_string(event, "actor_ref") == Some("user")
                && resident_event_string(event, "source_kind")
                    == Some("supervisor_resident_user_message"))
                .then(|| resident_event_string(event, "message_text"))
                .flatten()
                .filter(|message| !message.trim().is_empty())
                .map(str::to_string)
        })
        .ok_or_else(|| "supervisor_resident_latest_user_message_missing".to_string())
}

pub(crate) fn submit_supervisor_resident_message(
    workflow_state_path: &Path,
    request: &SubmitSupervisorResidentAnswerRequest,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    let _guard = resident_conversation_lock()
        .lock()
        .map_err(|_| "supervisor_resident_conversation_lock_poisoned".to_string())?;
    reap_supervisor_temporary_homes_once()?;
    submit_supervisor_resident_answer_with(
        &RealSupervisorResidentMcpHostSpawner,
        resident_hosts(),
        workflow_state_path,
        request,
    )
}

#[tauri::command]
pub(crate) async fn submit_supervisor_resident_answer(
    request: SubmitSupervisorResidentAnswerRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<SupervisorResidentAnswerOutcome, String> {
    let workflow_state_path = state.workflow_state_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        submit_supervisor_resident_message(&workflow_state_path, &request)
    })
    .await
    .map_err(|error| format!("主管用户消息执行线程异常：{error}"))?
}

fn resident_run_id(project_root: &str) -> String {
    format!(
        "supervisor-resident:{}",
        crate::stable_id(&crate::project_id(project_root))
    )
}

fn resident_host_key(workflow_state_path: &Path, project_root: &str) -> String {
    format!(
        "{}:{}",
        workflow_state_path.display(),
        crate::project_id(project_root)
    )
}

fn resident_thread_invalid(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    [
        "thread not found",
        "unknown thread",
        "invalid thread",
        "session not found",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn resident_host_exited(error: &str) -> bool {
    error.starts_with("supervisor_resident_host_exited:")
}

fn resident_workbench_executable() -> Result<PathBuf, String> {
    // Unit tests normally use the test harness as `current_exe`. The ignored
    // P1-A live proof supplies the already-built desktop binary on its command
    // line, so it can exercise the same private-home MCP path without adding a
    // product-facing launcher or touching any real-home configuration.
    #[cfg(test)]
    if let Some(executable) = std::env::var_os("SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE") {
        let executable = PathBuf::from(executable);
        if executable.is_file() {
            return Ok(executable);
        }
        return Err(format!(
            "P1-A 真跑工作台可执行文件不可用：{}",
            executable.display()
        ));
    }
    std::env::current_exe().map_err(|error| format!("定位工作台 MCP 可执行文件失败：{error}"))
}

fn resident_initial_tool_arguments(project_root: &str, prompt: &str) -> Value {
    json!({
        "prompt": prompt,
        "cwd": project_root,
        "sandbox": "read-only",
        "approval-policy": "untrusted",
        "developer-instructions": SUPERVISOR_RESIDENT_DEVELOPER_INSTRUCTIONS,
        "config": {
            "model_reasoning_effort": DEFAULT_REASONING_EFFORT,
            "features": {"multi_agent": false}
        }
    })
}

fn resident_rebuild_facts(
    workflow_state_path: &Path,
    project_root: &str,
    workflow_id: &str,
) -> Result<String, String> {
    let target_project_id = crate::project_id(project_root);
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;
    let mut blackboard_facts = snapshot
        .project_blackboards
        .iter()
        .filter(|blackboard| {
            blackboard.project_id == target_project_id
                && (workflow_id.trim().is_empty() || blackboard.workflow_id == workflow_id)
        })
        .flat_map(|blackboard| blackboard.entries.iter())
        .filter(|entry| !entry.summary.trim().is_empty())
        .map(|entry| {
            let source_refs = entry
                .source_refs
                .iter()
                .map(|source| source.label.as_str())
                .filter(|label| !label.trim().is_empty())
                .collect::<Vec<_>>();
            let refs = if source_refs.is_empty() {
                String::new()
            } else {
                format!("（来源：{}）", source_refs.join("、"))
            };
            format!(
                "- [{}] {}：{}{}",
                entry.status, entry.title, entry.summary, refs
            )
        })
        .collect::<Vec<_>>();
    blackboard_facts.sort();
    let blackboard = if blackboard_facts.is_empty() {
        "（当前没有可注入的项目黑板既有条目。）".to_string()
    } else {
        blackboard_facts.join("\n")
    };
    // Reuse the existing formal-memory top5 renderer. It is intentionally not a
    // transcript and preserves its established active/project/update ordering.
    let memory = crate::recall_project_memory_summary_at(workflow_state_path, project_root)
        .unwrap_or_else(|| "（当前没有活跃正式记忆。）".to_string());
    Ok(format!(
        "===== 换代/首轮核心事实（不是聊天记录）=====\n项目黑板既有条目：\n{blackboard}\n\n正式记忆 top5：\n{memory}\n\n以上事实由工作台注入；若与未核实推测冲突，以这些事实和本轮已注入材料为准。"
    ))
}

// P1-A uses the same private-home `SupervisorCommandPlan` shape as the pilot.
// The only executable lifecycle change is `codex exec` -> `codex mcp-server`.
fn build_supervisor_resident_command_plan(
    project_root: &str,
    workflow_state_path: &Path,
    run_id: &str,
    generation: u64,
    workbench_executable: &Path,
) -> Result<SupervisorCommandPlan, String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT {
        return Err(crate::legacy_product_command_blocked_message(
            "supervisor_resident_session",
        ));
    }
    let quota_limits = SupervisorQuotaLimits {
        max_active_workers: DEFAULT_MAX_ACTIVE_WORKERS,
        max_follow_ups_per_worker: DEFAULT_MAX_FOLLOW_UPS_PER_WORKER,
        max_runtime_minutes: DEFAULT_MAX_RUNTIME_MINUTES,
    };
    let mcp_args = vec![
        "__mcp_server".to_string(),
        "--role".to_string(),
        "supervisor_orchestrator".to_string(),
        "--run-id".to_string(),
        run_id.to_string(),
        "--workflow-state-path".to_string(),
        workflow_state_path.display().to_string(),
        "--max-active-workers".to_string(),
        quota_limits.max_active_workers.to_string(),
        "--max-follow-ups-per-worker".to_string(),
        quota_limits.max_follow_ups_per_worker.to_string(),
        "--max-runtime-minutes".to_string(),
        quota_limits.max_runtime_minutes.to_string(),
    ];
    let artifact_run_id = format!("{run_id}:generation-{generation}");
    let (last_message_path, stderr_path) =
        supervisor_output_paths(workflow_state_path, &artifact_run_id)?;
    let plan = SupervisorCommandPlan {
        program: "codex".to_string(),
        argv: vec!["mcp-server".to_string()],
        current_dir: crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
        last_message_path,
        stderr_path,
        supervisor_mcp_command: workbench_executable.to_path_buf(),
        supervisor_mcp_args: mcp_args,
    };
    validate_supervisor_resident_command_plan(&plan, project_root)?;
    Ok(plan)
}

fn validate_supervisor_resident_command_plan(
    plan: &SupervisorCommandPlan,
    project_root: &str,
) -> Result<(), String> {
    if project_root != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        || plan.current_dir != crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT
    {
        return Err("主管常驻会话 cwd 必须锁死固定测试项目根".to_string());
    }
    if plan.argv.as_slice() != ["mcp-server"] {
        return Err("主管常驻会话只能启动 codex mcp-server".to_string());
    }
    if !plan
        .supervisor_mcp_args
        .windows(2)
        .any(|pair| pair[0] == "--role" && pair[1] == "supervisor_orchestrator")
    {
        return Err("主管常驻临时 home 只能挂 supervisor_orchestrator".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod resident_session_tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    include!("supervisor_resident_freeform_tests.rs");
}
