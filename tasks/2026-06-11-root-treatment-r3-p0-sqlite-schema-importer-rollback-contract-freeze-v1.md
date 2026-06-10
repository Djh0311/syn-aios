# Root Treatment / R3-P0 SQLite Schema Importer Rollback Contract Freeze v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R3-P0 任务包，用于在 R2 closing / R3 preflight review 后冻结 SQLite 统一存储的 schema、importer、幂等键、回滚 / 导出和 fixture 合同。

本任务只做合同冻结和任务准备，不创建 SQLite schema，不迁移真实数据，不改产品源码，不改 workflow state JSON，不新增 sidecar，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1、R2-B1 到 R2-B10 已完成并 checkpoint。
- R2-B10 后 `lib.rs` 为 13,949 行，已低于 R2 第一阶段 `<= 15,000` 水位线，但 R2 仍未全部完成。
- R2 closing / R3 preflight review commit：`126ee5d47e1b17c540e4e2f8e961198f3ffeceb6`。
- R2 closing / R3 preflight review 结论为 `DONE_WITH_CONCERNS`：建议先做 `R3-P0 SQLite schema / importer / rollback contract freeze`，不要直接迁移写路径。
- Rust 依赖中已经存在 `rusqlite = { version = "0.32", features = ["bundled"] }`，但当前主要用于只读读取 Codex 原生 sqlite，不等于工作台 workflow / sidecar 统一存储已存在。
- 当前生产 sidecar JSON kinds 为 14 detected / 0 unknown；workflow state 和多套 sidecar store 已有不同程度的 StoreLock / corrupt guard / revision guard / atomic write / backup 策略，但不统一。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3 未完成前不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
先冻结 R3 的 schema / importer / transaction / rollback 合同，再允许进入实际 schema 或 importer 实现。
```

## 1. 权威依据

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
- `evidence/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1-result.md`
- `evidence/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1-result.md`

建议读取的代码和脚本：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `scripts/harness/workbench-shape-gate.js`

## 2. 目标

本任务必须完成：

- 新增 R3 SQLite schema / importer / rollback 合同文档。
- 明确 R3 schema v0 的表域分组、主键 / natural key 策略、引用关系、审计 / runtime / memory / workflow / continuation / product command 的归属边界。
- 明确 JSON / sidecar importer 输入范围、导入顺序、幂等键、source hash、重复导入策略和 corrupt / missing / revision conflict 处理方式。
- 明确 R3 transaction boundary，至少覆盖：
  - candidate -> formal memory + memory audit。
  - observation -> candidate。
  - proposal confirmation -> authorization creation。
  - global boundary review -> authorization activation。
  - process fact decision -> observation。
  - product command Phase A/B trace -> continuation -> runtime log。
- 明确 DB -> JSON export / rollback 策略，确保旧 JSON / sidecar 观察期内可回退。
- 明确 crash injection / rollback / fixture 测试矩阵。
- 输出下一步 R3-A1 最小 schema + importer dry-run 任务建议，但不创建 R3-A1 任务包也可以；若创建，只能是草案任务包，不改产品源码。
- 输出 R3-P0 evidence / handoff。

本任务不要求改代码，不要求创建数据库，不要求运行 cargo / npm。

## 3. 允许读取

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本和 git 元数据。
- R0/R1/R2-B1 到 R2-B10 以及 R2 closing / R3 preflight review 的 evidence / handoff / supervisor checkpoint。

## 4. 允许写入

允许写入：

- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`

可选写入：

- `tasks/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`

默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或正式计划；入口文档由主管线 checkpoint 统一处理。

## 5. 禁止事项

本任务禁止：

- 不改产品源码。
- 不创建 SQLite schema 实现。
- 不新增 Rust storage module。
- 不新增 migration 文件。
- 不导入真实用户数据。
- 不迁移 SQLite。
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
- 不把 schema 合同冻结冒充为 R3 SQLite 迁移开始或完成。
- 不把已有 `rusqlite` 依赖冒充为工作台统一存储已实现。
- 不把 R3-P0 结果说成多 agent 并行真实执行已解锁。

