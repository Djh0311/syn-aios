    #[test]
    fn memory_entity_relation_preview_suggests_alias_and_similarity_candidates_readonly() {
        let dir = test_temp_dir("memory-entity-relation-alias-preview");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-alias-preview";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "Codex 工具别名治理",
            "同一工具在来源里可能写作 Codex CLI 或 codex tool。",
            "2026-06-05T10:00:00Z",
            "write-m10-alias-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![
                fixture_m10_memory_source("tool", "tool:codex-cli", "Codex CLI", "project"),
                fixture_m10_memory_source("tool", "tool:codex-tool", "codex tool", "project"),
                fixture_m10_memory_source(
                    "similarity_hit",
                    "similarity:codex",
                    "Codex CLI",
                    "project",
                ),
                fixture_m10_memory_source(
                    "similarity_hit",
                    "similarity:codex-tool",
                    "codex tool",
                    "project",
                ),
            ];
        });

        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:00:01Z",
        )
        .expect("entity relation preview should build");

        assert!(preview
            .entity_candidates
            .iter()
            .any(|candidate| candidate.entity_kind == MemoryEntityKind::Tool
                && candidate.display_name == "Codex CLI"));
        assert!(preview
            .merge_candidates
            .iter()
            .any(|candidate| candidate.reason.contains("alias / dedupe")));
        assert!(preview
            .merge_candidates
            .iter()
            .any(|candidate| candidate.source_kind == MemoryRelationSourceKind::SimilarityHit));
        assert!(
            !memory_entity_relation_store::sidecar_path(&path)
                .expect("entity relation sidecar path")
                .exists(),
            "preview must not write entity relation sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_llm_inferred_causal_relation_stays_candidate() {
        let dir = test_temp_dir("memory-entity-relation-llm-candidate");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-llm-candidate";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "LLM 推断导致任务包变更",
            "LLM inferred causal candidate fixture.",
            "2026-06-05T10:10:00Z",
            "write-m10-llm-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "llm_inferred",
                "llm:relation:001",
                "LLM 因果推断",
                "project",
            )];
        });
        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:10:01Z",
        )
        .expect("llm inferred preview should build");
        let candidate = preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.source_kind == MemoryRelationSourceKind::LlmInferred)
            .expect("llm inferred relation candidate should exist");

        assert_eq!(candidate.relation_kind, MemoryRelationKind::Causal);
        assert_eq!(candidate.status, MemoryRelationStatus::Candidate);
        assert!(candidate.requires_user_confirmation);
        let err = memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "尝试确认 LLM 推断关系，应被拒绝。".to_string(),
                expected_store_revision: Some(preview.store_revision),
            },
            "2026-06-05T10:10:02Z",
            "write-m10-llm-relation",
        )
        .expect_err("llm inferred relation must not become confirmed relation");

        assert!(err.contains("llm_inferred relation"), "{err}");
        assert!(
            !memory_entity_relation_store::sidecar_path(&path)
                .expect("entity relation sidecar path")
                .exists(),
            "rejected llm relation must not write sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_confirmed_causal_relation_explains_task_packet() {
        let dir = test_temp_dir("memory-entity-relation-task-packet");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-task-packet";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let record = create_formal_memory_for_task(
            &path,
            project_root,
            "接口 因果关系正式记忆",
            "接口契约变化导致任务包需要复核。",
            "2026-06-05T10:20:00Z",
            "write-m10-causal-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:contract-change",
                "接口契约资料",
                "project",
            )];
        });
        let before_confirm = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:20:01Z",
        )
        .expect("task packet should build before relation confirmation");
        assert_eq!(
            before_confirm.preview.included_memories[0].memory_id,
            record.memory_id
        );
        assert!(before_confirm.preview.included_memories[0]
            .relation_explanations
            .is_empty());

        let relation_preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:20:02Z",
        )
        .expect("relation preview should build");
        let causal_candidate = relation_preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.relation_kind == MemoryRelationKind::Causal)
            .expect("causal relation candidate should exist");
        let decision = memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: causal_candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "项目主管确认本项目低风险因果关系，用于解释召回原因。".to_string(),
                expected_store_revision: Some(relation_preview.store_revision),
            },
            "2026-06-05T10:20:03Z",
            "write-m10-causal-relation",
        )
        .expect("project director should confirm causal relation");
        assert_eq!(
            decision
                .relation
                .as_ref()
                .expect("confirmed relation should exist")
                .status,
            MemoryRelationStatus::Confirmed
        );

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:20:04Z",
        )
        .expect("task packet should include relation explanation after confirmation");
        let item = &output.preview.included_memories[0];
        assert!(!item.relation_explanations.is_empty());
        assert!(item.retrieval_reason.contains("已确认关系用于解释召回原因"));
        assert_eq!(output.entity_relation_store_revision, 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_secret_relation_source_is_not_exported_to_task_packet() {
        let dir = test_temp_dir("memory-entity-relation-secret-source");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-secret-source";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 secret 关系正式记忆",
            "接口 secret source 导致任务包需复核。",
            "2026-06-05T10:30:00Z",
            "write-m10-secret-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:secret-contract",
                "secret 接口资料",
                "secret",
            )];
        });
        let relation_preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:30:01Z",
        )
        .expect("secret relation preview should build");
        let causal_candidate = relation_preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.relation_kind == MemoryRelationKind::Causal)
            .expect("secret causal relation candidate should exist");
        memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: causal_candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "user-m10".to_string(),
                actor_role: "user".to_string(),
                confirmed_by: Some("user".to_string()),
                reason: "确认 secret source 关系，但任务包解释应被权限过滤。".to_string(),
                expected_store_revision: Some(relation_preview.store_revision),
            },
            "2026-06-05T10:30:02Z",
            "write-m10-secret-relation",
        )
        .expect("secret relation can be recorded but not exported");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:30:03Z",
        )
        .expect("task packet should build");
        assert!(output.preview.included_memories[0]
            .relation_explanations
            .is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_damaged_json_and_revision_conflict_are_rejected() {
        let dir = test_temp_dir("memory-entity-relation-guard");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-guard";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "实体 relation guard 记忆",
            "用于 revision 和 damaged JSON 测试。",
            "2026-06-05T10:40:00Z",
            "write-m10-guard-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:guard",
                "guard 文档",
                "project",
            )];
        });
        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:40:01Z",
        )
        .expect("preview should build");
        let entity_candidate = preview
            .entity_candidates
            .first()
            .expect("entity candidate should exist")
            .clone();
        memory_entity_relation_governance::record_alias_decision(
            &path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: entity_candidate.candidate_id.clone(),
                decision: MemoryEntityAliasDecisionKind::ConfirmAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "确认登记实体候选。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-05T10:40:02Z",
            "write-m10-alias-confirm",
        )
        .expect("alias decision should write sidecar");
        let conflict = memory_entity_relation_governance::record_alias_decision(
            &path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: entity_candidate.candidate_id,
                decision: MemoryEntityAliasDecisionKind::RejectAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "旧 revision 应拒绝。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-05T10:40:03Z",
            "write-m10-alias-conflict",
        )
        .expect_err("stale revision should reject write");
        assert!(conflict.contains("memory_entity_relation_store_conflict"));

        let damaged_dir = test_temp_dir("memory-entity-relation-damaged");
        let damaged_path = damaged_dir.join("workflow-state.v0.json");
        let damaged_sidecar =
            memory_entity_relation_store::sidecar_path(&damaged_path).expect("sidecar path");
        fs::create_dir_all(damaged_sidecar.parent().expect("sidecar parent"))
            .expect("damaged sidecar parent should exist");
        fs::write(&damaged_sidecar, "{not valid json")
            .expect("damaged entity relation sidecar should write");
        let damaged = memory_entity_relation_governance::record_alias_decision(
            &damaged_path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: "entity-candidate:missing".to_string(),
                decision: MemoryEntityAliasDecisionKind::ConfirmAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "损坏 JSON 应拒绝覆盖。".to_string(),
                expected_store_revision: None,
            },
            "2026-06-05T10:40:04Z",
            "write-m10-damaged",
        )
        .expect_err("damaged json should reject write");
        assert!(damaged.contains("JSON 损坏"), "{damaged}");
        assert_eq!(
            fs::read_to_string(&damaged_sidecar).expect("damaged sidecar should remain"),
            "{not valid json"
        );

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(damaged_dir);
    }
