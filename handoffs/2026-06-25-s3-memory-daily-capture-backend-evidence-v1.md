# 回交：S3 每日记忆采集·后端接源（daily-capture 帮手 + 4 映射器 + best-effort 挂钩）· 执行线 → 主导线 v1

日期：2026-06-25　性质：**轻档**（后端：结构化治理事件→候选；不真起 codex/不执行/不改闸/不自动写正式）　任务包：`tasks/2026-06-25-s3-memory-daily-capture-backend-wiring-v1.md`

## 0. 一句话结论

在已活的捕获机器（capture_bus/observation/candidate store）上接线通电：恢复 `capture_daily_memory_event` 帮手 + 4 个「治理事件→候选输入」映射器 + best-effort 挂到 3 个治理命令 wrapper。**只写候选不写正式、不漏敏感、best-effort 不拖垮治理命令、既有测试全绿**。**执行线未 commit。**

## 1. 建了什么

**恢复 + 扩 `memory_daily_loop.rs`**（= S0 `4414bac` 删的文件 git 捞回 + 扩 3 映射器；`mod memory_daily_loop;`）：
- `capture_daily_memory_event` 帮手（原样恢复）：调 `memory_capture_bus::capture_event` + 补 `memory_daily_loop_capture_requires_user_confirmation` warning。
- **4 个映射器**（治理事件 → `CaptureMemoryEventInput`，照模板 `operation_control_decision_capture_input`）：
  1. `worker_report_capture_input` ← `WorkerStructuredReportInput`（source_type=`worker_report`，role=worker）
  2. `plan_authorization_capture_input` ← `RecordPlanAuthorizationUserConfirmationInput`（source_type=`user_action`，role=user）
  3. `final_review_capture_input` ← `GlobalFinalResultReviewInput`（source_type=`final_review`，role=global_director）
  4. `operation_control_decision_capture_input`（模板原样恢复，source_type=`operation_control_decision`）
  - 每个：`candidate_policy="candidate_allowed"` + 候选 `requires_user_confirmation=true` + scope/source_refs/sensitive_level 填齐。
- `capture_governance_event_best_effort`：调帮手、**失败吞成 warning 不 Err**。

**挂钩 `commands.rs` 3 个 #[tauri::command] wrapper**（+ 1 个 helper `l5_capture_governance_best_effort`）：`record_worker_structured_report` / `record_plan_authorization_user_confirmation` / `record_global_final_result_review` —— `_at` 成功后 best-effort 采集，主返回不变。

## 2. 安全死线（§3·全守住）

- **只写候选不写正式** ✅：`candidate_allowed`；测试 `…without_formal_memory` 断言 **formal_memory sidecar 不存在**。正式仍走 M2 用户采纳。
- **不漏敏感** ✅：映射器 summary/claim/body **只用结构化字段拼**（角色/id/decision/计数），**不回显** worker/review 的自由文本 summary（可能夹带敏感）；`sensitive_level=internal`；bus 自身亦拦 `prompt body`/`.codex`/`full transcript` 等字串。脱敏断言钉死（每映射器测「不含 prompt body / 全文 transcript / .codex 路径」）。
- **best-effort 非阻塞** ✅：挂在 wrapper、`_at` 成功后调；capture 失败只 warning，**绝不改治理命令主返回**。测试 `l5_best_effort_swallows_capture_failure`（注入会被 bus 拒的输入 → 只回 warning、不 panic/Err）。
- **回归安全** ✅：**挂在 wrapper、不挂 `_at`** → 既有 `_at` 测试 0-diff、全绿（关键：`worker_structured_report_records_audit_without_observation_or_formal_memory` 断言「命令本身不自动建 observation/candidate」**仍成立**——采集是 wrapper 层旁路、不破这条不变量）。`cargo test --lib` **597 passed**（升不降）。
- **不碰** ✅：codex/runner/执行闸/`decide_real_execution_command`/`command_plan_for`/沙箱/A 线/前端 inbox 全 **0-diff**。

## 3. 验收门

| 门 | 结果 |
|---|---|
| 每映射器（脱敏 + candidate_allowed + source_type/role） | ✅ `l5_worker_report…` / `l5_plan_authorization…` / `l5_final_review…` / `l5_operation_control…` |
| 采集落盘（observation+candidate·无 formal） | ✅ `l5_daily_capture_creates_observation_and_candidate_without_formal_memory` + `l5_worker_report_capture_lands_candidate_without_formal_memory`（候选真攒进 `memory_candidate_store`）|
| best-effort | ✅ `l5_best_effort_swallows_capture_failure` |
| regression | ✅ `cargo test --lib` = **597 passed / 0 failed / 28 ignored**（dc2bcb4 基线 + S3 consultant 6 + 本包 7）；既有治理/记忆测试全绿 |
| 0-diff | ✅ real_execution_command/session_continuation_store/workflow_chain_controller 0 行；commands.rs 仅 3 wrapper 体 + 1 helper（无删除行碰 gate/decide）|
| fmt / git diff --check | ✅ 我的文件 fmt 干净；git diff --check 干净 |
| 范围 | `memory_daily_loop.rs`(新) + `commands.rs`(helper+3 wrapper 挂钩) + `lib.rs`(mod) |

## 4. 给主导线/咨询的判断点

1. **source_type 实况是 8 种、非任务包说的「14 种」**：bus 验证器（`memory_capture_bus.rs:367`）只认 `user_action / product_command / runtime_log / readback / worker_report / operation_control_decision / process_fact_decision / final_review`。本包 4 源都在内（方案采纳用 `user_action`——没有 `plan_authorization` 这个 type，用户确认即用户动作）。canon/任务包「14 种」与代码不符，请咨询核对正本。
2. **挂法选了 wrapper-best-effort（非 inline `_at`、非 daily-batch 扫描）**：因既有测试明确断言「治理命令本身不自动建 observation/candidate」——inline 进 `_at` 会破这条不变量 + 回归。wrapper 挂钩保住不变量、0 回归。代价：wrapper 挂钩不被单测覆盖（采集逻辑由 memory_daily_loop 的 7 个单测覆盖；真机看见候选需 §5 deferred 的前端 inbox）。
3. **process_fact_decision 源未接**（任务包只点名 4 源；bus 有这个 type）——同模式后续可加。

## 5. 收口 + 本包没做（deferred）

- 执行线不 commit；主导线核实物（重跑 597 + 扫 diff：只候选不正式、不漏敏感、没碰 codex/闸/前端）→ commit + CURRENT 回写。
- deferred：前端 inbox 复位（记忆中心布局重做）/ 4 源之外更多源 / 候选 LM 蒸馏 / 自动写正式记忆（M2 不动）。本包只到「后端从真活动攒候选进 store」。
