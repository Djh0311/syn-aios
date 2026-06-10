# Stage K / K2 General Codex Resume And New Session Product Entry Level A Evidence v1

日期：2026-06-10

状态：Level A 完成，Phase B 普通入口接线完成，R1/R2/N1/N2 env-gated 真实执行测试 harness 完成。补充：K2-R1 已在后续 checkpoint 中真实执行并通过，见 `evidence/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1.md`；K2 尚未完成，R2/N1/N2 仍未执行。

## 1. 任务包

- `tasks/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-v1.md`

## 2. Level A 改动范围

改动文件：

- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `test-fixtures/stage-k-isolated-project/README.md`
- `tmp/stage-k-k2-real-execution-prompts/k2-r1-mario-test-resume-read-only.txt`
- `tmp/stage-k-k2-real-execution-prompts/k2-r2-mario-test-resume-workspace-write.txt`
- `tmp/stage-k-k2-real-execution-prompts/k2-n1-isolated-new-session-read-only.txt`
- `tmp/stage-k-k2-real-execution-prompts/k2-n2-isolated-new-session-workspace-write.txt`
- `tasks/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-v1.md`
- `evidence/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-level-a-v1.md`
- `handoffs/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-level-a-v1-result.md`

## 3. 代码事实

### 3.1 Product Command new-session Phase B 契约暴露

- 后端新增 `run_real_execution_product_command_new_session_phase_b_at`，复用既有 `run_real_execution_product_command_new_session_phase_b_with_runner`。
- Tauri command 新增 `run_real_execution_product_command_new_session_phase_b`。
- Tauri invoke handler 已注册该 command。
- 前端新增 `RunRealExecutionProductCommandNewSessionPhaseBInput` 和 `runRealExecutionProductCommandNewSessionPhaseB`。
- 本轮没有新增裸 `Command::new("codex")`，真实执行仍归口既有 runner / Product Command bridge。

### 3.2 Agent 普通层 K2 发送预览和非真实产品流

- 普通输入框提交后调用 `previewRealExecutionProductCommand`，source_kind 为 `codex_control`。
- 支持 `resume` 和 `new_session` 两种预览模式。
- `新建对话` 按钮不再是 K1 disabled 占位，而是切换到 K2 new-session preview mode；文案明确“只生成预览，不直接创建”。
- 预览卡显示项目、对话 / 新建对话、读回状态、结果数和记忆影响。
- 预览卡新增 `写入准备`、`用户确认`、`记录预检` 三步非真实产品流，分别调用 `prepareRealExecutionProductCommand`、`confirmRealExecutionProductCommand` 和 `runRealExecutionProductCommandPhaseA`。
- `记录预检` 只进入 Phase A no-op trace；不会发送 prompt，不会执行真实 Codex，不会调用 Phase B。

### 3.3 Agent 普通层 Phase B 产品入口接线

- 预览卡新增 `确认执行 Codex` 按钮。
- 按钮只在 Phase A 输出存在且无阻断、且没有既有 Phase B 输出时启用。
- `resume` 调用 `runRealExecutionProductCommandPhaseB`。
- `new_session` 调用 `runRealExecutionProductCommandNewSessionPhaseB`。
- Phase B authorization matrix 从当前 preview request 派生：project root、target session / work item、sandbox、prompt summary/ref/hash、allowed write roots、readback plan 和 rollback plan 必须与 preview 一致。
- 用户修改任务正文会清空旧 preview、prepare、decision、Phase A 和 Phase B 状态，避免旧权限预览对应新 prompt body。
- 本轮没有点击该按钮，没有执行真实 Codex，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。

### 3.4 N1/N2 隔离项目前置准备

- 新增 `test-fixtures/stage-k-isolated-project/README.md`。
- 该 fixture 仅用于 K2 `new_session` 受控探针。
- K2-N2 唯一允许写入区仍限定为 `.workbench/stage-k/k2/new-session-write-probe.md`。
- README hash：`cf1289518849fc1a6947c2c034717f5c4e5afaa0726d56b5de9c733bdd1c201c`。
- 本轮只创建 fixture 文件，没有执行 new-session，没有调用 Codex。

### 3.5 R1/R2/N1/N2 canonical prompt freeze

