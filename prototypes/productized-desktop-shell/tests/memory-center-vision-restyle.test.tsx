import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { deriveMemoryManagementSummary } from "../src/lib/memoryCenter";
import { CandidateMemoryDetail, FormalMemoryDetail, MemoryLintFindingDetail } from "../src/views/memory/MemoryDetailPanels";
import { MemoryCenterView } from "../src/views/MemoryCenterView";
import { assert, findButtonByText } from "./helpers/offlineInteractionTestUtils";
import { memoryCenterCoreFixtures } from "./helpers/offlineMemoryCenterCoreFixtures";
import { memoryCenterGovernanceFixtures } from "./helpers/offlineMemoryCenterGovernanceFixtures";
import { offlineScenarioEnvironmentFixtures } from "./helpers/offlineScenarioEnvironmentFixtures";

const { project, workflowStateWithDerivedWorkflow } = offlineScenarioEnvironmentFixtures();
const { formalMemoryStore, memoryCaptureStore, memoryCandidateStore, observationStore } = memoryCenterCoreFixtures();
const { memoryLintStore, memoryEntityRelationStore } = memoryCenterGovernanceFixtures();

const globalRecord = {
  ...formalMemoryStore.records[0],
  memory_id: "mem:formal:offline:global",
  claim: "全局约束也必须保留人工确认。",
  scope: {
    ...formalMemoryStore.records[0].scope,
    scope_id: "scope:global:offline",
    scope_type: "global" as const,
    project_id: null,
    workflow_id: null,
  },
};
const formalStoreWithGlobal = {
  ...formalMemoryStore,
  records: [...formalMemoryStore.records, globalRecord],
};

const summary = deriveMemoryManagementSummary({
  projects: [project],
  workflowState: workflowStateWithDerivedWorkflow,
  formalMemoryStore: formalStoreWithGlobal,
  memoryCaptureStore,
  memoryCandidateStore,
  observationStore,
  memoryLintStore,
  memoryEntityRelationStore,
});
const candidate = summary.candidate_memories.find((item) => item.claim === "候选需要确认要求和风险提示。");
const formal = summary.formal_memories.find((item) => item.memory_id === "mem:formal:offline:included");
const lint = memoryLintStore.findings.find((item) => item.finding_id === "memlint:memory-center:blocking");

assert(candidate, "restyle fixture 缺少可采纳候选");
assert(formal, "restyle fixture 缺少正式记忆");
assert(lint, "restyle fixture 缺少 lint finding");

const markup = renderToStaticMarkup(
  <MemoryCenterView
    projects={[project]}
    workflowState={workflowStateWithDerivedWorkflow}
    formalMemoryStore={formalStoreWithGlobal}
    memoryCaptureStore={memoryCaptureStore}
    memoryCandidateStore={memoryCandidateStore}
    observationStore={observationStore}
    memoryLintStore={memoryLintStore}
    memoryEntityRelationStore={memoryEntityRelationStore}
    hasRealSnapshot
    onRequestAction={() => {}}
  />,
);

for (const group of ["candidate", "lint", "formal-project", "formal-global"]) {
  assert(markup.includes(`data-memory-group="${group}"`), `记忆中心缺少 ${group} 语义组`);
}
assert(markup.includes("候选 · 等你确认 <span class=\"n\">1"), "候选组必须使用日常收件箱的真数");
assert(markup.includes("要你看 <span class=\"n\">2"), "lint 组只能统计 open 的阻断 / 待复核 finding");
assert(markup.includes("正式 · 按项目 <span class=\"n\">2"), "非全局正式记忆不可从项目组静默丢失");
assert(markup.includes("正式 · 全局 <span class=\"n\">1"), "全局正式记忆必须进入独立语义组");
assert((markup.match(/data-memory-detail-kind=/g) ?? []).length === 1, "初始页面只能展示一个右栏详情 face");
assert(markup.includes('data-memory-detail-kind="candidate"'), "候选存在时应以候选作为单选默认详情");
assert(!markup.includes('data-memory-detail-kind="formal"'), "候选默认详情时不应叠加正式详情");
assert(!markup.includes('data-memory-detail-kind="lint"'), "候选默认详情时不应叠加 lint 详情");
assert(markup.includes("记住（转正式）"), "已确认且未采纳候选必须保留转正式动作");
assert(markup.includes("出方案会带上"), "真实快照存在时必须展示任务包召回真数");
assert(!markup.includes("daily-memory-candidate-inbox"), "重排后候选组不应保留独立收件箱卡片");

const candidateClicks: string[] = [];
const candidateElement = (
  <CandidateMemoryDetail
    item={candidate}
    canAdopt
    canDiscard
    onAdopt={() => candidateClicks.push("adopt")}
    onDiscard={() => candidateClicks.push("discard")}
  />
);
const adoptButton = findButtonByText(candidateElement, "记住（转正式）");
const discardButton = findButtonByText(candidateElement, "不要");
assert(adoptButton && discardButton, "已确认候选必须保留采纳和暂不处理的真实入口");
(adoptButton.props?.onClick as (() => void))();
(discardButton.props?.onClick as (() => void))();
assert(candidateClicks.join(",") === "adopt,discard", "候选详情动作必须接到传入的真实决策回调");
const candidateMarkup = renderToStaticMarkup(
  <CandidateMemoryDetail item={candidate} sourceRefs={memoryCandidateStore.candidates[0].source_refs} />,
);
assert(candidateMarkup.includes("source:memory-center:candidate:001"), "候选详情必须保留可核对的 source_ref_id");
assert(!candidateMarkup.includes("和现有记忆"), "没有 open lint finding 时不得展示空的现有记忆关系行");
const candidateWithLintMarkup = renderToStaticMarkup(
  <CandidateMemoryDetail item={candidate} hasOpenLintFinding sourceRefs={memoryCandidateStore.candidates[0].source_refs} />,
);
assert(candidateWithLintMarkup.includes("和现有记忆"), "存在 open lint finding 时必须展示真实的现有记忆关系行");

