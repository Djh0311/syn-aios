# Stage K / K2 R1 Mario Test Resume Read-Only Real Execution Handoff v1

日期：2026-06-10

结论：K2-R1 真实 `resume/read-only` 受控验收通过。它证明 K2 Product Command -> decision -> Phase A -> Phase B -> continuation/runtime/audit/readback 链路可以对指定 `mario test` session 完成一次真实 Codex resume，并保持项目核心文件不变。

关键结果：

- `cargo test --lib real_execution_command::tests::k2_r1_real_mario_test_resume_read_only_requires_env_authorization -- --ignored --exact --nocapture` 通过。
- `last_message` 包含 `K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10`。
- `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=false`。
- `readback_status=succeeded`、`result_count=1`。
- `mario test` 的 `index.html`、`styles.css`、`game.js`、`README.md` hash 前后一致。

证据：

- `evidence/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1.md`
- `tmp/stage-k-k2-real-execution/runs/stage-k-k2-r1-mario-test-resume-read-only-2dc6d059fe53-1781031189185198000/`

边界：

- 本次确实发送 prompt，确实执行真实 Codex，确实写入 `/Users/yoyi/.codex`。
- 未写 `mario test` 项目核心文件。
- 未执行 R2/N1/N2。
- 未启动 Tauri / Browser / Chrome / 截图工具。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

下一步：

- 等复核线最终报告回收后，进入 K2-R2 `resume/workspace-write` 前检查。
- R2 必须只允许写 `/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`，并重新记录核心 hash、changed files、allowed file hash 和 readback。
- K2 仍不能收口；R2/N1/N2 和 K2 总验收仍未完成。
