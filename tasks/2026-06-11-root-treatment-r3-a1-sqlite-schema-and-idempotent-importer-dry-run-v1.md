# Root Treatment / R3-A1 SQLite Schema And Idempotent Importer Dry Run v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R3-A1 任务包，用于按 R3-P0 合同实现最小 SQLite schema file、临时 DB initializer、idempotent dry-run importer 和专用 fixtures。

本任务只做 schema / dry-run importer 的开发准备和离线验证；不创建生产 DB，不迁移真实 JSON / sidecar，不双写，不切 DB 读路径，不改 workflow state 顶层 schema，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review 已完成。
- R3-P0 SQLite schema / importer / rollback contract freeze 已完成，completion commit：`7022f03d20c77c56a84e9cc9bd2b32aca9b786e6`。
- R3-P0 合同文档为 `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`。
- 当前 checkpoint commit：`5924ea9945fb0707a50053a93c458c1995cafc1f`。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3-A1 仍不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
先用临时 DB 和 fixture 证明 schema 可初始化、importer 可 dry-run、幂等 / corrupt / forbidden field 分类可测，再考虑后续 apply importer 或双写。
```

## 1. Execution Mode

Execution Mode：Multi-agent exception, sequential implementation line。

Multi-Agent Justification：

- R3-A1 涉及 DB/schema/importer/data-integrity 风险，适合由现有 Stage R 开发线承担实现，主管线保留集成和验收权。
- 写入范围虽集中，但实现需要比入口同步更长的上下文和测试循环；复用现有 Stage R 开发线可以减少新建对话和上下文维护成本。
- 本任务不得并行多写同一文件；若需要复核线，必须在实现线回交后由主管线单独派发只读复核。

Coordination Cost：

- 只派发一条开发线；不拆成多个 worker。
- 入口文档默认不在实现线同步，避免每个小步都维护权威入口。

Fallback If Coordination Fails：

- 若开发线返回 `NEEDS_DECISION` / `BLOCKED` / 越界读写，主管线停止集成，回到单线审查并重写任务包边界。

## 2. Model Routing

Assigned Model：strongest available if runtime supports it; otherwise runtime default。

Reasoning Effort：high / xhigh。

Reason For Choice：

- 本任务涉及 SQLite schema、importer 幂等、敏感字段过滤、fixture 分类和后续迁移硬门槛，错误成本高于普通文档或 UI 修补。

Escalation Trigger：

- 任何涉及生产 DB、真实数据迁移、读写路径切换、`.codex`、secret / transcript、schema 合同变更、写入范围扩张、测试失败无法解释、或 Product Command / continuation / runtime log transaction 边界不清的情况。

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
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`
- `evidence/2026-06-11-root-treatment-r3-p0-completion-authority-sync-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-p0-completion-authority-sync-supervisor-checkpoint-v1-result.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `scripts/harness/workbench-shape-gate.js`

## 4. 目标

R3-A1 必须完成：

- 新增最小 SQLite schema 文件或 Rust schema module，包含 R3-P0 合同中 metadata / source、workflow、memory / observation、workflow governance、runtime / continuation / product command 五组核心表的第一版 DDL。
- 新增临时 DB initializer，只能创建调用方显式传入路径的临时 / 测试 DB；不得推导或写入生产数据目录。
- 新增 dry-run importer module，可读取 fixture 目录中的 `workflow-state.v0.json` 和允许的 sidecar 文件，输出 deterministic dry-run report。
- Dry-run importer 必须不写源 JSON / sidecar，不写生产 DB，不修改 workflow state，不创建 runtime side effects。
- Dry-run report 至少包含：
  - batch id 或 deterministic fixture run id。
  - mode：`dry_run`。
  - source file inventory。
  - accepted / missing_optional / rejected_corrupt / rejected_unknown / rejected_sensitive / skipped_duplicate 分类。
  - proposed inserts / updates / skips / conflicts / warnings counts。
  - source hash / record hash / natural key 摘要。
- 实现幂等 dry-run：同一 fixture 连续运行两次，报告中相同 natural key + same hash 必须分类为 deterministic skip 或 unchanged，不能变成冲突。
- 实现 forbidden-field fixture：prompt body、secret、token、credential、full transcript、rollout body 等禁止字段必须被分类为 `rejected_sensitive` 或 batch reject，不能进入 proposed inserts。
- 实现 alias fixture：`runtime-log.v1.json` / `runtime-logs.v1.json` 必须按合同识别 plural canonical、singular legacy alias / ref label。
- 新增专用 fixtures，覆盖 R3-P0 合同的最小 fixture 矩阵。
- 新增 focused Rust tests。
- 写 R3-A1 evidence / handoff。

## 5. 允许读取

允许读取：

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本、git 元数据。
- R0 / R1 / R2 / R3-P0 的 evidence / handoff / supervisor checkpoint。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目数据，除非它已经作为本仓库内测试 fixture 明确存在。

## 6. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅允许增加 `mod workbench_sqlite_schema;` / `mod workbench_sqlite_importer;` 之类模块声明，不允许新增 `#[tauri::command]` 或 app startup hook。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1/**`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1-result.md`

