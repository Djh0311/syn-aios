# Root Treatment / R4-A45 Workflow Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`8552993912465d9f06ff9fd659b2c64ba95aca06`

Implementation commit：`067098a33ee993b526b2f23558e4454f7eaa22a4`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`9dd9d2b318744f90305e51f5d6ab7663ada78374`

本文是 Root Treatment / Stage R 的 R4-A45 任务包；R4-A45 继续对应官方计划 R4-6：离线交互测试按域拆分。R4-A45 只接受为 workflow action 相关纯 fixture builder 从 `offlineTaskFieldTestUtils.ts` 抽离到独立 helper；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A44 已完成并回填 hash；当前入口文档指向 R4-A45。
- 用户已确认从 R4-A17 起采用“中等粒度 fixture cluster 拆分”，但不能为凑行数移动行为断言。
- `offlineTaskFieldTestUtils.ts` 当前混放 task field helper 和 workflow action helper，职责边界不够清晰。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变化，不纳入本轮 commit。

核心判断：

```text
R4-A45 只抽非 task field 的 workflow action fixture builder；不改测试行为、不改产品代码。
```

## 1. Execution Mode

Execution Mode：Supervisor-led workflow action fixture extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineWorkflowActionFixtures.ts`

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许抽离的纯 fixture builder：

- `buildBootstrapProjectWorkflowAction(...)`
- `buildUserReviewedInstructionPreviewAction(...)`
- `buildPermissionDecisionAction(...)`
- `buildBindNodeSessionAction(...)`
- `buildUnbindNodeSessionAction(...)`
- `buildAdvanceWorkItemStateAction(...)`
- `expectedInitializeWorkflowStateAction(...)`

应留在 `offlineTaskFieldTestUtils.ts` 的内容：

- task draft form helper。
- copy / generate task file helper。
- update / correct task field helper。
- task field correction fixture。
- `listValue(...)`。

External changes not owned by R4-A45：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A45 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 抽走 render、button 查找、click 流程、pending action、action payload、cancel/confirm、`assert` / `assertDeepEqual` 行为断言。
- 用被测 builder 生成 expected payload，导致断言变成自比自。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineWorkflowActionFixtures.ts`，只导出 workflow action 相关纯对象构造函数。
2. 从 `offlineTaskFieldTestUtils.ts` 移除上述 workflow action builder，仅保留 task field / task package helper。
3. 更新 `offline-permission-dialog.test.tsx` imports：workflow action helper 从新文件引入，task field helper 继续从原文件引入。
4. 主测试保留所有行为断言、render、button click、pending action 和 payload 检查。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offline-permission-dialog.test.tsx` 行数变化。
- `offlineTaskFieldTestUtils.ts` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 Workflow action fixture helper extraction 完成。
- `offlineTaskFieldTestUtils.ts` 职责收敛为 task field / task package helper。
- 主测试继续保留行为断言。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
