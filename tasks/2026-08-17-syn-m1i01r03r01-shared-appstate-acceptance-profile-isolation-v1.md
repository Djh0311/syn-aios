# SYN-M1I01R03R01 共享 AppState 验收 profile 隔离

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 独立验收拒绝 `061eefe` 后的窄组合边界纠正

总线判定：普通产品仍应安装 M1 / M3。隔离验收路由 `try_new_with_isolated_product_profile` 不得因复用普通 M4 构造而安装这两个权威。

## 授权边界

- 主工作树有用户 WIP：不 reset / stash / clean / `git add -A` / `git add .` / `commit -a`。
- 不改 M1 登记语义、M3 request / API 所有权、角色会话、M5、renderer、Tauri command、壳文档、stage-14、M5R07 leaf、authorization、M6。
- 不改写冻结 M1–M4 正文 / hash / schema。
- 不改写既有 M1R03 / M3O01R01 报告与 unfinished note。
- 不声称 M1 / M3 已解阻、M5 已通过、stage 已关闭、M6 已激活或真实 App 证据。

## 产品结果

普通 Tauri 产品继续安装两个权威端口。隔离验收与遗留组合保持未安装；其 Result accessor 返回稳定不可用码。profile 选择必须显式。测试必须走真实隔离验收构造函数，或同一显式 profile 分支。

## 本包精确路径

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`（仅定向测试）
- `prototypes/productized-desktop-shell/src-tauri/src/m3_project_role_session_authority.rs`（仅定向测试）
- `docs/contracts/m1-m3-shared-appstate-acceptance-profile-isolation-v1.md`
- `tasks/2026-08-17-syn-m1i01r03r01-shared-appstate-acceptance-profile-isolation-v1.md`
- `docs/harness/reports/M1I01R03R01-shared-appstate-acceptance-profile-isolation.md`
- `docs/harness/unfinished/M1I01R03R01-shared-appstate-acceptance-profile-isolation.md`

## 验证

- `git diff --check`
- `cargo test --lib --offline -- m1_project_index -- --test-threads=1`
- `cargo test --lib --offline -- m3_project_role_session_authority -- --test-threads=1`
- `cargo check --lib --offline`

不 push / merge / rebase / deploy / release。
