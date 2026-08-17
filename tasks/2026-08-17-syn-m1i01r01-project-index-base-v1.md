# SYN-M1I01R01 project_index 基座纠正

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 独立总线拒绝 `88cb02e` 后的窄纠正包

`88cb02e` 的 P0 是冻结合同所有权：`project_index` 只拥有 `ProjectId` 和 `ProjectRootRef`。该 candidate 却创建、持久化并对外主张 ActorId / RoleRef / ScopeRef / CurrentObjectRef / ExecutionChannel / PermissionProfile / PermissionSnapshotRef / IdentitySnapshot。这些对象仍归其具名冻结 owner。本包删除该越权实现，不改名续持。

## 授权边界

- 不 reset / stash / clean / `git add -A` / push / merge / rebase / deploy / release。
- 不启动产品 App，不接 provider / 网络。
- 不动 stage-12、M3 / M5 / M6 源、冻结 M1–M3 正文 / hash / schema、Harness stage / leaf / authorization、壳 WIP、`linux-schema.json`、m6 WIP、M3O01 草稿。
- 不声称 M3O01 已解阻，不创建 M3 RoleSession。
- 角色身份留给后续独立、与冻结 owner 对齐的包。

## 产品结果

服务器-only `project_index` 基座：

1. 不透明随机 `project:<uuid>` 签发；
2. 显式精确别名登记 / 解析；
3. 原子本地 registry；
4. fail-closed 访问；
5. 向消费者暴露 `M1ProjectIndexReadPort`；
6. 登记 / 签发不得被 M5、renderer 或 Tauri command 调用；
7. 普通 `AppState` 只安装读端口；丢失的 registry 不得被静默初始化成空白 registry；
8. 未登记 / legacy 输入保持不可用。

同时纠正：跨进程串行化 load / validate / duplicate-check / mint / persist；禁止 saturating 修订运算；校验每一个仍保留的持久字段；传播 rename 后的目录 sync / open 错误。

## 验证

- 触及的 Rust 上 `cargo fmt`
- `cargo check --lib --offline`
- 定向 `cargo test --lib --offline -- m1_project_index`
- `git diff --check`

不 push / merge / rebase / deploy / release。不宣称独立验收。
