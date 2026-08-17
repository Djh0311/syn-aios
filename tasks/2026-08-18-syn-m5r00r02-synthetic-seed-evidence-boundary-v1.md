# SYN-M5R00R02 普通 Tauri constructor 合成 seed 证据边界返修

日期：2026-08-18

基线：M5R00R01 working-copy candidate（尚未提交）

被拒原因：R01 已让真实普通 Tauri constructor 返回 `Ok(AppState)`，但成功测试调用 production wrapper 时会把仓库 bundled `codex-index.json` 物化进 synthetic profile；该历史静态 index 含真实个人路径。即使没有外部写，也不应把真实个人资料当本叶 fixture。本叶证据必须只用 synthetic source / index / tasks。

## 唯一目标

保留 production `try_new_with_tauri_app_data_root` 的真实 non-test caller 和完整行为，同时抽出最小私有 helper，使测试可以给同一普通 Tauri 组合路径传入 synthetic index/tasks seed。成功、幂等、重建和 fail-closed 测试不得读取或物化仓库 bundled index 内容。

## 写域

只允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅把 `try_new_with_tauri_app_data_root` 的现有“来源重放 + ordinary ports 构造”抽成带显式 seed 参数的私有 helper；production wrapper 继续传仓库 bundled paths，真实 `index_host_app_entrypoints.rs` caller 不变
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`：仅让新增 M5R00 测试创建和传入 synthetic index/tasks seed，并静态证明 production wrapper 调用同一 helper；不得读取 bundled index 内容

不得修改其他路径、合同、task、Harness、报告、Cargo、M5/M6 或受保护 WIP。不得 Git 写。

## 验收

1. production `try_new_with_tauri_app_data_root` 仍存在、仍是普通启动 caller，并把真实 bundled index/tasks 路径交给同一 helper；M1 来源重放仍发生在 shared composition 之前。
2. helper 不能是 `#[cfg(test)]` 的平行实现；production wrapper 和测试共用它。
3. 新增 M5R00 测试的 index seed 为最小 synthetic JSON，tasks seed 为 synthetic Markdown，均位于测试临时根；测试不读取、复制或解析仓库 bundled `codex-index.json`。
4. 成功、第二次启动、第三次重建仍全部 `Ok(AppState)`；id/revision/registry bytes 不变。missing/corrupt source/registry 仍 fail closed。
5. 静态断言：production M1 replay 不解析 legacy index；test helper 不引用 `../../index-kernel/codex-index.json`。
6. 跑 `cargo test --lib m1_project_index --offline`、`cargo check --lib --offline`、`git diff --check`，返回原始命令、exit、摘要与路径。

不进入 M5R07，不接真实资料/provider/账号，不做网络业务写或 Git 历史动作。
