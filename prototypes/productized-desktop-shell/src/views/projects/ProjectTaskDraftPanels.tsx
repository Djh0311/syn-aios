import { memo, useEffect, useState } from "react";
import { Badge } from "../../components/Badge";
import { summarizeTaskPackageMemoryInjection } from "../../lib/candidateGovernance";
import type {
  PendingAction,
  ProjectRecord,
  TaskDraftSummary,
  TaskPackageDispatchReadiness,
  TaskPackageFields,
  TaskPackagePreview,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { DetailLine } from "./ProjectOverviewPanels";

export function ProjectWorkflowDraftPanel({
  project,
  workflowState,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
}: {
  project: ProjectRecord;
  workflowState: WorkflowStateSnapshot | null;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const assignedRole = "codex-dev";
  const fallbackSelectedTaskDraft = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);

  return (
    <section className="workflow-draft-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目工作流草稿</p>
          <h3>{projectWorkflow ? "当前项目已有本地工作流草稿" : "当前项目还没有本地工作流草稿"}</h3>
        </div>
        <Badge tone={projectWorkflow ? "candidate" : "unknown"}>{projectWorkflow ? "已创建" : "未创建"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="工作流" value={projectWorkflow?.workflow_id || "未创建"} />
        <DetailLine label="状态" value={projectWorkflow?.state || "未创建"} />
        <DetailLine label="节点" value={String(projectWorkflow?.node_count ?? 0)} />
        <DetailLine label="边" value={String(projectWorkflow?.edge_count ?? 0)} />
        <DetailLine label="任务草稿" value={`${projectWorkflow?.task_draft_count ?? 0} 个`} />
      </div>
      <div className="workflow-state-actions">
        <button
          className="primary-button"
          type="button"
          disabled={Boolean(projectWorkflow)}
          onClick={() =>
            onRequestAction({
              kind: "bootstrap-project-workflow",
              label: "创建项目默认工作流草稿",
              path: project.project_root,
              source: "索引内项目路径",
              boundary:
                "给工作台自己的 workflow-state.v0.json 写入项目、workflow、默认节点、默认边和 audit；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
            })
          }
        >
          创建默认工作流草稿
        </button>
      </div>
      {projectWorkflow ? (
        <div className="task-draft-box">
          <form
            className="task-draft-form"
            onSubmit={(event) => {
              event.preventDefault();
              const formData = new FormData(event.currentTarget);
              const title = String(formData.get("task-title") ?? "").trim();
              const objective = String(formData.get("task-objective") ?? "").trim();
              if (!title || !objective) return;
              onRequestAction({
                kind: "create-task-draft",
                label: "创建任务包草稿",
                path: project.project_root,
                source: "索引内项目路径",
                boundary:
                  "只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行。",
                taskDraft: {
                  projectRoot: project.project_root,
                  title,
                  objective,
                  assignedRole,
                },
              });
            }}
          >
            <label>
              <span>标题</span>
              <input name="task-title" required placeholder="任务包草稿标题" />
            </label>
            <label>
              <span>目标说明</span>
              <textarea
                name="task-objective"
                required
                placeholder="这次任务要完成什么"
                rows={3}
              />
            </label>
            <label>
              <span>指派角色</span>
              <select defaultValue={assignedRole} disabled>
                <option value="codex-dev">Codex 开发线</option>
              </select>
            </label>
            <button className="primary-button" type="submit">
              创建任务包草稿
            </button>
          </form>
          <div className="task-draft-list" aria-label="任务包草稿列表">
            {projectWorkflow.task_drafts.length ? (
              projectWorkflow.task_drafts.map((taskDraft) => (
                <div className={`task-draft-item ${taskDraft.work_item_id === fallbackSelectedTaskDraft?.work_item_id ? "selected" : ""}`} key={taskDraft.work_item_id}>
                  <strong>{taskDraft.title}</strong>
                  <span>{taskDraft.state}</span>
                  <em>{taskDraft.artifact_type || "artifact 类型缺失"}</em>
                  {taskDraft.artifact_path ? (
                    <details className="agent-boundary-details">
                      <summary className="agent-boundary-summary">开发者详情</summary>
                      <em>{taskDraft.artifact_path}</em>
                    </details>
                  ) : null}
                  {taskDraft.work_item_id === fallbackSelectedTaskDraft?.work_item_id ? <b>当前选中</b> : <b>选择</b>}
                </div>
              ))
            ) : (
              <p className="muted small-note">当前工作流下还没有任务包草稿；下一步先创建任务包草稿。</p>
            )}
          </div>
          <div className="task-preview-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">任务包 Markdown 预览</p>
                <h3>预览，不是已派发任务包</h3>
              </div>
              <Badge tone="unknown">选择草稿后渲染</Badge>
            </div>
            <p className="muted small-note">有任务草稿时可以点“预览 Markdown”查看只读文本。</p>
            <p className="muted small-note">编辑字段表单会绑定当前选中的任务草稿。</p>
            <TaskDraftSelectionController
              projectRoot={project.project_root}
              taskDrafts={projectWorkflow.task_drafts}
              fallbackSelectedTaskDraft={fallbackSelectedTaskDraft}
              onRequestAction={onRequestAction}
              onRenderTaskPreview={onRenderTaskPreview}
              onInspectDispatchReadiness={onInspectDispatchReadiness}
            />
          </div>
        </div>
      ) : (
        <p className="state-warning">当前项目还没有工作流；请先创建默认工作流草稿，再登记任务包草稿。</p>
      )}
      <p className="muted small-note">这是给工作台自己的小账本写入草稿，不会派发给真实 Codex 会话，也不会生成任务包文件。</p>
    </section>
  );
}

const TaskDraftSelectionController = memo(function TaskDraftSelectionController({
  projectRoot,
  taskDrafts,
  fallbackSelectedTaskDraft,
  onRequestAction,
  onRenderTaskPreview,
  onInspectDispatchReadiness,
}: {
  projectRoot: string;
  taskDrafts: TaskDraftSummary[];
  fallbackSelectedTaskDraft: TaskDraftSummary | null;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const [selectedWorkItemId, setSelectedWorkItemId] = useState<string | null>(fallbackSelectedTaskDraft?.work_item_id ?? null);
  const selectedTaskDraft = selectedTaskDraftFor(taskDrafts, selectedWorkItemId);

  useEffect(() => {
    setSelectedWorkItemId((current) => nextSelectedWorkItemId(taskDrafts, current));
  }, [taskDrafts]);

  if (!taskDrafts.length) {
    return <p className="muted small-note">当前工作流下还没有任务包草稿；无法预览或保存字段。</p>;
  }

  if (!selectedTaskDraft) {
    return <p className="state-warning">当前选中的任务草稿不存在；请重新选择。</p>;
  }

  return (
    <>
      <div className="workflow-state-actions" aria-label="选择任务草稿">
        {taskDrafts.map((taskDraft) => (
          <button
            className={taskDraft.work_item_id === selectedTaskDraft.work_item_id ? "primary-button" : "secondary-button"}
            type="button"
            key={taskDraft.work_item_id}
            onClick={() => setSelectedWorkItemId(taskDraft.work_item_id)}
          >
            {taskDraft.work_item_id === selectedTaskDraft.work_item_id ? "当前选中" : "选择"}
          </button>
        ))}
      </div>
      <TaskPreviewController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
        onRenderTaskPreview={onRenderTaskPreview}
      />
      <TaskFileGenerationController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
      />
      <TaskDispatchReadinessController
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
        onInspectDispatchReadiness={onInspectDispatchReadiness}
      />
      <TaskDispatchFieldCorrectionEditor
        projectRoot={projectRoot}
        selectedTaskDraft={selectedTaskDraft}
        onRequestAction={onRequestAction}
      />
      <TaskFieldsEditor projectRoot={projectRoot} selectedTaskDraft={selectedTaskDraft} onRequestAction={onRequestAction} />
    </>
  );
});

function TaskPreviewController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
  onRenderTaskPreview,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
}) {
  const [selectedPreview, setSelectedPreview] = useState<TaskPackagePreview | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedPreview(null);
    setPreviewError(null);
  }, [selectedTaskDraft.work_item_id]);

  async function loadPreview() {
    if (!onRenderTaskPreview) {
      setPreviewError("当前运行环境没有接入预览渲染入口。");
      return;
    }
    setPreviewLoading(true);
    setPreviewError(null);
    try {
      const preview = await onRenderTaskPreview(projectRoot, selectedTaskDraft.work_item_id);
      setSelectedPreview(preview);
    } catch (error) {
      setSelectedPreview(null);
      setPreviewError(messageOf(error));
    } finally {
      setPreviewLoading(false);
    }
  }

  return (
    <>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={() => void loadPreview()}>
          预览 Markdown
        </button>
      </div>
      {previewError ? <p className="state-warning">{previewError}</p> : null}
      {previewLoading ? <p className="muted small-note">正在渲染预览。</p> : null}
      {selectedPreview ? (
        <>
          {selectedPreview.warnings.map((warning) => (
            <p className="state-warning" key={warning}>
              {warning}
            </p>
          ))}
          <pre className="task-preview-code">{selectedPreview.markdown}</pre>
          <div className="workflow-state-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction({
                  kind: "copy-task-preview",
                  label: "复制任务包 Markdown 预览",
                  path: projectRoot,
                  source: "索引内项目路径",
                  boundary: "只复制预览文本到剪贴板；不写真实任务文件、不派发真实 Codex 会话。",
                  taskPreview: {
                    projectRoot,
                    workItemId: selectedPreview.work_item_id,
                  },
                })
              }
            >
              复制预览文本
            </button>
          </div>
        </>
      ) : (
        <p className="muted small-note">请选择一个任务包草稿查看 Markdown 预览。</p>
      )}
    </>
  );
}

