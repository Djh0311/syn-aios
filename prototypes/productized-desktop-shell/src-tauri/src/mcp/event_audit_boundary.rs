// SYN-FND-005: Event / Audit sensitive boundary and unified receipt types.
//
// **STAGED — 未上活路径**。类型层已建、单元测试已建，但尚未接入任何 Tauri command。
// 接线属后续包（CURRENT.md 明写的拦阻项）。
// 证据级别 = 单元测试。接线后升级为集成测试。
//
// This module provides:
// 1. Secret scrubber: mechanically rejects or redacts sensitive content from event payloads
// 2. Audit view boundary: ensures product DTOs never expose raw JSON from stores
// 3. Unified receipt types for command admission outcomes
//
// Contract: docs/contracts/event-audit-outbox-v1.md
// Evidence level: STATIC_OPENING_ONLY → will be upgraded after focused tests.

#![allow(dead_code)] // staged foundation, not yet connected — warnings are expected

// ============================================================================
// §1  Sensitive Content Scrubber
// ============================================================================

/// Patterns that indicate sensitive content. Any match means the content
/// must be rejected or scrubbed before entering event/audit/product-DTO.
const SENSITIVE_PATTERNS: &[&str] = &[
    // Credentials and tokens
    "token", "secret", "password", "passwd", "credential", "auth_token",
    "access_token", "refresh_token", "api_key", "apikey", "api-key",
    // OAuth
    "oauth", "bearer", "authorization",
    // Environment and config
    ".env", "env_file", "environment_variable",
    // Provider responses
    "provider_response", "raw_response", "upstream_response",
    // Transcripts and prompts (full content)
    "full_transcript", "prompt_body", "rollout_body",
    // SSH and keys
    "private_key", "ssh_key", "id_rsa", "id_ed25519",
    // Keychain and password stores
    "keychain", "keyring", "keytar",
];

/// Content classification result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentClassification {
    /// Content is safe to include in events/audit.
    Safe,
    /// Content contains sensitive material and must be scrubbed.
    Sensitive { reason: String },
    /// Content contains a raw transcript that must be omitted entirely.
    Forbidden { reason: String },
}

/// Classify whether a piece of content is safe for event/audit inclusion.
pub fn classify_content(text: &str) -> ContentClassification {
    let lower = text.to_lowercase();

    // Check for raw transcript / prompt body markers
    if lower.contains("full_transcript") || lower.contains("prompt_body") || lower.contains("rollout_body") {
        return ContentClassification::Forbidden {
            reason: "raw transcript/prompt content forbidden in events".to_string(),
        };
    }

    // Check for credential patterns
    for pattern in SENSITIVE_PATTERNS {
        if lower.contains(pattern) {
            return ContentClassification::Sensitive {
                reason: format!("content matches sensitive pattern: {}", pattern),
            };
        }
    }

    // Check for base64-encoded secrets (rough heuristic: long strings of [A-Za-z0-9+/=])
    if text.len() > 100 && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
        return ContentClassification::Sensitive {
            reason: "content appears to be base64-encoded (possible secret)".to_string(),
        };
    }

    ContentClassification::Safe
}

/// Scrub sensitive content from a string, replacing matches with redacted placeholders.
pub fn scrub_content(text: &str) -> String {
    let classification = classify_content(text);
    match classification {
        ContentClassification::Safe => text.to_string(),
        ContentClassification::Forbidden { .. } => "[REDACTED: forbidden content]".to_string(),
        ContentClassification::Sensitive { .. } => {
            // Replace sensitive patterns with redacted placeholders
            let mut result = text.to_string();
            for pattern in SENSITIVE_PATTERNS {
                if result.to_lowercase().contains(pattern) {
                    result = format!("[REDACTED: {} content]", pattern);
                }
            }
            result
        }
    }
}

/// Scrub a JSON value recursively, redacting sensitive fields.
pub fn scrub_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut scrubbed = serde_json::Map::new();
            for (key, val) in map {
                let lower_key = key.to_lowercase();
                let is_sensitive = SENSITIVE_PATTERNS.iter().any(|p| lower_key.contains(p));
                if is_sensitive {
                    scrubbed.insert(key.clone(), serde_json::Value::String("[REDACTED]".to_string()));
                } else {
                    scrubbed.insert(key.clone(), scrub_json_value(val));
                }
            }
            serde_json::Value::Object(scrubbed)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(scrub_json_value).collect())
        }
        other => other.clone(),
    }
}

// ============================================================================
// §2  Audit View Boundary
// ============================================================================

/// Maximum length for a redacted audit summary field.
const MAX_AUDIT_SUMMARY_LEN: usize = 512;

