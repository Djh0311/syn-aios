# Station 3a Control-Core Bridge Evidence v1

Date: 2026-07-12

Status: `PASS__RISK_CLEANED__READY_FOR_3B`

The v3/v4/v5/v6 runs remain historical safety and failure evidence; none is completion evidence. The fresh v7 UI run is the only Station 3a completion run. It proved the role-separated proposal/task-package path, one fresh worker session, one real execution attempt, one accepted inspection, advisory `finalize(pass)`, and `report_user`, with independent byte verification of the output file. Station 3a is closed and the repository may move to the separate Station 3b approval gate; Station 3b has not started.

## Review Repair Closure

- The 2026-07-12 implementation review temporarily set this slice to `NEEDS_REWORK`: `dispatch_worker` incorrectly required `codex_exec_resume` in the task-tool allowlist, and the idempotency key included mutable workflow revision.
- Both blockers are repaired and covered by controller regressions. This status was restored only after the targeted, full-library, typecheck, offline-interaction, offline-compile, formatting, and diff-hygiene checks recorded below.
- A stale pre-repair `.app` was opened once during the first UI attempt. Its run is retained only as a failed historical fact and is not used as v3 evidence.
- The debug `.app` used for the historical v3 run was rebuilt at 2026-07-12 16:14. The completion binary used by v7 is identified separately below.

## Delivered Boundary

- The supervisor model emits one strict `SupervisorActionProposalV1` JSON object per step.
- The opening contract lists complete strict JSON shapes for all seven action kinds. Its dispatch example uses nested `target.node_id` and `target.work_item_id`; legacy top-level fields and `target.worker_id` remain invalid and the parser was not relaxed.
- The prepared worker objective deterministically contains only the proposal's worker acceptance criteria in original order; Syn-control and supervisor-judgment criteria remain in their own persisted fields and never enter the worker prompt.
- Syn binds every parsed action to the active supervisor run, project, workflow, authorization snapshot, workflow revision, task-package fingerprint, and derived permission scopes before adapter execution.
- MCP exposes only `read_worker_report`, `wait_for_worker`, and `read_key_file`. Worker dispatch, follow-up, advisory final mark, and report execution now run only through the host control-core bridge.
- The host runs one supervisor `codex exec` step at a time, consumes `last_message`, records the authoritative result, and starts a next step only when the result is non-terminal and within budget.
- Temporary `CODEX_HOME`, `0700` directory, `0600` MCP config, `auth.json` symlink, orphan cleanup, and token-refresh detection are retained. The temporary `approval_policy = "never"` configuration was removed.

## Safety Assertions

- Unknown/prohibited proposal fields, malformed JSON, trailing natural language, invalid kinds, and malformed nested targets produce `protocol_invalid` with no action reservation.
- A parse failure records a `system_protocol_invalid` action-ledger diagnostic and a supervisor-session `protocol_invalid` audit diagnostic before any adapter call or worker launch. The next supervisor step receives the concrete parse error and the matching JSON example for its original action kind exactly once.
- A second parse failure records a second diagnostic and ends the supervisor session as `waiting_user`. It says the current invalid action did not execute; it says `本单未执行` only when no earlier worker exists. When a worker exists it reports that prior dispatch and its latest authoritative state instead. It does not create `user_cancelled` or attempt a third automatic correction.
- The model cannot supply paths, write roots, authorization identifiers, approval flags, sandbox settings, argv, credentials, action IDs, or ledger revisions.
- Dispatch identifies exactly one current authorized prepared dispatch before the adapter can run; the adapter derives write roots from active authorization, never model input. The task tool allowlist does not need to name the runner's internal `codex_exec_resume` mechanism: permission comes from active authorization, the prepared dispatch, and declared adapter capability.
- Authorization/run binding drift, workflow revision drift, quotas, adapter failures, and supervisor process failures are recorded as honest non-user statuses. No path writes `user_cancelled` without a separate user-cancel event.
- `finalize: pass` requires prior authoritative inspected worker evidence. `request_user_decision` only creates `waiting_user` in the action ledger.
- Idempotency is a SHA-256 identity of the stable run, project, workflow, authorization, action kind, and action target/effect parameters. It deliberately excludes the mutable workflow revision.
- A replay that finds an unfinished reservation is recovered as `waiting_user` without calling the adapter again, because the external worker may already have launched before the prior process crashed.
- `codex exec` exit code zero only records process completion. A worker `blocked` report becomes `waiting_user`; malformed worker output becomes `report_invalid` then `waiting_user`; only a complete structured report with non-placeholder evidence can enter downstream acceptance.

