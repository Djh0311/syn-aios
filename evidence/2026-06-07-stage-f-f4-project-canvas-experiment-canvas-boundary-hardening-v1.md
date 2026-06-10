# Evidence: Stage F / F4 Project Canvas / Experiment Canvas Boundary Hardening v1

日期：2026-06-07

## 结论

F4 已完成。接受为项目工作流画布和一级实验 / 模板画布的边界硬化完成：

- 一级 `CanvasView` 明确显示 experiment / template / canvas library 语境。
- 一级实验运行文案明确不是项目 workflow，不会自动写正式项目事实、正式记忆或项目 workflow。
- 项目页工作流画布明确显示 project / workflow / authorization / control core 语境。
- 项目画布显示事实源来自 `workflow state 派生读模型`，运行和变更仍必须经过方案授权、控制核心、权限和审计。
- 离线测试覆盖实验画布边界常量、项目画布边界常量、项目画布静态文案和误导文案黑名单。

不接受为 F5 阶段 F 验收完成、项目画布和实验画布合一、MCP canvas run 正式项目工作流、真实 worker / Codex 执行、runtime log、diagnostics、阶段 G 真实 Tauri 验收或中间版本最终验收完成。

## 改动范围

产品代码：

- `prototypes/productized-desktop-shell/src/lib/canvasSurfaceBoundaries.ts`
  - 新增 `CanvasSurfaceBoundary` 纯前端只读边界声明。
  - 新增 `experimentCanvasBoundary` 和 `projectWorkflowCanvasBoundary`。
  - 新增误导文案黑名单常量用于测试。
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
  - 一级画布侧栏和 Tauri fallback 显示实验画布边界面板。
  - 将新增节点和节点编辑中的 `项目主管` 收敛为 `实验主管`。
  - 将运行按钮和 notice 收敛为 `实验画布运行（非项目 workflow）`。
  - 运行区补充“不会自动写正式项目事实、正式记忆或项目 workflow”的说明。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 项目工作流画布侧栏新增项目 / 实验画布边界卡片。
  - 显示 `项目工作流画布`、`workflow state 派生读模型`、`方案授权 / 控制核心 / 权限 / 审计`、`React Flow 仅负责渲染`、`实验画布不会写入本项目事实`。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增实验画布边界面板和项目画布边界 badge 的紧凑样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 增加 F4 边界常量断言。
  - 增加项目画布 UI 文案断言。
  - 扩展误导文案黑名单断言。

文档收尾：

- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
  - 修正 F4 推荐任务包状态残留，从“已写，状态为待执行”收敛为“已完成”。

Rust / 后端：

- 未修改 Rust。
- 未扩展 `WorkbenchSnapshot`。
- 未新增 Tauri command。
- 未新增 store、sidecar、数据库迁移。
- 未改 workflow state JSON 顶层结构或状态枚举。

## 一级画布边界

一级画布现在显示：

- `实验 / 模板画布`
- `experiment / template / canvas library`
- `不会写项目事实`
- `不会写正式记忆`
- `不会写项目 workflow state`
- `不是项目 workflow 事实源`
- `MCP canvas run 非默认项目工作流`

运行区显示实验运行只属于实验画布语境；正式项目运行请回项目工作流。本轮没有移除既有实验运行能力，但文案不再暗示它是正式项目 workflow。

## 项目画布边界

项目工作流画布侧栏现在显示：

- `项目工作流画布`
- `workflow state 派生读模型`
- `方案授权 / 控制核心 / 权限 / 审计`
- `React Flow 仅负责渲染`
- `实验画布不会写入本项目事实`

F4 没有改变 F1-F3 的事实源：项目画布仍来自 `ProjectWorkflowCanvasReadModel`，React Flow 仍只负责渲染。

## 验证

通过：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，输出 `offline interaction tests passed: 11`。

```text
npm run build
```

结果：通过。Vite 构建成功；保留既有 chunk size warning。

## Smoke / 截图

尝试启动 Vite dev server：

```text
npm run dev -- --host 127.0.0.1
```

结果：失败。

```text
Error: listen EPERM: operation not permitted 127.0.0.1:5173
```

随后按权限规则申请非沙箱本地端口启动，申请被安全审查拒绝：本轮 UI smoke 是可选验收，用户未单独授权 unsandboxed localhost dev server。

因此：

- 浏览器 / 真实窗口 smoke 未完成。
- 真实 Tauri / 截图验收未完成。
- 本轮不接受为真实窗口验收完成；该项仍交给 G3。

## 扫描

误导完成态扫描：

```text
rg -n '实验运行已写项目状态|MCP canvas run 已成为正式 workflow|实验画布已并入项目|独立 CanvasDefinition 是项目事实源|已写正式记忆|已派发项目 worker|worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收' prototypes/productized-desktop-shell/src --glob '!**/canvasSurfaceBoundaries.ts'
```

结果：无命中。`canvasSurfaceBoundaries.ts` 内仅保存黑名单常量，用于测试，不是可见误导完成态。

事实源边界扫描：

```text
rg -n 'CanvasDefinition.*项目事实|CanvasRunState.*workflow|MCP canvas run.*项目|experiment.*workflow state|template.*workflow state|CanvasView.*事实源' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

结果：命中均为 F4 允许边界文案或测试断言：

- `MCP canvas run 非默认项目工作流`
- 黑名单 `独立 CanvasDefinition 是项目事实源`

敏感 / 真实执行关键词扫描：

```text
rg -n 'codex exec|codex exec resume|/Users/yoyi/.codex|auth\.json|\.env|token|secret|keychain|OAuth|provider credential' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

结果：有既有命中，集中在既有权限边界文案、历史执行能力、后端 guard、测试 fixture 和敏感词过滤。本轮未新增真实执行路径，未读写 `/Users/yoyi/.codex`。

收尾口径复查：

```text
rg -n 'F4[^。\n]*待执行|stage-f-f4[^。\n]*待执行|Project Canvas / Experiment Canvas Boundary Hardening[^。\n]*待执行|项目画布 / 实验画布边界硬化[^。\n]*待执行' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md
```

结果：无命中。

## 本轮未做

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取完整 transcript / rollout。
- 未读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未启动 MCP canvas run。
- 未把独立 `CanvasView` / `CanvasDefinition` / `CanvasRunState` 写入项目事实。
- 未新增一级入口、右侧顶级入口或项目页 tab。
- 未新增真实 worker dispatch、自动重试、runtime log 或 diagnostics。
