fn resident_submit_proposal_config(fixture: &Fixture) -> McpServerConfig {
    let mut config = fixture.config.clone();
    config.run_id = "supervisor-resident:s1-submit-proposal-fixture".to_string();
    let project_id = crate::project_id(PROJECT);
    let latest_user_message = "请基于刚才的共识落一张可确认的 S1 方案卡。";
    fs::write(
        &fixture.state_path,
        serde_json::to_vec(&json!({
            "projects": [{
                "project_id": project_id,
                "display_name": "S1 resident proposal fixture",
                "root_path": PROJECT
            }],
            "workflows": [{
                "workflow_id": WORKFLOW,
                "project_id": crate::project_id(PROJECT),
                "state": "draft"
            }],
            "workflow_chain_runs": [],
            "audit_events": [{
                "event_type": "supervisor_resident_user_message_recorded",
                "project_id": crate::project_id(PROJECT),
                "workflow_id": WORKFLOW,
                "message_id": "user:s1-submit-proposal",
                "message_text": latest_user_message,
                "actor_ref": "user",
                "source_kind": "supervisor_resident_user_message"
            }]
        }))
        .expect("resident proposal workflow state json"),
    )
    .expect("resident proposal workflow state");
    record_resident_session_created(
        &config,
        PROJECT,
        WORKFLOW,
        "thread-s1-submit-proposal",
        4321,
        1,
    )
    .expect("durable resident session binding");
    // H2 binds submit_proposal to the canonical user message of the active
    // finite turn; this S1 fixture must model that server-owned boundary,
    // rather than granting a bare historical thread permission.
    record_resident_turn_prepared(
        &config,
        PROJECT,
        WORKFLOW,
        4321,
        1,
        Some("user:s1-submit-proposal"),
    )
    .expect("active canonical user-message turn");
    record_resident_session_reused(
        &config,
        PROJECT,
        WORKFLOW,
        "thread-s1-submit-proposal",
        4321,
        1,
    )
    .expect("active resident turn binding");
    config
}

fn valid_submit_proposal_arguments() -> Value {
    json!({
        "user_goal": "为固定测试项目整理并确认 S1 实施方案。",
        "goal_summary": "补齐自由对话与私有 proposal 工具的验收方案。",
        "scope_note": "用户确认前不启动工作流。",
        "reasoning": ["对话共识已覆盖范围、风险和验收。"],
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
            "write_roots": [PROJECT],
            "target_files": ["src/s1-proof.txt"],
            "tools": ["shell(读写·写域由沙箱锁定)"],
            "checks": ["cargo test --lib"]
        },
        "suggest_workflow": true,
        "tasks": [{
            "title": "补齐 S1 证明",
            "task_goal": "在固定测试项目内补齐 S1 验收所需的最小证明。",
            "target_role": "codex-dev",
            "depends_on": [],
            "acceptance_criteria": ["证明可由指定检查回读。"],
            "report_format": ["说明改动和验证结果。"]
        }]
    })
}

fn proposal_store_for(fixture: &Fixture) -> crate::ProjectConsultationProposalStoreV1 {
    crate::project_consultation_proposal_store::load_store(&fixture.state_path, now_ms())
        .expect("proposal store")
}

fn workflow_chain_runs_for(fixture: &Fixture) -> Value {
    serde_json::from_slice::<Value>(&fs::read(&fixture.state_path).expect("workflow state"))
        .expect("workflow state json")["workflow_chain_runs"]
        .clone()
}

#[test]
fn s1_resident_submit_proposal_creates_only_pending_card_without_starting_chain() {
    let fixture = Fixture::new();
    let config = resident_submit_proposal_config(&fixture);

    let response = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": valid_submit_proposal_arguments()}),
        &FakeInvoker,
    )
    .expect("strict resident proposal should create a pending card");
    let receipt: Value = serde_json::from_str(
        response["content"][0]["text"]
            .as_str()
            .expect("tool receipt text"),
    )
    .expect("tool receipt json");
    assert_eq!(
        receipt["status"],
        "proposal_created_pending_user_confirmation"
    );

    let store = proposal_store_for(&fixture);
    assert_eq!(
        store.proposals.len(),
        1,
        "exactly one proposal card is written"
    );
    let proposal = &store.proposals[0];
    assert_eq!(
        proposal.status,
        crate::ProjectConsultationProposalStatus::PendingUserConfirmation
    );
    assert_eq!(
        proposal.user_requirement_snapshot, "请基于刚才的共识落一张可确认的 S1 方案卡。",
        "the card must bind the canonical user message, not model prose"
    );
    assert_eq!(
        store
            .audit_events
            .iter()
            .map(|event| event.event_type.as_str())
            .collect::<Vec<_>>(),
        vec!["project_consultation_proposal_created"],
        "the existing proposal-store audit family remains the only card write"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        json!([]),
        "submitting a card must not start or advance a workflow chain"
    );
}

#[test]
fn s1_resident_submit_proposal_rejects_toxic_task_graph_without_card_or_chain() {
    let fixture = Fixture::new();
    let config = resident_submit_proposal_config(&fixture);
    let mut arguments = valid_submit_proposal_arguments();
    arguments["tasks"][0]["task_goal"] =
        json!("先用 read_file 工具读取 README.md，再补齐 S1 证明。");

    let error = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": arguments}),
        &FakeInvoker,
    )
    .expect_err("toxic task graph must be refused before proposal storage");
    assert_eq!(
        error, "方案卡没有生成；详细诊断已保留。",
        "the tool must return the stable human-facing failure, not untrusted task text"
    );
    assert!(
        proposal_store_for(&fixture).proposals.is_empty(),
        "toxic graph must not create a proposal card"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        json!([]),
        "toxic graph must not start or advance a workflow chain"
    );
}

