# Root Treatment / R4-A1 Page Read Model Inventory And Query Contract v1

日期：2026-06-11

状态：已完成。本文是 Root Treatment / Stage R 的 R4-A1 任务包，用于在 R3-A13 Level A transaction acceptance / cutover gap matrix 完成后，启动 R4 read model / frontend slimming。R4-A1 只做页面数据需求盘点、按页读模型合同和最小后端/前端类型边界准备；不改视觉风格、不实际重做布局、不实现 MCP 看图工具、不切 SQLite production DB、不解冻 backlog 功能。

规划基线 commit：`6786faf`

## 0. 全局主管理解

已知事实：

- R3-A13 Level A 已完成，implementation commit 为 `d96ed042341fa816e62b149f0ea451516f0e5ad2`，checkpoint hash 修正 commit 为 `6786faf`。
- R3-A13 只接受为 fixture / temp SQLite transaction acceptance 和 cutover gap matrix Level A；R3 Level B 未执行。
- 当前前端仍主要依赖巨型 `WorkbenchSnapshot`，`App.tsx` 把同一 snapshot 切给项目、智能体、运行中、记忆、知识库、设置、Skill、Harness 等页面。
- R4 正式目标是“读模型和前端瘦身”，不是 UI 视觉重做。
- Xuanji / Mobbin / inkwash 等 UI 信息层级参考只能影响后续解冻或 R4 的组件边界预留，不能在 R4-A1 顺手改视觉。

核心判断：

```text
R4-A1 先把“每个页面真正需要哪些字段”写成合同，并实现最小 page read model skeleton / 类型边界；后续 R4-A2+ 再逐页替换巨型 WorkbenchSnapshot 依赖。R4-A1 不追求一次把所有页面改完。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package。

Multi-Agent Policy：

- 主管线负责冻结任务包、验收和入口同步。
- 可保留一条复核线做只读审查，不新增过多线程。
- 开发实现可以由主管线直接完成，避免 R4 任务被拆得过碎。

Level split：

- Level A：页面数据需求矩阵 + page read model contract + 最小 skeleton。默认执行。
- Level B：逐页迁移真实 UI 数据来源。R4-A1 不执行，留给 R4-A2+。

Fallback If Scope Expands：

- 如果实现需要视觉重做、布局重排、真实 Tauri 截图、SQLite production read-cut、真实 Codex、`.codex`、secret / full transcript、provider credential，立即停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- R3-A13 task / evidence / handoff。

建议读取：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts`
- `prototypes/productized-desktop-shell/src/views/HomeView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`

## 3. Page Read Model Inventory

R4-A1 必须冻结以下页面数据需求矩阵：

| Page | User-facing data | Developer/internal data | Must not show as primary |
| --- | --- | --- | --- |
| Home | 主对象入口、运行中摘要、待处理摘要、索引状态 | snapshot source / diagnostics refs | raw sidecar、full audit path、schema dump |
| Projects | 项目列表、项目详情、工作流画布摘要、任务包状态、节点详情摘要 | audit/evidence refs、dispatch/readback diagnostics | raw transcript、完整 task package 文本、内部 schema |
| Agents | 项目选择、会话选择、对话流、输入/执行 readiness | adapter descriptors、operation/provider/session boundary | 控制中心式全量边界面板、未实现执行按钮 |
| Running Workflows | 运行队列、待确认、失败/阻断、readback 状态 | runtime refs、diagnostic refs | raw runtime log、internal ids 默认铺开 |
| Memory | 正式记忆、候选、观察、lint、任务记忆包摘要 | revision、audit refs、sidecar health | candidate/observation 冒充正式记忆 |
| Knowledge | 资料、笔记、引用、关联记忆、候选入口 | index diagnostics、source refs | 知识命中冒充正式记忆 |
| Settings | 普通设置、开发者入口、系统健康 | diagnostics、developer nav、data locations | 把开发/内部入口放在主导航 |
| Skill | 可复用能力、适用场景、可用性、最近使用 | plugin metadata、字段缺口 | 首屏字段/schema 堆叠 |
| Harness | 运行器能力、可运行范围、最近运行、配置状态 | adapter/resource fields | 首屏候选资源/raw config |

## 4. R4-A1 最小目标

允许实现以下最小代码：

- 新增后端 `PageReadModelContract` / `WorkbenchPageReadModelInventory` 类型。
- 在 `WorkbenchSnapshot` 内新增只读 `page_read_model_inventory` 字段，作为 R4-A1 合同输出。
- 前端 TS 类型同步。
- 可在 Settings 开发者区或现有诊断区只读展示 page read model inventory 摘要；普通主页面不必接入。
- 离线测试补一条 inventory 字段存在和主导航边界测试。

