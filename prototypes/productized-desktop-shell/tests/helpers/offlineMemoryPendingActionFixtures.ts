import type {
  FormalMemoryLifecyclePreview,
  FormalMemoryStoreV1,
  MaturePatternCandidate,
  MemoryRelationCandidate,
  PendingAction,
} from "../../src/lib/types";

const workflowProjectId = "project:offline-fixture-projects-codex-workbench";
const workflowId = "workflow:offline-fixture-projects-codex-workbench:default";

type FormalMemoryRecord = FormalMemoryStoreV1["records"][number];

type MemoryPendingActionFixtureInput = {
  projectRoot: string;
  formalMemoryStoreRevision: number;
  memoryCandidateStoreRevision: number;
  memoryLintStoreRevision: number;
  memoryEntityRelationStoreRevision: number;
  memoryPatternStoreRevision: number;
  includedMemory: {
    memory_id: string;
    record: FormalMemoryRecord;
  };
  relationCandidate: MemoryRelationCandidate;
  maturePatternCandidate: MaturePatternCandidate;
};

export function memoryPendingActionFixtures(input: MemoryPendingActionFixtureInput) {
  const formalMemoryLifecyclePreview: FormalMemoryLifecyclePreview = {
    preview_id: "formal-memory-lifecycle-preview:test",
    operation_kind: "deprecate",
    store_revision: input.formalMemoryStoreRevision,
    target_memory_ids: [input.includedMemory.memory_id],
    impact: {
      affected_memory_ids: [input.includedMemory.memory_id],
      created_memory_ids: [],
      status_changes: [
        {
          memory_id: input.includedMemory.memory_id,
          before_status: "memory_active",
          after_status: "memory_deprecated",
        },
      ],
      created_memory_count: 0,
      new_version_count: 1,
      task_packet_eligibility_change: "非活跃记忆默认不进任务包入选列表。",
      source_ref_count: input.includedMemory.record.source_refs.length,
      display_text: "影响 1 条正式记忆，新增 1 个版本；非活跃记忆默认不进任务包入选列表。",
      warnings: ["formal_memory_lifecycle_versions_and_audit_recorded"],
    },
    required_approval: {
      required: true,
      approval_kind: "project_director_or_user_confirmation",
      required_actor_role: "project_director_or_user",
      reason: "项目内正式记忆 lifecycle 需要项目主管或用户确认。",
    },
    before_records: [input.includedMemory.record],
    proposed_records: [
      {
        ...input.includedMemory.record,
        record_version: input.includedMemory.record.record_version + 1,
        status: "memory_deprecated",
      },
    ],
    display_text: "废弃预览：影响 1 条 / 新版本 1 个",
    warnings: ["formal_memory_lifecycle_versions_and_audit_recorded"],
  };

  const lifecycleAction: PendingAction = {
    kind: "record-formal-memory-lifecycle-operation",
    label: "正式记忆 废弃",
    path: input.projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；非活跃记忆默认不进任务包。",
    formalMemoryLifecycle: {
      project_root: input.projectRoot,
      project_id: workflowProjectId,
      workflow_id: workflowId,
      operation_kind: "deprecate",
      memory_id: input.includedMemory.memory_id,
      memory_ids: [],
      revise: null,
      merge: null,
      split: null,
      scope_change: null,
      actor_id: "project-director-ui",
      actor_role: "project_director",
      reason: "废弃正式记忆测试。",
      expected_store_revision: input.formalMemoryStoreRevision,
      expected_record_versions: {
        [input.includedMemory.memory_id]: input.includedMemory.record.record_version,
      },
      confirmed_by: "project-director-ui",
      confirmation_summary: "已查看影响面。",
    },
    formalMemoryLifecyclePreview,
  };

  const relationAction: PendingAction = {
    kind: "record-memory-relation-candidate-decision",
    label: "确认关系候选",
    path: input.projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-entity-relations.v1.json；已确认关系只用于解释召回原因，不改变任务包入选列表。",
    memoryRelationCandidateDecision: {
      project_root: input.projectRoot,
      relation_candidate_id: "relation-candidate:v1:contract",
      decision: "confirm_relation",
      actor_id: "project-director-memory-center",
      actor_role: "project_director",
      confirmed_by: "project_director",
      reason: "项目主管确认关系候选。",
      expected_store_revision: input.memoryEntityRelationStoreRevision,
    },
    memoryRelationCandidate: input.relationCandidate,
  };

  const maintenanceAction: PendingAction = {
    kind: "run-memory-maintenance",
    label: "运行记忆维护任务",
    path: input.projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-lint.v1.json 的维护运行 / 发现项 / 报告；不会自动修改正式记忆、候选、观察、实体关系或工作流状态。",
    memoryMaintenanceRun: {
      project_root: input.projectRoot,
      project_id: workflowProjectId,
      workflow_id: workflowId,
      actor_id: "project-director-memory-center",
      actor_role: "project_director",
      lint_intent: "maintenance_run",
      candidate_key: null,
      task_id: "memory-maintenance:m11",
      revoked_source_ids: [],
      expected_formal_store_revision: input.formalMemoryStoreRevision,
      expected_candidate_store_revision: input.memoryCandidateStoreRevision,
      expected_lint_store_revision: input.memoryLintStoreRevision,
      dry_run: false,
    },
  };

  const maturePatternAction: PendingAction = {
    kind: "record-mature-pattern-decision",
    label: "用户确认成熟模式候选",
    path: input.projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "用户确认后写 memory-patterns.v1.json，并通过正式记忆受控路径写 formal-memories.v1.json；候选和报告未确认不进入任务包。",
    maturePatternDecision: {
      project_root: input.projectRoot,
      candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
      decision: "confirm_as_formal_memory",
      actor_id: "user-memory-center",
      actor_role: "user",
      confirmed_by: "user",
      reason: "用户确认成熟模式候选。",
      expected_pattern_store_revision: input.memoryPatternStoreRevision,
      expected_formal_store_revision: input.formalMemoryStoreRevision,
    },
    maturePatternCandidate: input.maturePatternCandidate,
  };

  const quarantineMaturePatternAction: PendingAction = {
    kind: "record-mature-pattern-decision",
    label: "隔离成熟模式候选",
    path: input.projectRoot,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-patterns.v1.json 的候选决定；不写正式记忆，不改来源材料，不影响任务包入选列表。",
    maturePatternDecision: {
      project_root: input.projectRoot,
      candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
      decision: "quarantine",
      actor_id: "user-memory-center",
      actor_role: "user",
      confirmed_by: null,
      reason: "用户隔离成熟模式候选。",
      expected_pattern_store_revision: input.memoryPatternStoreRevision,
      expected_formal_store_revision: null,
    },
    maturePatternCandidate: input.maturePatternCandidate,
  };

  return {
    formalMemoryLifecyclePreview,
    lifecycleAction,
    relationAction,
    maintenanceAction,
    maturePatternAction,
    quarantineMaturePatternAction,
  };
}
