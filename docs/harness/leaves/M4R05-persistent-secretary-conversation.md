# M4R05 持续 Secretary 对话

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：复用 M3 RoleSession/Turn/ConversationTransport 接通首页输入、发送、响应/失败与跨重启历史恢复。
干完的标准：同一后端解析的 Secretary/PersonalScope/daily 会话连续两轮；强退重启恢复历史并继续；fake provider 失败可见；无消息/无事件零调用；对话不自动创建正式对象或扩权。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- docs/harness/

## 步骤

1. 复跑 R01 Secretary conversation 红灯探针。
2. 接普通 command registry 到既有 M3 RoleSession/Turn/transport，不建第二真源。
3. 接 UI composer、pending/response/failure 和持久历史读取。
4. 覆盖两轮、duplicate message、fake failure、SIGKILL/restart/continue 与零空转。
5. 跑聚焦回归与非测试构建，独立审查后精确提交并归档。
