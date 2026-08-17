# M1I01R01 project_index 基座纠正

阶段：stage-14 仍开。本文件只是独立 M1-owner 纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。不 closeout M5R07，不改 M3O01 未跟踪文档。

目标：纠正被独立总线拒绝的 `88cb02e`。`project_index` 只保留 `ProjectId` 与 `ProjectRootRef`。删除越权的角色身份实现，不把那些对象改名后继续持有。本包不声称 M3O01 已解阻，不创建 M3 RoleSession。

来源：独立总线拒绝 `88cb02e3426ede7b9500d3b6c6263720877c3c11`；用户要求在当前工作树做窄纠正。

产品：`docs/contracts/m1-project-index-base-correction-v1.md`，纠正后的 `m1_project_index.rs`，`M1ProjectIndexReadPort`，普通 `AppState` 只安装读端口。

不许动：

- M5R07 current / stage-14 / authorization
- M3 / M5 / M6 源文件、壳 WIP、`linux-schema.json`、M3O01 草稿
- 冻结 M1–M3 正文 / hash / schema
- renderer / Tauri command / 原始 registry 外露
- 用 path / locator / scratch / caller boolean / M5 helper 派生 `ProjectId`
