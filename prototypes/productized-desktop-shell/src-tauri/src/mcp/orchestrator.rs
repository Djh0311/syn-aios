// Editable Canvas v1 — orchestrator.
// Owns the lifecycle of a "run" (a car-shop session driven against one canvas).
// Decides who to spawn next based on state.json (single-line v1):
//   - status=running, busy=None, no outbox      → wake director
//   - status=running, busy=Some, no outbox      → spawn / resume subagent
//   - status=running, busy=Some, outbox=Some    → wake director
//   - status=finished | aborted                 → stop the loop
//
// K2.5 seals real run entry points: product code may keep read/status/storage
// helpers, but MCP canvas must not spawn Codex outside the unified Product
// Command boundary.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::storage::{self, CanvasAuditEvent, CanvasRunState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRunRequest {
    pub canvas_id: String,
    pub goal: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartRunResult {
    pub run_id: String,
    pub state_path: String,
    pub run: CanvasRunState,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunStatus {
    pub run: CanvasRunState,
    pub last_decision: Option<LoopDecision>,
}

#[derive(Debug, Clone, Serialize)]
pub enum LoopDecision {
    SpawnDirector {
        director_node_id: String,
    },
    SpawnSubagent {
        node_id: String,
        task: String,
    },
    SpawnDirectorReview {
        director_node_id: String,
        outbox_node_id: String,
    },
    NoOp {
        reason: String,
    },
}

#[derive(Default)]
pub struct OrchestratorState {
    inner: Arc<Mutex<HashMap<String, RunSlot>>>,
}

#[derive(Default)]
struct RunSlot {
    last_decision: Option<LoopDecision>,
    cancel_requested: bool,
}

impl OrchestratorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(&self, _req: StartRunRequest) -> Result<StartRunResult, String> {
        Err(
            crate::real_execution_command::mcp_canvas_real_execution_blocked_message(
                "mcp_orchestrator_start_run",
            ),
        )
    }

    pub fn abort_run(&self, run_id: &str, reason: String) -> Result<RunStatus, String> {
        let mut state = storage::load_run_state(run_id)?;
        if state.status != "running" {
            return Err(format!("run 当前状态 {} 不允许 abort", state.status));
        }
        let now = storage::iso_now();
        state.status = "aborted".to_string();
        state.abort_reason = Some(reason.clone());
        state.busy_node_id = None;
        state.inbox = None;
        state.updated_at = now.clone();
        storage::save_run_state(&state)?;
        storage::append_audit(
            run_id,
            &CanvasAuditEvent {
                ts: now,
                actor: serde_json::json!({ "kind": "user" }),
                action: "abort".to_string(),
                target_node_id: None,
                payload: Some(serde_json::json!({ "reason": reason })),
            },
        )?;
        let mut runs = self.inner.lock().expect("orchestrator runs lock poisoned");
        if let Some(slot) = runs.get_mut(run_id) {
            slot.cancel_requested = true;
        }
        Ok(RunStatus {
            run: state,
            last_decision: None,
        })
    }

    pub fn get_status(&self, run_id: &str) -> Result<RunStatus, String> {
        let state = storage::load_run_state(run_id)?;
        let runs = self.inner.lock().expect("orchestrator runs lock poisoned");
        let last = runs.get(run_id).and_then(|s| s.last_decision.clone());
        Ok(RunStatus {
            run: state,
            last_decision: last,
        })
    }

    /// Sealed legacy experiment entry; Product Command owns real execution.
    pub fn tick(&self, _run_id: &str) -> Result<LoopDecision, String> {
        Err(
            crate::real_execution_command::mcp_canvas_real_execution_blocked_message(
                "mcp_orchestrator_tick",
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{OrchestratorState, StartRunRequest};

    #[test]
    fn mcp_orchestrator_real_run_entries_are_sealed() {
        let state = OrchestratorState::new();
        let start = state
            .start_run(StartRunRequest {
                canvas_id: "canvas-test".to_string(),
                goal: "should not run".to_string(),
            })
            .expect_err("start_run should be sealed before storage or runner access");
        assert!(start.contains("mcp_canvas_real_execution_blocked:mcp_orchestrator_start_run"));

        let tick = state
            .tick("run-test")
            .expect_err("tick should be sealed before runner access");
        assert!(tick.contains("mcp_canvas_real_execution_blocked:mcp_orchestrator_tick"));
    }
}
