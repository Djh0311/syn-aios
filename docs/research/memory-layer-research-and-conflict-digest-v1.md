# 记忆层研究与冲突报告汇编 v1

日期：2026-06-16
出自：咨询线（Claude）。性质：把记忆层相关的**研究报告、冲突报告、研究谱系**梳理到一处。报告本体仍是各自文件；本文做导航 + 处置（disposition）记录 + 防混淆警告。**"已确定"的事实与决策不在本文**，在姊妹篇 `docs/memory-layer-consolidated-canon-v1.md`。

**一句话判据**：想知道"某份记忆研究研究了啥、结论咋样、采纳没采纳"，看 §2 对应条目的 disposition；想知道"研究结论变成正本没"，看姊妹篇 §6。

## 0. 这份文档是什么

- 是记忆层研究/冲突报告的**索引 + 处置记录**，不是研究本体、也不是正本。
- 关键作用之一：标清哪些研究**已吸收**、哪些**借零件不借范式**、哪些**根本不属记忆层**，并防"第二轮"命名撞车（§3）。

## 1. 谱系时间线

```
2026-06-03  记忆层 round-1 深挖（自有原型）        → 已吸收，落成 M1 切片 + 设计对象表
2026-06-04/05 odysseus/gepa/paseo 参考研究        → 工作台/运行时/优化层，非记忆，已停泊
2026-06-11  设计文档 baseline：§3 外部项目"第二轮"  → 9 个图谱/理解类项目，已吸收进设计正本
            + R3 schema 契约冻结（17 张记忆表）
2026-06-14  round-2 深挖报告（SQLite/编排）        → 借零件、拒范式
            + 三项目 vs 正本 冲突报告              → 即拒范式的判定本体
2026-06-16  R5 收口：设计文档 +15 行 §1.1          → 蓝图经验沉淀口径（与上述 SQLite 研究无关）
```

## 2. 各报告逐条（含处置）

### 2.1 记忆层 round-1 深挖（2026-06-03）— 已吸收
- 文件：`evidence/2026-06-03-memory-layer-deep-research-and-implementation-slice-v1.md`（+ handoff）。
- 研究对象：项目**自有原型记忆代码** + 设计文档（非外部 GitHub 项目）。
- 结论：定义记忆层必须包含 ObservationStore / MemoryCandidate / MemoryPage / MemoryVersion / MemorySourceRef / MemoryLintFinding / TaskMemoryPacket；明确记忆层 ≠ 聊天摘要/向量库/图谱/Obsidian vault/markdown 文件夹/当前候选 JSON；把 `agent-memory-governance.md` 降级为 harness 参考。
- **处置：已吸收** → 成为 M1 实现切片，并给设计文档的对象表打底。

### 2.2 设计文档 §3「外部项目第二轮结论」（2026-06-11 baseline）— 已吸收
- 位置：**就在** `docs/memory-layer-design-v1.md` §3.1–3.9 内，无独立报告文件。
- 研究对象：GBrain、GraphRAG、Graphify、LLMWiki、AgentMemory、MAGMA、CodeGraph、Understand Anything、Everything-Claude-Code（图谱/理解类）。
- 结论：逐项"可吸收 / 不能照搬 / 落地到本设计"映射，全并入七层与核心对象。
- **处置：已吸收进设计正本。** ⚠️ 见 §3 命名撞车。

### 2.3 round-2 深挖报告（2026-06-14）— 借零件 / 拒范式
- 文件：`docs/research/2026-06-14-memory-agent-research-v2.md`（266 行，《第二轮深挖报告：编排候选对决 + SQLite 落地》）。
- 研究对象：**编排轴**（agency-agents 232 personas / agency-swarm 绑 OpenAI）+ **记忆/SQLite 轴**（SimpleMem 的 evolver/cross、mem0 核心 + OpenMemory）。12 条可证伪断言，10 确认 / 2 推翻。
- 产出：单 `memories` 表 DDL + status 列候选→正式门；"抄 OpenMemory 骨架 + evolver 富字段"。
- **处置：借零件 / 拒范式**（详见 §4，判定见 2.4）。

### 2.4 三项目 vs 正本 冲突报告（2026-06-14）— 拒范式的判定本体
- 文件：`docs/research/2026-06-14-three-projects-vs-canon-conflict-report-v1.md`（58 行，咨询线写）。
- 对比：round-2 的外部范式 × 正本（设计文档 §1–18 + 治理 schema + R3 契约）。
- 发现：记忆层冲突 A1–A12 + 编排层冲突 B1–B3。
- **判定**：单表 schema 是把 R3 的 17 表**降级**（拿浅换深）；"借证据和零件可以，借范式不行；schema 仍以 R3 契约为正本"。
- **处置：这份就是"拒范式"的决定本体**；该决定现已钉入姊妹篇 `docs/memory-layer-consolidated-canon-v1.md` §6.1。
- 边界：该报告读透了设计 §1–18 + 治理 schema + R3 契约；设计 §19–26（知识库边界等）未读，其结论不依赖之。

### 2.5 round-2 的 round-1 前身 — 不在仓库
- round-2 报告自述"我上一轮说得不够准"，存在一个 round-1 SQLite/编排前身，但它当时落在**非 git 的 `syn-research/`**，只有 round-2 被正式拷进 `docs/research/`；本仓库无 `syn-research/` 目录，故前身**不可追**。如需谱系完整性，这是一处缺口。

