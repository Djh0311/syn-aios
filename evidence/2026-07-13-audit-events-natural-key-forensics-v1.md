# audit_events natural-key 法证判定 v1

日期: 2026-07-13
任务包: `tasks/2026-07-13-audit-events-natural-key-forensics-package-v1.md`
基线: `cc31c8a`
档位: 纯只读法证。唯一写入为本文件；没有运行 importer/apply/rehearsal，没有写 live 根或生产 DB。

## 范围、方法与快结论

- 事实源: `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json` 的 `audit_events`。读取时共 `1474` 条；文件 SHA-256 为 `bf3e6f473c05e9b67adfd4f3135b2541503e76183dc0f5a8648ec2b636d8846d`。
- 分组法: 对非空 `event_id` 做相等分组；组内再比对完整 JSON 字段。结果为 `16` 组、`51` 条记录，且所有组内记录的完整 payload 都不同。
- **已证实的根因**: `stable_id` 将非字母数字归一为 `-` 后无条件 `take(96)`（`src-tauri/src/lib.rs:1067-1080`）。16 组的每一条均可由当前构造式重算得到原 `event_id`：原实体不同且原字符串均长于 96，归一后前 96 字符相同，再叠加同一批共享的毫秒时间戳，故得到同号。
- **边界校正**: 截断构造仍在产出（最近 30 条有 20 条中段恰为 96），但本快照中实际撞号的最晚一组是 `2026-07-11T00:47:03.621Z`；不能把“风险仍在”表述成“07-11 后仍已观测到撞号”。
- **旧代校正**: 4 条 `audit_event_id` 旧 schema 记录不是无 natural key。主 store importer 已按 `event_id -> audit_event_id -> id` 取 key（`workbench_sqlite_importer.rs:41,862-912,955-959`）；它们都能取到唯一 `audit_event_id`，不是 hash/index fallback。

数据坐标说明: 主 JSON 是单个序列化文件，以下 `audit_events[index]` 是可复查的数据坐标；所有时间戳均为原始 epoch milliseconds，ISO 时间仅作阅读辅助。

## A. 16 组撞号逐组画像

共同字段: 所有 16 组都落在 `workflow:users-yoyi-codex-workflow-mario-test:default`（从 `target_ref` 解析）；这 51 条记录自身未存 `project_id`。每组内 `created_at` 完全相同，`target_ref` 指向不同 planned task。`推断` 其原因是 C4 在循环外一次取得 `timestamp_ms`（`c4_c6_workflow_governance_entrypoints.rs:76`），随后在同一 prepare 批次中为多个 task 建 work item/dispatch（`:119-225`）。

### A01-A14: `authorized_prepared_dispatch_created`

所有这些组的内容差异字段均为 `project_director_planned_task_id`、`target_ref`、`reason`；即同一 prepare 批次中不同 worker task 的记录，而非同一实体的重复写入。生成器是 `push_authorized_prepared_dispatch_created_audit`，模板为 `audit:authorized-prepared-dispatch-created:{stable_id(dispatch_id)}:{timestamp}`（`c4_c6_workflow_governance_entrypoints.rs:2597-2623`）。`dispatch_id` 本身由 `authorized-prepared-dispatch:{stable_id(planned_task_id)}:{timestamp}` 组成（`:221-225`）。

