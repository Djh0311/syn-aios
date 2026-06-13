# Root Treatment / Stage R 剩余执行计划 v1（合并正本）

日期：2026-06-13
出自：咨询 / 主管线（Claude）。
性质：**Stage R 剩余执行（R4 后半 → R5）的唯一执行正本。**合并自两份——R4 硬目标执行规划（handoff）+ 后端 util 去重方案（plan）。**承接**官方开发计划 `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`（R0-R5 总框架）：本文不重复总框架，只定剩余执行的时序与子项。原两份已标作废 / 历史，后续以本文为准。

> **拍板摘要**：接下来 R 阶段的活本来散在两份（前端 H 系列、后端 U 系列），容易漂移。合并成这一条时序线：**先拆完前端（R4-H）→ 清后端重复 util（R-U）→ 真切 SQLite（R3 Level B）→ 文档对齐（R5）**。一份正本，照着走。

## 1. 时序总览

```
R4-H 前端硬目标（进行中）
  → R-U 后端 util 去重（无行为变化）
    → R3 Level B SQLite 真实切换（需用户在场）
      → R5 文档与蓝图对齐 → Stage R 收口
```

时序理由：R4-H 碰前端、R-U 碰后端 store、R3 也碰 store——R-U 排在 R4 之后、R3 之前，不和 R4 撞（不同文件），又趁 R3 还没动 store 先清掉重复。

## 2. R4-H 前端硬目标（并入自 R4 硬目标规划）

依据 `decisions/2026-06-13-root-treatment-r2-late-stage-closure-track-v1.md` §5。

进度：
- **H1 ✅** `types.ts` 分域（4,998 → 93）。
- **H2 ✅** WorkbenchSnapshot 按页查询（批一，前端切按页取，`types.rs` 降到 5,229）。
- **H3 ✅** View 按目标布局区块拆分（批二）已完成——设计稿 `docs/plans/2026-06-13-root-treatment-r4-h3-project-agent-view-layout-block-split-design-v1.md` 已确认；已按顺序完成 AgentView（**H3-4 ✅** 对话区 / **H3-5 ✅** 开发者面板）与 ProjectsView（**H3-1 ✅** 壳和概览 / **H3-2 ✅** 中央工作流画布 / **H3-3 ✅** 右侧详情、治理、记忆和执行面板）。

边界：只拆结构、行为视觉零变更；不碰 store / Rust 业务 / 真实执行。拆分基准按 Xuanji §16 目标布局区块，不按现状外观。

## 3. R-U 后端 util 去重（并入自 util 去重方案，无行为变化）

基于 2026-06-13 代码扫描：后端重复 util `sha256_hex` × 23、`short_hash` × 14、`sidecar_path` × 12、`normalize` × 12、`remove_file_if_exists` / `fixture_dir` × 4–6。根因：逐文件开发、无查重门。前端较干净（仅 `DetailLine` / `SummaryTile` 重复）。

做（分批，每包独立复核、独立 commit）：
- **U1 ✅** `sha256_hex` / `short_hash` → `utils/hash.rs`，已完成并经独立复核 `STATUS: CLEAR`，implementation commit `e6325e8`。
- **U2 ✅** `sidecar_path`（加 `store_name` 参数）→ `utils/store_paths.rs`，已完成并经独立复核 `STATUS: CLEAR_WITH_P2`，P2 已补齐，implementation commit `1ba8f01`。
- **U3 ✅** `remove_file_if_exists` / `fixture_dir` → `utils/fs_ops.rs`，已完成并经独立复核 `STATUS: CLEAR_WITH_P2`，P2 已补齐，implementation commit `bc436dd`。
- **U4 ✅** `normalize` + 特化 → `utils/normalization.rs`，已完成并经独立复核 `STATUS: CLEAR_WITH_P2`，P2 已补正，implementation commit `16e96bd`；规则不同 / 业务特化项保持 deferred。
- **U5 ✅** 前端 `DetailLine` / `SummaryTile` → `src/components/`，已完成并经独立复核 `STATUS: CLEAR`，implementation commit `c4335e1`。
- **U-Gate** 查重门形态先写草稿（2-3 种方案 + 推荐），暂不实现、不接入 harness / CI。

不做：store 模式（`load_store` / `empty_store` / `validate_store`）**不强行合并**——每店数据结构 / 业务规则不同，合并会碰状态机 / JSON，违反 `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`。

边界：严格无行为变化（`cargo test --lib` 全绿为铁证），符合模块拆分边界 decision；不碰 store 业务逻辑 / JSON / 状态机；不迁 SQLite。停止条件：若抽某 util 牵连改 store 语义 → 停，该 util 留原地记 deferred。

## 4. R3 Level B SQLite 真实切换

指向窗口计划 `docs/plans/2026-06-13-root-treatment-r3-level-b-execution-window-plan-v1.md`（已写，待用户排期 + 在场执行）。**多 agent 硬门槛：R3 收口前不开多 agent 并行真实执行。**

## 5. R5 文档与蓝图对齐

承接官方开发计划 R5：蓝图正本迁移 / 对齐口径、两份吸收文档与 M1-M13/C1-C6 查重、Stage L 纯文档项并入。

## 6. 通用约束（全程）

- 每包：任务包 → 实现 → 离线验证（typecheck / 相关测试 / shape gate / `git diff --check`；Rust 包加 `cargo test --lib`）→ 独立复核线 `CLEAR` → commit → checkpoint → 停复核点。
- **复核分级**：脚本能验的（拆文件 / util 去重，机器焊死行为）同源复核够；判断密集的（R3 Level B 真实切换）跨模型或用户在场。
- 冻结边界：不解冻 backlog、不真实接入 planned adapters、未授权不碰 `~/.codex`、R3 收口前不开多 agent 并行。
- 轮班制度 `handoffs/2026-06-12-supervisor-line-rotation-protocol-v1.md` 适用。

## 7. 待确认

- R-U 的 Stage R 子项编号（主管线定）。
- U-Gate 查重门形态（gate 脚本 / 文档规则，动工时定）。
