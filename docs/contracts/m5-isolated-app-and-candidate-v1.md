# M5 隔离 App 与阶段候选合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R07 冻结）**
- 关系：只解释如何用冻结 DTO 接现有项目壳；**不改 M1–M4 正文与 hash，不重画页面布局**。

## 规则

- 项目默认入口走持久 Project Supervisor。
- Proposal / 授权 / 运行 / 恢复 / 报告 / 独立审查 / 结果决定只在用户明确动作后展开。
- 隔离证据使用隔离 app-data 与 scratch projects、fake roles/runtime。
- 窗口截图 / 真实 Tauri 交互若未执行，必须记 `NOT_EXECUTED`，不得写成 PASS。
- 旧执行入口只登记 compatibility，不物理删除。
- 本叶完成后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`；不关闭 stage-14，不宣布 M5 完成。
