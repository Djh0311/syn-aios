# Stage L / L5 Memory Capture To Candidate Daily Loop v1

日期：2026-06-16

状态：待执行。本文是 Stage L 的 L5 任务包，用于把"工作台日常操作 → capture → observation → 候选 → 用户确认 → 正式记忆"打通成**日常可见、可确认的产品闭环**。L5 是产品闭环接线，**不改记忆 schema（仍是 R3 契约的 17 张表）、不新增真实执行、不自动写 FormalMemory、不碰 `/Users/yoyi/.codex`**。

一句话判据：如果日常工作台操作能自动喂进 capture/observation、生成的候选能在日常流里被用户看到并经确认门进入正式记忆，且验证证明没有改 schema、没有自动正式化、没有新增真实执行，则 L5 可进入复核。

## 0. 全局主管理解（基于 2026-06-16 只读代码核验）

已知事实：

- 记忆层 M1–M13 已建（`accepted_with_deferred_items`）：capture（`memory_capture_bus.rs` 1012 行）→ observation（`observation_store.rs` 688 行）→ candidate（`memory_candidate_store.rs` 546 行）→ formal（`formal_memory_store.rs` 411 行）+ 采纳（`formal_memory_lifecycle.rs` 1816 行 `adopt_memory_candidate_to_formal_memory`）。**管道齐全，逻辑不缺。**
- capture 已定义 8 个 source_type（`user_action` / `product_command` / `runtime_log` / `readback` / `worker_report` / `operation_control_decision`[L3 新增] / `process_fact_decision` / `final_review`，见 `memory_capture_bus.rs`），并按 `candidate_policy`（audit_only / blocked_sensitive / observation_only / candidate_allowed）分流。
- **真实缺口（已核）**：
  - 前端 `captureMemoryEvent` **零调用**——日常 UI 操作完全不喂记忆。
  - 后端只有**唯一一条**自动接线：`project_workflow_automation.rs:2837`（K3 Level-A 项目主管确认过程事实 → 写 observation，"只写 observation，不生成正式记忆"）。其余 `observation_store::` 调用都是 `load_store` 只读。
  - 候选只在专门的 `MemoryCenterView.tsx`（1340 行）里、经 M2 PendingAction→PermissionDialog 用户确认门采纳；**日常流（首页/运行页/项目工作流）里没有候选复核入口、没有"今天有 N 条待确认"的提示**。
- 采纳门（M2）健全：候选→正式必须用户确认；普通聊天不自动捕获；敏感/secret 走 `blocked_sensitive`；每条正式记忆带 source_refs。这些是硬规，L5 全部保留。

本任务核心判断：

