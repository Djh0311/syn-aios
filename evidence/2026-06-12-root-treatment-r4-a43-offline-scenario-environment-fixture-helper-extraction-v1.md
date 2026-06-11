# Evidence: Root Treatment / R4-A43 Offline Scenario Environment Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a43-offline-scenario-environment-fixture-helper-extraction-v1.md`

Planning baseline commit：`d2135d17018a3ecf8c1d8926401552a6f1bb89ef`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A43 继续 R4-6 offline interaction test splitting，只抽 offline interaction 主测试中的离线场景环境装配和少量纯 expected data builder。

覆盖范围：

- base workbench fixture、project workflow state fixture、authorization workflow fixture 的集中装配。
- plan authorization / project consultation proposal summary fixture 装配。
- derived workflow / C6 result summary fixture 装配。
- workflow state ready/prepared/completed/generated variants fixture 装配。
- not-ready dispatch readiness fixture。
- Project Canvas prepared workflow 的纯对象构造 fixture。
- update-task-fields 的独立 expected action builder。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineScenarioEnvironmentFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a43-offline-scenario-environment-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a43-offline-scenario-environment-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a43-offline-scenario-environment-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A43。

## 3. 具体实现

新增 helper：

- `offlineScenarioEnvironmentFixtures()`
- `preparedProjectWorkflowFixture(...)`

在既有 helper 中新增：

- `expectedUpdateTaskFieldsAction(...)`

主测试仍保留：

- `runShellScenario` 行为链路。
- React render / static markup / visible text。
- button 查找、click、pending action 和 payload 检查。
- 所有 `assert` / `assertDeepEqual` 行为断言。

`expectedUpdateTaskFieldsAction(...)` 是独立 expected payload builder，没有调用被测 `buildUpdateTaskFieldsAction(...)`。

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
offline-permission-dialog.test.tsx: 3,497 -> 3,414
offlineScenarioEnvironmentFixtures.ts: 105
offlineTaskFieldTestUtils.ts: 276 -> 302
```

过程说明：主管线曾在 `/Users/yoyi/workspace/product-line` 根目录误跑 `npm run typecheck` 和 `npm run test:offline-interaction`，因根目录没有 `package.json` 失败；随后在正确的前端包目录重跑并通过。该失败不涉及产品代码、不涉及真实执行、不涉及 `.codex`。

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

- 工作树范围符合 A43：owned changes 仅涉及测试 helper、主测试和任务包；`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已按外部变更排除。
- `offlineScenarioEnvironmentFixtures.ts` 只装配离线环境 fixture、summary fixture、workflow variants 和 not-ready readiness；关键字检查未发现 I/O、Tauri、network、child_process、真实 Codex 或 `.codex` access。
- `offlineTaskFieldTestUtils.ts` 的 `expectedUpdateTaskFieldsAction` 是独立 expected payload builder；未调用被测 `buildUpdateTaskFieldsAction`。
- `offline-permission-dialog.test.tsx` 仅替换为环境 helper；`runShellScenario`、render、button、click、pending action、payload、`assert` / `assertDeepEqual` 行为断言仍在主测试中。
- 主测试仍先调用被测 `buildUpdateTaskFieldsAction`，再与 helper 生成的 expected object 做 `assertDeepEqual`。
- 任务包只声明 A43 fixture extraction 边界，未把 A43 冒充为 R4 完成、真实 Tauri 验收、真实 Codex 执行或 backlog 解冻。

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
