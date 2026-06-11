# Evidence: Root Treatment / R4-A47 Derived Project Workflow Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a47-derived-project-workflow-fixture-helper-extraction-v1.md`

Planning baseline commit：`03fa13e80c69c807911f553541702b062e11d7e2`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填。

## 1. 本轮目标

R4-A47 继续 R4-6 offline interaction test splitting，将 `offlineDerivedWorkflowFixtures.ts` 中的 pending workflow result summary 和 derived project workflow 纯测试数据 cluster 抽离到独立 helper。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineDerivedProjectWorkflowFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a47-derived-project-workflow-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a47-derived-project-workflow-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a47-derived-project-workflow-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineDerivedWorkflowFixtures.ts`

未修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A47。

## 3. 具体实现

新增 `offlineDerivedProjectWorkflowFixtures.ts`，导出：

- `derivedProjectWorkflowFixture(input)`

该 helper 返回：

- `pendingWorkflowResultSummary`
- `derivedWorkflow`

抽离内容包括 derived workflow 的 nodes、task packages、ledger entries、subagent reports、review results、exceptions、interface boundaries、state machine、acceptance scenarios 和 warnings。

`offlineDerivedWorkflowFixtures.ts` 保留组装入口：

- 调用 `derivedProjectWorkflowFixture(...)`。
- 调用 `projectBlackboardFixture(projectRoot)`。
- 拼出 `workflowStateWithDerivedWorkflow`。
- 返回 `pendingWorkflowResultSummary` 和 `workflowStateWithDerivedWorkflow`。

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
offlineDerivedWorkflowFixtures.ts: 468 -> 37
offlineDerivedProjectWorkflowFixtures.ts: 新增 460
offline-permission-dialog.test.tsx: 仍为 3,404，未修改
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

- 工作树范围符合 A47：tracked diff 只有 `offlineDerivedWorkflowFixtures.ts` 和外部 `backlog.md`；新增 owned 文件为 `offlineDerivedProjectWorkflowFixtures.ts` 与 A47 任务包。`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已排除。
- `offlineDerivedProjectWorkflowFixtures.ts` 只导出 `derivedProjectWorkflowFixture(input)`，返回 `pendingWorkflowResultSummary` 与 `derivedWorkflow`，内容为纯 fixture/object data。
- `offlineDerivedWorkflowFixtures.ts` 保留 project blackboard helper import，并新增 derived project workflow helper import；仍是 `workflowStateWithDerivedWorkflow` 组装入口。
- `nodes`、`task_packages`、`ledger_entries`、`subagent_reports`、`review_results`、`exceptions`、`interface_boundaries`、`state_machine`、`acceptance_scenarios`、`warnings` 仍保持测试数据/边界语义，未变成产品执行能力。
- `offline-permission-dialog.test.tsx` 未修改，render、button click、pending action、payload、cancel/confirm、assert 行为断言未被触碰。
- 执行入口扫描未发现 I/O、Tauri/network/child_process、真实 Codex 执行或 `.codex` access；任务包明确排除 R4 完成、真实执行、真实 Tauri 验收、R3 Level B 和 backlog 解冻。

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
