# M0 — SQLite 迁移完整性合同 v1

> 资料状态（2026-08-09）：历史迁移合同和来源证据，不是当前迁移计划或切换授权。当前数据事实看 `docs/current-state.md`；任何迁移、切换和真实数据动作都需要新的计划、验证和明确授权。

日期:2026-07-13 · 基线 commit:`d952a7d` · 任务包:`tasks/2026-07-13-sqlite-migration-completeness-m0-m1-m3-package-v1.md`
勘察正本:本合同的每一条分类/坐标均来自 2026-07-13 只读勘察(交叉核过 live JSON + 源码 + 8-agent adversarial cross-check),坐标已按今日基线校正。

> **状态门**:本合同只定义「休眠的 JSON→SQLite→JSON 迁移机器要完整到什么程度、每个字段落哪类」。**不翻闸**:不 production apply 到真实 DB、不 read-cut、不 stop-write。产品行为不变(JSON 仍是活库)。M5/M6 真切换拍板锁着,需用户单独授权。

---

## 一、分类法(4 类 + fail-closed)

每个顶层文件、每个主 store 顶层字段,**分类且仅分类一次**:

| 类 | 定义 | 迁移策略 |
|---|---|---|
| **persistent-domain-fact** | 有活写入者、有自然键、是产品领域真相 | 落表 + 导出(往返一致) |
| **compat-projection** | Serialize-only 的 read-model,非持久容器 | 不落表(改它无效);仅备注 |
| **runtime-transient** | 进程/会话运行期临时件,重启即可再生 | **不导入**;M2 归类留证 |
| **historical-archive** | 历史快照/旧库/已停写残留 | **不当活事实导入**;留 hash,不删 |
| **UNKNOWN(fail-closed)** | 代码与磁盘证据冲突、来源不可核 | 保守:数据不丢(可入归档表)但**显式标 unknown-provenance**,绝不冒充活事实 |

**fail-closed 原则**:未知项默认拒绝当活领域事实。宁可标 UNKNOWN 并保留原文,不静默吞、不静默复活。

---

## 二、顶层文件分类(live 根 12 JSON + backups + 91 txt + 旧库)

live 根:`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/`(2026-07-13 只读采数)

| 文件 | 类 | live 计数 | 迁移策略 |
|---|---|---|---|
| `workflow-state.v0.json`(主 store·5.9MB) | persistent-domain-fact | 见§三 | 主容器,落表+导出 |
| `formal-memories.v1.json` | persistent-domain-fact | records/versions/audit | **已完整**(落+导) |
| `memory-candidates.v1.json` | persistent-domain-fact | candidates/events | 落表已有,**导出缺**→M1(b) |
| `memory-capture-events.v1.json` | persistent-domain-fact | events | 落表已有,**导出缺**→M1(b) |
| `observations.v1.json` | persistent-domain-fact | observations/events | 落表已有,**导出缺**→M1(b) |
| `plan-authorizations.v1.json` | persistent-domain-fact | authorizations/audit | 落表已有,**导出缺**→M1(b) |
| `project-proposals.v1.json` | persistent-domain-fact | proposals/decisions/audit | 落表已有,**导出缺**→M1(b) |
| `memory-lint.v1.json` | persistent-domain-fact | runs=1(findings/maint=0) | **落表+导出全缺**→M1(a) |
| `global-supervisor-reviews.v1.json` | persistent-domain-fact | reviews=10,boundary=39,两 audit | **schema/import/apply/export 全缺**→M1(账本) |
| `supervisor-action-control.v1.json` | persistent-domain-fact | actions=30 | **全缺**→M1(账本) |
| `supervisor-orchestrator.v1.json` | persistent-domain-fact | sessions=23,audit=189 | **全缺**→M1(账本) |
| `exec-process-registry.v1.json` | **runtime-transient** | entries=0,audit=3 | **不导入**(OS 进程租约:pid+started_at+cmdline;导入=复活死租约) |
| `backups/`(98 JSON 快照) | historical-archive | 98 | 不导入,留档 |
| 91 × `supervisor:*.txt`(45 last-message + 46 stderr) | runtime-transient | 91 | **不导入**(preflight 判 non_json_file);M2 归类 |
| 旧库 `r3-migration-work/…workbench-state.v1.sqlite`(mtime 06-15/16) | historical-archive | — | 陈旧非基线,不作 M3 源 |

