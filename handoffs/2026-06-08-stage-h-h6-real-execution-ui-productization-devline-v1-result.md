# Handoff: Stage H / H6 Real Execution UI Productization Devline v1

日期：2026-06-08

## 回交结论

H6 开发线已完成最小 UI 产品化改动并通过前端验证。

## 改了哪些文件

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-devline-v1.md`

## 为什么改

- 智能体页需要安全展示真实执行状态，而不是默认读取完整 transcript。
- 项目工作流页需要把 H5 dispatch、任务包、任务记忆包、权限、readback、worker report 和 process fact 放到同一处摘要。
- 权限弹层需要更明确说明真实 Codex、Codex home 副作用和失败/readback 边界。

## 验证命令

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

```text
typecheck: passed
offline interaction tests passed: 12
build: passed
```

## 剩余风险

- 未执行真实 Tauri / GUI 截图验收，H6 acceptance 不能声明完成。
- 未执行真实 Codex，也未触发 H5/H2 新执行点。
- 真实 Tauri 截图仍需验证线或主管线授权后执行。

## 是否触碰 UI

是。触碰已有页面局部 UI 和状态摘要：

- 智能体页。
- 项目工作流侧栏。
- 权限弹层。

没有新增一级入口、tab、右侧入口或自由执行控制台。

## 是否需要主管复核

需要主管复核。

建议复核点：

- 自动 transcript 读取改为手动读取是否满足 H6 / G3-B 智能体页截图安全路径。
- H6 合并摘要是否覆盖任务包要求的真实执行状态产品化。
- 是否安排验证线执行真实 Tauri 截图清单。
