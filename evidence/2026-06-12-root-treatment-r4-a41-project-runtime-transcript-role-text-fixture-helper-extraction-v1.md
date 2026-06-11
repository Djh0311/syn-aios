# Evidence: Root Treatment / R4-A41 Project Runtime Transcript Role Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a41-project-runtime-transcript-role-text-fixture-helper-extraction-v1.md`

Planning baseline commit：`e49866d04b666f5bb75af4dff99f72e32ee90405`

Implementation commit：`645e92430c826863d6b713a75fdd7c512921a82f`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`7773a2fb3ec2fd5274e2c64811a154f03302e2b0`

## 1. 本轮目标

R4-A41 继续 R4-6 offline interaction test splitting，只抽 Project Canvas / Runtime Log / Transcript Session / Offline Role 相关场景中的 expected / forbidden / class / id list fixture。

覆盖范围：

- Project Canvas boundary expected texts、detail layer/kind labels、user summary labels、state example ids。
- Runtime Log management expected texts、sensitive forbidden texts。
- Session Center expected texts/classes、Transcript expected texts。
- Offline Role missing field labels、dispatch dialog expected texts、role panel expected texts。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectRuntimeTranscriptRoleTextFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a41-project-runtime-transcript-role-text-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a41-project-runtime-transcript-role-text-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a41-project-runtime-transcript-role-text-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A41。

## 3. 具体实现

新增 helper：

- `projectRuntimeTranscriptRoleTextFixtures`

抽离内容：

- `projectCanvasBoundaryExpectedTexts`
- `projectCanvasDetailLayers`
- `projectCanvasDetailKinds`
- `projectCanvasUserSummaryLabels`
- `projectCanvasStateExampleIds`
- `runtimeLogManagementExpectedTexts`
- `runtimeLogSensitiveForbiddenTexts`
- `sessionCenterExpectedTexts`
- `sessionCenterExpectedClasses`
- `transcriptExpectedTexts`
- `offlineRoleMissingFieldLabels`
- `offlineRoleDispatchDialogExpectedTexts`
- `offlineRolePanelExpectedTexts`

主测试仍保留：

- `deriveProjectWorkflowCanvasReadModel`、`projectCanvasStateExamples`、runtime log store、session/transcript fixture、offline role parser/action builder。
- dynamic project root 拼接、spread 组合、typed operation id。
- `ProjectWorkflowCanvas` / `RightDetailPanel` / `AgentView` / `ChatTranscript` / `PermissionDialog` / `OfflineRoleOrchestrationPanel` render。
- button 查找、click、pending action、payload 检查。
- `assert` / `assertDeepEqual` 行为断言。

## 4. 验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过。

```text
offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page selectors test passed
```

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过；0 errors，保留既有 warning：

```text
tauri_command_total_increased 97/96
```

```text
git diff --check
```

结果：通过，无输出。

行数：

```text
offline-permission-dialog.test.tsx: 3,576 -> 3,503
offlineProjectRuntimeTranscriptRoleTextFixtures.ts: 132
```

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核确认：

- `offlineProjectRuntimeTranscriptRoleTextFixtures.ts` 无 import，只导出 `projectRuntimeTranscriptRoleTextFixtures`，内容是 text/class/id list；未见 I/O、产品 import、Tauri/network/child_process、真实 Codex 或 `.codex` access。
- Project Canvas 的 dynamic spread、typed mutation kind、read model derivation 和行为断言仍在主测试。
- Runtime log fixture、render、sensitive forbidden 断言仍在主测试。
- Transcript/session fixture、filter `assertDeepEqual`、render/class 断言仍在主测试。
- Offline Role parse/build action、dynamic `project.project_root`、button click、payload/assertDeepEqual 仍在主测试。
- A41 owned scope 只有测试 helper、主测试和任务包；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 是外部变更并已排除。
- 任务包明确写明不可接受 R4 完成、全部拆分完成、真实执行、真实 Tauri 验收、R3 Level B 或 backlog 解冻等越界声明。

## 6. 边界确认

本轮没有修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot，没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮不接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。
