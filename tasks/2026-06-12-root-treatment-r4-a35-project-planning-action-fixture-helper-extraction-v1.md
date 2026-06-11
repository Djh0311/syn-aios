# Root Treatment / R4-A35 Project Planning Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`b85be57d1f214d02b90a72f24d93104cd8c8f65e`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A35 任务包；R4-A35 继续对应官方计划 R4-6：离线测试拆分。R4-A35 只接受为项目咨询 / 全局边界复核 / 项目主管计划 / 总指导回收相关纯 action 与 request fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A34 已完成并 checkpoint，`offline-permission-dialog.test.tsx` 当前约 4,595 行。
- `runShellScenario` 中 C2/C3/C4 规划链路和总指导回收段仍有内联 expected payload / request fixture。
- 这些对象是纯测试数据；按钮查找、点击、`PermissionDialog` render、UI 文案检查和 deep equality 断言仍应留在主测试。

核心判断：

```text
R4-A35 只抽项目规划链路相关纯 expected payload / request fixture；主测试继续保留交互流程和行为断言。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectPlanningActionFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载 C2/C3/C4 / director review 的 expected payload、request fixture 或 action fixture。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A35：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A35 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 把 UI 文案断言、按钮点击流程、render 流程、权限弹层行为或 forbidden 文案检查搬进 helper。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineProjectPlanningActionFixtures.ts`。
2. 抽离：
   - `projectConsultationProposalDecisionPayloadFixture`
   - `globalBoundaryReviewPayloadFixture`
   - `projectDirectorTaskPlanRequestFixture`
   - `directorReviewActionFixture`
   - 相关 summary 常量。
3. 更新 `offline-permission-dialog.test.tsx` 使用 helper。
4. 保持所有断言仍在主测试中可见。

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

R4-A35 可接受条件：

- 主测试行数下降，且抽离内容只包含纯 fixture / request / expected payload。
- `offline-permission-dialog.test.tsx` 行为断言仍保留。
- 验证通过。
- 复核线无 P0/P1；如有 P2，必须分类处理或写入 deferred。
- checkpoint 前入口文档同步到 R4-A35 完成、下一步 R4-A36。

R4-A35 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
