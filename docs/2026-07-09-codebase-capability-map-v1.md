# 代码库能力地图 v1（2026-07-09·便宜模型普查→主导线核过收编）

> **⛔ 已被 v2 取代（2026-07-10）**:正本 = `docs/2026-07-09-codebase-capability-map-v2.md`(真角色·补全命令 pub 面·概念反查索引·主导线核过)。本 v1 骨架版留档。

> **性质与局限（主导线 07-09 核后如实标）**:本图是**骨架级索引**,不是精修正本。用法=**"某能力大概在哪个区/哪个文件"的快速定位**,写"加新能力"包前先来这查有没有现成的(防重造轮子·配 `dup-check` 包)。**已知局限**:① 多数"角色"是模板套话(如"后端能力封装"),别当权威描述;② 便宜模型 grep 漏了 `#[tauri::command]`/非 pub `fn`,故 `commands.rs` 等关键大文件 pub 面原为空——主导线已手补 5 个最重要的(见下标 ★);③ "疑似两套"节非全量 dup 审计,只记普查时撞见的(真 dup 详见 `dup-check` 包核定:humanize_consult_error ×2)。**增量精修**:碰到哪个文件就顺手把它那行从套话改成实话。

## 旧图核对（5 份·还准/过期/取代）
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`：还准（定位方向仍在；行号因文件拆迁已漂移）。
- `docs/plans/2026-06-24-s2-1-seam-wiring-map-v1.md`：部分过期（画布 HUD 嵌套关系有大量修复痕迹），但“功能在画布细线内被埋”结论仍准。
- `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md`：部分过期（契约事实核对仍可复用；事件/前端承接有追加实现）。
- `docs/harness-catalog.md`：还准（已记录退役与接线状态，能力索引仍可参考）。
- `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`：还准（产品现状快照，仍是近期“真有功能”基线）。

覆盖与增量原则：本图以现状全量 `prototypes/productized-desktop-shell/src-tauri/src`（99 rs）与 `prototypes/productized-desktop-shell/src`（101 ts/tsx）为底盘；老图仅做入口线索，不重画。

## 后端能力（按 8 区域）
### ① 会话/relay(会话读写、codex 运行与通道)
- `codex_db.rs` — codex/会话/执行链相关核心能力 — `pub struct CodexThreadRow; pub struct CodexThreadPage; pub struct CodexThreadPageOptions; pub fn default_state_db_path; pub fn read_threads`
- `codex_local_runner.rs` — codex/会话/执行链相关核心能力 — `pub(crate) trait CodexLocalRunner; pub(crate) trait CodexLocalPhaseAProcessRunner; pub(crate) trait CodexLocalPhaseBProcessRunner; pub(crate) struct CodexLocalPhaseAProcessResult; pub(crate) struct CodexLocalPhaseBProcessResult`
- `codex_transcript.rs` — codex/会话/执行链相关核心能力 — `pub(crate) struct TranscriptThreadMetadata; pub(crate) struct TranscriptReadPageRequest; pub(crate) fn read_transcript_from_rollout; pub(crate) fn read_transcript_page_from_rollout; pub(crate) fn transcript_viewer_boundary`
- `command_registry.rs` — 对外命令注册与前端入口绑定 — `(无显式pub/导出)`
- `commands.rs` — ★主导线补·**Tauri 命令层**(前端所有 invoke 的后端入口·~几十命令) — `load_workbench_snapshot; query_workbench_page_read_model; record_operation_control_decision; preview/confirm/run_manual_codex_relay_once(relay 单次三步); record_worker_structured_report; load_codex_session_page(会话列表·并显工作台会话)`
- `diagnostics_provider_session_entrypoints.rs` — 后端能力封装 — `(无显式pub/导出)`
- `index_host_app_entrypoints.rs` — 对外命令注册与前端入口绑定 — `pub fn run`
- `runtime_session_attention.rs` — 后端能力封装 — `pub(crate) fn derive_runtime_session_attention`
- `session_continuation_store.rs` — 后端能力封装 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn empty_store_with_warning; pub(crate) fn confirm_continuation; pub(crate) fn run_stub`
- `workbench_snapshot_types.rs` — 工作台持久化/快照/索引链路 — `(无显式pub/导出)`

