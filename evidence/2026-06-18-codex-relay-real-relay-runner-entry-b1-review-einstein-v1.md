# B1 Real Relay Runner Entry Review - Einstein

日期：2026-06-18
角色：Agent E / Evaluator / focused code reviewer
状态：CLEAR_WITH_NOTE

## Acceptance Target

独立只读复核 B1「真 relay runner-entry」包是否只补 env-gated 真发入口与结构/mock 测，且在本包内没有真跑 Codex、没有设置 `MANUAL_RELAY_REAL_CODEX_CONFIRM`、没有放宽旧闸、没有扩大 readback 或前端/Tauri 命令面。

## Scope

已读：

- `tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md`
- `CURRENT.md`
- `AGENTS.md`
- `skills/using-superpowers/SKILL.md`
- `skills/evaluator-acceptance-review/SKILL.md`
- `skills/requesting-code-review/SKILL.md`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- 旧闸 5 文件的 scoped diff / targeted rg：`session_continuation_store.rs`、`k3_b1_recovery.rs`、`real_execution_command.rs`、`codex_local_runner.rs`、`h5_project_dispatch_bridge.rs`

未做：

- 未运行测试。
- 未运行 true Codex。
- 未设置 `MANUAL_RELAY_REAL_CODEX_CONFIRM`。
- 未读取 `/Users/yoyi/.codex`、auth、secret、token、session transcript 内容。
- 未修改产品代码。

## Findings

None for P0 / P1 / P2.

## Requirement Checks

1. B1 ignored true runner exists and is env-gated: PASS.
   - `manual_relay_b1_real_codex_runner_entry_requires_user_present_env` is `#[ignore]` and explicitly says user-present env authorization is required.
   - It reads `MANUAL_RELAY_REAL_CODEX_CONFIRM` and asserts exact value `CONFIRMED_USER_PRESENT_REAL_RELAY` before constructing the B1 real run.
   - It then sets `run_input.mock_behavior = "real_codex_env_gated"` and calls `run_manual_relay_once`, which would enter the real Codex running path only after the env check.
   - The real-mode config path calls `ensure_real_codex_env_authorized()` before returning `real_codex_executed: true`.

2. No true Codex run / no env authorization set by this review: PASS.
   - Reviewer did not run tests or ignored tests.
   - Reviewer did not set the env var.
   - `rg` in `manual_relay.rs` found no `std::env::set_var`; only `std::env::var` in ignored env-gated tests and `std::env::remove_var` in the non-ignored no-env negative test.

3. Mock and placeholder process modes are test-only in production: PASS.
   - `run_manual_relay_once` rejects placeholder process behavior unless `placeholder_process_mode_allowed()`.
   - `placeholder_process_mode_allowed()` returns `true` only under `#[cfg(test)]` and `false` under `#[cfg(not(test))]`.
   - `mock_codex_process:*` and `mock_codex_process_sleep:*` are similarly rejected unless `mock_codex_process_mode_allowed()`, which is also `#[cfg(test)]` only.
   - Placeholder process config still returns `manual_relay_placeholder_process_mode_unexpected` outside the separate placeholder sleep branch.

4. Readback remains workbench-managed last-message only: PASS.
   - Preview creates `.../manual-relay-runs/<relay>/last-message.txt`.
   - Command construction substitutes the workbench-managed last-message path for the `--output-last-message` placeholder.
   - `read_last_message_summary` reads only that path and returns hash/size/status.
   - Readback plan states `last_message_only_no_full_transcript_read` and warning `manual_relay_does_not_read_rollout_body`.
   - No product path inspected here reads transcript, rollout body, or `.codex`; `.codex` matches in `manual_relay.rs` are denied-material policy/test samples.

5. Stop path kills only registered attempt in runner tests: PASS.
   - `stop_manual_relay_attempt` removes exactly the requested `relay_attempt_id` from the active registry and kills only that stored child.
   - B1 mock stop test starts a `mock_codex_process_sleep:*` attempt, stops by that returned attempt id, and asserts `process_kind == "mock_codex"`, `real_process_killed`, and `!real_codex_executed`.
   - This is backend/mock-runner evidence only; it does not imply product UI stop wiring.

6. Old gates and scope expansion: PASS within allowed scope.
   - Scoped `git diff --` for the five old-gate files was empty.
   - Scoped `git diff --name-only -- <allowed task/manual/old-gate paths>` returned only `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`.
   - `git diff -- manual_relay.rs` showed changes limited to placeholder test-only gating plus B1 mock/ignored-real runner tests and test helpers.

## Evidence

Commands / inspections run:

- `cat skills/using-superpowers/SKILL.md`
- `cat skills/evaluator-acceptance-review/SKILL.md`
- `cat skills/requesting-code-review/SKILL.md`
- `cat tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md`
- `cat CURRENT.md`
- `cat AGENTS.md`
- `rg -n "real_codex_env_gated|ensure_real_codex_env_authorized|MANUAL_RELAY_REAL_CODEX_CONFIRM|manual_relay_real|mock_codex_process|placeholder_process|run_manual_relay_once|real_codex_executed|output-last-message|last-message|Command::new|spawn|kill|transcript|rollout|\\.codex|read_to_string|read_to_end" manual_relay.rs`
- `git diff -- manual_relay.rs`
- `git diff -- session_continuation_store.rs k3_b1_recovery.rs real_execution_command.rs codex_local_runner.rs h5_project_dispatch_bridge.rs`
- `awk` line inspections for `manual_relay.rs` sections: preview/last-message setup, `run_manual_relay_once`, env gate/process config, spawn/readback, stop path, strict path verification, B1 mock tests, B1 ignored real runner, B1 fixture helper, and test-only wait/finalize helpers.
- `rg -n "MANUAL_RELAY_REAL_CODEX_CONFIRM|real_codex_env_gated|mock_codex_process|placeholder_process|Command::new|codex exec|tauri::command|manual_relay|codex" <old-gate files>`
- `git diff --name-only -- <allowed task/manual/old-gate paths>`

## Residual Notes / Risks

- CLEAR_WITH_NOTE rather than plain CLEAR because this evaluator did not read frontend files or Tauri command registry files; those paths were outside the allowed read list. Within the allowed scope, no frontend/Tauri command expansion is visible, and the main agent's provided shape-gate evidence says the command total warning is pre-existing.
- Ignored B1 true runner was intentionally not run. Actual first true spawn remains a separate user-present step with `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`.
- No tests were rerun by this evaluator, by package instruction. Test safety relies on the main agent's provided fresh test evidence plus this code/diff inspection.

## Decision

STATUS: CLEAR_WITH_NOTE

Safe to hand back before commit, subject to the residual scope caveat above and the existing rule that the first true Codex run remains a separate user-present step.
