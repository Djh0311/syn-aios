import { Pill } from "./SpecPrimitives";
import type { PendingAction, WorkflowStateSnapshot } from "../lib/types";

type WorkflowStatePanelProps = {
  workflowState: WorkflowStateSnapshot | null;
  loading: boolean;
  error: string | null;
  onReload: () => void;
  onRequestAction: (action: PendingAction) => void;
};

export function WorkflowStatePanel({
  workflowState,
  loading,
  error,
  onReload,
  onRequestAction,
}: WorkflowStatePanelProps) {
  const path =
    workflowState?.path ||
    "/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json";

  return (
    <section className="workflow-state-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">本地事实层 v0</p>
          <h3>工作流状态文件</h3>
        </div>
        <Pill tone={workflowState?.exists ? "candidate" : "unknown"}>
          {workflowState?.exists ? "已初始化" : "未初始化"}
        </Pill>
      </div>

      <div className="workflow-state-grid">
        <StateCell label="状态文件路径" value={path} />
        <StateCell label="存在状态" value={workflowState?.exists ? "存在" : "不存在"} />
        <StateCell label="结构版本" value={workflowState?.schema_version || "未初始化"} />
        <StateCell label="工作流版本" value={workflowState?.workflow_version ? String(workflowState.workflow_version) : "未初始化"} />
        <StateCell label="工作流" value={String(workflowState?.counts.workflows ?? 0)} />
        <StateCell label="节点" value={String(workflowState?.counts.nodes ?? 0)} />
        <StateCell label="连线" value={String(workflowState?.counts.edges ?? 0)} />
        <StateCell label="复核" value={String(workflowState?.counts.reviews ?? 0)} />
        <StateCell label="审计事件" value={String(workflowState?.counts.audit_events ?? 0)} />
        <StateCell label="运行器资源" value={String(workflowState?.counts.harness_resources ?? 0)} />
      </div>

      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onReload} disabled={loading}>
          {loading ? "读取中" : "重新读取事实层"}
        </button>
        <button
          className="primary-button"
          type="button"
          onClick={() =>
            onRequestAction({
              kind: "initialize-workflow-state",
              label: "初始化工作流事实层",
              path,
              source: "Tauri 应用数据目录",
              boundary:
                "只写 workflow-state.v0.json 和同目录备份；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
            })
          }
        >
          初始化工作流事实层
        </button>
      </div>

      {error ? <p className="state-warning">读取事实层失败：{error}</p> : null}
      {(workflowState?.warnings ?? ["状态文件不存在；不会自动创建。"]).map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
    </section>
  );
}

function StateCell({ label, value }: { label: string; value: string }) {
  return (
    <div className="state-cell">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
