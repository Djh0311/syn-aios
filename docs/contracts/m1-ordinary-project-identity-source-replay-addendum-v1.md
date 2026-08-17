# M1 普通项目身份显式来源与启动重放增补合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-18

适用 leaf：`M5R00`

## 0. 合同定位

本文件只补充普通 Tauri 产品启动时，老项目取得 M1 canonical `ProjectId` 的显式创建 / 迁移入口。它不改写任何冻结合同、既有增补合同正文或旧 hash。

下列冻结合同 hash 继续作为只读边界：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |

`docs/contracts/m1-project-index-ordinary-authority-v1.md` 的 owner、随机 canonical id、精确别名、持久化和 fail-closed 边界继续有效；只有“普通启动绝不登记”的旧限制由本增补收窄为：普通 Tauri 启动必须先消费本文规定的服务器侧显式来源并重放，除此之外仍不得登记。

本文件不授权 renderer / UI 创建来源，不把 legacy index、path、cwd 或 M5 helper 升格为权威，不创建 M3 RoleSession，不改变 M5R07 的既有 scoped PASS，也不激活 M6 或壳采纳。

## 1. 显式来源

普通 Tauri 启动只消费 ordinary app-data 根下固定的普通文件：

`m1-ordinary-project-identity-source-v1.json`

该文件是服务器侧、预置的 M1 创建 / 迁移输入，不是 authorization 或 receipt。实现不得声称它证明用户授权。它必须是 canonical ordinary app-data 根的直接子文件、非 symlink、非目录，且严格解析以下字段；未知字段拒绝：

- `schema_version`：固定 `m1.ordinary-project-identity-source.v1`
- `source_id`：非空、无控制字符的稳定来源标识
- `source_revision`：大于零
- `projects`：非空项目数组
- 每项：
  - `entry_id`：来源内唯一、非空、无控制字符
  - `mode`：只能是 `create` 或 `migrate_legacy_project`
  - `source_ref`：非空、无控制字符的来源引用；只作来源说明，不作为 `ProjectId`
  - `exact_alias`：来源内唯一的精确 resolver 输入

来源文件不得由启动代码从 `codex-index.json`、workflow-state、项目 path/cwd、locator/slug、scratch、UI 输入或 M5 helper 自动生成，也不得在来源缺失时由实现补出默认文件。

## 2. 创建、迁移与幂等重放

普通 Tauri constructor 必须在普通产品其他组合继续装配前，调用 M1 owner 的显式来源重放入口：

1. 严格读取并验证来源；
2. 在 `project_index` 的同一跨进程临界区内加载、验证 registry；
3. 对来源中的每个 `exact_alias`：
   - registry 未登记时，由 `project_index` 随机签发新的 `project:<uuid-v4>` 并登记；ID 不从 alias、path、source ref 或 entry id 派生；
   - registry 已精确登记一次时，复用原 canonical id，视为幂等重放；
   - 重复、冲突或 registry 不可信时整体拒绝，不再继续启动；
4. 只有确有新增时才原子持久化一次 registry；相同来源重复启动不得新增项目、改变既有 id、增加 registry revision 或重写 registry 字节；
5. AppState 重建后，同一 alias 必须仍解析到同一类型化 `M1ProjectId`。

普通启动不得读取 legacy index 的项目列表后逐项登记；本文的 `migrate_legacy_project` 只说明这条显式来源记录的用途，不授予自动导入或来源猜测。

内部 isolated / acceptance / legacy profile 以及现有 M5 fixture 不因本增补获得普通来源重放权。只有真实普通 Tauri constructor 是本包新增的 non-test caller。

## 3. Fail closed

来源重放发生在普通 Tauri AppState 可用之前。以下任一条件必须返回稳定错误并拒绝普通启动；不得退回 `lib.rs::project_id()`、不得按 path 派生、不得自动导入 legacy index、不得静默创建空 registry：

| 条件 | 稳定码 |
|---|---|
| 来源文件缺失 | `m1_ordinary_project_identity_source_missing` |
| 来源文件不可读或不是 direct regular file | `m1_ordinary_project_identity_source_unreadable` / `m1_ordinary_project_identity_source_malformed` |
| 来源 schema 不支持 | `m1_ordinary_project_identity_source_unsupported` |
| 来源字段、mode、重复 entry / alias 不合法 | `m1_ordinary_project_identity_source_malformed` |
| 已建立 registry 缺失 | `m1_project_index_registry_missing` |
| registry 损坏、不受支持或重复 | 既有 M1 registry 稳定码 |

失败前若 registry 无需新增，不写 registry；来源验证失败时零 registry 写入。

## 4. 定向证据

在 synthetic app-data 根上至少证明：

1. 首次普通 Tauri constructor 读取一条显式来源并登记 opaque canonical id；
2. 相同来源重复调用完全幂等，project id、registry revision 与 registry 字节不变；
3. 重建 AppState 后同一 alias 解析为同一 id；
4. 来源缺失、来源损坏、registry 缺失与 registry 损坏均拒绝，且不走 path / legacy fallback；
5. 静态断言普通启动 caller 存在，且实现不解析 legacy index 来登记。

这些只是在 disposable checkout 上绑定候选 SHA 的合成 / 离线证据，不等于真实用户资料迁移、真实日常运行、发布或部署。

## 5. 非目标

- 不提供 UI、renderer command 或 Tauri command 来编辑来源
- 不读取、改写或迁移真实个人项目资料
- 不修改 M3 identity source，除非 M1 类型变化导致其现有编译 / fail-closed 合同必须同步
- 不修改 M5 execution kernel、M5R07、M6、页面布局或旧 `project_id()` 的其他历史调用者
- 不 closeout stage-14，不激活 M6，不 push、merge、rebase、部署或发布
