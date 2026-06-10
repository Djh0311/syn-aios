# Unified Product Command Routing PCR6 UI Product Linkage v1

日期：2026-06-09

状态：已完成。

UI 线任务。本文用于在 PCR1-PCR5 已完成后，把统一 Product Command Routing 的 Level A 链路接入普通产品 UI：项目页、智能体页、运行中工作流页、权限弹层和秘书只读解释。PCR6 不授权真实 `codex exec` / `codex exec resume`，不发送真实 prompt，不读写 `/Users/yoyi/.codex`，不启动 Tauri / Browser / Chrome / 截图工具，不同步权威入口。

## 0. 前置事实

- PCR0 已冻结方向：真实执行必须归口统一 Product Command Routing。
- PCR1 已建立 `real-execution-product-commands.v1.json` 类型、store skeleton 和 `WorkbenchSnapshot.real_execution_product_commands` 读模型。
- PCR2 已完成 preview / prepare 服务；prepare 可以写 product-command sidecar，但不真实执行。
- PCR3 已完成 decision / confirmation 服务；approved 只代表用户授权决定，不代表已发送、运行中或已完成。
- PCR4 已完成 Phase A no-op / fake runner；可以写 product attempt、continuation、runtime log ref、audit ref、readback boundary，但 `runner_call_allowed=false`、`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- PCR5 已完成 legacy alias / 入口可达面收束；旧 workflow / machine / canvas 普通入口保持 blocked / sealed。
- 入口文档仍留到 PCR8 或 PCR10 checkpoint 同步；PCR6 不更新 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 1. 目标

PCR6 目标：

1. 在智能体页展示统一执行链路摘要：准备执行、等待确认、Phase A 已记录、已阻断、读回不可用 / 失败 / 超时。
2. 在项目工作流节点详情展示统一 product command 状态，不再把 legacy 派发记录误读为正式统一命令。
3. 在运行中工作流页展示 product command 计数和最新状态，和 runtime session attention 并列但不混同。
4. PermissionDialog 对 product command permission envelope 使用产品化文案，说明目标、风险、写入范围、`.codex` 边界、失败/读回策略。
5. 秘书只解释影响面、风险和下一步查看建议，不生成批准、真实派发、自动重试、resume 或 stop 动作。
6. 开发者信息，如 product command id、attempt id、runtime log ref、audit ref、sidecar path，进入折叠详情或开发者区，不铺在普通首屏。
7. 保持桌面端 UI 清晰；本任务不做手机端布局。

## 2. 非目标

PCR6 不做：

- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 不调用 Phase B real runner 或 H3-B real runner。
- 不新增 `Command::new("codex")`。
- 不新增真实执行自由控制台。
- 不新增自动重试 / stop / restart / kill 真实进程。
- 不修改 Rust runner、workflow state 顶层结构或 sidecar schema。
- 不把 PCR4 Phase A fake/no-op 包装成真实执行完成。
- 不做 PCR7 failure / stop / retry 状态扩展。
- 不做 PCR8 checkpoint 文档同步。
- 不做 PCR9 Level B。
- 不做真实 Tauri / Browser / Chrome / 截图验收。

## 3. 文件范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`，仅限新增/调整本任务 UI class。
- 本任务包。

默认不修改：

- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `tasks/README.md`
- `src-tauri/src/*`
- `src/lib/tauri.ts`
- `src/lib/types.ts`
- `workflow-state.v0.json`

如实现中发现必须改 TS 类型或后端读模型，暂停并回交主管线；不得自行扩大 PCR6 范围。

## 4. 产品信息层级

普通用户首屏显示：

- `准备执行`
- `等待确认`
- `已确认，等待 Phase A 记录`
- `Phase A 已记录`
- `已阻断`
- `读回不可用`
- `读回失败`
- `读回超时`
- `无统一执行命令`

普通用户首屏不显示：

- `H5 命令`
- `H6 真实执行状态`
- `product_command_id`
- `attempt_id`
- `runtime_log_ref`
- `audit_ref`
- `sidecar_path`
- raw log
- full prompt
- full transcript
- rollout
- secret / token / credential

开发者详情可以显示：

- product command id
- store revision
- latest attempt id
- runtime log refs
- audit refs
- sidecar path
- legacy entry status
- runner entry status
- Level B authorization boundary

## 5. 实现要求

### 5.1 统一执行链路派生 helper

建议在前端局部新增轻量 helper，优先放在使用文件内，只有复用明显时再抽到 `src/lib`：

- 输入：`WorkbenchSnapshot.real_execution_product_commands`。
- 输出：产品状态标签、tone、摘要、readback 文案、开发者详情字段。
- `pending_decision_count > 0` 显示为 `等待确认`。
- `running_attempt_count > 0` 只显示为 `Phase A / 运行记录可见`，不得说成真实 Codex 运行中，除非 read model 明确真实执行。
- `blocked_attempt_count > 0` 或 readiness / guard blocked 显示为 `已阻断`。
- `last_attempt_status` 有值时显示最近状态。
- `result_count=null` 显示为 `未知 / 不可用`，不能显示为 `0`。

