# 决策：记忆 source_type 代码 8 种 vs canon 文档 11-14——二层不同词表·暂以代码 8 种为准 v1

日期：2026-06-25

## 背景（核实物发现）

S3 每日记忆采集·后端接源核实物时，4 路只读 workflow + 主导线复核查清：

- **代码实况**：`memory_capture_bus.rs` 的 source_type 验证器**只接受 8 种**——`operation_control_decision` / `worker_report` / `process_fact_decision` / `final_review` / `user_action` / `product_command` / `runtime_log` / `readback`。
- **canon 文档**：`docs/memory-layer-design-v1.md` §5.3（MemorySourceRef）列 **11-14 种**（`user_confirmed_proposal` / `workflow_summary` / `stage_report` / `director_review` / `handoff` / `evidence` …），与代码词表**多不重叠**。
- 二者像**两层不同词表**：**capture-event 级**（8 种·事件分类，capture bus 用）vs **source_ref/provenance 级**（11-14 种·正式记忆来源，design 文档用）。canon 没把这个分层清楚文档化。`user_confirmed_proposal` 在代码别处出现，但**不在 capture bus 验证器里**。

## 决策

- **暂以代码 capture bus 的 8 种为准**（它是真在跑的验证器）。新接的治理采集源**用这 8 种里语义最贴的**。
- **方案采纳用 `user_action`**（→ `plan_adopted` 观察类型）：canon 想要的 `user_confirmed_proposal` 不在 capture bus 验证器、用不了；`user_action` 语义站得住（批方案 = 用户动作 = 方案被采纳）。
- **canon-vs-code 对齐 = deferred**：以后真做记忆层精确化时，一并理清「capture 8 种 vs source_ref 11-14 种」分层 + 是否补 `user_confirmed_proposal` 等精确 type。**别现在为这个改 capture bus 验证器**（出 S3 接源轻档范围；改验证器=动既有记忆基建，另议）。

## 影响

- 当前采集候选的 source_type 偏泛（如 `user_action` 而非 `user_confirmed_proposal`），但**合法、可用、候选 body 仍传达语义**，不阻断咨询 `read_memory`。
- 任务包 `tasks/2026-06-25-s3-memory-daily-capture-backend-wiring-v1.md` 原写「14 source_type」是错的（照搬 canon 文档）——**已改为「8 种代码实况」**。
- 呼应记忆 `codex-workbench-canon-drift-r3-memory-and-git-baseline`：canon 文档与代码实况会漂，状态以磁盘/代码为准。
