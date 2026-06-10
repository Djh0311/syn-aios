# Unified Product Command Routing PCR7 Failure Stop Retry Product State v1

日期：2026-06-09

状态：已完成。

PCR7 是统一 Product Command Routing 的失败 / 停止 / 重试产品状态任务。它接在 PCR6 UI product linkage 之后，目标是让工作台能用统一命令读模型表达“为什么失败、是否被阻断、是否需要用户重新确认、是否只是读回不可用”，但不自动重试、不 kill 真实进程、不执行真实 Codex。

本任务默认 Level A：不授权真实 `codex exec` / `codex exec resume`，不发送真实 prompt，不读写 `/Users/yoyi/.codex`，不启动 Tauri / Browser / Chrome / 截图工具，不同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 0. 前置事实

- PCR0-PCR5 已把统一真实执行入口、sidecar、decision、Phase A no-op/fake runner 和 legacy sealing 收起来。
- PCR6 已完成 UI product linkage，并经复核线确认 P2 关闭。
- 当前 `RealExecutionProductCommandReadModel` 只有命令数、等待确认数、运行记录数、阻断数和最近 attempt 状态，不足以解释 failure / stop / retry。
- 现有系统已有 runtime attention / readback boundary / diagnostic summary，但它们不是统一 product command 的失败状态事实源。

## 1. 目标

PCR7 必须完成：

1. 后端 read model 新增失败 / 停止 / 重试产品状态摘要，覆盖用户拒绝、guard 阻断、诊断阻断、重复阻断、记忆过期阻断、超时、读回不可用、读回失败、runner 失败、用户请求停止、重试需要新确认。
2. 失败 / 停止 / 重试状态必须从现有 product command store 的 preview / decision / attempt 派生，不新增真实 runner。
3. 前端 Agent / Projects / Running / Right rail 能展示这些状态的用户可懂摘要。
4. 秘书只解释“为什么不能继续 / 下一步要查看什么 / 是否需要重新确认”，不得生成批准、派发、重试、stop、resume、restart action proposal。
5. `result_count=null` 继续显示为“未知 / 不可用”，不得显示为 0。
6. stop / retry 只能显示为产品状态或后续需要用户确认，不能触发真实进程控制。

## 2. 非目标

PCR7 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不新增 `Command::new("codex")`。
- 不新增真实 stop / kill / restart / retry runner。
- 不新增自由聊天输入框、真实执行按钮、重试按钮、停止按钮。
- 不修改 `workflow-state.v0.json` 顶层结构。
- 不迁移数据库，不新增 provider credential / model verification。
- 不做 PCR8 checkpoint 文档同步。
- 不做 PCR9 Level B 真实执行。
- 不做真实 Tauri / Browser / Chrome / 截图验收。

## 3. 允许修改文件

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，仅限测试挂载或导出必要 helper。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包。

默认不修改：

- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `tasks/README.md`
- `src-tauri/src/commands.rs`，除非确实需要暴露只读 helper；默认不新增 command。
- `src/lib/tauri.ts`
- `workflow-state.v0.json`

如实现中发现必须新增 Tauri command、改 workflow state schema、接真实 runner 或读写 `.codex`，必须停止并回交主管线。

## 4. 必须覆盖的产品状态

至少覆盖以下状态。命名可微调，但语义必须稳定，并在 Rust / TS / UI / test 中一致：

- `user_rejected`：用户拒绝或要求修改，不能继续执行。
- `blocked_by_guard`：guard / readiness 阻断。
- `blocked_by_diagnostics`：诊断降级阻断。
- `duplicate_blocked`：重复命令或重复 attempt 阻断。
- `blocked_stale_memory`：任务记忆包缺失或过期阻断。
- `timed_out`：执行或读回超时，不能说成已停止 agent。
- `readback_unavailable`：没有可用读回来源，结果数未知。
- `readback_failed`：读回尝试失败或不可信，结果数未知。
- `runner_failed`：runner / attempt 失败，但不得自动重试。
- `manual_stop_requested`：用户请求停止的产品状态；本任务不 kill 进程，只能表达“停止请求需要后续受控执行能力”。
- `retry_requires_new_user_confirmation`：重试需要新的 product command decision 或明确复用仍有效的 allowed-once decision；默认显示为需要重新确认。

建议新增：

```text
RealExecutionProductCommandFailureStopRetrySummary
RealExecutionProductCommandFailureStopRetryItem
```

并挂到：

```text
WorkbenchSnapshot.real_execution_product_commands.failure_stop_retry_summary
```

如开发线认为更简单，也可以命名为 `control_state_summary` / `recovery_state_summary`，但必须清楚表达 failure / stop / retry。

## 5. 派生规则

开发线应优先从现有 store 派生，不写新 sidecar：

