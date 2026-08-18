# Grok task package: M6D01 cross-project and organization contract freeze

You are the preferred implementation writer for the current Syn Harness leaf `M6D01-cross-project-and-organization-contract-freeze.md`. Work in `/home/synadmin/workspace/syn`. One writer only; do not spawn subagents. Do not commit, stage, reset, stash, clean, push, rebase, merge, tag, deploy, or release. Codex will independently review and commit.

## Read first

1. `AGENTS.md`
2. `docs/harness/leaves/M6D01-cross-project-and-organization-contract-freeze.md`
3. `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`, especially §§3–7
4. `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` §1
5. `docs/contracts/m5-project-summary-projection-v1.md`
6. Existing contract style: `docs/contracts/project-orchestration-v1.md`, `role-session-v1.md`, `handoff-v1.md`, `attention-decision-v1.md`
7. `docs/contracts/README.md`, `manifest.v1.json`, and `verify-syn-fnd-001.mjs`

## Exact write scope

You may create only:

- `docs/contracts/m6-cross-project-and-organization-v1.md`
- new JSON fixtures under `docs/contracts/fixtures/m6-org-001/`

Do not edit any existing contract, manifest, verifier, Harness file, source file, task file, or any other path. In particular, do not touch `src-tauri/src/*.rs`, the protected untracked `m6_*.rs`, or `gen/schemas/linux-schema.json`.

Codex has preliminarily established that `manifest.v1.json` is the frozen exact ten-contract M1 registry: its verifier hard-codes count/order/exports and later M3–M5 addenda are intentionally not registered there. Recheck this fact. If confirmed, leave the manifest byte-identical and state why in your return; do not break the accepted M1 verifier merely to list M6.

## Required contract content

Create a self-contained v1 contract with front matter and machine-readable fenced JSON where useful. It must say explicitly that M6D01 freezes contract/fixtures only and implements no service, repository, projection, UI, provider, or runtime behavior.

Freeze all of the following without weakening M1–M5:

1. Owners, imports, exports and required fields for at least `CrossProjectAdvisory`, `AdvisoryApplicationProjection`, `StableMember`, `TemporaryAgent`, `Availability`, `ConsultHandoff` (as M3 Handoff specialization/ref, not a replacement truth), `MultiViewConsultation`, `ChildRunRef`, and any named value/ref types required to make joins unambiguous.
2. ProjectSummary consumer gate: only `ProjectSummaryQueryPort`; exact consumer global RoleSession/scope/expiry/policy; project owner; summary id/schema version/version/watermark/hash; source refs. No direct project store/projection/root/raw transcript/secret read and no project writeback.
3. A non-degrading freshness state machine with `fresh`, `stale`, `missing`, `denied`, `degraded`. Define deterministic precedence and explicitly forbid denied/missing/degraded from being silently represented as stale/fresh or replaced with cached content.
4. CrossProjectAdvisory exact join: AdvisoryId, global RoleSession, ConsultHandoff, each consumed summary id + project id + schema version + version + watermark + hash + policy decision, generated_at, source links. Any missing or mismatched field is fail-closed/zero-write.
5. Adoption/application boundary: user adoption creates only a `DecisionRequest`; each project owner later executes an independent authoritative command/grant path and receipt. The application projection owns no project result and never changes advisory lifecycle. Define `applied/failed/rolled_back/unknown`, partial apply, compensation/rollback observation, and immutable history.
6. Stable vs temporary member strict typing, lifecycle, scope/role assignments, same-name collision, explicit human promotion with `promoted_from`, retention/deactivation while preserving refs. Provider/model/thread/process is never member identity.
7. Capability/permission are read-only refs with `source + revision + observed_at`; directory is never authority. Availability has source/observed_at/TTL; stale becomes unknown and cannot authorize. Contact produces a receipt and grants no capability.
8. TemporaryAgent full M5 immutable execution envelope, exactly: `project_id`, `orchestration_id`, `workflow_run_id`, `work_item_id`, `node_id`, `dispatch_id`, `attempt_id`, `grant_id`, `worker_role_session_id`, authoritative receipt, trusted actor, hashes. No report self-claim, missing-field compatibility, or runtime-trace derivation. ChildRunRef is reference only and creates no member/organization hierarchy.
9. Multi-view independence: same minimal sourced question packet; separate RoleSession/Workcell/context packet; no reading peer conclusions before submit; consensus/disagreement/evidence side by side; explicit budget/timeout/result states; only a user-pending decision, no command/grant/fact.
10. Migration matrix: legacy single-project review stays adapter/history; Agent Center session never auto-promotes; TemporaryAgent rebuilds from immutable refs without report body; summary version/watermark change marks advisory stale without overwriting history; unmappable legacy quarantines. Include rollback limits.
11. Formal fail-closed rules and forbidden fields/material: no raw files, raw summaries, transcript, prompt/provider response, secrets, stdout/stderr/tool output in contract payloads.

## Required fixtures

Create one or more versioned JSON fixture files under `docs/contracts/fixtures/m6-org-001/`. They must be deterministic, include explicit `polarity`, `rule_id`, input, expected code, and `expected_mutated_targets`. Include positive cases plus at least these negative cases:

- stale summary cannot be consumed as fresh;
- denied scope cannot degrade to stale/missing or reuse cache;
- foreign project owner / owner mismatch;
- missing watermark;
- TemporaryAgent presented as StableMember;
- stale availability used as permission;
- TemporaryAgent execution envelope with at least one required field missing.

Also cover exact advisory joins, adoption creating only DecisionRequest, partial application projection, contact-no-grant, multi-view pre-submit isolation, and unmappable legacy quarantine. Negative cases must expect rejection/quarantine and zero mutation, never tolerance.

## Validation and return

Run only read-only/offline checks:

- JSON parse every new fixture;
- a small deterministic `node -e` assertion covering unique fixture IDs, required fixture keys, all required negative categories, and negative `expected_mutated_targets=[]`;
- `node docs/contracts/verify-syn-fnd-001.mjs` to prove the frozen M1 baseline still passes unchanged;
- `git diff --check`;
- `git status --short` and confirm only the two allowed new contract/fixture areas changed by you (pre-existing Harness runtime/WIP may remain, do not touch it).

Return: files written, contract decisions, fixture counts by polarity, exact commands/exits, manifest decision, and any blocker. Do not claim service/runtime/UI implementation.
