# Memory Layer Implementation Slice v1

日期：2026-06-03

状态：草案，已基于记忆层权威设计完成深度复核；用于中间版本记忆层落地拆分。本文不是产品代码实现，也不是自动授权真实写入正式记忆。

## 0. 先说薄弱点

- 当前工作台只完成了黑板候选和记忆候选 sidecar 的最小闭环，还没有正式记忆系统。
- `MemoryRecord` 当前只是目标类型，不是可写、可召回、可审计的正式记忆。
- 如果只做 SQLite 表、页面或候选确认，不能算记忆层完成。
- 如果把 Obsidian、向量库、图谱或 LLM 摘要放到第一目标，会偏离当前最重要闭环。
- 正式记忆写入会影响后续 agent 行为，必须由控制核心、来源、版本、权限、冲突和审计共同约束。
- 本文把任务分成中等大小批次，避免拆得过碎；但每一批仍必须有清楚验收。

## 1. 依据

最终设计权威：

- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `CURRENT.md`
- `tasks/README.md`

说明：

- `docs/memory-layer-design-v1.md` 是最终工作台记忆层设计的主依据。它来自对 GBrain、GraphRAG、Graphify、LLMWiki、AgentMemory、MAGMA、CodeGraph、Understand Anything、Everything Claude Code 等项目的研究提炼。
- `docs/agent-memory-governance.md` 不是最终工作台记忆层权威设计。它更像 harness / agent 工程治理规则，只能作为风险提醒和历史参考，不能覆盖最终记忆层设计。
- 如果 harness 记忆治理规则和 `docs/memory-layer-design-v1.md` 冲突，以最终工作台记忆层设计为准。

当前实现依据：

