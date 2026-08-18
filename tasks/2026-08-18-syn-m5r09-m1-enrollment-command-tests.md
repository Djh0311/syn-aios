# Grok 窄包：M5R09 enrollment command 直接测试

生产 core 已提交为 `599f555`。本包只给该 command 加直接测试，不改生产逻辑或 registry。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

该文件进入本包的受保护工作副本 SHA-256 是 `000cfa48ca4712990f8be83e1e939d2e663fbb1c2f1783c41a678aa556e0973e`，Git blob 快照是 `7944ed100af99cd738d7814db192dd0829653cea`；其中既有未归属 WIP 必须逐字保全。只在文件末尾新增独立 `#[cfg(test)] mod m5r09_m1_enrollment_command_tests`，不要改前文、不要整文件格式化、不要 git 操作。

## 测试

所有测试函数前缀 `m5r09_m1_enrollment_command_`。用临时 parent、名字严格为 `M1_ORDINARY_APP_DATA_DIR_NAME` 的 app-data root、合成 index/tasks seed 和 `AppState::try_new_with_tauri_ordinary_product_seeds`，覆盖：

1. 唯一 exact root：构造 UNENROLLED state 后先把原 seed 改成空 index，证明 command 仍读取已安装的 `state.index_path`；首次调用 `_with_state` 返回 `created`、opaque project id、source ref `product-index:<root>`、source/registry revision 1；重复返回 `already_enrolled`、同 id、同 revision、source 只有一个 entry；丢弃并重建 AppState 后再次调用仍同 id/revision。
2. 两个独立 fixture：零匹配、两条相同 exact-root 的多匹配；都返回精确 `m1_enrollment_product_index_exact_match_required`，且 source、registry、`.m1-project-index.established` 均不存在。
3. `serde_json::from_value::<M1ProjectIdentityEnrollmentRequest>` 对仅 `project_root` 成功，对额外 `project_id` 或 `source_ref` 失败。
4. 静态可达性/边界：`include_str!("command_registry.rs")` 含 `enroll_m1_project_identity,`；从 `fn enroll_m1_project_identity(` 到下一个 `/// Fixed no-request` 的 production span 含 `read_index`、`parse_projects`、`enroll_ordinary_project`，不含 `project_id(`、`stable_id(`、`legacy`。

不要碰真实项目/资料。测试 cleanup 只删自己创建的精确 temp parent。

## 验证

- `cargo test --lib --offline m5r09_m1_enrollment_command_ -- --test-threads=1`
- `cargo test --lib --offline m5r09_m1_enrollment_ -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包保留；若未跑明确报告，候选流程会重跑）
- 本文件 `git diff --check`

完成新增与前两条定向测试后立即退出，不读 harness/历史或无关文件。
