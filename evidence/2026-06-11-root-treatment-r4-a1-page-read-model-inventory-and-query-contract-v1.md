# Root Treatment R4-A1 Page Read Model Inventory And Query Contract Evidence v1

日期：2026-06-11

结论：`accepted_contract_only`

R4-A1 已完成为页面读模型 inventory / contract skeleton。接受范围是页面数据需求矩阵、`WorkbenchSnapshot.page_read_model_inventory`、前后端类型边界和设置页开发者区只读展示。不接受为 R4 完成、按页 Tauri command 完成、页面数据来源迁移完成、`WorkbenchSnapshot` 废弃、UI 重做、真实 Tauri / 截图验收、R3 Level B 或多 agent 并行真实执行解锁。

## Implementation

- Implementation commit：`93bc0f2ec5eb2f6e18297e43b5731afa4344876e`
- Checkpoint commit：`6519ad3`
- Checkpoint hash 回填 commit：`03fd247`
- Task package：`tasks/2026-06-11-root-treatment-r4-a1-page-read-model-inventory-and-query-contract-v1.md`
- 新增后端模块：`prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- 新增前端类型模块：`prototypes/productized-desktop-shell/src/lib/pageReadModel.ts`
- 新增测试 fixture：`prototypes/productized-desktop-shell/tests/fixtures/pageReadModelFixture.ts`
- 新增 R4-A1 前端小测试：`prototypes/productized-desktop-shell/tests/r4-page-read-model-settings.test.tsx`

## Contract Matrix

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

## Shape Gate

`node scripts/harness/workbench-shape-gate.js --mode check` 通过。

关键水位：

- `lib.rs`: `13965 / 25925`，decreased。
- `types.rs`: `5386 / 5386`，same。
- `types.ts`: `5149 / 5149`，same。
- `offline-permission-dialog.test.tsx`: `9369 / 9369`，same。
- Tauri commands：`96 total; 0 in lib.rs`。
- Sidecar JSON kinds：`14 detected; 0 unknown`。

过程说明：最初尝试在 `types.rs` / `types.ts` / 主离线测试文件内直接补类型和断言会触发 ratchet failure。最终改为新增小模块和小测试文件，保持巨型文件不增长。这是 R4 治理目标内的形状修正，不是功能扩张。

## Verification

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 14`，并输出 `r4 page read model settings test passed`。
- `npm run build`：通过，仅保留既有 Vite chunk-size warning。
- `cargo test --lib page_read_model`：通过，1 passed。
- `cargo test --lib`：通过，469 passed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。

## Boundary

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 未新增 Tauri command。
- 未新增 sidecar JSON。
- 未切 SQLite production DB。
- 未迁移页面真实数据来源。
- 未改 UI 视觉风格 / 布局。
- 未启动 Tauri / Browser / Chrome / 截图工具。

## Deferred

- R4-A2+ 需要决定下一步是后端按页查询 skeleton，还是先做 Projects / Agents selector 分域。
- `WorkbenchSnapshot` 仍是当前页面主数据来源。
- R3 Level B 仍未执行。
- Stage L / L1-L6 继续 `deferred_during_root_treatment`。
