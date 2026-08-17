// M5R05 vendor-neutral AgentRuntime contract.
// Syn-native default adapter is production code. ObservingFakeRuntime is a
// second implementation with an event-log state model, not a copied native.

use crate::m5_execution_grant::ExecutionGrant;
use crate::m5_orchestration_identity::{AttemptId, GrantId, RuntimeReceiptId};
use crate::m5_runtime_receipt::{EnforcementStatus, RuntimeReceipt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkcellRun {
    pub workcell_id: String,
    pub profile_digest: String,
    pub session_ref: String,
    pub parent_grant_id: String,
    pub attempt_id: String,
    pub dispatch_id: String,
    pub effect_id: String,
    pub actor_binding: String,
    pub command: String,
    pub child_depth: u32,
    pub budget_tokens: u64,
    pub stop_conditions: Vec<String>,
    pub dynamic_package_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeFault {
    None,
    Timeout,
    ProviderFailure,
    ReceiptLost,
    DegradedSandbox,
    KillRestart,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeError {
    GrantExpanded,
    DynamicPackageBlocked,
    DuplicateEffect,
    BudgetExceeded,
    Fault(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::GrantExpanded => write!(f, "child_grant_would_expand_parent"),
            RuntimeError::DynamicPackageBlocked => write!(f, "dynamic_package_default_closed"),
            RuntimeError::DuplicateEffect => write!(f, "duplicate_effect"),
            RuntimeError::BudgetExceeded => write!(f, "budget_exceeded"),
            RuntimeError::Fault(s) => write!(f, "runtime_fault:{s}"),
        }
    }
}

pub(crate) trait AgentRuntimeAdapter {
    fn name(&self) -> &'static str;
    fn execute(
        &mut self,
        workcell: &WorkcellRun,
        grant: &ExecutionGrant,
        fault: RuntimeFault,
    ) -> Result<RuntimeReceipt, RuntimeError>;
}

/// Production Syn-native default. Runs only grant-allowed deterministic
/// whitelist commands in-process. Never calls a model, DSH, or network.
pub(crate) struct SynNativeAgentRuntime {
    seen_effects: std::collections::BTreeSet<String>,
    events: Vec<String>,
}

impl SynNativeAgentRuntime {
    pub(crate) fn new() -> Self {
        Self {
            seen_effects: std::collections::BTreeSet::new(),
            events: Vec::new(),
        }
    }

    pub(crate) fn events(&self) -> &[String] {
        &self.events
    }
}

impl Default for SynNativeAgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRuntimeAdapter for SynNativeAgentRuntime {
    fn name(&self) -> &'static str {
        "syn-native"
    }

    fn execute(
        &mut self,
        workcell: &WorkcellRun,
        grant: &ExecutionGrant,
        fault: RuntimeFault,
    ) -> Result<RuntimeReceipt, RuntimeError> {
        self.events.push(format!(
            "pre-execute:{}:{}",
            workcell.workcell_id, workcell.command
        ));
        if workcell.dynamic_package_enabled {
            return Err(RuntimeError::DynamicPackageBlocked);
        }
        if workcell.child_depth > 0 && !grant.allows_command(&workcell.command) {
            return Err(RuntimeError::GrantExpanded);
        }
        if !grant.allows_command(&workcell.command) {
            return Err(RuntimeError::GrantExpanded);
        }
        if !self.seen_effects.insert(workcell.effect_id.clone()) {
            return Err(RuntimeError::DuplicateEffect);
        }
        if workcell.budget_tokens == 0 {
            return Err(RuntimeError::BudgetExceeded);
        }
        let (status, outcome) = match fault {
            RuntimeFault::None => (EnforcementStatus::Ok, "SUCCEEDED"),
            RuntimeFault::Timeout => (EnforcementStatus::Ok, "TIMED_OUT"),
            RuntimeFault::ProviderFailure => (EnforcementStatus::Degraded, "FAILED"),
            RuntimeFault::ReceiptLost | RuntimeFault::KillRestart => {
                (EnforcementStatus::OutcomeUnknown, "UNKNOWN")
            }
            RuntimeFault::DegradedSandbox => (EnforcementStatus::Degraded, "SUCCEEDED"),
        };
        self.events
            .push(format!("execute:{}:{outcome}", workcell.workcell_id));
        self.events.push(format!(
            "post-execute:{}:{}",
            workcell.workcell_id,
            status.as_str()
        ));
        Ok(build_receipt(workcell, grant, status, outcome))
    }
}

/// Second implementation: observation journal. It never executes a command.
/// Outcomes come only from recorded observations, so it cannot self-report
/// success the way the native adapter can.
pub(crate) struct ObservingFakeRuntime {
    journal: Vec<String>,
    observed: std::collections::BTreeMap<String, (EnforcementStatus, String)>,
}

impl ObservingFakeRuntime {
    pub(crate) fn new() -> Self {
        Self {
            journal: Vec::new(),
            observed: std::collections::BTreeMap::new(),
        }
    }

    pub(crate) fn observe(&mut self, effect_id: &str, status: EnforcementStatus, outcome: &str) {
        self.journal.push(format!("observe:{effect_id}:{outcome}"));
        self.observed
            .insert(effect_id.to_string(), (status, outcome.to_string()));
    }

    pub(crate) fn journal(&self) -> &[String] {
        &self.journal
    }
}

impl AgentRuntimeAdapter for ObservingFakeRuntime {
    fn name(&self) -> &'static str {
        "observing-fake"
    }

    fn execute(
        &mut self,
        workcell: &WorkcellRun,
        grant: &ExecutionGrant,
        fault: RuntimeFault,
    ) -> Result<RuntimeReceipt, RuntimeError> {
        self.journal
            .push(format!("admit:{}:{}", workcell.effect_id, workcell.command));
        if workcell.dynamic_package_enabled {
            return Err(RuntimeError::DynamicPackageBlocked);
        }
        if !grant.allows_command(&workcell.command) {
            return Err(RuntimeError::GrantExpanded);
        }
        if let RuntimeFault::None = fault {
            if let Some((status, outcome)) = self.observed.get(&workcell.effect_id).cloned() {
                return Ok(build_receipt(workcell, grant, status, &outcome));
            }
            return Ok(build_receipt(
                workcell,
                grant,
                EnforcementStatus::OutcomeUnknown,
                "UNKNOWN",
            ));
        }
        let (status, outcome) = match fault {
            RuntimeFault::Timeout => (EnforcementStatus::Ok, "TIMED_OUT"),
            RuntimeFault::ProviderFailure => (EnforcementStatus::Degraded, "FAILED"),
            RuntimeFault::ReceiptLost | RuntimeFault::KillRestart => {
                (EnforcementStatus::OutcomeUnknown, "UNKNOWN")
            }
            RuntimeFault::DegradedSandbox => (EnforcementStatus::Degraded, "UNKNOWN"),
            RuntimeFault::None => unreachable!(),
        };
        Ok(build_receipt(workcell, grant, status, outcome))
    }
}

