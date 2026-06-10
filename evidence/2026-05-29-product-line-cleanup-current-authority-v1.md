# Evidence: product-line current authority cleanup v1

## 依据

- 任务包：`archive/tasks/2026-05-29-product-line-cleanup-current-authority-v1.md`
- 当前权威入口：`CURRENT.md`
- 归档索引：`archive/README.md`

## 做了什么

- 新增 `CURRENT.md` 作为当前权威入口。
- 重写 `README.md`，只保留定位、权威入口、当前主线和边界。
- 重写 `STAGE_PLAN.md`，把阶段状态改成当前主线口径。
- 重写 `tasks/README.md`，移除 200 个文件式任务流水账，只保留当前状态、已完成能力、未完成能力和下一步建议。
- 重写 `DEV_LINES.md`、`PROTOTYPE_WORK_LINES.md`、`principles.md`、`backlog.md`，统一当前主线和后置项。
- 创建 `archive/` 并移动旧任务包、旧 evidence、旧 handoff 和旧参考决策。
- 修正当前保留决策里指向已归档 handoff / task 的路径。

## 归档统计

归档后：

- `archive/tasks/`：48 个 Markdown。
- `archive/evidence/`：50 个 Markdown。
- `archive/handoffs/`：89 个 Markdown。
- `archive/decisions/`：2 个 Markdown。

当前目录保留：

- `tasks/README.md`
- `decisions/` 下 8 个当前权威决策。

## 当前口径

当前主线：

- Codex 会话管理。
- Codex 工作流编排。

当前不是：

- 任务包管理器。
- 多 agent 工作台。
- 个人知识库。
- 向量搜索系统。
- 真实业务自动编排已完成。

## 当前下一步

下一步建议任务：

- 工作流节点 safe probe 真实确认派发 v1。

依据：

- `archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md` 接受代码路径，但明确真实 safe probe 尚未执行。

## 删除情况

本轮没有删除 Markdown 文件。

原因：

- 历史 evidence / handoff 仍有审计价值。
- 不能凭文件名判断无价值。
- 不确定项按任务要求归档。

## 未做

- 没改产品功能代码。
- 没改 Tauri / React / Rust 实现。
- 没运行 Codex CLI。
- 没写 `/Users/yoyi/.codex`。
- 没读取业务会话正文。
- 没运行 harness。
- 没做真实 UI 验证。