export function TaskFileGenerationController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  const generatedPath = selectedTaskDraft.artifact_path?.trim() || "";

  return (
    <div className="task-file-generation-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">真实任务包文件</p>
          <h3>{generatedPath ? "该草稿已有生成文件" : "从当前草稿生成文件"}</h3>
        </div>
        <Badge tone={generatedPath ? "candidate" : "unknown"}>{generatedPath ? "已生成" : "未生成"}</Badge>
      </div>
      {generatedPath ? <p className="path-text">{generatedPath}</p> : null}
      <div className="workflow-state-actions">
        <button
          className={generatedPath ? "secondary-button" : "primary-button"}
          type="button"
          disabled={Boolean(generatedPath)}
          onClick={() =>
            onRequestAction({
              kind: "generate-task-file",
              label: "生成任务包文件",
              path: projectRoot,
              source: "索引内项目路径",
              boundary:
                "写入 /Users/yoyi/workspace/product-line/tasks/ 下的新 Markdown 文件，并更新工作台自己的 workflow-state.v0.json；不覆盖已有任务包、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
              taskFileGeneration: {
                project_root: projectRoot,
                work_item_id: selectedTaskDraft.work_item_id,
              },
            })
          }
        >
          {generatedPath ? "已生成" : "生成任务包文件"}
        </button>
      </div>
    </div>
  );
}

