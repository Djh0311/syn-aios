import React from "react";
import { PermissionDialog } from "../../src/components/PermissionDialog";
import type { PendingAction } from "../../src/lib/types";
import { assert, assertDeepEqual, findButtonByText, visibleText } from "./offlineInteractionTestUtils";

export type OfflinePermissionScenario = {
  name: string;
  root: React.ReactNode;
  buttonText: string;
  expectedAction: PendingAction;
};

export type CapturedActionState = {
  get: () => PendingAction | null;
  set: (action: PendingAction | null) => void;
};

export function runPermissionScenario(scenario: OfflinePermissionScenario, capturedAction: CapturedActionState) {
  capturedAction.set(null);
  const button = findButtonByText(scenario.root, scenario.buttonText);
  assert(button, `${scenario.name}: 找不到按钮 ${scenario.buttonText}`);
  const onClick = button.props?.onClick;
  assert(typeof onClick === "function", `${scenario.name}: 按钮没有 onClick`);

  onClick({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(capturedAction.get(), scenario.expectedAction, `${scenario.name}: 待确认动作不匹配`);

  let canceled = false;
  let confirmed = false;
  const dialog = (
    <PermissionDialog
      action={capturedAction.get()}
      busy={false}
      onCancel={() => {
        canceled = true;
      }}
      onConfirm={() => {
        confirmed = true;
      }}
    />
  );

  const text = visibleText(dialog);
  for (const expectedText of [
    "本机动作确认",
    scenario.expectedAction.label,
    "目标路径",
    scenario.expectedAction.path,
    "路径来源",
    scenario.expectedAction.source,
    "取消",
    expectedDialogConfirmLabel(scenario.expectedAction.kind),
  ]) {
    assert(text.includes(expectedText), `${scenario.name}: 弹层缺少文本 ${expectedText}`);
  }

  const cancelButton = findButtonByText(dialog, "取消");
  assert(cancelButton, `${scenario.name}: 找不到取消按钮`);
  const cancel = cancelButton.props?.onClick;
  assert(typeof cancel === "function", `${scenario.name}: 取消按钮没有 onClick`);
  cancel({ preventDefault() {}, stopPropagation() {} });
  assert(canceled, `${scenario.name}: 取消按钮没有触发关闭回调`);
  assert(!confirmed, `${scenario.name}: 测试不应触发确认执行`);
}

function expectedDialogConfirmLabel(kind: PendingAction["kind"]) {
  if (kind === "run-workflow-machine") return "确认启动多轮真实执行";
  if (kind === "execute-node-dispatch") return "确认真实派发";
  if (kind === "copy-task-preview") return "确认复制";
  if (
    kind === "initialize-workflow-state" ||
    kind === "bootstrap-project-workflow" ||
    kind === "update-task-fields" ||
    kind === "correct-dispatch-fields" ||
    kind === "advance-work-item-state" ||
    kind === "bind-node-session" ||
    kind === "unbind-node-session"
  ) {
    return "确认写入状态";
  }
  if (
    kind === "record-director-review" ||
    kind === "record-permission-decision" ||
    kind === "record-worker-structured-report" ||
    kind === "record-project-director-process-fact-decision" ||
    kind === "record-global-final-result-review" ||
    kind === "generate-stage-c-acceptance-summary" ||
    kind === "offline-role-dispatch" ||
    kind === "offline-role-result-handoff" ||
    kind === "offline-director-review"
  ) {
    return "确认记录";
  }
  if (
    kind === "record-blackboard-candidate-decision" ||
    kind === "record-memory-candidate-decision" ||
    kind === "record-memory-entity-alias-decision" ||
    kind === "record-memory-entity-merge-decision" ||
    kind === "record-memory-relation-candidate-decision" ||
    kind === "record-mature-pattern-decision" ||
    kind === "record-project-consultation-proposal-decision" ||
    kind === "record-global-boundary-review" ||
    kind === "record-user-result-decision"
  ) {
    return "确认提交决定";
  }
  if (kind === "create-task-draft") return "确认创建草稿";
  if (
    kind === "create-memory-candidate" ||
    kind === "create-memory-candidate-from-observation" ||
    kind === "adopt-memory-candidate-to-formal-memory"
  ) {
    return "确认创建候选";
  }
  if (
    kind === "generate-task-file" ||
    kind === "record-formal-memory-lifecycle-operation" ||
    kind === "run-memory-maintenance" ||
    kind === "create-project-consultation-proposal" ||
    kind === "prepare-authorized-auto-dispatch"
  ) {
    return "确认创建记录";
  }
  if (kind === "preview-user-reviewed-instruction") return "确认边界预览";
  return "确认继续";
}
