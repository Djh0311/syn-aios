# Task Package：final-skeleton-16 项目工作流页最终收敛 v1

状态：待执行。  
用途：把 Skeleton-15 后暴露出的右侧详情混杂问题和项目工作流页任务包中心感一起收敛。  
对应总包：`2026-06-01-final-workbench-skeleton-execution-package-v1.md` 的 Skeleton-16。

## 1. 先说薄弱点

- Skeleton-15 把“秘书只读摘要”接进了通知、待办、审计、项目运行的右侧详情里，这让秘书和其他内容混在一起。
- 用户已明确要求：秘书摘要要单独做一个入口，不要和其他内容放在一起。
- 项目工作流页仍有任务包中心感和内部治理面板外露风险。
- 本轮是 UI 和信息架构收敛，不是新增自动执行能力。

一句话目标：

```text
右侧入口：秘书单独入口。
项目工作流页：项目列表 + 工作流画布 + 节点详情/抽屉。
任务包、账本、候选治理和状态机是内部信息，不做主界面中心。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`

前置依据：

- `tasks/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`
- `evidence/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`
- `handoffs/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1-result.md`
- `docs/workbench-system-architecture-v1.md`
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 3. 已知事实

- Skeleton-15 已完成秘书只读读模型和 `SecretaryBrief`。
- `SecretaryBrief` 现在渲染在 `RightDetail` 中，所有右侧详情都会看到它。
- 用户不接受这种混放方式。
- 项目工作流画布已经使用 React Flow，只读事实源来自项目 workflow state / 派生读模型。
- 独立 `CanvasView` 已降权为实验 / 模板画布，不是项目事实源。
- 候选治理最小闭环已完成，但候选确认仍不是正式事实或正式记忆写入。

## 4. 总目标

完成两个收敛：

1. 右侧详情入口收敛：秘书摘要单独入口，不和通知、待办、审计、项目运行混在一起。
2. 项目工作流页收敛：只保留项目工作流主入口感，降低任务包、账本、候选治理、状态机等内部面板的主界面权重。

## 5. 全局禁止

