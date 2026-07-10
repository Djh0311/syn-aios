import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanTaskSessionBindingState } from "../src/views/projects/ProjectJiaobanPanel";
import type { ProjectDirectorPlannedTask } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-task-session-binding] ${message}`);
}

const tasks: ProjectDirectorPlannedTask[] = [
  {
    planned_task_id: "task-internal-no-display-1",
    title: "搭好页面骨架",
    objective: "搭好页面骨架",
    scope: {
      project_id: "project-1",
      workflow_id: "workflow-1",
      target_role: "codex-dev",
      task_package_kind: "implementation",
      allowed_read_scope: [],
      allowed_write_scope: [],
      callable_tool_capabilities: [],
      required_checks: [],
      stop_conditions: [],
    },
    depends_on: [],
    acceptance_criteria: [],
    report_format: [],
    status: "planned",
    blocked_reasons: [],
  },
  {
    planned_task_id: "task-internal-no-display-2",
    title: "补上验收",
    objective: "补上验收",
    scope: {
      project_id: "project-1",
      workflow_id: "workflow-1",
      target_role: "codex-dev",
      task_package_kind: "implementation",
      allowed_read_scope: [],
      allowed_write_scope: [],
      callable_tool_capabilities: [],
      required_checks: [],
      stop_conditions: [],
    },
    depends_on: [],
    acceptance_criteria: [],
    report_format: [],
    status: "planned",
    blocked_reasons: [],
  },
];

const output = renderToStaticMarkup(
  <JiaobanTaskSessionBindingState
    tasks={tasks}
    sessions={[]}
    bindings={tasks.map((task) => ({
      planned_task_id: task.planned_task_id,
      session_choice: "new",
    }))}
    error={null}
    starting={false}
    onBindingChange={() => {}}
    onStart={() => {}}
    onReplan={() => {}}
    onStop={() => {}}
  />,
);

assert(output.includes("先给每项任务选对话"), "应有逐任务绑定标题");
assert(output.includes("每个任务默认用自己的新会话"), "应说明默认逐任务新会话");
assert(output.includes("搭好页面骨架") && output.includes("补上验收"), "应展示任务标题");
assert(output.includes("开始跑"), "映射齐全时应有开始跑");
assert(output.includes("开个新的（为这项任务新建一个对话）"), "无旧会话时仍可选新会话");
assert(!output.includes("task-internal-no-display"), "界面不得暴露原始任务编号");

console.log("jiaoban-task-session-binding: 逐任务会话绑定面板离线 DOM 断言全过");
