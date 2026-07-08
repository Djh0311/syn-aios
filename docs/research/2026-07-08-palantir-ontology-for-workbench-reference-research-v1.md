# Palantir Ontology 与本体驱动 Agent —— 工作台参考约束研究 v1

状态：外部参考研究。对照最终蓝图与 `docs/workbench-system-architecture-v1.md`，**不覆盖项目主轴**（项目仍是最高级业务对象）。体例对齐 `docs/research/2026-06-05-odysseus-*` 与 `2026-06-05-paseo-*`：可吸收为蓝图约束 / 明确不吸收 / 阶段边界 / 推荐后续路线。

参考资料（一手优先，第三方交叉）：

- Palantir 官方：`palantir.com/docs/foundry/architecture-center/ontology-system`、`.../ontology/overview`、`.../ontology-sdk/overview`、`.../architecture-center/aip-architecture`；blog `connecting-ai-to-decisions-with-the-palantir-ontology`、`connecting-agents-to-decisions`。
- 第三方批判：Lokad `review-of-palantir-com`（genuine but not novel）、HASH `the-problem-with-palantir`（锁定）、getperspective `palantir-forward-deployed-engineering-playbook`（FDE 成本）。
- 开源对位：dbt Semantic Layer / Cube Core（MCP 暴露治理指标）/ Open Semantic Interchange（OSI 中立标准）；OG-RAG、Microsoft GraphRAG（结构化世界模型 vs 裸 RAG）。

---

## 1. 一句话结论

**工作台的现有架构本质上已经是一个"本体系统"，只是没用这个名字。** 控制核心（`§5.3`）= 治理、适配器层（`§5.6`）= 动作、项目黑板"候选转事实"（`§5.4`）= Palantir 的 stage 待审、事件账本 + 审计账本（`§5.5`）= 血缘。Palantir 花十几亿美元、靠驻场工程师搭出来的东西，工作台的骨架方向是对的——**这份研究首先是一次架构方向的正面背书**。

本体思想能给工作台补的**唯一临门一脚**是：把"适配器能力"统一提升为一等的、带 `schema + 前置校验 + 风险等级 + 权限 + 是否需人审 + 写回目标 + 审计锚点` 的**「动作类型（Action Type）」**，作为 agent 唯一的动手接口。这正是蓝图北极星"**接住 vibe**"缺的那层——**治理化的动作层**。

**借思想，拒平台。**

---

## 2. Palantir 本体是什么（精炼）

本体分三层：

- **语义层（名词）**：对象类型 + 属性 + 关系（Object / Property / Link）。企业里"真实存在的东西"——客户、订单、设备。
- **动能层（动词）**：动作类型（Action Type）。**能改对象状态、并把变更写回源系统**（原子事务 / 批 / 流 / CDC）。这是本体区别于纯读的 BI 语义层、知识图谱的命门——官方自称 "kinetic / decision-centric ontology"。
- **动态层（逻辑）**：Functions，绑在对象上的规则 / 模型 / 计算。

**每个动作天生带三样**：权限校验 + 高危需人工审批 + 完整审计。这就是它敢让 AI 真去改现实的原因。

**AIP（在本体上搭 agent）的核心主张**："LLM 必须锚定本体才有用"。本体给 agent 三样：

1. **语义地图**——agent 不裸扫数据库、不自由翻 schema，只在本体这层受控导航（"对象查询"）。
2. **工具 = 本体动作**——Action Type 自动暴露成 agent 的工具，agent **只能在被授权的参数边界内填参、提交**，碰不到底层表；真正的执行 / 权限 / 审计交给平台。
3. **写回天然带治理**——高危动作默认只能 **stage（暂存）交人审**；只有跑熟的、可信的低危流程，组织才"外科手术式地"放开自动闭环。每个动作记"谁、何时、通过哪个 agent、干了什么"。

一句话：**架构性防错，不是靠 prompt 求 AI 别乱来。**

**批判（拒的部分）**：第三方定性是 "genuine but not novel"——把语义建模做得很严，不是发明了新范式。它咬死"我们不是语义层"很大程度是防被拿去和 dbt/Snowflake 比价。真成本极高（靠 FDE 驻场人肉搭 + 本体漂移），专有锁定实打实（业务逻辑锁进 Foundry 专属代码，迁移丢的不是数据是逻辑）。行业已用开源补齐（dbt/Cube + MCP + OSI 中立标准）。

