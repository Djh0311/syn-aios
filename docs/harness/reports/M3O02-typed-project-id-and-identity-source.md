# M3O02 类型化 M1ProjectId 与身份源 fail-closed 报告

日期：2026-08-17

任务包：`M3O02`

基线：`9bec690a4e8a3756d60195a875524390ff2d77e6`

本报告记录本纠正包做了什么。它不是独立验收，也不把 M3 / M5 标成已解阻或已完成。

## 1. 产品

- M3 provision / load / restore 请求字段改为类型化 `M1ProjectId`，raw `project_id_claim` 不再进入该 API。
- M1 增加受限 typed-id verifier：只按同一普通 app-data 根复核已类型化 ID。不登记别名，不外露 registry 存储。
- 普通 `AppState` 把该 verifier 接入 M3 权威；隔离验收与遗留保持未安装。
- 无 verifier、跨根类型化 ID、M1 registry 缺席 / 缺失 / 损坏、未知 ID：在任何 M3 repository create/load/restore 之前 fail closed。
- 同根合法类型化 ID 返回稳定新码 `m3_identity_source_unavailable`。零项目三角色会话写入，无成功 view。
- 不从 M5、path/locator/scratch、通用 identity resolver、M4 Secretary 或固定 local actor 伪造身份源。

## 2. 未声称

- 不声称 M3 / M5 已解阻或已完成。
- 不创建活动 ProjectSupervisor / Worker / IndependentReviewer RoleSession。
- 不改 M5R07 current、stage-14、authorization、M6 / stage-15。
- 不证明真实 App、renderer、Tauri command、provider、网络、发布或独立验收。

## 3. 证据范围

只证明 disposable checkout 上的离线 scoped checks。

| 项 | 值 |
|---|---|
| 实现 candidate | `461c9444661dddc0f9f8ed6ec6a83c9b48e059b3` |
| Disposable checkout | `/tmp/m3o02-disposable-461c944` |
| Disposable HEAD | `461c9444661dddc0f9f8ed6ec6a83c9b48e059b3` |
| current leaf | `M5R07-project-ui-isolated-app-and-stage-candidate` |
| authorization | `{"schemaVersion":1,"authorized":false}` |
| `git diff --check` | clean |
| `cargo test --lib --offline -- m1_project_index m3_project_role_session_authority -- --test-threads=1` | 26 passed；0 failed |
| `cargo check --lib --offline` | exit 0（既有 warning，无本包新增 error） |

## 4. 载体

实现与证据分两次提交。证据 binding commit 的 exact SHA 以本文件入库后的 git 对象为准。
