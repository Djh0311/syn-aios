# SYN-M2 named reference-slice reconciliation

recorded-at: `2026-08-05T18:56:12+08:00`
authority: `docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md`, `docs/contracts/syn-dat-001-mechanism-contract-v1.md`, and frozen `docs/contracts/syn-dat-001-migration-checklist-v1.md`
candidate-state: `M2 READY_FOR_MAINLINE_CLOSEOUT candidate for the named scratch reference slice only; mainline acceptance pending; not M2 COMPLETE; M3 NOT_ACTIVE`

This is a reconciliation artifact, not an edit to the frozen checklist or a
claim of live migration. It identifies exactly one M2 truth:
`workflow-state-sidecar.repository.m2.v1` backed by the concrete SQLite
repository. It records schema version, repository port version and caller mode
in explicit M2/R4 receipts/events/audit evidence. Ordinary UI and legacy
internal callers without stable command id, idempotency key and expected
revision remain `GUARDED_LEGACY / NOT_MIGRATED`; they are not silently promoted
by this artifact.

## Frozen checklist reconciliation

| Frozen checklist checkbox/domain | State | Concrete evidence boundary |
| --- | --- | --- |
| Workflow reference slice: receipts, events, audit, snapshots, checkpoints, UoW, aggregate, `update_work_item_state` | `MIGRATED` for explicit M2/R4 scratch calls | One `BEGIN IMMEDIATE` concrete SQLite implementation through `workflow-state-sidecar.repository.m2.v1`; command identity and revision CAS focused tests. |
| Workflow reference slice: `outbox_items` and result command | `MIGRATED` only through the explicitly armed R4 acceptance effect | The actual `update_work_item_state` owning command/receipt/revision/effect/correlation is reused in one concrete SQLite UoW; an independent normalized result command completes only that armed scratch effect. It is fixture-only, has no provider and no normal-product side effect. |
| Workflow reference slice: projector/current snapshot/checkpoint | `MIGRATED` for named slice | Snapshot hash is `sha256(canonical snapshot content)`; the JSON written to disk is reread, parsed and canonically hashed before checkpoint advancement. Semantic-equivalent JSON keeps its hash; disk tamper rejects/degrades without dropping source fact or rewriting DB from JSON. |
| Workflow reference slice: temp DB, atomicity, single writer, receipt persistence | `MIGRATED` for named scratch slice | Fresh temp DB, transaction/replay/concurrent-result tests, R4 isolated App evidence. |
| Schema ports and frozen tables | `MIGRATED` for additive scratch schema | `foreign_keys=ON`; exact frozen FK targets/actions (`commands`, `command_receipts`, `correlation_chains`, `projectors`, all `RESTRICT`); required PK/unique/index order; and migration-row-plus-schema-drift refusal are exercised by M2 schema tests. |
| Receipt ports | `MIGRATED` for explicit M2/R4 caller | Command receipt includes repository port/schema/caller mode provenance; duplicate replays same receipt and divergent command identity fails closed. |
| Grant validation | `MIGRATED` as admission guard only | Fresh grant/source/binding/revocation validation. Valid grant-bearing report returns typed `NOT_MIGRATED/HOLD` and writes nothing; invalid variants also write nothing. |
| Conversation / RoleSession, Memory, Knowledge | `NOT_MIGRATED / HOLD` | Future M3/M7/M8 domains; not implemented or credited here. |
| DAT-001B live manifest and DAT-007 live cutover | `LIVE_MANIFEST_READ_ONLY` / `NOT_MIGRATED / NO_CUTOVER` | No live Workbench data was read or written by this reconciliation; no live migration, promote, rollback or export occurred. |
| Claim review, result-user decision, source-owner apply-result | `NOT_MIGRATED / HOLD` | Separate frozen-v1 owners and commands; not conflated with grant validation. |
| Station 3b and Code Map advisory | `DEFERRED_TO_M3` / non-blocking advisory | Neither is implemented or used to enlarge this M2 slice. |

## Four narrow correction outcomes

1. **Grant-bearing report boundary.** A grant-bearing payload is validated but
   not admitted as execution/review/decision truth. The typed
   `NOT_MIGRATED/HOLD` result leaves claim-like, receipt, event, audit,
   `updated_at`, attempt, dispatch, work item, workflow, review, decision and
   director-chain facts unchanged. In addition, `manual/offline` reports with
   any dispatch, attempt or grant join are rejected at both the public and
   lower writer boundary before capture/backup/audit/DB/JSON work. Legacy
   no-grant behavior remains `GUARDED_LEGACY`.
2. **Canonical snapshot/parity.** Content changes produce new hashes,
   canonical-equivalent JSON produces the same hash, rebuild produces the same
   hash, and projection-hash mismatch stops checkpoint advancement while
   preserving authoritative facts.
