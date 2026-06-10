# Stage H / H2.2 Real Resume Authorization Readiness Read Model And Readonly UI v1 Evidence

日期：2026-06-07

结论：H2.2 接受为 H2 真实 resume 授权准备读模型和智能体页只读 UI 完成；不接受为 H2 通用真实 resume 产品化完成。

## 1. 范围

本轮完成：

- 新增前端 H2 real resume authorization readiness 读模型。
- 新增 H2 readiness TS 类型。
- 智能体页新增只读“H2 real resume 授权准备”面板。
- 补齐 H2.2 局部样式。
- 离线测试覆盖 readiness 状态、缺失项、非执行文案和按钮黑名单。
- 同步 H2.2 任务包、evidence、handoff 和权威入口。

## 2. 边界

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth/token/.env/secret/keychain/OAuth/provider credential/full transcript。
- 调用 H2.0 Tauri command。
- 创建 fixture 项目。
- 启动 Tauri、GUI 或截图。
- 新增执行按钮、发送按钮、resume 按钮、确认按钮、授权按钮或重试按钮。
- 把 H2 标记为完成。
- 进入 H3/H4/H5。

## 3. 新增 / 更新文件

新增：

- `tasks/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- `evidence/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- `handoffs/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1-result.md`
- `prototypes/productized-desktop-shell/src/lib/h2RealResumeAuthorization.ts`

更新：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

## 4. UI 显示边界结果

H2.2 只在智能体页既有会话 / continuation 边界区域新增只读面板：

- 显示 readiness status、target session、project root、recommended fixture。
- 显示授权矩阵条目：操作类型、测试项目、推荐 fixture、project root、target cwd、target session、prompt summary、prompt hash/ref、allowed write roots、`.codex` 最小范围、sandbox、timeout、readback plan、runtime/audit/evidence、rollback、用户确认、全局主管确认。
- 显示警告：`h2_readiness_is_not_execution_authorization`、`no_prompt_sent_in_h2_readiness`、`no_codex_home_read_write_in_h2_readiness`、`readback_unavailable_is_not_zero_results`。
- 不新增一级入口、右侧入口、项目页 tab、画布入口或管理入口。
- 不渲染执行类按钮。

## 5. 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，`offline interaction tests passed: 12`。
- `npm run build` 通过，仅保留既有 Vite chunk size warning。

验证过程发现并修复：

- 初次验证发现 H2 readiness 面板被接入到 `AgentSessionCenter`，但派生变量定义在外层 `AgentView`，导致 TypeScript 和离线测试失败。
- 已把 readiness 派生移到实际渲染组件可见作用域内，并重新通过验证。

## 6. 未完成 / 保留项

- 真实窗口 / 截图验收未完成。本轮没有启动 Tauri、GUI 或截图，因此不能声称 H2.2 UI 已完成真实窗口验收。
- H2 真实 resume 仍未执行。
- H2 授权矩阵仍缺 target session、prompt hash/ref、`.codex` 最小范围、rollback、用户确认和全局主管确认等执行前授权项。

## 7. 主管复核结论

H2.2 可以接受为执行前授权准备的产品内只读可视化完成。

H2 仍处于待授权状态。下一步不能直接进入 H3；必须先由用户和全局主管确认 H2 授权矩阵后，才能进入真实 runner / 真实 resume 执行切片。
