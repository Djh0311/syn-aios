# M3 类型化 M1ProjectId 与身份源纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M3O02`

基线：`9bec690a4e8a3756d60195a875524390ff2d77e6`

## 0. 合同定位

本文件只补充 M3 对已类型化 `M1ProjectId` 的消费，以及 M1 受限复核能力。它不改写冻结合同正文、hash 或 schema，也不改写下列既有纠正合同正文：

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

`M1I01R03R01` 只证明普通 Tauri 安装 M1/M3、隔离验收与遗留保持未安装。本纠正不解阻 M3/M5，不创建活动项目三角色 RoleSession，不授权 M6 / stage-15 / F0，不改 M5R07 current。

## 1. 类型化请求

M3 provision / load / restore 的请求只携带类型化 `M1ProjectId` 与角色。raw `project_id_claim: String` 不再是合法请求字段。调用方不能把 path、index locator、scratch 或 M5 helper 字符串传入该 API。

## 2. M1 受限 verifier

`project_index` 增加一个仅服务器可见的受限能力：对**已经类型化**的 `M1ProjectId` 按同一普通 app-data 根复核。

该能力：

- 不登记别名
- 不把 registry 路径、文件句柄或存储内容交给 M3
- 不签发 ActorId、RoleRef、ScopeRef、CurrentObjectRef、ExecutionChannel、PermissionSnapshot 或 IdentitySnapshot

跨根类型化 ID 返回 `m1_project_id_foreign_root`。同根但 registry 缺席 / 缺失 / 损坏 / 未知 ID 分别返回既有 M1 稳定码：`m1_project_index_unavailable`、`m1_project_index_registry_missing`、`m1_project_index_registry_malformed`、`m1_project_id_unknown`。

## 3. 组合

受限 verifier 只接入普通 `AppState` 的 M3 权威。隔离验收与遗留组合继续未安装，accessor 仍为 `m3_project_role_session_authority_unavailable`。

已安装但未接入 verifier 时，M3 在任何 repository 动作前返回 `m3_project_id_verifier_unavailable`。

## 4. 身份源仍不可用

同根合法类型化 `M1ProjectId` 通过复核后，M3 仍不得签发或恢复 RoleSession。合法 ActorId、RoleRef、Scope / CurrentObject / ExecutionChannel、ServerResolvedBinding 与 PermissionSnapshot 源尚不存在。稳定新码：`m3_identity_source_unavailable`。

禁止用下列材料伪造这些源：

- M5 helper / `m5_m3_identity`
- path / locator / scratch
- 通用 `resolve_identity`
- M4 Secretary 固定身份
- 固定 local actor

结果：零 M3 会话写入，无成功 view。

## 5. 非目标

- 不改 M5 源、renderer、Tauri command、M3 repository/schema、M6、壳文档
- 不改 stage-14 / M5R07 / authorization
- 不把 M3 / M5 标成已解阻或已完成
