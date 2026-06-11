# Evidence: Root Treatment / R4-A32 Task Workflow Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r4-a32-task-workflow-action-fixture-helper-extraction-v1.md`

Planning baseline commit：`e209cef`

Implementation commit：`03dfa4d68fda2fea8047116c1380de74f8ee2716`

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：`9296298af0334aeeb3dbf794bfac95dedac6fc67`

## 1. 本轮目标

R4-A32 继续 R4-6 offline interaction test splitting，只抽离任务 / 工作流 pending action 与 task field 相关离线 fixture cluster。

本轮不改变产品行为、不修改 `runShellScenario` 的按钮查找、表单提交、`PermissionDialog` render、UI 文案检查、取消确认、deep equality 行为断言或测试入口列表，不接入真实运行时，不解冻 backlog 功能。

## 2. 改动范围

新增：

- `tasks/2026-06-12-root-treatment-r4-a32-task-workflow-action-fixture-helper-extraction-v1.md`

修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

未修改：

- 前端产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- `runShellScenario` 的按钮查找、表单提交、`PermissionDialog` render、UI 文案检查、取消确认、deep equality 行为断言或测试入口列表。
- `backlog.md`，该文件仍为外部未暂存改动。
- `docs/own-agent-and-company-vision-v1.md`，该文件仍为外部未跟踪文件。

## 3. 行数变化

`wc -l` 记录：

```text
4681 prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
 183 prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts
```

主测试从 R4-A31 后的 `4,743` 行下降到 `4,681` 行，减少 `62` 行。

说明：本切片低于 250 行软目标，但 bootstrap workflow、create task draft、copy task preview、generate task file、field correction、task field update 是完整 task/workflow action fixture cluster；继续扩大将跨入 render、点击、表单提交、UI 文案断言或取消确认等行为验收。

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

- hash 回填扫描和收尾旧口径扫描时，各有一条 `rg` 命令误把带反引号的 pattern 放进 shell 双引号，zsh 分别触发了两次字面量 `pending` 命令替换并返回 `command not found`。
- 随后已用单引号 pattern 重跑安全扫描，hash placeholder 无命中。
- 该偏差未修改文件，未启动 Tauri / Browser / Chrome / Vite dev，未执行真实 Codex，未读写 `/Users/yoyi/.codex`。

未运行：

- Rust 测试：本切片只改 TS 测试 helper 和任务文档，不改 Rust / Tauri。
- `npm run build`：本切片只做离线测试 fixture 抽离，已由 typecheck 与 offline interaction 覆盖。

## 5. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema。
- 修改 `backlog.md`。

敏感 / 真实执行关键词扫描分类：

- `offlineTaskFieldTestUtils.ts` 中没有 `codex exec`、`codex exec resume`、`/Users/yoyi/.codex`、Tauri invoke、文件读写或进程启动命中。
- 主测试命中均为历史禁止文案、边界断言或 fixture preview，不是新增真实执行路径。

## 6. 复核状态

复核线已只读检查并返回：

- 复核线程：`019eb51c-61fe-7fc3-8973-b22a4ce58911`
- `STATUS: CLEAR`
- P0：无
- P1：无
- P2：无
- 可接受为 R4-A32 implementation 完成，不阻断 implementation commit。

复核确认：

- 任务包把范围限定为 task/workflow pending action 与字段输入 fixture 抽离，且明确不能冒领为 R4 完成或真实执行验收。
- helper 扩展后仍是纯 fixture / builder：新增 helper 都只返回静态对象或 `Map`，未见文件读取、进程启动、Tauri 调用或运行时接触。
- 主测试保留按钮查找、表单提交、`PermissionDialog` render、取消确认和 `assertDeepEqual` 行为断言，只把内联 fixture 改为 helper 调用。
- 未发现产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema 或真实执行路径修改。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 仍是外部变更，不纳入本轮结论。

## 7. 不能声明

R4-A32 即使复核通过，也仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
