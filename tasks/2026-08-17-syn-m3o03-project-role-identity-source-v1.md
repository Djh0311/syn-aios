# SYN-M3O03 服务器持有的 ProjectRoleIdentitySource

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 用户明确授权的独立 M3-owner 窄前置纠正

基线：`d26856fdda7eed84df685aa2d1ac37950355bd4c`。`M3O02` 只独立接受为 fail-closed。本包不是 M5R07 施工，不激活 M6 / stage-15。

## 授权边界

- 保全全部已跟踪 / 未跟踪 WIP；不 reset / stash / clean / rebase / merge / push / deploy / release / `git add -A` / `git add .` / `commit -a`。
- 不接真实 provider、账号、凭据、用户数据、消息、connector、远端服务或产品 App 窗口。
- 不改 stage / current leaf / authorization / M5 报告 / 冻结 M1–M4 正文。
- 允许实现面：新合同 `docs/contracts/m3-project-role-identity-source-v1.md`；新 `src-tauri/src/m3_project_role_identity_source.rs`；只窄改 `m3_project_role_session_authority.rs` 与 `lib.rs`；仅当不可避免时才给 `m3_role_session_repository.rs` 只读窄开口；只新增 M3O03 任务 / 报告 / unfinished。
- 不改 `m5_*.rs`、M6、commands、renderer、冻结 M1 project index、M3 session schema、壳文档、产品计划。

## 产品结果

1. 普通产品 `AppState` 安装服务器-only、版本化的 M3-owned `ProjectRoleIdentitySource`。隔离验收与遗留保持未安装。
2. 源只接受经同一根复核的类型化 `M1ProjectId` 与三角色；为每个精确 project/role 持久化唯一服务器解析 actor / role / scope / object / channel / permission snapshot。
3. scope / object 精确绑定 canonical project；不接受也不持久化 raw path / root / alias / locator / cwd / M5 材料。
4. permission snapshot 默认拒绝，零 execution / provider / runner / grant 权威；永不成为 M5 ExecutionGrant。
5. provision / load / restore 每次都先复核同根类型化 M1 ID，并在任何写入前传播稳定 M1 故障码。
6. 源缺失 / 损坏 / 重复 / 篡改 / 版本不匹配 / role-project 不匹配 / binding 或 fingerprint 漂移 / permission 漂移 / 会话不活动或缺失：一律 fail closed。
7. load / restore 不得创建、修复、resume，也不得接受调用方自选 `role_session_id`。
8. 首次 provision 幂等：先写 PREPARED 身份包，仅当 source / binding / session 精确匹配后才可读；中断状态只能由同一 provision 输入完成。
9. 经既有 M3 repository 驱动 RoleSession；优先独立源存储，不改现有 M3 schema。

## 验证

只在 disposable checkout 取证：`git diff --check`、定向 M1+M3 离线测试、`cargo check --lib --offline`。不声称 M3/M5 完成，不关闭任何任务。