| 组 | 完整 event_id | 数据坐标 | 条数 | 原始时间 / ISO | 不同实体（planned task suffix） |
|---|---|---|---:|---|---|
| A01 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783010517930` | `[626,628,631]` | 3 | `1783010517930` / `2026-07-02T16:41:57.930Z` | `...-1`, `...-2`, `...-4` |
| A02 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783190207289` | `[656,657,659]` | 3 | `1783190207289` / `2026-07-04T18:36:47.289Z` | `...-1`, `...-2`, `...-3` |
| A03 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783190776457` | `[668,669,670]` | 3 | `1783190776457` / `2026-07-04T18:46:16.457Z` | `...-1`, `...-2`, `...-3` |
| A04 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783242927888` | `[775,776,777,778]` | 4 | `1783242927888` / `2026-07-05T09:15:27.888Z` | `...-1`, `...-2`, `...-3`, `...-4` |
| A05 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783298153847` | `[838,839,840,841]` | 4 | `1783298153847` / `2026-07-06T00:35:53.847Z` | `...-1`, `...-2`, `...-3`, `...-4` |
| A06 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783301788405` | `[878,879,880]` | 3 | `1783301788405` / `2026-07-06T01:36:28.405Z` | `...-1`, `...-2`, `...-3` |
| A07 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783310366637` | `[904,905]` | 2 | `1783310366637` / `2026-07-06T03:59:26.637Z` | `...-1`, `...-2` |
| A08 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783324215280` | `[931,932,933]` | 3 | `1783324215280` / `2026-07-06T07:50:15.280Z` | `...-1`, `...-2`, `...-3` |
| A09 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783417639402` | `[1009,1010]` | 2 | `1783417639402` / `2026-07-07T09:47:19.402Z` | `...-1`, `...-2` |
| A10 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783431636435` | `[1034,1035,1036]` | 3 | `1783431636435` / `2026-07-07T13:40:36.435Z` | `...-1`, `...-2`, `...-3` |
| A11 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783587367314` | `[1071,1072,1073,1074]` | 4 | `1783587367314` / `2026-07-09T08:56:07.314Z` | `...-1`, `...-2`, `...-3`, `...-4` |
| A12 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783670670512` | `[1117,1118]` | 2 | `1783670670512` / `2026-07-10T08:04:30.512Z` | `...-1`, `...-2` |
| A13 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783728667103` | `[1169,1170,1171,1172]` | 4 | `1783728667103` / `2026-07-11T00:11:07.103Z` | `...-1`, `...-2`, `...-3`, `...-4` |
| A14 | `audit:authorized-prepared-dispatch-created:authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-:1783730823621` | `[1197,1198,1199,1200,1202,1204]` | 6 | `1783730823621` / `2026-07-11T00:47:03.621Z` | `...-1`, `...-2`, `...-3`, `...-4`, `...-5`, `...-6` |

### A15-A16: `project_director_task_plan_created`

这些组的唯一差异字段是 `target_ref`。生成器模板为 `audit:project-director-task-plan-created:{stable_id(work_item_id)}:{timestamp}`（`c4_c6_workflow_governance_entrypoints.rs:2347-2378`）；`work_item_id` 又由长 `workflow_id` 加 `stable_id(planned_task_id)` 组成（`:2704-2708`）。

| 组 | 完整 event_id | 数据坐标 | 条数 | 原始时间 / ISO | 不同实体（planned task suffix） |
|---|---|---|---:|---|---|
| A15 | `audit:project-director-task-plan-created:work-item-workflow-users-yoyi-codex-workflow-mario-test-default-project-director-planned-task-wo:1783010517930` | `[625,627,630]` | 3 | `1783010517930` / `2026-07-02T16:41:57.930Z` | `...-1`, `...-2`, `...-4` |
| A16 | `audit:project-director-task-plan-created:work-item-workflow-users-yoyi-codex-workflow-mario-test-default-project-director-planned-task-wo:1783730823621` | `[1201,1203]` | 2 | `1783730823621` / `2026-07-11T00:47:03.621Z` | `...-5`, `...-6` |

### A 的等式核验

- 对 16/16 组逐条从 live `workflow_node_dispatches` 找到同 `work_item_id + c4_planned_task_id + prepared` 的 `dispatch_id`，或直接取 A15/A16 的 `target_ref`。51 个 raw source 均彼此不同且长度大于 96；同组 `stable_id(raw_source)` 都是相同的 96 字符；以相应生成器模板重算后，51/51 与存储的完整 `event_id` 相等。
- 代表样本 A01 的 3 个 raw dispatch source 分别为 `authorized-prepared-dispatch:planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-{1,2,4}:1783010517930`，各长 `111`；三者都归一成 `authorized-prepared-dispatch-planned-task-workflow-users-yoyi-codex-workflow-mario-test-default-`（长 `96`），再加同一 `1783010517930` 得 A01 的同号。
- 汇总: `authorized_prepared_dispatch_created` 为 `14` 组 / `46` 条；`project_director_task_plan_created` 为 `2` 组 / `5` 条。最早 `2026-07-02T16:41:57.930Z`，最晚 `2026-07-11T00:47:03.621Z`。
- 判定: **确定事实，不是推断**。截断丢失实体尾部身份是必要条件；同一毫秒批量写入是第二必要条件。

## B. 旧代 schema 事件清点与迁移走向

| 数据坐标 | `audit_event_id` | `event_type` | 关联主体 |
|---|---|---|---|
| `audit_events[296]` | `audit:uiwork-workflow-registered:1780224144221` | `uiwork_workflow_registered` | project/workflow `...documents-uiwork...` |
| `audit_events[297]` | `audit:uiwork-work-item-ready:1780224144221` | `uiwork_work_item_ready` | 同 workflow；work item `...inkwash-ui-replacement-v1` |
| `audit_events[298]` | `audit:uiwork-four-role-sessions-bound:1780224144221` | `uiwork_four_role_sessions_bound` | 同 workflow；同 work item |
| `audit_events[355]` | `audit:workflow-machine-stale-run-cleaned:1780227015824` | `workflow_machine_stale_run_cleaned` | 同 workflow；同 work item；有 `run_id` |

- 总数为 **4**，不是“至少 4”。主 store 共 `1474` 条，`event_id` 非空 `1470` 条，`audit_event_id + event_type` 且无 `event_id` 为 `4` 条，所有 key candidate 都缺失为 `0` 条；4 个旧 id 自身也无重复。
- 两代字段对照:

| schema | 标识字段 | kind 字段 | 本快照数量 | importer natural key |
|---|---|---|---:|---|
| 当前 | `event_id` | `event_type` | 1470 | `event_id` |
| 旧代 | `audit_event_id` | `event_type` | 4 | `audit_event_id` |

- importer 主 store `WORKFLOW_ARRAYS` 对 `audit_events` 的候选顺序是 `event_id, audit_event_id, id`（`workbench_sqlite_importer.rs:31-43`）；`collect_array_records` 对候选取首个字符串（`:840-912`），底层 `natural_key` 是 `find_map`（`:955-959`）。所以在新批次、没有 previous-record 冲突时，这 4 条进入 `accepted`，并不会进入 `hash_key_fallback` 或“无 key”路径。
- apply 层同样为主 store audit events 列出 `audit_event_id, event_id, id`（`workbench_sqlite_apply.rs:324-339`），并将所得 natural key 写入 `workflow_audit_events.event_id`（`:803-824`）。SQLite 表仍是 `event_id TEXT PRIMARY KEY`（`workbench_sqlite_schema.rs:84`）。因此旧代字段**已被 key 选择兼容**，但不存在 `event_id` 的结构一致性问题仍然真实存在。

## C. 生成器全清单与活跃性

### C0. 清单方法

对 `src/**/*.rs` 搜索字面量 `"audit:` 得 `114` 个命中、跨 `34` 个文件。下面按三类完整归档：C1 是会写主 `workflow-state.v0.json.audit_events` 的构造位点；C2 是 sidecar/audit-ref 构造；C3 是测试 fixture 或消费文字，非生产 event-id writer。这样不会把 `audit_refs` 误报为 root `event_id` 生成器。

