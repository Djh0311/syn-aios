# Root Treatment R0 Shape Gate, Task Template, And Governance Package Type v1 Result

日期：2026-06-10

## 结论

R0 本轮可接受为：workbench shape gate、任务包“形状影响”必填节、治理任务包类型、解冻后 `1:3` 治理配额和 evidence / handoff commit hash 要求已建立。

不接受为：R1 完成、R2 `lib.rs` 解体完成、R3 SQLite 迁移完成、R4 前端瘦身完成、Stage L 恢复、K3-B1 retry、K3-B2 或 backlog 功能解冻。

## 已完成

- 新增 `scripts/harness/workbench-shape-gate.js`。
- 更新 `TASK_TEMPLATE.md`。
- 新增 `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`。
- 新增 R0 evidence：`evidence/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1.md`。
- 按主管追加边界，未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## Shape Gate 摘要

baseline / check 均通过：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 12
```

关键指标：

- `lib.rs`：25,925 lines，R0 水位线 25,925。
- Tauri commands：96 total，0 in `lib.rs`。
- sidecar JSON kinds：14 detected，0 unknown。
- ratchet files：12。
- gate script：407 lines。

## 模板摘要

`TASK_TEMPLATE.md` 已要求每个任务包填写：

- 新增代码落点。
- 是否触碰棘轮文件。
- 预计行数变化。
- 是否新增 Tauri command。
- 是否新增 sidecar JSON。
- 是否需要 shape gate 豁免和 decision。
- 本任务基线 commit / 完成 commit。

治理任务包验收口径已写入模板：行为不变 + 形状指标改善 + evidence 记录前后指标。

解冻后配额已写入模板和 R0 说明文档：每 3 个功能任务包至少配 1 个治理任务包；例外必须写 `decisions/**`。

## Commit 记录

- R-Preflight baseline：`ed01c6f281e3fd7a38548da948046e8366cc368d`
- R-Preflight 收口：`b409ab92d36b44f63911a4f12b057e5577f8aeb5`
- 本线实际 start commit：`a40b7b56ab949cd26b145ba0eccf9f3921886ea0`
- R0 completion commit：未创建。最终 `git status --short` 发现共享工作树里存在非 R0 变更 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`，按主管追加边界不得硬提交。

## 验证

已运行：

- `node --check scripts/harness/workbench-shape-gate.js`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，0 errors / 0 warnings。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。

Fresh verify：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- `git status --short`

结果：

- baseline：通过，0 errors / 0 warnings。
- check：通过，0 errors / 0 warnings。
- `git diff --check`：通过，无输出。
- `git status --short`：包含 R0 文件，同时包含非 R0 文件 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`。

提交状态：

- 未提交。
- 原因：共享工作树已有非本线 R1 范围改动，R0 线按主管追加边界不做部分提交。

未跑 `npm` / `cargo`：本轮未改产品 Rust / TS / TSX 业务代码。

## 边界

- 未触碰禁止项。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未改产品业务逻辑、workflow state schema、真实 runner、sidecar store 或 SQLite。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：sidecar 扫描仍是源码字符串级 baseline，动态拼接路径需后续收紧或由 R3 SQLite 收口消化。
- P2：command 总数增加先作为 warning；当前硬规则是新增 command 不得进入 `lib.rs`。
