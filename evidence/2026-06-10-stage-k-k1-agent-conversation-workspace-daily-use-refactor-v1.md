# Stage K / K1 Agent Conversation Workspace Daily-Use Refactor Evidence v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_deferred_items`。本文记录 K1 智能体对话页日常可用重构的代码事实、验证结果和边界。K1 未授权真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未改 Rust 后端、runner、Product Command 语义、workflow state 或任何 sidecar schema。

## 1. 任务包

- `tasks/2026-06-10-stage-k-k1-agent-conversation-workspace-daily-use-refactor-v1.md`

## 2. 改动文件

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 3. 代码事实

### 3.1 普通层对话工作区

- `AgentView` 普通标题从 `智 能 体` 收敛为 `智能体`，描述为“选择项目和对话，继续处理任务。”
- `agent-conversation-bar` 继续作为普通选择条，包含项目选择、对话选择、状态说明。
- 状态说明改为：输入任务后先生成确认材料，确认项目、权限和记忆影响，再进入执行。
- 新增 `新建对话` 占位按钮，保持 disabled，并明确“下一步由 K2 接入真实新会话”，不暗示 K1 已可真实创建 session。
- `agent-chat-composer` 仍只生成发送预览 / 确认材料，不直接发送 prompt。

### 3.2 桌面滚动和空间治理

- 新增 Stage K / K1 桌面样式覆盖块。
- `.stage.agent-stage` 在桌面尺寸下固定 `overflow: hidden; padding: 0;`，避免智能体页外层上下滚动失控。
- `agent-session-shell` 固定为会话列表 + 对话区布局，只有会话列表和消息流内部滚动。
- `chat-stream` 和 `agent-chat-composer` 不再受 `920px` 固定窄宽限制，减少右侧对话区留白。
- 复核线 P2 提醒开发者详情打开后可能被外层裁切；已补 `.agent-boundary-details[open]` 的 `max-height` 和内部滚动。
- 保持桌面优先；本轮没有做手机端 UI。

### 3.3 开发者材料边界

- 既有 `CodexControlEntryPanel`、统一执行状态、adapter/provider、session continuation、H2/H3、runtime attention、diagnostics、operation boundary 等仍保留在默认收起的 `agent-boundary-details` 开发者详情中。
- 本轮没有删除开发者边界证据，也没有把内部状态升级成普通操作入口。

## 4. 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 14`
- `npm run build`：通过，仅保留既有 Vite chunk size warning

扫描：

- 普通层误导文案扫描命中主要集中在 `AgentView.tsx` 的开发者详情面板、既有 `J1_DEFAULT_DENIED_PATHS`、既有测试 fixture 和边界测试断言；K1 新增普通层没有新增真实执行、Phase B、`.codex` 写入或 Product Command 可执行承诺。
- 敏感路径扫描命中既有开发者详情边界文案、`J1_DEFAULT_DENIED_PATHS`、任务包禁止项和测试 fixture；K1 未新增读取 `/Users/yoyi/.codex`、secret、token、`.env`、keychain、OAuth、provider credential 或 full transcript 的代码路径。

## 5. 未执行

- 未运行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite preview / 截图工具。
- 未做真实 Tauri 截图验收；该项仍交给 K6 dogfood。
- 未改 Rust 后端。
- 未同步 `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `README.md`；按 K0 规则，K1 默认不滚动同步全部入口。

## 6. 接受边界

复核结论：

- 复核线只读审查无 P0/P1，允许 K1 收口为前端 UI-only checkpoint。
- P2 已修补：开发者详情打开后提供内部滚动；任务包状态已从进行中改为已完成。

可接受为：

- K1 前端产品化切片完成：智能体页普通层更接近日常对话工作区，桌面外层滚动和过窄对话区得到修补，内部材料继续后撤到开发者详情。

不接受为：

- K2 通用真实 `resume / new session` 产品入口完成。
- 真实 Codex 已执行或 prompt 已发送。
- 真实新会话已创建。
- `.codex` 已获新读写授权。
- 工作流真实派发、记忆捕获体验、FormalMemory 写入、K6 Tauri dogfood 或 Stage K 完成。
