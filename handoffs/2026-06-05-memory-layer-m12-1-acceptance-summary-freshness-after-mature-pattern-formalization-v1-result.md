# Handoff：Memory Layer M12.1 Acceptance Summary Freshness After Mature Pattern Formalization v1

日期：2026-06-05

## 本轮结果

M12.1 已完成。用户确认 mature pattern candidate 正式化后，`record_mature_pattern_decision` 当次返回的 M1-M12 `acceptance_summary` 已使用写入后的 fresh formal memory store。

核心修补：

- confirm 写入 formal mature pattern memory 后重新 load formal store。
- `build_acceptance_summary(...)` 使用 fresh formal store。
- reject / quarantine / request changes 不写 formal store，summary 仍基于原 store。

## 接受为什么

接受为 M12.1 完成，因为：

- `formal_memory` gate 在用户确认写入第一条 formal mature pattern memory 后返回 `passed`。
- gate evidence 能看到 `record 1 / version 1 / audit 1`。
- `task_packet` gate 在同一次返回中变为 `passed`，且没有 `blocking_reason`。
- reject 路径仍返回 `formal_memory_output: None`，formal gate 为 `blocked`，evidence 保持 `record 0 / version 0 / audit 0`。
- quarantine 路径仍不写 formal store。
- `cargo test --lib` 和相关筛选测试通过。

## 不接受为什么

本轮不接受为：

- M13 最终权威验收完成。
- M12 真实窗口 / 截图验收完成。
- 成熟模式新能力新增。
- UI、前端读模型、Tauri command 或 sidecar 新增。
- mature pattern candidate 派生规则、用户确认 guard 或 task packet recall 选择逻辑变更。
- 真实 worker / Codex 已执行。

## 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- `tasks/README.md`
- `evidence/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1-result.md`

## 验证

通过：

- `cargo test --lib mature_pattern`，5 passed
- `cargo test --lib memory_cluster`，2 passed
- `cargo test --lib formal_memory`，29 passed
- `cargo test --lib task_memory_packet`，10 passed
- `cargo test --lib`，221 passed / 1 ignored
- `rustfmt --check src/mature_pattern_governance.rs src/lib.rs`
- `npm run typecheck`

说明：

- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- 因本轮未改前端，未跑 `npm run test:offline-interaction`、`npm run build` 或真实窗口 / 截图验收。

## M13 后续

M13 可以继续拆分 / 执行中间版本记忆系统最终权威验收。

M13 仍需要注意：

- M12.1 只修补 acceptance summary freshness，不是最终验收结论。
- M12 真实窗口 / 截图验收缺口仍存在。
- M1-M12 gate 摘要是 M13 的输入材料之一，不替代 M13。
- mature pattern candidate、cluster report、maintenance report、relation candidate、observation、knowledge hit、LLM summary 或 graph/index report 仍不能绕过正式记忆状态机。

继续保持：

- 不执行真实 worker / Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不把跨项目主题报告直接变成 worker 可用事实。
