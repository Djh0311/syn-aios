# Root Treatment R3-A6 Production Cutover Contract And Rollback Operator Freeze v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A6 contract freeze 已完成，等待主管 checkpoint / 入口同步 / commit。

## CONTRACT PATH

- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`

## WHAT WAS FROZEN

- Production data roots and denied paths。
- Mode contract：fixture_only / dry_run / production_preflight / copied_snapshot_apply / production_apply / read_cut / stop_write_json / rollback_operator。
- Backup and recovery manifest requirements。
- Transaction and lock requirements。
- Read-cut and stop-write gates。
- Rollback operator contract。
- Evidence / handoff contract for future production tasks。
- Recommended R3-A7 到 R3-A13 split。

## NEXT RECOMMENDED TASK

R3-A7 production preflight scanner / report。

R3-A7 只能只读生产工作台自有 JSON / sidecar metadata、hash、schema、revision 和 backup readiness；不得创建 production DB，不得写 production root，不得读写 `/Users/yoyi/.codex`，不得读取 secret / transcript。

## HARD BOUNDARIES

- R3-A6 不创建生产 DB。
- R3-A6 不迁移真实 JSON / sidecar。
- R3-A6 不切产品读写路径。
- R3-A6 不停写 JSON / sidecar。
- R3-A6 不执行真实 Codex。
- R3-A6 不解锁多 agent 并行真实执行。
- Stage L / K3-B1 / K3-B2 仍 deferred during root treatment。

## WHAT NOT TO CLAIM

- 不声明 R3 SQLite 迁移开始或完成。
- 不声明生产 DB 创建完成。
- 不声明生产双写期开始。
- 不声明生产读切 DB 完成。
- 不声明 JSON / sidecar 停写。
- 不声明 rollback production workflow 完成。
- 不声明多 agent 并行真实执行解锁。
- 不声明 Stage L / K3-B1 / K3-B2 已恢复。
