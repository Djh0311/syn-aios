# Root Treatment / R3-A9 Supervisor Checkpoint v1

日期：2026-06-11

## STATUS

`DONE`

主管线已回收 R3-A9 Level A：production DB initializer + apply with backup manifest / no read-cut fixture rehearsal。

Implementation commit：`52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`

## Accepted Scope

R3-A9 只接受为：

- fixture / temp production DB initializer + apply helper contract。
- backup manifest / apply manifest / export manifest / rollback boundary 证据链。
- DB -> JSON export dry-run verification。
- failure injection matrix。
- Level A source root 仅限 repo fixture 或 temp root。

R3-A9 不接受为：

- Level B real workbench-owned production DB apply。
- 真实 workbench state root 导入。
- production read-cut。
- JSON / sidecar stop-write。
- rollback production workflow。
- R3 完成。
- 多 agent 并行真实执行解锁。

## Fresh Verify

主管线复跑：

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo test --lib sqlite_snapshot`：PASS，13 passed。
- `cargo test --lib sqlite_preflight`：PASS，8 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_observation`：PASS，15 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，424 passed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。

提交前 P2 tightening 后复跑：

- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。
- forbidden true-flag scan：PASS，无命中。
- sensitive marker scan：仅命中 helper denied-marker 常量。

Known warning：Cargo 仍提示既有 `JsonRpcError::invalid_params` unused；非 R3-A9 引入。

## Review Line

只读复核线 thread `019eb474-2fab-77a0-a327-ad055749b1e1` 结论：

- `STATUS: CLEAR`
- P0：无。
- P1：无。
- P2：无。
- 建议主管线提交。

复核线曾指出两个 P2，主管线提交前已修补：

- backup manifest write failure injection 从直接返回改为 backup root 准备后、DB 创建前失败，并断言 marker 存在、DB/report/正式 backup manifest 不存在。
- evidence / task package 的 forbidden true-flag 扫描说明改为不造成机械扫描自命中。

## Authority Sync

主管线同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`

同步口径：

- R3-A9 Level A 已完成。
- Level B 未执行。
- 当前下一步是 R3-A10 limited read-cut planning / task package 或 R3-A9 Level B 决策。
- 两条路径都必须先有任务包、回滚策略和 fresh verify。

## Boundary Confirmation

- 未执行 Level B。
- 未读取真实 workbench state root。
- 未创建真实 workbench-owned production DB。
- 未切产品读路径到 DB。
- 未停写 JSON / sidecar。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未启动 Tauri / Browser / Chrome / Vite / screenshot。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
