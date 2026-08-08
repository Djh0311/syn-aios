# M2a T1-R2 真机验收记录 · 2026-08-04

任务：`tasks/2026-08-03-syn-m2a-t1-r2-package-v1.md`（reference slice 真实接线，第二次返工）
验收环境：macOS (darwin arm64)，tauri-cli 2.11.4，cargo 1.95.0
被测代码：worktree `product-line-syn-fnd-002`，branch `syn-fnd-002-dev`（未提交前的工作区状态）
证据等级：**ISOLATED-RUNTIME**（真隔离 App + console invoke 打生产命令 + 读库核对）

## 隔离机制

```bash
HOME=/private/tmp/m2a-iso \
RUSTUP_HOME=/Users/yoyi/.rustup \
CARGO_HOME=/Users/yoyi/.cargo \
tauri dev --config /private/tmp/m2a-override.json   # {"app":{"withGlobalTauri":true}}
```

注：任务书原文 `HOME=/tmp/m2a-iso`；macOS `/tmp` 是 `/private/tmp` 的符号链接，storage-mode
校验要求 canonical 路径（`validate_clean_canonical_path`），故 HOME 用 canonical 形式，同一目录。

## 预置（operator 级手工准备，非测试代码）

- 空 workflow-state.v0.json（全空数组，通过 schema 校验）；
- `runtime-artifacts/storage-mode.v1.json`：db_primary_json_projection，canonical 路径；
- `runtime-artifacts/workbench.sqlite`：空库 + 全量 schema（从 `workbench_sqlite_schema.rs` /
  `workbench_sqlite_schema_m2.rs` 的 DDL 字面量机械提取，87+29 条，75 表）+ 两条 schema_migrations。
- 启动日志坐实：`storage mode=db_primary_json_projection db_path_hash=737cce04…`，无 blocked；
  启动后 DB `workflow_audit_events=1` 与 JSON `audit_events=1` 一致（启动 mode audit 双写），对账绿。

## store 文件绝对路径（指导线自查）

- JSON store：`/private/tmp/m2a-iso/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- SQLite store：`/private/tmp/m2a-iso/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/workbench.sqlite`
- App 日志：`/private/tmp/m2a-iso/Library/Logs/local.codex.governance.workbench/CodexGovernanceWorkbench.log`
- tauri dev 全程日志：`/tmp/m2a-tauri-dev.log`

## setup（console invoke，生产命令）

```js
window.__TAURI__.core.invoke('bootstrap_project_workflow', { request: { path: PR } })
window.__TAURI__.core.invoke('create_task_draft', { request: { project_root: PR, title: "M2A T1R2 验收任务", objective: "…", assigned_role: "codex-dev" } })
```

PR = `/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project`（dev 索引内既有项目）。
结果：workflow + 7 nodes + 1 work item（state=draft）落 DB 与 JSON 双写。

work_item_id = `work-item:workflow:users-yoyi-workspace-product-line-tmp-stage-j-j2-b-isolated-project:default:1785781587617`

## 三场景（console invoke → 生产命令 `update_work_item_state`）

| # | 操作 | console 返回 | DB 核对结果 | 判定 |
|---|---|---|---|---|
| ① 合法 | draft → ready_to_dispatch | `已推进工作项状态：草稿 -> 待派发` | command_receipts +1（COMMITTED, receipt_id=`15d4cf6c-85b4-4820-b1cc-bdde135f2c18`）；events +1（WorkItemStateUpdated）；audit_records +1（COMMITTED, `draft -> ready_to_dispatch`）；current_snapshots +1（revision=1）；work_items.state → `ready_to_dispatch`；workflow_audit_events +1 | ✅ |
| ② 同幂等键 | 与①完全相同的 invoke | `幂等重放：该状态推进命令已处理，返回既有 receipt，未新增任何变更`，audit_event_id=`idempotent-replay:15d4cf6c-…`（与①同一 receipt_id） | 全部表行数零变化（receipts 仍 1、events 仍 1、audit 仍 1、snapshots 仍 1） | ✅ |
| ③ 非法 | ready_to_dispatch → failed | 错误返回 `非法工作项状态跳转：ready_to_dispatch -> failed` | command_receipts +1（DENIED, error_code=`POLICY_DENIED`, receipt_id=`44475aea-…`）；audit_records +1（DENIED, `policy_denied: 非法工作项状态跳转…`）；events 仍 1（零 event mutation）；work_items.state 仍 `ready_to_dispatch`（业务状态零变化） | ✅ |

验收后终态行数：`command_receipts=2`（1 COMMITTED + 1 DENIED）、`events=1`、
`audit_records=2`、`current_snapshots=1`、`work_items=1(ready_to_dispatch)`、
`workflow_audit_events=4`（1 启动 + 1 bootstrap + 1 draft + 1 转移；②③零新增）。

JSON store 与 DB 一致（work_item state=ready_to_dispatch，audit_events=4）。

## 真实数据零接触

真实 store `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
mtime = 2026-08-01 01:35:38（早于本次会话 2026-08-04），隔离全程未触碰；App 自报路径全在
`/private/tmp/m2a-iso/**`（console 返回的 path/backup_path 坐实）。

## 接线判据 grep（A1/A2/A4）

- A1：`workflow_run_dispatch_entrypoints.rs:642` `repository.with_immediate_transaction(` 与
  `:647` `crate::m2_update_work_item_state::update_work_item_state_m2_with_transaction(`
  均为可执行调用语句（同一事务闭包内）。
- A2：`grep -n "M2 接线" workflow_run_dispatch_entrypoints.rs` 零命中（R1 装饰行已删）。
- A4：`grep -n "允许所有状态转换" m2_update_work_item_state.rs` 零命中（stub 已删）；
  policy 真闸 = `control_core::validate_work_item_state_transition`（链内 DB 状态判定）。
