# Root Treatment / R4-A44 Shell Remaining Expected Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`2488ac146a615ee5399f19807a10eeed3a7d5af7`

Implementation commit：`e26aac0af5494fb91bf65b99dc795e4d93a7fe97`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`cd16d89cf21d91b928b53598a2ffdfe848e2b2cd`

本文是 Root Treatment / Stage R 的 R4-A44 任务包；R4-A44 继续对应官方计划 R4-6：离线测试拆分。R4-A44 只接受为 offline interaction 主测试中 Shell 剩余 expected action object fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A43 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,414 行。
- 当前剩余大块主要是 `runShellScenario` 行为链路；其中 render、button click、pending action、payload、cancel/confirm 和 UI 文案断言都必须留在主测试。
- 剩余安全 fixture cluster 自然偏小：`correct-dispatch-fields` 和 `initialize-workflow-state` 两个 inline expected action object 仍在主测试中。

核心判断：

```text
R4-A44 只抽 Shell 场景剩余 expected action object；不为凑行数移动行为断言。
```

## 1. Execution Mode

Execution Mode：Supervisor-led shell expected action fixture extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 在现有 task field helper 中新增独立 expected action builder：
  - `expectedCorrectDispatchFieldsAction(...)`
  - `expectedInitializeWorkflowStateAction(...)`
- 主测试把两个 inline expected object 替换成 helper 调用。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A44：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A44 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 抽走 `runShellScenario`、render、button 查找、click 流程、pending action、action payload、cancel/confirm、`assert` / `assertDeepEqual` 行为断言。
- 用被测 builder 生成 expected payload，导致断言变成自比自。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 在 `offlineTaskFieldTestUtils.ts` 增加两个独立 expected action builder。
2. 主测试继续保留：
   - `buildCorrectDispatchFieldsAction(...)` 被测 builder 调用。
   - workflow state 初始化按钮点击和 captured action 检查。
   - `PermissionDialog` render。
   - cancel / confirm 断言。
   - 所有 UI 文案和行为断言。
3. 主测试只把 inline expected object 替换为 helper 调用。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offline-permission-dialog.test.tsx` 行数变化。
- `offlineTaskFieldTestUtils.ts` 行数变化。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 Shell remaining expected action fixture helper extraction 完成。
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
