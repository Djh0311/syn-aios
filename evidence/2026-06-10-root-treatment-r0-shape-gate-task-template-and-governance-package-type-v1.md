# Root Treatment R0 Shape Gate, Task Template, And Governance Package Type v1

日期：2026-06-10

## 结论

R0 本轮完成：

- 新增通用 workbench shape gate：`scripts/harness/workbench-shape-gate.js`。
- 更新 `TASK_TEMPLATE.md`，新增“形状影响”必填节、治理任务包类型、解冻后 `1:3` 治理配额和 commit hash 回传要求。
- 新增 R0 制度说明：`docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口文档同步按主管追加边界留给主管线统一处理。

R0 可接受为制度前置上线，不接受为 R1、R2、R3、R4、R5、Stage L 恢复、K3-B1 retry、K3-B2 或 backlog 功能解冻。

## Commit 记录

- R-Preflight baseline commit：`ed01c6f281e3fd7a38548da948046e8366cc368d`
- R-Preflight 收口 commit：`b409ab92d36b44f63911a4f12b057e5577f8aeb5`
- R0 任务包创建 / 本线实际 start commit：`a40b7b56ab949cd26b145ba0eccf9f3921886ea0`
- R0 completion commit：未创建。原因是最终 `git status --short` 发现共享工作树里存在非 R0 变更 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`，按主管追加边界不得硬提交。

## Shape Gate Baseline 摘要

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
```

结果：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 12
Git HEAD: a40b7b56ab949cd26b145ba0eccf9f3921886ea0
```

关键指标：

| 指标 | 当前值 |
| --- | ---: |
| `src-tauri/src/lib.rs` | 25,925 lines |
| `real_execution_command.rs` | 8,763 lines |
| `ProjectsView.tsx` | 6,069 lines |
| `AgentView.tsx` | 3,365 lines |
| `src-tauri/src/types.rs` | 5,386 lines |
| `src/lib/types.ts` | 5,149 lines |
| `styles.css` | 8,464 lines |
| `offline-permission-dialog.test.tsx` | 9,369 lines |
| Tauri commands | 96 total |
| `lib.rs` 内 `#[tauri::command]` | 0 |
| sidecar JSON kinds | 14 detected / 0 unknown |
| ratchet files | 12 |
| gate script | 407 lines |

当前 ratchet waterline：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：25,925
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`：9,369
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`：8,763
- `prototypes/productized-desktop-shell/src/styles.css`：8,464
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`：6,069
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`：5,386
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`：5,237
- `prototypes/productized-desktop-shell/src/lib/types.ts`：5,149
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`：5,059
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`：3,365
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`：2,429
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`：2,050

## Shape Gate Check 摘要

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 12
```

当前检查覆盖：

- `lib.rs` 行数不得高于 25,925。
- ratchet list 文件只降不升。
- `lib.rs` 内 `#[tauri::command]` 必须保持 0。
- Tauri command 总数超过 96 时产生 warning。
- 新增 / 未登记 sidecar JSON kind 产生 error。
- 超过新文件上限但未进入 ratchet list 的 Rust / TS / TSX 文件产生 error。
- gate 脚本超过 500 行软上限时产生 warning。

## 模板与制度落点

`TASK_TEMPLATE.md` 已新增：

- `形状影响` 必填节。
- 任务类型字段：功能任务包 / 治理任务包 / 紧急缺陷 / spike。
- 新增代码落点、棘轮文件、预计行数变化、command、sidecar、豁免 decision、start/end commit 字段。
- 治理任务包规则：行为不变 + 形状指标改善 + evidence 记录前后指标。
- 解冻后治理配额：每 3 个功能任务包至少配 1 个治理任务包，例外必须写 `decisions/**`。
- 默认验证：shape gate baseline、shape gate check、`git diff --check`。

说明文档落点：

- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`

该文档记录 gate CLI、退出码、硬规则、P2、治理任务包类型、`1:3` 配额和 evidence / handoff commit hash 要求。

## 验证记录

已运行：

```bash
node --check scripts/harness/workbench-shape-gate.js
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：

- `node --check`：通过。
- baseline：通过，0 errors / 0 warnings。
- check：通过，0 errors / 0 warnings。

Fresh verify：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，0 errors / 0 warnings。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。
- `git diff --check`：通过，无输出。
- `git status --short`：发现 R0 范围文件和一个非 R0 共享工作树变更：

```text
 M TASK_TEMPLATE.md
 M prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs
?? docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md
?? evidence/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1.md
?? handoffs/2026-06-10-root-treatment-r0-shape-gate-task-template-and-governance-package-type-v1-result.md
?? scripts/harness/workbench-shape-gate.js
```

提交状态：

- 未提交。
- 阻断原因：`workflow_state_store.rs` 属于 R1 / 存储止血线范围，不属于 R0；共享工作树存在其他线程改动时，本线不得硬提交。

本轮未跑 `npm run build` / `cargo test --lib`，原因是 R0 未修改产品 Rust / TS / TSX 业务代码，只新增只读 Node gate 和制度文档 / 模板。

## 边界确认

- 未改产品业务逻辑。
- 未改 Rust / Tauri runner 真实执行语义。
- 未改 workflow state JSON schema。
- 未新增 sidecar store。
- 未迁移 SQLite。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / L1-L6。
- 未启动 K3-B1 retry 或 K3-B2。
- 未做 UI 布局重做、无限画布、UI 视觉反馈 MCP 工具或 backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：sidecar 扫描基于源码字符串和 R0 允许清单；可阻断常规新增 `*.vN.json` 名称，但动态拼接 sidecar 名称需在 R2/R3 前继续收紧或由 SQLite 迁移消化。
- P2：Tauri command 总数增加目前为 warning；R0 硬阻断范围先限定为“不得新增到 `lib.rs`”，R2 命令注册拆分后可进一步收紧 command surface 规则。
