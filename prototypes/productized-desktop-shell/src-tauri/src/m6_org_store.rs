//! M6-owned advisory, decision-request, and application-projection store.

use crate::m6_org_dto::{
    M6OrgAdvisoryApplicationProjection, M6OrgApplicationOutcome, M6OrgCrossProjectAdvisory,
    M6OrgDecisionRequest, M6OrgPerProjectApplicationObservation,
};
use crate::m6_org_schema::ensure_m6_org_schema;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) const M6_ORG_STORE_RELATIVE_PATH: &str = "m6/organization.sqlite";

pub(crate) struct M6OrgStore {
    connection: Connection,
}

impl M6OrgStore {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "m6_org_store_parent_missing".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("m6_org_store_parent_create:{error}"))?;
        let connection =
            Connection::open(path).map_err(|error| format!("m6_org_store_open:{error}"))?;
        ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    #[cfg(test)]
    pub(crate) fn open_in_memory() -> Result<Self, String> {
        let connection =
            Connection::open_in_memory().map_err(|error| format!("m6_org_store_mem:{error}"))?;
        ensure_m6_org_schema(&connection)?;
        Ok(Self { connection })
    }

    pub(crate) fn path_from_m5_store(m5_store_path: &Path) -> Result<PathBuf, String> {
        let m5_dir = m5_store_path
            .parent()
            .ok_or_else(|| "m6_org_m5_store_parent_missing".to_string())?;
        let app_data_root = m5_dir
            .parent()
            .ok_or_else(|| "m6_org_app_data_root_missing".to_string())?;
        Ok(app_data_root.join(M6_ORG_STORE_RELATIVE_PATH))
    }

    pub(crate) fn load_advisory_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<M6OrgCrossProjectAdvisory>, String> {
        self.load_payload(
            "SELECT payload_json FROM m6_cross_project_advisories WHERE idempotency_key=?1",
            idempotency_key,
            "m6_org_advisory_idempotency_load",
        )
    }

    pub(crate) fn load_advisory(
        &self,
        advisory_id: &str,
    ) -> Result<Option<M6OrgCrossProjectAdvisory>, String> {
        self.load_payload(
            "SELECT payload_json FROM m6_cross_project_advisories WHERE advisory_id=?1",
            advisory_id,
            "m6_org_advisory_load",
        )
    }

    pub(crate) fn record_advisory(
        &mut self,
        advisory: &M6OrgCrossProjectAdvisory,
    ) -> Result<M6OrgCrossProjectAdvisory, String> {
        if let Some(existing) = self.load_advisory_by_idempotency(&advisory.idempotency_key)? {
            if existing.request_hash != advisory.request_hash {
                return Err("m6_org_advisory_idempotency_collision".to_string());
            }
            return Ok(existing);
        }
        let payload = serde_json::to_string(advisory)
            .map_err(|error| format!("m6_org_advisory_serialize:{error}"))?;
        let audit_payload = serde_json::to_string(&json!({
            "advisory_id": advisory.advisory_id,
            "global_role_session_id": advisory.global_role_session_id,
            "consumed_summary_count": advisory.consumed_summaries.len(),
            "finding_count": advisory.findings.len(),
            "project_writeback": false
        }))
        .map_err(|error| format!("m6_org_advisory_audit_serialize:{error}"))?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_advisory_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_cross_project_advisories (
                    advisory_id, idempotency_key, request_hash, lifecycle_status,
                    freshness_state, revision, payload_json, created_at_ms, updated_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    advisory.advisory_id,
                    advisory.idempotency_key,
                    advisory.request_hash,
                    advisory.lifecycle_status,
                    advisory.freshness_state.as_str(),
                    advisory.revision as i64,
                    payload,
                    advisory.created_at_ms,
                    advisory.created_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_advisory_insert:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_org_audit_events (
                    event_id, event_type, target_ref, payload_json, created_at_ms
                 ) VALUES (?1,'CrossProjectAdvisoryRecorded',?2,?3,?4)",
                params![
                    format!("m6-audit:advisory:{}", advisory.advisory_id),
                    advisory.advisory_id,
                    audit_payload,
                    advisory.created_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_advisory_audit_insert:{error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_advisory_commit:{error}"))?;
        Ok(advisory.clone())
    }

    pub(crate) fn mark_issued_advisories_stale_for_source_changes(
        &mut self,
        current: &[crate::m6_org_dto::M6OrgConsumedProjectSummaryRef],
        now_ms: i64,
    ) -> Result<Vec<String>, String> {
        let current_by_project = current
            .iter()
            .map(|summary| (summary.project_id.as_str(), summary))
            .collect::<BTreeMap<_, _>>();
        let candidates = {
            let mut statement = self
                .connection
                .prepare(
                    "SELECT payload_json FROM m6_cross_project_advisories
                     WHERE lifecycle_status='ISSUED' ORDER BY advisory_id",
                )
                .map_err(|error| format!("m6_org_stale_prepare:{error}"))?;
            let rows = statement
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|error| format!("m6_org_stale_query:{error}"))?;
            let mut candidates = Vec::new();
            for row in rows {
                let payload = row.map_err(|error| format!("m6_org_stale_row:{error}"))?;
                candidates.push(
                    serde_json::from_str::<M6OrgCrossProjectAdvisory>(&payload)
                        .map_err(|error| format!("m6_org_stale_decode:{error}"))?,
                );
            }
            candidates
        };
        let mut stale_ids = Vec::new();
        for mut advisory in candidates {
            let changed = advisory.consumed_summaries.iter().any(|recorded| {
                current_by_project
                    .get(recorded.project_id.as_str())
                    .is_some_and(|latest| {
                        recorded.version != latest.version
                            || recorded.source_watermark != latest.source_watermark
                            || recorded.summary_hash != latest.summary_hash
                    })
            });
            if !changed {
                continue;
            }
            advisory.lifecycle_status = "STALE".to_string();
            advisory.freshness_state = crate::m6_org_dto::M6OrgFreshnessState::Stale;
            advisory.revision = advisory
                .revision
                .checked_add(1)
                .ok_or_else(|| "m6_org_advisory_revision_overflow".to_string())?;
            let payload = serde_json::to_string(&advisory)
                .map_err(|error| format!("m6_org_stale_serialize:{error}"))?;
            let transaction = self
                .connection
                .transaction()
                .map_err(|error| format!("m6_org_stale_tx:{error}"))?;
            transaction
                .execute(
                    "UPDATE m6_cross_project_advisories
                     SET lifecycle_status='STALE', freshness_state='stale', revision=?2,
                         payload_json=?3, updated_at_ms=?4
                     WHERE advisory_id=?1 AND lifecycle_status='ISSUED'",
                    params![
                        advisory.advisory_id,
                        advisory.revision as i64,
                        payload,
                        now_ms
                    ],
                )
                .map_err(|error| format!("m6_org_stale_update:{error}"))?;
            transaction
                .execute(
                    "INSERT INTO m6_org_audit_events (
                        event_id, event_type, target_ref, payload_json, created_at_ms
                     ) VALUES (?1,'CrossProjectAdvisoryMarkedStale',?2,?3,?4)",
                    params![
                        format!(
                            "m6-audit:stale:{}:{}",
                            advisory.advisory_id, advisory.revision
                        ),
                        advisory.advisory_id,
                        serde_json::to_string(&json!({
                            "advisory_id": advisory.advisory_id,
                            "reason": "summary_version_watermark_or_hash_changed",
                            "history_overwritten": false,
                            "project_writeback": false
                        }))
                        .map_err(|error| format!("m6_org_stale_audit_serialize:{error}"))?,
                        now_ms
                    ],
                )
                .map_err(|error| format!("m6_org_stale_audit_insert:{error}"))?;
            transaction
                .commit()
                .map_err(|error| format!("m6_org_stale_commit:{error}"))?;
            stale_ids.push(advisory.advisory_id);
        }
        Ok(stale_ids)
    }

    pub(crate) fn load_decision_request(
        &self,
        decision_request_id: &str,
    ) -> Result<Option<M6OrgDecisionRequest>, String> {
        self.load_payload(
            "SELECT payload_json FROM m6_decision_requests WHERE decision_request_id=?1",
            decision_request_id,
            "m6_org_decision_load",
        )
    }

    pub(crate) fn record_decision_request(
        &mut self,
        decision: &M6OrgDecisionRequest,
    ) -> Result<M6OrgDecisionRequest, String> {
        if let Some(existing) = self.load_payload::<M6OrgDecisionRequest>(
            "SELECT payload_json FROM m6_decision_requests WHERE idempotency_key=?1",
            &decision.idempotency_key,
            "m6_org_decision_idempotency_load",
        )? {
            if existing.source_object_ref != decision.source_object_ref
                || existing.source_revision != decision.source_revision
                || existing.requesting_actor_id != decision.requesting_actor_id
            {
                return Err("m6_org_decision_idempotency_collision".to_string());
            }
            return Ok(existing);
        }
        let advisory = self
            .load_advisory(&decision.source_object_ref)?
            .ok_or_else(|| "m6_org_advisory_not_found".to_string())?;
        if advisory.lifecycle_status != "ISSUED" {
            return Err("m6_org_advisory_not_issuable_for_adoption".to_string());
        }
        let payload = serde_json::to_string(decision)
            .map_err(|error| format!("m6_org_decision_serialize:{error}"))?;
        let audit_payload = serde_json::to_string(&json!({
            "decision_request_id": decision.decision_request_id,
            "advisory_id": decision.source_object_ref,
            "source_owner_ref": decision.source_owner_ref,
            "source_revision": decision.source_revision,
            "status": "PENDING",
            "creates_project_command": false,
            "creates_grant": false,
            "creates_workflow": false,
            "creates_project_fact": false
        }))
        .map_err(|error| format!("m6_org_decision_audit_serialize:{error}"))?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_decision_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_decision_requests (
                    decision_request_id, advisory_id, idempotency_key, status,
                    payload_json, created_at_ms
                 ) VALUES (?1,?2,?3,'PENDING',?4,?5)",
                params![
                    decision.decision_request_id,
                    decision.source_object_ref,
                    decision.idempotency_key,
                    payload,
                    decision.created_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_decision_insert:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_org_audit_events (
                    event_id, event_type, target_ref, payload_json, created_at_ms
                 ) VALUES (?1,'AdvisoryAdoptionDecisionRequested',?2,?3,?4)",
                params![
                    format!("m6-audit:decision:{}", decision.decision_request_id),
                    decision.decision_request_id,
                    audit_payload,
                    decision.created_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_decision_audit_insert:{error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_decision_commit:{error}"))?;
        Ok(decision.clone())
    }

    pub(crate) fn record_application_observation(
        &mut self,
        observation: &M6OrgPerProjectApplicationObservation,
    ) -> Result<M6OrgAdvisoryApplicationProjection, String> {
        let advisory = self
            .load_advisory(&observation.advisory_id)?
            .ok_or_else(|| "m6_org_advisory_not_found".to_string())?;
        let decision = self
            .load_decision_request(&observation.decision_request_id)?
            .ok_or_else(|| "m6_org_decision_request_not_found".to_string())?;
        if decision.source_object_ref != observation.advisory_id {
            return Err("m6_org_application_decision_advisory_mismatch".to_string());
        }
        if let Some(existing) = self.load_payload::<M6OrgPerProjectApplicationObservation>(
            "SELECT payload_json FROM m6_advisory_application_observations
             WHERE authoritative_command_receipt_ref=?1",
            &observation.authoritative_command_receipt_ref,
            "m6_org_application_receipt_load",
        )? {
            if existing != *observation {
                return Err("m6_org_application_receipt_collision".to_string());
            }
            return self.application_projection(
                &advisory,
                &decision.decision_request_id,
                observation.observed_at_ms,
            );
        }
        let payload = serde_json::to_string(observation)
            .map_err(|error| format!("m6_org_application_serialize:{error}"))?;
        let audit_payload = serde_json::to_string(&json!({
            "observation_id": observation.observation_id,
            "advisory_id": observation.advisory_id,
            "decision_request_id": observation.decision_request_id,
            "project_id": observation.project_id,
            "authoritative_command_receipt_ref": observation.authoritative_command_receipt_ref,
            "outcome": observation.outcome.as_str(),
            "owns_project_result": false,
            "changes_advisory_lifecycle": false
        }))
        .map_err(|error| format!("m6_org_application_audit_serialize:{error}"))?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| format!("m6_org_application_tx:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_advisory_application_observations (
                    observation_id, advisory_id, decision_request_id,
                    authoritative_command_receipt_ref, payload_json, observed_at_ms
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    observation.observation_id,
                    observation.advisory_id,
                    observation.decision_request_id,
                    observation.authoritative_command_receipt_ref,
                    payload,
                    observation.observed_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_application_insert:{error}"))?;
        transaction
            .execute(
                "INSERT INTO m6_org_audit_events (
                    event_id, event_type, target_ref, payload_json, created_at_ms
                 ) VALUES (?1,'AdvisoryApplicationObserved',?2,?3,?4)",
                params![
                    format!("m6-audit:application:{}", observation.observation_id),
                    observation.observation_id,
                    audit_payload,
                    observation.observed_at_ms
                ],
            )
            .map_err(|error| format!("m6_org_application_audit_insert:{error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("m6_org_application_commit:{error}"))?;
        self.application_projection(
            &advisory,
            &decision.decision_request_id,
            observation.observed_at_ms,
        )
    }

    fn application_projection(
        &self,
        advisory: &M6OrgCrossProjectAdvisory,
        decision_request_id: &str,
        projected_at_ms: i64,
    ) -> Result<M6OrgAdvisoryApplicationProjection, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload_json FROM m6_advisory_application_observations
                 WHERE advisory_id=?1 AND decision_request_id=?2
                 ORDER BY observed_at_ms, observation_id",
            )
            .map_err(|error| format!("m6_org_projection_prepare:{error}"))?;
        let rows = statement
            .query_map(params![advisory.advisory_id, decision_request_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| format!("m6_org_projection_query:{error}"))?;
        let mut history = Vec::new();
        for row in rows {
            let payload = row.map_err(|error| format!("m6_org_projection_row:{error}"))?;
            history.push(
                serde_json::from_str::<M6OrgPerProjectApplicationObservation>(&payload)
                    .map_err(|error| format!("m6_org_projection_decode:{error}"))?,
            );
        }
        let outcomes = history
            .iter()
            .map(|observation| observation.outcome)
            .collect::<BTreeSet<_>>();
        let compensation_observation_refs = history
            .iter()
            .filter(|observation| observation.outcome == M6OrgApplicationOutcome::RolledBack)
            .map(|observation| observation.observation_id.clone())
            .collect::<Vec<_>>();
        let projection_revision = history.len() as u64;
        Ok(M6OrgAdvisoryApplicationProjection {
            application_projection_id: format!(
                "m6-application-projection:{}:{}",
                advisory.advisory_id, decision_request_id
            ),
            advisory_id: advisory.advisory_id.clone(),
            advisory_revision: advisory.revision,
            decision_request_id: decision_request_id.to_string(),
            observations: history.clone(),
            partial_apply: history.len() >= 2 && outcomes.len() >= 2,
            compensation_observation_refs,
            history,
            projected_at_ms,
            projection_revision,
        })
    }

    fn load_payload<T: serde::de::DeserializeOwned>(
        &self,
        sql: &str,
        key: &str,
        error_prefix: &str,
    ) -> Result<Option<T>, String> {
        let payload = self
            .connection
            .query_row(sql, [key], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|error| format!("{error_prefix}:{error}"))?;
        payload
            .map(|payload| {
                serde_json::from_str(&payload)
                    .map_err(|error| format!("{error_prefix}_decode:{error}"))
            })
            .transpose()
    }

    #[cfg(test)]
    pub(crate) fn count_rows(&self, table: &str) -> Result<i64, String> {
        let allowed = [
            "m6_cross_project_advisories",
            "m6_decision_requests",
            "m6_advisory_application_observations",
            "m6_org_audit_events",
        ];
        if !allowed.contains(&table) {
            return Err("m6_org_test_table_not_allowed".to_string());
        }
        self.connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|error| format!("m6_org_count_rows:{error}"))
    }
}