### ② 派发/执行(工作流执行与运行闸)
- `h4_execution_boundary.rs` — 专项治理/恢复/派发执行子流程 — `pub(crate) fn is_h4_active_attempt_status; pub(crate) fn h4_result_count; pub(crate) fn h4_unknown_result_status; pub(crate) fn h4_unknown_result_warning`
- `h5_project_dispatch_bridge.rs` — 专项治理/恢复/派发执行子流程 — `pub(crate) fn preview_h5_project_workflow_dispatch_at`
- `k3_b1_recovery.rs` — 专项治理/恢复/派发执行子流程 — `pub(crate) struct K3B1RecoveryReadModel; pub(crate) struct K3B2GateStatus; pub(crate) struct K3B1RecoveryOption; pub(crate) struct ManualExactCommandContract; pub(crate) struct ManualRecoverySubmissionContract`
- `real_execution_command.rs` — 后端能力封装 — `pub(crate) struct K2ExecutionPointConfig; pub(crate) struct ProductCommandBoundarySpec; pub(crate) struct RealExecutionCommandGateInput; pub(crate) struct RealExecutionCommandDecision; pub(crate) fn legacy_product_command_boundary_spec`
- `workflow_chain_controller.rs` — ★主导线补·**链驱动状态机底座**(拓扑序/节点态/停请求/runaway 上限·director_agent 的链靠它) — `workflow_chain_topological_order; chain_run_record; chain_node_state; chain_run_stop_requested; chain_run_max_nodes`
- `workflow_execution_entrypoints.rs` — ★主导线补·**节点派发授权+执行闸口** — `inspect_workflow_node_dispatch_authorization; ensure_authorized_for_prepare; ensure_valid_dispatch_state; validate_user_reviewed_instruction; classify_codex_resume_failure(错误分类·复用 fix8)`
- `workflow_run_dispatch_entrypoints.rs` — ★主导线补·**派发解析+运行检查**(真派发 thread 从节点活绑定取·非 dispatch 记录) — `inspect_workflow_run_check_at; find_task_package_artifact_for_work_item; update_work_item_state_at`
- `workflow_state_lifecycle_task_package.rs` — ★主导线补·**工作流状态初始化+任务包草案/预览** — `read_workflow_state_snapshot; initialize_workflow_state_at; bootstrap_project_workflow_at; create_task_draft_at; render_task_package_preview_at`
- `workflow_state_store.rs` — 工作流状态机与生命周期编排 — `pub(crate) fn read_value; pub(crate) fn validate_value; pub(crate) fn write_validated; pub(crate) fn backup_file; pub(crate) fn atomic_write`

### ③ 工作流编排/状态机(任务流组织)
- `workflow_audit.rs` — 工作流核心能力实现 — `pub(crate) struct WorkItemStateChangedAudit; pub(crate) fn work_item_state_changed; pub(crate) struct WorkflowPermissionDecisionRecordedAudit; pub(crate) fn workflow_permission_decision_recorded; pub(crate) struct K3B1RecoveryDecisionRecordedAudit`

