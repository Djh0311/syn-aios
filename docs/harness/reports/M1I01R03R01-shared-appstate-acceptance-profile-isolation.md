# M1I01R03R01 共享 AppState 验收 profile 隔离报告

日期：2026-08-17

任务包：`M1I01R03R01`

被拒绝 candidate：`061eefee9291dbeddf792af6d78dc48bb5b0f8e5`

本报告记录独立验收拒绝 `061eefe` 的窄组合缺陷，以及本纠正包做了什么。它不是独立验收，也不把 M1 / M3 标成已解阻。既有 M1R03 / M3O01R01 报告保留为历史 candidate 证据。

## 1. 拒绝原因

`061eefe` 的隔离验收构造 `AppState::try_new_with_isolated_product_profile` 直接调用 `try_new_with_ordinary_product_ports`，因而安装 `M1ProjectIndexAuthorityHandle` 与 `M3ProjectRoleSessionAuthorityHandle`。这与验收 / 遗留保持未安装的合同冲突。

## 2. 纠正

- 共享 M4 组合新增显式 `SharedProductAuthorityProfile`。
- 普通 Tauri 产品选择 `OrdinaryInstalled`，继续安装两个权威。
- 隔离验收选择 `IsolatedUninstalled`，两个槽位保持 `None`。
- 遗留 `try_new()` 仍直接保持未安装。
- accessor 继续返回 `m1_project_index_unavailable` 与 `m3_project_role_session_authority_unavailable`。
- 定向测试调用真实 `try_new_with_isolated_product_profile`，不只用手写 `AppState` 字面量。

## 3. 未声称

- 不声称 M1 / M3 已解阻。
- 不创建活动 RoleSession，不改 M5R07 current。
- 不证明真实 App、renderer、Tauri command、provider、网络、发布或独立验收。

## 4. 证据范围

只证明离线 scoped checks。不证明真实 App、provider、网络、发布或独立验收。

- 被拒绝 candidate：`061eefee9291dbeddf792af6d78dc48bb5b0f8e5`
- 实现 commit：`ca413a967a2f6423ef2b62b9a8605f8f3567af3f`
- `git diff --check`：clean
- `cargo test --lib --offline -- m1_project_index -- --test-threads=1`：18 passed；0 failed
- `cargo test --lib --offline -- m3_project_role_session_authority -- --test-threads=1`：6 passed；0 failed
- `cargo check --lib --offline`：exit 0（既有 warning，无本包新增 error）
