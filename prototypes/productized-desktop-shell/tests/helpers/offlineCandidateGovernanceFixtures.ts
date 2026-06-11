import type {
  BlackboardCandidateStoreV1,
  MemoryCandidate,
  MemoryCandidateStoreV1,
  ObservationStoreV1,
} from "../../src/lib/types";
import { candidateMemoryLifecycleFixtures } from "./offlineCandidateMemoryLifecycleFixtures";

const workflowProjectId = "project:offline-fixture-projects-codex-workbench";
const workflowId = "workflow:offline-fixture-projects-codex-workbench:default";

export function candidateGovernanceFixtures(projectRoot: string) {
  const blackboardCandidateStore: BlackboardCandidateStoreV1 = {
    schema_version: "blackboard_candidate_persistence.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    revision: 3,
    records: [
      {
        candidate_key: "bbcand:v1:offline-report",
        project_id: projectRoot,
        workflow_id: workflowId,
        source_entry_id: "blackboard:offline:report:001",
        entry_kind: "subagent_report",
        target_kind: "workflow_fact",
        state: "candidate_confirmed_for_followup",
        title_snapshot: "离线子汇报",
        summary_snapshot: "只确认后续处理，不写正式事实。",
        source_refs: [{ source_kind: "subagent_report", source_id: "report:offline:001", label: "子智能体汇报" }],
        updated_at: "2026-06-03T00:00:00Z",
        warnings: [],
      },
    ],
    audit_events: [],
    updated_at: "2026-06-03T00:00:00Z",
    warnings: [],
  };

  const confirmedMemoryCandidate: MemoryCandidate = {
    candidate_id: "memcand:offline:001",
    candidate_key: "memcand:v1:offline-preference",
    schema_version: "memory_governance.v1",
    scope: {
      scope_id: "scope:user:yoyi",
      scope_type: "user_preference",
      role_ids: [],
      document_refs: [],
      model_export_policy: "local_only",
      valid_from: "2026-06-03T00:00:00Z",
    },
    memory_type: "user_preference",
    claim: "用户要求先指出风险。",
    body: "候选已确认保留，但不是正式长期记忆。",
    source_refs: [
      {
        source_ref_id: "source:user-confirmed:001",
        source_type: "user_confirmed_proposal",
        source_id: "task:offline",
        source_title: "离线确认",
        captured_at: "2026-06-03T00:00:00Z",
        authority_level: "user_confirmed",
        sensitive_level: "private",
      },
    ],
    generated_by_role: "user",
    generated_from: "explicit_user_confirmation",
    status: "candidate_confirmed",
    risk_level: "low",
    sensitive_level: "private",
    requires_user_confirmation: true,
    review_reason: "离线候选治理测试",
    conflicts: [],
    audit_refs: [],
    adoption: null,
    created_at: "2026-06-03T00:00:00Z",
    updated_at: "2026-06-03T00:00:00Z",
  };

  const confirmedMemoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    revision: 2,
    candidates: [confirmedMemoryCandidate],
    events: [],
    updated_at: "2026-06-03T00:00:00Z",
  };

  const adoptedMemoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    revision: 3,
    candidates: [
      {
        ...confirmedMemoryCandidate,
        candidate_id: "memcand:offline:002",
        candidate_key: "memcand:v1:offline-project",
        adoption: {
          adopted_memory_id: "mem:formal:offline:002",
          adopted_version_id: "memver:formal:offline:002",
          adopted_audit_event_id: "audit:formal:offline:002",
          adopted_at: "2026-06-03T00:00:02Z",
          adopted_by_role: "project_director",
          adoption_reason: "项目主管采纳低风险本项目记忆候选。",
        },
      },
    ],
    events: [],
    updated_at: "2026-06-03T00:00:02Z",
  };

  const observationStore: ObservationStoreV1 = {
    store_version: "observation_store.v1",
    project_id: workflowProjectId,
    workflow_id: workflowId,
    revision: 2,
    observations: [
      {
        observation_id: "obs:v1:offline:001",
        observation_key: "obs:v1:offline-recorded",
        schema_version: "memory_observation.v1",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: workflowProjectId,
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-04T00:00:00Z",
        },
        observation_type: "worker_report",
        summary: "开发线汇报：观察入口已经写入 sidecar。",
        source_refs: [
          {
            source_ref_id: "obs-source:offline:001",
            source_kind: "worker_report",
            source_id: "worker-report:offline:001",
            project_id: workflowProjectId,
            workflow_id: workflowId,
            summary: "工作者汇报摘要，不复制完整会话记录。",
            sensitive_level: "internal",
            created_at: "2026-06-04T00:00:00Z",
          },
        ],
        status: "recorded",
        generated_by_role: "worker",
        actor_id: "codex-dev",
        risk_level: "low",
        sensitive_level: "internal",
        candidate_key: null,
        audit_refs: [],
        created_at: "2026-06-04T00:00:00Z",
        updated_at: "2026-06-04T00:00:00Z",
      },
      {
        observation_id: "obs:v1:offline:002",
        observation_key: "obs:v1:offline-candidate-created",
        schema_version: "memory_observation.v1",
        project_id: workflowProjectId,
        workflow_id: workflowId,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: workflowProjectId,
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-04T00:00:00Z",
        },
        observation_type: "project_director_confirmation",
        summary: "项目主管确认 observation 可生成候选。",
        source_refs: [
          {
            source_ref_id: "obs-source:offline:002",
            source_kind: "director_review",
            source_id: "director-review:offline:002",
            project_id: workflowProjectId,
            workflow_id: workflowId,
            summary: "项目主管确认摘要。",
            sensitive_level: "internal",
            created_at: "2026-06-04T00:00:02Z",
          },
        ],
        status: "candidate_created",
        generated_by_role: "project_director",
        actor_id: "project-director-offline",
        risk_level: "low",
        sensitive_level: "internal",
        candidate_key: "memcand:v1:from-observation",
        audit_refs: [],
        created_at: "2026-06-04T00:00:02Z",
        updated_at: "2026-06-04T00:00:03Z",
      },
    ],
    events: [
      {
        audit_ref_id: "audit:observation-candidate-created:offline",
        event_type: "observation_candidate_created",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        target_kind: "observation",
        target_id: "obs:v1:offline:002",
        before_status: "recorded",
        after_status: "candidate_created",
        reason: "项目主管确认生成候选。",
        created_at: "2026-06-04T00:00:03Z",
      },
    ],
    updated_at: "2026-06-04T00:00:03Z",
    warnings: [],
  };

  const emptyMemoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    revision: 0,
    candidates: [],
    events: [],
    updated_at: "2026-06-04T00:00:00Z",
  };

  const {
    formalMemoryStore,
    adoptedFormalMemoryStore,
    memoryLintStore,
    taskMemoryPacketPreview,
  } = candidateMemoryLifecycleFixtures({ workflowProjectId, workflowId });

  return {
    blackboardCandidateStore,
    confirmedMemoryCandidate,
    confirmedMemoryCandidateStore,
    adoptedMemoryCandidateStore,
    observationStore,
    emptyMemoryCandidateStore,
    formalMemoryStore,
    adoptedFormalMemoryStore,
    memoryLintStore,
    taskMemoryPacketPreview,
  };
}