`stable_id` 的“是”均表示受 `lib.rs:1067-1080` 的 96 字符截断影响；“否”表示使用 timestamp、PID、`short_hash`、`stable_fragment` 或常量，不经过该截断器。活跃性含义: “最近”=本快照最后 30 条已观测到对应 writer/family；“已编译未在最近 30 证明”不等于 dead code。

### C1. 主 store `audit_events` writer

| 位点（逐个） | 模板 / 实体源 | stable_id 96 | 活跃性证据 |
|---|---|---|---|
| `c4_c6_workflow_governance_entrypoints.rs:564` | `process-fact-decision` / `request.report_id` | 是 | 已编译；最近 30 未见 |
| `:697` | `global-final-review` / `review_id` | 是 | 已编译；最近 30 未见 |
| `:798` | `user-result-decision` / `decision_id` | 是 | 已编译；最近 30 未见 |
| `:879` | `stage-c-acceptance-summary` / `request.workflow_id` | 是 | 已编译；最近 30 未见 |
| `:2366` | `project-director-task-plan-created` / `work_item_id` | 是 | **最近 30 有 4 条**；A15/A16 已撞号 |
| `:2606` | `authorized-prepared-dispatch-created` / `dispatch_id` | 是 | A01-A14 已撞号；最近 30 当前走 sibling deferred |
| `:2636` | `authorized-prepared-dispatch-thread-deferred` / `dispatch_id` | 是 | **最近 30 有 4 条** |
| `:2665` | 动态 blocked `event_type` / `task.planned_task_id` | 是 | 已编译；最近 30 未见 |
| `commands.rs:3312` | `workflow-submit` / `workflow_id` | 是 | 已编译；最近 30 未见 |
| `director_agent.rs:3466` | `supervisor-task-session-birth` / `native_thread_id` | 是 | 已编译；最近 30 未见 |
| `mcp/supervisor_orchestrator.rs:673` | `supervisor-task-session-abandoned` / `stable_fragment(native_thread_id)` + nanos | 否 | 已编译；最近 30 未见 |
| `operation_control.rs:353` | `l3-operation` / `operation_id` | 是 | 已编译；最近 30 未见 |
| `project_workflow_automation.rs:1884` | `k3-b-work-item` / `config.work_item_id` | 是 | 已编译；最近 30 未见 |
| `:1990` | `k3-work-item` / `work_item_id` | 是 | 已编译；最近 30 未见 |
| `:2929` | `k3-project-workflow-automation` / `plan.automation_id` | 是 | 已编译；最近 30 未见 |
| `:2981` | `j2-b-b1-project-workflow-automation` / `phase_b.product_command_id` | 是 | 已编译；最近 30 未见 |
| `:3040` | `j2-b-b2-project-workflow-automation` / `phase_b.product_command_id` | 是 | 已编译；最近 30 未见 |
| `:3103` | `k3-b-project-workflow-automation` / `phase_b.product_command_id` | 是 | 已编译；最近 30 未见 |
| `store_hygiene.rs:222` | `work-item-state:canvas-run-residue` / residue identity | 否 | 已编译；最近 30 未见 |
| `:249` | `canvas-run-residue-swept` / timestamp | 否 | 已编译；最近 30 未见 |
| `workflow_chain_controller.rs:251` | dynamic chain event / `chain_run_id` | 是 | 已编译；最近 30 未见 |
| `workflow_execution_entrypoints.rs:328` | `workflow-node-dispatch-prepared` / `dispatch_id` | 是 | **最近 30 有 3 条** |
| `:418` | `workflow-node-dispatch-started` / `dispatch_id` | 是 | **最近 30 有 3 条** |
| `:528` | `workflow-node-dispatch-completed` / `dispatch_id` | 是 | **最近 30 有 3 条** |
| `:544` | `workflow-node-dispatch-readback` / `dispatch_id` | 是 | **最近 30 有 3 条** |
| `:748` | `workflow-node-dispatch-failed` / `dispatch_id` | 是 | 已编译；最近 30 未见 |
| `:800` | `workflow-node-dispatch-readback` / `dispatch_id` | 是 | 已编译；最近 30 未见 |
| `:899` | `workflow-dispatch-director-review` / `work_item_id` | 是 | 已编译；最近 30 未见 |
| `:986` | `workflow-permission-decision` / `request_id` | 是 | 已编译；最近 30 未见 |
| `:1138` | `offline-role-dispatch-prepared` / `dispatch_id` | 是 | 已编译；最近 30 未见 |
| `:1259` | `offline-role-result-handoff` / dispatch/work-item identity | 是 | 已编译；最近 30 未见 |
| `:1357` | `offline-director-review` / work-item identity | 是 | 已编译；最近 30 未见 |
| `workflow_run_dispatch_entrypoints.rs:489` | `work-item-state` / `request.work_item_id` | 是 | 已编译；最近 30 未见 |
| `:701` | `workflow-node-session` / `request.node_id` | 是 | 已编译；最近 30 未见 |
| `:921` | `workflow-binding-id-migrated` / timestamp | 否 | 最近 30 的首尾均有该 type |
| `:996` | `workflow-node-session-unbind` / `request.binding_id` | 是 | 已编译；最近 30 未见 |
| `workflow_state_lifecycle_task_package.rs:44` | `init` / timestamp | 否 | 已编译；最近 30 未见 |
| `:102` | `bootstrap` / `project_root` | 是 | 已编译；最近 30 未见 |
| `:349` | `task-draft` / normalized title | 是 | 已编译；最近 30 未见 |
| `:405` | `task-node-draft` / normalized title | 是 | 已编译；最近 30 未见 |
| `:617` | dynamic `task-fields*` / `request.work_item_id` | 是 | 已编译；最近 30 未见 |
| `:756` | `task-file` / `request.work_item_id` | 是 | 已编译；最近 30 未见 |
| `:789` | `task-memory-injection` / `request.work_item_id` | 是 | 已编译；最近 30 未见 |

