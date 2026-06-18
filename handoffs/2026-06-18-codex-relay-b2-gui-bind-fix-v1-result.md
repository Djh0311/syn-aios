# Codex Relay B2 GUI Bind Fix Result v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-b2-gui-bind-fix-v1.md`
Base HEAD: `157738c`
Commit: not committed; awaiting consulting-line review and user decision.

## Summary

The GUI bind bug is fixed in code and regression tests. GUI direct relay now binds from the clicked `selectedSession.project_root`, not from stale project dropdown state. The conversation dropdown no longer hides other-project Codex sessions behind the currently selected project.

No real Codex relay was run. No ignored true-run tests were executed. No `.codex` access was added.

## Changed Files

- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`
- `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-v1.md`
- `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-artifacts/tauri-after-bind-fix.png`
- `handoffs/2026-06-18-codex-relay-b2-gui-bind-fix-v1-result.md`

## Key Behavior

- Codex session + nonempty `project_root` -> GUI direct relay enabled, target project = selected session project root.
- Codex session + missing `project_root` -> blocked with `当前会话未记录项目路径`.
- Non-Codex session -> blocked with `仅 Codex 会话可用`.
- No selected session -> blocked with `未绑定会话`.
- Direct-send request uses the same derived project root for `target_project_root`, `target_cwd`, and `allowed_write_roots`.

## Verification

- `npm run test:offline-interaction`: passed, including new bind-fix regressions.
- `npm run typecheck`: passed.
- `npm run build`: passed with existing Vite large chunk warning.
- `cargo test --lib manual_relay`: 16 passed / 2 ignored.
- `cargo test --lib`: 544 passed / 24 ignored.
- Old gate focused tests:
  - `codex_local_runner`: 12 passed.
  - `session_continuation_store`: 16 passed / 4 ignored.
  - `real_execution_command`: 36 passed / 7 ignored.
  - `k3_b1_recovery`: 5 passed.
  - `h5_project_dispatch_bridge`: 4 passed.
- `cargo fmt -- --check`: passed.
- `git diff --check`: passed.
- Tauri desktop app launched directly with existing Vite server reused; screenshot captured in `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-artifacts/tauri-after-bind-fix.png`.

## Tauri Note

Port `5173` was already occupied by Vite, so ordinary `tauri:dev` would fail by trying to start Vite again. The working desktop launch command was:

```text
/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev --config '{"build":{"beforeDevCommand":""}}' --no-dev-server-wait
```

macOS denied Accessibility control for automated clicking, so the screenshot remains on the current selected non-bound state. The code/test proof covers the bind behavior; user-side final confirmation is to manually select a Codex session in the launched Tauri app.

## Boundary

- No Send click was performed in this fix package.
- No true Codex relay was run.
- No backend relay safety gate was modified.
- No old K3/H2/H5 gate file changed.
- No product global read/write path, auto-chain, K3-B1/K3-B2 unlock, role injection, or multi-agent unlock was added.

## Independent Review

Review line: Erdos.

Review file: `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-review-erdos-v1.md`

Status: CLEAR. No P0/P1/P2.
