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

fn resident_fixture_state_path(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "p1-a-supervisor-resident-{label}-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir_all(&root).expect("create resident fixture root");
    let path = root.join("workflow-state.v0.json");
    fs::write(&path, "{}").expect("write minimal workflow state");
    path
}

fn resident_fixture_cleanup(state_path: &Path) {
    if let Some(root) = state_path.parent() {
        let _ = fs::remove_dir_all(root);
    }
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
fn resident_mock_reuses_one_thread_for_three_turns_and_keeps_private_mcp_shape() {
    let state_path = resident_fixture_state_path("three-turns");
    let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![resident_mock_spec(
        4101,
        alive,
        vec![
            resident_mock_turn("thread-resident-1", "首轮回答：记住 ALPHA。"),
            resident_mock_turn("thread-resident-1", "第二轮引用 ALPHA。"),
            resident_mock_turn("thread-resident-1", "第三轮仍引用 ALPHA。"),
        ],
    )]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:three-turns",
        "请记住 ALPHA，并回答首轮。",
        "project_consult",
    )
    .expect("first resident turn");
    let second = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:three-turns",
        "请引用你刚才记住的内容。",
        "project_consult",
    )
    .expect("second resident turn");
    let third = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:three-turns",
        "请再次确认上下文。",
        "director_plan",
    )
    .expect("third resident turn");

    assert_eq!(first.thread_id, "thread-resident-1");
    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(third.thread_id, first.thread_id);
    assert!(second.content.contains("ALPHA"));
    assert!(third.content.contains("ALPHA"));

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
        Value::String("thread-resident-1".to_string())
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
fn resident_mock_rebuilds_after_host_death_with_new_thread_and_core_facts() {
    let state_path = resident_fixture_state_path("replace-dead-host");
    let old_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let new_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            4201,
            old_alive.clone(),
            vec![resident_mock_turn("thread-old", "首轮保留事实。")],
        ),
        resident_mock_spec(
            4202,
            new_alive,
            vec![resident_mock_turn("thread-new", "换代后由核心事实重建。")],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:replace",
        "首轮建立项目事实。",
        "project_consult",
    )
    .expect("initial resident host");
    old_alive.store(false, std::sync::atomic::Ordering::SeqCst);
    let rebuilt = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:replace",
        "宿主已死，请自动换代并回答。",
        "director_plan_preview",
    )
    .expect("replacement resident host");

    assert_eq!(first.thread_id, "thread-old");
    assert_eq!(rebuilt.thread_id, "thread-new");
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
        .expect("replacement opening prompt")
        .contains("换代/首轮核心事实（不是聊天记录）"));
    assert!(calls[1].arguments["prompt"]
        .as_str()
        .expect("replacement opening prompt")
        .contains("项目黑板既有条目"));
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    let audit_types = resident_audit_event_types(&state_path);
    assert!(audit_types.contains(&"supervisor_resident_session_replaced".to_string()));
    resident_fixture_cleanup(&state_path);
}

#[test]
fn resident_mock_rebuilds_in_same_turn_after_thread_invalid() {
    let state_path = resident_fixture_state_path("replace-thread-invalid");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            4251,
            first_alive,
            vec![
                resident_mock_turn("thread-invalid-old", "首轮保留事实。"),
                Err("supervisor_resident_mcp_terminal_error:thread not found".to_string()),
            ],
        ),
        resident_mock_spec(
            4252,
            second_alive,
            vec![resident_mock_turn(
                "thread-invalid-new",
                "线程失效后由核心事实同轮重建。",
            )],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    let first = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:thread-invalid",
        "首轮建立项目事实。",
        "project_consult",
    )
    .expect("initial resident host");
    let rebuilt = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:thread-invalid",
        "本轮 thread 已失效，必须自动换代并继续回答。",
        "project_consult",
    )
    .expect("thread-invalid replacement must finish the same turn");

    assert_eq!(first.thread_id, "thread-invalid-old");
    assert_eq!(rebuilt.thread_id, "thread-invalid-new");
    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply", "codex"]
    );
    assert!(calls[2].arguments["prompt"]
        .as_str()
        .expect("replacement opening prompt")
        .contains("换代/首轮核心事实（不是聊天记录）"));
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    let audit_types = resident_audit_event_types(&state_path);
    assert!(audit_types.contains(&"supervisor_resident_session_replaced".to_string()));
    resident_fixture_cleanup(&state_path);
}

