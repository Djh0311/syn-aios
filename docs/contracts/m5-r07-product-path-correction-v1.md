# M5R07 产品路径修正补充合同 v1

- 版本：v1（2026-08-17）
- 状态：**FROZEN（M5R07 返修补充）**
- 关系：补充 M5R04 / M5R06 / M5R07 冻结合同；**不改 M1–M4 正文与 hash**。

## 规则

- 普通产品路径身份真源是 M3 `RoleSession` 不可变 view，只经 AppState 已安装的 `ProjectRoleSessionAuthority` provision/load；M5 不得创建/恢复 RoleSession，不得直开 M3 repository。
- 项目身份只经已安装的 M1 authority/read port 解析已显式登记的 canonical `M1ProjectId`；调用方 alias/canonical text 只是 resolver 输入。禁止 path hash、index locator、scratch claim、自动登记或 M5 helper fallback。
- 调用方不得提交 `role_session_id`；后续只消费服务器首次响应的 canonical project/binding。
- 共享 isolated product profile 保持 M1/M3 未安装，M5 open fail-closed；不得把旧 isolated Tauri full-loop receipt 写成 authority PASS。
- 渲染器不得选择或扩大 Grant `allowed_commands` / scope / policy。批准只绑定已存储提案上的 `authorized_action` 与服务器 policy。
- 普通项目 UI 必须用正式 command 逐步驱动：runtime receipt → worker report → independent review → result decision → summary。
- 隔离 helper 仅在 `SYN_M5R07_ISOLATED_ACCEPTANCE=1` 下可用。
- Summary consumer、source ref、deep-link 由服务器从 RoleSession 与已持久 source 派生；deep-link 必须能回源解析。
- 隔离 UI receipt 由后端 store 状态派生，不得回写前端自报 grant/spawn。