### C2. Sidecar / audit-ref 构造（非主 store event_id）

| 位点（逐个） | 身份材料 | stable_id 96 | 角色 |
|---|---|---|---|
| `blackboard_candidate_store.rs:115` | timestamp + `short_hash(candidate_key)` | 否 | blackboard audit ref |
| `exec_process_registry.rs:524` | event type + PID + timestamp | 否 | registry audit event |
| `formal_memory_lifecycle.rs:70`; `formal_memory_store.rs:85` | event type + timestamp + short hash | 否 | formal-memory audit ids |
| `manual_relay.rs:346` | preview identity | 否 | relay preview ref |
| `memory_candidate_store.rs:86,177,276` | timestamp + short hash candidate key | 否 | candidate audit refs |
| `memory_entity_relation_governance.rs:1147` | timestamp + short hash of relation identity | 否 | relation audit id |
| `observation_store.rs:90,195` | timestamp + short hash observation key | 否 | observation audit refs |
| `plan_authorization_store.rs:86,170,254,434,497` | authorization id / scope tuple | 是 | authorization sidecar audit refs |
| `project_consultation_proposal_store.rs:89,304` | proposal id / decision | 是 | proposal sidecar audit ids |
| `session_continuation_store.rs:115,233,238,446,712,717,722,1024,1029,1034,1343,1348,1353,1586` | timestamp + short hash continuation/attempt identity | 否 | continuation audit refs |
| `memory_daily_loop.rs:60-63,191,290-293,385` | operation/work-item/authorization refs | 否 | **consumer-created refs, not root events** |

