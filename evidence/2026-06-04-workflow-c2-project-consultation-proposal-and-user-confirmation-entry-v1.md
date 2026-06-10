# Evidence: Workflow C2 Project Consultation Proposal And User Confirmation Entry v1

日期：2026-06-04

## 结论

C2 已完成：工作台新增项目咨询方案草案 sidecar、草案创建 / Markdown 渲染 / 用户决定后端命令、用户确认到 C1 `PlanAuthorization` 的受控联动，以及项目工作流侧栏“项目咨询方案草案”卡片和确认动作。

接受为：

- `project-proposals.v1.json` sidecar 骨架已落地，包含 revision、proposals、decisions、audit_events、lock、备份、原子写和损坏 JSON 拒绝覆盖。
- 后端已有 `ProjectConsultationProposal` / `ProjectConsultationProposalScopeDraft` / `ProjectConsultationProposalDecision` / `ProjectConsultationProposalReadModel` 等类型。
- 用户 `confirm` proposal 会创建 C1 `PlanAuthorization`，写入 `source_proposal_id`，再记录 C1 用户确认。
- 用户确认后授权状态停在 `pending_global_boundary_review`，C1 guard 仍返回 `needs_review`，不会自动派发。
- 用户 `request_changes` / `reject` 只写 proposal decision / audit，不创建授权。
- 项目工作流侧栏显示方案草案、状态、目标、范围计数、步骤、确认 / 要求修改 / 拒绝动作和“待全局复核”边界文案。

不接受为：

- 全局主管方案边界复核完成。
- 授权已经 active。
- 项目主管已经自动派发 worker。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- 真实项目咨询 LLM / Codex 会话已接入。
- 自动化工作流产品化闭环完成。

## 关键实现

- 新增后端模块：`prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`。
- 更新后端类型：`prototypes/productized-desktop-shell/src-tauri/src/types.rs`。
- 更新 Tauri command：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`、`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`。
- 更新前端类型和 wrapper：`prototypes/productized-desktop-shell/src/lib/types.ts`、`prototypes/productized-desktop-shell/src/lib/tauri.ts`。
- 新增前端读模型 helper：`prototypes/productized-desktop-shell/src/lib/projectConsultationProposal.ts`。
- 更新项目工作流 UI：`prototypes/productized-desktop-shell/src/App.tsx`、`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`、`prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`、`prototypes/productized-desktop-shell/src/styles.css`。
- 更新离线 UI 测试：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留既有 chunk size warning。
- `cargo test --lib project_consultation_proposal`
  - 5 passed。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib`
  - 182 passed, 1 ignored。
- `rustfmt --check src/project_consultation_proposal_store.rs src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs`
- Vite 本地 HTTP smoke：
  - `npm run dev -- --port 5174` 启动成功。
  - `curl -sS http://127.0.0.1:5174/` 返回 Vite HTML。

说明：

- 真实窗口 / 浏览器截图验收未完成：当前对话没有暴露 in-app browser / screenshot 工具，项目本地也没有安装 Playwright 包。已记录为 C2 残余验收缺口。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未创建新的 Codex session。
- 未迁移数据库。
- 未新增 `workflow-state.v0.json` 顶层数组。
- 未修改 workflow / work item / node 既有状态枚举。
- 未让 C2 用户确认后的 authorization 进入 `active`。
- 未显示“worker 已执行 / worker 已启动 / 已自动执行 / 授权已生效可自动派发”。
- 边界注意：本轮因系统技能规则读取过 `/Users/yoyi/.codex/skills/ui-ux-pro-max/SKILL.md` 以执行 UI 质量约束；未读写 Codex 会话、sqlite、session index、rollout、凭据或 `/Users/yoyi/.codex` 下业务状态文件。

## 后续

下一步建议进入 C3：全局主管方案边界复核和授权生效。C3 必须复用 C1 / C2 的 proposal 和 authorization 回链，不能绕过 C1 guard 直接派发真实 worker。