**未在 live 根但被白名单接受(结构性潜伏,非磁盘驱动)**:`memory-entity-relations.v1.json`、`memory-patterns.v1.json`、`blackboard-candidates.v1.json`、`real-execution-product-commands.v1.json`、`runtime-logs.v1.json`、`session-continuations.v1.json`。M1 仍为其补齐落表/导出(代码级完整),M3 往返不涉(磁盘无)。

---

## 三、主 store(`workflow-state.v0.json`)顶层字段逐项分类

**Meta 标量**(schema_version=`workflow_state_v0`〔字符串非点分〕· workflow_version=1 · **revision=11**〔非包文档写的 10;主 store mtime 晚于快照〕· workspace_id · source_kind=`workspace_state` · permission_level · created_at · updated_at):

| 字段 | 类 | 当前捕获? | 决策 |
|---|---|---|---|
| schema_version / workflow_version / revision | persistent-domain-fact | ✅ `workflow_state_meta` | 保留;revision 保真(见§六①) |
| workspace_id / source_kind / permission_level | persistent-domain-fact | ❌ 投影丢弃 | **残留缺口**(见§七·R2):记入 `meta_json` 已全存,但投影未回吐;M1 不扩(往返以 `meta_json` 全量为准,投影标量子集不阻断语义一致) |
| created_at / updated_at | runtime-transient(时间戳) | ❌ | 不单独回吐;`meta_json` 已含 |

**数组**(13 活 + 5 缺 = 18):

| 数组 | 类 | live 计数 | 自然键 | 状态 |
|---|---|---|---|---|
| projects/agent_adapters/workflows/nodes/edges/work_items/artifacts/reviews/audit_events/capabilities/harness_resources/workflow_node_session_bindings/workflow_node_dispatches | persistent-domain-fact | — | 各自 id | **已完整**(13 组落+导) |
| **execution_attempts** | persistent-domain-fact | 148 | `attempt_id` | **缺**→M1(c) |
| **permission_requests** | persistent-domain-fact〔provenance-flag〕 | 1 | `request_id` | **缺**→M1(c)。⚠ 未见活 CREATOR(仅 decision-mutator + test fixture),1 条来源存疑;按 shape 归活事实但留旗 |
| **workflow_chain_runs** | persistent-domain-fact | 37 | `chain_run_id` | **缺**→M1(c)。无 Rust struct(raw `json!`,内嵌 `nodes[]`)→整条 record_json 落 blob,不拆子表 |
| **workflow_execution_controls** | persistent-domain-fact | 148 | `control_id` | **缺**→M1(c)。⚠ read-model 字段名 `execution_controls` 与持久键 `workflow_execution_controls` 不对称——以持久键为准 |
| **workflow_machine_runs** | **UNKNOWN(fail-closed)→ historical-archive** | 10 | `run_id` | **缺**→M1(c)。⚠ 代码-数据冲突:key 在 src 0 命中、`run_workflow_machine_at`(entrypoints.rs:1496)已封死 Err、无持久 struct = 死码;却有 10 条磁盘残留。**决策**:落归档表保数据(往返不丢)+ M0 标 unknown-provenance,**不接活数组接线**、不冒充活事实。 |

`ProjectWorkflowSummary`(types.rs:2851/:2864-2866)= **compat-projection**(Serialize-only read-model,非持久容器)——改它无效,M1 不碰。

---

## 四、accepted-source 落表+导出策略(六处一致的目标态)

「六处」= importer 白名单(WORKFLOW_ARRAYS / OPTIONAL_SIDECARS)⇄ apply `source_kind_for_file` ⇄ apply `records_for_source` ⇄ apply `insert_domain_record` ⇄ schema DDL ⇄ exporter projection。补合同漏一处即新不对称。

