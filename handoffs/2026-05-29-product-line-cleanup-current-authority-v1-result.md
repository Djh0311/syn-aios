# product-line 当前权威清理与归档 v1 结果

## 薄弱点先说

- 本轮只清理 Markdown 权威入口和归档结构，没有验证产品 UI。
- 本轮没有重新评审所有历史技术正确性，归档只表示“不作为当前入口”。
- 旧文件内部仍可能有历史路径，但当前入口和保留决策已指向归档路径。
- 没有删除 Markdown 文件，所以 `archive/` 仍然很大；这是保留审计依据的代价。

## 做了什么

- 新增 `CURRENT.md` 作为当前权威入口。
- 更新 `README.md`、`STAGE_PLAN.md`、`tasks/README.md`、`DEV_LINES.md`、`PROTOTYPE_WORK_LINES.md`、`principles.md`、`backlog.md`。
- 创建 `archive/` 结构和 `archive/README.md`。
- 归档旧任务包、旧 evidence、旧 handoff 和旧参考决策。
- 保留 8 个当前权威决策在 `decisions/`。
- 写入本轮 evidence 和 handoff。

## 文件变更

新增：

- `CURRENT.md`
- `archive/README.md`
- `evidence/2026-05-29-product-line-cleanup-current-authority-v1.md`
- `handoffs/2026-05-29-product-line-cleanup-current-authority-v1-result.md`

更新：

- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `DEV_LINES.md`
- `PROTOTYPE_WORK_LINES.md`
- `principles.md`
- `backlog.md`
- 保留在 `decisions/` 下的当前权威决策路径引用。

移动：

- `tasks/*.md` -> `archive/tasks/`，保留 `tasks/README.md`。
- `evidence/*.md` -> `archive/evidence/`。
- `handoffs/*.md` -> `archive/handoffs/`。
- 非当前权威决策 -> `archive/decisions/`。

## 当前权威

当前入口：

- `CURRENT.md`

当前目录入口：

- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `DEV_LINES.md`
- `PROTOTYPE_WORK_LINES.md`
- `principles.md`
- `backlog.md`
- `archive/README.md`

## 当前主线

- Codex 会话管理。
- Codex 工作流编排。
- 任务包作为内部协议、审计、导出和交接物。

## 当前下一步

当前待派发任务包：暂无。

下一步建议：

- 工作流节点 safe probe 真实确认派发 v1。

依据：

- `archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md`

## 安全复核

- 没有写 `/Users/yoyi/.codex`。
- 没有读取 `auth.json`、`.env`、密钥或授权文件。
- 没有读取业务会话正文。
- 没有运行 Codex CLI。
- 没有运行 harness。
- 没有改产品功能代码。

## 回收建议

接受为“product-line 当前权威入口清理和历史归档完成”。

不要接受为：

- 产品功能完成。
- 真实 safe probe 已执行。
- 真实业务自动编排已完成。
