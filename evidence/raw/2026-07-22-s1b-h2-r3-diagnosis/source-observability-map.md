# S1B-H2-R3 源码边界与可观测性图（只读）

冻结源码锚点：

1. `supervisor_resident_oneshot_session.rs:1633-1660` 先以既有 Batch 2 路写 canonical recorded，身份为 message/client/target_ref。
2. `:1975-1995` 随后调用 consult；任意 `Err` 被统一折叠为 `message_recorded_supervisor_incomplete`，没有 message-scoped error family 或 terminal audit。
3. `:555-680` 在 prepared 前有多个可失败门：stale-session reaper / reconciliation load、session load、executable、resume 或 initial command plan、private home、initial facts。
4. `:917-992` runner 先尝试输出目录与 stderr 初始化，再 spawn，随后写 prepared，最后才登记 process group。
5. `:710-760` durable binding 发生在 `thread.started` 的 callback；`:769-887` 才记录 turn exit；`:1997-2040` 成功 turn 后才写 injected 与 supervisor reply。
6. `exec_process_registry.rs:395-404` 的正常 unregister 无历史 audit；空 registry 不能反证历史 spawn。

当前缺口：prepared 是 message 到 run 的第一个持久桥（含 active_message_id、PID、generation）。在其之前，现有代码没有 `consult_started` 或 `consult_failed {message_id, stable_family}`；入口又吞掉完整错误。因此 `recorded + no prepared` 只能限定区间，不能从存量状态精确选择某一个 preflight 分支或 output-directory 创建失败。
