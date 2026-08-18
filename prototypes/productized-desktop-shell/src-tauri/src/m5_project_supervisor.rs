// M5R04: persistent Project Supervisor. Identity comes from an M3 RoleSession
// id; this module never invents a parallel session truth source and never
// starts dispatch without the M5R02 grant chain.

use crate::m5_orchestration_service::{
    prepare_and_dispatch, AuthorizedExecutionRequest, AuthorizedExecutionResult, ChainFault,
};
use crate::m5_orchestration_store::M5OrchestrationStore;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SupervisorSessionRef {
    pub role_session_id: String,
    pub project_id: String,
    pub actor_id: String,
    pub role: String,
    pub status: String,
}

pub(crate) trait ProjectSupervisorRoleSessionPort {
    fn load(&self, role_session_id: &str) -> Result<SupervisorSessionRef, String>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupervisorBinding {
    pub binding_id: String,
    pub project_id: String,
    pub role_session_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SupervisorAction {
    Chat { text: String },
    Read { query: String },
    SubmitProposal { goal: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupervisorTurn {
    pub kind: String,
    pub created_proposal: bool,
    pub created_grant: bool,
    pub spawned: bool,
    pub text: String,
}

pub(crate) fn ensure_supervisor_schema(store: &M5OrchestrationStore) -> Result<(), String> {
    store
        .connection()
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS m5_supervisor_bindings (
                binding_id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                role_session_id TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                UNIQUE(project_id, role_session_id)
            );
            CREATE TABLE IF NOT EXISTS m5_supervisor_turns (
                turn_id TEXT PRIMARY KEY,
                binding_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                text TEXT NOT NULL,
                created_proposal INTEGER NOT NULL,
                created_grant INTEGER NOT NULL,
                spawned INTEGER NOT NULL,
                created_at_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS m5_supervisor_proposals (
                proposal_id TEXT PRIMARY KEY,
                binding_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                goal TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at_ms INTEGER NOT NULL,
                authorized_action TEXT NOT NULL DEFAULT 'none'
            );
            "#,
        )
        .map_err(|e| format!("supervisor_schema:{e}"))?;
    let _ = store.connection().execute(
        "ALTER TABLE m5_supervisor_proposals ADD COLUMN authorized_action TEXT NOT NULL DEFAULT 'none'",
        [],
    );
    Ok(())
}

pub(crate) fn open_or_resume_supervisor(
    store: &M5OrchestrationStore,
    sessions: &dyn ProjectSupervisorRoleSessionPort,
    role_session_id: &str,
    expected_project_id: &str,
    now_ms: i64,
) -> Result<SupervisorBinding, String> {
    ensure_supervisor_schema(store)?;
    if role_session_id.trim().is_empty() || expected_project_id.trim().is_empty() {
        return Err("empty_role_session_identity_rejected".to_string());
    }
    let session = sessions.load(role_session_id)?;
    if session.role != "project_supervisor" {
        return Err("role_is_not_project_supervisor".to_string());
    }
    if session.project_id != expected_project_id {
        return Err("role_session_project_mismatch".to_string());
    }
    if let Some(existing) = load_binding(store, expected_project_id, role_session_id)? {
        if existing.actor_id != session.actor_id {
            return Err("role_session_actor_mismatch".to_string());
        }
        return Ok(existing);
    }
    let binding = SupervisorBinding {
        binding_id: format!("sup-{}", uuid::Uuid::new_v4()),
        project_id: expected_project_id.to_string(),
        role_session_id: role_session_id.to_string(),
        actor_id: session.actor_id,
    };
    store
        .connection()
        .execute(
            "INSERT INTO m5_supervisor_bindings (
                binding_id, project_id, role_session_id, actor_id, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5)",
            params![
                binding.binding_id,
                binding.project_id,
                binding.role_session_id,
                binding.actor_id,
                now_ms
            ],
        )
        .map_err(|e| format!("insert_supervisor_binding:{e}"))?;
    Ok(binding)
}

pub(crate) fn require_complete_binding(binding: &SupervisorBinding) -> Result<(), String> {
    if binding.binding_id.trim().is_empty()
        || binding.project_id.trim().is_empty()
        || binding.role_session_id.trim().is_empty()
        || binding.actor_id.trim().is_empty()
    {
        return Err("empty_binding_rejected".to_string());
    }
    Ok(())
}

pub(crate) fn load_binding_by_id(
    store: &M5OrchestrationStore,
    binding_id: &str,
    project_id: &str,
) -> Result<SupervisorBinding, String> {
    ensure_supervisor_schema(store)?;
    let binding = store
        .connection()
        .query_row(
            "SELECT binding_id, project_id, role_session_id, actor_id
             FROM m5_supervisor_bindings
             WHERE binding_id=?1 AND project_id=?2",
            params![binding_id, project_id],
            |row| {
                Ok(SupervisorBinding {
                    binding_id: row.get(0)?,
                    project_id: row.get(1)?,
                    role_session_id: row.get(2)?,
                    actor_id: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_binding_by_id:{e}"))?
        .ok_or_else(|| "supervisor_binding_not_found".to_string())?;
    require_complete_binding(&binding)?;
    Ok(binding)
}

pub(crate) fn verify_binding_against_session(
    binding: &SupervisorBinding,
    session: &SupervisorSessionRef,
) -> Result<(), String> {
    require_complete_binding(binding)?;
    if session.role_session_id != binding.role_session_id
        || session.project_id != binding.project_id
        || session.actor_id != binding.actor_id
        || session.role != "project_supervisor"
    {
        return Err("binding_role_session_exact_join_failed".to_string());
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupervisorProposal {
    pub proposal_id: String,
    pub binding_id: String,
    pub project_id: String,
    pub goal: String,
    pub status: String,
    pub authorized_action: String,
}

pub(crate) fn load_supervisor_proposal(
    store: &M5OrchestrationStore,
    proposal_id: &str,
    project_id: &str,
    binding_id: &str,
) -> Result<SupervisorProposal, String> {
    ensure_supervisor_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT proposal_id, binding_id, project_id, goal, status,
                    COALESCE(authorized_action, 'none')
             FROM m5_supervisor_proposals
             WHERE proposal_id=?1 AND project_id=?2 AND binding_id=?3",
            params![proposal_id, project_id, binding_id],
            |row| {
                Ok(SupervisorProposal {
                    proposal_id: row.get(0)?,
                    binding_id: row.get(1)?,
                    project_id: row.get(2)?,
                    goal: row.get(3)?,
                    status: row.get(4)?,
                    authorized_action: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_proposal:{e}"))?
        .ok_or_else(|| "supervisor_proposal_not_found".to_string())
}

pub(crate) fn handle_supervisor_action(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    action: SupervisorAction,
    now_ms: i64,
) -> Result<SupervisorTurn, String> {
    ensure_supervisor_schema(store)?;
    require_complete_binding(binding)?;
    let _ = load_binding_by_id(store, &binding.binding_id, &binding.project_id)?;
    let turn = match action {
        SupervisorAction::Chat { text } | SupervisorAction::Read { query: text } => {
            SupervisorTurn {
                kind: "read_or_chat".into(),
                created_proposal: false,
                created_grant: false,
                spawned: false,
                text,
            }
        }
        SupervisorAction::SubmitProposal { goal } => {
            let proposal_id = format!("prop-{}", uuid::Uuid::new_v4());
            let authorized_action =
                crate::m5_m3_identity::classify_whitelisted_action(&goal).to_string();
            store
                .connection()
                .execute(
                    "INSERT INTO m5_supervisor_proposals (
                        proposal_id, binding_id, project_id, goal, status, created_at_ms,
                        authorized_action
                    ) VALUES (?1,?2,?3,?4,'DRAFT',?5,?6)",
                    params![
                        proposal_id,
                        binding.binding_id,
                        binding.project_id,
                        goal,
                        now_ms,
                        authorized_action
                    ],
                )
                .map_err(|e| format!("insert_proposal:{e}"))?;
            SupervisorTurn {
                kind: "proposal".into(),
                created_proposal: true,
                created_grant: false,
                spawned: false,
                text: proposal_id,
            }
        }
    };
    persist_turn(store, binding, &turn, now_ms)?;
    Ok(turn)
}

/// Formal user AuthorizationDecision entry. Isolated scenes and the UI must
/// call this; they must not call prepare_and_dispatch themselves.
pub(crate) fn record_user_authorization_decision(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    proposal_id: &str,
    decision: &str,
    request: Option<AuthorizedExecutionRequest>,
) -> Result<Option<AuthorizedExecutionResult>, String> {
    require_complete_binding(binding)?;
    let live = load_binding_by_id(store, &binding.binding_id, &binding.project_id)?;
    match decision {
        "REJECTED" => {
            store
                .connection()
                .execute(
                    "UPDATE m5_supervisor_proposals SET status='REJECTED'
                     WHERE proposal_id=?1 AND project_id=?2 AND binding_id=?3 AND status='DRAFT'",
                    params![proposal_id, live.project_id, live.binding_id],
                )
                .map_err(|e| format!("reject_proposal:{e}"))?;
            Ok(None)
        }
        "APPROVED" => {
            let proposal =
                load_supervisor_proposal(store, proposal_id, &live.project_id, &live.binding_id)?;
            if proposal.status != "DRAFT" {
                return Err("supervisor_proposal_not_draft".to_string());
            }
            if proposal.authorized_action != crate::m5_m3_identity::WHITELISTED_COMMAND {
                return Err("proposal_has_no_authorized_action".to_string());
            }
            let request = match request {
                Some(request) => {
                    if request.project_id != live.project_id
                        || request.principal_actor_id != live.actor_id
                    {
                        return Err("authorization_actor_or_project_mismatch".to_string());
                    }
                    if request.proposal_id != proposal.proposal_id {
                        return Err("authorization_proposal_mismatch".to_string());
                    }
                    if request.allowed_commands != vec![proposal.authorized_action.clone()] {
                        return Err("renderer_grant_scope_rejected".to_string());
                    }
                    if request.policy_decision_ref
                        != crate::m5_m3_identity::policy_decision_ref_for_action(
                            &proposal.authorized_action,
                        )
                    {
                        return Err("renderer_policy_rejected".to_string());
                    }
                    request
                }
                None => return Err("worker_view_required".to_string()),
            };
            approve_supervisor_proposal(store, &live, proposal_id)?;
            Ok(Some(prepare_and_dispatch(
                store,
                request,
                ChainFault::None,
            )?))
        }
        other => Err(format!("illegal_authorization_decision:{other}")),
    }
}

/// Kept as an internal alias after an AuthorizationDecision has already been
/// recorded. New callers must use record_user_authorization_decision.
pub(crate) fn authorize_and_dispatch_from_supervisor(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    proposal_id: &str,
    request: AuthorizedExecutionRequest,
) -> Result<AuthorizedExecutionResult, String> {
    record_user_authorization_decision(store, binding, proposal_id, "APPROVED", Some(request))?
        .ok_or_else(|| "authorization_did_not_dispatch".to_string())
}

pub(crate) fn approve_supervisor_proposal(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    proposal_id: &str,
) -> Result<(), String> {
    let changed = store
        .connection()
        .execute(
            "UPDATE m5_supervisor_proposals SET status='APPROVED'
             WHERE proposal_id=?1 AND project_id=?2 AND binding_id=?3 AND status='DRAFT'",
            params![proposal_id, binding.project_id, binding.binding_id],
        )
        .map_err(|e| format!("approve_proposal:{e}"))?;
    if changed != 1 {
        return Err("supervisor_proposal_not_draft".to_string());
    }
    Ok(())
}

fn persist_turn(
    store: &M5OrchestrationStore,
    binding: &SupervisorBinding,
    turn: &SupervisorTurn,
    now_ms: i64,
) -> Result<(), String> {
    store
        .connection()
        .execute(
            "INSERT INTO m5_supervisor_turns (
                turn_id, binding_id, project_id, kind, text, created_proposal,
                created_grant, spawned, created_at_ms
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                format!("turn-{}", uuid::Uuid::new_v4()),
                binding.binding_id,
                binding.project_id,
                turn.kind,
                turn.text,
                turn.created_proposal as i64,
                turn.created_grant as i64,
                turn.spawned as i64,
                now_ms
            ],
        )
        .map_err(|e| format!("insert_turn:{e}"))?;
    Ok(())
}

pub(crate) fn count_grants_for_project(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Result<i64, String> {
    store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM m5_execution_grants WHERE project_id=?1",
            [project_id],
            |row| row.get(0),
        )
        .or_else(|_| Ok(0))
}

pub(crate) fn latest_grant_dispatch(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Result<Option<(String, String)>, String> {
    store
        .connection()
        .query_row(
            "SELECT grant_id, dispatch_id FROM m5_dispatches
             WHERE project_id=?1 ORDER BY created_at_ms DESC LIMIT 1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|e| format!("latest_grant_dispatch:{e}"))
}

fn load_binding(
    store: &M5OrchestrationStore,
    project_id: &str,
    role_session_id: &str,
) -> Result<Option<SupervisorBinding>, String> {
    store
        .connection()
        .query_row(
            "SELECT binding_id, project_id, role_session_id, actor_id
             FROM m5_supervisor_bindings
             WHERE project_id=?1 AND role_session_id=?2",
            params![project_id, role_session_id],
            |row| {
                Ok(SupervisorBinding {
                    binding_id: row.get(0)?,
                    project_id: row.get(1)?,
                    role_session_id: row.get(2)?,
                    actor_id: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_binding:{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MapSessions(HashMap<String, SupervisorSessionRef>);

    impl ProjectSupervisorRoleSessionPort for MapSessions {
        fn load(&self, role_session_id: &str) -> Result<SupervisorSessionRef, String> {
            self.0
                .get(role_session_id)
                .cloned()
                .ok_or_else(|| "m3_role_session_missing".to_string())
        }
    }

    fn session(id: &str, project: &str, actor: &str) -> SupervisorSessionRef {
        SupervisorSessionRef {
            role_session_id: id.into(),
            project_id: project.into(),
            actor_id: actor.into(),
            role: "project_supervisor".into(),
            status: "ACTIVE".into(),
        }
    }

    #[test]
    fn resume_returns_same_binding_after_reopen() {
        let dir = std::env::temp_dir().join(format!("m5r04-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sup.sqlite");
        let sessions = MapSessions(HashMap::from([(
            "rs-1".into(),
            session("rs-1", "proj-a", "actor-a"),
        )]));
        let first = {
            let store = M5OrchestrationStore::open(&path).unwrap();
            open_or_resume_supervisor(&store, &sessions, "rs-1", "proj-a", 1000).unwrap()
        };
        let store = M5OrchestrationStore::open(&path).unwrap();
        let second = open_or_resume_supervisor(&store, &sessions, "rs-1", "proj-a", 2000).unwrap();
        assert_eq!(first.binding_id, second.binding_id);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_projects_do_not_share_bindings() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let sessions = MapSessions(HashMap::from([
            ("rs-a".into(), session("rs-a", "proj-a", "actor-a")),
            ("rs-b".into(), session("rs-b", "proj-b", "actor-b")),
        ]));
        let a = open_or_resume_supervisor(&store, &sessions, "rs-a", "proj-a", 1000).unwrap();
        let b = open_or_resume_supervisor(&store, &sessions, "rs-b", "proj-b", 1000).unwrap();
        assert_ne!(a.binding_id, b.binding_id);
        let err = open_or_resume_supervisor(&store, &sessions, "rs-a", "proj-b", 1000).unwrap_err();
        assert_eq!(err, "role_session_project_mismatch");
    }

    #[test]
    fn chat_and_read_create_no_proposal_grant_or_spawn() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let sessions = MapSessions(HashMap::from([(
            "rs-1".into(),
            session("rs-1", "proj-a", "actor-a"),
        )]));
        let binding = open_or_resume_supervisor(&store, &sessions, "rs-1", "proj-a", 1000).unwrap();
        let chat = handle_supervisor_action(
            &store,
            &binding,
            SupervisorAction::Chat {
                text: "what's open?".into(),
            },
            1100,
        )
        .unwrap();
        let read = handle_supervisor_action(
            &store,
            &binding,
            SupervisorAction::Read {
                query: "facts".into(),
            },
            1200,
        )
        .unwrap();
        assert!(!chat.created_proposal && !chat.created_grant && !chat.spawned);
        assert!(!read.created_proposal && !read.created_grant && !read.spawned);
        let grants: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_execution_grants", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        assert_eq!(grants, 0);
    }

    #[test]
    fn cannot_dispatch_without_approved_proposal() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let sessions = MapSessions(HashMap::from([(
            "rs-1".into(),
            session("rs-1", "proj-a", "actor-a"),
        )]));
        let binding = open_or_resume_supervisor(&store, &sessions, "rs-1", "proj-a", 1000).unwrap();
        let turn = handle_supervisor_action(
            &store,
            &binding,
            SupervisorAction::SubmitProposal {
                goal: "echo hello".into(),
            },
            1300,
        )
        .unwrap();
        assert!(turn.created_proposal);
        assert!(!turn.created_grant);
        let rejected =
            record_user_authorization_decision(&store, &binding, &turn.text, "REJECTED", None)
                .unwrap();
        assert!(rejected.is_none());
        let grants: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_execution_grants", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        assert_eq!(grants, 0);
        let err = authorize_and_dispatch_from_supervisor(
            &store,
            &binding,
            &turn.text,
            AuthorizedExecutionRequest {
                project_id: "proj-a".into(),
                proposal_id: turn.text.clone(),
                deciding_actor_id: "actor-a".into(),
                worker_role_session_id: "worker-1".into(),
                principal_actor_id: "actor-a".into(),
                workflow_ref: "wf".into(),
                source_object_ref: "obj:1".into(),
                allowed_commands: vec!["echo".into()],
                cwd_ref: "/tmp/scratch".into(),
                write_root_refs: vec!["/tmp/scratch".into()],
                object_refs: vec!["obj:1".into()],
                scope_fingerprint: "scope".into(),
                policy_decision_ref: "pol".into(),
                now_ms: 1400,
                ttl_ms: 60_000,
            },
        )
        .unwrap_err();
        assert_eq!(err, "supervisor_proposal_not_draft");
    }

    #[test]
    fn fresh_store_schema_does_not_create_m5_role_sessions() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        ensure_supervisor_schema(&store).unwrap();
        let created: i64 = store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='m5_role_sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(created, 0);
        let source = include_str!("m5_project_supervisor.rs");
        assert!(!source.contains(&format!(
            "CREATE TABLE IF NOT EXISTS m5_{}",
            "role_sessions"
        )));
    }

    #[test]
    fn leftover_m5_role_sessions_table_is_not_read_or_written() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        let leftover_table = format!("m5_{}", "role_sessions");
        store
            .connection()
            .execute_batch(&format!(
                "CREATE TABLE {leftover_table} (
                    role_session_id TEXT PRIMARY KEY,
                    actor_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL
                );
                INSERT INTO {leftover_table}
                    (role_session_id, actor_id, role, project_id, status, created_at_ms)
                VALUES ('legacy-rs', 'legacy-actor', 'project_supervisor', 'proj-a', 'ACTIVE', 1);"
            ))
            .unwrap();
        let sessions = MapSessions(HashMap::from([(
            "rs-1".into(),
            session("rs-1", "proj-a", "actor-a"),
        )]));
        let binding = open_or_resume_supervisor(&store, &sessions, "rs-1", "proj-a", 1000).unwrap();
        handle_supervisor_action(
            &store,
            &binding,
            SupervisorAction::Chat {
                text: "ignore leftover table".into(),
            },
            1100,
        )
        .unwrap();
        let leftover: (String, i64) = store
            .connection()
            .query_row(
                &format!("SELECT actor_id, COUNT(*) FROM {leftover_table}"),
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(leftover, ("legacy-actor".into(), 1));
        assert_eq!(binding.role_session_id, "rs-1");
        assert_eq!(binding.actor_id, "actor-a");
    }
}
