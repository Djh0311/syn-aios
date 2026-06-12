# Evidence: Root Treatment / R4-A50 Strategy Adjustment And Ratchet Gate Hardening v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a50-strategy-adjustment-and-ratchet-gate-hardening-v1.md`

Planning baseline commit：`f3382efc5f3d87e7d21eef91c945a2d0516ce77f`

Implementation commit：`b18071e26f42f127f48202651377b132e7ec0dbe`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`810d2e62e61622c028976753e2fed7ebf29c7cd9`

## 1. 本轮目标

R4-A50 落实 `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` 的策略调整：

- 停止继续立项不降低棘轮指标的低产出 helper 拆分。
- 将 shape gate 的 ratchet 语义从初始水位线改为历史最低收口值。
- 将已裁决的 Tauri command total 97 固化为 gate 基线，清除重复 warning。
- 在正式计划中补 R4-6 停止线和后续立项规则。

本轮是治理策略与 gate 硬化，不是产品功能解冻，不执行真实 Codex，不启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 2. 改动范围

新增：

- `tasks/2026-06-12-root-treatment-r4-a50-strategy-adjustment-and-ratchet-gate-hardening-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a50-strategy-adjustment-and-ratchet-gate-hardening-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a50-strategy-adjustment-and-ratchet-gate-hardening-v1-result.md`
- `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md`
- `docs/own-agent-and-company-vision-v1.md`

修改：

- `scripts/harness/workbench-shape-gate.js`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `backlog.md`

未修改：

- 产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema、真实执行路径。
- `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 将在 checkpoint commit 同步。

## 3. 具体实现

`workbench-shape-gate.js`：

- 新增 `RATCHET_POLICY = "historical_lowest_closed_value"`。
- `COMMAND_BASELINE_TOTAL` 从 96 调整为 97，并新增 R4-A2 read-only skeleton command 的裁决说明。
- `RATCHET_WATERLINES` 更新为历史最低收口值：
  - `lib.rs = 13965`
  - `offline-permission-dialog.test.tsx = 3404`
  - `ProjectsView.tsx = 5897`
  - `AgentView.tsx = 3118`
  - `types.ts = 4998`
  - 其他 ratchet 文件保持当前水位。
- gate 输出展示 `Ratchet policy: historical_lowest_closed_value`。
- `ratchet_file_increased` 文案改为 historical-low ratchet waterline。

正式计划：

- R4-6 完成判据：主文件 `offline-permission-dialog.test.tsx` ≤ 2,000 行，或剩余内容已属单一域不可再拆，以先到者为准。
- 后续 R4-6 任务包必须写明预计降低哪个棘轮指标多少行。
- 不降低任何棘轮指标的 helper 拆分包不得立项。
- 已低于 2,000 行新文件上限的 helper 文件不得单独作为拆分对象。
- 原“R4-A50 继续中等粒度离线交互测试按域拆分”取消；R4-A50 改为策略调整与 shape gate 硬化。
- 后续方向改为 R2 后段 inline tests 迁移复评、R3 Level B 窗口计划、checkpoint 轮转方案、R4 硬目标。

咨询线落账：

- `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` 是本轮策略输入。
- `docs/own-agent-and-company-vision-v1.md` 明确标注“愿景/设计依据文档，不是任务授权”。
- `backlog.md` 新增条目均为解冻后候选，不在 R4-A50 实施。

## 4. 验证

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，0 errors，0 warnings。

关键输出：

```text
Ratchet policy: historical_lowest_closed_value
Tauri commands: 97 total; 0 in lib.rs
```

负向验收：

1. 临时给 `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts` 追加 2 行注释。
2. 运行 `node scripts/harness/workbench-shape-gate.js --mode check`。
3. gate 按预期失败：

```text
Status: fail
Errors: 1
Warnings: 0
ratchet_file_increased
projectCanvas.ts: 2052/2050
```

4. 已用 `apply_patch` 撤回临时行。
5. 重新运行 shape gate：通过，0 errors，0 warnings。
6. `git diff -- prototypes/productized-desktop-shell/src/lib/projectCanvas.ts` 无输出。

```text
git diff --check
```

结果：通过，无输出。

未运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test`

原因：R4-A50 不改产品源码、UI、CSS、Rust/Tauri 产品路径或测试行为；必要验收是 shape gate、负向 gate 和 diff check。

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: 无
P1: 无
P2: 无
```

复核确认：

- Diff 范围符合 A50：只改 shape gate、正式计划、`backlog.md`，新增 A50 任务包、strategy handoff、vision 文档。
- 未触碰产品代码、UI/CSS、Rust/Tauri 产品路径、DB/schema 或真实执行路径。
- Shape gate 已改为 `historical_lowest_closed_value`，Tauri command baseline 固化为 97。
- Ratchet 水位已收紧到历史低点。
- 正式计划已写入 R4-6 停止线和棘轮收益立项规则，并取消 A50 继续拆 helper 的方向。
- Strategy handoff 明确不是新功能授权、不授权 R3 Level B；vision 文档是愿景/设计依据，不是任务授权；backlog 新增内容均为解冻后候选。

Residual risk：

- `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 的 A49/A50 暂态入口仍需在 checkpoint 清理。
- 复核线未重跑 shape gate 和负向验收，只做静态只读 diff、文本边界和 `git diff --check` 复核；主管线验证已记录在本 evidence 第 4 节。

## 6. 边界确认

本轮没有修改产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema 或真实执行路径。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot，没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮不接受为：

- R4 完成。
- R4-6 全部完成。
- R3 Level B 执行或完成。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- UI 行为 / 视觉修改。
- backlog 功能解冻或愿景文档功能实施。
