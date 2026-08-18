# Grok 窄包：M5R09 ordinary M1 enrollment 后端接线

本包只把已提交的 M1 enrollment authority 接到 ordinary AppState、一个显式 Tauri command 与真实 command graph。增补合同：`docs/contracts/m1-project-enrollment-addendum-v1.md`。当前产品基线包含提交 `62e75ab`。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

`commands.rs` 已有未归属 WIP，必须逐字保全其既有差异。不要格式化整个文件或整个 crate，不要修改其他文件，不要 git add/commit。

## 实现

1. ordinary Tauri 构造器把严格 `replay_ordinary_identity_source` 改为已提交的 `replay_ordinary_identity_source_if_present`：source 与 registry 都从未 established 时允许继续构造 `UNENROLLED` AppState；损坏、unsupported、symlink/非普通文件、不可读、registry present 但 source 缺失、established registry 丢失/损坏仍返回原有固定错误。不得创建空 source、registry 或 marker；M1 解析/治理写仍由现有 authority fail-closed。
2. 新增且只新增一个显式 command，建议命名 `enroll_m1_project_identity`。请求 DTO 必须 `deny_unknown_fields` 且唯一字段为 `project_root`；renderer 不能传 project id、source path/ref/revision、registry path 或 entry id。
3. command 每次从 `AppState` 已安装的固定 `index_path` 重新 `read_index`，按 `project_root` 精确匹配，必须恰好一条；零条或多条在任何 M1/source/registry 写前返回固定错误。不得调用 path-derived `project_id` 或导入 legacy identity。
4. 服务端组装 `exact_alias = exact root` 与 `source_ref = product-index:<exact-root>`，调用 `state.m1_project_index_authority()?.enroll_ordinary_project(...)`。返回最小 serializable DTO：canonical `project_id`、exact alias、source ref、source revision、registry revision、`created`/`already_enrolled` 状态。
5. 把 command 注册进 `workbench_command_handler!` 的真实 `tauri::generate_handler!` 图。不增加启动时、列表加载或其他自动登记路径。

## 直接测试

测试名统一前缀 `m5r09_m1_enrollment_backend_`，至少覆盖：

- missing source + never-established registry 的 ordinary 构造成功，且构造后 source/registry/marker 均不存在，M1 resolve/业务身份仍 fail-closed；
- corrupt、symlink source 与 established-missing registry 仍使构造失败且零覆盖；
- command 对固定 product index 中唯一 exact root 首次登记成功、重复调用同 id 且 source/registry revision 不增长，重建 AppState 后仍同 id；
- index 零匹配与重复 exact-root 多匹配均在 source/registry/marker 写前拒绝；
- command 从安装后的 server index 读取，而非调用方额外字段或旧 seed；请求 DTO 拒绝额外 `project_id`/`source_ref`；
- `command_registry.rs` 真实注册该 command；生产 command span 不含 `project_id(`、`stable_id(` 或前端 supplied canonical id。

测试只用临时 app-data、合成 index/tasks，不碰真实项目或资料。

## 交付验证

- `cargo test --lib --offline m5r09_m1_enrollment_backend_ -- --test-threads=1`
- `cargo test --lib --offline m1_project_index::tests:: -- --test-threads=1`
- `cargo check --lib --offline`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包必须保留；若未跑必须明确，主管会在候选流程补跑）
- 仓库根：`git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/commands.rs prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

不要问用户，不声称 leaf/stage/M5 完成。