- 不改首页内容。
- 不重做首页 UI。
- 不做秘书聊天。
- 不让秘书直接改事实。
- 不让秘书直接派发任务。
- 不让秘书批准权限。
- 不让秘书写正式记忆。
- 不把秘书摘要塞回通知、待办、审计、项目运行详情中。
- 不把项目画布做成通用节点自动化平台。
- 不启动 MCP canvas run。
- 不把独立 `CanvasView` 合并为项目事实源。
- 不写 workflow state JSON。
- 不改 `workflow-state.v0.json` 结构。
- 不写正式事实。
- 不写正式 `MemoryRecord`。
- 不迁移数据库。
- 不接 Obsidian、向量库或图数据库。
- 不执行真实 Codex。
- 不执行 `codex exec` 或 `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不运行 harness。
- 不写真实业务项目目录。
- 不批量格式化 `src/lib.rs` 或 `src/mcp/**`。

## 6. 第一段：秘书摘要单独入口

### 6.1 目标

把秘书摘要从现有右侧详情内容里移出，新增独立入口。

建议方式：

- `RightPanelKey` 增加 `secretary`。
- 右侧竖向入口增加“秘书”或“秘书摘要”入口。
- 点击该入口后，右侧详情只显示 `SecretaryBrief` 和秘书只读边界。
- 通知、待办、审计、项目运行详情不再渲染 `SecretaryBrief`。

### 6.2 入口边界

允许：

- 右侧窄栏增加一个独立图标入口。
- 使用现有 `SecretaryBrief` 组件。
- 入口标题可以是“秘书”或“秘书摘要”。
- hover / title / aria-label 标明“秘书只读摘要”。

禁止：

- 新增左侧主导航“秘书”页面。
- 把秘书固定成首页模块。
- 把秘书塞进项目画布右侧栏。
- 把秘书摘要作为通知、待办、审计、项目运行的子内容。
- 增加任何写入、确认、执行按钮。

### 6.3 验收

- 未打开秘书入口时，通知、待办、审计、项目运行详情中不出现“秘书只读摘要”。
- 打开秘书入口时，右侧详情只显示秘书摘要相关内容。
- `SecretaryActionProposal` 仍不接 `PendingAction`。
- `SecretaryBrief` 仍只读。

## 7. 第二段：项目工作流页最终收敛

### 7.1 目标

项目界面应更接近：

```text
项目列表 + 项目工作流画布
```

项目工作流页主视觉只服务工作流画布和节点详情。

### 7.2 收敛方向

需要检查 `ProjectsView.tsx` 当前仍外露哪些内容：

- 任务包草稿 / 任务包预览。
- 工作流账本。
- 子智能体汇报。
- 审查结果。
- 异常通知。
- 状态机。
- 完成闸门。
- 候选治理条。
- 审计摘要。

处理原则：

- 能进入节点详情的，进入节点详情。
- 更适合全局处理的，进入右侧独立入口或对应中心。
- 保留必要操作，但不能让“任务包管理器”成为项目工作流页主界面。
- 不删除已有安全边界、确认弹层或候选治理能力。
- 不把候选治理伪装成正式事实写入。

### 7.3 画布边界

- 项目工作流画布仍是项目 workflow 主入口。
- React Flow 仍只是渲染和交互层，不是事实源。
- 事实源仍来自 workflow state / sidecar / 派生读模型。
- 独立 `CanvasView` 仍是实验 / 模板画布，不参与项目事实。

## 8. UI 要求

- 右侧图标栏保持窄入口。
- 秘书入口独立，不和其他右侧入口内容混放。
- 项目工作流页减少说明性文字。
- 项目工作流页不要出现大段“这是如何使用”的文本。
- 固定格式区域要有稳定尺寸，避免 hover、计数、空态文案导致布局跳动。
- 不使用首页级大标题处理工作流页内部面板。
- 不把多个卡片套在卡片里。

## 9. 测试要求

必须补离线测试，至少覆盖：

1. 右侧通知详情不渲染“秘书只读摘要”。
2. 右侧待办详情不渲染“秘书只读摘要”。
3. 右侧审计详情不渲染“秘书只读摘要”。
4. 右侧项目运行详情不渲染“秘书只读摘要”。
5. 右侧秘书入口渲染“秘书只读摘要”。
6. 秘书入口没有写入按钮或会触发 `PendingAction` 的操作。
7. 项目工作流页主区域仍能渲染项目画布。
8. 项目工作流页不把任务包预览 / 账本 / 候选治理作为主视觉中心。

## 10. 验证命令

在：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

必须跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

如果改了 Rust，再额外跑：

```text
cargo test --lib
```

如果本轮只改前端，不需要 Rust 验证。

真实窗口 / 截图验收：

- 如果当前对话有可用浏览器或 Tauri 截图工具，必须补截图证据。
- 至少截图：
  - 右侧秘书独立入口展开状态。
  - 通知 / 待办 / 审计 / 项目运行中不含秘书摘要的状态。
  - 项目工作流页主区域。
- 如果当前对话没有真实窗口工具，不能声称完整 UI 验收完成，必须在 evidence / handoff 里列为未验证。

## 11. 验收标准

接受为：

- 秘书摘要已从其他右侧详情中移出。
- 秘书摘要有独立入口。
- 秘书入口仍是只读模型展示，不是执行入口。
- 项目工作流页主视觉回到画布和节点详情。
- 任务包、账本、候选治理、状态机不再作为主界面中心。
- 测试覆盖右侧入口分离和项目工作流主区域。
- 没有写 workflow state。
- 没有写正式事实。
- 没有写正式记忆。

不接受为：

- 秘书聊天完成。
- 秘书自动执行完成。
- 项目画布可编辑运行完成。
- 通用节点自动化平台完成。
- 正式任务包管理器完成。
- 正式记忆管理完成。
- Obsidian / 知识库集成完成。

## 12. 必须输出

执行完成后必须新增：

- `evidence/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md`
- `handoffs/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`

如有截图证据，建议放在：

- `evidence/tauri-verification/2026-06-03-final-skeleton-16/`

## 13. 完成后

普通情况下进入最终统一验收前的剩余清理。

如果本轮发现项目工作流页仍然过重，先不要进入最终验收；需要单开“项目工作流页 UI 复核 / 真实窗口验收”任务。
