# Grok 窄包：M5R09 UNENROLLED ordinary AppState

只实现增补合同的首次启动语义，不实现 enrollment command 或前端。当前基线包含 M1 enrollment authority 提交 `62e75ab`；current leaf 已把唯一必要相邻测试路径纳入写域。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m5_ordinary_control_acceptance.rs`

这三文件当前均无未归属工作副本差异。不要修改其他文件，不要格式化整个文件或 crate，不要 git add/commit。

## 实现与测试

1. 在 ordinary Tauri 构造器中，把严格 `replay_ordinary_identity_source` 调用替换为已提交的 `replay_ordinary_identity_source_if_present`；忽略 `Replayed`/`Unenrolled` 的成功枚举值，只传播真实错误。
2. source 与 registry 都从未 established 时允许构造 AppState；不得因此创建 ordinary identity source、M1 registry 或 established marker。构造后的 M1 authority 已安装，但 exact-alias/canonical resolve 在未登记时仍返回固定 unavailable/unknown 错误，零 M1 业务写。
3. corrupt、unsupported、symlink/非普通文件、unreadable、registry present 但 source 缺失、established registry missing/corrupt 仍按原固定错误拒绝构造，不降级。
4. 只更新两条被新合同直接替代的旧反例：
   - `m1_project_index.rs` 中原 `m1_ordinary_identity_source_missing_and_corrupt_fail_closed_without_registry_write`：missing 子场景改为 `m5r09_m1_enrollment_backend_...` 前缀的 UNENROLLED 构造成功、零 M1 文件、resolve fail-closed；同一测试后续 corrupt/unsupported/invalid-mode 拒绝覆盖保留。
   - `m5_ordinary_control_acceptance.rs` 中原 `constructor_rejects_missing_identity_source`：改为 `m5r09_m1_enrollment_backend_...` 前缀，断言构造成功、M1 resolve fail-closed、source/registry/marker 不存在。不得改本文件其他 M5 执行链、helper 或 acceptance driver。

## 交付验证

- `cargo test --lib --offline m5r09_m1_enrollment_backend_ -- --test-threads=1`
- `cargo test --lib --offline m1_project_index::tests:: -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包必须保留；若未跑必须明确，候选流程仍会重跑）
- 仓库根对三个允许文件执行 `git diff --check -- <paths>`

不要读取无关文件，不问用户，不声称 leaf/stage/M5 完成。