### ④ 记忆五层(观察/候选/正式记忆)
- `formal_memory_lifecycle.rs` — 记忆候选与观察流水治理 — `pub(crate) fn preview_operation; pub(crate) fn record_operation`
- `formal_memory_store.rs` — 记忆/候选/候选店与持久化 — `pub(crate) fn sidecar_path; pub(crate) fn lock_path_for_sidecar; pub(crate) fn load_store; pub(crate) fn create_record; pub(crate) fn summarize_store`
- `lib_memory_entity_relation_tests.rs` — 记忆候选与观察流水治理 — `(无显式pub/导出)`
- `lib_memory_lint_mature_pattern_tests.rs` — 记忆候选与观察流水治理 — `(无显式pub/导出)`
- `lib_memory_store_context_tests.rs` — 记忆/候选/候选店与持久化 — `(无显式pub/导出)`
- `lib_observation_candidate_tests.rs` — 记忆候选与观察流水治理 — `(无显式pub/导出)`
- `lib_task_memory_packet_tests.rs` — 记忆候选与观察流水治理 — `(无显式pub/导出)`
- `memory_candidate_store.rs` — 记忆/候选/候选店与持久化 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn create_candidate; pub(crate) fn record_decision; pub(crate) fn adopt_candidate_to_formal_memory`
- `memory_capture_bus.rs` — 审核事件采集/一致性校验 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn capture_event`
- `memory_consistency.rs` — 审核事件采集/一致性校验 — `pub(crate) fn derive_store_integrity_findings`
- `memory_context_entrypoints.rs` — 记忆候选与观察流水治理 — `pub(crate) fn adopt_memory_candidate_to_formal_memory_at`
- `memory_daily_loop.rs` — 记忆候选与观察流水治理 — `pub(crate) struct MemoryDailyLoopContext; pub(crate) struct MemoryDailyLoopWriteIds; pub(crate) fn operation_control_decision_capture_input; pub(crate) fn capture_daily_memory_event; pub(crate) fn worker_report_capture_input`
- `memory_entity_relation_governance.rs` — 记忆候选与观察流水治理 — `pub(crate) fn preview_candidates; pub(crate) fn record_alias_decision; pub(crate) fn record_merge_decision; pub(crate) fn record_relation_decision`
- `memory_entity_relation_store.rs` — 记忆/候选/候选店与持久化 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn with_locked_store; pub(crate) fn summarize_store`
- `memory_lint_engine.rs` — 记忆候选与观察流水治理 — `pub(crate) const CLAIM_SIMILARITY_THRESHOLD; pub(crate) fn build_findings; pub(crate) fn build_maintenance_report; pub(crate) fn open_blocking_findings_for_memory; pub(crate) fn is_open_blocking`
- `memory_lint_store.rs` — 记忆/候选/候选店与持久化 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn run_lint; pub(crate) fn summarize_store`
- `observation_store.rs` — 记忆候选与观察流水治理 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn create_observation; pub(crate) fn create_memory_candidate_from_observation; pub(crate) fn summarize_store`
- `task_memory_injection.rs` — 记忆候选与观察流水治理 — `pub(crate) fn snapshot_from_build_output; pub(crate) fn write_snapshot_to_artifact; pub(crate) fn snapshot_from_artifact; pub(crate) fn summary_from_snapshot; pub(crate) fn missing_summary`
- `task_memory_packet_builder.rs` — 记忆候选与观察流水治理 — `pub(crate) fn build_preview`
- `workbench_sqlite_observation_period.rs` — 记忆候选与观察流水治理 — `pub(crate) enum SqliteObservationFailurePoint; pub(crate) struct SqliteObservationRehearsalReport; pub(crate) struct SqliteObservationSample; pub(crate) struct SqliteObservationProjectedFile; pub(crate) struct SqliteExportVerification`
- `workbench_sqlite_observation_period/tests.rs` — 记忆候选与观察流水治理 — `(无显式pub/导出)`

### ⑤ 治理/闸(流程规则与门控)
- `c4_c6_workflow_governance_entrypoints.rs` — 工作流核心能力实现 — `(无显式pub/导出)`
- `consultant_agent.rs` — 咨询与任务拆解辅助 — `pub(crate) struct ConsultationRisk; pub(crate) struct ConsultationExecutionScope; pub(crate) struct ConsultationProposal; pub(crate) trait ConsultantAgent; pub(crate) struct ProjectContext`
- `control_core.rs` — 后端能力封装 — `pub(crate) enum BlackboardCandidateDecisionOutcome; pub(crate) fn work_item_transition_allowed; pub(crate) fn validate_work_item_state_transition; pub(crate) fn validate_dispatch_prepare; pub(crate) fn validate_dispatch_start`
- `director_agent.rs` — 项目主管总结与风险判断 — `pub(crate) trait DirectorAgent; pub(crate) enum DirectorFinalMarkDecision; pub(crate) struct DirectorFinalMark; pub(crate) struct DirectorFinalMarkContext; pub(crate) trait DirectorFinalMarker`
- `global_supervisor_agent.rs` — 全局主管复核与边界决策 — `pub(crate) const GLOBAL_SUPERVISOR_PROFILE_VERSION; pub(crate) const GLOBAL_SUPERVISOR_MODEL_LABEL; pub(crate) struct SupervisorReviewJson; pub(crate) struct SupervisorTaskJson; pub(crate) fn parse_supervisor_review`
- `lib_stage_c_governance_tests.rs` — 模块测试（验证同文件行为） — `(无显式pub/导出)`
- `lib_workflow_governance_boundary_tests.rs` — 工作流核心能力实现 — `(无显式pub/导出)`
- `mature_pattern_governance.rs` — 后端能力封装 — `pub(crate) fn preview_mature_patterns; pub(crate) fn record_mature_pattern_decision; pub(crate) fn build_acceptance_summary`
- `operation_control.rs` — 后端能力封装 — `pub(crate) struct OperationControlReadModel; pub(crate) struct OperationControlItem; pub(crate) struct OperationDeveloperDetail; pub(crate) struct OperationAuditBoundary; pub(crate) struct OperationRuntimeBoundary`

### ⑥ 读模型(读模型聚合与摘要)
- `page_read_model.rs` — 后端能力封装 — `pub(crate) struct PageReadModelContract; pub(crate) struct WorkbenchPageReadModelInventory; pub(crate) struct PageReadModelQueryInput; pub(crate) struct PageReadModelSchemaContract; pub(crate) struct PageSnapshotFieldCoverage`
- `run_history_read_model.rs` — 后端能力封装 — `pub(crate) struct RunHistoryEntry; pub(crate) struct RunHistoryChain; pub(crate) struct RunHistoryReviewFlags; pub(crate) struct RunHistoryList; pub(crate) fn list_project_run_history_at`
- `workflow_read_model.rs` — 工作流读回读模型与摘要层 — `pub(crate) fn derive_project_blackboards; pub(crate) struct WorkflowLedgerDerivationFns; pub(crate) fn derive_workflow_ledger_entries`
- `workflow_read_model_entrypoints.rs` — 工作流读回读模型与摘要层 — `(无显式pub/导出)`
- `workflow_state_json_helpers.rs` — 工作流状态机与生命周期编排 — `(无显式pub/导出)`

### ⑦ 类型/工具(utils,mcp,入口抽象)
- `blackboard_candidate_store.rs` — 后端能力封装 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn record_decision`
- `global_supervisor_review_store.rs` — 审核事件采集/一致性校验 — `pub(crate) struct GlobalSupervisorTaskVerdict; pub(crate) struct GlobalSupervisorReviewRecord; pub(crate) struct GlobalSupervisorReviewAuditEvent; pub(crate) struct GlobalSupervisorBoundaryReviewRecord; pub(crate) struct GlobalSupervisorBoundaryReviewAuditEvent`
- `lib.rs` — 项目主入口/总线与日志存储 — `pub mod codex_db; pub mod mcp; pub fn run_workflow_machine_cli`
- `main.rs` — 项目主入口/总线与日志存储 — `(无显式pub/导出)`
- `manual_relay.rs` — 后端能力封装 — `pub(crate) struct ManualRelayPreviewInput; pub(crate) struct ManualRelayEnvelope; pub(crate) struct ManualRelayTargetBinding; pub(crate) struct ManualRelayPayload; pub(crate) struct ManualRelayPolicy`
- `mature_pattern_store.rs` — 后端能力封装 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn with_locked_store; pub(crate) fn summarize_store`
- `mcp/commands.rs` — MCP 客户端与工具/传输边界 — `pub(crate) fn mcp_canvas_real_execution_blocked_message; pub fn canvas_load; pub fn canvas_save; pub fn save_workflow_template; pub fn list_workflow_templates`
- `mcp/mod.rs` — MCP 客户端与工具/传输边界 — `pub mod commands; pub mod orchestrator; pub mod storage; pub enum McpRole; pub struct McpServerConfig`
- `mcp/orchestrator.rs` — MCP 客户端与工具/传输边界 — `pub struct StartRunRequest; pub struct StartRunResult; pub struct RunStatus; pub enum LoopDecision; pub struct OrchestratorState`
- `mcp/protocol.rs` — MCP 客户端与工具/传输边界 — `pub struct JsonRpcRequest; pub struct JsonRpcResponse; pub fn ok; pub fn error; pub struct JsonRpcError`
- `mcp/storage.rs` — MCP 客户端与工具/传输边界 — `pub struct CanvasNode; pub struct Position; pub struct CanvasEdge; pub struct CanvasDefinition; pub struct WorkflowTemplate`
- `mcp/tools.rs` — MCP 客户端与工具/传输边界 — `pub fn list_tools; pub fn call_tool`
- `plan_authorization_store.rs` — 计划授权与权限动作记录 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn create_authorization; pub(crate) fn record_user_confirmation; pub(crate) fn record_global_boundary_review`
- `project_consultation_proposal_store.rs` — 后端能力封装 — `pub(crate) fn sidecar_path; pub(crate) fn load_store; pub(crate) fn create_proposal; pub(crate) fn render_markdown; pub(crate) fn record_decision`
- `project_workflow_automation.rs` — 工作流核心能力实现 — `pub(crate) const J2_B_B1_PROJECT_ROOT; pub(crate) const J2_B_B2_PROJECT_ROOT; pub(crate) fn run_project_workflow_automation_phase_a_at; pub(crate) fn run_project_workflow_automation_j2_b_b1_at; pub(crate) fn run_project_workflow_automation_j2_b_b1_with_runner`
- `runtime_log_store.rs` — 项目主入口/总线与日志存储 — `pub(crate) fn sidecar_path; pub(crate) fn load_store_or_derive; pub(crate) fn load_store; pub(crate) fn ensure_appendable; pub(crate) fn append_session_continuation_attempt`
- `secretary_agent.rs` — 后端能力封装 — `pub(crate) struct SecretaryExplainFacts; pub(crate) fn load_secretary_explain_facts; pub(crate) fn build_secretary_explain_prompt; pub(crate) struct SecretaryExplainOutcome; pub(crate) fn run_secretary_explain_core`
- `store_hygiene.rs` — 后端能力封装 — `pub(crate) struct SweepCanvasRunResidueRequest; pub(crate) struct CanvasRunResidueItem; pub(crate) struct SweepCanvasRunResidueResult; pub(crate) fn sweep_canvas_run_residue`
- `types.rs` — 后端能力封装 — `(无显式pub/导出)`
- `utils/fs_ops.rs` — 工具函数与共享基础设施 — `pub(crate) fn remove_file_if_exists; pub(crate) fn fixture_dir`
- `utils/hash.rs` — 工具函数与共享基础设施 — `pub(crate) const WORKBENCH_SOURCE_AGGREGATE_HASH_ALGORITHM; pub(crate) struct WorkbenchSourceAggregateHashEntry; pub(crate) fn sha256_hex; pub(crate) fn sha256_hex_bytes; pub(crate) fn short_hash`
- `utils/mod.rs` — 工具函数与共享基础设施 — `pub(crate) mod fs_ops; pub(crate) mod hash; pub(crate) mod normalization; pub(crate) mod store_paths`
- `utils/normalization.rs` — 工具函数与共享基础设施 — `pub(crate) fn normalize_slash_lowercase`
- `utils/store_paths.rs` — 工具函数与共享基础设施 — `pub(crate) fn sidecar_path`
- `worker_protocol.rs` — 专项治理/恢复/派发执行子流程 — `pub(crate) fn derive_worker_protocol_read_model`
- `worker_report.rs` — 专项治理/恢复/派发执行子流程 — `pub(crate) struct WorkerReport; pub(crate) const WORKER_REPORT_CONTRACT_TEXT; pub(crate) fn build_goals_with_contract; pub(crate) fn parse_worker_report; pub(crate) fn help_signal_from_raw`

