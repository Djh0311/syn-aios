# S1B-H1 真 CLI 单工具批准 harness 验证 v1

日期：2026-07-19
结论：**通过。** 测试专用 wrapper 已让真实 `exec resume` 的 `submit_proposal` 调用到达既有 MCP handler，并在副本 store 落下一张待用户确认卡；随后真实 invalid-resume 完成归档换代与事实重建。产品批准语义、产品 argv、沙箱和 MCP 白名单未改。

## 一、案发与根因

修前真实 rollout 的首轮不是“模型没调工具”：

- 模型生成了完整 `submit_proposal`；
- `thread_settings_applied.approval_policy` 仍为 `never`；
- `mcp_tool_call_end.result` 为 `user cancelled MCP tool call`；
- proposal 74→74，唯一 marker=0，chain 不动。

对新 thread 的真实探针证明单工具预批准字段可用；但把同一 `-c` 放在外层 `exec` 前，旧 thread 的 resume 仍被取消。`codex exec resume --help` 显示 `resume` 子命令自己接受 `-c/--config`，因此根因是**覆盖落在错误的 CLI 解析层**，不是产品 handler 或 proposal store 失败。

实现依据：

- [Codex 配置 schema](https://github.com/openai/codex/blob/main/codex-rs/core/config.schema.json)
- [Codex auto-review 与批准策略说明](https://learn.chatgpt.com/docs/sandboxing/auto-review)

## 二、修复边界

只改：

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`

测试 wrapper 的两支：

```text
initial: codex --strict-config -c enabled_tools=[submit_proposal] -c submit_proposal.approval_mode=approve exec ...
resume:  codex --strict-config exec ... resume -c enabled_tools=[submit_proposal] -c submit_proposal.approval_mode=approve ...
```

边界断言：

- 仅允许 `codex exec`；
- 仅暴露并预批准 `supervisor_orchestrator.submit_proposal`；
- 不设置 `default_tools_approval_mode`；
- 不设置/放宽 `approval_policy`、`approvals_reviewer` 或 sandbox；
- 不含 `--full-auto`、`dangerously-bypass`；
- 不改产品 `config.toml`，不新增 command / sidecar；
- wrapper 父进程等待并透传真实 Codex 退出码，保持既有进程登记可识别。

## 三、真实 ignored 用例

命令所需测试专用变量：

- `SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE=<受控 debug workbench>`
- `SYN_S1B_LIVE_MCP_TOOL_APPROVAL_HARNESS_CONFIRM=CONFIRMED_TEST_ONLY_SUBMIT_PROPOSAL_APPROVAL`
- `SYN_S1B_LIVE_REAL_CODEX=<绝对真实 codex 路径>`
- `SYN_S1B_LIVE_WORKFLOW_STATE_PATH=<副本 workflow-state.v0.json>`
- `SYN_S1B_LIVE_WORKFLOW_ID=workflow:users-yoyi-codex-workflow-mario-test:default`

执行：

```text
cargo test --offline s1b_live_resume_tool_card_and_replacement_require_explicit_harness_authorization -- --ignored --nocapture --test-threads=1
```

结果：

```text
1 passed; 0 failed; finished in 71.89s
```

### 3.1 真工具落卡

- proposal store revision：131→132。
- proposal id：`proposal:project-users-yoyi-codex-workflow-mario-test-workflow-users-yoyi-codex-workflow-mario-test-default-s1b-live-card-mario-20260719-s1b-live-card-mario-20260719-readme:1784409636528`
- status：`pending_user_confirmation`。
- `user_goal` 与 `goal_summary` 均含 `S1B_LIVE_CARD_MARIO_20260719`。
- orchestrator `supervisor_tool_call` 为 `accepted`，结果明确 `proposal_created_pending_user_confirmation`。
- workflow chain run 数在用例前后保持 40；未批准、未起链、未派发 worker。

### 3.2 同 thread 续接

- 落卡轮与后两轮均复用 generation 6 的 thread `019f76c0-6639-74d3-a3f4-0688e58498ed`。
- 后两轮都回引 `S1B_LIVE_FACT_MARIO_20260719`，证明不是只比对 thread id。

### 3.3 真 invalid-resume 换代

- 用例只把登记 thread 的末位改为不存在的 `019f76c0-6639-74d3-a3f4-0688e58498e0`。
- 真 CLI 返回非零，stderr 含 `no rollout found ... (code -32600)`，且没有 stdout `thread.started`。
- 审计分类：`resume_exit_without_thread_started`。
- generation 6 归档到 `archive/generation-6-1784409662650856000`。
- active home 元数据为 generation 7。
- replacement thread：`019f771a-ac76-7bc1-b92e-a8204cf92f9f`。
- replacement 首轮回引 `S1B_LIVE_FACT_MARIO_20260719`，并明确未执行、未批准、未起链。

### 3.4 进程对账

- 副本 `exec-process-registry.v1.json` 最终 `entries=[]`。
- `ps` 未见包含本次 scratchpad 路径、临时 `s1b-live-codex-approval-harness` 或新旧测试 thread id 的残留进程。
- 同机另有一条 07-18 已启动、指向真实 App workflow-state 的既有 supervisor MCP 进程；它不属于本副本测试，未清理，也未被当成“全机零主管进程”。

## 四、聚合闸

- `cargo test --offline s1b_ -- --nocapture`：16 passed / 0 failed / 1 ignored。
- `cargo test --offline s1_ -- --nocapture`：11 / 0 / 1 ignored。
- `cargo test --offline -q m5b_`：9 / 0。
- `cargo test --offline -q m5c_`：5 / 0。
- `cargo test --offline --lib`：1009 / 0 / 44 ignored。
- `cargo check --offline`：通过；既有 594 warnings 未清零。
- `pnpm typecheck`：通过。
- `pnpm test:offline-interaction`：15 组通过。
- shape baseline：Status pass，13 errors / 5 warnings / 5 infos。
- shape check：预期 exit 1，同为 13 / 5 / 5，零净增。
- `git diff --check`、改动 Rust 文件 `rustfmt --check`：通过。

## 五、历史失败与不外推

- 修前多次 `user cancelled MCP tool call` 保留为 harness 诊断证据，不改写成产品回归。
- 一次复跑因漏带 `SYN_P1_A_RESIDENT_WORKBENCH_EXECUTABLE` 被私有家一致性闸拒绝；补回既有测试绑定后通过，未改私有家。
- 一次沙箱内复跑无法读取子进程 `ps/lstart`，按原真机口径在获准的沙箱外复跑；这不是产品失败。
- 本证据证明的是**真实 CLI + 副本 store + 既有 MCP handler**。它不等于真实 Tauri 页面已目视显示该卡，也不等于用户已批准或完整“批→跑”链已通过。
