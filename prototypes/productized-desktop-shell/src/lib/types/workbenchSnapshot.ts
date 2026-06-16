import type {
  Diagnostics,
  IndexSummary,
  PluginRecord,
  ProjectRecord,
  SessionRecord,
  SkillRecord,
  TaskEntry,
} from "../workbenchCoreTypes";
import type { WorkbenchPageReadModelInventory } from "../pageReadModel";
import type {
  AgentAdapterDescriptor,
  DiagnosticSummary,
  ProviderAvailabilitySummary,
  RuntimeLogStoreV1,
  RuntimeSessionAttention,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionOperationDescriptor,
  SessionRunStatusSummary,
} from "./agentSession";
import type {
  K3B1RecoveryReadModel,
  OperationControlReadModel,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  WorkerProtocolReadModel,
} from "./execution";

export type WorkbenchSnapshot = {
  summary: IndexSummary;
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  skills: SkillRecord[];
  plugins: PluginRecord[];
  tasks: TaskEntry[];
  agent_adapters: AgentAdapterDescriptor[];
  session_operations: SessionOperationDescriptor[];
  provider_availability: ProviderAvailabilitySummary[];
  session_continuation_previews: SessionContinuationPreview[];
  session_continuation_store: SessionContinuationStoreV1;
  runtime_session_attention: RuntimeSessionAttention[];
  session_run_status_summaries: SessionRunStatusSummary[];
  runtime_log_store: RuntimeLogStoreV1;
  worker_protocol: WorkerProtocolReadModel;
  real_execution_product_commands?: RealExecutionProductCommandReadModel | null;
  project_workflow_automation?: ProjectWorkflowAutomationReadModel | null;
  k3_b1_recovery?: K3B1RecoveryReadModel | null;
  operation_control?: OperationControlReadModel | null;
  page_read_model_inventory: WorkbenchPageReadModelInventory;
  diagnostic_summary: DiagnosticSummary;
  diagnostics: Diagnostics;
};
