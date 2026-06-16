# 记忆层 · 外部研究 + 蓝图平等对抗 · 咨询线交付与排期建议 v1

- 日期：2026-06-16
- 出自：研究/咨询线（Claude）
- 性质：研究发现 + 改进建议汇总，供咨询线**梳理 + 后续开发方向阶段排期**。
- 建议落地位置：`product-line/docs/research/`（与 `2026-06-14-three-projects-vs-canon-conflict-report-v1.md`、`2026-06-14-memory-agent-research-v2.md` 同处）。本文先存于研究区，由用户/咨询线决定何时按流程并入。

---

## 0. 状态分栏（务必先读，别把建议当 canon）

| 层级 | 内容 | 权威性 |
|---|---|---|
| **已采纳 canon**（本文不改） | 蓝图 `local-ai-workbench-blueprint-v1.md` §17；`memory-layer-design-v1.md` 七层 31 对象；R3 冻结 **~17 表契约**；记忆切片 **M1–M13 已实现并冻结**（`accepted_with_deferred_items`） | 权威 |
| **研究发现**（实证、已对抗核实） | Part B / Part C | 事实，带来源 |
| **改进建议**（待决策采纳） | Part A | **未采纳**，排期输入 |

> **关键校正（必须随文带走）**：M1–M13 + 17 表是**已落地的冻结契约，不是纸面提案**。下文所有"瘦身/塌缩"类建议都是 **"对后续阶段的范围与启用节奏建议"**，不是"推倒已建的"。推倒已建是**新**浪费。建议按 **"17 张冻结正本 / 第一版建库范围 / 31 逻辑对象 / 最终形态"** 分栏理解，让加表按数据增长解锁，而非重来。

---

## Part A — 改进建议（排期输入，核心）

### A0. 一屏结论

研究跨 5 轮、覆盖约 14 个外部项目 + 9 个早期吸收参考，末轮做了一次 **"蓝图 vs 外部范式"平等对抗（9 轴）**。两条主结论：

1. **外部项目：全部"借零件、拒范式"。** 没有一个能当记忆模型采纳；它们反复独立**印证了 canon**（多层非单表、向量=可重建缓存、必须有候选→正式门）。真正可借的是几条**合法省 token 杠杆**。
2. **对抗分析：蓝图赢"方向 / 不可逆性 / 终局多 agent 安全"，外部赢"当前实现量 / 首个价值时间 / 薄数据召回"。9 轴全判 context-dependent。** 蓝图真正的错**不是"做了门"**，而是把 **"不可补的 schema 维度"** 和 **"此刻过重的执行（对象数/版本表/双写/召回顺序）"** 捆成一次性全量上线，**并把第一阶段最值钱的语义召回押到了最后**。

### A1. 🔒 锁死第一天（hold firm，不排期改动 —— 这 4 条蓝图凭实力赢外部）

这 4 条是"事后真补不回"的不可逆地基，**即便单用户薄数据期也必须第一天就位**：

1. **污染前隔离带 + 子智能体禁写正式记忆。** §7"已被模型读过的不可抹除"→ 污染一旦被另一 agent 读进上下文即**不可逆**；外部 `user_id` FK 只隔离"谁的记忆"，结构上挡不住"同域内 A 写脏给 B 读"。多 agent 一上线就补不回。
2. **append-only（不覆盖）+ 拒绝频率衰减驱动状态 + 审计不可物删。** 覆盖让"它改过"都查不到；频率衰减会优先淘汰最该留的低频安全反例/合规红线。注意：这条要的是**"不覆盖"的不可逆性**本身（可用最轻的 `status_history`/git 实现）——**重的"独立一等版本表 + 完整版本链对象"可后置**（见 A4），但不覆盖的语义不能丢。
3. **`model_export_policy` 外发/脱敏维度第一天进 schema。** 全套对照里**唯一"事后真补不回"的点**——接云模型那一刻缺这一维 = 确定性数据外泄通道；等接入再加列要回填全部历史记忆的 export 判定、无源可依。成本极低（一个字段，单用户期可空操作）。
4. **权威/缓存物理分缝（§18.7）。** `memory_id` 在 SQLite，向量/图/索引只是其**可重建派生**、命中≠事实。外部"语义命中那行就是记忆本体"，embedding 换代/库损坏即事实漂移。对"把成功运行沉淀成可复用资产"的系统，资产稳定身份比召回便利更根本。

