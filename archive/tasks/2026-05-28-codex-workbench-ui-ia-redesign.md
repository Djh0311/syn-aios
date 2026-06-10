# 任务包：Codex 工作台 UI 与信息架构重设计

## 任务名

重设计 Codex 工作台首页入口、项目级可视化工作流、Skill 管理看板和 Harness 管理看板。

## 所属开发线

信息架构线。

这是现有信息架构线任务，不新增常设开发线。

## 背景

当前产品化桌面壳一期已完成，但 UI 仍偏索引浏览器，不是用户需要的 Codex 工作台。

当前确认的新方向：

- 第一屏必须是首页总览。
- 第一屏只有四个入口：Agent、项目、Skill 管理、Harness 管理。
- 每个入口下方显示最近打开或最近使用，不显示数量。
- 当前只编排 Codex，先把 Codex 做好做完善。
- 项目打开后进入可视化、生动的编排工作流。
- 项目内左侧有窄功能列表快捷切换。
- Skill 管理要清晰分类，看被哪个 agent 用、能在哪些 agent 用。
- Harness 管理要清楚展示框架、版本、来源、功能、使用场景，后续支持多个仓库和多个版本来源。

依据：

- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-review.md`

## 目标

- 产出新版信息架构，不继续沿用旧索引浏览壳作为最终 UI 方向。
- 明确首页四入口布局和每个入口最近使用信息。
- 明确 Agent 页：
  - 当前只做 Codex。
  - 未接入 agent 空白显示。
  - 不展示假能力。
- 明确项目页：
  - 项目列表。
  - 项目详情。
  - 项目内左侧窄功能列表。
  - 默认可视化工作流视图。
  - 右侧详情面板。
- 明确项目级工作流模型：
  - Director。
  - Codex 会话。
  - 任务包。
  - Handoff。
  - Evidence。
  - Review。
  - 状态：待派发、执行中、等待用户、待回收、已接受、需修改、暂停。
- 明确 Skill 管理看板：
  - 分类。
  - 被哪个 agent 使用。
  - 能在哪个 agent 使用。
  - 被哪些项目使用。
  - 推荐关系。
  - 删除、编辑、选择 agent 加载后置。
- 明确 Harness 管理看板：
  - 框架。
  - 版本。
  - 来源。
  - 功能。
  - 使用场景。
  - 适用项目。
  - 关联命令或验证入口。
  - 多仓库和多版本来源预留。
- 给桌面应用线输出可执行页面拆分和字段来源。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-review.md`
- `product-line/prototypes/productized-desktop-shell/README.md`
- `product-line/prototypes/index-kernel/codex-index.json`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

如果需要补充信息架构草图或页面说明，可写入：

- `product-line/prototypes/productized-desktop-shell/docs/`

## 禁止事项

- 不改前端实现。
- 不改 Tauri / Rust 后端。
- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不做个人知识库。
- 不做多 agent 接入。
- 不做向量搜索。
- 不做模型辅助调度。
- 不做真实 skill 删除、编辑或加载。
- 不做完整 harness 多仓库管理实现。
- 不把未接入 agent 写成已可用能力。

## 验收标准

- 有 evidence 和 handoff。
- 首页四入口明确，不显示数量。
- 入口下方“最近打开/最近使用”规则明确。
- Agent 页明确只做 Codex，其他 agent 空白显示。
- 项目页明确打开项目后进入可视化工作流。
- 项目内左侧窄功能列表明确。
- 可视化工作流节点、状态、右侧详情面板明确。
- Skill 管理看板字段和后置能力明确。
- Harness 管理看板字段和后置能力明确。
- 明确哪些字段来自当前 `codex-index.json`，哪些字段需要后续新增。
- 明确哪些内容进入桌面应用线下一步实现，哪些暂停。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些 evidence / handoff
4. 新版页面结构
5. 项目内可视化工作流结构
6. Skill 管理看板结构
7. Harness 管理看板结构
8. 字段来源和缺口
9. 哪些能力后置
10. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否准确反映用户确认的 UI 方向。
- 是否仍只编排 Codex。
- 是否没有把未接入 agent 写成可用能力。
- 是否能指导桌面应用线重做 UI。
- 是否没有扩大到知识库、多 agent、向量搜索、模型调度或 release 范围。
