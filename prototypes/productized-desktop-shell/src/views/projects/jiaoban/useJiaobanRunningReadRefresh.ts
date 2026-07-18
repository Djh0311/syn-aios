import { useCallback, useEffect, useRef } from "react";
import { getProjectWorkflowChainStatus } from "../../../lib/tauri";
import type { ProjectWorkflowChainStatus } from "../../../lib/types";
import type { JiaobanPhase } from "./JiaobanArtifactViews";

type UseJiaobanRunningReadRefreshArgs = {
  manualPhase: JiaobanPhase | null;
  projectRoot: string | null;
  workflowId: string | null;
  onChainStatus: (status: ProjectWorkflowChainStatus) => void;
  onWorkflowStateReadRefresh?: () => Promise<void>;
};

// P3-A 专用：跑态先读既有 chain status，再补读只读 workflow snapshot（含黑板过程消息）。
// 这是展示读面，不写链、黑板或终标；成功链操作可排一笔强制补读，普通 tick 不并发堆积。
export function useJiaobanRunningReadRefresh({
  manualPhase,
  projectRoot,
  workflowId,
  onChainStatus,
  onWorkflowStateReadRefresh,
}: UseJiaobanRunningReadRefreshArgs): () => void {
  // 全量 derive 有可感成本，故串行；同一 status 下仍可能新增审计，禁签名去重。
  const refreshInFlightRef = useRef(false);
  const pendingActionRefreshRef = useRef(false);
  const refreshCallbackRef = useRef(onWorkflowStateReadRefresh);

  useEffect(() => {
    refreshCallbackRef.current = onWorkflowStateReadRefresh;
  }, [onWorkflowStateReadRefresh]);

  const requestSnapshotRefresh = useCallback((queueAfterInFlight: boolean) => {
    const refresh = refreshCallbackRef.current;
    if (!refresh) {
      pendingActionRefreshRef.current = false;
      return;
    }
    if (refreshInFlightRef.current) {
      if (queueAfterInFlight) pendingActionRefreshRef.current = true;
      return;
    }
    refreshInFlightRef.current = true;
    const finish = () => {
      refreshInFlightRef.current = false;
      if (!pendingActionRefreshRef.current) return;
      pendingActionRefreshRef.current = false;
      requestSnapshotRefresh(false);
    };
    try {
      void refresh().catch(() => {}).finally(finish);
    } catch {
      // 读面同步适配错误不能覆盖已经成功的链操作。
      finish();
    }
  }, []);

  const refreshAfterSuccessfulChainAction = useCallback(() => {
    requestSnapshotRefresh(true);
  }, [requestSnapshotRefresh]);

  useEffect(() => {
    // 只依赖定位原语：workflow snapshot 重建对象不能 cleanup→首 tick 自触发。
    if (manualPhase !== "running" || !projectRoot || !workflowId) return;
    let active = true;
    const poll = async () => {
      try {
        const status = await getProjectWorkflowChainStatus(projectRoot, workflowId);
        if (!active) return;
        if (status) onChainStatus(status);
        // 无 audit cursor：每次成功 status tick 都请求全量派生；上一笔未完便跳过本 tick。
        requestSnapshotRefresh(false);
      } catch {
        // 轮询失败不致命——进度暂缺不影响永不冻。
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [manualPhase, onChainStatus, projectRoot, requestSnapshotRefresh, workflowId]);

  return refreshAfterSuccessfulChainAction;
}
