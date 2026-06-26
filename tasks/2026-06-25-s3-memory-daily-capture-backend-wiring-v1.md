# 实现任务包：S3 每日记忆采集·后端接源（恢复 daily-capture 帮手 + 治理事件→候选映射器 + 挂钩）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**轻档**（后端命令逻辑：结构化事件→候选；不真起 codex、不执行、不改安全闸、不自动写正式记忆）。
上游：S3「每日自动记忆采集」完成标准（S0 commit `4414bac` 指定「原 memory_daily_loop 点子接线通电」）；记忆层正本 `docs/memory-layer-consolidated-canon-v1.md`；模板 = 删掉的 `memory_daily_loop.rs`（git 可捞，见 §2）+ 现存 live 源 `project_workflow_automation.rs:2763/2838`。

## 0. 接手须知

- 你是**执行线**。流水线：实现 + stub/offline 测试 → 主导线核实物。子线不 `git add`/`commit`。
- 先读：① `git show <4414bac^>:…/memory_daily_loop.rs`（捞出删掉的模板：`operation_control_decision_capture_input` 映射器 + `capture_daily_memory_event` 帮手 + 2 个测试，**照它做**）② 现存 live 源 `project_workflow_automation.rs:2763`（建 `CaptureMemoryEventInput`）+ `:2838`（`memory_capture_bus::capture_event`）③ `memory_capture_bus.rs` 的 `capture_event` 契约 ④ 正本 §3.6 脱敏 / **source_type（注：代码 capture bus 验证器实际只接受 8 种；canon 文档列 11-14 是漂移，见 `decisions/2026-06-25-memory-source-type-canon-vs-code-drift-v1.md`——以代码 8 种为准）** / candidate_policy。
- **全程中文。子线不 commit。** 一句话：**恢复 daily-capture 帮手 + 给选定治理事件各写一个「事件→候选输入」映射器 + 接到真活动让候选真攒进 `candidate_store`；只写候选不写正式、不漏敏感、不碰 codex/执行/闸。**

## 1. 拍板摘要

- **要做的事**：在**已活的捕获机器**（capture_bus / observation / candidate store 都在）上「接线通电」——恢复被删的 `capture_daily_memory_event` 帮手，给 4 个治理事件各写映射器并挂钩，让真治理活动**自动攒出待确认候选**。
- **代价**：一轮后端。做完后记忆**第一次从真活动里攒东西**（喂咨询 `read_memory` 有料 + 为 inbox 复位备数据）。
- **不做的后果**：候选机器空转、记忆永远是空的、咨询读不到项目积累。
- **关键澄清**：**只写候选**（`candidate_allowed` + `requires_user_confirmation`），**不自动写正式记忆**（正式仍走 M2 用户采纳）；**不接前端 inbox**（归记忆中心布局重做）；**不蒸馏/不接 LM**（纯结构化映射）。

## 一句话判据

判某改动在不在本包内——问：**「是不是在恢复 daily-capture 帮手 + 写治理事件→候选映射器 + 挂到真活动，且只写候选不写正式、不漏敏感、不碰 codex/执行/闸/前端?」** 是 → 做；否（尤其要自动写正式记忆 / 漏敏感入候选 / 碰 codex 执行 / 改闸 / 动前端 inbox）→ **停、回主导线。**

## 2. 建什么（照模板）

**A · 恢复帮手**：把删掉的 `capture_daily_memory_event`（→ `memory_capture_bus::capture_event` + 补 `requires_user_confirmation` warning）恢复进一个聚焦模块（如 `memory_daily_loop.rs` 重建 或 `memory_capture_sources.rs`）。

**B · 4 个映射器**（每个 = 治理事件 → `CaptureMemoryEventInput`，照 `operation_control_decision_capture_input` 模板：`candidate_policy="candidate_allowed"`、候选草稿 `requires_user_confirmation=true`、设 `scope`/`source_refs`/`source_type`/`sensitive_level`、summary/claim/body **不含 prompt body / `.codex` / 全文 transcript**）：
1. **worker report** ← `record_worker_structured_report`（source_type 用代码接受的 8 种里对的，即 `worker_report`）
2. **方案采纳** ← `record_plan_authorization_user_confirmation`
3. **全局复核** ← `record_global_final_result_review`
4. **L3 operation-control** ← 直接恢复模板里的 `operation_control_decision_capture_input`

