# Evidence: Root Treatment / R4-A40 Memory Knowledge Secretary Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a40-memory-knowledge-secretary-text-fixture-helper-extraction-v1.md`

Planning baseline commit：`dc3409686bd324bddfd5849a84ff0dc7c991896a`

Implementation commit：`b2802795de657da80fc6f38cf6df39cc6f969af2`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`126f93054ce68ca945f2545c10302081e7ebe2cf`

## 1. 本轮目标

R4-A40 继续 R4-6 offline interaction test splitting，只抽 memory / knowledge / secretary 相关场景中的 expected / forbidden / class text arrays。

覆盖范围：

- 工作流观察 / 正式记忆 / lint / 任务记忆包预览。
- 记忆中心主界面和 class 检查。
- lifecycle / relation / maintenance / mature pattern / quarantine 确认弹层。
- 知识库资料页与提出记忆候选确认弹层。
- 秘书只读摘要与右侧秘书入口。

本轮不改变产品行为，不修改 UI、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryKnowledgeTextFixtures.ts`
- `tasks/2026-06-12-root-treatment-r4-a40-memory-knowledge-secretary-text-fixture-helper-extraction-v1.md`
- `evidence/2026-06-12-root-treatment-r4-a40-memory-knowledge-secretary-text-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a40-memory-knowledge-secretary-text-fixture-helper-extraction-v1-result.md`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 产品源码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema、真实执行路径。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A40。

## 3. 具体实现

新增 helper：

- `memoryKnowledgeTextFixtures`

抽离内容：

- `observationWorkflowExpectedTexts` / `observationWorkflowForbiddenTexts`
- `formalMemorySummaryForbiddenTexts`
- `memoryLintSummaryExpectedTexts`
- `taskPackageMemoryInjectionSummaryExpectedTexts`
- `taskMemoryPacketWorkflowExpectedTexts` / `taskMemoryPacketWorkflowForbiddenTexts`
- `memoryCenterExpectedTexts` / `memoryCenterForbiddenTexts` / `memoryCenterExpectedClasses`
- `lifecycleDialogExpectedTexts`
- `relationDialogExpectedTexts`
- `maintenanceDialogExpectedTexts` / `maintenanceDialogForbiddenTexts`
- `maturePatternDialogExpectedTexts` / `maturePatternDialogForbiddenTexts`
- `quarantineDialogExpectedTexts`
- `knowledgeViewExpectedTexts` / `knowledgeViewForbiddenTexts`
- `knowledgeCandidateDialogExpectedTexts`
- `secretaryBriefExpectedTexts` / `secretaryBriefForbiddenTexts`
- `secretaryPanelExpectedTexts` / `secretaryPanelForbiddenTexts`

主测试仍保留：

- data fixture 获取。
- `summarize*` / `derive*` read model checks。
- `MemoryCenterView` / `KnowledgeBaseView` / `SecretaryBrief` / `RightDetailPanel` render。
- button 查找、click、pending action 和 payload 检查。
- `assert` / `assertDeepEqual` 行为断言。

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

说明：第一次从 `prototypes/productized-desktop-shell` 子目录误跑 shape gate，因脚本相对路径不存在得到 `MODULE_NOT_FOUND`；随后在 repo root 重新运行通过。该失败不是产品或测试失败。

```text
git diff --check
```

结果：通过，无输出。

行数：

```text
offline-permission-dialog.test.tsx: 3,794 -> 3,576
offlineMemoryKnowledgeTextFixtures.ts: 269
```

## 5. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

复核回交：

```text
STATUS: CLEAR
P0: None
P1: None
P2: None
```

复核确认：

- `offlineMemoryKnowledgeTextFixtures.ts` 无 import，只导出 `memoryKnowledgeTextFixtures`，内容是 expected / forbidden / class text arrays；未见 I/O、产品 import、Tauri/network/child_process、真实 Codex 或 `.codex` access。
- 主测试仍保留数据 fixture、summary derivation、`MemoryCenterView` / `KnowledgeBaseView` / `SecretaryBrief` / `RightDetailPanel` render、button 查找点击、pending action / payload 断言。
- A40 owned scope 只有测试 helper、主测试和任务包；`backlog.md` 与 `docs/own-agent-and-company-vision-v1.md` 是外部变更并已排除。
- 任务包明确写明不可接受 R4 完成、全部拆分完成、真实执行、真实 Tauri 验收、R3 Level B 或 backlog 解冻等越界声明。

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
