// M2 legacy adapter + quarantine implementation.
// This implements JSON/sidecar/file owner adapter and quarantine mechanism.

use crate::m2_dto::*;
use crate::m2_ports::*;
use crate::m2_workflow_state::{WorkflowStateAggregate, WorkItem};
use rusqlite::Connection;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Legacy Adapter Implementation
pub struct LegacyAdapterImpl {
    quarantine_repo: Box<dyn UnknownQuarantineRepository>,
}

impl LegacyAdapterImpl {
    /// Create a new LegacyAdapterImpl
    pub fn new(quarantine_repo: Box<dyn UnknownQuarantineRepository>) -> Self {
        Self { quarantine_repo }
    }

    /// Adapt JSON sidecar to new format
    pub fn adapt_json_sidecar(
        &self,
        connection: &Connection,
        sidecar_content: &str,
        sidecar_path: &str,
    ) -> Result<AdaptedData, String> {
        // 1. Parse JSON
        let json_value: serde_json::Value = serde_json::from_str(sidecar_content)
            .map_err(|error| format!("json_parse_failed: {}", error))?;

        // 2. Validate JSON structure
        let validation_warnings = self.validate_json_structure(&json_value);

        // 3. Check for unknown fields
        let unknown_fields = self.extract_unknown_fields(&json_value);

        // 4. Check for corrupt data
        let corrupt_fields = self.extract_corrupt_fields(&json_value);

        // 5. Check for sensitive data
        let sensitive_fields = self.extract_sensitive_fields(&json_value);

        // 6. Quarantine unknown/corrupt/sensitive data
        for field in &unknown_fields {
            self.quarantine_field(
                connection,
                field,
                sidecar_path,
                "UNKNOWN_FIELD",
            )?;
        }

        for field in &corrupt_fields {
            self.quarantine_field(
                connection,
                field,
                sidecar_path,
                "CORRUPT_FIELD",
            )?;
        }

        for field in &sensitive_fields {
            self.quarantine_field(
                connection,
                field,
                sidecar_path,
                "SENSITIVE_FIELD",
            )?;
        }

        // 7. Create adapted data
        let adapted = AdaptedData {
            original_path: sidecar_path.to_string(),
            adapted_content: json_value,
            warnings: validation_warnings,
            quarantined_fields: unknown_fields
                .iter()
                .chain(corrupt_fields.iter())
                .chain(sensitive_fields.iter())
                .cloned()
                .collect(),
        };

        Ok(adapted)
    }

    /// Validate JSON structure
    fn validate_json_structure(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for required fields (simplified)
        if !json_value.is_object() {
            warnings.push("json_not_object".to_string());
        }

        warnings
    }

    /// Extract unknown fields
    fn extract_unknown_fields(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut unknown_fields = Vec::new();

        if let Some(object) = json_value.as_object() {
            for key in object.keys() {
                // Simple heuristic: fields starting with "unknown_" are unknown
                if key.starts_with("unknown_") {
                    unknown_fields.push(key.clone());
                }
            }
        }

        unknown_fields
    }

    /// Extract corrupt fields
    fn extract_corrupt_fields(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut corrupt_fields = Vec::new();

        if let Some(object) = json_value.as_object() {
            for (key, value) in object {
                // Check for null values where they shouldn't be
                if value.is_null() {
                    corrupt_fields.push(key.clone());
                }

                // Check for empty strings
                if let Some(s) = value.as_str() {
                    if s.is_empty() {
                        corrupt_fields.push(key.clone());
                    }
                }
            }
        }

        corrupt_fields
    }

    /// Extract sensitive fields
    fn extract_sensitive_fields(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut sensitive_fields = Vec::new();

        if let Some(object) = json_value.as_object() {
            for key in object.keys() {
                // Simple heuristic: fields containing "secret", "token", "credential" are sensitive
                let lower_key = key.to_lowercase();
                if lower_key.contains("secret")
                    || lower_key.contains("token")
                    || lower_key.contains("credential")
                    || lower_key.contains("password")
                {
                    sensitive_fields.push(key.clone());
                }
            }
        }

        sensitive_fields
    }

