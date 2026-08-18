# Grok 窄包：M5R09 enrollment command production core

这是上一 command 包的进一步拆分。本包只写生产 core 和一行真实注册，不写测试。

## 只许修改

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

进入本包 SHA-256：`commands.rs=dbfc172e5dd89436d460030aa0e708e23ac9f86cf4774cd5b30e01be81c33d3d`，`command_registry.rs=f8ff199b2d8974d77cdea9db1fed2bcb0e4c0f3b97c6c0fadb8b141d4c0083d3`。前者含受保护旧 WIP，必须逐字保全。不要改其他文件、不要整文件格式化、不要 git 操作。

## 精确实现

在 `commands.rs` 顶部 `load_workbench_snapshot` 之后加入：

- `M1ProjectIdentityEnrollmentRequest`：`Deserialize`、`#[serde(deny_unknown_fields)]`、唯一字段 `project_root: String`；
- `M1ProjectIdentityEnrollmentDto`：`Serialize`，字段 `project_id/exact_alias/source_ref/source_revision/registry_revision/status`；
- `#[tauri::command] fn enroll_m1_project_identity(request, state) -> Result<DTO,String>` 仅委托 `_with_state`；
- `_with_state` 每次 `read_index(state)`，`parse_projects(&index)` 后筛选 `project.project_root == request.project_root`，必须 `matches.len() == 1`，否则在任何 authority 调用前返回 `m1_enrollment_product_index_exact_match_required`；
- server 组装 `source_ref = format!("product-index:{}", exact_root)`，调用 `state.m1_project_index_authority().map_err(|e| e.code)?.enroll_ordinary_project(&m1_project_index::M1EnrollOrdinaryProjectRequest { exact_alias, source_ref })`；
- status 精确映射 `Created -> "created"`、`AlreadyEnrolled -> "already_enrolled"`，其余字段原样 DTO 化。

生产 span 严禁调用 `project_id(`、`stable_id(`、hash/path-derived helper 或 legacy import。然后在 `workbench_command_handler!` 的 `generate_handler!` 列表紧接 `load_workbench_snapshot` 加一行 `enroll_m1_project_identity`。

## 验证

- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包保留；若未跑明确报告，候选流程会重跑）
- 两个允许文件 `git diff --check`

不要读 harness、历史或无关文件；完成最小编辑后立即验证并退出。
