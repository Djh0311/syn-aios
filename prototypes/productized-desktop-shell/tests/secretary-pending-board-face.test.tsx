// B3·秘书「待你拍板」呈现·离线 DOM 断言（renderToStaticMarkup·同 harness 风格）。
// §4：三组渲染 / 空组不渲染 / 全空文案 / 词表（无枚举原文·无「审批」·边界话在）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { SecretaryPendingBoardSection } from "../src/components/SecretaryBrief";
import type { SecretaryPendingBoard } from "../src/lib/secretaryReadModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[secretary-pending-board-face] ${message}`);
  }
}

function html(board: SecretaryPendingBoard): string {
  return renderToStaticMarkup(<SecretaryPendingBoardSection board={board} />);
}

// 1) 三组齐全渲染 + 去处提示 + 边界话。
{
  const out = html({
    total: 3,
    pending_proposals: [
      { entry_id: "p1", title: "方案「改计分板」等你批", detail: "今天生成", where_hint: "在交办页批" },
    ],
    supervisor_reminders: [
      { entry_id: "s1", title: "主管看过上一单结果，建议你亲自核验", detail: "打开页面亲手玩一遍。", where_hint: "在交办页交货区看" },
    ],
    memory_candidate_entry: { entry_id: "m1", title: "2 条记忆候选等你确认", detail: "候选不等于工作台已经长期记住", where_hint: "在记忆中心处理" },
  });
  assert(out.includes("待你拍板（3）"), "标题带总数");
  assert(out.includes("待批方案") && out.includes("全局主管提醒") && out.includes("记忆候选"), "三组标题在");
  assert(out.includes("改计分板") && out.includes("亲自核验") && out.includes("记忆候选等你确认"), "条目在");
  assert(out.includes("在交办页批") && out.includes("在记忆中心处理"), "去处提示在");
  assert(out.includes("这些是提醒，不是命令"), "边界话在");
}

// 2) 空组不渲染（只有方案组）。
{
  const out = html({
    total: 1,
    pending_proposals: [{ entry_id: "p1", title: "方案「x」等你批", detail: "", where_hint: "在交办页批" }],
    supervisor_reminders: [],
    memory_candidate_entry: null,
  });
  assert(out.includes("待批方案"), "非空组渲染");
  assert(!out.includes("全局主管提醒"), "空组不渲染");
  assert(!out.includes("记忆候选"), "空聚合条不渲染");
}

// 3) 全空 → 「桌面干净」。
{
  const out = html({ total: 0, pending_proposals: [], supervisor_reminders: [], memory_candidate_entry: null });
  assert(out.includes("桌面干净，没有等你的事"), "全空文案");
  assert(!out.includes("待批方案"), "全空无组");
}

// 4) 词表：无「审批」、无枚举原文/黑话。
{
  const out = [
    html({
      total: 2,
      pending_proposals: [{ entry_id: "p", title: "方案「y」等你批", detail: "3 天前生成的旧方案，建议先看看还作不作数", where_hint: "在交办页批" }],
      supervisor_reminders: [{ entry_id: "s", title: "主管说有份方案对不上你的目标", detail: "方案只读。", where_hint: "在交办页批卡上看" }],
      memory_candidate_entry: null,
    }),
    html({ total: 0, pending_proposals: [], supervisor_reminders: [], memory_candidate_entry: null }),
  ].join("");
  assert(!out.includes("审批"), "词表：无「审批」");
  assert(!out.includes("mismatch") && !out.includes("needs_human_check") && !out.includes("pending_user_confirmation"), "词表：无枚举原文");
  assert(!out.includes("proposal_id") && !out.includes("sidecar"), "词表：无黑话");
}

console.log("secretary-pending-board-face: 4 组离线 DOM 断言全过");
