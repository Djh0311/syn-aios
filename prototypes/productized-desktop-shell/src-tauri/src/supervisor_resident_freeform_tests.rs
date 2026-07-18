use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct ResidentMockCall {
    tool_name: String,
    arguments: Value,
}

struct ResidentMockHostSpec {
    pid: u32,
    alive: Arc<std::sync::atomic::AtomicBool>,
    turns: VecDeque<Result<SupervisorResidentTurn, String>>,
}

struct ResidentMockHost {
    pid: u32,
    alive: Arc<std::sync::atomic::AtomicBool>,
    turns: VecDeque<Result<SupervisorResidentTurn, String>>,
    calls: Arc<Mutex<Vec<ResidentMockCall>>>,
    terminated: Arc<std::sync::atomic::AtomicUsize>,
}

impl SupervisorResidentMcpHost for ResidentMockHost {
    fn pid(&self) -> u32 {
        self.pid
    }

    fn is_alive(&mut self) -> Result<bool, String> {
        Ok(self.alive.load(std::sync::atomic::Ordering::SeqCst))
    }

    fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<SupervisorResidentTurn, String> {
        self.calls
            .lock()
            .expect("resident mock calls")
            .push(ResidentMockCall {
                tool_name: tool_name.to_string(),
                arguments: arguments.clone(),
            });
        self.turns
            .pop_front()
            .unwrap_or_else(|| Err("resident_mock_missing_scripted_turn".to_string()))
    }

