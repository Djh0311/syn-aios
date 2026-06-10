# Stage K / K1 Agent Conversation Workspace Daily-Use Refactor v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_deferred_items`。本文是 Stage K 的 K1 任务包，用于把智能体页从“控制中心 / 边界面板集合”收敛为“日常可用 Codex 对话工作区”。K1 是前端产品化任务，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不改后端执行语义，不改 workflow state / sidecar schema。

## 0. 全局主管理解

已知事实：

- K0 已完成并冻结 Stage K 目标：把 Stage J checkpoint 推进为日常可用工作台。
- Stage J / J5 已做过首轮智能体页信息层级修补，但当前 `AgentView.tsx` 仍保留大量 J1/H2/H3/adapter/provider/readback/runtime/internal 面板，只是默认收进“开发者详情”。
- 用户当前明确要求智能体页像 Codex 对话界面：选择项目、选择对话、显示对话框、输入任务即可开始；不要上下滚动失控，不要像控制中心。
- K1 只能解决普通 UI 层级和对话体验，不能把 K2 的真实 `resume / new session` 产品入口提前实现。

假设：

- 本轮只改桌面 UI，不做手机端适配。
- 普通层保留“生成发送预览”入口，但它仍只打开开发者详情 / 后续确认材料，不发送 prompt。
- 开发者 / 内部材料可以保留在页面下方折叠区或设置开发者区，但不能占据普通对话工作区。

## 1. 权威依据

必须服从：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

参考事实：

- Stage J / J5 task / evidence / handoff。
- Stage J / J1-A、J1-B、J2、J3、J4、J6 task / evidence / handoff。
- PCR10、H2/H3/H4/H5 真实执行归口和 deferred 边界。

## 2. 接受范围

K1 可接受为：

- 智能体页普通层成为对话工作区：项目选择、对话选择、会话列表、消息区、输入框、发送前确认说明和当前状态清晰可见。
- 页面主区域固定在桌面窗口内，外层不再上下滚动失控；会话列表和消息流分别在自己的容器内滚动。
- 普通用户层不默认展示 Product Command、runtime log refs、audit refs、readback enum、sidecar、store revision、raw ids、H/J/PCR 阶段术语、provider / adapter 长边界或 rollout path。
- 开发者详情仍可承接 Product Command、adapter/provider、session continuation、runtime/readback、diagnostics 等材料，默认不打扰普通对话。
- `result_count=null` 继续显示为未知 / 不可用，不显示成 0。
- UI 文案全中文，保留必要英文产品名：Codex、Skill、Harness、runner 等可用英文。

K1 不接受为：

- K2 通用真实 `resume / new session` 产品入口完成。
- 真实 Codex 已执行、prompt 已发送、`.codex` 已读写。
- Product Command 后端语义变更。
- 工作流真实派发、记忆捕获、FormalMemory 写入或 Stage K 完成。
- 真实 Tauri 全量验收完成。

## 3. UI 显示边界确认

本任务改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端可见 UI。
- [x] 改读模型摘要或状态显示。
- [ ] 改 Tauri command / 后端执行语义。

普通层允许显示：

- 项目选择。
- 对话选择 / 新建对话占位入口。
- 会话列表、搜索、读取状态。
- 对话消息、读取失败 / 读取中 / 无可显示对话。
- 任务输入框。
- 发送前确认说明：项目、对话、权限、记忆影响需要确认。
- 当前状态：可发送、等待选择、等待读取、读取失败、不可发送。

普通层禁止默认显示：

- `Product Command`
- `runtime log`
- `audit refs`
- `readback enum`
- `sidecar`
- `store revision`
- `rollout path`
- `H2 / H3 / J1 / PCR`
- `adapter/provider boundary`
- raw command / raw id / raw path 长串

开发者详情允许显示：

- 统一命令读模型。
- adapter / provider / operation boundary。
- session continuation / runtime / readback / diagnostics。
- 会话来源路径和过程事件。

## 4. 修改范围

允许修改：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 如测试需要，可修改前端纯类型 / fixture。

默认不修改：

- `src-tauri/**`
- `workflow-state.v0.json`
- Product Command / runner / continuation / runtime log / memory store schema
- 权威入口文档，除非 K1 收口时主管线决定登记 checkpoint

## 5. 实施步骤

1. 任务包冻结：写明 K1 只改 UI 信息层级，不授权真实执行。
2. Agent 页结构调整：把普通对话区整理为三段：顶部选择条、左侧会话列表、右侧消息与输入区。
3. 滚动治理：外层固定高度，列表 / 消息流内部滚动；输入框固定在消息区底部。
4. 文案清理：普通层移除阶段号、Product Command、sidecar、store revision、rollout path、adapter/provider 长边界。
5. 开发者详情保留：原内部面板继续默认折叠，不删执行边界证据。
6. 测试补齐：离线测试覆盖普通层主文案、开发者详情默认后撤、禁止误导文案。
7. 验证和复核：跑前端验证，交复核线只读审查。

## 6. 验收命令

必须跑：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

建议扫描：

- 普通层误导文案扫描：`Product Command|runtime log|audit refs|sidecar|store revision|H2|H3|J1|PCR|rollout path`
- 敏感路径扫描：确认 K1 未新增 `/Users/yoyi/.codex`、secret、token、`.env`、keychain、OAuth、provider credential 读取路径。

真实 Tauri：

- K1 若能启动真实 Tauri，可补一张智能体页截图。
- 若因权限 / 端口 / 工具限制无法截图，必须记录为 K6 dogfood deferred，不可冒领。

## 7. 回交要求

开发线回交必须包含：

- 改动文件。
- 普通层现在显示哪些内容。
- 哪些内部材料被后撤到开发者详情。
- 验证命令和结果。
- 边界确认。
- 未完成项。

复核线只读审查：

- 不改代码。
- 不启动 Tauri / Browser / Chrome。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 返回 P0/P1/P2 和是否允许 K1 收口。
