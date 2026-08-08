# SYN-M2A Remaining One-Shot R2：收口更正记录

时间：`2026-08-05T18:56:12+08:00`
权威：`docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md`、`tasks/2026-08-03-syn-m2a-kickoff-v1.md` 与 `tasks/2026-08-04-syn-m2a-remaining-one-shot-package-v1.md`。
结论：**READY_FOR_MAINLINE_CLOSEOUT candidate；M2 仍为 IN_PROGRESS，主线验收待定，非 COMPLETE；M3 未激活。**

本记录不覆盖旧 T2/T3/T4 证据；历史段保留当时的审计事实，文末的“第三轮窄修正”给出当前可复核边界。

## 权威 M2 条目 → 当前实现/证据 → 结论

| 权威条目 | 当前可复核事实 | 证据等级 | 结论 |
| --- | --- | --- | --- |
| DAT-002 / T4 grant foundation | grant 由 persisted authorization source 生成；DB-primary reserve 在 `BEGIN IMMEDIATE` 内重读 source、binding、prepared 状态和 quota。`s3_director_dispatch_db_primary_grant_ledger_round_trip` 通过。 | TEMP-INTEGRATION | 部分 PASS；不等于完整报告终结原子性。 |
| DAT-003 / T1 reference slice | `update_work_item_state` 的 DB-primary 路径在同一 immediate transaction 内调用 M2 UoW、状态、audit/receipt/snapshot 逻辑；局部 allowed/denied/idempotency 测试通过。 | STATIC + TEMP-INTEGRATION | 仅接线证据；未形成与 outbox/projector 同一运行样本的 exit proof。 |
| DAT-004 outbox/result command | 模块与 unit/constructor tests 存在，但无同一 production caller/reference-slice 证据。 | UNIT | HOLD。 |
| DAT-005 projector/shadow/parity | 模块与 unit/constructor tests 存在，但无同一 production caller、checkpoint/rebuild/parity runtime 证据。 | UNIT | HOLD。 |
| DAT-006 legacy/quarantine/recovery | 模块与 unit/constructor tests 存在；unknown/corrupt/sensitive 的 live disposition 未完成。 | UNIT | HOLD。 |
| DAT-007 migration/cutover/rollback/export | 未执行真实数据迁移、主读切换、rollback 或 export。 | NONE | HOLD。 |
| DAT-008 / T2 isolated App | R4 root/binding/marker 与 DB-primary block fallback 有 focused tests；不存在受支持的 non-GUI driver 来用真实产品调用链重跑 S1-S6。 | UNIT / STATIC | HOLD。 |
| DAT-001B / T3 | 新 value-free manifest 记录真实 Workbench root、mode、hash、stat、SQLite integrity/count；restricted shapes 与 `execution_attempts` 仍未获 owner/disposition。 | LIVE_MANIFEST_READ_ONLY | HOLD，非 cutover PASS。 |

## T2 独立验收

S1–S6 都是 `NOT_ACCEPTED / HOLD`。旧 S2 尤其无效：gate 最后时间与 kill 记录相差约 `19m35s`，超过 `120s` gate 预算。旧 S3/S4/S6 仅保留片段性锚点，缺与 PID/信号精确配对的 DB/JSON 前后查询和 hash；不得倒推为通过。

`scripts/run-r4-isolated-app-preflight.mjs` 是 GUI bundle launcher；它不能产生 workflow mutation IPC/CLI 证据。手工 seed 双侧 SQLite/JSON 或新 Node 脚本自行写 store 会绕开产品 caller，故本轮没有伪造重跑。需要专门的 M2 debug-only real-caller driver 才能在 fresh R4 root 产生 PID、gate、signal、restart、原始 command result、四表/JSON/DB 差分证据。

## Grant/report 与后续阶段边界

- 通过的聚焦测试：`mcp::execution_grant::tests` 5/5、`worker_report::tests` 24/24、DB-primary grant round trip 1/1、forged execution-report ingress 1/1、`fnd006_acceptance` 10/10。
- 这些是 grant/source/ingress 拒绝与 temp-store 证据。当前 report ledger 写、已验证终标和 director chain complete 仍为分段事务；因此不得声称 forged report 的完整业务终结原子性已经满足 M2 exit。
- Station 3b 的真实 RoleSession/attempt runtime 属 M3；M2 只保留 `DEFERRED_TO_M3` 边界，不以它阻塞本阶段已定义的 ports，也不以 fixture 写入拒绝声称 M3 已实现。

