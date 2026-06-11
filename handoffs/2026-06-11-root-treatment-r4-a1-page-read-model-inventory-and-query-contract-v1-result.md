# Root Treatment R4-A1 Page Read Model Inventory And Query Contract Handoff v1

日期：2026-06-11

结论：R4-A1 已完成，状态为 `accepted_contract_only`。

## 做了什么

- 新增后端 `page_read_model` 小模块，派生固定页面读模型合同 inventory。
- `WorkbenchSnapshot` 新增 `page_read_model_inventory` 字段。
- 新增前端 `pageReadModel.ts` 类型模块。
- 设置页开发者区新增只读“页面读模型合同”面板，显示合同数量、迁移边界和页面合同清单。
- 离线测试 runner 支持多入口，并新增 R4-A1 设置页合同展示测试。
- 通过 shape gate，且没有让 `types.rs`、`types.ts`、`offline-permission-dialog.test.tsx` 超过水位线。

Implementation commit：`93bc0f2ec5eb2f6e18297e43b5731afa4344876e`

## 关键文件

- `prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/r4-page-read-model-settings.test.tsx`
- `prototypes/productized-desktop-shell/tests/fixtures/pageReadModelFixture.ts`

## 验证

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`。
- `npm run build`：通过，仅既有 Vite chunk-size warning。
- `cargo test --lib page_read_model`：通过。
- `cargo test --lib`：通过，469 passed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。

## 页面数据需求矩阵

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

## 不能声明

- 不能声明 R4 完成。
- 不能声明已完成所有页面按页查询。
- 不能声明 `WorkbenchSnapshot` 已废弃。
- 不能声明 `ProjectsView` / `AgentView` 已拆分完成。
- 不能声明前端 UI 已重做。
- 不能声明真实 Tauri / 截图验收完成。
- 不能声明 R3 Level B 已执行。
- 不能声明多 agent 并行真实执行已解锁。

## 下一步建议

下一步进入 R4-A2。建议优先做“后端按页查询 skeleton / selector contract”，但仍不迁移所有页面；另一条可选路线是先拿 Projects / Agents 两个高风险页面做读模型 selector 分域。无论哪条，都必须继续让 shape gate 通过，不能把 R4-A1 的合同当成按页查询已经完成。
