# Evidence: Root Treatment / R4-A36 Right Rail Common Props Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a36-right-rail-common-props-fixture-helper-extraction-v1.md`

Planning baseline commit：`62f9b0e888e06d55e37610be46d68ecdd6e51d1a`

Implementation commit：`49543338fdb071242e75f0932d8f208bf361a43f`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`c72fedd83e73e996d21e781521603afbf6b3b7ef`

## 1. 本轮目标

R4-A36 继续 R4-6 offline interaction test splitting，只抽右侧详情面板 common props / summary title 纯 fixture。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineRightRailFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a36-right-rail-common-props-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a36-right-rail-common-props-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a36-right-rail-common-props-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A36。

## 3. 具体实现

新增 helper：

- `rightDetailPanelCommonPropsFixture`
- `rightRailPanelSummaryTitles`
- `RightDetailPanelCommonPropsFixture`

主测试调整：

- E6 runtime attention 场景的 `RightDetailPanel` common props 改为 helper。
- G1 runtime log 场景的 `RightDetailPanel` common props 改为 helper。
- right rail secretary surface 场景的 common props 和 summary title map 改为 helper。

主测试仍保留：

- `RightDetailPanel` render、UI 文案检查、forbidden 文案检查、行为断言和测试入口列表。

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

在 `/Users/yoyi/workspace/product-line`：

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

## 5. 行数

- `offline-permission-dialog.test.tsx`：4,555 -> 4,535。
- `offlineRightRailFixtures.ts`：新增 58 行。

## 6. 复核

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

复核结论：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核要点：

- A36-owned files 在任务包范围内；`backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 被排除。
- helper 是 type-only imports、common props defaults / no-op callbacks 和 summary title 常量。
- 未发现 fs/file read、Tauri、network、child process、real Codex 或 `.codex` access。
- 主测试保留 `RightDetailPanel` render、断言和测试入口列表。

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

R4-A36 不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
