# Stage J / J5 UI Information Hierarchy And Real Tauri Product Acceptance v1

日期：2026-06-09

状态：已完成，结论为 `accepted_with_deferred_items`。接在 J4 `accepted_with_deferred_items` 之后。J5 的目标是把 Stage J 已经形成的自由操控 Codex、自动化工作流编排、运行队列、记忆捕获和用户确认链路整理成普通用户能直接使用的桌面 UI；不是换风格，不做手机端，不新增真实执行授权。

回收记录：

- `evidence/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`
- `handoffs/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1-result.md`
- 真实 Tauri 截图：`evidence/tauri-verification/2026-06-09-stage-j-j5/01-agent-workbench-tauri-window.png`

## 0. 先说薄弱点

- 当前智能体页已经有 Codex 控制入口、统一执行链路、adapter / provider / continuation / diagnostics 等信息，但普通用户看到的是“控制中心”而不是“对话界面”。
- J4 已经让运行队列和用户确认队列可见，但 J5 前普通页面仍容易把产品动作、开发者边界、raw 状态和诊断堆在同一层。
- 左侧主入口已包含 `想法箱 / 知识库 / 记忆层`，图标已沿用 inkwash 原型；J5 需要锁住这个事实，避免后续回退。
- 真实 Tauri 截图验收仍是缺口；如果本轮环境不能完成真实 Tauri，必须诚实记录，不得冒领。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`
- `evidence/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`
- `handoffs/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1-result.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`

## 2. UI 显示边界确认

平台边界：

- 只面向桌面端 Tauri 工作台。
- 不做手机端 UI。
- 不做 mobile-first responsive 设计。
- 不改变既有 inkwash（水墨壳 + 静纸芯）视觉方向。

普通用户首屏应该展示：

- 我现在在哪个项目。
- 我选择的是哪个智能体 / 会话。
- 我能输入什么任务。
- 当前会话说了什么。
- 发送前会发生什么，需要我确认什么。

普通用户首屏不应该展示：

- raw sidecar 字段。
- full command argv。
- provider credential / model verification 内部边界长文案。
- adapter descriptor 全量矩阵。
- runtime log / audit refs / internal id 长列表。
- legacy/H/PCR 历史细节。

开发者内容进入：

- `设置 / 开发者`，或智能体页内默认收起的 `开发者详情`。
- 不能再作为智能体页主视觉。

## 3. J5 目标

1. 智能体页改成对话工作区：
   - 顶部或左侧提供项目选择。
   - 提供会话选择。
   - 主体显示对话流。
   - 底部提供任务输入框。
   - 普通用户能理解“这里可以开始对话”。
2. 智能体页不再像控制中心：
   - `Codex 控制`、`统一执行链路`、adapter / provider / continuation / diagnostics 进入默认收起的开发者详情。
   - 普通视图只保留必要的发送边界摘要。
3. 智能体页避免页面级上下滚动：
   - 页面本体固定在工作区高度内。
   - 允许会话列表和对话流各自内部滚动。
   - 不引入移动端断点。
4. 左侧栏入口锁定：
   - 主入口为 `项目 / 智能体 / 想法箱 / 知识库 / 记忆层 / Skill / Harness / 运行中工作流`。
   - `运行中工作流` 图标为三条横向波浪线风格，即当前 `≋`。
   - 设置入口保持在底部；开发 / 内部入口不回到普通主导航。
5. J1-J4 关键路径准备真实 Tauri 验收：
   - 若能启动真实 Tauri，则采集关键截图和手动验收记录。
   - 若环境阻断真实 Tauri，记录失败原因和降级证据，不声明真实 Tauri 验收完成。

## 4. 非目标

J5 不做：

- 不新增真实 `codex exec` / `codex exec resume` 授权。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不新增后端 store、Tauri command、Rust runner 或 DB migration。
- 不做 planned adapters 真实接入。
- 不做 provider credential store 或 model verification。
- 不自动 retry / stop / restart。
- 不自动写 FormalMemory。
- 不把 J5 说成 J6 或 Stage J 完成。

## 5. 实现要求

### 5.1 智能体页

最小改动建议：

- 在 `AgentView.tsx` 增加或重组一个 `AgentConversationWorkspace` 形态。
- 普通主视图顺序：
  1. 项目 / 会话 / 运行方式选择条。
  2. 会话列表。
  3. 对话流。
  4. 底部任务输入框和发送前边界摘要。
- 发送按钮在 J5 默认仍不能绕过 Product Command；如果没有完整执行授权，只显示“生成预览 / 需要确认”类按钮。
- 任务正文仍不得写入 sidecar / runtime log / memory。
- 对话框要显示 readback unknown 边界：未知 / 不可用不等于 0。

### 5.2 开发者详情

把以下内容继续收纳到默认折叠区：

- `CodexControlEntryPanel`
- `UnifiedExecutionStatusPanel`
- `AgentAdapterCapabilityPanel`
- `ProviderAvailabilityPanel`
- `SessionContinuationPreviewPanel`
- `ControlledSessionContinuationPanel`
- `H2RealResumeAuthorizationPanel`
- `H2RealResumeExecutionDecisionPanel`
- `RuntimeSessionAttentionPanel`
- `AdapterSdkCliDiagnosticsPanel`
- `SessionOperationBoundaryPanel`

普通用户首屏只保留短摘要，不铺完整面板。

### 5.3 左侧栏

必须保持：

- `想法箱` glyph `✎`
- `知识库` glyph `▢`
- `记忆层` glyph `◐`
- `运行中工作流` glyph `≋`

不得把 `建议方案 / 实验画布 / 工具 / 模型/凭据` 放回普通主导航。

## 6. 验收清单

必须验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 禁止误导文案扫描：不能出现 `自动重试中`、`已自动修复`、`已写正式记忆`、`结果数：0` 等误导状态。
- 左侧主导航测试仍覆盖 `想法箱 / 知识库 / 记忆层 / 运行中工作流` 和 glyph。
- 智能体页测试覆盖：
  - 普通视图有项目选择、会话选择、对话流和任务输入。
  - 开发者详情默认收起。
  - 普通视图不再把 `Codex 控制` / `统一执行链路` 作为主面板铺开。
  - 运行中工作流 glyph 为 `≋`。

真实 Tauri：

- 优先尝试真实 Tauri 窗口验收。
- 如被权限、端口、环境或工具阻断，必须写入 evidence / handoff。
- 不能用普通浏览器 smoke 冒充真实 Tauri。

## 7. 回收要求

完成后新增：

- `evidence/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`
- `handoffs/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1-result.md`

同步 checkpoint：

- `CURRENT.md`
- `tasks/README.md`
- `README.md`
- `STAGE_PLAN.md`
- `AUTHORITY.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`

只有在代码验证、UI 边界扫描和复核线无 P0/P1 后，才能把 J5 收口为 `accepted_with_deferred_items`。J5 不等于 Stage J 完成；后续仍需 J6。
