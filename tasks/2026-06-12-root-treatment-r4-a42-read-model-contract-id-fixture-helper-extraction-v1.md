# Root Treatment / R4-A42 Read Model Contract Id Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`35e7c90095fc4e6ac222220c74f5c32d1c63d612`

Implementation commit：`345a2daf997dcd1e272f02e4fdd533b35bb0f4ad`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`fc58d1fce4b075dfc7332ea2c93118546e430fa7`

本文是 Root Treatment / Stage R 的 R4-A42 任务包；R4-A42 继续对应官方计划 R4-6：离线测试拆分。R4-A42 只接受为 offline interaction 主测试中静态 read model contract id / kind / status / warning fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A41 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,503 行。
- 主测试中仍残留多组静态 id / kind / status / warning 清单。
- 这些清单是测试 fixture；read model derivation、JSX render、button 查找、click、payload 和行为断言仍应留在主测试。

核心判断：

```text
R4-A42 只抽静态 contract id/kind/status/warning fixture；不抽动态数据构造、行为断言、render、click 或产品逻辑。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline contract fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineReadModelContractFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载静态 read model contract id / kind / status / warning list。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A42：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A42 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 抽走 read model derivation、data fixture、dynamic project root 拼接、JSX render、button 查找、click 流程、action payload、`assert` / `assertDeepEqual` 行为断言。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineReadModelContractFixtures.ts`。
2. 抽离：
   - real execution / run queue 的 forbidden action kinds、confirmation kinds、readback null statuses。
   - Project Canvas 的 node / edge / mutation kind 静态清单。
   - Adapter / session operation / provider / adapter SDK 的 capability ids、operation ids、warning ids、status ids。
   - Session continuation / H2 readiness 的 operation ids、guard statuses、decision check ids、readiness item ids。
   - Right rail non-secretary panel ids 和 transcript cleaning expected id lists。
3. 主测试继续保留：
   - `derive*` read model 检查。
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

- R4-6 read model contract id / kind / status / warning fixture helper extraction 完成。
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
