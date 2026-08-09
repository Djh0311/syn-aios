# M3C04 ConversationTransportPort 与 fake provider 重启语义

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：把 start / continue / poll / stop / resume 收成只消费冻结 context、binding 与 grant 的适配端口，用确定性 fake provider 证明 terminal、timeout、cancel、stop receipt 和 restart 不重复发送。
干完的标准：adapter 不决定 scope、不直接推进业务状态；in-flight 有 durable receipt 才恢复，无证据时进入可见 orphan/failed；同一 idempotency key 不重复 provider effect。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m3_conversation_transport.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs

## 步骤

1. 冻结端口请求、响应、receipt 与 adapter 不变量。
2. 写 fake provider 的 start/poll/stop/resume、timeout、cancel 和 crash fixtures。
3. 接 repository，证明 effect 注册与 readback 幂等、restart 不静默重发。
4. 保留 legacy transport 为受守卫 adapter，不接真实 provider。
5. 跑聚焦测试、非测试构建和回归，独立提交。
