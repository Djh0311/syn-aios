# Grok 返修窄包：M5R09 enrollment authority tests and completion

前一 Grok 会话因 `max turns` 退出，已在 `m1_project_index.rs` 写入 enrollment 类型、source-first persist、locked replay 与 replay-if-present，但没有新增任何 `m5r09_m1_enrollment_authority_` 测试，也留下 `status` unused-assignment warning。主管已独立跑 `cargo check --lib --offline`，exit 0；这不构成交活。

唯一允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

只做以下返修：

1. 复核现有半成品严格满足增补合同：source 写入先于 registry replay；同 alias+source 幂等；alias/source 双向冲突；source revision 只在新增时增长；同锁并发不重复；missing source 只有 registry/marker 从未 established 时可返回 `Unenrolled`；损坏/symlink/unsupported/unreadable 与 established registry 异常仍 fail-closed。发现同范围缺陷就最小修正。
2. 消除本包新增的 `status` unused-assignment warning，不整理其他既有 warning。
3. 新增前缀为 `m5r09_m1_enrollment_authority_` 的直接测试，至少覆盖：首次登记；重复与 authority 重开后同 id/source+registry revision 不增长；source 已存在而 registry 未落地时 replay 恢复；并发相同请求只有一个 entry/id；alias 冲突与 source-ref 冲突零改写；missing-never-established 返回 Unenrolled 且零文件；source 缺失但 registry present/established-missing 拒绝；损坏与 symlink source 零覆盖。
4. 不改 AppState、commands、前端或其他文件；不要 git add/commit。

交付前运行并报告：

- `cargo test --lib --offline m5r09_m1_enrollment_authority_ -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（本产品包必须保留的完整矩阵；若未跑必须明确）
- 仓库根 `git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

不要重读整个仓库，不问用户，不声称 leaf/stage/M5 完成。
