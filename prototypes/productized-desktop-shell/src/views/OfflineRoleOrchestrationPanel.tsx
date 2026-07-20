import { Pill } from "../components/SpecPrimitives";
import { DetailLine } from "../components/WorkbenchPrimitives";
import type { OfflineRoleDispatchProposal, PendingAction, ProjectRecord, ProjectWorkflowSummary, SessionRecord } from "../lib/types";

type RoleDefinition = {
  role_id: string;
  label: string;
  description: string;
};

type ParseResult =
  | {
      ok: true;
      proposal: OfflineRoleDispatchProposal;
      missing: [];
    }
  | {
      ok: false;
      proposal: null;
      missing: string[];
    };

type OfflineRoleResult = {
  role_label: string;
  summary: string;
  returned_to_director: string;
};

const roleDefinitions: RoleDefinition[] = [
  {
    role_id: "director",
    label: "总指导",
    description: "接收需求、拆计划、回收结果",
  },
  {
    role_id: "codex-dev",
    label: "开发线",
    description: "执行用户审核后的具体改动",
  },
  {
    role_id: "validation",
    label: "验证线",
    description: "检查结果、验证边界和失败原因",
  },
  {
    role_id: "review",
    label: "回收线",
    description: "整理 evidence、handoff 和回收意见",
  },
];

const requiredFields = [
  "派发给",
  "任务名",
  "目标",
  "执行目录",
  "允许读取",
  "允许写入",
  "禁止事项",
  "验收标准",
  "超时",
  "回传要求",
];

export const defaultOfflineDispatchBlock = `派发给：开发线
任务名：README 极小修改验证
目标：在 README 末尾追加一行指定文本，并回传修改结果。
执行目录：/Users/yoyi/codex-workflow-mario-test
允许读取：/Users/yoyi/codex-workflow-mario-test/README.md
允许写入：/Users/yoyi/codex-workflow-mario-test/README.md
禁止事项：不读取 auth.json、.env、密钥、token 或授权文件；不读取完整会话记录；不修改其他文件
验收标准：README 出现目标行；允许范围外文件不变化
超时：600
回传要求：薄弱点；改了哪些文件；验证结果；风险`;

