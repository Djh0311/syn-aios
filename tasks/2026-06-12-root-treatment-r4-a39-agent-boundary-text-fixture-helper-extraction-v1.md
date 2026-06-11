# Root Treatment / R4-A39 Agent Boundary Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`95312f2a85a5dc97f6770860dad7d04b360cd318`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A39 任务包；R4-A39 继续对应官方计划 R4-6：离线测试拆分。R4-A39 只接受为 Agent 相关边界场景的 expected / forbidden / button / proposal text fixture 抽离完成，覆盖会话操作边界、Provider / 模型 / 凭据边界、Adapter SDK / CLI diagnostics 边界、Session Continuation preview、E5 Level A、H2 real resume readiness、E6 runtime attention；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A38 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,967 行。
- 主测试中 Agent boundary 场景仍保留多组 inline expected / forbidden / button / proposal text arrays。
- 这些 arrays 是测试 fixture，不是产品行为；adapter/session/provider derivation、guard/readiness 检查、secretary risk/suggestion kind 判断、render 和断言仍应留在主测试。

核心判断：

```text
R4-A39 只抽 Agent boundary 文案 fixture；不抽 read model / guard / readiness 断言，不抽 JSX，不抽秘书 kind 判断，不抽产品逻辑。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline text fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAgentBoundaryTextFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载 Agent boundary 场景的 expected / forbidden / button / proposal text arrays。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A39：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A39 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 把 JSX render、button 查找、click 流程、read model / guard / readiness / secretary kind 检查、`assert` / `assertDeepEqual` 行为断言搬进 helper。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineAgentBoundaryTextFixtures.ts`。
2. 抽离：
   - 会话操作边界 expected text、forbidden button/text/proposal text。
   - Provider availability expected / forbidden / button / proposal text。
   - Adapter SDK / CLI diagnostics expected / forbidden / button text。
   - Session Continuation preview expected / forbidden / button / proposal text。
   - E5 Level A expected / forbidden / button / proposal text。
   - H2 real resume readiness expected / forbidden / button / proposal text。
   - E6 runtime attention expected / forbidden / button / proposal text。
3. 更新 `offline-permission-dialog.test.tsx` 使用 helper。
4. 保持主测试中所有 derivation、schema/status/kind/guard/readiness/secretary checks、render、markup 和行为断言可见。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须记录：

- `offline-permission-dialog.test.tsx` 前后行数。
- 新 helper 行数。
- shape gate 输出。
- 复核线结论。

## 6. Acceptance

R4-A39 可接受条件：

- 主测试行数下降，且抽离内容只包含只读 text fixture。
- Agent boundary 场景的 read model / guard / readiness / secretary checks 和行为断言仍保留在主测试中。
- 验证通过。
- 复核线无 P0/P1；如有 P2，必须分类处理或写入 deferred。
- checkpoint 前入口文档同步到 R4-A39 完成、下一步 R4-A40。

R4-A39 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