`session_continuation_store.rs:1938`、`codex_local_runner.rs:2462`、`runtime_log_store.rs:943,969,1068,1078`、`memory_capture_bus.rs:626,645,735,752`、`supervisor_action_controller.rs:1520,1562`、`workbench_sqlite_repository.rs:1100,1103`、`workbench_sqlite_transaction_acceptance.rs:15`、`workflow_audit.rs:131,162` 以及 `lib_*_tests.rs` 的命中均是测试 fixture、断言、常量或消费字面量，不是生产 writer。它们已由上述 `114` 命中的剩余项覆盖。

### C3. 活跃止血评估

- 最近 30 条范围为 `2026-07-12T15:20:48.603Z` 至 `2026-07-13T09:04:35.266Z`；其中 **20** 条的最后一个 identity segment 恰长 `96`。
- 这 20 条按 event type 为: `project_director_task_plan_created=4`、`authorized_prepared_dispatch_thread_deferred=4`、`workflow_node_dispatch_prepared=3`、`workflow_node_dispatch_started=3`、`workflow_node_dispatch_completed=3`、`workflow_node_dispatch_readback_completed=3`。
- 因而止血对象不能只修 A 的两个可见模板；所有 C1 中 `stable_id` 用作 audit identity 的 writer 都是条件性风险面。不能直接全局改 `stable_id`，因为它同时参与 workflow/work-item/node/其他既有 ID；应在独立包中仅改 audit identity 的构造，并为历史格式保留读取兼容。

