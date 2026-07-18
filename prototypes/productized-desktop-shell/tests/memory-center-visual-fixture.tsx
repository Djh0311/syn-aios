import { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { MemoryCenterView } from "../src/views/MemoryCenterView";
import "../src/styles.css";
import "../src/views/memory/memoryCenter.css";
import { memoryCenterCoreFixtures } from "./helpers/offlineMemoryCenterCoreFixtures";
import { memoryCenterGovernanceFixtures } from "./helpers/offlineMemoryCenterGovernanceFixtures";
import { offlineScenarioEnvironmentFixtures } from "./helpers/offlineScenarioEnvironmentFixtures";

const { project, workflowStateWithDerivedWorkflow } = offlineScenarioEnvironmentFixtures();
const { formalMemoryStore, memoryCaptureStore, memoryCandidateStore, observationStore } = memoryCenterCoreFixtures();
const { memoryLintStore, memoryEntityRelationStore } = memoryCenterGovernanceFixtures();
const seedCandidate = memoryCandidateStore.candidates[0];
const seedFormal = formalMemoryStore.records[0];
const seedLint = memoryLintStore.findings[0];

const visualCandidates = Array.from({ length: 18 }, (_, index) => ({
  ...seedCandidate,
  candidate_id: `memcand:visual:${index}`,
  candidate_key: `memcand:v1:visual:${index}`,
  claim: `候选记忆 ${index + 1}：控制边界需要人工确认。`,
  body: "这是用于容器内滚量尺的长候选说明。它只存在于浏览器夹具，不写入产品数据。".repeat(32),
  status: index % 2 === 0 ? "candidate_confirmed" as const : "candidate_needs_review" as const,
  adoption: null,
  updated_at: `2026-06-06T00:${String(index).padStart(2, "0")}:00Z`,
}));
const visualProjectFormals = Array.from({ length: 12 }, (_, index) => ({
  ...seedFormal,
  memory_id: index === 0 ? seedFormal.memory_id : `mem:formal:visual:project:${index}`,
  claim: `项目正式记忆 ${index + 1}：接口控制核心边界。`,
  updated_at: `2026-06-07T00:${String(index).padStart(2, "0")}:00Z`,
}));
const visualGlobalFormals = Array.from({ length: 8 }, (_, index) => ({
  ...seedFormal,
  memory_id: `mem:formal:visual:global:${index}`,
  claim: `全局正式记忆 ${index + 1}：变更前保留人工确认。`,
  scope: {
    ...seedFormal.scope,
    scope_id: `scope:visual:global:${index}`,
    scope_type: "global" as const,
    project_id: null,
    workflow_id: null,
  },
  updated_at: `2026-06-08T00:${String(index).padStart(2, "0")}:00Z`,
}));
const visualLintFindings = Array.from({ length: 14 }, (_, index) => ({
  ...seedLint,
  finding_id: `memlint:visual:${index}`,
  severity: index % 2 === 0 ? "blocking" as const : "needs_review" as const,
  summary: `检查发现 ${index + 1}：来源与当前正式记忆需要人工复核。`,
  claim: `候选记忆 ${index + 1}：控制边界需要人工确认。`,
  target_memory_id: visualProjectFormals[index % visualProjectFormals.length].memory_id,
  target_candidate_key: visualCandidates[index % visualCandidates.length].candidate_key,
  updated_at: `2026-06-09T00:${String(index).padStart(2, "0")}:00Z`,
}));

type ScrollMetric = {
  clientHeight: number;
  clientWidth: number;
  scrollHeight: number;
  scrollWidth: number;
  canScrollY: boolean;
  movedOnProbe: boolean;
};

function FixtureMetrics() {
  const [metrics, setMetrics] = useState<Record<string, ScrollMetric | number> | null>(null);

  useEffect(() => {
    const readMetric = (selector: string): ScrollMetric => {
      const element = document.querySelector<HTMLElement>(selector);
      if (!element) throw new Error(`fixture missing ${selector}`);
      const initialScrollTop = element.scrollTop;
      const canScrollY = element.scrollHeight > element.clientHeight;
      if (canScrollY) element.scrollTop = Math.min(40, element.scrollHeight - element.clientHeight);
      const movedOnProbe = element.scrollTop > initialScrollTop;
      element.scrollTop = initialScrollTop;
      return {
        clientHeight: element.clientHeight,
        clientWidth: element.clientWidth,
        scrollHeight: element.scrollHeight,
        scrollWidth: element.scrollWidth,
        canScrollY,
        movedOnProbe,
      };
    };
    const collect = () => {
      setMetrics({
        documentElement: readMetric("html"),
        body: readMetric("body"),
        stage: readMetric("#fixture-stage"),
        stagePad: readMetric(".stage-pad.memory-center"),
        list: readMetric(".mlist"),
        main: readMetric(".mmain"),
      });
    };
    const frame = requestAnimationFrame(collect);
    window.addEventListener("resize", collect);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("resize", collect);
    };
  }, []);

  return <pre hidden id="fixture-metrics">{metrics ? JSON.stringify(metrics) : "pending"}</pre>;
}

function FixtureFaceScenario() {
  const [result, setResult] = useState("default");

  useEffect(() => {
    const face = new URLSearchParams(window.location.search).get("face");
    const scenarioByFace: Record<string, { selector: string; expected: string }> = {
      candidate: { selector: '[data-memory-row-kind="candidate"]', expected: "candidate" },
      lint: { selector: '[data-memory-row-kind="lint"]', expected: "lint" },
      formal: { selector: '[data-memory-row-kind="formal"]', expected: "formal" },
      governance: { selector: ".memory-governance-trigger", expected: "governance" },
      confirmedCandidate: { selector: '[data-memory-row-id="memcand:v1:visual:16"]', expected: "candidate" },
    };
    const scenario = face ? scenarioByFace[face] : null;
    if (!scenario) return;
    let verificationFrame = 0;
    const clickFrame = requestAnimationFrame(() => {
      document.querySelector<HTMLButtonElement>(scenario.selector)?.click();
      verificationFrame = requestAnimationFrame(() => {
        const actual = document.querySelector("[data-memory-detail-kind]")?.getAttribute("data-memory-detail-kind") ?? "missing";
        setResult(actual === scenario.expected ? `pass:${actual}` : `fail:expected-${scenario.expected}:actual-${actual}`);
      });
    });
    return () => {
      cancelAnimationFrame(clickFrame);
      cancelAnimationFrame(verificationFrame);
    };
  }, []);

  return <output hidden id="fixture-face-result">{result}</output>;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <div id="fixture-shell">
    <main className="stage" id="fixture-stage">
      <MemoryCenterView
        projects={[project]}
        workflowState={workflowStateWithDerivedWorkflow}
        formalMemoryStore={{
          ...formalMemoryStore,
          records: [...visualProjectFormals, ...visualGlobalFormals],
        }}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={{ ...memoryCandidateStore, candidates: visualCandidates }}
        observationStore={observationStore}
        memoryLintStore={{ ...memoryLintStore, findings: visualLintFindings }}
        memoryEntityRelationStore={memoryEntityRelationStore}
        hasRealSnapshot
        onRequestAction={() => {}}
      />
    </main>
    <FixtureFaceScenario />
    <FixtureMetrics />
  </div>,
);