- decisions 中 `rejected` / `request_changes` -> `user_rejected`。
- preview.blocked_reasons 中 `memory_packet_missing` / `memory_packet_stale` -> `blocked_stale_memory`。
- preview.blocked_reasons 中 `diagnostics_degraded` 或 diagnostics_summary.blocks_real_execution -> `blocked_by_diagnostics`。
- preview.blocked_reasons 中 `duplicate_active` 或 duplicate_scope.duplicate_blocked -> `duplicate_blocked`。
- preview.readiness / guard_preview 明确不可运行但不属于上述分类 -> `blocked_by_guard`。
- attempts.status `timed_out` 或 readback_summary.status `timed_out` / `readback_timed_out` -> `timed_out`。
- readback_summary.status `readback_unavailable` -> `readback_unavailable`，`result_count` 必须为 null。
- readback_summary.status `readback_failed` -> `readback_failed`，`result_count` 必须为 null。
- attempts.status `failed` / `failed_stub` / `runner_failed` 或 failure_reason 非空且非阻断类 -> `runner_failed`。
- attempts.warnings 或 future decision reason 如出现 manual stop request，仅表达 `manual_stop_requested`；如果当前 store 没有真实来源，可输出 count 0，不要造假。
- 任一 failed / timed_out / readback_failed / user_rejected 状态需要再次执行时，输出 `retry_requires_new_user_confirmation=true` 或等价 item；不要自动重试。

## 6. UI 要求

普通用户首屏可显示：

- `需要重新确认`
- `用户已拒绝`
- `被安全边界阻断`
- `被诊断阻断`
- `重复执行已阻断`
- `记忆包缺失或过期`
- `读回不可用`
- `读回失败`
- `执行超时`
- `运行失败`
- `停止请求需受控处理`

普通用户首屏不得显示：

- `runner_failed`、`manual_stop_requested` 等 raw enum。
- `retry 自动开始`、`已自动重试`、`已停止 agent`、`已重启 agent`。
- `readback 0 条`、`结果数：0`。
- `Command::new("codex")`、raw prompt、full transcript、secret / token / credential。

开发者详情可以显示 raw status、attempt id、runtime log ref、audit refs、sidecar path，但必须折叠。

## 7. 秘书规则

秘书可以：

- 解释失败 / 阻断 / 读回不可用的原因。
- 建议“查看统一执行链路”“查看诊断”“重新确认前先检查任务记忆包”。

秘书不得：

- 生成 approve / dispatch / retry / stop / restart / resume / send action proposal。
- 代替用户确认 retry。
- 把 user rejected 解释成失败或重试候选。

## 8. 测试要求

至少补齐：

- Rust：read model 派生能覆盖 11 类 PCR7 状态。
- Rust：readback unavailable / failed / timed_out 的 `result_count` 仍为 null。
- Rust：retry 状态只表达 `requires_new_user_confirmation`，不写 attempt、不调用 runner。
- 前端离线：Agent / Projects / Running / Right rail 能显示 PCR7 产品状态。
- 前端离线：秘书只生成查看建议，不生成 retry / stop / resume / dispatch / approve。
- 扫描：普通 UI 不出现自动重试 / 已停止 / 结果数 0 等误导文案。

## 9. 验证命令

在 `prototypes/productized-desktop-shell` 下运行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

如修改 Rust，必须运行：

```bash
cargo test --lib real_execution_command
cargo test --lib runtime_log
cargo test --lib diagnostic
cargo test --lib
cargo fmt -- --check
```

如没有修改 Rust，必须在回交中说明原因。PCR7 预计会修改 Rust，因此默认要跑 Rust 验证。

## 10. 扫描要求

