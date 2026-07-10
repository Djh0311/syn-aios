// Tauri command registry split out during Root Treatment R2-B1.
// This file is included at crate root so command wrapper visibility and names stay unchanged.

// Store 卫生维护命令模块（canvas-run 历史残料合法归档）。挂在此处而非 lib.rs：
// 本文件 include! 进 crate root，故此声明等价于在 crate root 挂子模块，且保持 lib.rs 0-diff。
mod store_hygiene;

// worker 回程契约模块（报文契约 + 解析 + 链消费核心）。同 store_hygiene 借道挂载，保持 lib.rs 0-diff。
// 无 tauri command（纯协议/消费逻辑），故只挂 mod、不进 generate_handler!。
mod worker_report;

// B1·全局主管复核（advisory）：agent（读盘→只读 consult→意见）+ 复核记录 sidecar store。
// 同 worker_report 借道挂载，保持 lib.rs 0-diff；命令 run_global_supervisor_review 进 generate_handler!。
mod global_supervisor_agent;
mod global_supervisor_review_store;

// B3·秘书 agent（按需解释·零写入零 store）：读盘装配「待拍板事实」→ 只读 consult → 纯文本。
mod secretary_agent;

// 工作历史·后端读模型（纯只读·跨店按 workflow+时间窗拼单列表）；命令 list_project_run_history 进 generate_handler!。
mod run_history_read_model;

// A·运行错误人话翻译层（C6 观测补强·纯函数·无 tauri command）。供给类判据单一真源在此，
// runner/run_history 委托到它。同 worker_report 借道挂载·保持 lib.rs 0-diff。
mod run_error_translation;

macro_rules! workbench_command_handler {
    () => {
        tauri::generate_handler![
            load_workbench_snapshot,
            query_workbench_page_read_model,
            record_operation_control_decision,
            preview_manual_codex_relay,
            confirm_manual_codex_relay_once,
            run_manual_codex_relay_once,
            run_manual_codex_relay_gui_direct,
            run_manual_codex_relay_gui_direct_new_session,
            stop_manual_codex_relay_attempt,
            poll_manual_codex_relay_attempt,
            load_codex_session_transcript,
            load_codex_session_transcript_page,
            load_codex_session_page,
            load_workflow_state_snapshot,
            load_plan_authorization_store,
            create_plan_authorization,
            record_plan_authorization_user_confirmation,
            record_plan_authorization_global_boundary_review,
            record_global_boundary_review,
            revoke_plan_authorization,
            inspect_auto_dispatch_authorization,
            preview_project_director_task_plan,
            prepare_authorized_auto_dispatch,
            preview_h5_project_workflow_dispatch,
            preview_real_execution_product_command,
            prepare_real_execution_product_command,
            record_real_execution_product_command_decision,
            confirm_real_execution_product_command,
            run_real_execution_product_command_phase_a,
            run_real_execution_product_command_phase_b,
            run_real_execution_product_command_new_session_phase_b,
            run_project_workflow_automation_phase_a,
            run_project_workflow_automation_j2_b_b1,
            run_project_workflow_automation_j2_b_b2,
            run_project_workflow_automation_k3_b,
            record_worker_structured_report,
            record_project_director_process_fact_decision,
            record_global_final_result_review,
            record_user_result_decision,
            generate_stage_c_acceptance_summary,
            load_project_consultation_proposal_store,
            create_project_consultation_proposal,
            render_project_consultation_proposal_markdown,
            record_project_consultation_proposal_decision,
            load_session_continuation_store,
            confirm_controlled_session_continuation,
            run_controlled_session_continuation_stub,
            inspect_controlled_session_continuation_real_resume_authorization,
            run_controlled_session_continuation_real_resume_phase_a,
            run_controlled_session_continuation_real_resume_phase_b,
            load_blackboard_candidate_store,
            record_blackboard_candidate_decision,
            load_memory_capture_store,
            capture_memory_event,
            load_observation_store,
            create_observation,
            create_memory_candidate_from_observation,
            preview_task_memory_packet,
            load_memory_lint_store,
            load_memory_entity_relation_store,
            load_memory_pattern_store,
            preview_mature_patterns,
            record_mature_pattern_decision,
            preview_memory_entity_relation_candidates,
            record_memory_entity_alias_decision,
            record_memory_entity_merge_decision,
            record_memory_relation_candidate_decision,
            run_memory_lint,
            load_memory_candidate_store,
            create_memory_candidate,
            record_memory_candidate_decision,
            adopt_memory_candidate_to_formal_memory,
            load_formal_memory_store,
            create_formal_memory_record,
            preview_formal_memory_lifecycle_operation,
            record_formal_memory_lifecycle_operation,
            initialize_workflow_state,
            bootstrap_project_workflow,
            create_task_draft,
            render_task_package_preview,
            copy_task_package_preview,
            update_task_package_draft_fields,
            correct_task_package_dispatch_fields,
            generate_task_package_file,
            inspect_task_package_dispatch_readiness,
            inspect_workflow_run_check,
            update_work_item_state,
            bind_workflow_node_codex_session,
            unbind_workflow_node_codex_session,
            prepare_workflow_node_dispatch,
            execute_workflow_node_dispatch,
            execute_experiment_node_dispatch,
            execute_project_workflow_node,
            start_project_workflow_chain,
            stop_project_workflow_chain,
            get_project_workflow_chain_status,
            start_project_director_chain,
            apply_project_director_failed_action,
            auto_advance_authorized_role_loop,
            confirm_and_start_authorized_run,
            confirm_project_director_task_session_bindings,
            preview_pending_proposal_director_plan,
            run_project_consultation,
            global_supervisor_agent::run_global_supervisor_review,
            global_supervisor_agent::run_global_supervisor_boundary_review,
            global_supervisor_review_store::load_global_supervisor_review_store,
            secretary_agent::run_secretary_explain,
            run_history_read_model::list_project_run_history,
            list_project_workflows,
            submit_project_workflow_draft,
            get_project_workflow_nodes,
            read_workflow_node_dispatch_result,
            record_workflow_dispatch_director_review,
            record_workflow_permission_decision,
            prepare_offline_role_dispatch,
            record_offline_role_result_handoff,
            record_offline_director_review,
            run_workflow_machine,
            copy_indexed_path,
            open_indexed_project,
            reveal_indexed_rollout,
            mcp::commands::canvas_load,
            mcp::commands::canvas_save,
            mcp::commands::canvas_start_run,
            mcp::commands::canvas_abort_run,
            mcp::commands::canvas_run_status,
            mcp::commands::canvas_tick_run,
            mcp::commands::save_workflow_template,
            mcp::commands::list_workflow_templates,
            mcp::commands::load_workflow_template,
            mcp::commands::delete_workflow_template,
            store_hygiene::sweep_canvas_run_residue
        ]
    };
}