- `docs/agent-memory-governance.md`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`

## 2. 大白话定义

知识库是材料和思考空间。

记忆层是系统行为依据。

一段内容只有在被确认成正式记忆后，才可以影响项目主管怎么派活、worker 任务包里能看到什么、秘书怎么提醒、后续任务怎么继承上下文。

所以记忆层不是：

- 普通聊天摘要。
- agent 自己写的便签。
- 向量库。
- 图谱。
- Obsidian vault。
- Markdown 文件夹。
- 当前的 `memory-candidates.v1.json`。

记忆层是：

- 观察痕迹。
- 候选记忆。
- 正式记忆。
- 来源引用。
- 版本。
- 权限。
- 冲突和过期提示。
- 审计。
- 召回和任务包注入。

## 3. 已确认硬边界

- 普通聊天不自动进入长期记忆。
- worker 汇报不是正式事实。
- 项目主管结合证据确认项目内过程事实。
- 全局主管负责方案边界和最终结果，不逐条确认 worker 日常汇报。
- 用户确认高层方案、最终结果、用户偏好、跨项目影响、全局蓝图和高风险记忆。
- 秘书整理、提醒、解释和收纳想法；秘书不确认 worker 汇报、不判断项目过程事实、不写正式记忆。
- 候选记忆不能进入任务包冒充正式事实。
- 正式记忆必须有来源、状态、版本、权限、冲突检查和审计。
- 正式记忆修改不覆盖旧版本。
- `conflicted`、`deprecated`、`frozen`、`archived` 默认不进入 worker 任务包。
- 知识库内容、Obsidian CLI、Canvas、Bases、图谱和向量命中都不能绕过记忆状态机。

## 4. 当前基线

已经完成：

- `memory_governance.v1` schema 设计。
- `MemoryCandidate` sidecar：`memory-candidates.v1.json`。
- `FormalMemoryStore` sidecar：`formal-memories.v1.json`，包含正式 record、第一版 version 和 formal audit。
- `ObservationStore` sidecar：`observations.v1.json`。
- `MemoryLintStore` sidecar：`memory-lint.v1.json`，包含 deterministic finding 和 lint run。
- observation 可由项目主管生成 `candidate_needs_review` 记忆候选，observation 回链 `candidate_key`。
- 低风险本项目 `candidate_confirmed` 可由项目主管受控采纳为正式记忆；用户偏好 / 全局 / 跨项目 / 高风险仍受确认边界约束。
- `TaskMemoryPacketBuilder` 和任务记忆包预览。
- 预览 included list 只允许 active 正式记忆；candidate / observation 只能作为 excluded / review materials。
- 采纳候选前会执行最小 deterministic lint guard；open blocking finding 会阻断采纳。
- 任务记忆包预览会排除 open blocking lint finding 命中的正式记忆。
- M6 已把任务记忆包接入工作流任务包生成流程：artifact / markdown / prepared dispatch prompt 会携带冻结正式记忆快照，并由 readiness 检查 snapshot stale。
- 候选创建和候选状态变更命令。
- `candidate_confirmed` 文案和状态语义：只表示候选被确认保留，不表示正式记忆已写入。
- 控制核心校验：无来源候选拒绝、secret 记忆候选阻止外发、候选不能请求 `memory_*` 状态。
- 前端候选治理读模型和 UI 摘要。
- M7 已把全局 `记忆` 入口升级为只读记忆管理最小入口，可展示正式记忆、候选、观察来源、来源 / 版本 / 审计、lint / conflict 和任务包 eligibility 摘要。
- M8 已完成知识库 / Obsidian-compatible 接口占位和边界；知识库资料可作为来源并生成候选，但不能直接写正式记忆。
- M9 已完成正式记忆 lifecycle preview / record、版本化编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉 scope、确认权、版本、审计、影响面和记忆中心最小入口。
- M10 已完成实体和关系治理；最小 entity registry、alias / dedupe 候选、关系候选、已确认关系和任务包关系解释已形成受控闭环。
- 测试覆盖候选不变正式记忆。

没有完成：

- 正式记忆 `MemoryPage` 展示页和 Markdown 受控编辑形态。
- 完整维护任务系统和 finding 生命周期 UI。
- 权限撤回的完整生命周期处理。
- 知识库 / Obsidian 深度集成。
- 向量库、图索引、理解地图等派生索引。

## 5. 中间版本完成标准

中间版本记忆层必须至少跑通这个闭环：

```text
worker 汇报
-> 工作流账本
-> 项目主管确认过程事实
-> ObservationStore
-> MemoryCandidate
-> 项目主管或用户确认
-> MemoryRecord / MemoryPage
-> MemoryVersion
-> MemoryAuditEvent
-> TaskMemoryPacketBuilder
-> 下一个 worker 的任务包按权限召回正式记忆
```

验收时必须能证明：

- 候选不会被下一个 worker 当正式事实。
- 正式记忆有来源。
- 正式记忆写入有版本。
- 正式记忆写入有审计。
- 冲突、过期、权限不满足时不能偷偷进入任务包。
- 任务包能解释每条记忆为什么入选、为什么被排除。
- 用户偏好记忆优先级高，但不能只靠普通聊天自动成立。

重要修正：

- 上面是第一条真实闭环，不是中间版本记忆系统的完整终点。
- 中间版本最终目标是完整实现工作台记忆系统的产品能力，不能只做“写一条正式记忆并召回一条”的 demo。
- M1 到 M6 只能证明记忆层开始可运行；M7 只能证明记忆管理 UI 最小入口可读。
- M8 已补齐知识库边界的最小占位；M9 已补齐正式记忆生命周期操作；M10 已补齐实体和关系治理；M11 已补齐维护任务和记忆 lint；M12 已补齐成熟模式、跨项目主题报告和 M1-M12 gate 摘要；M12.1 已修补 acceptance summary freshness；M13 已完成最终权威验收，结论为 `accepted_with_deferred_items`。

## 6. 架构切分

记忆治理属于控制核心。

存储、检索、索引和知识库读取属于核心外适配器或可替换实现。

建议模块边界：

| 模块 | 作用 | 是否核心 |
| --- | --- | --- |
| `memory_governance` | 状态、权限、确认、冲突和审计规则 | 是 |
| `observation_store` | 记录可引用观察 | 是，第一版可独立 store |
| `memory_candidate_store` | 现有候选 store | 是，已存在最小版 |
| `formal_memory_store` | 正式记忆、版本、状态 | 是，必须受控 |
| `memory_audit_store` | 记忆审计事件 | 是 |
| `task_memory_packet_builder` | 召回过滤和任务包生成 | 是 |
| `memory_index_adapter` | 向量、图、FTS、理解地图 | 否，可重建 |
| `knowledge_adapter` | Obsidian-compatible 知识库读取 | 否，不能绕过治理 |

核心口径：

- 控制核心决定什么能成为正式记忆。
- 控制核心决定什么能进入任务包。
- 索引只能给候选和召回提供线索，不能当权威。
- 知识库只能提供来源和材料，不能自动改变 agent 行为。

## 7. 存储策略

产品目标不是 SQLite。

但正式记忆要满足查询、权限、版本、审计和召回，第一版工程上应通过 `FormalMemoryStore` 端口封装存储细节。

建议执行策略：

- 正式记忆 store 先通过受控后端接口读写，前端不能直接改文件。
- 如果本批次选择 SQLite，必须只把它当实现手段，验收仍看正式记忆闭环。
- 如果为了降低风险先用 JSON sidecar，必须保留 `FormalMemoryStore` 端口和迁移计划；不能接受为中间版本最终完成。
- Markdown 展示页可以后置；第一轮必须先保证状态、版本、来源、权限和审计权威。
- Obsidian-compatible 知识库仍然是材料区，不是正式记忆存储。

推荐中间版本最终存储：

- SQLite：正式记忆状态、版本、来源、权限、关系和审计。
- Markdown 展示页：人可读正式记忆页，可生成或受控编辑。
- 可重建索引：FTS、向量、图、理解地图。
- 知识库 vault：原始材料和草稿。

## 8. 执行批次

如果用户确认本文，后续可以按以下批次交给其他对话执行。执行过程中不需要每批都停下问用户；只有触发 stop 条件才停。

批次分工：

- M1 到 M6：打通第一条真实记忆闭环。
- M7：记忆管理 UI 最小入口。
- M8 已完成知识库边界最小占位；M9 已完成正式记忆生命周期操作；M10 已完成实体和关系治理；M11 已完成维护任务和记忆 lint；M12 已完成成熟模式、跨项目主题报告和 M1-M12 gate 摘要；M12.1 已完成 freshness 修补；M13 已完成最终权威验收。
- 做完 M6 不能宣称记忆层完成；只能宣称“第一条闭环完成”。

### M1：正式记忆受控存储和审计骨架

目标：

- 新增 `FormalMemoryStore` / `MemoryAuditStore` / `MemoryVersionStore` 的后端模块或端口。
- 支持创建正式记忆记录、创建第一版版本、写记忆审计事件。
- 只通过后端命令写入，前端不能直接改。

必须包含：

- `MemoryRecord` 正式状态只能使用 `memory_*`。
- 正式记忆至少一个 `source_refs[]`。
- 创建正式记忆必须同步创建 version 和 audit。
- 写入失败时不能留下半条记忆。
- 读模型能列出正式记忆和最近审计。

禁止：

- 不从候选自动升级。
- 不注入任务包。
- 不接向量库 / 图数据库 / Obsidian。
- 不执行真实 Codex。

验收：

- 创建一条带来源的正式记忆，能读到 record、version、audit。
- 无来源正式记忆创建失败。
- 修改不是覆盖，而是新增 version。
- `candidate_confirmed` 不会自动创建正式记忆。

### M2：候选到正式记忆的受控采纳

执行状态：已完成，记录见 `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md` 与 `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`。M2 采用独立候选 sidecar lock + formal then candidate adoption link 的非事务策略；成功路径会写正式 record / version / audit，并在候选 sidecar 保留 adoption 回链。普通 `candidate_confirmed` 仍不等于正式记忆。

目标：

- 把现有 `MemoryCandidate` 和新 `FormalMemoryStore` 接起来。
- 新增“从候选采纳为正式记忆”的控制核心命令。
- 区分项目主管可确认的低风险项目记忆和必须用户确认的记忆。

必须包含：

- 用户偏好、全局蓝图、跨项目、成熟模式、高风险记忆必须要求用户确认。
- 项目主管只可确认低风险本项目记忆。
- 采纳候选时生成正式 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`。
- 候选状态和正式记忆状态分开记录。
- 采纳后候选仍保留历史，不被覆盖删除。

