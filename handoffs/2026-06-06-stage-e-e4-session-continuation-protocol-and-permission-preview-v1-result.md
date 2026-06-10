# Handoff：Stage E / E4 Session Continuation Protocol And Permission Preview v1

日期：2026-06-06

## 1. 接受范围

E4 已完成，接受为会话继续协议和权限预览完成。

本轮落地：

- 后端 `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuardResult` 等最小模型。
- `WorkbenchSnapshot.session_continuation_previews[]`。
- E4 guard：missing binding、cwd 越界、敏感路径、planned adapter、缺 readback strategy、provider availability future-task 边界、缺用户确认。
- 智能体页局部“会话继续预览 / 权限预览”面板。
- 前端 fallback preview / guard。
- 秘书只读风险和查看建议。
- 后端单测、前端离线测试、类型检查、构建、扫描和文档回收。

证据：

- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`

## 2. 不接受范围

E4 不接受为：

- 真实发消息完成。
- 通用 `codex exec resume` 完成。
- prompt 已发送或 Codex 已收到任务。
- attempt / dispatch / readback / runtime log 已写入。
- worker / agent 已执行。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 继续会话能力完成。
- 阶段 G 真实 Tauri 全面验收完成。

E4 产品代码没有新增真实 Codex 执行路径，没有发送 prompt，没有迁移数据库，没有改 workflow state JSON 顶层结构。

偏差记录：收尾文档残留检查中，一条 `rg` 命令把带 Markdown 反引号的搜索模式放进 shell 双引号，shell 将反引号里的 `codex exec resume` 当作命令替换执行。输出显示没有 stdin prompt，并且访问 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 失败。这不是 E4 产品路径，但本轮不能严格声称“完全没有执行 Codex 命令 / 完全没有触碰 `/Users/yoyi/.codex`”。后续必须用单引号或 `rg -F` 搜索含反引号文本。

## 3. 修改文件

代码：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/sessionContinuation.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

文档：

- `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md`

## 4. 验证结果

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，11 passed
- `npm run build`，有既有 Vite chunk-size warning
- `cargo test --lib session_continuation`
- `cargo test --lib session_operation`
- `cargo test --lib provider_availability`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`，224 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs`

扫描：

- 禁止误导文案扫描无命中。
- 真实执行 / 敏感路径扫描有历史命中；E4 新增命中只用于敏感路径 guard 阻断。
- 另有一次收尾检查操作偏差：shell 反引号命令替换误触发 `codex exec resume`，没有 prompt，未完成 resume；详见 evidence。

## 5. 手动测试清单

在应用里测试时建议这样看：

1. 打开应用，进入左侧 `智能体` 页面。
2. 找到 adapter / provider / operation 区域附近的 `会话继续预览 / 权限预览` 面板。
3. 确认面板只显示 preview，不出现 `发送`、`发消息`、`resume`、`执行`、`重试` 等可点击按钮。
4. 确认 `codex-local` 的 send_message / resume 预览显示 target session、project、workflow、node、cwd、write roots、sandbox、readback 和 audit impact。
5. 确认 `codex-local` 完整绑定时状态仍是 `需要用户确认` 或 preview 状态，不显示已发送。
6. 确认 Claude Code / OpenClaw / OpenCode / OpenCode-like 显示 planned / blocked，不显示可继续会话。
7. 查看页面文案，确认出现“预览不是执行 / 不会发送 prompt / 不会执行 resume / 不会写 Codex 原生状态 / 不会写 attempt、dispatch 或 readback”等边界。
8. 打开右侧秘书只读摘要，确认秘书只提示会话继续边界和查看预览，不提供发送、resume、批准或重试动作。
9. 不要在本轮尝试真实发送；E4 没有真实发送入口。

本轮未做真实 Tauri 窗口截图验收，因此不能把上面手动检查当成阶段 G 验收已完成。

## 6. 下一步

下一步建议单独拆 E5：`codex-local` controlled send / resume minimal loop。

E5 必须先明确：

- 是否允许真实 `codex exec resume`。
- 是否允许写 `/Users/yoyi/.codex`。
- prompt 来源、preview、用户确认、attempt、dispatch / continuation record、readback 和失败记录如何写。
- 如何回滚和区分 readback unavailable。

未取得用户明确授权前，不应开始真实执行实现或验收。

## 7. 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
