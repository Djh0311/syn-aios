# M3O03 ProjectRoleIdentitySource 报告

日期：2026-08-17

任务包：`M3O03`

基线：`d26856fdda7eed84df685aa2d1ac37950355bd4c`

本报告记录本纠正包做了什么。它不是独立验收，也不把 M3 / M5 标成已解阻或已完成。

## 1. 产品

- 普通产品 `AppState` 安装服务器-only、版本化的 M3-owned `ProjectRoleIdentitySource`。隔离验收与遗留保持未安装。
- 源只接受经同一根复核的类型化 `M1ProjectId` 与 `ProjectSupervisor` / `Worker` / `IndependentReviewer`。
- 每个精确 project/role 持久化唯一服务器解析 actor / role / scope / object / channel / permission snapshot。scope / object 绑定 canonical project。
- permission snapshot 默认拒绝，零 execution / provider / runner / grant 权威，不是 M5 ExecutionGrant。
- 首次 provision 先写 PREPARED，仅在 source / binding / session 精确匹配后可读；同一输入幂等，中断状态只能由同一输入完成。
- load / restore 不创建、不修复、不 resume，也不接受调用方自选 `role_session_id`。
- 源缺失 / 损坏 / 重复 / 篡改 / 版本不匹配 / role-project 不匹配 / binding 或 permission 漂移 / 会话不活动或缺失：fail closed。
- M1 故障码在任何源写入或 M3 repository 动作前传播。未安装源时仍返回 `m3_identity_source_unavailable`。

## 2. 未声称

- 不声称 M3 / M5 已解阻或已完成。
- 不改 M5R07 current、stage-14、authorization、M6 / stage-15。
- 不证明真实 App、renderer、Tauri command、provider、网络、账号、发布或独立验收。
- 不关闭 M3O03 或其他任务。

## 3. 证据范围

只证明 disposable checkout 上的离线 scoped checks。

| 项 | 值 |
|---|---|
| 实现 candidate | `48d8dbcec3165c12173a04ab157867ef2482f411` |
| Disposable checkout | `/tmp/m3o03-disposable-48d8dbc` |
| Disposable HEAD | `48d8dbcec3165c12173a04ab157867ef2482f411` |
| current leaf | `M5R07-project-ui-isolated-app-and-stage-candidate` |
| authorization | `{"schemaVersion":1,"authorized":false}` |
| `git diff --check` | clean |
| `cargo test --lib --offline -- m1_project_index m3_project_role_session_authority m3_project_role_identity_source -- --test-threads=1` | 32 passed；0 failed |
| `cargo check --lib --offline` | exit 0（既有 warning，无本包新增 error） |

## 4. 载体

实现与证据分两次提交。证据 binding commit 的 exact SHA 以本文件入库后的 git 对象为准。
