# Root Treatment / R-U2 Rust Sidecar Path Util Dedup v1

日期：2026-06-13

状态：已完成。

性质：R-U 后端 util 去重第 2 包。本包只把 12 个 Rust 后端重复 `sidecar_path(workflow_state_path: &Path)` helper 收敛到 `src-tauri/src/utils/store_paths.rs`，由各 store 文件继续保留自己的 `SIDECAR_NAME` 原值和店标签；严格无行为变化。

Planning baseline：`6fca242`。

## 0. 主管线理解

用户要求按合并正本进入 R-U2：

- 前置 U1 已在 `6fca242` 收口。
- 将 12 个重复 `sidecar_path` helper 收敛到 `src-tauri/src/utils/store_paths.rs`。
- 公共 helper 增加文件名 / store_name 参数。
- 各店 `SIDECAR_NAME` 常量必须留在原文件原值，调用时传入公共 helper。
- 报错串处理选方案 (a)：传入逐店 store label，保留原报错文案。
- 严格无行为变化，以聚焦测试和 `cargo test --lib` 全绿为铁证。
- 不改 store 读写业务语义，不改 JSON / sidecar schema，不改状态机，不迁 SQLite。
- 完成后交独立复核线 CLEAR，再 commit，再停 U2 复核点。
- 完成 / 提交报告必须附 `git log --oneline -6`、`git status --short` 和关键验证命令原始尾部输出。

## 1. 当前扫描事实

当前重复定义为 12 个：

- `memory_lint_store.rs`
- `memory_entity_relation_store.rs`
- `session_continuation_store.rs`
- `plan_authorization_store.rs`
- `memory_candidate_store.rs`
- `observation_store.rs`
- `formal_memory_store.rs`
- `project_consultation_proposal_store.rs`
- `memory_capture_bus.rs`
- `mature_pattern_store.rs`
- `blackboard_candidate_store.rs`
- `runtime_log_store.rs`

12 个签名一致：

```rust
pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String>
```

函数体差异仅为：

- 各文件自己的 `SIDECAR_NAME`。
- 各文件自己的报错标签，例如 `memory lint`、`实体关系`、`continuation`、`runtime log`。

当前测试中存在 sidecar 文件名断言，例如 `offline-permission-dialog.test.tsx` 断言 `blackboard-candidates.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`formal-memories.v1.json`。因此本包禁止把文件名集中搬到 utils，避免接错文件名导致真实数据写错位置。

## 2. 目标

完成后：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs`。
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs` 增加 `pub(crate) mod store_paths;`。
- 公共 helper 形态：

```rust
pub(crate) fn sidecar_path(
    workflow_state_path: &Path,
    sidecar_name: &str,
    store_name: &str,
) -> Result<PathBuf, String>
```

- 12 个 store 文件删除本地重复 parent/join 实现，保留同名 wrapper：

```rust
pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "原店标签")
}
```

保留同名 wrapper 的理由：

- 外部调用点和测试仍使用 `crate::<store>::sidecar_path(...)`。
- 本包只做内部实现去重，不改变公开模块内 helper 入口。
- 可让每个 store 继续拥有自己的 `SIDECAR_NAME` 和店标签，降低文件名误接风险。

## 3. 报错文案方案

本包选择方案 (a)：把店标签也参数化，逐店保住原报错文案。

公共 helper 生成：

```rust
format!(
    "workflow state 路径没有父目录，无法推导 {store_name} sidecar：{}",
    workflow_state_path.display()
)
```

各店传入的 `store_name` 必须逐一保持原文：

- `memory lint`
- `实体关系`
- `continuation`
- `方案授权`
- `记忆候选`
- `observation`
- `正式记忆`
- `项目咨询方案`
- `memory capture`
- `成熟模式`
- `黑板候选`
- `runtime log`

选择方案 (a) 的理由：最稳，不需要证明所有旧报错文案都没有测试或下游依赖；同时保留用户可读诊断语义。

## 4. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs`
- 上述 12 个含重复 `sidecar_path` helper 的 Rust 文件。
- 本任务包。
- 对应 evidence / handoff / review evidence。
- 必要 checkpoint 入口文档。

