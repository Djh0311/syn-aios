# Evidence: Root Treatment / R4-A49 Authorization Workflow Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a49-authorization-workflow-fixture-helper-extraction-v1.md`

Planning baseline commit：`5325579935f6300100a8abf2d2041a9bb1c50118`

Implementation commit：`ae89584e057eb809cfe6ade176704021a115a73d`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`61eb2665724d838f05d3ded0ab17fbb9d99c45ba`

## 1. 本轮目标

R4-A49 继续 R4-6 offline interaction test splitting，将 `offlineAuthorizationWorkflowFixtures.ts` 中的 authorization / proposal / project director / workflow run check 纯测试数据 cluster 抽离到独立 helper。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowClusterFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a49-authorization-workflow-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a49-authorization-workflow-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a49-authorization-workflow-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowFixtures.ts`

未修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineScenarioEnvironmentFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A49。

## 3. 具体实现

新增 `offlineAuthorizationWorkflowClusterFixtures.ts`，导出：

- `authorizationWorkflowClusterFixtures(input)`

该 helper 返回：

- `blockedWorkflowRunCheck`
- `planAuthorizationStore`
- `projectConsultationProposalStore`
- `planAuthorizationStorePendingGlobal`
- `projectConsultationProposalStoreConfirmed`
- `projectConsultationProposalStoreActive`
- `projectDirectorTaskPlan`
- `runnableWorkflowRunCheck`

`offlineAuthorizationWorkflowFixtures.ts` 保留原组合入口：

- `authorizationWorkflowFixtures(projectRoot, sessionThreadId, workflowProjectId, workflowId)`

该入口只转调新 helper，并继续返回原字段名。调用侧不变。

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
offlineAuthorizationWorkflowFixtures.ts: 370 -> 15
offlineAuthorizationWorkflowClusterFixtures.ts: 新增 374
offline-permission-dialog.test.tsx: 仍为 3,404，未修改
```

补充扫描：

- `authorizationWorkflowFixtures` 调用侧仍只有 `offlineScenarioEnvironmentFixtures.ts`。
- `authorizationWorkflowClusterFixtures` 只被 `offlineAuthorizationWorkflowFixtures.ts` 调用。
- 针对新增/修改 helper 的 `codex exec`、`Command::new`、`child_process`、`/Users/yoyi/.codex`、`.codex`、`tauri`、`invoke(`、`fetch(`、`XMLHttpRequest`、`fs.`、`writeFile`、`readFile` 关键词扫描无命中。
- `offline-permission-dialog.test.tsx` diff 为空。

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

- 工作树范围符合 A49：tracked diff 只有 `offlineAuthorizationWorkflowFixtures.ts` 和外部 `backlog.md`；新增 owned 文件为 `offlineAuthorizationWorkflowClusterFixtures.ts` 与 A49 任务包。`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已排除。
- `offlineAuthorizationWorkflowClusterFixtures.ts` 只导出 `authorizationWorkflowClusterFixtures(input)`，返回原 authorization / proposal / project director / run check 纯测试数据 cluster。
- `offlineAuthorizationWorkflowFixtures.ts` 保留原 `authorizationWorkflowFixtures(...)` 组合入口，只转调新 helper；调用侧语义保持。
- `offline-permission-dialog.test.tsx` 未修改，仍 3,404 行；render、button click、pending action、payload、cancel/confirm、assert 行为断言未被触碰。
- 字段 / 文案 / status / revision / guard / scope / audit / display text 均表现为原 fixture 搬移，未变成产品执行能力。
- 执行入口扫描未发现 I/O、Tauri/network/child_process、真实 Codex 执行或 `.codex` access；任务包明确排除 R4 完成、真实执行、真实 Tauri 验收、R3 Level B 和 backlog 解冻。

Residual risk：

- 复核线未重新跑主管线的 typecheck / offline tests / shape gate；复核只做静态只读 diff、文本和边界检查。主管线验证已单独记录在本 evidence 第 4 节。

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
