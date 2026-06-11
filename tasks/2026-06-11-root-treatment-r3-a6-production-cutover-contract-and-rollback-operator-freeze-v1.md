# Root Treatment / R3-A6 Production Cutover Contract And Rollback Operator Freeze v1

日期：2026-06-11

状态：已完成，待主管 checkpoint 同步。本文是 Root Treatment / Stage R 的 R3-A6 任务包，用于在 R3-A1 到 R3-A5 fixture-only SQLite rehearsal 之后，冻结生产路径前置门槛、cutover contract、rollback operator contract、allowed roots / denied paths、backup / recovery 和 dry-run / apply 分界。R3-A6 默认不实现生产 DB、不迁移真实 JSON / sidecar、不切真实产品读写路径、不停写 JSON / sidecar。合同见 `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`；记录见 `evidence/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1.md` 与 `handoffs/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1-result.md`。

## 0. 全局主管理解

已知事实：

- R3-P0 已冻结 SQLite schema / importer / rollback 合同。
- R3-A1 已完成 schema file、temp DB initializer、idempotent dry-run importer 和 fixtures。
- R3-A2 已完成 temp DB apply importer、schema hardening、transaction failure injection 和 DB -> JSON export dry-run。
- R3-A3 已完成 fixture-only dual-write transaction rehearsal。
- R3-A4 已完成 fixture-only read-cut DB / JSON fallback / rollback recovery dry-run rehearsal。
- R3-A5 已完成 fixture-only observation / export / rollback verification rehearsal，implementation commit：`0e8255a8248601caf7b1d513131f43e4bb157589`，主管 checkpoint：`evidence/2026-06-11-root-treatment-r3-a5-supervisor-checkpoint-v1.md`。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3-A6 仍不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
R3-A6 是生产路径前的合同冻结和门禁任务，不是生产迁移任务。
```

R3-A6 必须回答：

- 哪些数据根、文件、表和路径允许进入后续生产前置演练。
- 哪些路径仍必须停留在 fixture-only。
- production DB 文件名、位置、备份、锁、事务、回滚和导出合同如何定义。
- dry-run、apply、read-cut、stop-write 和 rollback operator 的边界如何区分。
- 哪些验收证据必须先存在，才能进入后续生产 DB / 双写 / 读切任务包。

## 1. Execution Mode

Execution Mode：Supervisor-led contract freeze，默认单线执行。

Multi-Agent Policy：

- 本任务只写合同 / 任务包 / evidence / handoff，不需要开发线实现代码。
- 如需要架构复核线，可只读复核合同，不改文件、不提交。
- 全局主管保留最终写入、验证、commit 和入口同步权。

Fallback If Scope Expands：

- 若发现必须修改生产 store、Rust storage module、Tauri command、app startup 或真实数据路径，停止 R3-A6，拆出新的 R3-A7 / production preflight implementation 任务包。

## 2. Model Routing

Assigned Model：strongest available if runtime supports it; otherwise runtime default。

Reasoning Effort：high / xhigh。

Reason For Choice：

- R3-A6 是从 fixture-only rehearsal 转向生产前置工作的安全门槛；合同写错会导致后续生产迁移、回滚和多 agent 解锁边界混乱。

Escalation Trigger：

- 任何涉及生产 DB 创建、真实数据迁移、真实 JSON / sidecar 改写、JSON / sidecar 停写、产品路径读切、`.codex`、secret / transcript、真实 Codex 执行、Stage L / K3-B1 / K3-B2 或 backlog 功能解冻。

## 3. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a2-supervisor-checkpoint-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a3-supervisor-checkpoint-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a4-supervisor-checkpoint-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a5-supervisor-checkpoint-v1.md`

建议读取的代码 / 合同：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `scripts/harness/workbench-shape-gate.js`

## 4. 目标

R3-A6 必须产出一份足够支撑后续开发的生产切换合同文档，建议路径：

- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`

合同必须至少包含：

1. Production Data Roots
   - 生产 workflow state root 如何定位。
   - production DB 默认文件名 / 目录。
   - export root / backup root / rollback manifest root。
   - allowed roots 和 denied paths。
   - `/Users/yoyi/.codex` 明确 denied，除非未来单独任务包明确最小范围、备份和授权。

2. Mode Contract
   - `dry_run`：只读 JSON / sidecar，写 temp report / evidence。
   - `production_preflight`：允许读取生产 JSON / sidecar 元数据和 hash，仍不写 production DB。
   - `production_apply`：后续任务包才允许，R3-A6 不授权。
   - `read_cut`：后续任务包才允许，R3-A6 不授权。
   - `stop_write_json`：后续任务包才允许，R3-A6 不授权。
   - `rollback_operator`：R3-A6 只定义合同，不执行恢复。

3. Backup / Recovery Contract
   - production apply 前必须有 git commit、DB backup、JSON / sidecar backup、source root hash、manifest hash。
   - backup 保留策略和恢复顺序。
   - corrupt DB、schema mismatch、partial import、projection hash mismatch、rollback manifest missing / incomplete 时的处置。
   - 失败状态不得冒充 stable / completed。

4. Transaction / Lock Contract
   - production apply 必须单事务。
   - JSON / sidecar 写入锁与 SQLite transaction 的顺序。
   - crash / interruption failure injection 要求。
   - revision conflict / corrupt JSON / lock busy 不得覆盖原文件。

5. Read-Cut / Stop-Write Gates
   - 进入 read-cut 前必须满足哪些 A1-A5 evidence。
   - 进入 stop-write JSON 前必须满足哪些 observation period 和 export verification。
   - stop-write JSON 后 JSON 只能作为 export / backup / rollback projection，而不是主写路径。
   - readback、runtime log、audit、memory adoption 等关键链路必须逐项验证。

6. Rollback Operator Contract
   - who / actor：只能 supervisor decision。
   - input：last verified JSON projection、DB backup、rollback manifest、source root hash、export hash。
   - output：restored JSON projection / disabled DB read-cut / preserved DB for audit / rollback evidence。
   - production restore performed flag。
   - 不允许自动 rollback 删除用户真实数据。

7. Evidence / Handoff Contract
   - 每个后续生产任务包必须记录 start commit、end commit、DB path hash、source root hash、backup refs、manifest hash、allowed roots、denied paths、tests、rollback drill result。
   - 每个 failed / partial / blocked 必须写清不可声明项。

8. Task Split Recommendation
   - R3-A7：production preflight dry-run scanner / report，只读生产 JSON / sidecar hash，不建 production DB。
   - R3-A8：production DB initializer + import apply dry-run on copied temp production snapshot，不写真实 production root。
   - R3-A9：production DB apply with backup and rollback manifest，仍不 read-cut。
   - R3-A10：limited read-cut for one low-risk read model behind feature flag / fallback。
   - R3-A11：observation period and export verification on production path。
   - R3-A12：stop-write JSON decision and rollback drill。
   - R3-A13：transaction acceptance across memory + audit and R3 final acceptance。

## 5. 允许读取

允许读取：

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本、git 元数据。
- R3-P0 / R3-A1 / R3-A2 / R3-A3 / R3-A4 / R3-A5 的 evidence / handoff / supervisor checkpoint。
- 当前仓库内 fixtures。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目数据内容。R3-A6 是合同冻结；如果需要真实生产 root 信息，只能写成“后续生产 preflight 任务再读取 hash / metadata”，本轮不读取。

## 6. 允许写入

允许写入：

- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a6-production-cutover-contract-and-rollback-operator-freeze-v1-result.md`
- 本任务包状态更新。

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 7. 禁止事项

R3-A6 禁止：

- 不创建生产 DB。
- 不写用户真实数据目录。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不切任何产品读写路径到 DB。
- 不让真实 app read model 读 DB。
- 不停止 JSON / sidecar 写入。
- 不把 JSON 降为生产 fallback。
- 不在 app startup / Tauri command / UI 中接入生产 SQLite。
- 不改 workflow state 顶层 schema。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不新增 Tauri command。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不把 R3-A6 合同冻结冒充为生产迁移、R3 完成或多 agent 并行真实执行解锁。

## 8. 形状影响

- 任务类型：治理任务包 / production cutover contract freeze。
- 新增代码落点：无。
- 触碰棘轮文件：无。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`1caf4f9bd30b976f0c909f5e99db1968c293a285`。
- 本任务完成 commit：待主管线回收后记录。

## 9. 验收标准

R3-A6 可接受为：

- Production cutover contract 已冻结。
- Rollback operator contract 已冻结。
- allowed roots / denied paths 已明确。
- dry-run / production preflight / production apply / read-cut / stop-write / rollback mode 分界清楚。
- backup / recovery / manifest / hash / evidence 要求清楚。
- 后续 R3-A7 到 R3-A13 或等价任务拆分建议清楚。
- evidence / handoff 记录 start commit、end commit 或未提交状态、验证结果、P0/P1/P2 和禁止项确认。

R3-A6 不接受为：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。

## 10. 建议验证命令

必须跑：

```bash
git diff --check
git status --short
rg -n "R3-A6|production cutover|rollback operator|生产路径|停写 JSON|多 agent 并行真实执行" docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md
rg -n "生产 DB 创建完成|生产读切 DB 完成|JSON / sidecar 停写已完成|多 agent 并行真实执行解锁" CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md || true
```

如本轮只改文档，不需要跑 `cargo test` / `npm`；如任何源码或脚本被改动，必须补对应测试和 shape gate。

## 11. Evidence / Handoff 结构

Evidence 必须包含：

1. STATUS。
2. READ / WRITE SCOPE。
3. CONTRACT SUMMARY。
4. GATE MATRIX。
5. NEXT TASK SPLIT。
6. CHECKS RUN。
7. P0 / P1 / P2。
8. BOUNDARY CONFIRMATION。
9. DO NOT CLAIM。

Handoff 必须包含：

1. STATUS。
2. CONTRACT PATH。
3. NEXT RECOMMENDED TASK。
4. HARD BOUNDARIES。
5. WHAT NOT TO CLAIM。

## 12. 主管回收标准

主管线必须独立检查：

- 合同是否足够支撑 R3-A7 之后开发。
- 是否意外授权生产 DB / read-cut / stop-write。
- 是否遗漏 backup / rollback / evidence / allowed roots / denied paths。
- 是否保留 R3 未完成、多 agent 未解锁、Stage L/K 未恢复的边界。
- 入口文档是否只在 checkpoint 同步。

## 13. Do Not Claim

完成 R3-A6 后仍不得声明：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
