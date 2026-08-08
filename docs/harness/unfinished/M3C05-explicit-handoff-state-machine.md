# M3C05 显式 Handoff 状态机与结果回源

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：实现 Handoff aggregate 的 create / accept / reject / cancel / expire / return pending / returned / return failed / retry / source cancel 状态机、单写 repository 和幂等 receipt；请求不生成授权。
干完的标准：错误 recipient、过期、stale revision、重复接单、分歧结果、原对象不存在与回源失败都 fail closed；accepted 后不静默过期；结果只能由 source owner 新命令应用。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m3_handoff.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m3_role_session_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs

## 步骤

1. 先写完整合法/非法状态表与幂等失败测试。
2. 实现 aggregate、command、receipt 与 repository 原子写。
3. 证明 permission request 仅为请求，结果回源不越过 source owner。
4. 覆盖 crash/replay/receipt lost/return retry/original-object-missing。
5. 跑聚焦测试、临时库恢复、非测试构建与回归，独立提交。
