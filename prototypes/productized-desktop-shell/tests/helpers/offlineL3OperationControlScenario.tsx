import React from "react";
import { PermissionDialog } from "../../src/components/PermissionDialog";
import type {
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  PendingAction,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "../../src/lib/types";
import { executionRunQueueTextFixtures } from "./offlineExecutionRunQueueTextFixtures";
import { assert, visibleText } from "./offlineInteractionTestUtils";

// G4 宿主迁移(2026-07-20·预登记 M2)：原宿主=RunningWorkflowsView(死视图整删)。
// L3 决策面/按钮生成面=死视图内联 JSX 壳,无活宿主(App 零 import 实证)——壳断言退役(预登记 R5-R7);
// 弹窗语义=PermissionDialog 活组件,迁移为字面 action 直渲,断言逐字平移,覆盖不丢。
const operationDecisionAction: PendingAction = {
  kind: "record-operation-control-decision",
  label: "确认记录 重试 决策",
  path: "workbench://operation-control/retry",
  source: "Tauri 应用数据目录",
  boundary: "L3 只登记运行控制决策和待处理状态；不调用 runner、不执行 Codex、不停止或重启真实进程。",
  operationControlAction: {
    operation_id: "retry",
    label: "重试",
    current_status: "available",
    status_after_confirmation: "confirmed_recorded",
    current_gate: "requires_user_confirmation_and_new_authorized_window",
    would_write_if_real: "workbench_state_only",
    risk_disclosure: "重试在 L3 只会记录决策，不会触发真实操作。",
    readback_status: "not_attempted_l3_decision_only",
    readback_result_count: null,
    audit_event_type: "operation_decision_recorded",
    runtime_status_after_confirmation: "operation_decision_recorded_pending_real_authorization",
    does_execute_in_l3: false,
    requires_separate_authorized_window: true,
    blocks_k3_b2: true,
  },
};

export function runL3OperationControlScenario({
  snapshot: _snapshot,
  workflowStateWithProjectWorkflow: _workflowStateWithProjectWorkflow,
  memoryCaptureStore: _memoryCaptureStore,
  memoryCandidateStore: _memoryCandidateStore,
}: {
  snapshot: WorkbenchSnapshot;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  memoryCaptureStore: MemoryCaptureStoreV1;
  memoryCandidateStore: MemoryCandidateStoreV1;
}) {
  // 语义核(原按钮生成断言的语义部分,R6 保留):decision-only action 形状逐字锁。
  assert(
    operationDecisionAction.operationControlAction?.does_execute_in_l3 === false,
    "L3 操作控制 action 不应声明执行",
  );
  assert(
    operationDecisionAction.operationControlAction?.status_after_confirmation === "confirmed_recorded",
    "L3 操作控制确认后状态不匹配",
  );
  assert(
    operationDecisionAction.operationControlAction?.readback_result_count === null,
    "L3 操作控制 readback 结果数应为 null",
  );

  const l3DialogText = visibleText(
    <PermissionDialog action={operationDecisionAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of executionRunQueueTextFixtures.l3OperationControlDialogExpectedTexts) {
    assert(l3DialogText.includes(expectedText), `L3 操作控制弹窗缺少 ${expectedText}`);
  }
  for (const forbiddenText of executionRunQueueTextFixtures.l3OperationControlForbiddenTexts) {
    assert(!l3DialogText.includes(forbiddenText), `L3 操作控制弹窗不应出现 ${forbiddenText}`);
  }
}