### ⑧ 平台底座(工作台sqlite/快照/投影/测试)
- （本区域无显式文件）

## 前端能力（按六面 + 工具）
### ① 首页 / 工作台壳（公共骨架）
- `App.tsx` — 工作台首页壳与公共骨架 — `export { RightDetailPanel, workspaceRailItems };export function App`
- `components/ActiveWorkbenchView.tsx` — 工作台首页壳与公共骨架 — `export type ActiveWorkbenchViewProps;export function renderActiveWorkbenchView`
- `components/Badge.tsx` — 工作台首页壳与公共骨架 — `export function Badge`
- `components/Metric.tsx` — 工作台首页壳与公共骨架 — `export function Metric`
- `components/RightDetailPanel.tsx` — 工作台首页壳与公共骨架 — `export function RightDetailPanel`
- `components/SourceStylePlaceholder.tsx` — 工作台首页壳与公共骨架 — `export function SourceStylePlaceholder`
- `components/WorkbenchPrimitives.tsx` — 工作台首页壳与公共骨架 — `export function SummaryTile;export function DetailLine`
- `components/WorkbenchShell.tsx` — 工作台首页壳与公共骨架 — `export function WorkbenchShell`
- `components/WorkflowStatePanel.tsx` — 工作台首页壳与公共骨架 — `export function WorkflowStatePanel`
- `views/HomeView.tsx` — 工作台首页壳与公共骨架 — `export function HomeView`
- `views/SettingsView.tsx` — 工作台首页壳与公共骨架 — `export function SettingsView`
- `views/SkillsBoardView.tsx` — 工作台首页壳与公共骨架 — `export function SkillsBoardView`

