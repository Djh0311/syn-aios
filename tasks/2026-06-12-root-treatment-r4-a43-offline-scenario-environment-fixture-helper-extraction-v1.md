# Root Treatment / R4-A43 Offline Scenario Environment Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`d2135d17018a3ecf8c1d8926401552a6f1bb89ef`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A43 任务包；R4-A43 继续对应官方计划 R4-6：离线测试拆分。R4-A43 只接受为 offline interaction 主测试中离线场景环境装配和少量剩余 expected data builder 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A42 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,497 行。
- 主测试剩余最大块是 `runShellScenario` 行为链路；其中大量 render、button click、pending action、payload 和 UI 文案断言不应搬走。
- 顶部仍有离线 fixture 环境装配、summary fixture 装配、workflow variant fixture 装配，以及少量内联 expected object builder。

核心判断：

```text
R4-A43 只抽测试环境装配和纯 expected data builder；不抽场景断言、不抽 render/click、不改产品逻辑。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline scenario environment fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineScenarioEnvironmentFixtures.ts`
- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增离线测试环境装配 helper，把 `workbenchBaseFixtures`、workflow state、authorization workflow、derived workflow、C6 summary、workflow state variants 和 not-ready dispatch readiness 的装配从主测试移出。
- 在现有 task field helper 中补一个独立 expected update-task-fields action builder；不得用被测 builder 自己生成 expected。
- 如有项目画布 prepared workflow 的纯对象构造，可抽到测试 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A43：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A43 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 抽走 `runShellScenario`、render、button 查找、click 流程、pending action、action payload、`assert` / `assertDeepEqual` 行为断言。
- 用被测 builder 生成 expected payload，导致断言变成自比自。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineScenarioEnvironmentFixtures.ts`。
2. 抽离主测试顶部离线环境装配：
   - base workbench fixture。
   - project workflow state fixture。
   - authorization workflow fixture。
   - plan authorization / project consultation summary fixture 装配。
   - derived workflow / C6 result summary fixture 装配。
   - workflow state variant fixture。
   - not-ready dispatch readiness fixture。
3. 在 `offlineTaskFieldTestUtils.ts` 增加独立 expected update task fields action builder。
4. 主测试继续保留：
   - `runShellScenario` 行为链路。
   - React render / static markup / visible text。
   - button 查找、click、pending action 和 payload 检查。
   - 所有 `assert` / `assertDeepEqual` 行为断言。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offline-permission-dialog.test.tsx` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 offline scenario environment fixture helper extraction 完成。
- 主测试继续瘦身，行为断言留在主测试。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