export function OfflineRoleOrchestrationPanel({
  project,
  projectWorkflow = null,
  sessions,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow?: ProjectWorkflowSummary | null;
  sessions: SessionRecord[];
  onRequestAction: (action: PendingAction) => void;
}) {
  const parseResult = parseOfflineDispatchBlock(defaultOfflineDispatchBlock, project.project_root);
  const proposal = parseResult.ok ? parseResult.proposal : null;
  const roleResult = proposal ? buildOfflineStubResult(proposal) : null;
  const selectedWorkItem = selectedOfflineWorkItem(projectWorkflow);
  const preparedOfflineDispatch = selectedWorkItem
    ? projectWorkflow?.node_dispatches.find(
        (dispatch) =>
          dispatch.work_item_id === selectedWorkItem.work_item_id &&
          dispatch.prompt_kind === "offline_role_dispatch" &&
          dispatch.state === "prepared",
      ) ?? null
    : null;
  const completedOfflineDispatch = selectedWorkItem
    ? projectWorkflow?.node_dispatches.find(
        (dispatch) =>
          dispatch.work_item_id === selectedWorkItem.work_item_id &&
          dispatch.prompt_kind === "offline_role_dispatch" &&
          dispatch.state === "completed",
      ) ?? null
    : null;
  const reviewedOfflineDispatch = completedOfflineDispatch
    ? projectWorkflow?.director_reviews.find((review) => review.dispatch_id === completedOfflineDispatch.dispatch_id) ?? null
    : null;
  const preparedProposal = preparedOfflineDispatch?.offline_role_dispatch ?? proposal;
  const preparedRoleResult = preparedProposal ? buildOfflineStubResult(preparedProposal) : null;
  const canPrepareOfflineDispatch =
    Boolean(proposal) &&
    selectedWorkItem?.state === "ready_to_dispatch" &&
    !preparedOfflineDispatch &&
    !completedOfflineDispatch;

  function proposalFromForm(form: HTMLFormElement) {
    const formData = new FormData(form);
    const rawBlock = String(formData.get("dispatch-block") ?? "");
    const result = parseOfflineDispatchBlock(rawBlock, project.project_root);
    return result.ok ? result.proposal : null;
  }

  return (
    <form
      className="offline-role-orchestration-panel"
      aria-label="Codex 角色编排离线闭环"
      onSubmit={(event) => {
        event.preventDefault();
        const currentProposal = proposalFromForm(event.currentTarget);
        if (!currentProposal || !selectedWorkItem || !canPrepareOfflineDispatch) return;
        onRequestAction(buildOfflineRoleDispatchAction(project.project_root, selectedWorkItem.work_item_id, currentProposal));
      }}
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Codex 角色编排</p>
          <h3>总指导派发闭环</h3>
          <p className="path-text">离线演示：确认后只写工作台自己的 workflow-state.v0.json；不启动 Codex、不恢复会话、不写 /Users/yoyi/.codex。</p>
        </div>
        <Pill tone={proposal ? "candidate" : "warn"}>{proposal ? "派发块有效" : "缺字段"}</Pill>
      </div>

      <div className="offline-role-grid">
        {roleDefinitions.map((role) => {
          const matchingSessions = sessionsForRole(role, sessions);
          return (
            <div className="offline-role-card" key={role.role_id}>
              <span>{role.label}</span>
              <strong>{role.description}</strong>
              <em>{matchingSessions.length ? matchingSessions[0].title : "未绑定真实会话；本区只做离线编排"}</em>
            </div>
          );
        })}
      </div>

      <div className="offline-orchestration-grid">
        <label>
          <span>发给总指导</span>
          <textarea
            name="director-request"
            rows={8}
            defaultValue=""
            placeholder="在这里写需求。当前只是记录输入，不让 AI 自动补编计划。"
          />
        </label>
        <label>
          <span>总指导回复里的派发块</span>
          <textarea name="dispatch-block" rows={12} defaultValue={defaultOfflineDispatchBlock} />
        </label>
      </div>

      <div className="workflow-state-actions">
        <button className="secondary-button" type="submit" disabled={!canPrepareOfflineDispatch}>
          写入离线派发
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={!preparedOfflineDispatch || !selectedWorkItem}
          onClick={() => {
            if (!preparedOfflineDispatch || !selectedWorkItem || !preparedProposal || !preparedRoleResult) return;
            onRequestAction(
              buildOfflineRoleResultHandoffAction(
                project.project_root,
                selectedWorkItem.work_item_id,
                preparedOfflineDispatch.dispatch_id,
                preparedProposal,
                preparedRoleResult,
              ),
            );
          }}
        >
          写入角色回传
        </button>
        <button
          className="primary-button"
          type="button"
          disabled={!completedOfflineDispatch || !selectedWorkItem || Boolean(reviewedOfflineDispatch)}
          onClick={() => {
            if (!completedOfflineDispatch || !selectedWorkItem) return;
            onRequestAction(buildOfflineDirectorReviewAction(project.project_root, selectedWorkItem.work_item_id, completedOfflineDispatch.dispatch_id));
          }}
        >
          写入总指导回收
        </button>
      </div>

      {selectedWorkItem ? (
        <div className="dispatch-preview-block">
          <span>账本锚点</span>
          <strong>{selectedWorkItem.title}</strong>
          <em>{selectedWorkItem.state}</em>
          {/* 「开发者详情」折叠已废除（DESIGN.md §三·五）：工作项编号等机器信息归审计账本。 */}
        </div>
      ) : (
        <p className="state-warning">当前没有可编排工作项；离线编排只能预览，不能写入工作台事实层。</p>
      )}
      <p className="muted small-note">总指导输入只作为本地表单内容；提交时按当前派发块解析，不合法就不会生成待确认动作。下方预览来自默认示例，不代表文本框实时内容。</p>

      {parseResult.ok ? (
        <OfflineDispatchProposalPreview proposal={parseResult.proposal} />
      ) : (
        <div className="dispatch-preview-block">
          <span>派发块缺字段</span>
          <strong>{parseResult.missing.join("；") || "未知缺失"}</strong>
          <em>解析器只认固定字段，不从自由文本里猜业务目标。</em>
        </div>
      )}

      {roleResult ? (
        <div className="offline-role-result-card">
          <span>角色回传</span>
          <strong>{roleResult.role_label}</strong>
          <em>{roleResult.summary}</em>
          <span>回传总指导</span>
          <strong>{roleResult.returned_to_director}</strong>
        </div>
      ) : null}
      <OfflineLedgerPreview
        preparedDispatchId={preparedOfflineDispatch?.dispatch_id ?? null}
        completedDispatchId={completedOfflineDispatch?.dispatch_id ?? null}
        reviewDecision={reviewedOfflineDispatch?.decision ?? null}
      />
    </form>
  );
}

