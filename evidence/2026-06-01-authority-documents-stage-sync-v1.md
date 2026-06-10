# Authority Documents Stage Sync v1 Evidence

日期：2026-06-01 16:30 CST

## 范围

本轮只做权威文档清理，不改代码。

清理目标：

- 去掉旧的 `阶段 3B`、`工作流可控执行协议 v1`、`safe probe 真实确认派发 v1` 下一步口径。
- 让阶段、开发线、原则和工作流设计草案服从 `CURRENT.md` 和 `tasks/README.md`。
- 标清 `workflow-task-package-design-v1` 已落地部分和仍未落地部分。

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

## 做了什么

- `README.md`：删除旧“当前下一步建议是工作流可控执行协议 v1”，改为当前无待派发任务包，下一步先做阶段性总结，再决定秘书型 AI 或稳定性加固。
- `STAGE_PLAN.md`：从阶段 3B 改为阶段 3C，补充四角色工作流机器、uiwork、workflow task package design v1 的完成状态，并把阶段 4 改为最终蓝图能力切片候选。
- `DEV_LINES.md`：同步 Codex 会话线、桌面应用线、信息架构线的当前状态和下一步。
- `PROTOTYPE_WORK_LINES.md`：删除旧 safe probe 下一步，改为阶段性总结和下一阶段工作线选择。
- `principles.md`：不再让 `STAGE_PLAN.md` 单独决定当前方向，改为以 `CURRENT.md` 和 `tasks/README.md` 为准。
- `docs/workflow-task-package-design-v1.md`：在开头补当前实现状态；第 11 节从“全部需要补”改为“已落地 / 仍缺”的状态说明。
- `AUTHORITY.md`：更新 `STAGE_PLAN.md` 的风险说明，标明它已刷新但仍不替代 `CURRENT.md`。

## 复核

已用固定字符串搜索确认以下旧口径在权威入口范围内不再出现：

- `阶段 3B`
- `工作流可控执行协议 v1`
- `工作流节点 safe probe 真实确认派发 v1`
- `当前下一步建议`

搜索范围：

- `AUTHORITY.md`
- `README.md`
- `CURRENT.md`
- `STAGE_PLAN.md`
- `DEV_LINES.md`
- `PROTOTYPE_WORK_LINES.md`
- `principles.md`
- `docs/workflow-task-package-design-v1.md`
- `tasks/README.md`

## 偏差

复核时曾错误使用 shell 双引号包住带反引号的搜索模式，触发一次：

```text
zsh:1: command not found: STAGE_PLAN.md
```

随后改用 `rg -F` 固定字符串搜索。该偏差没有写文件，也没有影响清理结果，但它正是项目规则提醒过的 shell 反引号风险，后续搜索带 Markdown 反引号文本必须继续使用单引号或 `rg -F`。

## 未做

- 没有跑前端测试、Rust 测试或构建，因为本轮只改 Markdown 文档。
- 没有重写最终蓝图。
- 没有改变 `CURRENT.md` 的事实判断。
- 没有读写 `/Users/yoyi/.codex`。
