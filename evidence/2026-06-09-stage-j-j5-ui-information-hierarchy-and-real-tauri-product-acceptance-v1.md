# Stage J / J5 UI Information Hierarchy And Real Tauri Product Acceptance Evidence v1

日期：2026-06-09

结论：`accepted_with_deferred_items`。

J5 接受为 UI 信息层级收束和真实 Tauri 关键截图探针完成：智能体页普通层已从“控制中心”收敛为项目 / 对话 / 对话流 / 任务输入的对话工作区；开发者 / 内部边界内容默认收进详情；左侧栏主入口保持 `项目 / 智能体 / 想法箱 / 知识库 / 记忆层 / Skill / Harness / 运行中工作流`，并沿用 inkwash 原型图标组。J5 不接受为 Stage J 完成、真实执行新增授权、自动 retry / stop / restart、planned adapters 真实接入、FormalMemory 自动写入或最终蓝图完整工作台完成。

## 1. 范围

任务包：

- `tasks/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`

主要代码事实：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

真实 Tauri 截图：

- `evidence/tauri-verification/2026-06-09-stage-j-j5/01-agent-workbench-tauri-window.png`

## 2. 实现证据

- Agent 普通视图新增对话工作区：项目选择、对话选择、状态文案、会话列表、对话流和任务输入框。
- `CodexControlEntryPanel`、`UnifiedExecutionStatusPanel`、adapter / provider / continuation / runtime / diagnostics / operation boundary 等内部内容仍保留，但默认收进 `agent-boundary-details`。
- 任务输入按钮为 `生成预览 / 需要确认`，不绕过 Product Command，不直接发送 prompt，不直接写 runtime log / memory。
- embedded Agent 页使用固定高度工作区布局，列表和对话流内部滚动，避免页面级长滚。
- 左侧栏主入口和 glyph 由 `workbenchNavigation.ts` 锁定：
  - `项目`：`▤`
  - `智能体`：`◍`
  - `想法箱`：`✎`
  - `知识库`：`▢`
  - `记忆层`：`◐`
  - `Skill`：`✦`
  - `Harness`：`⬡`
  - `运行中工作流`：`≋`
- `inkwash-full.html` 原型中对应图标组已核对：想法箱 `✎`、知识库 `▢`、记忆 `◐`，运行中工作流继续使用三条横向波浪线 `≋`。

## 3. 复核线结论

复核线只读审查 J5 代码修补，结论：

- P0：未发现。
- P1：未发现。
- P2 / deferred：真实 Tauri 窗口与截图验收原先未执行；后续主管线已补一张真实 Tauri 关键截图，本 evidence 记录为关键截图探针完成，不声称全量真实 Tauri UI 自动化验收完成。

复核线允许主管线把 J5 首轮代码修补视为可回收，并确认：

- 普通 Agent UI 已先呈现对话工作区。
- 开发者内容默认收进 details。
- Product Command / 统一执行链路能力保留，没有被删掉。
- 未发现 J5 新增真实执行、prompt 发送、`.codex` 产品数据读写、后端 store / command 或自动 FormalMemory 写入。
- 左侧栏入口和 inkwash glyph 未回退。

## 4. 真实 Tauri 截图验收

执行过真实 Tauri dev 窗口验收：

- `npm run tauri:dev` 启动后，进程检查确认本轮 `target/debug/codex-governance-workbench` PID `83553` 存在。
- macOS Accessibility 读取到窗口标题 `Codex 治理工作台`，窗口 bounds 为 `{360, 90, 1280, 820}`。
- 截图保存为 `evidence/tauri-verification/2026-06-09-stage-j-j5/01-agent-workbench-tauri-window.png`。
- 视觉核对确认截图是 `Codex 治理工作台` 真实 Tauri 窗口，不是普通浏览器 smoke。

截图覆盖：

- 左侧 rail 显示 inkwash 图标组，包含想法箱、知识库、记忆层、运行中工作流入口。
- 智能体页普通视图显示项目选择、对话选择、当前会话、对话流区域和任务输入框。
- 开发者详情位于底部默认折叠区。
- 秘书输入条仍在底部，不替代智能体主对话区。

截图过程风险：

- 曾误截到 Codex 桌面窗口和 Open Design 预览窗口；这些图片已识别为无效，不作为 J5 证据。
- 最终保留的 `01-agent-workbench-tauri-window.png` 已经人工核对为真实 Tauri 工作台窗口。
- 本轮只完成 J5 关键截图探针，不声明完整 Tauri 自动化 UI 验收或全页面截图矩阵完成。

## 5. 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。

离线测试覆盖：

- 左侧主导航包含 `想法箱 / 知识库 / 记忆层 / 运行中工作流`。
- 左侧主导航 glyph 为 `✎ / ▢ / ◐ / ≋`。
- Agent 普通视图有项目选择、对话选择、对话流和任务输入。
- 开发者详情默认收起。

## 6. 过程边界

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送真实 prompt。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 新增后端 store、Tauri command、Rust runner 或 DB migration。
- 自动写 FormalMemory。
- 接入 planned adapters 真实执行。

过程偏差：

- 为遵循 Product Design skill 工作流，本轮读取了 `/Users/yoyi/.codex/plugins/cache/...` 下的 Product Design skill / reference / user-context 技能说明文件。
- 该读取属于工具技能元数据读取，不是产品代码路径；未读取用户 Codex 会话数据、secret、token、auth、完整 transcript 或 rollout，也未写 `/Users/yoyi/.codex`。
- 因此本轮不能写成“完全没有访问 `/Users/yoyi/.codex`”，只能写成“产品实现和测试未读写 `/Users/yoyi/.codex`，过程上读取过 Codex 插件技能说明元数据”。

## 7. 不接受为

J5 不接受为：

- Stage J 完成。
- J6 最终验收完成。
- 通用无限制自由 Codex 控制台。
- 自动 retry / stop / restart 已实现。
- 任意项目无限制读写。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动 FormalMemory 写入。
- 完整真实 Tauri UI 自动化验收完成。

## 8. 下一步

下一步进入 J6：Stage J 最终验收和后续路线冻结。
