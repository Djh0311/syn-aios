import { Badge } from "../components/Badge";
import { formatDate, shortId, warningText } from "../lib/format";
import type { PendingAction, SessionRecord } from "../lib/types";

type SessionsViewProps = {
  sessions: SessionRecord[];
  onRequestAction: (action: PendingAction) => void;
};

export function SessionsView({ sessions, onRequestAction }: SessionsViewProps) {
  return (
    <section className="view-stack">
      <div className="section-heading">
        <div>
          <p className="eyebrow">会话索引</p>
          <h2>会话页</h2>
        </div>
        <p className="muted">只展示标题和元数据，不读取回放正文。</p>
      </div>

      <div className="table-wrap">
        <table>
          <thead>
            <tr>
              <th>标题</th>
              <th>编号</th>
              <th>项目</th>
              <th>更新时间</th>
              <th>状态</th>
              <th>模型</th>
              <th>回放</th>
              <th>警告</th>
              <th>动作</th>
            </tr>
          </thead>
          <tbody>
            {sessions.map((session) => (
              <tr key={session.thread_id}>
                <td className="session-title">{session.title}</td>
                <td className="path-text">{shortId(session.thread_id)}</td>
                <td className="path-text">{session.project_root || "未知项目"}</td>
                <td>{formatDate(session.updated_at_ms)}</td>
                <td>
                  <Badge tone={session.archived ? "neutral" : "candidate"}>
                    {session.archived ? "已归档" : "未归档"}
                  </Badge>
                </td>
                <td>{session.model || "未知"} / {session.reasoning_effort || "未知"}</td>
                <td>
                  <Badge tone={session.rollout_exists ? "candidate" : "warning"}>
                    {session.rollout_exists ? "存在" : "未知/缺失"}
                  </Badge>
                </td>
                <td>{warningText(session.warnings)}</td>
                <td>
                  <div className="action-row compact">
                    <button
                      className="secondary-button"
                      type="button"
                      disabled={!session.rollout_path}
                      onClick={() =>
                        session.rollout_path &&
                        onRequestAction({
                          kind: "reveal-rollout",
                          label: "定位回放文件",
                          path: session.rollout_path,
                          source: "索引内回放记录路径",
                        })
                      }
                    >
                      定位
                    </button>
                    <button
                      className="secondary-button"
                      type="button"
                      disabled={!session.rollout_path}
                      onClick={() =>
                        session.rollout_path &&
                        onRequestAction({
                          kind: "copy",
                          label: "复制回放路径",
                          path: session.rollout_path,
                          source: "索引内回放记录路径",
                        })
                      }
                    >
                      复制
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
