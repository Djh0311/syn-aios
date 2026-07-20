import { Pill } from "../../components/SpecPrimitives";
import type { FormalMemoryListItem, MemoryCandidateListItem } from "../../lib/memoryCenter";
import type {
  MaturePatternCandidate,
  MaturePatternPreviewOutput,
  MemoryClusterReport,
  MemoryEntityCandidate,
  MemoryEntityMergeCandidate,
  MemoryRelation,
  MemoryRelationCandidate,
} from "../../lib/types";
import { sourceText } from "./MemoryDetailPanels";

const badgePillTone = { neutral: "plain", candidate: "candidate", warning: "warn", unknown: "unknown" } as const;

// 批2·P0修复:列表卡可选中(点哪条详情看哪条——此前详情死绑数组第一项,其余记录永久点不开)。
// onSelect 可选:不传=纯展示(旧调用点零破坏);键盘导航按宪法§八显式不做。
export function FormalMemoryItem({
  item,
  selected = false,
  onSelect,
}: {
  item: FormalMemoryListItem;
  selected?: boolean;
  onSelect?: () => void;
}) {
  return (
    <div
      className={`workflow-compact-item formal-memory-item${selected ? " is-selected" : ""}${onSelect ? " is-selectable" : ""}`}
      onClick={onSelect}
      aria-pressed={onSelect ? selected : undefined}
      role={onSelect ? "button" : undefined}
    >
      <div className="memory-item-topline">
        <strong>{item.kind_label} / {item.status_label}</strong>
        <Pill tone={badgePillTone[item.task_eligibility.badge_tone]}>{item.task_eligibility.label}</Pill>
      </div>
      <span>{item.claim}</span>
      <em>来源：{sourceText(item.source_summaries)}</em>
      <em>{item.version_summary}</em>
      <em>审计 {item.audit_summary}</em>
      <em>{item.scope_label} / {item.permission_summary} / {item.model_export_summary}</em>
      <em>{item.conflict_summary}</em>
      {item.task_eligibility.included_in_task_package ? <em>任务包冻结快照已引用</em> : null}
    </div>
  );
}

export function CandidateMemoryItem({
  item,
  selected = false,
  onSelect,
}: {
  item: MemoryCandidateListItem;
  selected?: boolean;
  onSelect?: () => void;
}) {
  return (
    <div
      className={`workflow-compact-item candidate-memory-item${selected ? " is-selected" : ""}${onSelect ? " is-selectable" : ""}`}
      onClick={onSelect}
      aria-pressed={onSelect ? selected : undefined}
      role={onSelect ? "button" : undefined}
    >
      <div className="memory-item-topline">
        <strong>{item.kind_label} / {item.status_label}</strong>
        <Pill tone={badgePillTone[item.task_position.badge_tone]}>{item.task_position.label}</Pill>
      </div>
      <span>{item.claim}</span>
      <em>{item.formal_memory_boundary}</em>
      <em>{item.risk_summary}</em>
      <em>{item.confirmation_summary}</em>
      <em>{item.adoption_summary}</em>
      <em>来源：{sourceText(item.source_summaries)}</em>
      <em>{item.lint_summary}</em>
    </div>
  );
}

export function EntityCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryEntityCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  return (
    <div className="workflow-compact-item memory-entity-candidate-item">
      <div className="memory-item-topline">
        <strong>实体候选 / {entityKindLabel(candidate.entity_kind)}</strong>
        <Pill tone="warn">{candidate.status}</Pill>
      </div>
      <span>{candidate.display_name}</span>
      <em>{candidate.reason}</em>
      <em>来源 {sourceKindLabel(candidate.source_kind)}</em>
      {/* 「开发者详情」折叠已废除（DESIGN.md §三·五 禁令2）：机器字段卡片上零入口。 */}
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm}>登记实体 / 别名</button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

export function MergeCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryEntityMergeCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  return (
    <div className="workflow-compact-item memory-merge-candidate-item">
      <div className="memory-item-topline">
        <strong>去重候选</strong>
        <Pill tone={candidate.source_kind === "similarity_hit" ? "warn" : "candidate"}>{sourceKindLabel(candidate.source_kind)}</Pill>
      </div>
      <span>{candidate.left_label} / {candidate.right_label}</span>
      <em>{candidate.reason}</em>
      <em>相似度命中仅作候选；确认动作只登记治理决定。</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm}>确认去重候选</button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

export function RelationCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryRelationCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  const confirmationBlocked = candidate.source_kind === "llm_inferred" || candidate.source_kind === "similarity_hit";
  return (
    <div className="workflow-compact-item memory-relation-candidate-item">
      <div className="memory-item-topline">
        <strong>关系候选 / {relationKindLabel(candidate.relation_kind)}</strong>
        <Pill tone={candidate.relation_kind === "causal" ? "warn" : "candidate"}>{sourceKindLabel(candidate.source_kind)}</Pill>
      </div>
      <span>{candidate.subject_label} {"->"} {candidate.object_label}</span>
      <em>{candidate.reason}</em>
      <em>{candidate.requires_user_confirmation ? "需要用户确认" : "项目主管或用户可确认"}</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm} disabled={confirmationBlocked}>
          确认关系
        </button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

export function ConfirmedRelationItem({ relation }: { relation: MemoryRelation }) {
  return (
    <div className="workflow-compact-item memory-confirmed-relation-item">
      <div className="memory-item-topline">
        <strong>已确认关系 / {relationKindLabel(relation.relation_kind)}</strong>
        <Pill tone="candidate">已确认</Pill>
      </div>
      <span>{relation.subject_label} {"->"} {relation.object_label}</span>
      <em>{relation.confirmation_reason}</em>
      <em>已确认关系用于解释召回原因。</em>
      {/* 「开发者详情」折叠已废除（DESIGN.md §三·五 禁令2）：机器字段卡片上零入口。 */}
    </div>
  );
}

