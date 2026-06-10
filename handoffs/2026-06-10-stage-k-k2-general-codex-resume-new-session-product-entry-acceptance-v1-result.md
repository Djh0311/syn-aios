# Stage K / K2 General Codex Resume And New Session Product Entry Acceptance Handoff v1

日期：2026-06-10

结论：K2 已完成并收口为 `accepted_with_deferred_items`。K2 现在可接受为通用 `codex-local` `resume` / `new_session` 产品入口、Product Command 归口、权限确认、Phase A 非真实预检、Phase B 普通入口接线，以及 R1/R2/N1/N2 四个受控真实执行点完成。

关键结果：

- R1 `resume/read-only`：`mario test` 指定总指导 session 真实执行通过，`result_count=1`，项目核心 hash 不变。
- R2 `resume/workspace-write`：`mario test` 指定开发线 session 真实执行通过，只写 allowed file。
- N1 `new_session/read-only`：Stage K 隔离项目真实新会话通过，项目文件集不变。
- N2 `new_session/workspace-write`：Stage K 隔离项目真实新会话通过，只写 allowed file。
- 四个真实点均确实发送 prompt、执行真实 Codex 并写入 `/Users/yoyi/.codex`。
- K2 总验收 fresh verify 通过：Rust 默认测试、前端 typecheck/offline/build 和 `cargo fmt -- --check` 均通过。

证据：

- `evidence/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-acceptance-v1.md`
- `evidence/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1.md`
- `evidence/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1.md`
- `evidence/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1.md`
- `evidence/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1.md`

边界：

- K2 不接受为 K3 项目工作流真实编排、K4 记忆捕获体验、K5 failure/retry/control hardening、K6 dogfood 或 Stage K 完成。
- 不接受为任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 或 FormalMemory 自动写入。
- 本轮 K2 总验收未启动 Tauri / Browser / Chrome / 截图工具。

Xuanji UI 参考已登记：

- 只作为后续 UI 信息架构 / 布局 / 层级参考。
- 不采用其风格，不复制其源码、命名、图标、品牌资产或具体实现。
- 合适落点为 K5/K6 或单独 K-UI 信息层级任务。

下一步：

- 进入 K3：项目工作流真实自动化编排产品化。
- K3 应复用 K2 的 Product Command / Phase B / runtime / audit / readback 能力，把用户目标 -> run units -> 真实执行 -> readback -> 过程事实 -> 用户可读结果串成可重复闭环。