```text
L5 不是造新机器，是让已建好的 capture→候选→确认 管道在日常里真正流起来、被看见、被确认；不改 schema、不自动正式化。
```

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`（§0 记忆闭环现状 / §1 强化闭环 / §2 L5 行 / §5 接受口径）
- `docs/memory-layer-design-v1.md`（§2 治理硬规 / §4 七层 / §6 采纳 / §14 不自动入 / §16 角色权限 / §18 存储）
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md` §3.3（记忆域 17 表 = schema 正本，L5 不得改）
- L1/L3 范例：`evidence/2026-06-10-stage-l-l1-...-v1.md`、`evidence/2026-06-16-stage-l-l3-...-v1.md`（"产品面 only、不接真实执行"对齐风格）
- M2/M3/M7 记录：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`、`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`、`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`、`docs/plans/task-package-ui-display-boundary-rule-v1.md`

（注：`docs/memory-layer-consolidated-canon-v1.md` 是咨询线汇编的便捷索引，当前未提交、不作为本包绑定权威；schema 与治理以上面的 design + R3 契约正本为准。）

## 2. 目标

L5 必须完成：

- **扩 capture 覆盖（重点补前端空白）**：把日常工作台关键操作接进 capture→observation。至少覆盖 §4 列出的操作；每条按 source_type + candidate_policy 分流，遵守敏感拦截。
- **建日常确认面**：给候选一个**日常可见、可达**的复核入口（不只埋在记忆中心），让用户在日常流里看到"待确认候选"并经确认门采纳。
- **确认仍走用户门**：复核面里的"采纳"复用 M2 受确认采纳链路（`adopt_memory_candidate_to_formal_memory`），**不自动正式化**；支持单条/批量确认与"暂不/拒绝"。
- **保边界与出处**：capture→observation→候选 全程保留 source_refs、scope、sensitive_level；敏感走 `blocked_sensitive` 不生成候选。
- **不改 schema**：复用现有 17 表与 sidecar；不新增表/不改冻结字段。
- 继续保持桌面端 Tauri 边界，不做手机端 UI。

## 3. 不做项

L5 不做：

- 不改 R3 记忆 schema（17 表）、不加表、不改冻结字段。
- 不自动写 FormalMemory；不把候选自动当正式；不绕过 M2 用户确认门。
- 不让子智能体/LLM 自治写正式记忆。
- 不自动捕获普通聊天；不把敏感/secret/token/transcript/prompt body/`.codex` 写进 capture/observation/候选。
- 不新增真实执行：不 `codex exec`/`resume`、不调 runner、不 `Command::new("codex")`、不放宽既有 real-resume 门、不读写 `/Users/yoyi/.codex`。
- 不把 capture/确认动作伪装成"已执行真实操作"或"已成功"。
- 不做手机端 UI、不改视觉风格主调。

## 4. capture 覆盖冻结

L5 至少把以下日常操作接进 capture（前端在对应确认/完成点调用 `captureMemoryEvent`，或后端在对应落账点调用 `capture_event`），每条标明 source_type 与默认 policy：

| 操作 | source_type | 默认 policy | observation_type |
|---|---|---|---|
| 用户确认运行控制决策（L3 retry/stop/restart/resume 确认后） | `operation_control_decision` | candidate_allowed | process_fact |
| 用户确认方案/计划采纳 | `user_action` | candidate_allowed | plan_adopted |
| 项目主管确认过程事实（C5/K3，扩展现有唯一后端接线的覆盖面） | `process_fact_decision` | observation_only→候选 | process_fact |
| 全局主管最终复核签字 | `final_review` | candidate_allowed | global_director_review |
| worker report 回收落账 | `worker_report` | observation_only | worker_report |

约束：

- 每条 capture 必须带 source_refs（操作来源、workflow/run/node 引用、hash），不带 prompt body/secret/`.codex` 原文。
- 敏感命中 → `blocked_sensitive`，只记审计、不生成候选。
- 普通聊天不在覆盖内。
- 覆盖面可按切片推进；本包不要求一次接满所有 8 种，但必须补上"前端零调用"这一硬缺口，并明确列出本包接了哪几条、留了哪几条到后续。

## 5. 日常确认面（产品形态）

L5 建一个**日常可见的候选复核面**（"记忆待办 / 候选收件箱"）：

- 显示当前待确认候选数与列表（claim 摘要、来源、scope、风险、生成时间）。
- **可达性**：从日常流直接进得去——推荐在首页/运行页放一个常驻入口或角标（"N 条记忆候选待确认"），点开进复核面；不要求用户特意去记忆中心翻。
- 复核动作：单条采纳 / 批量采纳 / 暂不 / 拒绝；采纳一律走 M2 用户确认门（经 PermissionDialog 看采纳预览），**不自动正式化**。
- 普通用户主层讲清：这是什么、为什么生成、采纳会发生什么（写入正式记忆、带出处、可回滚为新版本）、不采纳会怎样（留候选/可拒绝）。

**主管线产品判断（留给用户定，默认从丰富）**：日常面默认做成**常驻可见**（角标+收件箱），而不是又一个要主动去翻的隐藏面板——符合"日常闭环"目标。若用户要更轻（只在记忆中心加一个"待确认"标签页、不放全局角标），按用户口径收窄。

## 6. 后端 / 数据边界

优先复用：`memory_capture_bus`（capture）、`observation_store`（observation）、`memory_candidate_store`（候选）、`formal_memory_lifecycle`（采纳）、`page_read_model`（读模型）、既有 corrupt/revision/duplicate guard。

新增落点（尺寸现状：上述模块均低于 3000 行闸；`page_read_model.rs` 1440、`MemoryCenterView.tsx` 1340、`memoryCenter.ts` 1068、`types/memory.ts` 1391 均低于各自闸但偏大）：

- 如需编排"操作→capture→observation"统一入口，新增 **`src-tauri/src/memory_daily_loop.rs`**（薄编排层，调用既有 store，不复制逻辑）。
- 日常复核面读模型：在 `page_read_model.rs` 增"候选收件箱"查询，或新增小读模型模块。
- 前端 L5 新类型放**新文件**（如 `src/lib/types/memoryDailyLoop.ts`），不再堆进 1391 行的 `types/memory.ts`。
- 复核面 UI 优先**新组件**，避免把 `MemoryCenterView.tsx` 顶过 2000 行。

禁止：不改 schema；不把 `.codex` 当读源；不经测试/helper 隐式触发真实执行；不把候选自动标为正式。

## 7. UI 显示边界

遵守 `docs/workbench-frontend-display-boundary-v1.md`。主层显示候选摘要/来源/可做动作；详情层才放 refs/hash/scope 细节；不默认铺开 prompt body/transcript/secret/`.codex`/大段 JSON。

UI 禁止文案：`已自动记入正式记忆`、`记忆已自动正式化`、`无需确认`、`已执行真实操作`、`安全审查已绕过`。

## 8. 治理边界（硬规，全程保持）

- 不自动入正式记忆；候选→正式必须用户确认（M2 门）。
- 每条正式记忆带 source_refs；无来源只能留候选。
- 子智能体/LLM 不能直接写正式记忆。
- 普通聊天不自动捕获；敏感/secret 不进候选。
- 记忆 schema 正本 = R3 17 表，L5 不改。

## 9. 分线职责

- 全局主管线：审 L5 是否越界、最终接受；不把候选/确认升级成"已正式化/已执行"。
- 记忆线：capture/observation/候选/采纳预览的接线与边界；不自动正式化。
- UI/Tauri 线：日常复核面与可达入口；不新增真实执行按钮、不把开发者信息铺主层。
- 执行/runner 线：不参与（L5 不碰真实执行）。
- 复核线：只读复核 P0/P1/P2、schema 未改、治理硬规、UI 层级、测试与证据。

## 10. 建议实施切片

1. **L5-A**：capture 覆盖接线——补前端 `captureMemoryEvent` 调用（§4 操作点）+ 必要的薄后端编排；保 source_refs/敏感拦截；单测。
2. **L5-B**：日常候选复核面 + 可达入口（角标/收件箱）+ 单条/批量采纳（走 M2 门）。
3. **L5-C**：读模型（候选收件箱查询）+ runtime/audit/记忆边界接线 + 空态/错误态。
4. **L5-D**：证据、handoff、扫描、主管线接受复核。

任一切片若需要改 schema 或接真实执行，**停下，拆独立任务包并报主管线**。

## 11. 验证要求

改产品代码至少跑：`npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、相关 Rust 单测（memory_capture_bus / observation_store / memory_candidate_store / formal_memory_lifecycle / memory_daily_loop / page_read_model）、`cargo fmt -- --check`、`node scripts/harness/workbench-shape-gate.js --mode check`。

