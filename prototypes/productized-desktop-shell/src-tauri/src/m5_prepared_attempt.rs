// SYN-PRJ-001 / M5R02: PreparedAttempt 状态机（M1 legal_states）
//
// Grant persist/readback 完成前不得进入 DISPATCHED。任意字符串 Grant
// 不能把 Attempt 变成可运行。

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::m5_orchestration_identity::{
    AttemptId, AuthorizationId, GrantId, NodeId, OrchestrationId, WorkItemId, WorkflowRunId,
};

/// M1 PreparedAttempt.state
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) enum AttemptState {
    PreparedNonRunnable,
    GrantPendingNonRunnable,
    GrantReadyNonRunnable,
    Dispatched,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    TimedOut,
    UnknownReadback,
}

impl AttemptState {
    pub(crate) fn as_m1_str(&self) -> &'static str {
        match self {
            AttemptState::PreparedNonRunnable => "PREPARED_NON_RUNNABLE",
            AttemptState::GrantPendingNonRunnable => "GRANT_PENDING_NON_RUNNABLE",
            AttemptState::GrantReadyNonRunnable => "GRANT_READY_NON_RUNNABLE",
            AttemptState::Dispatched => "DISPATCHED",
            AttemptState::Running => "RUNNING",
            AttemptState::Succeeded => "SUCCEEDED",
            AttemptState::Failed => "FAILED",
            AttemptState::Cancelled => "CANCELLED",
            AttemptState::TimedOut => "TIMED_OUT",
            AttemptState::UnknownReadback => "UNKNOWN_READBACK",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "PREPARED_NON_RUNNABLE" => Ok(AttemptState::PreparedNonRunnable),
            "GRANT_PENDING_NON_RUNNABLE" => Ok(AttemptState::GrantPendingNonRunnable),
            "GRANT_READY_NON_RUNNABLE" => Ok(AttemptState::GrantReadyNonRunnable),
            "DISPATCHED" => Ok(AttemptState::Dispatched),
            "RUNNING" => Ok(AttemptState::Running),
            "SUCCEEDED" => Ok(AttemptState::Succeeded),
            "FAILED" => Ok(AttemptState::Failed),
            "CANCELLED" => Ok(AttemptState::Cancelled),
            "TIMED_OUT" => Ok(AttemptState::TimedOut),
            "UNKNOWN_READBACK" => Ok(AttemptState::UnknownReadback),
            other => Err(format!("unknown_attempt_state:{other}")),
        }
    }

    pub(crate) fn is_terminal(&self) -> bool {
        matches!(
            self,
            AttemptState::Succeeded
                | AttemptState::Failed
                | AttemptState::Cancelled
                | AttemptState::TimedOut
                | AttemptState::UnknownReadback
        )
    }

    /// 只有 DISPATCHED/RUNNING 才允许进入执行。GRANT_READY 仍不可运行。
    pub(crate) fn is_runnable(&self) -> bool {
        matches!(self, AttemptState::Dispatched | AttemptState::Running)
    }
}

impl fmt::Display for AttemptState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_m1_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AttemptTransitionError {
    InvalidTransition {
        from: AttemptState,
        to: AttemptState,
    },
    MissingGrant,
    InvalidGrant(String),
    AlreadyCompleted,
}

