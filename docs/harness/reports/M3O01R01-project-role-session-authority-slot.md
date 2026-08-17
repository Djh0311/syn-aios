# M3O01R01 AppState 权威槽位边界报告

日期：2026-08-17

任务包：`M3O01R01`

被拒绝 candidate：`8b39d2b0f8a19b15085f369babf8da5eb29770f9`

本报告记录独立验收拒绝 `8b39d2b` 的窄合同缺陷，以及本纠正包做了什么。它不是独立验收，也不把 M3 标成已解阻。

## 1. 拒绝原因

`8b39d2b` 的 `m3_project_role_session_authority_missing_port_is_unavailable` 只构造 `Option<&dyn Port> = None` 和一个错误值。它没有经过真实 `AppState` 槽位。验收 / 遗留未安装组合因此没有被证明会返回 `m3_project_role_session_authority_unavailable`。

## 2. 纠正

- `require_installed_authority` 是服务器-only 槽位边界：缺失句柄返回 `m3_project_role_session_authority_unavailable`。
- `AppState::m3_project_role_session_authority_port` 改为 `Result`，走该边界。
- 普通产品构造仍安装权威；验收 / 遗留仍未安装。
- 定向测试覆盖 `AppState::try_new()`、未安装 fixture、已安装 fixture，以及普通产品构造。已安装端口对规范 `project:<uuid>` 仍返回 `m3_canonical_project_id_source_unavailable`。

## 3. 未声称

- 仓内仍没有可被本端口消费的普通权威 canonical `ProjectId` 源。
- 不声称 M3 已解阻，不创建活动 RoleSession，不改 M5R07 current。
- 不证明真实 App、renderer、Tauri command、provider、网络、发布或独立验收。

## 4. 证据范围

只证明离线 scoped checks。不证明真实 App、provider、网络、发布或独立验收。

- 被拒绝 candidate：`8b39d2b0f8a19b15085f369babf8da5eb29770f9`
- `cargo check --lib --offline`：exit 0（既有 warning，无本包新增 error）
- `cargo test --lib --offline -- m3_project_role_session_authority`：5 passed；0 failed
- `git diff --check`：clean

实现 SHA 另作 evidence binding commit。