    fn terminate(&mut self) {
        self.alive.store(false, std::sync::atomic::Ordering::SeqCst);
        self.terminated
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

struct ResidentMockSpawner {
    specs: Mutex<VecDeque<ResidentMockHostSpec>>,
    calls: Arc<Mutex<Vec<ResidentMockCall>>>,
    plans: Arc<Mutex<Vec<SupervisorCommandPlan>>>,
    terminated: Arc<std::sync::atomic::AtomicUsize>,
}

impl ResidentMockSpawner {
    fn new(specs: Vec<ResidentMockHostSpec>) -> Self {
        Self {
            specs: Mutex::new(specs.into_iter().collect()),
            calls: Arc::new(Mutex::new(Vec::new())),
            plans: Arc::new(Mutex::new(Vec::new())),
            terminated: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

impl SupervisorResidentMcpHostSpawner for ResidentMockSpawner {
    fn spawn(
        &self,
        plan: &SupervisorCommandPlan,
        _config: &McpServerConfig,
        _workflow_state_path: &Path,
    ) -> Result<Box<dyn SupervisorResidentMcpHost>, String> {
        self.plans
            .lock()
            .expect("resident mock plans")
            .push(plan.clone());
        let spec = self
            .specs
            .lock()
            .expect("resident mock specs")
            .pop_front()
            .ok_or_else(|| "resident_mock_missing_host".to_string())?;
        Ok(Box::new(ResidentMockHost {
            pid: spec.pid,
            alive: spec.alive,
            turns: spec.turns,
            calls: self.calls.clone(),
            terminated: self.terminated.clone(),
        }))
    }
}

fn resident_mock_turn(thread_id: &str, content: &str) -> Result<SupervisorResidentTurn, String> {
    Ok(SupervisorResidentTurn {
        thread_id: thread_id.to_string(),
        content: content.to_string(),
    })
}

fn resident_mock_spec(
    pid: u32,
    alive: Arc<std::sync::atomic::AtomicBool>,
    turns: Vec<Result<SupervisorResidentTurn, String>>,
) -> ResidentMockHostSpec {
    ResidentMockHostSpec {
        pid,
        alive,
        turns: turns.into_iter().collect(),
    }
}

fn resident_fixture_state_path(label: &str) -> (PathBuf, String) {
    let root = std::env::temp_dir().join(format!(
        "s1-supervisor-resident-{label}-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir_all(&root).expect("create resident fixture root");
    let path = root.join("workflow-state.v0.json");
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    crate::bootstrap_project_workflow_at(
        &path,
        &crate::ProjectRecord {
            project_root: project_root.to_string(),
            name: "S1 freeform resident fixture".to_string(),
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
    (path, crate::default_workflow_id(project_root))
}

fn resident_fixture_cleanup(state_path: &Path) {
    if let Some(root) = state_path.parent() {
        let _ = fs::remove_dir_all(root);
    }
}

fn resident_message_request(
    workflow_id: &str,
    message_text: &str,
) -> SubmitSupervisorResidentAnswerRequest {
    SubmitSupervisorResidentAnswerRequest {
        project_id: crate::project_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        workflow_id: workflow_id.to_string(),
        message_text: message_text.to_string(),
    }
}

fn resident_canonical_events(state_path: &Path) -> Vec<Value> {
    crate::read_workflow_state_value(state_path)
        .expect("read resident canonical state")
        .get("audit_events")
        .and_then(Value::as_array)
        .expect("canonical audit events")
        .clone()
}

fn resident_audit_event_types(state_path: &Path) -> Vec<String> {
    let sidecar = crate::utils::store_paths::sidecar_path(
        state_path,
        "supervisor-orchestrator.v1.json",
        "主管编排",
    )
    .expect("resident supervisor sidecar path");
    let value: Value = serde_json::from_slice(&fs::read(sidecar).expect("resident sidecar"))
        .expect("resident sidecar JSON");
    value["audit_events"]
        .as_array()
        .expect("resident audit events")
        .iter()
        .filter_map(|event| event["event_type"].as_str().map(str::to_string))
        .collect()
}

fn resident_kill_live_host(
    hosts: &Mutex<SupervisorResidentHostMap>,
    state_path: &Path,
    project_root: &str,
) -> u32 {
    let host_key = resident_host_key(state_path, project_root);
    let slot = hosts
        .lock()
        .expect("live resident host map")
        .get(&host_key)
        .cloned()
        .expect("live resident host slot");
    let mut slot = slot.lock().expect("live resident host slot lock");
    let host = slot.host.as_mut().expect("live resident host");
    let pid = host.pid();
    host.terminate();
    pid
}

#[test]
fn s1_freeform_resident_keeps_private_mcp_shape_for_three_same_thread_messages() {
    let (state_path, workflow_id) = resident_fixture_state_path("private-mcp-shape");
    let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![resident_mock_spec(
        5001,
        alive,
        vec![
            resident_mock_turn("thread-s1-private", "首轮自由对话：记住 ALPHA。"),
            resident_mock_turn("thread-s1-private", "第二轮自由对话：ALPHA。"),
            resident_mock_turn("thread-s1-private", "第三轮自由对话：仍是 ALPHA。"),
        ],
    )]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "请记住 ALPHA。"),
    )
    .expect("first freeform resident message");
    let second = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "请引用刚才的标记。"),
    )
    .expect("second freeform resident message");
    let third = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "请再次确认上下文。"),
    )
    .expect("third freeform resident message");

    assert_eq!(first.thread_id.as_deref(), Some("thread-s1-private"));
    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(third.thread_id, first.thread_id);
    assert!(second
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("ALPHA")));
    assert!(third
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("ALPHA")));

    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply", "codex-reply"]
    );
    assert_eq!(
        calls[0].arguments["cwd"],
        Value::String(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string())
    );
    assert_eq!(
        calls[0].arguments["sandbox"],
        Value::String("read-only".to_string())
    );
    assert_eq!(
        calls[1].arguments["threadId"],
        Value::String("thread-s1-private".to_string())
    );
    assert!(calls[0].arguments["prompt"]
        .as_str()
        .expect("opening prompt")
        .contains("正式记忆 top5"));

    let plans = spawner.plans.lock().expect("resident plans");
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].argv, vec!["mcp-server".to_string()]);
    let config = supervisor_mcp_config_toml(&plans[0]).expect("private resident config");
    assert_eq!(config.matches("[mcp_servers.").count(), 1);
    assert!(config.contains("[mcp_servers.supervisor_orchestrator]"));
    assert!(!config.contains("mcp_servers.private"));

    let audit_types = resident_audit_event_types(&state_path);
    assert!(audit_types.contains(&"supervisor_resident_session_created".to_string()));
    assert_eq!(
        audit_types
            .iter()
            .filter(|event_type| *event_type == "supervisor_resident_session_reused")
            .count(),
        2
    );
    assert_eq!(
        audit_types
            .iter()
            .filter(|event_type| *event_type == "supervisor_resident_consult_merged")
            .count(),
        3
    );
    resident_fixture_cleanup(&state_path);
}

