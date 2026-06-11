# Root Treatment / R4-A49 Authorization Workflow Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`5325579935f6300100a8abf2d2041a9bb1c50118`

Implementation commit：待回填。

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：待回填。

本文是 Root Treatment / Stage R 的 R4-A49 任务包；R4-A49 继续对应官方计划 R4-6：离线交互测试按域拆分。R4-A49 只接受为 authorization / proposal / project director / run check 纯测试数据 cluster 从 `offlineAuthorizationWorkflowFixtures.ts` 抽离到独立 helper；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A48 已完成并回填 hash；当前入口文档指向 R4-A49。
- 用户已确认 R4 后续提速采用“中等粒度 fixture cluster 拆分”，但不能为凑行数移动行为断言。
- `offlineAuthorizationWorkflowFixtures.ts` 当前 370 行，内部包含 workflow run check、plan authorization store、project consultation proposal store、pending global review、active proposal 和 project director task plan 等纯测试数据。
- `offlineScenarioEnvironmentFixtures.ts` 只通过 `authorizationWorkflowFixtures(...)` 组合入口消费这些对象。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变化，不纳入本轮 commit。

核心判断：

```text
R4-A49 只抽 authorization workflow fixture cluster；不移动主测试行为断言，不改产品代码。
```

## 1. Execution Mode

Execution Mode：Supervisor-led authorization workflow fixture extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowClusterFixtures.ts`

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAuthorizationWorkflowFixtures.ts`
- 本任务包、对应 evidence / handoff。

允许抽离：

- `blockedWorkflowRunCheck`
- `blockedAutoDispatchGuardResult`
- `planAuthorizationStore`
- `projectConsultationProposalStore`
- `planAuthorizationStorePendingGlobal`
- `projectConsultationProposalStoreConfirmed`
- `projectConsultationProposalStoreActive`
- `projectDirectorTaskPlan`
- `runnableWorkflowRunCheck`

External changes not owned by R4-A49：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A49 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 修改 `offlineScenarioEnvironmentFixtures.ts` 的场景组合语义，除非只是必要 import 路径调整；优先保持调用侧不变。
- 修改 `offline-permission-dialog.test.tsx` 的 render、button 查找、click 流程、pending action、action payload、cancel/confirm、`assert` / `assertDeepEqual` 行为断言。
- 改变 authorization / proposal / project director / run check fixture 字段值、revision、status、warning 文案、guard result、scope、decision、audit event 或 display text 语义。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineAuthorizationWorkflowClusterFixtures.ts`，导出 `authorizationWorkflowClusterFixtures(input)`。
2. helper 返回 authorization workflow cluster 的原有同名对象，字段和文案逐字不变。
3. `offlineAuthorizationWorkflowFixtures.ts` 保留 `authorizationWorkflowFixtures(...)` 组合入口，改为调用新 helper 并返回原字段名。
4. 不修改主测试和产品源码。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offlineAuthorizationWorkflowFixtures.ts` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 authorization workflow fixture helper extraction 完成。
- `offlineAuthorizationWorkflowFixtures.ts` 职责进一步收敛。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
