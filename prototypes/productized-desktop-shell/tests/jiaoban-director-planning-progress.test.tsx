import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanRunningState,
  isDirectorPlanningPhase,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { ProjectWorkflowChainStatus } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-director-planning-progress] ${message}`);
}

const noop = () => {};
const runningChain: ProjectWorkflowChainStatus = {
  chain_run_id: "planning-progress-chain",
  state: "running",
  nodes: [{ node_id: "first-task", state: "running" }],
};

const planningOutput = renderToStaticMarkup(
  <JiaobanRunningState
    chainStatus={null}
    directorPlanningElapsedMinutes={1}
    isNewSession={false}
    onStop={noop}
    sessionChoice={null}
    latestSessionThreadId={null}
  />,
);
assert(planningOutput.includes("主管正在拆任务 · 已 1 分钟"), "拆任务期必须显示已等待分钟数");

const longPlanningOutput = renderToStaticMarkup(
  <JiaobanRunningState
    chainStatus={null}
    directorPlanningElapsedMinutes={2}
    isNewSession={false}
    onStop={noop}
    sessionChoice={null}
    latestSessionThreadId={null}
  />,
);
assert(longPlanningOutput.includes("模型在长考;若超时会自动停下重试,不用干等"), "两分钟后必须给出等待说明");

const runningOutput = renderToStaticMarkup(
  <JiaobanRunningState
    chainStatus={runningChain}
    directorPlanningElapsedMinutes={9}
    isNewSession={false}
    onStop={noop}
    sessionChoice={null}
    latestSessionThreadId={null}
  />,
);
assert(!runningOutput.includes("已 9 分钟"), "主管拆完进入执行期后不得残留拆任务计时");
assert(!runningOutput.includes("模型在长考"), "主管拆完后不得残留拆任务等待说明");
assert(isDirectorPlanningPhase("running", null), "运行且无链时才开启拆任务计时");
assert(!isDirectorPlanningPhase("blocked", null), "退出运行相位必须关闭拆任务计时");
assert(!isDirectorPlanningPhase("done", runningChain), "终态不得保留拆任务计时");

console.log("jiaoban-director-planning-progress: 拆任务分钟、人话和相位退出清理离线 DOM 断言全过");