### ② 项目页（交办/流程画布/执行）
- `views/CanvasView.tsx` — 项目页的流程、画布和任务面板 — `export function CanvasViewWithProvider`
- `views/projects/ProjectCanvasDetailPrimitives.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectCanvasDetailLine`
- `views/projects/ProjectGallery.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectGallery`
- `views/projects/ProjectJiaobanPanel.tsx` — 项目页的流程、画布和任务面板 — `export type ProjectJiaobanPanelProps;export function ProjectJiaobanPanel;export const NEW_SESSION_CHOICE;export function shouldRequestBoundaryReview;export function JiaobanHistoryColumn`
- `views/projects/ProjectOverviewPanels.tsx` — 项目页的流程、画布和任务面板 — `export { DetailLine };export function ProjectOverview;export function ProjectAgentMovedPanel;export function ProjectToolPlaceholder`
- `views/projects/ProjectReferencePanels.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectHandoffEvidencePanel;export function ProjectResourcesPanel`
- `views/projects/ProjectTaskDraftPanels.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectWorkflowDraftPanel;export function TaskFileGenerationController;export function TaskDispatchReadinessController;export function TaskDispatchReadinessShell;export function TaskDispatchReadinessDetails`
- `views/projects/ProjectWorkflowCanvasView.tsx` — 项目页的流程、画布和任务面板 — `export type ProjectWorkflowCanvasSidePanelProps;export function ProjectWorkflowCanvasView;export function ProjectWorkflowReactFlowCanvas;export function ProjectCanvasAttentionPanel;export function ProjectCanvasEditBoundaryPanel`
- `views/projects/ProjectWorkflowDerivedPanels.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectCanvasDerivedSummary;export function DerivedWorkflowSummary`
- `views/projects/ProjectWorkflowExecutionHelpers.ts` — 项目页的流程、画布和任务面板 — `export function projectWorkflowDispatchesForCurrentWorkItem;export function projectProductCommandStatusLabel;export function projectAttemptStatusLabel;export function projectRuntimeAttentionValue;export function projectProductResultCountLabel`
- `views/projects/ProjectWorkflowExecutionPanels.tsx` — 项目页的流程、画布和任务面板 — `export function WorkItemOrchestrationCard`
- `views/projects/ProjectWorkflowGovernancePanels.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectDirectorTaskPlanCard;export function AutoAdvanceRoleLoopButton;export function buildPrepareAuthorizedAutoDispatchAction;export function ProjectConsultationProposalCard;export function GlobalBoundaryReviewCard`
- `views/projects/projectWorkflowLabels.ts` — 项目页的流程、画布和任务面板 — `export function stateLabel;export function stateActionLabel;export function roleLabel;export function runCheckTone;export function runCheckStatusLabel`
- `views/projects/ProjectWorkflowMemoryPanels.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectBlackboardPanel;export function CandidateGovernanceStrip`
- `views/projects/ProjectWorkflowRecoveryPanels.tsx` — 项目页的流程、画布和任务面板 — `export function K3B1RecoveryCard`
- `views/projects/ProjectWorkflowRunCheckPanel.tsx` — 项目页的流程、画布和任务面板 — `export const WorkflowRunCheckPanel;export function WorkflowRunCheckDetails`
- `views/projects/ProjectWorkflowSidePanel.tsx` — 项目页的流程、画布和任务面板 — `export { WorkflowRunCheckDetails };export function ProjectCanvasSidePanel;export function ProjectCanvasNodeDetailView`
- `views/projects/ProjectWorkflowUnifiedExecutionCard.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectUnifiedExecutionStateCard`
- `views/projects/ProjectWorkspaceShell.tsx` — 项目页的流程、画布和任务面板 — `export type ProjectWorkspaceToolKey;export type ProjectToolKey;export const projectTools;export type ProjectDetailProps;export type ProjectWorkspaceShellProps`
- `views/ProjectsView.tsx` — 项目页的流程、画布和任务面板 — `export function ProjectsView;export function filterProjectSessionsForProject;export function ProjectDetail`
- `views/WorkflowCanvasEngine.tsx` — 项目页的流程、画布和任务面板 — `export function WorkflowCanvasEngine`

