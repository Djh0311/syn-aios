// 交办·会话选择件(共享:批态/绑定态/卡住态都用)——阶段3拆巨石第二刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
import { useMemo, useState } from "react";
import type {
  ProjectDirectorPlannedTask,
  ProjectDirectorTaskSessionBinding,
  SessionRecord,
} from "../../../lib/types";

export const NEW_SESSION_CHOICE = "__new_session__";

// 哨兵/未定（新会话单——真 id 前端拿不到，别猜别反查）→ 给了 latestSessionThreadId 才显
// 「看最近对话」（最近≈本单但不保证，词表不吹大）；两者皆无 → 零渲染。
// 纯导航：走已存在的 onOpenAgentSession（App 级 setFocusedAgentThreadId + 切智能体页，现成）。
// 无 hooks·export——离线 DOM 断言可直接走元素树点 onClick（harness 限制：带 hooks 的只能静态标记断言）。
export function JiaobanRawSessionLink({
  sessionChoice,
  latestSessionThreadId = null,
  onOpenAgentSession,
}: {
  sessionChoice: string | null;
  latestSessionThreadId?: string | null;
  onOpenAgentSession: ((threadId: string) => void) | undefined;
}) {
  if (!onOpenAgentSession) return null;
  const realThreadId =
    sessionChoice && sessionChoice !== NEW_SESSION_CHOICE ? sessionChoice : null;
  const target = realThreadId ?? latestSessionThreadId;
  if (!target) return null;
  return (
    <button
      type="button"
      className="jiaoban-linklike jiaoban-raw-session-link"
      onClick={() => onOpenAgentSession(target)}
    >
      {realThreadId ? "看原始对话" : "看最近对话"}
    </button>
  );
}

// 开工前逐任务绑定面板。每项默认新会话；旧会话只会精确绑定到其自己的 node + work item。
// export 供离线 DOM 断言（无 hooks·映射校验仍由后端做最终裁决）。
export function JiaobanTaskSessionBindingState({
  tasks,
  sessions,
  bindings,
  error,
  starting,
  onBindingChange,
  onStart,
  onReplan,
  onStop,
}: {
  tasks: ProjectDirectorPlannedTask[];
  sessions: SessionRecord[];
  bindings: ProjectDirectorTaskSessionBinding[];
  error: string | null;
  starting: boolean;
  onBindingChange: (plannedTaskId: string, sessionChoice: string | null) => void;
  onStart: () => void;
  onReplan: () => void;
  onStop: () => void;
}) {
  const bindingsByTask = new Map(bindings.map((binding) => [binding.planned_task_id, binding]));
  const canStart =
    tasks.length > 0 &&
    tasks.every((task) => bindingsByTask.has(task.planned_task_id)) &&
    !error;

  return (
    <div className="project-canvas-detail-card" aria-label="任务会话绑定">
      <div className="panel-heading">
        <div>
          <h3>先给每项任务选对话</h3>
        </div>
      </div>
      <p className="role-loop-plain-lead">
        每个任务默认用自己的新会话；想让某个任务接着某条旧对话，就给它单独选。
      </p>

      {tasks.length > 0 && !error ? (
        <div className="role-loop-plain" aria-label="逐任务会话选择">
          {tasks.map((task, index) => {
            const binding = bindingsByTask.get(task.planned_task_id);
            const sessionChoice =
              binding?.session_choice === "existing" ? (binding.session_id ?? null) : NEW_SESSION_CHOICE;
            return (
              <div className="jiaoban-task-session-row" key={task.planned_task_id}>
                <p className="jiaoban-field-label">{task.title}</p>
                <JiaobanSessionPicker
                  sessions={sessions}
                  sessionChoice={sessionChoice}
                  onSessionChoiceChange={(nextChoice) => onBindingChange(task.planned_task_id, nextChoice)}
                  label="这项任务用哪个对话"
                  inputName={`jiaoban-task-session-${index}`}
                  newSessionText="开个新的（为这项任务新建一个对话）"
                />
              </div>
            );
          })}
        </div>
      ) : (
        <p className="jiaoban-consult-error" role="alert">
          {error ?? "任务清单没有准备好，不能开始跑。请重新出方案或停下。"}
        </p>
      )}

      <div className="workflow-state-actions">
        {canStart ? (
          <button className="primary-button" type="button" disabled={starting} onClick={onStart}>
            {starting ? "正在开始跑…" : "开始跑"}
          </button>
        ) : null}
        <button className="secondary-button" type="button" disabled={starting} onClick={onReplan}>
          重新出方案
        </button>
        <button className="secondary-button" type="button" disabled={starting} onClick={onStop}>
          停下
        </button>
      </div>
    </div>
  );
}

