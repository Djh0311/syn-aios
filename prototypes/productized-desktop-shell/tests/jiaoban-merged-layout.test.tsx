// 修宪三栏(2026-07-14·交互正本 §四.2)：左工作历史独立栏 + 中交办主卡 + 右画布动态宽。离线 DOM 断言。
// 本文件原锁 M2 旧形态(32px rail + 历史悬浮覆盖层)，修宪后按新语义更新；组数不减。
import type { ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import type { JiaobanPhase } from "../src/views/projects/ProjectJiaobanPanel";
import { buildJiaobanArtifactCanvasViews } from "../src/views/projects/jiaoban/JiaobanArtifactViews";
import { ProjectWorkflowCanvasView } from "../src/views/projects/ProjectWorkflowCanvasView";
import type { ProjectRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-merged-layout] ${message}`);
}

function openingTagBefore(html: string, marker: string, requiredAttribute: string): string {
  const markerIndex = html.indexOf(marker);
  assert(markerIndex >= 0, `应找到正文标记：${marker}`);
  const attributeIndex = html.lastIndexOf(requiredAttribute, markerIndex);
  assert(attributeIndex >= 0, `${marker} 应属于 ${requiredAttribute}`);
  const tagStart = html.lastIndexOf("<", attributeIndex);
  const tagEnd = html.indexOf(">", attributeIndex);
  assert(tagStart >= 0 && tagEnd >= 0, `${marker} 的容器起始标签应完整`);
  return html.slice(tagStart, tagEnd + 1);
}

function tabOpeningTag(html: string, label: string): string {
  const closing = `>${label}</button>`;
  const labelIndex = html.indexOf(closing);
  assert(labelIndex >= 0, `应找到「${label}」视图 tab`);
  const tagStart = html.lastIndexOf("<button", labelIndex);
  assert(tagStart >= 0, `「${label}」应由 button 承载`);
  return html.slice(tagStart, labelIndex + 1);
}

function isHiddenTag(openingTag: string): boolean {
  return /\shidden(?:=""|(?=[\s>]))/.test(openingTag);
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

function html(phase: JiaobanPhase, initialHistoryOpen = true, previewCanvas: ReactNode = null): string {
  return renderToStaticMarkup(
    <JiaobanMergedLayout
      phase={phase}
      history={<div>历史交办记录</div>}
      main={<button className="primary-button" type="button">允许并开始</button>}
      previewCanvas={previewCanvas}
      workflowPanel={<div>只读画布节点</div>}
      onOpenWorkflow={noop}
      initialHistoryOpen={initialHistoryOpen}
    />,
  );
}

// 1) 三栏始终同在；不靠 phase 分支卸载任何一栏。
{
  const out = html("say");
  assert(out.includes("jiaoban-history-column"), "工作历史独立栏在");
  assert(out.includes('aria-label="交办主区"'), "交办主区在");
  assert(out.includes('aria-label="工作流运行视图"'), "只读画布区在");
  // 07-15 走查修·原断言锁旧形态:定稿窄条=纯一句话——header/跳转钮只在画布宽态渲染
  // (180px 窄条塞 header 真机竖排成「工作/流进/度」一字一行,按钮顶爆栏宽)。
  assert(!out.includes("在工作流页打开"), "窄条态不渲染工作流跳转钮");
  assert(!out.includes("工作流进度"), "窄条态不渲染画布标题");
  const wide = html("running");
  assert(wide.includes("在工作流页打开"), "画布宽态才给完整工作流跳转入口");
  assert(wide.includes("正在执行"), "宽态无预演图时保留运行标题");
}

// 2) 七态只切内容，不再切布局主次：同一个三栏壳，且宽度只由「有没有工序图」驱动，不由相位直接驱动。
for (const phase of ["say", "authorize", "binding", "running", "done", "waiting_decision", "blocked"] as const) {
  const out = html(phase);
  assert(out.includes('class="jiaoban-merged-layout'), `${phase} 应使用同一三栏布局壳`);
  assert(!out.includes(removedLayoutHook("data-", "primary")), `${phase} 不应再驱动主区切换`);
  assert(!out.includes(`jiaoban-merged-layout--${phase}`), `${phase} 不应再生成相位布局 class`);
}

// 3) 修宪：历史=独立栏，默认展开、内容占栏内空间；可一键收起成窄条。不再是悬浮覆盖层。
{
  const expanded = html("authorize");
  assert(expanded.includes('aria-expanded="true"'), "历史独立栏默认展开");
  assert(expanded.includes('id="jiaoban-history-drawer"'), "展开时历史内容在栏内");
  assert(expanded.includes("历史交办记录"), "展开时历史内容真渲染");
  assert(!expanded.includes(removedLayoutHook("jiaoban-history-", "overlay")), "历史不得再是悬浮覆盖层");
  assert(!expanded.includes(removedLayoutHook("jiaoban-history-", "rail")), "32px 收合条形态应退役");
  assert(expanded.indexOf("允许并开始") < expanded.indexOf("工作流运行视图"), "允许并开始留在交办区域而非画布区");

  const collapsed = html("authorize", false);
  assert(collapsed.includes('aria-expanded="false"'), "历史可一键收起");
  assert(collapsed.includes("is-history-collapsed"), "收起时布局收窄成窄条");
  assert(!collapsed.includes("历史交办记录"), "收起时不渲染历史内容");
  assert(collapsed.includes('aria-label="交办主区"'), "收起历史后交办主区仍在");
}

// 3b) 画布动态宽：有工序图可画=宽；说态还没方案=收窄成提示条（用户 2026-07-15 拍「有图才宽」，保住 M1 节点选会话）。
{
  const noCanvas = html("say");
  assert(noCanvas.includes("is-canvas-hint"), "说态无工序图时画布收窄成提示条");
  assert(!noCanvas.includes("is-canvas-wide"), "说态画布不应占宽");
  assert(noCanvas.includes("出方案后，这里会出现工序图预演。"), "收窄态给出下一步（宪法 D7 空态必答下一步）");

  const withCanvas = html("authorize", true, <div>预演工序图</div>);
  assert(withCanvas.includes("is-canvas-wide"), "批态有预演图时画布变宽");
  assert(!withCanvas.includes("is-canvas-hint"), "有图时不应还是提示条");
  assert(withCanvas.includes("预演工序图"), "批态预演图真渲染（M1 节点选会话的落点）");
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

// 7b) 右区=信息展开面：方案/交货实体卡随视图常驻，非激活正文 hidden；有视图时 say 也展开右区。
{
  const authorizeViews = buildJiaobanArtifactCanvasViews({
    phase: "authorize",
    selectedHistoryId: null,
    activeViewKey: "proposal",
    proposalInteractive: true,
    proposalContent: <div>方案实体卡与批准动作</div>,
    deliveryContent: null,
    graphContent: <div>方案工序图正文</div>,
    governanceContent: <div>方案治理全文</div>,
    howRunContent: <div>方案运行配置</div>,
  });
  assert(authorizeViews, "authorize 有方案时应组装右区视图");
  const authorizeOut = renderToStaticMarkup(
    <JiaobanMergedLayout
      phase="authorize"
      history={<div>历史</div>}
      main={<div>纯对话</div>}
      previewCanvas={null}
      workflowPanel={null}
      onOpenWorkflow={noop}
      canvasViews={authorizeViews}
      activeCanvasView="proposal"
      onCanvasViewChange={noop}
    />,
  );
  assert(authorizeOut.includes("jiaoban-canvas-view-tabs"), "authorize 多视图应渲染切换 chips");
  assert((authorizeOut.match(/role="tabpanel"/g) ?? []).length === 4, "authorize 四视图正文应常驻为四个 tabpanel");
  const authorizeLabels = ["方案", "工序图", "治理保证", "怎么跑"];
  for (let index = 1; index < authorizeLabels.length; index += 1) {
    assert(
      authorizeOut.indexOf(`>${authorizeLabels[index - 1]}</button>`) <
        authorizeOut.indexOf(`>${authorizeLabels[index]}</button>`),
      `authorize tab 顺序应为 ${authorizeLabels.join(" / ")}`,
    );
  }
  assert(tabOpeningTag(authorizeOut, "方案").includes('aria-selected="true"'), "方案 tab 应默认激活");
  for (const label of ["工序图", "治理保证", "怎么跑"]) {
    assert(tabOpeningTag(authorizeOut, label).includes('aria-selected="false"'), `${label} tab 应为非激活`);
  }
  assert(
    !isHiddenTag(openingTagBefore(authorizeOut, "方案实体卡与批准动作", 'role="tabpanel"')),
    "激活方案正文不得 hidden",
  );
  for (const marker of ["方案工序图正文", "方案治理全文", "方案运行配置"]) {
    assert(isHiddenTag(openingTagBefore(authorizeOut, marker, 'role="tabpanel"')), `${marker} 应常驻但 hidden`);
  }
  assert(authorizeOut.includes("在工作流页打开"), "authorize 宽态完整工作流跳转钮仍在");

  const doneViews = buildJiaobanArtifactCanvasViews({
    phase: "done",
    selectedHistoryId: null,
    activeViewKey: "delivery",
    proposalInteractive: false,
    proposalContent: <div>交货后的方案留痕</div>,
    deliveryContent: <div>交货实体卡与属实沉淀动作</div>,
    graphContent: <div>交货工序图正文</div>,
    governanceContent: null,
    howRunContent: null,
  });
  assert(doneViews, "done 有交货时应组装右区视图");
  const doneOut = renderToStaticMarkup(
    <JiaobanMergedLayout
      phase="done"
      history={<div>历史</div>}
      main={<div>交货短讯</div>}
      previewCanvas={null}
      workflowPanel={null}
      onOpenWorkflow={noop}
      canvasViews={doneViews}
      activeCanvasView="delivery"
      onCanvasViewChange={noop}
    />,
  );
  assert((doneOut.match(/role="tabpanel"/g) ?? []).length === 3, "done 交货、方案留痕与工序图正文应常驻");
  assert(tabOpeningTag(doneOut, "交货").includes('aria-selected="true"'), "done 应默认激活交货 tab");
  assert(
    !isHiddenTag(openingTagBefore(doneOut, "交货实体卡与属实沉淀动作", 'role="tabpanel"')),
    "激活交货正文不得 hidden",
  );
  assert(
    isHiddenTag(openingTagBefore(doneOut, "交货工序图正文", 'role="tabpanel"')),
    "done 非激活工序图应常驻但 hidden",
  );
  assert(
    isHiddenTag(openingTagBefore(doneOut, "交货后的方案留痕", 'role="tabpanel"')),
    "done 非激活方案留痕应常驻但 hidden",
  );

  const sayArtifactViews = buildJiaobanArtifactCanvasViews({
    phase: "say",
    selectedHistoryId: null,
    activeViewKey: "proposal",
    proposalInteractive: false,
    proposalContent: <div>说态回看的方案</div>,
    deliveryContent: null,
    graphContent: null,
    governanceContent: null,
    howRunContent: null,
  });
  assert(sayArtifactViews, "say 点方案短讯后应组装 artifact views");
  assert(
    buildJiaobanArtifactCanvasViews({
      phase: "say",
      selectedHistoryId: null,
      activeViewKey: "graph",
      proposalInteractive: false,
      proposalContent: <div>未主动点开的方案</div>,
      deliveryContent: null,
      graphContent: null,
      governanceContent: null,
      howRunContent: null,
    }) === undefined,
    "普通 say 未点定稿物时不得主动展开右区",
  );
  const sayWithViews = renderToStaticMarkup(
    <JiaobanMergedLayout
      phase="say"
      history={<div>历史</div>}
      main={<div>说态输入框</div>}
      previewCanvas={null}
      workflowPanel={null}
      onOpenWorkflow={noop}
      canvasViews={sayArtifactViews}
      activeCanvasView="proposal"
      onCanvasViewChange={noop}
    />,
  );
  assert(sayWithViews.includes("is-canvas-wide"), "say 只要带视图就应展开右区");
  assert(!sayWithViews.includes("is-canvas-hint"), "say 带视图时不得退成提示窄条");
  assert(sayWithViews.includes("jiaoban-canvas-view-tabs"), "say 带视图时应保留视图切换");

  const runtimeViews = buildJiaobanArtifactCanvasViews({
    phase: "running",
    selectedHistoryId: null,
    activeViewKey: "graph",
    proposalInteractive: false,
    proposalContent: <div>运行中可回看的方案</div>,
    deliveryContent: null,
    graphContent: null,
    governanceContent: null,
    howRunContent: null,
  });
  assert(runtimeViews?.map((view) => view.key).join(",") === "graph,proposal", "运行族应保留工序图默认与方案回看");
  const runtimeOut = renderToStaticMarkup(
    <JiaobanMergedLayout
      phase="running"
      history={<div>历史</div>}
      main={<div>运行对话</div>}
      previewCanvas={null}
      workflowPanel={<div>既有工作流画布</div>}
      onOpenWorkflow={noop}
      canvasViews={runtimeViews}
      activeCanvasView="graph"
      onCanvasViewChange={noop}
    />,
  );
  assert(runtimeOut.includes("既有工作流画布"), "运行族无工序图时 graph tab 应回落既有工作流画布");
  assert(!runtimeOut.includes("预演图关着"), "运行族不得显示批准前的预演空话");

  const runtimeWorkStateViews = buildJiaobanArtifactCanvasViews({
    phase: "running",
    selectedHistoryId: null,
    activeViewKey: "graph",
    proposalInteractive: false,
    proposalContent: null,
    deliveryContent: null,
    graphContent: <div>运行工序图</div>,
    workStateContent: <div>正在干与停下动作</div>,
    governanceContent: null,
    howRunContent: null,
  });
  assert(runtimeWorkStateViews?.map((view) => view.key).join(",") === "graph", "无方案时运行处置仍应落右区工序图");
  const runtimeWorkStateOut = renderToStaticMarkup(runtimeWorkStateViews?.[0]?.content);
  assert(
    runtimeWorkStateOut.includes("运行工序图") && runtimeWorkStateOut.includes("正在干与停下动作"),
    "工序图应承载运行处置，不能回流中栏",
  );
}

// 7) 07-15 真机走查·交办页 chrome 与说态卡对齐定稿。
{
  const { ProjectWorkspaceShell } = await import("../src/views/projects/ProjectWorkspaceShell");
  const shellOut = renderToStaticMarkup(
    <ProjectWorkspaceShell
      project={{
        project_root: "/tmp/chrome-align-test",
        name: "chrome 对齐",
        active_hint: false,
        thread_count: 0,
        latest_updated_at_ms: null,
        context_warnings: [],
        warnings: [],
      } as unknown as ProjectRecord}
      sessions={[]}
      workflowState={null}
      onRequestAction={noop}
      onProposalStoreRefresh={(async () => {}) as never}
      onRenderTaskPreview={(async () => ({})) as never}
      onInspectDispatchReadiness={(async () => ({})) as never}
    />,
  );
  // 信息规范四问:空值状态条整条不上脸;「阶段」机器词格删除;词表:运行器→harness。
  assert(!shellOut.includes("项目状态条"), "无 harness/技能声明时状态条整条不渲染");
  assert(!shellOut.includes("未要求运行器") && !shellOut.includes("未声明技能"), "空值格不再上脸");
  assert(!shellOut.includes("运行器"), "「运行器」词表废止(状态条面)");
  assert(!shellOut.includes(">阶段<"), "「阶段」机器词格删除(进度人话在主卡 pill)");

  const { JiaobanSayState } = await import("../src/views/projects/jiaoban/JiaobanAuthorizeStates");
  const sayOut = renderToStaticMarkup(
    <JiaobanSayState goal="" onGoalChange={noop} onSubmit={noop} lastStopHint={null} loading={false} error={null} onEditAgain={noop} />,
  );
  assert(!sayOut.includes("说一句话，AI 会读你的项目"), "说态卡教育句不上脸(定稿=标题+占位+出方案)");
  assert(sayOut.includes("sr-only"), "textarea 的可及标签保留(读屏)");
  assert(sayOut.includes("出方案"), "主动作在");

  const { JiaobanHistoryColumn } = await import("../src/views/projects/jiaoban/JiaobanHistory");
  const historyOut = renderToStaticMarkup(
    <JiaobanHistoryColumn
      entries={[]}
      total={0}
      loading={false}
      filter="all"
      onFilterChange={noop}
      selectedId={null}
      currentProposalId={null}
      latestBlockedId={null}
      onSelectEntry={noop}
      onBackToCurrent={noop}
      onNewJiaoban={noop}
      onContinueRun={noop}
    />,
  );
  assert(historyOut.includes('class="secondary-button jiaoban-history-new"'), "历史头 [+] 降为次级小钮");
  assert(historyOut.includes("工作历史") && !historyOut.includes("项目对话"), "左栏只改锚点，不改既有标题");
  assert(historyOut.includes("+ 新交办"), "空态保留主动作 [+新交办](D7 空态答下一步)");
}

console.log("jiaoban-merged-layout: 6 组离线 DOM / 独立历史栏 / 画布动态宽 / 三栏顺序 / chrome·说态卡对齐断言全过");
