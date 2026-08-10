# M4R02 普通产品来源与个人对象组合

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：从已点名内部 source owner 的普通产品入口接入 M4，并给 PersonalAction、Reminder、Notification 与 typed Decision 建立正常产品组合入口。
干完的标准：fixture 只调用 source owner 产品入口；普通 constructor/source dispatcher 非测试调用链进入 M4；个人对象生命周期、重启、幂等、owner 隔离与 quarantine 通过，协调动作不反写 owner。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- docs/harness/

## 步骤

1. 复跑 R01 source-ingress/personal-object 红灯探针。
2. 以普通 source owner command/event 和 production dispatcher 接 M4，不直调 repository。
3. 接显式 PersonalAction/Reminder、事件型 Notification 和 typed Decision projection。
4. 验证幂等、revision/watermark、owner 隔离、敏感 quarantine、重启和非反写。
5. 跑聚焦回归与非测试构建，独立审查后精确提交并归档。
