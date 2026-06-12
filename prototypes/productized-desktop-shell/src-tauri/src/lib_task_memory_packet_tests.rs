#[test]
fn task_memory_packet_includes_active_formal_memory() {
    let dir = test_temp_dir("task-memory-packet-include-active");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-include-active";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let record = create_formal_memory_for_task(
        &path,
        project_root,
        "接口完成事实可供后续任务使用",
        "接口实现已经完成，后续 worker 可以基于该正式记忆继续处理接口验收。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-include-active",
    );

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "接口 验收"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    assert_eq!(output.preview.included_memories.len(), 1);
    assert_eq!(
        output.preview.included_memories[0].memory_id,
        record.memory_id
    );
    assert!(output.preview.included_memories[0]
        .retrieval_reason
        .contains("active formal memory"));
    assert!(output
        .warnings
        .contains(&"preview_only_not_injected".to_string()));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_candidates_as_unconfirmed() {
    let dir = test_temp_dir("task-memory-packet-candidate");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-candidate";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let candidate = memory_candidate_store::create_candidate(
        &path,
        &fixture_bound_memory_candidate_input(project_root),
        "2026-06-04T01:00:00Z",
        "write-task-memory-candidate",
    )
    .expect("candidate should be created");

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "候选"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(
            &output,
            TaskMemoryPacketExclusionReason::CandidateUnconfirmed
        ),
        1
    );
    assert!(output.preview.review_materials.iter().any(|material| {
        material.source_kind == "memory_candidate"
            && material.source_id == candidate.candidate.candidate_key
    }));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_observation_as_not_formal() {
    let dir = test_temp_dir("task-memory-packet-observation");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-observation";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    let observation = create_recorded_observation(&path, project_root);

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "worker 汇报"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(
            &output,
            TaskMemoryPacketExclusionReason::ObservationNotFormalMemory
        ),
        1
    );
    assert!(output.preview.review_materials.iter().any(|material| {
        material.source_kind == "observation"
            && material.source_id == observation.observation.observation_key
    }));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_inactive_formal_memories() {
    let dir = test_temp_dir("task-memory-packet-inactive");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-inactive";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    for (index, claim) in [
        "接口 冲突正式记忆",
        "接口 废弃正式记忆",
        "接口 冻结正式记忆",
        "接口 归档正式记忆",
    ]
    .iter()
    .enumerate()
    {
        create_formal_memory_for_task(
            &path,
            project_root,
            claim,
            "接口相关但状态不允许进入任务记忆包。",
            &format!("2026-06-04T01:00:0{index}Z"),
            &format!("write-task-memory-inactive-{index}"),
        );
    }
    mutate_formal_store(&path, |store| {
        store.records[0].status = MemoryLifecycleStatus::MemoryConflicted;
        store.records[1].status = MemoryLifecycleStatus::MemoryDeprecated;
        store.records[2].status = MemoryLifecycleStatus::MemoryFrozen;
        store.records[3].status = MemoryLifecycleStatus::MemoryArchived;
    });

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "接口"),
        "2026-06-04T01:00:10Z",
    )
    .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
        1
    );
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Stale),
        3
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_model_export_blocked() {
    let dir = test_temp_dir("task-memory-packet-export-blocked");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-export-blocked";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "接口 blocked export 正式记忆",
        "该正式记忆只允许本地上下文，不允许外发模型上下文。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-export-blocked",
    );
    mutate_formal_store(&path, |store| {
        store.records[0].scope.model_export_policy = "blocked".to_string();
    });
    let mut input = fixture_task_memory_packet_input(project_root, "接口");
    input.model_context_policy = "external_model_context".to_string();

    let output = preview_task_memory_packet_at(&path, &input, "2026-06-04T01:00:01Z")
        .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::ModelExportBlocked),
        1
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_permission_blocked() {
    let dir = test_temp_dir("task-memory-packet-permission");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-permission";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "接口 跨项目正式记忆",
        "该记录被测试改为其他项目 scope，应被权限规则排除。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-permission",
    );
    mutate_formal_store(&path, |store| {
        store.records[0].scope.project_id = Some("project:other".to_string());
    });

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "接口"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::PermissionBlocked),
        1
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_token_limit() {
    let dir = test_temp_dir("task-memory-packet-token");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-token";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "接口 大段正式记忆",
        "接口 ".repeat(200).as_str(),
        "2026-06-04T01:00:00Z",
        "write-task-memory-token",
    );
    let mut input = fixture_task_memory_packet_input(project_root, "接口");
    input.max_estimated_tokens = 20;

    let output = preview_task_memory_packet_at(&path, &input, "2026-06-04T01:00:01Z")
        .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::TokenLimit),
        1
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_excludes_not_relevant() {
    let dir = test_temp_dir("task-memory-packet-not-relevant");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-not-relevant";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "缓存策略正式记忆",
        "构建缓存已完成，与支付网关无关。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-not-relevant",
    );

    let output = preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "payment gateway"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    assert!(output.preview.included_memories.is_empty());
    assert_eq!(
        excluded_reason_count(&output, TaskMemoryPacketExclusionReason::NotRelevant),
        1
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_preview_is_readonly() {
    let dir = test_temp_dir("task-memory-packet-readonly");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-readonly";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "接口 readonly 正式记忆",
        "接口正式记忆用于只读预览测试。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-readonly-formal",
    );
    memory_candidate_store::create_candidate(
        &path,
        &fixture_bound_memory_candidate_input(project_root),
        "2026-06-04T01:00:01Z",
        "write-task-memory-readonly-candidate",
    )
    .expect("candidate should be created");
    create_recorded_observation(&path, project_root);
    let formal_before = formal_memory_store::load_store(&path, "2026-06-04T01:00:02Z")
        .expect("formal store should load")
        .revision;
    let candidate_before = memory_candidate_store::load_store(&path, "2026-06-04T01:00:02Z")
        .expect("candidate store should load")
        .revision;
    let observation_before = observation_store::load_store(&path, "2026-06-04T01:00:02Z")
        .expect("observation store should load")
        .revision;

    preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "接口"),
        "2026-06-04T01:00:03Z",
    )
    .expect("task memory packet preview should build");

    assert_eq!(
        formal_memory_store::load_store(&path, "2026-06-04T01:00:04Z")
            .expect("formal store should load")
            .revision,
        formal_before
    );
    assert_eq!(
        memory_candidate_store::load_store(&path, "2026-06-04T01:00:04Z")
            .expect("candidate store should load")
            .revision,
        candidate_before
    );
    assert_eq!(
        observation_store::load_store(&path, "2026-06-04T01:00:04Z")
            .expect("observation store should load")
            .revision,
        observation_before
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn task_memory_packet_preview_does_not_execute_worker() {
    let dir = test_temp_dir("task-memory-packet-no-worker");
    let path = dir.join("workflow-state.v0.json");
    let project_root = "/tmp/task-memory-packet-no-worker";
    bootstrap_project_workflow_at(&path, &fixture_project(project_root))
        .expect("workflow state should include project");
    create_formal_memory_for_task(
        &path,
        project_root,
        "接口 no worker 正式记忆",
        "接口正式记忆用于验证预览不会创建派发。",
        "2026-06-04T01:00:00Z",
        "write-task-memory-no-worker",
    );
    let before = read_json_file(&path);

    preview_task_memory_packet_at(
        &path,
        &fixture_task_memory_packet_input(project_root, "接口"),
        "2026-06-04T01:00:01Z",
    )
    .expect("task memory packet preview should build");

    let after = read_json_file(&path);
    assert_eq!(after["node_dispatches"], before["node_dispatches"]);
    assert_eq!(after["execution_attempts"], before["execution_attempts"]);
    assert_eq!(
        after["workflow_execution_controls"],
        before["workflow_execution_controls"]
    );
    assert_eq!(after, before, "preview must not write workflow state");

    let _ = fs::remove_dir_all(dir);
}
