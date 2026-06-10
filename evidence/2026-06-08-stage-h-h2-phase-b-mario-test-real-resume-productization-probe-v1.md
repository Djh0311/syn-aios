# Evidence：Stage H / H2 Phase B Mario Test Real Resume Productization Probe v1

日期：2026-06-08

## 1. 结论

H2 Phase B `mario test` 真实 resume 产品化探针已完成，结论为：

```text
accepted_as_h2_phase_b_mario_test_real_resume_productization_probe
```

接受为：

- 用户已授权在测试项目和 `mario test` 范围内执行真实权限动作。
- 后端已具备 `codex-local` Phase B real resume runner 产品路径：结构化 argv、stdin prompt、prompt hash 校验、guard、continuation attempt、audit、runtime log、workbench-managed readback。
- 对 `/Users/yoyi/Documents/mario test` 的 session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 完成真实 `codex exec resume` 探针。
- 本次真实执行发送了 prompt，真实执行了 Codex，并写入 `/Users/yoyi/.codex`。
- readback 通过 workbench-managed last message 成功返回固定标记：`H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`。
- `/Users/yoyi/Documents/mario test` 四个项目文件 hash 与 H2 Phase B 记录一致，未发现项目文件变化。

不接受为：

- H3 真实新会话完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。
- 任意项目 / 任意 session 的无限制通用执行完成。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动重试、取消、恢复或完整 failure recovery 产品化。
- 自由聊天式裸 Codex 控制器。

## 2. 授权和执行对象

执行对象：

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
workflow_id: workflow:mario-test:h2-phase-b
node_id: node:global-director:h2-phase-b
session_id: 019e798a-6ce5-76c3-b8ee-33bd0fda841f
adapter_id: codex-local
operation: resume
sandbox: workspace-write
timeout_ms: 120000
```

成功执行目录：

```text
/Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume/run-1780855910749629000/
```

关键记录：

```text
workflow_state_path: /Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume/run-1780855910749629000/workflow-state.v0.json
continuation_sidecar: /Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume/run-1780855910749629000/session-continuations.v1.json
runtime_log_sidecar: /Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume/run-1780855910749629000/runtime-logs.v1.json
last_message_path: /Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume/run-1780855910749629000/runtime/h2-phase-b/mario.last-message.txt
```

成功 run 摘要：

```text
authorization_status: phase_b_real_resume_executed
attempt.status: succeeded
prompt_sent: true
real_codex_executed: true
writes_codex_home: true
readback.status: succeeded
readback.result_count: 1
```

## 3. 实现落点

主要代码：

- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

本轮主管复核补丁：

- `session_continuation_store.rs` 增加 Phase B 当前 warning 汇总过滤。
- 过滤项只影响当前 `ControlledSessionContinuation.warnings` 摘要，不改历史 confirmation audit event。
- 修复目标：Phase B 成功后当前摘要不能继续显示 `level_a_stub_only`、`prompt_sent_false`、`real_codex_executed_false`、`writes_codex_home_false`、`level_b_real_execution_requires_user_approval`。
- 新增单测覆盖：历史确认 audit 保留 Level A 语义，Phase B 当前 continuation 摘要不保留反事实 warning。

## 4. 过程缺陷和修复

本轮记录两个必须保留的过程缺陷：

1. 第一次真实探针曾暴露边界缺陷：真实 runner 成功，但测试失败，因为 sidecar warning 曾持久化 `last_message_summary:H2_PHASE_B...`。这会把 readback 正文混入 sidecar 摘要。随后已修复为不持久化 last-message 正文摘要。
2. 全局主管复核成功 run 时发现 continuation 顶层 warnings 仍继承 Level A 旧状态，例如 `prompt_sent_false` 和 `real_codex_executed_false`。这会导致当前 UI / 读模型同时显示真实执行成功和未执行。已修复为 Phase B 写回当前摘要时过滤过期 pre-Phase-B warning；历史 confirmation audit event 不篡改。

第二次真实探针成功后扫描：

```text
rg -n "H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08|last_message_summary" \
  session-continuations.v1.json runtime-logs.v1.json workflow-state.v0.json
