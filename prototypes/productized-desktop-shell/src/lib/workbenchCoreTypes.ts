export type FileCandidate = {
  kind?: string | null;
  name?: string | null;
  path: string;
  warnings: string[];
};

export type HarnessCandidate = {
  entry_type?: string | null;
  name?: string | null;
  path: string;
  source?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type HarnessEntrypoint = {
  entry_type?: string | null;
  name?: string | null;
  path: string;
  source_kind?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type HarnessResource = {
  root_path: string;
  display_name?: string | null;
  harness_kind?: string | null;
  agent_type?: string | null;
  adapter_id?: string | null;
  source_kind?: string | null;
  capabilities: string[];
  manifest_path?: string | null;
  readme_path?: string | null;
  version?: string | null;
  entrypoints: HarnessEntrypoint[];
  permission_level?: string | null;
  size_bytes?: number | null;
  updated_at_ms?: number | null;
  warnings: string[];
};

export type ProjectRecord = {
  project_root: string;
  name: string;
  active_hint: boolean;
  thread_count: number;
  active_thread_count: number;
  archived_thread_count: number;
  latest_updated_at_ms?: number | null;
  authority_files: FileCandidate[];
  handoff_files: FileCandidate[];
  evidence_files: FileCandidate[];
  harness_candidates: HarnessCandidate[];
  harness_resources: HarnessResource[];
  context_warnings: string[];
  warnings: string[];
};

export type SessionRecord = {
  thread_id: string;
  title: string;
  project_root?: string | null;
  updated_at_ms?: number | null;
  archived: boolean;
  rollout_exists: boolean;
  rollout_path?: string | null;
  model?: string | null;
  reasoning_effort?: string | null;
  thread_source?: string | null;
  warnings: string[];
};

export type CodexSessionPageRequest = {
  page_size?: number | null;
  offset?: number | null;
  include_archived?: boolean | null;
  archived_only?: boolean | null;
  query?: string | null;
};

export type CodexSessionPage = {
  sessions: SessionRecord[];
  page_size: number;
  offset: number;
  has_more: boolean;
  include_archived: boolean;
  archived_only: boolean;
  warnings: string[];
  source: string;
};

export type CodexTranscriptEvent = {
  event_id: string;
  timestamp?: string | null;
  event_type?: string | null;
  actor?: string | null;
  role?: string | null;
  turn_id?: string | null;
  call_id?: string | null;
  tool_name?: string | null;
  text?: string | null;
  arguments?: unknown;
  output?: unknown;
  stdout?: string | null;
  stderr?: string | null;
  exit_code?: number | string | null;
  metadata?: Record<string, unknown> | null;
  warnings: string[];
};

export type CodexTranscriptPagination = {
  mode: string;
  page_size: number;
  returned_events: number;
  total_line_count: number;
  selected_line_count: number;
  has_older: boolean;
  older_before_line?: number | null;
};

export type CodexTranscriptPageRequest = {
  thread_id: string;
  limit?: number | null;
  before_line?: number | null;
};

export type CodexTranscript = {
  thread_id: string;
  rollout_path: string;
  project_path?: string | null;
  title?: string | null;
  created_at_ms?: number | null;
  updated_at_ms?: number | null;
  viewer_boundary: CodexTranscriptViewerBoundary;
  events: CodexTranscriptEvent[];
  summary: {
    total_events: number;
    event_type_counts: Record<string, number>;
    unknown_event_count: number;
    warning_count: number;
    encrypted_content_event_count: number;
    sensitive_like_event_count: number;
  };
  pagination?: CodexTranscriptPagination | null;
  warnings: string[];
  source_stats: {
    index_thread_count?: number | null;
    jsonl?: {
      line_count?: number;
      parsed_line_count?: number;
      bad_json_line_count?: number;
      selected_line_count?: number;
    };
    raw_type_counts?: Record<string, number>;
    payload_type_counts?: Record<string, number>;
  };
};

export type CodexTranscriptViewerBoundary = {
  view_kind: string;
  reads_session_history: boolean;
  is_execution_readback: boolean;
  real_execution_readback_performed: boolean;
  execution_readback_scope: string;
  warnings: string[];
};

export type SkillRecord = {
  skill_id: string;
  title: string;
  description?: string | null;
  path: string;
  source_type: string;
  plugin_name?: string | null;
  plugin_version?: string | null;
  warnings: string[];
};

export type PluginRecord = {
  plugin_name: string;
  plugin_version: string;
  homepage?: string | null;
  skill_count: number;
  has_apps: boolean;
  has_mcp_servers: boolean;
  warnings: string[];
};

export type TaskEntry = {
  status: string;
  title: string;
};

export type Diagnostics = {
  index_path: string;
  tasks_path: string;
  generated_at?: string | null;
  top_level_warning_count: number;
  context_warning_count: number;
  allowed_project_path_count: number;
  allowed_rollout_path_count: number;
  release_bundle_enabled: boolean;
  notes: string[];
};

export type IndexSummary = {
  generated_at?: string | null;
  project_count: number;
  session_count: number;
  skill_count: number;
  plugin_count: number;
  task_count: number;
  warning_count: number;
};
