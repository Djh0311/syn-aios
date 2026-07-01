import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { SettingsView } from "../src/views/SettingsView";
import { devNavItems } from "../src/lib/workbenchNavigation";
import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../src/lib/types";
import { pageReadModelInventoryFixture } from "./fixtures/pageReadModelFixture";

const snapshot = {
  summary: { generated_at: "2026-06-11T00:00:00Z", project_count: 0, session_count: 0, skill_count: 0, plugin_count: 0, task_count: 0, warning_count: 0 },
  projects: [],
  sessions: [],
  skills: [],
  plugins: [],
  tasks: [],
  agent_adapters: [],
  session_operations: [],
  provider_availability: [],
  session_continuation_previews: [],
  session_continuation_store: { continuations: [], attempts: [], audit_events: [], warnings: [] },
  runtime_session_attention: [],
  session_run_status_summaries: [],
  runtime_log_store: { entries: [], warnings: [] },
  worker_protocol: { warnings: [] },
  real_execution_product_commands: null,
  project_workflow_automation: null,
  page_read_model_inventory: pageReadModelInventoryFixture(),
  diagnostic_summary: { degraded_states: [] },
  diagnostics: {},
} as unknown as WorkbenchSnapshot;

const workflowState = {
  counts: { workflows: 0 },
} as unknown as WorkflowStateSnapshot;

const settingsText = visibleText(
  <SettingsView
    snapshot={snapshot}
    workflowState={workflowState}
    workflowStateError={null}
    hasRealSnapshot={true}
    developerItems={devNavItems}
    onNavigate={() => {}}
  />,
);

for (const expectedText of [
  "页面读模型合同",
  "R4-A1",
  "2 个页面合同",
  "仅冻结合同",
  "页面仍使用既有 WorkbenchSnapshot",
  "HomePageReadModel",
  "AgentsPageReadModel",
  "首屏禁止：控制中心式全量边界面板 / 未实现执行按钮",
]) {
  assert(settingsText.includes(expectedText), `R4-A1 设置页合同展示缺少 ${expectedText}`);
}

for (const forbiddenText of ["已切到按页查询", "执行 codex", "恢复会话", "密钥值", "令牌值"]) {
  assert(!settingsText.includes(forbiddenText), `R4-A1 设置页合同展示不应出现 ${forbiddenText}`);
}

console.log("r4 page read model settings test passed");

function visibleText(root: React.ReactNode): string {
  return renderToStaticMarkup(root)
    .replace(/<[^>]*>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#x27;/g, "'")
    .replace(/&quot;/g, '"');
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
