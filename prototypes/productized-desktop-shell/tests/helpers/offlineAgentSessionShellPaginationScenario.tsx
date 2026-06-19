import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { assert, visibleText } from "./offlineInteractionTestUtils";
import type { PendingAction, SessionRecord } from "../../src/lib/types";
import { AgentSessionCenter } from "../../src/views/AgentView";
import { runConversationEngineScenario } from "./offlineConversationEngineScenario";

export function runAgentSessionShellPaginationScenario({
  archivedSession,
  captureAction,
  projectSession,
}: {
  archivedSession: SessionRecord;
  captureAction: (action: PendingAction) => void;
  projectSession: SessionRecord;
}) {
  const manySessions = Array.from({ length: 90 }, (_, index) => ({
    ...projectSession,
    thread_id: `offline-thread-window-${index}`,
    title: `Large list fixture ${index}`,
    updated_at_ms: 10_000 - index,
  }));
  const largeCenter = (
    <AgentSessionCenter
      sessions={manySessions}
      selectedThreadId={manySessions[0].thread_id}
      selectedSession={manySessions[0]}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={manySessions.length}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />
  );
  const largeCenterText = visibleText(largeCenter);
  const largeCenterMarkup = renderToStaticMarkup(largeCenter);
  assert(largeCenterText.includes("显示 40 / 90"), "大列表应默认只显示首个窗口");
  assert(largeCenterText.includes("显示更多会话"), "大列表应提供继续渲染入口");
  assert(largeCenterText.includes("Large list fixture 39"), "虚拟窗口应包含第 40 条");
  assert(!largeCenterMarkup.includes("Large list fixture 40"), "第 41 条应在加载更多前不进入 DOM");
  assert(!largeCenterMarkup.includes("Large list fixture 89"), "远端会话应在加载更多前不进入 DOM");
  assert(largeCenterMarkup.includes("显示更多会话"), "大列表缺少显示更多按钮");

  const archivedCenterText = visibleText(
    <AgentSessionCenter
      sessions={[archivedSession]}
      selectedThreadId={null}
      selectedSession={null}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      initialReadFilter="archived"
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(archivedCenterText.includes("当前只看归档"), "归档会话只能出现在归档视图说明中");
  runConversationEngineScenario({ captureAction, session: projectSession });
}
