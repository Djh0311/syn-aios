# M1I01R01 project_index 基座纠正报告

日期：2026-08-17

任务包：`M1I01R01`

被拒绝 candidate：`88cb02e3426ede7b9500d3b6c6263720877c3c11`

本报告记录独立总线拒绝 `88cb02e` 的原因，以及本纠正包做了什么。它不是独立验收，也不把 M3O01 标成已解阻。

## 1. 拒绝原因

P0：冻结合同所有权。`identity-scope-v1` 只把 `ProjectId` 和 `ProjectRootRef` 交给 `project_index`。`88cb02e` 创建、持久化并返回 ActorId、RoleRef、ScopeRef、CurrentObjectRef、ExecutionChannel、PermissionProfile、PermissionSnapshotRef 和 IdentitySnapshot。这些对象仍归其具名冻结 owner。把它们改名为“项目角色身份权威”不能变成合法增补。

P1 / P2：登记没有跨写者串行化，同一别名可能双成功、不同别名可能丢更新；`registry_revision` 使用 saturating 加法；已建立 registry 缺失或损坏时会写成空白新文件；rename 后的目录 sync / open 错误被吞掉。

## 2. 纠正

- 删除 `m1_project_role_identity.rs`，registry 不再保存角色 / actor / scope / permission / identity 字段。
- 消费者只看见 `M1ProjectIndexReadPort`。普通 `AppState` 只安装读端口。
- 登记 / mint 留在测试用服务器写面，不进入 AppState、renderer、Tauri command 或 M5。
- 读打开缺失且从未建立的 registry 时返回未安装，不写空白文件；已建立后缺失 / 损坏 / 不受支持 / 非普通文件 fail closed。
- 含 Actor / Role / Scope / Identity 等越权字段的 registry 因 `deny_unknown_fields` fail closed，不得导入。
- 登记临界区用排他锁覆盖 load / validate / duplicate-check / mint / persist。
- 修订使用 checked 加法；每一个仍保留字段都校验；rename 后目录 open / sync 失败传播。

## 3. 证据范围

只证明离线 `cargo check --lib --offline` 与定向 `m1_project_index` 单测。不证明真实 App、provider、网络、发布或独立验收。
