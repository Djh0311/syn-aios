# Root Treatment / R4-A46 Project Blackboard Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`4c22573a941110a6b410fdf6dfc9e75d67384004`

Implementation commit：`4c10149278ca0a1d0324504ba54dc79f6cd1eb8c`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`1a0d2f4d2f1f1e523dc9a1884ed0cab5c988e749`

本文是 Root Treatment / Stage R 的 R4-A46 任务包；R4-A46 继续对应官方计划 R4-6：离线交互测试按域拆分。R4-A46 只接受为 `offlineDerivedWorkflowFixtures.ts` 中 project blackboard 纯测试数据 cluster 抽离到独立 helper；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A45 已完成并回填 hash；当前入口文档指向 R4-A46。
- `offline-permission-dialog.test.tsx` 剩余内容以行为断言和渲染检查为主，不适合为凑行数继续搬主测试断言。
- `offlineDerivedWorkflowFixtures.ts` 当前 629 行，其中 `project_blackboards` 是纯测试数据 cluster，不包含断言或执行逻辑。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变化，不纳入本轮 commit。

核心判断：

```text
R4-A46 只抽 project blackboard fixture cluster；不移动主测试行为断言。
```

## 1. Execution Mode

Execution Mode：Supervisor-led project blackboard fixture extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectBlackboardFixtures.ts`

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineDerivedWorkflowFixtures.ts`
- 本任务包、对应 evidence / handoff。

允许抽离：

- `project_blackboards` 中的单个 project blackboard fixture。
- project blackboard entries、promotion decisions、warnings、source refs 等纯对象数据。

External changes not owned by R4-A46：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A46 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 修改 `offline-permission-dialog.test.tsx` 的 render、button 查找、click 流程、pending action、action payload、cancel/confirm、`assert` / `assertDeepEqual` 行为断言。
- 改变 project blackboard fixture 的字段值、candidate/promotion/warning 语义或 source refs。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineProjectBlackboardFixtures.ts`，导出 `projectBlackboardFixture(projectRoot)`。
2. `offlineDerivedWorkflowFixtures.ts` 改为 import 新 helper，并用 `project_blackboards: [projectBlackboardFixture(projectRoot)]` 组装。
3. 不修改主测试和产品源码。
4. 保持字段和文案逐字不变，降低审查成本。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offlineDerivedWorkflowFixtures.ts` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 Project blackboard fixture helper extraction 完成。
- `offlineDerivedWorkflowFixtures.ts` 职责收敛，project blackboard 测试数据独立。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
