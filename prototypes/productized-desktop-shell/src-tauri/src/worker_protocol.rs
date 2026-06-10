use crate::{
    AdapterContractChecklist, AdapterDataLocationDescriptor, AdapterDegradedMode,
    AdapterHealthSummary, AgentAdapterDescriptor, ControlledApiCliSemantics,
    CredentialRequirementDescriptor, DiagnosticEventSchemaDescriptor, DispatchGuardResult,
    DispatchRequest, ExternalCallRiskEnvelope, MultiWorkerDispatchPlan, PermissionEnvelope,
    ProjectCapabilityPolicy, ProviderAvailabilitySummary, ReadbackResult, RunAttention,
    RunPersistenceHandle, RunRelation, RunUnit, RuntimeLogStoreV1, RuntimeSessionAttention,
    SessionContinuationPreview, SessionContinuationStoreV1, SessionOperationDescriptor,
    TaskMemoryPacketRef, WorkThread, WorkerAdapterProtocolDescriptor, WorkerCapabilityDescriptor,
    WorkerHandoff, WorkerLane, WorkerProtocolReadModel, WorkerProtocolSourceRef,
    WorkerReportCandidate,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn derive_worker_protocol_read_model(
    adapters: &[AgentAdapterDescriptor],
    session_operations: &[SessionOperationDescriptor],
    provider_availability: &[ProviderAvailabilitySummary],
    previews: &[SessionContinuationPreview],
    continuation_store: &SessionContinuationStoreV1,
    runtime_attention: &[RuntimeSessionAttention],
    runtime_log_store: &RuntimeLogStoreV1,
    generated_at: &str,
) -> WorkerProtocolReadModel {
    let worker_adapters =
        derive_worker_adapters(adapters, session_operations, provider_availability);
    let work_threads = derive_work_threads(previews, continuation_store, runtime_attention);
    let run_units = derive_run_units(continuation_store, runtime_attention, runtime_log_store);
    let credential_requirements = derive_credential_requirements(provider_availability);
    let external_call_risk_envelopes =
        derive_external_call_risk_envelopes(&worker_adapters, provider_availability);
    let (dispatch_requests, dispatch_guards, permission_envelopes, task_memory_packet_refs) =
        derive_dispatch_protocol(previews);
    let project_capability_policies =
        derive_project_capability_policies(&dispatch_requests, &worker_adapters);
    let run_relations = derive_run_relations(&run_units);
    let worker_lanes = derive_worker_lanes(&work_threads, &run_units, &runtime_attention);
    let multi_worker_dispatch_plans = derive_multi_worker_dispatch_plans(
        &dispatch_requests,
        &run_units,
        &worker_lanes,
        &run_relations,
    );
    let adapter_contract_checklists = derive_adapter_contract_checklists(
        &worker_adapters,
        &credential_requirements,
        &external_call_risk_envelopes,
    );
    let controlled_api_cli_semantics =
        derive_controlled_api_cli_semantics(&worker_adapters, session_operations);
    let diagnostic_event_schemas = derive_diagnostic_event_schemas(&worker_adapters);
    let adapter_health_summaries = derive_adapter_health_summaries(
        &worker_adapters,
        &credential_requirements,
        &external_call_risk_envelopes,
        &run_units,
    );
    let adapter_degraded_modes =
        derive_adapter_degraded_modes(&worker_adapters, &adapter_health_summaries);
    let adapter_data_locations = derive_adapter_data_locations(
        &worker_adapters,
        &work_threads,
        &run_units,
        runtime_log_store,
    );
    let (worker_handoffs, readback_results, worker_report_candidates) =
        derive_handoffs_and_readback(continuation_store, runtime_attention);

    let mut warnings = vec![
        "worker_protocol_read_model_only".to_string(),
        "does_not_execute_worker".to_string(),
        "does_not_read_codex_home".to_string(),
        "codex_local_is_mapping_not_fact_model".to_string(),
        "planned_adapters_not_connected".to_string(),
    ];
    if worker_adapters
        .iter()
        .any(|adapter| adapter.lifecycle_status == "planned")
    {
        warnings.push("planned_worker_adapters_descriptor_only".to_string());
    }
    if dispatch_guards
        .iter()
        .any(|guard| guard.blocks_execution || guard.requires_user_confirmation)
    {
        warnings.push("dispatch_requires_control_core_and_user_confirmation".to_string());
    }
    if external_call_risk_envelopes
        .iter()
        .any(|risk| risk.project_policy_status != "allowed_with_confirmation")
    {
        warnings.push("capability_policy_blocks_or_limits_real_execution".to_string());
    }
    if multi_worker_dispatch_plans
        .iter()
        .any(|plan| plan.verifier_lane_required)
    {
        warnings.push("multi_worker_plan_requires_reviewer_or_verifier_lane".to_string());
    }
    if adapter_contract_checklists
        .iter()
        .any(|checklist| checklist.status != "ready_for_controlled_adapter_contract")
    {
        warnings.push("adapter_contract_checklist_has_blocking_items".to_string());
    }
    if controlled_api_cli_semantics
        .iter()
        .any(|semantics| semantics.universal_api_backdoor_blocked)
    {
        warnings.push("cli_parity_requires_control_core_permission_audit".to_string());
    }

    WorkerProtocolReadModel {
        schema_version: "worker_protocol_read_model.v1".to_string(),
        generated_at: generated_at.to_string(),
        source_policy:
            "Derived from existing workbench read models; no new sidecar, no runner, no .codex read/write."
                .to_string(),
        worker_adapters,
        work_threads,
        run_units,
        credential_requirements,
        external_call_risk_envelopes,
        project_capability_policies,
        run_relations,
        worker_lanes,
        multi_worker_dispatch_plans,
        adapter_contract_checklists,
        controlled_api_cli_semantics,
        diagnostic_event_schemas,
        adapter_health_summaries,
        adapter_degraded_modes,
        adapter_data_locations,
        dispatch_requests,
        dispatch_guards,
        permission_envelopes,
        task_memory_packet_refs,
        worker_handoffs,
        readback_results,
        worker_report_candidates,
        warnings: dedupe(warnings),
    }
}

fn derive_worker_adapters(
    adapters: &[AgentAdapterDescriptor],
    session_operations: &[SessionOperationDescriptor],
    provider_availability: &[ProviderAvailabilitySummary],
) -> Vec<WorkerAdapterProtocolDescriptor> {
    adapters
        .iter()
        .map(|adapter| {
            let provider = provider_availability
                .iter()
                .find(|summary| summary.adapter_id == adapter.adapter_id);
            let operation_warnings = session_operations
                .iter()
                .filter(|operation| operation.adapter_id == adapter.adapter_id)
                .flat_map(|operation| operation.warnings.clone())
                .collect::<Vec<_>>();
            let provider_id = provider
                .map(|summary| summary.provider_id.clone())
                .unwrap_or_else(|| adapter.provider.clone());
            let mut warnings = adapter.warnings.clone();
            warnings.extend(operation_warnings);
            if adapter.adapter_id != "codex-local" {
                warnings.push("adapter_is_planned_descriptor_only".to_string());
            }

            WorkerAdapterProtocolDescriptor {
                worker_adapter_id: format!("worker-adapter:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                worker_kind: neutral_worker_kind(&adapter.agent_type),
                display_name: adapter.display_name.clone(),
                provider_id: provider_id.clone(),
                lifecycle_status: if adapter.status == "planned" {
                    "planned".to_string()
                } else if adapter.status == "available" {
                    "available_with_guard".to_string()
                } else {
                    adapter.status.clone()
                },
                execution_status: adapter.execution_status.clone(),
                credential_status: provider
                    .map(|summary| summary.credential_status.clone())
                    .unwrap_or_else(|| adapter.credential_status.clone()),
                model_status: provider
                    .map(|summary| summary.model_status.clone())
                    .unwrap_or_else(|| adapter.model_access_status.clone()),
                source_policy: if adapter.adapter_id == "codex-local" {
                    "codex-local maps into neutral WorkerAdapter; protocol must remain adapter-neutral."
                        .to_string()
                } else {
                    "planned descriptor only; no provider call, credential check, or runtime connection."
                        .to_string()
                },
                capability_descriptors: adapter
                    .capabilities
                    .iter()
                    .map(|capability| WorkerCapabilityDescriptor {
                        capability_id: format!(
                            "worker-capability:{}:{}",
                            adapter.adapter_id, capability.kind
                        ),
                        capability_kind: capability.kind.clone(),
                        label: capability.label.clone(),
                        status: capability.status.clone(),
                        risk_level: capability_risk_level(&capability.kind).to_string(),
                        execution_boundary: capability.boundary.clone(),
                        provider_id: Some(provider_id.clone()),
                        credential_requirement_id: Some(format!(
                            "credential-requirement:{}",
                            adapter.adapter_id
                        )),
                        risk_envelope_id: Some(format!(
                            "external-call-risk:{}:{}",
                            adapter.adapter_id, capability.kind
                        )),
                        project_policy_status: capability_policy_status(
                            adapter,
                            &capability.kind,
                        ),
                        source_refs: capability
                            .evidence_refs
                            .iter()
                            .map(|ref_id| source_ref("adapter_capability", ref_id, &capability.label))
                            .collect(),
                        warnings: capability.warnings.clone(),
                    })
                    .collect(),
                source_refs: vec![source_ref(
                    "agent_adapter_descriptor",
                    &adapter.adapter_id,
                    &adapter.display_name,
                )],
                warnings: dedupe(warnings),
            }
        })
        .collect()
}

fn derive_work_threads(
    previews: &[SessionContinuationPreview],
    continuation_store: &SessionContinuationStoreV1,
    runtime_attention: &[RuntimeSessionAttention],
) -> Vec<WorkThread> {
    let mut threads: BTreeMap<String, WorkThread> = BTreeMap::new();
    for preview in previews {
        let thread_id = work_thread_id(
            &preview.adapter_id,
            preview.project_id.as_deref(),
            preview.workflow_id.as_deref(),
            preview.node_id.as_deref(),
            preview.target_session_id.as_deref(),
        );
        threads
            .entry(thread_id.clone())
            .or_insert_with(|| WorkThread {
                work_thread_id: thread_id.clone(),
                adapter_id: preview.adapter_id.clone(),
                lifecycle_status: if preview.guard_result.blocks_execution {
                    "blocked_preview".to_string()
                } else if preview.guard_result.requires_user_confirmation {
                    "waiting_permission".to_string()
                } else {
                    "preview_available".to_string()
                },
                project_id: preview.project_id.clone(),
                workflow_id: preview.workflow_id.clone(),
                node_id: preview.node_id.clone(),
                work_item_id: preview.work_item_id.clone(),
                run_persistence_handle: preview.target_session_id.as_ref().map(|session_id| {
                    persistence_handle_from_parts(
                        &preview.adapter_id,
                        Some(session_id),
                        preview.project_id.clone(),
                        preview.workflow_id.clone(),
                        preview.node_id.clone(),
                        preview.work_item_id.clone(),
                        vec![source_ref(
                            "session_continuation_preview",
                            &preview.preview_id,
                            &preview.operation_id,
                        )],
                    )
                }),
                source_refs: vec![source_ref(
                    "session_continuation_preview",
                    &preview.preview_id,
                    &preview.operation_id,
                )],
                warnings: preview.user_visible_warnings.clone(),
            });
    }

    for continuation in &continuation_store.continuations {
        let thread_id = work_thread_id(
            &continuation.adapter_id,
            Some(&continuation.project_id),
            Some(&continuation.workflow_id),
            Some(&continuation.node_id),
            Some(&continuation.session_id),
        );
        threads
            .entry(thread_id.clone())
            .and_modify(|thread| {
                thread.lifecycle_status = continuation.status.clone();
                thread.source_refs.push(source_ref(
                    "controlled_session_continuation",
                    &continuation.continuation_id,
                    &continuation.operation_id,
                ));
                thread.warnings.extend(continuation.warnings.clone());
            })
            .or_insert_with(|| WorkThread {
                work_thread_id: thread_id,
                adapter_id: continuation.adapter_id.clone(),
                lifecycle_status: continuation.status.clone(),
                project_id: Some(continuation.project_id.clone()),
                workflow_id: Some(continuation.workflow_id.clone()),
                node_id: Some(continuation.node_id.clone()),
                work_item_id: continuation.work_item_id.clone(),
                run_persistence_handle: Some(persistence_handle_from_parts(
                    &continuation.adapter_id,
                    Some(&continuation.session_id),
                    Some(continuation.project_id.clone()),
                    Some(continuation.workflow_id.clone()),
                    Some(continuation.node_id.clone()),
                    continuation.work_item_id.clone(),
                    vec![source_ref(
                        "controlled_session_continuation",
                        &continuation.continuation_id,
                        &continuation.operation_id,
                    )],
                )),
                source_refs: vec![source_ref(
                    "controlled_session_continuation",
                    &continuation.continuation_id,
                    &continuation.operation_id,
                )],
                warnings: continuation.warnings.clone(),
            });
    }

    for attention in runtime_attention {
        let thread_id = work_thread_id(
            &attention.adapter_id,
            attention.project_id.as_deref(),
            attention.workflow_id.as_deref(),
            attention.node_id.as_deref(),
            attention.session_id.as_deref(),
        );
        threads.entry(thread_id).and_modify(|thread| {
            if attention.blocks_continuation {
                thread.lifecycle_status = "blocked".to_string();
            } else if attention.requires_user_action {
                thread.lifecycle_status = "needs_user".to_string();
            }
            thread.source_refs.push(source_ref(
                "runtime_session_attention",
                &attention.attention_id,
                &attention.status,
            ));
            thread.warnings.extend(attention.warnings.clone());
        });
    }

    threads
        .into_values()
        .map(|mut thread| {
            thread.source_refs = dedupe_source_refs(thread.source_refs);
            thread.warnings = dedupe(thread.warnings);
            thread
        })
        .collect()
}

fn derive_run_units(
    continuation_store: &SessionContinuationStoreV1,
    runtime_attention: &[RuntimeSessionAttention],
    runtime_log_store: &RuntimeLogStoreV1,
) -> Vec<RunUnit> {
    let continuation_by_id = continuation_store
        .continuations
        .iter()
        .map(|continuation| (continuation.continuation_id.as_str(), continuation))
        .collect::<BTreeMap<_, _>>();
    let mut attention_by_attempt: BTreeMap<String, Vec<RunAttention>> = BTreeMap::new();
    for attention in runtime_attention {
        for source in &attention.source_refs {
            if source.source_kind == "session_continuation_attempt" {
                attention_by_attempt
                    .entry(source.source_id.clone())
                    .or_default()
                    .push(run_attention_from_runtime(attention));
            }
        }
    }
    let runtime_log_warnings = runtime_log_store
        .entries
        .iter()
        .filter(|entry| entry.category == "execution")
        .flat_map(|entry| entry.warnings.clone())
        .collect::<Vec<_>>();

    continuation_store
        .attempts
        .iter()
        .map(|attempt| {
            let continuation = continuation_by_id
                .get(attempt.continuation_id.as_str())
                .copied();
            let adapter_id = continuation
                .map(|item| item.adapter_id.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let mut warnings = attempt.warnings.clone();
            warnings.extend(attempt.readback_summary.warnings.clone());
            warnings.extend(runtime_log_warnings.clone());
            RunUnit {
                run_unit_id: format!("run-unit:{}", attempt.attempt_id),
                adapter_id: adapter_id.clone(),
                work_thread_id: continuation.map(|item| {
                    work_thread_id(
                        &item.adapter_id,
                        Some(&item.project_id),
                        Some(&item.workflow_id),
                        Some(&item.node_id),
                        Some(&item.session_id),
                    )
                }),
                project_id: continuation.map(|item| item.project_id.clone()),
                workflow_id: continuation.map(|item| item.workflow_id.clone()),
                node_id: continuation.map(|item| item.node_id.clone()),
                work_item_id: continuation.and_then(|item| item.work_item_id.clone()),
                lifecycle_status: attempt.status.clone(),
                operation_id: continuation
                    .map(|item| item.operation_id.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                prompt_sent: attempt.prompt_sent,
                real_worker_executed: attempt.real_codex_executed,
                writes_adapter_home: attempt.writes_codex_home,
                writes_project_files: false,
                writes_workbench_state: attempt.writes_workbench_state,
                attention: attention_by_attempt
                    .remove(&attempt.attempt_id)
                    .unwrap_or_default(),
                source_refs: vec![source_ref(
                    "session_continuation_attempt",
                    &attempt.attempt_id,
                    &attempt.status,
                )],
                warnings: dedupe(warnings),
            }
        })
        .collect()
}

fn derive_credential_requirements(
    provider_availability: &[ProviderAvailabilitySummary],
) -> Vec<CredentialRequirementDescriptor> {
    provider_availability
        .iter()
        .map(|provider| {
            let planned_or_external = provider.availability_status == "planned"
                || provider.external_call_status == "external_call_blocked"
                || provider.credential_status == "credential_missing";
            let mut warnings = provider.warnings.clone();
            warnings.push("credential_descriptor_does_not_read_secret".to_string());
            if planned_or_external {
                warnings
                    .push("credential_required_before_real_external_adapter_execution".to_string());
            }
            CredentialRequirementDescriptor {
                requirement_id: format!("credential-requirement:{}", provider.adapter_id),
                adapter_id: provider.adapter_id.clone(),
                provider_id: provider.provider_id.clone(),
                credential_status: provider.credential_status.clone(),
                required_for_real_execution: planned_or_external,
                read_policy: "never_read_secret_material_in_worker_protocol".to_string(),
                verification_status: if planned_or_external {
                    "not_verified".to_string()
                } else {
                    "workbench_does_not_verify_local_cli_credentials".to_string()
                },
                user_action_required: planned_or_external || provider.requires_user_configuration,
                source_refs: vec![source_ref(
                    "provider_availability",
                    &provider.adapter_id,
                    &provider.provider_label,
                )],
                warnings: dedupe(warnings),
            }
        })
        .collect()
}

fn derive_external_call_risk_envelopes(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    provider_availability: &[ProviderAvailabilitySummary],
) -> Vec<ExternalCallRiskEnvelope> {
    let provider_by_adapter = provider_availability
        .iter()
        .map(|provider| (provider.adapter_id.as_str(), provider))
        .collect::<BTreeMap<_, _>>();
    worker_adapters
        .iter()
        .flat_map(|adapter| {
            let provider = provider_by_adapter.get(adapter.adapter_id.as_str()).copied();
            adapter
                .capability_descriptors
                .iter()
                .map(|capability| {
                    let provider_id = provider
                        .map(|summary| summary.provider_id.clone())
                        .unwrap_or_else(|| adapter.provider_id.clone());
                    let credential_risk = provider
                        .map(|summary| credential_risk_for_status(&summary.credential_status))
                        .unwrap_or("unknown")
                        .to_string();
                    let model_risk = provider
                        .map(|summary| model_risk_for_status(&summary.model_status))
                        .unwrap_or("unknown")
                        .to_string();
                    let external_call_status = provider
                        .map(|summary| summary.external_call_status.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let cost_risk = provider
                        .map(|summary| summary.cost_risk_status.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let data_egress_risk =
                        data_egress_risk_for_capability(&capability.capability_kind).to_string();
                    let project_policy_status = capability.project_policy_status.clone();
                    let mut warnings = capability.warnings.clone();
                    warnings.push("external_call_risk_envelope_read_model_only".to_string());
                    if adapter.lifecycle_status == "planned" {
                        warnings.push("planned_adapter_external_call_blocked".to_string());
                    }
                    ExternalCallRiskEnvelope {
                        envelope_id: format!(
                            "external-call-risk:{}:{}",
                            adapter.adapter_id, capability.capability_kind
                        ),
                        adapter_id: adapter.adapter_id.clone(),
                        provider_id,
                        capability_kind: capability.capability_kind.clone(),
                        external_call_status,
                        data_egress_risk,
                        cost_risk,
                        credential_risk,
                        model_risk,
                        project_policy_status,
                        user_visible_summary: format!(
                            "{} / {} requires policy, permission, audit, and runtime log before real execution.",
                            adapter.display_name, capability.label
                        ),
                        source_refs: capability.source_refs.clone(),
                        warnings: dedupe(warnings),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn derive_project_capability_policies(
    dispatch_requests: &[DispatchRequest],
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
) -> Vec<ProjectCapabilityPolicy> {
    let mut scopes = BTreeMap::<(Option<String>, Option<String>), Vec<&DispatchRequest>>::new();
    for request in dispatch_requests {
        scopes
            .entry((request.project_id.clone(), request.workflow_id.clone()))
            .or_default()
            .push(request);
    }
    if scopes.is_empty() {
        scopes.insert((None, None), Vec::new());
    }

    scopes
        .into_iter()
        .map(|((project_id, workflow_id), requests)| {
            let mut allowed = BTreeSet::new();
            let mut blocked = BTreeSet::new();
            let mut requires_confirmation = BTreeSet::new();
            for adapter in worker_adapters {
                for capability in &adapter.capability_descriptors {
                    match capability.project_policy_status.as_str() {
                        "allowed_with_confirmation" => {
                            allowed.insert(capability.capability_kind.clone());
                            requires_confirmation.insert(capability.capability_kind.clone());
                        }
                        _ => {
                            blocked.insert(capability.capability_kind.clone());
                        }
                    }
                }
            }
            for request in &requests {
                requires_confirmation.insert(format!("operation:{}", request.operation_id));
            }
            ProjectCapabilityPolicy {
                policy_id: format!(
                    "project-capability-policy:{}:{}",
                    project_id.as_deref().unwrap_or("global"),
                    workflow_id.as_deref().unwrap_or("all")
                ),
                project_id: project_id.clone(),
                workflow_id: workflow_id.clone(),
                policy_status: if requests.is_empty() {
                    "no_project_dispatch_scope".to_string()
                } else {
                    "derived_requires_control_core".to_string()
                },
                allowed_capability_kinds: allowed.into_iter().collect(),
                blocked_capability_kinds: blocked.into_iter().collect(),
                requires_user_confirmation: requires_confirmation.into_iter().collect(),
                source_refs: requests
                    .iter()
                    .map(|request| {
                        source_ref(
                            "dispatch_request",
                            &request.dispatch_request_id,
                            &request.operation_id,
                        )
                    })
                    .collect(),
                warnings: vec![
                    "project_capability_policy_read_model_only".to_string(),
                    "technical_availability_is_not_project_authorization".to_string(),
                ],
            }
        })
        .collect()
}

fn derive_run_relations(run_units: &[RunUnit]) -> Vec<RunRelation> {
    let mut grouped = BTreeMap::<(Option<String>, Option<String>), Vec<&RunUnit>>::new();
    for run in run_units {
        grouped
            .entry((run.project_id.clone(), run.workflow_id.clone()))
            .or_default()
            .push(run);
    }

    let mut relations = Vec::new();
    for ((project_id, workflow_id), mut runs) in grouped {
        runs.sort_by(|a, b| a.run_unit_id.cmp(&b.run_unit_id));
        for (index, run) in runs.iter().enumerate() {
            let parent = if index == 0 {
                None
            } else {
                Some(runs[index - 1].run_unit_id.clone())
            };
            relations.push(RunRelation {
                relation_id: format!("run-relation:{}:{}", index, run.run_unit_id),
                relation_kind: if parent.is_some() {
                    "sibling_sequence".to_string()
                } else {
                    "root".to_string()
                },
                parent_run_unit_id: parent,
                child_run_unit_id: run.run_unit_id.clone(),
                project_id: project_id.clone(),
                workflow_id: workflow_id.clone(),
                source_refs: run.source_refs.clone(),
                warnings: vec![
                    "run_relation_derived_from_workbench_state_not_adapter_ui".to_string()
                ],
            });
        }
    }
    relations
}

fn derive_worker_lanes(
    work_threads: &[WorkThread],
    run_units: &[RunUnit],
    runtime_attention: &[RuntimeSessionAttention],
) -> Vec<WorkerLane> {
    let mut scopes = BTreeMap::<(Option<String>, Option<String>, String), WorkerLane>::new();
    for thread in work_threads {
        let kind = lane_kind_for_thread(thread);
        let key = (
            thread.project_id.clone(),
            thread.workflow_id.clone(),
            kind.clone(),
        );
        scopes
            .entry(key)
            .and_modify(|lane| {
                lane.work_thread_ids.push(thread.work_thread_id.clone());
                lane.source_refs.extend(thread.source_refs.clone());
                lane.warnings.extend(thread.warnings.clone());
            })
            .or_insert_with(|| WorkerLane {
                lane_id: format!(
                    "worker-lane:{}:{}:{}",
                    thread.project_id.as_deref().unwrap_or("global"),
                    thread.workflow_id.as_deref().unwrap_or("all"),
                    kind
                ),
                lane_kind: kind,
                project_id: thread.project_id.clone(),
                workflow_id: thread.workflow_id.clone(),
                run_unit_ids: Vec::new(),
                work_thread_ids: vec![thread.work_thread_id.clone()],
                status: thread.lifecycle_status.clone(),
                reviewer_required: false,
                source_refs: thread.source_refs.clone(),
                warnings: thread.warnings.clone(),
            });
    }
    for run in run_units {
        let kind = lane_kind_for_run(run);
        let key = (
            run.project_id.clone(),
            run.workflow_id.clone(),
            kind.clone(),
        );
        scopes
            .entry(key)
            .and_modify(|lane| {
                lane.run_unit_ids.push(run.run_unit_id.clone());
                lane.source_refs.extend(run.source_refs.clone());
                lane.warnings.extend(run.warnings.clone());
                if run.lifecycle_status.contains("failed")
                    || run.lifecycle_status.contains("timed_out")
                {
                    lane.status = "needs_recovery".to_string();
                    lane.reviewer_required = true;
                }
            })
            .or_insert_with(|| WorkerLane {
                lane_id: format!(
                    "worker-lane:{}:{}:{}",
                    run.project_id.as_deref().unwrap_or("global"),
                    run.workflow_id.as_deref().unwrap_or("all"),
                    kind
                ),
                lane_kind: kind,
                project_id: run.project_id.clone(),
                workflow_id: run.workflow_id.clone(),
                run_unit_ids: vec![run.run_unit_id.clone()],
                work_thread_ids: Vec::new(),
                status: if run.lifecycle_status.contains("failed")
                    || run.lifecycle_status.contains("timed_out")
                {
                    "needs_recovery".to_string()
                } else {
                    run.lifecycle_status.clone()
                },
                reviewer_required: run.lifecycle_status.contains("failed")
                    || run.lifecycle_status.contains("timed_out"),
                source_refs: run.source_refs.clone(),
                warnings: run.warnings.clone(),
            });
    }
    for attention in runtime_attention {
        if attention.requires_user_action || attention.blocks_continuation {
            let key = (
                attention.project_id.clone(),
                attention.workflow_id.clone(),
                "reviewer".to_string(),
            );
            scopes
                .entry(key)
                .and_modify(|lane| {
                    lane.status = "needs_review".to_string();
                    lane.reviewer_required = true;
                    lane.source_refs.push(source_ref(
                        "runtime_session_attention",
                        &attention.attention_id,
                        &attention.status,
                    ));
                    lane.warnings.extend(attention.warnings.clone());
                })
                .or_insert_with(|| WorkerLane {
                    lane_id: format!(
                        "worker-lane:{}:{}:reviewer",
                        attention.project_id.as_deref().unwrap_or("global"),
                        attention.workflow_id.as_deref().unwrap_or("all")
                    ),
                    lane_kind: "reviewer".to_string(),
                    project_id: attention.project_id.clone(),
                    workflow_id: attention.workflow_id.clone(),
                    run_unit_ids: Vec::new(),
                    work_thread_ids: Vec::new(),
                    status: "needs_review".to_string(),
                    reviewer_required: true,
                    source_refs: vec![source_ref(
                        "runtime_session_attention",
                        &attention.attention_id,
                        &attention.status,
                    )],
                    warnings: attention.warnings.clone(),
                });
        }
    }

    scopes
        .into_values()
        .map(|mut lane| {
            lane.run_unit_ids = dedupe(lane.run_unit_ids);
            lane.work_thread_ids = dedupe(lane.work_thread_ids);
            lane.source_refs = dedupe_source_refs(lane.source_refs);
            lane.warnings = dedupe(lane.warnings);
            lane
        })
        .collect()
}

fn derive_multi_worker_dispatch_plans(
    dispatch_requests: &[DispatchRequest],
    run_units: &[RunUnit],
    worker_lanes: &[WorkerLane],
    run_relations: &[RunRelation],
) -> Vec<MultiWorkerDispatchPlan> {
    let mut scopes = BTreeSet::<(Option<String>, Option<String>)>::new();
    for request in dispatch_requests {
        scopes.insert((request.project_id.clone(), request.workflow_id.clone()));
    }
    for run in run_units {
        scopes.insert((run.project_id.clone(), run.workflow_id.clone()));
    }
    for lane in worker_lanes {
        scopes.insert((lane.project_id.clone(), lane.workflow_id.clone()));
    }
    if scopes.is_empty() {
        scopes.insert((None, None));
    }

    scopes
        .into_iter()
        .map(|(project_id, workflow_id)| {
            let request_ids = dispatch_requests
                .iter()
                .filter(|request| {
                    request.project_id == project_id && request.workflow_id == workflow_id
                })
                .map(|request| request.dispatch_request_id.clone())
                .collect::<Vec<_>>();
            let run_ids = run_units
                .iter()
                .filter(|run| run.project_id == project_id && run.workflow_id == workflow_id)
                .map(|run| run.run_unit_id.clone())
                .collect::<Vec<_>>();
            let lanes = worker_lanes
                .iter()
                .filter(|lane| lane.project_id == project_id && lane.workflow_id == workflow_id)
                .collect::<Vec<_>>();
            let relation_ids = run_relations
                .iter()
                .filter(|relation| {
                    relation.project_id == project_id && relation.workflow_id == workflow_id
                })
                .map(|relation| relation.relation_id.clone())
                .collect::<Vec<_>>();
            let verifier_required = lanes
                .iter()
                .any(|lane| lane.lane_kind == "reviewer" || lane.reviewer_required);
            let recovery_available = lanes.iter().any(|lane| lane.lane_kind == "recovery");
            MultiWorkerDispatchPlan {
                plan_id: format!(
                    "multi-worker-plan:{}:{}",
                    project_id.as_deref().unwrap_or("global"),
                    workflow_id.as_deref().unwrap_or("all")
                ),
                project_id: project_id.clone(),
                workflow_id: workflow_id.clone(),
                status: if request_ids.is_empty() && run_ids.is_empty() {
                    "no_dispatch_scope".to_string()
                } else if verifier_required {
                    "needs_review_or_verifier_lane".to_string()
                } else {
                    "derived_ready_for_guarded_dispatch".to_string()
                },
                dispatch_request_ids: request_ids,
                run_unit_ids: run_ids,
                lane_ids: lanes.iter().map(|lane| lane.lane_id.clone()).collect(),
                relation_ids,
                verifier_lane_required: verifier_required,
                recovery_lane_available: recovery_available,
                source_policy:
                    "Multi-worker plan is derived from project workflow and control-core state; adapters cannot autonomously spawn, kill, archive, or approve workers."
                        .to_string(),
                source_refs: lanes
                    .iter()
                    .flat_map(|lane| lane.source_refs.clone())
                    .collect::<Vec<_>>(),
                warnings: vec![
                    "multi_worker_dispatch_plan_read_model_only".to_string(),
                    "agent_autonomous_spawn_kill_archive_approve_blocked".to_string(),
                    "verifier_result_cannot_become_formal_fact_without_review".to_string(),
                ],
            }
        })
        .collect()
}

fn derive_adapter_contract_checklists(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    credential_requirements: &[CredentialRequirementDescriptor],
    risk_envelopes: &[ExternalCallRiskEnvelope],
) -> Vec<AdapterContractChecklist> {
    let credential_by_adapter = credential_requirements
        .iter()
        .map(|item| (item.adapter_id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let risks_by_adapter = risk_envelopes.iter().fold(
        BTreeMap::<&str, Vec<&ExternalCallRiskEnvelope>>::new(),
        |mut map, risk| {
            map.entry(risk.adapter_id.as_str()).or_default().push(risk);
            map
        },
    );

    worker_adapters
        .iter()
        .map(|adapter| {
            let credential = credential_by_adapter
                .get(adapter.adapter_id.as_str())
                .copied();
            let risks = risks_by_adapter
                .get(adapter.adapter_id.as_str())
                .cloned()
                .unwrap_or_default();
            let protocol_surface_ready = !adapter.capability_descriptors.is_empty();
            let credential_boundary_defined = credential
                .map(|item| !item.read_policy.trim().is_empty())
                .unwrap_or(false);
            let model_boundary_defined = !adapter.model_status.trim().is_empty()
                && adapter.model_status != "model_unverified"
                && adapter.model_status != "not_verified";
            let data_location_defined =
                adapter.adapter_id == "codex-local" || adapter.lifecycle_status != "planned";
            let mut missing_items = Vec::new();
            if !protocol_surface_ready {
                missing_items.push("capability_descriptor_missing".to_string());
            }
            if !credential_boundary_defined {
                missing_items.push("credential_boundary_missing".to_string());
            }
            if !model_boundary_defined {
                missing_items.push("model_boundary_or_verification_missing".to_string());
            }
            if !data_location_defined {
                missing_items.push("data_location_reserved_not_connected".to_string());
            }
            if adapter.lifecycle_status == "planned"
                || adapter.execution_status == "not_implemented"
            {
                missing_items.push("runtime_connection_not_implemented".to_string());
            }
            if risks
                .iter()
                .any(|risk| risk.project_policy_status != "allowed_with_confirmation")
            {
                missing_items.push("project_policy_blocks_real_execution".to_string());
            }
            let mut warnings = adapter.warnings.clone();
            warnings.push("adapter_contract_checklist_read_model_only".to_string());
            if !missing_items.is_empty() {
                warnings.push("adapter_contract_not_ready_for_real_execution".to_string());
            }
            AdapterContractChecklist {
                checklist_id: format!("adapter-contract-checklist:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                status: if missing_items.is_empty() {
                    "ready_for_controlled_adapter_contract".to_string()
                } else {
                    "blocked_or_reserved_contract".to_string()
                },
                protocol_surface_ready,
                control_core_required: true,
                permission_required: true,
                audit_required: true,
                runtime_log_required: true,
                credential_boundary_defined,
                model_boundary_defined,
                data_location_defined,
                missing_items: dedupe(missing_items),
                source_refs: adapter.source_refs.clone(),
                warnings: dedupe(warnings),
            }
        })
        .collect()
}

fn derive_controlled_api_cli_semantics(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    session_operations: &[SessionOperationDescriptor],
) -> Vec<ControlledApiCliSemantics> {
    worker_adapters
        .iter()
        .map(|adapter| {
            let operations = session_operations
                .iter()
                .filter(|operation| operation.adapter_id == adapter.adapter_id)
                .map(|operation| operation.operation_id.clone())
                .collect::<Vec<_>>();
            let planned = adapter.lifecycle_status == "planned"
                || adapter.execution_status == "not_implemented";
            ControlledApiCliSemantics {
                semantics_id: format!("controlled-api-cli-semantics:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                cli_surface: if adapter.adapter_id == "codex-local" {
                    "codex CLI command preview only unless explicit execution-point authorization"
                        .to_string()
                } else {
                    "planned CLI descriptor only; no command invocation".to_string()
                },
                api_surface:
                    "Workbench controlled API must call control_core, permission envelope, runtime log, and audit before any real execution."
                        .to_string(),
                parity_status: if planned {
                    "reserved_no_runtime_parity".to_string()
                } else {
                    "contract_parity_requires_guard".to_string()
                },
                control_core_path: "required_before_runner".to_string(),
                permission_path: "explicit_user_confirmation_required_for_real_execution"
                    .to_string(),
                audit_path: "runtime_log_and_audit_refs_required".to_string(),
                universal_api_backdoor_blocked: true,
                supported_operation_ids: dedupe(operations),
                source_refs: adapter.source_refs.clone(),
                warnings: dedupe(vec![
                    "cli_parity_read_model_only".to_string(),
                    "no_universal_app_api_backdoor".to_string(),
                    "control_core_permission_audit_required".to_string(),
                ]),
            }
        })
        .collect()
}

fn derive_diagnostic_event_schemas(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
) -> Vec<DiagnosticEventSchemaDescriptor> {
    worker_adapters
        .iter()
        .map(|adapter| DiagnosticEventSchemaDescriptor {
            schema_id: format!("diagnostic-event-schema:{}", adapter.adapter_id),
            adapter_id: adapter.adapter_id.clone(),
            event_kinds: vec![
                "adapter_health".to_string(),
                "dispatch_guard".to_string(),
                "permission_decision".to_string(),
                "runner_attempt".to_string(),
                "readback_boundary".to_string(),
                "degraded_mode".to_string(),
            ],
            severity_levels: vec![
                "info".to_string(),
                "warning".to_string(),
                "degraded".to_string(),
                "blocking".to_string(),
            ],
            required_fields: vec![
                "event_id".to_string(),
                "adapter_id".to_string(),
                "event_kind".to_string(),
                "severity".to_string(),
                "redacted_summary".to_string(),
                "source_refs".to_string(),
                "audit_refs".to_string(),
                "created_at".to_string(),
            ],
            redaction_policy: "no_secret_no_raw_transcript_no_provider_payload".to_string(),
            export_policy: "diagnostic_bundle_requires_separate_authorized_task".to_string(),
            source_refs: adapter.source_refs.clone(),
            warnings: vec!["diagnostic_event_schema_reserved_read_model_only".to_string()],
        })
        .collect()
}

fn derive_adapter_health_summaries(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    credential_requirements: &[CredentialRequirementDescriptor],
    risk_envelopes: &[ExternalCallRiskEnvelope],
    run_units: &[RunUnit],
) -> Vec<AdapterHealthSummary> {
    worker_adapters
        .iter()
        .map(|adapter| {
            let credential = credential_requirements
                .iter()
                .find(|item| item.adapter_id == adapter.adapter_id);
            let adapter_risks = risk_envelopes
                .iter()
                .filter(|risk| risk.adapter_id == adapter.adapter_id)
                .collect::<Vec<_>>();
            let adapter_runs = run_units
                .iter()
                .filter(|run| run.adapter_id == adapter.adapter_id)
                .collect::<Vec<_>>();
            let failed_run = adapter_runs.iter().any(|run| {
                run.lifecycle_status.contains("failed")
                    || run.lifecycle_status.contains("timed_out")
            });
            let blocked_by_policy = adapter_risks
                .iter()
                .any(|risk| risk.project_policy_status != "allowed_with_confirmation");
            let planned = adapter.lifecycle_status == "planned"
                || adapter.execution_status == "not_implemented";
            let missing_credential = credential
                .map(|item| item.credential_status == "credential_missing")
                .unwrap_or(false);
            let model_unverified = adapter.model_status == "model_unverified"
                || adapter.model_status == "not_verified";
            let (status, severity, reason) = if planned {
                (
                    "planned_unavailable".to_string(),
                    "warning".to_string(),
                    Some("adapter_runtime_not_implemented".to_string()),
                )
            } else if missing_credential || model_unverified || blocked_by_policy {
                (
                    "degraded_blocked_for_real_execution".to_string(),
                    "degraded".to_string(),
                    Some("credential_model_or_policy_blocks_real_execution".to_string()),
                )
            } else if failed_run {
                (
                    "degraded_runtime_attention".to_string(),
                    "degraded".to_string(),
                    Some("runtime_attempt_requires_review".to_string()),
                )
            } else {
                ("available_with_guard".to_string(), "info".to_string(), None)
            };
            AdapterHealthSummary {
                health_id: format!("adapter-health:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                status,
                severity,
                credential_status: credential
                    .map(|item| item.credential_status.clone())
                    .unwrap_or_else(|| adapter.credential_status.clone()),
                model_status: adapter.model_status.clone(),
                runtime_status: if adapter_runs.is_empty() {
                    "no_runtime_attempt".to_string()
                } else if failed_run {
                    "has_failed_or_timed_out_attempt".to_string()
                } else {
                    "has_recorded_attempt".to_string()
                },
                degraded_reason: reason,
                source_refs: adapter.source_refs.clone(),
                warnings: dedupe(vec![
                    "adapter_health_read_model_only".to_string(),
                    "does_not_probe_provider_or_credentials".to_string(),
                ]),
            }
        })
        .collect()
}

fn derive_adapter_degraded_modes(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    health_summaries: &[AdapterHealthSummary],
) -> Vec<AdapterDegradedMode> {
    worker_adapters
        .iter()
        .map(|adapter| {
            let health = health_summaries
                .iter()
                .find(|summary| summary.adapter_id == adapter.adapter_id);
            let blocks_real_execution = health
                .map(|summary| summary.status != "available_with_guard")
                .unwrap_or(true);
            AdapterDegradedMode {
                degraded_mode_id: format!("adapter-degraded-mode:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                mode: if blocks_real_execution {
                    "readonly_or_blocked".to_string()
                } else {
                    "guarded_execution_possible_only_after_permission".to_string()
                },
                blocks_real_execution,
                user_visible_summary: if blocks_real_execution {
                    format!(
                        "{} can stay visible as read-model material, but real execution is blocked or reserved.",
                        adapter.display_name
                    )
                } else {
                    format!(
                        "{} remains guarded; real execution still requires explicit permission.",
                        adapter.display_name
                    )
                },
                allowed_surfaces: vec![
                    "read_model".to_string(),
                    "diagnostic_summary".to_string(),
                    "permission_preview".to_string(),
                ],
                blocked_surfaces: if blocks_real_execution {
                    vec![
                        "direct_runner".to_string(),
                        "universal_api_backdoor".to_string(),
                        "provider_probe".to_string(),
                    ]
                } else {
                    vec!["universal_api_backdoor".to_string(), "secret_read".to_string()]
                },
                recovery_requirement: if blocks_real_execution {
                    "separate_adapter_task_with_credentials_model_runtime_and_policy_review"
                        .to_string()
                } else {
                    "execution_point_user_authorization_required".to_string()
                },
                source_refs: adapter.source_refs.clone(),
                warnings: vec!["adapter_degraded_mode_read_model_only".to_string()],
            }
        })
        .collect()
}

fn derive_adapter_data_locations(
    worker_adapters: &[WorkerAdapterProtocolDescriptor],
    work_threads: &[WorkThread],
    run_units: &[RunUnit],
    runtime_log_store: &RuntimeLogStoreV1,
) -> Vec<AdapterDataLocationDescriptor> {
    worker_adapters
        .iter()
        .map(|adapter| {
            let has_thread_handle = work_threads.iter().any(|thread| {
                thread.adapter_id == adapter.adapter_id && thread.run_persistence_handle.is_some()
            });
            let has_run = run_units
                .iter()
                .any(|run| run.adapter_id == adapter.adapter_id);
            let mut store_refs = vec!["workbench_snapshot.worker_protocol".to_string()];
            if has_run {
                store_refs.push("session-continuations.v1.json".to_string());
            }
            if !runtime_log_store.entries.is_empty() {
                store_refs.push("runtime-log.v1.json".to_string());
            }
            AdapterDataLocationDescriptor {
                data_location_id: format!("adapter-data-location:{}", adapter.adapter_id),
                adapter_id: adapter.adapter_id.clone(),
                persistence_kind: if has_thread_handle {
                    "native_thread_reference_metadata".to_string()
                } else if has_run {
                    "workbench_continuation_attempt_metadata".to_string()
                } else {
                    "descriptor_only".to_string()
                },
                workbench_store_refs: dedupe(store_refs),
                adapter_home_policy: if adapter.adapter_id == "codex-local" {
                    "no_codex_home_read_write_without_execution_point_authorization".to_string()
                } else {
                    "no_adapter_home_known_or_accessed".to_string()
                },
                project_write_policy:
                    "project_file_write_requires_permission_envelope_and_allowed_write_roots"
                        .to_string(),
                transcript_policy: "metadata_only_by_default_no_full_transcript".to_string(),
                secret_policy: "never_read_auth_token_env_keychain_oauth_provider_credentials"
                    .to_string(),
                source_refs: adapter.source_refs.clone(),
                warnings: vec!["adapter_data_location_descriptor_read_model_only".to_string()],
            }
        })
        .collect()
}

fn derive_dispatch_protocol(
    previews: &[SessionContinuationPreview],
) -> (
    Vec<DispatchRequest>,
    Vec<DispatchGuardResult>,
    Vec<PermissionEnvelope>,
    Vec<TaskMemoryPacketRef>,
) {
    let mut requests = Vec::new();
    let mut guards = Vec::new();
    let mut envelopes = Vec::new();
    let mut memory_refs = Vec::new();

    for preview in previews {
        let request_id = format!("dispatch-request:{}", preview.preview_id);
        let source_refs = vec![source_ref(
            "session_continuation_preview",
            &preview.preview_id,
            &preview.operation_id,
        )];
        requests.push(DispatchRequest {
            dispatch_request_id: request_id.clone(),
            adapter_id: preview.adapter_id.clone(),
            operation_id: preview.operation_id.clone(),
            project_id: preview.project_id.clone(),
            workflow_id: preview.workflow_id.clone(),
            node_id: preview.node_id.clone(),
            work_item_id: preview.work_item_id.clone(),
            target_session_id: preview.target_session_id.clone(),
            requested_by: preview.request.requested_by.clone(),
            prompt_source_kind: preview.prompt_source_kind.clone(),
            prompt_summary: preview.prompt_summary.clone(),
            source_refs: source_refs.clone(),
            warnings: preview.user_visible_warnings.clone(),
        });
        guards.push(DispatchGuardResult {
            dispatch_request_id: request_id.clone(),
            status: preview.guard_result.status.clone(),
            severity: if preview.guard_result.blocks_execution {
                "blocking".to_string()
            } else if preview.guard_result.requires_user_confirmation {
                "needs_user".to_string()
            } else {
                "info".to_string()
            },
            blocks_execution: preview.guard_result.blocks_execution,
            requires_user_confirmation: preview.guard_result.requires_user_confirmation,
            reasons: preview.guard_result.reasons.clone(),
            required_fixes: preview.guard_result.required_fixes.clone(),
            warnings: preview.guard_result.warnings.clone(),
        });
        envelopes.push(PermissionEnvelope {
            envelope_id: format!("permission-envelope:{}", preview.preview_id),
            adapter_id: preview.adapter_id.clone(),
            operation_id: preview.operation_id.clone(),
            status: preview.guard_result.status.clone(),
            explicit_approval_required: preview.guard_result.requires_user_confirmation,
            approved_for_real_execution: false,
            cwd: preview.target_cwd.clone(),
            allowed_write_roots: preview.allowed_write_roots_summary.clone(),
            denied_paths: vec![
                "/Users/yoyi/.codex unless explicitly authorized".to_string(),
                "auth/token/secret/.env/keychain/OAuth/provider credential".to_string(),
                "full transcript/rollout by default".to_string(),
            ],
            prompt_summary: preview.prompt_summary.clone(),
            risk_summary: "Neutral permission envelope; preview does not execute worker."
                .to_string(),
            source_refs: source_refs.clone(),
            warnings: preview.audit_impact.warnings.clone(),
        });
        memory_refs.push(TaskMemoryPacketRef {
            ref_id: format!("task-memory-packet-ref:{}", preview.preview_id),
            snapshot_id: None,
            fingerprint: None,
            included_count: 0,
            excluded_count: 0,
            review_material_count: 0,
            stale: false,
            source_refs,
            warnings: vec!["task_memory_packet_ref_not_attached_to_preview".to_string()],
        });
    }

    (requests, guards, envelopes, memory_refs)
}

fn derive_handoffs_and_readback(
    continuation_store: &SessionContinuationStoreV1,
    runtime_attention: &[RuntimeSessionAttention],
) -> (
    Vec<WorkerHandoff>,
    Vec<ReadbackResult>,
    Vec<WorkerReportCandidate>,
) {
    let continuation_by_id = continuation_store
        .continuations
        .iter()
        .map(|continuation| (continuation.continuation_id.as_str(), continuation))
        .collect::<BTreeMap<_, _>>();
    let mut readbacks = Vec::new();
    for attention in runtime_attention {
        readbacks.push(ReadbackResult {
            readback_id: format!("readback-result:{}", attention.attention_id),
            status: attention.readback_boundary.status.clone(),
            attempted: attention.readback_boundary.attempted,
            real_readback_performed: attention.readback_boundary.real_readback_performed,
            result_count: attention.readback_boundary.result_count,
            confidence: if attention.readback_boundary.real_readback_performed {
                "runtime_reported".to_string()
            } else {
                "boundary_only".to_string()
            },
            source_refs: attention
                .source_refs
                .iter()
                .map(|source| source_ref(&source.source_kind, &source.source_id, &source.label))
                .collect(),
            warnings: attention.readback_boundary.warnings.clone(),
        });
    }

    let mut handoffs = Vec::new();
    let mut report_candidates = Vec::new();
    for attempt in &continuation_store.attempts {
        let continuation = continuation_by_id
            .get(attempt.continuation_id.as_str())
            .copied();
        let adapter_id = continuation
            .map(|item| item.adapter_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let report_candidate = if readback_succeeded(&attempt.readback_summary.status)
            && readback_performed_from_source(&attempt.readback_summary.source_kind)
        {
            Some(WorkerReportCandidate {
                candidate_id: format!("worker-report-candidate:{}", attempt.attempt_id),
                adapter_id: adapter_id.clone(),
                project_id: continuation.map(|item| item.project_id.clone()),
                workflow_id: continuation.map(|item| item.workflow_id.clone()),
                node_id: continuation.map(|item| item.node_id.clone()),
                work_item_id: continuation.and_then(|item| item.work_item_id.clone()),
                status: "candidate_from_readback".to_string(),
                summary: "Readback completed; still requires project director process-fact review."
                    .to_string(),
                source_policy:
                    "Worker report candidate cannot directly become formal fact or formal memory."
                        .to_string(),
                source_refs: vec![source_ref(
                    "session_continuation_attempt",
                    &attempt.attempt_id,
                    &attempt.status,
                )],
                warnings: vec!["requires_project_director_process_fact_review".to_string()],
            })
        } else {
            None
        };
        if let Some(candidate) = report_candidate.clone() {
            report_candidates.push(candidate);
        }
        let readback = ReadbackResult {
            readback_id: format!("readback-result:attempt:{}", attempt.attempt_id),
            status: attempt.readback_summary.status.clone(),
            attempted: readback_attempted_from_status(&attempt.readback_summary.status),
            real_readback_performed: readback_performed_from_source(
                &attempt.readback_summary.source_kind,
            ),
            result_count: attempt.readback_summary.result_count,
            confidence: if readback_performed_from_source(&attempt.readback_summary.source_kind) {
                "attempt_reported".to_string()
            } else {
                "boundary_only".to_string()
            },
            source_refs: vec![source_ref(
                "session_continuation_attempt",
                &attempt.attempt_id,
                &attempt.status,
            )],
            warnings: attempt.readback_summary.warnings.clone(),
        };
        readbacks.push(readback.clone());
        handoffs.push(WorkerHandoff {
            handoff_id: format!("worker-handoff:{}", attempt.attempt_id),
            adapter_id,
            project_id: continuation.map(|item| item.project_id.clone()),
            workflow_id: continuation.map(|item| item.workflow_id.clone()),
            node_id: continuation.map(|item| item.node_id.clone()),
            work_item_id: continuation.and_then(|item| item.work_item_id.clone()),
            handoff_status: attempt.status.clone(),
            summary: format!(
                "Attempt {} ended with readback status {}.",
                attempt.status, attempt.readback_summary.status
            ),
            report_candidate,
            readback_result: Some(readback),
            source_refs: vec![source_ref(
                "session_continuation_attempt",
                &attempt.attempt_id,
                &attempt.status,
            )],
            warnings: attempt.warnings.clone(),
        });
    }

    (
        handoffs,
        dedupe_readbacks(readbacks),
        dedupe_report_candidates(report_candidates),
    )
}

fn run_attention_from_runtime(attention: &RuntimeSessionAttention) -> RunAttention {
    RunAttention {
        attention_id: attention.attention_id.clone(),
        kind: attention.kind.clone(),
        severity: attention.severity.clone(),
        status: attention.status.clone(),
        requires_user_action: attention.requires_user_action,
        blocks_continuation: attention.blocks_continuation,
        readback_status: attention.readback_boundary.status.clone(),
        result_count: attention.readback_boundary.result_count,
        source_refs: attention
            .source_refs
            .iter()
            .map(|source| source_ref(&source.source_kind, &source.source_id, &source.label))
            .collect(),
        warnings: attention.warnings.clone(),
    }
}

fn persistence_handle_from_parts(
    adapter_id: &str,
    native_thread_id: Option<&str>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
) -> RunPersistenceHandle {
    RunPersistenceHandle {
        handle_id: format!(
            "run-persistence:{}:{}",
            adapter_id,
            native_thread_id.unwrap_or("unbound")
        ),
        adapter_id: adapter_id.to_string(),
        native_thread_id: native_thread_id.map(str::to_string),
        project_id,
        workflow_id,
        node_id,
        work_item_id,
        persistence_kind: if native_thread_id.is_some() {
            "native_thread_reference".to_string()
        } else {
            "workbench_only".to_string()
        },
        read_policy: "metadata_only_by_default_no_full_transcript".to_string(),
        write_policy: "adapter_home_write_requires_explicit_real_execution_authorization"
            .to_string(),
        source_refs,
        warnings: vec!["run_persistence_handle_is_neutral_reference".to_string()],
    }
}

fn work_thread_id(
    adapter_id: &str,
    project_id: Option<&str>,
    workflow_id: Option<&str>,
    node_id: Option<&str>,
    session_id: Option<&str>,
) -> String {
    format!(
        "work-thread:{}:{}:{}:{}:{}",
        adapter_id,
        project_id.unwrap_or("unbound-project"),
        workflow_id.unwrap_or("unbound-workflow"),
        node_id.unwrap_or("unbound-node"),
        session_id.unwrap_or("unbound-session")
    )
}

fn neutral_worker_kind(agent_type: &str) -> String {
    match agent_type {
        "codex" => "local_cli_agent".to_string(),
        "claude-code" | "opencode" => "external_cli_agent_planned".to_string(),
        "openclaw" => "external_agent_planned".to_string(),
        _ => "adapter_neutral_worker".to_string(),
    }
}

fn capability_risk_level(kind: &str) -> &'static str {
    match kind {
        "safe_probe_dispatch" | "user_reviewed_dispatch" | "workflow_machine_run" => "high",
        "session_transcript_read" | "harness_resource_index" => "medium",
        _ => "low",
    }
}

fn capability_policy_status(adapter: &AgentAdapterDescriptor, kind: &str) -> String {
    if adapter.status == "planned" || adapter.execution_status == "not_implemented" {
        return "blocked_planned_adapter".to_string();
    }
    match kind {
        "safe_probe_dispatch" | "user_reviewed_dispatch" | "workflow_machine_run" => {
            "allowed_with_confirmation".to_string()
        }
        "session_transcript_read" => "read_only_metadata_first".to_string(),
        _ => "allowed_read_model_only".to_string(),
    }
}

fn credential_risk_for_status(status: &str) -> &'static str {
    match status {
        "not_required_by_workbench" => "managed_outside_workbench",
        "credential_missing" | "not_configured" => "missing",
        "not_read" => "not_read",
        _ => "unknown",
    }
}

fn model_risk_for_status(status: &str) -> &'static str {
    match status {
        "local_cli_managed" => "managed_outside_workbench",
        "model_unverified" | "not_verified" => "unverified",
        _ => "unknown",
    }
}

fn data_egress_risk_for_capability(kind: &str) -> &'static str {
    match kind {
        "session_transcript_read" => "transcript_sensitive_read_boundary",
        "safe_probe_dispatch" | "user_reviewed_dispatch" | "workflow_machine_run" => {
            "prompt_and_project_context_egress_risk"
        }
        _ => "low_metadata_or_read_model",
    }
}

fn lane_kind_for_thread(thread: &WorkThread) -> String {
    if thread.lifecycle_status.contains("blocked") || thread.lifecycle_status.contains("needs_user")
    {
        "reviewer".to_string()
    } else {
        "worker".to_string()
    }
}

fn lane_kind_for_run(run: &RunUnit) -> String {
    if run.lifecycle_status.contains("failed") || run.lifecycle_status.contains("timed_out") {
        "recovery".to_string()
    } else if run
        .attention
        .iter()
        .any(|attention| attention.requires_user_action || attention.blocks_continuation)
    {
        "reviewer".to_string()
    } else {
        "worker".to_string()
    }
}

fn readback_succeeded(status: &str) -> bool {
    matches!(status, "succeeded" | "completed")
}

fn readback_attempted_from_status(status: &str) -> bool {
    !matches!(status, "not_attempted" | "readback_unavailable")
}

fn readback_performed_from_source(source_kind: &str) -> bool {
    !source_kind.contains("no_transcript_read")
        && !source_kind.contains("no_raw_transcript_read")
        && !source_kind.contains("stub_no_transcript_read")
        && !source_kind.trim().is_empty()
}

fn source_ref(kind: &str, id: &str, label: &str) -> WorkerProtocolSourceRef {
    WorkerProtocolSourceRef {
        source_kind: kind.to_string(),
        source_id: id.to_string(),
        label: label.to_string(),
    }
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn dedupe_source_refs(values: Vec<WorkerProtocolSourceRef>) -> Vec<WorkerProtocolSourceRef> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| {
            seen.insert(format!(
                "{}:{}:{}",
                value.source_kind, value.source_id, value.label
            ))
        })
        .collect()
}

fn dedupe_readbacks(values: Vec<ReadbackResult>) -> Vec<ReadbackResult> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.readback_id.clone()))
        .collect()
}

fn dedupe_report_candidates(values: Vec<WorkerReportCandidate>) -> Vec<WorkerReportCandidate> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.candidate_id.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdapterCapability, AgentAdapterDescriptor, CodexLocalReadbackPlan, ContinuationAuditImpact,
        ContinuationFailureBoundary, ProviderAvailabilitySummary, ReadbackBoundaryStatus,
        ReadbackExpectation, RuntimeAttentionSourceRef, RuntimeLogBoundary, RuntimeLogStoreScope,
        SessionContinuationAttempt, SessionContinuationGuardResult,
        SessionContinuationReadbackSummary, SessionContinuationRequest,
        SessionContinuationStoreScope,
    };

    #[test]
    fn worker_protocol_maps_codex_local_without_executing() {
        let adapters = vec![codex_adapter()];
        let operations = vec![resume_operation()];
        let providers = vec![codex_provider()];
        let previews = vec![resume_preview()];
        let store = empty_store();
        let runtime_attention = vec![runtime_attention("readback_unavailable", None)];
        let runtime_log = empty_runtime_log();

        let model = derive_worker_protocol_read_model(
            &adapters,
            &operations,
            &providers,
            &previews,
            &store,
            &runtime_attention,
            &runtime_log,
            "2026-06-08T00:00:00Z",
        );

        assert_eq!(model.worker_adapters.len(), 1);
        assert_eq!(model.worker_adapters[0].adapter_id, "codex-local");
        assert_eq!(
            model.worker_adapters[0].source_policy,
            "codex-local maps into neutral WorkerAdapter; protocol must remain adapter-neutral."
        );
        assert_eq!(model.dispatch_requests.len(), 1);
        assert_eq!(model.permission_envelopes.len(), 1);
        assert!(!model.permission_envelopes[0].approved_for_real_execution);
        assert!(model
            .warnings
            .contains(&"does_not_execute_worker".to_string()));
    }

    #[test]
    fn worker_protocol_preserves_unknown_readback_as_null_count() {
        let model = derive_worker_protocol_read_model(
            &[],
            &[],
            &[],
            &[],
            &empty_store(),
            &[runtime_attention("readback_unavailable", None)],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        assert_eq!(model.readback_results.len(), 1);
        assert_eq!(model.readback_results[0].status, "readback_unavailable");
        assert_eq!(model.readback_results[0].result_count, None);
    }

    #[test]
    fn worker_protocol_creates_handoff_candidate_only_after_real_readback() {
        let mut store = empty_store();
        store
            .continuations
            .push(crate::ControlledSessionContinuation {
                record_version: 1,
                continuation_id: "continuation-1".to_string(),
                preview_id: "preview-1".to_string(),
                adapter_id: "codex-local".to_string(),
                operation_id: "resume".to_string(),
                project_id: "project-1".to_string(),
                project_root: "/tmp/project".to_string(),
                workflow_id: "workflow-1".to_string(),
                node_id: "node-1".to_string(),
                session_id: "thread-1".to_string(),
                work_item_id: Some("work-item-1".to_string()),
                target_cwd: "/tmp/project".to_string(),
                allowed_write_roots: vec!["/tmp/project".to_string()],
                sandbox: "workspace-write".to_string(),
                prompt_source_kind: "task_package_summary".to_string(),
                prompt_summary: "do work".to_string(),
                command_preview: "codex exec resume <redacted>".to_string(),
                readback_strategy: "required".to_string(),
                status: "completed".to_string(),
                execution_level: "level_b".to_string(),
                runner_kind: "codex-local".to_string(),
                user_confirmation_state: "confirmed".to_string(),
                guard_status: "allowed".to_string(),
                requested_by: "test".to_string(),
                confirmed_by: "user".to_string(),
                confirmation_reason: "fixture".to_string(),
                created_at: "2026-06-08T00:00:00Z".to_string(),
                updated_at: "2026-06-08T00:00:01Z".to_string(),
                audit_refs: vec![],
                warnings: vec![],
            });
        store.attempts.push(SessionContinuationAttempt {
            attempt_version: 1,
            attempt_id: "attempt-1".to_string(),
            continuation_id: "continuation-1".to_string(),
            runner_kind: "codex-local".to_string(),
            execution_level: "level_b".to_string(),
            status: "completed".to_string(),
            started_at: "2026-06-08T00:00:01Z".to_string(),
            finished_at: Some("2026-06-08T00:00:02Z".to_string()),
            timeout_ms: Some(1000),
            command_preview: "codex exec resume <redacted>".to_string(),
            prompt_sent: true,
            real_codex_executed: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            readback_summary: SessionContinuationReadbackSummary {
                status: "succeeded".to_string(),
                source_kind: "h2_phase_b_workbench_managed_last_message".to_string(),
                result_count: Some(1),
                unavailable_reason: None,
                warnings: vec![],
            },
            failure_reason: None,
            audit_refs: vec![],
            warnings: vec![],
        });

        let model = derive_worker_protocol_read_model(
            &[codex_adapter()],
            &[resume_operation()],
            &[codex_provider()],
            &[],
            &store,
            &[],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        assert_eq!(model.run_units.len(), 1);
        assert!(model.run_units[0].real_worker_executed);
        assert_eq!(model.worker_handoffs.len(), 1);
        assert_eq!(model.worker_report_candidates.len(), 1);
        assert!(model.worker_handoffs[0].report_candidate.is_some());
    }

    #[test]
    fn worker_protocol_keeps_planned_adapter_risk_blocked_without_credentials() {
        let mut planned = codex_adapter();
        planned.adapter_id = "opencode".to_string();
        planned.agent_type = "opencode".to_string();
        planned.display_name = "OpenCode".to_string();
        planned.provider = "opencode-planned".to_string();
        planned.status = "planned".to_string();
        planned.execution_status = "not_implemented".to_string();
        planned.credential_status = "not_configured".to_string();
        planned.model_access_status = "not_verified".to_string();
        planned.capabilities[0].kind = "workflow_machine_run".to_string();
        let provider = ProviderAvailabilitySummary {
            adapter_id: "opencode".to_string(),
            provider_id: "opencode-planned".to_string(),
            provider_label: "OpenCode".to_string(),
            provider_kind: "external_cli_planned".to_string(),
            adapter_status: "planned".to_string(),
            availability_status: "planned".to_string(),
            credential_status: "credential_missing".to_string(),
            model_status: "model_unverified".to_string(),
            external_call_status: "external_call_blocked".to_string(),
            cost_risk_status: "blocked_until_authorized".to_string(),
            user_visible_reason: "planned".to_string(),
            safe_to_display: true,
            requires_user_configuration: true,
            requires_future_task: true,
            warnings: vec!["planned_adapter_not_connected".to_string()],
        };

        let model = derive_worker_protocol_read_model(
            &[planned],
            &[],
            &[provider],
            &[],
            &empty_store(),
            &[],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        assert_eq!(model.credential_requirements.len(), 1);
        assert!(model.credential_requirements[0].required_for_real_execution);
        assert_eq!(
            model.external_call_risk_envelopes[0].project_policy_status,
            "blocked_planned_adapter"
        );
        assert!(model.external_call_risk_envelopes[0]
            .warnings
            .contains(&"planned_adapter_external_call_blocked".to_string()));
    }

    #[test]
    fn worker_protocol_derives_multi_worker_reviewer_and_recovery_lanes() {
        let mut store = empty_store();
        store
            .continuations
            .push(crate::ControlledSessionContinuation {
                record_version: 1,
                continuation_id: "continuation-1".to_string(),
                preview_id: "preview-1".to_string(),
                adapter_id: "codex-local".to_string(),
                operation_id: "resume".to_string(),
                project_id: "project-1".to_string(),
                project_root: "/tmp/project".to_string(),
                workflow_id: "workflow-1".to_string(),
                node_id: "node-1".to_string(),
                session_id: "thread-1".to_string(),
                work_item_id: Some("work-item-1".to_string()),
                target_cwd: "/tmp/project".to_string(),
                allowed_write_roots: vec!["/tmp/project".to_string()],
                sandbox: "workspace-write".to_string(),
                prompt_source_kind: "task_package_summary".to_string(),
                prompt_summary: "do work".to_string(),
                command_preview: "codex exec resume <redacted>".to_string(),
                readback_strategy: "required".to_string(),
                status: "failed".to_string(),
                execution_level: "level_b".to_string(),
                runner_kind: "codex-local".to_string(),
                user_confirmation_state: "confirmed".to_string(),
                guard_status: "allowed".to_string(),
                requested_by: "test".to_string(),
                confirmed_by: "user".to_string(),
                confirmation_reason: "fixture".to_string(),
                created_at: "2026-06-08T00:00:00Z".to_string(),
                updated_at: "2026-06-08T00:00:01Z".to_string(),
                audit_refs: vec![],
                warnings: vec![],
            });
        store.attempts.push(SessionContinuationAttempt {
            attempt_version: 1,
            attempt_id: "attempt-1".to_string(),
            continuation_id: "continuation-1".to_string(),
            runner_kind: "codex-local".to_string(),
            execution_level: "level_b".to_string(),
            status: "failed".to_string(),
            started_at: "2026-06-08T00:00:01Z".to_string(),
            finished_at: Some("2026-06-08T00:00:02Z".to_string()),
            timeout_ms: Some(1000),
            command_preview: "codex exec resume <redacted>".to_string(),
            prompt_sent: true,
            real_codex_executed: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            readback_summary: SessionContinuationReadbackSummary {
                status: "readback_failed".to_string(),
                source_kind: "h2_phase_b_workbench_managed_last_message".to_string(),
                result_count: None,
                unavailable_reason: Some("fixture_failure".to_string()),
                warnings: vec!["unknown_result_not_zero".to_string()],
            },
            failure_reason: Some("fixture_failure".to_string()),
            audit_refs: vec![],
            warnings: vec![],
        });

        let model = derive_worker_protocol_read_model(
            &[codex_adapter()],
            &[resume_operation()],
            &[codex_provider()],
            &[resume_preview()],
            &store,
            &[runtime_attention("readback_failed", None)],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        assert!(model
            .worker_lanes
            .iter()
            .any(|lane| lane.lane_kind == "reviewer" && lane.reviewer_required));
        assert!(model
            .worker_lanes
            .iter()
            .any(|lane| lane.lane_kind == "recovery"));
        assert!(model
            .multi_worker_dispatch_plans
            .iter()
            .any(|plan| plan.verifier_lane_required));
        assert!(model
            .warnings
            .contains(&"multi_worker_plan_requires_reviewer_or_verifier_lane".to_string()));
    }

    #[test]
    fn worker_protocol_i5_blocks_planned_adapter_contract_without_runtime_model_or_location() {
        let mut planned = codex_adapter();
        planned.adapter_id = "claude-code".to_string();
        planned.agent_type = "claude-code".to_string();
        planned.display_name = "Claude Code".to_string();
        planned.provider = "claude-code-planned".to_string();
        planned.status = "planned".to_string();
        planned.execution_status = "not_implemented".to_string();
        planned.credential_status = "not_configured".to_string();
        planned.model_access_status = "not_verified".to_string();
        planned.capabilities[0].kind = "workflow_machine_run".to_string();
        let provider = ProviderAvailabilitySummary {
            adapter_id: "claude-code".to_string(),
            provider_id: "claude-code-planned".to_string(),
            provider_label: "Claude Code".to_string(),
            provider_kind: "external_cli_planned".to_string(),
            adapter_status: "planned".to_string(),
            availability_status: "planned".to_string(),
            credential_status: "credential_missing".to_string(),
            model_status: "model_unverified".to_string(),
            external_call_status: "external_call_blocked".to_string(),
            cost_risk_status: "blocked_until_authorized".to_string(),
            user_visible_reason: "planned".to_string(),
            safe_to_display: true,
            requires_user_configuration: true,
            requires_future_task: true,
            warnings: vec!["planned_adapter_not_connected".to_string()],
        };

        let model = derive_worker_protocol_read_model(
            &[planned],
            &[],
            &[provider],
            &[],
            &empty_store(),
            &[],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        let checklist = &model.adapter_contract_checklists[0];
        assert_eq!(checklist.status, "blocked_or_reserved_contract");
        assert!(checklist.control_core_required);
        assert!(checklist.permission_required);
        assert!(checklist.audit_required);
        assert!(checklist.runtime_log_required);
        assert!(checklist
            .missing_items
            .contains(&"runtime_connection_not_implemented".to_string()));
        assert!(checklist
            .missing_items
            .contains(&"model_boundary_or_verification_missing".to_string()));
        assert!(checklist
            .missing_items
            .contains(&"data_location_reserved_not_connected".to_string()));
        assert!(model
            .warnings
            .contains(&"adapter_contract_checklist_has_blocking_items".to_string()));
    }

    #[test]
    fn worker_protocol_i5_cli_parity_requires_control_core_permission_and_audit() {
        let model = derive_worker_protocol_read_model(
            &[codex_adapter()],
            &[resume_operation()],
            &[codex_provider()],
            &[resume_preview()],
            &empty_store(),
            &[],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        let semantics = &model.controlled_api_cli_semantics[0];
        assert_eq!(semantics.adapter_id, "codex-local");
        assert!(semantics.universal_api_backdoor_blocked);
        assert_eq!(semantics.control_core_path, "required_before_runner");
        assert_eq!(
            semantics.permission_path,
            "explicit_user_confirmation_required_for_real_execution"
        );
        assert_eq!(semantics.audit_path, "runtime_log_and_audit_refs_required");
        assert!(semantics
            .supported_operation_ids
            .contains(&"resume".to_string()));
        assert!(model
            .warnings
            .contains(&"cli_parity_requires_control_core_permission_audit".to_string()));
    }

    #[test]
    fn worker_protocol_i5_health_degraded_and_data_location_stay_readonly() {
        let mut store = empty_store();
        store
            .continuations
            .push(crate::ControlledSessionContinuation {
                record_version: 1,
                continuation_id: "continuation-1".to_string(),
                preview_id: "preview-1".to_string(),
                adapter_id: "codex-local".to_string(),
                operation_id: "resume".to_string(),
                project_id: "project-1".to_string(),
                project_root: "/tmp/project".to_string(),
                workflow_id: "workflow-1".to_string(),
                node_id: "node-1".to_string(),
                session_id: "thread-1".to_string(),
                work_item_id: Some("work-item-1".to_string()),
                target_cwd: "/tmp/project".to_string(),
                allowed_write_roots: vec!["/tmp/project".to_string()],
                sandbox: "workspace-write".to_string(),
                prompt_source_kind: "task_package_summary".to_string(),
                prompt_summary: "do work".to_string(),
                command_preview: "codex exec resume <redacted>".to_string(),
                readback_strategy: "required".to_string(),
                status: "failed_stub".to_string(),
                execution_level: "level_a".to_string(),
                runner_kind: "codex-local".to_string(),
                user_confirmation_state: "confirmed".to_string(),
                guard_status: "allowed".to_string(),
                requested_by: "test".to_string(),
                confirmed_by: "user".to_string(),
                confirmation_reason: "fixture".to_string(),
                created_at: "2026-06-08T00:00:00Z".to_string(),
                updated_at: "2026-06-08T00:00:01Z".to_string(),
                audit_refs: vec![],
                warnings: vec![],
            });
        store.attempts.push(SessionContinuationAttempt {
            attempt_version: 1,
            attempt_id: "attempt-1".to_string(),
            continuation_id: "continuation-1".to_string(),
            runner_kind: "codex-local".to_string(),
            execution_level: "level_a".to_string(),
            status: "failed_stub".to_string(),
            started_at: "2026-06-08T00:00:01Z".to_string(),
            finished_at: Some("2026-06-08T00:00:02Z".to_string()),
            timeout_ms: Some(1000),
            command_preview: "codex exec resume <redacted>".to_string(),
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_workbench_state: true,
            readback_summary: SessionContinuationReadbackSummary {
                status: "readback_unavailable".to_string(),
                source_kind: "stub_no_transcript_read".to_string(),
                result_count: None,
                unavailable_reason: Some("stub".to_string()),
                warnings: vec!["unknown_result_not_zero".to_string()],
            },
            failure_reason: Some("stub".to_string()),
            audit_refs: vec![],
            warnings: vec!["stub_failed".to_string()],
        });

        let model = derive_worker_protocol_read_model(
            &[codex_adapter()],
            &[resume_operation()],
            &[codex_provider()],
            &[],
            &store,
            &[],
            &empty_runtime_log(),
            "2026-06-08T00:00:00Z",
        );

        let health = &model.adapter_health_summaries[0];
        assert_eq!(health.status, "degraded_runtime_attention");
        assert_eq!(health.runtime_status, "has_failed_or_timed_out_attempt");
        let degraded = &model.adapter_degraded_modes[0];
        assert!(degraded.blocks_real_execution);
        assert!(degraded
            .blocked_surfaces
            .contains(&"universal_api_backdoor".to_string()));
        let location = &model.adapter_data_locations[0];
        assert_eq!(
            location.adapter_home_policy,
            "no_codex_home_read_write_without_execution_point_authorization"
        );
        assert_eq!(
            location.secret_policy,
            "never_read_auth_token_env_keychain_oauth_provider_credentials"
        );
        assert!(model.diagnostic_event_schemas[0]
            .required_fields
            .contains(&"redacted_summary".to_string()));
        assert_eq!(
            model.diagnostic_event_schemas[0].redaction_policy,
            "no_secret_no_raw_transcript_no_provider_payload"
        );
    }

    fn codex_adapter() -> AgentAdapterDescriptor {
        AgentAdapterDescriptor {
            adapter_id: "codex-local".to_string(),
            agent_type: "codex".to_string(),
            agent_id: "codex-local".to_string(),
            display_name: "Codex".to_string(),
            provider: "local-codex-index".to_string(),
            status: "available".to_string(),
            permission_level: "read_only".to_string(),
            source_kind: "backend_read_model".to_string(),
            capabilities: vec![AdapterCapability {
                capability_id: "codex-local:workflow_machine_run".to_string(),
                kind: "workflow_machine_run".to_string(),
                label: "四角色工作流机器".to_string(),
                status: "requires_confirmation".to_string(),
                description: "requires confirmation".to_string(),
                boundary: "必须用户确认；本模型不执行。".to_string(),
                evidence_refs: vec!["binding-1".to_string()],
                warnings: vec![],
            }],
            implemented_action_kinds: vec![],
            hidden_unimplemented_adapters: vec![],
            warnings: vec![],
            execution_status: "available_with_user_confirmation".to_string(),
            credential_status: "not_read".to_string(),
            model_access_status: "local_read_model_only".to_string(),
            permission_boundary: "必须用户确认".to_string(),
            unavailable_reason: None,
            requires_user_setup: false,
        }
    }

    fn resume_operation() -> SessionOperationDescriptor {
        SessionOperationDescriptor {
            operation_id: "resume".to_string(),
            label: "resume".to_string(),
            category: "interactive_control".to_string(),
            current_status: "requires_future_task".to_string(),
            risk_level: "high".to_string(),
            adapter_id: "codex-local".to_string(),
            agent_type: "codex".to_string(),
            applies_to_session_state: "bound_or_existing_session".to_string(),
            requires_user_confirmation: true,
            writes_codex_home: true,
            writes_workbench_state: true,
            writes_project_files: false,
            reads_full_transcript: false,
            requires_credential: false,
            requires_model_access: true,
            requires_runtime_handle: false,
            audit_requirement: "audit required".to_string(),
            unavailable_reason: "requires authorization".to_string(),
            future_task_hint: "future task".to_string(),
            warnings: vec![],
        }
    }

    fn codex_provider() -> ProviderAvailabilitySummary {
        ProviderAvailabilitySummary {
            adapter_id: "codex-local".to_string(),
            provider_id: "local-codex-cli".to_string(),
            provider_label: "Codex 本地 CLI".to_string(),
            provider_kind: "local_cli".to_string(),
            adapter_status: "available".to_string(),
            availability_status: "available_readonly".to_string(),
            credential_status: "not_required_by_workbench".to_string(),
            model_status: "local_cli_managed".to_string(),
            external_call_status: "not_needed_for_readonly".to_string(),
            cost_risk_status: "unknown".to_string(),
            user_visible_reason: "readonly".to_string(),
            safe_to_display: true,
            requires_user_configuration: false,
            requires_future_task: false,
            warnings: vec![],
        }
    }

    fn resume_preview() -> SessionContinuationPreview {
        let request = SessionContinuationRequest {
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            project_id: Some("project-1".to_string()),
            project_root: Some("/tmp/project".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            session_id: Some("thread-1".to_string()),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: Some("/tmp/project".to_string()),
            allowed_write_roots: vec!["/tmp/project".to_string()],
            sandbox: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "task_package_summary".to_string(),
            prompt_summary: "do work".to_string(),
            readback_strategy: "required".to_string(),
            requested_by: "test".to_string(),
            user_confirmation_state: "missing".to_string(),
        };
        SessionContinuationPreview {
            preview_id: "preview-1".to_string(),
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            target_session_id: Some("thread-1".to_string()),
            target_session_title: Some("thread".to_string()),
            project_id: Some("project-1".to_string()),
            project_root: Some("/tmp/project".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            binding_id: Some("binding-1".to_string()),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: Some("/tmp/project".to_string()),
            allowed_write_roots_summary: vec!["/tmp/project".to_string()],
            sandbox_summary: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "task_package_summary".to_string(),
            prompt_summary: "do work".to_string(),
            readback_expectation: ReadbackExpectation {
                strategy: "required".to_string(),
                required: true,
                expected_sources: vec!["last_message".to_string()],
                unavailable_behavior: "result_count_null".to_string(),
                warnings: vec![],
            },
            failure_handling: ContinuationFailureBoundary {
                timeout_policy: "record_failed".to_string(),
                retry_policy: "manual".to_string(),
                failure_record: "write_attempt_failure".to_string(),
                user_visible_behavior: "show_readback_boundary".to_string(),
                warnings: vec![],
            },
            audit_impact: ContinuationAuditImpact {
                impact_kind: "preview_only".to_string(),
                writes_attempt_in_e4: false,
                writes_dispatch_in_e4: false,
                writes_readback_in_e4: false,
                future_audit_requirement: "audit required".to_string(),
                warnings: vec![],
            },
            provider_availability_summary: Some(codex_provider()),
            guard_result: SessionContinuationGuardResult {
                status: "needs_user_confirmation".to_string(),
                severity: "needs_user".to_string(),
                blocks_execution: false,
                allows_preview: true,
                requires_user_confirmation: true,
                reasons: vec!["confirmation_missing".to_string()],
                required_fixes: vec!["confirm".to_string()],
                warnings: vec![],
            },
            request,
            user_visible_warnings: vec!["user_confirmation_required_before_execution".to_string()],
        }
    }

    fn runtime_attention(status: &str, result_count: Option<i64>) -> RuntimeSessionAttention {
        let source_refs = vec![RuntimeAttentionSourceRef {
            source_kind: "session_continuation_preview".to_string(),
            source_id: "preview-1".to_string(),
            label: "preview".to_string(),
        }];
        RuntimeSessionAttention {
            attention_id: "attention-1".to_string(),
            project_id: Some("project-1".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            session_id: Some("thread-1".to_string()),
            adapter_id: "codex-local".to_string(),
            source_refs: source_refs.clone(),
            kind: "readback_boundary".to_string(),
            severity: "warning".to_string(),
            status: status.to_string(),
            title: "readback".to_string(),
            user_message: "readback boundary".to_string(),
            technical_summary: "boundary".to_string(),
            recommended_next_step: "review".to_string(),
            requires_user_action: true,
            blocks_continuation: false,
            readback_boundary: ReadbackBoundaryStatus {
                status: status.to_string(),
                reason: "unknown".to_string(),
                attempted: false,
                real_readback_performed: false,
                result_count,
                user_message: "unknown".to_string(),
                technical_summary: "unknown".to_string(),
                source_refs,
                warnings: vec!["unknown_result_not_zero".to_string()],
            },
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
            warnings: vec![],
        }
    }

    fn empty_store() -> SessionContinuationStoreV1 {
        SessionContinuationStoreV1 {
            schema_version: "session_continuation_store.v1".to_string(),
            store_version: 1,
            storage_kind: "sidecar_json_v0".to_string(),
            scope: SessionContinuationStoreScope {
                scope_kind: "workflow_state_sidecar".to_string(),
                workflow_state_path: None,
                sidecar_path: None,
                project_roots: vec![],
            },
            revision: 0,
            last_write_id: None,
            generated_by: "test".to_string(),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
            continuations: vec![],
            attempts: vec![],
            audit_events: vec![],
            warnings: vec![],
        }
    }

    fn empty_runtime_log() -> RuntimeLogStoreV1 {
        RuntimeLogStoreV1 {
            schema_version: "runtime_log_store.v1".to_string(),
            store_version: 1,
            storage_kind: "sidecar_json_v0".to_string(),
            scope: RuntimeLogStoreScope {
                scope_kind: "workflow_state_sidecar".to_string(),
                workflow_state_path: None,
                sidecar_path: None,
                project_roots: vec![],
            },
            revision: 0,
            last_write_id: None,
            generated_by: "test".to_string(),
            created_at: "2026-06-08T00:00:00Z".to_string(),
            updated_at: "2026-06-08T00:00:00Z".to_string(),
            boundary: RuntimeLogBoundary {
                runtime_log_definition: "runtime".to_string(),
                audit_event_definition: "audit".to_string(),
                separation_rule: "separate".to_string(),
                redaction_rule: "redact".to_string(),
                forbidden_payloads: vec![],
            },
            entries: vec![],
            summaries: vec![],
            warnings: vec![],
        }
    }

    #[allow(dead_code)]
    fn _readback_plan() -> CodexLocalReadbackPlan {
        CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![],
            unavailable_behavior: "null_count".to_string(),
            trust_policy: "boundary".to_string(),
            warnings: vec![],
        }
    }
}
