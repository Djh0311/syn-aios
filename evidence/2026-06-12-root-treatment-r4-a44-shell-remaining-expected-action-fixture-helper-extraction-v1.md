# Evidence: Root Treatment / R4-A44 Shell Remaining Expected Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a44-shell-remaining-expected-action-fixture-helper-extraction-v1.md`

Planning baseline commit：`2488ac146a615ee5399f19807a10eeed3a7d5af7`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A44 继续 R4-6 offline interaction test splitting，只抽 Shell 场景中剩余的两个 inline expected action object。

覆盖范围：

- `correct-dispatch-fields` expected action object。
- `initialize-workflow-state` expected action object。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `tasks/2026-06-12-root-treatment-r4-a44-shell-remaining-expected-action-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a44-shell-remaining-expected-action-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a44-shell-remaining-expected-action-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A44。

## 3. 具体实现

在既有 helper 中新增：

- `expectedCorrectDispatchFieldsAction(...)`
- `expectedInitializeWorkflowStateAction(...)`

主测试仍保留：

- `runShellScenario` 行为链路。
- 被测 `buildCorrectDispatchFieldsAction(...)` 调用。
- workflow state 初始化按钮点击和 captured action 检查。
- React render / static markup / visible text。
- button 查找、click、pending action、payload、cancel/confirm 检查。
- 所有 `assert` / `assertDeepEqual` 行为断言。

两个新增 expected builder 都是独立对象构造，没有调用被测 builder。

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
offline-permission-dialog.test.tsx: 3,414 -> 3,402
offlineTaskFieldTestUtils.ts: 302 -> 332
```

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

- owned changes 只在两个测试文件和 A44 任务包；`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已按外部变更排除。
- `expectedCorrectDispatchFieldsAction(...)` 和 `expectedInitializeWorkflowStateAction(...)` 是纯 expected action object builder；未调用 `buildCorrectDispatchFieldsAction(...)` 或产品执行路径。
- 主测试仍调用被测 `buildCorrectDispatchFieldsAction(...)`，随后用 helper expected 做 `assertDeepEqual`。
- `runShellScenario` 仍保留 render、button 查找、click、pending action、cancel/confirm、`PermissionDialog` 和行为断言。
- 关键字检查未发现新增 I/O、Tauri/network/child_process、真实 Codex 执行或 `.codex` 访问；命中项均为边界 / 禁止文案或既有 UI 断言。
- 任务包只声明 A44 expected action fixture extraction，未将 A44 冒充为 R4 完成、真实 Tauri 验收、真实 Codex 执行或 backlog 解冻。

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
