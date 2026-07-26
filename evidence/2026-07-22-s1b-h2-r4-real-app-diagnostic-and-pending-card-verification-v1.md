# S1B-H2-R4 真实 App 可归因对话与单张 Pending 卡验收 v1

日期：2026-07-22（+0800）  
任务包：tasks/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-package-v1.md

状态：R4-R2 已在发送首句前止损；新的 Gate 0/1 已通过，Gate 2 仅启动到 App 的启动期全语义对账，Gate 3/4 未执行；Gate 5 留有进程残留 blocker。

## Gate 0 已核对的安全事实

- R4 冻结的八个源码 SHA-256 全部匹配；HEAD 为 e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991；staged 集为空。既有脏项没有被清理、覆盖或改写。
- workflow-state 与 production SQLite、WAL/SHM 均无 holder；process registry entries 为 0；workflow-state 根目录没有 lock 文件。
- storage mode 仍为 DB-primary JSON projection；immutable read-only SQLite integrity_check 返回 ok。
- 已读取但未据此放行的基线为：recorded/injected/replied/diagnostic = 11/3/3/0，proposal/Pending/chain = 74/17/40，workflow revision = 289。这里的 Pending 对应持久状态 pending_user_confirmation。

## 阻塞事实

发现五个存活的 Node 进程，其 cwd 均为 productized-desktop-shell 源码目录；它们均始于 2026-07-17，且没有监听 TCP 端口，也没有持有 Workbench store。它们仍是 R4 Gate 0 要求清零的 scoped dev 残留，且无法在不读取完整 argv 或自行终止进程的前提下安全归属。

按任务包，不能把“未持有 store”替代“无 dev 残留”，也不能自行 kill。因此本轮以 BLOCKED_LIVE_HOLDER 停止。

## 未执行项

- 未运行 debug build，未冻结或启动任何 binary；
- 未启动真实 App，未发送任何用户消息；
- 未读取或写入私有 runner 原文、认证内容或用户正文；
- 未直接写真实 store，未产生新 canonical、diagnostic、proposal 或 Pending 卡；
- 未刷新卡片，未批准方案，未启动 chain 或 worker；
- 未修改固定测试项目，未 stage 或 commit。

## 恢复前提

由进程所有者正常关闭上述 scoped dev 残留后，R4 必须从新的 Gate 0 重新开始：重新检查 holder、八个源码 hash、真实 store/DB/JSON 基线、registry、fixed-project manifest，并重新 build/freeze binary。不得复用本次未完成 Gate 0 的 baseline 或旧 binary。

## R4-R2：全新 Gate 0、当前 binary 与发送前止损

本节是前述首次 Gate 0 止损后的独立现场轮次；没有复用其 holder 结论、store 基线或 binary。

### Gate 0（全绿）

- scoped App/dev/Codex/MCP/Vite/Cargo 残留、workflow-state/DB/WAL/SHM holder 与根 lock 均为空；registry `entries=0`。前轮五个 Node 残留均已不在。
- HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged 为空；既有未提交 H2/M5/文档工作线只冻结不清理。八个任务包源码 hash 均匹配：

  ```text
  552531eab5ae6f9beae7c857c6a438b8794dd52f9347e56052318232e3f509e8  supervisor_resident_oneshot_session.rs
  6c8ba3dbc0c38ad43a132651f216a35715120a1e82ab3ebc3208bdf97ee0da14  supervisor_resident_oneshot_tests.rs
  d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845  supervisor_orchestrator_resident_session.rs
  6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b  supervisor_orchestrator_submit_proposal.rs
  7f382cadf799f9dc6e4a34e86b22aca666d9bb8983dee717c235d85c2e03252e  workflow_read_model_entrypoints.rs
  47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2  useJiaobanConversationState.ts
  279a728ee6487f7e8afecf5e81ad4df1dccb06b2b68179fafc2444a5bce3cb92  knowledge_vault.rs
  d15bbdb16dee75dc415d1e4e050b275eb89e75a940f1a93c281e7845693519f0  workbench_sqlite_storage_mode_m5b_tests.rs
  ```

