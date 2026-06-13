# Evidence: Root Treatment / R4-H1 Frontend Types Domain Split Hard Target v1

日期：2026-06-13

状态：已完成。

## 1. 范围

本轮执行 `tasks/2026-06-13-root-treatment-r4-h1-frontend-types-domain-split-hard-target-v1.md`。

用户确认的停止边界：

- 按主管线建议确认 R2 后段收口口径。
- 执行 R4-H1 `types.ts` 分域。
- 完成验证、evidence / handoff / checkpoint 后停止。
- 不自动进入 R4-H2、R3 Level B、R5。
- 不把此前挂起的宽目标当作无限续跑许可。

## 2. R2 收口口径

已将 `decisions/2026-06-13-root-treatment-r2-late-stage-closure-track-v1.md` 从草案更新为已确认口径：

- R2 后段进入“明确下降轨道 + 冻结 deferred”状态。
- 当前不再继续开低收益 R2-T 迁移包。
- 后续若重开 R2，必须先提出能明确降低 `lib.rs` 棘轮指标、且不触碰禁迁清单的下降轨道。
- 唯一 deferred `compact_last_message_summary_preserves_workflow_machine_control_marker` 保持 deferred，不单独立项。

不接受为 R2 完成：`lib.rs <= 3,000` 目标尚未达成。

## 3. 类型分域实现

修改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/types/agentSession.ts`
- `prototypes/productized-desktop-shell/src/lib/types/execution.ts`
- `prototypes/productized-desktop-shell/src/lib/types/memory.ts`
- `prototypes/productized-desktop-shell/src/lib/types/workflow.ts`

分域结果：

- `types.ts` 保留为兼容 re-export 入口，并保留 `WorkbenchSnapshot` 聚合类型。
- `agentSession.ts` 承载 adapter、session operation、provider availability、session continuation、codex-local、runtime attention、runtime log、diagnostics 类型。
- `execution.ts` 承载 real execution product command、project workflow automation、worker protocol / run unit / adapter diagnostics 类型。
- `memory.ts` 承载 memory candidate、formal memory、observation、memory capture、lint、entity relation、mature pattern、task memory packet 类型。
- `workflow.ts` 承载 plan authorization、project director task plan、workflow state、project blackboard、task package、pending action、workflow dispatch 类型。
- `types/canvas.ts` 保持不变。

行数：

```text
93   prototypes/productized-desktop-shell/src/lib/types.ts
946  prototypes/productized-desktop-shell/src/lib/types/agentSession.ts
869  prototypes/productized-desktop-shell/src/lib/types/execution.ts
1391 prototypes/productized-desktop-shell/src/lib/types/memory.ts
1746 prototypes/productized-desktop-shell/src/lib/types/workflow.ts
127  prototypes/productized-desktop-shell/src/lib/types/canvas.ts
```

`types.ts` 从 4,998 行降到 93 行，下降 4,905 行；新增类型文件均低于 2,000 行。

## 4. 验证

已通过：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 14`。

```text
npm run build
```

结果：通过；保留既有 Vite chunk size warning。

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：pass，0 errors，0 warnings；`types.ts: 93/4998 (decreased)`。

```text
git diff --check
```

结果：通过。

```text
rg -n "\bany\b" prototypes/productized-desktop-shell/src/lib/types.ts prototypes/productized-desktop-shell/src/lib/types/agentSession.ts prototypes/productized-desktop-shell/src/lib/types/execution.ts prototypes/productized-desktop-shell/src/lib/types/memory.ts prototypes/productized-desktop-shell/src/lib/types/workflow.ts
```

结果：无命中。

```text
rg -n "from \"\\.\\/lib\\/types\"|from \"\\.\\/types\"" prototypes/productized-desktop-shell/src
```

结果：现有调用仍通过 `./lib/types` / `./types` 兼容入口引用；本轮未大规模改 import 路径。

## 5. 边界确认

本轮没有：

- 修改 Rust / Tauri / DB / sidecar schema / workflow state schema。
- 修改 UI、CSS、布局、文案或交互。
- 修改产品行为、读写路径、真实执行路径。
- 改变 `WorkbenchSnapshot` 字段语义。
- 删除类型字段、改字段名或放宽类型为 `any`。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行 R3 Level B。
- 进入 R4-H2 / R5。

## 6. 不接受为

本轮不接受为：

- R2 完成。
- R4 完成。
- `types.ts` 全部治理完成。
- `WorkbenchSnapshot` 按页查询完成。
- 页面真实数据源迁移完成。
- UI 重做或视觉优化完成。
- R3 Level B 执行。
- R5 开始。
- 真实 Codex 执行、`.codex` 读写或 backlog 功能解冻。
