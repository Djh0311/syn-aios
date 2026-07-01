import React from "react";
import { ProjectDetail } from "../../src/views/ProjectsView";
import type { PendingAction, ProjectRecord, SessionRecord, WorkbenchSnapshot, WorkflowStateSnapshot } from "../../src/lib/types";
import { assert, visibleText } from "./offlineInteractionTestUtils";

export function runK3B1BlockedRecoveryProductPathScenario({
  snapshot,
  project,
  session,
  workflowStateWithProjectWorkflow,
  onRequestAction,
}: {
  snapshot: WorkbenchSnapshot;
  project: ProjectRecord;
  session: SessionRecord;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
  onRequestAction: (action: PendingAction) => void;
}) {
  const recovery = snapshot.k3_b1_recovery;
  assert(recovery, "L1 snapshot 应包含 K3-B1 recovery 读模型");
  assert(recovery.current_state === "blocked_by_safety_review_again", "L1 当前状态必须是安全审查再次阻断");
  assert(recovery.k3_b2_gate.blocked, "L1 不得解锁 K3-B2");
  assert(recovery.manual_submission_contract.auto_accepts_success === false, "L1 手动回交不得自动接受成功");
  assert(recovery.readback_boundary.result_count === null, "L1 未真实执行时 result_count 必须是 null");

  const text = visibleText(
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      projectWorkflowAutomation={snapshot.project_workflow_automation}
      k3B1Recovery={recovery}
      selectedTool="workflow"
      onRequestAction={onRequestAction}
    />,
  );

  // K3B1RecoveryCard 已从项目工作流侧栏删除（UI 删除任务），其正向文案断言整段移除；
  // 下方 forbiddenText 反向断言保留（防虚假声称），read model（recovery.*）断言也保留。

  for (const forbiddenText of [
    "K3-B1 retry 成功",
    "K3-B2 可开始",
    "自动重试已启用",
    "已完成真实恢复",
    "读回 0 条",
    "安全审查已绕过",
    "已获得通用真实执行授权",
  ]) {
    assert(!text.includes(forbiddenText), `L1 recovery 卡片不得显示误导文案：${forbiddenText}`);
  }
}
