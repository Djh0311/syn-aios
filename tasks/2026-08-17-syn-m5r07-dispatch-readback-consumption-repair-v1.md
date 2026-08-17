# SYN-M5R07 最窄 correction：Dispatch readback 同事务复核与 capability 单次消费

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

本包接在 `071d202` 之上，不 amend、不重做既有 WIP。关闭独立 live review 四个直接 blocker：readback 先 BEGIN IMMEDIATE 再同事务 load/join/assert/origin/write；删除 exact_one 全局计数并按本链 deterministic ID + exact payload 校验；replay 逐字段比对；opaque capability 不可 Clone、按值单次消费，consumption now 复核 Grant ACTIVE。

不改 plan/current leaf/stage/auth，不 close M5/stage，不激活 M6。冻结合同正文、M1/M3/M6、Harness 与既有非 M5 WIP 不动。terminal execution readback 仍下一包。
