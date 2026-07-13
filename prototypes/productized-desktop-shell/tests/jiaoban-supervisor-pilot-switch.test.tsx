import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanOrchestrationModePicker,
  JiaobanSupervisorPilotRunningState,
  STATION_3B_READONLY_PROJECT_ROOT,
  classicModeUnavailableReason,
  supervisorPilotUnavailableReason,
} from "../src/views/projects/ProjectJiaobanPanel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-supervisor-pilot-switch] ${message}`);
}

const testProjectRoot = "/Users/yoyi/codex-workflow-mario-test";

assert(
  supervisorPilotUnavailableReason(testProjectRoot, [testProjectRoot]) === null,
  "固定测试项目写单应允许选择主管试点",
);
assert(
  supervisorPilotUnavailableReason(testProjectRoot, [`${testProjectRoot}/subdir`]) ===
    "主管编排写入试点只允许固定测试项目根。",
  "测试项目内的非精确写根也必须拒绝",
);
assert(
  supervisorPilotUnavailableReason("/tmp/not-the-test-project", []) === "主管编排试点仅限固定测试项目。",
  "非测试项目必须禁用主管试点",
);
assert(
  supervisorPilotUnavailableReason(testProjectRoot, []) === null,
  "只读测试单应允许选择主管试点",
);

// ===== 站 3b（2026-07-12 拍板）：真实项目 mario test 只读入口 =====
assert(STATION_3B_READONLY_PROJECT_ROOT === "/Users/yoyi/Documents/mario test", "3b 常量必须与拍板路径一致");
assert(
  supervisorPilotUnavailableReason(STATION_3B_READONLY_PROJECT_ROOT, []) === null,
  "3b 项目只读单应允许选择主管试点",
);
assert(
  supervisorPilotUnavailableReason(STATION_3B_READONLY_PROJECT_ROOT, [STATION_3B_READONLY_PROJECT_ROOT]) ===
    "站 3b 项目仅限只读单（零写根）。",
  "3b 项目带写根（即使写根就是它自己）必须拒绝主管试点",
);
assert(
  supervisorPilotUnavailableReason(STATION_3B_READONLY_PROJECT_ROOT, [testProjectRoot]) ===
    "站 3b 项目仅限只读单（零写根）。",
  "3b 项目带测试根写根同样拒绝",
);
assert(
  supervisorPilotUnavailableReason(`${STATION_3B_READONLY_PROJECT_ROOT}/subdir`, []) ===
    "主管编排试点仅限固定测试项目。",
  "3b 子目录不是 3b 项目（精确相等，无前缀魔法）",
);
assert(
  classicModeUnavailableReason(STATION_3B_READONLY_PROJECT_ROOT) ===
    "站 3b 项目只开通主管编排只读单；经典状态机仍锁固定测试项目。",
  "3b 项目经典模式必须锁且给人话理由",
);
assert(classicModeUnavailableReason(testProjectRoot) === null, "测试项目经典模式不受 3b 影响");

const station3bPickerMarkup = renderToStaticMarkup(
  <JiaobanOrchestrationModePicker
    mode="supervisor_pilot"
    disabled={false}
    disabledReason={null}
    classicDisabledReason={classicModeUnavailableReason(STATION_3B_READONLY_PROJECT_ROOT)}
    onChange={() => {}}
  />,
);
assert(
  station3bPickerMarkup.includes("站 3b 项目只开通主管编排只读单"),
  "3b 经典锁理由应上脸",
);
assert(station3bPickerMarkup.includes("disabled=\"\""), "3b 下经典 radio 应灰显");

const disabledMarkup = renderToStaticMarkup(
  <JiaobanOrchestrationModePicker
    mode="classic"
    disabled={false}
    disabledReason="主管编排写入试点只允许固定测试项目根。"
    onChange={() => {}}
  />,
);
assert(disabledMarkup.includes("主管编排写入试点只允许固定测试项目根。"), "禁用原因应上脸");
assert(disabledMarkup.includes("disabled=\"\""), "禁用的试点 radio 应灰显");

const enabledMarkup = renderToStaticMarkup(
  <JiaobanOrchestrationModePicker
    mode="supervisor_pilot"
    disabled={false}
    disabledReason={null}
    onChange={() => {}}
  />,
);
assert(enabledMarkup.includes("主管编排（试点）"), "合格单应显示主管试点模式");
assert(!enabledMarkup.includes("disabled=\"\""), "合格单的主管试点不应灰显");