#[test]
fn s1_freeform_rebuilds_in_same_turn_after_thread_invalid() {
    let (state_path, workflow_id) = resident_fixture_state_path("thread-invalid");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            5051,
            first_alive,
            vec![
                resident_mock_turn("thread-s1-invalid-old", "首轮自由对话。"),
                Err("supervisor_resident_mcp_terminal_error:thread not found".to_string()),
            ],
        ),
        resident_mock_spec(
            5052,
            second_alive,
            vec![resident_mock_turn(
                "thread-s1-invalid-new",
                "线程失效后，已在本轮用核心事实重建。",
            )],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "先建立自由对话。"),
    )
    .expect("initial resident host");
    let rebuilt = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "本轮 thread 已失效，继续讨论。"),
    )
    .expect("thread-invalid replacement must finish the same user-message turn");

    assert_eq!(first.thread_id.as_deref(), Some("thread-s1-invalid-old"));
    assert_eq!(rebuilt.thread_id.as_deref(), Some("thread-s1-invalid-new"));
    assert_eq!(
        spawner
            .calls
            .lock()
            .expect("resident calls")
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply", "codex"]
    );
    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert!(calls[2].arguments["prompt"]
        .as_str()
        .expect("replacement opening prompt")
        .contains("换代/首轮核心事实（不是聊天记录）"));
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert!(resident_audit_event_types(&state_path)
        .contains(&"supervisor_resident_session_replaced".to_string()));
    resident_fixture_cleanup(&state_path);
}

#[test]
fn s1_freeform_parse_failure_retires_host_before_a_later_clean_generation() {
    let (state_path, workflow_id) = resident_fixture_state_path("parse-failure");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            5061,
            first_alive,
            vec![
                resident_mock_turn("thread-s1-parse-old", "首轮正常。"),
                Err("supervisor_resident_mcp_event_parse_failed:bad-json".to_string()),
            ],
        ),
        resident_mock_spec(
            5062,
            second_alive,
            vec![resident_mock_turn("thread-s1-parse-new", "后续独立新代。")],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "先建立会话。"),
    )
    .expect("initial resident host");
    let error = match submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "这轮会触发坏事件。"),
    ) {
        Err(error) => error,
        Ok(_) => panic!("parse error must conservatively stop"),
    };
    assert_eq!(error, "supervisor_resident_mcp_event_parse_failed:bad-json");
    assert!(hosts.lock().expect("host map").is_empty());
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    let later = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "下一轮才允许建立干净新代。"),
    )
    .expect("later clean generation");
    assert_eq!(later.thread_id.as_deref(), Some("thread-s1-parse-new"));
    assert_eq!(
        spawner
            .calls
            .lock()
            .expect("resident calls")
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply", "codex"]
    );
    resident_fixture_cleanup(&state_path);
}

#[test]
fn s1_freeform_project_slots_have_independent_locks() {
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let (left, left_created) = resident_host_slot(&hosts, "project:left").expect("left slot");
    let (right, right_created) = resident_host_slot(&hosts, "project:right").expect("right slot");
    assert!(left_created && right_created);
    let _left_guard = left.lock().expect("left slot lock");
    assert!(
        right.try_lock().is_ok(),
        "one project turn must not lock a different project's resident host"
    );
}

#[test]
fn s1_freeform_public_user_message_transport_cannot_bypass_canonical_persistence() {
    let error = consult_supervisor_resident_turn(
        Path::new("/private/tmp/s1-resident-no-state-access/workflow-state.v0.json"),
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:s1:public-user-message-reject",
        "这条原文若被接受就会绕过 canonical 持久化。",
        "user_message",
    )
    .expect_err("public user_message transport must reject before any host or state access");
    assert_eq!(
        error,
        "supervisor_resident_user_message_requires_answer_command"
    );
}

