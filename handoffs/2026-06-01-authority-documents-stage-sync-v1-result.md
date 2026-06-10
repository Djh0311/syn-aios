# Authority Documents Stage Sync v1 Handoff

## 结论

已完成权威文档清理。

旧的 `阶段 3B`、`工作流可控执行协议 v1`、`safe probe 真实确认派发 v1` 下一步口径，已从当前权威入口范围内清掉。

当前入口规则：

- 当前事实先看 `CURRENT.md`。
- 当前任务看 `tasks/README.md`。
- 权威索引看 `AUTHORITY.md`。
- 阶段说明看 `STAGE_PLAN.md`，但它不替代 `CURRENT.md`。
- 工作流和任务包设计草案看 `docs/workflow-task-package-design-v1.md`，但它不替代 evidence、handoff 或当前事实。

## 改动文件

- `AUTHORITY.md`
- `README.md`
- `STAGE_PLAN.md`
- `DEV_LINES.md`
- `PROTOTYPE_WORK_LINES.md`
- `principles.md`
- `docs/workflow-task-package-design-v1.md`
- `evidence/2026-06-01-authority-documents-stage-sync-v1.md`
- `handoffs/2026-06-01-authority-documents-stage-sync-v1-result.md`

## 验证

已用 `rg -F` 固定字符串搜索确认以下旧口径不再出现在当前权威入口范围：

- `阶段 3B`
- `工作流可控执行协议 v1`
- `工作流节点 safe probe 真实确认派发 v1`
- `当前下一步建议`

本轮没有跑代码测试。原因：只修改 Markdown 文档。

## 残留风险

- `CURRENT.md` 仍然很长，有历史流水账压力；但它现在仍是最高事实入口。
- `tasks/README.md` 仍包含大量历史回收摘要；它是当前任务队列入口，但不适合当新人阅读材料。
- `docs/memory-layer-design-v1.md` 仍是完整草案，不是开发任务；下一步若做记忆层，应先产出 `memory-layer-implementation-slice-v1.md`。
- UI 当前仍需要真实 Tauri 窗口验收，不能把 Chrome headless 或普通浏览器 DOM 验证包装成完整 UI 验收。

## 建议下一步

先做阶段性总结：把当前工作流闭环能力、UI 状态、真实能力、只读草案和未验证边界分开写清楚。然后再决定下一阶段主线。
