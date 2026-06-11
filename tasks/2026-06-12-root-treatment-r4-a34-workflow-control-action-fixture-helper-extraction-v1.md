# Root Treatment / R4-A34 Workflow Control Action Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`d509004`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb51c-61fe-7fc3-8973-b22a4ce58911`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A34 任务包；R4-A34 继续对应官方计划 R4-6：离线测试拆分。R4-A34 只接受为 workflow control / node session / permission / work item state 相关 expected action fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A33 已完成并 checkpoint，`offline-permission-dialog.test.tsx` 当前约 4,648 行。
- `runShellScenario` / 项目工作流段仍有多段内联 expected pending action payload：用户审核指令边界、权限决定、节点会话绑定、节点会话解绑、工作项推进，以及任务草稿表单 `FormData` stub。
- 这些 expected payload 和 form stub 是纯测试 fixture；按钮查找、点击、`PermissionDialog` render、UI 文案检查、取消确认、forbidden 文案检查和 deep equality 断言仍应留在主测试。

核心判断：

```text
R4-A34 只抽 workflow control 相关纯 expected action / form fixture；主测试继续保留交互流程和行为断言。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线复用既有线程，只读审查 diff 和验证结果，不改代码。
- 本切片按中等粒度 fixture cluster 推进；如果安全 cluster 低于 250 行，不为凑行数跨入 C5 worker report / process fact 等行为输出断言。

## 2. Read / Write Scope

Allowed read paths：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- R4-A33 task / evidence / handoff
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- existing files under `prototypes/productized-desktop-shell/tests/helpers/`
- shape gate output and package scripts as needed.

Allowed write paths：

- `tasks/2026-06-12-root-treatment-r4-a34-workflow-control-action-fixture-helper-extraction-v1.md`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineTaskFieldTestUtils.ts`
- `evidence/2026-06-12-root-treatment-r4-a34-workflow-control-action-fixture-helper-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r4-a34-workflow-control-action-fixture-helper-extraction-v1-result.md`
- checkpoint 入口文档只在验证和复核通过后同步。

External changes not owned by R4-A34：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Forbidden

R4-A34 禁止：

- 不改前端产品代码、CSS、Rust、Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 不改 UI 视觉风格、布局、导航、文案或页面数据来源。
- 不改测试入口列表。
- 不迁移按钮查找、点击、`PermissionDialog` render、UI 文案检查、取消确认、forbidden 文案检查或 deep equality 行为断言。
- 不抽 C5 worker report / process fact 等由组件运行时生成的行为输出断言。
- 不启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 不解冻 Stage L / Stage K / backlog 功能。

## 4. Implementation Tasks

1. 扩展 `tests/helpers/offlineTaskFieldTestUtils.ts`：
   - `buildUserReviewedInstructionPreviewAction`
   - `buildPermissionDecisionAction`
   - `buildBindNodeSessionAction`
   - `buildUnbindNodeSessionAction`
   - `buildAdvanceWorkItemStateAction`
   - `taskDraftFormDataFixture`
2. 更新 `offline-permission-dialog.test.tsx`：
   - 使用 helper 替代内联 expected action payload 和任务草稿 `FormData` stub。
   - 保留原有 `assertDeepEqual`、render、点击、表单提交、UI 文案和取消行为断言。
3. 写 evidence / handoff，记录行数变化、验证命令、边界和不能声明项。

## 5. Acceptance Criteria

R4-A34 可接受条件：

- helper 只包含 workflow control / node session / permission / work item state expected action 与 form fixture builder，不读取文件、不启动进程、不调用 Tauri、不接触工作台运行时状态。
- `offline-permission-dialog.test.tsx` 行数继续下降。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- shape gate 通过；允许继承既有 `tauri_command_total_increased 97/96` warning。
- `git diff --check` 通过。
- 不修改产品代码、CSS、Rust、Tauri command、sidecar、DB、workflow state schema 或真实执行路径。

## 6. Verification Plan

必须运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

本切片只改 TS 测试 helper 和文档，不默认运行 Rust 测试或 `npm run build`；如未运行必须在 evidence 中说明原因。

## 7. Review Plan

实现后复用既有复核线做只读审查。

复核重点：

- diff 是否只包含 R4-A34 允许范围，且不包含 `backlog.md` / `docs/own-agent-and-company-vision-v1.md`。
- helper 是否只包含 workflow control / node session / permission / work item state expected action 与 form fixture builder。
- 主测试是否未迁移行为断言、render、点击、表单提交、UI 文案检查、取消确认或测试入口列表。
- 是否没有产品代码、CSS、Rust、Tauri、DB、sidecar 或 workflow schema 修改。
- 验证命令是否通过，shape gate 是否无新增阻断。

## 8. Checkpoint Plan

复核通过后：

1. 写入 evidence / handoff。
2. 提交 implementation commit。
3. 同步 checkpoint 入口文档。
4. 提交 checkpoint commit。
5. 回填本任务包 / evidence / handoff 中的 commit hash，并提交 hash backfill。

## 9. 不能声明

R4-A34 完成后仍不能声明：

- R4 全部完成。
- 离线测试全部按域拆分完成。
- 产品 UI、真实 Tauri、截图验收或页面真实数据来源迁移完成。
- Stage L / Stage K / backlog 功能已解冻。
- 真实 Codex 执行、真实 resume、真实 new session、真实 provider/model verification 完成。