可选写入：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_fixture.rs`，仅在 fixture helper 明显需要独立文件且低于 500 行时允许。

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 7. 禁止事项

R3-A1 禁止：

- 不创建生产 DB。
- 不写用户真实数据目录。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不双写 DB + JSON。
- 不切任何产品读路径到 DB。
- 不在 app startup / Tauri command / UI 中接入 DB initializer 或 importer。
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
- 不把 schema file、temp DB initializer 或 dry-run importer 冒充为 R3 迁移开始 / 完成。
- 不夹带 R2 inline tests 巨石迁移。
- 不夹带 R4 前端按页读模型或 UI 瘦身。

## 8. 形状影响

- 任务类型：治理任务包 / SQLite contract implementation prep。
- 新增代码落点：`workbench_sqlite_schema.rs`、`workbench_sqlite_importer.rs`。
- 新增 fixture 落点：`src-tauri/fixtures/r3-a1/**`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1-3 行 module declaration；不得增长业务逻辑。
- 预计行数变化：
  - `workbench_sqlite_schema.rs` 预计 250-800 行，必须低于 3,000 行。
  - `workbench_sqlite_importer.rs` 预计 500-1,500 行，必须低于 3,000 行。
  - fixture helper 若新增，必须低于 500 行。
  - `lib.rs` 预计增加不超过 3 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`5924ea9945fb0707a50053a93c458c1995cafc1f`。
- 本任务完成 commit：待主管线回收后记录。

## 9. Fixture 矩阵

必须至少新增并测试：

- `valid-empty-workflow`：最小合法 `workflow-state.v0.json`。
- `valid-workflow-core`：project / workflow / node / edge / work item / artifact / review / audit。
- `memory-adoption-chain`：formal memory + candidate + observation adoption refs。
- `memory-capture-chain`：capture event -> observation -> candidate。
- `proposal-authorization-chain`：proposal + user decision + authorization。
- `process-fact-observation`：process fact decision -> observation。
- `product-command-runtime-chain`：product command + continuation + runtime log + readback summary。
- `runtime-log-alias`：plural canonical + singular legacy alias。
- `corrupt-primary`：primary workflow state corrupt，batch reject。
- `corrupt-optional-sidecar`：optional sidecar corrupt，source reject / warning。
- `duplicate-same-hash`：same natural key + same hash，idempotent skip / unchanged。
- `duplicate-different-hash`：same natural key + different record hash，conflict。
- `revision-conflict`：revision mismatch，conflict。
- `unknown-sidecar`：unknown `.v1.json` sidecar，rejected_unknown。
- `forbidden-sensitive-field`：prompt body / secret / full transcript / rollout body，rejected_sensitive。

如果为了控制工作量需要合并 fixture，必须在 evidence 中说明每个要求由哪个 fixture 覆盖。

## 10. 验收标准

R3-A1 可接受为：

- SQLite schema DDL constants 或 schema module 已实现，并能初始化临时 DB。
- Dry-run importer 能读取专用 fixture 目录并输出 deterministic report。
- 幂等、corrupt、missing、unknown、alias、forbidden sensitive field 分类均有测试覆盖。
- Source hash / record hash / natural key 进入 dry-run report。
- Importer 不覆盖源 JSON / sidecar。
- 未创建生产 DB。
- 未迁移真实数据。
- 未新增 Tauri command。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 记录 start commit、end commit 或未提交状态、写入范围、验证结果、P0/P1/P2 和禁止项确认。

R3-A1 不接受为：

- R3 SQLite 迁移开始或完成。
- apply importer 完成。
- 双写期开始。
- 读切 DB 完成。
- JSON / sidecar 停写。
- production DB 创建完成。
- transaction boundary 全部实现完成。
- DB -> JSON export / rollback 实现完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。

## 11. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_schema
cargo test --lib sqlite_importer_dry_run
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

如果 filtered cargo tests 没有匹配测试，必须记录 exact no-match 结果，并通过实际存在的 module / test name 或更广泛 `cargo test --lib` 覆盖；不得把 no-match 冒充通过。

建议补充扫描：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1
rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations' prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs
```

## 12. 必须回传

开发线回传必须包含：

1. STATUS：`DONE` / `DONE_WITH_CONCERNS` / `NEEDS_DECISION` / `BLOCKED`。
2. 读了哪些文件。
3. 写了哪些文件。
4. schema module 摘要。
5. temp DB initializer 摘要。
6. dry-run importer / idempotency 摘要。
7. fixture 覆盖矩阵。
8. forbidden sensitive field 处理摘要。
9. runtime-log alias 处理摘要。
10. 运行了哪些检查，结果是什么。
11. start commit / end commit 或未提交说明。
12. P0 / P1 / P2。
13. 是否触碰任何禁止项。

## 13. 主管回收标准

主管线回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0 示例：

- 创建生产 DB 或写用户真实数据。
- 读写 `/Users/yoyi/.codex`。
- 执行真实 Codex。
- 修改真实 workflow state / sidecar。
- 切产品读写路径到 DB。
- prompt body / secret / full transcript 被 importer 接受为 proposed insert。
- cargo 编译失败且未阻断。

P1 示例：

- 幂等 duplicate-different-hash 被静默覆盖。
- unknown sidecar 没有拒绝或没有 supervisor decision 标记。
- runtime-log singular/plural alias 处理与 R3-P0 合同不一致。
- dry-run report 不含 source hash / natural key。
- filtered tests no-match 但回报为通过。

P2 示例：

- schema v0 表覆盖仍偏 coarse，需要 R3-A2 细化。
- importer dry-run 尚未实现 apply mode。
- export / rollback 仍只是合同，未实现。
- Product Command / continuation / runtime log transaction 仍待 R3 后续实现。
