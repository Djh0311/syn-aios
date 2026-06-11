import type { PendingAction, WorkflowUserReviewedInstruction } from "../../src/lib/types";

export function buildBootstrapProjectWorkflowAction(projectRoot: string): PendingAction {
  return {
    kind: "bootstrap-project-workflow",
    label: "创建项目默认工作流草稿",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "给工作台自己的 workflow-state.v0.json 写入项目、workflow、默认节点、默认边和 audit；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
  };
}

export function buildUserReviewedInstructionPreviewAction(
  projectRoot: string,
  instruction: WorkflowUserReviewedInstruction,
): PendingAction {
  return {
    kind: "preview-user-reviewed-instruction",
    label: "确认用户审核业务指令边界",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只确认用户审核业务指令的结构化预览和边界；本版本不执行 codex exec resume、不发送 Codex 消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
    userReviewedInstruction: instruction,
  };
}

export function buildPermissionDecisionAction(projectRoot: string): PendingAction {
  return {
    kind: "record-permission-decision",
    label: "记录权限结论：批准",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只在用户确认后通过控制核心记录权限请求结论并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。",
    permissionDecision: {
      project_root: projectRoot,
      work_item_id: "work-item:offline:001",
      request_id: "permission:offline:001",
      decision: "approved",
    },
  };
}

export function buildBindNodeSessionAction(projectRoot: string): PendingAction {
  return {
    kind: "bind-node-session",
    label: "绑定节点 Codex 会话",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只把已有索引 Codex 会话绑定到工作台自己的 workflow-state.v0.json；不启动 Codex、不发送消息、不恢复会话、不读取完整会话正文、不写 Codex 状态库。",
    nodeSessionBinding: {
      project_root: projectRoot,
      node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
      work_item_id: "work-item:offline:001",
      thread_id: "offline-thread-001",
    },
  };
}

export function buildUnbindNodeSessionAction(projectRoot: string): PendingAction {
  return {
    kind: "unbind-node-session",
    label: "解除节点会话绑定",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只解除工作台自己的 workflow-state.v0.json 绑定并追加审计事件；不删除、不移动、不归档 Codex 原始会话；不写 .codex 或 Codex 状态库。",
    nodeSessionUnbinding: {
      project_root: projectRoot,
      binding_id: "binding:offline:codex-dev",
    },
  };
}

export function buildAdvanceWorkItemStateAction(projectRoot: string): PendingAction {
  return {
    kind: "advance-work-item-state",
    label: "推进工作项到执行中",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只写工作台自己的 workflow-state.v0.json；追加审计事件；不启动 Codex 命令行、不恢复会话、不派发真实 Codex 会话、不运行运行器、不写 .codex 或 Codex 状态库。",
    workItemStateUpdate: {
      project_root: projectRoot,
      work_item_id: "work-item:offline:001",
      next_state: "running",
    },
  };
}

export function expectedInitializeWorkflowStateAction(workflowStatePath: string): PendingAction {
  return {
    kind: "initialize-workflow-state",
    label: "初始化工作流事实层",
    path: workflowStatePath,
    source: "Tauri 应用数据目录",
    boundary: "只写 workflow-state.v0.json 和同目录备份；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
  };
}
