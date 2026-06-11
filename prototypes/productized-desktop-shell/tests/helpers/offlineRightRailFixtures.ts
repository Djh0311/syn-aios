import type { SecretaryContext } from "../../src/lib/secretaryReadModel";
import type { MemoryCandidateStoreV1, MemoryCaptureStoreV1, WorkbenchSnapshot, WorkflowStateSnapshot } from "../../src/lib/types";
import type { ViewKey } from "../../src/lib/workbenchNavigation";

export type RightDetailPanelCommonPropsFixture = {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  notice: string;
  error: boolean;
  workflowStateError: string | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  secretaryContext: SecretaryContext;
  onClose: () => void;
  onNavigate: (view: ViewKey) => void;
  onReloadWorkflowState: () => void;
};

export function rightDetailPanelCommonPropsFixture({
  snapshot,
  workflowState,
  secretaryContext,
  memoryCaptureStore = null,
  memoryCandidateStore = null,
  notice = "offline notice",
  error = false,
  workflowStateError = null,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState: WorkflowStateSnapshot | null;
  secretaryContext: SecretaryContext;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  notice?: string;
  error?: boolean;
  workflowStateError?: string | null;
}): RightDetailPanelCommonPropsFixture {
  return {
    snapshot,
    workflowState,
    notice,
    error,
    workflowStateError,
    memoryCaptureStore,
    memoryCandidateStore,
    secretaryContext,
    onClose: () => {},
    onNavigate: () => {},
    onReloadWorkflowState: () => {},
  };
}

export const rightRailPanelSummaryTitles = {
  notifications: "通知摘要",
  todos: "待处理事项",
  audit: "管理摘要",
  running: "运行中摘要",
} as const;
