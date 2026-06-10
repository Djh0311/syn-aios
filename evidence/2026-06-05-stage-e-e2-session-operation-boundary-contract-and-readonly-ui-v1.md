# Evidence：Stage E / E2 Session Operation Boundary Contract And Readonly UI v1

日期：2026-06-05

## 结论

已完成 `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`。

接受为：

- 阶段 E / E2 会话操作边界契约完成。
- 后端 `WorkbenchSnapshot.session_operations[]` 输出 `send_message`、`stop`、`restart`、`resume`、`export`、`delete`、`favorite` 七类操作边界。
- `codex-local` 和 Claude Code / OpenClaw / OpenCode / OpenCode-like planned adapters 都有机器可测的 per-adapter operation descriptor。
- 智能体页在既有入口内新增只读“会话操作边界”局部面板；没有新增一级入口、右侧顶级入口或项目页 tab。
- 秘书只读模型新增会话操作边界风险 / 查看建议；不生成发送、停止、重启、resume、导出、删除或收藏 action proposal。

不接受为：

- 会话中心真实发消息完成。
- 通用 `codex exec resume`、stop、restart 完成。
- 会话导出、删除、收藏完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- 外部模型或凭据管理完成。
- 运行日志、自动重试、取消恢复、运维诊断完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 `SessionOperationDescriptor`。
  - `WorkbenchSnapshot` 新增 `session_operations`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 新增 `derive_session_operation_descriptors` 和七类操作矩阵。
  - `build_snapshot_with_session_source` 从 `agent_adapters[]` 派生 `session_operations[]`。
  - 新增 Rust 单测 `session_operation_descriptors_cover_e2_boundary_matrix`。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增前端 `SessionOperationDescriptor` / status / risk 类型。
  - `WorkbenchSnapshot` 新增 `session_operations`。
- `prototypes/productized-desktop-shell/src/lib/sessionOperations.ts`
  - 新增前端纯读 fallback 派生器。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - `emptySnapshot` 和 `AgentView` 传参同步 `session_operations`。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - 新增只读“会话操作边界”局部面板。
  - 面板只显示状态、原因、未来前置和数据影响标记；不渲染操作按钮。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
  - 新增 `session_operation_boundary` 风险和 `inspect_session_operation_boundary` 查看建议。
  - 不新增可执行 action proposal。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增会话操作边界面板局部样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - fixture 新增 `session_operations`。
  - 新增 E2 离线场景，覆盖七类操作、不可执行状态、UI 无操作按钮和秘书不生成操作 proposal。

## 七类会话操作最终状态矩阵

`codex-local`：

| 操作 | E2 状态 | 风险 | 当前边界 |
| --- | --- | --- | --- |
| `send_message` | `requires_future_task` | `high` | 需要后续任务定义发送路径、用户确认、审计、readback 和失败处理；未来真实实现会写 Codex home 和工作台状态。 |
| `stop` | `blocked` | `high` | 缺少运行进程 registry、运行句柄、取消协议、超时和失败恢复。 |
| `restart` | `blocked` | `high` | restart 语义未定：新建会话、恢复旧会话或重跑任务需要后续决策。 |
| `resume` | `requires_future_task` | `high` | workflow dispatch 的受控 resume 属于项目工作流语境，不等于会话中心通用 resume。 |
| `export` | `planned` | `medium` | 需要导出格式、脱敏范围、目标位置、用户确认和审计；本轮不写导出文件。 |
| `delete` | `blocked_destructive` | `destructive` | 破坏性操作已阻断；需要备份、回滚、双确认、作用域、原生系统兼容和审计。 |
| `favorite` | `planned` | `low` | 需要工作台自有 metadata store、冲突策略和轻量审计；本轮不写 favorite store。 |

planned adapters：

| adapter | 操作覆盖 | E2 状态 |
| --- | --- | --- |
| `claude-code` | 同七类操作 | `send_message` / `stop` / `restart` / `resume` 为 `blocked`，`export` / `favorite` 为 `planned`，`delete` 为 `blocked_destructive`。 |
| `openclaw` | 同七类操作 | 同上；warning 包含 `planned_adapter_operation_not_available`。 |
| `opencode` | 同七类操作 | 同上；不声明真实命令、会话、凭据或模型访问。 |
| `opencode-like` | 同七类操作 | 同上；不显示可执行操作。 |

所有 operation descriptor 都包含：

- `session_operation_boundary_read_model_only`
- `no_session_operation_execution_in_e2`
- `no_codex_home_write_in_e2`

