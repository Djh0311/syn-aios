# Workflow Task Package Task 0-6 Evidence

## 范围

- 执行计划：`docs/plans/2026-06-01-workflow-task-package-design-v1-execution-plan.md`
- 执行范围：只做 Task 0-6。
- 未执行范围：Task 7-12 未确认，本轮没有开发。
- 边界：未迁移数据库，未写真实用户 workflow state，未读写 `/Users/yoyi/.codex`，未读 auth、token、`.env` 或完整 transcript。

## 开发前闸门

- 结论：Task 4、Task 5、Task 6 按计划中“确认 1、确认 2、确认 3”的已确认策略开发。
- 依据：计划文件的“Task 4-12 开发前确认闸门”明确写明 Task 4-6 已确认，Task 7-12 仍未确认。
- 影响：只做 v0 兼容读模型、运行前检查、任务包字段和失效规则；不做账本写入边界、子智能体完成权、状态机完成判定、接口后续扩展和端到端真实执行。
- 未定：Task 7-12 的规则仍未确认，本轮不作为已接受产品规则。

## 基线记录

- `npm run build`：接手前摘要记录为通过；本轮完成后重新运行通过。
- `npm run test:offline-interaction`：接手前摘要记录曾复现 `Invalid hook call`；Task 1 修复后通过。本轮补 UI 时曾再次触发一次同类错误，依据是测试栈指向 `WorkflowRunCheckPanel` 的 `useState`，随后改为 `memo` 包装后通过。
- `cargo test --lib`：接手前摘要记录在空 HOME 隔离下通过，未复现计划里“1 FAIL”；本轮完成后重新运行通过。
- 不确定：我没有把代码回退到修改前重新跑基线，所以“接手前”只依据上一执行摘要；最终结果依据本轮实际命令输出。

## 做了什么

- Task 1：前端离线测试改为 server render 路线，避免直接调用 hook 组件；离线测试脚本确保测试结束后退出。
- Task 2：Rust snapshot 测试使用隔离会话来源，避免读取真实 Codex sqlite 污染断言。
- Task 3：项目页主路径接入完整 `ProjectDetail`，保留项目列表、项目内工具栏、工作流编排、节点绑定、派发指令和任务包草稿入口。
- Task 4：后端派生 `Workflow`、`WorkflowNode`、`TaskPackage`、`WorkflowLedgerEntry` 等只读模型；前端补齐对应类型。
- Task 5：后端新增 `inspect_workflow_run_check`，前端新增运行前检查封装和只读状态条；缺字段显示 blocked、warning 或 empty，不自动补业务事实。
- Task 6：任务包字段展示补齐模型、读写范围、工具、技能、知识库、记忆、harness、验收标准、回传格式、禁止事项、版本和 stale 状态；人工编辑后 stale 的规则由后端测试覆盖。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-9x3mY5U7.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BxCF2HBh.css`
- `evidence/2026-06-01-workflow-task-package-plan-baseline.md`
- `handoffs/2026-06-01-workflow-task-package-task4-6-user-test.md`

说明：`dist/**` 是 `npm run build` 刷新的构建产物，不是手写产品规则。

## 测试环境

前端命令使用空 HOME：

```bash
HOME=/private/tmp/codex-workbench-empty-home
CODEX_HOME=/private/tmp/codex-workbench-empty-home/.codex
```

Rust 命令额外使用本机 Rust 工具链目录：

```bash
RUSTUP_HOME=/Users/yoyi/.rustup
CARGO_HOME=/Users/yoyi/.cargo
```

## 最终测试结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 4`。
- `npm run build`：通过；Vite 仍提示单个 chunk 超过 500 kB，这是体积 warning，不是失败。
- `cargo test --lib snapshot_keeps_metadata_without_session_body -- --nocapture`：通过，1 passed。
- `cargo test --lib workflow_task_package_read_model -- --nocapture`：通过，1 passed。
- `cargo test --lib workflow_run_check -- --nocapture`：通过，2 passed。
- `cargo test --lib task_package -- --nocapture`：通过，27 passed，1 ignored；ignored 是真实写文件确认测试，未执行。
- `cargo test --lib`：通过，73 passed，1 ignored。

## 验证到的边界

- 缺模型会 blocked，并显示“系统不会自动选择模型”。
- 缺读范围、缺写范围、缺验收标准会 blocked。
- 工具白名单和 harness 在未声明必需时显示 warning 或 empty，不自动补。
- 知识库和记忆引用未声明必需时保持 empty，不自动补。
- 任务包编辑和生成后有版本、fingerprint、stale 规则测试。
- 前端只展示运行前检查和任务包预览，不自动运行、派发或标记完成。

## 未验证

- 未启动 Tauri 窗口做真实点击验收。
- 未执行真实 `codex exec` 或 `codex exec resume`。
- 未写 `/Users/yoyi/.codex`。
- 未写真实用户 workflow state。
- 未迁移 SQLite。
