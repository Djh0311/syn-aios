# M5R09 受保护未归属 WIP 分层归责 v1

观察时间：`2026-08-18T18:31:50+08:00`

基线：`M5R08-protected-wip-attribution-v1.md` 的 34 项 opening WIP。统一 disposition 为 `PRESERVE_IN_PLACE / EXCLUDE_FROM_M5R09_CANDIDATE / NO_CLEAN / OWNER_REVIEW_REQUIRED`。本报告不推断作者，不暂存、覆盖或删除这些载体。

## 活动 Harness runtime（不承诺内容 hash）

| Git | path | 观察与漂移边界 |
|:---:|---|---|
| ` M` | `docs/harness/usage/.observed.json` | 观察时存在；hook 可继续改写 |
| ` M` | `docs/harness/usage/.observed.jsonl` | 观察时存在；hook 可继续追加 |
| `??` | `docs/harness/usage/host-events.json` | 观察时存在；runtime 可漂移 |
| `??` | `docs/harness/usage/host-health.json` | 观察时存在；runtime 可漂移 |
| `??` | `docs/harness/usage/.turns/` | M5R08 后由 Harness 新生成；观察时 1 个文件；runtime 可漂移 |
| `??` | `docs/harness/reports/2026-08-18-01a01376-2915-7c40-acf9-899811b2da98-01a01376-29b2-7f22-966a-ce82ce91acab.md` | M5R09 中由 Harness 新生成；不冒充 opening WIP 或候选内容 |

以上活动载体只绑定路径和观察时点，不给出或承诺内容 hash；它们全部留在工作树、排除于候选。

## 静态受保护 WIP（承诺观察时点内容 hash）

`SAME` 表示与 M5R08 报告的观察 hash 相同。`commands.rs` 的整文件 hash 因获准 M5R09 提交叠加而变化；候选 HEAD 之外的受保护残余仍为 `59 insertions / 56 deletions`，没有被 `git add -A` 或整文件暂存吞入。

