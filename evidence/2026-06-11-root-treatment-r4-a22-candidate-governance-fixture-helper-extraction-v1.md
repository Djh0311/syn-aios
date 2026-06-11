# Evidence: Root Treatment / R4-A22 Candidate Governance Fixture Helper Extraction v1

日期：2026-06-11

状态：实现完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a22-candidate-governance-fixture-helper-extraction-v1.md`

Planning baseline commit：`06e6959b040bf56e1be714580d0634fbe1b0f6d1`

Implementation commit：`TBD`

Review result：`STATUS: CLEAR`；P0 / P1 / P2 无。

Checkpoint commit：`TBD`

## 1. 本轮目标

R4-A22 继续 R4-6 offline test splitting，只抽离 candidate governance 相关纯测试 fixture cluster。

本轮不改变产品行为、不修改 candidate governance summary / UI / forbidden text 断言、不接入真实运行时、不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineCandidateGovernanceFixtures.ts`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-11-root-treatment-r4-a22-candidate-governance-fixture-helper-extraction-v1.md`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- candidate governance 场景断言、ProjectDetail render 检查或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。

## 3. 行数变化

`wc -l` 记录：

```text
6544 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 521 prototypes/productized-desktop-shell/tests/helpers/offlineCandidateGovernanceFixtures.ts
```

主测试从 R4-A21 后的 `7013` 行下降到 `6544` 行，减少 `469` 行。

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

第一次 shape gate 在 `prototypes/productized-desktop-shell` 下误跑，脚本相对路径不存在，失败原因为 cwd 错误：

```text
Error: Cannot find module '/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/scripts/harness/workbench-shape-gate.js'
```

随后已在 product-line 根目录重新运行并通过：

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

## 5. 过程偏差记录

本轮有两处过程偏差，均未改文件、未读写敏感路径、未触发真实执行：

- 一次只读类型签名检查误用了带 `&&` 的 shell 命令，不符合当前“单命令或并行工具”习惯。
- 一次 shape gate 使用了错误 cwd，导致脚本相对路径找不到；随后已用正确 cwd 重跑通过。

## 6. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- 修改 `backlog.md`。

## 7. 复核结果

复核线已只读检查并返回：

```text
STATUS: CLEAR
P0: 无
P1: 无
P2: 无
```

复核确认：

- 新 helper 只包含 candidate governance fixture builder。
- 主测试只改 import 与 fixture 初始化，未改 summary/UI/forbidden text 断言语义。
- 没有发现产品代码、CSS、Rust、Tauri command、DB、sidecar 或 workflow schema 修改信号。
- `backlog.md` 为外部 unrelated modified，未纳入本轮结论。

## 8. 不能声明

R4-A22 即使通过，也不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
