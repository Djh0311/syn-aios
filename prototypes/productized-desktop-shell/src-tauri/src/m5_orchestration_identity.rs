// SYN-PRJ-001: M5 编排身份标识合同
//
// 本模块定义 M5 全链稳定的编排身份标识类型。
// 这些类型在整个编排生命周期中保持不变，用于精确 join 各个阶段的对象。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 关联 ID - 贯穿整个编排链的顶层标识
///
/// 用于关联所有相关对象：Proposal、Authorization、Grant、Dispatch、Report、Review
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct CorrelationId(pub(crate) String);

impl CorrelationId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "corr:{}", self.0)
    }
}

/// 编排 ID - 唯一标识一次编排实例
///
/// 每次 Proposal -> Authorization -> Execution 链路使用唯一的 OrchestrationId
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct OrchestrationId(pub(crate) String);

impl OrchestrationId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrchestrationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "orch:{}", self.0)
    }
}

/// 工作流运行 ID - 标识一次具体的工作流执行
///
/// 一个 OrchestrationId 可能包含多个 WorkflowRunId（如重试）
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct WorkflowRunId(pub(crate) String);

impl WorkflowRunId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowRunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run:{}", self.0)
    }
}

/// 编排身份标识组合
///
/// 在整个编排生命周期中保持不变的三元组
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct OrchestrationIdentity {
    /// 顶层关联 ID
    pub correlation_id: CorrelationId,
    /// 编排实例 ID
    pub orchestration_id: OrchestrationId,
    /// 工作流运行 ID
    pub workflow_run_id: WorkflowRunId,
}

impl OrchestrationIdentity {
    pub(crate) fn new(
        correlation_id: CorrelationId,
        orchestration_id: OrchestrationId,
        workflow_run_id: WorkflowRunId,
    ) -> Self {
        Self {
            correlation_id,
            orchestration_id,
            workflow_run_id,
        }
    }

    /// 创建新的唯一编排身份标识
    pub(crate) fn generate() -> Self {
        let correlation_id = CorrelationId(uuid::Uuid::new_v4().to_string());
        let orchestration_id = OrchestrationId(uuid::Uuid::new_v4().to_string());
        let workflow_run_id = WorkflowRunId(uuid::Uuid::new_v4().to_string());
        Self::new(correlation_id, orchestration_id, workflow_run_id)
    }
}

/// 尝试 ID - 唯一标识一次执行尝试
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct AttemptId(pub(crate) String);

impl AttemptId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AttemptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "attempt:{}", self.0)
    }
}

/// 提案 ID
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct ProposalId(pub(crate) String);

impl ProposalId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 授权 ID
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct AuthorizationId(pub(crate) String);

impl AuthorizationId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 授权决策 ID - 用户对 Proposal 的决定
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct AuthorizationDecisionId(pub(crate) String);

impl AuthorizationDecisionId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 结果用户决策 ID - 执行后对 Report/Review 的最终决定
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct ResultUserDecisionId(pub(crate) String);

impl ResultUserDecisionId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 工作项 ID - 一次 Run 内的具体工作项
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct WorkItemId(pub(crate) String);

impl WorkItemId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 节点 ID - 工作流内节点
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct NodeId(pub(crate) String);

impl NodeId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 派发 ID - 一次 Grant 派发
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct DispatchId(pub(crate) String);

impl DispatchId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 运行回执 ID - 独立 verifier 产出的 authoritative runtime receipt
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct RuntimeReceiptId(pub(crate) String);

impl RuntimeReceiptId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// 报告 ID - 执行/手动/离线 claim 的唯一标识
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct ReportId(pub(crate) String);

impl ReportId {
    pub(crate) fn new(id: String) -> Self {
        Self(id)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_id_display() {
        let id = CorrelationId::new("test-123".to_string());
        assert_eq!(id.to_string(), "corr:test-123");
    }

    #[test]
    fn orchestration_id_display() {
        let id = OrchestrationId::new("orch-456".to_string());
        assert_eq!(id.to_string(), "orch:orch-456");
    }

    #[test]
    fn workflow_run_id_display() {
        let id = WorkflowRunId::new("run-789".to_string());
        assert_eq!(id.to_string(), "run:run-789");
    }

    #[test]
    fn attempt_id_display() {
        let id = AttemptId::new("att-012".to_string());
        assert_eq!(id.to_string(), "attempt:att-012");
    }

    #[test]
    fn generate_unique_identity() {
        let id1 = OrchestrationIdentity::generate();
        let id2 = OrchestrationIdentity::generate();
        assert_ne!(id1.correlation_id, id2.correlation_id);
        assert_ne!(id1.orchestration_id, id2.orchestration_id);
        assert_ne!(id1.workflow_run_id, id2.workflow_run_id);
    }

    #[test]
    fn identity_preserves_values() {
        let corr = CorrelationId::new("c1".to_string());
        let orch = OrchestrationId::new("o1".to_string());
        let run = WorkflowRunId::new("r1".to_string());
        let identity = OrchestrationIdentity::new(corr.clone(), orch.clone(), run.clone());
        assert_eq!(identity.correlation_id, corr);
        assert_eq!(identity.orchestration_id, orch);
        assert_eq!(identity.workflow_run_id, run);
    }
}
