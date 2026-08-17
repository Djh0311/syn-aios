# M1/M3 共享 AppState 验收 profile 隔离纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M1I01R03R01`

被拒绝 candidate：`061eefee9291dbeddf792af6d78dc48bb5b0f8e5`

## 0. 合同定位

本文件只补充共享 `AppState` 组合边界上的验收 profile 隔离。它不改写冻结合同正文、hash 或 schema，也不改写下列既有纠正合同：

- `docs/contracts/m1-project-index-ordinary-authority-v1.md`
- `docs/contracts/m3-project-role-session-authority-slot-boundary-v1.md`
- `docs/contracts/m3-project-role-session-authority-addendum-v1.md`

下列 hash 仍是只读核验输入：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |

本纠正不声称 M1 / M3 已解阻，不改变登记语义，不创建 RoleSession，不授权 M5 / M6 / stage-15 / F0，不改 M5R07 current。

## 1. 拒绝原因

`061eefe` 让普通产品 `AppState` 安装 M1 / M3 权威。真实隔离验收路由 `AppState::try_new_with_isolated_product_profile` 当时直接调用 `try_new_with_ordinary_product_ports`，因此也安装了这两个句柄。这与 M1R03 / M3O01R01 合同冲突：验收 / 遗留组合必须保持未安装。

## 2. 组合边界

M4 基础设施端口可以共享。M1 / M3 权威安装必须是显式 product-profile 选择，不得因共享构造而继承。

| 组合 | M1 权威 | M3 权威 | accessor |
|---|---|---|---|
| 普通 Tauri 产品 | 安装 | 安装 | 已安装端口 |
| 隔离验收 `try_new_with_isolated_product_profile` | 未安装 | 未安装 | `m1_project_index_unavailable` / `m3_project_role_session_authority_unavailable` |
| 遗留 `try_new()` | 未安装 | 未安装 | 同上 |

未安装判定必须经过真实隔离验收构造函数，或与该构造函数同一显式 profile 分支的直接组合。仅手写 `AppState` 字面量不算覆盖本条。

## 3. 非目标

- 不改 M1 登记语义、M3 request / API 所有权、角色会话动作
- 不改 M5 源、renderer、Tauri command、stage-14 / M5R07、authorization、M6、壳文档
- 不把 M1 / M3 标成已解阻