export function TaskDispatchReadinessController({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
  onInspectDispatchReadiness,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
}) {
  const [readiness, setReadiness] = useState<TaskPackageDispatchReadiness | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setReadiness(null);
    setError(null);
  }, [selectedTaskDraft.work_item_id]);

  async function inspect() {
    if (!onInspectDispatchReadiness) {
      setError("当前运行环境没有接入派发准备检查入口。");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      setReadiness(await onInspectDispatchReadiness(projectRoot, selectedTaskDraft.work_item_id));
    } catch (inspectError) {
      setReadiness(null);
      setError(messageOf(inspectError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <TaskDispatchReadinessShell
      readiness={readiness}
      loading={loading}
      error={error}
      onInspect={() => void inspect()}
      onGenerateReadyFile={() =>
        onRequestAction({
          kind: "generate-task-file",
          label: "生成可派发版本",
          path: projectRoot,
          source: "索引内项目路径",
          boundary:
            "只生成一个新的 product-line/tasks/*.md 任务包文件，并更新工作台自己的 workflow-state.v0.json；不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
          taskFileGeneration: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
          },
        })
      }
    />
  );
}

export function TaskDispatchReadinessShell({
  readiness,
  loading,
  error,
  onInspect,
  onGenerateReadyFile,
}: {
  readiness: TaskPackageDispatchReadiness | null;
  loading: boolean;
  error: string | null;
  onInspect: () => void;
  onGenerateReadyFile?: () => void;
}) {
  const ready = readiness?.status === "ready";

  return (
    <div className="task-dispatch-readiness-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">派发准备</p>
          <h3>{ready ? "任务包可作为后续派发入口" : "任务包还不能派发"}</h3>
        </div>
        <Badge tone={ready ? "candidate" : "unknown"}>{readiness ? readiness.status : "未检查"}</Badge>
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onInspect}>
          检查派发准备
        </button>
        <button className="secondary-button" type="button" disabled={!ready} onClick={onGenerateReadyFile}>
          生成可派发版本
        </button>
      </div>
      {loading ? <p className="muted small-note">正在检查派发准备。</p> : null}
      {error ? <p className="state-warning">{error}</p> : null}
      {readiness ? (
        <TaskDispatchReadinessDetails readiness={readiness} />
      ) : (
        <p className="muted small-note">检查后才会显示就绪、未就绪或阻断。</p>
      )}
    </div>
  );
}