| Git | SHA-256 | bytes | 对 M5R08 | path |
|:---:|---|---:|---|---|
| ` M` | `830722a25e6c702b500caa22f45d6c1cf9fe448f96667062932b85e186a610fa` | 46555 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/acceptance_runtime_profile.rs` |
| ` M` | `8979c93a95e6d0f5167b962b5531632ca3a785d5fb067f3485eee7b0b1a66a36` | 35571 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs` |
| ` M` | `08120086ac64c7ff6583d71c3211aa8ca97088a8dd903af3f0dbd1238cc53cd8` | 98061 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs` |
| ` M` | `5dbf6715a87d3083978d670a54eb9b48edf374e65384d077e12995ef798711da` | 399241 | M5R09 allowed overlay; residual 59+/56- | `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` |
| ` M` | `7d3a29371522af67d0c55ccb717d908926e64b93592e9760137e8abc2a1d8d4a` | 32081 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs` |
| ` M` | `5bb0df5c1f0fa6604f8af27da0dd28f63f954612d36371da80b174a1eca70331` | 4930 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m2_clock.rs` |
| ` M` | `210ec1a09aba9c3a806ae440dd98b479bda77b045ffd70857b847390e9bd4686` | 73722 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m2_r4_reference_slice_driver.rs` |
| ` M` | `8018820a59b26710504a005d9f1a4b6b9ad60d2eaf727d69a0c8ffbd2a826266` | 87489 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m2_update_work_item_state.rs` |
| ` M` | `a3e4ec501ef11a3a10b4f194d18a65d752e38a789f1b288efb285b1c7885c292` | 85430 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs` |
| ` M` | `2009f13457610229ada4a347d885c2280c03bc4dc9ab3f8760eaf4556a5698de` | 13261 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m4_source_dispatcher.rs` |
| ` M` | `dd839bb2c90b45b9ee27492d2d1c23bff2ab49e7b8805b0e5a6edd73e00dfe0e` | 16344 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/event_audit_boundary.rs` |
| ` M` | `22a3a7746326899ad13eea64dbbeb302ab80143bb705fd5719e34ca26138479a` | 44912 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/execution_grant.rs` |
| ` M` | `fedbf89e2e18701d96b52800d26afbe5c64c612aad0f8d5d6e0f35564742f8be` | 56218 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/identity_kernel.rs` |
| ` M` | `5aa8d79872844857cd2a1e149ed7cf6ba1e12fb482cba42902fbc5b250827ccb` | 11282 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/path_guard.rs` |
| ` M` | `3a537991ebe76a7030059d274d6bfeeaca3759c651cf66dc5a64d23c9d63d0a8` | 18183 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/storage.rs` |
| ` M` | `0be6791caedcf756445113087f22c21de0ec68cd4b832ccdc296b319a81609c3` | 39570 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs` |
| ` M` | `2e122e1e5b861534090c41988c562db533aa7b413e2ae71374b8366d200a4954` | 35802 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/ordinary_product_storage_bootstrap.rs` |
| ` M` | `d5c809929ca33afed309d81db3a30bb447345d399fcb72d7ae908fdacb91db2d` | 61234 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs` |
| ` M` | `2bd100fcb5f24bb26eb3a1927b98f7ebf1dd0a19145d1c01163463f3fec97e0d` | 80318 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs` |
| ` M` | `10605b2e2ddf7f8504fb30971f6c1e9936d8ddabbbeb63f6543a97f00c81c061` | 28466 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs` |
| ` M` | `db54ca720b9d6f3f63b446145f2ff14f94992120c1bac8fcbcf1b83628ded2ad` | 43904 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema_m2.rs` |
| `??` | `4643b4d6f257a568f8a3755f10ecfbf4cdf9df047b374104c7119542a678a108` | 1358 | SAME | `docs/harness/reports/2026-08-18-01a010f4-5eea-77c1-979d-d03cadd2a3a4-01a010f4-5f75-7de3-a0f2-f9874eedb5b9.md` |
| `??` | `08240abe0afac02413adf61a84f03c1c8fd138a2e71d7f8d3ddfd4c2f8a0eb8a` | 3087 | SAME | `docs/harness/reports/2026-08-18-01a0130d-aa03-7992-a314-1a03408c729e-01a0130d-aa7e-7390-a7bf-93c389047cc2.md` |
| `??` | `7e51a7ed92547e6c96f8d37d0ff7de836e9ee5b6102b1c6ba06ae075207c2a15` | 116888 | SAME | `prototypes/productized-desktop-shell/src-tauri/gen/schemas/linux-schema.json` |
| `??` | `620faa27056e7cfac6fb119731c62c04eb5b65d7a4e5641b5944bea4af76b58e` | 13544 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_cross_project_query.rs` |
| `??` | `2c576d9b6f89e97f8e5e5754071a3e3623f3e5d8920a93b3a78afc6a46bb276d` | 10828 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_global_supervisor_session.rs` |
| `??` | `6cd604b4ebb483e1aba268e617d8935d0c15d4dc7ee124fe1f636b455547d84e` | 10499 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_member_directory.rs` |
| `??` | `147bd08e35609daaf0f4bb979cec73dfd305f259c9b8504576801504a2b95e30` | 10098 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_member_directory.rs.bak` |
| `??` | `6155c26a9c819d14be4ca6f352633faa498e8f3e0c688f67f12eec023c2bd1d6` | 16002 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_organization_identity.rs` |
| `??` | `7db42ba1010ee8b6f2bc13d9109c2b2f0dd3d74568c66cc15da31015a81e261f` | 6084 | SAME | `prototypes/productized-desktop-shell/src-tauri/src/m6_temporary_agent_history.rs` |

结论：29 个可直接同比的静态路径保持 M5R08 观察 hash；`commands.rs` 只有获准 M5R09 提交造成整文件 postimage 变化，候选外旧 WIP 仍作为 59+/56- 留在 working tree。6 个 `m6_*.rs` 继续未跟踪，未激活 M6。
