// Part①②·工作历史左栏 + 秘书看板·离线 DOM 断言（renderToStaticMarkup·同 harness 风格）。
// §4：历史九态点/筛选 chips/最新卡住单才有[接着跑]/空历史/详情卡 time_window 近似；
//     看板四列/空列文案/去处按钮/边界话。词表死线：不露英文枚举原文。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanHistoryColumn, JiaobanHistoryDetail } from "../src/views/projects/ProjectJiaobanPanel";
import { SecretaryBoardView } from "../src/components/SecretaryBoardView";
import type { RunHistoryEntry } from "../src/lib/types";
import type { SecretaryContext, SecretaryPendingBoard } from "../src/lib/secretaryReadModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[history-and-board] ${message}`);
}

const noop = () => {};

function entry(over: Partial<RunHistoryEntry>): RunHistoryEntry {
  return {
    proposal_id: "p",
    workflow_id: "w",
    goal_text: "目标",
    created_at_ms: Date.now(),
    state: "pending",
    state_note: "等你批",
    advice_only: false,
    chain: null,
    review_flags: {},
    correlation: "exact",
    ...over,
  };
}

function historyHtml(props: Partial<Parameters<typeof JiaobanHistoryColumn>[0]>): string {
  return renderToStaticMarkup(
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
      {...props}
    />,
  );
}

// 1) 九态点 + 人话 note + 筛选 chips + 词表（不露英文枚举）。
{
  const entries: RunHistoryEntry[] = [
    entry({ proposal_id: "r", state: "running", state_note: "正在干 第2/3步", chain: { started_at: "0", done_count: 2, total_count: 3 } }),
    entry({ proposal_id: "b", state: "blocked", state_note: "卡住：任务超时" }),
    entry({ proposal_id: "w1", state: "pending", state_note: "方案好了，等你批" }),
    entry({ proposal_id: "a", state: "advice_only", state_note: "纯建议，没动手" }),
    entry({ proposal_id: "d", state: "delivered", state_note: "交货", review_flags: { result_verdict: "needs_human_check" } }),
  ];
  const out = historyHtml({ entries, total: 5, latestBlockedId: "b" });
  assert(out.includes("●") && out.includes("⚠") && out.includes("○") && out.includes("◐") && out.includes("✓"), "九态点齐");
  assert(out.includes("正在干 第2/3步") && out.includes("卡住：任务超时") && out.includes("纯建议，没动手"), "state_note 人话在");
  assert(out.includes("全部") && out.includes("等我的") && out.includes("跑着"), "筛选 chips 在");
  assert(!out.includes("needs_human_check") && !out.includes("delivered") && !out.includes("advice_only"), "词表：不露英文枚举");
}

// 2) 最新卡住单才有[接着跑]（两个 blocked，只 latestBlockedId 那单给行内快捷）。
{
  const entries: RunHistoryEntry[] = [
    entry({ proposal_id: "b2", state: "blocked", state_note: "卡住 2" }),
    entry({ proposal_id: "b1", state: "blocked", state_note: "卡住 1" }),
  ];
  const out = historyHtml({ entries, total: 2, latestBlockedId: "b2" });
  assert((out.match(/接着跑/g) || []).length === 1, "只最新卡住单有[接着跑]");
}

// 3) 空历史 → 人话 + [+ 新交办]。
{
  const out = historyHtml({ entries: [], total: 0 });
  assert(out.includes("这个项目还没交办过活"), "空历史文案");
  assert(out.includes("新交办"), "空历史给[+ 新交办]");
}

// 4) 详情卡：time_window 近似小字 + 字段 + verdict 人话化 + [回到当前]。
{
  const out = renderToStaticMarkup(
    <JiaobanHistoryDetail
      entry={entry({
        goal_text: "删一个巡逻怪",
        state: "delivered",
        state_note: "交货（有 1 项要看）",
        chain: { started_at: "0", done_count: 2, total_count: 3 },
        review_flags: { result_verdict: "needs_human_check", boundary_verdict: "mismatch" },
        correlation: "time_window",
      })}
      onBackToCurrent={noop}
    />,
  );
  assert(out.includes("删一个巡逻怪"), "目标全文");
  assert(out.includes("做到第 2/3 步"), "进度人话");
  assert(out.includes("建议你亲验") && out.includes("对不上目标"), "verdict 人话化");
  assert(out.includes("按时间近似"), "time_window 近似小字");
  assert(out.includes("回到当前"), "[回到当前]在");
  assert(!out.includes("needs_human_check") && !out.includes("mismatch") && !out.includes("time_window"), "词表：不露英文枚举");
}

// A·4b) 运行错误两层脸：失败单默认显人话摘要+族标、下钻原文；不灌裸错误到默认脸；成功单不显错误区。
{
  const failOut = renderToStaticMarkup(
    <JiaobanHistoryDetail
      entry={entry({
        goal_text: "删一个怪",
        state: "blocked",
        state_note: "跑挂了（查看详情）",
        error: {
          family: "codex_subsystem",
          human: "codex 自身某个子系统报错（如记忆/索引），一般不影响本次任务结果。",
          raw_snippet: "codex_memories_write::phase2::job: failed to claim job (no such table: jobs)",
        },
      })}
      onBackToCurrent={noop}
    />,
  );
  assert(failOut.includes("codex 子系统"), "族标人话化（不露 family 机器键）");
  assert(!failOut.includes("codex_subsystem"), "不露 family 英文键");
  assert(failOut.includes("一般不影响本次任务结果"), "默认脸显人话摘要");
  assert(failOut.includes("查看原文"), "下钻入口在");
  assert(failOut.includes("no such table: jobs"), "下钻 <details> 里带原文（藏在 details·非默认脸主体）");
  // 默认脸主体（<details> 之前）不灌裸错误：出错行显人话，不是 stderr。
  const beforeDetails = failOut.split("查看原文")[0] ?? "";
  assert(!beforeDetails.includes("no such table"), "默认脸不灌裸 stderr");

  // 成功单：无 error → 不渲染错误区。
  const okOut = renderToStaticMarkup(
    <JiaobanHistoryDetail
      entry={entry({ state: "delivered", state_note: "做完了" })}
      onBackToCurrent={noop}
    />,
  );
  assert(!okOut.includes("出错") && !okOut.includes("查看原文"), "成功单不显错误区");
}

// 5) 看板四列 + 空列文案 + 去处按钮 + 卡片内容 + 页脚边界话。
{
  const board: SecretaryPendingBoard = {
    total: 1,
    pending_proposals: [{ entry_id: "p1", title: "方案「加emoji」等你批", detail: "旧方案", where_hint: "在交办页批" }],
    supervisor_reminders: [],
    memory_candidate_entry: null,
  };
  const context = {
    pending_board: board,
    risk_signals: [{ risk_id: "k1", severity: "high", title: "怪物删到 0", summary: "git 可回滚" }],
    suggestions: [{ suggestion_id: "s1", title: "跑顺的流程可沉淀模版" }],
  } as unknown as SecretaryContext;
  const out = renderToStaticMarkup(<SecretaryBoardView context={context} onNavigate={noop} />);
  assert(out.includes("等你拍板") && out.includes("主管提醒") && out.includes("记忆候选") && out.includes("风险与建议"), "四列标题齐");
  assert(out.includes("这列干净"), "空列文案");
  assert(out.includes("去交办批"), "去处按钮在");
  assert(out.includes("加emoji") && out.includes("怪物删到 0"), "卡片内容在");
  assert(out.includes("秘书零写入"), "页脚边界话在");
}

console.log("history-and-board: 6 组离线 DOM 断言全过");
