# Stage H / H2.8 Real Execution Permission Dialog, Audit Summary, And Readiness Decision Surface Evidence v1

日期：2026-06-07

状态：已完成；非真实执行修补任务。

## 1. 结论

H2.8 接受为 H2 真实 `codex-local resume` 执行前 readiness 决策面、权限弹层预览、审计摘要、runtime log preview、readback 边界和 duplicate guard 加固完成。

H2.8 不接受为：

- H2 Phase B 已授权。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- fixture 已创建。
- H2 通用真实 resume 产品化完成。
- H3-B 已授权或已执行。
- 阶段 H 完成。

## 2. 实现摘要

本轮改动集中在桌面壳前端读模型和只读展示：

- 新增 H2.8 决策面类型：`H2RealResumeExecutionDecisionSurface`、permission preview、audit/runtime/readback preview、decision checks。
- 新增 `deriveH2RealResumeExecutionDecisionSurface(...)`，从 `SessionContinuationPreview` 和 `SessionContinuationStoreV1` 派生 final approval 前阻断状态。
- 智能体页新增 `H2.8 final approval 决策面` 面板，显示 operation、target session、permission envelope、allowed write roots、prompt envelope、`.codex` scope、readback plan、runtime log、audit、rollback、duplicate guard、diagnostics、用户 final approval 和全局主管 final review。
- 秘书读模型新增 H2.8 风险信号和查看建议，只解释风险和查看入口，不生成批准、发送、resume 或重试 action proposal。
- 离线交互测试新增 H2.8 默认 blocked、缺 target session、readback result_count=null、secret deny paths、duplicate queued/running attempt 阻断和秘书只读边界覆盖。

## 3. 关键文件

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/h2RealResumeAuthorization.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

同步入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`

## 4. 验证

已通过：

```text
npm run test:offline-interaction
offline interaction tests passed: 12

npm run typecheck
tsc --noEmit

npm run build
vite build completed
```

`npm run build` 保留既有 Vite chunk size warning。

未跑 Rust 测试：本轮未修改 Rust / Tauri command / store / runner / guard 后端路径，只修改前端 TS 类型、派生读模型、UI、秘书读模型、CSS 和离线测试。

`git status` / `git diff` 未能使用：当前 `/Users/yoyi/workspace/product-line` 不是 git repository。

## 5. 扫描

误导文案扫描：

```text
rg -n -F 'Codex 已收到任务' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

结果：仅命中 `src/lib/canvasSurfaceBoundaries.ts` 的黑名单 / forbidden phrase 常量，不是产品完成态文案。

以下扫描无命中：

```text
rg -n -F '真实 resume 已执行' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'prompt 已发送' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'readback 0 条' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'planned adapter 已接入' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

真实执行 / 敏感路径扫描：

```text
rg -n -F 'Command::new("codex")' prototypes/productized-desktop-shell/src-tauri/src
rg -n -F 'codex exec resume' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
rg -n -F '.codex' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

分类：

- 既有真实 runner / MCP runner / H5 真实执行路径仍存在，不是 H2.8 新增。
- H2.8 新增命中均为“不执行 / 不授权 / 预览 / 后续批准后才允许”的边界文案。
- `.codex` 命中包含既有 guard、测试 fixture、Codex sqlite 只读索引、permission dialog 边界文案和 H2.8 deny path / scope preview。

## 6. 边界确认

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 创建 H2 / H3 fixture。
- 修改 Rust runner / Tauri command / workflow state JSON 顶层结构。
- 调用外部 provider / 模型。
- 读取 auth/token/secret/`.env`/keychain/OAuth/provider credential 或完整 transcript。

过程偏差：

- 收尾时为判断是否可做浏览器 / 截图验证，读取了 Browser 插件技能说明：`/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.602.40724/skills/control-in-app-browser/SKILL.md`。
- 该读取不是用户 Codex 会话数据，不是 auth/token/secret，不写 `.codex`，也未执行 Codex 命令；但本轮不能无条件声称“完全没有读取 `/Users/yoyi/.codex` 路径下任何文件”。

## 7. 当前 Readiness

H2.8 之后：

```text
h2_phase_b_readiness = blocked_waiting_target_session
phase_b_authorization_request = not_ready
h2_8_decision_surface = completed_non_execution
```

H2.8 让 final approval 材料更可审核，但不改变 H2 Phase B 的授权状态。

## 8. 下一步

可选下一步：

- 继续补 H2 Phase B 阻断项：existing target session、fixture、permission envelope、allowed write roots、prompt ref/hash、`.codex` 最小范围、readback、runtime log、audit、rollback。
- 请求 H2 Phase B final approval / real fixture run，但必须单独任务包、执行点授权和证据回收。
- 继续 H3-B final approval 材料复核；真实新会话执行仍需执行点明确授权。
