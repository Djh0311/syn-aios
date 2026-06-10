# Evidence：Memory Layer M13 Final Authoritative Acceptance And Conclusion Freeze v1

日期：2026-06-05

## 最终结论

M13 已完成，最终结论为：

```text
accepted_with_deferred_items
```

接受为：

- 中间版本记忆系统最终权威验收完成。
- M1-M12.1 的关键证据、测试、边界、UI 可理解性和后置项已统一复核。
- 第一条真实记忆闭环、受控正式记忆系统、中间版本记忆系统和最终蓝图后置能力已区分。
- M12.1 mature pattern 正式化后 acceptance summary 使用 fresh formal store 的修补已纳入最终结论。

不接受为：

- 最终蓝图完整工作台完成。
- 阶段 G 真实 Tauri 全面验收完成。
- 真实 worker / Codex 已执行。
- GraphRAG、向量库、图数据库、自动索引重建、完整理解地图、Obsidian 原生同步或自动技能化完成。
- 候选、observation、knowledge hit、maintenance report、cluster report、relation candidate、LLM summary 或 graph/index report 可以绕过正式记忆状态机。

## 结论分层

第一条真实记忆闭环：

- 结论：accepted。
- 依据：M1-M6 已覆盖 formal memory store、context binding、candidate adoption、observation、task memory packet preview、lint blocking 和 workflow task package injection。
- 边界：M1-M6 只证明第一条真实闭环，不能单独代表完整记忆系统。

受控正式记忆系统：

- 结论：accepted for intermediate version。
- 依据：M1 / M1.1 / M2 / M9 已覆盖 record、source refs、version、audit、context binding、候选采纳、生命周期 revise / deprecate / freeze / archive / merge / split / promote / demote。
- deferred：完整 `MemoryPage` Markdown 展示页和外部编辑器受控编辑形态仍属最终蓝图 / 后续任务，不阻断当前受控正式记忆系统结论。

中间版本记忆系统：

- 结论：accepted_with_deferred_items。
- 依据：M7-M12.1 已补齐记忆中心 UI、知识库边界、实体关系、维护任务、成熟模式、跨项目主题报告、M1-M12 gate 摘要和 M12.1 fresh summary 修补。
- deferred：真实 Tauri 全面验收、运维日志、自动重试、模型 / adapter / 多 agent 深化、项目画布产品化深化仍后置。

最终蓝图能力：

- 结论：deferred。
- 后置能力包括 GraphRAG、向量库、图数据库、完整理解地图、自动技能化、Obsidian 原生同步、vault 扫描、完整运维日志、真实 worker / Codex 自动执行闭环。

## Gate 复核

| Gate | 结论 | 依据 / 测试 | 缺口或说明 |
| --- | --- | --- | --- |
| `observation_gate` | passed | M3 evidence；`cargo test --lib observation` 13 passed；普通聊天自动 capture 被拒绝，worker / process fact observation 受控记录。 | observation 仍不是正式记忆。 |
| `candidate_gate` | passed | M2 / M4 evidence；`cargo test --lib memory_candidate` 9 passed；`task_memory_packet_excludes_candidates_as_unconfirmed`。 | candidate 只能作为 review material 或采纳来源。 |
| `formal_memory_gate` | passed | M1 / M1.1 / M9 evidence；`cargo test --lib formal_memory` 29 passed；record / version / audit / source / status / scope 均覆盖。 | Markdown `MemoryPage` 完整形态后置。 |
| `context_binding_gate` | passed | M1.1 evidence；`formal_memory_context_*` 测试包含 project / workflow mismatch、missing project、project director cross-project。 | 无阻断项。 |
| `adoption_gate` | passed | M2 evidence；`memory_candidate_adoption_*` 测试覆盖低风险项目采纳、用户偏好必须用户、secret / cross-project / rejected 拒绝。 | sidecar 非事务 orphan 风险已在 M2 暴露，后续维护任务可继续增强补偿，不阻断。 |
| `task_packet_gate` | passed | M4 evidence；`cargo test --lib task_memory_packet` 10 passed；active formal memory included，candidate / observation / inactive / no permission / model export blocked 均 excluded。 | 无阻断项。 |
| `lint_gate` | passed | M5 / M11 evidence；`cargo test --lib memory_lint` 9 passed；blocking finding 阻断采纳和召回。 | M11 finding lifecycle UI 可继续后置深化。 |
| `workflow_injection_gate` | passed | M6 evidence；`cargo test --lib task_memory_injection` 5 passed；task package artifact / markdown / dispatch prompt 使用同一 frozen snapshot，stale 可检测。 | 不执行真实 worker。 |
| `memory_center_gate` | passed | M7-M12 evidence；`npm run test:offline-interaction` 9 scenarios passed；记忆中心区分正式、候选、observation、lint、relation、maintenance、mature pattern。 | 真实 Tauri 截图 deferred。 |
| `knowledge_boundary_gate` | passed | M8 evidence；知识库资料只作为 `knowledge_doc` source / candidate input，不直接写 formal memory。 | Obsidian 原生同步和 vault 自动扫描 deferred。 |
| `lifecycle_gate` | passed | M9 evidence；formal memory lifecycle tests 覆盖 revise / deprecate / freeze / archive / merge / split / promote / demote，且有 version / audit / confirmation。 | 无阻断项。 |
| `entity_relation_gate` | passed | M10 evidence；`cargo test --lib memory_entity_relation` 5 passed；LLM inferred causal relation stays candidate；confirmed relation only explains task packet。 | 图数据库 / GraphRAG deferred。 |
| `maintenance_gate` | passed | M11 evidence；maintenance run 只生成 finding / report / recommendation，不修改 formal memory / candidate / relation store；mature pattern signal 不自动正式化。 | 真实 Tauri 数据桥验收 deferred。 |
| `mature_pattern_gate` | passed | M12 evidence；`cargo test --lib mature_pattern` 5 passed；unconfirmed candidate / cluster report 不入包；用户确认后才写 formal mature pattern memory。 | M12 真实窗口 / 截图 deferred。 |
| `m12_1_freshness_gate` | passed | M12.1 evidence；用户确认写入后 reload fresh formal store；formal gate evidence `record 1 / version 1 / audit 1`，task packet gate passed。 | 无阻断项。 |
| `ui_gate` | deferred | `npm run typecheck` / `npm run test:offline-interaction` / `npm run build` 通过；前端源码禁用文案扫描无命中。 | 真实 Tauri 全面验收和截图未完成，转阶段 G / 专门 UI 验收，不阻断记忆系统语义验收。 |
| `security_boundary_gate` | passed | M13 本轮未执行真实 worker / Codex，未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未扫描完整 transcript，未接 GraphRAG / 向量库 / 图数据库。 | 历史 M11 evidence 记录过为 Browser smoke 读取 Browser 插件 skill 文件；这不是 M13 本轮行为，继续保留为历史边界记录。 |