允许的代码变化仅限：

- 增加 `crate::utils::store_paths` import。
- 将本地 `sidecar_path` 函数体改为调用公共 helper。
- 删除因 wrapper 不再直接使用而变成 unused 的 `PathBuf` import。
- 保留各店 `SIDECAR_NAME` 常量原值和位置。

## 5. 禁止范围

禁止：

- 把 12 个 `SIDECAR_NAME` 文件名集中搬到 `utils/store_paths.rs` 或任何共享 registry。
- 修改任一 sidecar 实际文件名。
- 修改任一 store 的 `load_store` / `empty_store` / `validate_store` / `write_store` / lock / backup / atomic replace 业务语义。
- 修改 JSON / sidecar schema 字段。
- 修改 workflow state schema。
- 修改状态机语义。
- 迁移 SQLite 或修改 SQLite schema / migration / read-cut / stop-write 决策。
- 修改真实 Codex runner / command 参数。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 解冻 backlog。

## 6. 停止线

若抽取 `sidecar_path` 牵连以下任一情况，必须停止：

- 任一 store 实际 sidecar 路径发生变化。
- 任一 store 读写语义需要改变。
- 任一 `SIDECAR_NAME` 需要移动或集中管理。
- 需要修改 JSON / sidecar schema、workflow state schema、状态机或 SQLite 迁移路径。
- 需要真实 Codex 执行或读取 `/Users/yoyi/.codex` 才能验证。

发生停止时，该 store 留原地并在 evidence 记为 deferred，不硬合。

## 7. 验证

必须通过并在 evidence 粘贴原始尾部输出：

- `cargo fmt -- --check`
- 聚焦测试：
  - `cargo test --lib memory_lint`
  - `cargo test --lib memory_entity_relation`
  - `cargo test --lib session_continuation`
  - `cargo test --lib plan_authorization`
  - `cargo test --lib memory_candidate`
  - `cargo test --lib observation`
  - `cargo test --lib formal_memory`
  - `cargo test --lib project_consultation`
  - `cargo test --lib memory_capture`
  - `cargo test --lib mature_pattern`
  - `cargo test --lib blackboard`
  - `cargo test --lib runtime_log`
- `cargo test --lib`，基线预期 `476 passed / 16 ignored`。
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须扫描：

- `rg -n "fn sidecar_path\\(" prototypes/productized-desktop-shell/src-tauri/src`
- `rg -n "const SIDECAR_NAME" <12 个 store 文件>`
- `git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`

## 8. 复核结果

独立复核 agent `Poincare`（`019ec19f-6366-7cf0-9a17-ecb07722429e`）回交 `STATUS: CLEAR_WITH_P2`，P0/P1 无；唯一 P2 为 evidence 的 `git status --short` 记录漏写 task/evidence/handoff 文件。该 P2 已补齐，不影响代码行为或提交放行；记录见 `evidence/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1-review-poincare-v1.md`。

复核确认：

- 12 个 store wrapper 是否仍存在且外部调用入口不变。
- 公共 helper 是否只做 parent error + join sidecar_name。
- 12 个 `SIDECAR_NAME` 原值是否逐店零变化。
- 12 个报错标签是否逐店保持原文。
- 是否没有改 `load_store` / `empty_store` / `validate_store` / write / lock / backup / atomic replace 业务语义。
- 是否没有改 JSON / sidecar schema、workflow state schema、状态机或 SQLite schema / migration。
- 是否没有新增真实 Codex 执行路径、`.codex` 访问或 runner 参数变更。
- 验证记录是否可信。

## 9. 不接受为

本包不接受为：

- R-U 全部完成。
- U3 / U4 / U5 / U-Gate 完成。
- store 模式合并完成。
- 任一 sidecar 文件名集中管理完成。
- R3 Level B 执行。
- SQLite 真实切换。
- 真实 Codex 执行。
- backlog 解冻。

## 10. 停止点

U2 完成、独立复核线 CLEAR、implementation commit 和 checkpoint commit 后，停在 U2 复核点；不得顺手进入 U3。