### A2. 🟢 第一版建库范围（约 7–8 张"轻门精简库" ≈ design §11.1 阶段0）

只硬上：
`formal_memory_records` + `formal_memory_versions`（轻量起步，见 A4）+ `formal_memory_audit_events` + `memory_candidates`(+`events`) + `memory_source_refs`（退化为可空指针，默认填 `session_id`）+ `memory_scopes`（**scope 坍缩为 owner/agent 两档，`model_export` 默认 `local_only`**）。

显式后置：`observations`/`observation_events`、`entity_relations`、`lint×2`、`pattern×2`、`blackboard×2`、派生理解索引层、向量/图库 —— 按 §11.4–11.6 既定后置（这正是 §7/§29 自己警告过、却没在入口落地的范围契约）。

### A3. ⏩ 前移进第一阶段（最该改的一条）

**把语义召回（向量 + RRF/rerank）从阶段5 前移进第一阶段**，并把 §19.2 改成：
`先语义/FTS 召回候选 → 召回后挂一道轻量 scope/撤权/sensitive 出口闸 → topK`。

理由：第一阶段定位 = **"召回相似历史成功运行"本质就是语义检索**，关键词召回会**系统性漏召**用户最热的 `similar_case` 需求。向量已被 §18.4 定义为可重建缓存，**前移不动任何冻结契约、不破坏可信**。这是跨轴反复出现、最影响第一阶段产品价值却被押到最后的**自伤**。

### A4. ⏸ 后置 + 触发条件（`governance_level` 开关驱动）

| 后置项 | 触发启用条件 |
|---|---|
| 多角色按 scope 分级确认门（§16 七角色矩阵） | 出现**第二个并发写者** / 真有跨项目记忆要沉淀 |
| 独立 `formal_memory_versions` 一等表 + 完整版本链 | 单用户开始需要**回溯/审计取证**（在此之前：不覆盖用 `status_history`/git 化 sidecar 实现，A1#2 的不可逆性已保住） |
| SQLite + Markdown 双写及防绕过护栏（§18.1/§18.8） | 真有**人机共读**需求（薄数据期没人读那些页；它不增加可信度，只增加可漂移面） |
| 派生理解索引层（向量图 / code-index / understanding-map / GraphRAG） | §11.4–11.6 既定后置 |
| 强制人工确认门 | 仅保留给 (a) 用户偏好/全局蓝图 scope，(b) 任何 `sensitive>低` 或 `model_export≠local_only` 的记忆；其余单用户期**单击即晋升甚至自动晋升** |
| 出处/审计强制档（14 source_type + 21 类审计动作 + 非召回事件审计行） | 降级到 OpenMemory 量级（status 列 + status_history + 可空 source_ref）；完整档随记忆密度上调 |

> ⚠️ **触发条件必须同时覆盖两个独立维度**：**(a) 多写者出现**，**(b) 外发面出现**（首份私密知识库授权 / 首次接入云模型）。只写"出现第二个 agent"会在引入外发面那一刻仍处宽松档，正好踩中 §21 不可逆外发。

### A5. 跨 9 轴最集中的单一改动：`governance_level` 开关

把 **"schema 维度"** 与 **"门的执行严格度"** 正式**解耦成一个 `governance_level` 开关**，而非一次性全量上线：
- **schema 维度**（`status` / `scope` / `model_export` / `source_ref` / `authority`）—— **第一天就在**（A1 的不可逆占位）。
- **执行严格度**（是否每条偏好/全局强制人工确认、是否全 17 表齐活、是否双写、语义召回顺序）—— 由 **"活跃并发 agent 数 × 库内正式记忆条数 + 外发面"** 驱动**渐进收紧**。单用户薄数据期默认宽松。

---

## Part A′ — 建议阶段骨架（供咨询细化排期，不替代你们的 Stage 命名）

