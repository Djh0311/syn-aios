// M2·交办 + 只读项目画布合一布局：离线 DOM / CSS 断言。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import type { JiaobanPhase } from "../src/views/projects/ProjectJiaobanPanel";
import { ProjectWorkflowCanvasView } from "../src/views/projects/ProjectWorkflowCanvasView";
import type { ProjectRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-merged-layout] ${message}`);
}

const noop = () => {};

function embeddedCanvasHtml(): string {
  return renderToStaticMarkup(
    <ProjectWorkflowCanvasView
      project={{ project_root: "/tmp/m2-layout-test", name: "M2 画布" } as ProjectRecord}
      sessions={[]}
      workflowState={null}
      blackboardCandidateStore={null}
      planAuthorizationStore={null}
      projectConsultationProposalStore={null}
      observationStore={null}
      memoryCandidateStore={null}
      formalMemoryStore={null}
      memoryLintStore={null}
      runtimeSessionAttention={[]}
      onRequestAction={noop}
      onOpenAgentSession={noop}
      renderSidePanel={() => <div>不应出现的详情抽屉</div>}
      embedded
    />,
  );
}

function html(phase: JiaobanPhase, initialHistoryOpen = false): string {
  return renderToStaticMarkup(
    <JiaobanMergedLayout
      phase={phase}
      history={<div>历史交办记录</div>}
      main={<button className="primary-button" type="button">允许并开始</button>}
      workflowPanel={<div>只读画布节点</div>}
      onOpenWorkflow={noop}
      initialHistoryOpen={initialHistoryOpen}
    />,
  );
}

// 1) 三个区域始终同在；不靠 phase 分支卸载交办或画布。
{
  const out = html("say");
  assert(out.includes("jiaoban-history-rail"), "历史收合条在");
  assert(out.includes('aria-label="交办主区"'), "交办主区在");
  assert(out.includes('aria-label="工作流运行视图"'), "只读画布区在");
  assert(out.includes("在工作流页打开"), "完整工作流跳转入口在");
}

// 2) 六态空间主次约定：执行期画布主导，其余交办主导。
for (const phase of ["say", "authorize", "binding", "running", "done", "blocked"] as const) {
  const out = html(phase);
  const expectedPrimary = phase === "running" ? "canvas" : "jiaoban";
  assert(out.includes(`data-phase="${phase}"`), `${phase} phase 标记在`);
  assert(out.includes(`data-primary="${expectedPrimary}"`), `${phase} 主区分配正确`);
}

// 3) 历史默认收合、可以显式展开，且主要人机门仍留在交办主区（在画布区之前）。
{
  const collapsed = html("authorize");
  assert(collapsed.includes('aria-expanded="false"'), "历史默认收合");
  assert(collapsed.includes('hidden=""'), "收合时历史抽屉隐藏");

  const expanded = html("authorize", true);
  assert(expanded.includes('aria-expanded="true"'), "历史可展开");
  assert(expanded.includes("is-history-open"), "展开时布局给历史抽屉让出宽度");
  assert(!expanded.includes('id="jiaoban-history-drawer" hidden=""'), "展开时历史抽屉可见");
  assert(expanded.indexOf("允许并开始") < expanded.indexOf("工作流运行视图"), "允许并开始留在交办区域而非画布区");
}

// 4) 根布局在 DOM 上明确收住最小宽高与溢出；详细 7:3 / 1:9 比例由 CSS class 管理。
{
  const out = html("running");
  assert(out.includes("min-width:0") && out.includes("min-height:0"), "根布局缩放契约在");
  assert(out.includes("jiaoban-merged-layout--running"), "运行期布局 CSS class 在");
}

// 5) 内嵌画布仍显示只读画布，但不带工作流页的编辑、运行与详情 HUD。
{
  const out = embeddedCanvasHtml();
  assert(out.includes("workflow-canvas--embedded") && out.includes("项目级工作流画布"), "内嵌只读画布在");
  for (const hiddenAction of ["新建工作流", "编辑工作流", "运行选中节点", "开始链", "工作流详情"]) {
    assert(!out.includes(hiddenAction), `内嵌画布不显示 ${hiddenAction}`);
  }
}

console.log("jiaoban-merged-layout: 5 组离线 DOM 断言全过");