## D. 消费方与改号爆炸半径

### D1. 当前 live 的持久引用实数

- 排除 `backups/` 后，active sidecars 一共存有 **141** 个 `audit_refs` 字符串，且均唯一: `plan-authorizations.v1.json` 的 `47` 个字段 / `140` 个 refs，`memory-capture-events.v1.json` 的 `1` 个字段 / `1` 个 ref。
- 与 16 个撞号 `event_id` 做完整相等及任一方向前缀比较，结果均为 **0**。与任一主 store `event_id` 完整相等的 refs 亦为 **0**。故“修这 51 条历史 ID 会断持久 live `audit_refs`”的实数是 **0 处**（仅限已读取的 active root；不对外部未纳入根的系统作推断）。
- `memory_daily_loop.rs:60-63,191,290-293,385` 有四个 runtime `audit_refs` 构造点，分别是 operation-control、worker-report、plan-auth-confirm、final-review。它们没有 A 的两个前缀，且本快照无命中 A01-A16；不会直接引用这 51 条。

### D2. 读模型与前端面

- `workflow_read_model.rs:20-65` 会逐条把 audit event 投影为 ledger item，并把 `event_id` 同时置入 `ledger_entry_id` 与 `audit_refs`，没有去重。A 的 51 条均符合目标 workflow filter，所以该 workflow 的派生账本中会出现 **51** 个相关 item，却只有 **16** 个不同的 ledger/audit identity。
- `projectCanvas.ts:1329-1334` 将 `event.event_id` 当作 canvas item id 和 `audit_event` ref；`runQueue.ts:500` 将 `recent_audit_events[].event_id` 直接传为 queue `audit_refs`；`ProjectWorkflowDerivedPanels.tsx:176-177` 显示 ledger 的 audit refs。这些是派生/展示传播，不是持久的反查索引。
- 主 store 代码中找到的 exact `event_id` 反查是 `c4_c6_workflow_governance_entrypoints.rs:1651-1665` 的 `find_worker_report_event`，并明确限制 `worker_structured_report_recorded | subagent_report`。A 的两种 event type 不满足该限制，因此 A01-A16 不会走此反查。
- 前端搜索没有发现针对主 workflow `audit_events` 的按 `event_id` RPC/命令反查。`ProjectJiaobanPanel.tsx:3040` 的 `key={event.event_id}` 属于 supervisor 账本事件流，数据源不同；不能据此把 A 的撞号外推为该组件已坏。

### D3. D 结论

历史改号的真实半径是: **0 个当前持久 `audit_refs` 需要同步改写；51 个当前可再生 read-model/画布/队列引用会在重投影时换成新 ID；16 个重复 key 会被消除。** 仍有未观测的外部导出、截图、人工记录风险，因此真实根改写仍须维护窗口和完整备份，不可因 `0` 把它降格成无风险操作。

## 决策备忘录: 三选一与独立止血