| 触发相位 | 启用内容 |
|---|---|
| **现在**（单用户·薄数据·评测吞吐） | 锁 A1 四条不可逆 schema 占位；建 A2 的 7–8 表轻门精简库；A3 语义召回前移；门=单击即晋升（除高敏感/外发/全局偏好） |
| 触发 **"多写者"**（第二个并发 agent 真写正式） | 启用多角色 scope 分级确认门；收紧执行严格度 |
| 触发 **"外发面"**（首份私密 KB 授权 / 首次接云模型） | `model_export` 执行严格化（schema 占位已在）；sensitive 出口闸 strict |
| 触发 **"记忆密度 / 回溯需求"** | 独立版本表 + 完整版本链；派生理解索引层（向量图/understanding-map）；双写 Markdown（若有人机共读需求） |

---

## Part B — 外部研究语料（借什么 / 拒什么，映射回 canon）

> **铁律（咨询线既有结论，本文沿用）**：借证据和零件可以，借范式不行。外部通病 = 自动捕获 + LLM 自主 + 单库简单 + 即时提交；canon = 受治理捕获 + 确定性控制 + 多层带出处。

### B1. 记忆类

| 项目 | License/成熟度 | 借（证据/零件） | 拒（范式） | 判定 |
|---|---|---|---|---|
| **mem0 / OpenMemory** | Apache-2.0 / 生产级 | OpenMemory 的 status 状态机 + `memory_status_history` + 身份 FK 骨架；user/session/agent 作用域 | 核心 ADD-only 自动抽取即提交；单 actor 翻 status | 借零件 |
| **SimpleMem**(evolver/cross) | MIT / 研究级 | evolver 证明"纯 SQLite + 内联向量 + FTS5"可行 + 富字段；cross WAL+FK | LanceDB 写死；`candidate.py` 是检索策略 decoy 非记录门 | 借零件 |
| **MemoryOS** | EMNLP'25 | 分级晋升策略（短→中 FIFO、中→长分页） | —（学思路） | 学 |
| **Thought-Retriever** | MIT / 研究 | 写入**双门**概念（LLM 置信 + 相似度去重） | 扁平 JSON、无身份连续性 | 学 |
| **NexSandglass** | MIT / 单人 | per-agent 目录隔离思路 | **文件当正本**（撞 §18 SQLite 权威）；自创术语 | 跳过集成/学 |
| **SurrealDB 簇**(agent-memory / remembrances-mcp / Spectron) | 无 license / MIT 开核 / 闭源 | 单引擎"向量+图+全文"混合；`DEFINE EVENT` 派生可重建；Spectron 三时态+supersession+provenance（learn） | 破坏性覆盖写（`DELETE`+`CREATE`）；朴素拼接检索；SurrealDB 当权威库 | 借零件 |
| **OpenViking** | AGPL-3.0 / 字节 2.5万★ | **L0/L1/L2 分层加载** → TaskMemoryPacket；`memory_diff.json` 审计形状 | `ExtractLoop` 自动抓会话；LLM 自主写/并/删无门；AGPL 引擎不可拷 | 借零件 |

### B2. 省 token 类

| 项目 | License/成熟度 | 借（零件） | 拒（范式） | 判定 |
|---|---|---|---|---|
| **caveman** | MIT / 7.3万★单人 | 保护段压缩器；**可逆压缩信封**（备份→校验→回滚）→ CompactionCheckpoint；诚实基准法；密钥外发 denylist + 安全文件 I/O | ⚠️ `caveman-compress` **就地改写真实 `CLAUDE.md`/记忆文件 + 发云** = §14/§17"自动压缩当正式记忆"反范式 | 借零件 |
| **claude-mem** | Apache-2.0（旧版曾标 AGPL，pin 版本）/ 8.2万★ | **3 层渐进披露召回**（索引→筛→按 id 取详情）；hooks→持久队列→异步 worker→蒸馏管线（**输出只落观察/候选**） | 自动抓每个工具调用 + LLM 自主写 + `permissionDecision:'allow'` 自动注入；README 绑 $CMEM 代币 | 借零件 |
| **houtini-lm** | 冲突(仓 MIT vs npm Apache) / 91★ | "**字节不进编排器上下文**"(`code_task_files`，**须加路径网关**)；确定性本地模型路由 + "建议不自动" | 无同意自动写计数器；⚠️ **未修的任意文件读取 PR #11** | 借零件 |