### 5.2 智能体页

必须做：

- 将现有 `H6RealExecutionStatusPanel` 产品化命名为统一执行链路摘要，UI 文案不暴露 `H6`。
- 展示 `codex-local`、权限确认、Phase A no-op/fake、读回边界、runtime/audit 引用摘要。
- 保留“本页不触发新的 codex exec / resume”的边界说明。
- 内部 refs 放入 `<details>` 或开发者详情块。

不得做：

- 不新增“执行”“发送”“恢复”“重试”按钮。
- 不调用 `runRealExecutionProductCommandPhaseA`。
- 不调用 `confirmRealExecutionProductCommand`。
- 不把 `approved` 写成“已执行”。

### 5.3 项目页

必须做：

- 节点详情中的执行状态摘要改为“统一执行链路”口径。
- 清楚区分三类事实：
- `legacy 项目派发记录`：历史记录 / 已封口，不是统一产品命令。
- `product command read model`：统一命令的准备、确认、attempt、读回边界。
- `runtime session attention`：运行关注 / 卡住 / 需要用户处理。
- 普通首屏不出现 `H5/H6` 阶段名。

不得做：

- 不新增项目节点真实派发按钮。
- 不把 legacy dispatch 的 native thread id 当作统一 command id。
- 不把 worker 汇报 / observation / candidate 当成正式结果。

### 5.4 运行中工作流页

必须做：

- 新增统一执行命令摘要卡片：命令数、等待确认数、Phase A / attempt 数、阻断数、最近状态。
- 和项目工作流、智能体运行关注并列显示，文案上说明它们是不同事实源。
- readback unavailable / failed / timed_out 都显示未知或不可用，不显示为 0。

不得做：

- 不显示 raw runtime log。
- 不把 runtime log refs 展开成日志正文。
- 不新增 stop / retry / restart 操作。

### 5.5 PermissionDialog

必须做：

- 当 action 类型或 payload 表明是 product command / real execution confirmation 时，按钮文案必须明确，例如 `确认执行准备`、`确认记录授权`、`确认 Phase A 记录`。
- 风险说明必须包括：目标项目/会话、写入范围、`.codex` 边界、失败/超时/读回不可用不自动重试。
- fallback 不能恢复为 `允许一次`。

不得做：

- 不降低真实 Codex 强确认按钮文案。
- 不把 Phase A fake/no-op 说成真实执行已完成。

### 5.6 右侧栏 / 秘书只读模型

必须做：

- 右侧 `运行中` 可显示统一执行链路摘要，但不和通知/待办混成一个列表。
- 秘书可以生成“查看统一执行链路”“需要用户确认”“读回不可用需要人工判断”这类查看建议。
- 秘书不得生成批准、真实派发、自动重试、stop、restart、resume action proposal。

## 6. 测试要求

至少补齐或更新离线覆盖：

- 统一 product command read model 能在 Agent / Running / Projects 中显示普通产品状态。
- `result_count=null` 显示为 `未知 / 不可用`。
- `H5 命令`、`H6 真实执行状态`、`允许一次` 不出现在普通 UI 文案。
- Product command UI 不调用真实执行 wrapper。
- 秘书只生成查看建议，不生成批准/派发/重试动作。

## 7. 验证命令

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
npm run typecheck
npm run test:offline-interaction
npm run build
```

PCR6 默认不跑 Rust；如没有改 Rust，不需要 `cargo test`。如意外触碰 Rust，必须停止并回交主管线。

## 8. 扫描要求

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'H5 命令|H6 真实执行状态|允许一次|结果数：0|启动实验画布运行|已启动实验画布运行' src tests
```

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'runRealExecutionProductCommandPhaseA|confirmRealExecutionProductCommand|recordRealExecutionProductCommandDecision|prepareRealExecutionProductCommand' src/App.tsx src/views src/components
```

第二组必须无普通 UI 调用命中，除非任务包后续经主管线明确批准接入非真实 preview；本轮默认不调用 wrapper。

```bash
cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
rg -n 'product_command_id|attempt_id|runtime_log_ref|audit_ref|sidecar_path' src/views src/components
```

第三组如有命中，必须确认在开发者详情、折叠详情或非普通首屏，不得铺在普通用户首屏。

## 9. 验收标准

PCR6 可接受为完成，当且仅当：

- 普通 UI 能看懂统一执行链路状态。
- 项目页、智能体页、运行中工作流页不再用 `H5/H6` 阶段名作为普通用户标题。
- PermissionDialog 不出现 `允许一次` fallback。
- Product command UI 没有新增真实执行按钮或 wrapper 调用。
- 秘书只做解释和查看建议。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 通过。
- 扫描完成并分类。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未同步权威入口。

## 10. 不接受条件

出现以下任一情况，PCR6 不接受：

- 执行真实 Codex。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 新增真实执行 / resume / retry / stop 按钮。
- 普通 UI 将 `approved`、Phase A no-op、readback unavailable 说成真实执行完成。
- 普通 UI 将 `result_count=null` 显示为 0。
- 普通 UI 继续显示 `H5 命令` / `H6 真实执行状态`。
- 同步 `CURRENT.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、`tasks/README.md`。