#[test]
fn resident_mock_parse_failure_retires_host_before_a_later_clean_generation() {
    let state_path = resident_fixture_state_path("parse-failure");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            4301,
            first_alive,
            vec![
                resident_mock_turn("thread-parse-old", "首轮正常。"),
                Err("supervisor_resident_mcp_event_parse_failed:bad-json".to_string()),
            ],
        ),
        resident_mock_spec(
            4302,
            second_alive,
            vec![resident_mock_turn("thread-parse-new", "后续独立新代。")],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());

    consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:parse-failure",
        "先建立会话。",
        "project_consult",
    )
    .expect("initial resident host");
    let error = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:parse-failure",
        "这轮会触发坏事件。",
        "project_consult",
    )
    .expect_err("parse error must conservatively stop");
    assert_eq!(error, "supervisor_resident_mcp_event_parse_failed:bad-json");
    assert!(hosts.lock().expect("host map").is_empty());
    assert_eq!(
        spawner.terminated.load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    let later = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        "workflow:resident:parse-failure",
        "下一轮才允许建立干净新代。",
        "project_consult",
    )
    .expect("later clean generation");
    assert_eq!(later.thread_id, "thread-parse-new");
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
fn resident_project_slots_have_independent_locks() {
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
#[ignore = "requires the fixed test project, the built workbench binary, and model API access"]
fn p1_a_live_fixed_project_reuses_thread_then_rebuilds_after_host_kill() {
    let executable = std::env::var_os("SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE")
        .expect("P1-A 真跑须以命令行提供已构建工作台可执行文件");
    assert!(
        Path::new(&executable).is_file(),
        "P1-A 真跑工作台文件不存在"
    );

    let state_path = resident_fixture_state_path("live-fixed-project");
    let workflow_id = "workflow:p1-a:live-fixed-project";
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let spawner = RealSupervisorResidentMcpHostSpawner;

    let first = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        workflow_id,
        "P1-A 只读常驻会话真跑。只回复 `P1A-ALPHA 已记住`；不要调用工具、读取或修改任何文件。",
        "project_consult",
    )
    .expect("P1-A live first turn");
    println!(
        "P1A_LIVE 原始模型回合1 threadId={} content={}",
        first.thread_id, first.content
    );

    let second = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        workflow_id,
        "继续同一会话：上一轮唯一标记是什么？只回复 `P1A-TURN2 P1A-ALPHA`；不要调用工具、读取或修改任何文件。",
        "project_consult",
    )
    .expect("P1-A live second turn");
    println!(
        "P1A_LIVE 原始模型回合2 threadId={} content={}",
        second.thread_id, second.content
    );

    let third = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        workflow_id,
        "继续同一会话：确认你仍记得一轮前的标记。只回复 `P1A-TURN3 P1A-ALPHA`；不要调用工具、读取或修改任何文件。",
        "director_plan",
    )
    .expect("P1-A live third turn");
    println!(
        "P1A_LIVE 原始模型回合3 threadId={} content={}",
        third.thread_id, third.content
    );

    assert_eq!(second.thread_id, first.thread_id);
    assert_eq!(third.thread_id, first.thread_id);
    assert!(second.content.contains("P1A-ALPHA"));
    assert!(third.content.contains("P1A-ALPHA"));

    let old_pid = resident_kill_live_host(
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
    );
    println!("P1A_LIVE 已杀常驻宿主 pid={old_pid}");

    let rebuilt = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        workflow_id,
        "旧宿主已被杀。请仅依据工作台注入的正式事实回答 `P1A-REBUILT 正式事实已注入`；不要调用工具、读取或修改任何文件。",
        "director_plan_preview",
    )
    .expect("P1-A live replacement turn");
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
        "P1A_LIVE 原始模型换代 old_pid={old_pid} new_pid={new_pid} old_threadId={} new_threadId={} content={}",
        first.thread_id, rebuilt.thread_id, rebuilt.content
    );

    assert_ne!(new_pid, old_pid);
    assert_ne!(rebuilt.thread_id, first.thread_id);
    assert!(rebuilt.content.contains("P1A-REBUILT"));
    drop(replacement);

    let audit_types = resident_audit_event_types(&state_path);
    println!("P1A_LIVE 审计 event_type={}", audit_types.join(","));
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
