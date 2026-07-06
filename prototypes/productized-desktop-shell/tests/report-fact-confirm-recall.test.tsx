// 刀B·事实确认离线 DOM 断言：绿✓行有[属实,沉淀]、黄牌行没有、点后「已沉淀 ✓」、
// buildFactMemoryCandidate 构造（候选≠正式·最保守合法档）。renderToStaticMarkup·同现有 harness。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanStepReportList,
  buildFactMemoryCandidate,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { DirectorChainStep } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[report-fact-confirm] ${message}`);
  }
}

function step(overrides: Partial<DirectorChainStep>): DirectorChainStep {
  return {
    planned_task_id: overrides.planned_task_id ?? "t1",
    title: overrides.title ?? "任务",
    state: overrides.state ?? "completed",
    report_summary: overrides.report_summary,
    report_warning: overrides.report_warning,
    report_status: overrides.report_status,
  };
}

const greenA = step({ planned_task_id: "a", title: "搭骨架", report_summary: "建了脚手架（done）", report_status: "done" });
const partialB = step({ planned_task_id: "b", title: "接业务", report_summary: "只做了一半", report_status: "partial" });
const noSummaryC = step({ planned_task_id: "c", title: "写用例", report_summary: null, report_status: null });

// 1) 绿✓且有自述 → 有 [属实,沉淀]；黄牌 / 无自述 → 没有。
{
  const html = renderToStaticMarkup(
    <JiaobanStepReportList steps={[greenA, partialB, noSummaryC]} onConfirmFact={() => {}} confirmedTaskIds={new Set()} />,
  );
  assert(html.includes("属实，沉淀"), "绿✓行有 [属实,沉淀] 按钮");
  const btnCount = html.split("jiaoban-fact-btn").length - 1;
  assert(btnCount === 1, `只绿✓且有自述的行有按钮（黄牌/无自述没有），实际 ${btnCount}`);
}

// 2) 已沉淀（confirmedTaskIds 含该 id）→ 显「已沉淀 ✓」、不再显按钮（防重复点）。
{
  const html = renderToStaticMarkup(
    <JiaobanStepReportList steps={[greenA]} onConfirmFact={() => {}} confirmedTaskIds={new Set(["a"])} />,
  );
  assert(html.includes("已沉淀"), "已沉淀行显示 ✓");
  assert(!html.includes("jiaoban-fact-btn"), "已沉淀不再显按钮");
}

// 3) 无 onConfirmFact（老数据/无 ctx）→ 整列表不带按钮，零回退。
{
  const html = renderToStaticMarkup(<JiaobanStepReportList steps={[greenA]} />);
  assert(!html.includes("属实"), "无 onConfirmFact → 不显按钮");
}

// 4) buildFactMemoryCandidate 构造：候选≠正式、最保守合法档、claim=自述、带项目/任务锚。
{
  const input = buildFactMemoryCandidate(greenA, {
    projectRoot: "/Users/yoyi/proj",
    projectId: "project:users-yoyi-proj",
    workflowId: "wf-1",
  });
  assert(input.memory_type === "workflow_summary", "memory_type=workflow_summary（核查后合法档）");
  assert(input.risk_level === "low", "risk_level=low（最保守）");
  assert(input.sensitive_level === "project", "sensitive_level=project");
  assert(input.scope.scope_type === "project", "scope=project");
  assert(input.scope.model_export_policy === "local_only", "scope 不外泄（local_only）");
  assert(input.claim === "建了脚手架（done）", "claim=自述");
  assert(input.generated_by_role === "user", "generated_by_role=user（用户确认）");
  assert(input.generated_from === "explicit_user_confirmation", "generated_from=用户确认");
  assert(input.requires_user_confirmation === true, "requires_user_confirmation");
  assert(input.source_refs.length === 1 && input.source_refs[0].anchor === "a", "source_ref 带任务锚");
  assert(input.project_id === "project:users-yoyi-proj", "project_id 透传");
  assert(input.scope.project_id === "project:users-yoyi-proj", "scope.project_id 与召回 filter 同口径");
}

console.log("report-fact-confirm-recall: 4 组离线 DOM 断言全过");
