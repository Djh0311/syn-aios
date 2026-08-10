# M4C05 Secretary 应用服务与持续协调上下文

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：在持久 Secretary RoleSession 和 Attention repository 上实现查询、解释、Handoff 与回执回源的应用服务，替代固定 cwd 一次性秘书路径。
干完的标准：持续上下文跨重启；内部查询只读；Handoff unavailable/pending/returned 可恢复；deterministic brief 不依赖模型；模型增强仅由用户事件触发且有 invocation receipt。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/tests/ [新增]
- docs/harness/

## 步骤

1. 写持续上下文、无模型降级、Handoff 回执和错误恢复测试。
2. 实现 Secretary use cases、内部查询 port 与 deterministic brief。
3. 把旧 run_secretary_explain 降为受控 adapter，不再使用固定项目身份。
4. 跑聚焦/重启/失败测试，独立审查后精确提交并归档。
