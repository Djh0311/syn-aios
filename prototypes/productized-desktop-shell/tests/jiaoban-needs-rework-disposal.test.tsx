import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanNeedsReworkDisposal } from "../src/views/projects/ProjectJiaobanPanel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-needs-rework-disposal] ${message}`);
}

const actions: string[] = [];
const element = JiaobanNeedsReworkDisposal({
  reason: "验收证据还不够，先补齐再交货。",
  actionsReady: true,
  starting: false,
  error: null,
  onContinue: () => actions.push("continue"),
  onAction: (action) => actions.push(action),
});
const output = renderToStaticMarkup(element);

assert(output.includes("主管退回理由：验收证据还不够"), "主界面应显示主管退回理由");
for (const label of ["接着跑（按原样重做）", "换个新会话重做", "退回主管重拆", "结束这单"]) {
  assert(output.includes(label), `主界面应给出「${label}」`);
}
assert(!output.includes("planned_task_id") && !output.includes("chain_run_id"), "按钮不应暴露内部标识");

const buttons: Array<{ props: { children?: unknown; onClick?: () => void } }> = [];
const walk = (node: unknown) => {
  if (!node || typeof node !== "object") return;
  const elementNode = node as {
    type?: unknown;
    props?: { children?: unknown; onClick?: () => void };
  };
  if (elementNode.type === "button" && elementNode.props) buttons.push(elementNode as (typeof buttons)[number]);
  const children = elementNode.props?.children;
  if (Array.isArray(children)) children.forEach(walk);
  else if (children) walk(children);
};
walk(element);
for (const button of buttons) button.props.onClick?.();

assert(
  actions.join(",") === "continue,change_session,rework,archive",
  `四个按钮应走各自用户动作，实际：${actions.join(",")}`,
);

console.log("jiaoban-needs-rework-disposal: 主界面四选一离线 DOM 断言全过");
