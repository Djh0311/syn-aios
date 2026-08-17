// M5R02: every known runner / side-effect entry is classified.
// Unknown entries are a hard fail.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunnerEntryClass {
    NewGrant,
    GuardedLegacy,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunnerEntry {
    pub id: &'static str,
    pub source_symbol: &'static str,
    pub class: RunnerEntryClass,
}

const ENTRIES: &[RunnerEntry] = &[
    RunnerEntry {
        id: "M5-SE-001",
        source_symbol: "admit_granted_side_effect",
        class: RunnerEntryClass::NewGrant,
    },
    RunnerEntry {
        id: "M5-SE-002",
        source_symbol: "run_m5_authorized_runtime_with_state",
        class: RunnerEntryClass::NewGrant,
    },
    RunnerEntry {
        id: "RUN-001",
        source_symbol: "run_h2_phase_b_with_runner",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-002",
        source_symbol: "run_manual_relay_once_with_process_mode_and_command_profile",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-003",
        source_symbol: "start_agent_conversation_transport",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-004",
        source_symbol: "start_supervisor_conversation_transport",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-005",
        source_symbol: "run_project_workflow_chain_at",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-006",
        source_symbol: "run_workflow_machine_at",
        class: RunnerEntryClass::Blocked,
    },
    RunnerEntry {
        id: "RUN-007",
        source_symbol: "start_project_director_chain",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-008",
        source_symbol: "run_real_execution_product_command_phase_b_at",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-009",
        source_symbol: "run_project_workflow_automation_j2_b_b1_at",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-010",
        source_symbol: "launch_supervisor_pilot",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-011",
        source_symbol: "run_supervisor_resident_with_watchdog_retry",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "RUN-012",
        source_symbol: "dispatch_from_mcp",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-001",
        source_symbol: "run_secretary_explain",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-002",
        source_symbol: "run_global_supervisor_review",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-003",
        source_symbol: "run_global_supervisor_boundary_review",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-004",
        source_symbol: "knowledge_open_relay_start",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-005",
        source_symbol: "reap_registered_orphans",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-006",
        source_symbol: "reap_supervisor_resident_stale_sessions_at",
        class: RunnerEntryClass::GuardedLegacy,
    },
    RunnerEntry {
        id: "BG-007",
        source_symbol: "reap_supervisor_temporary_homes_at",
        class: RunnerEntryClass::GuardedLegacy,
    },
];

const INVENTORY_IDS: &[&str] = &[
    "RUN-001", "RUN-002", "RUN-003", "RUN-004", "RUN-005", "RUN-006", "RUN-007", "RUN-008",
    "RUN-009", "RUN-010", "RUN-011", "RUN-012", "BG-001", "BG-002", "BG-003", "BG-004", "BG-005",
    "BG-006", "BG-007",
];

pub(crate) fn classify(id: &str) -> Option<RunnerEntryClass> {
    ENTRIES.iter().find(|e| e.id == id).map(|e| e.class)
}

pub(crate) fn unknown_inventory_ids() -> Vec<&'static str> {
    INVENTORY_IDS
        .iter()
        .copied()
        .filter(|id| classify(id).is_none())
        .collect()
}

