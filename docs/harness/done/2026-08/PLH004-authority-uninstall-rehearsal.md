# PLH004 唯一项目权威与旧 Harness 卸载演练

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：在隔离副本先把项目事实迁到 Lite 形状，再按 old manifest/hash 卸载 Adaptive Harness v0.5，验证项目资料、代码和 WIP 哨兵不变。

干完的标准：唯一权威链、AGENTS/CLAUDE 人工适配、旧文件四类表、Lite 安装/重复安装/卸载/恢复均通过。

允许动：

- `/private/tmp/product-line-harness-lite-uninstall-rehearsal-*` [新增]
- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]

## 步骤

1. 从 PLH003 的代码实物重建当前事实，不照抄旧 READY、旧 active-id 或历史包。
2. 适配 AGENTS、CLAUDE、README 和活动 bridge 为 Lite 单一入口。
3. 先迁移 CURRENT/AUTHORITY 有效事实，再退出 old-owned runtime/config/manifest。
4. 保留项目 commit-msg、catch ledger、业务决策、合同、证据和 Code Map 资料。
5. 验证旧活动接线为零、重复安装零写入、快照可恢复。
