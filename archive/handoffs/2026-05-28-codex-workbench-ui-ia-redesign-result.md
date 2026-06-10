# Codex 工作台 UI 与信息架构重设计交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workbench-ui-ia-redesign.md`
- 开发线：信息架构线
- 回传 evidence：`product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`

## 结论

建议接受为新版 Codex 工作台 UI 与信息架构 handoff。

这份 handoff 的目标是指导桌面应用线重做 UI，不是修改现有前端，也不是实现多 agent、知识库、向量搜索或模型调度。

依据：

- 当前 UI 方向决策已经确认：第一屏四入口，项目打开后进入项目级可视化工作流。
- 阶段计划阶段 3 已把 Codex 项目级可视化编排列为目标。
- 当前产品化桌面壳一期可保留为 Tauri 技术底座，但旧索引浏览 UI 不作为最终方向继续扩展。

## 先说薄弱点

- 如果桌面应用线只把旧 6 页换成四个导航按钮，仍然不是工作台。新版核心是项目级可视化流转，而不是入口名称。
- 当前 `codex-index.json` 没有完整任务流状态机和工作流边关系，所以第一版重做 UI 需要先用可推导关系展示，再把缺口标成“需要后续索引/本地事实库补充”。
- 首页入口下方要求“最近打开/最近使用”，但当前索引没有真实使用事件，只能先用最近活跃或最近修改作为临时口径。不能把它说成真实使用历史。
- Agent 页只能把 Codex 写成可用。OpenClaw、VS Code、OpenCode、Claude Code 必须空白或后置，不得展示假能力。
- Skill 推荐、Skill 加载、Skill 编辑删除、Harness 多仓库多版本都需要写入或新数据模型，不能混入当前实现范围。

## 新版页面结构

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

新版不再把旧的首页、项目、会话、Skills / Plugins、任务线、诊断作为主导航结构。

旧能力处理：

- 会话进入项目内左侧窄功能列表的“会话”视图。
- 任务包、handoff、evidence、review 进入项目内工作流和详情面板。
- 诊断进入设置或系统状态区域，不作为首页四入口。
- Plugins 信息并入 Skill 管理里的来源和可用能力说明。

## 首页

首页原则：

- 第一屏只放四个入口。
- 不显示数量。
- 每个入口下方只显示最近打开或最近使用。
- 不做复杂信息流，不做诊断面板，不做全局统计大屏。

四个入口：

- Agent
- 项目
- Skill 管理
- Harness 管理

入口下方规则：

- Agent：显示最近使用的 agent。当前只能显示 Codex；其他 agent 不显示为已接入。
- 项目：显示最近打开或最近活跃的项目。当前没有打开事件时，用 `projects.latest_updated_at_ms` 临时排序，并标注口径为最近活跃。
- Skill 管理：显示最近查看或最近使用的 skill。当前没有查看/使用事件时，可用最近出现在索引中的 skill 或按来源分组的前几个候选，不能标成真实最近使用。
- Harness 管理：显示最近查看或最近使用的 harness。当前没有事件时，用 `harness_candidates.updated_at_ms` 临时排序，并标注口径为最近修改。

字段来源：

- 四入口：`product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- Codex 当前可用：`product-line/STAGE_PLAN.md`、`product-line/README.md`
- 最近活跃项目：`codex-index.json.projects[].latest_updated_at_ms`
- 最近修改 harness：`codex-index.json.projects[].harness_candidates[].updated_at_ms`
- Skill 候选：`codex-index.json.skills[]`

字段缺口：

- 真实最近打开事件。
- 真实最近查看 skill 事件。
- 真实最近使用 harness 事件。

桌面应用线下一步实现：

- 首页四入口。
- 入口下方最多显示少量最近项。
- 取消首页数量统计。

暂停：

- 首页大屏统计。
- 首页诊断。
- 首页多 agent 状态墙。

## Agent 页

页面目的：

- 管理 agent 接入状态，但当前只编排 Codex。
- 为未来 agent 留空位，不展示假能力。

当前显示：

- Codex 卡片：状态为当前可用。
- Codex 可用范围：只读索引、项目/会话/skills/harness 元数据展示、低风险路径动作。
- Codex 不可用范围：不嵌入聊天窗口、不写 Codex 状态库、不迁移删除会话。

未接入 agent 空白显示：

- OpenClaw：未接入。
- VS Code：未接入。
- OpenCode：未接入。
- Claude Code：未接入。

空白位规则：

- 可以显示名称和“未接入”状态。
- 不显示项目数、会话数、可操作按钮。
- 不显示“即将支持”式承诺。
- 不把不可用能力放进主流程。

字段来源：

- Codex 可用状态：阶段计划、README、当前索引存在性。
- 其他 agent 名称：UI 方向决策中的未来可能对象。

字段缺口：

- agent 接入协议。
- agent 能力清单。
- agent 健康检查。
- agent 会话索引。

桌面应用线下一步实现：

- Agent 页只展示 Codex 可用卡片和未接入空白位。

暂停：

- OpenClaw / VS Code / OpenCode / Claude Code 接入。
- agent 启动、嵌入、进程控制。

## 项目页

页面结构：

```text
项目页
  项目列表
  项目详情
    左侧窄功能列表
      工作流
      会话
      任务包
      Handoff / Evidence
      Skills
      Harness
      设置
    中间主区域
      默认：可视化工作流
      辅助：列表或表格视图
    右侧详情面板
      当前选中节点详情
      可用操作和风险
