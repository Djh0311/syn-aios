# M4C08 旧读面兼容迁移与回切

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：把旧 secretaryReadModel、右栏即时派生、runtime attention、pendingAction 和 memory daily inbox 迁为 source refs/compatibility read-only，保留回切并隔离未知 owner。
干完的标准：shadow/parity 报告覆盖 source/status/priority reason；canonical source+watermark 重读；过期/未知 owner quarantine；旧面不能写新状态；回切不撤销 owner 事实；不物理删除。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx
- prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/tests/ [新增]
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
- docs/harness/

## 步骤

1. 建旧来源 inventory、parity fixtures 和 owner unknown 失败测试。
2. 实现 compatibility adapter、watermark/dedupe candidates 和 quarantine。
3. 切新读面为 primary，旧读面只读 fallback；验证双向不会写回。
4. 跑 parity/回切/前端全量测试，独立审查后精确提交并归档。
