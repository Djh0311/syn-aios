#[test]
fn dispatched_writer_and_bound_reviewer_raw_evidence_can_finalize_pass() {
    let fixture = Fixture::new();
    let mut authorization_store =
        crate::plan_authorization_store::load_store(&fixture.state_path, now_ms())
            .expect("authorization store");
    authorization_store.authorizations[0].scope.allowed_checks =
        vec!["交付文件必须为 8 字节且末尾无换行".to_string()];
    authorization_store.revision += 1;
    authorization_store.updated_at_ms = now_ms();
    let authorization_path =
        crate::plan_authorization_store::sidecar_path(&fixture.state_path).expect("auth path");
    fs::write(
        authorization_path,
        serde_json::to_vec(&authorization_store).expect("authorization json"),
    )
    .expect("write authorization store");

    fixture
        .control_core(
            "dispatch_worker",
            json!({
                "project_root": PROJECT,
                "workflow_id": WORKFLOW,
                "authorization_id": AUTH,
                "node_id": NODE,
                "work_item_id": "work-item:writer",
                "allowed_write": [PROJECT]
            }),
        )
        .expect("dispatch writer");
    fixture
        .control_core(
            "dispatch_worker",
            json!({
                "project_root": PROJECT,
                "workflow_id": WORKFLOW,
                "authorization_id": AUTH,
                "node_id": NODE,
                "work_item_id": "work-item:readonly-reviewer",
                "allowed_write": []
            }),
        )
        .expect("dispatch readonly reviewer");

    let mut workflow_state: Value =
        serde_json::from_slice(&fs::read(&fixture.state_path).expect("workflow state"))
            .expect("workflow state json");
    workflow_state["artifacts"] = json!([{
        "artifact_type": "task_package",
        "source_ref": "work-item:readonly-reviewer",
        "project_id": crate::project_id(PROJECT),
        "workflow_id": WORKFLOW,
        "project_director_planned_task_id": crate::supervisor_session_launcher::supervisor_pilot_readonly_reviewer_task_id(AUTH),
        "title": "只读复核：站4字节级实证",
        "task_name": "只读复核：站4字节级实证",
        "task_goal": format!(
            "独立复核字节；{}",
            crate::supervisor_session_launcher::SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_MARKER
        ),
        "allowed_write": [],
        "forbidden_actions": [
            crate::supervisor_session_launcher::SUPERVISOR_PILOT_READONLY_BYTE_REVIEW_MARKER
        ]
    }]);
    fs::write(
        &fixture.state_path,
        serde_json::to_vec(&workflow_state).expect("workflow state json"),
    )
    .expect("write reviewer task package");

    let reviewer = load_store(&fixture.config)
        .expect("supervisor store")
        .sessions[0]
        .workers
        .iter()
        .find(|worker| worker.work_item_id == "work-item:readonly-reviewer")
        .cloned()
        .expect("dispatched reviewer");
    let raw_review_return = "```json\n{\"did\":\"只读复核完成\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"wc -c + sha256sum + tail\"],\"review_evidence\":[{\"path\":\"/p/output.txt\",\"byte_count\":9,\"sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"trailing_newline\":true,\"read_method\":\"wc -c + sha256sum + tail\"}]}\n```";
    let projected = normalized_worker_report_from_raw(&reviewer, raw_review_return)
        .expect("parse raw reviewer return");
    update_store(
        &fixture.config,
        "seed-dispatched-reviewer-raw-return",
        |store| {
            session_mut(store, &fixture.config.run_id)
                .workers
                .iter_mut()
                .find(|worker| worker.work_item_id == "work-item:readonly-reviewer")
                .expect("reviewer worker")
                .last_report = Some(projected);
            Ok(())
        },
    )
    .expect("store reviewer projection");

    let result = fixture
        .control_core(
            "finalize",
            json!({
                "project_root": PROJECT,
                "workflow_id": WORKFLOW,
                "authorization_id": AUTH,
                "verdict": "pass",
                "reason": "复核原始回程已投影为结构化实证"
            }),
        )
        .expect("bound reviewer evidence may finalize pass");
    assert_eq!(result["verdict"], "pass");
    assert_eq!(
        result["review_evidence"]["required_fields"],
        json!(["byte_count", "trailing_newline"])
    );
    assert_eq!(
        result["review_evidence"]["readonly_reviewer_worker_ids"],
        json!(["worker-1"])
    );
}