impl fmt::Display for AttemptTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttemptTransitionError::InvalidTransition { from, to } => {
                write!(f, "invalid transition from {from} to {to}")
            }
            AttemptTransitionError::MissingGrant => write!(f, "missing grant"),
            AttemptTransitionError::InvalidGrant(reason) => write!(f, "invalid grant: {reason}"),
            AttemptTransitionError::AlreadyCompleted => write!(f, "attempt already completed"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PreparedAttempt {
    pub attempt_id: AttemptId,
    pub state: AttemptState,
    pub project_id: String,
    pub orchestration_id: OrchestrationId,
    pub workflow_run_id: WorkflowRunId,
    pub work_item_id: WorkItemId,
    pub node_id: NodeId,
    pub worker_role_session_id: String,
    pub authorization_id: AuthorizationId,
    pub authorization_revision: i64,
    pub grant_id: Option<GrantId>,
    pub revision: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl PreparedAttempt {
    pub(crate) fn new(
        attempt_id: AttemptId,
        project_id: String,
        orchestration_id: OrchestrationId,
        workflow_run_id: WorkflowRunId,
        work_item_id: WorkItemId,
        node_id: NodeId,
        worker_role_session_id: String,
        authorization_id: AuthorizationId,
        authorization_revision: i64,
        created_at_ms: i64,
    ) -> Self {
        Self {
            attempt_id,
            state: AttemptState::PreparedNonRunnable,
            project_id,
            orchestration_id,
            workflow_run_id,
            work_item_id,
            node_id,
            worker_role_session_id,
            authorization_id,
            authorization_revision,
            grant_id: None,
            revision: 1,
            created_at_ms,
            updated_at_ms: created_at_ms,
        }
    }

    pub(crate) fn state(&self) -> &AttemptState {
        &self.state
    }

    pub(crate) fn is_runnable(&self) -> bool {
        self.state.is_runnable()
    }

    pub(crate) fn is_completed(&self) -> bool {
        self.state.is_terminal()
    }

    fn bump(&mut self, now_ms: i64) {
        self.revision += 1;
        self.updated_at_ms = now_ms;
    }

    /// PREPARED_NON_RUNNABLE → GRANT_PENDING_NON_RUNNABLE
    pub(crate) fn begin_grant_binding(
        &mut self,
        now_ms: i64,
    ) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::PreparedNonRunnable {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::GrantPendingNonRunnable,
            });
        }
        self.state = AttemptState::GrantPendingNonRunnable;
        self.bump(now_ms);
        Ok(())
    }

    /// 记录已 mint 的 GrantId，状态仍为 GRANT_PENDING_NON_RUNNABLE。
    pub(crate) fn attach_minted_grant(
        &mut self,
        grant_id: GrantId,
        now_ms: i64,
    ) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::GrantPendingNonRunnable {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::GrantPendingNonRunnable,
            });
        }
        if grant_id.as_str().trim().is_empty() {
            return Err(AttemptTransitionError::InvalidGrant(
                "empty_grant_id".to_string(),
            ));
        }
        self.grant_id = Some(grant_id);
        self.bump(now_ms);
        Ok(())
    }

    /// ACTIVE Grant readback 后：GRANT_PENDING_NON_RUNNABLE → GRANT_READY_NON_RUNNABLE
    pub(crate) fn confirm_grant_ready(
        &mut self,
        expected_grant_id: &GrantId,
        now_ms: i64,
    ) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::GrantPendingNonRunnable {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::GrantReadyNonRunnable,
            });
        }
        match &self.grant_id {
            Some(id) if id == expected_grant_id => {}
            Some(_) => {
                return Err(AttemptTransitionError::InvalidGrant(
                    "grant_id_mismatch".to_string(),
                ))
            }
            None => return Err(AttemptTransitionError::MissingGrant),
        }
        self.state = AttemptState::GrantReadyNonRunnable;
        self.bump(now_ms);
        Ok(())
    }

    /// persist/readback 失败：回到可再 mint 的 PREPARED_NON_RUNNABLE
    pub(crate) fn recover_grant_failure(
        &mut self,
        now_ms: i64,
    ) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::GrantPendingNonRunnable
            && self.state != AttemptState::GrantReadyNonRunnable
        {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::PreparedNonRunnable,
            });
        }
        self.grant_id = None;
        self.state = AttemptState::PreparedNonRunnable;
        self.bump(now_ms);
        Ok(())
    }

    /// GRANT_READY_NON_RUNNABLE → DISPATCHED（此时才可运行）
    pub(crate) fn mark_dispatched(&mut self, now_ms: i64) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::GrantReadyNonRunnable {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::Dispatched,
            });
        }
        if self.grant_id.is_none() {
            return Err(AttemptTransitionError::MissingGrant);
        }
        self.state = AttemptState::Dispatched;
        self.bump(now_ms);
        Ok(())
    }

    pub(crate) fn start_execution(&mut self, now_ms: i64) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::Dispatched {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::Running,
            });
        }
        if self.grant_id.is_none() {
            return Err(AttemptTransitionError::MissingGrant);
        }
        self.state = AttemptState::Running;
        self.bump(now_ms);
        Ok(())
    }

    pub(crate) fn succeed(&mut self, now_ms: i64) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::Running {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::Succeeded,
            });
        }
        self.state = AttemptState::Succeeded;
        self.bump(now_ms);
        Ok(())
    }

    pub(crate) fn fail(&mut self, now_ms: i64) -> Result<(), AttemptTransitionError> {
        if self.state != AttemptState::Running {
            return Err(AttemptTransitionError::InvalidTransition {
                from: self.state.clone(),
                to: AttemptState::Failed,
            });
        }
        self.state = AttemptState::Failed;
        self.bump(now_ms);
        Ok(())
    }

    pub(crate) fn cancel(&mut self, now_ms: i64) -> Result<(), AttemptTransitionError> {
        if self.is_completed() {
            return Err(AttemptTransitionError::AlreadyCompleted);
        }
        self.state = AttemptState::Cancelled;
        self.bump(now_ms);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attempt() -> PreparedAttempt {
        PreparedAttempt::new(
            AttemptId::new("att-1".into()),
            "proj-1".into(),
            OrchestrationId::new("orch-1".into()),
            WorkflowRunId::new("run-1".into()),
            WorkItemId::new("wi-1".into()),
            NodeId::new("node-1".into()),
            "role-sess-1".into(),
            AuthorizationId::new("auth-1".into()),
            1,
            1000,
        )
    }

    #[test]
    fn new_attempt_is_not_runnable() {
        let a = attempt();
        assert_eq!(a.state(), &AttemptState::PreparedNonRunnable);
        assert!(!a.is_runnable());
        assert!(a.grant_id.is_none());
    }

    #[test]
    fn minted_grant_does_not_make_runnable() {
        let mut a = attempt();
        a.begin_grant_binding(1100).unwrap();
        a.attach_minted_grant(GrantId::new("g-1".into()), 1200)
            .unwrap();
        assert_eq!(a.state(), &AttemptState::GrantPendingNonRunnable);
        assert!(!a.is_runnable());
    }

    #[test]
    fn grant_ready_is_still_not_runnable() {
        let mut a = attempt();
        let gid = GrantId::new("g-1".into());
        a.begin_grant_binding(1100).unwrap();
        a.attach_minted_grant(gid.clone(), 1200).unwrap();
        a.confirm_grant_ready(&gid, 1300).unwrap();
        assert_eq!(a.state(), &AttemptState::GrantReadyNonRunnable);
        assert!(!a.is_runnable());
    }

    #[test]
    fn only_dispatch_makes_runnable() {
        let mut a = attempt();
        let gid = GrantId::new("g-1".into());
        a.begin_grant_binding(1100).unwrap();
        a.attach_minted_grant(gid.clone(), 1200).unwrap();
        a.confirm_grant_ready(&gid, 1300).unwrap();
        a.mark_dispatched(1400).unwrap();
        assert!(a.is_runnable());
        assert_eq!(a.state(), &AttemptState::Dispatched);
    }

    #[test]
    fn empty_grant_id_rejected() {
        let mut a = attempt();
        a.begin_grant_binding(1100).unwrap();
        let err = a
            .attach_minted_grant(GrantId::new("".into()), 1200)
            .unwrap_err();
        assert!(matches!(err, AttemptTransitionError::InvalidGrant(_)));
        assert!(!a.is_runnable());
    }

    #[test]
    fn grant_mismatch_rejected() {
        let mut a = attempt();
        a.begin_grant_binding(1100).unwrap();
        a.attach_minted_grant(GrantId::new("g-1".into()), 1200)
            .unwrap();
        let err = a
            .confirm_grant_ready(&GrantId::new("g-other".into()), 1300)
            .unwrap_err();
        assert!(matches!(err, AttemptTransitionError::InvalidGrant(_)));
        assert!(!a.is_runnable());
    }

    #[test]
    fn recover_clears_grant_and_stays_non_runnable() {
        let mut a = attempt();
        a.begin_grant_binding(1100).unwrap();
        a.attach_minted_grant(GrantId::new("g-1".into()), 1200)
            .unwrap();
        a.recover_grant_failure(1300).unwrap();
        assert_eq!(a.state(), &AttemptState::PreparedNonRunnable);
        assert!(a.grant_id.is_none());
        assert!(!a.is_runnable());
    }

    #[test]
    fn cannot_start_before_dispatch() {
        let mut a = attempt();
        let err = a.start_execution(2000).unwrap_err();
        assert!(matches!(
            err,
            AttemptTransitionError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn cancel_from_prepared() {
        let mut a = attempt();
        a.cancel(2000).unwrap();
        assert_eq!(a.state(), &AttemptState::Cancelled);
        assert!(!a.is_runnable());
    }

    #[test]
    fn state_displays_m1_names() {
        assert_eq!(
            AttemptState::PreparedNonRunnable.to_string(),
            "PREPARED_NON_RUNNABLE"
        );
        assert_eq!(
            AttemptState::GrantReadyNonRunnable.to_string(),
            "GRANT_READY_NON_RUNNABLE"
        );
    }
}
