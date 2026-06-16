# R3 B4b Stop-Write Decision Evidence

日期：2026-06-16

## 结论

B4b 受控停写决策窗口已跑通。结果是 `ready_but_not_executed`：系统判断前置条件满足，可以进入未来停写窗口的候选状态，但本窗没有执行真实停写。

这次只读了真实 B1 数据库、真实源目录、B3b projection 和 B3b observation report；只在 B4 工作目录写了 decision report 和 dry-run rollback manifest。

## 真实结果

- B1 DB hash 前后不变：`12d65f21ae383b72afd1b23347548974502ba60ca6a4143ca6b6fc94270f03ba`
- 源目录 B4 fallback hash 前后不变：`ae0797f8c5fc4c156cc0f5f15ed686af9f7871642e42afffb45530a621edd061`
- B3b projection hash：`87f62158ceef5dbe303d7c704dd47a2c3ae3775181e7ed1efbe59ff182e82175`
- B3b observation report hash：`9cd28f032c8bcd1b7ef9725cd1d8c92db05321a6656aa63834d0247304e1a8d8`
- 决策报告状态：`ready_but_not_executed`
- Rollback：`rollback_drill_only`
- Safety flags：除 `stop_write_decision_recorded=true` 外全部为 false

## 产物

- App 数据目录 report：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b4-stop-write-decision-20260616/reports/stop-write-decision-report.json`
- App 数据目录 rollback manifest：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/b4-stop-write-decision-20260616/rollback/stop-write-decision-rollback-manifest.json`
- 仓库 evidence 副本：`artifacts/stop-write-decision-report.json`
- 仓库 evidence 副本：`artifacts/stop-write-decision-rollback-manifest.json`

## 边界

- 未执行真实 stop-write。
- 未停写 JSON / sidecar。
- 未切产品全局读写路径。
- 未改 UI / Tauri / startup。
- 未建库、未迁移数据。
- 未执行真实 Codex。
- 未读取或写入 `/Users/yoyi/.codex`。
- R3 Level B 仍未收口；真停写必须另开窗口并由用户再次批准。
