# Handoff：Stage E / E5 Codex-local Controlled Send Resume Minimal Loop v1

日期：2026-06-06

## 1. 接受范围

E5 已完成 Level A，接受为：

- `codex-local` 受控 send / resume 最小代码路径。
- E4 preview / guard 到 E5 user confirmation record 的闭环。
- 工作台自有 sidecar：`session-continuations.v1.json`。
- `load_session_continuation_store`、`confirm_controlled_session_continuation`、`run_controlled_session_continuation_stub` 三个 Tauri command。
- `WorkbenchSnapshot.session_continuation_store` 后端读模型。
- Level A stub attempt / audit ref / readback unavailable placeholder。
- 智能体页既有区域内的 E5 只读状态面板。
- 秘书只读解释 E5 状态，但不生成执行 action proposal。

证据：

- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`

## 2. 不接受范围

E5 不接受为：

- 真实 `codex exec resume` 已执行。
- prompt 已发送给 Codex。
- Codex 已收到任务。
- 真实 readback 已完成。
- 真实会话继续已验收。
- planned adapters 具备 send / resume 能力。
- stop / restart / delete / export / favorite 完成。
- 完整 runtime log、自动重试、取消恢复或运维诊断完成。
- 阶段 E 总验收完成。
- 阶段 G 真实 Tauri 全面验收完成。

本轮是 Level A。Level B 真实执行 deferred，仍需要用户另行明确授权具体 target session、cwd、prompt、`.codex` 范围、回滚和证据。

## 3. 关键实现

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- 扩展 `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- 接入 `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- 接入 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

前端：

- 扩展 `prototypes/productized-desktop-shell/src/lib/types.ts`
- 新增 Tauri wrapper：`prototypes/productized-desktop-shell/src/lib/tauri.ts`
- 智能体页新增 E5 只读状态面板：`prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- 秘书只读模型解释 E5：`prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- 离线测试覆盖 E5 Level A UI / secretary 边界：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 4. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，11 scenarios passed
- `npm run build`，有既有 Vite chunk-size warning
- `cargo test --lib session_continuation`
- `cargo test --lib controlled_session_continuation`
- `cargo test --lib session_continuation_store`
- `cargo test --lib session_operation`
- `cargo test --lib provider_availability`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`，228 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs src/session_continuation_store.rs`

扫描：

- 禁止误导文案扫描无命中。
- 真实执行 / 敏感路径扫描有历史命中；E5 新增命中是 command preview 字符串和敏感路径 guard，不是新真实 runner。
- shell 安全扫描有大量历史 Markdown/code-fence 命中；本轮命令使用单引号，没有命令替换偏差。

## 5. 手动测试清单

在应用里测试建议：

1. 打开应用，进入左侧 `智能体` 页面。
2. 找到 `会话继续预览 / 权限预览` 面板，确认 E4 preview 仍显示 target session、project、workflow、node、cwd、sandbox、readback 和 audit impact。
3. 找到 `受控 continuation / E5 Level A` 面板。
4. 在没有 continuation sidecar 记录时，确认面板显示尚未创建 record，并说明 Level A 只能 stub 验收、真实执行未授权。
5. 如后续通过命令写入 stub record，确认面板显示 `stub 验收`、`readback unavailable`、`prompt_sent:false`、`real_codex_executed:false`、`writes_codex_home:false`。
6. 确认页面没有真实发送、resume、执行、重试按钮。
7. 确认页面不出现 `已发送`、`已 resume`、`Codex 已收到任务`、`真实 Codex 已执行`、`worker 执行中`、`readback 已完成`。
8. 打开右侧 `秘书只读摘要`，确认秘书只解释 E5 边界，不提供发送、resume、批准或重试动作。
9. 不要把 E5 面板当成真实会话继续验收；真实执行仍需 Level B 任务和用户授权。

本轮未做真实 Tauri 窗口截图验收，因此不能回收为阶段 G。

## 6. 下一步

建议下一步拆 E6：Runtime Session Attention And Readback Failure Boundary。

E6 应重点收敛：

- waiting permission
- running / timed out
- readback failed
- readback unavailable
- blocked by guard
- needs user

E6 不应直接做完整自动重试系统，也不应绕过 Level B 授权进入真实 `codex exec resume`。

## 7. 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
