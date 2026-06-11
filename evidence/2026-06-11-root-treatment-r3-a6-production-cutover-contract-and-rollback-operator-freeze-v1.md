# Root Treatment R3-A6 Production Cutover Contract And Rollback Operator Freeze v1

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A6 已完成生产路径前置门槛 / cutover contract / rollback operator contract freeze。本轮只写合同、evidence、handoff 和任务包状态；不改源码，不创建生产 DB，不迁移真实 JSON / sidecar，不切产品读写路径，不停写 JSON / sidecar。

## READ / WRITE SCOPE

### 读取

- 当前入口：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- R3 官方计划：`docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`。
- R3 schema / importer / rollback 合同：`docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`。
- R3-A5 supervisor checkpoint：`evidence/2026-06-11-root-treatment-r3-a5-supervisor-checkpoint-v1.md`。

### 写入

- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1.md`
- 本 evidence。
- `handoffs/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1-result.md`

## CONTRACT SUMMARY

合同冻结了：

- Production data roots：allowed roots / denied paths / production DB location contract。
- Mode contract：fixture_only、dry_run、production_preflight、copied_snapshot_apply、production_apply、read_cut、stop_write_json、rollback_operator。
- Backup / recovery contract：production apply 前必须具备 source root hash、backup manifest、DB path hash、rollback manifest、dry-run report、copied snapshot apply report 和 export verification。
- Transaction / lock contract：SQLite transaction、workflow state / sidecar lock、revision recheck、crash injection 和 cross-domain memory + audit transaction acceptance。
- Read-cut / stop-write gates：read-cut 和 stop-write 不得合并，stop-write 只能在 observation / export / rollback drill 后单独批准。
- Rollback operator contract：supervisor-only、dry-run by default、preserve DB for audit、no automatic retry、no destructive deletion。
- Evidence / handoff contract：每个后续生产任务包必须记录 commit、hash、manifest、allowed roots、denied paths、tests、rollback drill 和 do-not-claim。
- Future task split：R3-A7 到 R3-A13。

## GATE MATRIX

| Gate | R3-A6 decision |
| --- | --- |
| production DB create | not authorized |
| production preflight metadata/hash | next recommended task R3-A7 |
| copied production snapshot temp DB apply | later task R3-A8 |
| production DB apply | later task R3-A9 |
| read-cut | later task R3-A10 |
| stop-write JSON | later task R3-A12 |
| rollback production workflow | contract only, no restore |
| multi-agent parallel real execution | still locked until R3 final acceptance |

## NEXT TASK SPLIT

Immediate next recommended task:

- R3-A7 production preflight scanner / report：只读生产工作台自有 JSON / sidecar metadata、hash、schema、revision、backup readiness；不建 production DB，不写 production root，不读 `.codex`，不读取 secret / transcript。

后续建议：

- R3-A8 copied production snapshot temp DB apply and export verification。
- R3-A9 production DB initializer + apply with backup manifest, no read-cut。
- R3-A10 limited read-cut behind feature flag / fallback。
- R3-A11 production observation period and export verification。
- R3-A12 stop-write JSON decision and rollback drill。
- R3-A13 transaction acceptance across memory + audit and R3 final acceptance。

## CHECKS RUN

- `git diff --check`：pass。
- `rg -n "R3-A6|production cutover|rollback operator|生产路径|停写 JSON|多 agent 并行真实执行" docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`：pass, expected contract hits。
- `rg -n "生产 DB 创建完成|生产读切 DB 完成|JSON / sidecar 停写已完成|多 agent 并行真实执行解锁" CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md evidence/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1.md handoffs/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1-result.md`：ran；hits are do-not-claim / not-accepted / boundary wording only。

本轮只改文档，未运行 cargo / npm。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A6 只是 contract freeze，不是 production preflight implementation。
- P2：R3-A7 尚未创建；入口同步应在主管 checkpoint 完成后指向 R3-A7。

## BOUNDARY CONFIRMATION

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未切任何产品读写路径到 DB。
- 未让真实 app read model 读 DB。
- 未停止 JSON / sidecar 写入。
- 未把 JSON 降为生产 fallback。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## DO NOT CLAIM

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明生产 DB 创建完成。
- 不声明生产双写期开始。
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
