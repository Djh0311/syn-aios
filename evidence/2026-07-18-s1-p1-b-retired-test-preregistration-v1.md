# S1 P1-B 退役断言预登记 v1

日期：2026-07-18  
权威任务包：`tasks/2026-07-18-s1-freeform-supervisor-and-proposal-tool-package-v1.md`  
上位决定：`decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md`

## 目的与登记边界

本清单与测试注册替换属于同一未提交变更集：在交付验收前先明确哪些 P1-B 断言因产品方向被退役，哪些 P1-A 会话宿主/换代断言必须迁到新的自由消息夹具。旧测试文件保留在工作树中作为历史证据，但不再被 Rust 测试模块 `include!`；它不是静默删除。

P1-B 被退役的仅是“每一轮必须严格 JSON 二选一（方案或问题）”及 `protocol_invalid` 保守停，不是历史审计记录。S1 的结构化校验改由 resident 私有 `submit_proposal` MCP 工具承担。

## 明确退役的 P1-B 断言

| 历史测试 | 退役理由 | S1 替代责任 |
| --- | --- | --- |
| `p1_b_resident_turn_schema_is_strict_binary_and_stops_on_invalid_shapes` | 自由文本不再要求 `supervisor_resident_turn.v1`，不再产生该协议的保守停。 | `s1_freeform_messages_reuse_the_thread_without_a_protocol_gate`；MCP 严格参数测试。 |
| `p1_b_mock_question_answer_same_thread_proposal_then_duplicate_is_rejected` | `question_id`、待答状态和“答完自动解析 proposal”退役。 | `s1_freeform_messages_reuse_the_thread_without_a_protocol_gate`；`s1_resident_submit_proposal_*`。 |
| `p1_b_mock_recovers_exact_durable_answer_after_pre_injection_failure` | 用户文本不再是某题的 answer，也没有 answer 注入状态机。 | `s1_freeform_user_message_survives_host_rebuild_as_canonical_fact`。 |
| `p1_b_mock_supports_second_question_before_final_proposal` | 多轮问答是普通聊天，不再以 question schema 计数或收束。 | `s1_freeform_messages_reuse_the_thread_without_a_protocol_gate`。 |
| `p1_b_mock_dead_host_rebuilds_with_durable_answer_and_injects_it` | “durable answer”专名退役；保留的是可重建的 canonical 用户消息。 | `s1_freeform_user_message_survives_host_rebuild_as_canonical_fact`。 |
| `p1_b_live_fixed_project_question_answer_then_proposal_same_thread` | 真跑不再验证 question/answer JSON 协议；改为 A5 的自由三轮→工具落卡脚本。 | A5 ignored/live 验收（尚未执行，见最终回传）。 |

## 必须保留并已迁移的 P1-A 断言

| 原 P1-A 责任 | S1 注册测试 |
| --- | --- |
| 同 thread 续接、私有 MCP shape | `s1_freeform_resident_keeps_private_mcp_shape_for_three_same_thread_messages` |
| 宿主失活后的 rebuild 与 canonical 核心事实 | `s1_freeform_user_message_survives_host_rebuild_as_canonical_fact` |
| thread-invalid 当回合换代 | `s1_freeform_rebuilds_in_same_turn_after_thread_invalid` |
| 失败宿主退役、后续干净 generation | `s1_freeform_parse_failure_retires_host_before_a_later_clean_generation` |
| 项目槽互斥隔离 | `s1_freeform_project_slots_have_independent_locks` |
| 固定项目真机 kill/rebuild 预登记 | `s1_freeform_live_fixed_project_reuses_thread_then_rebuilds_after_host_kill`（`#[ignore]`） |

## 注册与审计口径

- 注册入口从 `supervisor_resident_session_tests.rs` 切到 `supervisor_resident_freeform_tests.rs`；历史文件保留、不编译。
- 新 canonical 事件仅为 `supervisor_resident_user_message_recorded`、`supervisor_resident_user_message_injected`、`supervisor_resident_supervisor_message_recorded`；读模型新增对应 `user_message`、`user_message_injected`、`supervisor_message` 词表映射，并已在本包回传中明列。
- 不创建 sidecar；方案仍沿既有 proposal store/audit 事件族落库，状态为 `PendingUserConfirmation`，不自动批准、不起链。

## 勘察补遗的发送路边界

本包只泛化既有 `submit_supervisor_resident_answer`（路②）并复用 resident 的同 thread `codex-reply` 器官（路④）。`manual_relay` 仍是智能体页的任意会话、preview+confirm 手动指挥路径；本包是固定项目主管 thread 的 canonical 用户消息入口。两者不共享新 sender，审计语义也不改。`readonly_codex_consult` 与智能体页两张对话脸均不在本包修改面。

## 验收前提

本文件只登记断言的替换关系，不宣称真跑或真机已通过。离线定向测试、四闸/M5 与 A5 live/ignored 证据须在最终回传分别列出；缺少真实 executable 或用户授权的模型调用时必须保持未验收状态。
