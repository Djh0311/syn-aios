# M1 project_index 基座纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M1I01R01`

被拒绝 candidate：`88cb02e3426ede7b9500d3b6c6263720877c3c11`

## 0. 为何拒绝 88cb02e

`identity-scope-v1` 冻结的 `domain_owner` 是：

| 对象 | 冻结 owner |
|---|---|
| `ProjectId` | `project_index` |
| `ProjectRootRef` | `project_index` |
| `ActorId` | `identity_scope_kernel` |
| `RoleRef` | `role_catalog` |
| `ScopeRef` | `identity_scope_kernel` |
| `CurrentObjectRef` | `identity_scope_kernel` |
| `ExecutionChannel` | `identity_scope_kernel` |
| `PermissionProfile` | `permission_policy_catalog` |
| `PermissionSnapshotRef` | `permission_snapshot_authority` |
| `IdentitySnapshot` | `identity_scope_kernel` |

`88cb02e` 让 `project_index` 创建、持久化并返回后八类对象，等于改写冻结所有权。把它改名为“增补身份权威”仍然越权。本纠正删除该实现，不把它改名后留下。

本文件不改写冻结合同正文、hash 或 schema。下列 hash 仍是只读核验输入：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

本纠正不声称 M3O01 已解阻，不创建 M3 `RoleSession`，不授权 M5 / M6 / stage-15 / F0，不改 M5R07 current。

## 1. Owner

`project_index` 只拥有 canonical `ProjectId` 与 `ProjectRootRef` 的签发和精确别名解析。角色身份、permission snapshot 与 IdentitySnapshot 仍归上表冻结 owner；后续另立与 owner 对齐的独立包，不在本包发明。

## 2. 读端口与登记面

消费者只看见 `M1ProjectIndexReadPort`：canonical id 解析、精确别名解析、`ProjectRootRef` 精确匹配。普通 `AppState` 只安装该读端口。

登记 / mint 是服务器内部写面：

- 不进入 renderer
- 不登记 Tauri command
- 不安装到普通 `AppState`
- 不得被 M5 调用

读端口打开已存在但损坏 / 不受支持 / 已建立后缺失的 registry 时 fail closed，不得把丢失的 registry 写成空白新文件。从未建立过的普通产品保持读端口未安装；未登记与 legacy 输入保持不可用。

## 3. 持久化

本地 app-data registry 只保存：

- `schema_version`
- `registry_revision`
- `projects[]` 的 `project_id`、`exact_alias`、`resolver_revision`

`88cb02e` 的 v1 registry（含 roles / actor / scope / permission 等）不受支持，不得迁移或自动导入。

登记的临界区必须跨进程串行覆盖：load → validate → duplicate-check → mint → persist。同一别名不能两个写者都成功；不同别名不能互相丢失更新。`registry_revision` 使用 checked 加法，禁止 saturating。每一个仍保留字段都必须校验。原子 rename 之后，目录 open / sync 失败必须传播。

## 4. Fail closed

| 条件 | 稳定码 |
|---|---|
| 读端口未安装 | `m1_project_index_unavailable` |
| 路径被当作 `ProjectId` | `m1_project_id_path_claim_rejected` |
| index locator / slug 被当作 `ProjectId` | `m1_project_id_index_locator_claim_rejected` |
| scratch 被当作 `ProjectId` | `m1_project_id_scratch_claim_rejected` |
| M5 helper 被当作 `ProjectId` | `m1_project_id_m5_helper_claim_rejected` |
| caller boolean 或其他畸形输入 | `m1_project_id_malformed` / `m1_alias_malformed` |
| 规范 `project:<uuid>` 未登记 | `m1_project_id_unknown` |
| 别名未登记 | `m1_alias_unknown` |
| 同一精确别名重复登记 | `m1_alias_duplicate` |
| 已知 `ProjectId` 与所报别名不一致 | `m1_alias_mismatch` |
| `resolver_revision` 与已存储值不一致 | `m1_resolver_revision_stale` |
| 已建立 registry 缺失 | `m1_project_index_registry_missing` |
| registry 损坏 | `m1_project_index_registry_malformed` |
| registry 版本不受支持 | `m1_project_index_registry_unsupported` |
| 修订溢出 | `m1_project_index_revision_overflow` |

## 5. 非目标

- 不实现或不声称实现角色身份、permission snapshot、IdentitySnapshot
- 不实现 M3 RoleSession / Turn / Handoff
- 不实现 ExecutionGrant、runner、provider、真实消息
- 不暴露 renderer / Tauri command / 原始 registry
- 不改冻结合同正文、hash 或 schema
- 不改 M3 / M5 / M6 文件
- 不把 M3O01 标成已解阻