function OfflineLedgerPreview({
  preparedDispatchId,
  completedDispatchId,
  reviewDecision,
}: {
  preparedDispatchId: string | null;
  completedDispatchId: string | null;
  reviewDecision: string | null;
}) {
  return (
    <div className="dispatch-preview-block">
      <span>离线编排账本</span>
      <strong>prepared：{preparedDispatchId || "未写入"}</strong>
      <em>completed：{completedDispatchId || "未回传"}</em>
      <em>review：{reviewDecision || "未回收"}</em>
    </div>
  );
}

export function OfflineDispatchProposalPreview({ proposal }: { proposal: OfflineRoleDispatchProposal }) {
  return (
    <div className="offline-dispatch-preview">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">派发预览</p>
          <h3>{proposal.task_title}</h3>
        </div>
        <Pill tone="candidate">{proposal.target_role_label}</Pill>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="目标角色" value={`${proposal.target_role_label} / ${proposal.target_role_id}`} emptyValue="未登记" />
        <DetailLine label="执行目录" value={proposal.execution_cwd} emptyValue="未登记" />
        <DetailLine label="超时" value={`${proposal.timeout_seconds} 秒`} emptyValue="未登记" />
        <DetailLine label="允许读取" value={proposal.allowed_reads.join("；")} emptyValue="未登记" />
        <DetailLine label="允许写入" value={proposal.allowed_writes.join("；")} emptyValue="未登记" />
      </div>
      <div className="dispatch-preview-block">
        <span>目标</span>
        <strong>{proposal.objective}</strong>
      </div>
      <div className="dispatch-preview-block">
        <span>禁止事项</span>
        <strong>{proposal.forbidden_actions.join("；")}</strong>
      </div>
      <div className="dispatch-preview-block">
        <span>验收标准</span>
        <strong>{proposal.acceptance_criteria.join("；")}</strong>
      </div>
      <div className="dispatch-preview-block">
        <span>回传要求</span>
        <strong>{proposal.required_return.join("；")}</strong>
      </div>
    </div>
  );
}

export function parseOfflineDispatchBlock(rawBlock: string, projectRoot: string): ParseResult {
  const fields = parseFixedFields(rawBlock);
  const missing = requiredFields.filter((field) => !fields.get(field)?.trim());
  const role = roleFromLabel(fields.get("派发给") ?? "");
  if (!role && !missing.includes("派发给")) missing.push("派发给");
  const timeoutSeconds = Number.parseInt(fields.get("超时") ?? "", 10);
  if (!Number.isFinite(timeoutSeconds) || timeoutSeconds <= 0) missing.push("超时");
  if (missing.length) {
    return { ok: false, proposal: null, missing: Array.from(new Set(missing)) };
  }

  return {
    ok: true,
    missing: [],
    proposal: {
      project_root: projectRoot,
      target_role_id: role?.role_id ?? "unknown",
      target_role_label: role?.label ?? fields.get("派发给") ?? "未知角色",
      task_title: fields.get("任务名") ?? "",
      objective: fields.get("目标") ?? "",
      execution_cwd: fields.get("执行目录") ?? "",
      allowed_reads: splitListField(fields.get("允许读取") ?? ""),
      allowed_writes: splitListField(fields.get("允许写入") ?? ""),
      forbidden_actions: splitListField(fields.get("禁止事项") ?? ""),
      acceptance_criteria: splitListField(fields.get("验收标准") ?? ""),
      timeout_seconds: timeoutSeconds,
      required_return: splitListField(fields.get("回传要求") ?? ""),
      raw_block: rawBlock,
    },
  };
}

export function buildOfflineRoleDispatchAction(projectRoot: string, workItemId: string, proposal: OfflineRoleDispatchProposal): PendingAction {
  return {
    kind: "offline-role-dispatch",
    label: `离线派发给${proposal.target_role_label}`,
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只把角色派发块写入工作台自己的 workflow-state.v0.json；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。",
    offlineRoleDispatch: {
      ...proposal,
      work_item_id: workItemId,
    },
  };
}

