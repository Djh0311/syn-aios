# M5R04 持久 Project Supervisor 报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R04
- 结果：`M5R04_PERSISTENT_PROJECT_SUPERVISOR=PASS`（服务层；AppState/UI 在 M5R07）
- 合同：`docs/contracts/m5-persistent-project-supervisor-v1.md`

## 证据

- 重启恢复同一 binding
- 两项目不串会话
- chat/read 零 Proposal / Grant / spawn
- 未批准 Proposal 不能 dispatch

`cargo test --lib --offline -- m5_`：70 passed。

## 边界

未把 Supervisor 装进普通产品 `AppState` / Tauri command；那是 M5R07 与现有项目壳接线的范围。本包提供非 `#[cfg(test)]` 的 `open_or_resume_supervisor` / `handle_supervisor_action` / `authorize_and_dispatch_from_supervisor` 作为 production caller。