**病灶三层(cross-check 校正版)**:
- **(a) importer 接受但 apply 丢**:4 sidecar(blackboard-candidates/memory-entity-relations/memory-lint/memory-patterns)。importer 已完整分类+收集(source_kind_for_name:1030、collect_sidecar_records:644/666/681/703),但 apply 的 `source_kind_for_file`(apply.rs:894)无这 4 arm→返回 `unknown_sidecar`→**真实丢点在 apply.rs:195 `else { continue }`**(source_ids 以 importer 的 source_kind 为键,取不到 `unknown_sidecar`),`records_for_source:454 Vec::new()` **对已接受源不可达**(包文档「丢在 :454」被 cross-check 校正)。7 张目标表 schema.rs:99-105 **已存在**→**(a) 零新增 DDL**。
- **(b) apply 落表但 exporter 不导**:5 源/10 表(memory-candidates/memory-capture-events/observations/plan-authorizations/project-proposals)。exporter.rs:74-108 只投影 5 文件→其余落 SQLite 后搁死,往返静默丢。
- **(c) 主 store 5 数组从没被 importer 收**:execution_attempts/permission_requests/workflow_chain_runs/workflow_execution_controls/workflow_machine_runs 不在 WORKFLOW_ARRAYS→`collect_workflow_records` 从不读。需白名单+schema 表+apply arm+exporter 投影全补。
- **账本**:3 主管账本(global-supervisor-reviews/supervisor-action-control/supervisor-orchestrator)四面全缺,需六处全补 + 新 DDL。exec-process-registry 四面缺=**正确姿势**(不导)。

**潜伏(c)**:apply `insert_domain_record:872 _ => Ok(0)` 未来 record_kind 不匹配静默吞→M1 改 **未知 kind fail-closed 报错**(区分「未知 kind=bug」与「已知 kind 但 ON CONFLICT 重复=合法 Ok(0)」;后者由各 arm 的 DO NOTHING 走,不进 `_`)。

---

## 五、M1 精确接线表(实现照此·每字符已核)

**layer (a)** — 4 sidecar,零新 DDL,apply 3 表 + exporter 4 投影:

| 文件 | source_kind(apply=importer) | 数组→record_kind(自然键)→目标表 |
|---|---|---|
| memory-lint.v1.json | `memory_lint` | runs→`memory_lint_run`[lint_run_id,run_id,id]→memory_lint_runs · findings→`memory_lint_finding`[finding_id,id]→memory_lint_findings |
| memory-entity-relations.v1.json | `memory_entity_relation` | relations→`memory_entity_relation`[relation_id,id]→memory_entity_relations |
| memory-patterns.v1.json | `memory_pattern` | candidates→`mature_pattern_candidate`[candidate_id,id]→mature_pattern_candidates · audit_events→`mature_pattern_audit_event`[audit_event_id,event_id,id]→mature_pattern_audit_events |
| blackboard-candidates.v1.json | `blackboard_candidate` | candidates→`blackboard_candidate`[candidate_key,id]→blackboard_candidates · audit_events→`blackboard_candidate_audit_event`[audit_event_id,event_id,id]→blackboard_candidate_audit_events |

**layer (b)** — 5 源,表已存在,只加 exporter 投影文件:
- memory-candidates.v1.json ← {candidates:memory_candidates, events:memory_candidate_events}
- memory-capture-events.v1.json ← {events:memory_capture_events}
- observations.v1.json ← {observations:observations, events:observation_events}
- plan-authorizations.v1.json ← {authorizations:plan_authorizations, audit_events:plan_authorization_audit_events}
- project-proposals.v1.json ← {proposals:project_proposals, decisions:project_proposal_decisions, audit_events:project_proposal_audit_events}

**layer (c)** — 主 store 5 数组,新 5 表 + importer WORKFLOW_ARRAYS + apply workflow_records spec + insert arm + workflow_state_projection:

| 数组 | 自然键 | 新表(PK + 索引列) |
|---|---|---|
| execution_attempts | attempt_id | execution_attempts(attempt_id PK, workflow_id, work_item_id, dispatch_id, project_id, …) |
| permission_requests | request_id | permission_requests(request_id PK, workflow_id, work_item_id, dispatch_id, project_id) |
| workflow_chain_runs | chain_run_id | workflow_chain_runs(chain_run_id PK, workflow_id, project_id) |
| workflow_execution_controls | control_id | workflow_execution_controls(control_id PK, workflow_id, work_item_id, project_id) |
| workflow_machine_runs〔归档〕 | run_id | workflow_machine_runs(run_id PK, workflow_id, work_item_id, project_id) — 只入不接活线 |

**账本** — 3 文件,六处全补 + 新 7 表(source_kind 新增):