export function TaskDispatchReadinessDetails({ readiness }: { readiness: TaskPackageDispatchReadiness }) {
  const memorySummary = summarizeTaskPackageMemoryInjection(readiness.memory_injection_summary);
  return (
    <>
      {readiness.artifact_path ? (
        <details className="agent-boundary-details">
          <summary className="agent-boundary-summary">开发者详情</summary>
          <p className="path-text">{readiness.artifact_path}</p>
        </details>
      ) : null}
      <div className="workflow-compact-list" aria-label="任务包记忆注入摘要">
        <div className="workflow-compact-item">
          <strong>任务包记忆注入摘要 / {memorySummary.snapshot_id ?? "未生成"}</strong>
          <span>{memorySummary.display_text}</span>
          <em>仅启用态正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。</em>
        </div>
      </div>
      {readiness.blocking_reasons.length ? (
        <ul className="state-warning-list">
          {readiness.blocking_reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
      {readiness.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
    </>
  );
}

export function TaskDispatchFieldCorrectionEditor({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  const [previewFields, setPreviewFields] = useState<TaskPackageFields>(() => emptyCorrectionFields(selectedTaskDraft));

  useEffect(() => {
    setPreviewFields(emptyCorrectionFields(selectedTaskDraft));
  }, [selectedTaskDraft.work_item_id, selectedTaskDraft.title]);

  return (
    <TaskDispatchFieldCorrectionShell
      projectRoot={projectRoot}
      selectedTaskDraft={selectedTaskDraft}
      previewFields={previewFields}
      onPreviewFieldsChange={setPreviewFields}
      onRequestAction={onRequestAction}
    />
  );
}

export function TaskDispatchFieldCorrectionShell({
  projectRoot,
  selectedTaskDraft,
  previewFields,
  onPreviewFieldsChange,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  previewFields: TaskPackageFields;
  onPreviewFieldsChange: (fields: TaskPackageFields) => void;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <form
      className="task-fields-form"
      onChange={(event) => {
        onPreviewFieldsChange(fieldsFromForm(event.currentTarget));
      }}
      onSubmit={(event) => {
        event.preventDefault();
        const fields = fieldsFromForm(event.currentTarget);
        onPreviewFieldsChange(fields);
        onRequestAction({
          kind: "correct-dispatch-fields",
          label: "保存派发字段修正",
          path: projectRoot,
          source: "索引内项目路径",
          boundary:
            "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
          dispatchFields: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
            fields,
          },
        });
      }}
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">修正任务字段</p>
          <h3>保存前先看字段预览</h3>
        </div>
        <Badge tone="warning">不自动补编</Badge>
      </div>
      <div className="task-fields-grid">
        <label>
          <span>任务名</span>
          <input name="task_name" defaultValue={selectedTaskDraft.title} placeholder="待补充" />
        </label>
        <label>
          <span>所属开发线</span>
          <select name="assigned_line" defaultValue="桌面应用线">
            <option value="桌面应用线">桌面应用线</option>
            <option value="Codex 开发线">Codex 开发线</option>
          </select>
        </label>
        <FieldTextarea name="background" label="背景" />
        <FieldTextarea name="goals" label="目标" />
        <FieldTextarea name="allowed_read" label="允许读取" />
        <FieldTextarea name="allowed_write" label="允许写入" />
        <FieldTextarea name="forbidden_actions" label="禁止事项" />
        <FieldTextarea name="acceptance_criteria" label="验收标准" />
        <FieldTextarea name="required_return" label="必须回传" />
        <FieldTextarea name="review_focus" label="总指导回收重点" />
      </div>
      <TaskFieldCorrectionPreview fields={previewFields} />
      <div className="workflow-state-actions">
        <button className="primary-button" type="submit">
          保存派发字段修正
        </button>
      </div>
    </form>
  );
}

export function TaskFieldCorrectionPreview({ fields }: { fields: TaskPackageFields }) {
  const missing = missingCorrectionFields(fields);
  return (
    <div className="task-field-preview">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">字段级预览</p>
          <h3>{missing.length ? "仍有字段缺失" : "字段已填写，可复检 readiness"}</h3>
        </div>
        <Badge tone={missing.length ? "unknown" : "candidate"}>{missing.length ? "not_ready" : "ready 候选"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="任务名" value={fields.task_name || "待补充"} />
        <DetailLine label="所属开发线" value={fields.assigned_line || "未登记"} />
        <DetailLine label="目标" value={fields.goals.join(" / ") || "待补充"} />
        <DetailLine label="允许写入" value={fields.allowed_write.join(" / ") || "待补充"} />
        <DetailLine label="验收标准" value={fields.acceptance_criteria.join(" / ") || "待补充"} />
        <DetailLine label="必须回传" value={fields.required_return.join(" / ") || "待补充"} />
      </div>
      {missing.length ? (
        <ul className="state-warning-list">
          {missing.map((field) => (
            <li key={field}>{field}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

function TaskFieldsEditor({
  projectRoot,
  selectedTaskDraft,
  onRequestAction,
}: {
  projectRoot: string;
  selectedTaskDraft: TaskDraftSummary;
  onRequestAction: (action: PendingAction) => void;
}) {
  return (
    <form
      className="task-fields-form"
      onSubmit={(event) => {
        event.preventDefault();
        const formData = new FormData(event.currentTarget);
        const fields: TaskPackageFields = {
          task_name: scalarFormValue(formData, "task_name"),
          assigned_line: scalarFormValue(formData, "assigned_line"),
          background: listFormValue(formData, "background"),
          goals: listFormValue(formData, "goals"),
          allowed_read: listFormValue(formData, "allowed_read"),
          allowed_write: listFormValue(formData, "allowed_write"),
          forbidden_actions: listFormValue(formData, "forbidden_actions"),
          acceptance_criteria: listFormValue(formData, "acceptance_criteria"),
          required_return: listFormValue(formData, "required_return"),
          review_focus: listFormValue(formData, "review_focus"),
        };
        onRequestAction({
          kind: "update-task-fields",
          label: "保存任务包字段",
          path: projectRoot,
          source: "索引内项目路径",
          boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
          taskFields: {
            project_root: projectRoot,
            work_item_id: selectedTaskDraft.work_item_id,
            fields,
          },
        });
      }}
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">编辑字段</p>
          <h3>结构化字段是事实来源</h3>
        </div>
        <Badge tone="candidate">task_package_v1</Badge>
      </div>
      <div className="task-fields-grid">
        <label>
          <span>任务名</span>
          <input name="task_name" key={selectedTaskDraft.work_item_id} defaultValue={selectedTaskDraft.title} placeholder="待补充" />
        </label>
        <label>
          <span>所属开发线</span>
          <select name="assigned_line" defaultValue="Codex 开发线">
            <option value="Codex 开发线">Codex 开发线</option>
            <option value="桌面应用线">桌面应用线</option>
          </select>
        </label>
        <FieldTextarea name="background" label="背景" />
        <FieldTextarea name="goals" label="目标" />
        <FieldTextarea name="allowed_read" label="允许读取" />
        <FieldTextarea name="allowed_write" label="允许写入" />
        <FieldTextarea name="forbidden_actions" label="禁止事项" />
        <FieldTextarea name="acceptance_criteria" label="验收标准" />
        <FieldTextarea name="required_return" label="必须回传" />
        <FieldTextarea name="review_focus" label="总指导回收重点" />
      </div>
      <div className="workflow-state-actions">
        <button className="primary-button" type="submit">
          保存字段
        </button>
      </div>
    </form>
  );
}

export function nextSelectedWorkItemId(taskDrafts: TaskDraftSummary[], current: string | null): string | null {
  if (!taskDrafts.length) return null;
  if (current && taskDrafts.some((taskDraft) => taskDraft.work_item_id === current)) {
    return current;
  }
  return taskDrafts[0].work_item_id;
}

export function selectedTaskDraftFor(taskDrafts: TaskDraftSummary[], selectedWorkItemId: string | null): TaskDraftSummary | null {
  if (!selectedWorkItemId) return taskDrafts[0] ?? null;
  return taskDrafts.find((taskDraft) => taskDraft.work_item_id === selectedWorkItemId) ?? null;
}

function FieldTextarea({ name, label }: { name: keyof Omit<TaskPackageFields, "task_name" | "assigned_line">; label: string }) {
  return (
    <label>
      <span>{label}</span>
      <textarea name={name} rows={3} placeholder="每行一项；空白会保存为空，不会补编业务。" />
    </label>
  );
}

function emptyCorrectionFields(selectedTaskDraft: TaskDraftSummary): TaskPackageFields {
  return {
    task_name: selectedTaskDraft.title,
    assigned_line: "桌面应用线",
    background: [],
    goals: [],
    allowed_read: [],
    allowed_write: [],
    forbidden_actions: [],
    acceptance_criteria: [],
    required_return: [],
    review_focus: [],
  };
}

function fieldsFromForm(form: HTMLFormElement): TaskPackageFields {
  const formData = new FormData(form);
  return {
    task_name: scalarFormValue(formData, "task_name"),
    assigned_line: scalarFormValue(formData, "assigned_line"),
    background: listFormValue(formData, "background"),
    goals: listFormValue(formData, "goals"),
    allowed_read: listFormValue(formData, "allowed_read"),
    allowed_write: listFormValue(formData, "allowed_write"),
    forbidden_actions: listFormValue(formData, "forbidden_actions"),
    acceptance_criteria: listFormValue(formData, "acceptance_criteria"),
    required_return: listFormValue(formData, "required_return"),
    review_focus: listFormValue(formData, "review_focus"),
  };
}

export function missingCorrectionFields(fields: TaskPackageFields): string[] {
  const missing: string[] = [];
  if (!fields.task_name.trim()) missing.push("任务名缺失");
  if (!fields.assigned_line.trim()) missing.push("所属开发线缺失");
  if (!fields.background.length) missing.push("背景缺失");
  if (!fields.goals.length) missing.push("目标缺失");
  if (!fields.allowed_read.length) missing.push("允许读取缺失");
  if (!fields.allowed_write.length) missing.push("允许写入缺失");
  if (!fields.forbidden_actions.length) missing.push("禁止事项缺失");
  if (!fields.acceptance_criteria.length) missing.push("验收标准缺失");
  if (!fields.required_return.length) missing.push("必须回传缺失");
  if (!fields.review_focus.length) missing.push("总指导回收重点缺失");
  return missing;
}

function scalarFormValue(formData: FormData, key: string): string {
  return String(formData.get(key) ?? "").trim();
}

function listFormValue(formData: FormData, key: string): string[] {
  return String(formData.get(key) ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
