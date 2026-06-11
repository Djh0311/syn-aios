# Root Treatment / R4-A48 Candidate Memory Lifecycle Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`d4c4bb9d05d1096303738b98dfbc41aee63c66a1`

Implementation commit：`a89b1142b572a665dbd7964d15cb6af7e3a2d949`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`f73e14493acd18784a7be014c0caee5c3714c5bf`

本文是 Root Treatment / Stage R 的 R4-A48 任务包；R4-A48 继续对应官方计划 R4-6：离线交互测试按域拆分。R4-A48 只接受为 `offlineCandidateGovernanceFixtures.ts` 中 candidate memory lifecycle 纯测试数据 cluster 抽离到独立 helper；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A47 已完成并回填 hash；当前入口文档指向 R4-A48。
- 用户已确认从 R4-A17 起采用“中等粒度 fixture cluster 拆分”，但不能为凑行数移动行为断言。
- `offlineCandidateGovernanceFixtures.ts` 当前 521 行，后半段 formal memory、adopted formal memory、memory lint、task memory packet preview 是纯数据 cluster。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变化，不纳入本轮 commit。

核心判断：

```text
R4-A48 只抽 candidate memory lifecycle fixture cluster；不移动主测试行为断言，不改产品代码。
```

## 1. Execution Mode

Execution Mode：Supervisor-led candidate memory lifecycle fixture extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineCandidateMemoryLifecycleFixtures.ts`

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineCandidateGovernanceFixtures.ts`
- 本任务包、对应 evidence / handoff。

允许抽离：

- `formalMemoryStore`
- `adoptedFormalMemoryStore`
- `memoryLintStore`
- `taskMemoryPacketPreview`

External changes not owned by R4-A48：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A48 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 修改 `offline-permission-dialog.test.tsx` 的 render、button 查找、click 流程、pending action、action payload、cancel/confirm、`assert` / `assertDeepEqual` 行为断言。
- 改变 memory lifecycle fixture 字段值、revision、warning 文案、lint status、task memory packet included / excluded / review material 语义。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineCandidateMemoryLifecycleFixtures.ts`，导出 `candidateMemoryLifecycleFixtures(input)`。
2. helper 返回 formal memory store、adopted formal memory store、memory lint store、task memory packet preview，保持字段和文案逐字不变。
3. `offlineCandidateGovernanceFixtures.ts` 改为 import 新 helper，并继续返回原来同名字段。
4. 不修改主测试和产品源码。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offlineCandidateGovernanceFixtures.ts` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 candidate memory lifecycle fixture helper extraction 完成。
- `offlineCandidateGovernanceFixtures.ts` 职责进一步收敛。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
