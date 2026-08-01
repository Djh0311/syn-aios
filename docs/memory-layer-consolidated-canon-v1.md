# 记忆层已确定汇编（正本索引）v1

日期：2026-06-16
出自：咨询线（Claude）。性质：把记忆层**已确定**的事实与决策梳理到一处，做导航入口；**不替代**详细正本（设计文档 / R3 schema 契约 / 治理 schema / 历史状态快照），每条都带指针指向源文件。今天的当前状态只看 `docs/harness/CURRENT.md`。

> **2026-08-01 正本修订：**`decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md` 已覆盖本文关于“普通聊天一律不捕获、正式记忆必须逐条人工确认、auto-skillification 全部 deferred”的默认口径。七层模型、17 表目标、来源、作用域、敏感性、版本、冲突、外发、append-only 审计和权威 / 索引分离继续有效。本文中的旧实施状态仍按当时事实阅读，不得改写成自动治理已经落地。

**拍板摘要**：本文做两件事——①把散在多份文档里的记忆层"已确定"事实汇成一份可导航的索引；②把目前**只活在研究/冲突报告里、没进 canon** 的两条决策正式钉入正本：(a) 记忆 schema 正本 = R3 契约 §3.3 的 17 张表，2026-06-14 round-2 研究提的"单表 schema"评估后**不采纳**（判为降级）、只借零件不借范式；(b) `docs/agent-memory-governance.md` 是 harness/工程 agent 记忆治理参考，**不是产品记忆层权威**。代价：纯文档、零代码、低风险。不批的后果：这两条决策继续漂在非控制文档里，下次有人没翻到冲突报告，可能重走评估、甚至误用单表 schema 覆盖 R3 正本。

**一句话判据**：想知道"记忆层某事定没定、正本在哪"，看本文对应小节的指针；想知道"某外部方案要不要采纳"，默认**借证据零件、不借范式，schema 以 R3 契约为正本**。

## 0. 这份文档是什么

- 是**索引/汇编**，不是新正本。详细内容仍以下列源文件为准。
- 研究报告、冲突报告、研究谱系**不在本文**，在姊妹篇 `docs/research/memory-layer-research-and-conflict-digest-v1.md`。
- 适用范围：产品记忆层（工作台的记忆功能）。不含工程 agent 跨会话记忆治理（那是 `agent-memory-governance.md`，见 §6.2）。

## 1. 层模型（七层）

记忆层分七层。正本：`docs/memory-layer-design-v1.md` §4（行 352–482）。

1. 观察层（§4.1）2. 候选层（§4.2）3. 正式记忆层（§4.3）4. 关系层（§4.4）5. 派生理解索引层（§4.5）6. 检索层（§4.6）7. 审计层（§4.7）

## 2. Schema 正本：R3 契约的 17 张记忆域表

**记忆域 schema 的冻结正本 = `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md` §3.3（行 101–123）。** 共 17 张表：

`memory_scopes` · `memory_source_refs` · `formal_memory_records` · `formal_memory_versions` · `formal_memory_audit_events` · `memory_candidates` · `memory_candidate_events` · `observations` · `observation_events` · `memory_capture_events` · `memory_lint_runs` · `memory_lint_findings` · `memory_entity_relations` · `mature_pattern_candidates` · `mature_pattern_audit_events` · `blackboard_candidates` · `blackboard_candidate_audit_events`

关键约束（§3.3 行 121–123）：候选采纳必须把 `memory_candidates.candidate_key` 链到 `formal_memory_records.memory_id` / `formal_memory_versions.version_id` / `formal_memory_audit_events.audit_event_id`；observation→candidate 必须把 `observations.observation_key` 链到 `memory_candidates.candidate_key`。脱敏：R3 §3.6——记忆体保留 `sensitive_level` 与 `model_export_policy`。

**逻辑对象 vs 物理表澄清**：设计文档 §5 列了约 31 个逻辑对象（含 code-index / understanding-map / cluster-report 等只设计、未进冻结 schema 的）；R3 的 17 张表是**记忆/观察域的物理冻结子集**。两边数目不同**不是冲突**，是"全部逻辑设计"与"已冻结物理表"的区别。

## 3. 核心治理规则（按 2026-08-01 修订）

正本：设计文档 §2 / §14 / §11.7 / §16；治理 schema `docs/plans/2026-06-01-memory-governance-schema-v1.md`（状态：草案待确认）。要点：

