# Evidence: Root Treatment / R4-A45 Workflow Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a45-workflow-action-fixture-helper-extraction-v1.md`

Planning baseline commit：`8552993912465d9f06ff9fd659b2c64ba95aca06`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填。

## 1. 本轮目标

R4-A45 继续 R4-6 offline interaction test splitting，将 `offlineTaskFieldTestUtils.ts` 中非 task field 的 workflow action fixture builder 抽离到独立 helper。

覆盖范围：

- `buildBootstrapProjectWorkflowAction(...)`
- `buildUserReviewedInstructionPreviewAction(...)`
- `buildPermissionDecisionAction(...)`
- `buildBindNodeSessionAction(...)`
- `buildUnbindNodeSessionAction(...)`
- `buildAdvanceWorkItemStateAction(...)`
- `expectedInitializeWorkflowStateAction(...)`

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineWorkflowActionFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a45-workflow-action-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a45-workflow-action-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a45-workflow-action-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A45。

## 3. 具体实现

新增 `offlineWorkflowActionFixtures.ts`，只包含 workflow action 相关纯对象构造函数。

`offlineTaskFieldTestUtils.ts` 移除 workflow action builder，保留：

- task draft form helper。
- copy / generate task file helper。
- update / correct task field helper。
- task field correction fixture。
- `listValue(...)`。

`offline-permission-dialog.test.tsx` 只调整 import 来源：

- workflow action helper 从 `offlineWorkflowActionFixtures.ts` 引入。
- task field / task package helper 继续从 `offlineTaskFieldTestUtils.ts` 引入。

主测试仍保留：

- `runShellScenario` 行为链路。
- React render / static markup / visible text。
- button 查找、click、pending action、payload、cancel/confirm 检查。
- 所有 `assert` / `assertDeepEqual` 行为断言。

## 4. 验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过。

```text
offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page selectors test passed
```

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过；0 errors，保留既有 warning：

```text
tauri_command_total_increased 97/96
```

```text
git diff --check
```

结果：通过，无输出。

行数：

```text
offline-permission-dialog.test.tsx: 3,402 -> 3,404
offlineTaskFieldTestUtils.ts: 332 -> 230
offlineWorkflowActionFixtures.ts: 新增 102
```

说明：本切片主测试因新增独立 import block 增加 2 行；接受目标是 workflow action fixture domain split，不包装为主测试显著瘦身。

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: 无
P1: 无
P2: 无
```

复核确认：

- owned changes 为 `offlineWorkflowActionFixtures.ts`、`offlineTaskFieldTestUtils.ts`、`offline-permission-dialog.test.tsx` 和 A45 任务包；`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已按外部变更排除。
- `offlineWorkflowActionFixtures.ts` 只导出预期 7 个 workflow action fixture/object builder；除 type import 外没有产品执行导入，也没有 I/O、Tauri/network/child_process、真实 Codex 或 `.codex` access。
- `offlineTaskFieldTestUtils.ts` 已移除 workflow action builder，保留 task draft/copy/generate/update/correct task field helper、`taskFieldCorrectionFixtures` 和 `listValue`。
- `offline-permission-dialog.test.tsx` diff 只是 import 来源调整；`runShellScenario`、`PermissionDialog`、button 查找/click、pending action、payload、cancel/confirm、`assertDeepEqual` 行为断言仍在主测试中。
- A45 任务包明确排除 R4 完成、真实执行、真实 Tauri 验收、R3 Level B 和 backlog 解冻等越界声明。

## 6. 边界确认

本轮没有修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot，没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮不接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