**C · 挂钩**：在各治理事件**被记录之后**调「映射器 + `capture_daily_memory_event`」，让候选攒进 `candidate_store`。**挂法你选**（命令内挂 / 或一个扫近期治理事件的 daily-batch 入口），但必须满足 §3 的**回归安全 + best-effort**。

## 3. 安全死线（本包死线·必须成立）

- **只写候选不写正式**：`candidate_allowed`，**断言不产出 formal_memory sidecar**（照模板测试 `…without_formal_memory`）。正式仍只走 M2 用户采纳。
- **不漏敏感**：映射器的 summary/claim/body/source_refs **不含 prompt body / `.codex` 路径 / 凭据 / 全文 transcript**；`sensitive_level` 设对（照模板的脱敏断言钉死）。
- **best-effort·非阻塞**：capture 失败只记 warning，**绝不让治理命令本体失败/改变其主返回**（采集是旁路，不能拖垮主流程）。
- **回归安全**：挂钩**不得**碰挂的现有测试——既有治理命令测试全绿、`cargo test --lib` 计数**不降**（若 capture 副作用动了某测试断言的 store 状态，要妥善处理、不许靠删测试糊弄）。
- **不碰**：codex / runner / 执行闸 / `command_plan_for` / 沙箱 / 前端 inbox **0-diff / 不动**。
- **碰线就停**：要自动写正式 / 漏敏感 / 碰 codex 执行 / 改闸 / 动前端 → 停、回主导线。

## 4. TDD 验收门（测试钉死·照模板）

- **每个映射器**：事件 → 输出 `candidate_allowed` + 对的 source_type/scope + 候选草稿 `requires_user_confirmation` + **脱敏断言**（summary 不含 prompt body/`.codex`）。（镜像 `l5_operation_control_capture_input_…`）
- **采集落盘**：帮手 → 产出 observation + candidate、**无 formal sidecar**。（镜像 `l5_daily_capture_…without_formal_memory`）
- **接线集成**：触发治理事件（如 record_worker_structured_report）后，候选真出现在 `candidate_store`（stub/offline）。
- **best-effort**：capture 失败时治理命令仍正常返回（注入失败用例验）。
- **regression**：既有记忆/治理测试全绿、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`（本包不碰前端，不需 typecheck/offline/build；若你选的挂法碰了前端 = 出范围、停）。

## 5. 本包不做（deferred）

- **前端 inbox 复位**（`DailyMemoryCandidateInbox` 现 orphan 在不渲染的 `RunningWorkflowsView`）→ 归**记忆中心界面布局重做**。
- 选定 4 条之外的源（同模式后续加）。
- 候选的 **LM 蒸馏/摘要**（结构化映射够用；LM 后续可选）。
- 自动写正式记忆（M2 用户采纳不动）。
- 任何 codex/执行/闸改动。

## 6. 回交

- 跑 §4 各门；回交：实现 diff（确认 codex/执行/闸/前端 0-diff）+ 每映射器脱敏断言证据 + 「采集只候选不正式」证据 + 接线集成证据（治理事件→候选进 store）+ best-effort 证据 + `cargo test --lib` 计数 → 主导线核实物（重跑计数 + 扫 diff 确认只候选不正式、不漏敏感、没碰 codex/闸/前端）。子线不 commit。

## 7. 不接受为

- 不接受为：自动写了正式记忆 / 候选里漏了敏感（prompt body/凭据/全文）/ 碰了 codex 执行/闸/沙箱 / 动了前端 / capture 失败拖垮了治理命令 / 靠删既有测试糊弄回归 / 计数下降。
- 不接受为 S3「每日自动记忆采集」整体完成（本包只到「后端从真活动攒候选进 store」；inbox 复位 + 更多源 + 真机看见 = 另外的事）。