## 11. 分工

主管线：

- 写任务包。
- 审核边界和最终验收。
- 不抢真实执行 Level B。

UI 开发线：

- 实现 Agent / Projects / Running / PermissionDialog / Secretary 的最小产品化 UI 修补。
- 跑前端验证。
- 不改 Rust。

复核线：

- 只读复核 UI 是否冒领真实执行、是否新增 wrapper 调用、是否仍暴露阶段名。
- 不改文件，不跑真实 Codex，不启动 Browser / Tauri。

## 12. 回交格式

开发线完成后中文回交：

1. 修改文件。
2. UI 状态口径。
3. 没有新增真实执行入口的证据。
4. 验证命令结果。
5. 扫描分类。
6. 不能声明完成事项。

## 13. PCR6 执行结果

状态：已完成。

本轮最小闭环：

- Agent 页普通展示改为“统一执行链路”，展示 `snapshot.real_execution_product_commands` 的命令数、等待确认数、受控记录数、阻断数、最近状态和读回未知 / 不可用边界；store / sidecar / runner 状态进入折叠开发者详情。主管线追加修补后，普通摘要已从 `codex-local / legacy 派发` 收敛为“本地适配器 / 历史派发”。
- Projects 节点详情改为“统一执行链路”口径，区分“旧派发记录”、“统一命令状态”、“运行关注”；sidecar / store / runner 状态进入折叠开发者详情。
- Running 页新增“统一执行命令”摘要卡，展示 `command_count`、`pending_decision_count`、`running_attempt_count`、`blocked_attempt_count`、`last_attempt_status`，并声明与项目工作流、智能体运行关注是不同事实源。
- Right rail 的 `running` 面板新增独立“统一执行链路”状态 pane，不混入通知 / 待办列表。
- Secretary 只新增统一执行链路风险和查看建议；action proposal 仍只包含打开类只读入口，不生成批准、派发、重试、stop、resume。
- PermissionDialog 未改逻辑；fallback 保持 `确认继续`，未恢复 `允许一次`。

验证结果：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 13`。
- `npm run build`：通过；执行的是生产构建 `vite build`，未启动 Vite dev server；仅保留既有 Vite chunk size warning。

扫描分类：

- `rg -n 'H5 命令|H6 真实执行状态|允许一次|结果数：0|启动实验画布运行|已启动实验画布运行' src tests`：仅命中 `tests/offline-permission-dialog.test.tsx` 中的禁止出现断言；`src` 单独扫描无命中。
- `rg -n 'runRealExecutionProductCommandPhaseA|confirmRealExecutionProductCommand|recordRealExecutionProductCommandDecision|prepareRealExecutionProductCommand' src/App.tsx src/views src/components`：无命中；普通 UI 未调用真实执行 / confirm / record / prepare wrapper。
- `rg -n 'product_command_id|attempt_id|runtime_log_ref|audit_ref|sidecar_path' src/views src/components`：命中均为开发者详情、审计引用、既有 attempt card 或运行日志摘要引用；本轮新增的 `sidecar_path` 仅在折叠开发者详情中展示。
- `rg -n 'H6RealExecutionStatusPanel|ProjectH6RealExecutionStateCard|h6-real-execution-panel|h6-project-execution-card|Phase A|product command read model|legacy 派发记录|legacy 目标会话|runtime attention|codex-local ·|legacy 派发' src tests/offline-permission-dialog.test.tsx`：`src` 无命中；仅剩 E6 测试说明里的 `runtime attention`，不是产品源码。

主管复核：

- 复核线第一次结论为“带 P2 通过”：无 P0/P1，P2 为普通 UI 仍有 `Phase A`、`product command read model`、`legacy`、`runtime attention` 等内部口径。
- 主管线已按 P2 做小范围修补：组件名从 `H6...` 收敛为统一执行命名，普通首屏改为“受控记录 / 统一命令状态 / 旧派发记录 / 运行关注 / 本地适配器 / 历史派发”等产品语言。
- 复核线二次结论：PCR6 P2 已关闭，无新的 P0/P1/P2，建议主管线将 PCR6 标记为已完成。

过程偏差记录：

- 在主管线纠偏前，曾读取 `.codex/plugins/cache` 下 Product Design 技能说明；纠偏后未继续读取任何技能 / 插件说明，未读写 `/Users/yoyi/.codex`，未启动 Browser / Chrome / Tauri / Vite dev，未执行真实 `codex exec` / `codex exec resume`。

不能声明完成事项：

- 本轮未做真实 Tauri / Browser / Chrome 截图验收。
- 本轮未接入 Level B，不代表真实 Codex 已执行。
- 本轮不代表 PCR7 failure / stop / retry 产品状态完成。
- 本轮不代表 PCR8 checkpoint 文档同步完成。
