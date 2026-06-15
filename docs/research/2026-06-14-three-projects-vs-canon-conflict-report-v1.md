# 三项目理论方法 × 我们正本 — 冲突报告 v1

日期：2026-06-14
出自：咨询线（Claude）。
性质：把第二轮研究的三/四个外部项目（agency-agents、agency-swarm、SimpleMem、mem0/OpenMemory）的**理论方法**，逐条对照我们的记忆层正本（`docs/memory-layer-design-v1.md`、`docs/plans/2026-06-01-memory-governance-schema-v1.md`、R3 SQLite 契约 `docs/plans/2026-06-11-...r3-sqlite-schema-importer-rollback-contract-v1.md`）与编排相关 decisions，找**冲突点**。研究本身价值见第二轮研究报告；本文只看"哪里跟我们撞"。

**一句话判据**：借证据和零件可以，借范式不行；尤其记忆层——别拿外部的单表 schema 覆盖我们已冻结的 R3 多表契约（拿浅的换深的）。

> **关键前提（重定义整件事）**：我们不是"还没有记忆层 SQLite 设计、要靠研究补"。我们**已有一份冻结的 R3 SQLite 契约**，记忆域是**约 17 张表**；研究那份"单 `memories` 表 + 状态枚举"恰是把它**塌缩成两张、丢掉其余十五张**。所以多数冲突，是研究的方案在**降级**我们的正本，不是反过来。

## 总纲：范式对立

- 外部项目内核假设：**自动捕获 + LLM 自主 + 单库简单**。
- 我们正本内核假设（反的）：**受治理捕获 + 确定性控制 + 多层带出处**。
- 故："借"只能借证据和零件，**永远别借范式**。

## Part A — 记忆层冲突

### 🔴 范式级（动了根）
- **A1 单表 vs 七层/十七表**：研究=一张 `memories` 表 + status 列；正本=七层（观察/候选/正式/关系/派生/检索/审计，`memory-design §4`），R3 契约 §3.3 已落成约 17 张分表（formal_record / formal_version / formal_audit / candidate / candidate_event / observation / observation_event / capture / scope / source_ref / lint / entity_relations / patterns / blackboard…）。研究把 formal+candidate 合一、其余 15 张没有 → 是降级不是简化。
- **A2 即时提交/自动抽取 vs 默认不入、子智能体禁写正式**：mem0/SimpleMem 核心自动抽取+即时 commit；正本头号硬规（`§14`/`§11.7`）——普通聊天不自动入、子智能体不能直接写正式记忆、LLM 推断不直接成权威。要 vendor 的项目代码**骨子里就是自动入库**；研究的"一律先写候选"是在修正项目、不是采纳项目。
- **A3 状态翻转 vs 多角色确认门**：研究/OpenMemory=`UPDATE status='formal'` 单 actor 翻一下；正本=候选与正式**两套存储**，晋升是多角色、按 scope 分级（`§6`/`§16`/治理 §10）——用户偏好/全局→**必须用户确认**，项目→项目主管；且 `candidate_confirmed` ≠ `memory_active`（治理 §10 明文禁止自动互转）。研究的门**必要但不充分**。

### 🟠 缺层/缺维（少了整块）
- **A4 来源**：研究只有 `session_id` 出处；正本要求每条正式记忆**必须**带 `memory_source_refs`（14 种 source_type + authority_level + sensitive_level，`§3`/治理 §7/R3 §3.6），`§14`"无来源只能候选"→ 按正本研究那些记忆**全部只能是候选**。
- **A5 作用域/外发（安全冲突）**：研究只有 user/agent/session FK；正本 `MemoryScope` 有 7 种 scope + **`model_export_policy`(local_only/redaction/blocked)** + 权限（治理 §6，R3 §3.6 要求保留）。研究 schema **无外发/脱敏维度，挡不住"私密/secret 进外发模型上下文"**。
- **A6 关系层缺失**：研究只有 supersedes；正本有 entity/temporal/causal/semantic 四类关系、LLM 因果默认候选（`§4.4`/`§5.10`）。研究答不了 `why`/`impact`/`timeline` 检索意图（`§9`）。
- **A7 版本**：研究=`updated_at`+`supersedes` 行内更新；正本 `formal_memory_versions` 是独立一等表、永不覆盖、改即新版本（`§5.9`/`§7`/R3 §3.3）。
- **A8 观察/capture/lint/黑板候选全缺**：正本把 ObservationStore、capture events、memory-lint、黑板候选（≠记忆候选，治理 §1.12）都分开；研究一律塞进"memory"。

