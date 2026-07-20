import { Pill } from "../../components/SpecPrimitives";
import type { MemoryManagementSummary } from "../../lib/memoryCenter";
import type { MemoryCenterPageReadModel } from "../../lib/pageSelectors";

const badgePillTone = { neutral: "plain", candidate: "candidate", warning: "warn", unknown: "unknown" } as const;

export function MemoryCenterStats({ pageReadModel }: { pageReadModel: MemoryCenterPageReadModel }) {
  return (
    <div className="stat-strip memory-center-stats">
      <StatCell label="正式" value={`${pageReadModel.formal_memory.record_count}`} helper="正式记忆" />
      <StatCell label="活跃" value={`${pageReadModel.formal_memory.active_count}`} helper="可评估入包" />
      <StatCell label="候选" value={`${pageReadModel.candidate_memory.candidate_count}`} helper="候选记忆" />
      <StatCell label="观察" value={`${pageReadModel.observation.observation_count}`} helper="观察来源" />
      <StatCell label="检查" value={`${pageReadModel.lint.open_count}`} helper={`阻断 ${pageReadModel.lint.blocking_count}`} warn={pageReadModel.lint.blocking_count > 0} />
      <StatCell label="维护" value={`${pageReadModel.maintenance.blocking_count}`} helper={`复核 ${pageReadModel.maintenance.needs_review_count} / 信息 ${pageReadModel.maintenance.info_count}`} warn={pageReadModel.maintenance.blocking_count > 0} />
      <StatCell label="成熟模式" value={`${pageReadModel.mature_pattern.candidate_count}`} helper={`确认 ${pageReadModel.mature_pattern.user_confirmation_required_count}`} warn={pageReadModel.mature_pattern.user_confirmation_required_count > 0} />
      <StatCell label="任务包" value={`${pageReadModel.task_package.snapshot_count}`} helper="冻结快照" />
    </div>
  );
}

export function MemoryWorkbenchSummary({
  pageReadModel,
  summary,
}: {
  pageReadModel: MemoryCenterPageReadModel;
  summary: MemoryManagementSummary;
}) {
  return (
    <section className="memory-center-panel memory-workbench-panel" aria-label="记忆工作台摘要">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">记忆工作台</p>
          <h3>捕获 / 候选 / 任务记忆包</h3>
        </div>
        <Pill tone={pageReadModel.memory_workbench.action_count ? "warn" : "candidate"}>
          {pageReadModel.memory_workbench.action_count} 待处理
        </Pill>
      </div>
      <div className="memory-workbench-strip" aria-label="记忆工作台关键数字">
        <MiniMetric label="捕获" value={`${pageReadModel.memory_workbench.capture_count}`} />
        <MiniMetric label="观察" value={`${pageReadModel.memory_workbench.observation_count}`} />
        <MiniMetric label="候选" value={`${pageReadModel.memory_workbench.candidate_count}`} />
        <MiniMetric label="待正式化" value={`${pageReadModel.memory_workbench.confirmed_pending_formalization_count}`} />
        <MiniMetric label="需补证" value={`${pageReadModel.memory_workbench.capture_compensation_count}`} warn={pageReadModel.memory_workbench.capture_compensation_count > 0} />
      </div>
      <div className="workflow-compact-list">
        <div className="workflow-compact-item memory-workbench-lede">
          <strong>记忆链路</strong>
          <span>{summary.memory_workbench_summary.display_text}</span>
          <em>{summary.memory_workbench_summary.boundary_text}</em>
        </div>
        <div className="workflow-compact-item memory-workbench-lede">
          <strong>任务记忆包</strong>
          <span>{summary.memory_workbench_summary.task_memory_packet_text}</span>
          <em>发送前应看清入选、排除和待审材料；候选和观察不会冒充正式记忆。</em>
        </div>
        {summary.memory_workbench_summary.action_items.slice(0, 6).map((item) => (
          <div className="workflow-compact-item memory-workbench-action" key={item.action_id}>
            <div className="memory-item-topline">
              <strong>{item.title}</strong>
              <Pill tone={badgePillTone[item.badge_tone]}>{memoryWorkbenchActionLabel(item.kind)}</Pill>
            </div>
            <span>{item.summary}</span>
            <em>下一步：{item.next_step}</em>
          </div>
        ))}
        {!summary.memory_workbench_summary.action_items.length ? (
          <p className="muted small-note">当前没有候选确认、正式化、补证或任务记忆包待审事项。</p>
        ) : null}
      </div>
    </section>
  );
}

function StatCell({
  label,
  value,
  helper,
  warn = false,
}: {
  label: string;
  value: string;
  helper: string;
  warn?: boolean;
}) {
  return (
    <div className="stat-cell">
      <div className="lbl">{label}</div>
      <div className={`val mono${warn ? " warn" : ""}`}>{value}</div>
      <div className="memory-stat-helper">{helper}</div>
    </div>
  );
}

function MiniMetric({ label, value, warn = false }: { label: string; value: string; warn?: boolean }) {
  return (
    <div className={`memory-mini-metric${warn ? " warn" : ""}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function memoryWorkbenchActionLabel(kind: string): string {
  if (kind === "review_candidate") return "候选待审";
  if (kind === "confirm_formalization") return "正式化";
  if (kind === "repair_capture_link") return "补证";
  if (kind === "review_task_memory_packet") return "任务包";
  if (kind === "resolve_memory_blocker") return "阻断";
  return kind;
}
