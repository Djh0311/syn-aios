# SYN-M5R07 最窄 correction：Dispatch readback 同事务复核与 capability 单次消费

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

本包接在 `071d202` 之上，不 amend、不重做既有 WIP。关闭独立 live review 四个直接 blocker：readback 先 BEGIN IMMEDIATE 再同事务 load/join/assert/origin/write；删除 exact_one 全局计数并按本链 deterministic ID + exact payload 校验；replay 逐字段比对；opaque capability 不可 Clone、按值单次消费，consumption now 复核 Grant ACTIVE。

不改 plan/current leaf/stage/auth，不 close M5/stage，不激活 M6。冻结合同正文、M1/M3/M6、Harness 与既有非 M5 WIP 不动。terminal execution readback 仍下一包。

2026-08-17 第三组 scoped implementation correction（接 `85b1abc`，不 amend）：

1. `run_admitted_workcell` 在任何 DurableOperation/execute 前，除 stored chain/token/Dispatch=DISPATCHED/Attempt=DISPATCHED 外，要求对应 outbox status=DELIVERED，并调用同一套 exact carrier verifier 证明 RecordDispatchReadback 与 MarkAttemptDispatched 两组 receipt/event/audit 均存在且内容 exact。
2. BEGIN 后首写前 outbox 必须与原 DispatchGrantedAttempt receipt exact 关联；两条 readback command receipt 的 policy_decision_ref 从 grant 权威链派生，禁止 pol-m5r02；current_object_ref/result_ref/committed_revision/时间绑定真实 Dispatch/Attempt 与 post-transition revision；events 绑定具体对象与真实 revision；exact replay loaders 比较全部持久列；未知 enum fail closed。
3. `m5_agent_runtime::run_conformance_suite` 仅测试使用；M5-SE-002 继续 exact `run_m5_authorized_runtime_with_state`；旧 raw workcell / isolated followthrough 仅 `#[cfg(test)]`。
