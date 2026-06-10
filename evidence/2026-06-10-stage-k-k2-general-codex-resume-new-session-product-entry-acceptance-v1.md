# Stage K / K2 General Codex Resume And New Session Product Entry Acceptance Evidence v1

日期：2026-06-10

状态：K2 总验收通过，结论为 `accepted_with_deferred_items`。K2 接受为通用 `codex-local` `resume` / `new_session` 产品入口、Product Command 归口、权限确认、Phase A 非真实预检、Phase B 普通入口接线，以及 R1/R2/N1/N2 四个受控真实执行点完成；不接受为 K3 项目工作流真实编排、K4 记忆捕获体验、K5 failure/retry/control hardening、K6 dogfood 或 Stage K 完成。

## 1. 验收范围

K2 已完成：

- Agent 普通输入区可生成 K2 结构化发送预览，支持继续已有对话和新建对话预览。
- Product Command 链路覆盖 `preview -> prepare -> user decision -> Phase A no-op -> Phase B`。
- 前端 Phase B 入口不拼 CLI，统一调用 Tauri wrapper / Product Command。
- Rust `R1/R2/N1/N2` 真实执行测试均为 `#[ignore]` + env-gated + exact test name，默认测试不会触发真实 Codex。
- R1 `resume/read-only`：`mario test` 指定总指导 session 真实执行成功。
- R2 `resume/workspace-write`：`mario test` 指定开发线 session 真实执行成功，只写 allowed file。
- N1 `new_session/read-only`：Stage K 隔离项目真实新会话成功，项目文件不变。
- N2 `new_session/workspace-write`：Stage K 隔离项目真实新会话成功，只写 allowed file。

K2 不接受为：

- 任意目录无限制自由执行。
- K3 项目工作流真实派发闭环完成。
- K4 记忆捕获 / candidate UX 完成。
- K5 自动 retry / stop / restart 完成。
- K6 真实 Tauri dogfood 完成。
- planned adapters 真实接入。
- provider credential / model verification。
- FormalMemory 自动写入。
- Stage K 完成。

## 2. 四个真实执行点

| 执行点 | 操作 | 项目 | sandbox | 结果 |
| --- | --- | --- | --- | --- |
| K2-R1 | `resume` | `/Users/yoyi/Documents/mario test` | `read-only` | 通过，`result_count=1`，项目核心 hash 不变 |
| K2-R2 | `resume` | `/Users/yoyi/Documents/mario test` | `workspace-write` | 通过，`result_count=1`，只写 `.workbench/stage-k/k2/resume-workspace-write-probe.md` |
| K2-N1 | `new_session` | `test-fixtures/stage-k-isolated-project` | `read-only` | 通过，`result_count=1`，fixture 文件集不变 |
| K2-N2 | `new_session` | `test-fixtures/stage-k-isolated-project` | `workspace-write` | 通过，`result_count=1`，只写 `.workbench/stage-k/k2/new-session-write-probe.md` |

对应记录：

- `evidence/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1.md`
- `handoffs/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1-result.md`
- `evidence/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1.md`
- `handoffs/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1-result.md`
- `evidence/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1.md`
- `handoffs/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1-result.md`
- `evidence/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1.md`
- `handoffs/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1-result.md`

## 3. 真实执行副作用

R1/R2/N1/N2 均确实：

- 发送固定验收 prompt。
- 执行真实 Codex。
- 写入 `/Users/yoyi/.codex` 的 Codex 自身状态。
- 写入工作台自己的 Product Command sidecar、continuation sidecar、runtime log 和 audit refs。

写入项目文件情况：

- R1：未写 `mario test` 项目文件。
- R2：只写 `/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md`，hash 为 `49bff6c0a17e68b5abca2fadc60578527aedd886539741f24f04eb3ed167a8c0`。
- N1：未写 Stage K 隔离项目文件。
- N2：只写 `test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md`，hash 为 `603b54aac32b919db4f2b19758c8e0e361c75dc1802cbc9bc33b549dc89d0a07`。

## 4. Fresh Verify

本轮 K2 总验收后重新验证：

- `cargo test --lib real_execution_command`：通过，`36 passed; 7 ignored`
- `cargo test --lib session_continuation`：通过，`17 passed; 4 ignored`
- `cargo test --lib codex_local_runner`：通过，`11 passed`
- `cargo test --lib runtime_log`：通过，`6 passed`
- `cargo test --lib`：通过，`323 passed; 14 ignored`
- `cargo fmt -- --check`：通过
- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`
- `npm run build`：通过，仅保留既有 Vite chunk size warning

说明：默认测试不带 `--ignored`，不会再次触发真实 Codex。

## 5. 边界确认

- K2 产品入口不允许前端拼 `codex exec` / `codex exec resume`。
- K2 真实执行走 Product Command / `codex-local` runner，不走裸 CLI。
- prompt body 不持久化到 Product Command sidecar、continuation sidecar 或 runtime log；相关测试已断言。
- `readback_unavailable` / `readback_failed` / `timed_out` 等 unknown-result 状态仍保持 `result_count=null`。
- 没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 本轮 K2 总验收未启动 Tauri / Browser / Chrome / 截图工具。

## 6. UI 参考登记

用户提供的 Xuanji UI 研究资料已登记为 Stage K 后续 UI 信息层级参考：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/research/xuanji-ui-design-extraction-report.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/research/xuanji-ui-source-snapshot-2026-06-10/README.md`

登记边界：

- 只学习信息架构、布局和层级设计。
- 不采用 Xuanji 的视觉风格。
- 不复制 Xuanji 源码、命名、图标、品牌资产或具体实现。
- 合适落点为 Stage K 后续 UI checkpoint，尤其是 K5/K6 或单独 K-UI 信息层级任务。

## 7. 下一步

K2 完成后，下一步进入 K3：项目工作流真实自动化编排产品化。

K3 必须基于 K2 的 Product Command / Phase B / runtime / audit / readback 能力继续推进，但不能把 K2 单次 `resume/new_session` 探针直接冒充项目工作流闭环完成。
