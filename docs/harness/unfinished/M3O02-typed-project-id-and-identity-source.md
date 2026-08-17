# M3O02 类型化 M1ProjectId 与身份源 fail-closed

阶段：stage-14 仍开。本文件只是独立 M3-owner 纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。不建立 stage-15，不启动 F0，不 closeout M5R07。

目标：让普通产品 M3 provision / load / restore 只消费类型化 `M1ProjectId`；经 M1 受限 verifier 按同一 app-data 根复核后，因合法身份源尚不存在而 fail closed。不创建活动项目三角色 RoleSession。

来源：用户明确授权本窄纠正包。`M1I01R03R01` `ca413a9` 不解阻 M3/M5。

产品：`docs/contracts/m3-typed-project-id-identity-source-v1.md`，M1 受限 typed-id verifier，M3 请求 API 改为 `M1ProjectId`，普通 `AppState` 接线。

不许动：

- M5R07 current / stage-14 关闭 / authorization 手填
- `m5_*.rs`、renderer、Tauri command、M3 repository/schema、M6、壳文档
- 冻结 M1–M4 正文与 hash
- 用 M5 / path / locator / scratch / 通用 identity resolver / M4 Secretary / 固定 local actor 伪造身份源
