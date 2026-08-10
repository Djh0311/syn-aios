# M4C09 隔离产品应用分层验收

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：用隔离 profile、合成结构化事件、两个 source owner 和 fake model 验收普通产品 M4 链的启动、强退、重启、去重、生命周期、日报、对话和回源。
干完的标准：预检 fail closed；debug App 首启/强退/重启通过；关注和会话恢复；日报重跑幂等；deep link 可见可点；模型故障有 deterministic 结果；空事件零调用；保存脱敏 receipt 与可见窗口证据。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m4_acceptance.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/tests/ [新增]
- prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs
- prototypes/productized-desktop-shell/scripts/run-m4-isolated-app-acceptance.mjs [新增]
- docs/harness/reports/ [新增]
- docs/harness/
- /private/tmp/syn-m4-acceptance- [新增]

## 步骤

1. 建隔离 launcher、profile、synthetic source/fake model 与 receipt schema。
2. 跑静态、单元、临时集成和 non-test build 预检。
3. 启动 debug App，完成可见交互、强退/重启、deep link 和故障场景。
4. 校验脱敏证据/环境不变，独立审查后精确提交并归档。
