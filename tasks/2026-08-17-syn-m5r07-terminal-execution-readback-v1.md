# SYN-M5R07 最窄 correction：RecordExecutionAttemptReadback 与 EXECUTED claim terminal gate

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

本包接 `dfa97f0c261fc1946605c22cc4bc0059b7b6c6ae`（leaf-only；parent `1433d51466e59352cc8859e1c47f176da04f25b0` 已对 gateway/Dispatch readback 获独立 scoped PASS），不 amend。只实现内部 `RecordExecutionAttemptReadback` + EXECUTED claim terminal gate。不新增 Tauri command。ordinary M1 composition 明确排除。

不改 plan/current leaf 生命周期/stage/auth，不 close M5/stage，不激活 M6。冻结合同正文、ordinary M1 注册/迁移、M3、M6、shared isolated constructor、前端、Harness report/receipts/evidence 不动。

范围：runtime 返回后由 `complete_dispatch_readback` 的 post-dispatch Attempt revision 驱动首次 execution readback；同一 UoW 写 immutable readback 表、command receipt、event、SCRUBBED_ATTEMPT_RECORD audit 与 Attempt CAS；EXECUTED claim 在 report-hash replay 或 INSERT 前必须加载持久 readback；产品命令从权威 readback 派生 status，禁止硬编码 SUCCEEDED。terminal WorkflowRun/WorkItem、UI RUNNING、receipt 签名/trace、跨进程双写、旧 claim 全量迁移不在本包。
