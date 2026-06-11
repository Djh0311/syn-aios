# Evidence: Root Treatment / R4-A46 Project Blackboard Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a46-project-blackboard-fixture-helper-extraction-v1.md`

Planning baseline commit：`4c22573a941110a6b410fdf6dfc9e75d67384004`

Implementation commit：`4c10149278ca0a1d0324504ba54dc79f6cd1eb8c`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`1a0d2f4d2f1f1e523dc9a1884ed0cab5c988e749`

## 1. 本轮目标

R4-A46 继续 R4-6 offline interaction test splitting，将 `offlineDerivedWorkflowFixtures.ts` 中的 project blackboard 纯测试数据 cluster 抽离到独立 helper。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectBlackboardFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a46-project-blackboard-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a46-project-blackboard-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a46-project-blackboard-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineDerivedWorkflowFixtures.ts`

未修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A46。

## 3. 具体实现

新增 `offlineProjectBlackboardFixtures.ts`，导出：

- `projectBlackboardFixture(projectRoot)`

该 helper 只包含 project blackboard 纯 fixture/object builder：

- entries。
- promotion decision。
- warnings。
- source refs。
- candidate / permission / memory / knowledge 边界文案。

`offlineDerivedWorkflowFixtures.ts` 只新增 helper import，并将原 inline object 替换为：

```ts
project_blackboards: [projectBlackboardFixture(projectRoot)]
```

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
offlineDerivedWorkflowFixtures.ts: 629 -> 468
offlineProjectBlackboardFixtures.ts: 新增 167
offline-permission-dialog.test.tsx: 仍为 3,404，未修改
```

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: 无
P1: 无
P2: 无
```

复核确认：

- 工作树范围符合 A46：tracked diff 只有 `offlineDerivedWorkflowFixtures.ts` 和外部 `backlog.md`；新增 owned 文件为 `offlineProjectBlackboardFixtures.ts` 与 A46 任务包。`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已排除。
- `offlineProjectBlackboardFixtures.ts` 只导出 `projectBlackboardFixture(projectRoot)`，内容是 `project_blackboards` 的纯 fixture/object data；只有 type import，没有 I/O、Tauri/network/child_process、真实 Codex 或 `.codex` access。
- `offlineDerivedWorkflowFixtures.ts` 仅新增 helper import，并替换为 `project_blackboards: [projectBlackboardFixture(projectRoot)]`。
- blackboard entries、`promotion_decision`、`warnings`、`source_refs` 仍保持 candidate/read-model/control-core 边界语义，未变成产品执行能力。
- `offline-permission-dialog.test.tsx` 未修改，主测试 render、button click、pending action、payload、cancel/confirm、assert 行为断言未被触碰。
- 任务包明确排除 R4 完成、真实执行、真实 Tauri 验收、R3 Level B 和 backlog 解冻。

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