## Test Scope

- Fake supervisor JSON plus fake adapter covers `dispatch_worker -> inspect_worker -> finalize(pass) -> report_user` with one authoritative action record and audit reference per step.
- A malformed top-level `node_id`/`work_item_id` dispatch is diagnosed with zero adapter calls and zero workers; a subsequent valid nested-target dispatch launches exactly one worker, remains idempotent on replay, and can proceed to `inspect_worker`.
- A correct dispatch followed by an invalid `inspect_worker` and a corrected `inspect_worker` still starts exactly one worker. A two-error session after dispatch keeps the prior-worker truth and never displays `本单未执行`.
- A worker report with empty evidence or the generic `worker 未附证据` placeholder is `report_invalid`; an explicit `blocked` report is `waiting_user`.
- A regression passes a user literal containing `station3a-control-core-proof-v4.txt` and its exact content through the persisted proposal snapshot and the first materialized task-package goal without rewriting either substring.
- The controller fixture uses the fixed-test authorization tool list `read_file`, `write_file`, and `apply_patch`; dispatch still succeeds because internal runner execution is not a task-tool capability.
- The crash-window regression simulates an adapter that launches one worker and advances workflow revision, then stops before `complete_action`. Replay recovers the original reservation as `waiting_user` and leaves the adapter launch count at one.
## Fixed-Test UI Run

- The accepted fresh run was launched only through the rebuilt workbench UI's `允许并开始` control against `/Users/yoyi/codex-workflow-mario-test`: `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783842259965530000`.
- The UI submission used `station3a-control-core-proof-v3.txt`, ASCII `station3a control core proof verified!`, 38 bytes, no trailing newline, one worker, and a unique-change check. This was an operator-selected probe literal, not a verified user-original requirement. `printf %s 'station3a control core proof verified!' | wc -c` returned `38`.
- The authoritative event stream records temporary-home creation, `control_core_dispatch_worker` with one worker, then `control_core_inspect_worker: denied: report_invalid: worker 最终消息不是符合回程契约的 JSON 代码块。`.
- The resulting UI state is `waiting_user` with `worker 回程格式无效，等待用户决定`. There is no `final_mark`, no accepted completion, and no `user_cancelled` claim.
- Because the worker's return was invalid, this evidence deliberately makes no claim that the target file was accepted, even if a physical file may exist. The invalid return itself did not execute acceptance; the earlier dispatch did execute once and is preserved truthfully.

## Fixed-Test UI Run v4

- Status: `SAFE_INTERCEPTION_SUCCEEDED__BUSINESS_VALIDATION_FAILED_V4`.
- The fresh v4 run was created only through the rebuilt workbench UI using `允许并开始`: `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783844142991226000`.
- The user-origin UI input preserved `station3a-control-core-proof-v4.txt`, the exact ASCII content `station3a control core proof v4 passed!`, UTF-8, `39 bytes`, and no trailing newline. The proposal view displayed those same literal filename and content values before authorization.
- The authoritative event stream records exactly one `control_core_dispatch_worker`: `dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:work-item-workflow-users-yoyi-codex-workflow-mario-test-default-project-director-planned-task-su:1783844265987`.
- The worker did not return a valid structured worker report. The control core recorded `control_core_inspect_worker: denied: report_invalid: worker 最终消息不是符合回程契约的 JSON 代码块。`, then stopped the run in `waiting_user`. There is no second dispatch, no accepted inspection, and no PASS/final mark.
- Read-only post-run verification reported `v4 target file: absent`; therefore this run makes no claim of file creation, readback, byte verification, or business completion.

## V5 Pre-Dispatch Session-Binding Gate

- Status: `SAFE_STOPPED_PRE_DISPATCH__NEEDS_REWORK`.
- Read-only runtime inspection found that distinct v3 and v4 dispatch/work-item records both used `native_thread_id` `019e7738-5e29-74e0-a22f-5c2481b64c38`. A new `dispatch_id` would not make that a new worker session, so v5 was stopped before any dispatch rather than treating it as independent execution.
- The UI-only v5 supervisor run was `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783846840880458000`. Its authoritative sidecar reports `workers: []`; its termination reason explicitly says exit code zero means only that the supervisor process ended. Read-only verification found `v5 target file: absent`.
- The repair makes supervisor-pilot materialization defer task-session binding, then creates and binds a fresh C1 task session for the exact work item immediately before authorized execution. It derives a forbidden set from historic workflow dispatches and session bindings, rejects a newly-created thread ID already present in that history before binding, records `fresh_task_session_bound`, and requires the execution result's `native_thread_id` to equal that exact new binding.
- Regression coverage verifies: C1 rejects a historical native thread before state mutation; supervisor dispatch derives the historical set and selects the exact work-item binding rather than an old node binding; and the actual materialization path preserves the raw user snapshot while only worker acceptance criteria enter the final worker prompt.
- This is a safe interception and implementation repair, not a v5 business run. A future run requires a newly authorized literal test input and fresh UI-created authorization, work item, and supervisor run; no v3, v4, or stopped v5 identity may be reused.

