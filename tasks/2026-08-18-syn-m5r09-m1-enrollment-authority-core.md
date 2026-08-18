# Grok 单文件窄包：M5R09 M1 enrollment authority core

只实现 `docs/contracts/m1-project-enrollment-addendum-v1.md` 的 M1 authority 核心。不要重读整个仓库或整个 `lib.rs`/`commands.rs`；不要处理 AppState、Tauri command 或前端。

唯一允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

必须实现：

1. 新增 ordinary enrollment request/outcome 与 authority 方法。request 只接受已由上层验证的 exact alias 和显式 source ref；canonical id 仍只由本 authority mint。
2. 在现有 source v1 schema 上做 source-first 持久化：同 alias+source 幂等；alias/source 冲突 fail-closed；新内容 source revision 单调递增；原子 temp+rename、file/dir sync；source 完成后再 replay registry并返回解析到的同一 id。
3. 并发相同请求只留一个 source entry、一个 registry project/id；不得死锁或产生重复 revision。
4. 新增“启动 replay-if-present”能力：只有 identity source 缺失且 registry/marker 从未 established 时返回明确 `Unenrolled`/`false`；source 损坏、symlink、unsupported、unreadable，或 established registry 丢失/损坏仍返回原错误。该方法本身不得创建任何文件。
5. 保留既有严格 `replay_ordinary_identity_source` 语义，不改 path-derived 禁令、read port 或其他阶段逻辑。

直接测试统一前缀 `m5r09_m1_enrollment_authority_`，至少覆盖首次登记、重复/重启同 id 与 revision、source 已写后 replay 恢复、并发相同登记、alias/source 冲突、missing-never-established 与 established-missing 的分型、损坏/symlink 零覆盖。

交付前至少运行：

- `cargo test --lib --offline m5r09_m1_enrollment_authority_ -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包必须保留的交节点前完整矩阵规则；若本轮时间不足必须明确未执行，主管不会据此接收节点）
- 仓库根 `git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

不要 git add/commit，不改文档/Harness，不声称 leaf/stage/M5 完成。最终只报告逐项实现、测试 exit/计数与仍未做的 AppState/command/frontend。