必扫并分类（按 `git status --short` 显式列文件）：`已自动记入正式记忆`、`记忆已自动正式化`、`无需确认`、`已执行真实操作`、`codex exec`、`codex exec resume`、`Command::new`、`/Users/yoyi/.codex`。命中只能来自历史/fixture/guard/否定声明，不能来自 L5 新增真实执行或自动正式化路径。

## 12. 接受标准

L5 可接受为：

- 日常关键操作能喂进 capture/observation（前端空白已补，列明覆盖面）。
- 候选在日常流里可见、可达、可经用户确认门采纳（单条/批量）。
- 全程保留 source_refs/scope/sensitive_level；敏感不入候选。
- 未改 schema、未自动正式化、未新增真实执行、未碰 `.codex`。

L5 不接受为：

- 记忆自动正式化 / 候选自动采纳。
- schema 变更。
- 真实执行 / runner / resume 门放宽。
- 普通聊天自动捕获。
- 完整记忆系统完成 / Stage L 完成。

## 13. evidence / handoff 要求

完成后新增：`evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md`、`handoffs/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1-result.md`、独立复核文件 `evidence/2026-06-16-stage-l-l5-...-review-<line>-v1.md`。

evidence 必须含：改动范围、是否改 schema、是否新增真实执行/自动正式化、capture 覆盖了哪几条操作（接了/留了）、日常面可达性、source_refs/敏感边界确认、测试与扫描、P0/P1/P2 复核结论。

handoff 必须含：当前状态、capture 覆盖面、日常确认面形态、是否仍守用户确认门、下一步建议。

## 14. L5 后续

- L5 完成后，剩余 capture 覆盖面（未接的 source_type）、知识库/evidence 联动等留后续或 L6。
- 真实浏览器/Tauri 可视化验收随 L1/L3 残余一并结转 L4。
- L5 不改变 L2（K3-B2）/ K3-B1 的 blocked/deferred 状态。
