# S1B 常驻宿主测试退役预登记 v1

日期：2026-07-19  
权威任务包：`tasks/2026-07-18-s1b-supervisor-transport-oneshot-resume-package-v1.md`

## 目的与注册链

旧活注册链是 `supervisor_session_launcher.rs` → `supervisor_resident_session.rs` → `supervisor_resident_freeform_tests.rs`。S1B 已按本文件完成替换：新活链为 `supervisor_session_launcher.rs` → `supervisor_resident_oneshot_session.rs` → `supervisor_resident_oneshot_tests.rs`；旧 persistent-host 实现与测试已移除。`supervisor_resident_session_tests.rs` 从未被 `include!`，仅作为历史未注册文件清理，不计入活测试减少。

## 退役符号

- `SUPERVISOR_RESIDENT_TURN_TIMEOUT=420`、`SupervisorResidentMcpHost*`、`SUPERVISOR_RESIDENT_HOSTS`、`CodexSupervisorResidentMcpHost` 与 `RealSupervisorResidentMcpHostSpawner`。
- 强制 `argv=["mcp-server"]` 的 command-plan validator、host slot map 与单 PID 常驻登记。

## 测试替换映射

| 旧责任 | S1B 替代测试 |
| --- | --- |
| 一个 mcp-server 承接三轮 `codex/codex-reply` | `s1b_three_oneshot_turns_resume_same_thread_and_keep_project_private_home`：首轮 `exec`、后两轮 `resume`、三次有限回合、同 thread/home。 |
| 首轮/续跑后宿主失活换代 | `s1b_invalid_resume_rotates_home_and_rebuilds_facts`：invalid resume 归档旧 home、重建事实、换新 thread。 |
| 420 秒整回合超时和 host 清退 | `s1b_watchdog_retries_once_and_second_silence_returns_human_message` 覆盖 120 秒 stdout 静默策略的 mock 结果、一次重试和人话停止；`s1b_watchdog_after_initial_thread_binding_retries_with_same_thread_resume` 断言首轮已 bind 时重试仍为同一 thread 的 `resume`；`s1b_process_group_cleanup_reaps_a_term_ignoring_descendant` 用本地 shell 夹具验证 TERM 后的 group KILL sweep。三者不冒充真实 Codex 120 秒或真实用户机器 `ps` 证据。 |
| 先回包后写 durable binding 的工具不可用面 | `s1b_first_thread_started_is_bound_before_real_submit_proposal_card_write`：同步 bind 后同回合真实本地 `submit_proposal` handler 落 `PendingUserConfirmation`，链保持不动；`s1b_first_turn_tool_waits_for_durable_binding_instead_of_racing_a_stale_thread` 覆盖工具先到的并发 race。 |
| 进程遗留状态 | `s1b_startup_marks_dead_running_turn_exited_without_destroying_binding`、`s1b_startup_marks_dead_prepared_turn_exited_without_destroying_binding` 与 `s1b_startup_reaps_dead_virgin_prepared_turn_without_thread_binding`；`s1b_cleanup_failure_keeps_pid_visible_until_process_group_reconciliation` 防止未证实 cleanup 被伪记为已退出。dead PID / group 由注入谓词模拟，证明 durable 状态更新，不是实际 `ps` 运行证据。 |
| 固定项目真实 3 轮 | `s1b_live_resume_tool_card_and_replacement_require_explicit_harness_authorization`（`#[ignore]`）：首个真实 `exec` 直接要求工具落卡，后两轮 `resume` 同 thread，再测换代；不在本次离线验收中冒充通过。 |

## 不触碰范围

`manual_relay`、`readonly_codex_consult`、两张智能体对话脸与 proposal 的 strict parser/store/human gate 不以本退役操作重写。
