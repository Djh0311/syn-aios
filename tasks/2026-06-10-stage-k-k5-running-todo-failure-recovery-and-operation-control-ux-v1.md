# Stage K / K5 Running Todo Failure Recovery And Operation Control UX v1

日期：2026-06-10

状态：已完成。

完成结论：`accepted_non_real_productization_slice`。记录见 `../evidence/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md` 与 `../handoffs/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1-result.md`。

本任务包用于在 K4 非真实记忆产品化切片完成后，继续推进 Stage K 原目标中的“自动化工作流 + 运行状态可理解 + 安全操作控制”。K5 本轮只做非真实 Codex 产品化切片：把现有 run queue、user confirmation queue、failure control、readback boundary、duplicate guard、stale cleanup 和 operation readiness 整理成普通用户可读 UI 与测试覆盖。

本文不授权新的真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，不启动 K3-B1 retry，不启动 K3-B2，不实现真实 retry / stop / restart。

## 1. 当前事实

- K4 已完成并收口为 `accepted_non_real_productization_slice`。
- Stage K architecture calibration v2 and gate 已完成，gate strict 通过，0 error / 0 warning。
- K3-B1 已执行但失败分类；retry 申请再次被安全审查拒绝。
- K3-B2 依赖 K3-B1 成功和复核，当前不得启动。
- J4 / H4 已有 run queue、failure control、duplicate guard、readback unknown-result 和用户确认边界。
- 现有 `runQueue.ts` 已能派生运行队列、待确认队列和失败控制摘要；`RunningWorkflowsView.tsx` 已有运行中工作流页基础展示。

## 2. 目标

K5 本轮交付：

1. 运行中工作流页新增或强化“操作控制 / 恢复建议”普通层摘要。
2. `readback_unavailable`、`readback_failed`、`timed_out`、`duplicate_blocked`、`blocked_by_guard`、`stale_cancelled` 等状态显示为未知 / 待处理 / 需确认，不能显示成真实 0 条结果。
3. retry / stop / restart / resume 只显示为“需重新确认 / 未实现 / 只读 readiness”，不能出现真实执行按钮或自动执行文案。
4. 待确认事项能按用户视角说明：要确认什么、风险是什么、会写哪里、不会自动发生什么。
5. 失败控制能按用户视角说明：失败分类、推荐下一步、是否允许重试提案、是否需要用户确认、是否涉及记忆补偿。
6. 开发者字段如 raw refs、store revision、sidecar path、runtime_log_ref、audit_refs 继续留在开发者详情或摘要层，不铺普通首屏。

## 3. 非目标

- 不执行真实 Codex。
- 不发送 prompt。
- 不做 K3-B1 retry。
- 不启动 K3-B2。
- 不真实 retry / stop / restart / resume。
- 不新增真实操作 Tauri command。
- 不 kill Codex 进程。
- 不自动清理真实 `.codex` 状态。
- 不自动写 FormalMemory。
- 不新增 provider credential store 或 model verification。
- 不接 planned adapters 真实执行。
- 不把普通浏览器 smoke 当真实 Tauri 验收。

## 4. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端读模型摘要。
- [x] 改已有页面局部 UI。
- [x] 改离线 UI 测试。
- [ ] 新增普通主导航入口。

普通 UI 应显示：

- 当前运行项：正在做什么、状态、人话原因、下一步。
- 待确认项：确认对象、风险、写项目 / 写 `.codex` / 写工作台记录。
- 失败恢复：失败分类、推荐下一步、能否提出重试、是否必须用户确认。
- 操作控制：retry / stop / restart / resume 的 readiness 和确认边界。
- 读回状态：未知 / 不可用 / 失败 / 超时均显示为“未知 / 不可用”，不显示为 0 条结果。
- 记忆补偿：如果失败或半完成捕获需要补证，提示去记忆页处理。

普通 UI 不显示：

- raw JSON。
- sidecar 绝对路径。
- store revision。
- prompt body。
- full transcript。
- raw stdout / stderr。
- `/Users/yoyi/.codex` 内部路径内容。
- H/J/K/PCR 阶段术语作为用户操作文案。
- 真实 `codex exec` / `codex exec resume` 命令串。

## 5. 改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

默认不改：

- Rust runner / Product Command 真实执行语义。
- Tauri command wrapper。
- `workflow-state.v0.json` 顶层结构。
- FormalMemory store schema。
- provider / credential / adapter 真实接入。

## 6. 验收

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`

扫描必须确认：

- 不出现真实 retry / stop / restart 已实现或自动执行文案。
- `readback_unavailable` / `readback_failed` / `timed_out` 不被显示为 0 条结果。
- K3-B1 / K3-B2 旧口径不被改写成已完成或可开始。
- 普通 UI 不暴露 raw sidecar / store revision / prompt body。
- 候选 / observation / capture 不被误写为 FormalMemory。

## 7. 接受口径

可接受为：

- K5 非真实 Codex 产品化切片完成。
- 运行中、待办、失败恢复和操作控制 readiness 在普通 UI 里可读。
- retry / stop / restart / resume 只作为需确认或 deferred 的产品边界展示。
- 不依赖 K3-B1 retry 或 K3-B2。

不接受为：

- 真实 retry / stop / restart / resume 已实现。
- 真实 Codex 已被再次执行。
- 自动清理真实 `.codex` 状态完成。
- K3-B1 retry 成功。
- K3-B2 可开始。
- K5 全量完成。
- Stage K 完成。

## 8. 完成记录

本轮已完成：

1. `RunQueueReadModel` 新增 `operation_control_summary` 前端只读摘要。
2. 运行中工作流页新增“操作控制 / 恢复建议”普通层。
3. retry / stop / restart / resume 仍保持只读 readiness / 需确认 / 后续任务，不新增真实执行按钮。
4. readback unavailable / failed / timed_out / null result count 继续显示为未知 / 不可用，不显示为 0。
5. 秘书只读模型改为“运行队列”产品口径，不生成 retry / stop / restart / resume / send action proposal。
6. 离线测试新增 K5 schema、文案和误导文案黑名单断言。

验证通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，14 passed
- `npm run build`，仅既有 Vite chunk size warning
- `node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict`，0 error / 0 warning

复核线最终结论：通过，无 P0/P1/P2，允许主管线将 K5 本轮收口为 `accepted_non_real_productization_slice`。