## 真实窗口 / 截图验收判断

M12 真实窗口 / 截图验收缺口处理为 deferred，不阻断 M13 最终结论。

理由：

- M13 本轮不改前端、不改读模型、不改 UI 文案，不引入新的可见交互风险。
- 前端三项验证已通过：typecheck、offline interaction、build。
- 后端和记忆治理语义已由 Rust 筛选测试和全量 `cargo test --lib` 覆盖。
- 真实 Tauri 全面验收属于阶段 G 目标，当前阶段计划已明确后置。
- 当前可用 Browser / screenshot 技能说明位于 `/Users/yoyi/.codex/...`；M13 stop 条件禁止为验收擅自读写该路径，因此本轮未调用该技能。

不接受为：

- M12 UI 已完成真实 Tauri 数据桥或真实窗口截图验收。
- 阶段 G 真实 Tauri 全面验收完成。

## UI 禁止文案复核

扫描范围：

```text
prototypes/productized-desktop-shell/src
```

扫描命令：

```text
rg -n "最终蓝图完整完成|真实 worker 已执行|真实 Codex 已执行|GraphRAG 已完成|自动技能化完成|候选已成为正式事实|聚类报告就是事实|已记住|系统已长期记住|候选已成为正式记忆|观察已成为正式记忆|worker 已收到记忆包|中间版本记忆层已完成" src
```

结果：无命中。

说明：未扫描 evidence / handoff 作为 UI 文案源，因为这些文件会包含负向边界记录，例如“不能说真实 worker 已执行”。

## 验证命令

前端：

```text
npm run typecheck
passed
```

```text
npm run test:offline-interaction
offline interaction tests passed: 9
```

```text
npm run build
passed
```

说明：`npm run build` 保留既有 Vite chunk size warning，不影响构建通过。

Rust：

```text
cargo test --lib formal_memory
29 passed; 0 failed
```

```text
cargo test --lib memory_candidate
9 passed; 0 failed
```

```text
cargo test --lib observation
13 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib task_memory_injection
5 passed; 0 failed
```

```text
cargo test --lib memory_lint
9 passed; 0 failed
```

```text
cargo test --lib memory_entity_relation
5 passed; 0 failed
```

```text
cargo test --lib mature_pattern
5 passed; 0 failed
```

```text
cargo test --lib memory_cluster
2 passed; 0 failed
```

```text
cargo test --lib
221 passed; 0 failed; 1 ignored
```

说明：Rust 测试仍保留既有 `JsonRpcError::invalid_params` dead_code warning。

格式：

```text
rustfmt --check src/formal_memory_store.rs src/formal_memory_lifecycle.rs src/memory_candidate_store.rs src/observation_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/mature_pattern_store.rs src/mature_pattern_governance.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
passed
```

## Deferred Items

这些 deferred items 不阻断 M13 记忆系统最终权威验收，但必须在后续阶段继续拆任务：

- 阶段 G：真实 Tauri 全面验收、截图、权限弹层、记忆中心真实数据桥、运行日志和运维诊断。
- 阶段 G / 工作流线：完整自动重试、恢复策略、错误码和真实 worker / Codex 自动执行闭环。
- 阶段 E：Claude Code / OpenClaw / OpenCode 等 adapter 真接入、模型凭据、成本和权限边界。
- 阶段 F：项目工作流画布产品化深化，React Flow 仍只是渲染映射，不是事实源。
- 最终蓝图后置：Obsidian 原生同步、vault 自动扫描、GraphRAG、向量库、图数据库、自动索引重建、完整理解地图、自动技能化。
- 记忆层深化：完整 `MemoryPage` Markdown 展示页、外部编辑器受控编辑、finding lifecycle UI 和维护补偿流程。

## 边界

M13 本轮：

- 未改产品代码。
- 未新增记忆能力。
- 未新增 sidecar。
- 未新增 Tauri command。
- 未新增前端入口、右侧入口、项目页 tab 或 UI 文案。
- 未改正式记忆 schema。
- 未改 workflow state 顶层结构。
- 未迁移数据库。
- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未扫描完整 transcript。
- 未接 Obsidian 原生写入、GraphRAG、向量库、图数据库或自动索引重建。
- 未把候选、observation、knowledge hit、maintenance report、cluster report、relation candidate、LLM summary 或 graph/index report 当正式事实。
