    #[test]
    fn workflow_dispatch_director_review_rejects_invalid_state_and_dispatch() {
        let dir = std::env::temp_dir().join(format!(
            "director-review-rejects-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "总指导拒绝夹具");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");

        let missing_dispatch = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                "dispatch:missing",
                "accepted",
            ),
        );
        assert!(missing_dispatch.is_err());
        assert!(missing_dispatch
            .unwrap_err()
            .contains("工作项当前状态不是待回收"));

        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let prepared_dispatch_id = append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "prepared",
            "thread-001",
        );
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "running",
            ),
        )
        .expect("work item should be running");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_for_review",
            ),
        )
        .expect("work item should be ready for review");

        let not_completed = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                &prepared_dispatch_id,
                "accepted",
            ),
        );
        assert!(not_completed.is_err());
        assert!(not_completed
            .unwrap_err()
            .contains("派发记录不是 completed"));

        let invalid_decision_dispatch_id = append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );
        let invalid_decision = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                &invalid_decision_dispatch_id,
                "approve-ish",
            ),
        );
        assert!(invalid_decision.is_err());
        assert!(invalid_decision.unwrap_err().contains("未知总指导回收结论"));
        let updated = read_json_file(&path);
        assert_eq!(updated["reviews"].as_array().unwrap().len(), 0);

        let _ = fs::remove_dir_all(dir);
    }
