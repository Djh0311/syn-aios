# Evidence: Root Treatment / R4-A42 Read Model Contract Id Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a42-read-model-contract-id-fixture-helper-extraction-v1.md`

Planning baseline commit：`35e7c90095fc4e6ac222220c74f5c32d1c63d612`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A42 继续 R4-6 offline interaction test splitting，只抽 offline interaction 主测试中的静态 read model contract id / kind / status / warning fixture。

覆盖范围：

- Real execution / run queue 的 forbidden suggestion / action proposal kinds、readback null statuses、failure statuses / classifications、confirmation kinds。
- Project Canvas 的 node / edge / mutation kind 静态清单。
- Adapter / session operation / provider / adapter SDK diagnostics 的 capability ids、operation ids、warning ids、status ids、policy string。
- Session continuation / H2 readiness 的 operation ids、guard statuses、reason fragments、blocking check ids、readiness item ids。
- Right rail non-secretary panel ids 和 transcript cleaning expected ids。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineReadModelContractFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a42-read-model-contract-id-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a42-read-model-contract-id-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a42-read-model-contract-id-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A42。

## 3. 具体实现

新增 helper：

- `readModelContractFixtures`

抽离内容：

- `executionForbiddenSuggestionKinds`
- `runQueueForbiddenActionProposalKinds`
- `runQueueReadbackNullStatuses`
- `runQueueFailureNullStatuses`
- `runQueueFailureClassifications`
- `runQueueConfirmationKinds`
- `projectCanvasNodeTypes`
- `projectCanvasEdgeTypes`
- `projectCanvasPreviewMutationKinds`
- `adapterHiddenUnimplementedIds`
- `adapterImplementedActionKinds`
- `adapterExpectedCapabilityKinds`
- `adapterConfirmationBoundaryFragments`
- `sessionOperationIds`
- `blockedSessionOperationStatuses`
- `sessionOperationRequiredWarnings`
- `providerSummaryRequiredWarnings`
- `plannedProviderBoundary`
- `adapterContractMissingItems`
- `adapterDiagnosticRedactionPolicy`
- `adapterDataLocationSecretPolicy`
- `continuationOperationIds`
- `plannedContinuationGuardStatuses`
- `plannedContinuationReasonFragments`
- `h2DecisionBlockingCheckIds`
- `h2MissingReadinessItemIds`
- `rightRailNonSecretaryPanelIds`
- `transcriptCleaningExpectedIds`

主测试仍保留：

- `derive*` read model 检查。
- dynamic project root、data fixture、workflow/session snapshot fixture。
- `ProjectWorkflowCanvas` / `AgentView` / `RunningWorkflowsView` / `RightDetailPanel` render。
- button 查找、click、pending action、payload 检查。
- `assert` / `assertDeepEqual` 行为断言。

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
offline-permission-dialog.test.tsx: 3,503 -> 3,497
offlineReadModelContractFixtures.ts: 144
```

说明：第一次 `npm run typecheck` 曾因 helper 数组类型过宽失败；随后只把 `adapterHiddenUnimplementedIds` 和 `rightRailNonSecretaryPanelIds` 改成精确 tuple 后重跑通过。

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核确认：

- `offlineReadModelContractFixtures.ts` 无 import，只包含静态 id/kind/status/warning/list fixture，并通过 `readModelContractFixtures` 聚合导出；未见 I/O、产品 import、Tauri/network/child_process、真实 Codex 或 `.codex` access。
- Real execution / run queue 的 read model derivation、render、secretary/action proposal 断言仍在主测试。
- Project Canvas 的动态 spread、read model derivation、node/edge/mutation 断言仍在主测试。
- Adapter/session/provider/diagnostics derivation 与状态/警告断言仍在主测试。
- Session continuation/H2 readiness 的 derivation、guard、dynamic `project.project_root`、render 断言仍在主测试。
- Right rail panel loop 与 transcript cleaning `assertDeepEqual` 仍在主测试。
- A42 owned scope 只有测试 helper、主测试和任务包；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 是外部变更并已排除。
- 任务包明确写明不可接受 R4 完成、全部拆分完成、真实执行、真实 Tauri 验收、R3 Level B 或 backlog 解冻等越界声明。

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