禁止：

- 不让秘书采纳正式记忆。
- 不让 worker 写正式记忆。
- 不让黑板候选直接变正式记忆。

验收：

- 低风险项目候选可由项目主管采纳为项目正式记忆。
- 用户偏好候选没有用户确认时不能采纳。
- 采纳后 UI 不能只显示“候选已确认”，必须能看到正式记忆 ID、来源、版本、审计。
- 拒绝候选不会生成正式记忆。

### M3：ObservationStore 和工作流观察入口

执行状态：已完成，记录见 `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md` 与 `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`。M3 新增独立 `observations.v1.json` sidecar，支持受控记录 worker 汇报等明确工作流观察，并由 `project_director` 从 `recorded` observation 生成 `candidate_needs_review` 记忆候选。observation 不是正式记忆，也不会注入任务包。

目标：

- 新增 `ObservationStore`。
- 从 worker 汇报、项目主管确认、全局主管复核、方案采纳、结果验收中记录 observation。
- 让 observation 可以生成记忆候选。

必须包含：

- observation 不是正式记忆。
- observation 必须带 source refs。
- observation 可以标记 `recorded`、`candidate_created`、`ignored`、`quarantined`。
- 只记录摘要和来源，不复制不必要敏感原文。

禁止：

- 不把 observation 直接注入任务包。
- 不把普通聊天自动做成 observation 后立即入记忆。

