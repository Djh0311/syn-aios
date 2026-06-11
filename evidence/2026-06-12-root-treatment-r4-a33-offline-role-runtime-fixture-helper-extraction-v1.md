# Evidence: Root Treatment / R4-A33 Offline Role Runtime Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a33-offline-role-runtime-fixture-helper-extraction-v1.md`

Planning baseline commit：`f86f53e`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A33 继续 R4-6 offline interaction test splitting，只抽离线角色编排 action/form fixture 与 E6 runtime session summary fixture。

本轮不改变产品行为、不修改 `runOfflineRoleOrchestrationScenario` / `runRuntimeSessionAttentionScenario` 的 render、按钮查找、点击、表单提交、UI 文案检查、forbidden 文案检查、deep equality 行为断言或测试入口列表，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `tasks/2026-06-12-root-treatment-r4-a33-offline-role-runtime-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/helpers/offlineRoleOrchestrationFixtures.ts`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineRuntimeDiagnosticFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- `runOfflineRoleOrchestrationScenario` / `runRuntimeSessionAttentionScenario` 的 render、按钮查找、点击、表单提交、UI 文案检查、forbidden 文案检查、deep equality 行为断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。
- `docs/own-agent-and-company-vision-v1.md`，该文件仍为外部未跟踪文件。

## 3. 行数变化

`wc -l` 记录：

```text
4648 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
  36 prototypes/productized-desktop-shell/tests/helpers/offlineRoleOrchestrationFixtures.ts
 239 prototypes/productized-desktop-shell/tests/helpers/offlineRuntimeDiagnosticFixtures.ts
```

主测试从 R4-A32 后的 `4,681` 行下降到 `4,648` 行，减少 `33` 行。

说明：本切片低于 250 行软目标，但它只抽安全的离线角色编排 action/form fixture 与 E6 summary fixture。继续扩大将跨入 C2/C3/C4/C5 行为输出、render、点击、UI 文案断言或 deep equality 行为验收。

## 4. 验证

已运行并通过：

```text
npm run typecheck
```

结果：

```text
tsc --noEmit
```

已运行并通过：

```text
npm run test:offline-interaction
```

结果：

```text
offline interaction tests passed: 14
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page selectors test passed
```

已运行并通过：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：

```text
Status: pass
Errors: 0
Warnings: 1
```

继承既有 warning：

```text
tauri_command_total_increased: current 97 / baseline 96
```

已运行并通过：

```text
git diff --check
```

结果：无输出。

过程提示：

- 有一条 `rg` 扫描命令因为 pattern 中包含 literal `\n` 被 ripgrep 拒绝；该命令未修改文件、未触发 shell command substitution、未启动任何外部运行时。
- 随后已用固定字符串扫描确认 `SessionRunStatusSummary` 不再留在主测试、离线角色缺字段块只保留在新 helper 中。

未运行：

- Rust 测试：本切片只改 TS 测试 helper 和任务文档，不改 Rust / Tauri。
- `npm run build`：本切片只做离线测试 fixture 抽离，已由 typecheck 与 offline interaction 覆盖。

## 5. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- 修改 `backlog.md`。

敏感 / 真实执行关键词扫描分类：

- `offlineRoleOrchestrationFixtures.ts` 中没有 `codex exec`、`codex exec resume`、Tauri invoke、文件读写或进程启动；`/Users/yoyi/.codex` 只出现在 expected boundary fixture 字符串中。
- `offlineRuntimeDiagnosticFixtures.ts` 新增 `runtimeSessionSummaryFixture` 只根据 `sessionId` 和 `attention` 生成 `SessionRunStatusSummary`。
- 主测试命中均为历史禁止文案、边界断言或 fixture preview，不是新增真实执行路径。

## 6. 复核状态

复核线已只读检查并返回：

- 复核线程：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无
- 可接受为 R4-A33 implementation 完成。

复核确认：

- 工作区可见变更符合 A33 允许范围；`backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 仍是外部变更，未见被纳入 A33 结论。
- 任务包边界与实现一致，明确只抽“离线角色编排 action/form fixture + runtime session summary fixture”，并禁止迁移 render、点击、表单提交、UI 文案、forbidden 文案和 deep equality 断言。
- 新 helper 只包含纯 fixture builder，没有文件读取、进程启动、Tauri、网络或真实 Codex 路径。
- runtime helper 新增的 `runtimeSessionSummaryFixture` 没有运行时副作用。
- 主测试中的 E6 runtime 和离线角色编排行为断言仍在原地。
- 未见产品代码、CSS、Rust、Tauri command、DB、sidecar、workflow state schema、真实执行路径相关改动。

## 7. 不能声明

R4-A33 即使复核通过，也仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
