# M4R06 五类旧读面的实际 shadow/parity/fallback

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：为五类旧读面接入实际 server-owned reader，逐条形成 exact tuple、canonical reread、parity/quarantine 和受守卫 fallback。
干完的标准：每类均有匹配、空态、无法连接和拒绝反例；至少一类生产 adapter 产生真实 PARITY 并可见 fallback；其余如实记录 EMPTY/UNJOINABLE/QUARANTINED；不产生写入/effect replay，不提前退役旧面。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_legacy_readers.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4r06_ordinary_legacy_read_driver.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs
- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/src/main.tsx
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- docs/harness/

## 步骤

1. 复跑 R01 legacy reader 红灯探针。
2. 逐类定位 server-owned reader 与普通产品来源入口，禁止 renderer 临时态升格。
3. 生成 exact tuple、canonical reread 和 PARITY/EMPTY/UNJOINABLE/QUARANTINED。
4. 接受守卫 read-only fallback 与精确回源，验证零写入和零 effect replay。
5. 跑聚焦回归与非测试构建，独立审查后精确提交并归档。
