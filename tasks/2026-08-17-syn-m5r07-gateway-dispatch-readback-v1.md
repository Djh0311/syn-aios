# SYN-M5R07 最窄 correction：独立锚链 admission 与 Dispatch readback

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

本包接在 `37ff442` 之后，不 amend。关闭 gateway/Dispatch readback 五个直接 blocker：Grant 禁止自选 Plan；全部 admission 在首写之前且 Plan/Grant 权限 exact equality；Dispatch PENDING 后 Attempt 保持 GRANT_READY，readback PASS 后才 DISPATCHED；正式 runtime 删除 synthetic fail_cell；registry 登记真实 admission symbol。

不改 plan/current leaf/stage/auth，不 close M5/stage，不激活 M6。冻结合同正文、M1/M3、shared isolated constructor、commands/lib_read、M6 与既有 WIP 不动。

已知下一独立窄包：runtime 后 Attempt 未形成 terminal execution readback 却 claim 可接受 EXECUTED。本包不扩写、不冒充关闭。