3. **DAT-004/schema runtime.** Normal DB-primary JSON projection is internal
   and rebuildable (`outbox_item=None`). The only external-effect model is the
   explicitly armed R4 acceptance effect attached to the actual owning
   `update_work_item_state` command, receipt, revision and correlation:
   receipt
   `COMMITTED → EXTERNAL_PENDING → EXTERNAL_RESULT`; outbox
   `DECLARED → AVAILABLE → LEASED → DELIVERED → RESULT_RECEIVED`; failures use
   `RETRY_WAIT`, `POISON` or `CANCELLED`; lease is 300 seconds, retry maximum is
   three, and retry timing is exponential with jitter. It has a distinct,
   normalized result command/idempotency/admission/receipt/event/audit path
   bound to the owning effect; no second sample or provider is used.
4. **Single port truth.** The generic `m2_ports`, `m2_outbox`, `m2_projector`,
   `m2_legacy_adapter` and `m2_domain_cutover` candidates are private,
   non-authoritative and receive no completion credit. The concrete versioned
   SQLite port above is the sole truth for this named slice.

## Verification index

- `cargo check --lib --quiet` → exit `0`.
- `cargo test --lib m2_reference_slice --quiet` → exit `0`, `10 passed`.
- `cargo test --lib m2_reference_slice_r4_fake_external_adapter --quiet` → exit `0`, `2 passed`; covers same-owner effect/result binding, lease/expiry/retry/replay and normalized result-envelope rejection cases.
- `cargo test --lib m2a_execution_report_ingress_tests --quiet` → exit `0`, `2 passed`; covers public/lower report kind-confusion and divergent replays before persistence/capture.
- `cargo test --lib workbench_sqlite_schema_m2::tests --quiet` → exit `0`, `7 passed`; includes exact FK target/action and same-named wrong-index drift refusal.
- `cargo test --lib workbench_sqlite_storage_mode::tests::m2_reference_slice_persisted_projection_tamper_is_fail_closed_without_checkpoint_or_db_rewrite --quiet` → exit `0`, `1 passed`.
- `cargo test --lib --no-fail-fast --quiet` in the host-permission environment → exit `0`, `1385 passed / 0 failed / 45 ignored` (1430 total).
- `node scripts/run-r4-isolated-app-preflight.mjs --m2-reference-slice` → exit `0`, seven bounded real-Tauri App scenarios (normal S1–S6 plus the armed same-slice effect/result/recovery scenario); current raw receipt SHA-256 `acc05a13c791717b83d90ddd714717d8f4fc78121b46eb07579737ae999f876a` at `/private/var/folders/nj/y6s1fvl936xgfwg20w08sk6r0000gn/T/syn-r4-acceptance-D4wpEO/m2-reference-slice-suite-receipt.json`.

The scratch rollback/export and precise migration matrix remain documented in
`syn-m2a-r2-closeout-correction-2026-08-05.md`. These tests and receipts do
not alter the frozen contract, live Harness, live Workbench, provider state,
Git refs, or M3 state.

## Third narrow correction — current candidate evidence

The current result supersedes only the former A/B/C repair hold; it does not
promote a live migration, provider flow, OFF_MACHINE verification, Harness
closeout, or M2 `COMPLETE` claim.

1. **A — declaration ledger and correlation.** In the same owning
   `BEGIN IMMEDIATE` transaction as the armed R4 `update_work_item_state`
   command/receipt/event/audit/outbox row, the concrete SQLite port now writes
   exactly one `OutboxItemDeclared` event and one
   `SCRUBBED_OUTBOX_RECORD` audit. Both retain the owning correlation. The
   independent result audit retains its result command id and that same owner
   correlation. Exact replay creates no additional owner facts; a mismatched
   binding fails closed with zero new ledger rows.
2. **B — complete aggregate snapshot and checkpoint.** The sole concrete port
   rereads the authoritative SQLite workflow, sorted nodes and sorted work
   items after the domain write, derives
   `workflow_state:{project}:{workflow}` and its canonical hash, then compares
   the full persisted JSON aggregate with the same DTO/hash. Startup repairs a
   missing/stale checkpoint only when the stored source watermark/event/receipt
   agrees; a disk mismatch fails closed and never writes JSON back into SQLite.
3. **C — port/schema/consumer trace.**
   `workflow-state-sidecar.repository.m2.v1` is the only named slice port.
   Its typed consumer registry records stable caller id, version and migration
   state for explicit M2, R4 acceptance, startup recovery and guarded legacy
   callsites; omitted registration fails a machine test. The frozen schema
   validator rejects both wrong FK action and wrong FK target.

Focused current reruns: `m2_r4_armed_declaration_records_frozen_ledger_and_rejects_binding_drift`,
`m2_reference_slice_full_aggregate_snapshot_rebuilds_checkpoint_after_crash_window`,
`m2_reference_slice_persisted_projection_tamper_is_fail_closed_without_checkpoint_or_db_rewrite`,
`m2_workflow_state_sidecar_consumer_registry_is_complete_and_rejects_omission`,
and `m2_schema_introspection_rejects_wrong_fk_action_and_same_named_index_shape` each exited `0`
with `1 passed / 0 failed`. The raw R4 receipt binds current source hashes,
App executable SHA-256 `6cf203361a5243e162c2281a43b6f038fab2e506c2706bf19f46a6bc2b3c0d60`,
HEAD `2a7229b…`, tree `81528b95…`, and a stable pre/post evidence run.
