# 主管线代班清单：Claude 代班期工作汇总 v1

日期：2026-06-12

性质：依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md` §6 的代班清单 handoff。主管线代班会话（Claude，claude-fable-5）在 T 系列可迁切片到底后写本清单收尾停下。本文不是回交完成：主管线脑切回 Codex 的 CURRENT.md checkpoint 由 Codex 实际回归时另行写入。

复核口径：代班期间每包均经独立复核线会话（claude-opus-4-8，跨模型）只读复核 `STATUS: CLEAR` 后由用户放行收口；三份复核结论文件已入库，是 Codex 回归后"换脑抽查"事后复检（takeover handoff §5）的输入。

## 1. 代班期间任务包清单

| 包 | 性质 | `lib.rs` 变化 | 复核 | 关键 commit |
| --- | --- | --- | --- | --- |
| R2-T12 task package preview / node session binding / read model 迁移（11 tests） | 迁移 | 6,544 → 6,006（−538） | CLEAR | impl `a3fce1f7`，清除 `bcb8864b`，sync `cf47fb85` |
| R2-T13 deferred 带逐测试复评（11 个裁决：可迁 9 / 禁迁确认 1 / deferred 维持 1） | 评估，零代码改动 | 6,006 → 6,006（0） | CLEAR | 任务包 `82d30751`，清除 `0f99690c`，sync `0895f56a` |
| R2-T14 workflow governance boundary + director review rejection 迁移（9 tests） | 迁移 | 6,006 → 5,567（−439） | CLEAR | impl `a61c8f97`，清除 `50fcbcf6`，sync 见本包 checkpoint |

合计：`lib.rs` 6,544 → 5,567，**净降 977 行**；迁出 20 个 inline tests 到 3 个 include 文件（`lib_task_package_preview_binding_read_model_tests.rs` 539 行、`lib_workflow_governance_boundary_tests.rs` 339 行、`lib_director_review_rejection_tests.rs` 102 行）；shape gate waterline 逐包锁至 5,567。另有人事事件留痕 commit `6fba75ef`（CLAUDE.md + 主管线/复核线两份接管职位档案）。

## 2. T 系列收口状态

T 系列可迁切片已到底：`lib.rs` 剩余 **35 个 inline tests = 禁迁 34 + deferred 1**。

- 禁迁 34（既定清单，分布）：K3-B runtime prompt guard 2、real-state 2（含 `#[ignore]` 1）、cross-store memory adoption / formal memory store 相邻组 13、workflow node dispatch prepare/execute/readback + legacy guard 12、workflow machine 1、director review 回收路径（StubCodexResumeRunner）1、offline role 端到端组 3。
- **deferred 切片 1 个**：`compact_last_message_summary_preserves_workflow_machine_control_marker`——纯函数、零 runner，但断言 workflow machine 控制标记语义，代班收紧口径（接管档案 §3 "workflow machine" 无限定词）下拿不准不赌。复评触发点：Codex 回归后按 R2-T0 原口径重判，或冻结清单修订。
- 每包 `#[test]` 守恒均经复核线核对：T12 前 55 → T12 后 44 → T14 后 35，无测试丢失（`cargo test --lib` 471 passed / 16 ignored 全程不变）。

## 3. 代班期间过程事件（如实留痕）

1. T12 实现期间用户修订接管档案 §5 并新增复核线职位档案（复核从"咨询线兼任"改为独立职位、配脑 opus-4-8、触发权在用户）。主管线在修订前自派的同模型审查已降级为自查留痕；主管线曾因旧版档案上下文短暂误判新职位档案不存在，经重读文件系统纠正。预防措施（已写入 T12 文档与 CLAUDE.md 记忆治理条目）：收口各步骤前重读权威档案最新状态，不信开机快照。
2. 一次 `git commit` 被 harness 权限层按 AGENTS.md commit 确认规则拦截，流程调整为：回合内完成全部离线工作，commit 序列经用户逐包放行后批量执行。
3. T12 自查曾抓出任务包 §6 计数笔误（禁迁 32 应为 33），已修正；T13 复核确认修正后三处一致。

## 4. 移交时现场状态

- `lib.rs` 5,567 行，shape gate waterline 5,567（historical_lowest_closed_value 棘轮），0 errors / 0 warnings。
- `cargo test --lib` 471 passed / 0 failed / 16 ignored；`cargo fmt -- --check`、`git diff --check` 干净。
- 既有冻结全部未动：未真实执行 codex、未读写 `~/.codex`、未改 UI/产品行为/schema、R3 Level B 未执行、backlog 未解冻、Stage L 仍 `deferred_during_root_treatment`。

## 5. 待 Codex 回归后的决策与复检输入

- 车道选择（用户指示，2026-06-12）：R2 后段按"明确下降轨道 + 冻结 deferred"收口、还是转 R4 硬目标，等用户和 Codex 回归后定；代班主管线不预定方向。
- 换脑抽查复检输入：T12/T13/T14 三份任务包 + 三份 evidence + 三份 handoff result + 三份复核结论文件（`evidence/*-review-claude-v1.md`）+ 本清单。
- deferred 1 个的复评（见第 2 节）。
- R3 Level B 窗口计划、checkpoint 轮转方案（P2-2）、R4 硬目标等既有后续治理项不变。