### B3. 真正值得拿走的 6 条合法省 token 杠杆（三/五个项目独立收敛 = 强佐证）

1. **分层/渐进加载上下文**（OpenViking L0/L1/L2、claude-mem 索引→取详情）→ 上下文工程 / 最小 TaskMemoryPacket
2. **字节不进编排器上下文**（houtini `code_task_files`，加路径限定+realpath 网关）→ 上下文工程
3. **可逆 + 带完整性校验的压缩信封**（caveman 备份→校验→回滚，**借信封、扔就地覆盖**）→ CompactionCheckpoint
4. **低风险任务路由本地模型**（houtini 确定性路由）→ 低风险路由本地模型（顺带：claude-mem 的蒸馏步骤本该跑本地而非付费 Claude）
5. **向量/索引 = 可重建缓存**（claude-mem、houtini）→ §18 现成存在证明
6. **诚实成本基准**（caveman 对照强基线 "Answer concisely."）→ 成本统计

### B4. 共同必拒范式（一句话）

**"把 LLM 输出自动写进单一无差别存储"**，三种递进形态：caveman 就地改写文件发云（最危险）→ claude-mem 自动抓+自动注入无门 → OpenViking 会话结束自动抓 → houtini 无同意计数器。canon 防御一致：LLM 推断/自动抓取**最多是 观察/候选/可重建缓存**；权威=SQLite+只读 Markdown；进正式永远过受控采纳门；worker 绝不写正式。

### B5. 已在 canon 里的早期参考（提示：勿重复研究）

`GBrain / GraphRAG / Graphify / LLMWiki / AgentMemory / MAGMA / CodeGraph / Understand-Anything / Everything-Claude-Code` —— 已被 `memory-layer-design-v1.md §3` 吸收并映射到对应层/对象，无需再立项研究。

---

## Part C — 蓝图 vs 外部 平等对抗（9 轴，详情）

> 规则：蓝图**不预设为权威**，与外部范式平起平坐；每轴一挑战者攻蓝图、一辩护者挺蓝图，独立裁判按"单用户·薄数据·评测环境第一阶段"实情判。裁判已实测磁盘坐实双方事实：**真实正式记忆 = 0 条**（memory 域 JSON 全在 `fixtures/r3-a*` 下、几百字节合成数据）。

| 轴 | 判定 | 一句话 |
|---|---|---|
| 捕获方式 | context-dependent | 方向取蓝图（污染前隔离带，外部 FK 给不出），实现量取外部（内核只需一表+status+history） |
| 信任/采纳门 | context-dependent | 门的**方向**蓝图对 + 高敏感/外发强制确认；门的**强制力**对项目级偏重、且把语义召回押后是自伤 |
| 存储形态 | context-dependent | 终局蓝图对（权威/缓存分缝 + 外发维度单库给不了）；薄数据起步外部对（单库内联召回更强、迁移近 1:1） |
| 检索 | context-dependent | 相似度该做第一入口（外部对），但召回后保留 export/撤权/sensitive 出口闸（蓝图对）；意图路由+5 套优先级延后 |
| 更新与遗忘 | context-dependent | append-only + 拒频率衰减蓝图对且 hold firm；但达成它选了最重一端（一表+history 即够） |
| 出处/审计 | context-dependent | 不可覆盖版本化 + 污染闸门方向蓝图对；强制每条 14-source + 21 审计 + 非召回审计 + 双写是档位错 |
| 摩擦 vs 累积 | context-dependent | 非对称/可逆性蓝图对（有门可降级成无门）；但空库悖论今天真成本、召回押后自伤 |
| 复杂度/YAGNI | context-dependent | 攻"开局铺满 31 对象"是 §11/§29 自己也推后的稻草人；但治理三件套+候选门+禁写正式必须第一天在；收敛到 7–8 表轻门库 |
| 多 agent 安全 | context-dependent | 今天重治理 YAGNI，明天隔离门不可替代；唯一不可补的 `model_export` 维度第一天就上、严格度渐进 |

