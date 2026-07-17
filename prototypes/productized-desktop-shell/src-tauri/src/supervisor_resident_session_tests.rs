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

fn resident_question_fixture_state_path(label: &str) -> (std::path::PathBuf, String) {
    let root = std::env::temp_dir().join(format!(
        "p1-b-supervisor-question-{label}-{}",
        crate::unix_timestamp_nanos()
    ));
    fs::create_dir_all(&root).expect("create resident question fixture root");
    let path = root.join("workflow-state.v0.json");
    let project_root = crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
    crate::bootstrap_project_workflow_at(
        &path,
        &crate::ProjectRecord {
            project_root: project_root.to_string(),
            name: "P1-B resident question fixture".to_string(),
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
    .expect("bootstrap fixed resident question workflow");
    let workflow_id = crate::default_workflow_id(project_root);
    (path, workflow_id)
}

fn resident_question_turn_json(
    expected: &crate::ResidentQuestionExpectation,
    question: &str,
) -> String {
    serde_json::json!({
        "schema_version": "supervisor_resident_turn.v1",
        "kind": "supervisor_question",
        "question_id": expected.question_id,
        "project_id": expected.project_id,
        "workflow_id": expected.workflow_id,
        "round": expected.round,
        "question": question,
    })
    .to_string()
}

fn resident_proposal_turn_json(summary: &str) -> String {
    serde_json::json!({
        "schema_version": "supervisor_resident_turn.v1",
        "kind": "proposal",
        "user_goal": "P1-B 固定测试项目问答闭环",
        "goal_summary": summary,
        "scope_note": "纯只读测试方案",
        "reasoning": ["用户答复已通过受控同 thread 注入。"],
        "risks": [{"severity": "info", "summary": "测试不修改项目", "mitigation": "保持 read-only。"}],
        "must_stop_points": ["需要真实写入时停止"],
        "next_steps": ["由用户确认方案"],
        "worker_acceptance_criteria": ["不修改任何项目文件"],
        "control_core_acceptance_criteria": ["只读沙箱与审计可核验"],
        "supervisor_acceptance_criteria": ["主管仅输出方案"],
        "execution_scope": null,
        "suggest_workflow": false,
    })
    .to_string()
}

fn resident_record_mock_question(
    spawner: &ResidentMockSpawner,
    hosts: &Mutex<SupervisorResidentHostMap>,
    state_path: &Path,
    workflow_id: &str,
    expected: &crate::ResidentQuestionExpectation,
    user_goal: &str,
) -> crate::ResidentSupervisorQuestion {
    let turn = consult_supervisor_resident_with(
        spawner,
        hosts,
        state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        workflow_id,
        "P1-B mock initial question turn",
        "project_consult",
    )
    .expect("mock question turn");
    let question = match crate::parse_resident_consultation_turn(&turn.content, expected)
        .expect("strict mock question turn")
    {
        crate::ResidentConsultationTurn::SupervisorQuestion(question) => question,
        crate::ResidentConsultationTurn::Proposal(_) => panic!("mock should ask a question"),
    };
    record_supervisor_resident_question_asked(
        state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &question,
        user_goal,
        &turn.thread_id,
    )
    .expect("record canonical mock question");
    question
}

fn resident_canonical_event_types(state_path: &Path) -> Vec<String> {
    crate::read_workflow_state_value(state_path)
        .expect("read resident canonical state")
        .get("audit_events")
        .and_then(Value::as_array)
        .expect("canonical audit events")
        .iter()
        .filter_map(|event| event["event_type"].as_str().map(str::to_string))
        .collect()
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
fn p1_b_resident_turn_schema_is_strict_binary_and_stops_on_invalid_shapes() {
    let expected = crate::ResidentQuestionExpectation {
        question_id: "resident-question:fixture:1".to_string(),
        project_id: crate::project_id(crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT),
        workflow_id: "workflow:p1-b:schema".to_string(),
        round: 1,
    };
    let question = resident_question_turn_json(&expected, "请明确唯一验收目标？");
    assert!(matches!(
        crate::parse_resident_consultation_turn(&question, &expected),
        Ok(crate::ResidentConsultationTurn::SupervisorQuestion(_))
    ));
    let proposal = resident_proposal_turn_json("严格方案 JSON 仍可通过既有闸");
    assert!(matches!(
        crate::parse_resident_consultation_turn(&proposal, &expected),
        Ok(crate::ResidentConsultationTurn::Proposal(_))
    ));

    for invalid in [
        "没有 json 的自由文本",
        r#"{"schema_version":"supervisor_resident_turn.v1","kind":"supervisor_question","question_id":"resident-question:fixture:1","project_id":"wrong","workflow_id":"workflow:p1-b:schema","round":1,"question":"x"}"#,
        r#"{"schema_version":"supervisor_resident_turn.v1","kind":"supervisor_question","question_id":"resident-question:fixture:1","project_id":"project:users-yoyi-codex-workflow-mario-test","workflow_id":"workflow:p1-b:schema","round":1,"question":"x","goal_summary":"mixed"}"#,
        r#"{"schema_version":"supervisor_resident_turn.v1","kind":"unknown"}"#,
        r#"说明文字```json
{"schema_version":"supervisor_resident_turn.v1","kind":"supervisor_question","question_id":"resident-question:fixture:1","project_id":"project:users-yoyi-codex-workflow-mario-test","workflow_id":"workflow:p1-b:schema","round":1,"question":"x"}
```"#,
    ] {
        let error = crate::parse_resident_consultation_turn(invalid, &expected)
            .expect_err("invalid resident turn must conservatively stop");
        assert!(
            error.starts_with("protocol_invalid:supervisor_resident_turn_"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn p1_b_mock_question_answer_same_thread_proposal_then_duplicate_is_rejected() {
    let (state_path, workflow_id) = resident_question_fixture_state_path("same-thread");
    let first_expected = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect("preissue first question id");
    let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![resident_mock_spec(
        5101,
        alive,
        vec![
            resident_mock_turn(
                "thread-p1b-q1",
                &resident_question_turn_json(&first_expected, "请给出唯一验收标记。"),
            ),
            resident_mock_turn(
                "thread-p1b-q1",
                &resident_proposal_turn_json("用户已答，输出待确认方案。"),
            ),
        ],
    )]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let question = resident_record_mock_question(
        &spawner,
        &hosts,
        &state_path,
        &workflow_id,
        &first_expected,
        "P1-B：先提问，再收到答复后只输出方案。",
    );
    let request = SubmitSupervisorResidentAnswerRequest {
        project_id: question.project_id.clone(),
        workflow_id: workflow_id.clone(),
        question_id: question.question_id.clone(),
        answer_text: "验收标记是 P1B-ANSWER-ALPHA。".to_string(),
    };
    let waiting_snapshot =
        crate::read_workflow_state_snapshot(&state_path).expect("read waiting question snapshot");
    let waiting_message = waiting_snapshot.project_blackboards[0]
        .entries
        .iter()
        .find(|entry| entry.kind == crate::BlackboardEntryKind::SupervisorMessage)
        .expect("waiting supervisor message should be derived");
    assert_eq!(waiting_message.status, "waiting_user");
    assert_eq!(
        waiting_message.question_id.as_deref(),
        Some(request.question_id.as_str())
    );
    assert_eq!(
        serde_json::to_value(waiting_message)
            .expect("serialize waiting supervisor message")["question_id"],
        Value::String(request.question_id.clone())
    );
    let mut absent_question_id = (*waiting_message).clone();
    absent_question_id.question_id = None;
    assert!(
        serde_json::to_value(&absent_question_id)
            .expect("serialize blackboard entry without question id")
            .get("question_id")
            .is_none(),
        "optional question_id must stay absent when it is not derived"
    );
    let pending = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect_err("an unanswered canonical question must block another pre-issued id");
    assert_eq!(
        pending,
        format!(
            "supervisor_resident_question_waiting_user_reply:{}",
            request.question_id
        )
    );

    let outcome = submit_supervisor_resident_answer_with(&spawner, &hosts, &state_path, &request)
        .expect("answer should resume same resident thread");
    assert_eq!(outcome.status, "proposal_created");
    assert!(outcome.reply_injected);
    assert_eq!(outcome.thread_id.as_deref(), Some("thread-p1b-q1"));
    assert!(outcome.proposal.is_some());
    assert!(outcome
        .supervisor_reply
        .as_deref()
        .is_some_and(|reply| reply.contains("待确认方案")));

    let calls_before_duplicate = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls_before_duplicate
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply"]
    );
    assert_eq!(
        calls_before_duplicate[1].arguments["threadId"],
        Value::String("thread-p1b-q1".to_string())
    );
    assert!(calls_before_duplicate[1].arguments["prompt"]
        .as_str()
        .expect("user reply prompt")
        .contains(&request.answer_text));

    let duplicate = submit_supervisor_resident_answer_with(&spawner, &hosts, &state_path, &request)
        .expect("duplicate must return existing outcome, not inject again");
    assert_eq!(duplicate.status, "already_answered");
    assert!(duplicate.reply_injected);
    assert_eq!(
        spawner.calls.lock().expect("resident calls").len(),
        calls_before_duplicate.len(),
        "duplicate user answer must not call codex-reply again"
    );

    let canonical_types = resident_canonical_event_types(&state_path);
    for event_type in [
        SUPERVISOR_RESIDENT_QUESTION_ASKED_EVENT,
        SUPERVISOR_RESIDENT_QUESTION_ANSWERED_EVENT,
        SUPERVISOR_RESIDENT_REPLY_INJECTED_EVENT,
    ] {
        assert!(canonical_types.contains(&event_type.to_string()));
    }
    let sidecar_types = resident_audit_event_types(&state_path);
    for event_type in [
        SUPERVISOR_RESIDENT_QUESTION_ASKED_EVENT,
        SUPERVISOR_RESIDENT_QUESTION_ANSWERED_EVENT,
        SUPERVISOR_RESIDENT_REPLY_INJECTED_EVENT,
    ] {
        assert!(sidecar_types.contains(&event_type.to_string()));
    }
    let snapshot =
        crate::read_workflow_state_snapshot(&state_path).expect("read question snapshot");
    let messages = snapshot.project_blackboards[0]
        .entries
        .iter()
        .filter(|entry| entry.kind == crate::BlackboardEntryKind::SupervisorMessage)
        .collect::<Vec<_>>();
    assert_eq!(
        messages.len(),
        2,
        "question and answer must both be readable"
    );
    assert!(messages.iter().any(|entry| entry.status == "answered"));
    assert!(messages.iter().all(|entry| {
        entry.question_id.as_deref() == Some(request.question_id.as_str())
    }));
    assert!(messages.iter().all(|entry| {
        entry
            .warnings
            .contains(&"supervisor_message_does_not_advance_workflow".to_string())
    }));
    resident_fixture_cleanup(&state_path);
}

#[test]
fn p1_b_mock_recovers_exact_durable_answer_after_pre_injection_failure() {
    let (state_path, workflow_id) = resident_question_fixture_state_path("recover-durable-answer");
    let first_expected = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect("preissue first question id");
    let first_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let second_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            5151,
            first_alive,
            vec![
                resident_mock_turn(
                    "thread-p1b-recover-old",
                    &resident_question_turn_json(&first_expected, "请给出恢复标记。"),
                ),
                Err("supervisor_resident_mcp_event_parse_failed:temporary".to_string()),
            ],
        ),
        resident_mock_spec(
            5152,
            second_alive,
            vec![resident_mock_turn(
                "thread-p1b-recover-new",
                &resident_proposal_turn_json("恢复后的同一用户答复已进入方案。"),
            )],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let question = resident_record_mock_question(
        &spawner,
        &hosts,
        &state_path,
        &workflow_id,
        &first_expected,
        "P1-B：答复持久化后若注入前失败，只能由同一用户原文恢复。",
    );
    let request = SubmitSupervisorResidentAnswerRequest {
        project_id: question.project_id,
        workflow_id: workflow_id.clone(),
        question_id: question.question_id,
        answer_text: "P1B-DURABLE-RECOVERY-DELTA".to_string(),
    };

    let first_error =
        match submit_supervisor_resident_answer_with(&spawner, &hosts, &state_path, &request) {
            Err(error) => error,
            Ok(_) => panic!("first post-answer transport failure must conservatively stop"),
        };
    assert_eq!(
        first_error,
        "supervisor_resident_mcp_event_parse_failed:temporary"
    );
    let wrong_answer = SubmitSupervisorResidentAnswerRequest {
        project_id: request.project_id.clone(),
        workflow_id: request.workflow_id.clone(),
        question_id: request.question_id.clone(),
        answer_text: "P1B-DIFFERENT-ANSWER".to_string(),
    };
    let wrong_answer_error = match submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &wrong_answer,
    ) {
        Err(error) => error,
        Ok(_) => panic!("a different answer must not revive a durable pending answer"),
    };
    assert_eq!(
        wrong_answer_error,
        "supervisor_resident_answer_already_recorded_text_mismatch"
    );

    let recovered = submit_supervisor_resident_answer_with(&spawner, &hosts, &state_path, &request)
        .expect("the exact explicit command input may recover its pending injection");
    assert_eq!(recovered.status, "proposal_created");
    assert_eq!(
        recovered.thread_id.as_deref(),
        Some("thread-p1b-recover-new")
    );
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
        .expect("recovery opening prompt")
        .contains(&request.answer_text));
    let canonical_types = resident_canonical_event_types(&state_path);
    assert_eq!(
        canonical_types
            .iter()
            .filter(|event_type| *event_type == SUPERVISOR_RESIDENT_QUESTION_ANSWERED_EVENT)
            .count(),
        1,
        "recovery must not append a second canonical user answer"
    );
    let sidecar_types = resident_audit_event_types(&state_path);
    assert_eq!(
        sidecar_types
            .iter()
            .filter(|event_type| *event_type == SUPERVISOR_RESIDENT_QUESTION_ANSWERED_EVENT)
            .count(),
        1,
        "recovery must not duplicate the M5 user-answer audit"
    );
    resident_fixture_cleanup(&state_path);
}

#[test]
fn p1_b_mock_supports_second_question_before_final_proposal() {
    let (state_path, workflow_id) = resident_question_fixture_state_path("second-question");
    let first_expected = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect("preissue first question id");
    let second_expected = crate::ResidentQuestionExpectation {
        question_id: format!("resident-question:{}:2", crate::stable_id(&workflow_id)),
        project_id: first_expected.project_id.clone(),
        workflow_id: workflow_id.clone(),
        round: 2,
    };
    let alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![resident_mock_spec(
        5201,
        alive,
        vec![
            resident_mock_turn(
                "thread-p1b-chain",
                &resident_question_turn_json(&first_expected, "第一问：选 A 还是 B？"),
            ),
            resident_mock_turn(
                "thread-p1b-chain",
                &resident_question_turn_json(&second_expected, "第二问：验收用什么标记？"),
            ),
            resident_mock_turn(
                "thread-p1b-chain",
                &resident_proposal_turn_json("两轮答复已齐，输出方案。"),
            ),
        ],
    )]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let first_question = resident_record_mock_question(
        &spawner,
        &hosts,
        &state_path,
        &workflow_id,
        &first_expected,
        "P1-B：允许两轮问题，第二轮答复后才出方案。",
    );
    let first_outcome = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &SubmitSupervisorResidentAnswerRequest {
            project_id: first_question.project_id.clone(),
            workflow_id: workflow_id.clone(),
            question_id: first_question.question_id,
            answer_text: "选择 A。".to_string(),
        },
    )
    .expect("first answer should produce second question");
    assert_eq!(first_outcome.status, "question_asked");
    let second_question = first_outcome.question.expect("second question outcome");
    assert_eq!(second_question.question_id, second_expected.question_id);
    assert_eq!(second_question.round, 2);

    let second_outcome = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &SubmitSupervisorResidentAnswerRequest {
            project_id: second_question.project_id,
            workflow_id: workflow_id.clone(),
            question_id: second_question.question_id,
            answer_text: "验收标记是 P1B-CHAIN-BETA。".to_string(),
        },
    )
    .expect("second answer should produce proposal");
    assert_eq!(second_outcome.status, "proposal_created");
    assert_eq!(
        second_outcome.thread_id.as_deref(),
        Some("thread-p1b-chain")
    );
    assert_eq!(
        spawner
            .calls
            .lock()
            .expect("resident calls")
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex-reply", "codex-reply"]
    );
    resident_fixture_cleanup(&state_path);
}

#[test]
fn p1_b_mock_dead_host_rebuilds_with_durable_answer_and_injects_it() {
    let (state_path, workflow_id) = resident_question_fixture_state_path("dead-host-answer");
    let first_expected = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect("preissue first question id");
    let old_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let new_alive = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let spawner = ResidentMockSpawner::new(vec![
        resident_mock_spec(
            5301,
            old_alive.clone(),
            vec![resident_mock_turn(
                "thread-p1b-old",
                &resident_question_turn_json(&first_expected, "旧宿主问题：给出恢复标记。"),
            )],
        ),
        resident_mock_spec(
            5302,
            new_alive,
            vec![resident_mock_turn(
                "thread-p1b-new",
                &resident_proposal_turn_json("换代后仍拿到了用户答复。"),
            )],
        ),
    ]);
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let question = resident_record_mock_question(
        &spawner,
        &hosts,
        &state_path,
        &workflow_id,
        &first_expected,
        "P1-B：宿主死亡后也要把答复带进换代回合。",
    );
    old_alive.store(false, std::sync::atomic::Ordering::SeqCst);
    let answer_text = "P1B-RECOVERY-GAMMA".to_string();
    let outcome = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &SubmitSupervisorResidentAnswerRequest {
            project_id: question.project_id,
            workflow_id: workflow_id.clone(),
            question_id: question.question_id,
            answer_text: answer_text.clone(),
        },
    )
    .expect("dead host answer must rebuild and inject in one command");
    assert_eq!(outcome.status, "proposal_created");
    assert_eq!(outcome.thread_id.as_deref(), Some("thread-p1b-new"));
    let calls = spawner.calls.lock().expect("resident calls").clone();
    assert_eq!(
        calls
            .iter()
            .map(|call| call.tool_name.as_str())
            .collect::<Vec<_>>(),
        ["codex", "codex"],
        "dead host must use P1-A replacement first turn, not silently drop answer"
    );
    let rebuilt_prompt = calls[1].arguments["prompt"]
        .as_str()
        .expect("replacement opening prompt");
    assert!(rebuilt_prompt.contains(&answer_text));
    assert!(rebuilt_prompt.contains("旧宿主问题：给出恢复标记。"));
    assert!(rebuilt_prompt.contains("第 1 轮用户答复"));
    let sidecar_types = resident_audit_event_types(&state_path);
    assert!(sidecar_types.contains(&"supervisor_resident_session_replaced".to_string()));
    assert!(sidecar_types.contains(&SUPERVISOR_RESIDENT_REPLY_INJECTED_EVENT.to_string()));
    resident_fixture_cleanup(&state_path);
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

#[test]
#[ignore = "requires the fixed test project, the built workbench binary, and model API access"]
fn p1_b_live_fixed_project_question_answer_then_proposal_same_thread() {
    let executable = std::env::var_os("SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE")
        .expect("P1-B 真跑须以命令行提供已构建工作台可执行文件");
    assert!(
        Path::new(&executable).is_file(),
        "P1-B 真跑工作台文件不存在"
    );

    let (state_path, workflow_id) = resident_question_fixture_state_path("live-question-answer");
    let expected_question = next_resident_question_expectation(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
    )
    .expect("preissue live question identity");
    let user_goal = "P1-B 固定测试项目真跑：本轮必须先按常驻主管协议提出一个具体问题；收到后续含 P1B-LIVE-ANSWER 的真实用户答复后，必须只输出 proposal，不得再提问题、不得调用工具、读取或修改任何文件。";
    let opening_prompt = format!(
        "{user_goal}\n\n{}",
        crate::resident_consultation_turn_schema_prompt(&expected_question)
    );
    let hosts: Mutex<SupervisorResidentHostMap> = Mutex::new(BTreeMap::new());
    let spawner = RealSupervisorResidentMcpHostSpawner;
    let first = consult_supervisor_resident_with(
        &spawner,
        &hosts,
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &workflow_id,
        &opening_prompt,
        "project_consult",
    )
    .expect("P1-B live question turn");
    println!(
        "P1B_LIVE 原始模型问句 threadId={} content={}",
        first.thread_id, first.content
    );
    let question = match crate::parse_resident_consultation_turn(&first.content, &expected_question)
        .expect("P1-B live question must satisfy strict schema")
    {
        crate::ResidentConsultationTurn::SupervisorQuestion(question) => question,
        crate::ResidentConsultationTurn::Proposal(_) => {
            panic!("P1-B live initial turn must ask one question")
        }
    };
    record_supervisor_resident_question_asked(
        &state_path,
        crate::WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
        &question,
        user_goal,
        &first.thread_id,
    )
    .expect("record P1-B live canonical question");

    let outcome = submit_supervisor_resident_answer_with(
        &spawner,
        &hosts,
        &state_path,
        &SubmitSupervisorResidentAnswerRequest {
            project_id: question.project_id,
            workflow_id: workflow_id.clone(),
            question_id: question.question_id,
            answer_text: "P1B-LIVE-ANSWER=42。具体目标：为固定测试项目给出一份纯只读的检查方案；允许范围：只描述现有项目状态，不读取或修改任何文件、不调用工具；验收标准：输出严格 supervisor_resident_turn.v1 proposal JSON，且 risks 必须是对象数组，每项都有 severity、summary、mitigation。此纯咨询必须令 execution_scope 为 JSON null、suggest_workflow 为 JSON literal false，二者都不能是数组、字符串或 null 以外的值。现在请按协议只输出 proposal。".to_string(),
        },
    )
    .expect("P1-B live answer continuation");
    println!(
        "P1B_LIVE 原始模型答复续跑 threadId={} content={}",
        outcome.thread_id.as_deref().unwrap_or("<missing>"),
        outcome.supervisor_reply.as_deref().unwrap_or("<missing>")
    );
    assert_eq!(outcome.status, "proposal_created");
    assert_eq!(outcome.thread_id.as_deref(), Some(first.thread_id.as_str()));
    assert!(outcome.proposal.is_some());

    let canonical_types = resident_canonical_event_types(&state_path);
    let sidecar_types = resident_audit_event_types(&state_path);
    println!(
        "P1B_LIVE canonical event_type={} sidecar event_type={}",
        canonical_types.join(","),
        sidecar_types.join(",")
    );
    for event_type in [
        SUPERVISOR_RESIDENT_QUESTION_ASKED_EVENT,
        SUPERVISOR_RESIDENT_QUESTION_ANSWERED_EVENT,
        SUPERVISOR_RESIDENT_REPLY_INJECTED_EVENT,
    ] {
        assert!(canonical_types.contains(&event_type.to_string()));
        assert!(sidecar_types.contains(&event_type.to_string()));
    }

    drop(hosts);
    resident_fixture_cleanup(&state_path);
}
