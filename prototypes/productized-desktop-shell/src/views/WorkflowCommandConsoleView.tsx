import { useCallback, useEffect, useMemo, useState } from "react";
import {
  getProjectWorkflowChainStatus,
  listProjectWorkflows,
  startProjectWorkflowChain,
  stopProjectWorkflowChain,
} from "../lib/tauri";
import type { ProjectWorkflowChainStatus, ProjectWorkflowListItem } from "../lib/types";

// 已拍下线·待退役：从左导航摘除，保留本视图与路由直到死码处置另案。
// P2 发令台：对工作流「发令」起链——一句话/打字启动，朝乙的「主管对话开干」铺第一步。
// 边界（principles §4）：对话只能「启动 / 停」，真跑仍走已 gated 的链控制器（圈固定测试项目、
// 经 path-lock + 沙箱、失败即停/可中断/审计/回滚）；这里不改 workflow-state、不绕状态机。
const TEST_PROJECT_ROOT = "/Users/yoyi/codex-workflow-mario-test";

type ConsoleProject = { project_root: string; name?: string };

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function WorkflowCommandConsoleView({
  projects,
  onNotice,
  onReloadWorkflowState,
}: {
  projects: ConsoleProject[];
  onNotice: (msg: string) => void;
  onReloadWorkflowState?: () => void;
}) {
  // 默认固定测试项目（链只在这真跑）；没有就退第一个项目。
  const defaultRoot = useMemo(
    () =>
      projects.find((p) => p.project_root === TEST_PROJECT_ROOT)?.project_root ??
      projects[0]?.project_root ??
      TEST_PROJECT_ROOT,
    [projects],
  );
  const [projectRoot, setProjectRoot] = useState(defaultRoot);
  const [workflows, setWorkflows] = useState<ProjectWorkflowListItem[]>([]);
  const [command, setCommand] = useState("");
  const [log, setLog] = useState<string[]>([
    "发令台：打『按计划跑 <工作流名>』起链、『停 <工作流名>』停链。对话只启动/停，不改状态。",
  ]);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);
  const [running, setRunning] = useState(false);
  const [activeWorkflowId, setActiveWorkflowId] = useState<string | null>(null);

  const appendLog = useCallback(
    (line: string) => setLog((prev) => [...prev.slice(-49), line]),
    [],
  );

  const refreshWorkflows = useCallback(async () => {
    if (!projectRoot) return;
    try {
      setWorkflows(await listProjectWorkflows(projectRoot));
    } catch (error) {
      appendLog(`读工作流失败：${messageOf(error)}`);
    }
  }, [projectRoot, appendLog]);

  useEffect(() => {
    void refreshWorkflows();
  }, [refreshWorkflows]);

  // 链跑期间每 2.5s 轮询运行态（复用 #19 的只读命令）。
  useEffect(() => {
    if (!running || !activeWorkflowId) return;
    let active = true;
    const poll = async () => {
      try {
        const status = await getProjectWorkflowChainStatus(projectRoot, activeWorkflowId);
        if (active && status) setChainStatus(status);
      } catch {
        // 轮询失败不致命。
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [running, activeWorkflowId, projectRoot]);

  // 简单意图匹配：命令文本里包含哪个工作流标题就选哪个（取最长匹配，防短标题误命中）。
  const matchWorkflow = useCallback(
    (text: string): ProjectWorkflowListItem | null => {
      const hits = workflows
        .filter((w) => w.title && text.includes(w.title))
        .sort((a, b) => b.title.length - a.title.length);
      return hits[0] ?? null;
    },
    [workflows],
  );

  const send = useCallback(async () => {
    const text = command.trim();
    if (!text) return;
    appendLog(`> ${text}`);
    setCommand("");
    const isStop = /停|stop/i.test(text);
    const isStart = /跑|开干|按计划|启动|run|start/i.test(text);
    if (!isStop && !isStart) {
      appendLog("只认『按计划跑 <工作流名>』起链 / 『停 <工作流名>』停链——对话只启动·停，不改状态机。");
      return;
    }
    const wf = matchWorkflow(text);
    if (!wf) {
      appendLog(`没匹配到工作流。可用：${workflows.map((w) => w.title).join(" / ") || "（无）"}`);
      return;
    }
    if (isStop) {
      try {
        const result = await stopProjectWorkflowChain({
          project_root: projectRoot,
          workflow_id: wf.workflow_id,
        });
        appendLog(`已请求停「${wf.title}」：${result.message}`);
        onReloadWorkflowState?.();
      } catch (error) {
        appendLog(`停链失败：${messageOf(error)}`);
      }
      return;
    }
    // 起链（同步阻塞到整条链跑完/停下；期间可发『停 …』中断）。
    setRunning(true);
    setActiveWorkflowId(wf.workflow_id);
    appendLog(`▶▶ 起链「${wf.title}」…（圈固定测试项目、按拓扑序逐节点真跑；发『停 ${wf.title}』可中断）`);
    try {
      const result = await startProjectWorkflowChain({
        project_root: projectRoot,
        workflow_id: wf.workflow_id,
      });
      setChainStatus({
        chain_run_id: result.chain_run_id,
        state: result.state,
        nodes: result.nodes as { node_id: string; state: string }[],
      });
      appendLog(`链「${result.state}」：${result.dispatched_count} 节点已派发。${result.message}`);
      onNotice(`发令台：链「${result.state}」`);
      onReloadWorkflowState?.();
    } catch (error) {
      appendLog(`起链被拦截或失败：${messageOf(error)}`);
    } finally {
      setRunning(false);
    }
  }, [command, matchWorkflow, projectRoot, workflows, appendLog, onNotice, onReloadWorkflowState]);

  return (
    <div className="stage command-console-stage" style={{ display: "flex", flexDirection: "column", gap: 16, padding: 20, height: "100%", boxSizing: "border-box" }}>
      <header>
        <h2 style={{ margin: "0 0 4px" }}>发令台</h2>
        <p className="meta-text" style={{ margin: 0 }}>
          对工作流「发令」起链：打 <code>按计划跑 &lt;工作流名&gt;</code> / <code>停 &lt;工作流名&gt;</code>。
          链在固定测试项目真跑（path-lock + 沙箱）；对话只启动/停、不绕状态机。
        </p>
      </header>

      <section style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <span className="meta-text">项目：</span>
        {projects.length > 1 ? (
          <select value={projectRoot} onChange={(e) => setProjectRoot(e.target.value)}>
            {projects.map((p) => (
              <option key={p.project_root} value={p.project_root}>
                {p.name || p.project_root}
              </option>
            ))}
          </select>
        ) : (
          <code>{projectRoot}</code>
        )}
        {projectRoot !== TEST_PROJECT_ROOT ? (
          <span className="meta-text" style={{ color: "var(--warning)" }}>
            ⚠️ 非固定测试项目，链会被 path-lock 闸拒（零执行）
          </span>
        ) : null}
      </section>

      <section>
        <p className="meta-text" style={{ margin: "0 0 4px" }}>可发令的工作流（{workflows.length}）：</p>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {workflows.length ? (
            workflows.map((w) => (
              <button
                key={w.workflow_id}
                className="secondary-button"
                type="button"
                onClick={() => setCommand(`按计划跑 ${w.title}`)}
                title={`点这里把命令填成「按计划跑 ${w.title}」`}
              >
                {w.title}（{w.node_count} 节点）
              </button>
            ))
          ) : (
            <span className="meta-text">（无工作流；先去项目画布建一个）</span>
          )}
        </div>
      </section>

      {chainStatus ? (
        <section className="command-console-status" aria-label="链运行态">
          <span className="meta-text">链 {chainStatus.state}　</span>
          {chainStatus.nodes.map((n) => {
            const label = (n.node_id.split(":node:")[1] ?? n.node_id).split("-")[0];
            const icon =
              n.state === "completed" ? "✓" : n.state === "running" ? "⏳" : n.state === "failed" ? "✗" : "•";
            return (
              <span key={n.node_id} style={{ marginRight: 10 }}>
                {icon}
                {label}
              </span>
            );
          })}
        </section>
      ) : null}

      <section
        aria-label="命令日志"
        style={{
          flex: 1,
          minHeight: 120,
          overflowY: "auto",
          fontFamily: "monospace",
          fontSize: "var(--text-sm)",
          lineHeight: 1.6,
          border: "1px solid var(--hair)",
          borderRadius: 6,
          padding: 10,
          whiteSpace: "pre-wrap",
        }}
      >
        {log.map((line, i) => (
          <div key={i}>{line}</div>
        ))}
      </section>

      <section style={{ display: "flex", gap: 8 }}>
        <input
          type="text"
          value={command}
          placeholder="按计划跑 <工作流名>　/　停 <工作流名>"
          onChange={(e) => setCommand(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") void send();
          }}
          style={{ flex: 1 }}
        />
        <button className="primary-button" type="button" onClick={() => void send()} disabled={running}>
          {running ? "连环跑中…" : "发令"}
        </button>
      </section>
    </div>
  );
}
