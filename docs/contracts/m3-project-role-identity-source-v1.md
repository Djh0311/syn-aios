# M3 ProjectRoleIdentitySource 纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M3O03`

基线：`d26856fdda7eed84df685aa2d1ac37950355bd4c`

## 0. 合同定位

本文件只补充 M3 对普通项目三角色的服务器-only 身份源。它不改写冻结合同正文、hash 或 schema，也不改写下列既有纠正合同正文：

- `docs/contracts/m3-typed-project-id-identity-source-v1.md`
- `docs/contracts/m3-project-role-session-authority-addendum-v1.md`
- `docs/contracts/m3-project-role-session-authority-slot-boundary-v1.md`
- `docs/contracts/m1-project-index-ordinary-authority-v1.md`
- `docs/contracts/m1-m3-shared-appstate-acceptance-profile-isolation-v1.md`

下列 hash 仍是只读核验输入：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

`M3O02` 只独立接受为 fail-closed：同根合法类型化 `M1ProjectId` 在身份源不存在时返回 `m3_identity_source_unavailable`。本纠正补齐该源，不解阻 M3/M5，不授权 M6 / stage-15 / F0，不改 M5R07 current。

## 1. Owner

`ProjectRoleIdentitySource` 是 M3 服务器权威。它只安装在普通产品 `AppState` 组合里，经既有 M3 项目角色会话权威端口消费。隔离验收与遗留组合继续未安装。

该源：

- 不进入 renderer，不登记 Tauri command
- 不把原始源存储或 M3 repository 交给调用方
- 不是通用 identity resolver，不是 M4 / M5 helper，不是固定 local actor，不做 legacy import
- 不接受 raw path / root / alias / locator / cwd / M5 材料

## 2. 输入

每次 provision / load / restore 必须先用 M1 受限 verifier 按同一普通 app-data 根复核类型化 `M1ProjectId`，并只接受 `M3ProjectRole`：`ProjectSupervisor`、`Worker`、`IndependentReviewer`。

M1 稳定故障码必须在任何源写入、任何 M3 repository create/load/restore 之前原样传播。

未安装该源时，已安装权威在 M1 复核后继续返回 `m3_identity_source_unavailable`。

## 3. 持久化快照

每个精确 `(canonical project, role)` 至多一条身份记录。成功记录必须包含服务器解析且互不相同的：

- `actor_id`
- `role_ref`
- `scope_ref`
- `current_object_ref`
- `execution_channel`
- `permission_snapshot_ref`
- `owner_fingerprint`
- 源绑定的 `role_session_id`

`scope_ref` 与 `current_object_ref` 必须由该 canonical `M1ProjectId` 派生并精确绑定该项目。不得把 path、root、alias、locator、cwd 或 M5 材料写入这些字段。

`IndependentReviewer` 的 actor、role、fingerprint 必须与另外两角都不同。

## 4. Permission

permission snapshot 默认拒绝：`allow_capabilities` 为空，零 execution / provider / runner / grant 权威。它不是、也不得变成 M5 `ExecutionGrant`。

## 5. Provision 幂等与 PREPARED

首次 provision 必须先持久化 `PREPARED` 身份包，再经既有 M3 repository 签发 RoleSession。只有 source / binding / session 三者精确匹配后，该记录才变为可读。

同一 `(project, role)` 的再次 provision 必须幂等返回已匹配 view，不得另写第二条身份。

中断的 `PREPARED` 只能由同一 provision 输入完成。不同 project/role、或与已存储派生快照不一致的输入，不得完成、修复或覆盖该中断状态。

## 6. Load / restore

load / restore：

- 不得创建身份或会话
- 不得修复损坏 / 中断 / 漂移记录
- 不得调用 resume
- 不得接受调用方自选 `role_session_id`；只使用源绑定的会话

可读源缺失时，不得把 `PREPARED` 提升为成功 view。

## 7. Fail closed

| 条件 | 稳定码 |
|---|---|
| 权威端口未安装 | `m3_project_role_session_authority_unavailable` |
| 未接入 verifier | `m3_project_id_verifier_unavailable` |
| 未安装身份源 | `m3_identity_source_unavailable` |
| 源文件或可读记录缺失 | `m3_project_role_identity_source_missing` |
| 源损坏 | `m3_project_role_identity_source_corrupt` |
| 同一 project/role 多于一条 | `m3_project_role_identity_source_duplicate` |
| 完整性或指纹材料被改 | `m3_project_role_identity_source_tampered` |
| 源版本不匹配 | `m3_project_role_identity_source_version_mismatch` |
| 记录的 role/project 与请求或派生绑定不一致 | `m3_project_role_identity_source_role_project_mismatch` |
| 中断 PREPARED 与当前输入不一致 | `m3_project_role_identity_source_input_mismatch` |
| 记录尚未可读 | `m3_project_role_identity_source_not_readable` |
| binding / owner fingerprint 漂移 | `m3_project_role_session_binding_drift` |
| permission 漂移 | `m3_project_role_session_permission_drift` |
| 无可用会话 | `m3_project_role_session_unavailable` |
| 会话存在但非活动 | `m3_project_role_session_inactive` |

M1 同根复核失败继续使用既有 M1 稳定码。

## 8. 存储

优先使用独立于 M3 RoleSession schema 的版本化源存储。不得改冻结 M1 project index，不得改现有 M3 session schema。只有在没有其他办法做只读打开时，才允许给既有 M3 repository 增加只读窄开口。

## 9. 非目标

- 不改 M5 / M6 源、commands、renderer、stage-14 / M5R07 / authorization
- 不把 M3 / M5 标成已解阻或已完成
- 不证明真实 App、provider、网络、账号、发布或独立验收
