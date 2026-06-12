# Root Treatment / R4-H1 Frontend Types Domain Split Hard Target v1

日期：2026-06-13

状态：已完成。

性质：R4 硬目标首批任务包。本轮按用户确认的停止边界执行：只做 R2 收口口径确认和 R4-H1 `types.ts` 分域，完成后停止，不自动进入 R4-H2 / R3 Level B / R5。

Planning baseline：`7deb7f38f4574831e8a2e4561c29a534808635a8`。

## 0. 全局主管理解

R4-A7 曾做过一次前端类型分域，当前 `src/lib/types.ts` 仍为 4,998 行，只有 `src/lib/types/canvas.ts` 已拆出。R4-A50 后，后续 R4 包必须降低棘轮指标，不能再做不降行数的 helper 拆分。

本包目标是继续把 `types.ts` 从单体导出文件拆成领域类型文件，同时保持现有 `import type { ... } from "./lib/types"` 兼容。

## 1. 目标

将 `prototypes/productized-desktop-shell/src/lib/types.ts` 中的前端类型按领域拆分到 `src/lib/types/` 下，降低 `types.ts` 棘轮水位。

首批建议拆分域：

- `agentSession.ts`：adapter、session operation、provider availability、session continuation、runtime attention、session run status。
- `execution.ts`：real execution product command、project workflow automation、run / attempt / readback / permission decision 相关类型。
- `memory.ts`：formal memory、candidate、observation、lint、entity relation、mature pattern、memory capture 相关类型。
- `workflow.ts`：workflow state snapshot、task package、authorization、proposal、project workflow / node / dispatch 相关类型。

保留：

- `types.ts` 作为兼容 re-export 入口。
- `workbenchCoreTypes.ts` 不在本包中重写。
- `types/canvas.ts` 保持现状，除非需要同步 re-export。

## 2. 形状影响

预期：

- `types.ts` 从 4,998 行下降至少 800 行，目标下降 1,200 行以上。
- 新增类型文件每个低于 2,000 行。
- 不新增 runtime 逻辑，不改变序列化字段，不改变后端契约。
- 不修改 UI 视觉或页面行为。

若实际下降低于 800 行，执行线必须在 evidence 中解释原因；若无法降低棘轮指标，本包不得收口为完成。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/types/*.ts`
- 仅当类型拆分导致 import 需要最小修复时，允许修改前端 TS import，不允许改业务逻辑。
- 当前任务包、evidence、handoff。

允许新增：

- `src/lib/types/agentSession.ts`
- `src/lib/types/execution.ts`
- `src/lib/types/memory.ts`
- `src/lib/types/workflow.ts`
- 必要时新增 `src/lib/types/index.ts`，但不得让 import 路径大规模抖动。

## 4. 禁止范围

禁止：

- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 修改 UI、CSS、布局、文案或交互。
- 修改产品行为、读写路径、真实执行路径。
- 改变 `WorkbenchSnapshot` 字段语义。
- 为了拆类型而删除类型、改字段名或放宽类型为 `any`。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 5. 实现步骤

1. 读取 `types.ts` 的导出分布，生成拆分清单。
2. 先抽 agent/session/provider/continuation/runtime 类型到 `types/agentSession.ts`。
3. 再抽 execution / automation 类型到 `types/execution.ts`。
4. 再抽 memory 相关类型到 `types/memory.ts`。
5. 再抽 workflow 相关类型到 `types/workflow.ts`。
6. `types.ts` 保留 re-export 和无法安全归域的少量类型。
7. 跑 typecheck、离线测试、build 和 shape gate。

## 6. 验证

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `rg -n "from \"\\.\\/lib\\/types\"|from \"\\.\\/types\"" prototypes/productized-desktop-shell/src`
- `wc -l prototypes/productized-desktop-shell/src/lib/types.ts prototypes/productized-desktop-shell/src/lib/types/*.ts`

## 7. 复核要求

复核线重点检查：

- `types.ts` 是否真实下降。
- 新文件是否低于 2,000 行。
- 字段名和导出兼容性是否保持。
- 是否引入 `any`、删除字段或修改产品契约。
- 是否误改 UI / CSS / Rust / schema。

## 8. 不接受为

本包不接受为：

- R4 完成。
- `types.ts` 全部分域完成。
- `WorkbenchSnapshot` 按页查询完成。
- 页面真实数据源迁移完成。
- UI 重做或视觉优化完成。
- R3 Level B 执行。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。

## 9. 完成记录

完成日期：2026-06-13。

实现结果：

- `src/lib/types.ts` 从 4,998 行降到 93 行，继续作为兼容 re-export 入口。
- 新增 `src/lib/types/agentSession.ts`，946 行。
- 新增 `src/lib/types/execution.ts`，869 行。
- 新增 `src/lib/types/memory.ts`，1,391 行。
- 新增 `src/lib/types/workflow.ts`，1,741 行。
- 现有 `src/lib/types/canvas.ts` 保持不变，127 行。

验证已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，通过，仅保留既有 Vite chunk size warning
- `node scripts/harness/workbench-shape-gate.js --mode check`，pass，0 errors，0 warnings，`types.ts: 93/4998 (decreased)`
- `git diff --check`
- `rg -n "\bany\b" ...` 对本轮拆分类型文件无命中

记录：

- Evidence：`evidence/2026-06-13-root-treatment-r4-h1-frontend-types-domain-split-hard-target-v1.md`
- Handoff：`handoffs/2026-06-13-root-treatment-r4-h1-frontend-types-domain-split-hard-target-v1-result.md`
