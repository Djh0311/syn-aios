import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SourceStylePlaceholder } from "./SourceStylePlaceholder";
import {
  inspectAutoDispatchAuthorization,
  inspectTaskPackageDispatchReadiness,
  inspectWorkflowRunCheck,
  loadCodexSessionTranscript,
  loadCodexSessionTranscriptPage,
  previewMaturePatterns,
  renderTaskPackagePreview,
} from "../lib/tauri";
import type {
  BlackboardCandidateStoreV1,
  FormalMemoryStoreV1,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  MemoryEntityRelationStoreV1,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  ObservationStoreV1,
  PendingAction,
  PlanAuthorizationStoreV1,
  PreviewProjectDirectorTaskPlanInput,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../lib/types";
import { devNavItems, type ViewKey } from "../lib/workbenchNavigation";
import { AgentView } from "../views/AgentView";
import { CanvasViewWithProvider } from "../views/CanvasView";
import { WorkflowCommandConsoleView } from "../views/WorkflowCommandConsoleView";
import { HarnessBoardView } from "../views/HarnessBoardView";
import { HomeView } from "../views/HomeView";
import { KnowledgeBaseView } from "../views/KnowledgeBaseView";
import { MemoryCenterView } from "../views/MemoryCenterView";
import { ProjectsView } from "../views/ProjectsView";
import { SettingsView } from "../views/SettingsView";
import { SkillsBoardView } from "../views/SkillsBoardView";

type BrowserPreviewData = {
  loadSessionPage: Parameters<typeof AgentView>[0]["onLoadSessionPage"];
  loadTranscript: Parameters<typeof AgentView>[0]["onLoadTranscript"];
  loadTranscriptPage: Parameters<typeof AgentView>[0]["onLoadTranscriptPage"];
};

export type ActiveWorkbenchViewProps = {
  view: ViewKey;
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  workflowStateLoading: boolean;
  workflowStateError: string | null;
  hasRealSnapshot: boolean;
  focusedAgentThreadId?: string | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore?: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore?: ProjectConsultationProposalStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  memoryEntityRelationStore?: MemoryEntityRelationStoreV1 | null;
  memoryPatternStore?: MemoryPatternStoreV1 | null;
  browserPreviewData?: BrowserPreviewData;
  onRequestAction: (action: PendingAction) => void;
  onNavigate: (view: ViewKey) => void;
  onReloadWorkflowState: () => void;
  onNotice: (msg: string) => void;
  onOpenAgentSession: (threadId: string) => void;
  // fix8：出方案成功刷店（穿到交办面板）。App 的 reloadCandidateStores 穿下来。
  onProposalStoreRefresh?: () => Promise<void>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  onPreviewFormalMemoryLifecycle?: Parameters<typeof MemoryCenterView>[0]["onPreviewFormalMemoryLifecycle"];
  onPreviewMemoryEntityRelationCandidates?: Parameters<typeof MemoryCenterView>[0]["onPreviewMemoryEntityRelationCandidates"];
};

export function renderActiveWorkbenchView({
  view,
  snapshot,
  onRequestAction,
  onNavigate,
  workflowState,
  workflowStateLoading,
  workflowStateError,
  onReloadWorkflowState,
  onNotice,
  hasRealSnapshot,
  onOpenAgentSession,
  onProposalStoreRefresh,
  focusedAgentThreadId,
  blackboardCandidateStore,
  planAuthorizationStore,
  projectConsultationProposalStore,
  observationStore,
  memoryCaptureStore,
  memoryCandidateStore,
  formalMemoryStore,
  memoryLintStore,
  memoryEntityRelationStore,
  memoryPatternStore,
  onPreviewTaskMemoryPacket,
  onPreviewProjectDirectorTaskPlan,
  onPreviewFormalMemoryLifecycle,
  onPreviewMemoryEntityRelationCandidates,
  browserPreviewData,
}: ActiveWorkbenchViewProps) {
  if (view === "agents") {
    return (
      <AgentView
        sessions={snapshot.sessions}
        projects={snapshot.projects}
        adapterDescriptors={snapshot.agent_adapters}
        sessionOperationDescriptors={snapshot.session_operations}
        providerAvailabilitySummaries={snapshot.provider_availability}
        sessionContinuationPreviews={snapshot.session_continuation_previews}
        sessionContinuationStore={snapshot.session_continuation_store}
        runtimeSessionAttention={snapshot.runtime_session_attention}
        sessionRunStatusSummaries={snapshot.session_run_status_summaries}
        realExecutionProductCommands={snapshot.real_execution_product_commands}
        projectWorkflowAutomation={snapshot.project_workflow_automation}
        workerProtocol={snapshot.worker_protocol}
        workflowState={workflowState}
        focusedThreadId={focusedAgentThreadId}
        onLoadSessionPage={browserPreviewData?.loadSessionPage}
        onLoadTranscript={browserPreviewData?.loadTranscript ?? loadCodexSessionTranscript}
        onLoadTranscriptPage={browserPreviewData?.loadTranscriptPage ?? loadCodexSessionTranscriptPage}
        onRequestAction={onRequestAction}
      />
    );
  }

  if (view === "projects") {
    return (
      <ProjectsView
        projects={snapshot.projects}
        sessions={snapshot.sessions}
        workflowState={workflowState}
        blackboardCandidateStore={blackboardCandidateStore}
        planAuthorizationStore={planAuthorizationStore}
        projectConsultationProposalStore={projectConsultationProposalStore}
        observationStore={observationStore}
        memoryCandidateStore={memoryCandidateStore}
        formalMemoryStore={formalMemoryStore}
        memoryLintStore={memoryLintStore}
        runtimeSessionAttention={snapshot.runtime_session_attention}
        realExecutionProductCommands={snapshot.real_execution_product_commands}
        projectWorkflowAutomation={snapshot.project_workflow_automation}
        k3B1Recovery={snapshot.k3_b1_recovery}
        workflowStateLoading={workflowStateLoading}
        workflowStateError={workflowStateError}
        onReloadWorkflowState={onReloadWorkflowState}
        onRequestAction={onRequestAction}
        onProposalStoreRefresh={onProposalStoreRefresh}
        onLoadTranscript={browserPreviewData?.loadTranscript ?? loadCodexSessionTranscript}
        onRenderTaskPreview={(projectRoot, workItemId) =>
          browserPreviewData
            ? Promise.reject(new Error("浏览器预览模式：任务包预览需用 Tauri 桌面壳。"))
            : renderTaskPackagePreview({ project_root: projectRoot, work_item_id: workItemId })
        }
        onInspectDispatchReadiness={(projectRoot, workItemId) =>
          browserPreviewData
            ? Promise.reject(new Error("浏览器预览模式：派发准备检查需用 Tauri 桌面壳。"))
            : inspectTaskPackageDispatchReadiness({ project_root: projectRoot, work_item_id: workItemId })
        }
        onInspectWorkflowRunCheck={(projectRoot, workflowId) =>
          browserPreviewData
            ? Promise.reject(new Error("浏览器预览模式：运行检查需用 Tauri 桌面壳。"))
            : inspectWorkflowRunCheck({ project_root: projectRoot, workflow_id: workflowId })
        }
        onInspectAutoDispatchAuthorization={
          browserPreviewData
            ? () => Promise.reject(new Error("浏览器预览模式：自动派发授权检查需用 Tauri 桌面壳。"))
            : inspectAutoDispatchAuthorization
        }
        onPreviewTaskMemoryPacket={onPreviewTaskMemoryPacket}
        onPreviewProjectDirectorTaskPlan={onPreviewProjectDirectorTaskPlan}
        onOpenAgentSession={onOpenAgentSession}
        onNotice={onNotice}
      />
    );
  }

  if (view === "skills") {
    return <SkillsBoardView skills={snapshot.skills} plugins={snapshot.plugins} projects={snapshot.projects} />;
  }

  if (view === "harness") {
    return <HarnessBoardView projects={snapshot.projects} />;
  }

  if (view === "workflow") {
    // 实验画布（沙盒）：已扶正到主栏入口。项目工作流的「运行状态」归项目面
    // （画布架构方案 P1/P2 项目面运行状态视图），不再单列「运行中工作流」入口。
    return (
      <div className="canvas-view-fullwindow">
        <CanvasViewWithProvider
          canvasId="default"
          sessions={snapshot.sessions}
          onNotice={onNotice}
        />
      </div>
    );
  }

  if (view === "command-console") {
    // P2 发令台：对工作流发令起链（对话只启动/停，真跑仍走 gated 链控制器·圈测试项目）。
    return (
      <WorkflowCommandConsoleView
        projects={snapshot.projects}
        onNotice={onNotice}
        onReloadWorkflowState={onReloadWorkflowState}
      />
    );
  }

  if (view === "ideas") {
    const taskItems = snapshot.tasks.map((task) => `${task.status} · ${task.title}`);
    const projectWarningItems = snapshot.projects
      .filter((project) => project.context_warnings.length || project.warnings.length)
      .map((project) => `${project.name} · ${project.context_warnings.length + project.warnings.length} 条上下文提醒`)
      .slice(0, 6);
    return (
      <SourceStylePlaceholder
        title="想法箱"
        kicker="想法入口"
        hasRealSnapshot={hasRealSnapshot}
        items={taskItems}
        summary="收纳跨项目任务线索和上下文提醒；当前只读展示已有索引，转任务需要单独确认。"
        primaryStat={`${taskItems.length} 条线索`}
        secondaryStat="转任务后置"
        sections={[
          {
            title: "任务线索",
            eyebrow: "来自当前快照 tasks",
            items: taskItems.slice(0, 8),
            emptyText: "当前索引没有提供任务线索。",
          },
          {
            title: "项目提醒",
            eyebrow: "来自项目上下文警告",
            items: projectWarningItems,
            emptyText: "当前项目索引没有上下文提醒。",
          },
        ]}
        boundary={{
          title: "想法边界",
          text: "后续可接入捕获、归并、转任务和秘书建议；转任务必须另走用户确认，本页现在不写任务、不改项目、不触发执行。",
          status: "只读",
        }}
      />
    );
  }

  if (view === "proposal") {
    const projectProposalItems = snapshot.projects
      .slice(0, 6)
      .map((project) => `${project.name} · ${project.context_warnings.length + project.warnings.length} 条边界提醒`);
    const workflowItems = (workflowState?.project_workflows ?? [])
      .slice(0, 6)
      .map((workflow) => `${workflow.title || workflow.workflow_id} · ${workflow.state} / ${workflow.task_draft_count} 个任务草稿`);
    return (
      <SourceStylePlaceholder
        title="建议方案"
        kicker="方案入口"
        hasRealSnapshot={hasRealSnapshot}
        items={projectProposalItems}
        summary="集中查看方案草案、项目边界和工作流复核入口；真实方案确认仍在项目页权限弹层完成。"
        primaryStat={`${projectProposalItems.length} 个项目`}
        secondaryStat="确认后置"
        sections={[
          {
            title: "项目边界",
            eyebrow: "来自项目索引",
            items: projectProposalItems,
            emptyText: "当前没有可展示的项目边界摘要。",
          },
          {
            title: "工作流关联",
            eyebrow: "来自 workflow state",
            items: workflowItems,
            emptyText: "当前没有读取到项目工作流状态。",
          },
        ]}
        boundary={{
          title: "方案边界",
          text: "本页只作为方案入口和摘要，不批准范围、不创建授权、不代表全局复核完成。",
          status: "入口",
        }}
      />
    );
  }

  if (view === "knowledge") {
    return (
      <KnowledgeBaseView
        projects={snapshot.projects}
        workflowState={workflowState}
        formalMemoryStore={formalMemoryStore}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        hasRealSnapshot={hasRealSnapshot}
        onRequestAction={onRequestAction}
      />
    );
  }

  if (view === "memory") {
    return (
      <MemoryCenterView
        projects={snapshot.projects}
        workflowState={workflowState}
        formalMemoryStore={formalMemoryStore}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        observationStore={observationStore}
        memoryLintStore={memoryLintStore}
        memoryEntityRelationStore={memoryEntityRelationStore}
        memoryPatternStore={memoryPatternStore}
        hasRealSnapshot={hasRealSnapshot}
        onRequestAction={onRequestAction}
        onPreviewFormalMemoryLifecycle={onPreviewFormalMemoryLifecycle}
        onPreviewMemoryEntityRelationCandidates={onPreviewMemoryEntityRelationCandidates}
        onPreviewMaturePatterns={previewMaturePatterns}
      />
    );
  }

  if (view === "tools") {
    const harnessItems = snapshot.projects
      .flatMap((project) => project.harness_resources.map((resource) => `${project.name} · ${resource.display_name ?? resource.root_path}`))
      .slice(0, 8);
    const adapterActionItems = snapshot.session_operations
      .slice(0, 6)
      .map((operation) => `${operation.label} · ${operation.adapter_id}`);
    return (
      <>
        <SourceStylePlaceholder
          title="工具"
          kicker="工具入口"
          hasRealSnapshot={hasRealSnapshot}
          items={harnessItems}
          summary="展示运行器资源和适配器动作索引；这里不提供直接运行按钮，避免工具入口绕过项目授权。"
          primaryStat={`${harnessItems.length} 个资源`}
          secondaryStat="执行后置"
          sections={[
            {
              title: "运行器资源",
              eyebrow: "来自项目资源索引",
              items: harnessItems,
              emptyText: "当前索引没有提供运行器资源。",
            },
            {
              title: "适配器动作",
              eyebrow: "来自 session operations",
              items: adapterActionItems,
              emptyText: "当前没有可展示的适配器动作索引。",
            },
          ]}
          boundary={{
            title: "工具边界",
            text: "工具真实执行必须回到项目页、权限弹层或既有受控链路；本页不接入 Codex 运行、不碰凭据。唯一例外：下方「清理画布历史残料」做合法可逆的状态归档（ready_for_review→paused · 不删 · 带审计）。",
            status: "维护",
          }}
        />
        <CanvasRunResidueSweeperCard />
      </>
    );
  }

  if (view === "models") {
    const adapterItems = snapshot.agent_adapters
      .slice(0, 8)
      .map((adapter) => `${adapter.display_name} · ${adapter.adapter_id}`);
    const providerItems = snapshot.provider_availability
      .slice(0, 8)
      .map((provider) => `${provider.provider_label} · ${provider.availability_status}`);
    return (
      <SourceStylePlaceholder
        title="模型 / 凭据"
        kicker="模型入口"
        hasRealSnapshot={hasRealSnapshot}
        items={adapterItems}
        summary="说明模型、供应方、适配器和凭据边界；只展示可见摘要，不读取或展示密钥、令牌和认证材料。"
        primaryStat={`${adapterItems.length} 个适配器`}
        secondaryStat="凭据不可见"
        sections={[
          {
            title: "适配器",
            eyebrow: "来自 adapter descriptors",
            items: adapterItems,
            emptyText: "当前索引没有提供适配器摘要。",
          },
          {
            title: "供应方状态",
            eyebrow: "来自 provider availability",
            items: providerItems,
            emptyText: "当前没有可展示的供应方状态。",
          },
        ]}
        boundary={{
          title: "凭据边界",
          text: "凭据配置只能显示边界和状态摘要；本页不接触密钥原文、不验证供应方、不改变模型执行语义。",
          status: "安全",
        }}
      />
    );
  }

  if (view === "settings") {
    return (
      <SettingsView
        snapshot={snapshot}
        workflowState={workflowState}
        workflowStateError={workflowStateError}
        hasRealSnapshot={hasRealSnapshot}
        developerItems={devNavItems}
        onNavigate={onNavigate}
      />
    );
  }

  return <HomeView snapshot={snapshot} workflowState={workflowState} onNavigate={onNavigate} />;
}

type CanvasRunResidueItem = {
  work_item_id: string;
  workflow_id: string;
  age_days: number;
  swept: boolean;
};

type SweepCanvasRunResidueResult = {
  dry_run: boolean;
  matched_count: number;
  swept_count: number;
  items: CanvasRunResidueItem[];
  audit_event_id: string | null;
  backup_path: string | null;
  message: string;
};

// dev-only 维护触发口：canvas-run 历史残料合法归档（ready_for_review → paused）。
// 先 dry-run 盘点，用户确认后再执行。直接 invoke（本包文件面不含 lib/tauri.ts，故不加 wrapper）。
function CanvasRunResidueSweeperCard() {
  const [preview, setPreview] = useState<SweepCanvasRunResidueResult | null>(null);
  const [done, setDone] = useState<SweepCanvasRunResidueResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // 页面内两步确认（Tauri webview 不弹 window.confirm，会静默失败）。
  const [confirming, setConfirming] = useState(false);

  const runSweep = async (dryRun: boolean) => {
    setBusy(true);
    setError(null);
    try {
      const result = await invoke<SweepCanvasRunResidueResult>("sweep_canvas_run_residue", {
        request: { project_root: null, dry_run: dryRun, now_ms: Date.now() },
      });
      if (dryRun) {
        setPreview(result);
        setDone(null);
      } else {
        setDone(result);
        setPreview(null);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section
      style={{
        marginTop: 16,
        padding: 16,
        border: "1px solid var(--border-subtle, #333)",
        borderRadius: 10,
      }}
    >
      <h3 style={{ margin: "0 0 4px", fontSize: 15 }}>清理画布历史残料</h3>
      <p style={{ margin: "0 0 12px", opacity: 0.7, fontSize: 13, lineHeight: 1.5 }}>
        把「canvas-run 形状 + 待审(ready_for_review) + 超 7 天」的历史工作项合法归档为 paused（一步合法 ·
        可逆 · 不删记录 · 带审计）。先盘点，确认后再执行。
      </p>
      <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
        <button
          type="button"
          disabled={busy}
          onClick={() => {
            setConfirming(false);
            void runSweep(true);
          }}
        >
          {busy ? "处理中…" : "盘点（dry-run）"}
        </button>
        {preview && preview.matched_count > 0 ? (
          confirming ? (
            <>
              <span style={{ fontSize: 13 }}>确认归档 {preview.matched_count} 条？</span>
              <button
                type="button"
                disabled={busy}
                onClick={() => {
                  setConfirming(false);
                  void runSweep(false);
                }}
              >
                确认执行
              </button>
              <button type="button" disabled={busy} onClick={() => setConfirming(false)}>
                取消
              </button>
            </>
          ) : (
            <button type="button" disabled={busy} onClick={() => setConfirming(true)}>
              执行归档（{preview.matched_count} 条 → paused）
            </button>
          )
        ) : null}
      </div>
      {error ? (
        <p style={{ color: "var(--danger, #e55b5b)", marginTop: 12, fontSize: 13 }}>出错：{error}</p>
      ) : null}
      {preview ? (
        <div style={{ marginTop: 12 }}>
          <p style={{ margin: "0 0 6px", fontSize: 13 }}>
            找到 {preview.matched_count} 条{preview.matched_count > 0 ? "，预览如下：" : "。"}
          </p>
          {preview.matched_count === 0 ? (
            <p style={{ opacity: 0.6, fontSize: 13 }}>没有命中的残料，无需清理。</p>
          ) : (
            <ul
              style={{
                margin: 0,
                paddingLeft: 18,
                maxHeight: 220,
                overflow: "auto",
                fontSize: 12,
                lineHeight: 1.6,
              }}
            >
              {preview.items.map((item) => (
                <li key={item.work_item_id}>
                  <code>{item.work_item_id}</code>
                  <span style={{ opacity: 0.6 }}> · {item.age_days} 天</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      ) : null}
      {done ? (
        <div style={{ marginTop: 12 }}>
          <p style={{ margin: 0, color: "var(--success, #4caf72)", fontSize: 13 }}>{done.message}</p>
          {done.backup_path ? (
            <p style={{ margin: "4px 0 0", opacity: 0.6, fontSize: 12 }}>
              已备份：<code>{done.backup_path}</code>
            </p>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
