# Stage K / K2 N1 Isolated New Session Read-Only Real Execution Handoff v1

日期：2026-06-10

结论：K2-N1 真实 `new_session/read-only` 受控验收通过。它证明 K2 Product Command -> decision -> Phase A -> Phase B -> continuation/runtime/audit/readback 链路可以在 Stage K 隔离项目里创建一次真实 Codex 新会话，并保持项目文件不变。

关键结果：

- `cargo test --lib real_execution_command::tests::k2_n1_real_isolated_new_session_read_only_requires_env_authorization -- --ignored --exact --nocapture` 通过。
- `last_message` 包含 `K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10`。
- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=false`。
- `readback_status=succeeded`、`result_count=1`。
- Stage K 隔离项目文件集保持不变，`README.md` hash 为 `cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`。

证据：

- `evidence/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1.md`
- `tmp/stage-k-k2-real-execution/runs/stage-k-k2-n1-isolated-new-session-read-only-b19d41bf5e37-1781043480047061000/`

边界：

- 本次确实发送 prompt，确实执行真实 Codex，确实写入 `/Users/yoyi/.codex`。
- 未写 fixture 项目文件。
- 未启动 Tauri / Browser / Chrome / 截图工具。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

下一步：

- 进入 K2-N2 `new-session/workspace-write` 执行回收。
- N2 必须只写 `test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`，执行后记录 allowed file hash。
- K2 仍不能收口；N2 和 K2 总验收仍未完成。