#[test]
fn s1_resident_submit_proposal_rejects_unknown_and_wrong_nested_fields_before_store() {
    let fixture = Fixture::new();
    let config = resident_submit_proposal_config(&fixture);

    let mut unknown_nested = valid_submit_proposal_arguments();
    unknown_nested["risks"][0]["untrusted_flag"] = json!(true);
    let unknown_error = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": unknown_nested}),
        &FakeInvoker,
    )
    .expect_err("unknown nested field must be rejected at the tool boundary");
    assert_eq!(
        unknown_error, "方案卡没有生成；详细诊断已保留。",
        "untrusted MCP field names must not be projected to the caller"
    );
    assert!(
        proposal_store_for(&fixture).proposals.is_empty(),
        "unknown nested field must not reach proposal storage"
    );

    let mut wrong_nested = valid_submit_proposal_arguments();
    wrong_nested["tasks"][0]["depends_on"] = json!("not-an-array");
    let wrong_error = call_tool_with_invoker(
        &config,
        json!({"name": "submit_proposal", "arguments": wrong_nested}),
        &FakeInvoker,
    )
    .expect_err("wrong nested type must be rejected at the tool boundary");
    assert_eq!(
        wrong_error, "方案卡没有生成；详细诊断已保留。",
        "untrusted MCP parameter diagnostics must remain private"
    );
    assert!(
        proposal_store_for(&fixture).proposals.is_empty(),
        "wrong nested field must not reach proposal storage"
    );
    assert_eq!(
        workflow_chain_runs_for(&fixture),
        json!([]),
        "schema failures must not start or advance a workflow chain"
    );
}

#[test]
fn s1b_h2_resident_submit_proposal_audit_write_failure_never_returns_raw_detail() {
    let fixture = Fixture::new();
    let private_marker = "H2_PRIVATE_AUDIT_WRITE_FAILURE";
    let blocked_parent = fixture.root.join(private_marker);
    fs::write(&blocked_parent, "not a directory").expect("audit write failure fixture");
    let mut config = fixture.config.clone();
    config.run_id = "supervisor-resident:h2-audit-write-failure".to_string();
    config.supervisor_workflow_state_path = Some(blocked_parent.join("workflow-state.v0.json"));

    for (result_status, expected) in [
        ("denied", "方案卡没有生成；内部审计未完成。"),
        ("accepted", "方案已落卡，但工具审计未完成。"),
    ] {
        let error = append_audit(
            &config,
            "submit_proposal",
            &format!("private={private_marker}"),
            "raw detail must not escape",
            result_status,
        )
        .expect_err("audit write must fail through the stable resident facade");
        assert_eq!(error, expected);
        assert!(!error.contains(private_marker));
    }
}

#[test]
fn station3a_mcp_toolface_is_read_only_and_rejects_side_effect_names() {
    let fixture = Fixture::new();
    let pilot_toolface = list_tools(&fixture.config);
    let pilot_tools = pilot_toolface["tools"]
        .as_array()
        .expect("pilot tools array");
    let pilot_names = pilot_tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        pilot_names,
        BTreeSet::from(["read_key_file", "read_worker_report", "wait_for_worker"])
    );

    let mut resident_config = fixture.config.clone();
    resident_config.run_id = "supervisor-resident:toolface-fixture".to_string();
    let resident_toolface = list_tools(&resident_config);
    let resident_tools = resident_toolface["tools"]
        .as_array()
        .expect("resident tools array");
    let resident_names = resident_tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        resident_names,
        BTreeSet::from([
            "read_key_file",
            "read_worker_report",
            "submit_proposal",
            "wait_for_worker"
        ])
    );
    let proposal_tool = resident_tools
        .iter()
        .find(|tool| tool["name"] == "submit_proposal")
        .expect("resident submit_proposal tool");
    let schema = &proposal_tool["inputSchema"];
    assert_eq!(schema["additionalProperties"], false);
    let proposal_fields = schema["properties"]
        .as_object()
        .expect("proposal schema properties")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        proposal_fields,
        BTreeSet::from([
            "control_core_acceptance_criteria",
            "execution_scope",
            "goal_summary",
            "must_stop_points",
            "next_steps",
            "reasoning",
            "risks",
            "scope_note",
            "suggest_workflow",
            "supervisor_acceptance_criteria",
            "tasks",
            "user_goal",
            "worker_acceptance_criteria",
        ])
    );
    for server_owned_field in [
        "project_id",
        "project_root",
        "workflow_id",
        "thread_id",
        "actor_id",
        "user_requirement_snapshot",
    ] {
        assert!(
            schema["properties"].get(server_owned_field).is_none(),
            "{server_owned_field} must remain server-owned"
        );
    }
    assert_eq!(
        schema["properties"]["risks"]["items"]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["properties"]["execution_scope"]["anyOf"][1]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["properties"]["tasks"]["items"]["additionalProperties"],
        false
    );
    let direct_submit_error = call_tool_with_invoker(
        &resident_config,
        json!({"name": "submit_proposal", "arguments": {}}),
        &FakeInvoker,
    )
    .expect_err("resident-prefixed run without durable binding must be rejected");
    assert!(
        direct_submit_error.contains("未找到可验证的常驻主管会话绑定"),
        "{direct_submit_error}"
    );
    for rejected in [
        "dispatch_worker",
        "follow_up_worker",
        "final_mark",
        "report_user",
    ] {
        assert!(
            fixture.call(rejected, json!({})).is_err(),
            "{rejected} must not be an MCP action"
        );
    }
}
