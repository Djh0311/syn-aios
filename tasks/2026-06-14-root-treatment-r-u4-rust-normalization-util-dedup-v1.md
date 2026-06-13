# Root Treatment / R-U4 Rust Normalization Util Dedup v1

日期：2026-06-14

状态：待执行。

性质：R-U 后端 util 去重。本包只把规则完全相同的 Rust `normalize(value)` helper 收敛到 `src-tauri/src/utils/normalization.rs`；规则不同或触及业务语义的 normalization 记 deferred，不硬合。

Planning baseline：`0a27b91`。

## 0. 主管线理解

用户要求 U4：

- `normalize + 特化 → utils/normalization.rs`。
- 先扫各处 normalize。
- 规则相同的合并。
- 规则不同 / 要动到 store 业务含义的，记 deferred + 理由，不合。
- 完成定义不是“全部强合”，而是可合的合了，不可合的有清单。
- 独立复核 CLEAR 后提交 implementation commit，再写 checkpoint。
- 不进入 R3 Level B、不实现 U-Gate、不读写 `/Users/yoyi/.codex`。

## 1. 扫描事实

`rg -n "^fn normalize|^fn normalize_|^fn .*normalize|^pub fn normalize" prototypes/productized-desktop-shell/src-tauri/src --glob '*.rs'` 显示：

### 1.1 可合并同形 helper

以下本地 `fn normalize(value: &str) -> String` 函数体完全同形：

```rust
value.trim().replace('\\', "/").to_lowercase()
```

涉及文件：

- `blackboard_candidate_store.rs`
- `memory_entity_relation_governance.rs`
- `memory_candidate_store.rs`
- `memory_lint_engine.rs`
- `codex_local_runner.rs`
- `control_core.rs`（函数名为 `normalize_symbol`，函数体同形，但语义名是 symbol；本包可改为 wrapper 调公共 helper，保留函数名）
- `formal_memory_store.rs`
- `task_memory_packet_builder.rs`
- `session_continuation_store.rs`
- `formal_memory_lifecycle.rs`
- `observation_store.rs`

说明：同形候选为 11 处，其中 `control_core.rs` 是特化函数名 `normalize_symbol`，为保持调用语义，本包只把函数体代理到公共 helper，不删除本地函数名。

### 1.2 Deferred，不合并

- `memory_capture_bus.rs`：`value.trim().to_ascii_lowercase()`，不做 slash normalize，且 ASCII lowercase；deferred。
- `mature_pattern_governance.rs`：trim + lowercase 后只保留 ascii alphanumeric / whitespace / 非 ASCII 的特化 key；deferred。
- `c4_c6_workflow_governance_entrypoints.rs`：`normalize_c4_symbol` 额外把 `-` 替换为 `_`；deferred。
- `workflow_execution_entrypoints.rs`：`normalize_director_review_decision` 是带校验的业务枚举 normalize；deferred。
- `codex_transcript.rs`：path canonical / normalized path 是路径处理；deferred。
- `control_core.rs` 的 `normalized_absolute_path`：路径解析和校验；deferred。
- 敏感路径检测中局部 `let normalized = value.to_ascii_lowercase()`：属于安全检测逻辑，不合并。
- `alias_key`、tokenize、candidate key、pattern key 等在 normalize 后继续有业务特化规则；本包只允许其底层同形 `normalize` 调公共 helper，不抽特化规则。

## 2. 目标

完成后：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs`。
- `utils/mod.rs` 新增 `pub(crate) mod normalization;`。
- 公共 helper：

```rust
pub(crate) fn normalize_slash_lowercase(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}
```

- 10 个同形 `fn normalize` 本地定义归零，调用点改用公共 helper import。
- `control_core.rs::normalize_symbol` 保留函数名，但函数体代理到 `normalize_slash_lowercase(value)`，保留调用语义。
- deferred 项写入 evidence / handoff，不硬合。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- 同形 helper 所在 Rust 文件中的 import 与 helper 定义。
- 本任务包、对应 evidence / handoff / review evidence、checkpoint 入口文档。

允许的代码变化仅限：

- 增加公共 helper。
- 删除同形本地 `fn normalize` 定义或把特化命名 helper 改为 wrapper。
- 增加 `use crate::utils::normalization::normalize_slash_lowercase as normalize;` 或等价 import。
- 增加公共 helper 的行为单测。

## 4. 禁止范围

禁止：

- 改变任何 JSON / sidecar / workflow state schema。
- 改变 store 业务规则、状态机、权限、runner、Codex 执行参数。
- 合并 `memory_capture_bus.rs` 的 ASCII normalize。
- 合并 `mature_pattern_governance.rs`、`normalize_c4_symbol`、`normalize_director_review_decision`、path normalization、敏感路径检测或 key/token 特化逻辑。
- 迁 SQLite 或进入 R3 Level B。
- 实现或接入 U-Gate。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 解冻 backlog。

## 5. 停止线

若抽取某处 normalization 需要改变业务语义、函数可见性、状态判断、JSON 字段、路径校验或测试断言，则该处停止并记 deferred，不硬合。

若 `cargo test --lib` 或聚焦测试暴露行为变化，停止并回滚本包未安全部分。

## 6. 验证

必须通过并在 evidence 粘贴原始尾部输出：

- `cargo fmt -- --check`
- 聚焦测试：
  - `cargo test --lib memory_candidate_store`
  - `cargo test --lib formal_memory_store`
  - `cargo test --lib session_continuation_store`
  - `cargo test --lib codex_local_runner`
  - `cargo test --lib control_core`
- `cargo test --lib`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须扫描：

- `rg -n "^fn normalize\\(value: &str\\)|^fn normalize_symbol" prototypes/productized-desktop-shell/src-tauri/src --glob '*.rs'`
- `rg -n "normalize_slash_lowercase" prototypes/productized-desktop-shell/src-tauri/src --glob '*.rs'`

## 7. 复核判据

独立复核线必须确认：

- 公共 helper 行为等于同形原规则：`trim().replace('\\', "/").to_lowercase()`。
- 可合并同形 `fn normalize` 已删除或通过 import 归一；`control_core::normalize_symbol` 仅 wrapper，不改行为。
- Deferred 清单准确，未强合规则不同项。
- 未改 store 业务 / JSON / schema / 状态机 / runner / SQLite 迁移。
- 验证记录可信。

## 8. 不接受为

本包不接受为 R-U 全部完成、U-Gate 完成、查重门实现、R3 Level B 执行、SQLite 真实切换、真实 Codex 执行、`.codex` 读写、backlog 解冻或所有 normalization 规则强制统一完成。

## 9. 停止点

任务包提交后进入实现；实现经独立复核 CLEAR、implementation commit 和 checkpoint commit 后，停在 U4 复核点，继续 U-Gate 草稿。