## V6 Progress-Failure Run

- Status: `SAFE_EXECUTION_SUCCEEDED__SUPERVISOR_PROGRESS_FAILED_V6`.
- Run: `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783850209472624000`.
- The worker wrote and correctly reported the 39-byte v6 proof, but the supervisor repeated the same completed `inspect_worker` action until the 12-action quota stopped the run. There was no accepted `finalize` or `report_user`; v6 is not PASS.
- The controller idempotency key prevented duplicate worker launches, so this exposed a supervisor progress-contract gap rather than a worker duplication or authorization failure.
- Repair: the runtime contract and next-step prompt now state that `inspect_worker` with `status="completed"` and `evidence_present=true` is complete and cannot be repeated. The supervisor must advance to `finalize`, `follow_up_worker`, or `request_user_decision`; after completed `finalize`, it must advance to `report_user`.

## V7 Fixed-Test UI Completion

- Status: `PASS__READY_FOR_3B`.
- Fresh run: `supervisor:workflow-users-yoyi-codex-workflow-mario-test-default:1783852010526616000`.
- Fresh authorization: `plan-auth:project-users-yoyi-codex-workflow-mario-test-workflow-users-yoyi-codex-workflow-mario-test-default-proof-proof:1783852009221`.
- Fresh work item: `work-item:workflow:users-yoyi-codex-workflow-mario-test:default:project-director:planned-task-supervisor-pilot-13fdceab2063105b96ba0295`.
- Exactly one worker dispatch was started and completed: `dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:work-item-workflow-users-yoyi-codex-workflow-mario-test-default-project-director-planned-task-su:1783852212107`. Two additional ledger rows were prepared dispatches only; they did not start workers.
- The worker used a fresh native thread `019f55e0-7886-7242-91ce-43928715595d`, one execution attempt (`attempt_no=1`), and zero follow-ups.
- The authoritative action order was exactly `dispatch_worker -> inspect_worker -> finalize(pass) -> report_user`. There were four accepted action records, one accepted inspection with `evidence_present=true`, one final mark, no repeated inspection, no second dispatch, no protocol error, no quota error, and no user-cancel claim.
- The target `/Users/yoyi/codex-workflow-mario-test/station3a-control-core-proof-v7.txt` independently verified as 39 bytes, last byte `33`, no trailing newline, SHA-256 `7777cfb8a53af75923f665191c80e5acf83c81436658c0b4cc61a25a420c18f3`.
- The raw worker return is a single valid JSON code block and is frozen at `evidence/raw/2026-07-12-station3a-v7/worker-last-message.txt` with SHA-256 `797a03764dab32ab3a46f8e83bab3369f8e7948b90a75f94738c4211e8a76add`.
- The runtime ledger identified the exact rebuilt executable and content hash: `codex-governance-workbench@0.1.0:bytes=59983480:mtime=1783851735:sha256=6f1bb237b274f89fbac21709eea8f1f582752ee7b553dd5c78e24173682c8012`. It also persisted supervisor and worker contract hashes.
- Complete before/after sidecars, UI screenshots, supervisor step outputs, the raw worker return, a byte-for-byte target snapshot, an independent verification transcript, and a passing SHA-256 manifest are frozen under `evidence/raw/2026-07-12-station3a-v7/`.

## Post-v7 Independent Audit and Risk Cleanup

