# M4C02 普通产品 Secretary RoleSession 与 PersonalScope 接线

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：把 M3 RoleSession repository/read runtime 以普通产品可用、后端固定 Secretary/PersonalScope 的方式注入 AppState，不复用固定项目 cwd 或 acceptance-only gate。
干完的标准：普通产品启动可创建/恢复 Secretary RoleSession；personal scope、daily channel、permission snapshot 与 owner fingerprint 后端确定；跨项目和错误 scope fail closed；重启、幂等和回切测试通过。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs [新增]
- prototypes/productized-desktop-shell/tests/ [新增]
- docs/harness/

## 步骤

1. 写普通产品默认 unavailable 的失败测试与 PersonalScope identity fixtures。
2. 建立 M4-owned personal identity 与 Secretary runtime config，不放宽 M1/M3 守卫。
3. 接 AppState 和固定 command 入口，覆盖新建、恢复、重启、错误 scope 与幂等。
4. 跑聚焦 Rust/前端合同测试和非测试构建，独立审查后精确提交并归档。
