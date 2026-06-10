# Evidence: authority and plan cleanup v1

日期：2026-06-02

## 触发原因

用户要求整理当前权威文档和计划。

## 薄弱点

- 本轮因为“整理一下”触发当前环境的 `neat-freak` 整理技能规则，读取了 `/Users/yoyi/.codex/skills/neat-freak/SKILL.md`。除此之外没有读取其他 `/Users/yoyi/.codex` 内容。
- 本轮只整理文档，没有复核产品代码实现。
- `CURRENT.md` 仍然保留大量历史判断，后续可以单独做瘦身；本轮只清理当前入口和计划冲突。

## 本轮确认的问题

1. `CURRENT.md` 一处写“当前没有待派发任务包”，但同文件和 `tasks/README.md` 又写下一步是 `final-skeleton-07-canvas-component-state-examples-v1`。
2. `tasks/README.md` 和 `STAGE_PLAN.md` 仍把当前阶段描述为“阶段 3C：Codex 角色编排闭环”，没有反映最终工作台骨架总包已经进入 `final-skeleton-07` 前。
3. `README.md` 的下一步仍停在架构切片和黑板建议，没有体现画布专项、骨架后两条最近主线。
4. `AUTHORITY.md` 没有把最终工作台骨架总执行包列为当前阶段计划入口。
5. 总执行包底部仍建议先执行 `final-skeleton-00`，但 `final-skeleton-00` 到 `final-skeleton-06` 已完成。

## 改动

- 更新 `README.md`：
  - 当前定位改为“最终工作台骨架执行”。
  - 增加最终工作台骨架总执行包和项目画布节点 schema 计划入口。
  - 下一步改为 `final-skeleton-07-canvas-component-state-examples-v1`。
  - 补入骨架完成后的最近两条主线：Codex 原生感会话中心 / 多智能体会话底座、项目工作流画布产品化深化。
- 更新 `AUTHORITY.md`：
  - 日期更新为 2026-06-02。
  - 明确 `STAGE_PLAN.md` 只解释阶段，不是当前事实入口。
  - 把最终工作台骨架总执行包列入当前阶段计划。
  - 增加项目工作流画布节点 schema 计划。
  - 把架构落地计划改成已完成架构切片依据，不再当当前待派发入口。
- 更新 `CURRENT.md`：
  - 当前主线改为最终工作台骨架执行。
  - 当前阶段目标改为继续总执行包。
  - 当前任务改为 `final-skeleton-07-canvas-component-state-examples-v1`。
  - 下一步建议改为 `final-skeleton-07`、`final-skeleton-08`、`final-skeleton-09`。
  - 补入骨架完成后的最近两条主线和模型/凭据的后续位置。
  - 修正真实 Tauri 验收线口径：已有最小截图线，但不是完整自动化验收。
- 更新 `tasks/README.md`：
  - 当前阶段改为最终工作台骨架执行。
  - 当前任务明确为 `final-skeleton-07-canvas-component-state-examples-v1`。
  - 当前执行目标改为画布组件状态样例、React Flow 最小实现、节点详情面板。
  - 修正真实 Tauri 验收线暂停口径。
- 更新 `STAGE_PLAN.md`：
  - 当前阶段改为阶段 4：最终工作台骨架执行。
  - 阶段 4 改为进行中。
  - 补入当前入口、已完成 `final-skeleton-00` 到 `final-skeleton-06`、下一项 `final-skeleton-07`。
  - 补入骨架完成后的最近两条主线。
- 更新 `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`：
  - 修正真实 Tauri 验收线状态。
  - 修正推荐执行顺序，标明已完成项和当前画布专项进行中。
  - 新增“骨架完成后的最近两项”。
  - 当前建议子任务改为 `final-skeleton-07-canvas-component-state-examples-v1`。

## 自检

已检查以下旧口径：

- `当前没有待派发任务包`
- `阶段 3C 进行中`
- `建议先执行：`
- `默认真实窗口验证线`

检查结果：当前入口文件里这些旧口径已清理；`final-skeleton-00` 只作为已完成历史项保留。

## 未做

- 未改代码。
- 未跑测试。
- 未启动 Tauri。
- 未执行真实 `codex exec` 或 `codex exec resume`。
- 未读写 workflow state JSON。
- 未写真实业务项目目录。
- 未迁移数据库。