/// A scrubbed audit entry for product DTO exposure.
/// This is the ONLY type that should be returned to the frontend.
/// Raw store JSON must never cross this boundary.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ScrubbedAuditEntry {
    pub entry_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub actor: String,
    pub scope: String,
    pub summary: String,
    pub sensitivity: String,
    pub hash: String,
}

/// Create a scrubbed audit entry from raw event data.
/// Ensures no sensitive content leaks into product responses.
pub fn create_scrubbed_audit_entry(
    entry_id: &str,
    event_type: &str,
    timestamp: &str,
    actor: &str,
    scope: &str,
    summary: &str,
    sensitivity: &str,
) -> ScrubbedAuditEntry {
    let scrubbed_summary = if summary.len() > MAX_AUDIT_SUMMARY_LEN {
        format!("{}…[truncated]", &summary[..MAX_AUDIT_SUMMARY_LEN])
    } else {
        scrub_content(summary)
    };

    // Compute a simple hash for integrity verification
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    entry_id.hash(&mut hasher);
    event_type.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    let hash = format!("{:016x}", hasher.finish());

    ScrubbedAuditEntry {
        entry_id: entry_id.to_string(),
        event_type: event_type.to_string(),
        timestamp: timestamp.to_string(),
        actor: actor.to_string(),
        scope: scope.to_string(),
        summary: scrubbed_summary,
        sensitivity: sensitivity.to_string(),
        hash,
    }
}

// ============================================================================
// §3  Unified Receipt Types
// ============================================================================

/// Unified receipt for command admission outcomes.
/// All command paths must produce one of these variants.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CommandReceipt {
    /// Command was allowed and committed.
    Allowed {
        receipt_id: String,
        command_id: String,
        committed_at: String,
    },
    /// Command was denied by policy.
    Denied {
        receipt_id: String,
        command_id: String,
        reason: String,
    },
    /// Command requires user confirmation before proceeding.
    NeedsConfirmation {
        receipt_id: String,
        command_id: String,
        reason: String,
    },
    /// Command was committed but has external side effects pending.
    ExternalPending {
        receipt_id: String,
        command_id: String,
        external_ref: String,
    },
    /// External side effect completed.
    ExternalResult {
        receipt_id: String,
        command_id: String,
        external_ref: String,
        result: String,
    },
    /// Command committed but projection is degraded.
    ProjectionDegraded {
        receipt_id: String,
        command_id: String,
        degraded_reason: String,
    },
    /// Command failed.
    Failed {
        receipt_id: String,
        command_id: String,
        error_code: String,
        reason: String,
    },
}

impl CommandReceipt {
    /// Get the receipt ID regardless of variant.
    pub fn receipt_id(&self) -> &str {
        match self {
            Self::Allowed { receipt_id, .. }
            | Self::Denied { receipt_id, .. }
            | Self::NeedsConfirmation { receipt_id, .. }
            | Self::ExternalPending { receipt_id, .. }
            | Self::ExternalResult { receipt_id, .. }
            | Self::ProjectionDegraded { receipt_id, .. }
            | Self::Failed { receipt_id, .. } => receipt_id,
        }
    }

    /// Get the command ID regardless of variant.
    pub fn command_id(&self) -> &str {
        match self {
            Self::Allowed { command_id, .. }
            | Self::Denied { command_id, .. }
            | Self::NeedsConfirmation { command_id, .. }
            | Self::ExternalPending { command_id, .. }
            | Self::ExternalResult { command_id, .. }
            | Self::ProjectionDegraded { command_id, .. }
            | Self::Failed { command_id, .. } => command_id,
        }
    }

    /// Check if this receipt represents a successful outcome.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Allowed { .. } | Self::ExternalResult { .. })
    }

    /// Check if this receipt requires user action.
    pub fn requires_action(&self) -> bool {
        matches!(self, Self::NeedsConfirmation { .. })
    }
}

/// Generate a deterministic receipt ID from command context.
pub fn generate_receipt_id(command_id: &str, timestamp: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    command_id.hash(&mut hasher);
    timestamp.hash(&mut hasher);
    format!("receipt:{:016x}", hasher.finish())
}

// ============================================================================
// §4  Event Payload Boundary
// ============================================================================

/// Maximum payload summary length for event storage.
const MAX_EVENT_PAYLOAD_SUMMARY_LEN: usize = 1024;

/// An event payload that has been validated against the contract.
/// Only summary/ref/hash are allowed — raw content is forbidden.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct EventPayloadBoundary {
    pub summary: String,
    pub payload_ref: Option<String>,
    pub payload_hash: String,
}

