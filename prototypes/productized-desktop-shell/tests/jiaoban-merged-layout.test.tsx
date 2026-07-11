// M2·交办 + 只读项目画布合一布局：离线 DOM 断言。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanHistoryOverlay, JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import type { JiaobanPhase } from "../src/views/projects/ProjectJiaobanPanel";
import { ProjectWorkflowCanvasView } from "../src/views/projects/ProjectWorkflowCanvasView";
import type { ProjectRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-merged-layout] ${message}`);
}

const noop = () => {};
const removedLayoutHook = (...parts: string[]) => parts.join("");

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

// 2) 六态只切内容，不再切布局主次；左栏与画布始终同宽度规则。
for (const phase of ["say", "authorize", "binding", "running", "done", "blocked"] as const) {
  const out = html(phase);
  assert(out.includes('class="jiaoban-merged-layout"'), `${phase} 应使用同一左右布局壳`);
  assert(!out.includes(removedLayoutHook("data-", "primary")), `${phase} 不应再驱动主区切换`);
  assert(!out.includes(`jiaoban-merged-layout--${phase}`), `${phase} 不应再生成相位布局 class`);
}

// 3) 历史默认收合；展开为覆盖层，既不挤网格也能点外部收回。
{
  const collapsed = html("authorize");
  assert(collapsed.includes('aria-expanded="false"'), "历史默认收合");
  assert(!collapsed.includes("jiaoban-history-overlay"), "收合时不渲染历史覆盖层");

  const expanded = html("authorize", true);
  assert(expanded.includes('aria-expanded="true"'), "历史可展开");
  assert(expanded.includes("jiaoban-history-overlay"), "展开时历史应为悬浮覆盖层");
  assert(!expanded.includes(removedLayoutHook("is-history-", "open")), "展开历史不得挤压主布局");
  assert(expanded.includes('id="jiaoban-history-drawer"'), "展开时历史抽屉可见");
  assert(expanded.indexOf("允许并开始") < expanded.indexOf("工作流运行视图"), "允许并开始留在交办区域而非画布区");

  const dismissed: string[] = [];
  const overlay = JiaobanHistoryOverlay({ history: <div>历史</div>, onDismiss: () => dismissed.push("dismiss") });
  const onClick = (overlay as unknown as { props: { onClick?: (event: unknown) => void } }).props.onClick;
  const backdrop = {};
  onClick?.({ target: backdrop, currentTarget: backdrop });
  onClick?.({ target: {}, currentTarget: backdrop });
  assert(dismissed.length === 1, "只点覆盖层空白处才收起历史");
}

// 4) 左右区域的 DOM 顺序固定：交办控制栏在左、画布在右，且不保留旧相位布局 class。
{
  const out = html("running");
  assert(out.includes("min-width:0") && out.includes("min-height:0"), "根布局缩放契约在");
  assert(
    out.indexOf('aria-label="交办主区"') < out.indexOf('aria-label="工作流运行视图"'),
    "交办控制栏应先于右侧画布",
  );
  assert(!out.includes(removedLayoutHook("jiaoban-merged-layout--", "running")), "旧运行相位比例 class 应删除");
}

// 5) 内嵌画布仍显示只读画布，但不带工作流页的编辑、运行与详情 HUD。
{
  const out = embeddedCanvasHtml();
  assert(out.includes("workflow-canvas--embedded") && out.includes("项目级工作流画布"), "内嵌只读画布在");
  for (const hiddenAction of ["新建工作流", "编辑工作流", "运行选中节点", "开始链", "工作流详情"]) {
    assert(!out.includes(hiddenAction), `内嵌画布不显示 ${hiddenAction}`);
  }
}

console.log("jiaoban-merged-layout: 5 组离线 DOM / 覆盖层 / 左右分栏断言全过");