- 秘书 / 记忆机制可以从对话、事件、项目结果和用户纠正中自发形成 CaptureEvent、Observation 和 Candidate；不是每条聊天都捕获。
- 子智能体**不能**直接写正式记忆（§14 行 1658、§16.6）。
- Candidate → FormalMemory 必须经过来源、scope、敏感性、冲突、时效、外发和策略治理；低风险类别可按已冻结政策自动沉淀，高影响、冲突或推断类进入用户 / 项目 owner 决定。当前代码仍以确认链为主。
- 每条正式记忆**必须**带来源引用（14 种 source_type + authority_level + sensitive_level）；无来源只能当候选（§3 行 84、§5.3、§14）。
- 作用域/外发：`MemoryScope` 7 种 scope + `model_export_policy`(local_only / redaction / blocked)（§5.1；R3 §3.6 要求保留）。
- LLM 推断的关系（尤其因果）默认候选（§4.4、§5.10、§17.4）。
- 版本永不覆盖：改即新版本；审计不可物理删除（§5.9、§7、§18.2）。

## 4. 存储架构（已确认）

正本：设计文档 §18。

- §18.1 **双层**：SQLite = 权威结构化状态/权限/版本/关系/审计；Markdown 展示页 = 人可读视图、共享同一 `memory_id`，**不是知识库笔记**，不能在状态机外被编辑成正式变更。
- §18.4 向量库 = **可重建缓存**，不进权威表、不能当唯一源。

### 4.1 当前实施注记（2026-07-14）

SQLite 内的记忆表是 2026-07-13/14 的历史导入快照，不是实时镜像；当前 memory stores 仍由 JSON sidecar 主写。是否把记忆 store 纳入 `db_primary` 另开任务决定，本切片不切换读写路径。

## 5. 实现状态：M1–M13（`accepted_with_deferred_items`）

历史实现依据：当时的 `CURRENT.md` 快照（M1–M13/M12.1 逐条口径）+ 终验冻结 `evidence/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`。今天的当前实现以 inventory 和 `docs/harness/CURRENT.md` 为准。摘要：

- **M1** 正式记忆 store + 审计骨架；显式创建才生成正式记忆，`candidate_confirmed` 不自动转正式。
- **M1.1** 正式记忆上下文绑定 guard（project_id/workflow_id/scope 校验）。
- **M2** 候选→正式 受控采纳；用户确认类候选不可绕过。
- **M3** ObservationStore + 工作流观察入口；当时普通聊天自动捕获被拒。2026-08-01 目标方向已修订，但当前实现未因此自动改变。
- **M4** TaskMemoryPacketBuilder + 预览；候选/观察只作复核材料、不进 included。
- **M5** 冲突 + memory-lint 最小阻断（阻断 finding 挡采纳与召回）。
- **M6** 工作流任务包注入 + 首条端到端闭环（无真实 worker）。
- **M7** 记忆中心 UI 最小入口（只读展示）；真机截图验收 deferred。
- **M8** 知识库 / Obsidian 兼容接口 placeholder + 边界；原生同步 deferred。
- **M9** 正式记忆生命周期操作（改/废/冻/解冻/归档/合并/拆分/升降 scope + 版本审计）。
- **M10** 实体与关系治理；LLM 因果留候选；图数据库 / GraphRAG deferred。
- **M11** 维护 job + memory-lint（只报告、不自动改记忆）。
- **M12 / M12.1** 成熟模式 + 跨项目记忆 + M1–M12 集成验收 + 验收摘要新鲜度；当时 auto-skillification / GraphRAG / 真实 Codex deferred。当前允许生成受治理 SkillCandidate / SkillDraft，但尚未实现。
- **M13** 最终权威验收，结论 `accepted_with_deferred_items`。

## 6. 本汇编钉入正本的关键决策

以下两条此前**只在非控制文档里**，现正式纳入正本（详证见姊妹篇研究/冲突汇编）：

### 6.1 记忆 schema 正本 = R3 契约；外部"单表 schema"不采纳

