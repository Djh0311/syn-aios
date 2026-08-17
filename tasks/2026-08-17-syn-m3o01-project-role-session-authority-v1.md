# SYN-M3O01 服务器持有的项目角色会话权威

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 用户明确授权的独立 M3-owner 窄纠正包

独立 M5R07 验收判定 M5 越权持有 ProjectSupervisor / Worker / IndependentReviewer RoleSession 的 provision/load/restore。本包由 M3 owner 收回该权威，不改 M5R07 current，不改 stage-14，不改 authorization，不进入 M6 / stage-15 / F0。

## 授权边界

- 主工作树有用户 WIP：不 reset / stash / clean / `git add -A`。
- 不动六份已跟踪壳文档、未跟踪壳文档、`linux-schema.json`、任何 `m6_*.rs`。
- 不改 M5 文件。
- 不改冻结 M1–M3 正文 / hash，不改 schema 语义。
- 只新增独立任务、增补合同、M3 服务器权威端口、普通 `AppState` 最小安装、定向离线测试与本包证据。
- 不新增 renderer / Tauri command / 原始 repository 外露 / M5 自造身份。

## 产品结果

普通产品 `AppState` 安装一个仅服务器可见的 M3 项目角色会话权威端口。该端口是 ProjectSupervisor、Worker、IndependentReviewer 三个精确活动 RoleSession 的唯一合法 provision / load / restore owner，成功时只返回不可变的服务器精确 project / role / actor / session / binding / revision 信息。

不可用、错配、重复、非活动、binding 漂移或 permission 漂移一律 fail closed。

## 硬门：ProjectId

只消费已存在的权威 canonical `ProjectId` 源。路径、index locator、scratch、M5 helper 都不是权威。若仓内不存在该源，停止实现任何猜测式签发，不把上述输入提升为 `ProjectId`。

## 验证

- `cargo check --lib --offline`
- 定向 `cargo test --lib --offline -- m3_project_role_session_authority`
- `git diff --check`

不 push / merge / rebase / deploy / release。
