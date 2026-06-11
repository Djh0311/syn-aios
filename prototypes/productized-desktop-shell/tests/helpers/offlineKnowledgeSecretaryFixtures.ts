import type {
  BlackboardCandidateStoreV1,
  FormalMemoryStoreV1,
  MemoryCandidateStoreV1,
  ProjectRecord,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../../src/lib/types";

const workflowProjectId = "project:offline-fixture-projects-codex-workbench";
const workflowId = "workflow:offline-fixture-projects-codex-workbench:default";

export function knowledgeBaseBoundaryFixtures(
  project: ProjectRecord,
  workflowStateWithDerivedWorkflow: WorkflowStateSnapshot,
): {
  formalMemoryStore: FormalMemoryStoreV1;
  knowledgeWorkflowState: WorkflowStateSnapshot;
  memoryCandidateStore: MemoryCandidateStoreV1;
  projectWithKnowledge: ProjectRecord;
} {
  const knowledgePath = "/offline-fixture/projects/codex-workbench/docs/interface-contract.md";
  const projectWithKnowledge: ProjectRecord = {
    ...project,
    authority_files: [
      {
        kind: "knowledge_doc",
        name: "接口契约资料",
        path: knowledgePath,
        warnings: [],
      },
    ],
  };
  const formalMemoryStore: FormalMemoryStoreV1 = {
    store_version: "formal_memory_store.v1",
    project_id: workflowProjectId,
    workflow_id: workflowId,
    revision: 9,
    records: [
      {
        memory_id: "mem:formal:knowledge:001",
        schema_version: "memory_governance.v1",
        record_version: 1,
        scope: {
          scope_id: "scope:project:knowledge",
          scope_type: "project",
          project_id: workflowProjectId,
          workflow_id: workflowId,
          role_ids: [],
          document_refs: [knowledgePath],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T01:00:00Z",
        },
        memory_type: "project_memory",
        claim: "接口契约资料可以作为正式记忆来源。",
        body: "正式记忆引用 knowledge_doc 来源，但资料本身不是正式记忆。",
        source_refs: [
          {
            source_ref_id: "source:knowledge:formal:001",
            source_type: "knowledge_doc",
            source_id: "knowledge-doc:interface-contract",
            source_path: knowledgePath,
            source_title: "接口契约资料",
            anchor: "接口边界",
            captured_at: "2026-06-05T01:00:00Z",
            authority_level: "knowledge_material",
            sensitive_level: "project",
          },
        ],
        status: "memory_active",
        supersedes_memory_id: null,
        superseded_by_memory_id: null,
        conflict_refs: [],
        audit_refs: [],
        created_at: "2026-06-05T01:00:00Z",
        updated_at: "2026-06-05T01:00:00Z",
      },
    ],
    versions: [],
    audit_events: [],
    updated_at: "2026-06-05T01:00:00Z",
    warnings: [],
  };
  const memoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    project_id: workflowProjectId,
    workflow_id: workflowId,
    revision: 10,
    candidates: [
      {
        candidate_id: "memcand:knowledge:001",
        candidate_key: "memcand:v1:knowledge-interface",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:knowledge",
          scope_type: "project",
          project_id: workflowProjectId,
          workflow_id: workflowId,
          role_ids: [],
          document_refs: [knowledgePath],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T01:00:01Z",
        },
        memory_type: "project_memory",
        claim: "知识库资料可提出候选。",
        body: "候选仍需确认和受控采纳，不能直接进入正式记忆。",
        source_refs: [
          {
            source_ref_id: "source:knowledge:candidate:001",
            source_type: "knowledge_doc",
            source_id: "knowledge-doc:interface-contract",
            source_path: knowledgePath,
            source_title: "接口契约资料",
            anchor: "候选锚点",
            captured_at: "2026-06-05T01:00:01Z",
            authority_level: "knowledge_material",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "project_director",
        generated_from: "knowledge_summary",
        status: "candidate_needs_review",
        risk_level: "low",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "从明确知识库资料提出候选。",
        conflicts: [],
        audit_refs: [],
        adoption: null,
        created_at: "2026-06-05T01:00:01Z",
        updated_at: "2026-06-05T01:00:01Z",
      },
    ],
    events: [],
    updated_at: "2026-06-05T01:00:01Z",
  };
  const knowledgeWorkflowState: WorkflowStateSnapshot = {
    ...workflowStateWithDerivedWorkflow,
    project_workflows: [
      {
        ...workflowStateWithDerivedWorkflow.project_workflows[0],
        derived_workflow: {
          ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!,
          task_packages: [
            {
              ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!.task_packages[0],
              available_knowledge_refs: [knowledgePath],
            },
          ],
        },
      },
    ],
  };

  return {
    formalMemoryStore,
    knowledgeWorkflowState,
    memoryCandidateStore,
    projectWithKnowledge,
  };
}

export function secretaryReadModelFixtures(
  snapshot: WorkbenchSnapshot,
  project: ProjectRecord,
): {
  blackboardCandidateStore: BlackboardCandidateStoreV1;
  memoryCandidateStore: MemoryCandidateStoreV1;
  secretarySnapshot: WorkbenchSnapshot;
} {
  const secretarySnapshot: WorkbenchSnapshot = {
    ...snapshot,
    summary: {
      ...snapshot.summary,
      warning_count: 2,
    },
    diagnostics: {
      ...snapshot.diagnostics,
      top_level_warning_count: 1,
      notes: ["offline diagnostic warning"],
    },
  };
  const blackboardCandidateStore: BlackboardCandidateStoreV1 = {
    schema_version: "blackboard_candidate_persistence.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    revision: 4,
    records: [
      {
        candidate_key: "bbcand:v1:offline-pending",
        project_id: workflowProjectId,
        project_root: project.project_root,
        workflow_id: workflowId,
        source_entry_id: "blackboard:offline:risk:001",
        entry_kind: "risk",
        target_kind: "workflow_risk",
        state: "candidate_pending_control_core",
        title_snapshot: "方向风险候选",
        summary_snapshot: "direction_risk_fixture",
        source_refs: [{ source_kind: "blackboard_entry", source_id: "blackboard:offline:risk:001", label: "方向风险候选" }],
        updated_at: "2026-06-03T00:00:00Z",
        warnings: [],
      },
    ],
    audit_events: [],
    updated_at: "2026-06-03T00:00:00Z",
    warnings: [],
  };
  const memoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    revision: 5,
    candidates: [
      {
        candidate_id: "memcand:offline:secretary:001",
        candidate_key: "memcand:v1:offline-secretary",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: workflowProjectId,
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-03T00:00:00Z",
        },
        memory_type: "project_memory",
        claim: "项目需要保留候选治理边界。",
        body: "候选不是正式记忆，必须等待控制核心和用户确认。",
        source_refs: [
          {
            source_ref_id: "source:offline:secretary:001",
            source_type: "stage_report",
            source_id: "stage:offline",
            source_title: "离线秘书测试",
            captured_at: "2026-06-03T00:00:00Z",
            authority_level: "derived_summary",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "secretary",
        generated_from: "secretary_suggestion",
        status: "candidate_needs_review",
        risk_level: "medium",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "离线秘书只读模型测试",
        conflicts: [],
        audit_refs: [],
        created_at: "2026-06-03T00:00:00Z",
        updated_at: "2026-06-03T00:00:00Z",
      },
    ],
    events: [],
    updated_at: "2026-06-03T00:00:00Z",
  };

  return {
    blackboardCandidateStore,
    memoryCandidateStore,
    secretarySnapshot,
  };
}
