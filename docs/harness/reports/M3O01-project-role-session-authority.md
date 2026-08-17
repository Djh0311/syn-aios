# M3O01 项目角色会话权威报告

日期：2026-08-17

任务包：`M3O01`

本报告记录独立 M3-owner 窄纠正包做了什么。它不是独立验收，也不把 M3O01 标成已解阻。

## 1. 做了什么

- 新增服务器-only `m3_project_role_session_authority.rs`：ProjectSupervisor / Worker / IndependentReviewer 的 provision / load / restore 端口。
- 普通 `AppState` 构造安装该端口。验收 / 遗留组合保持未安装。
- 未安装返回 `m3_project_role_session_authority_unavailable`。
- 已安装端口不消费 M1 读端口、path、index locator、scratch、M5 helper 作为签发源。
- 伪称 path / locator / scratch / M5 helper 为 `ProjectId` 时返回文档中的拒绝码。
- 其余 claim 返回 `m3_canonical_project_id_source_unavailable`。零业务签发。

## 2. 未声称

- 仓内仍没有可被本端口消费的普通权威 canonical `ProjectId` 源。
- 不声称 M3 已解阻，不创建活动 RoleSession，不改 M5R07 current。
- 不证明真实 App、renderer、Tauri command、provider、网络、发布或独立验收。

## 3. 证据范围

- `cargo check --lib --offline`
- 定向 `cargo test --lib --offline -- m3_project_role_session_authority`
- `git diff --check`
