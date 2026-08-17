# M3 项目角色会话权威增补合同 v1

状态：`ADDITIVE SUPPLEMENT / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M3O01`

## 0. 合同定位

本文件只补充 M3 对普通项目三角色 RoleSession 的服务器权威端口，不改写下列冻结合同，也不改变其 hash 或 schema 语义：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

若本增补与上述冻结合同、产品正本或当前用户指令冲突，以后者为准并停止相关施工。本增补不改 M5 文件，不授权 M6 / stage-15 / F0，不把 M5R07 从 current 换下。

## 1. Owner

ProjectSupervisor、Worker、IndependentReviewer 三个精确活动 `RoleSession` 的合法 provision / load / restore 只属于 M3 服务器权威端口。该端口安装在普通产品 `AppState` 组合里，不进入 renderer、不登记 Tauri command、不把原始 repository 交给调用方、不接受 M5 自造身份。

成功返回值是不可变的服务器精确信息，至少包含：canonical project id、role、actor、role session id、binding、permission snapshot、owner fingerprint、session revision。调用方不得回写这些字段。

## 2. Fail closed

以下任一成立，零业务签发、零恢复、零身份返回：

| 条件 | 稳定码 |
|---|---|
| 权威端口未安装 | `m3_project_role_session_authority_unavailable` |
| 不存在权威 canonical `ProjectId` 源 | `m3_canonical_project_id_source_unavailable` |
| 路径被当作 `ProjectId` | `m3_project_id_path_claim_rejected` |
| index locator 被当作 `ProjectId` | `m3_project_id_index_locator_claim_rejected` |
| scratch 被当作 `ProjectId` | `m3_project_id_scratch_claim_rejected` |
| M5 helper 被当作 `ProjectId` | `m3_project_id_m5_helper_claim_rejected` |
| 无可用会话 | `m3_project_role_session_unavailable` |
| 同一 role/project 出现多于一个活动候选 | `m3_project_role_session_duplicate` |
| 会话存在但非活动 / 未完成 | `m3_project_role_session_inactive` |
| 角色、scope 或身份错配 | `m3_project_role_session_mismatch` |
| binding 漂移 | `m3_project_role_session_binding_drift` |
| permission 漂移 | `m3_project_role_session_permission_drift` |

## 3. ProjectId 消费规则

`identity-scope-v1` 已规定：canonical server-resolved identifier 才是身份真源；`ProjectRootRef.normalized_root_alias`、cwd、route slug、label、caller boolean 只是 resolver 输入。`ProjectId.domain_owner` 是 `project_index`，opening status 是 `PARTIAL_LEGACY`。

本端口只消费已存在的权威 canonical `ProjectId` 源。下列对象明确不是该源，不得用于签发或恢复：

- `ProjectId::from_root` / `stable_id(project_root)` / `lib.rs` 的 path hash
- index JSON 的 `project_root` 查找或 isolated profile locator
- `scratch-` / `project:scratch-` 前缀
- `m5_m3_identity::official_project_id`、`resolve_project_id_from_index` 或任何 `m5:` 材料

2026-08-17 只读核验结论：仓内没有独立于 path / index locator / scratch / M5 helper 的权威 canonical `ProjectId` 源。因此本包不得猜测式签发项目三角色会话；已安装端口对 provision / load / restore 一律 fail closed。补齐真正的 `project_index` 权威是后续独立授权，不在本包发明。
