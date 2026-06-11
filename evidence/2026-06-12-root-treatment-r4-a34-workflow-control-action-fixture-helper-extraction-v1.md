# Evidence: Root Treatment / R4-A34 Workflow Control Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a34-workflow-control-action-fixture-helper-extraction-v1.md`

Planning baseline commit：`d509004`

Implementation commit：`994a207d38d7b1213240924f068e235694a64dff`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：`02f847d6660bc075e570e04259ac55f9c150d248`

## 1. 本轮目标

R4-A34 继续 R4-6 offline interaction test splitting，只抽 workflow control / node session / permission / work item state 相关 expected action fixture 和任务草稿 `FormData` fixture。

本轮不改变产品行为、不修改 `runShellScenario` 的按钮查找、点击、表单提交、`PermissionDialog` render、UI 文案检查、取消确认、forbidden 文案检查、deep equality 行为断言或测试入口列表，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `tasks/2026-06-12-root-treatment-r4-a34-workflow-control-action-fixture-helper-extraction-v1.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- `runShellScenario` 的按钮查找、点击、表单提交、`PermissionDialog` render、UI 文案检查、取消确认、forbidden 文案检查、deep equality 行为断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。
- `docs/own-agent-and-company-vision-v1.md`，该文件仍为外部未跟踪文件。

## 3. 行数变化

`wc -l` 记录：

```text
4595 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 276 prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts
```

主测试从 R4-A33 后的 `4,648` 行下降到 `4,595` 行，减少 `53` 行。

说明：本切片低于 250 行软目标，但只抽安全的 expected payload / form fixture。继续扩大将跨入 C5 worker report / process fact 等由组件运行时生成的行为输出断言。

## 4. 验证

初次运行：

```text
npm run typecheck
```

结果：失败。

原因：

```text
tests/offline-permission-dialog.test.tsx(3765,7): error TS2345:
Argument of type 'WorkflowUserReviewedInstruction | null | undefined' is not assignable to parameter of type 'WorkflowUserReviewedInstruction'.
```

修复：在主测试中加入 `userReviewedInstruction` 存在性断言后再传入 helper。

重新运行并通过：

```text
npm run typecheck
```

结果：

```text
tsc --noEmit
```

重新运行并通过：

```text
npm run test:offline-interaction
```

结果：

```text
offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page selectors test passed
```

已运行并通过：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：

```text
Status: pass
Errors: 0
Warnings: 1
```

继承既有 warning：

```text
tauri_command_total_increased: current 97 / baseline 96
```

已运行并通过：

```text
git diff --check
```

结果：无输出。

未运行：

- Rust 测试：本切片只改 TS 测试 helper 和任务文档，不改 Rust / Tauri。
- `npm run build`：本切片只做离线测试 fixture 抽离，已由 typecheck 与 offline interaction 覆盖。

## 5. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- 修改 `backlog.md`。

敏感 / 真实执行关键词扫描分类：

- `offlineTaskFieldTestUtils.ts` 中新增 `/Users/yoyi/.codex`、`codex exec resume` 等字符串均为 expected boundary fixture 文案，不是执行路径。
- helper 没有文件读取、进程启动、Tauri invoke、网络调用或真实 Codex 调用。
- 主测试命中均为历史禁止文案、边界断言或 fixture preview，不是新增真实执行路径。

## 6. 复核状态

复核线已只读检查并返回：

- 复核线程：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无
- 可接受为 R4-A34 implementation 完成。

复核确认：

- 工作区可见 diff 符合 A34 允许范围；`backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 仍是外部变更，未纳入本轮结论。
- 任务包边界写明本轮只抽 workflow control / node session / permission / work item state 的 expected action 与 form fixture，且禁止迁移 render、点击、`PermissionDialog`、UI 文案、forbidden 文案和 deep equality 断言。
- helper 新增内容是纯 builder，只返回静态对象或 `FormData` stub class，没有文件读取、进程启动、Tauri、网络、真实 Codex 或 `/Users/yoyi/.codex` 接触。
- 主测试里的行为断言仍留在原位，没有被藏进 helper。
- 未见产品代码、CSS、Rust、Tauri command、DB、sidecar、workflow state schema、真实执行路径改动。

## 7. 不能声明

R4-A34 即使复核通过，也仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
