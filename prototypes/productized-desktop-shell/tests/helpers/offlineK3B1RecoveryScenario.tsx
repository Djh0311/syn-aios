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

  for (const expectedText of [
    "K3-B1 被安全审查再次阻断",
    "会向外部服务发送项目/session 派生 prompt",
    "写入 Codex 本地状态",
    "手动运行并回交",
    "重新授权申请",
    "更窄本地执行桥",
    "结果数：未知/不可用",
    "K3-B2 继续阻断",
    "主管线复核前不改变成功状态",
    "K3_B1_REAL_EXECUTION_AUTHORIZED=stage-k-k3-b1-mario-test-workflow-read-only",
    "cargo test --lib project_workflow_automation::tests::k3_b1_real_mario_test_workflow_resume_requires_env_authorization -- --ignored --exact --nocapture",
    "ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039",
  ]) {
    assert(text.includes(expectedText), `L1 recovery 卡片缺少文案：${expectedText}`);
  }

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