### ③ 智能体页（会话与消息交互）
- `views/agent/AgentAdapterBoundaryPanels.tsx` — 智能体页的会话、转录和手动 relay — `export function AgentAdapterCapabilityPanel;export function ProviderAvailabilityPanel;export function SessionOperationBoundaryPanel`
- `views/agent/AgentChatComposer.tsx` — 智能体页的会话、转录和手动 relay — `export type AgentConversationSendMode;export function AgentChatComposer;export function readbackStatusLabel;export function codexControlReasonLabel`
- `views/agent/AgentContinuationBoundaryPanels.tsx` — 智能体页的会话、转录和手动 relay — `export function SessionContinuationPreviewPanel;export function ControlledSessionContinuationPanel;export function H2RealResumeAuthorizationPanel;export function H2RealResumeExecutionDecisionPanel;export function RuntimeSessionAttentionPanel`
- `views/agent/AgentConversationShell.tsx` — 智能体页的会话、转录和手动 relay — `export const MANUAL_RELAY_FRONTEND_TIMEOUT_MS;export const J1_DEFAULT_DENIED_PATHS;export type RelayBindingState;export type ManualRelayPollFailureDecision;export function nextManualRelayPollFailureDecision`
- `views/agent/AgentDeveloperPanels.tsx` — 智能体页的会话、转录和手动 relay — `export function AgentDeveloperPanels`
- `views/agent/AgentExecutionPanels.tsx` — 智能体页的会话、转录和手动 relay — `export function CodexControlEntryPanel;export function UnifiedExecutionStatusPanel`
- `views/agent/agentLabels.ts` — 智能体页的会话、转录和手动 relay — `export const J1_DEFAULT_DENIED_PATHS;export function codexControlPreviewLabel;export function codexControlReasonLabel;export type AgentUserFacingError;export function manualRelayReasonLabel`
- `views/agent/AgentSessionList.tsx` — 智能体页的会话、转录和手动 relay — `export const NO_PROJECT_KEY;export const NO_PROJECT_LABEL;export type SessionReadFilter;export const SESSION_READ_FILTERS;export function softwareKeyOf`
- `views/agent/AgentSoftwareFilterBar.tsx` — 智能体页的会话、转录和手动 relay — `export function AgentSoftwareFilterBar`
- `views/agent/TranscriptViews.tsx` — 智能体页的会话、转录和手动 relay — `export function TranscriptTimeline;export function ChatTranscript;export function WarningStrip;export function readbackCountLabel`
- `views/agent/useAgentSessionPage.ts` — 智能体页的会话、转录和手动 relay — `export function useAgentSessionPage`
- `views/agent/useAgentTranscriptLoader.ts` — 智能体页的会话、转录和手动 relay — `export function useAgentTranscriptLoader`
- `views/AgentView.tsx` — 智能体页的会话、转录和手动 relay — `export function AgentView`

### ④ 记忆页（候选/正式记忆与知识）
- `components/DailyMemoryCandidateInbox.tsx` — 记忆页的候选、正式记忆和治理 — `export function DailyMemoryCandidateInbox`
- `views/KnowledgeBaseView.tsx` — 记忆页的候选、正式记忆和治理 — `export function KnowledgeBaseView`
- `views/memory/MemoryActionBuilders.ts` — 记忆页的候选、正式记忆和治理 — `export function buildLifecycleRequest;export function confirmationForOperation;export function primaryProjectRoot;export function maturePatternDecisionLabel;export function maturePatternDecisionReason`
- `views/memory/MemoryDetailPanels.tsx` — 记忆页的候选、正式记忆和治理 — `export function FormalMemoryDetail;export function CandidateMemoryDetail;export function operationLabel;export function sourceText`
- `views/memory/MemoryListPanels.tsx` — 记忆页的候选、正式记忆和治理 — `export function FormalMemoryItem;export function CandidateMemoryItem;export function EntityCandidateItem;export function MergeCandidateItem;export function RelationCandidateItem`
- `views/memory/MemoryWorkbenchSummary.tsx` — 记忆页的候选、正式记忆和治理 — `export function MemoryCenterStats;export function MemoryWorkbenchSummary`
- `views/MemoryCenterView.tsx` — 记忆页的候选、正式记忆和治理 — `export function MemoryCenterView`

