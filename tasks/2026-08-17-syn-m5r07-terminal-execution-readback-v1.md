# SYN-M5R07 最窄 correction：RecordExecutionAttemptReadback 与 EXECUTED claim terminal gate

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: AWAITING_INDEPENDENT_ACCEPTANCE

本包接 `dfa97f0c261fc1946605c22cc4bc0059b7b6c6ae`（leaf-only；parent `1433d51466e59352cc8859e1c47f176da04f25b0` 已对 gateway/Dispatch readback 获独立 scoped PASS），不 amend。只实现内部 `RecordExecutionAttemptReadback` + EXECUTED claim terminal gate。不新增 Tauri command。ordinary M1 composition 明确排除。

不改 plan/current leaf 生命周期/stage/auth，不 close M5/stage，不激活 M6。冻结合同正文、ordinary M1 注册/迁移、M3、M6、shared isolated constructor、前端、Harness report/receipts/evidence 不动。

范围：runtime 返回后由 `complete_dispatch_readback` 的 post-dispatch Attempt revision 驱动首次 execution readback；同一 UoW 写 immutable readback 表、command receipt、event、SCRUBBED_ATTEMPT_RECORD audit 与 Attempt CAS；EXECUTED claim 在 report-hash replay 或 INSERT 前必须加载持久 readback；产品命令从权威 readback 派生 status，禁止硬编码 SUCCEEDED。terminal WorkflowRun/WorkItem、UI RUNNING、receipt 签名/trace、跨进程双写、旧 claim 全量迁移不在本包。

## R01 独立返修事实

独立 exact archive `/tmp/syn-m5-terminal-f2b4ec1-5Vlcjv` 对 `f2b4ec1` 复现：正向 execution_readback 8/0，但错误 replay revision 成功、空 trace 成功、unknown durable state 伪装 FAILED、错 project durable op 成功、篡 persisted readback trace 后 carrier assert 成功。本 child 只修这 6 组已独立复现的冻结合同 direct blockers，不 amend `f2b4ec1`，不扩 authenticated_actor owner。

1. `first_record_execution_attempt_readback` 同一 UoW、任何写前调用既有 production `IndependentRuntimeReceiptVerifier`。空 trace/effect 与既有 verifier 拒绝项不能落权威 readback。
2. `m5_controlled_execution` `map_op` 去掉 `parse(...).unwrap_or(Failed)`。`load_operation` / `load_operation_by_effect` 对未知 state 传播稳定 `unknown_op_state`；FAILED receipt 不可靠伪装终态化。只改这一直接 load 语义。
3. EXECUTED 在 hash replay / INSERT 前调用强化后的 `assert_execution_attempt_readback_carriers`（含 self-integrity）。`ExecutionReceipt.execution_id` exact `persisted.receipt_id`，`output_hash` exact `persisted.trace_hash`。缺/篡 carrier 或 embedded receipt 零 claim。
4. `assert_execution_readback_chain_exact` 覆盖现成 project / orchestration / workflow_run / work_item / node、authorization id/revision、grant/attempt/dispatch/binding/outbox joins；durable op 的 project / orchestration / workflow_run 也 exact。复用 `load_joined_dispatch_chain` 与已有字段，不新造 owner。
5. existing replay 必须接收 `expected_attempt_revision` 并 exact 等于 `existing.source_attempt_revision`。任何 carrier assert / claim 消费前先验证 `source+1==committed`，从 persisted `RuntimeReceipt` 全部字段重算 derived state 与 canonical hash 并和 record exact，再验 Attempt state/revision 与 formal carriers。divergent 零写。已提交历史不因后来 Grant expiry 失败。
6. 同 `report_hash` 命中 existing 前已完成 terminal/readback gate；再 exact 比较本次 validated report/chain 与 existing 的 project / orch / run / item / node / dispatch / attempt / grant / worker session / authoritative receipt / authenticated actor / claim status。只扩 `ClaimRecord` loader 到这些字段。完全相同才幂等返回，否则稳定 `claim_report_hash_divergent` 且零新写。不迁移旧 claim 全库。