export function MaturePatternCandidateItem({
  candidate,
  onConfirm,
  onReject,
  onQuarantine,
  onRequestChanges,
}: {
  candidate: MaturePatternCandidate;
  onConfirm: () => void;
  onReject: () => void;
  onQuarantine: () => void;
  onRequestChanges: () => void;
}) {
  const actionable = candidate.status === "candidate";
  return (
    <div className="workflow-compact-item mature-pattern-candidate-item">
      <div className="memory-item-topline">
        <strong>成熟模式候选</strong>
        <Pill tone={maturePatternStatusTone(candidate.status)}>{candidate.status}</Pill>
      </div>
      <span>{candidate.title}</span>
      <em>{candidate.claim}</em>
      <em>{candidate.requires_user_confirmation ? "需要用户确认" : "仍需显式决定"}；候选未确认，不会进入任务包入选列表。</em>
      <em>{candidate.review_summary}</em>
      {/* 「开发者详情」折叠已废除（DESIGN.md §三·五 禁令2）：机器字段卡片上零入口。 */}
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm} disabled={!actionable}>
          用户确认为正式记忆
        </button>
        <button type="button" className="secondary-button" onClick={onReject} disabled={!actionable}>
          拒绝
        </button>
        <button type="button" className="secondary-button" onClick={onQuarantine} disabled={!actionable}>
          隔离
        </button>
        <button type="button" className="secondary-button" onClick={onRequestChanges} disabled={!actionable}>
          要求补来源
        </button>
      </div>
    </div>
  );
}

export function MemoryClusterReportItem({ report }: { report: MemoryClusterReport }) {
  return (
    <div className="workflow-compact-item memory-cluster-report-item">
      <div className="memory-item-topline">
        <strong>跨项目主题报告</strong>
        <Pill tone={report.staleness === "fresh" ? "candidate" : "warn"}>{stalenessLabel(report.staleness)}</Pill>
      </div>
      <span>{report.title}</span>
      <em>{report.display_text}</em>
      <em>报告可下钻来源，但不是正式事实，也不会进入任务包入选清单。</em>
      {/* 「开发者详情」折叠已废除（DESIGN.md §三·五 禁令2）：机器字段卡片上零入口。 */}
    </div>
  );
}

export function AcceptanceSummaryItem({ acceptanceSummary }: { acceptanceSummary: MaturePatternPreviewOutput["acceptance_summary"] | null }) {
  if (!acceptanceSummary) {
    return (
      <div className="workflow-compact-item memory-acceptance-summary-item">
        <strong>验收门禁摘要</strong>
        <span>尚未生成集成验收预览；不能声称完整验收摘要已刷新。</span>
        <em>本摘要只覆盖记忆系统集成验收，最终权威验收仍在后续阶段。</em>
      </div>
    );
  }

  return (
    <div className="workflow-compact-item memory-acceptance-summary-item">
      <div className="memory-item-topline">
        <strong>验收门禁摘要</strong>
        <Pill tone={acceptanceSummary.blocked_count ? "warn" : "candidate"}>
          {acceptanceSummary.passed_count}/{acceptanceSummary.gate_count}
        </Pill>
      </div>
      <span>{acceptanceSummary.display_text}</span>
      <em>阻断 {acceptanceSummary.blocked_count} / 后置 {acceptanceSummary.deferred_count} / 范围 {acceptanceSummary.scope_label}</em>
      {acceptanceSummary.gates.slice(0, 6).map((gate) => (
        <em key={gate.gate_id}>
          {gate.label}：{gate.status} / {gate.evidence}
        </em>
      ))}
      <em>集成验收摘要不替代后续最终权威验收。</em>
    </div>
  );
}

function entityKindLabel(kind: string): string {
  if (kind === "project") return "项目";
  if (kind === "workflow") return "工作流";
  if (kind === "session") return "会话";
  if (kind === "role") return "角色";
  if (kind === "knowledge_doc") return "知识资料";
  if (kind === "tool") return "工具";
  if (kind === "model") return "模型";
  if (kind === "harness") return "运行器";
  if (kind === "proposal") return "建议方案";
  if (kind === "memory_record") return "正式记忆";
  if (kind === "memory_candidate") return "候选记忆";
  return kind;
}

function relationKindLabel(kind: string): string {
  if (kind === "entity") return "实体";
  if (kind === "temporal") return "时间";
  if (kind === "causal") return "因果";
  if (kind === "semantic") return "语义";
  return kind;
}

function sourceKindLabel(kind: string): string {
  if (kind === "manual") return "手动";
  if (kind === "formal_memory") return "正式记忆";
  if (kind === "memory_candidate") return "候选记忆";
  if (kind === "observation") return "观察";
  if (kind === "knowledge_doc") return "知识资料";
  if (kind === "task_package") return "任务包";
  if (kind === "llm_inferred") return "LLM 推断仅作候选";
  if (kind === "similarity_hit") return "相似度命中仅作候选";
  return kind;
}

function stalenessLabel(staleness: string): string {
  if (staleness === "fresh") return "新鲜";
  if (staleness === "stale") return "过期";
  if (staleness === "unknown") return "未知";
  return staleness;
}

function maturePatternStatusTone(status: MaturePatternCandidate["status"]) {
  if (status === "candidate" || status === "changes_requested") return "warn";
  if (status === "confirmed") return "candidate";
  return "unknown";
}
