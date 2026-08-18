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

// A·系统状态读模型（首页系统状态区块 + 顶栏健康点·纯只读·零写点）；
// 命令 load_system_status_read_model 进 generate_handler!。同上借道挂载·保持 lib.rs 0-diff。
mod system_status_read_model;

// B·审计账本读模型（审计账本页·主 store + 各 sidecar 审计的只读聚合流·分页+按类过滤）；
// 命令 query_audit_ledger_read_model 进 generate_handler!。
mod audit_ledger_read_model;

// L3 知识库第一片：工作台自管 vault（md 文件即真相·路径锁三例拒绝·AI 写入仅经用户确认闸）。
// 5 命令（list/read/create/write/ai_write）进 generate_handler!。同上借道挂载，保持 lib.rs 0-diff。
mod knowledge_vault;

// L3 Syn 原生知识工作区 N1：固定 vault 的可重建索引与 typed snapshot/read/search 命令。
// 文件真相仍只在 knowledge_vault 的 Markdown/Canvas/附件目录中，不落第二索引文件。
mod knowledge_index;

// L3 N6：host-owned、短期的 knowledge_open dispatch/ack relay；只在内存承载
// 已验证 Markdown 的 intent，不能成为 vault、binding 或 workflow 的第二真相源。
mod knowledge_open_relay;
#[cfg(test)]
mod knowledge_open_relay_tests;

// L3 可选 Obsidian 兼容层：固定官方 App/CLI/URI 桥；不接受前端传来的 binary、vault 或任意子命令。
mod obsidian_integration;

// A·运行错误人话翻译层（C6 观测补强·纯函数·无 tauri command）。供给类判据单一真源在此，
// runner/run_history 委托到它。同 worker_report 借道挂载·保持 lib.rs 0-diff。
mod run_error_translation;

// Station 2 supervisor pilot launcher and sidecar-only read model.
mod supervisor_action_controller;
mod supervisor_action_protocol;
mod supervisor_session_launcher;