**最深张力（不可调和内核）**：两套范式服务于**不同时间坐标**——项目的"**当下**"（单用户·薄数据·评测吞吐）和"**定位**"（多 agent 控制平面·失败样本喂回训练）分别坐落其上。一对方向相反的不可逆：**写入侧**外发/已读/覆盖"发生即不可撤销"（§21）→ 逼治理在污染前就位 → 推向"第一天建门"；**预算侧**空转治理骨架的成本是沉没的、每天付摩擦税 → 推向"先蓄水再修闸"。**唯一同时尊重两者的解：把"不可逆维度"（schema 占位，第一天锁死）与"可逆优化"（严格度/对象数/双写/召回顺序，渐进加载）物理切开**——9 轴反复收敛到这同一条解。

---

## Part D — 风险与边界（随交付一起带）

1. **唯一真正不可逆的错**：把"轻门精简库"误读成向外部单库投降，**顺手砍掉 `model_export` 外发占位或权威/缓存分缝**。砍对象数与双写可以，砍这两条不行。
2. **反向过度修正**：因"方向是蓝图对"就把渐进触发条件设得形同虚设、继续在薄数据单用户期全量铺 31 对象+多角色门+双写+逐条审计 → 治理持续领先于数据，每天付冷召回与确认仪式摩擦税。
3. **冻结的 M1–M13 沉没成本会被两向滥用**：被用来拒绝任何瘦身（"已冻结不能动"）→ 锁死过重形态；或被忽略（仍按"为空库预建"打）→ 误估第一阶段重量。正解 = 分栏写死"17 张冻结正本 / 第一版建库范围 / 31 逻辑对象 / 最终形态"。
4. **语义召回继续押后** = 不流血但持续失血的产品价值损失；向量是可重建缓存、前移不破任何冻结契约，无理由不前移。
5. **渐进开关触发条件**必须同时覆盖"多写者出现"与"外发面出现"两个独立维度，否则引入外发面那刻仍宽松档、踩中 §21 不可逆外发。
6. **别把"省 token 90%"当架构权衡依据**：已核实多为注水自测（真实约 10–20%），且 token 节省与捕获治理正交（分层加载/候选延迟 embedding 蓝图可自行叠加）。

---

## 来源

**外部项目**：[mem0](https://github.com/mem0ai/mem0)（+`openmemory/api`）· [SimpleMem](https://github.com/aiming-lab/SimpleMem) · [MemoryOS](https://github.com/BAI-LAB/MemoryOS) · [Thought-Retriever](https://github.com/ulab-uiuc/Thought-Retriever) · [NexSandglass](https://github.com/lovevin1314-tech/NexSandglass-Agent-DedicatedMemory) · [surrealdb/agent-memory](https://github.com/surrealdb/agent-memory) · [remembrances-mcp](https://github.com/madeindigio/remembrances-mcp) · [OpenViking](https://github.com/volcengine/OpenViking) · [caveman](https://github.com/JuliusBrussee/caveman) · [claude-mem](https://github.com/thedotmack/claude-mem) · [houtini-lm](https://github.com/houtini-ai/lm)

**canon 一手源**：蓝图 `…/docs/architecture/local-ai-workbench-blueprint-v1.md`（§17）· `product-line/docs/memory-layer-design-v1.md` · `product-line/docs/plans/2026-06-11-…r3-sqlite-schema-importer-rollback-contract-v1.md` · `product-line/docs/plans/memory-layer-implementation-slice-v1.md` · `product-line/docs/research/2026-06-14-three-projects-vs-canon-conflict-report-v1.md` · `product-line/docs/research/2026-06-14-memory-agent-research-v2.md` · `product-line/CURRENT.md`

---

*本文为研究/咨询线交付物，含未采纳建议；并入 canon 前请按 product-line 流程（task package + 复核）处理。*
