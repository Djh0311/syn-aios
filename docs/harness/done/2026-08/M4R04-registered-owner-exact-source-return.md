# M4R04 注册 owner 的精确回源

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：让 server-minted route 经注册 source-owner resolver 解析成有限 typed 导航，并由目标页面实际消费 focus。
干完的标准：不同 owner/同 object id 不串；unknown owner、stale ref、revision mismatch 和 missing target 明确失败；不使用 raw path/URL/callback/renderer 猜路由或通用 Projects 冒充精确回源。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_source_owner_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_source_route_resolver.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5f1.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4r04_ordinary_route_driver.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/main.tsx
- prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/src/views/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- docs/harness/

## 步骤

1. 复跑 R01 route 红灯探针。
2. 建立 closed owner registry、server resolver 和 typed navigation target。
3. 让目标页面消费 focus，并把失败态反馈给 Secretary UI。
4. 覆盖 owner/object collision、unknown/stale/revision mismatch/missing 反例。
5. 跑聚焦回归与非测试构建，独立审查后精确提交并归档。