macro_rules! workbench_command_handler {
    () => {{
        let workbench_handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
            load_workbench_snapshot,
            enroll_m1_project_identity,
            load_m4c09_acceptance_status,
            load_secretary_conversation,
            send_secretary_message,
            resolve_secretary_source_route,
            query_workbench_page_read_model,
            record_operation_control_decision,
            preview_manual_codex_relay,
            confirm_manual_codex_relay_once,
            run_manual_codex_relay_once,
            run_manual_codex_relay_gui_direct,
            run_manual_codex_relay_gui_direct_new_session,
            stop_manual_codex_relay_attempt,
            poll_manual_codex_relay_attempt,
            start_agent_conversation_transport,
            start_supervisor_conversation_transport,
            poll_conversation_transport_attempt,
            stop_conversation_transport_attempt,
            load_agent_role_session_directory,
            load_agent_role_session_detail,
            load_jiaoban_role_session_directory,
            load_jiaoban_role_session_detail,
            load_secretary_role_session_status,
            load_global_supervisor_role_session_status,
            start_agent_role_session_continuation,
            start_jiaoban_role_session_continuation,
            load_agent_m3c07_acceptance_status,
            operate_agent_m3c07_acceptance,
            load_jiaoban_m3c07_acceptance_status,
            operate_jiaoban_m3c07_acceptance,
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
            supervisor_session_launcher::submit_supervisor_resident_answer,
            supervisor_session_launcher::launch_supervisor_pilot,
            supervisor_session_launcher::load_supervisor_pilot_read_model,
            global_supervisor_agent::run_global_supervisor_review,
            global_supervisor_agent::run_global_supervisor_boundary_review,
            global_supervisor_review_store::load_global_supervisor_review_store,
            secretary_agent::run_secretary_explain,
            secretary_agent::load_secretary_home_context,
            secretary_agent::operate_secretary_coordination,
            secretary_agent::operate_secretary_personal_object,
            load_secretary_legacy_read_compatibility_report,
            load_secretary_daily_report,
            recover_secretary_daily_catch_up,
            run_history_read_model::list_project_run_history,
            system_status_read_model::load_system_status_read_model,
            audit_ledger_read_model::query_audit_ledger_read_model,
            knowledge_vault::knowledge_vault_list_notes,
            knowledge_vault::knowledge_vault_read_note,
            knowledge_vault::knowledge_vault_create_note,
            knowledge_vault::knowledge_vault_write_note,
            knowledge_vault::knowledge_vault_ai_write,
            knowledge_index::knowledge_workspace_snapshot,
            knowledge_index::knowledge_workspace_vault_manifest,
            knowledge_index::knowledge_workspace_search,
            knowledge_index::knowledge_workspace_graph,
            knowledge_index::knowledge_workspace_read_markdown,
            knowledge_open_relay::acknowledge_knowledge_open_relay_intent,
            knowledge_vault::knowledge_workspace_create_directory,
            knowledge_vault::knowledge_workspace_create_markdown,
            knowledge_vault::knowledge_workspace_write_markdown,
            knowledge_vault::knowledge_canvas::knowledge_workspace_read_canvas,
            knowledge_vault::knowledge_canvas::knowledge_workspace_create_canvas,
            knowledge_vault::knowledge_canvas::knowledge_workspace_write_canvas,
            knowledge_vault::knowledge_attachments::knowledge_workspace_import_attachment,
            knowledge_vault::knowledge_attachments::knowledge_workspace_read_attachment,
            knowledge_vault::knowledge_recovery::knowledge_workspace_create_recovery_backup,
            knowledge_vault::knowledge_recovery::knowledge_workspace_list_recovery_backups,
            knowledge_vault::knowledge_recovery::knowledge_workspace_restore_recovery_backup,
            knowledge_vault::knowledge_workspace_move_entry,
            knowledge_vault::knowledge_workspace_rename_entry,
            knowledge_vault::knowledge_workspace_delete_entry,
            obsidian_integration::obsidian_integration_status,
            obsidian_integration::obsidian_integration_open_vault,
            obsidian_integration::obsidian_integration_open_note,
            obsidian_integration::obsidian_integration_open_search,
            obsidian_integration::obsidian_integration_read_note,
            obsidian_integration::obsidian_integration_search_notes,
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
            store_hygiene::sweep_canvas_run_residue,
            crate::m5_product_commands::open_m5_project_supervisor,
            crate::m5_product_commands::submit_m5_project_supervisor_turn,
            crate::m5_product_commands::record_m5_authorization_decision,
            crate::m5_product_commands::run_m5_authorized_runtime,
            crate::m5_product_commands::record_m5_worker_report,
            crate::m5_product_commands::record_m5_independent_review,
            crate::m5_product_commands::record_m5_result_decision,
            crate::m5_product_commands::load_m5_project_summary,
            crate::m5_product_commands::rebuild_m5_project_summary,
            crate::m5_product_commands::open_m5_source_deep_link,
            crate::m5_product_commands::load_m5_isolated_acceptance_status,
            crate::m5_product_commands::write_m5_isolated_ui_receipt,
            crate::m5_ordinary_control_acceptance::load_m5_ordinary_control_acceptance_status,
            crate::m5_ordinary_control_acceptance::seed_m5_ordinary_known_no_effect_terminal,
            crate::m5_ordinary_control_acceptance::write_m5_ordinary_control_backend_receipt,
            crate::m5_ordinary_control_acceptance::write_m5_ordinary_control_dom_receipt,
            crate::m5_product_commands::load_m5_global_advice_fixture,
            crate::m5_product_commands::load_m5_execution_control,
            crate::m5_product_commands::apply_m5_execution_control
        ];
        move |invoke| {
            let command = invoke.message.command().to_owned();
            match crate::m3_acceptance::reject_unapproved_tauri_command(&command)
                .and_then(|_| crate::m4_acceptance::reject_unapproved_tauri_command(&command))
            {
                Ok(()) => workbench_handler(invoke),
                Err(error) => {
                    invoke.resolver.reject(error);
                    true
                }
            }
        }
    }};
}
