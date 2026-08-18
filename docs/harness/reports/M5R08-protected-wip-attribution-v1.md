# M5R08 受保护未归属 WIP 只读归责清单 v1

观察时间：`2026-08-18T15:07:58+08:00`

观察基线：M5R07 独立验收结论记录的 34 项工作树载体，即 23 个 tracked ` M` + 11 个 untracked `??`。本清单只做路径、Git 状态、内容 SHA-256、可证明来源/语义域和 disposition 记账；不推断作者、不暂存、不覆盖、不删除。

统一 disposition：`PRESERVE_IN_PLACE / EXCLUDE_FROM_M5R08_CANDIDATE / NO_CLEAN / OWNER_REVIEW_REQUIRED`。语义域只按路径和 verdict 已给分类，不等于代码已验收。

| # | Git | SHA-256 | bytes | path | 来源 / 语义归属 |
|---:|:---:|---|---:|---|---|
| 1 | ` M` | `9c909e50c2e340171a2a343763829892ac7e3dbfa4f03912c1f1b6bfbfb58c24` | 1762 | `docs/harness/usage/.observed.json` | M5R07 verdict 前已存在；Harness usage runtime |
| 2 | ` M` | `7c31a0df0c3f7c8f6636f88ff02c62173eeefdcfaa149e0d178f93d93b784661` | 128085 | `docs/harness/usage/.observed.jsonl` | M5R07 verdict 前已存在；Harness usage runtime |
| 3 | ` M` | `830722a25e6c702b500caa22f45d6c1cf9fe448f96667062932b85e186a610fa` | 46555 | `prototypes/productized-desktop-shell/src-tauri/src/acceptance_runtime_profile.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 4 | ` M` | `8979c93a95e6d0f5167b962b5531632ca3a785d5fb067f3485eee7b0b1a66a36` | 35571 | `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 5 | ` M` | `08120086ac64c7ff6583d71c3211aa8ca97088a8dd903af3f0dbd1238cc53cd8` | 98061 | `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 6 | ` M` | `dbfc172e5dd89436d460030aa0e708e23ac9f86cf4774cd5b30e01be81c33d3d` | 387858 | `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` | M5R07 verdict 前已存在的 59+/56- 未暂存 WIP，叠加 M5R08 已提交 hunk 后的工作文件 hash；旧 delta 仍独立留在 index 外 |
| 7 | ` M` | `7d3a29371522af67d0c55ccb717d908926e64b93592e9760137e8abc2a1d8d4a` | 32081 | `prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 8 | ` M` | `5bb0df5c1f0fa6604f8af27da0dd28f63f954612d36371da80b174a1eca70331` | 4930 | `prototypes/productized-desktop-shell/src-tauri/src/m2_clock.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 9 | ` M` | `210ec1a09aba9c3a806ae440dd98b479bda77b045ffd70857b847390e9bd4686` | 73722 | `prototypes/productized-desktop-shell/src-tauri/src/m2_r4_reference_slice_driver.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 10 | ` M` | `8018820a59b26710504a005d9f1a4b6b9ad60d2eaf727d69a0c8ffbd2a826266` | 87489 | `prototypes/productized-desktop-shell/src-tauri/src/m2_update_work_item_state.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 11 | ` M` | `a3e4ec501ef11a3a10b4f194d18a65d752e38a789f1b288efb285b1c7885c292` | 85430 | `prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 12 | ` M` | `2009f13457610229ada4a347d885c2280c03bc4dc9ab3f8760eaf4556a5698de` | 13261 | `prototypes/productized-desktop-shell/src-tauri/src/m4_source_dispatcher.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 13 | ` M` | `dd839bb2c90b45b9ee27492d2d1c23bff2ab49e7b8805b0e5a6edd73e00dfe0e` | 16344 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/event_audit_boundary.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 14 | ` M` | `22a3a7746326899ad13eea64dbbeb302ab80143bb705fd5719e34ca26138479a` | 44912 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/execution_grant.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 15 | ` M` | `fedbf89e2e18701d96b52800d26afbe5c64c612aad0f8d5d6e0f35564742f8be` | 56218 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 16 | ` M` | `5aa8d79872844857cd2a1e149ed7cf6ba1e12fb482cba42902fbc5b250827ccb` | 11282 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/path_guard.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 17 | ` M` | `3a537991ebe76a7030059d274d6bfeeaca3759c651cf66dc5a64d23c9d63d0a8` | 18183 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/storage.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 18 | ` M` | `0be6791caedcf756445113087f22c21de0ec68cd4b832ccdc296b319a81609c3` | 39570 | `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 19 | ` M` | `2e122e1e5b861534090c41988c562db533aa7b413e2ae71374b8366d200a4954` | 35802 | `prototypes/productized-desktop-shell/src-tauri/src/ordinary_product_storage_bootstrap.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 20 | ` M` | `d5c809929ca33afed309d81db3a30bb447345d399fcb72d7ae908fdacb91db2d` | 61234 | `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 21 | ` M` | `2bd100fcb5f24bb26eb3a1927b98f7ebf1dd0a19145d1c01163463f3fec97e0d` | 80318 | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 22 | ` M` | `10605b2e2ddf7f8504fb30971f6c1e9936d8ddabbbeb63f6543a97f00c81c061` | 28466 | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 23 | ` M` | `db54ca720b9d6f3f63b446145f2ff14f94992120c1bac8fcbcf1b83628ded2ad` | 43904 | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema_m2.rs` | M5R07 verdict 前已存在；未归属 legacy product Rust WIP |
| 24 | `??` | `4643b4d6f257a568f8a3755f10ecfbf4cdf9df047b374104c7119542a678a108` | 1358 | `docs/harness/reports/2026-08-18-01a010f4-5eea-77c1-979d-d03cadd2a3a4-01a010f4-5f75-7de3-a0f2-f9874eedb5b9.md` | M5R07 verdict 前已存在；untracked dated Harness report |
| 25 | `??` | `08240abe0afac02413adf61a84f03c1c8fd138a2e71d7f8d3ddfd4c2f8a0eb8a` | 3087 | `docs/harness/reports/2026-08-18-01a0130d-aa03-7992-a314-1a03408c729e-01a0130d-aa7e-7390-a7bf-93c389047cc2.md` | M5R07 verdict 前已存在；untracked dated Harness report |
| 26 | `??` | `f7658bc20acb6328f1fbe1a6a32f2f6f496b090eac5950d7b53bb26ef8a5062a` | 6558 | `docs/harness/usage/host-events.json` | M5R07 verdict 前已存在；Harness usage runtime |
| 27 | `??` | `e69b44cd04910cf7083ce301e096f40004558c0e1f584cf19cd3579c95e59cc7` | 4896 | `docs/harness/usage/host-health.json` | M5R07 verdict 前已存在；Harness usage runtime |
| 28 | `??` | `7e51a7ed92547e6c96f8d37d0ff7de836e9ee5b6102b1c6ba06ae075207c2a15` | 116888 | `prototypes/productized-desktop-shell/src-tauri/gen/schemas/linux-schema.json` | M5R07 verdict 前已存在；untracked generated Tauri schema |
| 29 | `??` | `620faa27056e7cfac6fb119731c62c04eb5b65d7a4e5641b5944bea4af76b58e` | 13544 | `prototypes/productized-desktop-shell/src-tauri/src/m6_cross_project_query.rs` | M5R07 verdict 前已存在；untracked M6 candidate, M6 not active |
| 30 | `??` | `2c576d9b6f89e97f8e5e5754071a3e3623f3e5d8920a93b3a78afc6a46bb276d` | 10828 | `prototypes/productized-desktop-shell/src-tauri/src/m6_global_supervisor_session.rs` | M5R07 verdict 前已存在；untracked M6 candidate, M6 not active |
| 31 | `??` | `6cd604b4ebb483e1aba268e617d8935d0c15d4dc7ee124fe1f636b455547d84e` | 10499 | `prototypes/productized-desktop-shell/src-tauri/src/m6_member_directory.rs` | M5R07 verdict 前已存在；untracked M6 candidate, M6 not active |
| 32 | `??` | `147bd08e35609daaf0f4bb979cec73dfd305f259c9b8504576801504a2b95e30` | 10098 | `prototypes/productized-desktop-shell/src-tauri/src/m6_member_directory.rs.bak` | M5R07 verdict 前已存在；untracked M6 candidate backup, M6 not active |
| 33 | `??` | `6155c26a9c819d14be4ca6f352633faa498e8f3e0c688f67f12eec023c2bd1d6` | 16002 | `prototypes/productized-desktop-shell/src-tauri/src/m6_organization_identity.rs` | M5R07 verdict 前已存在；untracked M6 candidate, M6 not active |
| 34 | `??` | `7db42ba1010ee8b6f2bc13d9109c2b2f0dd3d74568c66cc15da31015a81e261f` | 6084 | `prototypes/productized-desktop-shell/src-tauri/src/m6_temporary_agent_history.rs` | M5R07 verdict 前已存在；untracked M6 candidate, M6 not active |

## 观察口径与新增运行载体

- 34 项计数与 verdict 一致：21 个 modified product Rust + 6 个 untracked M6 候选 + 1 个 generated schema + 4 个 Harness usage runtime 文件 + 2 个 dated report。
- `commands.rs` 在 M5R08 包 1 采用 preimage-to-postimage 精确暂存；提交后工作树相对 HEAD 仍为 `59 insertions / 56 deletions`，证明旧 WIP 没被该提交吞入。
- 本轮开始后 Harness 于 `2026-08-18T14:01:51+08:00` 新生成 `docs/harness/usage/.turns/`（观察时 1 个文件，目录 canonical content hash `7f086db158802bd2164ca0c1e576cc03d07797441b205f76824926c035bb64bd`，539815 bytes）。它不是 verdict 的 34 项之一，因此不伪装成 opening WIP；它同样保持 untracked、原位保全并排除于 M5R08 候选。
- `.observed.json` / `.observed.jsonl` 是活动 Harness runtime 文件，后续 hook 可能继续改变其工作树 hash；表中 hash 只绑定上述观察时点，不把 mutable usage 文件纳入候选。