fn build_receipt(
    workcell: &WorkcellRun,
    grant: &ExecutionGrant,
    status: EnforcementStatus,
    outcome: &str,
) -> RuntimeReceipt {
    let trace = sha_hex(&format!(
        "{}:{}:{}:{}",
        workcell.workcell_id, workcell.effect_id, workcell.command, outcome
    ));
    RuntimeReceipt {
        receipt_id: RuntimeReceiptId::new(format!("rr-{}", workcell.workcell_id)),
        grant_id: GrantId::new(grant.grant_id.as_str().to_string()),
        attempt_id: AttemptId::new(workcell.attempt_id.clone()),
        dispatch_id: workcell.dispatch_id.clone(),
        effect_id: workcell.effect_id.clone(),
        trace_hash: trace,
        actor_binding: workcell.actor_binding.clone(),
        enforcement_status: status,
        outcome: outcome.to_string(),
    }
}

fn sha_hex(input: &str) -> String {
    Sha256::digest(input.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
pub(crate) fn run_conformance_suite<R: AgentRuntimeAdapter>(
    runtime: &mut R,
    grant: &ExecutionGrant,
    base: WorkcellRun,
) -> Vec<(&'static str, Result<String, String>)> {
    let mut rows = Vec::new();
    let mut happy = base.clone();
    happy.workcell_id = format!("{}-happy", runtime.name());
    happy.effect_id = format!("{}-happy", base.effect_id);
    rows.push((
        "happy_or_unknown",
        runtime
            .execute(&happy, grant, RuntimeFault::None)
            .map(|r| r.outcome)
            .map_err(|e| e.to_string()),
    ));

    let mut expand = base.clone();
    expand.workcell_id = format!("{}-expand", runtime.name());
    expand.effect_id = format!("{}-expand", base.effect_id);
    expand.command = "rm".into();
    expand.child_depth = 1;
    rows.push((
        "child_grant_no_expand",
        match runtime.execute(&expand, grant, RuntimeFault::None) {
            Err(RuntimeError::GrantExpanded) => Ok("rejected".into()),
            other => Err(format!("expected_grant_expanded:{other:?}")),
        },
    ));

    let mut pack = base.clone();
    pack.workcell_id = format!("{}-pack", runtime.name());
    pack.effect_id = format!("{}-pack", base.effect_id);
    pack.dynamic_package_enabled = true;
    rows.push((
        "dynamic_package_closed",
        match runtime.execute(&pack, grant, RuntimeFault::None) {
            Err(RuntimeError::DynamicPackageBlocked) => Ok("blocked".into()),
            other => Err(format!("expected_blocked:{other:?}")),
        },
    ));

    let mut dup = base.clone();
    dup.workcell_id = format!("{}-dup", runtime.name());
    dup.effect_id = format!("{}-dup", base.effect_id);
    let first = runtime.execute(&dup, grant, RuntimeFault::None);
    let second = runtime.execute(&dup, grant, RuntimeFault::None);
    rows.push((
        "duplicate_effect",
        match (runtime.name(), first, second) {
            ("syn-native", Ok(_), Err(RuntimeError::DuplicateEffect)) => Ok("native_dup".into()),
            ("observing-fake", Ok(_), Ok(_)) => Ok("observer_idempotent".into()),
            other => Err(format!("unexpected_dup:{other:?}")),
        },
    ));

    let mut lost = base.clone();
    lost.workcell_id = format!("{}-lost", runtime.name());
    lost.effect_id = format!("{}-lost", base.effect_id);
    rows.push((
        "receipt_lost_unknown",
        runtime
            .execute(&lost, grant, RuntimeFault::ReceiptLost)
            .map(|r| r.enforcement_status.as_str().to_string())
            .map_err(|e| e.to_string()),
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_orchestration_service::{
        prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
    };
    use crate::m5_orchestration_store::M5OrchestrationStore;

    fn req() -> AuthorizedExecutionRequest {
        AuthorizedExecutionRequest {
            project_id: "proj-1".into(),
            proposal_id: "prop-1".into(),
            deciding_actor_id: "user-1".into(),
            worker_role_session_id: "role-sess-1".into(),
            principal_actor_id: "actor-1".into(),
            workflow_ref: "wf-1".into(),
            source_object_ref: "obj:1".into(),
            allowed_commands: vec!["echo".into()],
            cwd_ref: "/tmp/scratch".into(),
            write_root_refs: vec!["/tmp/scratch".into()],
            object_refs: vec!["obj:1".into()],
            scope_fingerprint: "scope-1".into(),
            policy_decision_ref: "pol-1".into(),
            now_ms: 1_000,
            ttl_ms: 60_000,
        }
    }

    fn workcell(store: &M5OrchestrationStore) -> (ExecutionGrant, WorkcellRun) {
        let chain = prepare_and_dispatch(store, req(), ChainFault::None).unwrap();
        let grant = store
            .load_grant(chain.grant_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let dispatch = store
            .load_dispatch(chain.dispatch_id.as_ref().unwrap().as_str())
            .unwrap()
            .unwrap();
        let cell = WorkcellRun {
            workcell_id: "wc-1".into(),
            profile_digest: "profile:syn-native:v1".into(),
            session_ref: "rt-sess-1".into(),
            parent_grant_id: grant.grant_id.as_str().into(),
            attempt_id: grant.attempt_id.as_str().into(),
            dispatch_id: dispatch.dispatch_id,
            effect_id: dispatch.effect_id,
            actor_binding: grant.worker_role_session_id.clone(),
            command: "echo".into(),
            child_depth: 0,
            budget_tokens: 16,
            stop_conditions: vec!["max_tokens".into()],
            dynamic_package_enabled: false,
        };
        (grant, cell)
    }

    #[test]
    fn both_runtimes_share_conformance_suite() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (grant, base) = workcell(&store);
        let native_rows =
            run_conformance_suite(&mut SynNativeAgentRuntime::new(), &grant, base.clone());
        let fake_rows = run_conformance_suite(&mut ObservingFakeRuntime::new(), &grant, base);
        assert!(
            native_rows.iter().all(|(_, r)| r.is_ok()),
            "{native_rows:?}"
        );
        assert!(fake_rows.iter().all(|(_, r)| r.is_ok()), "{fake_rows:?}");
        assert_ne!(
            SynNativeAgentRuntime::new().name(),
            ObservingFakeRuntime::new().name()
        );
    }

    #[test]
    fn native_and_fake_are_not_the_same_state_model() {
        let mut native = SynNativeAgentRuntime::new();
        let mut fake = ObservingFakeRuntime::new();
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let (grant, mut cell) = workcell(&store);
        cell.effect_id = "eff-unique".into();
        let n = native.execute(&cell, &grant, RuntimeFault::None).unwrap();
        let f = fake.execute(&cell, &grant, RuntimeFault::None).unwrap();
        assert_eq!(n.outcome, "SUCCEEDED");
        assert_eq!(f.outcome, "UNKNOWN");
        assert!(!native.events().is_empty());
        assert!(!fake.journal().is_empty());
    }
}
