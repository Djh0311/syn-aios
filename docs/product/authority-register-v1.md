# Syn 权威登记表 v1

日期：2026-08-09
状态：**当前权威导航正本。**

本文只回答每类材料能决定什么。它不自动授予开发、真实运行、数据迁移、提交、合并或发布权限。

## 1. 三条分开的权威链

### 1.1 产品方向

由高到低：

1. 当前用户明确指令；
2. [`../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md`](../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md)、[`../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md`](../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md)与它们确认的[`syn-product-canon-v1.md`](syn-product-canon-v1.md)；
3. [`knowledge-infrastructure-canon-v1.md`](knowledge-infrastructure-canon-v1.md)等专题正本；
4. 本文第 3 节精确列出的当前有效专题决定；
5. [`../workbench-system-architecture-v1.md`](../workbench-system-architecture-v1.md)和[`../workbench-frontend-display-boundary-v1.md`](../workbench-frontend-display-boundary-v1.md)。

新指令与旧正文冲突时，以新指令为准，并尽快回写对应正本。候选、计划、代码现状和验收报告不能反向改写产品方向。

### 1.2 当前实现事实

由当前代码、存储合同、直接检查和对应证据决定。主要入口是：

- [`../current-state.md`](../current-state.md)；
- 当前源码、存储所有权合同和直接验证；
- 与点名版本相符的验收证据。

[`../2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`](../2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md)只是 2026-08-01 的实现盘点快照，使用时必须回到当前源码重核，不再承担持续更新的“当前清单”。

产品正本描述目标，不得冒充已经实现；当前代码描述现状，也不得冒充产品最终要求。

### 1.3 开发与执行权限

只由当前用户指令、项目 `AGENTS.md`、开发护栏（Harness）的当前计划链和授权文件共同决定。产品正本、阶段计划、报告、交接和历史任务包本身不授予施工权限。

## 2. 状态分类

| 状态 | 含义 | 可以做什么 | 不可以做什么 |
|---|---|---|---|
| 当前产品正本 | 已确认的长期产品定义 | 约束所有后续设计 | 冒充当前实现或施工授权 |
| 当前架构 / 专题正本 | 某一专题的现行边界 | 约束对应设计 | 越过产品正本扩大目标 |
| 当前有效决定 | 对具体争议的正式拍板 | 修订或补充正本 | 被其他无关材料整包引用后无限扩张 |
| 当前实现底账 | 当前代码和接线事实 | 说明现在有什么 | 决定未来必须维持现状 |
| 当前开发计划 | 迁移顺序和阶段退出条件 | 组织后续施工 | 自动激活任务或冻结永久产品细节 |
| 候选 | 仍待用户拍板 | 提供讨论素材 | 当作正本、任务或权限 |
| 被取代 / 停止 / 历史 | 解释来路和旧事实 | 回溯原因 | 恢复成当前路线 |
| 报告 / 交接 / 证据 | 证明特定版本和场景 | 支撑事实强度 | 定义新产品路线或把局部验收升级成真实发布 |
| 工程治理 | 管开发协作、验证和记录 | 约束施工方式 | 定义 Syn 最终是什么 |

## 3. 当前产品和架构正本

| 文件 | 当前职责 | 不负责 |
|---|---|---|
| [`syn-product-canon-v1.md`](syn-product-canon-v1.md) | Syn 的完整产品定义 | 具体实现顺序和代码现状 |
| [`knowledge-infrastructure-canon-v1.md`](knowledge-infrastructure-canon-v1.md) | 所有智能体共用的知识与技能获取基础层 | 冻结具体检索引擎和表结构 |
| [`../workbench-system-architecture-v1.md`](../workbench-system-architecture-v1.md) | 当前目标软件模块边界 | 产品愿景和当前实现完成度 |
| [`../workbench-frontend-display-boundary-v1.md`](../workbench-frontend-display-boundary-v1.md) | 普通、专业和开发界面的显示边界 | 具体视觉稿和阶段授权 |
| [`../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md`](../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md) | Syn 原生治理核心、可替换 Agent Runtime 与受治理自升级方向 | 当前实现完成度或自动升级授权 |
| [`../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md`](../../decisions/2026-08-09-syn-product-canon-authority-and-knowledge-infrastructure-v1.md) | 本轮产品正本、材料分层和知识基础设施的确认决定 | 当前代码完成度 |
| [`../../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`](../../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md) | 事件驱动、个人 / 项目范围和角色运行方式 | 当前接线完成度 |
| [`../../decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`](../../decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md) | 当前记忆自捕获、每日整理和技能候选政策 | 具体自动阈值和实现方案 |
| [`../../decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`](../../decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md) | 原生知识工作区及其界面路线 | 整个知识基础设施的定义 |

以上是当前决定的精确清单。`decisions/**` 中其他文件全部按历史决定、来源材料、验收记录或过期授权阅读；即使旧正文仍写着“当前、已采纳、最终”，也不会仅凭文件自述恢复效力。以后若有旧决定需要重新成为现行补充，必须先更新本登记表。

## 4. 已吸收但不再单独竞争的材料

