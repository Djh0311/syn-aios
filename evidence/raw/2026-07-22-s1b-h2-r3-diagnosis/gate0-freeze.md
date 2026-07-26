# S1B-H2-R3 Gate 0 脱敏冻结记录

- 取证时间：2026-07-22（+0800）；仅在 App 已关闭、相关进程、holder 与 registry entries 均为空时读取。
- 进程探测：Workbench / Tauri dev / Vite / Codex / MCP 查询均无匹配；workflow-state 与 production SQLite（含 WAL/SHM）无 holder。
- Git：HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 集为空；既有 tracked/untracked 脏项已保留，均不在 R3 冻结的八个源码文件中。
- 现场 JSON：workflow revision `289`；target workflow canonical `recorded/injected/replied = 11/3/3`；proposal/Pending `74/17`；chain `40`；supervisor sessions/audits `25/253`；registry entries `0`。
- 现场 DB：immutable 只读 `integrity_check=ok`；业务投影与 JSON 一致；无新的 `storage_mode_degraded_json_only`。

| 现场对象 | SHA-256 | mtime（epoch） |
| --- | --- | ---: |
| workflow state | `e086668326d4e6b19abe5697109a5b09b861b68dd88b34d75f9264a1ae2ee19c` | `1784666700` |
| supervisor sidecar | `699043d7ed8b06b988d41fe358ca90658710457ea4985ab9c23dd60ea780c1ac` | `1784430553` |
| proposal sidecar | `3d7d965e02fb12761d5f7e9d85218fd154050131edf77e92951f90540238f631` | `1784378355` |
| process registry | `fff2b73613e14dc67f6b34cf68a1d3dc1b93a6e5851e446032c00588d947b5c0` | `1784666598` |
| production SQLite | `347e05db558ea9c1c76c6c4395167c55c1a86ec7e3473addc9e17fa1b105053d` | `1784666700` |

| R3 冻结源码 | SHA-256 |
| --- | --- |
| `supervisor_resident_oneshot_session.rs` | `86bae55ccc9cd9e1499eae9396b987ea9ef18a31c43f872ad97c0e5e79db2da3` |
| `supervisor_resident_oneshot_tests.rs` | `82b15432fa35e47b4b6bcc26cab1a20906f8f307b491b8d326602b1bb7ea9c58` |
| `mcp/supervisor_orchestrator_resident_session.rs` | `d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845` |
| `workflow_read_model_entrypoints.rs` | `7f382cadf799f9dc6e4a34e86b22aca666d9bb8983dee717c235d85c2e03252e` |
| `exec_process_registry.rs` | `4057ea384e46c39d2c8f101213e48cf8f4e76fc5ce68522a4a6e1ba13c9ee848` |
| `useJiaobanConversationState.ts` | `47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2` |
| `jiaoban-conversation-center.test.tsx` | `1fa3f464ecc827fda5ed7e6c7c9d99060a4034efbd50f8c357864993d2144c6d` |
| `mcp/supervisor_orchestrator_submit_proposal.rs` | `6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b` |

未保存用户正文、私有 runner 正文、认证资料或 `CODEX_HOME` 内容。