| 选项 | 益处 | 代价 / 风险 | 档位与前置 |
|---|---|---|---|
| a. 修数据 | 51 条恢复唯一 `event_id`，4 条可补当前字段名；恢复 `workflow_audit_events.event_id PRIMARY KEY` 的语义，export/read-model 不需永久接受同号历史。D 的持久 direct ref=0，使同步范围小于预期。 | 写真实根；需对 51 条分配可重算、可审计的新 ID，决定是否保留旧 ID alias，并重新核 JSON/schema/备份；外部未受控引用仍未知。 | **重档**。维护窗口、双 hash、完整备份、逐条 mapping、用户明确授权。 |
| b. 修合同 | 不改 live 历史，可让迁移先容纳 51 条内容不同但同 `event_id` 的记录；旧代 `audit_event_id` fallback 已存在，只需保留并测试。 | 不能只改 importer natural key: schema 当前 `workflow_audit_events.event_id PRIMARY KEY`（`workbench_sqlite_schema.rs:84`），apply 是 `ON CONFLICT(event_id) DO NOTHING`（`workbench_sqlite_apply.rs:803-824`）。必须同时重设 SQLite 主键/导出/回读语义，否则仍丢 50 条；永久背负“event_id 非唯一”的合同。 | **轻档的代码操作风险，但中等合同面**。需单独 M5 前包，覆盖 importer/apply/export/repository/replay。 |
| c. 保持 fail-closed | 不损伤 live 数据，也不会把不完整审计导入 SQLite；当前 M3/M5 闸继续诚实阻断。 | M5 无期限停留；问题仍在 live JSON，未来同形批次仍可能新增冲突。 | **零写 / 当前状态**。无需代码，但不是修复。 |

### 独立止血（不属于上表任一选择）

- 必做评估结论: C1 的截断 audit writers 仍活跃。独立包应把 **audit identity** 改为完整 SHA-256 或包含稳定 hash 的无截断格式，并覆盖所有 C1 的 `stable_id` audit 写点；不能改全局 `stable_id`。
- 风险: 这是 live 写路径变更，必须验证同毫秒多实体批次、同一实体重试、旧 ID 读取兼容和下游 `audit_refs` 的新增格式；不应在本法证包顺手修改。

### 推荐（决定权在用户）

**推荐顺序: 先保持 fail-closed，另开“窄止血”包；随后在用户愿意安排维护窗口时选择 a. 修数据。**

理由: a 保持 `event_id` 唯一这一既有 schema/读模型合同，且 D 已证明当前 active root 的持久 direct refs 为 0；b 虽不写 live 数据，却不是小改 importer，必须重写 SQLite 的唯一性和 round-trip 合同，长期语义债更高。此推荐不替代用户对真实根写入的授权。

## 核收回传（10 项）

1. 完成内容: A-D 法证与三选一决策备忘录均在本文件。
2. 改动文件: 仅新增本 evidence；零 `.rs/.ts/.tsx` 或配置改动。
3. 数据证据: live `audit_events=1474`，16 组 / 51 条，旧代总数 4，文件 hash 见本文首节。
4. 根因结论: 51/51 可由 96 截断 + 同毫秒批次精确重算，不是推断。
5. 旧代结论: 当前 importer 已 fallback `audit_event_id`；“合同完全不认旧字段”不成立。
6. 消费面实数: active 持久 `audit_refs=141`，撞号 direct/prefix 引用 `0`；派生账本受影响为 51 items / 16 identities。
7. 活跃性: 最近 30 条有 20 条 96 截断 identity；最新实际撞号为 2026-07-11，不虚报后续撞号。
8. 推荐: 先 fail-closed + 独立窄止血，再在维护窗口优先 a；用户决定。
9. 收尾核验: source SHA-256 复核仍为 `bf3e6f473c05e9b67adfd4f3135b2541503e76183dc0f5a8648ec2b636d8846d`；tracked diff 与 cached diff 均为空，相对起点仅新增本 evidence。
10. 其他阻断/异常: 无；本包按红线未运行任何会写的演练或测试。
