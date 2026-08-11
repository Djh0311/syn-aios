import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  createSecretarySourceRouteRequest,
  parseSecretarySourceRouteResolution,
  sameSecretarySourceRouteResolution,
  shouldReleaseConsumedSecretarySourceReadCut,
} from "../src/lib/secretaryReadModel";
import {
  M4_PROPOSAL_SOURCE_OWNER_REF,
  M4_SECRETARY_SOURCE_ROUTE_RESOLUTION_SCHEMA,
  M4_WORK_ITEM_SOURCE_OWNER_REF,
  type SecretarySourceFocus,
} from "../src/lib/types/m4Secretary";
import type {
  ProjectConsultationProposal,
  ProjectConsultationProposalStoreV1,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../src/lib/types";
import {
  resolveSecretarySourceProjectSelection,
} from "../src/views/ProjectsView";
import { ProjectWorkflowDraftPanel } from "../src/views/projects/ProjectTaskDraftPanels";
import {
  JiaobanRegisteredProposalSourceFocus,
  ProjectJiaobanPanel,
} from "../src/views/projects/ProjectJiaobanPanel";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

const routeRef = (character: string) => `source-route:sha256:${character.repeat(64)}`;

const project: ProjectRecord = {
  project_root: "/isolated/project-a",
  name: "Project A",
  active_hint: true,
  thread_count: 0,
  active_thread_count: 0,
  archived_thread_count: 0,
  authority_files: [],
  handoff_files: [],
  evidence_files: [],
  harness_candidates: [],
  harness_resources: [],
  context_warnings: [],
  warnings: [],
};

const workflowState = {
  exists: true,
  initialized: true,
  path: "/isolated/profile/workflow-state.json",
  counts: {
    projects: 1,
    agent_adapters: 0,
    workflows: 1,
    nodes: 0,
    edges: 0,
    work_items: 1,
    artifacts: 0,
    reviews: 0,
    audit_events: 0,
    capabilities: 0,
    harness_resources: 0,
  },
  project_workflows: [{
    project_id: "project-a",
    project_root: project.project_root,
    workflow_id: "workflow-a",
    title: "Workflow A",
    state: "draft",
    node_count: 0,
    edge_count: 0,
    task_draft_count: 1,
    task_drafts: [{
      work_item_id: "same-object-id",
      workflow_id: "workflow-a",
      title: "Exact work item",
      state: "draft",
      next_states: [],
      recent_audit_events: [],
    }],
    node_session_bindings: [],
    node_dispatches: [],
    director_reviews: [],
    execution_controls: [],
    permission_requests: [],
    execution_attempts: [],
  }],
  warnings: [],
} as unknown as WorkflowStateSnapshot;

const exactProposal: ProjectConsultationProposal = {
  proposal_id: "same-object-id",
  schema_version: "project_consultation_proposal.v1",
  project_id: "project-a",
  workflow_id: "workflow-a",
  title: "Exact registered Proposal",
  user_goal: "Open the exact registered Proposal without unlocking automatic execution.",
  goal_summary: "Exact registered Proposal source",
  proposed_steps: ["Read the registered owner record"],
  scope_draft: {
    allowed_role_ids: [],
    allowed_agent_ids: [],
    allowed_read_roots: [project.project_root],
    allowed_write_roots: [],
    allowed_tools: [],
    allowed_checks: [],
    allowed_task_package_kinds: [],
    stop_conditions: [],
  },
  risks: [],
  worker_acceptance_criteria: ["The exact owner record is visible"],
  control_core_acceptance_criteria: [],
  supervisor_acceptance_criteria: [],
  acceptance_criteria: ["The exact owner record is visible"],
  status: "pending_user_confirmation",
  created_by_role: "project_consultant",
  created_at_ms: 1,
  updated_at_ms: 1,
  suggest_workflow: false,
};

const proposalStore = {
  schema_version: "project_consultation_proposal_store.v1",
  revision: 7,
  proposals: [exactProposal],
  decisions: [],
  audit_events: [],
  updated_at_ms: 1,
  warnings: [],
} as unknown as ProjectConsultationProposalStoreV1;

const wrongWorkflowOnSameProjectRoot = {
  ...workflowState.project_workflows[0],
  project_id: "project-wrong",
  workflow_id: "workflow-wrong",
  title: "Wrong workflow",
  task_drafts: [{
    ...workflowState.project_workflows[0].task_drafts[0],
    workflow_id: "workflow-wrong",
    title: "Wrong same-id work item",
  }],
};
const workflowStateWithSameRootCollision = {
  ...workflowState,
  project_workflows: [wrongWorkflowOnSameProjectRoot, ...workflowState.project_workflows],
} as unknown as WorkflowStateSnapshot;

function expectThrows(action: () => unknown, expectedFragment: string, label: string) {
  let message = "";
  try {
    action();
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes(expectedFragment), label);
}

const workResolution = parseSecretarySourceRouteResolution({
  schema_version: M4_SECRETARY_SOURCE_ROUTE_RESOLUTION_SCHEMA,
  source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF,
  source_object_type: "workflow_attention",
  canonical_source_object_id: "same-object-id",
  source_revision: "5",
  source_route_ref: routeRef("a"),
  target: {
    kind: "WORK_ITEM",
    project_id: "project-a",
    workflow_id: "workflow-a",
    work_item_id: "same-object-id",
    source_revision: "5",
  },
});
const workFocus: SecretarySourceFocus = {
  attempt_id: 1,
  source_owner_ref: workResolution.source_owner_ref,
  source_object_type: workResolution.source_object_type,
  canonical_source_object_id: workResolution.canonical_source_object_id,
  source_revision: workResolution.source_revision,
  source_route_ref: workResolution.source_route_ref,
  target: workResolution.target,
};

const proposalResolution = parseSecretarySourceRouteResolution({
  schema_version: M4_SECRETARY_SOURCE_ROUTE_RESOLUTION_SCHEMA,
  source_owner_ref: M4_PROPOSAL_SOURCE_OWNER_REF,
  source_object_type: "proposal_decision",
  canonical_source_object_id: "same-object-id",
  source_revision: "7",
  source_route_ref: routeRef("b"),
  target: {
    kind: "CONSULTATION_PROPOSAL",
    project_id: "project-a",
    workflow_id: "workflow-a",
    proposal_id: "same-object-id",
    source_revision: "7",
  },
});
const proposalFocus: SecretarySourceFocus = {
  attempt_id: 2,
  source_owner_ref: proposalResolution.source_owner_ref,
  source_object_type: proposalResolution.source_object_type,
  canonical_source_object_id: proposalResolution.canonical_source_object_id,
  source_revision: proposalResolution.source_revision,
  source_route_ref: proposalResolution.source_route_ref,
  target: proposalResolution.target,
};

assertDeepEqual(
  createSecretarySourceRouteRequest({ source_route_ref: routeRef("c") }),
  { source_route_ref: routeRef("c") },
  "renderer request carries only the sealed route capability",
);
expectThrows(
  () => createSecretarySourceRouteRequest({ source_route_ref: "projects" }),
  "source_route_request",
  "raw route/path guesses are rejected before invoke",
);
expectThrows(
  () => createSecretarySourceRouteRequest({
    source_route_ref: routeRef("c"),
    source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF,
  } as never),
  "unknown_source_route_request_field",
  "renderer cannot add owner authority to the resolver request",
);

assert(workResolution.target.kind === "WORK_ITEM", "strict parser accepts finite WorkItem target");
assert(proposalResolution.target.kind === "CONSULTATION_PROPOSAL", "strict parser accepts finite Proposal target");
assert(
  sameSecretarySourceRouteResolution(workResolution, workResolution),
  "the owner read cut accepts an unchanged second sealed-route resolution",
);
assert(
  !sameSecretarySourceRouteResolution(
    workResolution,
    parseSecretarySourceRouteResolution({
      ...workResolution,
      source_revision: "6",
      target: { ...workResolution.target, source_revision: "6" },
    }),
  ),
  "the owner read cut rejects a route whose authoritative revision changes",
);
assert(
  shouldReleaseConsumedSecretarySourceReadCut({
    phase: "CONSUMED",
    route_state_ref: workResolution.source_route_ref,
    focus_attempt_id: 7,
    focus_route_ref: workResolution.source_route_ref,
    read_cut_attempt_id: 7,
  }),
  "an exact consumed source cut can return to ordinary live owner reads",
);
assert(
  !shouldReleaseConsumedSecretarySourceReadCut({
    phase: "CONSUMING",
    route_state_ref: workResolution.source_route_ref,
    focus_attempt_id: 7,
    focus_route_ref: workResolution.source_route_ref,
    read_cut_attempt_id: 7,
  })
    && !shouldReleaseConsumedSecretarySourceReadCut({
      phase: "CONSUMED",
      route_state_ref: workResolution.source_route_ref,
      focus_attempt_id: 7,
      focus_route_ref: workResolution.source_route_ref,
      read_cut_attempt_id: 8,
    })
    && !shouldReleaseConsumedSecretarySourceReadCut({
      phase: "CONSUMED",
      route_state_ref: workResolution.source_route_ref,
      focus_attempt_id: 7,
      focus_route_ref: proposalResolution.source_route_ref,
      read_cut_attempt_id: 7,
    }),
  "an in-flight, route-mismatched, or attempt-mismatched source cut stays sealed",
);
expectThrows(
  () => parseSecretarySourceRouteResolution({
    ...workResolution,
    raw_path: "/isolated/project-a",
  }),
  "unknown_source_route_resolution_field:raw_path",
  "strict response rejects raw path/extra capabilities",
);
expectThrows(
  () => parseSecretarySourceRouteResolution({
    ...workResolution,
    source_owner_ref: M4_PROPOSAL_SOURCE_OWNER_REF,
  }),
  "work_item_binding_invalid",
  "same object id cannot cross an owner binding",
);
expectThrows(
  () => parseSecretarySourceRouteResolution({
    ...proposalResolution,
    target: { ...proposalResolution.target, source_revision: "8" },
  }),
  "revision_mismatch",
  "top-level and finite-target revisions must match exactly",
);

assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: workFocus,
    projects: [],
    workflowState: null,
    proposalStore: null,
    hasRealSnapshot: false,
    workflowStateLoading: true,
    workflowStateError: null,
  }),
  { status: "PENDING" },
  "a Home source cannot turn an in-flight index/workflow read into target missing",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: proposalFocus,
    projects: [project],
    workflowState,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: true,
    workflowStateError: null,
  }),
  { status: "PENDING" },
  "a refresh barrier keeps a stale non-empty Proposal store from being consumed",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: workFocus,
    projects: [project],
    workflowState: null,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: "workflow_read_failed",
  }),
  { status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_PROJECT_MISSING" },
  "a completed failed workflow read does not leave the source focus pending forever",
);

assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: workFocus,
    projects: [project],
    workflowState,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "READY", project_root: project.project_root, tool: "task-packages" },
  "WorkItem focus chooses the exact task-package owner page",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: workFocus,
    projects: [project],
    workflowState: workflowStateWithSameRootCollision,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "READY", project_root: project.project_root, tool: "task-packages" },
  "same-root workflow collisions stay bound to the resolved project/workflow identity",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: proposalFocus,
    projects: [project],
    workflowState,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "READY", project_root: project.project_root, tool: "jiaoban" },
  "Proposal focus chooses the exact consultation owner page despite identical object id",
);

const noWorkItemState = {
  ...workflowState,
  project_workflows: workflowState.project_workflows.map((workflow) => ({
    ...workflow,
    task_drafts: [],
  })),
};
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: workFocus,
    projects: [project],
    workflowState: noWorkItemState,
    proposalStore,
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_RECORD_MISSING" },
  "same-id Proposal never substitutes for a missing WorkItem",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: proposalFocus,
    projects: [project],
    workflowState,
    proposalStore: { ...proposalStore, revision: 8 },
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "READY", project_root: project.project_root, tool: "jiaoban" },
  "a newer store revision does not make an exact resolved Proposal stale",
);
assertDeepEqual(
  resolveSecretarySourceProjectSelection({
    focus: proposalFocus,
    projects: [project],
    workflowState,
    proposalStore: { ...proposalStore, revision: 8, proposals: [] },
    hasRealSnapshot: true,
    workflowStateLoading: false,
    workflowStateError: null,
  }),
  { status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_RECORD_MISSING" },
  "a missing exact Proposal cannot be substituted after the store advances",
);

const workItemMarkup = renderToStaticMarkup(
  <ProjectWorkflowDraftPanel
    project={project}
    workflowState={workflowStateWithSameRootCollision}
    secretarySourceFocus={workFocus}
    onRequestAction={() => undefined}
  />,
);
assert(
  workItemMarkup.includes("Exact work item") && !workItemMarkup.includes("Wrong same-id work item"),
  "WorkItem owner page consumes the exact workflow instead of a same-root same-id record",
);
for (const expected of [
  'data-secretary-source-focus-status="CONSUMED"',
  `data-secretary-source-owner="${M4_WORK_ITEM_SOURCE_OWNER_REF}"`,
  'data-secretary-source-object-id="same-object-id"',
  'data-secretary-source-revision="5"',
  `data-secretary-source-route-ref="${routeRef("a")}"`,
]) {
  assert(workItemMarkup.includes(expected), `exact WorkItem record exposes ${expected}`);
}

const proposalMarkup = renderToStaticMarkup(
  <JiaobanRegisteredProposalSourceFocus proposal={exactProposal} focus={proposalFocus} />,
);
for (const expected of [
  "Exact registered Proposal source",
  'data-secretary-source-focus-status="CONSUMED"',
  `data-secretary-source-owner="${M4_PROPOSAL_SOURCE_OWNER_REF}"`,
  'data-secretary-source-object-type="proposal_decision"',
  'data-secretary-source-object-id="same-object-id"',
  'data-secretary-source-revision="7"',
  `data-secretary-source-route-ref="${routeRef("b")}"`,
]) {
  assert(proposalMarkup.includes(expected), `exact non-test Proposal source exposes ${expected}`);
}
for (const forbiddenAction of ["允许并开始", "重新出方案", "按我说的改", "怎么跑"] as const) {
  assert(
    !proposalMarkup.includes(forbiddenAction),
    `registered Proposal read focus does not expose ${forbiddenAction}`,
  );
}
const nonRolloutProjectMarkup = renderToStaticMarkup(
  <ProjectJiaobanPanel
    project={project}
    sessions={[]}
    workflowState={workflowState}
    projectConsultationProposalStore={proposalStore}
    planAuthorizationStore={null}
    secretarySourceFocus={proposalFocus}
    onRequestAction={() => undefined}
    onOpenAgentSession={() => undefined}
  />,
);
assert(
  nonRolloutProjectMarkup.includes("Exact registered Proposal source")
    && nonRolloutProjectMarkup.includes('data-secretary-source-focus-status="CONSUMED"')
    && !nonRolloutProjectMarkup.includes("交办面需在桌面壳中打开")
    && !nonRolloutProjectMarkup.includes("这个项目现在用智能体直连"),
  "the full non-rollout Jiaoban entry consumes the exact registered Proposal before mounting browser workflow effects",
);
const missingProposalMarkup = renderToStaticMarkup(
  <JiaobanRegisteredProposalSourceFocus
    proposal={exactProposal}
    focus={{
      ...proposalFocus,
      target: {
        kind: "CONSULTATION_PROPOSAL",
        project_id: "project-a",
        workflow_id: "workflow-a",
        proposal_id: "proposal-missing",
        source_revision: "7",
      },
    }}
  />,
);
assert(
  missingProposalMarkup.includes('data-secretary-source-focus-status="FAILED"')
    && !missingProposalMarkup.includes("Exact registered Proposal source"),
  "a mismatched Proposal focus fails closed instead of exposing or consuming another record",
);
const forgedProposalBindingMarkup = renderToStaticMarkup(
  <JiaobanRegisteredProposalSourceFocus
    proposal={exactProposal}
    focus={{ ...proposalFocus, source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF }}
  />,
);
assert(
  forgedProposalBindingMarkup.includes('data-secretary-source-focus-status="FAILED"')
    && !forgedProposalBindingMarkup.includes("Exact registered Proposal source"),
  "a target-shaped Proposal with a forged top-level owner binding fails closed",
);

const nodeProcess = (globalThis as typeof globalThis & { process?: { cwd?: () => string } }).process;
if (!nodeProcess?.cwd) throw new Error("M4R04 frontend static receipt needs Node cwd");
const nodeFsSpecifier: string = "node:fs";
const { readFileSync } = await import(nodeFsSpecifier) as {
  readFileSync: (path: string, encoding: "utf8") => string;
};
const root = nodeProcess.cwd();
const appSource = readFileSync(`${root}/src/App.tsx`, "utf8");
const handlerStart = appSource.indexOf("const openSecretaryDeepLink");
const handlerEnd = appSource.indexOf("const operateSecretaryAction", handlerStart);
const routeHandler = appSource.slice(handlerStart, handlerEnd);
assert(handlerStart >= 0 && handlerEnd > handlerStart, "App source-route handler slice exists");
assert(routeHandler.includes("resolveSecretarySourceRoute({ source_route_ref"), "App sends route ref only");
const routeResolveCalls = [...routeHandler.matchAll(/resolveSecretarySourceRoute\(\{/g)].map((match) => match.index);
const ownerReadIndex = routeHandler.indexOf("loadWorkflowStateSnapshot()");
const clearQueryIndex = routeHandler.indexOf('setQuery("")');
const focusPublishIndex = routeHandler.indexOf("setSecretarySourceFocus(focus)");
assert(
  routeResolveCalls.length === 2
    && routeResolveCalls[0] < ownerReadIndex
    && ownerReadIndex < routeResolveCalls[1]
    && routeResolveCalls[1] < clearQueryIndex
    && clearQueryIndex < focusPublishIndex
    && routeHandler.includes("sameSecretarySourceRouteResolution(initialResolution, resolution)"),
  "ordinary owner reads are bracketed by two exact validations and clear unrelated search before focus",
);
assert(!routeHandler.includes('navigate("projects"'), "App does not guess Projects through generic navigation");
assert(routeHandler.includes("SECRETARY_SOURCE_ROUTE_RESOLUTION_FAILED"), "resolver failures use the fixed UI marker");
assert(routeHandler.includes("setActiveView(origin.view)"), "consumer failure restores the origin view");
const reloadStart = appSource.indexOf("async function reload()");
const homeReadStart = appSource.indexOf("void reloadSecretaryHome()", reloadStart);
const ownerBarrierStart = appSource.indexOf("setWorkflowStateLoading(true)", reloadStart);
assert(
  reloadStart >= 0 && ownerBarrierStart > reloadStart && ownerBarrierStart < homeReadStart,
  "the owner-read barrier rises before the independent Home read starts",
);

const shellSource = readFileSync(`${root}/src/components/WorkbenchShell.tsx`, "utf8");
const projectsSource = readFileSync(`${root}/src/views/ProjectsView.tsx`, "utf8");
const mainSource = readFileSync(`${root}/src/main.tsx`, "utf8");
const workflowTypesSource = readFileSync(`${root}/src/lib/types/workflow.ts`, "utf8");
const governancePanelsSource = readFileSync(
  `${root}/src/views/projects/ProjectWorkflowGovernancePanels.tsx`,
  "utf8",
);
const homeSource = readFileSync(`${root}/src/views/HomeView.tsx`, "utf8");
const boardSource = readFileSync(`${root}/src/components/SecretaryBoardView.tsx`, "utf8");
const proposalSource = readFileSync(`${root}/src/views/projects/ProjectJiaobanPanel.tsx`, "utf8");
for (const [source, token, label] of [
  [shellSource, "data-active-view", "active-view receipt"],
  [shellSource, "data-secretary-source-route-ref", "route-state capability binding"],
  [shellSource, "data-workbench-refresh", "ordinary global refresh selector"],
  [homeSource, "data-secretary-source-route-action", "Home source action selector"],
  [boardSource, "data-secretary-source-route-action", "Board source action selector"],
  [proposalSource, "data-secretary-source-focus-status", "Proposal consumption selector"],
] as const) {
  assert(source.includes(token), `${label} is present`);
}
assert(
  projectsSource.includes('status: "PENDING"')
    && projectsSource.includes('data-secretary-source-focus-status="PENDING"')
    && projectsSource.includes("workflowStateLoading"),
  "Projects keeps an exact source focus pending while its owner reads are in flight",
);
assert(
  appSource.includes("secretarySourceTargetReadCut")
    && appSource.includes("activeSecretarySourceReadCut?.snapshot ?? displaySnapshot")
    && appSource.includes("activeSecretarySourceReadCut?.workflow_state ?? workflowState")
    && appSource.includes("loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)")
    && appSource.includes("loadWorkflowStateSnapshot()")
    && appSource.includes("loadProjectConsultationProposalStore()")
    && appSource.includes("secretarySourceTargetReadCut?.attempt_id === secretarySourceFocus.attempt_id"),
  "route focus consumes an attempt-bound exact index/workflow/Proposal read cut after validation",
);
assert(
  appSource.includes("releaseConsumedSecretarySourceFocus(true)")
    && appSource.includes("releaseConsumedSecretarySourceFocus(false)")
    && appSource.includes('proposalFocus = secretarySourceFocus?.target.kind === "CONSULTATION_PROPOSAL"')
    && appSource.includes('origin?.attempt_id === secretarySourceFocus?.attempt_id')
    && appSource.includes('origin?.view !== "projects"')
    && appSource.includes('setActiveView(restoreOrigin ? origin.view : "home")')
    && appSource.includes('secretarySourceRouteState.source_route_ref !== secretarySourceFocus.source_route_ref')
    && appSource.includes('outcome.source_route_ref !== secretarySourceFocus.source_route_ref')
    && appSource.includes('outcome.target_kind !== secretarySourceFocus.target.kind')
    && projectsSource.includes("previousSelectedProjectRoot"),
  "explicit refresh/action release restores live reads without mounting non-rollout Proposal effects or changing the WorkItem tool",
);
assert(
  mainSource.includes('secretarySourceRoutePhase === "FAILED"')
    && mainSource.includes("routeStateMatchesClick")
    && mainSource.includes("shell.dataset.secretarySourceRouteRef === selected.action.source_route_ref")
    && mainSource.includes("m4r04_source_resolver_failed")
    && mainSource.includes("m4r04_source_consumer_failed"),
  "the actual-App observer preserves the bounded resolver/consumer failure layer instead of masking it as timeout",
);
for (const family of [
  "focus_pending_timeout",
  "focus_consumed_contract_timeout",
  "focus_consumer_missing_timeout",
  "resolver_response_invalid",
  "consumer_record_missing",
] as const) {
  assert(mainSource.includes(family), `actual-App diagnostics preserve bounded ${family}`);
}
for (const field of [
  "worker_acceptance_criteria",
  "control_core_acceptance_criteria",
  "supervisor_acceptance_criteria",
] as const) {
  assert(workflowTypesSource.includes(`${field}: string[]`), `${field} is required on the ordinary create DTO`);
  assert(mainSource.includes(`${field}: [`), `R04 ordinary Proposal creation supplies ${field}`);
  assert(governancePanelsSource.includes(`${field}:`), `the regular product builder supplies ${field}`);
}
