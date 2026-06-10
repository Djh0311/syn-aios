# Handoff：Stage E / E6 Runtime Session Attention And Readback Failure Boundary v1

日期：2026-06-06

## 1. 接受范围

E6 已完成，接受为：

- `RuntimeSessionAttention` / `ReadbackBoundaryStatus` / `SessionRunStatusSummary` 最小读模型。
- `WorkbenchSnapshot.runtime_session_attention[]`。
- `WorkbenchSnapshot.session_run_status_summaries[]`。
- 智能体页 `运行关注 / E6` 只读面板。
- 右侧 `运行中` / `通知` / `待办` 内部摘要。
- 秘书只读模型解释 runtime attention 风险和查看建议。
- 离线 UI 测试覆盖 waiting permission、guard blocked、readback unavailable、readback failed。

证据：

- `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`

## 2. 不接受范围

E6 不接受为：

- 真实 `codex exec` / `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- 自动重试完成。
- stop / restart 完成。
- 完整 runtime log store 完成。
- 诊断中心或阶段 G 真实 Tauri 全面验收完成。
- planned adapters 可发送 / resume。

本轮没有进入 E5 Level B；真实执行仍必须另行授权具体 session、cwd、prompt、读写范围、回滚和证据。

## 3. 关键实现

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- 扩展 `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- 接入 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

前端：

- 扩展 `prototypes/productized-desktop-shell/src/lib/types.ts`
- 智能体页新增 E6 只读状态面板：`prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- 右侧 `运行中` / `通知` / `待办` 接入 runtime summary：`prototypes/productized-desktop-shell/src/App.tsx`
- 秘书只读模型解释 E6：`prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- 离线测试覆盖 E6 UI / secretary 边界：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 4. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，11 scenarios passed
- `npm run build`，有既有 Vite chunk-size warning
- `cargo test --lib runtime_session_attention`
- `cargo test --lib session_continuation`
- `cargo test --lib session_operation`
- `cargo test --lib provider_availability`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`，230 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs src/session_continuation_store.rs src/runtime_session_attention.rs`

扫描：

- 禁止误导文案扫描无命中。
- 真实执行 / 敏感路径扫描有历史 runner、guard、fixture 和边界文案命中；E6 没有新增真实 runner 或 `.codex` 读写。

## 5. 手动测试清单

在应用里测试建议：

1. 打开应用，进入左侧 `智能体` 页面。
2. 找到 `运行关注 / E6` 面板。
3. 确认面板只显示 attention 数量、session summary、readback boundary、下一步查看建议。
4. 确认 `readback unavailable` 或 `readback failed` 不显示成真实 0 条读回。
5. 确认页面没有发送、resume、重试、停止、重启按钮。
6. 打开右侧 `运行中`，确认能看到 session run summary 和 readback 状态，点击只导航到查看页面。
7. 打开右侧 `通知`，确认只显示阻断 / failed / unavailable 摘要，不显示 raw log。
8. 打开右侧 `待办`，确认只显示需要用户查看的 attention，不替用户批准。
9. 打开右侧 `秘书只读摘要`，确认秘书只解释 E6 边界和查看建议，不提供发送、resume、批准、重试、停止或重启动作。
10. 全程确认没有出现 `真实 prompt 已发送`、`Codex 已收到任务`、`真实 readback 已完成`、`已自动重试`、`已停止 agent` 等文案。

本轮未做真实 Tauri 窗口截图验收，因此不能回收为阶段 G。

## 6. 下一步

建议下一步写并执行 E7：Stage E acceptance / E-to-F handoff。

E7 应重点复核：

- E1-E6 evidence / handoff 是否齐全。
- E5 Level A 与 E6 runtime attention 是否仍未越界到真实执行。
- planned adapters 是否仍不可执行。
- 是否允许进入阶段 F。

E7 不应直接进入 Level B 真实 `codex exec resume`，也不应启动 G1 runtime log。

## 7. 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
