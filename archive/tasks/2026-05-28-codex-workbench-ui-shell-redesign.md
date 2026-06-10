# 任务包：Codex 工作台新版 UI 骨架实现

## 任务名

按新版信息架构重做产品化桌面壳 UI 骨架。

## 所属开发线

桌面应用线。

这是现有桌面应用线任务，不新增常设开发线。

## 背景

产品化桌面壳一期已经完成，但旧 UI 偏索引浏览器，不是最终 Codex 工作台方向。

信息架构线已回收新版 Codex 工作台 UI 与信息架构：

- 首页只放四个入口：Agent、项目、Skill 管理、Harness 管理。
- 入口下方显示最近打开或最近使用，不显示数量。
- Agent 页当前只展示 Codex，未接入 agent 空白显示。
- 项目打开后默认进入项目级可视化工作流。
- 项目详情采用左侧窄功能列表、中间工作流画布、右侧详情面板。
- Skill 管理从目录树升级为关系看板。
- Harness 管理从散落脚本升级为框架看板。

依据：

- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-result.md`
- `product-line/STAGE_PLAN.md`

## 目标

- 在 `product-line/prototypes/productized-desktop-shell/` 中重做 UI 骨架。
- 保留 Tauri 2 + Rust + React + TypeScript + Vite 技术底座。
- 保留只读索引读取、路径白名单和权限确认模式。
- 首页改为四入口：
  - Agent
  - 项目
  - Skill 管理
  - Harness 管理
- 首页入口下方显示最近打开或最近使用，不显示数量。
- Agent 页只展示 Codex 可用卡片和未接入 agent 空白位。
- 项目页实现：
  - 项目列表。
  - 项目详情。
  - 左侧窄功能列表。
  - 中间默认工作流画布。
  - 右侧详情面板。
- 项目级工作流至少表达：
  - 项目中心节点。
  - Codex 会话节点。
  - Handoff 节点。
  - Evidence 节点。
  - Harness 候选节点。
  - 缺少 Director / Review / 边关系时显示缺口说明。
- Skill 管理页实现看板骨架：
  - 分类。
  - Agent 使用关系。
  - 项目使用关系。
  - 推荐关系占位。
  - 缺字段说明。
- Harness 管理页实现看板骨架：
  - 框架 / 类型。
  - 版本和来源。
  - 功能和场景。
  - 项目适配和验证入口。
  - 缺字段说明。
- 旧会话页、任务线页、诊断页不再作为主导航入口；相关能力迁移到项目内功能列表、设置或详情区。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-result.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不做非 Codex agent 接入。
- 不把 OpenClaw / VS Code / OpenCode / Claude Code 写成已可用能力。
- 不做个人知识库。
- 不做向量搜索。
- 不做模型辅助调度。
- 不做真实 skill 删除、编辑或加载。
- 不做完整 harness 多仓库多版本管理。
- 不自动运行 harness。
- 不做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 不为了 UI 实现拉取外网依赖；如果本地依赖不足，要说明缺口。

## 建议实现边界

- 继续使用当前 `codex-index.json`。
- 不要求本轮引入 React Flow 依赖；如果本地没有依赖，不要联网安装。
- 工作流画布可以先用 React/CSS 实现可视化节点和连线骨架。
- 对缺失数据要显式显示“缺少数据”或“后续模型补充”。
- 首页“最近打开/最近使用”当前没有真实事件时，按 IA handoff 的临时口径展示：
  - 项目用最近活跃。
  - Harness 用最近修改。
  - Skill 用索引候选或来源分组。
  - Agent 只显示 Codex。
- 保留现有权限确认弹层和后端路径白名单。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `npm run test:offline-interaction` 通过，或按新版 UI 更新对应测试并通过。
- `cargo test --offline` 通过。
- 首页只有四个主入口，且不显示数量。
- Agent 页只展示 Codex 和未接入空白位。
- 项目详情默认进入工作流视图。
- 项目详情有左侧窄功能列表、中间工作流、右侧详情面板。
- Skill 管理和 Harness 管理是看板骨架，不是旧目录列表。
- 未接入 agent 不显示假能力。
- 缺字段有显式说明。
- 不展示敏感内容，不写 Codex 状态库。
- 验证后无 5173 监听残留。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增或更新了哪些测试
4. 新版首页如何实现
5. 项目级工作流如何实现
6. Skill 管理看板如何实现
7. Harness 管理看板如何实现
8. 哪些字段来自当前索引，哪些仍是缺口
9. 是否触碰任何禁止事项
10. 验证命令和结果
11. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否准确落实新版 IA。
- 是否仍只做 Codex。
- 是否没有把未接入 agent 写成可用能力。
- 是否没有展示正文或敏感内容。
- 是否保留路径白名单和权限确认。
- 是否通过验证命令。
- 是否没有把 UI 骨架包装成完整发布版。