- The independent handoff audit reconfirmed the v7 business chain, then found a separate P0 ledger-identity defect: legacy `binding_id` values were built from 96-character-truncated slugs. The live state had 71 binding records but only 61 unique IDs, so unbind could select the wrong record and SQLite `PRIMARY KEY ... ON CONFLICT DO NOTHING` could silently omit records.
- Binding IDs now use SHA-256 over the full workflow, node, work-item, and native-thread identity. The validator rejects duplicate binding rows; regressions cover long shared prefixes, exact unbind behavior, SQLite binding/dispatch conservation, and a migration-specific fail-closed guard for ambiguous legacy dispatch references. Existing references migrate only when they match a real legacy-ID candidate; collisions are narrowed only inside that candidate group. A missing reference may be filled from one unique workflow/node/work-item binding, while an old SHA or unknown retained-history reference is never globally rebound. A global foreign-key rule was deliberately not imposed because prepared dispatches may precede binding and retained historical dispatches may outlive a binding row.
- The first rebuilt-debug startup migration made a write-before backup, migrated all 71 legacy binding rows, and left 71 unique `binding:sha256:<64 hex>` values. A second independent audit correctly found that this first pass had not updated dispatch references: 350 non-empty dispatch references were stale and two early prepared rows had no reference.
- The repaired startup migration again ran without launching a supervisor or worker. It made a second write-before backup, repaired all 352 dispatch rows, and left 352/352 references present and resolvable with zero orphans. The audit records `migrated_binding_count=0`, `migrated_dispatch_count=352`, and `migrated_count=352`. Both migration stages, the incomplete intermediate state, automatic backups, final state, verification transcript, and hashes are frozen under `evidence/raw/2026-07-12-station3a-binding-id-migration/`.
- `authorization_snapshot_hash` and `task_package_fingerprint` now hold real SHA-256 values. The first dispatch freezes the task-package fingerprint for later inspect/finalize/report actions, and any authorization snapshot drift stops the run as `authorization_stale`.
- Fresh task-session binding now records `create_fresh_task_session`, the real requesting actor, and supervisor/control-core permission provenance instead of claiming a user selected an existing session. A valid inspected worker report is persisted back into the supervisor worker record.
- These repairs did not rerun or rewrite the v7 business evidence and did not start Station 3b. They remove the audit blocker before the separate 3b approval gate.

## Role and Control Boundary Closure

- New proposals persist separate non-empty `worker_acceptance_criteria`, `control_core_acceptance_criteria`, and `supervisor_acceptance_criteria`; the approval UI displays the same three columns.
- Only worker acceptance criteria enter the materialized worker goal and actual `worker_prompt`. Syn control duties and supervisor judgment duties do not enter the worker instruction.
- Syn owns authorization, exact work-item binding, fresh-session creation, authorization/work-item single-flight, quotas, idempotency, audit, failure cleanup, and adapter invocation. The supervisor model supplies judgment only; it cannot supply authority.
- Session creation/binding/dispatch is covered through an injectable production-path test, including historical-session rejection, concurrent reservation, and failed-dispatch cleanup. The supervisor pilot forbids node-level binding fallback.
- Public supervisor MCP remains read-only. Side-effect actions use the host control-core bridge.

## Remaining Non-Blocking Engineering Risks

- `finalize` remains advisory (`workflow_chain_state_written=false`) by the accepted machine-ruling design. Station 3b must not reinterpret it as an automatic user decision.
- One v7 supervisor step logged a model-list refresh timeout on stderr, but the step completed, the action ledger closed, and the final supervisor process exited zero. This is a latency/provider diagnostic, not a Station 3a correctness failure; retain it for performance follow-up.
- Repository-wide `cargo check --offline` retains the existing warning baseline, and `cargo fmt --check` retains the three historical drift files. They are not attributed to Station 3a and were not bulk-formatted.
- Untracked `.claude/`, `.playwright-cli/`, research documents, and prototype directories predate or belong to other work lines. They were deliberately preserved and not swept into this closure.

## Verification

- `cargo test --lib station3a_completed_inspection_must_advance_instead_of_repeating -- --nocapture`: passed.
- canonical supervisor-contract synchronization test: passed.
- `cargo test --lib --quiet`: `867 passed; 0 failed; 43 ignored; 910 total`.
- `npm run typecheck`: `tsc --noEmit` exited zero.
- `npm run test:offline-interaction`: `offline interaction tests passed: 15`.
- `cargo check --offline`: finished successfully; existing library warning count is `570`.
- `cargo fmt --check`: no new drift; it reports only pre-existing formatting drift in `src/codex_db.rs`, `src/codex_local_runner.rs`, and `src/mcp/storage.rs`.
- `git diff --check`: exited zero with no output.
- The rebuilt debug app used by v7 is `prototypes/productized-desktop-shell/src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app`; its binary SHA-256 is recorded above and matches the runtime ledger.