```

项目列表字段：

- 项目名称或路径尾名。
- 项目根路径。
- 最近活跃时间。
- 会话摘要。
- authority / handoff / evidence 是否存在。
- harness 候选是否存在。
- context warning。

字段来源：

- `project_root`：`codex-index.json.projects[].project_root`
- 最近活跃时间：`projects[].latest_updated_at_ms`
- 会话摘要：`thread_count`、`active_thread_count`、`archived_thread_count`
- authority / handoff / evidence：`authority_files`、`handoff_files`、`evidence_files`
- harness 候选：`harness_candidates`
- warning：`context_warnings`、`warnings`

字段缺口：

- 项目别名。
- 项目类型人工确认。
- 最近打开事件。
- 项目收藏或固定。
- 项目内工作流布局配置。

项目详情默认视图：

- 默认进入“工作流”，不是会话列表。
- 会话列表只是左侧窄功能列表中的一个辅助视图。
- 项目内所有信息围绕工作流节点和详情面板组织。

桌面应用线下一步实现：

- 项目列表。
- 项目详情三栏结构。
- 左侧窄功能列表。
- 默认工作流视图。
- 右侧详情面板。

暂停：

- 项目配置写入。
- 项目类型自动判定。
- 项目收藏固定写入。

## 项目级可视化工作流

工作流目标：

- 表达 Director、Codex 会话、任务包、handoff、evidence、review 的流转。
- 让用户看清“任务从哪里来、派给谁、产物回到哪里、是否被接受”。
- 不再把项目理解成一堆无序会话和文件。

节点类型：

- Director
- Codex 会话
- 任务包
- Handoff
- Evidence
- Review

边关系：

- Director 生成任务包。
- 任务包派发到 Codex 会话。
- Codex 会话产生 Handoff。
- Codex 会话产生 Evidence。
- Handoff / Evidence 进入 Review。
- Review 返回 Director 判断。
- Review 可产生后续任务包。

状态：

- 待派发
- 执行中
- 等待用户
- 待回收
- 已接受
- 需修改
- 暂停

状态展示规则：

- 状态是节点属性，不是单独列表页。
- 看板可以作为辅助筛选，但默认视图是工作流。
- 不确定状态显示“未知”或“缺少依据”，不能猜。

节点详情面板：

- 选中 Director：显示当前阶段、当前权威、待派发任务、风险。
- 选中任务包：显示任务名、所属线、允许读取、允许写入、禁止事项、验收标准、必须回传。
- 选中 Codex 会话：显示会话标题、会话编号、项目路径、更新时间、rollout 路径、warning。
- 选中 Handoff：显示路径、更新时间、关联任务、回收状态候选。
- 选中 Evidence：显示路径、更新时间、证据类型候选、关联任务候选。
- 选中 Review：显示接受、需修改、暂停、废弃等判断和依据。

当前索引可直接支持：

- Codex 会话节点：`threads[]`
- Handoff 节点：`projects[].handoff_files[]`
- Evidence 节点：`projects[].evidence_files[]`
- 部分项目 authority 节点：`projects[].authority_files[]`
- Harness 候选节点：`projects[].harness_candidates[]`

需要后续新增：

- Director 节点实体。
- 任务包节点解析。
- Review 节点解析。
- 节点边关系。
- 工作流坐标。
- 节点状态可靠计算。
- 等待用户状态来源。

临时实现建议：

- 先按文件关系生成只读工作流：项目中心节点、会话节点、handoff/evidence 节点、harness 候选节点。
- 任务包和 Review 如果能从项目文件中扫描到，再作为候选节点；否则显示缺口。
- 不要让用户误以为状态机已经完整。

桌面应用线下一步实现：

- 使用可视化工作流作为项目详情默认视图。
- 右侧详情面板跟随节点切换。
- 提供按状态筛选的辅助控件。

暂停：

- 复杂跨 agent 画布。
- 自动布局写回。
- 工作流配置持久化写入。
- 模型辅助调度。

## Skill 管理看板

页面目的：

- 从“目录树”升级为“Skill 关系看板”。
- 让用户知道 skill 属于哪类、被哪个 agent 用、能在哪些 agent 用、被哪些项目使用、哪些项目可推荐。

看板分区：

- 分类：系统、用户、本地插件、外部来源、未知。
- Agent 使用：当前被哪个 agent 使用。
- Agent 可用：理论上能在哪些 agent 使用。
- 项目使用：被哪些项目使用。
- 推荐关系：哪些项目可推荐使用。
- 风险：缺说明、重复、来源不明、解析失败、版本未知。
- 后置操作：删除、编辑、选择 agent 加载。

字段：

- Skill 名称。
- Skill ID。
- 描述。
- 来源类型。
- 路径。
- 插件名。
- 插件版本。
- 当前使用 agent。
- 可用 agent。
- 使用项目。
- 推荐项目。
- 加载状态。
- 风险和 warning。

当前索引可直接支持：

- Skill 名称：`skills[].title`
- Skill ID：`skills[].skill_id`
- 描述：`skills[].description`
- 路径：`skills[].path`
- 来源类型：`skills[].source_type`
- 插件名：`skills[].plugin_name`
- 插件版本：`skills[].plugin_version`
- warning：`skills[].warnings`
- 插件能力线索：`plugins[].has_apps`、`plugins[].has_mcp_servers`

需要后续新增：

- 当前被哪个 agent 使用。
- 能在哪些 agent 使用。
- 被哪些项目使用。
- 推荐项目。
- 是否已加载。
- 删除、编辑、加载动作。
- skill 来源仓库。
- skill 版本策略。

当前规则：

- Codex 是唯一可用 agent。
- 其他 agent 只能作为未接入占位，不参与推荐和加载。
- 没有证据的推荐关系显示未知，不自动推荐。

桌面应用线下一步实现：

- Skill 管理看板。
- 按来源和插件分组。
- 显示 Codex 当前可见 skills。
- 显示缺口字段为空态。

暂停：

- 真实删除 skill。
- 编辑 skill。
- 选择 agent 加载 skill。
- skills 仓库化。
- 自动推荐。

## Harness 管理看板

页面目的：

- 让 harness 从散落脚本变成可管理的验证入口和框架台账。
- 展示框架、版本、来源、功能、使用场景、适用项目、关联命令或验证入口。
- 为多仓库、多版本来源预留模型。

看板分区：

- 框架：测试、构建、启动、迁移、截图、试玩、发布前检查、未知。
- 版本：已识别版本、版本未知、多版本候选。
- 来源：项目目录、工具目录、脚本目录、文档、外部仓库、未知。
- 功能：验证什么。
- 使用场景：开发中、回归、发布前、问题复现、游戏试玩、未知。
- 适用项目：项目列表和项目类型。
- 验证入口：命令、脚本、文档链接、日志入口。
- 风险：无说明、无最近运行记录、路径缺失、候选过多、版本未知。

字段：

- Harness 名称。
- 框架类型。
- 版本。
- 来源。
- 来源仓库。
- 功能说明。
- 使用场景。
- 适用项目。
- 关联命令或入口路径。
- 最近验证状态。
- 最近运行时间。
- warning。

当前索引可直接支持：

- 名称：`harness_candidates[].name`
- 入口类型：`harness_candidates[].entry_type`
- 路径：`harness_candidates[].path`
- 来源目录：`harness_candidates[].source`
- 更新时间：`harness_candidates[].updated_at_ms`
- 文件大小：`harness_candidates[].size_bytes`
- warning：`harness_candidates[].warnings`
- 适用项目候选：候选所在 `projects[].project_root`

需要后续新增：

- 框架名。
- 版本号。
- 来源仓库。
- 功能说明。
- 使用场景。
- 关联命令语义。
- 最近验证状态。
- 最近运行日志。
- 多仓库来源。
- 多版本来源。
- 有用、废弃、加强等人工或评估状态。

当前规则：

- 第一版只能显示候选和缺口。
- 不自动运行 harness。
- 不自动判定 harness 有用或没用。
- 不做完整多仓库管理。

桌面应用线下一步实现：

- Harness 管理看板。
- 按项目、入口类型、来源目录分组。
- 标出版本、功能、场景等缺口。
- 支持打开入口路径和复制路径时走现有权限确认。

暂停：

- 多仓库多版本完整实现。
- 自动运行。
- 自动废弃。
- 自动加强。
- 写项目级验证命令清单。

## 字段来源与缺口总表

可直接来自当前 `codex-index.json`：

- 项目路径、会话数、最近活跃时间。
- 会话标题、编号、项目路径、rollout 路径、更新时间、归档状态、来源、模型摘要、warning。
- authority / handoff / evidence 文件候选。
- Skill 名称、描述、路径、来源、插件名、插件版本、warning。
- Plugin 名称、版本、manifest 路径、apps / MCP server 线索。
- Harness 候选名称、入口类型、路径、来源目录、更新时间、warning。

来自产品线文档：

- 首页四入口。
- 当前只做 Codex。
- 未接入 agent 空白显示。
- 阶段边界。
- 任务状态定义。
- 当前技术底座。

需要后续新增数据：

- 最近打开事件。
- 最近查看 / 最近使用 skill 事件。
- 最近使用 harness 事件。
- agent 能力表。
- 项目别名和项目类型确认。
- 工作流节点和边。
- Director / Review 实体。
- 任务包状态可靠解析。
- Skill 使用关系和推荐关系。
- Harness 框架、版本、来源仓库、功能、场景、运行结果。
- 用户确认记录和写入审计。

## 后置能力

后置到阶段 3 继续补模型或实现：

- 标准任务包生成。
- 回收结果登记。
- current / paused / historical / superseded 标记。
- 工作流状态更可靠计算。

后置到阶段 4：

- 写项目级配置。
- 项目级 skills 推荐。
- 项目级验证命令清单。
- Skill 加载配置。

后续版本再评估：

- 多 agent 接入。
- OpenClaw / VS Code / OpenCode / Claude Code 编排。
- 嵌入或启动 agent 窗口。
- 个人知识库。
- 向量搜索。
- 模型辅助调度。
- 复杂跨 agent 画布。
- skills 仓库。
- 完整 harness 多仓库多版本管理。

## 给桌面应用线的实现入口

下一步应实现：

- 首页四入口，不显示数量。
- Agent 页只展示 Codex 和未接入空白位。
- 项目列表和项目详情三栏结构。
- 项目详情默认工作流视图。
- 左侧窄功能列表。
- 右侧节点详情面板。
- Skill 管理看板。
- Harness 管理看板。
- 所有缺字段显示“缺少数据”或“后续模型补充”，不留空装作正常。

下一步不应实现：

- 多 agent 接入。
- Skill 删除、编辑、加载。
- Harness 多仓库多版本完整管理。
- 自动运行 harness。
- 写 Codex 状态库。
- 读取会话正文。
- 个人知识库、向量搜索、模型调度。

## 验收对照

- 有 evidence 和 handoff：已完成。
- 首页四入口明确，不显示数量：已完成。
- 入口下方“最近打开/最近使用”规则明确：已完成，并说明当前事件缺口。
- Agent 页明确只做 Codex，其他 agent 空白显示：已完成。
- 项目页明确打开项目后进入可视化工作流：已完成。
- 项目内左侧窄功能列表明确：已完成。
- 可视化工作流节点、状态、右侧详情面板明确：已完成。
- Skill 管理看板字段和后置能力明确：已完成。
- Harness 管理看板字段和后置能力明确：已完成。
- 明确哪些字段来自当前 `codex-index.json`，哪些字段需要后续新增：已完成。
- 明确哪些内容进入桌面应用线下一步实现，哪些暂停：已完成。
- 没有改前端实现：已遵守。
- 没有改 Tauri / Rust 后端：已遵守。
- 没有写 `/Users/yoyi/.codex`：已遵守。
- 没有读取或展示密钥、正文、工具输出、命令输出、输入历史或记忆正文：已遵守。
- 没有扩大到知识库、多 agent、向量搜索、模型调度或 release 范围：已遵守。
