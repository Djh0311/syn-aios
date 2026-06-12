# Root Treatment / R4-A50 Strategy Adjustment And Ratchet Gate Hardening v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`f3382efc5f3d87e7d21eef91c945a2d0516ce77f`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：待回填。

本文是 Root Treatment / Stage R 的 R4-A50 任务包。R4-A50 不再继续做低产出的 helper 拆分，而是落实 `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` 中已经授权的 P1-1 / P1-3 / P2-3 调整：补定 R4-6 停止线、取消“不降低棘轮指标”的 A50 拆分方向、把 shape gate 改为历史最低收口值水位线，并把已裁决的 Tauri command 97 写入基线。

## 0. 全局主管理解

已知事实：

- R4-A49 已完成并回填 hash。
- 最新策略审查指出：A45-A49 后续 helper 拆分已不降低 `offline-permission-dialog.test.tsx` 等棘轮指标，继续拆会造成治理成本高、产出低。
- shape gate 当前仍使用初始水位线，导致已下降文件存在无声回涨空间。
- Tauri command total 97 已经由 R4-A2 只读 skeleton command 裁决为合法，但 gate 仍以 96 为基线产生重复 warning。
- 用户递交最新策略 handoff 即视为授权按本文调整 R 阶段执行策略；本文未授权真实 Codex 执行、R3 Level B、UI 重做或 backlog 功能解冻。

核心判断：

```text
R4-A50 是治理策略与 gate 硬化任务，不是继续拆 helper，也不是功能解冻。
```

## 1. Execution Mode

Execution Mode：Supervisor-led strategy adjustment and shape gate hardening with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `scripts/harness/workbench-shape-gate.js`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- 当前任务包、对应 evidence / handoff。

允许同步到 checkpoint：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`

允许记录为咨询线落账、但不实施其中功能：

- `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md`
- `docs/own-agent-and-company-vision-v1.md`
- `backlog.md`

## 3. Prohibited

R4-A50 禁止：

- 修改产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema 或真实执行路径。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 执行 R3 Level B、K3-B1 retry、K3-B2、多 agent 并行真实执行。
- 解冻 backlog 功能或实施愿景文档里的功能项。
- 把 R4-A50 冒充为 R4 完成、R3 Level B 完成、生产读切完成、UI 重做完成或真实执行恢复完成。

## 4. Expected Implementation

1. `workbench-shape-gate.js`：
   - 将 Tauri command total baseline 从 96 调整为 97，并添加已裁决说明。
   - 将 `RATCHET_WATERLINES` 更新为当前历史最低收口值：`lib.rs=13965`、`offline-permission-dialog.test.tsx=3404`、`ProjectsView.tsx=5897`、`AgentView.tsx=3118`、`types.ts=4998`，其他未下降文件保持当前值。
   - 报告中明确 ratchet policy 为 `historical_lowest_closed_value`。
   - 正常 gate 不再出现 `tauri_command_total_increased` warning。
2. 官方计划：
   - 明确 R4-6 停止线：主文件 ≤ 2,000 行，或剩余内容已属单一域不可再拆，以先到者为准。
   - 明确后续 R4-6 立项必须降低棘轮指标；不降低棘轮指标的 helper 拆分包不得立项。
   - 把 R4-A50 从“继续拆分”改为策略调整 / gate 硬化。
   - 登记后续方向：R4 硬目标、R2 后段 inline tests 复评、R3 Level B 窗口计划、checkpoint 轮转方案。
3. 不改产品行为。

## 5. Verification

必须通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- 人为对任一 ratchet 文件临时 +1 行，`node scripts/harness/workbench-shape-gate.js --mode check` 必须 fail；随后撤回临时行。
- `git diff --check`

可选但建议：

- `npm run typecheck`
- `npm run test:offline-interaction`

## 6. Acceptance Boundary

可接受为：

- R4 strategy adjustment 落地。
- shape gate ratchet waterlines 改为历史最低收口值。
- Tauri command 97 已裁决 warning 固化进基线。
- R4-A50 不再是低产出 helper 拆分。

不可接受为：

- R4 完成。
- R4-6 全部完成，除非另有停止线判断和证据。
- R3 Level B 执行或完成。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- UI 行为 / 视觉修改。
- backlog 功能解冻。