## Code Map 与全量库测

- 受管 Code Map 文件已恢复原状，未改 `.harness/manifest`。现役 `config-check --strict` 与 `active-path-audit --strict` 通过，但 overlay 是 `INCOMPLETE_SHADOW`，staged shadow 是 `WARN MAP_UPDATE_REQUIRED`。没有受支持的 managed upgrade 路径，故 advisory 清零为 `HOLD / HARNESS_CAPABILITY_GAP`。
- `cargo check --lib` 成功。`cargo test --lib --no-fail-fast` 为 `1326 passed / 23 failed / 45 ignored`；失败集中在 RoleSession、自动推进、C4/C6 及主管链 fixtures。它们不被本记录伪装成 M2 可接受的噪声，也未在本轮向 M3/M5 扩写修复。

## Diff 归属与不越界声明

返修前已是 `29 tracked + 7 untracked` 的 dirty 现场，当前 tracked diff 为 59 路（`5704` insertions / `1425` deletions，含既有 WIP）。Git 没有保存逐 hunk 的开工快照，不能诚实把每一条现存 dirty hunk 归因给 R2。

- 直接 M2 候选：`m2_*`、`workbench_sqlite_*`、`workflow_*`、`mcp/execution_grant.rs`、`plan_authorization_store.rs`、`worker_report.rs`、`commands.rs`、R4 profile/fixture 相关文件。
- 仅编译适配/待单独归因：`consultant_agent.rs`、`secretary_agent.rs`、`supervisor_*`、`director_agent.rs`、`index_host_app_entrypoints.rs` 及相邻 lifecycle 文件；本轮收口未继续扩写它们，不能自动作为 M2 接受面。
- 不应在 M2 收口中接纳为产品结论：受管 Code Map 实现、`.harness/manifest`、live Harness、`AUTHORITY.md` 与原 13 项战略 WIP；均未由本记录触碰或改写。

本记录只新增 value-free 项目证据，不写 Workbench 数据、不读 `/Users/yoyi/.codex`、不 stage/commit/merge/push，也不改变 live Harness。

## 后续 M2 reference-slice addendum（2026-08-05T05:29:15+08:00）

本 addendum 不改写上表在当时的审计结论。其后以同一
`workflow-state-sidecar / workflow_state / update_work_item_state` 切片实现并复核了
窄生产接线：`workflow_run_dispatch_entrypoints.rs` 的真实调用者现持有
M2 UoW 后的 constrained outbox claim/result；DB-primary startup 以同一
DB/JSON reconciliation 做 shadow/parity read，S4 restart 记录了
`RETRY_WAIT → RESULT_RECEIVED` 与 `PROJECTION_DEGRADED → EXTERNAL_RESULT`。

`workbench_sqlite_storage_mode::tests::m2_reference_slice_*` 的四个 scratch
tests 现在覆盖 allowed/denied/replay、expiry/retry/poison/cancellation、checkpoint
以及 unknown/corrupt/sensitive/unjoinable sidecar 的 value-free quarantine/rebuild。
新的真实-App R4 run（见
`t2-r4-real-app-s1-s6-2026-08-05.md` 的 addendum）复核 S1–S6、outbox retry、DB busy
与 corrupt SQLite 的 fail-closed isolation。因此旧表中 DAT-004、DAT-005、DAT-006、
DAT-008 的“无 production caller / 无 real-App runner”事实已被后续证据取代，
但只限该具名 reference slice。

M2 仍为 `IN_PROGRESS / PARTIAL(HOLD)`：DAT-001B 的 live owner/disposition、DAT-007
真实 cutover、live rollback/export、Code Map staged advisory 与全量库测环境门均未由
本 addendum 解决；M3 仍未激活。

## 2026-08-05 current correction after true-Tauri rerun

The preceding addendum is historical. It is superseded only where later
same-slice evidence exists:

