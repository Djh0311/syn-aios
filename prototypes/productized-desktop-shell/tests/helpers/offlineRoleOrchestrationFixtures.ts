import type { OfflineRoleDispatchProposal, PendingAction } from "../../src/lib/types";

export const missingOfflineDispatchBlock = "派发给：开发线\n任务名：缺字段测试";

export function expectedOfflineRoleDispatchAction(
  projectRoot: string,
  workItemId: string,
  proposal: OfflineRoleDispatchProposal,
): PendingAction {
  return {
    kind: "offline-role-dispatch",
    label: "离线派发给开发线",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只把角色派发块写入工作台自己的 workflow-state.v0.json；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。",
    offlineRoleDispatch: {
      ...proposal,
      work_item_id: workItemId,
    },
  };
}

export function offlineRoleDispatchFormDataFixture(dispatchBlock: string, directorRequest = "请总指导拆给开发线。"): typeof FormData {
  return class {
    get(name: string) {
      if (name === "dispatch-block") return dispatchBlock;
      if (name === "director-request") return directorRequest;
      return null;
    }
  } as unknown as typeof FormData;
}

export function missingOfflineRoleDispatchFormDataFixture(): typeof FormData {
  return offlineRoleDispatchFormDataFixture(missingOfflineDispatchBlock);
}