```

结果：无命中。

说明：固定 marker 只存在于 workbench-managed last message 文件，不进入 continuation sidecar、runtime log sidecar 或 workflow state。

## 5. Readback

last message 内容：

```text
H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08
```

readback 说明：

- 只读取 workbench-managed last message。
- 未读取完整 transcript / rollout 作为 readback。
- `result_count = 1` 表示 last message readback 成功。
- `readback_unavailable` / `readback_failed` / `timed_out` 路径仍保持 `result_count = null`，不能显示为真实 0 条结果。

## 6. Runtime Log 和 Audit

runtime log：

- `runtime-log:workflow-run:session-continuation:v1:51fd2f4a011059f5d6a2fc77f90aba033834641e0bfcfea01aa3ec695d1e4522`
- `runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-08T00:01:00Z:09a276f40fad45aa`
- `runtime-log:readback:session-continuation-attempt:h2-phase-b:2026-06-08T00:01:00Z:09a276f40fad45aa`

audit event：

- `audit:session-continuation-confirmed:2026-06-08T00:00:00Z:09a276f40fad45aa`
- `audit:session-continuation-h2-phase-b-started:2026-06-08T00:01:00Z:09a276f40fad45aa`
- `audit:session-continuation-h2-phase-b-completed:2026-06-08T00:01:00Z:09a276f40fad45aa`

边界：

- runtime log 只保存脱敏摘要、refs、状态和分类。
- audit 记录决策、actor、状态变化和原因。
- runtime log 与 audit 互相引用，但不互相替代。
- prompt body 不进入 sidecar / audit / runtime log。
- raw stdout / stderr 不进入普通运行日志；当前只保留截断 stderr summary 作为诊断 warning。

## 7. 项目文件 hash

复核 hash：

| file | sha256 |
| --- | --- |
| `/Users/yoyi/Documents/mario test/index.html` | `f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf` |
| `/Users/yoyi/Documents/mario test/styles.css` | `6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f` |
| `/Users/yoyi/Documents/mario test/game.js` | `814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd` |
| `/Users/yoyi/Documents/mario test/README.md` | `02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5` |

结论：

- 这些 hash 与成功探针记录一致。
- 本轮未发现 `mario test` 项目文件变化。

## 8. 验证记录

已通过：

```text
cargo test --lib session_continuation_store -- --nocapture
cargo test --lib codex_local_runner -- --nocapture
cargo test --lib runtime_log_store -- --nocapture
cargo test --lib runtime_session_attention -- --nocapture
cargo test --lib
rustfmt --check src/session_continuation_store.rs src/codex_local_runner.rs src/commands.rs src/types.rs src/lib.rs
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果摘要：

- `cargo test --lib session_continuation_store -- --nocapture`：11 passed，1 ignored。
- `cargo test --lib codex_local_runner -- --nocapture`：10 passed。
- `cargo test --lib runtime_log_store -- --nocapture`：1 passed。
- `cargo test --lib runtime_session_attention -- --nocapture`：2 passed。
- `cargo test --lib`：249 passed，2 ignored。
- `npm run test:offline-interaction`：12 scenarios passed。
- `npm run build`：通过，保留既有 Vite chunk-size warning。
- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。

说明：

- 本轮未复跑 ignored 的真实 mario probe，避免重复写 `/Users/yoyi/.codex`。
- 真实执行事实来自前述成功 run 目录和已保存证据。

## 9. 边界确认

本轮真实探针做了：

- 执行真实 `codex exec resume`。
- 发送 H2 Phase B 固定安全探针 prompt。
- 写入 `/Users/yoyi/.codex`，属于用户授权的真实 Codex resume 副作用。
- 写入工作台自有 continuation sidecar。
- 写入 runtime log sidecar。
- 写入 audit event。
- 写入 workbench-managed last message readback 文件。

本轮没有做：

- 没有执行 H3 真实新会话。
- 没有执行 H5 项目工作流真实派发。
- 没有接 planned adapters。
- 没有读取 auth/token/`.env`/secret/keychain/OAuth/provider credential。
- 没有读取完整 transcript / rollout 作为 readback。
- 没有把 prompt body 写入 argv、sidecar、audit、runtime log 或 evidence。
- 没有把 readback unavailable / failed 写成 0 条结果。
- 没有把本次单项目 probe 宣称为阶段 H 完成。

## 10. 过程偏差留痕

全局主管收尾扫描时曾出现一次非产品路径偏差：扫描命令把 Markdown 反引号放进 shell 双引号，触发了 ``codex exec resume`` 命令替换。输出显示没有通过 stdin 提供 prompt，并尝试访问 `/Users/yoyi/.codex/state_5.sqlite` 后因 readonly database 失败。

该偏差不是 H2 产品代码路径，也不是本轮成功 H2 Phase B probe；后续扫描命令必须使用单引号或避免反引号，防止 shell command substitution。

## 11. 下一步

下一步可以进入 H3-B final approval / real new session fixture run，但仍需单独执行点授权。

H3-B 不能自动继承 H2 Phase B 的授权；执行前仍必须确认 fixture、work item / workflow / node 绑定、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、readback、runtime log、audit、evidence 和 rollback。

H4/H5/H6/H7 仍是后续阶段，不能因为 H2 Phase B 成功而跳过。