---

## 3. 与工作台架构的映射（核心一节）

| Palantir 本体 | 工作台已有 | 差距 |
|---|---|---|
| 对象 / 关系（语义层，名词） | 项目单元 `§6` + 当前快照 `§5.5` + 事实层 | 已有**治理对象**（项目/工作流/节点/记忆/权限）；尚未系统建模**业务领域对象**（订单/SKU/文件/代码符号） |
| 动作类型（动能层，动词，带治理） | 适配器层 `§5.6` + capability registry `§5.8` + 控制核心"候选转事实" | **缺一等的 ActionType 对象**——现在 agent（codex）靠任务包 + prompt 干活，工具是 codex 自带 shell/file，不是"从项目授权的动作集合里选" |
| Functions（动态层，逻辑） | 工作流编排 `§9` + 控制核心策略 + 后置优化层 `§5.7` | 方向一致 |
| 治理（权限 / 审计 / 血缘） | 控制核心 `§5.3` + 事件账本 + 审计账本 `§5.5/§10` + 记忆生命周期 `§8.2` | **工作台这块甚至比 Palantir 讲得更细**（候选/待确认/已采纳/冻结/废弃/历史） |
| kinetic write-back | 适配器写回 + 事件账本 | 已有 |
| stage 暂存待审 | 项目黑板"只有控制核心能把候选升级为正式事实"`§5.4` | **这就是 Palantir 的 stage**——同一个机制 |
| Ontology SDK（统一类型化访问对象/动作） | `§5.9` Paseo 参考里的 `WorkerAdapter/DispatchRequest/PermissionEnvelope` 抽象方向 | 尚未做成统一的"对象查询 + 动作调用"接口 |

**结论**：工作台已经具备本体的语义层雏形、治理层（更细）、写回、以及 stage/候转事实机制。**唯一系统性缺的是把"受控动作"提升为一等对象 ActionType，并作为 agent 唯一的动手接口**；其次是（更后置的）业务领域对象层和统一 SDK。

**这也解释了蓝图北极星**："接住 vibe" = 用户想到新功能不打断当前工作，让 agent 接手去做。要让 agent 敢接、且不闯祸，它必须只能通过**受治理的动作**动手——这正是本体动作层。**"接住 vibe 的工具"的学名，可能就是本体的动作层。** 对应架构里的**手脑解耦**（脑 = 控制核心 `§5.3`，手 = 适配器 `§5.6`）——本体动作层就是让"手"变成"一套被治理的动词"，而不是"一把打开数据库的钥匙"。

---

## 4. 可吸收为蓝图约束

- **把 adapter 能力提升为 ActionType 一等对象**：字段至少含 `name / input schema / precheck 前置校验 / risk 风险等级 / permission 权限 / need_human_review 是否需人审 / writeback_target 写回目标 / audit 锚点`。agent 只能从**项目授权的 ActionType 集合**里选来动手。这不是新增约束，而是把 `§5.8` 已经要求的"capability registry 必须声明风险等级 / owner scope / credential / 是否需任务包授权"**做成 agent 真正的工具集**，而非只是一张声明表。
- **"动作即工具，工具自带治理"作为正面框架**：对齐 `§5.8`"不允许 agent 自由调用工具完成整件事""worker 只能读任务包允许的最小上下文和工具"。本体给这个约束一个正面表述——不是"限制 agent"，而是"给 agent 一套受治理的动词"。
- **stage / 待人审的信任阶梯**：本体"高危默认 stage、低危可外科手术式授权自动闭环"，与工作台 `§5.4` 黑板"候选转事实"、以及 `CURRENT.md` 里 Phase B "advisory 主管 + 升档 = 信任阶梯单独拍"是**同一个机制**。本体给了业界背书：升档判据 = 某类动作跑熟、可信、低危，才从 stage 放开到自动闭环。
- **Ontology SDK 思想（后置）**：未来给 adapter/agent 一个统一类型化的"对象查询 + 动作调用"接口（类比 `fetchPage / applyAction / executeFunction`），对齐 `§5.9` Paseo 的 `WorkerAdapter/RunUnit/DispatchRequest/PermissionEnvelope` 抽象方向。全程沿用控制核心的权限与审计。
- **业务领域对象进本体（更后置）**：当工作台用于真实项目（如用户的宠物电商 / ERP），agent 要推理的不只是工作流节点，还有项目的领域对象（订单 / SKU / 文件 / 代码符号）。`CodeIndexAdapter / KnowledgeAdapter`（`§5.6`）是这块入口。本体提示：让项目能声明自己的领域对象，agent 在结构化对象上推理而非裸文档——收益主要在多跳推理与跨实体关系（GraphRAG/OG-RAG 证据），单纯"查数"用不上，别过早上。