### 2.6 odysseus / gepa / paseo 参考研究（2026-06-04/05）— 非记忆，停泊
- 文件：`docs/research/2026-06-04-odysseus-...`、`2026-06-05-odysseus-...v2`、`2026-06-05-odysseus-vs-final-blueprint-...`、`2026-06-05-gepa-...v2`、`2026-06-05-gepa-...optimization-layer-recommendation`、`2026-06-05-paseo-...`。
- 主题：工作台架构 / 多 agent 运行时 / 提示优化（GEPA）。各自登记在 `docs/workbench-system-architecture-v1.md`，**不属记忆层**。
- **处置：停泊**（GEPA 明确为"后置优化层候选"，见 CURRENT.md）。**梳理记忆层时应排除这几份。**

### 2.7 记忆层研究 + 蓝图平等对抗 咨询交付（2026-06-16，最新最全）
- 文件：`docs/research/2026-06-16-memory-layer-research-and-blueprint-adversarial-handoff-v1.md`（母本在 `syn-research/`，咨询线 2026-06-16 并入此处留底；168 行）。
- 是什么：把全部 5 轮研究（约 14 个外部项目 + 9 个早期参考）+ 末轮"蓝图 vs 外部范式 9 轴平等对抗"汇成的**结构化交付**，专为咨询线"梳理 + 排期"设计。是 2.3 / 2.4 的**升级合订版**——后续看记忆研究，看这份即可。
- 结论（印证 canon）：① 外部项目全部"借零件、拒范式"，反复独立印证 canon（多层非单表、向量 = 可重建缓存、必须候选→正式门）；② 9 轴全判 context-dependent——蓝图赢方向 / 不可逆性 / 终局多 agent 安全，外部赢当前实现量 / 首价值时间 / 薄数据召回；蓝图真正的问题不是"做了门"，而是把不可逆 schema 维度与此刻过重的执行捆成一次性全量上线、且把语义召回押到最后。
- **待采纳前瞻路径（Part A，排期输入，未并 canon）**：🔒 第一天锁死 4 条不可逆地基（污染前隔离 + 子 agent 禁写正式 / append-only 不覆盖 / `model_export` 维度 / 权威-缓存分缝）；🟢 第一版只建约 7–8 表轻门库；⏩ 把语义召回前移进第一阶段（向量是可重建缓存、前移不破冻结契约）；⏸ 其余按 `governance_level` 开关 + 触发条件渐进启用（触发须**同时**覆盖"多写者"与"外发面"）；Part A′ 给了 4 相位阶段骨架。
- 一条实测事实（值得记）：当前**真实正式记忆 = 0 条**，memory 域 JSON 全是 `fixtures/r3-a*` 合成数据（机器建好、尚无真实数据）。
- 外部工具安全提示（别误用）：caveman `caveman-compress` 就地改写真实文件 + 发云；houtini-lm 有未修的任意文件读取 PR——均列为"拒"，仅借其可逆压缩信封 / 路由零件。
- 处置（2026-06-16 更新，用户拍板）：跨模型红队（GPT + Claude）后，**"当前 17 表全形态对单人阶段过度"已被采信并拍板**，见 `decisions/2026-06-16-memory-layer-form-acknowledgement-and-use-driven-trim-v1.md`。路线改为**真用观察 → 期末主动过度审计 → 趁数据少据实裁**；前瞻路径（轻门 / 触发器 / 语义召回前移）是**那次审计的输入**，不是现在的建库任务。原"保持全形态"结论已推翻。

## 3. ⚠️ "第二轮"命名撞车（务必分清）

有两个都叫"第二轮"的东西，**研究对象不相干、处置相反**：

| | 设计文档 §3「外部项目**第二轮**结论」 | 2026-06-14「**第二轮**深挖报告」 |
|---|---|---|
| 时间 | 2026-06-11（baseline） | 2026-06-14 |
| 对象 | GBrain/GraphRAG/AgentMemory… 图谱/理解类 | mem0/OpenMemory/evolver/SimpleMem SQLite + 编排 |
| 处置 | **已吸收**进设计正本 | **拒范式**、只借零件 |

风险：有人扫到"第二轮已吸收"，会**误以为 SQLite 单表研究也进了设计文档**——它没有。

## 4. 借了什么 / 拒了什么（round-2）

- **拒（范式级）**：单表 schema 覆盖多表；自动捕获 + 即时提交；单 actor 状态翻转当确认门；子智能体/LLM 自治写正式记忆。
- **借（零件 / 证据）**：候选→正式门方向获独立佐证；候选延迟 embedding；content-hash 幂等（我们已有）；SQLite + 向量同库可行性证据。
- 这些"借"是**信号/参考**，不改 R3 的 17 表正本。

## 5. 结论归属

- 由本组研究/冲突报告导出的**正本级决定**（schema 正本=R3 契约、外部借零件不借范式、agent-memory-governance 降级）已写入 `docs/memory-layer-consolidated-canon-v1.md` §6。
- 本文只保管**研究过程与处置记录**；研究报告原件继续留在 `docs/research/` 各自文件。