### 🟡 细节（可修）
- **A9 向量内联 vs 向量是可重建缓存**：evolver 把 `embedding_json` 内联进主表、研究照抄；正本 §18.4（已确认）向量库=可重建缓存、不进权威表、不能当唯一源。
- **A10 扁平 importance 排序 vs 按问题分来源**：研究 hydration=`ORDER BY importance`；正本 §15 明说"记忆层不能做成一条简单排行榜"，优先级随"问什么"变。
- **A11 access_count/reinforcement 衰减 vs 状态不靠频率**：正本 §3.5/§3.6 不用访问频率/相似度替代状态、不对审计事实 decay。这些字段只能当信号，不驱动状态/衰减。
- **A12 Markdown 展示层缺失**：正本 §18.1（已确认）双层=SQLite + Markdown 人机共读页；研究纯 SQLite，少了这层（且关系到"念旧/人读"内核）。

### 🟢 其实吻合（验证，不是冲突）
- SQLite 当结构化权威库：正本 §18 早定，研究同意=验证。
- 候选→正式门是核心：我们早设计了；研究独立收敛到同一结论=强力佐证方向对。
- "没有库现成带这道门"：证明这道门是我们的净新增（且已设计）。
- 迁移机制（单事务/幂等/回滚）：R3 契约 §5 写得**比研究深**（含 outbox、崩溃注入点、脱敏 manifest）；研究是简化重 derive。

## Part B — 编排层冲突

- **B1 LLM 涌现委派 vs 确定性"角色即节点"**：agency-swarm=LLM 在固定拓扑上非确定性 tool-call；正本（decisions `codex-workflow-min-model`、`editable-canvas-codex-as-director`、`project-workflow-canvas-authority`）=agent 当工作流节点、确定性 order/branch/parallel、主管掌控。非确定性撞画布/主管模型。
- **B2 OpenAI 托管 vs 本地优先**：agency-swarm 硬绑 OpenAI Responses API（第二轮已核实：`codex_input.py:11` CODEX_BASE_URL=chatgpt.com/backend-api/codex、`openai-agents==0.14.8`、Popen 全在 openclaw 网关、零本地 CLI exec）；项目内核是本地 agent 编排。直撞。
- **B3 自由 persona vs 职位档案约束**：agency-agents 只硬性 `{name,description,color}`；正本（CLAUDE.md"职位档案约束职位"、记忆 §16 角色权限）角色是带权限的治理职位。232 个松散 persona 不能直接用，须映射到权限角色上。
- 🟢 "调度器自建 / agency-swarm 不是调度器" 与需求一致，不是冲突。

## 底线

冲突的根只有一条：**它们为"自动+自治+单库"优化，我们为"治理+确定+多层带出处"立本。** 借证据和零件，绝不借范式。尤其记忆层：**别用研究的 `memories` 单表 schema 覆盖 R3 契约——那是拿浅换深。** 研究的价值是①验证 candidate→formal + SQLite 方向对，②几个可借细节（候选延迟 embedding、content-hash 幂等我们已有、sqlite+向量可行性证据）。schema 仍以 R3 契约为正本。

## 核实说明 / 边界

- 记忆层正本读透 `memory-layer-design §1–18` 全 + 治理 schema 全文 + R3 契约全文；`§19–26`（知识库边界等）未读，结论不依赖之。
- 编排轴据第二轮已核实研究 + decisions 清单 + CLAUDE.md 原则；架构/愿景正本未逐字读，B 系列结论按此口径。
- B2 的 OpenAI 绑定、OpenMemory 的 SQLite+状态机+FK、SimpleMem 的 evolver/cross 真 sqlite3，均由咨询线对 clone 的真代码核实命中（2026-06-14）。