验收：

- worker 汇报进入 observation。
- 项目主管确认过程事实后生成 candidate。
- observation 生成 candidate 后状态变为 `candidate_created`。
- 隔离 observation 不生成 candidate。

### M4：任务记忆包生成器和预览

执行状态：已完成，记录见 `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md` 与 `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`。M4 新增 `TaskMemoryPacketBuilder` 和 `preview_task_memory_packet`，可生成 included / excluded / review materials 预览；candidate / observation 只能作为待审查材料，不能进入正式 included list。M4 不等于任务包注入，也不表示 worker 已收到记忆包。

目标：

- 新增 `TaskMemoryPacketBuilder`。
- 项目主管派 worker 前能生成任务记忆包预览。
- 预览说明哪些记忆入选、哪些被排除、排除原因是什么。

必须包含过滤：

- 是否正式记忆。
- 状态是否 `memory_active`。
- 是否冲突。
- 是否过期。
- 是否有权限。
- 是否可给当前模型。
- 是否超过任务需要。

排除原因至少包含：

- `candidate_unconfirmed`
- `permission_blocked`
- `conflicted`
- `stale`
- `model_export_blocked`
- `token_limit`
- `not_relevant`

禁止：

- 不执行真实 worker。
- 不把候选、观察、知识库命中、LLM 摘要伪装成正式记忆。

验收：

- active 正式记忆能进入预览。
- candidate 不进入正式记忆列表，只能显示为待审查材料。
- conflicted / deprecated / frozen / archived 记忆被排除。
- 每条入选和排除都有 reason。

### M5：冲突和记忆 lint 最小阻断

执行状态：已完成，记录见 `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md` 与 `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`。M5 新增 `memory-lint.v1.json` sidecar、deterministic lint engine、采纳前 blocking guard 和任务记忆包预览 blocking 排除；M5 只生成 finding / run，不自动修改正式记忆状态、不新增正式记忆版本、不写 `MemoryRecord.conflict_refs[]`。

目标：

- 实现最小 `MemoryConflict` / `MemoryLintFinding`。
- 在采纳候选和生成任务包时阻断明显冲突。

第一版只做确定性规则：

- 同 scope + 同 memory_type + claim 高相似或同 key。
- 新候选与 active 正式记忆 claim 矛盾标记。
- 来源权限撤回标记。
- 当前权威文档或用户最新确认覆盖旧记忆时标记。

必须包含：

- blocking 冲突阻止进入任务包。
- 冲突处理写 audit。
- 维护任务只能生成 finding，不能自动改正式记忆。

禁止：

- 不让 LLM 推断冲突直接废弃正式记忆。
- 不由系统自行合并，也不物理移除正式记忆。

验收：

- 冲突正式记忆不会进入任务包。
- 用户最新确认覆盖旧偏好时，旧偏好被标记待复核或废弃建议。
- 维护任务发现 stale 只生成 finding。

### M6：工作流任务包注入和端到端闭环

执行状态：已完成，记录见 `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md` 与 `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`。M6 已把 `TaskMemoryPacket` 接入任务包生成链路，task package artifact / markdown / prepared dispatch prompt 使用同一份冻结 snapshot；readiness 能识别 snapshot 缺失或 stale。M6 只接受为第一条真实记忆闭环完成，不接受为中间版本记忆层完成、完整正式记忆生命周期完成或真实 worker 已执行。

