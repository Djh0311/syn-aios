# Stage K / K2 R2 Mario Test Resume Workspace-Write Real Execution Handoff v1

日期：2026-06-10

结论：K2-R2 真实 `resume/workspace-write` 受控验收通过。它证明 K2 Product Command -> decision -> Phase A -> Phase B -> continuation/runtime/audit/readback 链路可以对指定 `mario test` 开发线 session 完成一次真实 Codex resume，并只写入授权的 workspace-write probe 文件。

关键结果：

- `cargo test --lib real_execution_command::tests::k2_r2_real_mario_test_resume_workspace_write_requires_env_authorization -- --ignored --exact --nocapture` 通过。
- `last_message` 和 allowed file 均包含 `K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10`。
- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=true`。
- `readback_status=succeeded`、`result_count=1`。
- changed project files 只有 `.workbench/stage-k/k2/resume-workspace-write-probe.md`。
- `mario test` 的 `index.html`、`styles.css`、`game.js`、`README.md` hash 前后一致。

证据：

- `evidence/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1.md`
- `tmp/stage-k-k2-real-execution/runs/stage-k-k2-r2-mario-test-resume-workspace-write-03091a7bfc9e-1781043140606548000/`
- `/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`

边界：

- 本次确实发送 prompt，确实执行真实 Codex，确实写入 `/Users/yoyi/.codex`。
- 只写 allowed file，未写 `mario test` 核心文件。
- 未执行 N1/N2。
- 未启动 Tauri / Browser / Chrome / 截图工具。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

下一步：

- 进入 K2-N1 `new-session/read-only` 前检查。
- N1 必须使用 Stage K 隔离项目，read-only 且 `allowed_write_roots=[]`，执行后确认 fixture 文件集不变，readback marker 成功，prompt body 不持久化。
- K2 仍不能收口；N1/N2 和 K2 总验收仍未完成。
