# Root Treatment / R2-B6 Workflow Execution Control Offline Role And Machine Extraction v1

日期：2026-06-11

状态：已完成，主管线回收为 `accepted_with_p2`。本文是 Root Treatment / Stage R 的 R2 第六批治理任务包，用于把 `src-tauri/src/lib.rs` 中 workflow dispatch execution control、offline role dispatch 和 workflow machine 相关逻辑物理抽出到独立 helper 文件，继续推进小批次、行为不变、可回滚的 `lib.rs` 解体路径。

R2-B6 是行为不变的形状治理任务，不新增产品能力，不执行真实 Codex，不迁移 SQLite，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1 已完成并 checkpoint。
- R2-B1 已完成 command registry extraction，completion commit `13016917442070fc2f59a130b2748eb0cba06a34`。
- R2-B2 已完成 lib map and workflow state JSON helper extraction，completion commit `76ed0ef46d9b0a2a83f6e77ce533d6c8741c93cf`。
- R2-B3 已完成 workflow state lifecycle and task package chain extraction，completion commit `208fabaa4cae8aeda45cdce4c66cbe7f2cf8e6c3`。
- R2-B4 已完成 workflow run binding and legacy dispatch entrypoints extraction，completion commit `66a0cff5a4fb94101c1830a174dc908448ec8dba`。
- R2-B5 已完成 workflow read model dispatch summary and readback stats extraction，completion commit `35cacc22ec813152e9357a42bc82e7ef581d2509`。
- R2-B6 已完成 workflow execution control offline role and machine extraction，completion commit `2dd766be84e977d75e77f31ec2dbf9d463f45690`。
- 当前 `lib.rs` 为 19,401 行；workflow dispatch execution control、offline role dispatch 和 workflow machine 已抽出到 `workflow_execution_entrypoints.rs`。

R2-B6 的核心判断：

```text
把 dispatch execution control / offline role dispatch / workflow machine 这一组可测试的执行编排逻辑从 lib.rs 移出；只搬位置，不改行为。
```

说明：默认采用保守 `include!` helper，例如 `workflow_execution_entrypoints.rs`，让函数仍在 crate root 展开，避免一次性改大量可见性。如果开发线判断需要缩小为两批，必须先做低风险连续块并在 evidence 中说明未搬部分；不得扩大到 memory、provider、diagnostics、SQLite 或 UI。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b5-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b5-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`

## 2. 目标

R2-B6 必须完成：

- 新增小型 helper 文件，例如 `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`。
- 将 `lib.rs` 中 workflow dispatch execution control、offline role dispatch、workflow machine run loop 和相邻 execution result helper 物理移出。
- `lib.rs` 原位置保留一个 `include!("workflow_execution_entrypoints.rs")` 或等价保守入口；若使用正式 `mod`，必须解释原因并证明行为不变。
- `lib.rs` 行数必须继续低于 21,463。
- 新增 Rust 文件必须低于 3,000 行。
- 不改任何函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 不新增 `#[tauri::command]`。
- 不新增 sidecar JSON 种类。
- 写 R2-B6 evidence / handoff。

建议优先覆盖的函数 / 区域：

- workflow dispatch authorization / prepared / started / completed / failed / readback dispatch 相关 helper。
- workflow dispatch director review / permission decision 相关 helper。
- offline role dispatch / handoff / director review 相关 helper。
- workflow machine request validation、round steps、step instruction、prompt render、execution root、acceptance、started / finished append、result from state 相关 helper。

允许开发线缩小范围，但必须说明原因；不得扩大到 memory、runtime diagnostics、provider / adapter、SQLite、UI、Tauri app assembly 或 tests 巨石。

## 3. 允许读取

- 全部项目源码和文档。
- git 元数据。
- R0/R1/R2-B1/R2-B2/R2-B3/R2-B4/R2-B5 evidence / handoff / supervisor checkpoint。

## 4. 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs` 或同等命名的新 helper 文件
- `evidence/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1-result.md`

本线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口同步由主管线 checkpoint 统一做。

## 5. 禁止事项

R2-B6 禁止：

- 不改产品业务逻辑。
- 不新增 Tauri command。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不迁移 SQLite。
- 不改 workflow state 顶层 schema。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不顺手拆 memory、runtime diagnostics、provider / adapter、SQLite、UI 或 tests 巨石。

## 6. 形状影响

- 任务类型：治理任务包。
- 新增代码落点：`src-tauri/src/workflow_execution_entrypoints.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，目标是行数继续下降。
- 预计行数变化：`lib.rs` 预计减少约 1,500-2,800 行；新增 Rust 文件必须小于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`93c20bb5b515ced0f0306ec57d61b222a592a08a`。
- 本任务完成 commit：`2dd766be84e977d75e77f31ec2dbf9d463f45690`。

## 7. 验收标准

R2-B6 可接受为：

- 指定 workflow dispatch execution control / offline role dispatch / workflow machine 相关 helper 已从 `lib.rs` 物理抽出。
- `lib.rs` 行数低于 21,463。
- 新增 Rust 文件低于 3,000 行。
- Tauri command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。
- `cargo test --lib workflow_dispatch` 通过，或如 filter 命中范围过宽 / 不稳定，必须记录真实输出并补跑更具体 filters。
- `cargo test --lib workflow_node_dispatch` 通过。
- `cargo test --lib offline_role` 通过。
- `cargo test --lib workflow_machine` 通过。
- `cargo test --lib workflow_permission` 通过。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline` 和 `--mode check` 通过。
- `git diff --check` 通过。
- evidence / handoff 记录 start commit、end commit、前后行数、验证结果和 P2。

R2-B6 不接受为：

- R2 全部完成。
- `lib.rs <= 15,000` 第一阶段目标完成，除非实际达到并由主管线单独确认。
- memory、runtime diagnostics、provider adapter、SQLite 迁移或 tests 巨石拆分完成。
- workflow state schema 迁移完成。
- 新真实执行授权或 Stage L 恢复。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_dispatch
cargo test --lib workflow_node_dispatch
cargo test --lib offline_role
cargo test --lib workflow_machine
cargo test --lib workflow_permission
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

如果某个 filter 因无匹配或环境问题失败，必须记录完整原因，并由更宽的相关测试或 `cargo test --lib` 覆盖；不得把失败冒充完成。

## 9. 必须回传

开发线回传必须包含：

1. 做了什么。
2. 改了哪些文件。
3. `lib.rs` 前后行数。
4. 新 helper 文件行数。
5. 抽出函数 / 类型清单。
6. command 总量和 `lib.rs` 内 command 数量。
7. shape gate baseline / check 摘要。
8. Rust 测试和格式化结果。
9. start commit / end commit。
10. P0 / P1 / P2。
11. 是否触碰任何禁止项。

## 10. 总指导回收动作

总指导回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0/P1 示例：

- helper 抽出后编译失败。
- workflow dispatch / offline role / workflow machine 相关测试失败且未解释。
- Tauri command 总量变化。
- `lib.rs` 行数没有下降。
- 改了 workflow state schema、公开 command 契约或用户可见错误文案。
- 未跑 shape gate。
- 未形成独立 commit。

P2 示例：

- 仍使用 `include!` 作为保守过渡。
- 本轮只抽出 workflow dispatch execution control / offline role dispatch / workflow machine，不代表 memory、runtime diagnostics、provider adapter、SQLite 或 tests 巨石拆分完成。
- 相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段再迁移 tests。
