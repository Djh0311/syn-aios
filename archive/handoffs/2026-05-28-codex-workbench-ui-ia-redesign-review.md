# Codex 工作台 UI 与信息架构重设计总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-ia-redesign.md`
- 开发线：信息架构线
- 信息架构 evidence：`product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`
- 信息架构 handoff：`product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-result.md`

## 结论

接受为“Codex 工作台新版 UI 与信息架构 handoff”。

不接受为“前端已经实现”，也不接受为“多 agent 工作台已设计完成”。

依据：

- Handoff 明确旧产品化桌面壳仍偏索引浏览器，新版应转向 Codex 工作台。
- Handoff 明确首页四入口：Agent、项目、Skill 管理、Harness 管理。
- Handoff 明确首页入口下方显示最近打开或最近使用，不显示数量。
- Handoff 明确 Agent 页当前只展示 Codex，OpenClaw / VS Code / OpenCode / Claude Code 空白未接入。
- Handoff 明确项目详情默认进入项目级可视化工作流，采用左侧窄功能列表、中间画布、右侧详情面板。
- Handoff 明确 Skill 管理看板和 Harness 管理看板结构。
- Handoff 明确字段来源和缺口，没有把缺字段写成已完成能力。
- Handoff 明确未改前端、未改 Tauri/Rust、未写 `/Users/yoyi/.codex`、未读取正文或密钥。

## 先说薄弱点

- 这轮只完成信息架构产物，没有改前端实现。依据：handoff 明确“不是修改现有前端”。
- “最近打开/最近使用”当前没有真实事件存储。依据：handoff 把它列为字段缺口。
- 工作流节点和边关系还不完整。依据：当前 `codex-index.json` 没有 Director、Review、节点边关系、工作流坐标和可靠状态机。
- Skill 推荐、Skill 使用关系、Skill 加载状态还缺数据。依据：handoff 列为后续新增字段。
- Harness 框架、版本、来源仓库、功能说明、使用场景、最近验证状态还缺数据。依据：handoff 列为后续新增字段。
- 旧产品化桌面壳可以保留为技术底座，但旧 6 页结构不能作为最终 UI 方向继续扩展。依据：UI 方向决策和本轮 handoff。

## 接受的新版页面结构

新版主结构：

```text
首页
  Agent
  项目
  Skill 管理
  Harness 管理

Agent 页
  Codex
  未接入 agent 空白位

项目页
  项目列表
  项目详情
    左侧窄功能列表
    中间默认工作流画布
    右侧详情面板

Skill 管理页
  Skill 分类看板
  Agent 适配关系
  项目使用关系
  推荐关系占位

Harness 管理页
  Harness 框架看板
  版本和来源
  功能和场景
  项目适配和验证入口
```

旧能力迁移规则：

- 会话进入项目内“会话”视图。
- 任务包、handoff、evidence、review 进入项目工作流和详情面板。
- 诊断进入设置或系统状态区域，不作为首页入口。
- Plugins 信息并入 Skill 管理的来源和可用能力说明。

## 接受的工作流模型

项目级工作流节点：

- Director
- Codex 会话
- 任务包
- Handoff
- Evidence
- Review

状态：

- 待派发
- 执行中
- 等待用户
- 待回收
- 已接受
- 需修改
- 暂停

布局原则：

- 默认视图是可视化工作流，不是会话列表。
- 状态是节点属性，不是单独列表页。
- 看板可以作为辅助筛选，不替代工作流画布。
- 右侧详情面板跟随节点切换。

## 接受的 Skill 管理方向

Skill 页从目录树升级为关系看板。

接受的字段方向：

- Skill 分类。
- 当前被哪个 agent 使用。
- 能在哪个 agent 使用。
- 被哪些项目使用。
- 推荐关系。
- 来源、插件、版本和 warning。

当前实现边界：

- Codex 是唯一可用 agent。
- 其他 agent 不参与推荐或加载。
- 删除、编辑、选择 agent 加载后置。
- 没有证据的推荐显示未知，不自动推荐。

## 接受的 Harness 管理方向

Harness 页从散落脚本升级为框架看板。

接受的字段方向：

- 框架。
- 版本。
- 来源。
- 功能。
- 使用场景。
- 适用项目。
- 关联命令或验证入口。
- 最近验证状态。
- warning。

当前实现边界：

- 第一版只显示候选和缺口。
- 不自动运行 harness。
- 不自动判断有用或废弃。
- 多仓库、多版本来源后置。

## 安全和范围判断

接受当前范围控制。

依据：

- 当前仍只编排 Codex。
- 没有把 OpenClaw / VS Code / OpenCode / Claude Code 写成已接入能力。
- 没有改前端、Tauri 或 Rust。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取或展示密钥、正文、工具输出、命令输出、输入历史或记忆正文。
- 没有扩大到个人知识库、向量搜索、模型调度或 release。

## 下一步

下一步应派给桌面应用线：实现 Codex 工作台新版 UI 骨架。

实现范围应只包括：

- 首页四入口，不显示数量。
- Agent 页只展示 Codex 和未接入空白位。
- 项目列表和项目详情三栏结构。
- 项目详情默认工作流视图。
- 左侧窄功能列表。
- 右侧节点详情面板。
- Skill 管理看板。
- Harness 管理看板。
- 缺字段显示“缺少数据”或“后续模型补充”。

不应实现：

- 多 agent 接入。
- Skill 删除、编辑、加载。
- Harness 多仓库多版本完整管理。
- 自动运行 harness。
- 写 Codex 状态库。
- 读取会话正文。
- 个人知识库、向量搜索、模型调度。
