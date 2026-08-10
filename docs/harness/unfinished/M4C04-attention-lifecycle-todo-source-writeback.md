# M4C04 完整关注生命周期、个人待办与来源回写

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：完成 read/dismiss/snooze/close/reopen/carry-over、Notification/Reminder 和用户明确 standalone personal Todo；owner 动作只通过原 source command 回写。
干完的标准：生命周期跨重启、CAS/幂等和审计通过；ack/close 不改变 owner 事实；OpenLoop 不自动克隆 Todo；用户明确创建 Todo 才产生 PersonalAction；source writeback 成功/失败回执可回源。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/tests/ [新增]
- docs/harness/

## 步骤

1. 写状态机、CAS、重放、OpenLoop/Todo 分型和 owner 不变测试。
2. 实现 coordination commands、Reminder/Notification/PersonalAction 与 receipts。
3. 实现 source-owner command port 和 fail-closed 回写，不缓存可执行 pending payload。
4. 跑聚焦/故障注入/重启测试，独立审查后精确提交并归档。