## 为什么 E2 没有实现真实操作

- 发消息和通用 resume 会写 Codex home，并需要 prompt 预览、用户确认、运行日志、readback 和失败处理；E2 未授权这些执行链路。
- stop 和 restart 需要运行句柄、进程 registry、取消协议、幂等审计和失败恢复；当前会话中心只有历史会话浏览，不是运行控制器。
- export 需要完整 transcript 范围、脱敏策略、目标位置和文件写入审计；E2 禁止读取完整 transcript 作为开发证据，也未授权导出 store。
- delete 是破坏性操作，需要备份、回滚、双确认和原生系统兼容；E2 明确阻断。
- favorite 需要工作台自有 metadata store；E2 禁止新增 favorite store 或 session operation sidecar。

## workflow dispatch 不等于会话中心发消息

现有 `execute-node-dispatch` / 受控 resume 属于项目工作流 / dispatch 语境：

- 它依赖项目 workflow state、节点绑定、任务包、权限确认和 readback。
- 它是高风险 workflow action，必须在对应工作流任务里确认。
- 它不是智能体会话中心里的通用聊天输入框，也不是任意会话的发送消息能力。

因此 E2 将 `resume` 标为 `requires_future_task`，并在 warning 中保留 `workflow_dispatch_is_not_session_center_resume`。

`reveal-rollout` 仍只是现有本机辅助定位动作，不计入 E2 七类会话操作。

## planned adapters 保持不可执行

- planned adapters 仍来自 E1 descriptor：`planned`、`not_implemented`、`not_configured`、`not_verified`。
- E2 对 planned adapters 只派生 operation boundary，不增加 `implemented_action_kinds`，不增加 adapter capability。
- planned adapter operation descriptor 的 `applies_to_session_state = planned_adapter_without_session_source`。
- planned adapter operation warning 包含 `planned_adapter_operation_not_available`。

## UI 证据

- 智能体页新增“会话操作边界”局部面板。
- 面板显示每个 adapter 的七类操作状态、不可用原因、未来前置、审计 / 数据影响标记。
- 面板没有消息输入框、prompt 编辑框，也没有发消息、停止、重启、resume、导出、删除或收藏按钮。
- 离线测试通过 `renderToStaticMarkup` 检查 `<button>` 文本集合，确认这些操作名称没有作为可点击按钮出现。
- 秘书只读模型只新增风险 / 建议，不新增会话操作 action proposal。

## 禁止文案扫描

命令：

```text
rg -n '已发送|已停止|已重启|已 resume|已导出|已删除|已收藏|Claude Code 已支持发送|OpenClaw 已支持会话控制|OpenCode 已接入会话操作|真实 Codex 已执行|自动派发已开始' prototypes/productized-desktop-shell/src
```

结果：

```text
prototypes/productized-desktop-shell/src/views/CanvasView.tsx:243:      onNotice("已停止实验运行。");
```

解释：

- 这是既有独立实验画布运行提示，不是智能体会话中心，也不是 E2 新增文案。
- 它不表示会话中心 stop 已实现，不涉及 `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite`。
- 本轮未改 `CanvasView.tsx`，避免顺手触碰 MCP / 可编辑画布运行逻辑。

## 验证命令

在 `prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 10`。
- `npm run build`：通过；仍有既有 Vite chunk size warning。

在 `prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib session_operation`：通过，1 passed。
- `cargo test --lib adapter_descriptor`：通过，2 passed。
- `cargo test --lib agent_adapter`：通过，2 passed。
- `cargo test --lib`：通过，222 passed，1 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`：通过。

备注：

- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning，本轮未处理。
- `npm run build` 保留既有 Vite chunk size warning，本轮未处理。

## 未做验收

- 未启动真实 Tauri 窗口。
- 未做真实窗口 / 截图验收。
- 因此本轮不接受为阶段 G 真实 Tauri 全面验收完成。

## 边界确认

本轮没有：

- 读写 `/Users/yoyi/.codex`。
- 读取 auth、token、`.env`、keychain、OAuth session、provider credential 或完整真实 transcript。
- 执行 `codex exec` 或 `codex exec resume`。
- 调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 调用外部模型 provider。
- 启动真实 worker、workflow machine 或 MCP canvas run。
- 新增 credential store、adapter sidecar、session operation sidecar、favorite store、export store 或数据库迁移。
- 修改 `workflow-state.v0.json` 顶层结构。
- 写正式事实、正式记忆或正式审计事件。
- 新增一级入口、右侧顶级入口或项目页 tab。
