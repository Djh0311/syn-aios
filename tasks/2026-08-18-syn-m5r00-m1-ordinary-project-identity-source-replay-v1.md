# SYN-M5R00 M1 普通项目身份显式来源重放

日期：2026-08-18

基线：`main@233ae9b5010118ded0d36cdc7343d47569991247`

执行者：Grok `grok-4.6 --reasoning-effort high`，本包是唯一窄任务。

## 目标

按 `docs/contracts/m1-ordinary-project-identity-source-replay-addendum-v1.md`，给真实普通 Tauri constructor 增加服务器侧显式项目身份来源重放：首次创建 / 迁移、重复幂等、重启同一解析；来源或 registry 不可用时阻止普通启动。不得读取 legacy index 自动登记，不得用 path 派生 canonical id。

## 写域

只允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，且只限普通 Tauri constructor 的登记调用点、必要 AppState 接线与直接相关定向测试接线
- `prototypes/productized-desktop-shell/src-tauri/src/m3_project_role_identity_source.rs`，仅当 M1 类型变化直接导致现有编译或 fail-closed 合同必须同步；预计不需要

不得修改合同、task、Harness、报告、Cargo 文件、`index_host_app_entrypoints.rs`、任何 `m5_*.rs`、`worker_report.rs`、`m6_*.rs`、stage-12、D0C04/D0C05、页面或其他路径。不得 Git add/commit/reset/stash/clean。

目标文件 opening SHA-256：

- `m1_project_index.rs`：`21cfc8693ea72188a4dd362fef186c7f6e0f94826d0a90e645c263d36930262f`
- `lib.rs`：`a0f39cf0e7f53d85178aca1eb457b11749a28cd9a594f02e8f95f270c1f69658`
- `m3_project_role_identity_source.rs`：`d13803d931aa01811f0a31dde64758f2f1c1f4eb8dfd2540629ecf414232ddaf`

## 实现要求

1. 在 M1 owner 内严格读取 ordinary app-data 根直接子文件 `m1-ordinary-project-identity-source-v1.json`；必须 regular file、非 symlink，serde `deny_unknown_fields`，严格校验 contract 字段、mode、非空和唯一性。
2. 用现有 `project_index` 跨进程锁执行一次 load/validate/replay/persist。未登记 alias 才 mint random UUID v4；已登记 alias 返回同一 id。相同来源重放不得增加 revision 或重写 registry。
3. 只在真实 `AppState::try_new_with_tauri_app_data_root` 上调用。调用必须发生在共享普通产品组合构造之前，使来源缺失 / 损坏先 fail closed。不要改变 `try_new_with_ordinary_product_ports`，避免给 isolated/M5 fixture 偷装来源重放。
4. 不删除或泛改旧 `project_id()`；本包只保证新普通启动身份入口不使用它，不把 path/alias/source ref 派生为 id。
5. 缺失 / 损坏来源与缺失 / 损坏 registry 都返回合同稳定码；不 fallback、不自动生成来源、不解析 `codex-index.json` 项目列表、不自动导入 legacy index。
6. 在 `m1_project_index.rs` 内增加定向 synthetic 测试，直接覆盖真实普通 Tauri constructor：首次登记、重复幂等（id/revision/registry bytes 不变）、AppState 重建同一解析、来源缺失、来源损坏、registry 缺失、registry 损坏拒绝。保留既有 isolated / manual authority 测试语义。
7. 保持冻结 owner：M1 只拥有 ProjectId / exact alias；不创建 Actor/Role/Scope/Permission/M3 RoleSession，不暴露 renderer/Tauri command/原始存储。

## 自检

先跑最窄测试与静态检查；允许在当前 worktree 做快速自检，但最终证据由主管在候选提交的 disposable checkout 生成。至少返回：

- `cargo test --lib m1_project_index --offline` 的命令、exit 和摘要
- `cargo check --lib --offline` 的命令、exit 和摘要
- `git diff --check` 的 exit
- 实际修改路径
- 对每项验收标准的代码 / 测试定位

现有受保护 dirty WIP 与未跟踪 M6 / schema / Harness usage 文件全部不碰、不归责。遇到需改写域外文件、冻结合同或 M5 fixture 才能继续，立即停止并报告，不自行扩大。
