# Root Treatment / R4-A41 Project Runtime Transcript Role Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`e49866d04b666f5bb75af4dff99f72e32ee90405`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A41 任务包；R4-A41 继续对应官方计划 R4-6：离线测试拆分。R4-A41 只接受为 Project Canvas / Runtime Log / Transcript Session / Offline Role 相关 text / class / id list fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A40 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,576 行。
- 主测试中还剩 Project Canvas、Runtime Log、Transcript Session、Offline Role 场景的若干 inline text / class / id list。
- 这些 list 是测试 fixture；动态 project root 拼接、spread 组合、typed operation id、data derivation、render、button click、payload 和行为断言仍应留在主测试。

核心判断：

```text
R4-A41 只抽静态 list fixture；不抽动态拼接、read model derivation、action payload、button 行为、JSX render 或产品逻辑。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline text/list fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectRuntimeTranscriptRoleTextFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载 Project Canvas / Runtime Log / Transcript Session / Offline Role 的 expected / forbidden / class / id list。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A41：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A41 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 抽走动态 project root 拼接、dynamic spread、typed operation id、data fixture、summary derivation、JSX render、button 查找、click 流程、action payload、`assert` / `assertDeepEqual` 行为断言。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineProjectRuntimeTranscriptRoleTextFixtures.ts`。
2. 抽离：
   - Project Canvas boundary expected texts、detail layer/kind labels、user summary labels、state example ids。
   - Runtime Log management expected texts、sensitive forbidden texts。
   - Session Center expected texts/classes、Transcript expected texts。
   - Offline role missing field labels、dispatch dialog expected texts、role panel expected texts。
3. 主测试继续保留：
   - `deriveProjectWorkflowCanvasReadModel`、`projectCanvasStateExamples`、runtime log store、session/transcript fixture、offline role parser/action builder。
   - dynamic project root 拼接、spread 组合、typed operation id。
   - render、button click、pending action、payload 和行为断言。

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

- R4-6 Project Canvas / Runtime Log / Transcript Session / Offline Role text/list fixture helper extraction 完成。
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

## 7. Result

R4-A41 已完成并通过复核线 `STATUS: CLEAR`。

完成内容：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineProjectRuntimeTranscriptRoleTextFixtures.ts`，只承载 Project Canvas / Runtime Log / Transcript Session / Offline Role expected / forbidden / class / id list fixture。
- 更新 `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，将相关 inline lists 替换为 `projectRuntimeTranscriptRoleTextFixtures.*`。
- 主测试仍保留 dynamic project root 拼接、spread 构造、typed operation id、data fixture、summary derivation、JSX render、button 查找、click 流程、pending action、payload、`assert` / `assertDeepEqual` 行为断言。

验证通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，`offline interaction tests passed: 14`
- `node scripts/harness/workbench-shape-gate.js --mode check`，0 errors，保留既有 warning `tauri_command_total_increased 97/96`
- `git diff --check`

行数：

- `offline-permission-dialog.test.tsx`：3,576 -> 3,503。
- `offlineProjectRuntimeTranscriptRoleTextFixtures.ts`：新增 132 行。

边界确认：

- 未修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A41。
