import { Badge } from "../../components/Badge";
import type { TaskPackage, WorkflowStateSnapshot } from "../../lib/types";
import { DetailLine, WorkflowNode, listText, runCheckTone } from "./projectWorkflowLabels";

type DerivedWorkflow = NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]>;

export function ProjectCanvasDerivedSummary({
  workflow,
  selectedTaskPackage,
}: {
  workflow: DerivedWorkflow;
  selectedTaskPackage: TaskPackage | null;
}) {
  const gate = workflow.state_machine.completion_gate;
  return (
    <section className="project-canvas-detail-card">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作流详情摘要</p>
          <h3>{workflow.title}</h3>
        </div>
        <Badge tone={runCheckTone(workflow.run_check_status)}>{workflow.run_check_status}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="任务包" value={selectedTaskPackage?.task_package_id ?? `${workflow.task_packages.length} 个`} />
        <DetailLine label="账本" value={`${workflow.ledger_entries.length} 条摘要`} />
        <DetailLine label="子汇报" value={`${workflow.subagent_reports.length} 条`} />
        <DetailLine label="审查" value={`${workflow.review_results.length} 条`} />
        <DetailLine label="异常" value={`${workflow.exceptions.length} 条`} />
        <DetailLine label="完成闸门" value={gate.can_complete ? "可完成" : gate.missing.join("；") || "缺少条件"} />
      </div>
      {workflow.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      <p className="muted small-note">任务包、账本、状态机、子汇报和黑板候选只在详情侧展示；主区域只保留项目画布。</p>
    </section>
  );
}

export function DerivedWorkflowSummary({
  workflow,
  selectedTaskPackage,
}: {
  workflow: DerivedWorkflow;
  selectedTaskPackage: TaskPackage | null;
}) {
  return (
    <section className="derived-workflow-summary">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">派生 v1 读模型</p>
          <h3>{workflow.title}</h3>
        </div>
        <Badge tone={runCheckTone(workflow.run_check_status)}>{workflow.run_check_status}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="节点" value={`${workflow.nodes.length} 个`} />
        <DetailLine label="任务包" value={`${workflow.task_packages.length} 个`} />
        <DetailLine label="当前阶段" value={workflow.current_stage || "未登记"} />
        <DetailLine label="owner" value={workflow.owner_role || "未登记"} />
        <DetailLine label="风险" value={workflow.risk_level || "未登记"} />
      </div>
      {workflow.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
      {selectedTaskPackage ? (
        <TaskPackageReadModelPreview taskPackage={selectedTaskPackage} />
      ) : (
        <p className="muted small-note">派生读模型里还没有任务包；不会根据草稿标题自动生成业务事实。</p>
      )}
      <WorkflowBlueprintCanvas workflow={workflow} selectedTaskPackage={selectedTaskPackage} />
      <WorkflowLedgerPanel workflow={workflow} />
      <WorkflowReportReviewExceptionPanel workflow={workflow} />
      <WorkflowStateMachinePanel workflow={workflow} />
      <WorkflowInterfaceBoundaryPanel workflow={workflow} />
      <WorkflowAcceptanceScenarioPanel workflow={workflow} />
    </section>
  );
}

function TaskPackageReadModelPreview({ taskPackage }: { taskPackage: TaskPackage }) {
  return (
    <div className="task-package-read-model-preview">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">任务包预览字段</p>
          <h3>{taskPackage.task_goal || "任务目标未登记"}</h3>
        </div>
        <Badge tone={taskPackage.stale || taskPackage.missing_fields.length ? "warning" : "candidate"}>
          v{taskPackage.version} / {taskPackage.stale ? "过期" : "新鲜"}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="模型" value={taskPackage.model_id || "缺失：缺模型"} />
        <DetailLine label="允许读取" value={listText(taskPackage.allowed_read_scope, "缺失：缺读范围")} />
        <DetailLine label="允许写入" value={listText(taskPackage.allowed_write_scope, "缺失：缺写范围")} />
        <DetailLine label="工具白名单" value={listText(taskPackage.callable_tool_capabilities, "空：未声明工具")} />
        <DetailLine label="技能" value={listText(taskPackage.available_skills, "空：未声明技能")} />
        <DetailLine label="知识库引用" value={listText(taskPackage.available_knowledge_refs, "空：未声明知识库")} />
        <DetailLine label="记忆引用" value={listText(taskPackage.available_memory_refs, "空：未声明记忆")} />
        <DetailLine label="运行器" value={listText(taskPackage.harness_requirements, "空：未要求运行器")} />
        <DetailLine label="验收标准" value={listText(taskPackage.acceptance_criteria, "缺失：缺验收标准")} />
        <DetailLine label="回传格式" value={listText(taskPackage.report_format, "缺失：缺回传格式")} />
        <DetailLine label="禁止事项" value={listText(taskPackage.forbidden_actions, "缺失：缺禁止事项")} />
        <DetailLine label="超时策略" value={taskPackage.timeout_policy || "未登记"} />
        <DetailLine label="失败策略" value={taskPackage.failure_policy || "未登记"} />
      </div>
      {taskPackage.missing_fields.length ? (
        <ul className="state-warning-list">
          {taskPackage.missing_fields.map((field) => (
            <li key={field}>缺失：{field}</li>
          ))}
        </ul>
      ) : null}
      {taskPackage.stale ? (
        <p className="state-warning">
          任务包已过期；人工编辑或节点、权限、模型、知识库、记忆、运行器、验收标准变化后必须重新检查。
        </p>
      ) : null}
      {taskPackage.stale_reasons.map((reason) => (
        <p className="state-warning" key={reason}>
          过期原因：{reason}
        </p>
      ))}
    </div>
  );
}

function WorkflowBlueprintCanvas({
  workflow,
  selectedTaskPackage,
}: {
  workflow: DerivedWorkflow;
  selectedTaskPackage: TaskPackage | null;
}) {
  const mainNodes = [
    { id: "consultation", title: "consultation", detail: "方案 / 方向确认", tone: "gap" as const },
    { id: "director", title: "director", detail: "项目主管", tone: "project" as const },
    { id: "subagent", title: "subagent", detail: "执行子智能体", tone: "codex" as const },
    { id: "review", title: "review", detail: "审查", tone: "artifact" as const },
    { id: "report", title: "汇报", detail: "最终汇报", tone: "harness" as const },
  ];
  return (
    <div className="workflow-blueprint-canvas">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目工作流画布</p>
          <h3>方案视图 / 运行状态视图</h3>
        </div>
        <Badge tone="unknown">项目事实</Badge>
      </div>
      <div className="workflow-state-actions" aria-label="工作流视图切换">
        <button className="secondary-button" type="button">方案视图</button>
        <button className="secondary-button" type="button">运行状态视图</button>
      </div>
      <div className="workflow-blueprint-nodes">
        {mainNodes.map((node) => (
          <WorkflowNode key={node.id} title={node.title} detail={node.detail} meta="主节点" tone={node.tone} />
        ))}
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="规则违反数量" value={String(workflow.state_machine.completion_gate.missing.length)} />
        <DetailLine label="证据完整度" value={workflow.state_machine.completion_gate.can_complete ? "完整" : "缺失"} />
        <DetailLine label="运行检查" value={workflow.run_check_status} />
        <DetailLine label="任务包" value={selectedTaskPackage?.task_package_id ?? "未选择"} />
        <DetailLine label="事实源" value="项目工作流状态 / 派生读模型" />
        <DetailLine label="画布边界" value="不做通用节点执行器" />
      </div>
      <div className="node-detail-panel">
        <p className="eyebrow">节点详情</p>
        <div className="workflow-draft-grid">
          <DetailLine label="知识权限" value={selectedTaskPackage ? listText(selectedTaskPackage.available_knowledge_refs, "空：显式资料引用为空") : "未选择任务包"} />
          <DetailLine label="tool permission" value={selectedTaskPackage ? listText(selectedTaskPackage.callable_tool_capabilities, "empty：没有工具白名单") : "未选择任务包"} />
          <DetailLine label="model" value={selectedTaskPackage?.model_id || "missing：必须显式指定"} />
          <DetailLine label="skills" value={selectedTaskPackage ? listText(selectedTaskPackage.available_skills, "empty：未声明技能") : "未选择任务包"} />
          <DetailLine label="验收标准" value={selectedTaskPackage ? listText(selectedTaskPackage.acceptance_criteria, "缺失：缺验收标准") : "未选择任务包"} />
          <DetailLine label="复核要求" value={workflow.review_results.length ? `${workflow.review_results.length} 条审查结果` : "未登记"} />
          <DetailLine label="运行器要求" value={selectedTaskPackage ? listText(selectedTaskPackage.harness_requirements, "空：运行器不是普通节点") : "未选择任务包"} />
          <DetailLine label="账本记录" value={`${workflow.ledger_entries.length} 条摘要`} />
          <DetailLine label="审计链接" value={workflow.ledger_entries.flatMap((entry) => entry.audit_refs).slice(0, 2).join("；") || "未登记"} />
        </div>
      </div>
      <p className="muted small-note">手动确认、知识读取、工具调用、普通权限读取不作为默认主节点；运行器只影响检查、任务包模板和完成判定。</p>
    </div>
  );
}

function WorkflowLedgerPanel({ workflow }: { workflow: DerivedWorkflow }) {
  return (
    <div className="workflow-ledger-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作流账本</p>
          <h3>只追加摘要和引用</h3>
        </div>
        <Badge tone="unknown">{workflow.ledger_entries.length} 条</Badge>
      </div>
      <div className="workflow-compact-list">
        {workflow.ledger_entries.slice(0, 6).map((entry) => (
          <div className="workflow-compact-item" key={entry.ledger_entry_id}>
            <strong>{entry.entry_type}</strong>
            <span>{entry.summary || "未登记摘要"}</span>
            <em>来源：{entry.source_refs.join("；") || "无"} / 审计：{entry.audit_refs.join("；") || "无"} / 工具：{entry.tool_call_refs.join("；") || "无全文"}</em>
          </div>
        ))}
        {!workflow.ledger_entries.length ? <p className="muted small-note">暂无账本摘要；不会把工具输出全文铺进画布。</p> : null}
      </div>
    </div>
  );
}

function WorkflowReportReviewExceptionPanel({ workflow }: { workflow: DerivedWorkflow }) {
  return (
    <div className="workflow-report-review-grid">
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">子智能体汇报</p>
            <h3>只能提交汇报、风险和权限请求</h3>
          </div>
          <Badge tone="unknown">{workflow.subagent_reports.length}</Badge>
        </div>
        {workflow.subagent_reports.slice(0, 3).map((report) => (
          <div className="workflow-compact-item" key={report.report_id}>
            <strong>{report.actor_role || "unknown"} / {report.acceptance_status}</strong>
            <span>{report.summary}</span>
            <em>证据：{report.evidence_refs.join("；") || "无"} / 风险：{report.direction_risks.join("；") || "无"}</em>
          </div>
        ))}
        {!workflow.subagent_reports.length ? <p className="muted small-note">暂无子智能体汇报。</p> : null}
      </div>
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">审查结果</p>
            <h3>通过不等于完成</h3>
          </div>
          <Badge tone="unknown">{workflow.review_results.length}</Badge>
        </div>
        {workflow.review_results.slice(0, 3).map((review) => (
          <div className="workflow-compact-item" key={review.review_id}>
            <strong>{review.result}</strong>
            <span>{review.summary || "未登记摘要"}</span>
            <em>{review.requires_director_confirmation ? "仍需项目主管确认" : "无需主管确认"} / can_complete={String(review.can_complete_node)}</em>
          </div>
        ))}
        {!workflow.review_results.length ? <p className="muted small-note">暂无审查结果。</p> : null}
      </div>
      <div className="workflow-report-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">异常通知</p>
            <h3>待办中心 / 运行中入口</h3>
          </div>
          <Badge tone={workflow.exceptions.length ? "warning" : "candidate"}>{workflow.exceptions.length}</Badge>
        </div>
        {workflow.exceptions.slice(0, 4).map((exception) => (
          <div className="workflow-compact-item" key={exception.exception_id}>
            <strong>{exception.exception_type} / {exception.status}</strong>
            <span>{exception.summary}</span>
            {exception.warnings.length ? <em>{exception.warnings.join("；")}</em> : null}
          </div>
        ))}
        {!workflow.exceptions.length ? <p className="muted small-note">暂无异常、待处理确认或运行中阻塞。</p> : null}
      </div>
    </div>
  );
}

function WorkflowStateMachinePanel({ workflow }: { workflow: DerivedWorkflow }) {
  const gate = workflow.state_machine.completion_gate;
  return (
    <div className="workflow-state-machine-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">状态机和完成判定</p>
          <h3>{gate.can_complete ? "项目主管可确认完成" : "项目主管完成闸门未满足"}</h3>
        </div>
        <Badge tone={gate.can_complete ? "candidate" : "warning"}>{gate.can_complete ? "可完成" : "阻断"}</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="工作流允许迁移" value={workflow.state_machine.workflow_allowed_transitions.slice(0, 4).join("；")} />
        <DetailLine label="工作流拒绝迁移" value={workflow.state_machine.workflow_rejected_transitions.join("；")} />
        <DetailLine label="节点允许迁移" value={workflow.state_machine.node_allowed_transitions.slice(0, 4).join("；")} />
        <DetailLine label="节点拒绝迁移" value={workflow.state_machine.node_rejected_transitions.join("；")} />
        <DetailLine label="缺失项" value={gate.missing.join("；") || "无"} />
      </div>
      {workflow.state_machine.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </div>
  );
}

function WorkflowInterfaceBoundaryPanel({ workflow }: { workflow: DerivedWorkflow }) {
  const boundaries = workflow.interface_boundaries;
  const rows = [
    boundaries.proposal_interface,
    boundaries.memory_candidate_interface,
    boundaries.knowledge_refs_interface,
    boundaries.tool_capability_registry,
    boundaries.model_pool_selector,
    boundaries.harness_requirement_provider,
    boundaries.audit_refs_interface,
  ];
  return (
    <div className="workflow-interface-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">接口边界</p>
          <h3>保守默认</h3>
        </div>
        <Badge tone="unknown">桩执行</Badge>
      </div>
      <div className="workflow-compact-list">
        {rows.map((boundary) => (
          <div className="workflow-compact-item" key={boundary.interface_id}>
            <strong>{boundary.interface_id}</strong>
            <span>允许：{boundary.allowed.join("；") || "无"}</span>
            <em>阻止：{boundary.blocked.join("；") || "无"}</em>
          </div>
        ))}
      </div>
      {boundaries.warnings.map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </div>
  );
}

function WorkflowAcceptanceScenarioPanel({ workflow }: { workflow: DerivedWorkflow }) {
  return (
    <div className="workflow-acceptance-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">端到端验收场景</p>
          <h3>测试样例和界面展示验收</h3>
        </div>
        <Badge tone="unknown">{workflow.acceptance_scenarios.length}</Badge>
      </div>
      <div className="workflow-compact-list">
        {workflow.acceptance_scenarios.map((scenario) => (
          <div className="workflow-compact-item" key={scenario.scenario_id}>
            <strong>{scenario.scenario_id} / {scenario.title}</strong>
            <span>{scenario.status}</span>
            <em>{scenario.expected.join("；")}</em>
          </div>
        ))}
      </div>
    </div>
  );
}
