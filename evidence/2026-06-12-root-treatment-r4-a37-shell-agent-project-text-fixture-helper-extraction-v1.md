# Evidence: Root Treatment / R4-A37 Shell Agent Project Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a37-shell-agent-project-text-fixture-helper-extraction-v1.md`

Planning baseline commit：`9c8568f0eb7762fc4e3b3eef719a16db31f4d4c3`

Implementation commit：`80648a18829af06d10810cd08c997a763f72f000`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`b5d0a65f269d819932d730a91395f562b0a3b83a`

## 1. 本轮目标

R4-A37 继续 R4-6 offline interaction test splitting，只抽 `runShellScenario` 中 Shell / Agent / Project / Workflow / Skill / Harness 相关 expected / forbidden text 和导航期望数据。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineShellScenarioTextFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a37-shell-agent-project-text-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a37-shell-agent-project-text-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a37-shell-agent-project-text-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A37。

## 3. 具体实现

新增 helper：

- `shellScenarioTextFixtures`
- `shellProposalDialogExpectedTexts`
- `shellDerivedWorkflowExpectedTexts`

主测试调整：

- 首页、设置、运行中、Agent、项目入口、项目内 Agent、工作流草稿、工作流画布、方案确认、全局复核、项目主管拆任务、派生工作流、运行前检查、C5/C6、权限/总指导/绑定/解绑/任务文件/字段修正/事实层、Skill 和 Harness 的 expected / forbidden text 改为 helper 引用。
- primary nav label / glyph 期望数据改为 helper 引用。

主测试仍保留：

- JSX render、`visibleText` / `renderToStaticMarkup`、button 查找和 click、`assert` / `assertDeepEqual` 行为断言、payload 对比、class/order 断言、forbidden text 断言循环和测试入口列表。

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

- `offline-permission-dialog.test.tsx`：4,535 -> 4,045。
- `offlineShellScenarioTextFixtures.ts`：新增 320 行。

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

- A37 owned scope 限定为任务包、新 helper 和主测试；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 被排除。
- helper 无 import，只导出 text/nav fixture 与两个 projectRoot 拼接函数。
- helper 无 I/O、产品 import、Tauri/network/child process、真实 Codex 或 `.codex` access。
- `runShellScenario` 仍保留 JSX render、visible text extraction、button lookup/click、payload assert、class/order assert 和 forbidden text assert。

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

R4-A37 不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
