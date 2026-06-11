# Evidence: Root Treatment / R4-A48 Candidate Memory Lifecycle Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a48-candidate-memory-lifecycle-fixture-helper-extraction-v1.md`

Planning baseline commit：`d4c4bb9d05d1096303738b98dfbc41aee63c66a1`

Implementation commit：`a89b1142b572a665dbd7964d15cb6af7e3a2d949`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`f73e14493acd18784a7be014c0caee5c3714c5bf`

## 1. 本轮目标

R4-A48 继续 R4-6 offline interaction test splitting，将 `offlineCandidateGovernanceFixtures.ts` 中的 candidate memory lifecycle 纯测试数据 cluster 抽离到独立 helper。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineCandidateMemoryLifecycleFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a48-candidate-memory-lifecycle-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a48-candidate-memory-lifecycle-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a48-candidate-memory-lifecycle-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineCandidateGovernanceFixtures.ts`

未修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A48。

## 3. 具体实现

新增 `offlineCandidateMemoryLifecycleFixtures.ts`，导出：

- `candidateMemoryLifecycleFixtures(input)`

该 helper 返回：

- `formalMemoryStore`
- `adoptedFormalMemoryStore`
- `memoryLintStore`
- `taskMemoryPacketPreview`

`offlineCandidateGovernanceFixtures.ts` 保留组合入口，并继续返回原字段名：

- blackboard candidate store。
- memory candidate stores。
- observation store。
- empty candidate store。
- memory lifecycle helper 返回的四个对象。

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
offlineCandidateGovernanceFixtures.ts: 521 -> 239
offlineCandidateMemoryLifecycleFixtures.ts: 新增 308
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

- 工作树范围符合 A48：tracked diff 只有 `offlineCandidateGovernanceFixtures.ts` 和外部 `backlog.md`；新增 owned 文件为 `offlineCandidateMemoryLifecycleFixtures.ts` 与 A48 任务包。`backlog.md`、`docs/own-agent-and-company-vision-v1.md` 已排除。
- `offlineCandidateMemoryLifecycleFixtures.ts` 只导出 `candidateMemoryLifecycleFixtures(input)`，并返回 `formalMemoryStore`、`adoptedFormalMemoryStore`、`memoryLintStore`、`taskMemoryPacketPreview`。
- `offlineCandidateGovernanceFixtures.ts` 只是 import 新 helper；仍保留 candidate governance 组合入口，并在返回中保留原字段名。
- formal memory records/versions/audit events、adopted formal memory audit、lint findings/runs、task memory packet included/excluded/review_materials/warnings 仍是纯测试数据和预览/候选语义，未变成产品执行能力。
- `offline-permission-dialog.test.tsx` 未修改，render、button click、pending action、payload、cancel/confirm、assert 行为断言未被触碰。
- 执行入口扫描未发现 I/O、Tauri/network/child_process、真实 Codex 执行或 `.codex` access；任务包明确排除 R4 完成、真实执行、真实 Tauri 验收、R3 Level B 和 backlog 解冻。

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
