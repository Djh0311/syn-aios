// B3·秘书「待你拍板」derive 单测（纯函数·离线）。
// §4：三组入选/排除判据（pending 入 confirmed 不入；needs_human_check 入 pass 不入；mismatch 入
// caution 不入）+ 空店/缺参零炸（向后兼容）+ 现有字段回归断言（语义 0-diff）。
import { deriveSecretaryContext } from "../src/lib/secretaryReadModel";
import { emptySnapshot } from "../src/lib/emptySnapshot";
import type {
  GlobalSupervisorBoundaryReviewRecord,
  GlobalSupervisorReviewRecord,
  GlobalSupervisorReviewStoreV1,
  ProjectConsultationProposal,
  ProjectConsultationProposalStoreV1,
} from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[secretary-pending-board] ${message}`);
  }
}

function proposal(overrides: Partial<ProjectConsultationProposal>): ProjectConsultationProposal {
  return {
    proposal_id: "p1",
    schema_version: "project_consultation_proposal.v1",
    project_id: "proj",
    workflow_id: "wf-1",
    title: "改计分板",
    user_goal: "改计分板",
    goal_summary: "改计分板",
    proposed_steps: [],
    scope_draft: {
      allowed_role_ids: [],
      allowed_agent_ids: [],
      allowed_read_roots: [],
      allowed_write_roots: [],
      allowed_tools: [],
      allowed_checks: [],
      allowed_task_package_kinds: [],
      stop_conditions: [],
    },
    risks: [],
    acceptance_criteria: [],
    status: "pending_user_confirmation",
    created_by_role: "project_consultant",
    created_at_ms: Date.now(),
    updated_at_ms: Date.now(),
    ...overrides,
  };
}

function proposalStore(proposals: ProjectConsultationProposal[]): ProjectConsultationProposalStoreV1 {
  return {
    schema_version: "project_consultation_proposal_store.v1",
    revision: 1,
    updated_at_ms: 1,
    proposals,
    decisions: [],
    audit_events: [],
  } as unknown as ProjectConsultationProposalStoreV1;
}

function review(overrides: Partial<GlobalSupervisorReviewRecord>): GlobalSupervisorReviewRecord {
  return {
    review_id: "r1",
    project_id: "proj",
    workflow_id: "wf-1",
    chain_started_at: "1000",
    status: "ready",
    overall: "pass",
    summary: "",
    suggested_action: "none",
    human_note: "",
    tasks: [],
    unavailable_reason: null,
    model: "m",
    profile_version: "v",
    created_at_ms: 1,
    updated_at_ms: 1,
    ...overrides,
  };
}

function boundary(overrides: Partial<GlobalSupervisorBoundaryReviewRecord>): GlobalSupervisorBoundaryReviewRecord {
  return {
    review_id: "b1",
    project_id: "proj",
    proposal_id: "p-x",
    status: "ready",
    verdict: "looks_ok",
    points: [],
    summary: "",
    unavailable_reason: null,
    model: "m",
    profile_version: "v",
    created_at_ms: 1,
    updated_at_ms: 1,
    ...overrides,
  };
}

function reviewStore(
  reviews: GlobalSupervisorReviewRecord[],
  boundaries: GlobalSupervisorBoundaryReviewRecord[],
): GlobalSupervisorReviewStoreV1 {
  return {
    schema_version: "global_supervisor_review_store.v1",
    revision: 1,
    updated_at_ms: 1,
    reviews,
    audit_events: [],
    boundary_reviews: boundaries,
    boundary_audit_events: [],
  };
}

// 1) 缺参/空店零炸（旧调用形不带新参 → pending_board 全空·total 0）+ 现有字段回归。
{
  const context = deriveSecretaryContext({ snapshot: emptySnapshot });
  assert(context.pending_board.total === 0, "缺参 → 待拍板空");
  assert(context.pending_board.pending_proposals.length === 0, "缺参 → 无方案组");
  assert(context.pending_board.memory_candidate_entry === null, "缺参 → 无记忆聚合条");
  // 现有字段语义 0-diff 回归：结构照旧、只读警示照旧。
  assert(context.source_kind === "derived_read_model", "现有字段：source_kind 不变");
  assert(Array.isArray(context.risk_signals) && Array.isArray(context.suggestions), "现有字段：风险/建议数组在");
  assert(context.warnings.includes("secretary_context_is_read_only"), "现有字段：只读警示在");
  const withNull = deriveSecretaryContext({ snapshot: emptySnapshot, proposalStore: null, supervisorReviewStore: null });
  assert(withNull.pending_board.total === 0, "显式 null 也零炸");
}

// 2) 方案组判据：pending 入（今天/旧标注）·confirmed 不入。
{
  const context = deriveSecretaryContext({
    snapshot: emptySnapshot,
    proposalStore: proposalStore([
      proposal({ proposal_id: "p-new", title: "今天的方案" }),
      proposal({ proposal_id: "p-old", title: "旧方案", created_at_ms: Date.now() - 3 * 24 * 60 * 60 * 1000 }),
      proposal({ proposal_id: "p-done", title: "已确认的", status: "user_confirmed" }),
    ]),
  });
  const board = context.pending_board;
  assert(board.pending_proposals.length === 2, `pending 入 confirmed 不入：${board.pending_proposals.length}`);
  const oldEntry = board.pending_proposals.find((entry) => entry.title.includes("旧方案"));
  assert(oldEntry && oldEntry.detail.includes("天前"), "旧方案标「天前」（照批卡 stale 口径）");
  const newEntry = board.pending_proposals.find((entry) => entry.title.includes("今天的方案"));
  assert(newEntry && newEntry.detail.includes("今天"), "今天的标「今天」");
  assert(board.pending_proposals.every((entry) => entry.where_hint.includes("交办")), "去处提示：在交办页批");
  assert(board.total === 2, "total 对");
}

// 3) 主管提醒判据：needs_human_check / human_verify 入·pass 不入·mismatch 入·caution 不入·unavailable 不入。
{
  const context = deriveSecretaryContext({
    snapshot: emptySnapshot,
    supervisorReviewStore: reviewStore(
      [
        review({ review_id: "r-check", overall: "needs_human_check", human_note: "打开页面亲手玩一遍。再看看分数。" }),
        review({ review_id: "r-verify", overall: "pass", suggested_action: "human_verify", summary: "建议顺手看一眼" }),
        review({ review_id: "r-pass", overall: "pass", suggested_action: "none" }),
        review({ review_id: "r-unavail", status: "unavailable", overall: "needs_human_check" }),
      ],
      [
        boundary({ review_id: "b-mismatch", verdict: "mismatch", summary: "方案只读，对不上要动手的目标。细节略。" }),
        boundary({ review_id: "b-caution", verdict: "caution", summary: "验收偏薄" }),
        boundary({ review_id: "b-ok", verdict: "looks_ok" }),
      ],
    ),
  });
  const reminders = context.pending_board.supervisor_reminders;
  assert(reminders.length === 3, `needs_human_check + human_verify + mismatch = 3：实际 ${reminders.length}`);
  assert(
    reminders.some((entry) => entry.detail.includes("亲手玩一遍")),
    "human_note 首句进 detail",
  );
  assert(
    !reminders.some((entry) => entry.detail.includes("再看看分数")),
    "只取首句（句号截断）",
  );
  assert(
    reminders.some((entry) => entry.title.includes("对不上")),
    "mismatch 人话标题",
  );
  assert(!reminders.some((entry) => entry.detail.includes("验收偏薄")), "caution 不入（噪音）");
  // 词表：呈现字段（title/detail/where_hint）不露枚举原文（entry_id 是内部键不上屏、不在此列）。
  const shown = reminders.map((entry) => `${entry.title}|${entry.detail}|${entry.where_hint}`).join(" ");
  assert(!shown.includes("mismatch") && !shown.includes("needs_human_check"), "词表：呈现字段不露 verdict/overall 枚举原文");
}

// 4) 记忆候选：引用现有计数、>0 才有聚合条目（emptySnapshot 无候选 → null 已在组 1 验）。
//    这里验去处提示词表。
{
  const context = deriveSecretaryContext({ snapshot: emptySnapshot });
  assert(context.pending_board.memory_candidate_entry === null, "无候选 → 无条目");
}

console.log("secretary-pending-board: 4 组 derive 断言全过");
