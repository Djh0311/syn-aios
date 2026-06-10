# Root Treatment / R2-B1 Command Registry Extraction v1

日期：2026-06-10

状态：待执行。本文是 Root Treatment / Stage R 的 R2 第一批治理任务包，用于把 Tauri command registry 从 `src-tauri/src/lib.rs` 的 `run()` 中拆出，验证 R0 shape gate / git 分批治理流程，并为后续 `lib.rs` 解体建立低风险入口。

R2-B1 是行为不变的形状治理任务，不新增产品能力，不执行真实 Codex，不迁移 SQLite，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R0 已完成 workbench shape gate、任务包形状影响节、治理任务包类型和解冻后 `1:3` 治理配额，commit `7563e6a9d11a92217e1baf34ed71b70722bbc17c`。
- R1 已完成 workflow state 最终写入 / rename 文件级 StoreLock、corrupt guard 和 backup retention 测试夹具，commit `7a1ac89173306b50868064b64fb852f57c0550af`。
- R0/R1 主管 checkpoint 已完成，commit `b0a6447`。
- 当前 `lib.rs` 仍是 25,925 行，`run()` 内直接持有 `tauri::generate_handler![...]` command registry。
- `commands.rs` 通过 `include!("commands.rs")` 保守拆分进 crate root，当前 Tauri command wrappers 仍保持原命名和可见性。

R2-B1 的核心判断：

```text
先把 command registry 从 lib.rs 物理移出，保持命令行为不变，验证 R2 分批治理路径。
```

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `evidence/2026-06-10-root-treatment-r0-r1-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-10-root-treatment-r0-r1-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

## 2. 目标

R2-B1 必须完成：

- 新增小型 command registry 文件，例如 `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`。
- 将 `tauri::generate_handler![...]` 命令列表从 `lib.rs` 移入该文件。
- `lib.rs` 的 `run()` 只调用 registry 宏 / helper，不再直接展开完整命令列表。
- 不改任何 Tauri command wrapper 函数签名、名称、参数或返回值。
- 不新增任何 `#[tauri::command]`。
- 不新增 sidecar JSON 种类。
- 不改 workflow state 顶层 schema。
- 记录 `lib.rs` 前后行数和 shape gate 前后结果。
- 写 R2-B1 evidence / handoff。

允许的实现方式：

- 优先使用 `include!("command_registry.rs")` + `macro_rules!`，让 registry 在 crate root 展开，避免为了第一批治理一次性修改 96 个 command wrapper 可见性。
- 如果需要改成模块函数，必须解释为什么不能用宏，并证明不会改变命令注册行为。

## 3. 允许读取

- 全部项目源码和文档。
- git 元数据。
- R0/R1 evidence / handoff。

## 4. 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- 如确有必要：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `evidence/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`
- `handoffs/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1-result.md`

本线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口同步由主管线 checkpoint 统一做。

## 5. 禁止事项

R2-B1 禁止：

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
- 不顺手拆 workflow 读模型、记忆领域、runtime diagnostics 或 SQLite。

## 6. 形状影响

- 任务类型：治理任务包。
- 新增代码落点：`src-tauri/src/command_registry.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，目标是行数下降。
- 预计行数变化：`lib.rs` 预计减少约 90-120 行；新增 registry 文件预计小于 180 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`b0a6447`。
- 本任务完成 commit：待完成后记录。

## 7. 验收标准

R2-B1 可接受为：

- `lib.rs` 不再直接包含完整 `tauri::generate_handler![...]` command 列表。
- 新 registry 文件承载同一组命令。
- Tauri command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。
- `lib.rs` 行数低于 R0 水位线 25,925。
- 新增文件低于 Rust 3,000 行。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline` 和 `--mode check` 通过。
- `git diff --check` 通过。
- evidence / handoff 记录 start commit、end commit、前后行数、验证结果和 P2。

R2-B1 不接受为：

- R2 全部完成。
- `lib.rs <= 15,000` 第一阶段目标完成。
- workflow 读模型、记忆领域、runtime diagnostics 或 SQLite 迁移完成。
- command surface 重构完成。
- 新真实执行授权或 Stage L 恢复。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

如果 `cargo test --lib` 因环境问题失败，必须记录完整失败摘要，并至少跑更聚焦的编译 / 测试命令；不得把失败冒充完成。

## 9. 必须回传

开发线回传必须包含：

1. 做了什么。
2. 改了哪些文件。
3. `lib.rs` 前后行数。
4. command 总量和 `lib.rs` 内 command 数量。
5. shape gate baseline / check 摘要。
6. Rust 测试和格式化结果。
7. start commit / end commit。
8. P0 / P1 / P2。
9. 是否触碰任何禁止项。

## 10. 总指导回收动作

总指导回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0/P1 示例：

- command registry 迁移后编译失败。
- Tauri command 总量减少或增加但没有解释。
- 新增 command 到 `lib.rs`。
- 改了 command wrapper 行为。
- 未跑 shape gate。
- 未形成独立 commit。

P2 示例：

- 仍使用 `include!` 作为保守过渡，后续 R2 批次再收敛成正式模块。
- 只减少约百行，不代表 R2 第一阶段水位线目标完成。