- DAT-003–006 now have a constrained production caller through the versioned
  `workflow-state-sidecar.repository.m2.v1` DB-primary port. Focused scratch
  tests cover command identity/revision, denial, replay, lease/retry/result,
  checkpoint/rebuild/parity and value-free quarantine/rebuild. Generic M2
  candidate modules still are not credited merely because they compile.
- DAT-008 now has `REAL_TAURI_APP / ISOLATED_SCRATCH` runtime-ready registered
  command/IPC S1–S6 evidence, including actual PID/PPID/argv, signals,
  source/executable/DB/WAL/SHM/JSON fingerprints and stable Git provenance;
  see `t2-r4-real-app-s1-s6-2026-08-05.md`.
- DAT-001B remains `LIVE_MANIFEST_READ_ONLY` and DAT-007 remains
  `NOT_MIGRATED / NO_CUTOVER`; neither is misreported as a completed live
  migration or treated as a prerequisite to the named scratch-slice evidence.
- Code Map `INCOMPLETE_SHADOW / MAP_UPDATE_REQUIRED` remains a non-blocking
  advisory; no managed script or manifest was changed. The current restricted
  host's full library result is `1365 passed / 1 failed / 45 ignored`, with
  the sole failure a resident fixture's PID `lstart` `EPERM` rather than an
  M2 behavior regression.

M2 nevertheless remains `IN_PROGRESS / PARTIAL(HOLD)`. The grant-bearing
report path's accepted report, verified workflow terminal state and director
completion are not one DB-primary transaction: correcting that safely crosses
frozen C4/C6/director contracts and requires `CONTRACT_VERSION_REVIEW_REQUIRED`.
No M3 or M5 lifecycle was implemented, and this correction does not claim
M2 closeout.

## 2026-08-05 scratch export / no-op rollback and migration-state matrix

`workbench_sqlite_storage_mode::tests::m2_reference_slice_scratch_export_manifest_and_noop_rollback_preserve_sidecar`
passes in a temporary `DbPrimaryFixture`. It reuses the existing restricted
`export_confirmed_db_to_json_dry_run` path: the resulting value-free receipt
has a canonical `workflow-state.v0.json` projection hash and a content-addressed
source reference, while the old sidecar and SQLite bytes remain identical
before/after the dry-run export and after a cache-clearing normal restart. It
does not read, write, delete, or export real Workbench data.

| Surface | Exact state | Evidence boundary / disposition |
| --- | --- | --- |
| Isolated `workflow-state-sidecar` reference slice | `DB_PRIMARY_JSON_PROJECTION / SHADOW_PROJECTION / NO_LIVE_CUTOVER` | Scratch export/readback and no-op rollback-restart PASS; old sidecar retained and byte-identical. |
| Live workflow domain | `NOT_MIGRATED / NO_CUTOVER` | No live Workbench read/write, migration, promote, rollback or export was attempted. |
| Restricted key shapes | `HOLD_NO_ORDINARY_STORE` | Owner/sensitivity/disposition remains governed by the read-only DAT-001B boundary. |
| `execution_attempts` | `HOLD_NO_CUTOVER` | No live read/write; canonical attempt lifecycle remains outside this M2 slice. |
| Other domains | `NOT_MIGRATED / HOLD` | Outside the named reference slice; no completion is implied. |

This improves DAT-006/DAT-007 scratch evidence only. It does not clear the
separate `CONTRACT_VERSION_REVIEW_REQUIRED` hold for grant-bearing report
terminal atomicity and does not make M2 complete.

## 2026-08-05 grant-boundary supersession

This addendum preserves the historical text above, including its former
`CONTRACT_VERSION_REVIEW_REQUIRED` diagnosis, but supersedes that diagnosis
for the M2 grant-bearing report path. The frozen v1 owner boundary does not
permit report admission to create execution truth, review truth, a decision,
or a director completion. M2 therefore now performs fresh
grant/source/binding/revocation validation and returns the typed
`NOT_MIGRATED/HOLD` boundary with **zero persistence**.

