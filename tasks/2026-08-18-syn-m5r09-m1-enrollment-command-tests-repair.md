# Grok 返修窄包：enrollment command 测试去重

前一会话因 tool 回执延迟把同一 `m5r09_m1_enrollment_command_tests` module 连续追加三次，并在定向编译发现 `expect_err` 要求成功类型实现 `Debug`。本包只修新增尾部测试。

唯一允许修改：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`。

进入测试包前的受保护 blob 是 `7944ed100af99cd738d7814db192dd0829653cea`；blob 之后只应保留一份测试 module。不得修改 blob 中任何前文字节或 production core。

返修：

1. 保留第一份完整 `#[cfg(test)] mod m5r09_m1_enrollment_command_tests`，删除后面两份逐字重复 module，使 module 名和每个测试函数全文件各出现一次。
2. 不给 production DTO/request 补 `Debug`。把两个 command `Result<DTO,String>.expect_err(...)` 改为显式 `match`（`Ok(_) => panic!(...)`, `Err(error) => error`）；把两个 serde extra-field `expect_err` 改为 `assert!(serde_json::from_value::<...>(...).is_err())`。
3. 只在保留的测试 module 内修随后定向测试直接暴露的问题；不改 production core、旧 WIP 或其他文件。

验证：

- `rg -n '^mod m5r09_m1_enrollment_command_tests|^    fn m5r09_m1_enrollment_command_' commands.rs` 必须显示一个 module、四个测试。
- `cargo test --lib --offline m5r09_m1_enrollment_command_ -- --test-threads=1`
- `cargo test --lib --offline m5r09_m1_enrollment_ -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包保留；若未跑明确报告，候选流程会重跑）
- 本文件 `git diff --check`

不要 git 操作，不问用户，不扩大。