- R1 prompt：`tmp/stage-k-k2-real-execution-prompts/k2-r1-mario-test-resume-read-only.txt`
- R1 hash：`2dc6d059fe5373ba547da91bd2b28296ab0ec15450cb7264f26243a3bff86e1d`
- R2 prompt：`tmp/stage-k-k2-real-execution-prompts/k2-r2-mario-test-resume-workspace-write.txt`
- R2 hash：`03091a7bfc9e8a9b86bcc79f421f8b0ab982cd513cca7e1b8346afc709205c49`
- N1 prompt：`tmp/stage-k-k2-real-execution-prompts/k2-n1-isolated-new-session-read-only.txt`
- N1 hash：`b19d41bf5e37cd41af5630cd71241f729e576ed4409574b10594c56e2d359833`
- N2 prompt：`tmp/stage-k-k2-real-execution-prompts/k2-n2-isolated-new-session-workspace-write.txt`
- N2 hash：`3ff79f634ab4eaaf341878e62e0d8542d39b4c1d4f9cee67d69ba823849ebead`
- Prompt 文件只用于后续受控真实执行点；本轮未发送 prompt。

### 3.6 K2 execution point harness

- Rust 新增 K2 R1/R2/N1/N2 execution point config、prompt `include_str!`、hash 校验和 prepare / decision / Phase A / Phase B input helper。
- R1/R2 走 resume Phase B helper；N1/N2 走 new-session Phase B helper。
- 默认测试使用 fake runner / no-real path，验证 Product Command chain、duplicate 阻断、N1 read-only empty allowed roots、N2 workspace-write allowed root、readback unavailable result_count `null`。
- R1/R2/N1/N2 真实执行测试已作为 `#[ignore]` 测试加入，必须同时满足测试名显式选择和对应环境变量确认；默认测试不会执行真实 Codex。
- `session_continuation_store` 的 H3 授权检查允许 read-only new-session `allowed_write_roots=[]`，以匹配 K2-N1 字段。
- 本轮没有新增裸 `Command::new("codex")`。

## 4. 验证

已通过：

- `cargo fmt -- --check`
- `cargo test --lib real_execution_command`：33 passed / 3 ignored
- `cargo test --lib session_continuation`：17 passed / 4 ignored
- `cargo test --lib codex_local_runner`：11 passed
- `npm run typecheck`
- `npm run test:offline-interaction`：14 passed
- `npm run build`：通过，仅既有 Vite chunk size warning

本轮 Phase B 普通入口接线后重新验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。

任务正文变更清空旧预览修补后再次验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。

K2 execution point harness 并入后验证：

- `cargo test --lib real_execution_command`：36 passed / 7 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib session_continuation`：17 passed / 4 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo fmt -- --check`：通过。
- `cargo test --lib`：323 passed / 14 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：14 passed。
- `npm run build`：通过，仅既有 Vite chunk size warning。

## 5. 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite preview / 截图工具。
- 已创建 Stage K 隔离测试项目 fixture README；未执行 N1/N2。
- 未写 `mario test`。
- 本 Level A 记录形成时未做 R1/R2/N1/N2 真实执行；后续已完成 R1，见独立 R1 evidence。
- R1/R2/N1/N2 的 ignored/env-gated 测试入口已存在，但本轮没有运行任何 ignored 测试。
- 未同步 `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `README.md`。

过程偏差：

- 有一次本地 `rg` 扫描命令把 Markdown 反引号放入 shell 双引号，触发了 `mario` 命令替换尝试并返回 `zsh:1: command not found: mario`。该命令没有执行真实 Codex、没有读写 `.codex`、没有产品代码路径副作用；随后已用单引号重新扫描且无旧口径命中。

## 6. 接受边界

可接受为：

- K2 Level A 契约、发送预览、prepare、用户确认、Phase A 非真实预检和 Phase B 普通入口接线完成。
- K2 R1/R2/N1/N2 execution point harness、ignored/env-gated 真实测试入口和 fake-runner safety tests 完成。

不接受为：

- K2 完成。
- K2 全部真实 Codex 执行完成。
- 真实 `resume` / `new session` 产品入口完成。
- R2/N1/N2 验收点完成。
- K3 工作流真实派发、K4 记忆捕获体验或 Stage K 完成。
