# SYN-M5R00R01 真实普通 Tauri constructor 成功证据返修

日期：2026-08-18

基线：M5R00 第一轮 working-copy candidate（尚未提交）

被拒原因：第一轮新增来源重放本身通过，但成功类测试在 `AppState::try_new_with_tauri_app_data_root` 因 `lib.rs` 既有 bundled tasks seed 使用错误的 `../../tasks/README.md` 而失败后，回退到 `try_new_with_ordinary_product_ports` 读取 registry。它只证明真实 constructor 调用了重放，不证明普通 Tauri AppState 成功构造并取得正式身份，不满足 leaf 的普通启动路径完成标准。

## 唯一目标

让 synthetic 普通 Tauri constructor 在显式来源存在时真实返回 `Ok(AppState)`，并由该返回值直接证明首次登记、重复幂等和重建同一解析。不得用“先让真实 constructor 失败以产生 registry，再改走内部 constructor”的替代证明。

## 写域

只允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅纠正 `try_new_with_tauri_app_data_root` 的 bundled tasks seed，使它与仓库真实 `tasks/README.md` 对齐；不得改其他 constructor / AppState 行为
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`：仅移除成功类测试的 fallback，并把断言改为真实 Tauri constructor 必须 `Ok`

不得修改任何其他产品文件、合同、task、Harness、报告或 Cargo；不得 Git 写。

## 验收

1. `test -f prototypes/productized-desktop-shell/src-tauri/../../../tasks/README.md` 为真，普通 Tauri constructor 使用该真实 repo task seed；不得新建替代 seed。
2. `app_state_after_ordinary_tauri_constructor` 不得在 `Err` 后调用 `ordinary_app_state`；成功类测试必须对真实 constructor 的 `Ok(AppState)` 直接解析 M1 alias。
3. 首次登记、相同来源第二次启动、第三次 AppState 重建均由真实 constructor 成功完成；project id、registry revision 与 bytes 保持相同。
4. 缺失 / 损坏来源及缺失 / 损坏 registry 仍直接由真实 constructor fail-closed。
5. 跑 `cargo test --lib m1_project_index --offline`、`cargo check --lib --offline`、`git diff --check`，返回命令、exit、摘要和实际路径。

仍不得进入 M5R07，不得修改受保护 WIP，不得真实数据 / provider / 网络业务写。
