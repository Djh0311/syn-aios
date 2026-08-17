# M3O01R01 AppState 权威槽位边界

阶段：stage-14 仍开。本文件只是独立 M3-owner 纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。

目标：纠正 `8b39d2b` 的窄合同缺陷。未安装 M3 项目角色会话权威必须经过真实 `AppState` 槽位边界返回 `m3_project_role_session_authority_unavailable`。普通产品安装与验收 / 遗留未安装行为保持不变。已安装端口继续 fail closed。不声称 M3 已解阻。

来源：独立验收拒绝 `8b39d2b0f8a19b15085f369babf8da5eb29770f9`；用户要求在当前工作树做窄纠正。

产品：`docs/contracts/m3-project-role-session-authority-slot-boundary-v1.md`，`AppState` 服务器-only Result accessor，真实槽位测试。
