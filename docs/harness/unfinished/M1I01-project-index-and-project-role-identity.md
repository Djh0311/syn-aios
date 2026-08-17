# M1I01 服务器持有的 project_index 与项目角色身份权威

阶段：stage-14 仍开。本文件只是独立 M1-owner 任务投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。不建立 stage-15，不启动 F0，不 closeout M5R07，不改 M3O01 未跟踪文档。

目标：在不改冻结 M1–M3 正文 / hash / schema 语义、不改 M3 / M5 / M6 文件、不碰壳文档 / linux-schema / Harness stage/leaf/auth 的前提下，补齐服务器-only `project_index` 所有权，以及服务器-only 项目三角色身份权威；只做显式隔离项目登记与精确别名解析，供后续 M3O01 消费权威 canonical `ProjectId`。本包不是 M3 / M5 / M6 施工。

来源：用户明确继续并要求 Grok 实际实现本窄包；M3O01 因仓内没有独立于 path / locator / scratch / M5 helper 的 canonical `ProjectId` 源而 fail closed。

产品：`docs/contracts/m1-project-index-and-role-identity-addendum-v1.md`，`m1_project_index.rs`，`m1_project_role_identity.rs`，普通 `AppState` 最小安装。

不许动：

- M5R07 current / stage-14 关闭 / authorization 手填
- M3O01 未跟踪文档与任务原文
- M3 / M5 / M6 源文件
- 冻结 M1–M3 正文与 hash
- 六份已跟踪壳文档、未跟踪壳文档、`linux-schema.json`、`m6_*.rs`
- renderer / Tauri command / 原始 registry 外露
- 用 path / index locator / scratch / caller boolean / M5 helper 派生或签发 `ProjectId`
- 自动导入 legacy index / 做 live 迁移 / 猜测未来项目创建 owner
