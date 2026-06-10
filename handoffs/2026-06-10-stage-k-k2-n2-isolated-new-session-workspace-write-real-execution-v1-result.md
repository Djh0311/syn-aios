# Stage K / K2 N2 Isolated New Session Workspace-Write Real Execution Handoff v1

日期：2026-06-10

结论：K2-N2 真实 `new_session/workspace-write` 受控验收通过。它证明 K2 Product Command -> decision -> Phase A -> Phase B -> continuation/runtime/audit/readback 链路可以在 Stage K 隔离项目里创建一次真实 Codex 新会话，并只写授权的 workspace-write probe 文件。

关键结果：

- `cargo test --lib real_execution_command::tests::k2_n2_real_isolated_new_session_workspace_write_requires_env_authorization -- --ignored --exact --nocapture` 通过。
- `last_message` 和 allowed file 均包含 `K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10`。
- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=true`。
- `readback_status=succeeded`、`result_count=1`。
- changed project files 只有 `.workbench/stage-k/k2/new-session-write-probe.md`。
- allowed file hash 为 `603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07`。
- fixture `README.md` hash 保持 `cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`。

证据：

- `evidence/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1.md`
- `tmp/stage-k-k2-real-execution/runs/stage-k-k2-n2-isolated-new-session-workspace-write-3ff79f634ab4-1781043540559078000/`
- `test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`

边界：

- 本次确实发送 prompt，确实执行真实 Codex，确实写入 `/Users/yoyi/.codex`。
- 只写 allowed file，未写 fixture 其他文件。
- 未启动 Tauri / Browser / Chrome / 截图工具。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

下一步：

- 进入 K2 总验收。
- K2 总验收必须把 R1/R2/N1/N2 统一回收为 `accepted_with_deferred_items`，并明确 K3/K4/K5/K6 尚未完成。
