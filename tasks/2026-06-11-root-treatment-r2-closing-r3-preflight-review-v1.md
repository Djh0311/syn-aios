# Root Treatment / R2 Closing And R3 Preflight Review v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R2 closing / R3 preflight review 任务包，用于在 R2-B10 达成第一阶段 `lib.rs <= 15,000` 水位线后，复核剩余 `lib.rs` 结构、inline tests 巨石、R3 SQLite 前置风险和后续治理顺序。

本任务是只读审查和决策准备任务，不迁移 SQLite，不改产品代码，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1 已完成并 checkpoint。
- R2-B1 到 R2-B10 已完成并提交。
- R2-B10 completion commit：`d5f423d97c1f2dac4bca33f84c34e46b0b4716a6`。
- R2-B10 supervisor checkpoint commit：`5339987ad2bc3510039140e92429327116d78988`。
- R2-B10 checkpoint hash backfill commit：`b7c7276`。
- `lib.rs` 当前为 13,949 行，已低于第一阶段 15,000 行水位线。
- `lib.rs` 仍包含 task package render / finder helper、shared workflow utility、atomic path / time helper、workbench snapshot assembly 和大量 inline tests。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
先做 R2 closing / R3 preflight review，再决定继续 R2 后段拆分还是进入 R3 前置任务。
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
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1-result.md`
- `tasks/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`

建议读取的代码和脚本：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `scripts/harness/workbench-shape-gate.js`

## 2. 目标

本任务必须完成：

- 复核 R2-B10 后的 `lib.rs` 剩余结构，标出继续 R2 后段拆分的候选区块。
- 复核 inline tests 巨石的规模、主要测试域和迁移优先级。
- 复核 R3 SQLite 前置风险，包括现有 JSON / sidecar 写入边界、导入器范围、事务边界、回滚演练、fixture 数据迁移和 shape gate 影响。
- 判断下一步应优先：
  - 继续 R2 后段拆分；
  - 先做 R2 tests migration；
  - 进入 R3 SQLite schema / importer preflight；
  - 或补一个小型 R2/R3 bridge 任务。
- 输出 R2 closing / R3 preflight review evidence / handoff。
- 给主管线一个明确建议：下一任务包名称、目标、允许读写范围、验证矩阵和不做项。

本任务不要求改代码，也不要求跑全量 cargo / npm。

## 3. 允许读取

- `product-line` 内全部源码、文档、任务包、evidence、handoff、脚本和 git 元数据。
- R0/R1/R2-B1 到 R2-B10 的 evidence / handoff / supervisor checkpoint。

## 4. 允许写入

- `evidence/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1-result.md`

主管线可在回收后另行更新入口文档；复核线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 或正式计划。

## 5. 禁止事项

本任务禁止：

- 不改产品源码。
- 不迁移 SQLite。
- 不建 SQLite schema。
- 不导入真实用户数据。
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
- 不把审查建议冒充为 R3 已完成。
- 不把 `lib.rs <= 15,000` 冒充为 R2 全部完成。

## 6. 形状影响

- 任务类型：治理审查任务包。
- 新增代码落点：无。
- 触碰棘轮文件：无。
- 预计行数变化：无产品代码变化；新增 evidence / handoff。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`b7c7276`。
- 本任务完成 commit：待完成后记录。

## 7. 审查清单

复核线至少输出以下矩阵：

1. `lib.rs` 剩余结构矩阵：
   - 区块名。
   - 当前行号范围。
   - 主要职责。
   - 是否适合 R2 后段抽出。
   - 推荐拆分方式：`include!` 过渡 / 正式 `mod` / 等待 R3。
   - 风险。
   - 推荐测试。

2. inline tests 矩阵：
   - 当前测试模块起止位置。
   - 主要测试域。
   - 可优先迁移的测试组。
   - 迁移目标文件建议。
   - 不建议现在迁移的测试组。

3. R3 SQLite preflight 矩阵：
   - 当前 JSON / sidecar store。
   - 是否有 StoreLock / corrupt guard / revision guard。
   - 迁移表候选。
   - 导入器输入。
   - 双写风险。
   - 回滚 / 导出策略。
   - 测试 fixture 需求。

4. 下一任务建议：
   - 任务包名称。
   - 执行模式：单线 / 多线。
   - 读写范围。
   - 验证命令。
   - 是否需要主管自审高风险动作。
   - 是否需要用户再授权。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
git status --short
```

建议跑：

```bash
rg -n "include!|#\\[cfg\\(test\\)\\]|fn test_|\\#\\[test\\]" prototypes/productized-desktop-shell/src-tauri/src/lib.rs
wc -l prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/*.rs
git log --oneline -12
```

可选跑：

```bash
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
```

如果不跑可选命令，必须说明原因。只读审查结论不能依赖未运行的测试。

## 9. 必须回传

复核线回传必须包含：

1. STATUS：`DONE` / `DONE_WITH_CONCERNS` / `NEEDS_DECISION` / `BLOCKED`。
2. 读了哪些文件。
3. 写了哪些文件。
4. `lib.rs` 剩余结构矩阵摘要。
5. inline tests 矩阵摘要。
6. R3 SQLite preflight 矩阵摘要。
7. 推荐下一任务包。
8. 运行了哪些检查。
9. P0 / P1 / P2。
10. 是否触碰任何禁止项。

## 10. 主管回收标准

本任务可接受为：

- R2-B10 后的 R2 closing / R3 preflight review 完成。
- 下一步治理顺序有清晰建议。
- R2 第一阶段水位线达成事实被正确记录，但不冒充 R2 完成。
- R3 风险被拆成可执行前置任务，而不是直接开始迁移。

本任务不接受为：

- R2 全部完成。
- R3 SQLite 迁移开始或完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 恢复。
- 真实 Codex 执行授权。
- 产品功能或 UI 修补完成。

P0/P1 示例：

- 复核线读写 `/Users/yoyi/.codex`。
- 复核线改产品源码或迁移 SQLite。
- 复核结论把审查建议冒充为完成事实。
- 推荐下一步缺少回滚 / 验证 / 影响面。

P2 示例：

- 只读静态审查，没有跑全量 cargo / npm。
- 行号基于当前文件快照，后续任务开始前仍需再确认。
- R3 schema 仍需单独任务包冻结。
