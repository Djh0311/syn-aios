# Root Treatment / R4-A36 Right Rail Common Props Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

Planning baseline commit：`62f9b0e888e06d55e37610be46d68ecdd6e51d1a`

Implementation commit：`49543338fdb071242e75f0932d8f208bf361a43f`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`c72fedd83e73e996d21e781521603afbf6b3b7ef`

本文是 Root Treatment / Stage R 的 R4-A36 任务包；R4-A36 继续对应官方计划 R4-6：离线测试拆分。R4-A36 只接受为右侧详情面板测试 common props / summary title 纯 fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A35 已完成并 checkpoint，`offline-permission-dialog.test.tsx` 当前 4,555 行。
- 主测试中 E6 runtime attention、G1 runtime log、right rail secretary surface 三段仍重复构造 `RightDetailPanel` common props。
- 这些 common props 是纯测试 setup：snapshot、workflowState、notice、error、secretaryContext 和 no-op callbacks。

核心判断：

```text
R4-A36 只抽右侧详情面板 common props / summary title fixture；主测试继续保留 panel render、UI 文案检查和 forbidden 文案断言。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线使用 A35 卡死例外启用后的只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineRightRailFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载 `RightDetailPanel` common props fixture 和 right rail summary title fixture。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A36：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A36 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 把 `RightDetailPanel` render、UI 文案检查、forbidden 文案检查或测试入口列表搬进 helper。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineRightRailFixtures.ts`。
2. 抽离：
   - `rightDetailPanelCommonPropsFixture`
   - `rightRailPanelSummaryTitles`
3. 更新 `offline-permission-dialog.test.tsx` 使用 helper。
4. 保持所有 render 和断言仍在主测试中可见。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须记录：

- `offline-permission-dialog.test.tsx` 前后行数。
- shape gate 输出。
- 复核线结论。

## 6. Acceptance

R4-A36 可接受条件：

- 主测试行数下降，且抽离内容只包含纯 common props / summary title fixture。
- `offline-permission-dialog.test.tsx` 行为断言仍保留。
- 验证通过。
- 复核线无 P0/P1；如有 P2，必须分类处理或写入 deferred。
- checkpoint 前入口文档同步到 R4-A36 完成、下一步 R4-A37。

R4-A36 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