// 会话收纳：默认收起一行「用哪个对话干：接现有 · <最近一条标题> ▾」，点开才展开选择。
// 展开后：最近 5 条直列 + 其余折叠/可搜；新会话始终可选，逐任务绑定面板可用它直接开工。
// export 供离线 DOM 断言（带 hooks → 测试只静态标记断言，不平铺调用）。
export function JiaobanSessionPicker({
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  onOpenAgentSession,
  label = "用哪个对话干",
  inputName = "jiaoban-session",
  newSessionText = "开个新的（为这单活新建一个对话）",
}: {
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  // 「看原始对话」桥（可选）：批卡传、卡住脸不传（卡住脸自己有面级入口，防一脸双入口）。
  onOpenAgentSession?: (threadId: string) => void;
  label?: string;
  inputName?: string;
  newSessionText?: string;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [showRest, setShowRest] = useState(false);

  // 最近在前。
  const sorted = useMemo(
    () => [...sessions].sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0)),
    [sessions],
  );
  const selected = sorted.find((s) => s.thread_id === sessionChoice) ?? null;
  const newSelected = sessionChoice === NEW_SESSION_CHOICE || sessionChoice === null;
  // 方案a fix：收起行必须说真话——选了新建就显「开个新的」，别拿最新旧会话标题冒充（真机踩到的帮凶）。
  const summaryTitle =
    newSelected
      ? newSessionText
      : selected?.title || selected?.thread_id || `${sorted[0]?.title ?? "选一条对话"}（默认）`;

  const recent = sorted.slice(0, 5);
  const rest = sorted.slice(5);
  const filteredRest = query.trim()
    ? sorted.filter(
        (s) =>
          (s.title || s.thread_id).toLowerCase().includes(query.trim().toLowerCase()) &&
          !recent.includes(s),
      )
    : rest;

  return (
    <div className="jiaoban-session-pick" aria-label={label}>
      <div className="jiaoban-session-summary-row">
        <button
          type="button"
          className="jiaoban-session-summary"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
        >
          <span className="jiaoban-field-label">{label}：</span>
          <span className="jiaoban-session-summary-value">
            {newSelected ? summaryTitle : `接现有 · ${summaryTitle}`}
          </span>
          <span aria-hidden="true" className="jiaoban-session-caret">
            {open ? "▴" : "▾"}
          </span>
        </button>
        {/* 行尾「看原始对话」：只在选了「接现有:X」时显（新建/未定 → 会话还没生，没得看——诚实不显）。 */}
        <JiaobanRawSessionLink
          sessionChoice={sessionChoice}
          onOpenAgentSession={onOpenAgentSession}
        />
      </div>

      {open ? (
        <div className="jiaoban-session-expand">
          {/* 方案a fix：「开个新的」用显式哨兵（不再用 null 一词两用）——重挂载/默认效果都不会吞掉这个选择。 */}
          <label className="jiaoban-radio">
            <input
              type="radio"
              name={inputName}
              checked={newSelected}
              onChange={() => onSessionChoiceChange(NEW_SESSION_CHOICE)}
            />
            {newSessionText}
          </label>

          {sorted.length === 0 ? (
            <p className="muted small-note" style={{ margin: 0 }}>
              还没有旧对话，也可以直接为这项任务开新的。
            </p>
          ) : null}

          {recent.map((session) => (
            <label className="jiaoban-radio" key={session.thread_id}>
              <input
                type="radio"
                name={inputName}
                checked={sessionChoice === session.thread_id}
                onChange={() => onSessionChoiceChange(session.thread_id)}
              />
              接现有：{session.title || session.thread_id}
            </label>
          ))}

          {rest.length > 0 ? (
            <div className="jiaoban-session-rest">
              {!showRest ? (
                <button
                  type="button"
                  className="jiaoban-linklike"
                  onClick={() => setShowRest(true)}
                >
                  还有 {rest.length} 条更早的对话，展开选…
                </button>
              ) : (
                <>
                  <input
                    type="text"
                    className="jiaoban-session-search"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="搜对话标题…"
                  />
                  {filteredRest.map((session) => (
                    <label className="jiaoban-radio" key={session.thread_id}>
                      <input
                        type="radio"
                        name={inputName}
                        checked={sessionChoice === session.thread_id}
                        onChange={() => onSessionChoiceChange(session.thread_id)}
                      />
                      接现有：{session.title || session.thread_id}
                    </label>
                  ))}
                  {filteredRest.length === 0 ? (
                    <p className="muted small-note" style={{ margin: 0 }}>
                      没有匹配的对话。
                    </p>
                  ) : null}
                </>
              )}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