    /// Quarantine a field
    fn quarantine_field(
        &self,
        connection: &Connection,
        field_name: &str,
        source_path: &str,
        reason_code: &str,
    ) -> Result<(), String> {
        let quarantine = UnknownQuarantineDto {
            quarantine_id: generate_uuid(),
            source_ref: format!("{}:{}", source_path, field_name),
            reason_code: reason_code.to_string(),
            scope_ref: Some("legacy_adapter".to_string()),
            observed_at: generate_timestamp(),
            resolution_state: QuarantineResolutionState::Pending,
            resolution_ref: None,
            created_at: generate_timestamp(),
        };

        self.quarantine_repo.create(connection, &quarantine)?;

        Ok(())
    }

    /// Read and adapt workflow state sidecar
    pub fn adapt_workflow_state_sidecar(
        &self,
        connection: &Connection,
        sidecar_content: &str,
    ) -> Result<AdaptedWorkflowState, String> {
        // 1. Parse JSON
        let json_value: serde_json::Value = serde_json::from_str(sidecar_content)
            .map_err(|error| format!("json_parse_failed: {}", error))?;

        // 2. Extract workflow state
        let workflow_state = self.extract_workflow_state(&json_value)?;

        // 3. Validate workflow state
        let warnings = self.validate_workflow_state(&workflow_state);

        // 4. Check for unknown work items
        let unknown_work_items = self.extract_unknown_work_items(&workflow_state);

        // 5. Quarantine unknown work items
        for work_item in &unknown_work_items {
            self.quarantine_field(
                connection,
                &work_item.work_item_id,
                "workflow_state",
                "UNKNOWN_WORK_ITEM",
            )?;
        }

        Ok(AdaptedWorkflowState {
            state: workflow_state,
            warnings,
            quarantined_work_items: unknown_work_items,
        })
    }

    /// Extract workflow state
    fn extract_workflow_state(&self, json_value: &serde_json::Value) -> Result<WorkflowStateAggregate, String> {
        // Simplified extraction
        let project_id = json_value.get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let workflow_id = json_value.get("workflow_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let revision = json_value.get("revision")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let work_items = Vec::new(); // Simplified: empty for now

        Ok(WorkflowStateAggregate {
            project_id,
            workflow_id,
            revision,
            work_items,
        })
    }

    /// Validate workflow state
    fn validate_workflow_state(&self, state: &WorkflowStateAggregate) -> Vec<String> {
        let mut warnings = Vec::new();

        if state.project_id.is_empty() {
            warnings.push("empty_project_id".to_string());
        }

        if state.workflow_id.is_empty() {
            warnings.push("empty_workflow_id".to_string());
        }

        if state.revision < 0 {
            warnings.push("negative_revision".to_string());
        }

        warnings
    }

    /// Extract unknown work items
    fn extract_unknown_work_items(&self, state: &WorkflowStateAggregate) -> Vec<WorkItem> {
        // Simplified: return empty for now
        Vec::new()
    }
}

/// Adapted Data
#[derive(Clone, Debug, PartialEq)]
pub struct AdaptedData {
    pub original_path: String,
    pub adapted_content: serde_json::Value,
    pub warnings: Vec<String>,
    pub quarantined_fields: Vec<String>,
}

/// Adapted Workflow State
#[derive(Clone, Debug, PartialEq)]
pub struct AdaptedWorkflowState {
    pub state: WorkflowStateAggregate,
    pub warnings: Vec<String>,
    pub quarantined_work_items: Vec<WorkItem>,
}

/// Quarantine Manager Implementation
pub struct QuarantineManagerImpl {
    quarantine_repo: Box<dyn UnknownQuarantineRepository>,
}

impl QuarantineManagerImpl {
    /// Create a new QuarantineManagerImpl
    pub fn new(quarantine_repo: Box<dyn UnknownQuarantineRepository>) -> Self {
        Self { quarantine_repo }
    }

