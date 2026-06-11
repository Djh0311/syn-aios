import type { MaturePatternPreviewOutput, MemoryPatternStoreV1 } from "../../src/lib/types";

export function memoryPatternFixtures(): {
  memoryPatternStore: MemoryPatternStoreV1;
  maturePatternPreview: MaturePatternPreviewOutput;
} {
  const maturePatternScope = {
    scope_id: "scope:global:mature-pattern:memory-center",
    scope_type: "global" as const,
    project_id: null,
    workflow_id: null,
    session_id: null,
    role_ids: [],
    document_refs: [],
    permission_policy_ref: "memory_policy:global:user_confirmed",
    model_export_policy: "local_only" as const,
    valid_from: "2026-06-05T00:00:10Z",
  };
  const maturePatternSourceRef = {
    source_ref_id: "source:mature-pattern:v1:maintenance",
    source_type: "evidence" as const,
    source_id: "memory-maintenance-report:v1:offline",
    source_path: "/offline-fixture/evidence/m11.md",
    source_title: "M11 mature pattern signal",
    anchor: "重复控制核心边界",
    captured_at: "2026-06-05T00:00:10Z",
    authority_level: "evidence" as const,
    sensitive_level: "project" as const,
  };
  const memoryPatternStore: MemoryPatternStoreV1 = {
    store_version: "memory_patterns.v1",
    project_id: null,
    workflow_id: null,
    revision: 12,
    mature_pattern_candidates: [
      {
        candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
        pattern_kind: "repeated_review_boundary",
        scope: maturePatternScope,
        title: "跨项目重复边界：控制核心写入必须走确认",
        claim: "跨项目重复出现控制核心写入边界，成熟模式候选需要用户确认后才能成为正式记忆。",
        body: "该候选来自维护 signal、正式记忆和观察来源的重复主题；候选未确认，不会进入任务包。",
        source_refs: [maturePatternSourceRef],
        member_refs: [
          {
            member_ref_id: "cluster-member:v1:formal-memory",
            member_kind: "formal_memory",
            member_id: "mem:formal:offline:included",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "接口验收必须保留控制核心边界。",
            source_refs: [maturePatternSourceRef],
          },
          {
            member_ref_id: "cluster-member:v1:observation",
            member_kind: "observation",
            member_id: "observation:v1:memory-center",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "观察来源说明候选不是正式记忆。",
            source_refs: [maturePatternSourceRef],
          },
        ],
        signal_refs: ["mature_pattern_signal:v1:maintenance"],
        status: "candidate",
        requires_user_confirmation: true,
        review_summary: "秘书或全局主管只能汇总；用户确认前不写正式记忆。",
        created_at: "2026-06-05T00:00:10Z",
        updated_at: "2026-06-05T00:00:10Z",
        warnings: ["mature_pattern_candidate_requires_user_confirmation"],
      },
    ],
    cluster_reports: [
      {
        report_id: "memory-cluster-report:v1:control-core-boundary",
        report_kind: "cross_project_theme",
        scope_type: "global",
        title: "跨项目主题报告：控制核心边界",
        project_ids: ["project:offline-fixture-projects-codex-workbench", "project:offline-fixture-projects-other"],
        member_refs: [
          {
            member_ref_id: "cluster-member:v1:formal-memory",
            member_kind: "formal_memory",
            member_id: "mem:formal:offline:included",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "接口验收必须保留控制核心边界。",
            source_refs: [maturePatternSourceRef],
          },
        ],
        source_refs: [maturePatternSourceRef],
        status: "derived_report",
        staleness: "fresh",
        display_text: "跨项目主题报告只解释重复主题和来源下钻，不是正式事实。",
        created_at: "2026-06-05T00:00:10Z",
        warnings: ["memory_cluster_report_not_formal_memory"],
      },
    ],
    audit_events: [],
    updated_at: "2026-06-05T00:00:10Z",
    warnings: ["memory_pattern_store_m12_minimal_sidecar"],
  };

  const maturePatternPreview: MaturePatternPreviewOutput = {
    store_revision: memoryPatternStore.revision,
    mature_pattern_candidates: memoryPatternStore.mature_pattern_candidates,
    cluster_reports: memoryPatternStore.cluster_reports,
    acceptance_summary: {
      summary_id: "memory-acceptance-summary:v1:m12",
      scope_label: "M1-M12 memory layer",
      gate_count: 4,
      passed_count: 3,
      blocked_count: 0,
      deferred_count: 1,
      gates: [
        {
          gate_id: "gate:m1-formal-store",
          label: "M1 formal memory store",
          status: "passed",
          evidence: "formal records include source, version and audit refs",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m4-task-packet",
          label: "M4 task packet recall",
          status: "passed",
          evidence: "活跃正式记忆可以进入入选列表",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m11-maintenance",
          label: "M11 maintenance finding boundary",
          status: "passed",
          evidence: "maintenance report creates findings only",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m13-final-freeze",
          label: "M13 final authority freeze",
          status: "deferred",
          evidence: "outside M12 scope",
          blocking_reason: null,
        },
      ],
      display_text: "M1-M12 门禁摘要：通过 3 / 阻断 0 / 后置 1。",
      warnings: ["m12_is_not_m13_final_acceptance"],
      created_at: "2026-06-05T00:00:10Z",
    },
    summary: {
      sidecar_name: "memory-patterns.v1.json",
      revision: memoryPatternStore.revision,
      mature_pattern_candidate_count: 1,
      cluster_report_count: 1,
      confirmed_pattern_count: 0,
      display_text: "成熟模式候选 1 / 跨项目主题报告 1 / 待用户确认 1。",
      warnings: [],
    },
    warnings: ["memory_cluster_report_not_formal_memory"],
  };

  return {
    memoryPatternStore,
    maturePatternPreview,
  };
}
