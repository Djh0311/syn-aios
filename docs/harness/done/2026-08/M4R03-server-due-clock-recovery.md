# M4R03 服务端到期时钟与恢复

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：让普通产品 server scheduler 驱动 snoozed OpenLoop 与 Reminder 到期推进，renderer 不拥有时钟。
干完的标准：到期自动恢复/单次触发；到期前强退、到期后重启、重复/并发 tick 和 CAS 冲突均不漏不重；普通产品 composition 证明真实 scheduler 调用链。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4r03_ordinary_clock_driver.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_scheduler.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src/main.tsx
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- docs/harness/

写域增补说明：R03 的 green receipt 必须由普通产品 composition 直接证明，不得以 repository seed 或 transition 直调冒充。新增的 host driver 与 renderer bridge 只编排现有普通 Tauri command registry、普通 AppState、source dispatcher 和 scheduler；它们不新增产品命令、不替代 handler，也不获得 renderer clock/fire 权限。隔离验证只替换 app-data root，并使用本机 server clock。

## 步骤

1. 复跑 R01 due-clock 红灯探针。
2. 把到期扫描和幂等 transition 接入普通 server scheduler。
3. 覆盖 startup recovery、重复/并发 tick、CAS 冲突和 crash/restart。
4. 验证 renderer 无 clock 真值、空事件零模型和 effect 不重放。
5. 跑聚焦回归与非测试构建，独立审查后精确提交并归档。
