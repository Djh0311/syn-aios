// M5R06 official ProjectSummary projector. Consumers never read the project
// store, file root, or full snapshot.

use crate::m5_orchestration_store::M5OrchestrationStore;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum QueryError {
    ProjectNotFound(String),
    InsufficientPermission(String),
    SummaryStale(String),
    ConsumerExpired(String),
    StorageError(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::ProjectNotFound(id) => write!(f, "project not found: {id}"),
            QueryError::InsufficientPermission(r) => write!(f, "insufficient permission: {r}"),
            QueryError::SummaryStale(r) => write!(f, "summary stale: {r}"),
            QueryError::ConsumerExpired(r) => write!(f, "consumer expired: {r}"),
            QueryError::StorageError(r) => write!(f, "storage error: {r}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SourceRef {
    pub source_type: String,
    pub source_id: String,
    pub last_updated_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProjectSummary {
    pub project_id: String,
    pub orchestration_id: String,
    pub schema_version: String,
    pub version: u64,
    pub watermark_ms: i64,
    pub summary_hash: String,
    pub source_refs: Vec<SourceRef>,
    pub fact_count: u32,
    pub unverified_claim_count: u32,
    pub open_run_count: u32,
    pub rebuilt_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SummaryConsumer {
    pub role_session_id: String,
    pub role: String,
    pub scope_project_id: String,
    pub expires_at_ms: i64,
}

pub(crate) trait ProjectSummaryQueryPort {
    fn get_summary(
        &self,
        project_id: &str,
        consumer: &SummaryConsumer,
        now_ms: i64,
    ) -> Result<ProjectSummary, QueryError>;
}

pub(crate) fn ensure_summary_schema(store: &M5OrchestrationStore) -> Result<(), String> {
    store
        .connection()
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS m5_project_summaries (
                project_id TEXT PRIMARY KEY,
                orchestration_id TEXT NOT NULL,
                schema_version TEXT NOT NULL,
                version INTEGER NOT NULL,
                watermark_ms INTEGER NOT NULL,
                summary_hash TEXT NOT NULL,
                source_refs_json TEXT NOT NULL,
                fact_count INTEGER NOT NULL,
                unverified_claim_count INTEGER NOT NULL,
                open_run_count INTEGER NOT NULL,
                rebuilt_at_ms INTEGER NOT NULL
            );
            "#,
        )
        .map_err(|e| format!("summary_schema:{e}"))?;
    Ok(())
}

pub(crate) fn rebuild_project_summary(
    store: &M5OrchestrationStore,
    project_id: &str,
    now_ms: i64,
) -> Result<ProjectSummary, String> {
    ensure_summary_schema(store)?;
    let conn = store.connection();
    let fact_count: i64 = count_or_zero(
        conn,
        "SELECT COUNT(*) FROM m5_project_facts WHERE project_id=?1",
        project_id,
    );
    let unverified: i64 = count_or_zero(
        conn,
        "SELECT COUNT(*) FROM m5_claims WHERE project_id=?1 AND claim_status='RECORDED_UNVERIFIED'",
        project_id,
    );
    let open_runs: i64 = count_or_zero(
        conn,
        "SELECT COUNT(*) FROM m5_workflow_runs WHERE project_id=?1 AND status IN ('CREATED','ACTIVE')",
        project_id,
    );
    let orchestration_id: String = conn
        .query_row(
            "SELECT orchestration_id FROM m5_workflow_runs WHERE project_id=?1
             ORDER BY created_at_ms DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .or_else(|_| {
            conn.query_row(
                "SELECT orchestration_id FROM m5_plan_authorizations WHERE project_id=?1 LIMIT 1",
                [project_id],
                |row| row.get(0),
            )
        })
        .unwrap_or_else(|_| "orch:unknown".to_string());

    let mut refs = Vec::new();
    if fact_count > 0 {
        refs.push(SourceRef {
            source_type: "project_fact".into(),
            source_id: format!("facts:{project_id}"),
            last_updated_ms: now_ms,
        });
    }
    if unverified > 0 {
        refs.push(SourceRef {
            source_type: "unverified_claim".into(),
            source_id: format!("claims:{project_id}"),
            last_updated_ms: now_ms,
        });
    }
    if open_runs > 0 {
        refs.push(SourceRef {
            source_type: "workflow_run".into(),
            source_id: format!("runs:{project_id}"),
            last_updated_ms: now_ms,
        });
    }
    let previous = load_summary_row(store, project_id)?;
    let version = previous.map(|s| s.version + 1).unwrap_or(1);
    let summary_hash = sha_hex(&format!(
        "{project_id}:{orchestration_id}:{fact_count}:{unverified}:{open_runs}:{now_ms}"
    ));
    let summary = ProjectSummary {
        project_id: project_id.to_string(),
        orchestration_id,
        schema_version: "m5.project-summary.v1".into(),
        version,
        watermark_ms: now_ms,
        summary_hash,
        source_refs: refs.clone(),
        fact_count: fact_count as u32,
        unverified_claim_count: unverified as u32,
        open_run_count: open_runs as u32,
        rebuilt_at_ms: now_ms,
    };
    let refs_json = serde_json::to_string(&refs).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT OR REPLACE INTO m5_project_summaries (
            project_id, orchestration_id, schema_version, version, watermark_ms,
            summary_hash, source_refs_json, fact_count, unverified_claim_count,
            open_run_count, rebuilt_at_ms
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![
            summary.project_id,
            summary.orchestration_id,
            summary.schema_version,
            summary.version as i64,
            summary.watermark_ms,
            summary.summary_hash,
            refs_json,
            summary.fact_count,
            summary.unverified_claim_count,
            summary.open_run_count,
            summary.rebuilt_at_ms
        ],
    )
    .map_err(|e| format!("persist_summary:{e}"))?;
    Ok(summary)
}

pub(crate) struct PersistentProjectSummaryPort<'a> {
    store: &'a M5OrchestrationStore,
}

impl<'a> PersistentProjectSummaryPort<'a> {
    pub(crate) fn new(store: &'a M5OrchestrationStore) -> Self {
        Self { store }
    }
}

impl PersistentProjectSummaryPort<'_> {
    pub(crate) fn get_summary_unchecked(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSummary>, String> {
        load_summary_row(self.store, project_id)
    }
}

impl ProjectSummaryQueryPort for PersistentProjectSummaryPort<'_> {
    fn get_summary(
        &self,
        project_id: &str,
        consumer: &SummaryConsumer,
        now_ms: i64,
    ) -> Result<ProjectSummary, QueryError> {
        if now_ms >= consumer.expires_at_ms {
            return Err(QueryError::ConsumerExpired(
                consumer.role_session_id.clone(),
            ));
        }
        if consumer.scope_project_id != project_id {
            return Err(QueryError::InsufficientPermission(
                "cross_project_summary_denied".into(),
            ));
        }
        if !matches!(
            consumer.role.as_str(),
            "secretary" | "global_supervisor" | "project_supervisor"
        ) {
            return Err(QueryError::InsufficientPermission(consumer.role.clone()));
        }
        let summary = load_summary_row(self.store, project_id)
            .map_err(QueryError::StorageError)?
            .ok_or_else(|| QueryError::ProjectNotFound(project_id.to_string()))?;
        if now_ms > summary.watermark_ms + 60_000 {
            return Err(QueryError::SummaryStale(format!(
                "watermark={} now={now_ms}",
                summary.watermark_ms
            )));
        }
        if summary_contains_raw_text(&summary) {
            return Err(QueryError::StorageError(
                "summary_must_not_copy_source_text".into(),
            ));
        }
        Ok(summary)
    }
}

fn summary_contains_raw_text(summary: &ProjectSummary) -> bool {
    summary
        .source_refs
        .iter()
        .any(|r| r.source_id.contains('\n') || r.source_id.len() > 200)
}

fn load_summary_row(
    store: &M5OrchestrationStore,
    project_id: &str,
) -> Result<Option<ProjectSummary>, String> {
    ensure_summary_schema(store)?;
    store
        .connection()
        .query_row(
            "SELECT project_id, orchestration_id, schema_version, version, watermark_ms,
                    summary_hash, source_refs_json, fact_count, unverified_claim_count,
                    open_run_count, rebuilt_at_ms
             FROM m5_project_summaries WHERE project_id=?1",
            [project_id],
            |row| {
                let refs_json: String = row.get(6)?;
                Ok(ProjectSummary {
                    project_id: row.get(0)?,
                    orchestration_id: row.get(1)?,
                    schema_version: row.get(2)?,
                    version: row.get::<_, i64>(3)? as u64,
                    watermark_ms: row.get(4)?,
                    summary_hash: row.get(5)?,
                    source_refs: serde_json::from_str(&refs_json).unwrap_or_default(),
                    fact_count: row.get::<_, i64>(7)? as u32,
                    unverified_claim_count: row.get::<_, i64>(8)? as u32,
                    open_run_count: row.get::<_, i64>(9)? as u32,
                    rebuilt_at_ms: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load_summary:{e}"))
}

fn count_or_zero(conn: &rusqlite::Connection, sql: &str, project_id: &str) -> i64 {
    conn.query_row(sql, [project_id], |row| row.get(0))
        .unwrap_or(0)
}

fn sha_hex(input: &str) -> String {
    Sha256::digest(input.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m5_orchestration_service::{
        prepare_and_dispatch, AuthorizedExecutionRequest, ChainFault,
    };

    fn req() -> AuthorizedExecutionRequest {
        AuthorizedExecutionRequest {
            project_id: "proj-1".into(),
            proposal_id: "prop-1".into(),
            deciding_actor_id: "user-1".into(),
            worker_role_session_id: "role-sess-1".into(),
            principal_actor_id: "actor-1".into(),
            workflow_ref: "wf-1".into(),
            source_object_ref: "obj:1".into(),
            allowed_commands: vec!["echo".into()],
            cwd_ref: "/tmp/scratch".into(),
            write_root_refs: vec!["/tmp/scratch".into()],
            object_refs: vec!["obj:1".into()],
            scope_fingerprint: "scope-1".into(),
            policy_decision_ref: "pol-1".into(),
            now_ms: 1_000,
            ttl_ms: 60_000,
        }
    }

    fn consumer(project: &str, role: &str, exp: i64) -> SummaryConsumer {
        SummaryConsumer {
            role_session_id: format!("rs-{role}"),
            role: role.into(),
            scope_project_id: project.into(),
            expires_at_ms: exp,
        }
    }

    #[test]
    fn rebuild_is_deterministic_for_same_facts() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let a = rebuild_project_summary(&store, "proj-1", 2000).unwrap();
        let b = rebuild_project_summary(&store, "proj-1", 2000).unwrap();
        assert_eq!(a.open_run_count, b.open_run_count);
        assert_eq!(a.fact_count, b.fact_count);
        assert_eq!(a.schema_version, "m5.project-summary.v1");
        assert_eq!(b.version, a.version + 1);
    }

    #[test]
    fn query_rejects_cross_project_and_expired_and_stale() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        rebuild_project_summary(&store, "proj-1", 2000).unwrap();
        let port = PersistentProjectSummaryPort::new(&store);
        assert!(matches!(
            port.get_summary("proj-1", &consumer("proj-other", "secretary", 9000), 3000),
            Err(QueryError::InsufficientPermission(_))
        ));
        assert!(matches!(
            port.get_summary("proj-1", &consumer("proj-1", "secretary", 1000), 3000),
            Err(QueryError::ConsumerExpired(_))
        ));
        assert!(matches!(
            port.get_summary(
                "proj-1",
                &consumer("proj-1", "secretary", 9_000_000),
                80_000
            ),
            Err(QueryError::SummaryStale(_))
        ));
        let ok = port
            .get_summary("proj-1", &consumer("proj-1", "secretary", 9000), 3000)
            .unwrap();
        assert_eq!(ok.project_id, "proj-1");
        assert!(ok.source_refs.iter().all(|r| r.source_id.len() < 80));
    }

    #[test]
    fn rebuild_does_not_write_owner_tables() {
        let store = M5OrchestrationStore::open_in_memory().unwrap();
        prepare_and_dispatch(&store, req(), ChainFault::None).unwrap();
        let before: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_workflow_runs", [], |row| {
                row.get(0)
            })
            .unwrap();
        rebuild_project_summary(&store, "proj-1", 2000).unwrap();
        let after: i64 = store
            .connection()
            .query_row("SELECT COUNT(*) FROM m5_workflow_runs", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(before, after);
    }
}
