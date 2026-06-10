# Control Core Command Convergence v1 Result

日期：2026-06-01

## 本轮完成

完成 `tasks/2026-06-01-control-core-command-convergence-v1.md` 的保守切片。

已完成：

- 新增后端控制核心 helper：`src-tauri/src/control_core.rs`。
- 工作项状态推进接入控制核心状态表。
- 派发准备现在必须是 `ready_to_dispatch`，草稿态不能写 prepared dispatch。
- 派发启动、完成、失败写回接入控制核心状态校验。
- 总指导回收和离线总指导回收接入控制核心校验。
- 离线角色派发和离线角色回传接入控制核心校验。
- 工作流机器启动状态和最终收口状态接入控制核心校验。
- 新增 `record_workflow_permission_decision` 后端命令，记录权限请求批准/拒绝并追加 audit。
- 前端 `record-permission-decision` pending action 改为调用后端命令。
- 黑板候选只做控制核心边界：允许 pending/rejected 作为边界语义，直接升级正式事实、正式记忆或 workflow state 会被拒绝；本轮不持久写入黑板确认状态。

## 不接受为

不接受为：

- 控制核心最终版。
- 事件账本完整迁移。
- 黑板可自由写入完成。
- 黑板候选持久审批流完成。
- 正式记忆存储完成。
- 秘书能力完成。
- 真实业务自动编排完成。
- 真实 Codex 执行验证完成。

## 改动文件

| 文件 | 内容 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs` | 新增控制核心校验 helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 接入控制核心校验，新增权限确认落账函数和测试。 |
| `prototypes/productized-desktop-shell/src-tauri/src/types.rs` | 新增 `WorkflowPermissionDecisionRequest`。 |
| `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` | 新增权限确认 Tauri command 包装。 |
| `prototypes/productized-desktop-shell/src/lib/tauri.ts` | 新增 `recordWorkflowPermissionDecision`。 |
| `prototypes/productized-desktop-shell/src/App.tsx` | 权限确认 pending action 改走后端命令。 |
| `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` | 更新权限确认边界文案。 |
| `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx` | 更新权限确认确认弹层文案。 |
| `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` | 更新离线权限确认预期。 |
| `CURRENT.md` | 标记本任务完成，更新下一步建议。 |
| `tasks/README.md` | 任务队列改为暂无待派发任务，记录本任务完成。 |
| `evidence/2026-06-01-control-core-command-convergence-v1.md` | 新增执行证据。 |

## 验证结果

全部通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/control_core.rs`
- `cargo test --lib`

结果摘要：

- 离线交互测试：`offline interaction tests passed: 2`。
- Rust：85 passed、1 ignored。
- Vite 仍有既有 chunk 大小 warning。
- Rust 仍有既有 warning：`JsonRpcError::invalid_params` 未使用。

## 手动测试清单

在应用里可以这样测，不需要执行真实 Codex：

1. 打开项目页，进入一个有工作流的项目。
2. 找到项目工作流主入口，确认项目页仍是主入口，独立 Canvas 不作为事实源。
3. 找到“权限请求队列”里 pending 的权限请求，点击“批准”或“拒绝”。
4. 在确认弹层里检查文案：应说明“通过控制核心写入工作台自己的 workflow state，并追加 audit event”，且仍写明不启动 Codex、不 resume、不写 `/Users/yoyi/.codex`。
5. 确认后，权限请求应从 pending 变成 approved/rejected，审计面板或 workflow audit 里应能看到 `workflow_permission_decision_recorded`。
6. 尝试对非 pending 权限请求再次批准/拒绝，后端应拒绝。
7. 在草稿态 work item 上尝试准备节点派发，后端应拒绝“控制核心已拒绝准备派发”；只有进入 `ready_to_dispatch` 后才能写 prepared dispatch。
8. 查看项目黑板，黑板条目仍是候选状态，不应出现“已正式写入事实/记忆”的 UI 结果。

## 仍然存在的架构风险

- `lib.rs` 仍聚合大量状态读写、审计、适配器和测试代码。控制核心 helper 是收口入口，不是最终模块边界。
- 权限确认现在只记录权限请求结论，不自动推进 `waiting_for_permission` 工作项；是否推进需要下一轮产品规则确认。
- 黑板候选没有持久确认/拒绝状态，当前只是 helper 和测试证明直接升级会被拒绝。
- 读回统计 `read_workflow_node_dispatch_result` 仍是状态写入命令，但本轮只梳理，没有接入更细的控制核心 helper。

## 下一步建议

建议下一步先别扩大到秘书或记忆真实存储。

可选下一步：

- 控制核心第二切片：把状态写入、审计事件构造、读模型派生继续从 `lib.rs` 分层拆出。
- 黑板 D-followup：先设计黑板候选持久确认状态的 schema 和迁移计划，再做写入命令。
- 权限流 followup：明确权限确认后是否允许控制核心推进 `waiting_for_permission -> running/failed/cancelled`。