目标：

- 把 `TaskMemoryPacket` 接入项目主管派发 worker 的任务包生成流程。
- 后续 worker 能接收最小必要正式记忆。
- 写任务包生成审计。

必须包含：

- 任务包里展示正式记忆 claim、来源摘要、入选理由、禁止事项。
- 任务包保存 included / excluded 记录。
- 任务结束后，worker 汇报仍先进入工作流账本和 observation，不自动成为记忆。
- 任务包本身不是长期记忆。

禁止：

- 不给 worker 扫完整记忆库。
- 不把任务包内容回灌成正式记忆。
- 不在未授权时执行真实 `codex exec` / `codex exec resume`。

验收场景：

```text
worker A 汇报接口完成
-> 项目主管确认过程事实
-> 生成 observation
-> 生成 candidate
-> 项目主管采纳为低风险项目正式记忆
-> worker B 派发前生成 TaskMemoryPacket
-> worker B 的任务包包含“接口完成”正式记忆和来源 / 入选理由
```

### M7：记忆管理 UI 最小入口

状态：已完成，记录见 `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md` 与 `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`。只接受为只读记忆管理最小入口；不接受为生命周期操作、知识库接口、关系治理、维护任务、完整记忆系统或真实截图验收完成。

目标：

- 让用户和项目主管能看清正式记忆、候选、来源、版本、审计和冲突。

最小入口：

- 正式记忆列表。
- 候选列表。
- 记忆详情。
- 来源面板。
- 版本列表。
- 审计摘要。
- 冲突提示。
- 任务包预览。

硬要求：

- 正式记忆和候选视觉区分。
- `candidate_confirmed` 不能显示成“已记住”。
- 正式记忆必须显示来源、版本、状态、是否可进入任务包。
- 高风险采纳必须显示影响范围。

### M8：知识库和 Obsidian 接口占位

状态：已完成，记录见 `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md` 与 `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`。只接受为知识库最小入口、`knowledge_doc` 来源引用、候选生成入口、正式记忆 / 候选 / 任务包知识引用反向摘要和 Obsidian-compatible 边界占位；不接受为 Obsidian 原生同步、vault 自动扫描、正式记忆生命周期、关系治理或完整记忆系统完成。

目标：

- 只建立边界和来源引用，不做完整 Obsidian 原生能力。

允许：

- 记忆来源可引用 `knowledge_doc`。
- 知识库材料可生成候选。
- 正式记忆详情可反向链接知识库来源。

禁止：

- 不让 Obsidian CLI 直接写正式记忆。
- 不自动扫描 vault 生成正式记忆。
- 不把 Canvas / Graph / Bases 当正式记忆。

### M9：正式记忆生命周期操作

状态：已完成，记录见 `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md` 与 `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`。只接受为正式记忆 lifecycle preview / record、版本化编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉 scope、确认权、版本、审计、影响面和记忆中心最小入口；不接受为关系治理、维护任务、成熟模式、跨项目记忆、完整记忆系统或真实 worker / Codex 执行。

目标：

- 支持正式记忆的编辑、废弃、冻结、解冻、归档、合并、拆分、上升为全局记忆、下沉为项目记忆。
- 每次操作都创建新版本和审计事件。
- 让用户和项目主管能处理旧记忆、错记忆、重复记忆和范围放错的记忆。

必须包含：

- 编辑正式记忆不是覆盖旧内容，而是新增 `MemoryVersion`。
- 废弃不是物理删除，只影响未来召回。
- 冻结后不能直接改，只能解冻或创建替代版本。
- 合并 / 拆分必须保留来源、旧 memory id、审计和影响范围。
- 上升为全局记忆、下沉为项目记忆必须记录原因、确认人和适用范围。
- 用户偏好、全局蓝图、跨项目记忆和成熟模式的生命周期变化必须用户确认。

禁止：

- 不允许 Markdown 或 Obsidian CLI 绕过状态机。
- 不允许维护任务自动改正式记忆。
- 不允许 UI 直接改文件后视为正式变更。

验收：

