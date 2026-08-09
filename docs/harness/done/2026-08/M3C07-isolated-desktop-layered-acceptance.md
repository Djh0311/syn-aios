# M3C07 隔离桌面分层验收与迁移回切证据

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：在隔离 profile、临时 SQLite 和 fake provider 下分层验收角色会话 new/continue/stop/restart、跨项目/Station 3b 拒绝、Handoff 重放与前端恢复，记录迁移和回切证据。
干完的标准：contract/unit/temp-repository/fake-provider/non-test-build/frontend/isolated-desktop 各层结论独立；真实 provider/Codex 消息保持未进入；失败注入与回切不恢复安全旁路。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m3_acceptance.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/views/AgentView.tsx
- prototypes/productized-desktop-shell/src/views/agent/M3AcceptancePanel.tsx [新增]
- prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx
- prototypes/productized-desktop-shell/tests/m3-isolated-desktop-acceptance.test.tsx [新增]
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
- prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs
- docs/harness/reports/M3C07-isolated-desktop-layered-acceptance.md [新增]
- docs/harness/reports/M3C07-isolated-desktop-evidence/ [新增，仅窗口截图与脱敏 launcher receipt]

范围校正：原任务包误写了不存在且不符合现有 runner 约定的 `src/lib/roleSessionReadModel.test.ts`，并漏列隔离 runtime 注入、fake-only transport、固定 host 验收命令、真实前端交互面和隔离启动器。M3C06 已提交的普通生产路径仍默认 `M3_BINDING_UNAVAILABLE`；本叶新增入口只能在 debug build、已校验的 isolated profile 和显式 M3C07 mode 同时成立时启用。

窄操作授权：只允许运行本仓库自己构建的 debug App bundle，只允许创建、启动、强退和重启合成 isolated profile 的目标子进程，并保存窗口级截图、交互与脱敏 receipt。真实 provider、真实 Codex 消息、真实账号、凭据、真实项目数据和外部 connector 仍未授权。

## 步骤

1. 冻结隔离 profile、临时路径、fake provider 和禁止访问真实 provider 的哨兵。
2. 覆盖每种角色 new/continue/stop 与 start/pending/terminal 三点 restart。
3. 覆盖跨项目、伪造 thread、Station 3b、permission drift 和 Handoff replay。
4. 运行非测试构建、前端类型/单测和隔离桌面观察，证据逐层记录。
5. 验证 rollback 只切 read/UI path，不移除后端守卫；独立提交。
