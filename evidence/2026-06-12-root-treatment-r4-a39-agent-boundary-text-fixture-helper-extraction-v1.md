# Evidence: Root Treatment / R4-A39 Agent Boundary Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a39-agent-boundary-text-fixture-helper-extraction-v1.md`

Planning baseline commit：`95312f2a85a5dc97f6770860dad7d04b360cd318`

Implementation commit：`aa3cfc420a9487d7afe3adbccc898bd5772c2fdb`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`9f2f96b0703a9e1fb858a187d56e35d858c66617`

## 1. 本轮目标

R4-A39 继续 R4-6 offline interaction test splitting，只抽 Agent 相关边界场景中的 expected / forbidden / button / proposal text arrays。

覆盖范围：

- 会话操作边界。
- Provider / 模型 / 凭据边界。
- Adapter SDK / CLI diagnostics 边界。
- Session Continuation preview。
- E5 Level A。
- H2 real resume readiness。
- E6 runtime attention。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineAgentBoundaryTextFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a39-agent-boundary-text-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a39-agent-boundary-text-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a39-agent-boundary-text-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A39。

## 3. 具体实现

新增 helper：

- `agentBoundaryTextFixtures`

抽离内容：

- 会话操作边界 expected text、forbidden button/text/proposal text。
- Provider availability serialized forbidden fragment、expected / forbidden / button / proposal text。
- Adapter SDK / CLI diagnostics expected / forbidden / button text。
- Session Continuation preview expected / forbidden / button / proposal text。
- E5 Level A expected / forbidden / button / proposal text。
- H2 real resume readiness expected / forbidden / button / proposal text。
- E6 runtime attention expected / forbidden / button / proposal text。

主测试仍保留：

- Adapter / session / provider / continuation / readiness / runtime derivation。
- read model schema/status/kind/guard/readiness 检查。
- Secretary risk/suggestion/action proposal kind 检查。
- JSX render、`visibleText` / `renderToStaticMarkup`。
- `assert` / `assertDeepEqual` 行为断言、button text 断言循环和测试入口列表。

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

- `offline-permission-dialog.test.tsx`：3,967 -> 3,794。
- `offlineAgentBoundaryTextFixtures.ts`：新增 204 行。

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

- helper 无 import，只导出 `agentBoundaryTextFixtures` text arrays；`.codex` / `codex exec resume` 字符串只是静态边界/forbidden text。
- 主测试仍保留 session/provider/adapter/continuation/readiness derivation 与 guard/readiness checks。
- 主测试仍保留 Secretary risk/suggestion/action proposal kind checks、JSX render、`visibleText` / `renderToStaticMarkup` 和 button text assertions。
- A39 owned scope 只包含 tests/helper 与 task/evidence/handoff；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 被排除为外部变更。
- 未发现产品代码、UI/CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径变化。

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

R4-A39 不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
