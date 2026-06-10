# final-skeleton-16 项目工作流页最终收敛 evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮没有真实浏览器或 Tauri 窗口截图：当前对话没有暴露可用浏览器 / Tauri 截图工具，工具发现只返回线程读取工具，所以不能声称完成真实窗口 UI 验收。
- 项目工作流页仍保留候选治理操作能力，但已经从画布下方独立主 strip 降为项目画布侧栏详情卡。
- 任务包草稿功能仍存在于内部可访问的旧工具分支；本轮没有删除历史任务包能力，只收敛当前项目工作流主入口。

## 本轮目标

按 `tasks/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md` 完成两个小收敛：

1. 秘书摘要成为独立右侧入口，不再混入通知、待办、审计、项目运行详情。
2. 项目工作流页主视觉回到项目画布和节点详情，候选治理不再作为主界面中心。

## 已实现

修改：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

新增：

- `evidence/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md`
- `handoffs/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1-result.md`

实现内容：

- `RightPanelKey` 增加 `secretary`。
- 右侧竖栏增加 `秘书只读摘要` 独立入口。
- `RightDetailPanel` 对 `secretary` 走独立只读分支，只显示秘书边界和 `SecretaryBrief`。
- 通知、待办、审计、项目运行详情不再渲染 `SecretaryBrief`。
- `CandidateGovernanceStrip` 从项目工作流画布下方移入 `ProjectCanvasSidePanel`。
- 候选治理 DOM class 从独立主区块 `project-candidate-governance` 收敛为 `project-canvas-detail-card project-candidate-governance-card`。
- 未新增秘书写入按钮、执行按钮或 `PendingAction`。

## 红灯测试

先补离线测试后收敛实现：

- 初次运行 `npm run test:offline-interaction` 失败，原因是项目页仍存在独立 `project-candidate-governance` strip。
- 移动候选治理后再次运行失败，原因是非秘书右侧详情中仍残留原 `SecretaryBrief` section。
- 删除残留秘书 section 后，右侧入口分离和项目页候选治理降权测试通过。

测试覆盖：

- 通知详情不渲染“秘书只读摘要”。
- 待办详情不渲染“秘书只读摘要”。
- 审计详情不渲染“秘书只读摘要”。
- 项目运行详情不渲染“秘书只读摘要”。
- 秘书独立入口渲染“秘书只读摘要”。
- 秘书独立入口除关闭按钮外没有操作按钮。
- 项目工作流页仍渲染项目画布。
- 项目工作流页不再把候选治理作为独立主 strip。
- 候选治理仍保留为项目画布侧栏详情卡。

## 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 8`。
- `npm run build`：通过；仍有 Vite chunk size warning，构建产物生成成功。

未运行 Rust 验证：

- 本轮未修改 Rust。

## 边界自检

流程偏差：

- 收尾核对时有一次 `rg` 命令把含反引号的 `/Users/yoyi/.codex` 文本放进 shell 双引号，触发 zsh 命令替换尝试并返回 `permission denied: /Users/yoyi/.codex`。
- 该命令没有读写 `/Users/yoyi/.codex`，随后已按项目规则用单引号重跑核对成功。
- 后续搜索含反引号文本必须继续使用单引号或 `rg -F`。

本轮没有做：

- 没有执行真实 Codex。
- 没有执行 `codex exec` 或 `codex exec resume`。
- 没有读写 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env`、完整 transcript。
- 没有运行 harness。
- 没有启动 MCP canvas run。
- 没有写 workflow state JSON。
- 没有改 `workflow-state.v0.json` 结构。
- 没有写正式事实。
- 没有写正式 `MemoryRecord`。
- 没有迁移数据库。
- 没有修改 Rust。

## 截图验收状态

未完成真实窗口 / 截图验收。

原因：

- 当前对话没有暴露浏览器或 Tauri 截图工具。
- `tool_search` 只发现线程读取工具，未发现本轮可用的浏览器截图能力。

因此本轮只接受为离线测试、TypeScript 和生产构建通过的 UI 信息架构收敛；不接受为真实 Tauri 窗口视觉验收完成。

## 下一步判断

可以进入后续最终统一验收或另开真实 Tauri 截图验收切片。

继续保持：

- 秘书是只读协作层，不是执行器。
- 候选治理只处理候选层，不升级为正式事实或正式记忆。
- 项目工作流画布是项目 workflow 主入口，但不是通用节点自动化平台。