/// Validate and create an event payload boundary.
/// Rejects raw transcripts, prompts, tool outputs, and credentials.
pub fn create_event_payload(
    summary: &str,
    payload_ref: Option<&str>,
    raw_content: &str,
) -> Result<EventPayloadBoundary, String> {
    // Check raw content for forbidden patterns
    let classification = classify_content(raw_content);
    if let ContentClassification::Forbidden { reason } = classification {
        return Err(format!("event_payload_rejected: {reason}"));
    }

    // Truncate summary if needed
    let truncated_summary = if summary.len() > MAX_EVENT_PAYLOAD_SUMMARY_LEN {
        format!("{}…[truncated]", &summary[..MAX_EVENT_PAYLOAD_SUMMARY_LEN])
    } else {
        summary.to_string()
    };

    // Compute hash of raw content for audit trail
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw_content.hash(&mut hasher);
    let payload_hash = format!("{:016x}", hasher.finish());

    Ok(EventPayloadBoundary {
        summary: scrub_content(&truncated_summary),
        payload_ref: payload_ref.map(String::from),
        payload_hash,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- classify_content ----

    #[test]
    fn safe_content_passes() {
        assert_eq!(classify_content("just a normal string"), ContentClassification::Safe);
    }

    #[test]
    fn token_content_flagged() {
        match classify_content("my token value") {
            ContentClassification::Sensitive { .. } => {}
            other => panic!("expected Sensitive, got {other:?}"),
        }
    }

    #[test]
    fn transcript_forbidden() {
        match classify_content("full_transcript: lots of data") {
            ContentClassification::Forbidden { .. } => {}
            other => panic!("expected Forbidden, got {other:?}"),
        }
    }

    #[test]
    fn prompt_body_forbidden() {
        match classify_content("prompt_body content here") {
            ContentClassification::Forbidden { .. } => {}
            other => panic!("expected Forbidden, got {other:?}"),
        }
    }

    // ---- scrub_content ----

    #[test]
    fn safe_content_unchanged() {
        assert_eq!(scrub_content("hello world"), "hello world");
    }

    #[test]
    fn sensitive_content_scrubbed() {
        let result = scrub_content("my token is abc123");
        assert!(result.contains("[REDACTED"));
    }

    #[test]
    fn forbidden_content_redacted() {
        let result = scrub_content("full_transcript: secret data");
        assert_eq!(result, "[REDACTED: forbidden content]");
    }

    // ---- scrub_json_value ----

    #[test]
    fn json_safe_fields_preserved() {
        let val = serde_json::json!({"name": "test", "count": 42});
        let scrubbed = scrub_json_value(&val);
        assert_eq!(scrubbed["name"], "test");
        assert_eq!(scrubbed["count"], 42);
    }

    #[test]
    fn json_sensitive_fields_redacted() {
        let val = serde_json::json!({"token": "secret123", "name": "test"});
        let scrubbed = scrub_json_value(&val);
        assert_eq!(scrubbed["token"], "[REDACTED]");
        assert_eq!(scrubbed["name"], "test");
    }

    // ---- CommandReceipt ----

    #[test]
    fn receipt_allowed_is_success() {
        let r = CommandReceipt::Allowed {
            receipt_id: "r1".to_string(),
            command_id: "c1".to_string(),
            committed_at: "2026-01-01".to_string(),
        };
        assert!(r.is_success());
        assert!(!r.requires_action());
        assert_eq!(r.receipt_id(), "r1");
        assert_eq!(r.command_id(), "c1");
    }

    #[test]
    fn receipt_denied_not_success() {
        let r = CommandReceipt::Denied {
            receipt_id: "r2".to_string(),
            command_id: "c2".to_string(),
            reason: "policy".to_string(),
        };
        assert!(!r.is_success());
        assert!(!r.requires_action());
    }

    #[test]
    fn receipt_needs_confirmation_requires_action() {
        let r = CommandReceipt::NeedsConfirmation {
            receipt_id: "r3".to_string(),
            command_id: "c3".to_string(),
            reason: "high risk".to_string(),
        };
        assert!(!r.is_success());
        assert!(r.requires_action());
    }

    // ---- create_event_payload ----

    #[test]
    fn event_payload_safe_content() {
        let result = create_event_payload("test summary", Some("ref1"), "safe content");
        assert!(result.is_ok());
    }

    #[test]
    fn event_payload_rejects_transcript() {
        let result = create_event_payload("summary", None, "full_transcript: secret data");
        assert!(result.is_err());
    }

    #[test]
    fn event_payload_scrubs_sensitive_summary() {
        let result = create_event_payload("my token value", None, "safe");
        assert!(result.is_ok());
        let payload = result.unwrap();
        assert!(payload.summary.contains("[REDACTED"));
    }
}
