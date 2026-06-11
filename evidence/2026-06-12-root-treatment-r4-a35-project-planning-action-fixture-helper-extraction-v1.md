# Evidence: Root Treatment / R4-A35 Project Planning Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a35-project-planning-action-fixture-helper-extraction-v1.md`

Planning baseline commit：`b85be57d1f214d02b90a72f24d93104cd8c8f65e`

Implementation commit：`0d4271a67d2a3fd88d0b5655cee0b367691b5009`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`a78b20455929e4f0366a38f11ea9fa7a2226fe15`

## 1. 本轮目标

R4-A35 继续 R4-6 offline interaction test splitting，只抽项目咨询 / 全局边界复核 / 项目主管计划 / 总指导回收相关纯 action / request / expected payload fixture。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectPlanningActionFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a35-project-planning-action-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a35-project-planning-action-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a35-project-planning-action-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A35。

## 3. 具体实现

新增 helper：

- `projectConsultationProposalDecisionPayloadFixture`
- `globalBoundaryReviewPayloadFixture`
- `projectDirectorTaskPlanRequestFixture`
- `directorReviewActionFixture`
- `projectConsultationProposalDecisionSummary`
- `globalBoundaryReviewSummary`

主测试调整：

- C2 项目咨询方案确认的 expected payload 改为 helper。
- C3 全局边界复核的 expected payload 改为 helper。
- C4 项目主管拆任务 request 改为 helper。
- 总指导回收 expected action 改为 helper。

主测试仍保留：

- 按钮查找、点击、`PermissionDialog` render、UI 文案检查、forbidden 文案检查、deep equality 行为断言和测试入口列表。

## 4. 验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`：

```text
npm run typecheck
```

结果：通过，`tsc --noEmit`。

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

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 首次误跑：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：失败，原因是当前目录不对，`MODULE_NOT_FOUND`；未反映产品或测试失败。

在 `/Users/yoyi/workspace/product-line` 重跑：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，`Status: pass`，`Errors: 0`，`Warnings: 1`。

既有 warning：

```text
tauri_command_total_increased: current 97 / baseline 96
```

在 `/Users/yoyi/workspace/product-line`：

```text
git diff --check
```

结果：通过，无输出。

过程偏差：

- checkpoint stale scan 曾有一条 `rg` 命令把含 Markdown 反引号的 pattern 放进 shell 双引号，触发 shell 命令替换并输出 `zsh:1: command not found: STATUS:`。
- 该偏差没有执行真实 Codex、没有发送 prompt、没有读写 `/Users/yoyi/.codex`，也没有修改产品代码；随后已改用单引号重新扫描。

## 5. 行数

- `offline-permission-dialog.test.tsx`：4,595 -> 4,555。
- `offlineProjectPlanningActionFixtures.ts`：新增 83 行。

## 6. 复核

原复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911` 长时间 active 且无 agent 输出；按“旧线程卡死例外”启用新只读复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

复核结论：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核要点：

- A35-owned files 在任务包范围内；`backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 被排除。
- helper 只有 type-only imports、summary 常量和四个对象返回函数。
- 未发现 filesystem、Tauri invoke、network、process/env、child process、Codex execution 或 `.codex` access。
- 主测试保留行为断言和测试入口列表。

## 7. 边界确认

本轮没有：

- 修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 8. 不能声明

R4-A35 不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