- 2026-06-14 round-2 研究（`docs/research/2026-06-14-memory-agent-research-v2.md`）提议把记忆落成"单 `memories` 表 + status 列"的 SQLite schema（抄 mem0/OpenMemory/evolver）。
- 冲突报告（`docs/research/2026-06-14-three-projects-vs-canon-conflict-report-v1.md`）评估判定：该单表方案是把已冻结的 17 表记忆 schema **降级**（塌成两张、丢 15 张），属"拿浅换深"。
- **决定（2026-06-16 修正，用户拍板）**：17 表 / 七层是**终局形态设计、成立**；但**对当前单人、0 真实记忆阶段偏重、被架空**。故 **(a)** 不采纳外部"单表拍平"（丢掉 `model_export`/`scope`/`status`/`source_ref` 等不可逆维度）；**(b)** 也**不把"现在建满 17 表全形态"当要求**——它是终局目标、非当前任务；**(c)** 路线改为**真用观察 → 期末主动过度审计 → 趁数据少时据实裁**，不预砍、也不"继续建满"。完整决策见 `decisions/2026-06-16-memory-layer-form-acknowledgement-and-use-driven-trim-v1.md`。
- **保留（不可逆，任何裁剪不得砍）**：子 agent 不直接写正式 / append-only + status history / `model_export` 外发维度 / 权威-缓存分缝。旧“逐条用户确认门不可裁”已由 2026-08-01 修订为“任何晋升必须经过可审计政策；高影响类别仍需确认”。
- **已认可可借的零件**（来自冲突报告"其实吻合/可借"）：候选→正式门方向（外部独立收敛到同一结论=佐证我们方向对）、候选延迟 embedding、content-hash 幂等（我们已有）、SQLite + 向量可行性证据。这些是**信号/参考**，不改 17 表正本。
- **2026-06-16 修正来由**：此前"保持 17 表全形态"的结论，经跨模型红队（GPT[记忆层原设计者，被要求攻己] + Claude[独立]）判定为**自评 + 承认过度却不改动作**，已推翻——见上"决定（修正）"。研究三件的前瞻路径（轻门 / 触发器 / 语义召回前移）作为**未来"过度审计"的输入**，不是现在的建库任务；研究处置见 `docs/research/memory-layer-research-and-conflict-digest-v1.md` §2.7。

### 6.2 `agent-memory-governance.md` 是 harness 参考，不是产品记忆层权威

- `docs/agent-memory-governance.md` 管的是**工程 agent 跨会话记忆**（什么能记/不能记），属 harness/工程治理；**不是**产品记忆层（工作台记忆功能）的设计权威。此前该降级只在 2026-06-03 round-1 研究证据里说过，现纳入正本。

## 7. Deferred 与当前新方向

GraphRAG、向量数据库、图数据库、完整理解图谱、原生 Obsidian 同步、vault 自动扫描、真实 worker / 真实 Codex 自动执行仍为 HOLD，各需另开任务包与对应授权。2026-08-01 已批准“每日整理生成 SkillCandidate / SkillDraft，并在治理、验证、版本和权限门后启用”的产品方向；具体自动策略和启用阈值仍需实施合同，外部动作 Skill 永不自动扩权。

### 7.1 Syn 记忆注入面边界（2026-07-14）

Syn 记忆进入 worker 的唯一合法面是任务包的冻结 prompt block。本切片不读写 `~/.codex/memories`，不修改 worker prompt 注入实现，不触 capture/source_type 词表或双召回结构。

## 8. 文档地图（谁拥有什么）

| 主题 | 正本 |
|---|---|
| 层模型 / 对象 / 治理 / 存储架构 | `docs/memory-layer-design-v1.md`（设计正本，3250 行） |
| 记忆域物理 schema（17 表） | `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md` §3.3 |
| 早期治理对象 schema | `docs/plans/2026-06-01-memory-governance-schema-v1.md`（草案待确认） |
| M1–M13 历史实现口径 | 当时的历史 `CURRENT.md` 快照 + `evidence/2026-06-05-memory-layer-m13-final-...-v1.md`；今天的状态看 `docs/harness/CURRENT.md` |
| 实现切片来源 | `docs/plans/memory-layer-implementation-slice-v1.md` |
| 研究 / 冲突报告 / 谱系 | `docs/research/memory-layer-research-and-conflict-digest-v1.md`（姊妹篇） |
| 工程 agent 记忆治理（非产品） | `docs/agent-memory-governance.md` |

> 当前入口见 `docs/harness/AUTHORITY.md`；本文继续只做记忆层索引，并必须与 2026-08-01 记忆修订一起阅读。