R4-A1 不要求：

- 不新增按页 Tauri command。
- 不把任一页面改为只读新 command。
- 不拆 `ProjectsView.tsx` / `AgentView.tsx` 大组件。
- 不改 CSS / 视觉风格。
- 不改导航结构。

## 5. 文件落点

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx` 或现有诊断只读区域（如确有必要）
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `tasks/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1-result.md`
- 当前入口文档和正式计划 checkpoint。

禁止修改：

- 不改真实 Codex runner。
- 不改 SQLite production path。
- 不新增 Tauri command，除非任务执行中证明只新增 contract 不足以验收且复核同意。
- 不改 UI 视觉风格 / 布局。
- 不启动真实 Tauri / 截图验收。

## 6. 验收

必须通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib page_read_model`
- `cargo test --lib`
- `cargo fmt -- --check`
- `git diff --check`

扫描：

- 不得出现“R4 已完成 / 页面已按页查询完成 / WorkbenchSnapshot 已废弃”冒领。
- 不得新增真实 `codex exec` / `codex exec resume`。
- 不得新增 `.codex` / secret / token / credential / full transcript 真实读取路径。

## 7. 禁止声明

R4-A1 禁止声明：

- R4 完成。
- 已完成所有页面按页查询。
- `WorkbenchSnapshot` 已废弃。
- `ProjectsView` / `AgentView` 已拆分完成。
- 前端 UI 已重做。
- 真实 Tauri / 截图验收完成。
- R3 Level B 已执行。
- 多 agent 并行真实执行已解锁。

## 8. 形状预算

- 是否允许新增 Rust 文件：已新增小模块 `src-tauri/src/page_read_model.rs`，用于满足 shape gate 不增长巨型文件的治理要求。
- `lib.rs` 新增行数目标：`<= 120`。
- `types.rs` 新增行数目标：`<= 120`。
- `types.ts` 新增行数目标：`<= 120`。
- UI 改动目标：`<= 80` 行。
- 是否允许新增 Tauri command：否。
- 是否允许新增 sidecar JSON 种类：否。
- 本任务规划基线 commit：`6786faf`
- 本任务 implementation commit：`93bc0f2ec5eb2f6e18297e43b5731afa4344876e`
- 本任务 checkpoint commit：待入口同步提交后回填。

执行说明：

- 实现时新增 `src-tauri/src/page_read_model.rs`、`src/lib/pageReadModel.ts`、`tests/fixtures/pageReadModelFixture.ts` 和 `tests/r4-page-read-model-settings.test.tsx`。
- 该调整偏离“默认不新增 Rust 文件”的初始偏好，但原因是 `workbench-shape-gate` 禁止 `types.rs` / `types.ts` / `offline-permission-dialog.test.tsx` 继续增长；新增小模块比继续喂大文件更符合 Root Treatment 治理目标。
- R4-A1 仍未新增 Tauri command、未新增 sidecar JSON、未切页面真实数据来源。

## 9. 完成结果

完成项：

- 后端新增 `page_read_model` 小模块，输出 `WorkbenchPageReadModelInventory` / `PageReadModelContract`。
- `WorkbenchSnapshot.page_read_model_inventory` 输出 9 个页面合同：Home、Projects、Agents、Running Workflows、Memory、Knowledge、Settings、Skill、Harness。
- 前端新增 `pageReadModel.ts` 类型模块；`SettingsView` 开发者区只读展示页面读模型合同摘要。
- 离线测试 runner 支持多个测试入口，并新增 R4-A1 设置页合同展示小测试。
- shape gate 保持 ratchet 文件不增长：`types.rs`、`types.ts`、`offline-permission-dialog.test.tsx` 均回到水位线。

未完成项：

- 未新增按页 Tauri command。
- 未把任何页面迁移到按页查询。
- 未拆 `ProjectsView.tsx` / `AgentView.tsx`。
- 未改 UI 视觉风格 / 布局。
- 未做真实 Tauri / 截图验收。
- 未执行 R3 Level B。

## 10. 交接要求

完成后必须写：

- evidence：`evidence/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1-result.md`

handoff 必须包含：

- 页面数据需求矩阵。
- 新增字段 / 类型 / helper。
- 验证命令结果。
- 未完成项：按页 command、逐页迁移、大组件拆分、UI 重做。
- 下一步建议：R4-A2 后端按页查询 skeleton 或 R4-A2 Projects/Agents 首批页面迁移。