const runningMarkup = renderToStaticMarkup(
  <JiaobanSupervisorPilotRunningState
    runId="supervisor:fixture:1"
    ledgerError={null}
    readModel={{
      run_id: "supervisor:fixture:1",
      launch_status: "running",
      project_root: testProjectRoot,
      workflow_id: "workflow:fixture",
      authorization_id: "authorization:fixture",
      started_at_ms: 1,
      ended_at_ms: null,
      termination_reason: "",
      metrics: {
        denied_tool_call_count: 0,
        max_follow_ups_per_worker: 2,
        follow_up_count: 1,
        follow_up_budget_respected: true,
        max_runtime_minutes: 30,
        session_timed_out: false,
        ledger_replay_event_count: 1,
        ledger_replay_ready: true,
      },
      audit_events: [
        {
          event_id: "audit:fixture",
          tool: "supervisor_dispatch_worker",
          result_summary: "已登记任务包",
          result_status: "accepted",
          created_at_ms: 1,
        },
      ],
    }}
  />,
);
assert(runningMarkup.includes("主管进行中"), "试点运行态应上脸");
assert(runningMarkup.includes("supervisor_dispatch_worker：已登记任务包"), "账本事件流应上脸");

const exitedMarkup = renderToStaticMarkup(
  <JiaobanSupervisorPilotRunningState
    runId="supervisor:fixture:2"
    ledgerError={null}
    readModel={{
      run_id: "supervisor:fixture:2",
      launch_status: "exited",
      project_root: testProjectRoot,
      workflow_id: "workflow:fixture",
      authorization_id: "authorization:fixture",
      started_at_ms: 1,
      ended_at_ms: 2,
      termination_reason: "completed",
      metrics: {
        denied_tool_call_count: 0,
        max_follow_ups_per_worker: 2,
        follow_up_count: 0,
        follow_up_budget_respected: true,
        max_runtime_minutes: 30,
        session_timed_out: false,
        ledger_replay_event_count: 0,
        ledger_replay_ready: true,
      },
      audit_events: [],
    }}
  />,
);
assert(!exitedMarkup.includes("主管进行中…"), "主管终态不能保留进行中横幅");
assert(exitedMarkup.includes("主管已结束"), "主管终态应显示已结束横幅");

const protocolInvalidMarkup = renderToStaticMarkup(
  <JiaobanSupervisorPilotRunningState
    runId="supervisor:fixture:3"
    ledgerError={null}
    readModel={{
      run_id: "supervisor:fixture:3",
      launch_status: "waiting_user",
      project_root: testProjectRoot,
      workflow_id: "workflow:fixture",
      authorization_id: "authorization:fixture",
      started_at_ms: 1,
      ended_at_ms: 2,
      termination_reason: "主管连续两次输出格式错误，本单未执行",
      metrics: {
        denied_tool_call_count: 0,
        max_follow_ups_per_worker: 2,
        follow_up_count: 0,
        follow_up_budget_respected: true,
        max_runtime_minutes: 30,
        session_timed_out: false,
        ledger_replay_event_count: 2,
        ledger_replay_ready: true,
      },
      audit_events: [
        {
          event_id: "audit:protocol-invalid",
          tool: "supervisor_action_protocol",
          result_summary: "主管连续两次输出格式错误，本单未执行。第二次错误：未知字段：node_id",
          result_status: "waiting_user",
          created_at_ms: 2,
        },
      ],
    }}
  />,
);
assert(protocolInvalidMarkup.includes("主管连续两次输出格式错误，本单未执行"), "连续格式错误终态应上脸");
assert(protocolInvalidMarkup.includes("未知字段：node_id"), "页面必须显示真实格式错误原因");

const dispatchedThenInvalidMarkup = renderToStaticMarkup(
  <JiaobanSupervisorPilotRunningState
    runId="supervisor:fixture:4"
    ledgerError={null}
    readModel={{
      run_id: "supervisor:fixture:4",
      launch_status: "waiting_user",
      project_root: testProjectRoot,
      workflow_id: "workflow:fixture",
      authorization_id: "authorization:fixture",
      started_at_ms: 1,
      ended_at_ms: 2,
      termination_reason:
        "主管连续两次输出格式错误，当前无效动作未执行。本单此前已派发 1 个 worker；最近 worker worker-1 当前状态 completed，结果：已派发。",
      metrics: {
        denied_tool_call_count: 0,
        max_follow_ups_per_worker: 2,
        follow_up_count: 0,
        follow_up_budget_respected: true,
        max_runtime_minutes: 30,
        session_timed_out: false,
        ledger_replay_event_count: 3,
        ledger_replay_ready: true,
      },
      audit_events: [
        {
          event_id: "audit:prior-worker",
          tool: "supervisor_action_protocol",
          result_summary:
            "主管连续两次输出格式错误，当前无效动作未执行。本单此前已派发 1 个 worker；最近 worker worker-1 当前状态 completed，结果：已派发。第二次错误：target.worker_id 不允许",
          result_status: "waiting_user",
          created_at_ms: 2,
        },
      ],
    }}
  />,
);
assert(dispatchedThenInvalidMarkup.includes("此前已派发 1 个 worker"), "已有 worker 必须如实上脸");
assert(!dispatchedThenInvalidMarkup.includes("本单未执行"), "已有 worker 时不得谎称整单未执行");
