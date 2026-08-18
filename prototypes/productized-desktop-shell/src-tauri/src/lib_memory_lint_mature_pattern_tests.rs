    #[test]
    fn memory_lint_blocks_conflicting_candidate_adoption() {
        let dir = test_temp_dir("memory-lint-conflicting-adoption");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-conflicting-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:conflict:formal",
            "evidence",
            "2026-06-04T02:00:00Z",
            "write-lint-conflict-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口缓存禁止启用");

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(1),
            ),
            "2026-06-04T02:00:02Z",
            "write-lint-conflict-adoption",
            "write-lint-conflict-formal-adoption",
        )
        .unwrap_err();

        assert!(err.contains("memory_lint_blocking_findings"));
        let lint_store = memory_lint_store::load_store(&path, "2026-06-04T02:00:03Z")
            .expect("lint store should load");
        assert_eq!(lint_store.findings.len(), 1);
        assert_eq!(
            lint_store.findings[0].finding_type,
            MemoryLintFindingType::CandidateConflictsWithActiveMemory
        );
        assert_eq!(
            lint_store.findings[0].severity,
            MemoryLintFindingSeverity::Blocking
        );
        assert_eq!(lint_store.runs[0].status, MemoryLintRunStatus::Blocked);
        let formal_store = formal_memory_store::load_store(&path, "2026-06-04T02:00:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(formal_store.versions.len(), 1);
        assert_eq!(
            formal_store
                .audit_events
                .iter()
                .filter(|event| event.event_type == "memory_candidate_adopted_to_formal_memory")
                .count(),
            0
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_allows_non_conflicting_candidate_adoption() {
        let dir = test_temp_dir("memory-lint-non-conflicting-adoption");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-non-conflicting-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:non-conflict:formal",
            "evidence",
            "2026-06-04T02:10:00Z",
            "write-lint-non-conflict-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口文档需要保留验收步骤");

        let adopted = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(1),
            ),
            "2026-06-04T02:10:02Z",
            "write-lint-non-conflict-adoption",
            "write-lint-non-conflict-formal-adoption",
        )
        .expect("non-conflicting candidate should adopt");

        assert_eq!(
            adopted.candidate_status,
            MemoryLifecycleStatus::CandidateConfirmed
        );
        let lint_store = memory_lint_store::load_store(&path, "2026-06-04T02:10:03Z")
            .expect("lint store should load");
        assert_eq!(lint_store.runs[0].status, MemoryLintRunStatus::Succeeded);
        assert_eq!(lint_store.findings.len(), 0);
        let formal_store = formal_memory_store::load_store(&path, "2026-06-04T02:10:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 2);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_duplicate_claim_generates_finding() {
        let dir = test_temp_dir("memory-lint-duplicate-claim");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-duplicate-claim-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled",
            "source:lint:duplicate:001",
            "evidence",
            "2026-06-04T02:20:00Z",
            "write-lint-duplicate-001",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled now",
            "source:lint:duplicate:002",
            "evidence",
            "2026-06-04T02:20:01Z",
            "write-lint-duplicate-002",
        );
        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T02:20:02Z",
            "write-lint-duplicate-run",
        )
        .expect("lint run should succeed");

        let duplicate = output
            .new_findings
            .iter()
            .find(|finding| finding.finding_type == MemoryLintFindingType::DuplicateClaim)
            .expect("duplicate claim finding should be present");
        assert_eq!(duplicate.severity, MemoryLintFindingSeverity::NeedsReview);
        assert!(duplicate.summary.contains("0.80"));
        assert!(output
            .new_findings
            .iter()
            .any(|finding| finding.finding_type == MemoryLintFindingType::DerivedIndexStale));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_authority_superseded_does_not_mutate_formal_memory() {
        let dir = test_temp_dir("memory-lint-authority-superseded");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-authority-superseded-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存策略需要保留验收记录",
            "source:lint:authority:old",
            "evidence",
            "2026-06-04T02:30:00Z",
            "write-lint-authority-old",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存策略需要保留验收记录",
            "source:lint:authority:new",
            "user_confirmed",
            "2026-06-04T02:30:01Z",
            "write-lint-authority-new",
        );
        let before = formal_memory_store::load_store(&path, "2026-06-04T02:30:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T02:30:02Z",
            "write-lint-authority-run",
        )
        .expect("lint run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-04T02:30:03Z")
            .expect("formal store should load");

        assert!(output
            .new_findings
            .iter()
            .any(|finding| finding.finding_type == MemoryLintFindingType::AuthoritySuperseded));
        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert!(after
            .records
            .iter()
            .all(|record| record.status == MemoryLifecycleStatus::MemoryActive));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_revoked_source_excludes_task_packet_memory() {
        let dir = test_temp_dir("memory-lint-revoked-source");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-revoked-source-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口权限验收需要保留",
            "source:lint:revoked",
            "evidence",
            "2026-06-04T02:40:00Z",
            "write-lint-revoked-formal",
        );
        let mut input =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::TaskPacketGuard);
        input.revoked_source_ids = vec!["source:lint:revoked".to_string()];
        run_memory_lint_at(
            &path,
            &input,
            "2026-06-04T02:40:01Z",
            "write-lint-revoked-run",
        )
        .expect("lint run should succeed");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口 权限 验收"),
            "2026-06-04T02:40:02Z",
        )
        .expect("task memory packet should preview");

        assert_eq!(output.preview.included_memories.len(), 0);
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
            1
        );
        assert!(output
            .preview
            .excluded_items
            .iter()
            .any(|item| item.detail.contains("memory lint open blocking finding")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_open_blocking_finding_excludes_task_packet_memory() {
        let dir = test_temp_dir("memory-lint-blocking-packet");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-blocking-packet-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:blocking:formal",
            "evidence",
            "2026-06-04T02:50:00Z",
            "write-lint-blocking-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口缓存禁止启用");
        let mut input = fixture_memory_lint_run_input(
            project_root,
            MemoryLintRunIntent::CandidateAdoptionGuard,
        );
        input.candidate_key = Some(candidate.candidate_key);
        run_memory_lint_at(
            &path,
            &input,
            "2026-06-04T02:50:02Z",
            "write-lint-blocking-run",
        )
        .expect("lint run should write blocking finding");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口 缓存"),
            "2026-06-04T02:50:03Z",
        )
        .expect("task memory packet should preview");

        assert_eq!(output.preview.included_memories.len(), 0);
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
            1
        );
        assert!(output
            .preview
            .warnings
            .contains(&"memory_lint_blocking_findings_excluded".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_maintenance_run_is_readonly_for_formal_memory() {
        let dir = test_temp_dir("memory-lint-maintenance-readonly");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-maintenance-readonly-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled",
            "source:lint:readonly:001",
            "evidence",
            "2026-06-04T03:00:00Z",
            "write-lint-readonly-001",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled now",
            "source:lint:readonly:002",
            "evidence",
            "2026-06-04T03:00:01Z",
            "write-lint-readonly-002",
        );
        let before = formal_memory_store::load_store(&path, "2026-06-04T03:00:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:00:02Z",
            "write-lint-readonly-run",
        )
        .expect("lint run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-04T03:00:03Z")
            .expect("formal store should load");
        let summary = memory_lint_store::summarize_store(&output.store);

        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert_eq!(summary.open_count, 2);
        assert_eq!(summary.needs_review_count, 1);
        assert_eq!(summary.info_count, 1);
        assert!(summary.recent_maintenance_report.is_some());
        assert!(summary.display_text.contains("不会自动修改正式记忆"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_damaged_json_is_not_overwritten() {
        let dir = test_temp_dir("memory-lint-damaged-json");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-damaged-json-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let sidecar = memory_lint_store::sidecar_path(&path).expect("lint sidecar path");
        fs::write(&sidecar, "{ damaged json").expect("damaged lint sidecar should write");

        let err = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:10:00Z",
            "write-lint-damaged-run",
        )
        .unwrap_err();

        assert!(err.contains("memory lint sidecar JSON 损坏"));
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "{ damaged json");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_revision_conflict_is_rejected() {
        let dir = test_temp_dir("memory-lint-revision-conflict");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-revision-conflict-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:20:00Z",
            "write-lint-revision-first",
        )
        .expect("first lint run should write store");
        let mut stale =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview);
        stale.expected_lint_store_revision = Some(0);

        let err = run_memory_lint_at(
            &path,
            &stale,
            "2026-06-04T03:20:01Z",
            "write-lint-revision-stale",
        )
        .unwrap_err();

        assert!(err.contains("memory_lint_store_conflict"));
        let store = memory_lint_store::load_store(&path, "2026-06-04T03:20:02Z")
            .expect("lint store should load");
        assert_eq!(store.revision, 1);
        assert_eq!(store.runs.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_source_secret_and_index_findings_readonly() {
        let dir = test_temp_dir("memory-maintenance-source-secret-index");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-source-secret-index";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "缺来源记忆不能召回",
            "source:m11:missing",
            "evidence",
            "2026-06-05T11:00:00Z",
            "write-m11-missing-formal",
        );
        create_formal_memory_for_task(
            &path,
            project_root,
            "secret token 不能外发",
            "正文包含 password 和 secret token，仅用于维护扫描测试。",
            "2026-06-05T11:00:01Z",
            "write-m11-secret-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[1].source_refs[0].source_id = Some("source:m11:secret".to_string());
        });
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![];
            store.records[1].source_refs[0].sensitive_level = "secret".to_string();
            store.records[1].scope.model_export_policy = "local_only".to_string();
        });
        let before = formal_memory_store::load_store(&path, "2026-06-05T11:00:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T11:00:02Z",
            "write-m11-maintenance-run",
        )
        .expect("maintenance run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-05T11:00:03Z")
            .expect("formal store should load");

        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert!(output.run.report_id.is_some());
        let report = output
            .report
            .as_ref()
            .expect("maintenance report should exist");
        assert_eq!(output.store.maintenance_reports.len(), 1);
        assert!(report.display_text.contains("维护任务只生成 finding"));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::MissingSource
                    && finding.severity == MemoryLintFindingSeverity::Blocking
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::SensitiveExportRisk
                    && finding.severity == MemoryLintFindingSeverity::Blocking
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::PrivateSourceRisk
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::DerivedIndexStale
                    && finding.severity == MemoryLintFindingSeverity::Info
            ));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::SourceIntegrity
            && check.blocking_count > 0));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::SensitiveExportRisk
            && check.finding_count > 0));

        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "secret token 缺来源"),
            "2026-06-05T11:00:04Z",
        )
        .expect("task packet should preview");
        assert_eq!(
            excluded_reason_count(&packet, TaskMemoryPacketExclusionReason::Conflicted),
            2
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_entity_drift_and_relation_revoked_readonly() {
        let dir = test_temp_dir("memory-maintenance-entity-relation-drift");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-entity-relation-drift";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "实体漂移维护测试",
            "source:m11:entity",
            "evidence",
            "2026-06-05T11:10:00Z",
            "write-m11-entity-formal",
        );
        memory_entity_relation_store::with_locked_store(
            &path,
            "2026-06-05T11:10:01Z",
            "write-m11-entity-relation-store",
            |store| {
                store.project_id = Some(project_id(project_root));
                store.workflow_id = Some(default_workflow_id(project_root));
                store.merge_candidates.push(MemoryEntityMergeCandidate {
                    merge_candidate_id: "merge-candidate:m11:codex".to_string(),
                    left_entity_candidate_id: "entity-candidate:left".to_string(),
                    right_entity_candidate_id: "entity-candidate:right".to_string(),
                    left_label: "Codex CLI".to_string(),
                    right_label: "codex tool".to_string(),
                    normalized_key: "codex".to_string(),
                    source_kind: MemoryRelationSourceKind::SimilarityHit,
                    status: MemoryRelationStatus::Candidate,
                    requires_user_confirmation: true,
                    reason: "alias / dedupe 候选需要人工复核。".to_string(),
                    created_at: "2026-06-05T11:10:01Z".to_string(),
                    warnings: vec![],
                });
                store.relations.push(MemoryRelation {
                    relation_id: "relation:m11:revoked".to_string(),
                    relation_kind: MemoryRelationKind::Semantic,
                    subject_entity_id: "entity:codex".to_string(),
                    object_entity_id: "entity:task".to_string(),
                    subject_label: "Codex CLI".to_string(),
                    object_label: "任务包".to_string(),
                    predicate: "explains".to_string(),
                    source_kind: MemoryRelationSourceKind::Manual,
                    source_refs: vec![MemoryRelationSource {
                        source_kind: MemoryRelationSourceKind::Manual,
                        source_id: Some("source:relation:revoked".to_string()),
                        source_path: Some("docs/relation.md".to_string()),
                        source_title: Some("关系来源".to_string()),
                        authority_level: "evidence".to_string(),
                        sensitive_level: "project".to_string(),
                    }],
                    status: MemoryRelationStatus::Confirmed,
                    confirmed_by: "project_director".to_string(),
                    confirmation_role: "project_director".to_string(),
                    confirmation_reason: "测试关系来源撤回。".to_string(),
                    created_at: "2026-06-05T11:10:01Z".to_string(),
                    updated_at: "2026-06-05T11:10:01Z".to_string(),
                    warnings: vec![],
                });
                store.revision += 1;
                Ok(())
            },
        )
        .expect("entity relation store should write test fixture");
        let entity_before = memory_entity_relation_store::load_store(&path, "2026-06-05T11:10:02Z")
            .expect("entity relation store should load");

        let mut input =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun);
        input.revoked_source_ids = vec!["source:relation:revoked".to_string()];
        let output = run_memory_lint_at(
            &path,
            &input,
            "2026-06-05T11:10:02Z",
            "write-m11-entity-maintenance-run",
        )
        .expect("maintenance run should succeed");
        let entity_after = memory_entity_relation_store::load_store(&path, "2026-06-05T11:10:03Z")
            .expect("entity relation store should load");

        assert_eq!(entity_after, entity_before);
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::EntityDrift
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::RelationSourceRevoked
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        let report = output
            .report
            .as_ref()
            .expect("maintenance report should exist");
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::EntityRelationDrift
            && check.needs_review_count > 0));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::PermissionRevocation
            && check.needs_review_count > 0));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory() {
        let dir = test_temp_dir("memory-maintenance-mature-pattern");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-mature-pattern";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        for (index, claim) in ["重复验收模式 A", "重复验收模式 B", "重复验收模式 C"]
            .iter()
            .enumerate()
        {
            let mut input = fixture_bound_memory_candidate_input(project_root);
            input.claim = claim.to_string();
            input.body = format!("{claim} 的候选说明。");
            let created = memory_candidate_store::create_candidate(
                &path,
                &input,
                &format!("2026-06-05T11:20:0{index}Z"),
                &format!("write-m11-mature-candidate-{index}"),
            )
            .expect("memory candidate should be created");
            memory_candidate_store::record_decision(
                &path,
                &RecordMemoryCandidateDecisionInput {
                    project_root: project_root.to_string(),
                    candidate_key: created.candidate.candidate_key,
                    requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                    reason: "确认保留候选；等待后续成熟模式人工复核。".to_string(),
                    actor_id: "project_director".to_string(),
                    actor_role: "project_director".to_string(),
                    expected_store_revision: Some(created.store_revision),
                },
                &format!("2026-06-05T11:20:1{index}Z"),
                &format!("write-m11-mature-candidate-confirm-{index}"),
            )
            .expect("memory candidate should be confirmed");
        }
        let formal_before = formal_memory_store::load_store(&path, "2026-06-05T11:20:00Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T11:20:01Z",
            "write-m11-mature-pattern-run",
        )
        .expect("maintenance run should succeed");
        let formal_after = formal_memory_store::load_store(&path, "2026-06-05T11:20:02Z")
            .expect("formal store should load");

        assert_eq!(formal_after.records, formal_before.records);
        assert_eq!(formal_after.versions, formal_before.versions);
        assert_eq!(formal_after.revision, formal_before.revision);
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::MaturePatternSignal
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
                    && finding.summary.contains("不会自动成为规则")
            ));
        assert!(output
            .report
            .as_ref()
            .expect("maintenance report should exist")
            .check_summaries
            .iter()
            .any(
                |check| check.check_kind == MemoryMaintenanceCheckKind::MaturePatternSignal
                    && check.needs_review_count > 0
            ));

        let _ = fs::remove_dir_all(dir);
    }

    fn fixture_m12_preview_input(project_root: &str) -> PreviewMaturePatternsInput {
        PreviewMaturePatternsInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
        }
    }

    fn prepare_m12_repeated_candidate_fixture(path: &Path, project_root: &str) {
        for claim in [
            "repeat review failure requires checklist step before release alpha",
            "repeat review failure requires checklist step before release beta",
            "repeat review failure requires checklist step before release gamma",
        ] {
            create_confirmed_candidate_with_claim(path, project_root, claim);
        }
        run_memory_lint_at(
            path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T12:00:00Z",
            "write-m12-maintenance-signal",
        )
        .expect("maintenance run should create mature pattern signal");
    }

    #[test]
    fn mature_pattern_preview_derives_candidates_and_memory_cluster_reports_readonly() {
        let dir = test_temp_dir("mature-pattern-preview-readonly");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-preview-readonly";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let trusted = mature_pattern_governance::trusted_canonical_fixture(project_root);

        let preview = mature_pattern_governance::preview_mature_patterns_for_canonical_project(
            &path,
            &trusted,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:00:01Z",
        )
        .expect("mature pattern preview should build");

        assert!(preview
            .mature_pattern_candidates
            .iter()
            .any(|candidate| candidate.pattern_kind == "maintenance_signal"
                && candidate.status == MaturePatternCandidateStatus::Candidate
                && candidate.requires_user_confirmation));
        assert!(preview
            .mature_pattern_candidates
            .iter()
            .any(|candidate| candidate.pattern_kind == "repeated_candidate"
                && candidate.member_refs.len() >= 2));
        assert!(preview
            .cluster_reports
            .iter()
            .any(|report| report.member_refs.len() >= 2
                && report
                    .display_text
                    .contains("报告可下钻来源，但不是正式事实")));
        assert!(preview
            .acceptance_summary
            .display_text
            .contains("M13 最终验收仍后置"));
        assert!(
            !mature_pattern_store::sidecar_path(&path)
                .expect("pattern sidecar path")
                .exists(),
            "preview must not write memory-patterns sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_cluster_report_and_unconfirmed_mature_pattern_do_not_enter_task_packet() {
        let dir = test_temp_dir("memory-cluster-report-not-formal");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-cluster-report-not-formal";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let trusted = mature_pattern_governance::trusted_canonical_fixture(project_root);
        let preview = mature_pattern_governance::preview_mature_patterns_for_canonical_project(
            &path,
            &trusted,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:10:00Z",
        )
        .expect("mature pattern preview should build");

        assert!(!preview.mature_pattern_candidates.is_empty());
        assert!(!preview.cluster_reports.is_empty());
        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "repeat review failure checklist"),
            "2026-06-05T12:10:01Z",
        )
        .expect("task memory packet should build");

        assert!(packet.preview.included_memories.is_empty());
        assert!(packet
            .preview
            .excluded_items
            .iter()
            .all(|item| item.source_kind != "memory_cluster_report"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mature_pattern_user_confirmation_writes_formal_memory_and_task_packet_can_recall() {
        let dir = test_temp_dir("mature-pattern-user-confirmation");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-user-confirmation";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let trusted = mature_pattern_governance::trusted_canonical_fixture(project_root);
        let preview = mature_pattern_governance::preview_mature_patterns_for_canonical_project(
            &path,
            &trusted,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:20:00Z",
        )
        .expect("mature pattern preview should build");
        let candidate = preview
            .mature_pattern_candidates
            .iter()
            .find(|candidate| candidate.pattern_kind == "repeated_candidate")
            .expect("repeated candidate should exist")
            .clone();

        let blocked = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &path,
            &trusted,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate.candidate_id.clone(),
                decision: MaturePatternDecisionKind::ConfirmAsFormalMemory,
                actor_id: "project-director-m12".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "项目主管尝试确认成熟模式，应被拒绝。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: Some(0),
            },
            "2026-06-05T12:20:01Z",
            "write-m12-project-director-blocked",
            "write-m12-formal-blocked",
        )
        .unwrap_err();
        assert!(blocked.contains("必须由用户确认"));

        let output = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &path,
            &trusted,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate.candidate_id.clone(),
                decision: MaturePatternDecisionKind::ConfirmAsFormalMemory,
                actor_id: "user-m12".to_string(),
                actor_role: "user".to_string(),
                confirmed_by: Some("user".to_string()),
                reason: "用户确认该重复评审失败模式可作为成熟模式正式记忆。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: Some(0),
            },
            "2026-06-05T12:20:02Z",
            "write-m12-user-confirm",
            "write-m12-formal-confirm",
        )
        .expect("user confirmation should write formal memory");

        assert_eq!(
            output.candidate.status,
            MaturePatternCandidateStatus::Confirmed
        );
        let formal_gate = output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "formal_memory")
            .expect("formal memory gate should exist");
        assert_eq!(formal_gate.status, "passed");
        assert!(
            formal_gate
                .evidence
                .contains("record 1 / version 1 / audit 1"),
            "formal memory gate should use fresh formal store after mature pattern formalization"
        );
        let task_packet_gate = output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "task_packet")
            .expect("task packet gate should exist");
        assert_eq!(task_packet_gate.status, "passed");
        assert!(task_packet_gate.blocking_reason.is_none());
        let formal_output = output
            .formal_memory_output
            .expect("formal mature pattern memory should be written");
        assert_eq!(formal_output.record.memory_type, "mature_pattern");
        assert_eq!(formal_output.record.scope.scope_type, "global");
        assert!(!formal_output.record.source_refs.is_empty());
        assert_eq!(
            formal_output.audit_event.event_type,
            "mature_pattern_user_confirmed_to_formal_memory"
        );
        let formal_store = formal_memory_store::load_store(&path, "2026-06-05T12:20:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(formal_store.versions.len(), 1);
        assert_eq!(formal_store.audit_events.len(), 1);

        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "repeat review failure checklist"),
            "2026-06-05T12:20:04Z",
        )
        .expect("task memory packet should build");
        assert_eq!(packet.preview.included_memories.len(), 1);
        assert_eq!(
            packet.preview.included_memories[0].memory_type,
            "mature_pattern"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mature_pattern_reject_quarantine_revision_and_damaged_json_do_not_mutate_formal_memory() {
        let dir = test_temp_dir("mature-pattern-reject-conflict-damaged");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-reject-conflict-damaged";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let trusted = mature_pattern_governance::trusted_canonical_fixture(project_root);
        let preview = mature_pattern_governance::preview_mature_patterns_for_canonical_project(
            &path,
            &trusted,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:30:00Z",
        )
        .expect("mature pattern preview should build");
        let candidate_id = preview.mature_pattern_candidates[0].candidate_id.clone();

        let conflict = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &path,
            &trusted,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate_id.clone(),
                decision: MaturePatternDecisionKind::Reject,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "expected revision mismatch should fail".to_string(),
                expected_pattern_store_revision: Some(99),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:01Z",
            "write-m12-revision-conflict",
            "write-m12-formal-unused",
        )
        .unwrap_err();
        assert!(conflict.contains("memory_pattern_store_conflict"));

        let reject_output = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &path,
            &trusted,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id,
                decision: MaturePatternDecisionKind::Reject,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "全局主管拒绝成熟模式候选，但不删除来源。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:02Z",
            "write-m12-reject",
            "write-m12-formal-unused-2",
        )
        .expect("reject should write pattern sidecar only");
        assert!(reject_output.formal_memory_output.is_none());
        let reject_formal_gate = reject_output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "formal_memory")
            .expect("formal memory gate should exist");
        assert_eq!(reject_formal_gate.status, "blocked");
        assert!(
            reject_formal_gate
                .evidence
                .contains("record 0 / version 0 / audit 0"),
            "reject summary should not report fresh formal memory"
        );
        let formal_store = formal_memory_store::load_store(&path, "2026-06-05T12:30:03Z")
            .expect("formal store should load");
        assert!(formal_store.records.is_empty());
        let pattern_store = mature_pattern_store::load_store(&path, "2026-06-05T12:30:03Z")
            .expect("pattern store should load");
        assert_eq!(pattern_store.revision, 1);
        assert_eq!(
            pattern_store.mature_pattern_candidates[0].status,
            MaturePatternCandidateStatus::Rejected
        );
        let quarantine_candidate_id = preview.mature_pattern_candidates[1].candidate_id.clone();
        let quarantine_output = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &path,
            &trusted,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: quarantine_candidate_id,
                decision: MaturePatternDecisionKind::Quarantine,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "全局主管隔离成熟模式候选，但不写正式记忆。".to_string(),
                expected_pattern_store_revision: Some(pattern_store.revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:04Z",
            "write-m12-quarantine",
            "write-m12-formal-unused-4",
        )
        .expect("quarantine should write pattern sidecar only");
        assert!(quarantine_output.formal_memory_output.is_none());
        let formal_store_after_quarantine =
            formal_memory_store::load_store(&path, "2026-06-05T12:30:05Z")
                .expect("formal store should load after quarantine");
        assert!(formal_store_after_quarantine.records.is_empty());

        let damaged_dir = test_temp_dir("mature-pattern-damaged-json");
        let damaged_path = damaged_dir.join("workflow-state.v0.json");
        let damaged_project_root = "/tmp/mature-pattern-damaged-json";
        bootstrap_project_workflow_at(&damaged_path, &fixture_project(damaged_project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&damaged_path, damaged_project_root);
        let damaged_trusted =
            mature_pattern_governance::trusted_canonical_fixture(damaged_project_root);
        let damaged_preview = mature_pattern_governance::preview_mature_patterns_for_canonical_project(
            &damaged_path,
            &damaged_trusted,
            &fixture_m12_preview_input(damaged_project_root),
            "2026-06-05T12:31:00Z",
        )
        .expect("mature pattern preview should build");
        let sidecar = mature_pattern_store::sidecar_path(&damaged_path).expect("sidecar path");
        fs::write(&sidecar, "{ damaged json").expect("test should write damaged pattern store");
        let damaged = mature_pattern_governance::record_mature_pattern_decision_for_canonical_project(
            &damaged_path,
            &damaged_trusted,
            &RecordMaturePatternDecisionInput {
                project_root: damaged_project_root.to_string(),
                candidate_id: damaged_preview.mature_pattern_candidates[0]
                    .candidate_id
                    .clone(),
                decision: MaturePatternDecisionKind::Quarantine,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "损坏 JSON 不应被覆盖。".to_string(),
                expected_pattern_store_revision: Some(damaged_preview.store_revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:31:01Z",
            "write-m12-damaged-json",
            "write-m12-formal-unused-3",
        )
        .unwrap_err();
        assert!(damaged.contains("JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&sidecar).expect("damaged sidecar should remain"),
            "{ damaged json"
        );

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(damaged_dir);
    }

