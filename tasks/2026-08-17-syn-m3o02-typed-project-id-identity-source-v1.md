# SYN-M3O02 类型化 M1ProjectId 与身份源 fail-closed

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 用户明确授权的独立 M3-owner 窄纠正包

基线：`9bec690a4e8a3756d60195a875524390ff2d77e6`。`M1I01R03R01` `ca413a9` 只通过 AppState 组合：普通 Tauri 安装 M1/M3，隔离验收与遗留保持未安装。这不解阻 M3/M5。

## 授权边界

- 保全全部已跟踪 / 未跟踪 WIP；不 reset / stash / clean / rebase / merge / push / deploy / release / `git add -A` / `git add .` / `commit -a`。
- 不接真实 provider、账号、凭据、用户数据、消息、connector、远端服务或产品 App 窗口。
- 不改 stage / current leaf / authorization / M5 报告 / 冻结 M1–M4 正文。
- 代码只改 `m1_project_index.rs`、`m3_project_role_session_authority.rs`、`lib.rs`。
- 其他路径只新增 M3O02 合同 / 任务 / 报告 / unfinished。
- 不改 `m5_*.rs`、renderer、Tauri command、M3 repository/schema、M6、壳文档。

## 产品结果

1. M3 provision / load / restore 只消费类型化 `M1ProjectId`，不再收 raw `project_id_claim`。
2. M1 增加受限 verifier：对已类型化 `M1ProjectId` 按同一普通 app-data 根复核。不登记别名，不外露 M1 storage。
3. verifier 只接入普通 `AppState` 的 M3 组合；隔离 / 遗留保持未安装。
4. 无 verifier、M1 registry 缺席 / 缺失 / 损坏、未知 ID、跨根类型化 ID：在任何 M3 repository create/load/restore 之前 fail closed。
5. 同根合法类型化 `M1ProjectId` 仍返回稳定新码 `m3_identity_source_unavailable`：合法 ActorId、RoleRef、Scope / CurrentObject / ExecutionChannel、ServerResolvedBinding、PermissionSnapshot 源尚不存在。零 M3 会话写入，无成功 view。
6. 不从 M5、path/locator/scratch、通用 identity resolver、M4 Secretary 或固定 local actor 伪造这些源。

## 验证

只在 disposable checkout 取证：`git diff --check`、定向 M1+M3 离线测试、`cargo check --lib --offline`。不声称 M3/M5 完成。
