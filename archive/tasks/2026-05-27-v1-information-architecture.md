# 任务包：第一版信息架构

## 所属开发线

信息架构线。

## 背景

第一版目标是只读治理 Codex。不能先做完整多 agent 嵌入，也不能先做自动写入。

## 目标

- 设计第一版桌面应用的信息架构。
- 明确首页、项目页、会话页、skills 页、harness 页、任务线页展示哪些字段。
- 区分 ERP 项目和游戏开发项目需要的不同信息。
- 给后续 UI 实现线提供页面结构。

## 允许读取

- `/Users/yoyi/workspace/product-line/README.md`
- `/Users/yoyi/workspace/product-line/STAGE_PLAN.md`
- `/Users/yoyi/workspace/product-line/DEV_LINES.md`
- `/Users/yoyi/workspace/codex-thread-context/019e6569-3663-7b62-a560-878c71d4de75/current-conversation.jsonl`

## 允许写入

- `/Users/yoyi/workspace/product-line/handoffs/`
- `/Users/yoyi/workspace/product-line/evidence/`

## 禁止事项

- 不写应用代码。
- 不引入具体 UI 框架结论。
- 不设计自动写入 Codex 状态库的交互。
- 不把 OpenClaw、VS Code、Claude Code 纳入第一版页面主流程。

## 验收标准

- 输出一份页面结构 handoff。
- 每个页面都说明字段来源和未知点。
- 明确第一版不做哪些交互。
- 明确哪些设计是阶段 2 才能实现。

## 必须回传

1. 做了什么
2. 新增了哪些 handoff / evidence
3. 第一版有哪些页面
4. 每个页面依赖哪些数据源
5. 哪些交互被排除
6. 风险和下一步建议
