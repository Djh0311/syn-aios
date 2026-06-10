# 中间版本权威口径固化交接

日期：2026-06-03

## 完成内容

本轮只做研究前的权威固化，未进入记忆层实现方案写作。

已更新：

- `docs/middleware-version-development-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`

新增 evidence：

- `evidence/2026-06-03-middle-version-authority-freeze-before-memory-research-v1.md`

## 当前结论

`docs/middleware-version-development-plan-v1.md` 现在不是“待修订旧草案”状态，而是“已确认中间版本权威口径 + 原始阶段草案保留为历史素材”状态。

后续执行必须按第 0 节解释：

- 用户确认方案和最终结果。
- 项目主管管理过程和确认项目内过程事实。
- 全局主管复核方案边界和最终结果，不逐条确认 worker 汇报。
- 秘书不当裁判，不写正式记忆。
- 中间版本必须完成自动化工作流和记忆层两个核心闭环。
- 记忆层完成标准是观察、候选、正式记忆、来源、版本、权限、冲突、审计、召回和任务包注入，而不是 SQLite 建表或候选 sidecar。

## 当前可派发任务

可交给其他对话：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`

注意：该任务只做后端 adapter capability read model，不接 Claude / OpenClaw / OpenCode，不执行真实 Codex，不读写 `/Users/yoyi/.codex`，不写正式事实或正式记忆。

## 当前待继续任务

下一步应继续做：

- 深入复核 `docs/memory-layer-design-v1.md` 和相关 schema / 实现。
- 然后再写 `docs/plans/memory-layer-implementation-slice-v1.md`。

不要直接凭记忆写实现切片。

## 安全注意

前序并行规划中曾发生一次 shell 反引号误触，双引号里的 Markdown 反引号触发了 `codex exec` 命令替换，并导致 Codex CLI 尝试打开 `/Users/yoyi/.codex/state_5.sqlite` 后失败。

后续搜索带反引号的文本必须用：

```text
rg -F 'codex exec'
```

或使用单引号包住搜索模式。不要在 shell 双引号里放未转义反引号。

## 未做

- 未改产品代码。
- 未写 `memory-layer-implementation-slice-v1.md`。
- 未改 workflow state JSON。
- 未迁移数据库。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未运行测试，因为本轮只改文档入口。

