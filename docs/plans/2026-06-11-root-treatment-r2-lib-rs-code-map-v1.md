# Root Treatment R2 lib.rs Code Map v1

日期：2026-06-11

## 目的

本文是 Root Treatment / Stage R / R2 的静态代码地图，用于指导后续 `lib.rs` 解体批次。它不授权新增产品功能，不授权真实 Codex 执行，不授权 UI / SQLite / workflow read model 之外的顺手拆分。

初始行号基于 R2-B2 helper 抽出后的 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`，当时总行数为 25,643。R2-B3 完成后为 24,635 行；R2-B4 完成后为 23,524 行。后续 R2 批次继续移动代码后，行号会漂移，应在新的治理任务包 evidence 中记录新的前后指标。

## 当前结构

| 行号区间 | 主要领域 | 当前职责 | 调用 / 测试落点 |
| --- | --- | --- | --- |
| 1-118 | crate 装配与保守 include | 模块声明、`AppState`、`types.rs`、`commands.rs`、`command_registry.rs` | `cargo test --lib`，shape gate command count |
| 119-243 | index / transcript 读取 | index JSON 读取、sqlite/index transcript fallback | transcript catalog / reader tests |
| 244-546 | workflow state 生命周期入口 | snapshot、initialize、bootstrap、默认项目工作流写入 | `cargo test --lib workflow_state` |
| 547-1253 | task package 写入链 | draft、preview、field update、file generation、dispatch readiness | task package tests |
| 1254-2382 | workflow run check / binding / dispatch 入口 | run check、work item state、session binding、dispatch prepare / execute / readback context | workflow run / dispatch tests |
| 2383-4877 | C4-C6 自动化工作流治理 | project director plan、authorized dispatch、worker report、process fact、final review、user decision、acceptance summary | C4 / C5 / C6 tests |
| 4878-5887 | workflow dispatch 执行控制 | authorization、prepared / started / completed / failed / readback dispatch、director review、permission decision | workflow dispatch / permission tests |
| 5888-6362 | offline role dispatch | offline dispatch、handoff、director review | offline role orchestration tests |
| 6363-6920 | workflow machine | 四角色 workflow machine、round step、final acceptance、run result | workflow machine tests |
| 6943-7556 | task package 渲染 / finder helper | work item / artifact finder、render fields、markdown、user reviewed instruction preview | task package render tests |
| 7556 | R2-B2 workflow state JSON helper include | `workflow_state_json_helpers.rs` 在 crate root 展开 | `cargo test --lib workflow_state` |
| 7558-8029 | memory command bridge / context guard | formal memory、candidate adoption、memory lint、observation、task memory packet context binding | memory / observation / task packet tests |
| 8030-8369 | shared workflow utility | state transitions、node IDs、index project lookup、stable IDs、task package path helpers | workflow transition / generated task tests |
| 8370-10481 | workflow snapshot / read model / dispatch summaries | counts、project workflow summaries、blackboard summaries、task packages、ledger、exceptions、dispatch summaries、readback stats | workflow read model / readback tests |
| 10482-10517 | atomic path / time helpers | JSON write helper、default paths、timestamps | workflow state store helper tests |
| 10518-10916 | workbench snapshot assembly | `WorkbenchSnapshot` assembly and session source overlay | snapshot tests |
| 10917-11493 | diagnostics / store integrity | diagnostic summary、store integrity、sidecar integrity | diagnostics tests |
| 11494-12509 | provider / continuation / adapter descriptors | provider availability、session continuation previews, guard, adapter descriptors | provider / continuation / adapter tests |
| 12510-12822 | session operation descriptors | session operation specs and descriptor derivation | session operation descriptor tests |
| 12823-13329 | index parsing / allowed paths | session loading, project/session/skill/plugin/task parsing, allowed path derivation | parsing / allowed path tests |
| 13330-13377 | host OS helper boundary | clipboard and `open` command wrappers | path / open command guarded tests |
| 13378-13397 | Tauri app assembly | builder, state management, invoke handler, setup | compile + shape gate |
| 13398-25643 | inline tests | broad historical test module covering many domains | `cargo test --lib`, focused filters by domain |

## R2 批次建议

1. R2-B2：已抽出 `workflow_state_json_helpers.rs`，保持 `include!`，只搬 workflow state JSON helper。
2. R2-B3：已把 workflow state 生命周期入口和 task package 写入链拆成 crate-root include；测试落点为 `cargo test --lib workflow_state`、task package filters 和 workflow run check。
3. R2-B4：已把前段连续 workflow run check / binding / legacy dispatch entrypoints 拆成 crate-root include；这是主管线基于风险的顺序调整，workflow read model 汇合顺延。
4. R2-B5：当前下一批，把 workflow read model / dispatch summary / readback stats 与既有 `workflow_read_model.rs` 边界汇合；测试落点为 workflow read model、dispatch readback 和 snapshot tests。
5. R2-B6：把 workflow dispatch 执行控制、offline role dispatch、workflow machine 分域；测试落点为 dispatch、permission、offline role、workflow machine filters。
6. R2-B7：把 memory / observation / task memory context guard 从 `lib.rs` 汇合到对应 memory 模块；测试落点为 memory candidate、formal memory、observation、task packet filters。
7. R2-B8：把 diagnostics、provider availability、session continuation 和 adapter descriptors 分域；测试落点为 diagnostics、provider、continuation、adapter filters。
8. R2-B9：整理 index parsing、allowed paths、host OS helper 和 app assembly；目标是让 `lib.rs` 只保留模块声明、include / mod 装配、`AppState` 和 `run()`。
9. R2 后段：测试模块按被抽出的领域逐步迁移，避免 `lib.rs` 代码下降但测试仍保持巨石。

## P2

- 本地图是人工静态地图，不是自动维护的 AST 或 call graph；后续行号需随任务包更新。
- 当前建议批次是治理顺序，不代表所有依赖已经解耦；每批仍必须以编译和聚焦测试证明行为不变。
- R2 只负责 `lib.rs` 解体，不代表 R3 SQLite、R4 按页读模型或 Stage L 恢复已完成。
