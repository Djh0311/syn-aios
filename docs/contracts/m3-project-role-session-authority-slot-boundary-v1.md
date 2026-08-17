# M3 项目角色会话权威槽位边界纠正 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M3O01R01`

被拒绝 candidate：`8b39d2b0f8a19b15085f369babf8da5eb29770f9`

## 0. 为何拒绝 8b39d2b

增补合同已规定：权威端口未安装时返回 `m3_project_role_session_authority_unavailable`。

`8b39d2b` 只在测试中构造该错误值，并断言一个手写 `Option<&dyn Port> = None`。那不是 `AppState` 槽位边界。验收 / 遗留构造把槽位置 `None` 之后，调用方仍拿不到稳定码。

本文件不改写冻结合同正文、hash 或 schema，也不改写 `m3-project-role-session-authority-addendum-v1.md` 的既有表。下列 hash 仍是只读核验输入：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

本纠正不声称 M3 已解阻，不创建活动 RoleSession，不授权 M5 / M6 / stage-15 / F0，不改 M5R07 current。

## 1. 槽位边界

普通产品 `AppState` 继续安装服务器-only 权威。验收 / 遗留组合继续保持未安装。

未安装判定必须经过 `AppState` 的服务器-only accessor / 槽位边界。缺失句柄映射为 `m3_project_role_session_authority_unavailable`。仅在测试里构造该错误值不算覆盖本条。

该 accessor 不进入 renderer，不登记 Tauri command，不把原始 repository 交给调用方。

## 2. 已安装端口

已安装端口仍不是普通签发源。path / index locator / scratch / M5 helper / M1 读端口都不得提升为 canonical `ProjectId`。在合法权威源出现之前，每一个 claim 继续 fail closed。

## 3. 非目标

- 不新增 Tauri command / renderer 接线 / 原始 repository 外露
- 不改 M5 / M6 源、lifecycle、authorization、壳文档
- 不伪造 `ProjectId`
- 不把 M3 标成已解阻