#[test]
fn s1_freeform_messages_reuse_the_thread_without_a_protocol_gate() {
    let (state_path, workflow_id) = resident_fixture_state_path("same-thread");
    let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![resident_mock_spec(
        5101,
        alive,
        vec![
            // Deliberately not JSON: ordinary supervisor text must not be parsed
            // as the retired `supervisor_resident_turn.v1` binary protocol.
            resident_mock_turn(
                "thread-s1-freeform",
                "{这是一段普通主管回复，不是 JSON 协议}",
            ),
            resident_mock_turn("thread-s1-freeform", "可以。你最希望先验证哪一部分？"),
        ],
    )]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "我想先讨论界面范围。"),
    )
    .expect("ordinary first message must succeed without protocol parsing");
    let second = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "请把问题说得更具体一点。"),
    )
    .expect("ordinary follow-up message must stay in the same resident thread");

    assert_eq!(first.status, "message_sent");
    assert_eq!(second.status, "message_sent");
    assert!(first.reply_injected && second.reply_injected);
    assert_eq!(first.thread_id.as_deref(), Some("thread-s1-freeform"));
    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(
        first.supervisor_reply.as_deref(),
        Some("{这是一段普通主管回复，不是 JSON 协议}")
    );

    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply"]
    );
    assert!(calls[0].arguments["developer-instructions"]
        .as_str()
        .expect("resident developer instructions")
        .contains("不要输出或要求固定 JSON 回合协议"));
    assert!(!calls[0].arguments["prompt"]
        .as_str()
        .expect("opening prompt")
        .contains("supervisor_resident_turn.v1"));
    assert!(calls[1].arguments["prompt"]
        .as_str()
        .expect("follow-up prompt")
        .contains("请把问题说得更具体一点。"));

    let events = resident_canonical_events(&state_path);
    let user_records = events
        .iter()
        .filter(|event| event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT)
        .collect::<Vec<_>>();
    assert_eq!(user_records.len(), 2);
    for event in user_records {
        assert_eq!(event["actor_ref"], "user");
        assert_eq!(event["source_kind"], "supervisor_resident_user_message");
        assert!(event.get("question_id").is_none());
    }
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT
            })
            .count(),
        2
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| {
                event["event_type"] == SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_RECORDED_EVENT
            })
            .count(),
        2
    );
    assert!(!events.iter().any(|event| {
        event["event_type"]
            .as_str()
            .is_some_and(|event_type| event_type.contains("protocol_invalid"))
    }));

    resident_fixture_cleanup(&state_path);
}

#[test]
fn s1_freeform_user_message_survives_host_rebuild_as_canonical_fact() {
    let (state_path, workflow_id) = resident_fixture_state_path("rebuild");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            5201,
            first_alive.clone(),
            vec![resident_mock_turn("thread-s1-old", "我会记住 ALPHA。")],
        ),
        resident_mock_spec(
            5202,
            second_alive,
            vec![resident_mock_turn(
                "thread-s1-new",
                "已从项目事实恢复 ALPHA。",
            )],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "请记住 ALPHA 是本轮的用户优先级。"),
    )
    .expect("first freeform message");
    first_alive.store(false, std::sync::atomic::Ordering::SeqCst);
    let rebuilt = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(&workflow_id, "宿主重建后继续讨论。"),
    )
    .expect("dead host must rebuild and continue the user conversation");

    assert_eq!(first.thread_id.as_deref(), Some("thread-s1-old"));
    assert_eq!(rebuilt.thread_id.as_deref(), Some("thread-s1-new"));
    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex"]
    );
    assert!(calls[1].arguments["prompt"]
        .as_str()
        .expect("rebuild opening prompt")
        .contains("ALPHA"));
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    let events = resident_canonical_events(&state_path);
    let first_user_record = events
        .iter()
        .position(|event| {
            event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_RECORDED_EVENT
                && event["message_text"] == "请记住 ALPHA 是本轮的用户优先级。"
                && event["actor_ref"] == "user"
        })
        .expect("first user message is durable before the host can be rebuilt");
    let first_injection = events
        .iter()
        .position(|event| {
            event["event_type"] == SUPERVISOR_RESIDENT_USER_MESSAGE_INJECTED_EVENT
                && event["actor_ref"] == "supervisor_resident"
        })
        .expect("first injection audit event");
    assert!(first_user_record < first_injection);

    resident_fixture_cleanup(&state_path);
}