export function buildOfflineRoleResultHandoffAction(
  projectRoot: string,
  workItemId: string,
  dispatchId: string,
  proposal: OfflineRoleDispatchProposal,
  result: OfflineRoleResult,
): PendingAction {
  return {
    kind: "offline-role-result-handoff",
    label: `离线记录${proposal.target_role_label}回传`,
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只把离线角色回传写入工作台自己的 workflow-state.v0.json；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。",
    offlineRoleResultHandoff: {
      project_root: projectRoot,
      work_item_id: workItemId,
      dispatch_id: dispatchId,
      target_role_id: proposal.target_role_id,
      summary: result.summary,
      markdown: `${result.summary}\n\n${result.returned_to_director}`,
    },
  };
}

export function buildOfflineDirectorReviewAction(projectRoot: string, workItemId: string, dispatchId: string): PendingAction {
  return {
    kind: "offline-director-review",
    label: "离线记录总指导回收",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只把离线总指导回收写入工作台自己的 workflow-state.v0.json；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。",
    offlineDirectorReview: {
      project_root: projectRoot,
      work_item_id: workItemId,
      dispatch_id: dispatchId,
      decision: "accepted",
      summary: "离线总指导回收：接受角色桩结果，继续下一步编排。",
    },
  };
}

export function buildOfflineStubResult(proposal: OfflineRoleDispatchProposal): OfflineRoleResult {
  return {
    role_label: proposal.target_role_label,
    summary: `离线桩结果：已接收《${proposal.task_title}》，没有执行真实 Codex 会话。`,
    returned_to_director: `请总指导回收：目标=${proposal.objective}；要求回传=${proposal.required_return.join("；")}。`,
  };
}

function parseFixedFields(rawBlock: string) {
  const fields = new Map<string, string>();
  for (const line of rawBlock.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const separatorIndex = trimmed.search(/[：:]/);
    if (separatorIndex <= 0) continue;
    const key = trimmed.slice(0, separatorIndex).trim();
    const value = trimmed.slice(separatorIndex + 1).trim();
    if (requiredFields.includes(key)) fields.set(key, value);
  }
  return fields;
}

function roleFromLabel(label: string): RoleDefinition | null {
  const normalized = label.trim().toLowerCase();
  if (!normalized) return null;
  if (["总指导", "director"].includes(normalized)) return roleDefinitions[0];
  if (["开发线", "developer", "codex-dev", "dev"].includes(normalized)) return roleDefinitions[1];
  if (["验证线", "validation", "verifier"].includes(normalized)) return roleDefinitions[2];
  if (["回收线", "review", "reviewer"].includes(normalized)) return roleDefinitions[3];
  return null;
}

function splitListField(value: string) {
  return value
    .split(/[；;\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function sessionsForRole(role: RoleDefinition, sessions: SessionRecord[]) {
  if (role.role_id === "director") return sessions.filter((session) => session.title.includes("总指导") || session.title.toLowerCase().includes("director"));
  if (role.role_id === "validation") return sessions.filter((session) => session.title.includes("验证") || session.title.toLowerCase().includes("valid"));
  if (role.role_id === "review") return sessions.filter((session) => session.title.includes("回收") || session.title.toLowerCase().includes("review"));
  return sessions.filter((session) => session.title.includes("开发") || session.title.toLowerCase().includes("dev"));
}

function selectedOfflineWorkItem(projectWorkflow: ProjectWorkflowSummary | null) {
  if (!projectWorkflow) return null;
  const completedOfflineDispatch = projectWorkflow.node_dispatches.find(
    (dispatch) =>
      dispatch.prompt_kind === "offline_role_dispatch" &&
      dispatch.state === "completed" &&
      !projectWorkflow.director_reviews.some((review) => review.dispatch_id === dispatch.dispatch_id),
  );
  const completedWorkItem = workItemForDispatch(projectWorkflow, completedOfflineDispatch);
  if (completedWorkItem) return completedWorkItem;
  const preparedOfflineDispatch = projectWorkflow.node_dispatches.find(
    (dispatch) => dispatch.prompt_kind === "offline_role_dispatch" && dispatch.state === "prepared",
  );
  const preparedWorkItem = workItemForDispatch(projectWorkflow, preparedOfflineDispatch);
  if (preparedWorkItem) return preparedWorkItem;
  return projectWorkflow.task_drafts.find((taskDraft) => taskDraft.state === "ready_to_dispatch") ?? null;
}

function workItemForDispatch(projectWorkflow: ProjectWorkflowSummary, dispatch: { work_item_id: string } | undefined) {
  if (!dispatch) return null;
  return projectWorkflow.task_drafts.find((taskDraft) => taskDraft.work_item_id === dispatch.work_item_id) ?? null;
}
