# 阶段6 M4 秘书、Attention 与日常节奏

总计划：product-line 唯一基线与 Harness Lite 切换
目标：在 M1 冻结边界、M2 有边界参考切片和 M3 已完成合同/隔离实现之上，把普通产品模式接入可持久恢复的 Secretary RoleSession 与 PersonalScope，建立 M4 自有的收件、关注、个人待办、提醒、待决定、日报和事件驱动协调闭环。

干完的标准：

- M4 实施合同冻结普通产品 M3 运行时桥接、Secretary/PersonalScope、M1/M2 复用上限、M4 自有存储、来源、去重、排序、时区、日报、记忆边界、迁移回切与证据等级。
- 普通产品模式中的 Secretary 使用 Syn 持有的持久 RoleSession；角色、个人范围、当前对象、日常通道和权限均由后端解析，前端缓存与固定项目 cwd 不再是身份来源。
- Inbox、OpenLoop、standalone personal Todo、Notification、Reminder、DecisionRequest projection 与 DailyReport 由 M4 单写并跨重启恢复；OpenLoop 不自动克隆为 Todo。
- read、dismiss、snooze、close、reopen、carry-over 全生命周期有稳定状态机；用户明确 owner 动作通过原 source command 回写，协调状态不冒充业务完成。
- 首页提供可回源情境和持续 Secretary 对话；模型不可用时 deterministic brief/report 仍工作。
- scheduler、同窗幂等、catch-up、时区和失败恢复有机械测试；空事件窗口 invocation count 为 0。
- 旧右栏/秘书摘要/daily 读面完成 shadow、parity、compatibility read-only 与回切，不物理删除。
- 隔离调试 App 使用合成事件、隔离配置和假模型通过启动、强退、重启、deep link 与脱敏证据验收；不冒充真实日常使用。
- 完整离线回归、构建、独立审查、当前状态/计划/交接同步和 stage-06 收口完成。

允许动：

- docs/contracts/m4-secretary-attention-daily-resolution-v1.md
- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/
- handoffs/
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs
- prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_scheduler.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_acceptance.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/roleSessionReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/src/views/HomeView.tsx
- prototypes/productized-desktop-shell/src/views/
- prototypes/productized-desktop-shell/src/styles/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
- prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs
- prototypes/productized-desktop-shell/scripts/run-m4-isolated-app-acceptance.mjs
- prototypes/productized-desktop-shell/dist/
- prototypes/productized-desktop-shell/src-tauri/target/
- refs/heads/main
- /private/tmp/product-line-syn-m4-
- /private/tmp/syn-m4-acceptance-

只读：

- M1 四份冻结合同、M3 实施合同、M2/M3 已归档验收证据与当前普通产品源码
- /Users/yoyi/workspace/product-line-syn-fnd-002
- /Users/yoyi/workspace/product-line-syn-m2-closeout
- 真实 Codex SQLite / rollout、真实项目、账号和外部来源，只作结构边界参考，不读取正文或写入

不许动：

- M1 冻结合同正文、M3 已归档合同语义、manifest opening OID 与历史验收结论
- 两个保全工作树的 index、tracked/untracked 内容与分支头
- 真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号、凭据和外部 connector
- 网络外部写入、远端、push、merge、rebase、部署和发布
- reset、clean、stash、破坏性删除、覆盖既有工作和任务包写域外顺手修改
- M5-M10 产品实现；M4 只消费或交接 ProjectSummary、正式记忆/个人模型/Skill 与 connector refs

## 叶子

- [x] M4C01 M4 实施合同与当前事实纠偏
- [ ] M4C02 普通产品 Secretary RoleSession 与 PersonalScope 接线
- [ ] M4C03 持久 Inbox 与 Attention source projection
- [ ] M4C04 完整关注生命周期、个人待办与来源回写
- [ ] M4C05 Secretary 应用服务与持续协调上下文
- [ ] M4C06 首页情境与持续 Secretary 对话
- [ ] M4C07 DailyReport、scheduler 与空事件零模型
- [ ] M4C08 旧读面兼容迁移与回切
- [ ] M4C09 隔离产品应用分层验收
- [ ] M4C10 全量回归、独立验收与阶段收口
