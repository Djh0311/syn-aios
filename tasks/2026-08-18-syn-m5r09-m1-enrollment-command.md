# Grok 窄包：M5R09 M1 enrollment command 与真实注册

前置已提交：M1 source-first authority `62e75ab`，UNENROLLED AppState `389f6de`。本包只新增一个服务端登记 command 及真实 command graph 注册，不做前端。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

`commands.rs` 进入本包前已有受保护的未归属差异（主管基准 SHA-256 `dbfc172e5dd89436d460030aa0e708e23ac9f86cf4774cd5b30e01be81c33d3d`）；必须逐字保全，只在稳定锚点做本包加法。`command_registry.rs` 基准 SHA-256 `f8ff199b2d8974d77cdea9db1fed2bcb0e4c0f3b97c6c0fadb8b141d4c0083d3`。不要修改其他文件，不要格式化整文件/crate，不要 git add/commit。

## 实现

1. 新增 `enroll_m1_project_identity` Tauri command。请求 DTO 使用 `#[serde(deny_unknown_fields)]`，唯一字段 `project_root: String`；不得接受 project id、source ref/path/revision、registry path 或 entry id。
2. command 的可测核心每次调用 `read_index(state)` 读取 AppState 已安装的 server product index；从 `parse_projects` 结果按 `project_root` 完全相等筛选，必须恰好一条。零条与多条分别/统一返回稳定 `m1_enrollment_product_index_exact_match_required`，且在 authority 调用前拒绝。
3. 服务端生成 `exact_alias = matched.project_root` 与 `source_ref = product-index:<exact-root>`，调用 `state.m1_project_index_authority()?.enroll_ordinary_project(&M1EnrollOrdinaryProjectRequest { ... })`。严禁 `project_id(`、`stable_id(`、hash/path-derived identity、legacy import 或调用方 supplied canonical id。
4. 返回 serializable DTO：`project_id`、`exact_alias`、`source_ref`、`source_revision`、`registry_revision`、字符串状态 `created`/`already_enrolled`。
5. 在 `workbench_command_handler!` 的实际 `tauri::generate_handler!` 列表注册 `enroll_m1_project_identity`，不增加任何自动调用。

## 直接测试

统一前缀 `m5r09_m1_enrollment_command_`，至少覆盖：

- 唯一 exact root 首次登记成功，重复同 id 且两个 revision 不增长；重建 ordinary AppState 后仍同 id；
- 零匹配和重复 exact-root 多匹配都在 source/registry/marker 写前拒绝；
- 构造后改写原 seed 不影响 command：它仍从 AppState 安装的 server index 读取；
- serde 请求拒绝额外 `project_id` 与 `source_ref`；
- command registry 真实注册；production command span 不含 path-derived helper。

全部使用临时 app-data 与合成 index/tasks。

## 交付验证

- `cargo test --lib --offline m5r09_m1_enrollment_command_ -- --test-threads=1`
- `cargo test --lib --offline m5r09_m1_enrollment_ -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包必须保留；若未跑明确报告，候选流程仍会重跑）
- 仓库根仅对两个允许文件执行 `git diff --check -- <paths>`

不要读 harness/历史或无关文件，不问用户，不声称 leaf/stage/M5 完成。