- 编辑一条正式记忆后能看到旧版和新版。
- 废弃记忆默认不再进入任务包。
- 冻结记忆不能被普通编辑。
- 合并两条记忆后旧记忆保留历史，新记忆有来源和审计。
- 项目记忆上升为全局记忆需要用户确认。

### M10：实体和关系治理

状态：已完成。记录见 `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`。

目标：

- 实现 `MemoryEntityRegistry`、`MemoryRelation`、`MemoryRelationCandidate` 的最小可用版本。
- 解决同一对象多个名字、重复记忆、关系不清和因果误写问题。

必须包含：

- 实体 registry 至少覆盖项目、会话、角色、文档、工具、模型、harness、建议方案。
- 支持 alias、merge、dedupe。
- 支持 `entity`、`temporal`、`causal`、`semantic` 四类基础关系。
- `llm_inferred` 和 `ambiguous` 关系默认只能是候选关系。
- 因果关系默认需要项目主管或用户确认。
- 关系进入任务包前也要经过状态、权限、来源和冲突检查。

禁止：

- 不把图谱推断直接当正式关系。
- 不把关系索引当权威来源。
- 不让相似度命中自行合并实体。

验收：

- 同一工具两个别名能被提示为同一实体候选。
- LLM 推断的因果关系只进入候选关系。
- 已确认关系能帮助任务包解释“为什么召回这条记忆”。
- 冲突或未审关系不能作为正式事实影响 worker。

### M11：维护任务和记忆 lint

目标：

- 实现记忆系统的后台维护能力，让记忆不会靠人手动整理到失控。
- 维护任务生成提醒、候选、冲突、隔离或审计，不自动改正式记忆。

必须覆盖：

- 过期检查。
- 冲突检查。
- 缺来源检查。
- 重复和实体漂移检查。
- 私密和安全扫描。
- 权限撤回影响未来召回。
- 派生索引维护状态。
- 成熟模式检查。

必须包含：

- 维护任务每次运行写运行记录或审计引用。
- blocking 问题能阻止记忆进入任务包。
- 缺来源严重时可以建议冻结或隔离，但不能自动补编来源。
- 权限撤回后，相关正式记忆或派生索引必须停止未来召回或进入复核。
- 索引失败不能导致正式记忆丢失。

禁止：

- 不自动重写正式记忆。
- 不由系统自行合并正式记忆。
- 不因索引重建改变正式事实。
- 不绕过权限扫描私密资料。

验收：

- 记忆引用的来源撤权后，后续任务包不再召回相关内容。
- 旧记忆和当前权威文档冲突时生成冲突或 stale finding。
- 缺来源正式记忆被标风险。
- 维护任务发现成熟模式只生成候选，不自动变全局记忆。

### M12：成熟模式、跨项目记忆和完整验收

状态：已完成，记录见 `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`。只接受为成熟模式候选、跨项目主题报告、用户确认后正式 mature pattern 记忆受控写入、任务包召回边界和 M1-M12 gate 摘要完成；不接受为自动技能化、跨项目摘要直接影响 worker、向量库 / 图数据库 / GraphRAG、真实 worker / Codex 执行或最终权威验收完成。

目标：

- 把多次工作流沉淀出的流程、错误、修复和审查经验变成成熟模式候选。
- 支持跨项目主题汇总和全局记忆候选，但必须由用户确认。
- 完成中间版本记忆系统总验收。

必须包含：

- `MaturePatternCandidate`。
- `MemoryClusterReport` 或等价跨项目主题报告。
- 全局主管或秘书可以汇总跨项目异常和重复模式，但不能直接写正式全局记忆。
- 用户确认后才能进入成熟模式记录或全局记忆。
- 适用范围变化必须写审计。

禁止：

- 不让跨项目摘要直接影响项目 worker。
- 不让 GraphRAG / 图谱 / 聚类报告伪装成正式记忆。
- 不把成熟模式自动变成技能或全局规则。

验收：

- 多次相似失败能生成成熟模式候选。
- 成熟模式候选未确认前不能进入 worker 任务包作为规则。
- 用户确认后，成熟模式能作为正式记忆进入合适任务包。
- 跨项目主题报告能下钻来源，不替代来源。

### M13：中间版本记忆系统最终验收

