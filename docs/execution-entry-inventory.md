# SYN-FND-006: 执行入口清单

> 资料状态（2026-08-09）：特定实现阶段生成的历史入口清单，不是当前授权清单。函数、行号和迁移状态会随代码变化；使用前必须从当前源码重新生成或逐项核对。

> 所有 Tauri/MCP/runner 实际执行或 spawn 路径逐项标记。

## 底层 Spawn 引擎（4 个）

| # | 引擎 | 文件 | 状态 | 说明 |
|---|------|------|------|------|
| S1 | codex_local_runner Phase B | codex_local_runner.rs:1150 | migrated | path-lock + continuation authorization；identity_kernel 已通过 build_report_input 间接接入 |
| S2 | supervisor_resident_oneshot | supervisor_resident_oneshot_session.rs:1650 | migrated | frozen config + resident session identity；MCP turn binding |
| S3 | supervisor_session_launcher | supervisor_session_launcher.rs:309 | migrated | load_authorized_launch_context() 完整授权链 |
| S4 | manual_relay | manual_relay.rs:2368/2825 | migrated | env MANUAL_RELAY_REAL_CODEX_CONFIRM 验用户在场 |

## Tauri Command 入口

### A. 对话传输（2 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T1 | start_agent_conversation_transport | commands.rs:361 | migrated | canonical project_root + HostProfile::AgentWorkspaceWrite |
| T2 | start_supervisor_conversation_transport | commands.rs:374 | migrated | resolve_supervisor_conversation_context() + HostProfile::SupervisorReadOnly + turn binding |

### B. Manual Relay（3 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T3 | run_manual_codex_relay_once | commands.rs:68 | migrated | env MANUAL_RELAY_REAL_CODEX_CONFIRM |
| T4 | run_manual_codex_relay_gui_direct | commands.rs:75 | migrated | env MANUAL_RELAY_REAL_CODEX_CONFIRM |
| T5 | run_manual_codex_relay_gui_direct_new_session | commands.rs:82 | migrated | env MANUAL_RELAY_REAL_CODEX_CONFIRM |

### C. Real Execution Product Command（3 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T6 | run_real_execution_product_command_phase_a | commands.rs:4541 | blocked | Phase A 不 spawn；path-lock |
| T7 | run_real_execution_product_command_phase_b | commands.rs:4554 | migrated | path-lock + continuation authorization |
| T8 | run_real_execution_product_command_new_session_phase_b | commands.rs:4567 | migrated | path-lock + continuation authorization |

### D. Project Workflow Automation（3 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T9 | run_project_workflow_automation_phase_a | commands.rs:4583 | blocked | Phase A 不 spawn |
| T10 | run_project_workflow_automation_j2_b_b1 | commands.rs:4599 | blocked | 写死 mario test，永远拦 |
| T11 | run_project_workflow_automation_j2_b_b2 | commands.rs:4611 | migrated | path-lock |
| T12 | run_project_workflow_automation_k3_b | commands.rs:4628 | migrated | path-lock |

### E. Session Continuation（2 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T13 | run_controlled_session_continuation_real_resume_phase_a | commands.rs:4878 | blocked | Phase A 不 spawn |
| T14 | run_controlled_session_continuation_real_resume_phase_b | commands.rs:4894 | migrated | path-lock + authorization |

### F. Workflow Dispatch（3 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T15 | execute_workflow_node_dispatch | workflow_execution_entrypoints.rs | migrated | inspect_workflow_node_dispatch_authorization() |
| T16 | execute_experiment_node_dispatch | workflow_execution_entrypoints.rs | migrated | 同上 |
| T17 | execute_project_workflow_node | workflow_execution_entrypoints.rs | migrated | 同上 |

### G. Supervisor（2 个执行入口）

| # | 函数 | 文件:行号 | 状态 | 校验方式 |
|---|------|-----------|------|----------|
| T18 | launch_supervisor_pilot | supervisor_session_launcher.rs:384 | migrated | load_authorized_launch_context() |
| T19 | submit_supervisor_resident_answer | supervisor_resident_oneshot_session.rs:3213 | migrated | resident_conversation_lock() + frozen config |

### H. 辅助 Spawn（3 个）

| # | 函数 | 文件:行号 | 状态 | 说明 |
|---|------|-----------|------|------|
| T20 | obsidian_integration_read_note | obsidian_integration.rs:552 | migrated | 固定 bundled CLI + 超时 |
| T21 | obsidian_integration_search_notes | obsidian_integration.rs:565 | migrated | 同上 |
| T22 | obsidian_integration_open_in_obsidian | obsidian_integration.rs:579 | migrated | macOS open |

## MCP 工具（8 个，全部纯状态操作）

| # | 工具 | 状态 | 说明 |
|---|------|------|------|
| M1 | list_team | migrated | 纯只读 |
| M2 | dispatch | migrated | 写 inbox 到 run state |
| M3 | read_outbox | migrated | 纯只读 |
| M4 | recycle | migrated | 更新 run state |
| M5 | stop | migrated | 更新 run state |
| M6 | finish | migrated | 更新 run status |
| M7 | submit_outbox | migrated | 写 outbox file |
| M8 | report_blocked | migrated | 写 outbox file |

## 统计

按上表明细逐行加算（S1–S4 + T1–T22 + M1–M8，共 34 行）：

- 总条目： 34（底层 spawn 引擎 4 + Tauri 命令入口 22 + MCP 工具 8）
- migrated: 30（引擎 4 + Tauri 18 + MCP 8）
- blocked: 4（T6/T9/T13 为 Phase A 不 spawn；T10 j2_b_b1 写死 mario test 永拦）
- guarded-legacy: 0
- not-in-scope: 0

> 注：Tauri 22 个入口中 migrated 18、blocked 4；T6/T9/T13 虽标 blocked，属"该阶段设计上不 spawn"而非失控入口。caller-controlled execution 入口无一处于未标记状态。
