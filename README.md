# Syn 个人智能工作台

Syn 是为当前用户长期服务的个人智能工作台。它让秘书、全局主管、项目主管、稳定成员和临时智能体围绕同一套事实、知识、记忆、技能、权限与审计持续协作。当前首先接入的是 Codex（代码智能体），以后可以接入其他智能体、模型和服务提供方；角色身份不与某一种实现永久绑定。

## 产品入口

- Syn 是什么、长期必须满足什么：`docs/product/syn-product-canon-v1.md`
- 哪些文件现在有效、各自能决定什么：`docs/product/authority-register-v1.md`
- 尚未拍板的问题：`docs/product/candidate-register-v1.md`
- 所有智能体怎样使用资料和技能说明：`docs/product/knowledge-infrastructure-canon-v1.md`
- 系统怎样分层和协作：`docs/workbench-system-architecture-v1.md`
- 普通界面、专业界面和开发界面分别显示什么：`docs/workbench-frontend-display-boundary-v1.md`
- 当前代码和开发事实：`docs/current-state.md`
- 当前总开发计划：`docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`

产品正本、架构正本、当前实现、候选设想、验收证据和施工权限是不同的东西。验收报告只证明点名版本和场景；交接只说明当时做到哪里；开发护栏只约束本轮怎么施工。三者都不单独定义 Syn 最终是什么。

## 开发入口

先读 `AGENTS.md`，再按轻量开发护栏的当前链进入：

```text
docs/harness/plan.md → 当前阶段 → 唯一当前任务包（leaf，当前叶）→ docs/harness/authorization.json
```

没有活动阶段和当前任务包时，不从旧计划、旧授权、交接或验收报告推导新的施工权限。远端、部署、发布、真实服务提供方、真实账号和真实消息仍需对应的明确授权。
