# M3C06 会话读模型与前端缓存退位

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：接入后端 RoleSession 读模型、固定角色/项目/对象/通道标签、历史会话目录、知识来源和资料缺口；Jiaoban 与 Agent Center cache 退成兼容显示缓存，不再决定恢复身份。
干完的标准：重载后从后端 DTO 恢复选择和会话；前端不得上传或覆盖 role/scope/permission 真值；旧 cache 只在同进程内显示 fallback，并有显式 legacy 标记和回切边界。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src/lib/conversationTransport.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/roleSessionReadModel.ts [新增]
- prototypes/productized-desktop-shell/src/views/AgentView.tsx
- prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx
- prototypes/productized-desktop-shell/src/views/agent/useAgentSessionPage.ts
- prototypes/productized-desktop-shell/src/views/agent/useAgentTranscriptLoader.ts
- prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx
- prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts
- prototypes/productized-desktop-shell/tests/role-session-read-model.test.ts [新增]
- prototypes/productized-desktop-shell/tests/agent-session-row.test.tsx
- prototypes/productized-desktop-shell/tests/shared-conversation-transport.test.tsx
- prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx
- prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
- prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs

## 步骤

1. 冻结后端 read DTO、loading/error/empty/quarantine 与 compatibility 状态。
2. 增加 Tauri read commands 和前端 typed client，不开放客户端写 owner 字段。
3. 先接 Agent Center，再接 Jiaoban；逐项移除 cache 的 truth-owner 作用。
4. 覆盖 reload、跨项目拒绝、缺资料、source link、legacy fallback 和 stale response。
5. 跑 Rust 聚焦、前端单测/类型检查与非测试构建，独立提交。

范围校正：原任务包误写了不存在的 `src/views/agent/AgentView.tsx`，且漏列真正决定 existing transport 的 `AgentConversationShell.tsx`、承载 Jiaoban 读模型展示与显式选择的 `ProjectJiaobanPanel.tsx`、M3 后端 runtime/repository 接线与离线测试入口。上述校正只为完成本叶既定目标；真实桌面、真实 provider 与迁移回切证据仍属于 M3C07。