### ⑤ 秘书页 / 待办（提示与建议）
- `components/PermissionDialog.tsx` — 秘书页的待办、解释和权限提示 — `export function PermissionDialog`
- `components/SecretaryBoardView.tsx` — 秘书页的待办、解释和权限提示 — `export function SecretaryBoardView`
- `components/SecretaryBrief.tsx` — 秘书页的待办、解释和权限提示 — `export function SecretaryBrief;export function SecretaryPendingBoardSection;export const SecretaryExplainSection`

### ⑥ 审计 / 运营面（运行状态与历史）
- `views/HarnessBoardView.tsx` — 审计/运营面与运行状态视图 — `export function HarnessBoardView`
- `views/OfflineRoleOrchestrationPanel.tsx` — 审计/运营面与运行状态视图 — `export const defaultOfflineDispatchBlock;export function OfflineRoleOrchestrationPanel;export function OfflineDispatchProposalPreview;export function parseOfflineDispatchBlock;export function buildOfflineRoleDispatchAction`
- `views/RunningWorkflowsView.tsx` — 审计/运营面与运行状态视图 — `export function RunningWorkflowsView`
- `views/WorkflowCommandConsoleView.tsx` — 审计/运营面与运行状态视图 — `export function WorkflowCommandConsoleView`

### ⑦ 前端工具层（通用类型/数据派生）
- `lib/adapterCapabilities.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function deriveAgentAdapterDescriptors`
- `lib/browserPreviewSnapshot.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const previewProjectRoot;export const browserPreviewSessions;export const browserPreviewSnapshot;export function browserPreviewTranscript;export function browserPreviewSessionPage`
- `lib/browserPreviewWorkflowState.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const browserPreviewWorkflowState;export const browserPreviewProposalStore;export const browserPreviewPlanAuthorizationStore`
- `lib/candidateGovernance.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type BlackboardCandidateOverlay;export function buildBlackboardCandidateOverlay;export type MemoryCandidateSummary;export function summarizeMemoryCandidateStore;export type FormalMemorySummary`
- `lib/canvasNodeData.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type CanvasCustomField;export function canvasScope;export type SessionPolicy;export type CanvasNodeData;export type CanvasNodeKindPreset`
- `lib/canvasSurfaceBoundaries.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type CanvasSurfaceBoundaryKind;export type CanvasSurfaceBoundaryItem;export type CanvasSurfaceBoundary;export const experimentCanvasBoundary;export const projectWorkflowCanvasBoundary`
- `lib/canvasSurfaceConfig.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type CanvasSurfaceCapabilities;export type CanvasSurfaceConfig;export const experimentCanvasSurfaceConfig;export function projectCanvasSurfaceConfig`
- `lib/conversationEngine.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type PendingUserMessageInput;export type ManualRelayPendingUserMessageInput;export type ManualRelayOptimisticUserMessageInput;export type ManualRelayAssistantMessageInput;export type ManualRelayLiveTranscriptEventsInput`
- `lib/conversationTurns.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function rawTypeOf;export function conversationTurns`
- `lib/emptySnapshot.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const emptySnapshot`
- `lib/format.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function formatDate;export function shortId;export function relativeTime;export function pathTail;export function warningText`
- `lib/h2RealResumeAuthorization.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function deriveH2RealResumeAuthorizationReadiness;export function deriveH2RealResumeExecutionDecisionSurface`
- `lib/knowledgeBase.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type KnowledgeSourceAnchor;export type KnowledgeMemoryLink;export type KnowledgeTaskReferenceSummary;export type KnowledgeMemoryCaptureSummary;export type KnowledgeCandidateDraft`
- `lib/memoryCenter.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type MemorySourceSummary;export type MemoryVersionSummary;export type MemoryAuditSummary;export type MemoryTaskEligibilitySummary;export type MemoryConflictSummary`
- `lib/memoryDailyLoop.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type DailyMemoryCandidateInboxItem;export type DailyMemoryCandidateInbox;export function deriveDailyMemoryCandidateInbox;export function buildAdoptMemoryCandidateAction;export function buildBatchAdoptMemoryCandidatesAction`
- `lib/pageReadModel.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type PageReadModelContract;export type WorkbenchPageReadModelInventory;export type PageReadModelQueryInput;export type PageReadModelSchemaContract;export type PageSnapshotFieldCoverage`
- `lib/pageReadModelRuntime.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const batchOneWorkbenchPageIds;export type BatchOneWorkbenchPageId;export type BatchOnePageReadModelResults;export async function loadWorkbenchSnapshotFromPageQueries;export function indexPageReadModelResults`
- `lib/pageSelectors.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type PageSelectorSourceBoundary;export type ProjectListItemReadModel;export type ProjectsPageReadModel;export type AgentProjectOptionReadModel;export type AgentSessionSummaryReadModel`
- `lib/planAuthorization.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const planAuthorizationStatusLabels;export const autoDispatchGuardStatusLabels;export const globalBoundaryReviewStatusLabels;export function summarizePlanAuthorizationStore;export function summarizeAutoDispatchGuardResult`
- `lib/projectCanvas.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type ProjectCanvasStatus;export type ProjectCanvasNodeType;export type ProjectCanvasBadgeTone;export type ProjectCanvasSourceRef;export type ProjectCanvasBadge`
- `lib/projectConsultationProposal.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const projectConsultationProposalStatusLabels;export type ProjectConsultationProposalSummary;export function summarizeProjectConsultationProposalStore`
- `lib/projectDirectorTaskPlan.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export const projectDirectorPlannedTaskStatusLabels;export function summarizeProjectDirectorTaskPlan`
- `lib/providerAvailability.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function deriveProviderAvailabilitySummaries`
- `lib/runQueue.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type RunQueueStatus;export type UserConfirmationKind;export type FailureRecoverability;export type RunQueueItem;export type UserConfirmationQueueItem`
- `lib/secretaryReadModel.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type SecretarySourceRef;export type SecretaryGlobalSummary;export type SecretaryProjectSummary;export type SecretarySuggestion;export type SecretaryRiskSignal`
- `lib/sessionContinuation.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function deriveSessionContinuationPreviews;export function inspectSessionContinuationGuard`
- `lib/sessionOperations.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function deriveSessionOperationDescriptors`
- `lib/tauri.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export function loadWorkbenchSnapshot;export function queryWorkbenchPageReadModel;export function loadSessionContinuationStore;export function confirmControlledSessionContinuation;export function runControlledSessionContinuationStub`
- `lib/tauriWindow.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export async function setTauriWindowTitle`
- `lib/types.ts` — 前端通用类型、状态派生和 Tauri 适配 — `(无显式export)`
- `lib/types/agentSession.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type AdapterCapabilityKind;export type AdapterCapabilityStatus;export type AdapterCapability;export type AgentAdapterType;export type AgentAdapterDescriptor`
- `lib/types/canvas.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type CanvasNodeRole;export type CanvasNode;export type CanvasEdge;export type CanvasScope;export type CanvasDefinition`
- `lib/types/execution.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type RealExecutionProductCommandRequest;export type RealExecutionProductCommandPermissionEnvelope;export type RealExecutionProductCommandReadiness;export type RealExecutionProductCommandGuardPreview;export type RealExecutionProductCommandDiagnosticsSummary`
- `lib/types/manualRelay.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type ManualRelayPreviewInput;export type ManualRelayTargetBinding;export type ManualRelayPayload;export type ManualRelayPolicy;export type ManualRelayFutureHooks`
- `lib/types/memory.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type MemoryScope;export type MemorySourceRef;export type MemoryLifecycleStatus;export type MemoryAuditRef;export type MemoryConflict`
- `lib/types/workbenchSnapshot.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type WorkbenchSnapshot`
- `lib/types/workflow.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type PlanAuthorizationStatus;export type PlanAuthorizationActorScope;export type PlanAuthorizationResourceScope;export type PlanAuthorizationStopCondition;export type AuthorizedExecutionScope`
- `lib/workbenchCoreTypes.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type FileCandidate;export type HarnessCandidate;export type HarnessEntrypoint;export type HarnessResource;export type ProjectRecord`
- `lib/workbenchNavigation.ts` — 前端通用类型、状态派生和 Tauri 适配 — `export type ViewKey;export type RightPanelKey;export type WorkbenchNavItem;export type WorkbenchNavGroup;export const homeNavItem`
- `main.tsx` — 前端通用类型、状态派生和 Tauri 适配 — `(无显式export)`
- `vite-env.d.ts` — 前端通用类型、状态派生和 Tauri 适配 — `(无显式export)`

## 普查中撞见的“疑似两套”（仅供主导线核对）
- `humanize_consult_error` 在 `secretary_agent.rs:185` 与 `global_supervisor_agent.rs:410` 都存在：同名“人话化咨询错误”二处实现，像是职责分层或重复封装，是否重复留主导线判。
- `open_issues` / `permission_requests` / `direction_risks` / `follow_up_suggestions` 在 `worker_report.rs:39-90` 是源头，在 `workflow_read_model_entrypoints.rs:943-994`、`director_agent.rs:1249-1261`、`ProjectWorkflowExecutionPanels.tsx:417-577` 是读模型与前端投影；像同一条链，不像第二套生产者。
- `capture_event` 在 `memory_capture_bus.rs:42` 是统一采集口，`director_agent.rs:624-696` 与 `project_workflow_automation.rs:2838-2945` 是往里送总结/记忆候选的调用点；像链路复用，不像并行第二套。