| 文件 | source_kind | 数组→record_kind(自然键)→新表 |
|---|---|---|
| global-supervisor-reviews.v1.json | `global_supervisor_review` | reviews→`supervisor_review`[review_id]→supervisor_reviews · audit_events→`supervisor_review_audit_event`[event_id]→supervisor_review_audit_events · boundary_reviews→`supervisor_boundary_review`[review_id]→supervisor_boundary_reviews · boundary_audit_events→`supervisor_boundary_audit_event`[event_id]→supervisor_boundary_audit_events |
| supervisor-action-control.v1.json | `supervisor_action_control` | actions→`supervisor_action`[action_id]→supervisor_actions |
| supervisor-orchestrator.v1.json | `supervisor_orchestrator` | sessions→`supervisor_orchestrator_session`[run_id]→supervisor_orchestrator_sessions · audit_events→`supervisor_orchestrator_audit_event`[event_id]→supervisor_orchestrator_audit_events |

内嵌数组(chain_runs.nodes[]、orchestrator.sessions[].workers[]/final_marks[])**整条 record_json 落 blob**,不拆子表(与现有全部表一致)。

---

## 六、两处潜伏 bug 修复

① **exporter.rs:166 `revision unwrap_or(1)`**:live 主 store revision=11,若 `workflow_state_meta` 缺失(first_record_json→None→meta={})会把 revision 打回 1→假对账。修:main-store 投影保真 revision(缺失时不静默降为 1)。
② **apply.rs:872 `_ => Ok(0)`**:改未知 record_kind fail-closed `Err("unknown_record_kind:{}")`。已知 kind 的重复仍由各 arm `ON CONFLICT DO NOTHING` 返回 0(合法),不进 `_`。

---

## 七、fail-closed 决策 & 残留(显式记账·不静默)

- **R1 workflow_machine_runs**:UNKNOWN→归档表落(保数据往返)+ 不接活线 + 标 unknown-provenance。见§三。
- **R2 主 store 3 标量**(workspace_id/source_kind/permission_level):`meta_json` 已全存,投影未回吐子集;M1 不扩投影 schema(避免碰更多 read-model),往返以 meta_json 全量为准。**记账**,非静默丢。
- **R3 memory-lint `maintenance_reports`**:importer 本身不收(只收 runs/findings),apply 镜像 importer→亦不收。**importer 侧残留**,非本包 apply/exporter 六处范围;M2 记入。
- **R4 exporter 既有 4 处硬编码 `revision:1`**(:186/196/205/216,formal/runtime/product/session 投影):**本包不改**(非§六①范围·改动会动既有 fixture 断言 hash·scope 保守)。M1 新增账本/sidecar 投影则保真 revision(从 meta 读,不硬编码)。**记账**。
- **R5 9 张空 schema 表**(export_batches/rollback_points/memory_scopes/memory_source_refs/authorized_execution_scopes/stage_c_reviews/stage_c_acceptance_summaries/runtime_source_refs/readback_results):schema 超前、无源文件驱动、不在六处范围。**记账**,不接线。
- **R6 permission_requests provenance**:1 条磁盘来源存疑(无活 creator);按 shape 归活事实,留旗,M3 往返仅验存在的 1 条。

---

## 八、对包/评审的校正(勘察纠错·已 cross-check 复核)

1. **revision=11 非 10**:包验收「live=10 不被打回 1」应读作「**保真=11**」。
2. **layer(a) 丢点 = apply.rs:195**(非包写的 :454);修复须**先补 source_kind_for_file arm**(否则 :195 continue),再 records_for_source + insert。
3. **supervisor-orchestrator 有独立模块** `mcp/supervisor_orchestrator.rs`(SupervisorStore:22),非包写的 launcher 内联(:2013/:2278 在 `#[cfg(test)]`)。M1 以磁盘 JSON shape 接线,不碰该模块活路径。
4. **production_apply rehearse fixture=r3-a9**(非包写的 r3-a2);r3-a2 只是 apply/exporter 的 **DB-path 闸**(非 source 闸);r3-a8=snapshot,r3-a13=transaction。
5. **fixture 硬闸是 DB-path 闸非 source 闸**:M3 可把 source 指向 live 只读拷贝、DB 落 temp_dir 即过闸,**零闸改**;但 raw live 拷贝会被 **source preflight 拦**(4 个非白名单 .v1.json=unknown_json_file + 91 txt=non_json_file),M3 须**先剪枝**再喂 Level-B。

---

*本合同是 M1 实现与 M3 验收的单一真相源。M1 照§五接线、§六修 bug;残留照§七记账不静默;M3 照§八-5 剪枝走 Level-B。*
