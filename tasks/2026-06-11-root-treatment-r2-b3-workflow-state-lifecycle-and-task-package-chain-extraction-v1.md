# Root Treatment / R2-B3 Workflow State Lifecycle And Task Package Chain Extraction v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R2 第三批治理任务包，用于把 `src-tauri/src/lib.rs` 中 workflow state 生命周期入口和 task package 写入链物理抽出到独立 helper 文件，继续验证小批次、行为不变、可回滚的 `lib.rs` 解体路径。

R2-B3 是行为不变的形状治理任务，不新增产品能力，不执行真实 Codex，不迁移 SQLite，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1 已完成并 checkpoint。
- R2-B1 已完成：command registry 从 `lib.rs::run()` 拆到 `command_registry.rs`，completion commit `13016917442070fc2f59a130b2748eb0cba06a34`。
- R2-B2 已完成：R2 代码地图已建立，15 个 workflow state JSON helper 抽到 `workflow_state_json_helpers.rs`，completion commit `76ed0ef46d9b0a2a83f6e77ce533d6c8741c93cf`。
- 当前 `lib.rs` 仍有 25,643 行，代码地图显示 244-1,253 行主要是 workflow state 生命周期入口和 task package 写入链。

R2-B3 的核心判断：

```text
先搬一组连续、测试落点清楚的 workflow / task package 写入 helper；只搬位置，不改行为。
```

这不是 R3 SQLite，不是 R4 前端瘦身，也不是恢复 Stage L。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b2-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b2-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`

## 2. 目标

R2-B3 必须完成：

- 新增小型 helper 文件，例如 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`。
- 将 `lib.rs` 中以下函数物理移入新 helper 文件，优先使用 `include!` 保守展开在 crate root，避免一次性修改函数可见性：
  - `read_workflow_state_snapshot`
  - `initialize_workflow_state_at`
  - `bootstrap_project_workflow_at`
  - `append_default_project_workflow`
  - `create_task_draft_at`
  - `render_task_package_preview_at`
  - `update_task_package_draft_fields_at`
  - `update_task_package_fields_at`
  - `generate_task_package_file_at`
  - `inspect_task_package_dispatch_readiness_at`
- `lib.rs` 原位置保留一个 `include!("workflow_state_lifecycle_task_package.rs")` 或等价保守入口。
- `lib.rs` 行数必须继续低于 25,643。
- 新增 Rust 文件必须低于 3,000 行。
- 不改任何函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 写 R2-B3 evidence / handoff。

允许的实现方式：

- 优先使用 `include!("workflow_state_lifecycle_task_package.rs")`，让 helper 仍在 crate root 展开。
- 如果需要正式 `mod`，必须解释为什么不能用 `include!`，并证明函数可见性和行为不变。
- 可以机械调整空行和 `rustfmt`，但不得顺手重命名用户可见概念或错误文案。

## 3. 允许读取

- 全部项目源码和文档。
- git 元数据。
- R0/R1/R2-B1/R2-B2 evidence / handoff。

## 4. 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `evidence/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1-result.md`

本线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口同步由主管线 checkpoint 统一做。

## 5. 禁止事项

R2-B3 禁止：

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
- 不顺手拆 workflow read model、记忆领域、runtime diagnostics、SQLite、UI 或 tests 巨石。

## 6. 形状影响

- 任务类型：治理任务包。
- 新增代码落点：`src-tauri/src/workflow_state_lifecycle_task_package.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，目标是行数继续下降。
- 预计行数变化：`lib.rs` 预计减少约 900-1,100 行；新增 helper 文件预计小于 1,200 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：主管线派发时填写。
- 本任务完成 commit：待完成后记录。

## 7. 验收标准

R2-B3 可接受为：

- 指定 workflow state 生命周期入口和 task package 写入链函数已从 `lib.rs` 物理抽出。
- `lib.rs` 行数低于 25,643。
- 新增 Rust 文件低于 3,000 行。
- Tauri command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。
- `cargo test --lib workflow_state` 通过。
- `cargo test --lib task_package` 通过。
- `cargo test --lib workflow_run_check` 通过；如果 filter 无匹配或不稳定，必须说明并至少由全量库测试覆盖。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline` 和 `--mode check` 通过。
- `git diff --check` 通过。
- evidence / handoff 记录 start commit、end commit、前后行数、验证结果和 P2。

R2-B3 不接受为：

- R2 全部完成。
- `lib.rs <= 15,000` 第一阶段目标完成。
- workflow read model、记忆领域、runtime diagnostics 或 SQLite 迁移完成。
- workflow state schema 迁移完成。
- 新真实执行授权或 Stage L 恢复。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib workflow_state
cargo test --lib task_package
cargo test --lib workflow_run_check
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
5. 抽出函数清单。
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
- workflow state / task package 测试失败且未解释。
- Tauri command 总量变化。
- `lib.rs` 行数没有下降。
- 改了 workflow state schema、公开 command 契约或用户可见错误文案。
- 未跑 shape gate。
- 未形成独立 commit。

P2 示例：

- 仍使用 `include!` 作为保守过渡。
- 本轮只抽出 lifecycle / task package 写入链，不代表 workflow read model 或 storage 迁移完成。
- task package 相关测试仍在 `lib.rs` inline tests 中，后续 R2 后段再迁移 tests。
