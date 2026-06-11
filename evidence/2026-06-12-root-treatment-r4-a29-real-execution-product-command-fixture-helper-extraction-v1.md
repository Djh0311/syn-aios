# Evidence: Root Treatment / R4-A29 Real Execution Product Command Fixture Helper Extraction v1

日期：2026-06-12

状态：复核通过，待 implementation / checkpoint hash 回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a29-real-execution-product-command-fixture-helper-extraction-v1.md`

Planning baseline commit：`e050e89`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：待回填

## 1. 本轮目标

R4-A29 继续 R4-6 offline interaction test splitting，只抽离 Real Execution Product Command / Project Workflow Automation 相关离线 read model fixture cluster。

本轮不改变产品行为、不修改任何 Agent / Running / Project / Secretary / Right rail render、button、class、UI 文案检查或 forbidden text 断言，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineRealExecutionProductCommandFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a29-real-execution-product-command-fixture-helper-extraction-v1.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- `runRealExecutionProductCommandBoundaryScenario` 的 render、button、class、UI 文案检查、秘书建议检查、右栏检查、forbidden text 断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。
- `docs/own-agent-and-company-vision-v1.md`，该文件仍为外部未跟踪文件。

## 3. 行数变化

`wc -l` 记录：

```text
5025 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 142 prototypes/productized-desktop-shell/tests/helpers/offlineRealExecutionProductCommandFixtures.ts
```

主测试从 R4-A28 后的 `5129` 行下降到 `5025` 行，减少 `104` 行。

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

## 6. 复核状态

复核线已只读检查并返回：

- 复核线程：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无
- 可接受为 R4-A29 implementation 完成，不阻断 implementation commit。

复核确认：

- diff 只包含 R4-A29 允许范围。
- 新 helper 只包含 Real Execution Product Command / Project Workflow Automation fixture builder。
- 主测试未改 render、button、class、UI 文案检查、秘书建议检查、右栏检查或 forbidden text 断言语义。
- 未发现产品代码、CSS、Rust、Tauri、DB、sidecar 或 workflow schema 修改。
- 未发现真实执行、`codex exec` / `codex exec resume` 或 `/Users/yoyi/.codex` 读写。

## 7. 不能声明

R4-A29 即使复核通过，也仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
