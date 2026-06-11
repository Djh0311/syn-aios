# Evidence: Root Treatment / R4-A21 Run Queue Fixture Helper Extraction v1

日期：2026-06-11

状态：实现完成并通过复核线 `STATUS: CLEAR_WITH_P2`。

任务包：`tasks/2026-06-11-root-treatment-r4-a21-run-queue-fixture-helper-extraction-v1.md`

Planning baseline commit：`83fec43e24b054c1745e7d1d435811403d631f4b`

Implementation commit：`TBD`

Review result：`STATUS: CLEAR_WITH_P2`；P0 / P1 无，P2 为 commit hash 元数据待回填，按 checkpoint hash backfill 流程关闭。

Checkpoint commit：`TBD`

## 1. 本轮目标

R4-A21 继续 R4-6 offline test splitting，只抽离 Stage J / K5 run queue 相关纯测试 fixture cluster。

本轮不改变产品行为、不修改 run queue / secretary / right rail 断言、不接入真实运行时、不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineRunQueueFixtures.ts`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-11-root-treatment-r4-a21-run-queue-fixture-helper-extraction-v1.md`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- run queue 场景断言、secretary/right rail 检查或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。

## 3. 行数变化

`wc -l` 记录：

```text
7013 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 352 prototypes/productized-desktop-shell/tests/helpers/offlineRunQueueFixtures.ts
 143 tasks/2026-06-11-root-treatment-r4-a21-run-queue-fixture-helper-extraction-v1.md
```

主测试从 R4-A20 后的 `7332` 行下降到 `7013` 行，减少 `319` 行。

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

## 6. 复核结果

复核线已只读检查并返回：

```text
STATUS: CLEAR_WITH_P2
P0: 无
P1: 无
P2: commit hash 元数据待回填
```

复核确认：

- 新 helper 是否只包含 Stage J / K5 run queue fixture builder。
- 主测试是否只改 import 与 fixture 初始化，未改 run queue/read model/UI/secretary/right rail 断言语义。
- 是否没有产品代码、CSS、Rust、Tauri command、DB、sidecar 或 workflow schema 修改。
- 验证命令是否足够覆盖本切片。

## 7. 不能声明

R4-A21 即使通过，也不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