---

## 5. 明确不吸收

- **不买 / 不复刻 Foundry 封闭平台**。整套 Palantir 范式是给大企业、七八位数预算、驻场团队准备的，与本地单人工作台正相反。
- **不学 FDE 驻场人肉搭大而全的形式化本体（OWL）**。那需要稀缺的知识工程 + 形式逻辑技能、数年投入。工作台按增量来。
- **不专有锁定**：对象 / 动作定义用工作台自己的 schema（已是 JSON→SQLite 方向），保持可导出、可重建——呼应 `§5.8`"向量库 / 索引只能是可重建派生能力"。
- **不把本体当全量事件溯源**：工作台 `§3.4` 已明确不做全量事件溯源。本体 = 当前快照 + 事件账本 + 审计，不是所有状态靠事件重放。这一点工作台和 Palantir 的取舍**一致**（Palantir 也是 ledger + snapshot，不是纯 ES）。
- **不因概念大就提前工程化**：principles `§5` 后置内容（多 agent / 知识库 / 向量 / 记忆工程化）不因本体研究存在而提前。领域对象层、统一 SDK 都是后期的事。

---

## 6. 阶段边界（尊重现状，最重要）

- **当前状态**：`CURRENT.md` 显示 Phase A（交办闭环）已整阶段收口，Phase B（秘书 + 全局主管 agent · 读口供 / 判黄牌）开门。本研究属"概念校准 + 蓝图约束"，**不改当前 Phase B 任务范围、不拆任务包、不授权实现**。
- **不 front-run**：ActionType 一等化、领域对象层、统一 SDK 均属蓝图后期（对应蓝图"乙"产品功能 + principles `§5` 后置），不进当前 backlog。
- **现在唯一可做的最小一步（可选，接缝区先写 schema）**：按 principles `§3`"接缝区先写 schema / 状态机 / 协议，再写实现"，可以**只补一份 `ActionType` 的 schema 定义**（把 `§5.8` capability registry 已要求的字段 + 本研究补的 `need_human_review / writeback_target / audit 锚点` 钉成一个明确对象），**不写任何实现、不碰适配器执行、不碰高危清单**。这既零风险，又把本体最值钱那块的接缝先钉下来，供 Phase B 之后落地时对齐。是否要做由主导线按当前排布决定。

---

## 7. 推荐后续路线（供蓝图后期，不进当前 backlog）

```text
ONTOLOGY-0  ActionType schema 接缝设计（只 schema/协议，不实现）
-> ONTOLOGY-1  把现有 adapter 能力登记成 ActionType（映射，不改执行体）
-> ONTOLOGY-2  agent 工具集从「项目授权的 ActionType 集合」派生（对齐 §5.8 最小工具）
-> ONTOLOGY-3  领域对象层（项目声明领域对象；后置到真实业务项目场景）
-> ONTOLOGY-4  统一对象/动作 SDK（OSDK 类比；最后做）
```

阶段边界：

- ONTOLOGY-0 是唯一现在可选做的（接缝 schema）；1–4 均须先从本约束倒推专题设计，再由全局主管批准阶段计划和任务包。
- 全程沿用控制核心、项目边界、任务包允许清单、用户确认和审计；不因"本体概念"绕过任何一道。

---

## 8. 一句话

工作台不用去学 Palantir，因为它已经在做本体了——**控制核心是治理，适配器是动作，黑板是 stage，事件账本是血缘。** 本体思想补的唯一临门一脚：把"受控动作"提升为一等对象、做成 agent 唯一的动手接口。这就是"接住 vibe"缺的治理化动作层。**借思想，拒平台。**
