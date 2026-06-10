# Handoff: Workflow C2 Project Consultation Proposal And User Confirmation Entry v1

日期：2026-06-04

## 当前状态

C2 已完成并通过代码验收。当前入口应更新为“C2 已完成，下一步 C3”。

## 已做内容

- 新增项目咨询方案 sidecar：`project-proposals.v1.json`。
- 新增 `ProjectConsultationProposal`、`ProjectConsultationProposalScopeDraft`、`ProjectConsultationProposalDecision`、`ProjectConsultationProposalReadModel` 等后端 / 前端类型。
- 新增后端 store：load / create / render markdown / record decision。
- 用户确认 proposal 时，会创建 C1 `PlanAuthorization`，写 `source_proposal_id`，并调用 C1 用户确认逻辑。
- 授权确认后保持 `pending_global_boundary_review`，不会变成 `active`。
- `request_changes` / `reject` 不创建授权。
- 项目工作流侧栏新增“项目咨询方案草案”卡片、确认范围 / 要求修改 / 拒绝方案动作。
- 确认弹层展示目标、读写范围、工具 / 检查、停止条件，并明确本轮不会启动真实 worker。
- 离线测试覆盖 C2 UI 文案、action payload、确认弹层边界和禁止文案。
- Rust 测试覆盖草案创建、必填拒绝、确认联动授权、guard 仍 needs_review、要求修改 / 拒绝不建授权、重复确认拒绝。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib project_consultation_proposal`
- `cargo test --lib plan_authorization`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`
- `rustfmt --check src/project_consultation_proposal_store.rs src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs`
- Vite HTTP smoke：`npm run dev -- --port 5174` + `curl -sS http://127.0.0.1:5174/`

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未接真实项目咨询 LLM / agent。
- 未创建新的 Codex session。
- 未让 authorization 变成 `active`。
- 未新增 workflow state 顶层结构。
- 未改 workflow / work item / node 状态枚举。
- 未做真实截图验收；当前对话没有浏览器截图工具，项目未安装 Playwright。
- 边界注意：因系统技能规则读取过 `/Users/yoyi/.codex/skills/ui-ux-pro-max/SKILL.md`；未读写 Codex 会话、sqlite、session index、rollout、凭据或 `/Users/yoyi/.codex` 下业务状态文件。

## 接手建议

下一步是 C3：全局主管方案边界复核和授权生效。

C3 注意：

- 只能基于 C2 confirmed proposal 和 C1 authorization 回链做全局主管边界复核。
- C3 才能把授权从 `pending_global_boundary_review` 推进到 `active`，且必须有明确复核记录。
- 不能绕过 C1 guard 自动派发。
- 不能把 C2 的用户确认解释成 worker 已执行或授权已生效。
- 继续不执行真实 Codex / worker，除非新任务包和用户另行明确授权。