- A valid grant-bearing report creates no pseudo `ExecutedReport` claim, no
  command receipt, no audit event, no top-level `updated_at` change, and no
  mutation of attempt, dispatch, work item, workflow, review, decision or
  director-chain state.
- Forged, expired, revoked, wrong-owner, wrong-actor/scope/subject,
  binding-mismatch and mid-flight attempt variants fail closed with the same
  zero-write boundary.
- The M1 legacy no-grant path is retained as `GUARDED_LEGACY`; it was not
  redefined as M2 claim/review/decision behavior.

The current slice reconciliation and direct evidence are recorded in
`syn-m2-reference-slice-reconciliation-2026-08-05.md`. This correction does
not implement `ReviewExecutionClaim`, `RecordResultUserDecision`, source-owner
apply-result, RoleSession, or M3/M5 lifecycle semantics, and it does not make
M2 complete.

## 第三轮窄修正（当前结论）

本节取代此前“同一 slice declaration/snapshot/consumer 仍未修复”的
`PARTIAL(HOLD)` 表述，保留其他历史审计文本作为时间锚点。

- **DAT-004 / DAT-008 armed R4 same-slice effect：PASS_BOUNDED。** 真实
  `update_work_item_state` owning UoW 现在原子写入 domain fact、owner
  receipt/event/audit、`OutboxItemDeclared`、`SCRUBBED_OUTBOX_RECORD` 与
  armed outbox。独立 result command 的 `command_id` 与 owner correlation
  不混写；replay 零新增，错误 binding 零写。普通产品路径仍不产生 effect/outbox。
- **DAT-005 snapshot/checkpoint：PASS_BOUNDED。** 唯一 concrete port 从
  authoritative SQLite 重读完整 workflow aggregate（workflow、排序 nodes、排序
  work items）生成 `workflow_state:{project}:{workflow}` canonical hash；写盘后
  JSON 同公式重读，restart 会按同一 watermark/event/receipt 恢复 checkpoint。
  同 workflow 其他 item/node 的变化会改 hash，key-order-only JSON 不会改 hash，
  篡改会 fail-closed 且 DB 不被 JSON 反写。
- **port/schema/consumer reconciliation：PASS_BOUNDED。**
  `workflow-state-sidecar.repository.m2.v1` 是这个具名 slice 唯一 concrete
  truth；显式 M2/R4/startup-recovery 与 guarded-legacy consumer 都有稳定
  caller id、port version 和 migration state，漏登记即测试失败。冻结 FK validator
  对 wrong action 与 wrong target 都拒绝。
- **真实运行证据：PASS_BOUNDED。**
  `node scripts/run-r4-isolated-app-preflight.mjs --m2-reference-slice` exit
  `0`，S1–S6 与 DAT-004/008 same-slice effect/result/recovery 共 7 个场景全部
  `PASS`。原始 launcher receipt:
  `/private/var/folders/nj/y6s1fvl936xgfwg20w08sk6r0000gn/T/syn-r4-acceptance-D4wpEO/m2-reference-slice-suite-receipt.json`；
  SHA-256 `acc05a13c791717b83d90ddd714717d8f4fc78121b46eb07579737ae999f876a`。
  它只证明 `REAL_TAURI_APP / ISOLATED_SCRATCH / CONSTRAINED_REFERENCE_SLICE`，
  不证明 provider、live Workbench、OFF_MACHINE、Harness closeout 或 M2 COMPLETE。
- **完整库测：PASS。** 主机权限 `cargo test --lib --no-fail-fast --quiet` exit
  `0`，`1385 passed / 0 failed / 45 ignored`；`cargo check --lib --quiet` exit
  `0`。五个 A/B/C focused tests 均为 `1 passed / 0 failed`。

因此当前状态是 `M2 IN_PROGRESS / READY_FOR_MAINLINE_CLOSEOUT candidate`，
仍待主线验收。遗留而未在本包清除的边界是 live Workbench
`LIVE_MANIFEST_READ_ONLY`、DAT-007 `NOT_MIGRATED / NO_CUTOVER`、后续
claim/review/decision/source-owner commands `NOT_MIGRATED/HOLD`、Code Map
advisory、OFF_MACHINE 与 Harness closeout；它们没有被伪装成已完成，也没有启动
M3。