pub(crate) fn new_grant_entries() -> Vec<&'static str> {
    ENTRIES
        .iter()
        .filter(|e| e.class == RunnerEntryClass::NewGrant)
        .map(|e| e.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_has_no_unknown_bypass() {
        assert!(
            unknown_inventory_ids().is_empty(),
            "unregistered runner entries: {:?}",
            unknown_inventory_ids()
        );
    }

    #[test]
    fn new_grant_entry_is_registered() {
        assert_eq!(classify("M5-SE-001"), Some(RunnerEntryClass::NewGrant));
        assert_eq!(classify("M5-SE-002"), Some(RunnerEntryClass::NewGrant));
        assert_eq!(classify("RUN-006"), Some(RunnerEntryClass::Blocked));
        assert!(new_grant_entries().contains(&"M5-SE-001"));
        assert!(new_grant_entries().contains(&"M5-SE-002"));
    }

    #[test]
    fn required_m5_se_002_binds_product_runtime_symbol() {
        let entry = ENTRIES
            .iter()
            .find(|entry| entry.id == "M5-SE-002")
            .expect("missing required M5-SE-002");
        assert_eq!(entry.source_symbol, "run_m5_authorized_runtime_with_state");
        assert_eq!(entry.class, RunnerEntryClass::NewGrant);
        let _bound: fn(
            &crate::AppState,
            crate::m5_product_commands::M5FormalStepRequest,
        ) -> Result<crate::m5_product_commands::M5FormalStepResponse, String> =
            crate::m5_product_commands::run_m5_authorized_runtime_with_state;
        assert_eq!(
            entry.source_symbol,
            stringify!(run_m5_authorized_runtime_with_state)
        );
        let se001 = ENTRIES
            .iter()
            .find(|entry| entry.id == "M5-SE-001")
            .expect("missing required M5-SE-001");
        assert_eq!(se001.source_symbol, "admit_granted_side_effect");
        let _gateway: fn(
            &crate::m5_orchestration_store::M5OrchestrationStore,
            &crate::m5_orchestration_identity::GrantId,
            crate::m5_gateway_traits::GrantUseRequest,
        ) -> Result<
            crate::m5_side_effect_entry::SideEffectAdmission,
            crate::m5_gateway_traits::GatewayError,
        > = crate::m5_side_effect_entry::admit_granted_side_effect;
    }

    #[test]
    fn tauri_runtime_wrapper_is_pure_delegate() {
        let src = include_str!("m5_product_commands.rs");
        let start = src
            .find("pub(crate) fn run_m5_authorized_runtime(")
            .expect("tauri wrapper");
        let next = src[start + 1..]
            .find("\npub(crate) fn ")
            .map(|idx| start + 1 + idx)
            .expect("next product fn");
        let wrapper = &src[start..next];
        assert!(
            wrapper.contains("run_m5_authorized_runtime_with_state(&state, request)"),
            "tauri wrapper must be a pure delegate"
        );
        assert!(!wrapper.contains("admit_current_granted_runtime"));
        assert!(!wrapper.contains("complete_dispatch_readback"));
        assert!(!wrapper.contains("run_admitted_workcell"));
        assert!(!wrapper.contains("run_authorized_workcell"));
    }

    #[test]
    fn product_workcell_has_no_unregistered_bypass() {
        let product = include_str!("m5_product_commands.rs");
        let isolated = include_str!("m5_isolated_acceptance.rs");
        let formal_start = product
            .find("pub(crate) fn run_m5_authorized_runtime_with_state(")
            .expect("formal runtime");
        let formal_end = product[formal_start + 1..]
            .find("\npub(crate) fn ")
            .map(|idx| formal_start + 1 + idx)
            .expect("next product fn");
        let formal = &product[formal_start..formal_end];
        assert_eq!(
            formal.matches("run_admitted_workcell").count(),
            1,
            "formal runtime must have exactly one admitted workcell callsite"
        );
        assert!(
            !product.contains("run_authorized_workcell"),
            "production runtime must not call the test-only workcell helper"
        );
        assert!(!product.contains("fail_cell"));
        assert!(!product.contains("-fail"));
        let follow = isolated
            .find("pub(crate) fn run_authorized_followthrough")
            .expect("followthrough");
        let prefix = &isolated[..follow];
        let attr = prefix.rfind("#[").expect("followthrough attribute");
        assert!(
            prefix[attr..].starts_with("#[cfg(test)]"),
            "isolated followthrough must stay test-only"
        );
        assert_eq!(
            isolated.matches("run_authorized_workcell(").count(),
            1,
            "isolated product surface must not expose extra workcell callers"
        );
        assert!(!isolated.contains("fail_cell"));
    }

    #[test]
    fn every_entry_has_exactly_one_class() {
        let mut seen = std::collections::BTreeSet::new();
        for entry in ENTRIES {
            assert!(seen.insert(entry.id), "duplicate entry {}", entry.id);
        }
    }
}