## 6. 形状影响

- 任务类型：治理任务包 / contract freeze。
- 新增代码落点：无产品代码新增；新增 docs / evidence / handoff。
- 是否触碰棘轮文件：否。
- 预计行数变化：无产品源码变化；新增文档约 300-600 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`126ee5d47e1b17c540e4e2f8e961198f3ffeceb6`。
- 本任务完成 commit：待完成后记录。

## 7. 合同文档必须包含

`docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md` 至少包含：

1. R3 不变量：
   - SQLite 是工作台自有事实库，不是 Codex 原生 `.codex/state_*.sqlite`。
   - prompt body / secret / token /完整 transcript 不进入 DB。
   - JSON / sidecar 观察期内保留可导出 / 可回滚。
   - R3 未收口前不解锁多 agent 并行真实执行。

2. Store inventory：
   - `workflow-state.v0.json`
   - formal memory / candidate / observation / lint / entity relation / mature pattern / capture bus
   - plan authorization / proposal / C4-C6 workflow arrays
   - product command / session continuation / runtime logs
   - blackboard candidates
   - `runtime-log.v1.json` 与 `runtime-logs.v1.json` 的 canonical / legacy alias 策略

3. Schema v0 proposal：
   - 表域分组。
   - 每组表候选。
   - natural key / foreign key 策略。
   - audit / runtime / readback / source ref / evidence ref 归属。
   - redaction 和 sensitive fields policy。

4. Importer contract：
   - 输入目录和文件清单。
   - source hash / import batch / dry-run 输出。
   - 幂等键。
   - missing / corrupt / unknown sidecar / duplicate / revision conflict 处理。
   - 导入顺序。
   - 不覆盖源 JSON / sidecar。

5. Transaction boundary：
   - 单事务必须覆盖的跨 store 操作。
   - 双写期写 DB + JSON 的一致性策略。
   - 失败顺序和 rollback 行为。
   - crash injection 点。

6. Export / rollback：
   - DB -> JSON / sidecar export 目标。
   - 恢复旧 JSON 读路径的条件。
   - 备份保留与导出 hash。
   - DB 损坏 / importer 中断 / 双写失败恢复流程。

7. Fixture / tests：
   - 最小 fixture 集。
   - idempotent import tests。
   - corrupt / partial / duplicate / alias / transaction rollback tests。
   - shape gate / cargo / diff 验证矩阵。

8. R3-A1 建议任务包：
   - 只做 schema 文件 + dry-run importer + fixture，不切读写路径。
   - 明确读写范围、验证命令、不做项。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
rg -n "workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations" prototypes/productized-desktop-shell/src-tauri/src
rg -n "rusqlite|sqlite|StoreLock|revision|corrupt|backup|rename|\\.v1\\.json" prototypes/productized-desktop-shell/src-tauri/Cargo.toml prototypes/productized-desktop-shell/src-tauri/src
git diff --check
git status --short
```

可选跑：

```bash
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
```

如果不跑可选命令，必须说明原因。合同冻结任务不能把未运行的 cargo/fmt 冒充为本轮 fresh verify。

## 9. 必须回传

开发线回传必须包含：

1. STATUS：`DONE` / `DONE_WITH_CONCERNS` / `NEEDS_DECISION` / `BLOCKED`。
2. 读了哪些文件。
3. 写了哪些文件。
4. schema v0 摘要。
5. importer / idempotency 摘要。
6. transaction / rollback 摘要。
7. fixture / verification 摘要。
8. R3-A1 建议。
9. 运行了哪些检查。
10. P0 / P1 / P2。
11. 是否触碰任何禁止项。

## 10. 主管回收标准

本任务可接受为：

- R3 SQLite schema / importer / rollback 合同冻结完成。
- R3-A1 可以按合同进入最小 schema + dry-run importer 任务准备。
- R3 风险被拆成可执行任务，而不是直接迁移。

本任务不接受为：

- R3 SQLite 迁移开始或完成。
- DB schema 实现完成。
- importer 实现完成。
- 双写期开始。
- 读切 DB 完成。
- JSON / sidecar 停写。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 恢复。
