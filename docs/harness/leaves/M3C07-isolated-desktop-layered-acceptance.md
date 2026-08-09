# M3C07 隔离桌面分层验收与迁移回切证据

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：在隔离 profile、临时 SQLite 和 fake provider 下分层验收角色会话 new/continue/stop/restart、跨项目/Station 3b 拒绝、Handoff 重放与前端恢复，记录迁移和回切证据。
干完的标准：contract/unit/temp-repository/fake-provider/non-test-build/frontend/isolated-desktop 各层结论独立；真实 provider/Codex 消息保持未进入；失败注入与回切不恢复安全旁路。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m3_acceptance.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src/lib/roleSessionReadModel.test.ts [新增]
- docs/harness/reports/M3C07-isolated-desktop-layered-acceptance.md [新增]

## 步骤

1. 冻结隔离 profile、临时路径、fake provider 和禁止访问真实 provider 的哨兵。
2. 覆盖每种角色 new/continue/stop 与 start/pending/terminal 三点 restart。
3. 覆盖跨项目、伪造 thread、Station 3b、permission drift 和 Handoff replay。
4. 运行非测试构建、前端类型/单测和隔离桌面观察，证据逐层记录。
5. 验证 rollback 只切 read/UI path，不移除后端守卫；独立提交。
