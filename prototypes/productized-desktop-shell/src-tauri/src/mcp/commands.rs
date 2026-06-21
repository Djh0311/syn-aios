// Tauri command surface for the editable canvas v1.
// Frontend calls these through @tauri-apps/api invoke().

use tauri::State;

use super::orchestrator::{
    LoopDecision, OrchestratorState, RunStatus, StartRunRequest, StartRunResult,
};
use super::storage::{self, CanvasDefinition, WorkflowTemplate, WorkflowTemplateSummary};

pub(crate) fn mcp_canvas_real_execution_blocked_message(command_name: &str) -> String {
    crate::real_execution_command::mcp_canvas_real_execution_blocked_message(command_name)
}

#[tauri::command]
pub fn canvas_load(canvas_id: String) -> Result<CanvasDefinition, String> {
    match storage::load_canvas(&canvas_id) {
        Ok(c) => Ok(c),
        Err(_) => {
            // Bootstrap an empty canvas so a fresh install can open the page.
            let now = storage::iso_now();
            let blank = CanvasDefinition {
                schema_version: "canvas-v1".to_string(),
                canvas_id: canvas_id.clone(),
                display_name: format!("画布 {canvas_id}"),
                project_root: None,
                nodes: Vec::new(),
                edges: Vec::new(),
                created_at: now.clone(),
                updated_at: now,
                warnings: Vec::new(),
            };
            storage::save_canvas(&blank)?;
            Ok(blank)
        }
    }
}

#[tauri::command]
pub fn canvas_save(canvas: CanvasDefinition) -> Result<(), String> {
    storage::save_canvas(&canvas)
}

// ---------- workflow templates (plan B) — data store only, no execution ----------

#[tauri::command]
pub fn save_workflow_template(template: WorkflowTemplate) -> Result<(), String> {
    storage::save_workflow_template(&template)
}

#[tauri::command]
pub fn list_workflow_templates() -> Result<Vec<WorkflowTemplateSummary>, String> {
    storage::list_workflow_templates()
}

#[tauri::command]
pub fn load_workflow_template(template_id: String) -> Result<WorkflowTemplate, String> {
    storage::load_workflow_template(&template_id)
}

#[tauri::command]
pub fn delete_workflow_template(template_id: String) -> Result<(), String> {
    storage::delete_workflow_template(&template_id)
}

#[tauri::command]
pub fn canvas_start_run(
    _state: State<'_, OrchestratorState>,
    _request: StartRunRequest,
) -> Result<StartRunResult, String> {
    Err(mcp_canvas_real_execution_blocked_message(
        "canvas_start_run",
    ))
}

#[tauri::command]
pub fn canvas_abort_run(
    state: State<'_, OrchestratorState>,
    run_id: String,
    reason: String,
) -> Result<RunStatus, String> {
    state.abort_run(&run_id, reason)
}

#[tauri::command]
pub fn canvas_run_status(
    state: State<'_, OrchestratorState>,
    run_id: String,
) -> Result<RunStatus, String> {
    state.get_status(&run_id)
}

#[tauri::command]
pub fn canvas_tick_run(
    _state: State<'_, OrchestratorState>,
    _run_id: String,
) -> Result<LoopDecision, String> {
    Err(mcp_canvas_real_execution_blocked_message("canvas_tick_run"))
}

#[cfg(test)]
mod tests {
    use super::mcp_canvas_real_execution_blocked_message;

    #[test]
    fn mcp_canvas_start_and_tick_product_entries_are_blocked() {
        let start = mcp_canvas_real_execution_blocked_message("canvas_start_run");
        assert!(start.contains("mcp_canvas_real_execution_blocked:canvas_start_run"));
        assert!(start.contains("legacy experiment canvas run is sealed"));

        let tick = mcp_canvas_real_execution_blocked_message("canvas_tick_run");
        assert!(tick.contains("mcp_canvas_real_execution_blocked:canvas_tick_run"));
        assert!(tick.contains("unified product command boundary"));
    }
}
