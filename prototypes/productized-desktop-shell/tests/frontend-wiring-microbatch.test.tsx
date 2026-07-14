import { renderToStaticMarkup } from "react-dom/server.browser";
import { emptySnapshot } from "../src/lib/emptySnapshot";
import type { AuditLedgerReadModelItem } from "../src/lib/tauri";
import { primaryNavGroups, primaryNavItems } from "../src/lib/workbenchNavigation";
import {
  AUDIT_EVENT_NOT_IN_CURRENT_PAGE_MESSAGE,
  AuditLedgerView,
  buildAuditLedgerMainRows,
  isMissingAuditEventFocus,
} from "../src/views/AuditLedgerView";
import { HomeView } from "../src/views/HomeView";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[frontend-wiring-microbatch] ${message}`);
}

const home = renderToStaticMarkup(
  <HomeView
    snapshot={emptySnapshot}
    systemStatus={{
      storage_mode: "db_primary",
      storage_healthy: false,
      observation_day: 0,
      last_degradation: { at_ms: 1_721_000_000_000, reason_human: "存储已降级，已回退 JSON" },
      recent_catches: [],
      gate_summary: "按默认闸走",
      warnings: ["降级期间继续展示历史账本。"],
    }}
    onNavigate={() => {}}
  />,
);

assert(home.includes("未进入观察期"), "observation_day=0 应显示未进入观察期");
assert(!home.includes("观察期第 0 天"), "observation_day=0 不应显示为第 0 天");
assert(home.includes("存储已降级，已回退 JSON"), "应显示最后一次降级的人话原因");
assert(home.includes("降级期间继续展示历史账本。"), "warnings 应软着陆显示人话");
assert(primaryNavItems.some((item) => item.key === "harness" && item.label === "harness"), "导航应显示 harness");
assert(
  !primaryNavGroups.some((group) => group.items.some((item) => item.key === "command-console")),
  "发令台已拍下线，不应继续出现在左侧导航",
);

const bPageItems: AuditLedgerReadModelItem[] = [
  {
    at_ms: 1_721_000_000_000,
    source: "workflow_state",
    event_type: "workflow_state_changed",
    human_summary: "工单已进入复核。",
    target_ref: "work-item:alpha",
    raw_json: { event_id: "current-event", target_ref: "work-item:alpha" },
  },
  {
    at_ms: 1_720_999_000_000,
    source: "workflow_state",
    event_type: "workflow_state_changed",
    human_summary: "没有可供右栏定位的事件编号。",
    target_ref: "event-id-must-not-come-from-target-ref",
    raw_json: { work_item_id: "work-item:beta" },
  },
];
const bRows = buildAuditLedgerMainRows(bPageItems);

assert(bRows[0]?.key === "audit-event:current-event", "B 当前页的 workflow_state event_id 应精确承接右栏 audit-event 深链");
assert(
  bRows[1]?.key !== "audit-event:event-id-must-not-come-from-target-ref",
  "target_ref 是归属对象，不能被误作 audit-event 深链键",
);
assert(
  !isMissingAuditEventFocus({ kind: "audit-event", id: "audit-event:current-event" }, bRows, true),
  "当前 B 页命中 audit-event 时应选中该条",
);
assert(
  isMissingAuditEventFocus({ kind: "audit-event", id: "audit-event:older-event" }, bRows, true),
  "B 当前页未命中 audit-event 时必须走明确提示，不能回落第一条",
);
assert(
  AUDIT_EVENT_NOT_IN_CURRENT_PAGE_MESSAGE === "目标事件不在最新一页(账本按时间倒序分页),可翻页查找",
  "未命中提示必须保持裁决文案",
);

const auditLedgerMarkup = renderToStaticMarkup(<AuditLedgerView snapshot={emptySnapshot} />);
assert(auditLedgerMarkup.includes("账本主流"), "B 应成为账本页主流区");
assert(auditLedgerMarkup.includes("搜索运行日志与健康诊断"), "本地搜索框只能属于并列运行与健康区");
assert(!auditLedgerMarkup.includes("搜索审计账本"), "B 主流不能把当前页搜索伪装成全局搜索");

console.log("frontend-wiring-microbatch: W1/W2/W3 接线断言全过");
