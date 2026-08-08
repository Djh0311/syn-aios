# 阶段5 M3 角色会话与显式交接

总计划：product-line 唯一基线与 Harness Lite 切换
目标：在 M0 干净正本、M1 冻结身份安全合同和 M2 有边界事务参考切片之上，建立可持久恢复的 RoleSession / Turn / ProviderHandle / ConversationContext 与显式 Handoff，使角色、范围、对象、通道和权限由 Syn 真源控制，并让前端缓存退出事实 owner。

干完的标准：

- M1 冻结合同保持不变；M3 增量实施合同、迁移矩阵、owner/scope 守卫和 repository 版本得到机械验证。
- RoleSession、Turn、provider handle、最小 ConversationContext 与 Handoff 在隔离临时库和 fake provider 下可新建、续接、停止、重启恢复、拒绝冲突并幂等回源。
- Agent existing-thread、跨项目、Station 3b 和 permission drift 在 provider spawn 前 fail closed；明确授权的外部项目建立独立范围，不复用错误 thread。
- 两套前端 module/React cache 退成兼容显示来源；后端读模型是恢复真源，旧路保留可回切入口但不绕过 M1 守卫。
- 离线单元、属性、临时 repository、fake provider、非测试构建、前端类型和隔离桌面验收按实际层级结算；不冒充真实 Codex 消息或发布通过。
- 当前状态、证据和后续边界完成回写；不自动激活 M4/M5。

允许动：

- docs/contracts/m3-role-session-turn-handoff-resolution-v1.md
- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/
- handoffs/
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/manual_relay/
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_handoff.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_acceptance.rs
- prototypes/productized-desktop-shell/src/lib/
- prototypes/productized-desktop-shell/src/views/agent/
- prototypes/productized-desktop-shell/src/views/projects/jiaoban/
- refs/heads/main
- /private/tmp/product-line-syn-m3-

只读：

- M1/M2 冻结合同、opening manifests、当前源码和已归档验收证据
- /Users/yoyi/workspace/product-line-syn-fnd-002
- /Users/yoyi/workspace/product-line-syn-m2-closeout
- Codex SQLite / rollout 结构与 supervisor binding，只作来源清单和隔离 fixture 设计

不许动：

- M1 冻结合同正文、manifest opening OID 与历史验收结论
- 两个保全工作树的 index、tracked/untracked 内容与分支头
- 真实 provider、真实 Codex 消息、真实账号、凭据、外部 connector、真实用户项目数据和远端
- reset、clean、stash、rebase、merge、push、部署、发布和破坏性清理
- M4-M10 的产品实现、完整知识检索/同步、记忆治理和 connector 实现

## 叶子

- [x] M3C01 RoleSession / Turn / Handoff 实施合同与迁移矩阵
- [ ] M3C02 Agent existing-thread owner / scope 后端守卫
- [ ] M3C03 RoleSession repository、schema 与 shadow import
- [ ] M3C04 ConversationTransportPort 与 fake provider 重启语义
- [ ] M3C05 显式 Handoff 状态机与结果回源
- [ ] M3C06 会话读模型与前端缓存退位
- [ ] M3C07 隔离桌面分层验收与迁移回切证据
- [ ] M3C08 M3 集成回归、现状回写与阶段收口