状态：已完成，记录见 `evidence/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md` 与 `handoffs/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1-result.md`。最终结论为 `accepted_with_deferred_items`。接受为中间版本记忆系统最终权威验收完成；不接受为最终蓝图完整工作台完成、阶段 G 真实 Tauri 全面验收完成、真实 worker / Codex 执行或 GraphRAG / 向量库 / 图数据库 / 自动技能化完成。

目标：

- 统一验证 M1 到 M12，确认记忆系统能够完整实现中间版最终目标。

必须验证：

- 观察、候选、正式记忆、版本、来源、权限、冲突、审计、召回、任务包注入全链路。
- 正式记忆生命周期操作。
- 关系和实体治理。
- 维护任务和权限撤回。
- 成熟模式和跨项目候选。
- 知识库边界。
- UI 可理解性。

验收结论必须区分：

- 第一条闭环完成。
- 正式记忆系统完成。
- 中间版本记忆系统完成。
- 最终蓝图完整能力仍后置的部分。

## 9. Stop 条件

执行任一批次时，遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要改 `workflow-state.v0.json` 结构。
- 需要数据库迁移或替换现有数据目录。
- 需要把候选直接写成正式记忆但没有来源、版本、审计或权限判断。
- 需要让秘书、worker 或 UI 直接写正式记忆。
- 需要接 Obsidian 原生写入、向量库或图数据库。
- 发现本文和 `docs/memory-layer-design-v1.md`、`docs/middleware-version-development-plan-v1.md` 冲突。

## 10. 验收总清单

中间版本记忆层最终验收时，至少要有这些测试或手动验证：

- 候选不能进入正式任务包。
- 正式记忆必须有来源。
- 正式记忆必须有 version。
- 正式记忆变更必须有 audit。
- 无权限记忆不能召回。
- `secret` 或 `model_export_policy = blocked` 不能进入外发模型上下文。
- 冲突记忆不能偷偷进入任务包。
- 过期或废弃记忆默认不召回。
- 用户最新明确确认覆盖旧记忆。
- 普通聊天不自动成为正式长期记忆。
- 知识库材料只生成候选，不自动成为正式记忆。
- 删除或重建索引不影响正式记忆和审计。
- 任务包解释入选和排除原因。
- worker 只能读任务包允许内容。
- 任务结束后汇报回到 observation / candidate 流程，不自动写正式记忆。
- 正式记忆编辑会生成新版本，不覆盖旧版本。
- 废弃、冻结、归档的记忆默认不召回。
- 合并、拆分、上升、下沉都有来源、版本、影响范围和审计。
- LLM 推断关系默认只是关系候选。
- 因果关系确认后才能影响任务包。
- 维护任务发现问题不自动改正式记忆。
- 权限撤回后未来任务包不再召回相关内容。
- 成熟模式候选必须用户确认后才能成为全局或成熟模式记忆。
- 派生索引、图谱报告、跨项目摘要可重建且不是权威事实。

## 11. 下一步建议

M1 到 M6 已完成第一条真实记忆闭环，M7 已完成记忆管理 UI 最小入口，M8 已完成知识库 / Obsidian-compatible 接口占位和边界，M9 已完成正式记忆生命周期操作，M10 已完成实体和关系治理，M11 已完成维护任务和记忆 lint，M12 已完成成熟模式、跨项目主题报告和 M1-M12 gate 摘要，M12.1 已完成 acceptance summary freshness 修补，M13 已完成最终权威验收并结论为 `accepted_with_deferred_items`。

已完成任务包名：

- `tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- `tasks/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`

当前下一步：

- 记忆层实施切片已完成到 M13；后续进入阶段 E / 阶段 G 相关任务。

如果用户确认本文为中间版本记忆层实施切片，可以按 M1 到 M13 顺序理解完成链路。M1 到 M6 是第一条真实闭环；M7 是记忆管理 UI 最小入口；M8 是知识库边界最小占位；M9 是正式记忆生命周期操作；M10 是实体和关系治理；M11 是维护任务和记忆 lint；M12 是成熟模式、跨项目主题报告和 M1-M12 gate 摘要；M12.1 是 freshness 修补；M13 是总验收。M13 之后仍不能宣称最终蓝图完整能力、真实 Tauri 全面验收、真实 worker / Codex 执行、GraphRAG、向量库、图数据库或自动技能化完成。
