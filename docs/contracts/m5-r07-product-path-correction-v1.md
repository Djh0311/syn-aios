# M5R07 产品路径修正补充合同 v1

- 版本：v1（2026-08-17）
- 状态：**FROZEN（M5R07 返修补充）**
- 关系：补充 M5R04 / M5R06 / M5R07 冻结合同；**不改 M1–M4 正文与 hash**。

## 规则

- 普通产品路径身份真源是 M3 `RoleSession` 不可变 view，只经 AppState 已安装的 `ProjectRoleSessionAuthority` provision/load；M5 不得创建/恢复 RoleSession，不得直开 M3 repository。
- 项目身份只经已安装的 M1 authority/read port 解析已显式登记的 canonical `M1ProjectId`；调用方 alias/canonical text 只是 resolver 输入。禁止 path hash、index locator、scratch claim、自动登记或 M5 helper fallback。
- 调用方不得提交 `role_session_id`；后续只消费服务器首次响应的 canonical project/binding。
- 共享 isolated product profile 保持 M1/M3 未安装，M5 open fail-closed；不得把旧 isolated Tauri full-loop receipt 写成 authority PASS。
- 普通真实 legacy `ProjectRecord` 目前没有可信 M1 普通项目创建/迁移 owner。UI `project_root` 只能作为预先显式登记的 exact alias resolver input；M5 不得自动登记或 fallback。synthetic ordinary 测试只证明预登记项目；部署 legacy UI 与 shared isolated full-loop 都不是 PASS。
- 渲染器不得选择或扩大 Grant `allowed_commands` / scope / policy。批准只绑定已存储提案上的 `authorized_action` 与服务器 policy。
- 普通项目 UI 必须用正式 command 逐步驱动：runtime receipt → worker report → independent review → result decision → summary。
- 隔离 helper 仅在 `SYN_M5R07_ISOLATED_ACCEPTANCE=1` 下可用。
- Summary consumer、source ref、deep-link 由服务器从 RoleSession 与已持久 source 派生；deep-link 必须能回源解析。
- 隔离 UI receipt 由后端 store 状态派生，不得回写前端自报 grant/spawn。

## 2026-08-18 已发生路径修正补记（M5R08）

- 此节只补记已发生且已独立核实的生产路径变化；不改上面的 M5R07 规则，也不改任何 M1–M4 冻结合同正文或 hash。
- `AppState::try_new_with_tauri_app_data_root` 通过 `try_new_with_tauri_ordinary_product_seeds` 取得普通 Tauri 产品种子。其中 tasks seed 已在提交 `99a5afc678949de50abd63876c57732024e53b18` 从相对 `CARGO_MANIFEST_DIR` 无效的 `../../tasks/README.md` 修正为仓库根的 `../../../tasks/README.md`。
- `CARGO_MANIFEST_DIR` 位于 `prototypes/productized-desktop-shell/src-tauri`；因此修正后的相对路径指向 `/home/synadmin/workspace/syn/tasks/README.md`，旧路径则落到 `prototypes/tasks/README.md`，仓库中不存在该种子。
- 该修正只决定普通 Tauri 构造器读取哪个仓库内 tasks seed；不授权真实项目写入，不改变 ProjectId owner，不提供 M6、壳采纳、部署或发布证据。
- M5R08 的 `m1_ordinary_identity_source_replay_has_tauri_caller_and_skips_legacy_index` 静态边界继续断言普通 wrapper 使用 `../../../tasks/README.md`；最终候选仍须在 disposable checkout 通过相关 M1 测试与 `cargo check --lib --offline`。
