// 刀A·口供上脸 + 黄牌：离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// 覆盖三态（全绿 / 含 partial / 含缺口供）+ 执行态优先 + report_warning + 无 steps 零渲染 + 词表不露黑话。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanStepReportList,
  jiaobanDoneTitle,
  stepReportFlag,
  countYellowFlags,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { DirectorChainStep } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[report-on-face] ${message}`);
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

function html(steps: DirectorChainStep[]): string {
  return renderToStaticMarkup(<JiaobanStepReportList steps={steps} />);
}

// 1) 全绿：done → 绿徽章，无黄牌，标题不变。
{
  const steps = [
    step({ planned_task_id: "a", title: "搭骨架", report_summary: "建了脚手架（done）", report_status: "done" }),
    step({ planned_task_id: "b", title: "接业务", report_summary: "接好了（done）", report_status: "done" }),
  ];
  const out = html(steps);
  assert(out.includes("搭骨架") && out.includes("接业务"), "全绿：应显示任务标题");
  assert(out.includes("建了脚手架（done）"), "全绿：应显示自述一句");
  assert(out.includes("自述：做好了"), "全绿：done → 绿徽章文案");
  assert(!out.includes("⚠"), "全绿：不应有黄牌");
  assert(countYellowFlags(steps) === 0, "全绿：黄牌数 0");
  assert(jiaobanDoneTitle(steps) === "✓ 做好了", "全绿：标题不变");
}

// 2) 含 partial：黄牌 + 标题联动。
{
  const steps = [
    step({ planned_task_id: "a", title: "搭骨架", report_summary: "建好了", report_status: "done" }),
    step({ planned_task_id: "b", title: "接业务", report_summary: "只做了一半", report_status: "partial" }),
  ];
  const out = html(steps);
  assert(out.includes("⚠"), "partial：应出黄牌 ⚠");
  assert(out.includes("自述：没干完"), "partial：黄牌文案「自述：没干完」");
  assert(countYellowFlags(steps) === 1, "partial：黄牌数 1");
  assert(jiaobanDoneTitle(steps) === "✓ 做好了（有 1 项要看一眼）", "partial：标题联动");
}

// 3) 含缺口供（没交汇报）：黄牌「没交汇报」+ 标题联动。
{
  const steps = [
    step({ planned_task_id: "a", title: "搭骨架", report_summary: "建好了", report_status: "done" }),
    step({ planned_task_id: "b", title: "接业务", report_summary: null, report_status: null }),
  ];
  const out = html(steps);
  assert(out.includes("没交汇报"), "缺口供：黄牌「没交汇报」");
  assert(countYellowFlags(steps) === 1, "缺口供：黄牌数 1");
  assert(jiaobanDoneTitle(steps).includes("有 1 项"), "缺口供：标题联动");
}

// 4) 执行态优先：failed/skipped 不被口供徽章覆盖、不计入黄牌 N。
{
  const failStep = step({ planned_task_id: "f", title: "崩的", state: "failed", report_status: null });
  assert(stepReportFlag(failStep).tone === "red", "failed → 红（执行态优先于自述）");
  assert(stepReportFlag(failStep).kind === "fail", "failed 不算黄牌");
  const skipStep = step({ planned_task_id: "s", title: "跳的", state: "skipped" });
  assert(stepReportFlag(skipStep).tone === "gray", "skipped → 灰");
  assert(countYellowFlags([failStep, skipStep]) === 0, "failed/skipped 不计入黄牌 N");
}

// 5) report_warning（落库失败）优先：黄牌显人话，哪怕 status=done。
{
  const s = step({
    planned_task_id: "w",
    title: "落库失败的",
    report_summary: "做了（done）",
    report_status: "done",
    report_warning: "任务「X」报文落库失败：磁盘满",
  });
  assert(stepReportFlag(s).kind === "yellow", "有 warning → 黄牌（即便 status done）");
  assert(stepReportFlag(s).badge.includes("落库失败"), "黄牌显 warning 人话");
}

// 6) 无 steps → 零渲染。
{
  assert(html([]) === "", "无 steps → 不渲染（零回退）");
}

// 7) 词表：不露内部 id / 不露黑话「口供」。
{
  const steps = [
    step({ planned_task_id: "secret-planned-id-123", title: "任务A", report_summary: "s", report_status: "done" }),
  ];
  const out = html(steps);
  assert(!out.includes("secret-planned-id-123"), "词表：不露 planned_task_id");
  assert(!out.includes("口供"), "词表：界面不露内部黑话「口供」");
}

console.log("report-on-face-yellow-flag: 7 组离线 DOM 断言全过");
