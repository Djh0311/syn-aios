# Handoff：Memory Layer M12 Mature Pattern Cross Project Memory And Complete Acceptance v1

日期：2026-06-05

## 本轮结果

M12 已完成。记忆层现在具备成熟模式候选、跨项目主题报告、用户确认后正式 mature pattern memory 受控写入、任务包召回边界和 M1-M12 gate 摘要。

完成内容：

- 新增 M12 sidecar：`memory-patterns.v1.json`。
- 新增 mature pattern preview：从 formal memory、memory candidate、observation、M11 maintenance signal、M10 relation / entity 线索 deterministic 派生成熟模式候选和跨项目主题报告。
- 新增 mature pattern decision：confirm / reject / quarantine / request changes 都写 M12 sidecar audit。
- 用户确认正式化：只有 `actor_role: "user"` 且 `confirmed_by: "user"` 的 confirm 能写 formal memory，且写入 record / version / audit / source refs。
- 任务包边界：未确认 mature pattern candidate 和 cluster report 不进入 included list；用户确认后的 active formal mature pattern memory 可按 scope、lint、model export、token 等规则被评估召回。
- 记忆中心复用既有 `记忆` 入口展示成熟模式候选、跨项目主题报告和 M1-M12 gate 摘要。
- 权威入口已更新到 M12 完成，下一步指向 M13 最终权威验收。

## 接受范围

只接受为：

- `MaturePatternCandidate` 可由 M11 signal、重复候选、重复观察、正式记忆和关系线索派生。
- `MemoryClusterReport` / 跨项目主题报告可下钻 member refs 和 source refs。
- 用户确认后 mature pattern / global scope formal memory 能通过正式记忆 store 受控写入。
- 未确认候选和主题报告不能进入 task memory packet included list。
- 已确认且权限允许的 formal mature pattern memory 可被 task packet builder 评估召回。
- M1-M12 gate 摘要可解释 observation、candidate、formal memory、version、source、permission、conflict、audit、relation、maintenance、mature pattern 和 recall 链路。

不接受为：

- M13 最终权威验收完成。
- 最终蓝图完整记忆系统完成。
- 自动技能化或自动全局规则完成。
- 跨项目摘要直接影响 worker。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整理解地图完成。
- 真实 worker / Codex 已执行。

## 已完成矩阵

- MaturePatternCandidate：已完成。
- MemoryClusterReport：已完成。
- 用户确认正式化：已完成，且 project director / secretary 不能替代用户确认。
- task packet recall：已完成未确认排除和用户确认后 formal memory 评估召回。
- M1-M12 acceptance summary：已完成最小 gate 摘要。
- UI 最小入口：已完成，复用 `记忆` 页面。
- 权限确认弹层：已完成，覆盖 confirm 和 quarantine 边界。
- 真实窗口 / 截图验收：未完成。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，9 scenarios passed
- `npm run build`
- `cargo test --lib mature_pattern`，5 passed
- `cargo test --lib memory_cluster`，2 passed
- `cargo test --lib task_memory_packet`，10 passed
- `cargo test --lib formal_memory`，29 passed
- `cargo test --lib memory_lint`，9 passed
- `cargo test --lib memory_entity_relation`，5 passed
- `cargo test --lib`，221 passed / 1 ignored
- `rustfmt --check src/mature_pattern_store.rs src/mature_pattern_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/formal_memory_lifecycle.rs src/memory_lint_store.rs src/memory_entity_relation_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`

说明：

- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run build` 仍有 Vite chunk size warning。
- rustfmt 最初发现 M12 新增 Rust 文件格式差异，已运行 rustfmt 并重跑 check / tests 通过。

## 未完成 / 风险

- 真实窗口 / 截图验收未完成；不能声称 M12 UI 已完成真实 Tauri 数据桥或真实窗口截图验收。
- 本轮没有启动 in-app Browser，因为 M12 明确禁止读写 `/Users/yoyi/.codex`，而当前 Browser skill 说明位于 `.codex` 插件缓存下。
- M1-M12 gate 摘要是集成验收材料，不替代 M13 最终权威验收。
- `MemoryClusterReport` 是派生报告，不是正式事实源。
- 用户确认后写入的 formal mature pattern memory 仍要受 lint、status、scope、permission、model export 和 token guard 控制。

## 当前权威入口

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`

## 下一步

进入 M13：中间版本记忆系统最终权威验收和最终结论冻结。

M13 建议优先确认：

- M1-M12 的 gate 摘要是否足以支撑最终权威验收，哪些仍要补真实窗口 / 截图。
- observation -> candidate -> formal memory -> version -> audit -> task packet recall 的整链是否有最终验收证据。
- mature pattern / cluster report / relation / maintenance / knowledge doc 的“不是正式事实源”边界是否全部进入最终验收结论。
- 是否需要单独补 M12 真实窗口 / 截图验收，还是并入阶段 G / M13。

继续保持：

- 不执行真实 worker / Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不让 knowledge doc、candidate、observation、relation candidate、LLM summary、maintenance report、mature pattern candidate、cluster report 或 graph/index report 绕过正式记忆状态机。