- 普通只读 SQLite `integrity_check=ok`；storage config 仍为 DB-primary JSON projection，确认的 JSON/DB 路径匹配。
- 计数级核心投影在 JSON 与 DB 均一致：`R0/I0/S0/D0=11/3/3/0`、`B0/P0/C0=74/17/40`；17 个主状态表、proposal/Pending、目标 supervisor session（generation `6`、thread tail `855fa3e0d87a`、无 active message）及 DB hash 列形状均一致。主 store/proposal/supervisor revision 分别为 `289/131/290`；DB import metadata revision `274` 是不同语义，未误作相等要求。initialized/degraded 计数为 `36/11`，后者是设计规定的 JSON-only 历史审计。
- 固定测试项目重新冻结：HEAD `caa02ded684d9e1d92d00c367949fab6f83430d1`；porcelain `14` 项、porcelain hash `15cef52ca18667cbeee677112bb682d55953659e7a03dbdcacd5c2de23e89a91`；全文件/业务 manifest 分别为 `f9c8867116851f688ee1311869c8703fd1f7f4f833cecd482eb42bb9115ad9a4` / `dd13eb1b5b01b68cfdfa88a5f4a2edb27c543a27ba3a02fe2d0e88c925a45250`。

计数级核验不能冒充 private `reconcile_db_vs_json` 的 natural-key/hash 全语义核验；该私有函数没有现成的纯只读 production command。这个限制在 Gate 2 启动时成为实际 blocker，见下。

### Gate 1（全绿）

- 在任务包规定目录运行 `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`，exit `0`；八个源码 hash 在 build 前后不变。
- 新裸 executable（不使用 `.app`）冻结为：`src-tauri/target/debug/codex-governance-workbench`，SHA-256 `2980c45e8a61b713eb029f32f71a51f693e6c7aae5756fd413ad570218d532b2`，大小 `66,548,968` bytes，mtime `1784702858`，Mach-O arm64。
- build 后重新核对 store hash、registry 和 holder；均没有因 build 漂移。

### Gate 2：启动期 blocker，首句未发送

- 只启动上述新冻结裸 executable；进入 UI 前，App 的启动期全语义 DB/JSON reconciliation 报出稳定家族 `db_json_reconciliation_not_green`，范围为 project-proposals 投影。未记录 proposal id、用户正文或原始错误。
- 这说明 Gate 0 的 count-level parity 不能替代 natural-key/hash 对账；没有证据可把它归因为 H2、MCP 或主管 transport，因此未猜测根因。
- 在任何 composer 动作前止损：没有新的 `client_request_id` / `message_id`，没有首句或第二句；R/I/S/D 保持 `11/3/3/0`，B/P/C 保持 `74/17/40`，没有 handler、tool receipt、主管 reply、Pending 卡、chain 或 worker 增量。
- 启动本身只新增两条正常 `storage_mode_initialized` 审计（JSON/DB 同由 `36` 到 `38`）；没有新增 `storage_mode_degraded_json_only`。主 store revision `289→291`、registry revision `1130→1132` 是该启动审计的正常伴随变化，不是用户业务写入。

### Gate 5：正常关闭后的现场卫生 blocker

- 通过 UI 发送正常 Quit；未使用 kill、信号终止或直接 store 写入。
- Quit 后两次只读 `ps`（间隔短暂收尾窗口）仍见本轮裸 executable PID `26611`，cwd 为 productized-desktop-shell；它没有 holder，registry 仍为 `entries=0`，但已构成未能清零的相关进程残留。
- 因此本轮最终裁决为 **`BLOCKED_LIVE_PROCESS_RESIDUAL`**。必须由进程所有者正常处理该残留后再进行任何新的现场动作；不得把“无 holder”误报为 Gate 5 全绿。
- 固定测试项目 Gate 5 hash 与 Gate 0 完全相同；未改项目、未点卡、未批准方案、未启动 chain/worker、未 stage/commit。

### 后续

下一入口不是重发，也不是现场修码：先处理进程残留，再执行只读的 `tasks/2026-07-22-s1b-h2-r4a-db-json-reconciliation-preflight-diagnosis-package-v1.md`。该包必须在新的现场授权下定位 project-proposals 的 natural-key/hash mismatch，并给出不扩大安全面或消息运输路的最小修复建议；R4 两句验收需另行重新 Gate 0。
