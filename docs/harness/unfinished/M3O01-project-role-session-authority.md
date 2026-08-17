# M3O01 服务器持有的项目角色会话权威

阶段：stage-14 仍开。本文件只是独立 M3-owner 任务投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。不建立 stage-15，不启动 F0，不 closeout M5R07。

目标：在不改冻结 M1–M3 正文 / hash / schema 语义、不改 M5 文件、不碰壳文档 / linux-schema / m6 的前提下，把 ProjectSupervisor / Worker / IndependentReviewer 的合法 RoleSession provision / load / restore 收回到普通产品 `AppState` 里的服务器-only M3 权威端口。

来源：用户明确授权本窄纠正包；独立 M5R07 验收发现 M3 所有权被 M5 越权。

产品：`docs/contracts/m3-project-role-session-authority-addendum-v1.md`，`m3_project_role_session_authority.rs`，普通 `AppState` 最小安装。端口已安装；因没有可消费的普通权威 canonical `ProjectId` 源，provision / load / restore 仍 fail closed。不声称已解阻。

不许动：

- M5R07 current / stage-14 关闭 / authorization 手填
- M5 源文件
- 冻结 M1–M3 正文与 hash
- 六份已跟踪壳文档、未跟踪壳文档、`linux-schema.json`、`m6_*.rs`
- renderer / Tauri command / 原始 repository 外露 / M5 自造身份
- 用 path / index locator / scratch / M5 helper 冒充 canonical `ProjectId`
