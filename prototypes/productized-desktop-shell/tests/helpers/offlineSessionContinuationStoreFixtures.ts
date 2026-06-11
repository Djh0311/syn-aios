import type { SessionContinuationPreview, SessionContinuationStoreV1 } from "../../src/lib/types";

interface ControlledSessionContinuationLevelAStoreFixtureInput {
  preview: SessionContinuationPreview;
  projectRoot: string;
  sessionThreadId: string;
}

interface H2DuplicateSessionContinuationStoreFixtureInput {
  baseStore: SessionContinuationStoreV1;
  confirmedPreview: SessionContinuationPreview;
  projectRoot: string;
  sessionThreadId: string;
}

export function controlledSessionContinuationLevelAStoreFixture(
  input: ControlledSessionContinuationLevelAStoreFixtureInput,
): {
  continuationId: string;
  attemptId: string;
  store: SessionContinuationStoreV1;
} {
  const { preview, projectRoot, sessionThreadId } = input;
  const continuationId = "session-continuation:v1:offline";
  const attemptId = "session-continuation-attempt:offline";
  const store: SessionContinuationStoreV1 = {
    schema_version: "session_continuation_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: "/offline-fixture/workflow-state.v0.json",
      sidecar_path: "/offline-fixture/session-continuations.v1.json",
      project_roots: [projectRoot],
    },
    revision: 2,
    last_write_id: "write-offline-stub",
    generated_by: "control_core",
    created_at: "2026-06-06T00:00:00Z",
    updated_at: "2026-06-06T00:01:00Z",
    continuations: [
      {
        record_version: 1,
        continuation_id: continuationId,
        preview_id: preview.preview_id,
        adapter_id: "codex-local",
        operation_id: "resume",
        project_id: preview.project_id ?? projectRoot,
        project_root: preview.project_root ?? projectRoot,
        workflow_id: preview.workflow_id ?? "workflow:offline-fixture-projects-codex-workbench:default",
        node_id: preview.node_id ?? "node:offline-dev",
        session_id: preview.target_session_id ?? sessionThreadId,
        target_cwd: preview.target_cwd ?? projectRoot,
        allowed_write_roots: preview.allowed_write_roots_summary,
        sandbox: preview.sandbox_summary,
        prompt_source_kind: preview.prompt_source_kind,
        prompt_summary: preview.prompt_summary,
        command_preview: "Level A preview only: codex exec resume <session>",
        readback_strategy: "required",
        status: "succeeded_stub",
        execution_level: "level_a_stub_only",
        runner_kind: "stub",
        user_confirmation_state: "confirmed",
        guard_status: "needs_user_confirmation",
        requested_by: "workbench_e4_preview",
        confirmed_by: "user",
        confirmation_reason: "离线 Level A stub 验收",
        created_at: "2026-06-06T00:00:00Z",
        updated_at: "2026-06-06T00:01:00Z",
        audit_refs: ["audit:session-continuation-confirmed:offline", "audit:session-continuation-stub-completed:offline"],
        warnings: ["level_a_stub_only", "real_codex_executed_false", "writes_codex_home_false"],
      },
    ],
    attempts: [
      {
        attempt_version: 1,
        attempt_id: attemptId,
        continuation_id: continuationId,
        runner_kind: "stub",
        execution_level: "level_a_stub_only",
        status: "succeeded_stub",
        started_at: "2026-06-06T00:01:00Z",
        finished_at: "2026-06-06T00:01:00Z",
        timeout_ms: 30000,
        command_preview: "Level A preview only: codex exec resume <session>",
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: {
          status: "readback_unavailable",
          source_kind: "stub_no_transcript_read",
          result_count: null,
          unavailable_reason: "Level A stub 不读取真实 transcript；unavailable 不等于空读回结果。",
          warnings: ["readback_unavailable_is_not_zero_results", "no_real_transcript_read_in_level_a"],
        },
        failure_reason: null,
        audit_refs: ["audit:session-continuation-stub-started:offline", "audit:session-continuation-stub-completed:offline"],
        warnings: [
          "stub_runner_only",
          "prompt_not_sent",
          "real_codex_execution_not_authorized",
          "codex_home_not_touched",
          "readback_unavailable_is_not_zero_results",
        ],
      },
    ],
    audit_events: [
      {
        event_version: 1,
        event_id: "audit:session-continuation-confirmed:offline",
        event_type: "session_continuation_preview_confirmed",
        continuation_id: continuationId,
        attempt_id: null,
        preview_id: preview.preview_id,
        actor_role: "user",
        before_status: null,
        after_status: "preview_confirmed",
        store_revision: 1,
        reason: "用户确认 Level A stub",
        created_at: "2026-06-06T00:00:00Z",
        warnings: ["level_a_stub_only"],
      },
    ],
    warnings: [],
  };

  return { continuationId, attemptId, store };
}

export function h2DuplicateSessionContinuationStoreFixture(
  input: H2DuplicateSessionContinuationStoreFixtureInput,
): SessionContinuationStoreV1 {
  const { baseStore, confirmedPreview, projectRoot, sessionThreadId } = input;
  return {
    ...baseStore,
    continuations: [
      {
        record_version: 1,
        continuation_id: "session-continuation:v1:h2-8-duplicate",
        preview_id: confirmedPreview.preview_id,
        adapter_id: "codex-local",
        operation_id: "resume",
        project_id: confirmedPreview.project_id ?? projectRoot,
        project_root: confirmedPreview.project_root ?? projectRoot,
        workflow_id: confirmedPreview.workflow_id ?? "workflow:offline",
        node_id: confirmedPreview.node_id ?? "node:offline",
        session_id: confirmedPreview.target_session_id ?? sessionThreadId,
        target_cwd: confirmedPreview.target_cwd ?? projectRoot,
        allowed_write_roots: confirmedPreview.allowed_write_roots_summary,
        sandbox: confirmedPreview.sandbox_summary,
        prompt_source_kind: confirmedPreview.prompt_source_kind,
        prompt_summary: confirmedPreview.prompt_summary,
        command_preview: "Level B preview only: codex exec resume <session>",
        readback_strategy: "required",
        status: "queued",
        execution_level: "level_b_real_user_approved",
        runner_kind: "codex_local_real",
        user_confirmation_state: "confirmed",
        guard_status: "needs_user_confirmation",
        requested_by: "h2_8_duplicate_fixture",
        confirmed_by: "user",
        confirmation_reason: "duplicate guard fixture",
        created_at: "2026-06-08T00:00:00Z",
        updated_at: "2026-06-08T00:01:00Z",
        audit_refs: ["audit:h2-8-duplicate"],
        warnings: ["duplicate_guard_fixture_only"],
      },
    ],
    attempts: [
      {
        attempt_version: 1,
        attempt_id: "session-continuation-attempt:h2-8-duplicate",
        continuation_id: "session-continuation:v1:h2-8-duplicate",
        runner_kind: "codex_local_real",
        execution_level: "level_b_real_user_approved",
        status: "queued",
        started_at: "2026-06-08T00:01:00Z",
        finished_at: null,
        timeout_ms: 120000,
        command_preview: "Level B queued preview: codex exec resume <session>",
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: {
          status: "readback_unavailable",
          source_kind: "queued_no_readback",
          result_count: null,
          unavailable_reason: "Queued attempt has no readback; unavailable is not zero results.",
          warnings: ["readback_unavailable_is_not_zero_results"],
        },
        failure_reason: null,
        audit_refs: ["audit:h2-8-duplicate"],
        warnings: ["duplicate_guard_fixture_only"],
      },
    ],
  };
}