| 文件 | 处置 |
|---|---|
| [`../own-agent-and-company-vision-v1.md`](../own-agent-and-company-vision-v1.md) | 稳定角色、组织伸缩、咨询多视角和用户决定权已吸收进产品正本；自动选模等仍未决内容进入候选登记。旧“固定模型”字样不再作为现行定义。 |
| [`../memory-layer-consolidated-canon-v1.md`](../memory-layer-consolidated-canon-v1.md) | 其中仍有效的记忆来源、版本、作用域、冲突和权威 / 索引分离原则已由新正本与 2026-08-01 记忆修订吸收；原文件只作历史导航。 |
| [`../memory-layer-design-v1.md`](../memory-layer-design-v1.md) | 保留为详细设计来源；与 2026-08-01 记忆修订冲突的旧默认失效，未确认对象和算法不是正本。 |
| [`../workflow-task-package-design-v1.md`](../workflow-task-package-design-v1.md) | 最小上下文、权限、汇报和验证边界继续作为设计输入；具体对象、状态和界面草案不自动升格。 |
| [`../middleware-version-development-plan-v1.md`](../middleware-version-development-plan-v1.md) | 已确认原则由新正本和总计划吸收；旧阶段顺序只作历史。 |

## 5. 统一候选入口

所有仍待拍板的产品和架构问题只从 [`candidate-register-v1.md`](candidate-register-v1.md)进入讨论。

以下两份旧候选全文只保留来源价值，不再各自充当活动入口：

- [`../conversation-model-adaptive-routing-governance-design-v1.md`](../conversation-model-adaptive-routing-governance-design-v1.md)；
- [`../syn-global-jarvis-capability-and-explicit-project-entry-recommendation-v1.md`](../syn-global-jarvis-capability-and-explicit-project-entry-recommendation-v1.md)。

前者的未决问题已经压缩到候选登记；后者已采纳的秘书和五类业务进入产品正本，被否的万能入口、猜项目和全局原始上下文池继续失效。

## 6. 当前计划和事实入口

- 总迁移顺序：[`../plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`](../plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md)，当前覆盖 M1–M11；阶段存在不等于已激活。
- M5/M6 当前修正实施计划：[`../plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md`](../plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md)，状态 `PLANNED / NOT_ACTIVE`；只组织事实恢复与阶段退出，不改产品正本、不自动激活 stage-14/15。
- 当前事实：[`../current-state.md`](../current-state.md)。
- 当前开发入口：[`../harness/plan.md`](../harness/plan.md)及其当前链；没有活动阶段时不得从历史计划恢复授权。

计划中的表名、字段名、阶段拆法和自动阈值，除非另有产品决定，不属于永久产品正本。

## 7. 报告、交接和工程治理

验收报告和交接只能证明特定版本、场景、输入和证据等级。开发护栏只决定授权范围内怎么施工。它们全部不能定义 Syn 最终是什么，也不能把隔离测试升级为真实桌面应用、真实服务提供方或发布验收。

根目录中的代码地图、状态机审计、架构审计、迁移合同、开发护栏审计、工作总结和使用痛点，分别属于现状索引、历史报告、工程治理或研究输入，不参与产品正本竞争。

[`../agent-memory-governance.md`](../agent-memory-governance.md)管理工程智能体跨会话记忆，不是 Syn 产品记忆层权威。

## 8. 目录级处置

| 目录或入口 | 统一状态 | 阅读规则 |
|---|---|---|
| `docs/product/` | 当前产品入口 | 只有这里的正本、登记和明确链接拥有对应现行职责 |
| `decisions/` | 混合历史目录 | 只有第 3 节精确列出的决定现行，其余降级 |
| `docs/plans/` | 当前计划与历史计划混合目录 | 只有计划索引列出的总计划、M1-M11、已登记修正计划和停放计划保留对应状态；其他默认历史 |
| `docs/contracts/` | 冻结工程合同基线 | 约束点名阶段的对象和安全边界，不自动表示代码实现或产品方向 |
| `docs/research/`、`docs/design/` | 研究与设计来源 | 可以支持新决定，不能直接开工或升格 |
| `tasks/` | 历史任务记录 | 完成或旧任务不恢复当前任务状态 |
| `evidence/`、`docs/evidence/` | 证据记录 | 只证明点名版本、输入、场景和证据等级 |
| `handoffs/` | 历史交接记录 | 只说明当时进度、保全和接手上下文 |
| `archive/` | 历史归档 | 不提供当前方向、任务或授权 |
| `docs/harness/` | 工程控制与历史记录 | 当前控制文件管理施工；完成报告和历史快照不定义产品 |

大量证据和交接存在交叉引用，本轮先用目录入口和状态规则统一降级，不做会制造断链的批量搬迁。以后若物理归档，必须先建立旧路径到新路径的映射并逐批验证。

## 9. 更新规则

- 用户确认新产品要求时，优先更新产品正本或专题正本，再记录决定。
- 新建议先进入统一候选登记，不能再在根目录创建一份自称权威的散落建议稿。
- 旧文件被吸收或取代后，写清去向；需要保留历史时保留来源，不让正文继续自称当前。
- 报告只写它真正证明的内容；计划只写迁移和退出条件；当前状态只写现在。
