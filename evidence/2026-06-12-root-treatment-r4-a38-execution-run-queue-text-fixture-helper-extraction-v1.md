# Evidence: Root Treatment / R4-A38 Execution Run Queue Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a38-execution-run-queue-text-fixture-helper-extraction-v1.md`

Planning baseline commit：`5515d7f102bde8fc995b629b5bd8485d7ac4ca99`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A38 继续 R4-6 offline interaction test splitting，只抽 `runRealExecutionProductCommandBoundaryScenario` 与 `runStageJRunQueueScenario` 中真实执行产品命令 / 自动编排 / 运行队列 / 操作控制相关 expected / forbidden text。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineExecutionRunQueueTextFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a38-execution-run-queue-text-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a38-execution-run-queue-text-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a38-execution-run-queue-text-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A38。

## 3. 具体实现

新增 helper：

- `executionRunQueueTextFixtures`

抽离内容：

- Agent 对话区、统一执行链路、自动编排、Codex 控制、失败 / 阻断 / 读回 expected text。
- RunningWorkflows 统一执行、自动编排、失败 / 阻断 / 读回 expected text。
- ProjectDetail 统一执行 / 自动编排 expected text。
- Secretary action proposal forbidden text。
- RightDetailPanel 统一执行 / 失败 expected text。
- combined markup forbidden text。
- Stage J/K5 运行队列 Running / Right rail expected text 和 forbidden text。

主测试仍保留：

- read model schema/status/kind 检查。
- JSX render、`visibleText` / `renderToStaticMarkup`。
- Secretary risk/suggestion/action proposal kind 检查。
- markup class/order 检查。
- `assert` / `assertDeepEqual` 行为断言、forbidden text 断言循环和测试入口列表。

## 4. 验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

```text
npm run typecheck
```

结果：通过，`tsc --noEmit`。

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

结果：通过，`Status: pass`，`Errors: 0`，`Warnings: 1`。

既有 warning：

```text
tauri_command_total_increased: current 97 / baseline 96
```

在 `/Users/yoyi/workspace/product-line`：

```text
git diff --check
```

结果：通过，无输出。

## 5. 行数

- `offline-permission-dialog.test.tsx`：4,045 -> 3,967。
- `offlineExecutionRunQueueTextFixtures.ts`：新增 165 行。

## 6. 复核

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

复核结论：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核要点：

- helper 无 import，只导出一个 `as const` text fixture object；`codex exec resume` / command-like strings 只作为 forbidden text fixture 存在，不是可执行代码。
- 主测试仍保留 read model 检查、JSX render、`visibleText` / `renderToStaticMarkup`、Secretary risk/suggestion/action-proposal kind 检查、markup class/order 检查和 forbidden markup assertions。
- A38 owned changes 只限 tests/helper 与 task/evidence/handoff；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 被排除为外部变更。
- 未发现产品代码、UI/CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径变化。

## 7. 边界确认

本轮没有：

- 修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 8. 不能声明

R4-A38 不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
