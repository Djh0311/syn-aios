# Grok 窄包：M5R09 ordinary identity source 平台正确 no-follow

本包只修复 ordinary identity source 打开路径把 Linux 常量错误复用于全部 Unix 的问题。当前主机仅安装 `x86_64-unknown-linux-gnu`；必须保留 Linux 行为，并为 Apple/BSD 建立明确、互斥、可静态复核的目标平台常量/cfg，但不得声称已在 macOS/BSD 实机运行。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

该文件当前为干净 tracked 文件。不要修改 `Cargo.toml`、`Cargo.lock` 或其他文件；不要 git 操作；不要格式化整文件。

## 实现约束

1. 删除将 `LINUX_O_NOFOLLOW`、`LINUX_O_NONBLOCK`、`LINUX_ELOOP` 用于整个 `cfg(unix)` 的做法。Linux/Android 与 Apple/BSD 必须由互斥、目标平台明确的 cfg 提供各自正确的 open flags 和 symlink-open errno；macOS/BSD 绝不能引用 Linux 常量名或数值。不要靠运行时 OS 字符串分支。
2. 不新增依赖。若使用本文件内常量，名称须是平台中性的接口并由目标 cfg 赋值；注释清楚写出来源平台族和边界。保持当前 Linux x86_64 的 `O_NOFOLLOW | O_NONBLOCK` 打开、final-component symlink 映射 malformed、FIFO/non-regular fail-closed 行为。
3. 非 Unix 路径不得误把 Linux `ELOOP=40` 当作平台 errno。现有非 Unix 逻辑如不能在本包内以标准库安全证明 no-follow，则必须维持 fail-closed/不虚构支持，并把边界写进代码注释；不要扩大到其他模块或依赖。
4. 更新本文件中直接锁定旧 Linux 常量名的测试，使其验证生产 helper 使用平台中性接口和目标 cfg；增加/调整能机械区分 Linux 与 Apple/BSD 常量族的编译期或单元级反例。不要把纯字符串扫描作为唯一证据；当前 Linux symlink/FIFO 行为测试仍必须实际执行。
5. 不改变 enrollment、registry、alias、revision、AppState 或 command 语义；不接真实项目资料。

## 交付验证

- `cargo test --lib --offline m5r08_m1_source_symlink_fails_closed_without_registry_or_marker -- --exact --test-threads=1`
- `cargo test --lib --offline m5r08_m1_source_fifo_fails_closed_without_blocking -- --exact --test-threads=1`
- 本文件新增/调整的平台常量直接测试（精确测试名）
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`
- `git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

如测试过滤名因 Rust module 路径需要后缀匹配，可用唯一后缀运行并报告实际计数。完成后立即退出；不要读取 harness、历史或无关文件，不接真实资料、账号、provider、凭据或外部业务。
