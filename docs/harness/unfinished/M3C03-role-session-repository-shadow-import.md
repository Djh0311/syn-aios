# M3C03 RoleSession repository、schema 与 shadow import

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：建立 M3 自有、版本化的 RoleSession / Turn / ProviderHandle / ConversationContext repository 与临时库 schema，复用底层即时事务而不复用 workflow sidecar 语义，并实现只读 shadow import 分类。
干完的标准：临时 SQLite 中的 create/resume/turn/handle/context 原子写、稳定 key、幂等、碰撞 quarantine、permission 收窄、restart orphan 与迁移分类通过；raw transcript 和前端 cache 不入真源。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_schema.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs

## 步骤

1. 先用临时库测试冻结 schema、唯一键、外键、状态与事务失败行为。
2. 实现 M3 domain types、repository port 和基于 immediate transaction 的 adapter。
3. 实现 Codex index/rollout、durable binding、valid continuation 的 shadow import 分类，不读取 raw transcript body。
4. 覆盖幂等、collision、orphan、permission drift、rollback 与 restart。
5. 跑聚焦测试、schema introspection、完整相关库测和非测试构建后独立提交。
