# M3O03 服务器持有的 ProjectRoleIdentitySource

阶段：stage-14 仍开。本文件只是独立 M3-owner 纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。不建立 stage-15，不启动 F0，不 closeout M5R07，不改 M3O02 已接受的 fail-closed 地位。

目标：在普通产品 `AppState` 里安装服务器-only、版本化的 M3-owned `ProjectRoleIdentitySource`。它只接受经同一 app-data 根复核的类型化 `M1ProjectId` 与 `M3ProjectRole`（`ProjectSupervisor` / `Worker` / `IndependentReviewer`），为每个精确 project/role 持久化唯一的服务器解析 actor / role / scope / object / channel / permission snapshot，并经既有 M3 repository 驱动 RoleSession provision / load / restore。

来源：用户明确授权本窄前置纠正。`M3O02` `d26856f` / `461c944` 只独立接受为 fail-closed，不解阻 M3/M5，不激活 M6 / stage-15。

产品：`docs/contracts/m3-project-role-identity-source-v1.md`，`m3_project_role_identity_source.rs`，普通 `AppState` 里对 M3 权威的最小接线。

不许动：

- M5R07 current / stage-14 关闭 / authorization 手填
- 冻结 M1 project index 与 M3 session schema
- `m5_*.rs`、M6、commands、renderer、stage lifecycle、产品计划
- 既有跟踪 / 未跟踪 WIP；不 reset / stash / clean / overwrite / `git add -A`
- 用 path / root / alias / locator / cwd / M5 材料、通用 resolver、M4/M5 helper、固定 local actor 或 legacy import 伪造身份源
