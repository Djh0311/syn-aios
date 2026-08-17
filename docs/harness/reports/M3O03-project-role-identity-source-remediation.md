# M3O03 ProjectRoleIdentitySource 返修报告

日期：2026-08-17

任务包：`M3O03`

被拒绝 candidate：`48d8dbcec3165c12173a04ab157867ef2482f411`

被拒绝证据：`57951234f996725055eebcee9c215f685832efaa`

本报告记录独立验收拒绝后的最窄返修。它不是独立验收，也不是 closeout。

## 1. 产品

- 首次成功持久化 PREPARED / READABLE 时写入 established marker。
- 已建立源 JSON 被删除后，provision / load / restore 稳定返回 `m3_project_role_identity_source_missing`。不把该缺失当首次空 store，不重建 PREPARED，不借仍存在的 SQLite session 恢复成功。
- 从未建立过源时，首次 provision 仍可建立。源文件仍在时，精确同输入的 PREPARED 仍可续跑。
- 接受 source-bound view 前，同一 canonical project 与 role 若有多于一个 ACTIVE 候选，稳定返回 `m3_project_role_session_duplicate`。
- repository 只增加只读 `list_active_sessions_for_project_role`。不改 M3 schema，不用 caller `role_session_id` / path / scratch / resume 绕过。

## 2. 未声称

- 不声称 M3 / M5 已解阻、已完成或已独立验收。
- 不改 M5R07 current、stage-14、authorization、M6 / stage-15。
- 不关闭 M3O03 或其他任务。
- 不证明真实 App、renderer、Tauri command、provider、网络、账号、发布。

## 3. 证据范围

只证明 disposable checkout 上的离线 scoped checks。

| 项 | 值 |
|---|---|
| 实现 candidate | `c477b577c9a2e050eb030a79f161a9d52fd9e28a` |
| Disposable checkout | `/tmp/m3o03-disposable-c477b57` |
| Disposable HEAD | `c477b577c9a2e050eb030a79f161a9d52fd9e28a` |
| current leaf | `M5R07-project-ui-isolated-app-and-stage-candidate` |
| authorization | `{"schemaVersion":1,"authorized":false}` |
| `git diff --check` | clean |
| `cargo test --lib --offline -- m1_project_index m3_project_role_session_authority m3_project_role_identity_source -- --test-threads=1` | 35 passed；0 failed |
| `cargo check --lib --offline` | exit 0（既有 warning，无本包新增 error） |

## 4. 载体

实现与证据分两次提交。证据 binding commit 的 exact SHA 以本文件入库后的 git 对象为准。
