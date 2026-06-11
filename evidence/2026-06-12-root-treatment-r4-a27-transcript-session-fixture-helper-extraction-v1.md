# Evidence: Root Treatment / R4-A27 Transcript / Session Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，等待 implementation / checkpoint hash 回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a27-transcript-session-fixture-helper-extraction-v1.md`

Planning baseline commit：`0bb2764`

Implementation commit：`TBD`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：`TBD`

## 1. 本轮目标

R4-A27 继续 R4-6 offline test splitting，只抽离 Transcript Cleaning / Session Center Hardening 相关纯测试 fixture cluster。

本轮不改变产品行为、不修改清洗、过滤、render、class、button、UI 文案检查或 forbidden text 断言，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTranscriptSessionFixtures.ts`

修改：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-12-root-treatment-r4-a27-transcript-session-fixture-helper-extraction-v1.md`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- Transcript / SessionCenter 场景的清洗、过滤、render、class、button、UI 文案检查、forbidden text 断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。
- `docs/own-agent-and-company-vision-v1.md`，该文件仍为外部未跟踪文件。

## 3. 行数变化

`wc -l` 记录：

```text
5408 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 174 prototypes/productized-desktop-shell/tests/helpers/offlineTranscriptSessionFixtures.ts
```

主测试从 R4-A26 后的 `5532` 行下降到 `5408` 行，减少 `124` 行。

说明：本切片低于 250 行软目标，但 Transcript Cleaning / Session Center Hardening 的可抽取部分是完整输入 fixture cluster；继续扩大将触碰 OfflineRoleOrchestration 或 UI 行为断言，不符合本轮边界。

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

过程偏差：

- 第一次 shape gate 误在 `prototypes/productized-desktop-shell` 子目录运行，脚本相对路径不成立，返回 `MODULE_NOT_FOUND`；未修改文件，随后已在 `/Users/yoyi/workspace/product-line` 根目录重跑并通过。

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
- 可接受为 R4-A27 implementation 完成，不阻断 implementation commit。

复核确认：

- diff 只包含 R4-A27 允许范围。
- 新 helper 只包含 Transcript / SessionCenter fixture builder。
- 主测试未改清洗、过滤、render、class、button 或断言语义。
- 未发现产品代码、CSS、Rust、Tauri、DB、sidecar 或 workflow schema 修改。
- 未发现真实执行、`codex exec` / `codex exec resume` 或 `/Users/yoyi/.codex` 读写。

## 7. 不能声明

R4-A27 即使通过，也不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