```bash
rg -n '已自动重试|自动重试已完成|已停止 agent|已重启 agent|真实派发已完成|真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|readback 0 条|结果数：0|失败已自动恢复' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

```bash
rg -n 'Command::new\\(\"codex\"\\)|codex exec|codex exec resume' prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/types.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs
```

第二组如有命中，必须分类为既有历史边界 / 测试说明；PCR7 不允许新增真实 runner 调用。

## 11. 验收标准

PCR7 可接受为完成，当且仅当：

- 后端读模型能表达 failure / stop / retry 产品状态。
- UI 能用产品语言解释状态，不暴露 raw enum 到普通首屏。
- stop / retry 仍是状态和确认要求，不是真实进程控制。
- 不新增真实执行入口或 wrapper 调用。
- 秘书不生成可执行 retry / stop / resume / dispatch proposal。
- `result_count=null` 不显示为 0。
- 验证命令通过，扫描分类完成。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未同步权威入口。

## 12. 不接受条件

出现以下任一情况，PCR7 不接受：

- 执行真实 Codex 或发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 新增真实 stop / retry / restart / kill。
- 自动重试或自动恢复。
- user rejected 被显示为失败后可自动重试。
- readback unavailable / failed / timed_out 显示为 0 条结果。
- 修改 workflow state 顶层结构。
- 同步入口文档。

## 13. 分工

主管线：

- 写任务包。
- 派发开发线。
- 复核边界和最终验收。
- 不抢真实执行 Level B。

开发线：

- 实现后端读模型、前端显示和离线测试。
- 跑验证命令。
- 不执行真实 Codex，不启动 GUI。

复核线：

- 只读复核实现是否冒领真实执行、是否新增真实 stop/retry、是否把 result_count null 显示为 0。
- 不改文件，不跑真实 Codex，不启动 Browser / Tauri。

## 14. 回交格式

开发线完成后回交：

1. 修改文件。
2. 新增/变更的 read model 字段。
3. 11 类状态覆盖方式。
4. UI 和秘书口径。
5. 验证命令结果。
6. 扫描分类。
7. 不能声明完成事项。

## 15. 开发线执行结果草稿

状态：已完成。

本轮在 Level A 边界内补充 failure / stop / retry 产品状态读模型和普通 UI 展示：

- Rust read model 新增 `RealExecutionProductCommandFailureStopRetrySummary` / `RealExecutionProductCommandFailureStopRetryItem`，挂载到 `RealExecutionProductCommandReadModel.failure_stop_retry_summary`。
- 状态从 product command store 的 `decisions` / `previews` / `attempts` 派生，不新增真实 runner、stop、kill、restart、retry。
- 覆盖 11 类状态：`user_rejected`、`blocked_by_guard`、`blocked_by_diagnostics`、`duplicate_blocked`、`blocked_stale_memory`、`timed_out`、`readback_unavailable`、`readback_failed`、`runner_failed`、`manual_stop_requested`、`retry_requires_new_user_confirmation`。
- Agent / Projects / Running / Right rail 显示中文产品口径：需要重新确认、用户已拒绝、被安全边界阻断、被诊断阻断、重复执行已阻断、记忆包缺失或过期、读回不可用、读回失败、执行超时、运行失败、停止请求需受控处理。
- raw `kind`、`source_refs`、warnings、sidecar path 只进入折叠开发者详情；普通首屏不新增真实执行、停止、重试入口。
- Secretary 仅增加风险解释和查看建议，不生成批准、派发、重试、停止、重启、resume、send action proposal。
- `result_count=null` 在 UI 中显示为“未知 / 不可用”，不显示为 0。

验证结果：

- `cargo fmt -- --check`：通过。
- `cargo test --lib real_execution_command`：通过，28 passed。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib diagnostic`：通过，4 passed。
- `cargo test --lib`：通过，297 passed，5 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，offline interaction tests passed: 13。
- `npm run build`：通过；Vite 仅提示 chunk size warning。

扫描分类：

- 误导文案扫描命中均为既有负向断言或禁止词清单：`tests/offline-permission-dialog.test.tsx` 中的 forbidden text 断言、`canvasSurfaceBoundaries.ts` 中的边界禁止短语列表。PCR7 未新增普通 UI 误导文案。
- 真实 runner 扫描 `Command::new("codex")|codex exec|codex exec resume`：无命中。

边界确认：

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`，未读取 `.codex/plugins/cache` 技能/插件说明。
- 未启动 Browser / Chrome / Tauri / Vite dev / screenshot。
- 未修改 workflow state 顶层结构，未同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 16. 主管复核和复核线结论

主管线复核：

- 独立读取任务包、Rust read model、前端 UI、秘书只读模型和离线测试。
- 确认 `failure_stop_retry_summary` 只从 product command store 的 `decisions` / `previews` / `attempts` 派生，不新增 runner、sidecar、真实 stop / retry / kill / restart。
- 确认 11 类状态已覆盖，`readback_unavailable` / `readback_failed` / `timed_out` 的 `result_count` 保持 `null`，UI 显示为“未知 / 不可用”。
- 发现 Projects 普通首屏两处运行关注 raw status 展示，已最小修补为 `projectRuntimeAttentionValue(attention)`，用中文产品口径映射。

复核线结论：

- PCR7 通过。
- P0 / P1：无。
- P2：原 Projects 运行关注 raw status 已关闭，无新增 P2。
- 确认 wrapper 扫描无命中，Rust 只读派生、11 状态覆盖、`result_count=null`、秘书不生成执行 proposal、权威入口未同步结论成立。

主管线最终验证：

- `cargo fmt -- --check`：通过。
- `cargo test --lib real_execution_command`：通过，28 passed。
- `cargo test --lib`：通过，297 passed，5 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，offline interaction tests passed: 13。
- `npm run build`：通过；Vite 仅提示既有 chunk size warning。
- 指定 Rust 文件真实 Codex 命令扫描无命中。
- 误导文案扫描命中均分类为测试禁用词或 canvas 边界禁止短语常量。

最终结论：PCR7 可接受为完成。PCR7 不代表 PCR8 checkpoint、PCR9 Level B 真实执行、真实 Tauri / Browser 截图验收或完整统一真实 send / resume 产品化完成。
