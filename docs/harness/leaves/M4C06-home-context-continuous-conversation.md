# M4C06 首页情境与持续 Secretary 对话

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：让首页消费后端 typed read model，展示可回源情境、持续 Secretary 对话和完整协调动作，不在 React 本地再造事实。
干完的标准：每项显示来源、owner、出现原因、最后变化、状态和 deep link；外部承诺/时间敏感优先；对话恢复稳定；专业入口保留；加载/空态/失败/键盘/窄屏均可用。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs
- prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/roleSessionReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/src/views/HomeView.tsx
- prototypes/productized-desktop-shell/src/styles.css
- prototypes/productized-desktop-shell/src/styles/
- prototypes/productized-desktop-shell/tests/ [新增]
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
- docs/harness/

## 步骤

1. 审计现有首页/秘书面，冻结 DTO、产品命令桥和交互状态；只把 C04 已有协调方法与 C05 typed brief 接到普通产品，不新增业务状态机。
2. 写来源、生命周期、deep link、对话恢复和错误态失败测试。
3. 接 typed read model 与 commands，完成响应式、键盘和无障碍状态。
4. 跑 typecheck/offline/build 和视觉审查，独立复核后精确提交并归档。