const lintMarkup = renderToStaticMarkup(
  <MemoryLintFindingDetail
    busyKind={null}
    error={null}
    finding={lint}
    targetMemory={formal}
    onLifecycleAction={() => {}}
  />,
);
assert(lintMarkup.includes("改写提案") && lintMarkup.includes("废弃"), "有正式目标的 lint 只能暴露既有 M9 动作");
assert(!lintMarkup.includes("保留"), "lint 详情不得伪造没有实现的保留决定");

const sourceOnlyLint = {
  ...lint,
  finding_id: "memlint:memory-center:source-only",
  target_memory_id: null,
  source_kind: "memory_record" as const,
  source_id: formal.memory_id,
};
const sourceOnlyLintMarkup = renderToStaticMarkup(
  <MemoryCenterView
    projects={[project]}
    workflowState={workflowStateWithDerivedWorkflow}
    formalMemoryStore={formalStoreWithGlobal}
    memoryCaptureStore={memoryCaptureStore}
    memoryCandidateStore={{ ...memoryCandidateStore, candidates: [] }}
    observationStore={observationStore}
    memoryLintStore={{ ...memoryLintStore, findings: [sourceOnlyLint] }}
    hasRealSnapshot
    onRequestAction={() => {}}
  />,
);
assert(sourceOnlyLintMarkup.includes("要你看 <span class=\"n\">1"), "source_id 关联的 lint 必须进入对应项目的待处理组");
assert(sourceOnlyLintMarkup.includes('data-memory-detail-kind="lint"'), "没有候选时 source_id lint 必须成为可选详情");
assert(sourceOnlyLintMarkup.includes("改写提案") && sourceOnlyLintMarkup.includes("废弃"), "source_id lint 必须解析到正式记忆的 M9 动作");

const formalFallbackMarkup = renderToStaticMarkup(
  <MemoryCenterView
    projects={[project]}
    workflowState={workflowStateWithDerivedWorkflow}
    formalMemoryStore={formalStoreWithGlobal}
    memoryCaptureStore={memoryCaptureStore}
    memoryCandidateStore={{ ...memoryCandidateStore, candidates: [] }}
    observationStore={observationStore}
    memoryLintStore={{ ...memoryLintStore, findings: [] }}
    hasRealSnapshot
    onRequestAction={() => {}}
  />,
);
assert((formalFallbackMarkup.match(/data-memory-detail-kind=/g) ?? []).length === 1, "正式默认态也必须只渲染一个详情 face");
assert(formalFallbackMarkup.includes('data-memory-detail-kind="formal"'), "候选和 lint 均为空时必须回落到正式记忆详情");

const noLintCandidateMarkup = renderToStaticMarkup(
  <MemoryCenterView
    projects={[project]}
    workflowState={workflowStateWithDerivedWorkflow}
    formalMemoryStore={formalStoreWithGlobal}
    memoryCaptureStore={memoryCaptureStore}
    memoryCandidateStore={memoryCandidateStore}
    observationStore={observationStore}
    memoryLintStore={{ ...memoryLintStore, findings: [] }}
    hasRealSnapshot
    onRequestAction={() => {}}
  />,
);
assert(noLintCandidateMarkup.includes('data-memory-detail-kind="candidate"'), "无 lint 时候选仍应作为默认详情");
assert(!noLintCandidateMarkup.includes("和现有记忆"), "MemoryCenter 不得把无 open lint 的占位文案显示成关系事实");

const reviewClicks: string[] = [];
const reviewElement = (
  <CandidateMemoryDetail
    item={candidate}
    canConfirm
    canDiscard
    canReject
    onConfirm={() => reviewClicks.push("confirm")}
    onDiscard={() => reviewClicks.push("discard")}
    onReject={() => reviewClicks.push("reject")}
  />
);
const confirmButton = findButtonByText(reviewElement, "属实（确认）");
const rejectButton = findButtonByText(reviewElement, "不要");
assert(confirmButton && rejectButton, "待复核候选必须保留先确认、暂不处理和拒绝的两步入口");
(confirmButton.props?.onClick as (() => void))();
(rejectButton.props?.onClick as (() => void))();
assert(reviewClicks.join(",") === "confirm,reject", "待复核候选操作必须接到 Memory Center 候选详情的真实回调");

const formalMarkup = renderToStaticMarkup(
  <FormalMemoryDetail item={formal} busyKind={null} error={null} onLifecycleAction={() => {}} />,
);
assert(formalMarkup.includes("来龙去脉"), "正式详情必须保留来源、版本、审计形成的时间线");
assert(formalMarkup.includes("正式记忆建立"), "正式详情时间线必须保留 record.created_at 的建立事实");
assert(formalMarkup.includes("memory_record_created"), "正式详情时间线必须保留真实审计事件");
