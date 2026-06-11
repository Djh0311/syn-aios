# Evidence: Root Treatment / R4-A23 Memory Center Core Store Fixture Helper Extraction v1

日期：2026-06-11

状态：实现完成，复核线 `STATUS: CLEAR`，等待 implementation commit。

任务包：`tasks/2026-06-11-root-treatment-r4-a23-memory-center-core-store-fixture-helper-extraction-v1.md`

Planning baseline commit：`a06751ca23bea40fa22dfd2a792fa1992164afaa`

Implementation commit：`TBD`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：`TBD`

## 1. 本轮目标

R4-A23 继续 R4-6 offline test splitting，只抽离 Memory Center core stores 相关纯测试 fixture cluster。

本轮不改变产品行为、不修改 memory center summary / UI / forbidden text 断言、不接入真实运行时、不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryCenterCoreFixtures.ts`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-11-root-treatment-r4-a23-memory-center-core-store-fixture-helper-extraction-v1.md`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- memory center 场景的 summary、MemoryCenterView render、UI 文案检查、forbidden text 断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。

## 3. 行数变化

`wc -l` 记录：

```text
6193 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 371 prototypes/productized-desktop-shell/tests/helpers/offlineMemoryCenterCoreFixtures.ts
```

主测试从 R4-A22 后的 `6544` 行下降到 `6193` 行，减少 `351` 行。

## 4. 验证

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
npm run typecheck
```

结果：

```text
tsc --noEmit
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

- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无
- 可接受为 R4-A23 implementation 完成，但不能声明 R4 完成、离线测试全部拆分完成、真实 Tauri/截图验收完成或页面真实数据来源迁移完成。

## 7. 不能声明

R4-A23 即使通过，也不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
