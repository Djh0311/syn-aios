# M1 project_index 与项目角色身份权威增补合同 v1

状态：`ADDITIVE SUPPLEMENT / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M1I01`

## 0. 合同定位

本文件只补充 `project_index` 对普通产品本地 app-data registry 的服务器权威，以及项目三角色的服务器-only 身份快照。它不改写下列冻结合同，也不改变其 hash 或 schema 语义：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

若本增补与上述冻结合同、产品正本或当前用户指令冲突，以后者为准并停止相关施工。本增补不改 M3 / M5 / M6 文件，不授权 M3 RoleSession provision、ExecutionGrant、M6 / stage-15 / F0，不把 M5R07 从 current 换下，不改 M3O01 未跟踪文档。

`identity-scope-v1` 已规定：`ProjectId.domain_owner` 是 `project_index`；canonical server-resolved identifier 才是身份真源；`ProjectRootRef.normalized_root_alias`、cwd、route slug、label、caller boolean 只是 resolver 输入。本包实现该 owner 的最小显式登记面，不改 opening status 字段，也不把 PARTIAL_LEGACY 升格为已完成迁移。

## 1. Owner

服务器-only `project_index` 是 canonical `ProjectId` 与 `ProjectRootRef` 精确别名绑定的唯一签发 / 解析 owner。服务器-only 项目角色身份权威是同一 owner 的只读身份面：它为已登记项目返回 `project_supervisor`、`worker`、`independent_reviewer` 的不可变身份快照。

该端口安装在普通产品 `AppState` 组合里，不进入 renderer、不登记 Tauri command、不把原始 registry 交给调用方。成功返回值只含不可变服务器精确信息。调用方不得回写这些字段。

本包不签发、不恢复、不加载 M3 `RoleSession`。M3O01 仍是项目三角色 RoleSession provision / load / restore 的独立 owner，只可消费本包已存在的 canonical `ProjectId`。

## 2. 显式登记

只有服务器侧显式登记可以把一个隔离项目写入本地 app-data registry。登记必须：

1. 签发不透明随机 canonical `project:<uuid>`，uuid 为 version-4 且大小写 / 连字符形式规范；
2. 为 `project_supervisor`、`worker`、`independent_reviewer` 各签发一条互不相同的不透明稳定 `actor:<uuid>` 与 `session-identity:<uuid>` 记录；
3. 若调用方同时提交精确别名，只把该别名存为 resolver 输入，绝不由其派生 ID；
4. 只原子替换 M1 registry 文件；不写 M3 / M4 / M5 store，不启动网络、provider 或真实 App。

未提供精确别名的登记仍然合法；该项目此后只能用已存储 canonical `ProjectId` 解析。

## 3. 解析与 fail closed

下列对象明确不是 canonical `ProjectId`，不得用于签发或当作 ID 查询成功：

- 文件系统路径、`ProjectId::from_root` / path hash、cwd
- index JSON 的 `project_root` / isolated profile locator / route slug
- `scratch-` / `project:scratch-` 前缀
- caller boolean
- `m5_m3_identity::official_project_id`、`resolve_project_id_from_index` 或任何 `m5:` 材料

只有预先登记且字节级相等的精确别名可以解析到其已存储 ID。legacy `codex-index.json` / workflow-state locator 记录永不自动导入。没有事先显式登记的普通 legacy 项目保持不可用。

| 条件 | 稳定码 |
|---|---|
| 权威端口未安装 | `m1_project_index_unavailable` |
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
| registry 损坏或不被支持 | `m1_project_index_registry_malformed` / `m1_project_index_registry_unsupported` |

## 4. 角色身份快照

成功的角色身份查询返回不可变服务器快照，至少包含：canonical project id、role、actor、session-identity、scope、current-object、channel、permission snapshot、owner fingerprint、revisions。`independent_reviewer` 的 actor、session-identity、role、fingerprint 必须与另外两角都不同。

这些 session identity 只绑定 least-privilege 无能力 session profile：`allow_capabilities` 为空，不签发 ExecutionGrant，不授予执行 / 写项目事实 / 外部副作用。执行授权仍属后续独立 owner。

## 5. 未来项目创建 / 迁移 owner

本包只覆盖显式隔离登记。普通产品里的项目创建、归档、以及把既有 `codex-index.json` / workflow-state locator 迁成 canonical `ProjectId`，属于后续必须由用户单独明确授权的独立 owner。

`identity-scope-v1` 已把 `ProjectId` / `ProjectRootRef` 的 `domain_owner` 定为 `project_index`；M5 阶段计划仍把“普通项目如何登记、归档与解析稳定 ProjectId”列为 HOLD。本包不猜测该未来 owner 的任务号、不发明 live 迁移、不把 PARTIAL_LEGACY opening 改写成已迁移。在该独立授权出现前，未显式登记的 legacy 项目保持不可用是正确行为。

## 6. 非目标

- 不实现 M3 RoleSession / Turn / Handoff
- 不实现 ExecutionGrant、runner、provider、真实消息
- 不暴露 renderer / Tauri command / 原始 registry
- 不改冻结合同正文、hash 或 schema
- 不改 M3 / M5 / M6 文件，不导入 legacy index
