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
        )
            -> Result<crate::m5_product_commands::M5FormalStepResponse, String> =
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
        let product_prod = production_prefix(product);
        assert!(
            !product_prod.contains("run_authorized_workcell"),
            "production runtime must not call the test-only workcell helper"
        );
        assert!(!product_prod.contains("fail_cell"));
        assert!(!product_prod.contains("-fail"));
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

    fn production_prefix(src: &str) -> &str {
        src.find("#[cfg(test)]")
            .map(|idx| &src[..idx])
            .unwrap_or(src)
    }

    fn item_has_cfg_test(src: &str, needle: &str) -> bool {
        let start = match src.find(needle) {
            Some(idx) => idx,
            None => return false,
        };
        let prefix = &src[..start];
        prefix
            .rfind("#[")
            .map(|attr| prefix[attr..].starts_with("#[cfg(test)]"))
            .unwrap_or(false)
    }

    #[test]
    fn run_conformance_suite_is_test_only() {
        let runtime = include_str!("m5_agent_runtime.rs");
        assert!(
            item_has_cfg_test(runtime, "pub(crate) fn run_conformance_suite"),
            "run_conformance_suite must stay test-only"
        );
        let production_sources = [
            ("m5_agent_runtime.rs", runtime),
            (
                "m5_controlled_execution.rs",
                include_str!("m5_controlled_execution.rs"),
            ),
            (
                "m5_product_commands.rs",
                include_str!("m5_product_commands.rs"),
            ),
            (
                "m5_isolated_acceptance.rs",
                include_str!("m5_isolated_acceptance.rs"),
            ),
            (
                "m5_orchestration_service.rs",
                include_str!("m5_orchestration_service.rs"),
            ),
            (
                "m5_orchestration_store.rs",
                include_str!("m5_orchestration_store.rs"),
            ),
            (
                "m5_runtime_admission.rs",
                include_str!("m5_runtime_admission.rs"),
            ),
            (
                "m5_side_effect_entry.rs",
                include_str!("m5_side_effect_entry.rs"),
            ),
            (
                "m5_project_supervisor.rs",
                include_str!("m5_project_supervisor.rs"),
            ),
            (
                "m5_project_summary.rs",
                include_str!("m5_project_summary.rs"),
            ),
            ("m5_gateway_traits.rs", include_str!("m5_gateway_traits.rs")),
            (
                "m5_execution_grant.rs",
                include_str!("m5_execution_grant.rs"),
            ),
            ("m5_claim_ledger.rs", include_str!("m5_claim_ledger.rs")),
            (
                "m5_runtime_receipt.rs",
                include_str!("m5_runtime_receipt.rs"),
            ),
            ("m5_dto.rs", include_str!("m5_dto.rs")),
            (
                "m5_runner_entry_registry.rs",
                include_str!("m5_runner_entry_registry.rs"),
            ),
        ];
        for (name, src) in production_sources {
            let production = production_prefix(src);
            assert!(
                !production.contains("run_conformance_suite("),
                "{name} production source must not call run_conformance_suite"
            );
        }
    }

    #[test]
    fn production_cannot_pass_caller_grant_to_runtime_execute() {
        let product = include_str!("m5_product_commands.rs");
        let isolated = include_str!("m5_isolated_acceptance.rs");
        let admission = include_str!("m5_runtime_admission.rs");
        let controlled = include_str!("m5_controlled_execution.rs");
        let runtime = include_str!("m5_agent_runtime.rs");
        let product_prod = production_prefix(product);
        let isolated_prod = production_prefix(isolated);
        let admission_prod = production_prefix(admission);
        let controlled_prod = production_prefix(controlled);
        assert!(!product_prod.contains("run_authorized_workcell"));
        assert!(!product_prod.contains("run_conformance_suite"));
        assert!(
            product_prod.contains("run_admitted_workcell"),
            "formal product runtime must stay on the admitted workcell"
        );
        assert!(
            item_has_cfg_test(isolated, "pub(crate) fn run_authorized_followthrough"),
            "isolated followthrough must stay test-only"
        );
        assert!(!isolated_prod.contains("run_authorized_workcell"));
        assert!(!isolated_prod.contains("run_conformance_suite"));
        assert!(!admission_prod.contains(".execute(workcell"));
        assert!(
            item_has_cfg_test(controlled, "pub(crate) fn run_authorized_workcell"),
            "raw workcell helper must stay test-only"
        );
        assert!(
            controlled_prod.contains("fn persist_and_execute_workcell("),
            "production execute must stay behind persist_and_execute_workcell"
        );
        assert_eq!(
            controlled_prod.matches(".execute(workcell").count(),
            1,
            "production controlled execution must have exactly one adapter execute"
        );
        assert!(
            !controlled_prod.contains("run_conformance_suite"),
            "production controlled execution must not run the raw conformance suite"
        );
        assert!(
            item_has_cfg_test(runtime, "pub(crate) fn run_conformance_suite"),
            "raw runtime suite must stay test-only"
        );
        assert_eq!(
            production_prefix(runtime)
                .matches("run_conformance_suite(")
                .count(),
            0
        );
    }

    #[test]
    fn control_commands_do_not_call_runtime_and_are_registered() {
        let product = include_str!("m5_product_commands.rs");
        let controlled = include_str!("m5_controlled_execution.rs");
        let registry = include_str!("command_registry.rs");
        let load_start = product
            .find("pub(crate) fn load_m5_execution_control_with_state(")
            .expect("load control");
        let apply_start = product
            .find("pub(crate) fn apply_m5_execution_control_with_state(")
            .expect("apply control");
        let load_end = product[load_start + 1..]
            .find("\npub(crate) fn ")
            .map(|idx| load_start + 1 + idx)
            .expect("next after load");
        let apply_end = product[apply_start + 1..]
            .find("\n#[tauri::command]")
            .map(|idx| apply_start + 1 + idx)
            .expect("tauri apply wrapper");
        let load = &product[load_start..load_end];
        let apply = &product[apply_start..apply_end];
        let controlled_load_start = controlled
            .find("pub(crate) fn load_execution_control(")
            .expect("controlled load");
        let controlled_load_end = controlled[controlled_load_start + 1..]
            .find("\nfn ")
            .map(|idx| controlled_load_start + 1 + idx)
            .expect("next after controlled load");
        let controlled_apply_start = controlled
            .find("pub(crate) fn apply_execution_control(")
            .expect("controlled apply");
        let controlled_apply_end = controlled[controlled_apply_start + 1..]
            .find("\npub(crate) fn ")
            .map(|idx| controlled_apply_start + 1 + idx)
            .expect("next after controlled apply");
        let controlled_apply_fault_start = controlled
            .find("pub(crate) fn apply_execution_control_with_fault(")
            .expect("controlled apply fault");
        let controlled_apply_fault_end = controlled[controlled_apply_fault_start + 1..]
            .find("\n#[cfg(test)]")
            .map(|idx| controlled_apply_fault_start + 1 + idx)
            .expect("next after controlled apply fault");
        let controlled_load = &controlled[controlled_load_start..controlled_load_end];
        let controlled_apply = &controlled[controlled_apply_start..controlled_apply_end];
        let controlled_apply_fault =
            &controlled[controlled_apply_fault_start..controlled_apply_fault_end];
        for src in [
            load,
            apply,
            controlled_load,
            controlled_apply,
            controlled_apply_fault,
        ] {
            assert!(!src.contains("run_admitted_workcell"));
            assert!(!src.contains("run_authorized_workcell"));
            assert!(!src.contains("run_m5_authorized_runtime_with_state"));
            assert!(!src.contains(".execute(workcell"));
        }
        assert!(registry.contains("crate::m5_product_commands::load_m5_execution_control"));
        assert!(registry.contains("crate::m5_product_commands::apply_m5_execution_control"));
        let _load: fn(
            &crate::AppState,
            crate::m5_dto::M5ExecutionControlLoadRequest,
        ) -> Result<crate::m5_dto::M5ExecutionControlResponse, String> =
            crate::m5_product_commands::load_m5_execution_control_with_state;
        let _apply: fn(
            &crate::AppState,
            crate::m5_dto::M5ExecutionControlApplyRequest,
        ) -> Result<crate::m5_dto::M5ExecutionControlResponse, String> =
            crate::m5_product_commands::apply_m5_execution_control_with_state;
        assert_eq!(
            ENTRIES
                .iter()
                .filter(
                    |entry| entry.source_symbol == "load_m5_execution_control_with_state"
                        || entry.source_symbol == "apply_m5_execution_control_with_state"
                )
                .count(),
            0,
            "control commands must not become runtime entries"
        );
        assert_eq!(classify("M5-SE-002"), Some(RunnerEntryClass::NewGrant));
        assert_eq!(
            ENTRIES
                .iter()
                .find(|entry| entry.id == "M5-SE-002")
                .map(|entry| entry.source_symbol),
            Some("run_m5_authorized_runtime_with_state")
        );
    }

    #[test]
    fn m5r08_runtime_static_guard_rejects_project_scoped_workcell() {
        let product = include_str!("m5_product_commands.rs");
        let formal_start = product
            .find("pub(crate) fn run_m5_authorized_runtime_with_state(")
            .expect("formal runtime");
        let formal_end = product[formal_start + 1..]
            .find("\npub(crate) fn ")
            .map(|idx| formal_start + 1 + idx)
            .expect("next product fn");
        let formal = &product[formal_start..formal_end];
        assert!(
            !formal.contains("format!(\"wc-{}\", binding.project_id)"),
            "ordinary runtime must not build workcell_id from only project_id"
        );
        assert!(!formal.contains("wc-{}\", binding.project_id"));
        assert!(
            !formal.contains("format!(\"rt-{}\", binding.project_id)"),
            "ordinary runtime session_ref must not collapse to project scope"
        );
        assert!(
            formal.contains("attempt_scoped_workcell_id"),
            "ordinary runtime must use attempt/grant-scoped workcell identity"
        );
        assert!(
            formal.contains("admitted.attempt_id()"),
            "workcell identity must consume the admitted attempt"
        );
        assert!(
            formal.contains("admitted_grant_id") || formal.contains("admitted.grant_id()"),
            "workcell identity must consume the admitted grant"
        );

        let product_prod = production_prefix(product);
        assert!(!product_prod.contains("format!(\"wc-{}\", binding.project_id)"));
        assert!(product_prod.contains("attempt_scoped_workcell_id"));

        let controlled = include_str!("m5_controlled_execution.rs");
        let persist_start = controlled
            .find("fn persist_and_execute_workcell(")
            .expect("persist_and_execute_workcell");
        let persist_end = controlled[persist_start + 1..]
            .find("\nfn ")
            .map(|idx| persist_start + 1 + idx)
            .expect("next after persist");
        let persist = &controlled[persist_start..persist_end];
        assert!(
            persist.contains("attempt_scoped_operation_id"),
            "durable operation_id must inherit attempt/grant scope"
        );
        assert!(
            !persist.contains("format!(\"op-{}\", workcell.workcell_id)"),
            "operation_id must not inherit a caller workcell_id that can collapse to project scope"
        );
        assert!(
            persist.contains("existing_operation_for_attempt_effect")
                || persist.contains("duplicate_effect"),
            "duplicate effect safety must be durable, not adapter-only"
        );

        let runtime = include_str!("m5_agent_runtime.rs");
        let receipt_start = runtime.find("fn build_receipt(").expect("build_receipt");
        let receipt_end = runtime[receipt_start + 1..]
            .find("\nfn ")
            .map(|idx| receipt_start + 1 + idx)
            .expect("next after receipt");
        let receipt = &runtime[receipt_start..receipt_end];
        assert!(
            receipt.contains("attempt_scoped_receipt_id"),
            "runtime receipt_id must inherit attempt/grant scope"
        );
        assert!(
            !receipt.contains("format!(\"rr-{}\", workcell.workcell_id)"),
            "receipt_id must not inherit a caller workcell_id that can collapse to project scope"
        );
        let helpers = production_prefix(runtime);
        assert!(helpers.contains("fn attempt_scoped_workcell_id("));
        assert!(helpers.contains("fn attempt_scoped_operation_id("));
        assert!(helpers.contains("fn attempt_scoped_receipt_id("));
        assert!(
            !helpers.contains("format!(\"wc-{}\", binding.project_id)")
                && !helpers.contains("format!(\"wc-{project_id}\")")
        );
    }
}