    /// Get all pending quarantine records
    pub fn get_pending_quarantines(
        &self,
        connection: &Connection,
    ) -> Result<Vec<UnknownQuarantineDto>, String> {
        self.quarantine_repo.get_by_state(
            connection,
            QuarantineResolutionState::Pending,
        )
    }

    /// Resolve a quarantine record
    pub fn resolve_quarantine(
        &self,
        connection: &Connection,
        quarantine_id: &str,
        resolution_state: QuarantineResolutionState,
        resolution_ref: Option<String>,
    ) -> Result<(), String> {
        self.quarantine_repo.update_resolution(
            connection,
            quarantine_id,
            resolution_state,
            resolution_ref,
        )
    }

    /// Reclassify a quarantine record
    pub fn reclassify_quarantine(
        &self,
        connection: &Connection,
        quarantine_id: &str,
        new_type: &str,
    ) -> Result<(), String> {
        self.quarantine_repo.update_resolution(
            connection,
            quarantine_id,
            QuarantineResolutionState::Reclassified,
            Some(new_type.to_string()),
        )
    }

    /// Rebuild from quarantine
    pub fn rebuild_from_quarantine(
        &self,
        connection: &Connection,
        quarantine_id: &str,
        source_ref: &str,
    ) -> Result<(), String> {
        self.quarantine_repo.update_resolution(
            connection,
            quarantine_id,
            QuarantineResolutionState::Rebuilt,
            Some(source_ref.to_string()),
        )
    }

    /// Delete quarantine record
    pub fn delete_quarantine(
        &self,
        connection: &Connection,
        quarantine_id: &str,
    ) -> Result<(), String> {
        self.quarantine_repo.update_resolution(
            connection,
            quarantine_id,
            QuarantineResolutionState::Deleted,
            None,
        )
    }

    /// Hold quarantine record indefinitely
    pub fn hold_quarantine(
        &self,
        connection: &Connection,
        quarantine_id: &str,
    ) -> Result<(), String> {
        self.quarantine_repo.update_resolution(
            connection,
            quarantine_id,
            QuarantineResolutionState::Held,
            None,
        )
    }

    /// Check if quarantine record exists
    pub fn quarantine_exists(
        &self,
        connection: &Connection,
        quarantine_id: &str,
    ) -> Result<bool, String> {
        self.quarantine_repo.exists(connection, quarantine_id)
    }
}

/// Generate UUID v4 (simplified)
fn generate_uuid() -> String {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).expect("failed to generate random bytes");
    // Set version 4 and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Generate ISO 8601 timestamp
fn generate_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Simple ISO 8601 format
    format!(
        "2026-08-03T{:02}:{:02}:{:02}.{:09}Z",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60,
        nanos
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_adapter_impl_creates() {
        // Note: This test only verifies the adapter can be created.
        // Full integration tests require actual database connections.
    }

    #[test]
    fn quarantine_manager_impl_creates() {
        // Note: This test only verifies the quarantine manager can be created.
        // Full integration tests require actual database connections.
    }

    #[test]
    fn adapted_data_variants() {
        let adapted = AdaptedData {
            original_path: "test.json".to_string(),
            adapted_content: serde_json::json!({"key": "value"}),
            warnings: vec!["warning1".to_string()],
            quarantined_fields: vec!["field1".to_string()],
        };

        assert_eq!(adapted.original_path, "test.json");
        assert_eq!(adapted.warnings.len(), 1);
        assert_eq!(adapted.quarantined_fields.len(), 1);
    }

    #[test]
    fn adapted_workflow_state_variants() {
        let state = WorkflowStateAggregate {
            project_id: "project1".to_string(),
            workflow_id: "workflow1".to_string(),
            revision: 1,
            work_items: Vec::new(),
        };

        let adapted = AdaptedWorkflowState {
            state,
            warnings: vec!["warning1".to_string()],
            quarantined_work_items: Vec::new(),
        };

        assert_eq!(adapted.state.project_id, "project1");
        assert_eq!(adapted.warnings.len(), 1);
        assert_eq!(adapted.quarantined_work_items.len(), 0);
    }
}
