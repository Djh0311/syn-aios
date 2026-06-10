# Handoff：Memory Layer M2 Candidate To Formal Adoption v1

日期：2026-06-03  
任务包：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`  
结论：已完成。

## 本轮做了什么

- 打通 `MemoryCandidate -> FormalMemoryStore` 的受控采纳链路。
- 新增后端采纳命令 `adopt_memory_candidate_to_formal_memory`。
- 新增候选 adoption 回链，候选 sidecar 保留历史并能反查正式记忆 ID / version / audit。
- 正式记忆采纳事件写入 `memory_candidate_adopted_to_formal_memory`，同步产生 record / version / audit。
- 前端项目候选治理卡新增采纳动作和 adoption ID 展示。
- 离线测试补充“确认候选不等于正式记忆”和“已采纳候选显示正式 ID”断言。

## 改动文件

主要 M2 文件：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

构建兼容修复（已被后续撤回 superseded，不属于当前有效代码）：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`

说明：

- 这部分曾用于处理 conversation recovery 类型和 props 漂移，但它属于错误方向的工作台侧 recovery 残留，不属于 M2 记忆采纳能力。
- 后续已撤回该 recovery 实现；当前有效状态以 `evidence/2026-06-03-recovery-withdrawal-and-m2-review-v1.md` 和 `handoffs/2026-06-03-recovery-withdrawal-and-m2-review-v1-result.md` 为准。
- 当前产品代码中 recovery Rust 模块、Tauri command、前端 recovery 类型 / props / 面板已扫描无残留。

## 验证结果

通过：

- `cargo test --lib memory_candidate_adoption -- --nocapture`
- `cargo test --lib memory_candidate`
- `cargo test --lib formal_memory`
- `cargo test --lib`
- `rustfmt --check src/memory_candidate_store.rs src/formal_memory_store.rs src/control_core.rs src/lib.rs src/commands.rs`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

保留 warning：

- Rust 既有 `JsonRpcError::invalid_params` unused warning。
- Vite 既有 chunk > 500 kB warning。

## 不能宣称

- 不能说自动采纳完成。
- 不能说任务包召回完成。
- 不能说任务包注入完成。
- 不能说正式记忆生命周期完成。
- 不能说 Obsidian / 知识库 / 向量库 / 图数据库完成。
- 不能说中间版本记忆层完成。

## 下一步

建议进入 M3：ObservationStore 和工作流观察入口。

M3 仍必须保持边界：

- observation 不是正式记忆。
- observation 不能直接注入任务包。
- 不能绕过 M1.1 / M2 的上下文、来源、权限、风险和审计规则。
