# Root Treatment / P2-1 R3 Level B Window Plan Document v1

日期：2026-06-13

状态：已完成；只写窗口计划文档，未执行 R3 Level B。

性质：P2-1 欠项排队任务包。本文只排队“写窗口计划文档”，不执行 R3 Level B。

Planning baseline：`7deb7f38f4574831e8a2e4561c29a534808635a8` 附近的主管线回归 checkpoint；本任务不改产品代码。

完成记录：

- 窗口计划：`docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`
- evidence：`evidence/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1.md`
- handoff：`handoffs/2026-06-13-root-treatment-p2-1-r3-level-b-window-plan-document-v1-result.md`

## 1. 背景

`handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` 的 P2-1 指出：R3 Level A 演练已完成多项，但真实 Level B 动作集中在尾部，缺少窗口计划。

当前事实：

- R3-A9 / A10 / A11 / A12 / A13 均只完成 Level A。
- 真实 workbench state root 未读取。
- 真实 workbench-owned production DB 未创建。
- 未切 app startup / Tauri command / UI / 产品全局读写路径。
- 未停写 JSON / sidecar。

## 2. 目标

产出窗口计划文档：

- `docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`

该文档必须写清：

- Level B 前置清单。
- 用户需在场步骤。
- 预计时长。
- allowed roots / denied paths。
- production DB path / backup path / report path / rollback manifest path。
- before / after source hashes。
- execution record 格式。
- fresh verify 清单。
- 中止条件。
- rollback / recovery 序列。
- 哪些动作必须再次用户拍板。

## 3. 允许范围

允许修改：

- `docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`
- 当前任务包、evidence、handoff。
- 必要时在 `CURRENT.md` 只写“窗口计划已写，未执行”checkpoint。

## 4. 禁止范围

禁止：

- 执行任何 R3 Level B。
- 读取真实 workbench state root。
- 创建真实 production DB。
- 切 read path / write path。
- 停写 JSON / sidecar。
- 执行 rollback 或 recovery。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / screenshot。
- 解冻 backlog 功能。

## 5. 验收

必须通过：

- 文档包含前置、步骤、备份、回滚、中止、fresh verify、用户在场点。
- 文档明确“不执行 Level B”。
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

可选：

- 不跑 `cargo test` / `npm`，但 evidence 必须说明原因。

## 6. 不接受为

本包不接受为：

- R3 Level B 已执行。
- R3 已完成。
- 生产 SQLite DB 已创建。
- read-cut / stop-write 已发生。
- rollback 已验证于真实数据。
- 多 agent 并行真实执行解锁。
- 真实 Codex 执行或 `.codex` 接触已授权。
