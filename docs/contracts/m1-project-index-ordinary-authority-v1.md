# M1 普通 project_index 权威纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M1I01R03`

## 0. 合同定位

本文件只补充普通产品 `AppState` 上的服务器-only M1 登记 / 读权威。它不改写下列冻结合同，也不改变其 hash 或 schema 语义：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

`docs/contracts/m1-project-index-and-role-identity-addendum-v1.md` 里把 `project_index` 写成项目三角色身份快照 owner、并为三角色签发 `actor:` / `session-identity:` 的旧措辞 **不再具有权威**。`M1I01R01` 已删除该越权实现。本包不恢复它，也不改写那份旧增补正文。

本纠正不声称 M1 / M3 已解阻，不创建 M3 `RoleSession`，不授权 M5 / M6 / stage-15 / F0，不改 M5R07 current。

## 1. Owner

`project_index` 只拥有 canonical `ProjectId` 与精确别名的显式签发 / 解析。ActorId、RoleRef、ScopeRef、CurrentObjectRef、ExecutionChannel、PermissionProfile、PermissionSnapshotRef、IdentitySnapshot 与 M3 `RoleSession` 仍归其冻结 owner。M3 还不消费本端口。

## 2. 普通 AppState 权威边界

普通产品 `AppState` 安装服务器-only `M1ProjectIndexAuthorityPort`：

- `register_exact_alias`：显式、带类型的服务器内部登记
- `resolve_canonical_project_id` / `resolve_exact_alias`：读取已登记 id

该边界返回 `Result`。验收 / 遗留组合保持未安装；未安装槽位返回 `m1_project_index_unavailable`。

启动、读打开、普通构造都不得写 registry。只有显式 `register_exact_alias` 可以把项目写入本地 app-data registry。不得从下列来源自动登记：

- legacy `codex-index.json` / workflow-state
- root / path / cwd
- index locator / slug
- scratch
- UI / M5 helper

不得把原始 registry 文件或存储句柄交给调用方。

## 3. 登记与重建

一次成功的精确别名登记必须：

1. 签发不透明随机 canonical `project:<uuid>`（version-4，规范连字符形式）
2. 把精确别名只存为 resolver 输入，不由其派生 ID
3. 原子替换 M1 registry 文件
4. 普通 `AppState` 用同一 app-data 根重建后，别名与 canonical id 解析到同一 `M1ProjectId`

已建立后丢失 registry 仍返回 `m1_project_index_registry_missing`，不得静默重建。从未建立的空状态读操作返回 `m1_project_index_unavailable`。

## 4. Fail closed

| 条件 | 稳定码 |
|---|---|
| 权威端口未安装 / 从未建立的空读 | `m1_project_index_unavailable` |
| 路径被当作 `ProjectId` | `m1_project_id_path_claim_rejected` |
| index locator / slug 被当作 `ProjectId` | `m1_project_id_index_locator_claim_rejected` |
| scratch 被当作 `ProjectId` 或登记源 | `m1_project_id_scratch_claim_rejected` |
| M5 helper 被当作 `ProjectId` 或登记源 | `m1_project_id_m5_helper_claim_rejected` |
| caller boolean 或其他畸形输入 | `m1_project_id_malformed` / `m1_alias_malformed` |
| 规范 `project:<uuid>` 未登记 | `m1_project_id_unknown` |
| 别名未登记 | `m1_alias_unknown` |
| 同一精确别名重复登记 | `m1_alias_duplicate` |
| 已建立 registry 缺失 | `m1_project_index_registry_missing` |
| registry 损坏 | `m1_project_index_registry_malformed` |
| registry 版本不受支持 | `m1_project_index_registry_unsupported` |

## 5. 非目标

- 不实现角色身份、permission snapshot、IdentitySnapshot
- 不实现 M3 RoleSession / Turn / Handoff
- 不让 M3 消费本端口
- 不暴露 renderer / Tauri command / 原始 registry
- 不改冻结合同正文、hash 或 schema
- 不改 M3 / M5 / M6 文件
- 不把 M1 / M3 标成已解阻