#[test]
#[ignore = "requires the fixed test project, the built workbench binary, and model API access"]
fn s1_freeform_live_fixed_project_reuses_thread_then_rebuilds_after_host_kill() {
    let executable = std::env::var_os("SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE")
        .expect("S1 真跑须以命令行提供已构建工作台可执行文件");
    assert!(Path::new(&executable).is_file(), "S1 真跑工作台文件不存在");

    let (state_path, workflow_id) = resident_fixture_state_path("live-fixed-project");
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let spawner = RealSupervisorResidentMcpHostSpawner;

    let first = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(
            &workflow_id,
            "S1 只读常驻会话真跑。只回复 `S1A-ALPHA 已记住`；不要调用工具、读取或修改任何文件。",
        ),
    )
    .expect("S1 live first turn");
    println!(
        "S1_LIVE 原始模型回合1 threadId={} content={}",
        first.thread_id.as_deref().unwrap_or("<missing>"),
        first.supervisor_reply.as_deref().unwrap_or("<missing>")
    );

    let second = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(
            &workflow_id,
            "继续同一会话：上一轮唯一标记是什么？只回复 `S1A-TURN2 S1A-ALPHA`；不要调用工具、读取或修改任何文件。",
        ),
    )
    .expect("S1 live second turn");
    let third = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(
            &workflow_id,
            "继续同一会话：确认你仍记得一轮前的标记。只回复 `S1A-TURN3 S1A-ALPHA`；不要调用工具、读取或修改任何文件。",
        ),
    )
    .expect("S1 live third turn");
    println!(
        "S1_LIVE 原始模型回合2 threadId={} content={}",
        second.thread_id.as_deref().unwrap_or("<missing>"),
        second.supervisor_reply.as_deref().unwrap_or("<missing>")
    );
    println!(
        "S1_LIVE 原始模型回合3 threadId={} content={}",
        third.thread_id.as_deref().unwrap_or("<missing>"),
        third.supervisor_reply.as_deref().unwrap_or("<missing>")
    );

    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(third.thread_id, first.thread_id);
    assert!(second
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("S1A-ALPHA")));
    assert!(third
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("S1A-ALPHA")));

    let old_pid = resident_kill_live_host(
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
    );
    println!("S1_LIVE 已杀常驻宿主 pid={old_pid}");

    let rebuilt = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &resident_message_request(
            &workflow_id,
            "旧宿主已被杀。请仅依据工作台注入的正式事实回答 `S1A-REBUILT 正式事实已注入`；不要调用工具、读取或修改任何文件。",
        ),
    )
    .expect("S1 live replacement turn");
    let host_key = resident_host_key(&state_path, crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT);
    let replacement = hosts
        .lock()
        .expect("live replacement host map")
        .get(&host_key)
        .cloned()
        .expect("live replacement host slot");
    let replacement = replacement.lock().expect("live replacement host slot lock");
    let new_pid = replacement
        .host
        .as_ref()
        .expect("live replacement host")
        .pid();
    println!(
        "S1_LIVE 原始模型换代 old_pid={old_pid} new_pid={new_pid} old_threadId={} new_threadId={} content={}",
        first.thread_id.as_deref().unwrap_or("<missing>"),
        rebuilt.thread_id.as_deref().unwrap_or("<missing>"),
        rebuilt.supervisor_reply.as_deref().unwrap_or("<missing>")
    );

    assert_ne!(new_pid, old_pid);
    assert_ne!(rebuilt.thread_id, first.thread_id);
    assert!(rebuilt
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("S1A-REBUILT")));
    drop(replacement);

    let audit_types = resident_audit_event_types(&state_path);
    println!("S1_LIVE 审计 event_type={}", audit_types.join(","));
    assert!(audit_types.contains(&"supervisor_resident_session_created".to_string()));
    assert_eq!(
        audit_types
            .iter()
            .filter(|event_type| *event_type == "supervisor_resident_session_reused")
            .count(),
        2
    );
    assert!(audit_types.contains(&"supervisor_resident_session_replaced".to_string()));
    assert_eq!(
        audit_types
            .iter()
            .filter(|event_type| *event_type == "supervisor_resident_consult_merged")
            .count(),
        4
    );

    drop(hosts);
    resident_fixture_cleanup(&state_path);
}
