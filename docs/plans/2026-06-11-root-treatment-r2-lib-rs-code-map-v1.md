# Root Treatment R2 lib.rs Code Map v1

日期：2026-06-11

## 目的

本文是 Root Treatment / Stage R / R2 的静态代码地图，用于指导后续 `lib.rs` 解体批次。它不授权新增产品功能，不授权真实 Codex 执行，不授权 UI / SQLite / workflow read model 之外的顺手拆分。

初始行号基于 R2-B2 helper 抽出后的 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，当时总行数为 25,643。R2-B3 完成后为 24,635 行；R2-B4 完成后为 23,524 行；R2-B5 完成后为 21,463 行；R2-B6 完成后为 19,401 行；R2-B7 完成后为 18,932 行；R2-B8 完成后为 17,042 行；R2-B9 完成后为 16,457 行。后续 R2 批次继续移动代码后，行号会漂移，应在新的治理任务包 evidence 中记录新的前后指标。

## 当前结构

| 行号区间 | 主要领域 | 当前职责 | 调用 / 测试落点 |
| --- | --- | --- | --- |
| 1-118 | crate 装配与保守 include | 模块声明、`AppState`、`types.rs`、`commands.rs`、`command_registry.rs` | `cargo test --lib`，shape gate command count |
| 119-243 | index / transcript 读取 | index JSON 读取、sqlite/index transcript fallback | transcript catalog / reader tests |
| 244 | R2-B3 workflow state lifecycle / task package include | `workflow_state_lifecycle_task_package.rs` 在 crate root 展开 | workflow state / task package tests |
| 246 | R2-B4 workflow run / binding / legacy dispatch include | `workflow_run_dispatch_entrypoints.rs` 在 crate root 展开 | workflow run / dispatch tests |
| 248-2759 | C4-C6 自动化工作流治理 | project director plan、authorized dispatch、worker report、process fact、final review、user decision、acceptance summary | C4 / C5 / C6 tests |
| 2760 | R2-B6 workflow execution include | `workflow_execution_entrypoints.rs` 在 crate root 展开 | dispatch / permission / offline role / workflow machine filters |
| 2762-3374 | task package 渲染 / finder helper | work item / artifact finder、render fields、markdown、user reviewed instruction preview | task package render tests |
| 3375 | R2-B2 workflow state JSON helper include | `workflow_state_json_helpers.rs` 在 crate root 展开 | `cargo test --lib workflow_state` |
| 3378 | R2-B7 memory command bridge / context guard include | `memory_context_entrypoints.rs` 在 crate root 展开 | memory / observation / task packet tests |
| 3380-3720 | shared workflow utility | state transitions、node IDs、index project lookup、stable IDs、task package path helpers | workflow transition / generated task tests |
| 3721 | R2-B5 workflow read model include | `workflow_read_model_entrypoints.rs` 在 crate root 展开 | workflow read model / readback / snapshot tests |
| 3723-3806 | atomic path / time helpers | JSON write helper、default paths、timestamps | workflow state store helper tests |
| 3807-4205 | workbench snapshot assembly | `WorkbenchSnapshot` assembly and session source overlay | snapshot tests |
| 4206 | R2-B8 diagnostics / provider / continuation / adapter include | `diagnostics_provider_session_entrypoints.rs` 在 crate root 展开 | diagnostics / provider / continuation / adapter / session operation filters |
| 4209 | R2-B9 index / host / app include | `index_host_app_entrypoints.rs` 在 crate root 展开 | transcript / snapshot / workflow state tests |
| 4211-16457 | inline tests | broad historical test module covering many domains | `cargo test --lib`, focused filters by domain |

## R2 批次建议

1. R2-B2：已抽出 `workflow_state_json_helpers.rs`，保持 `include!`，只搬 workflow state JSON helper。
2. R2-B3：已把 workflow state 生命周期入口和 task package 写入链拆成 crate-root include；测试落点为 `cargo test --lib workflow_state`、task package filters 和 workflow run check。
3. R2-B4：已把前段连续 workflow run check / binding / legacy dispatch entrypoints 拆成 crate-root include；这是主管线基于风险的顺序调整，workflow read model 汇合顺延。
4. R2-B5：已把 workflow read model / dispatch summary / readback stats 拆成 crate-root include；测试落点为 workflow read model、dispatch readback 和 snapshot tests。
5. R2-B6：已把 workflow dispatch 执行控制、offline role dispatch、workflow machine 分域；测试落点为 dispatch、permission、offline role、workflow machine filters。
6. R2-B7：已把 memory / observation / task memory context guard 从 `lib.rs` 抽出到 `memory_context_entrypoints.rs`；测试落点为 memory candidate、formal memory、observation、task packet filters。
7. R2-B8：已把 diagnostics、provider availability、session continuation 和 adapter descriptors 分域；测试落点为 diagnostics、provider、continuation、adapter、session operation filters。
8. R2-B9：已把 index parsing、allowed paths、host OS helper 和 app assembly 尾段抽出到 `index_host_app_entrypoints.rs`；测试落点为 transcript、snapshot、workflow state 和全量 lib tests。
9. R2-B10：下一步整理 C4-C6 自动化工作流治理；建议限定 project director plan、authorized dispatch、worker report、process fact、final review、user decision、acceptance summary。
10. R2 后段：测试模块按被抽出的领域逐步迁移，避免 `lib.rs` 代码下降但测试仍保持巨石。

## P2

- 本地图是人工静态地图，不是自动维护的 AST 或 call graph；后续行号需随任务包更新。
- 当前建议批次是治理顺序，不代表所有依赖已经解耦；每批仍必须以编译和聚焦测试证明行为不变。
- R2 只负责 `lib.rs` 解体，不代表 R3 SQLite、R4 按页读模型或 Stage L 恢复已完成。
