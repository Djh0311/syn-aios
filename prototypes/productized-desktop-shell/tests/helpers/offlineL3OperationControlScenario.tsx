import React from "react";
import { PermissionDialog } from "../../src/components/PermissionDialog";
import type {
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  PendingAction,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../../src/lib/types";
import { RunningWorkflowsView } from "../../src/views/RunningWorkflowsView";
import { executionRunQueueTextFixtures } from "./offlineExecutionRunQueueTextFixtures";
import { assert, findButtonByText, visibleText } from "./offlineInteractionTestUtils";

export function runL3OperationControlScenario({
  snapshot,
  workflowStateWithProjectWorkflow,
  memoryCaptureStore,
  memoryCandidateStore,
}: {
  snapshot: WorkbenchSnapshot;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  memoryCaptureStore: MemoryCaptureStoreV1;
  memoryCandidateStore: MemoryCandidateStoreV1;
}) {
  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );
  for (const expectedText of executionRunQueueTextFixtures.stageJRunningExpectedTexts) {
    assert(runningText.includes(expectedText), `K5 Running UI 缺少 ${expectedText}`);
  }
  for (const expectedText of executionRunQueueTextFixtures.l3OperationControlExpectedTexts) {
    assert(runningText.includes(expectedText), `L3 Running 操作控制缺少 ${expectedText}`);
  }

  const capturedActions: PendingAction[] = [];
  const runningControl = (
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
      onRequestAction={(action) => {
        capturedActions.push(action);
      }}
    />
  );
  for (const label of ["确认记录 重试 决策", "确认记录 停止 决策", "确认记录 重启 决策", "确认记录 恢复 决策"]) {
    capturedActions.length = 0;
    const button = findButtonByText(runningControl, label);
    assert(button, `L3 操作控制缺少确认按钮 ${label}`);
    const onClick = button.props?.onClick;
    assert(typeof onClick === "function", `L3 操作控制按钮 ${label} 没有 onClick`);
    onClick({ preventDefault() {}, stopPropagation() {} });
    const action = capturedActions[0];
    assert(action?.kind === "record-operation-control-decision", `L3 操作控制 ${label} 应生成 decision-only action`);
    assert(action.operationControlAction?.does_execute_in_l3 === false, `L3 操作控制 ${label} 不应声明执行`);
    assert(
      action.operationControlAction?.status_after_confirmation === "confirmed_recorded",
      `L3 操作控制 ${label} 确认后状态不匹配`,
    );
    assert(action.operationControlAction?.readback_result_count === null, `L3 操作控制 ${label} readback 结果数应为 null`);
  }

  const capturedAction = capturedActions[0] ?? null;
  const l3DialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of executionRunQueueTextFixtures.l3OperationControlDialogExpectedTexts) {
    assert(l3DialogText.includes(expectedText), `L3 操作控制弹窗缺少 ${expectedText}`);
  }
  for (const forbiddenText of executionRunQueueTextFixtures.l3OperationControlForbiddenTexts) {
    assert(!l3DialogText.includes(forbiddenText), `L3 操作控制弹窗不应出现 ${forbiddenText}`);
  }

  const confirmedSnapshot: WorkbenchSnapshot = {
    ...snapshot,
    operation_control: snapshot.operation_control
      ? {
          ...snapshot.operation_control,
          operations: snapshot.operation_control.operations.map((operation) =>
            operation.operation_id === "retry"
              ? {
                  ...operation,
                  status: "confirmed_recorded",
                  confirmation_label: `已记录 ${operation.label} 决策`,
                  user_visible_summary: `${operation.label} 决策已从 workflow-state audit 投影为 confirmed_recorded；仍未执行真实操作。`,
                  warnings: [...operation.warnings, "decision_audit_recorded"],
                }
              : operation,
          ),
        }
      : snapshot.operation_control,
  };
  const confirmedControl = (
    <RunningWorkflowsView
      snapshot={confirmedSnapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
      onRequestAction={() => {
        throw new Error("confirmed_recorded operation should not dispatch another L3 action");
      }}
    />
  );
  const confirmedText = visibleText(confirmedControl);
  assert(confirmedText.includes("决策已登记"), "L3 已记录操作应显示决策已登记");
  assert(confirmedText.includes("仍未执行真实操作"), "L3 已记录操作不能显示为已执行");
  const confirmedButton = findButtonByText(confirmedControl, "已记录 重试 决策");
  assert(confirmedButton?.props?.disabled === true, "L3 已记录操作按钮应禁用以避免重复确认");
}
