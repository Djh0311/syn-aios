//! M4C03 read-only Attention projection DTOs and deterministic ordering.
//!
//! This module has no connection or mutation capability. The repository
//! supplies already-validated rows; ordering is repeated mechanically in Rust
//! so SQLite collation or renderer order never becomes priority authority.

use crate::m4_secretary_domain::{
    m4_is_opaque_reference, m4_parse_rfc3339_utc_key, M4_WORKFLOW_ATTENTION_OBJECT_TYPE,
    M4_WORKFLOW_ATTENTION_SOURCE_TYPE,
};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SourceLinkRead {
    pub(crate) link_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) expected_source_revision: u64,
    pub(crate) opaque_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4InboxItemRead {
    pub(crate) inbox_item_id: String,
    pub(crate) source_identity_key: String,
    pub(crate) source_owner_ref: String,
    pub(crate) source_link: M4SourceLinkRead,
    pub(crate) current_source_status: String,
    pub(crate) status: String,
    pub(crate) priority_rank: i64,
    pub(crate) priority_reason_code: String,
    pub(crate) priority_reason_text: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) received_at_utc: String,
    pub(crate) last_source_change_at_utc: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) sensitivity: String,
    pub(crate) revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4OpenLoopRead {
    pub(crate) open_loop_id: String,
    pub(crate) source_identity_key: String,
    pub(crate) source_owner_ref: String,
    pub(crate) source_link: M4SourceLinkRead,
    pub(crate) current_source_status: String,
    pub(crate) status: String,
    pub(crate) why_open_code: String,
    pub(crate) priority_rank: i64,
    pub(crate) priority_reason_code: String,
    pub(crate) priority_reason_text: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) snoozed_until_utc: Option<String>,
    pub(crate) closure_reason_code: Option<String>,
    pub(crate) last_source_change_at_utc: String,
    pub(crate) scrubbed_summary_ref: String,
    pub(crate) sensitivity: String,
    pub(crate) revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4AttentionSnapshot {
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) inbox_items: Vec<M4InboxItemRead>,
    pub(crate) open_loops: Vec<M4OpenLoopRead>,
}

/// M4C04's standalone, explicitly user-created personal todo read shape.
///
/// This is deliberately stringly typed at the read boundary so it does not
/// become coupled to M4C04 write-side aggregate types.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4PersonalActionRead {
    pub(crate) personal_action_id: String,
    pub(crate) explicit_user_command_ref: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) due_at_utc: Option<String>,
    pub(crate) revision: String,
}

/// M4C04's in-app delivery/read/dismiss projection only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4NotificationRead {
    pub(crate) notification_id: String,
    pub(crate) source_ref: M4SourceLinkRead,
    pub(crate) subject_ref: String,
    pub(crate) notification_purpose_code: String,
    pub(crate) delivery_channel: String,
    pub(crate) status: String,
    pub(crate) created_at_utc: String,
    pub(crate) delivered_at_utc: Option<String>,
    pub(crate) read_at_utc: Option<String>,
    pub(crate) dismissed_at_utc: Option<String>,
    pub(crate) revision: String,
}

/// M4C04's local schedule and delivery state projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4ReminderRead {
    pub(crate) reminder_id: String,
    pub(crate) owner_ref: String,
    pub(crate) explicit_schedule_command_id: String,
    pub(crate) scheduled_for_utc: String,
    pub(crate) iana_timezone: String,
    pub(crate) status: String,
    pub(crate) last_fired_at_utc: Option<String>,
    pub(crate) snoozed_until_utc: Option<String>,
    pub(crate) revision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4DecisionRead {
    pub(crate) decision_projection_id: String,
    pub(crate) source_identity_key: String,
    pub(crate) source_event_key: String,
    pub(crate) source_ref: String,
    pub(crate) owner_status: String,
    pub(crate) local_visibility_status: String,
    pub(crate) decision_by_utc: Option<String>,
    pub(crate) source_revision: String,
    pub(crate) revision: String,
}

/// The only owner-writeback facts that M4C04 may expose.
///
/// In particular, this type contains neither source owner state nor a raw
/// callback, executable payload, credential, URL, or filesystem path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4OwnerWritebackReceiptRead {
    pub(crate) source_ref: M4SourceLinkRead,
    pub(crate) expected_source_revision: u64,
    pub(crate) explicit_intent_code: String,
    pub(crate) status: String,
    pub(crate) scrubbed_owner_receipt_ref: Option<String>,
    pub(crate) error_code: Option<String>,
}

/// Complete M4C04 coordination read snapshot.
///
/// It deliberately stops at M4C04 coordination data: there is no session,
/// conversation, handoff, daily brief, or daily report state here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4CoordinationSnapshot {
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) inbox_items: Vec<M4InboxItemRead>,
    pub(crate) open_loops: Vec<M4OpenLoopRead>,
    pub(crate) personal_actions: Vec<M4PersonalActionRead>,
    pub(crate) notifications: Vec<M4NotificationRead>,
    pub(crate) reminders: Vec<M4ReminderRead>,
    pub(crate) decisions: Vec<M4DecisionRead>,
    pub(crate) owner_writeback_receipts: Vec<M4OwnerWritebackReceiptRead>,
}

pub(crate) fn validate_m4c04_coordination_snapshot(
    snapshot: &M4CoordinationSnapshot,
) -> Result<(), String> {
    if !m4c04_is_safe_typed_ref(&snapshot.scope_ref)
        || !m4c04_is_lower_hex_digest(&snapshot.scope_source_watermark)
    {
        return Err("m4c04_snapshot_scope_invalid".to_string());
    }

    for action in &snapshot.personal_actions {
        validate_m4c04_personal_action(action)?;
    }
    for notification in &snapshot.notifications {
        validate_m4c04_notification(notification)?;
    }
    for reminder in &snapshot.reminders {
        validate_m4c04_reminder(reminder)?;
    }
    for decision in &snapshot.decisions {
        validate_m4r02_decision(decision)?;
    }
    for receipt in &snapshot.owner_writeback_receipts {
        validate_m4c04_owner_writeback_receipt(receipt)?;
    }
    Ok(())
}

/// Validates the complete M4C04 snapshot before applying only mechanical,
/// identifier-based ordering to the new local coordination collections.
pub(crate) fn sort_m4c04_coordination_snapshot(
    snapshot: &mut M4CoordinationSnapshot,
) -> Result<(), String> {
    validate_m4c04_coordination_snapshot(snapshot)?;
    sort_m4_inbox_items(&mut snapshot.inbox_items)?;
    sort_m4_open_loops(&mut snapshot.open_loops)?;
    snapshot
        .personal_actions
        .sort_by(|left, right| left.personal_action_id.cmp(&right.personal_action_id));
    snapshot
        .notifications
        .sort_by(|left, right| left.notification_id.cmp(&right.notification_id));
    snapshot
        .reminders
        .sort_by(|left, right| left.reminder_id.cmp(&right.reminder_id));
    snapshot.decisions.sort_by(|left, right| {
        left.decision_projection_id
            .cmp(&right.decision_projection_id)
    });
    snapshot.owner_writeback_receipts.sort_by(|left, right| {
        (
            &left.source_ref.source_owner_ref,
            &left.source_ref.object_type,
            &left.source_ref.canonical_source_object_id,
            left.expected_source_revision,
            &left.explicit_intent_code,
            &left.status,
            &left.scrubbed_owner_receipt_ref,
            &left.error_code,
        )
            .cmp(&(
                &right.source_ref.source_owner_ref,
                &right.source_ref.object_type,
                &right.source_ref.canonical_source_object_id,
                right.expected_source_revision,
                &right.explicit_intent_code,
                &right.status,
                &right.scrubbed_owner_receipt_ref,
                &right.error_code,
            ))
    });
    Ok(())
}

fn validate_m4c04_personal_action(action: &M4PersonalActionRead) -> Result<(), String> {
    if !m4c04_has_deterministic_id(&action.personal_action_id, "personal-action:")
        || !m4_is_opaque_reference(&action.explicit_user_command_ref)
        || !m4c04_is_title(&action.title)
        || !matches!(action.status.as_str(), "OPEN" | "COMPLETED" | "CANCELLED")
        || action
            .due_at_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
        || !m4c04_is_canonical_u64(&action.revision)
    {
        return Err("m4c04_personal_action_invalid".to_string());
    }
    Ok(())
}

fn validate_m4c04_notification(notification: &M4NotificationRead) -> Result<(), String> {
    let created_at = m4_parse_rfc3339_utc_key(&notification.created_at_utc)
        .ok_or_else(|| "m4c04_notification_timestamp_invalid".to_string())?;
    let delivered_at = m4c04_optional_utc_key(notification.delivered_at_utc.as_deref())?;
    let read_at = m4c04_optional_utc_key(notification.read_at_utc.as_deref())?;
    let dismissed_at = m4c04_optional_utc_key(notification.dismissed_at_utc.as_deref())?;

    if !m4c04_has_deterministic_id(&notification.notification_id, "notification:")
        || !validate_m4c04_source_link(&notification.source_ref)
        || !m4c04_is_typed_reference(&notification.subject_ref)
        || !m4c04_is_code(&notification.notification_purpose_code)
        || notification.delivery_channel != "IN_APP"
        || !m4c04_is_canonical_u64(&notification.revision)
        || delivered_at.is_some_and(|value| value < created_at)
        || read_at.is_some_and(|value| value < created_at)
        || dismissed_at.is_some_and(|value| value < created_at)
        || matches!((delivered_at, read_at), (None, Some(_)))
        || matches!((delivered_at, read_at), (Some(delivered), Some(read)) if read < delivered)
    {
        return Err("m4c04_notification_invalid".to_string());
    }

    let state_is_consistent = match notification.status.as_str() {
        "PENDING" => delivered_at.is_none() && read_at.is_none() && dismissed_at.is_none(),
        "DELIVERED" => delivered_at.is_some() && read_at.is_none() && dismissed_at.is_none(),
        "READ" => delivered_at.is_some() && read_at.is_some() && dismissed_at.is_none(),
        "DISMISSED" => dismissed_at.is_some(),
        _ => false,
    };
    if !state_is_consistent {
        return Err("m4c04_notification_state_invalid".to_string());
    }
    Ok(())
}

fn validate_m4c04_reminder(reminder: &M4ReminderRead) -> Result<(), String> {
    let scheduled_for = m4_parse_rfc3339_utc_key(&reminder.scheduled_for_utc)
        .ok_or_else(|| "m4c04_reminder_timestamp_invalid".to_string())?;
    let last_fired_at = m4c04_optional_utc_key(reminder.last_fired_at_utc.as_deref())?;
    let snoozed_until = m4c04_optional_utc_key(reminder.snoozed_until_utc.as_deref())?;

    if !m4c04_has_deterministic_id(&reminder.reminder_id, "reminder:")
        || !m4c04_is_typed_reference(&reminder.owner_ref)
        || !m4_is_opaque_reference(&reminder.explicit_schedule_command_id)
        || !m4c04_is_iana_timezone(&reminder.iana_timezone)
        || !m4c04_is_canonical_u64(&reminder.revision)
        || last_fired_at.is_some_and(|value| value < scheduled_for)
        || snoozed_until.is_some_and(|value| value < scheduled_for)
    {
        return Err("m4c04_reminder_invalid".to_string());
    }

    let state_is_consistent = match reminder.status.as_str() {
        "SCHEDULED" => last_fired_at.is_none() && snoozed_until.is_none(),
        "FIRED" => last_fired_at.is_some() && snoozed_until.is_none(),
        "SNOOZED" => snoozed_until.is_some(),
        "DISMISSED" | "CANCELLED" => snoozed_until.is_none(),
        _ => false,
    };
    if !state_is_consistent {
        return Err("m4c04_reminder_state_invalid".to_string());
    }
    Ok(())
}

fn validate_m4r02_decision(decision: &M4DecisionRead) -> Result<(), String> {
    if !m4c04_has_deterministic_id(&decision.decision_projection_id, "decision-projection:")
        || !m4c04_has_deterministic_id(&decision.source_identity_key, "source:")
        || !m4c04_has_deterministic_id(&decision.source_event_key, "source-event:")
        || decision.source_ref != decision.source_identity_key
        || !matches!(
            decision.owner_status.as_str(),
            "OPEN" | "ANSWERED" | "EXPIRED" | "WITHDRAWN"
        )
        || !matches!(
            decision.local_visibility_status.as_str(),
            "UNREAD" | "READ" | "DISMISSED"
        )
        || decision
            .decision_by_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
        || !m4c04_is_canonical_u64(&decision.source_revision)
        || !m4c04_is_canonical_u64(&decision.revision)
    {
        return Err("m4r02_decision_read_invalid".to_string());
    }
    Ok(())
}

fn validate_m4c04_owner_writeback_receipt(
    receipt: &M4OwnerWritebackReceiptRead,
) -> Result<(), String> {
    if !validate_m4c04_source_link(&receipt.source_ref)
        || receipt.expected_source_revision != receipt.source_ref.expected_source_revision
        || !m4c04_is_code(&receipt.explicit_intent_code)
    {
        return Err("m4c04_owner_writeback_reference_invalid".to_string());
    }

    match receipt.status.as_str() {
        "PENDING"
            if receipt.scrubbed_owner_receipt_ref.is_none() && receipt.error_code.is_none() =>
        {
            Ok(())
        }
        "SUCCEEDED"
            if receipt
                .scrubbed_owner_receipt_ref
                .as_deref()
                .is_some_and(m4_is_opaque_reference)
                && receipt.error_code.is_none() =>
        {
            Ok(())
        }
        "FAILED"
            if receipt
                .scrubbed_owner_receipt_ref
                .as_deref()
                .is_some_and(m4_is_opaque_reference)
                && receipt.error_code.as_deref().is_some_and(m4c04_is_code) =>
        {
            Ok(())
        }
        _ => Err("m4c04_owner_writeback_state_invalid".to_string()),
    }
}

fn validate_m4c04_source_link(source_ref: &M4SourceLinkRead) -> bool {
    matches!(
        source_ref.link_kind.as_str(),
        "INTERNAL_ROUTE" | "HANDOFF_REF" | "OWNER_COMMAND_REF"
    ) && m4c04_is_safe_typed_ref(&source_ref.source_owner_ref)
        && m4c04_is_safe_typed_ref(&source_ref.object_type)
        && m4c04_is_safe_typed_ref(&source_ref.canonical_source_object_id)
        && m4_is_opaque_reference(&source_ref.opaque_route_ref)
}

fn m4c04_optional_utc_key(
    value: Option<&str>,
) -> Result<Option<crate::m4_secretary_domain::M4UtcSortKey>, String> {
    value
        .map(|value| {
            m4_parse_rfc3339_utc_key(value).ok_or_else(|| "m4c04_timestamp_invalid".to_string())
        })
        .transpose()
}

fn m4c04_has_deterministic_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(m4c04_is_lower_hex_digest)
}

fn m4c04_is_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'_'))
}

fn m4c04_is_safe_typed_ref(value: &str) -> bool {
    m4c04_is_reference_text(value) && !m4c04_looks_like_raw_reference(value)
}

fn m4c04_is_typed_reference(value: &str) -> bool {
    m4c04_is_reference_text(value)
        && (m4_is_opaque_reference(value) || m4c04_is_m4_deterministic_id(value))
}

fn m4c04_is_m4_deterministic_id(value: &str) -> bool {
    let Some((prefix, digest)) = value.split_once(':') else {
        return false;
    };
    matches!(
        prefix,
        "source"
            | "source-event"
            | "inbox"
            | "open-loop"
            | "personal-action"
            | "notification"
            | "reminder"
            | "decision-projection"
    ) && m4c04_is_lower_hex_digest(digest)
}

fn m4c04_is_title(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.is_empty()
        && value.chars().count() <= 160
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.starts_with('/')
        && !value.contains('\\')
        && !lower.starts_with("http://")
        && !lower.starts_with("https://")
}

fn m4c04_is_reference_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn m4c04_is_canonical_u64(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
        && value.parse::<u64>().is_ok()
}

fn m4c04_is_iana_timezone(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.contains('/')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
        })
}

fn m4c04_is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn m4c04_looks_like_raw_reference(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.contains('@')
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains("/./")
        || value.contains("/../")
        || lower.contains("://")
        || [
            "password",
            "credential",
            "api_key",
            "apikey",
            "access_token",
            "refresh_token",
            "bearer",
        ]
        .into_iter()
        .any(|marker| lower.contains(marker))
}

pub(crate) fn m4_priority_reason_text(code: &str) -> Result<&'static str, String> {
    match code {
        "EXTERNAL_COMMITMENT_OR_TIME_CRITICAL" => Ok("外部承诺或时间紧迫"),
        "USER_DECISION_OR_BLOCKER" => Ok("需要你决定或来源已受阻"),
        "ACTIVE_CHANGED_ATTENTION" => Ok("当前需要关注或刚有重要变化"),
        "CARRIED_OVER" => Ok("此前未闭环，继续关注"),
        "INFORMATIONAL" => Ok("来源信息，当前无需行动"),
        _ => Err("m4_priority_reason_unknown".to_string()),
    }
}

pub(crate) fn sort_m4_inbox_items(items: &mut [M4InboxItemRead]) -> Result<(), String> {
    validate_read_timestamps(items.iter().map(|item| {
        (
            item.due_at_utc.as_deref(),
            item.last_source_change_at_utc.as_str(),
        )
    }))?;
    items.sort_by(|left, right| {
        compare_attention_order(
            left.priority_rank,
            left.due_at_utc.as_deref(),
            &left.last_source_change_at_utc,
            &left.source_owner_ref,
            &left.source_link.canonical_source_object_id,
            &left.inbox_item_id,
            right.priority_rank,
            right.due_at_utc.as_deref(),
            &right.last_source_change_at_utc,
            &right.source_owner_ref,
            &right.source_link.canonical_source_object_id,
            &right.inbox_item_id,
        )
    });
    Ok(())
}

pub(crate) fn sort_m4_open_loops(items: &mut [M4OpenLoopRead]) -> Result<(), String> {
    validate_read_timestamps(items.iter().map(|item| {
        (
            item.due_at_utc.as_deref(),
            item.last_source_change_at_utc.as_str(),
        )
    }))?;
    items.sort_by(|left, right| {
        compare_attention_order(
            left.priority_rank,
            left.due_at_utc.as_deref(),
            &left.last_source_change_at_utc,
            &left.source_owner_ref,
            &left.source_link.canonical_source_object_id,
            &left.open_loop_id,
            right.priority_rank,
            right.due_at_utc.as_deref(),
            &right.last_source_change_at_utc,
            &right.source_owner_ref,
            &right.source_link.canonical_source_object_id,
            &right.open_loop_id,
        )
    });
    Ok(())
}

fn validate_read_timestamps<'a>(
    values: impl Iterator<Item = (Option<&'a str>, &'a str)>,
) -> Result<(), String> {
    for (due_at, last_change_at) in values {
        if due_at.is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
            || m4_parse_rfc3339_utc_key(last_change_at).is_none()
        {
            return Err("m4_read_model_timestamp_invalid".to_string());
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compare_attention_order(
    left_rank: i64,
    left_due: Option<&str>,
    left_last_change: &str,
    left_owner: &str,
    left_source_object_id: &str,
    left_object_id: &str,
    right_rank: i64,
    right_due: Option<&str>,
    right_last_change: &str,
    right_owner: &str,
    right_source_object_id: &str,
    right_object_id: &str,
) -> Ordering {
    left_rank
        .cmp(&right_rank)
        .then_with(|| match (left_due, right_due) {
            (Some(left), Some(right)) => m4_parse_rfc3339_utc_key(left)
                .expect("validated M4 due timestamp")
                .cmp(&m4_parse_rfc3339_utc_key(right).expect("validated M4 due timestamp")),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
        // Most recent source change first.
        .then_with(|| {
            m4_parse_rfc3339_utc_key(right_last_change)
                .expect("validated M4 source timestamp")
                .cmp(
                    &m4_parse_rfc3339_utc_key(left_last_change)
                        .expect("validated M4 source timestamp"),
                )
        })
        .then_with(|| left_owner.cmp(right_owner))
        .then_with(|| left_source_object_id.cmp(right_source_object_id))
        .then_with(|| left_object_id.cmp(right_object_id))
}

// -------------------------------------------------------------------------
// M4C08 legacy read compatibility: shadow/parity only, never a write path.
// -------------------------------------------------------------------------

pub(crate) const M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION: &str =
    "syn.m4.secretary.legacy-read-compatibility.v1";
pub(crate) const M4_LEGACY_READ_PARITY_MATRIX_VERSION: &str = "syn.m4.legacy-read-parity/v1";
const M4_LEGACY_READ_COMPATIBILITY_MODE: &str = "M4_PRIMARY_LEGACY_READ_ONLY_FALLBACK";
const M4_LEGACY_READ_ROLLBACK_MODE: &str = "GUARDED_LEGACY_READ_ONLY";
const M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE: &str = "SOURCE_REF_AND_DEDUPE_CANDIDATE_ONLY";
const M4_LEGACY_READ_WRITE_AUTHORITY_NONE: &str = "NONE";

/// The C08 inventory is deliberately closed. These values name only former
/// read surfaces; none creates a source adapter or owner-command port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum M4LegacyReadSourceKind {
    SecretaryReadModelDeterministicSummary,
    RightRailNotificationAndTodoProjection,
    RuntimeAttentionProjection,
    ReactPendingActionVisibility,
    MemoryDailyInboxCandidate,
}

impl M4LegacyReadSourceKind {
    const ALL: [Self; 5] = [
        Self::SecretaryReadModelDeterministicSummary,
        Self::RightRailNotificationAndTodoProjection,
        Self::RuntimeAttentionProjection,
        Self::ReactPendingActionVisibility,
        Self::MemoryDailyInboxCandidate,
    ];

    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::SecretaryReadModelDeterministicSummary => {
                "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY"
            }
            Self::RightRailNotificationAndTodoProjection => {
                "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION"
            }
            Self::RuntimeAttentionProjection => "RUNTIME_ATTENTION_PROJECTION",
            Self::ReactPendingActionVisibility => "REACT_PENDING_ACTION_VISIBILITY",
            Self::MemoryDailyInboxCandidate => "MEMORY_DAILY_INBOX_CANDIDATE",
        }
    }
}

/// Machine-readable inventory. Its fields are constants, never a route,
/// callback, raw payload, or mutable legacy handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadSourceInventoryEntry {
    pub(crate) legacy_source_kind: String,
    pub(crate) compatibility_role: String,
    pub(crate) write_authority: String,
}

pub(crate) fn m4_legacy_read_source_inventory() -> Vec<M4LegacyReadSourceInventoryEntry> {
    M4LegacyReadSourceKind::ALL
        .into_iter()
        .map(|legacy_source_kind| M4LegacyReadSourceInventoryEntry {
            legacy_source_kind: legacy_source_kind.code().to_string(),
            compatibility_role: M4_LEGACY_READ_SOURCE_REF_ONLY_ROLE.to_string(),
            write_authority: M4_LEGACY_READ_WRITE_AUTHORITY_NONE.to_string(),
        })
        .collect()
}

/// The public C08 command receives no renderer data. Its fixed, server-owned
/// inventory intentionally carries no join tuple, so each ordinary-product
/// candidate stays report-only quarantine until an internal adapter supplies
/// an exact typed source tuple.
pub(crate) fn m4_legacy_read_inventory_only_candidates() -> Vec<M4LegacyReadCandidate> {
    M4LegacyReadSourceKind::ALL
        .into_iter()
        .map(|legacy_source_kind| M4LegacyReadCandidate {
            legacy_source_kind,
            legacy_item_ref: None,
            source_owner_ref: None,
            scope_ref: None,
            source_type: None,
            canonical_source_object_id: None,
            source_revision: None,
            source_owner_watermark: None,
            source_link: None,
            source_status_code: None,
            priority_reason_code: None,
            scope_source_watermark: None,
        })
        .collect()
}

/// A scrubbed, non-authoritative observation from one legacy read surface.
/// Missing fields cause quarantine rather than reconstruction from frontend
/// cache, labels, routes, or executable pending-action data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4LegacyReadCandidate {
    pub(crate) legacy_source_kind: M4LegacyReadSourceKind,
    pub(crate) legacy_item_ref: Option<String>,
    pub(crate) source_owner_ref: Option<String>,
    pub(crate) scope_ref: Option<String>,
    pub(crate) source_type: Option<String>,
    pub(crate) canonical_source_object_id: Option<String>,
    pub(crate) source_revision: Option<u64>,
    pub(crate) source_owner_watermark: Option<String>,
    pub(crate) source_link: Option<M4SourceLinkRead>,
    pub(crate) source_status_code: Option<String>,
    pub(crate) priority_reason_code: Option<String>,
    pub(crate) scope_source_watermark: Option<String>,
}

/// Exact current-source data re-read by the repository before C08 permits a
/// compatibility item to appear. It is canonical M4 source fact, not legacy
/// truth copied into a second store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyCanonicalSourceRead {
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: u64,
    pub(crate) source_owner_watermark: String,
    pub(crate) source_link: M4SourceLinkRead,
    pub(crate) source_status_code: String,
    pub(crate) priority_reason_code: String,
}

/// One read transaction's canonical source set plus its scope watermark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4LegacyCanonicalReadSnapshot {
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) sources: Vec<M4LegacyCanonicalSourceRead>,
}

/// A mechanically compared legacy candidate. Legacy values are never echoed;
/// only safe item refs, canonical values, and parity booleans leave this
/// boundary, so it cannot become a second fact store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadParityRow {
    pub(crate) legacy_source_kind: String,
    pub(crate) legacy_item_ref: Option<String>,
    pub(crate) disposition: String,
    pub(crate) reason_code: Option<String>,
    pub(crate) canonical_source: Option<M4LegacyCanonicalSourceRead>,
    pub(crate) canonical_scope_source_watermark: Option<String>,
    pub(crate) source_matches: bool,
    pub(crate) status_matches: bool,
    pub(crate) priority_reason_matches: bool,
    pub(crate) source_owner_watermark_matches: bool,
    pub(crate) scope_source_watermark_matches: bool,
    pub(crate) dedupe_key: Option<String>,
    pub(crate) dedupe_disposition: String,
}

/// Versioned output for shadow comparison and guarded rollback display.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadCompatibilityReport {
    pub(crate) schema_version: String,
    pub(crate) parity_matrix_version: String,
    pub(crate) mode: String,
    pub(crate) rollback_mode: String,
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) inventory: Vec<M4LegacyReadSourceInventoryEntry>,
    pub(crate) rows: Vec<M4LegacyReadParityRow>,
}

// The command boundary uses canonical decimal strings for every u64.  This
// keeps a legacy candidate's exact revision intact across JavaScript rather
// than allowing a renderer number to round it before the next read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadSourceLinkDto {
    pub(crate) link_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) expected_source_revision: String,
    pub(crate) opaque_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyCanonicalSourceReadDto {
    pub(crate) source_owner_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) source_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) source_revision: String,
    pub(crate) source_owner_watermark: String,
    pub(crate) source_link: M4LegacyReadSourceLinkDto,
    pub(crate) source_status_code: String,
    pub(crate) priority_reason_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadParityRowDto {
    pub(crate) legacy_source_kind: String,
    pub(crate) legacy_item_ref: Option<String>,
    pub(crate) disposition: String,
    pub(crate) reason_code: Option<String>,
    pub(crate) canonical_source: Option<M4LegacyCanonicalSourceReadDto>,
    pub(crate) canonical_scope_source_watermark: Option<String>,
    pub(crate) source_matches: bool,
    pub(crate) status_matches: bool,
    pub(crate) priority_reason_matches: bool,
    pub(crate) source_owner_watermark_matches: bool,
    pub(crate) scope_source_watermark_matches: bool,
    pub(crate) dedupe_key: Option<String>,
    pub(crate) dedupe_disposition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4LegacyReadCompatibilityReportDto {
    pub(crate) schema_version: String,
    pub(crate) parity_matrix_version: String,
    pub(crate) mode: String,
    pub(crate) rollback_mode: String,
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) inventory: Vec<M4LegacyReadSourceInventoryEntry>,
    pub(crate) rows: Vec<M4LegacyReadParityRowDto>,
}

/// The compatibility command is a read envelope, never an owner-command or
/// migration result. `UNAVAILABLE` intentionally carries no partial report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M4LegacyReadCompatibilityReportEnvelope {
    Ready {
        report: M4LegacyReadCompatibilityReportDto,
    },
    Unavailable {
        reason: String,
    },
}

impl M4LegacyReadCompatibilityReportEnvelope {
    pub(crate) fn ready(report: M4LegacyReadCompatibilityReport) -> Self {
        Self::Ready {
            report: report.into(),
        }
    }

    pub(crate) fn unavailable(reason: &str) -> Self {
        Self::Unavailable {
            reason: reason.to_string(),
        }
    }
}

impl From<M4LegacyReadCompatibilityReport> for M4LegacyReadCompatibilityReportDto {
    fn from(report: M4LegacyReadCompatibilityReport) -> Self {
        Self {
            schema_version: report.schema_version,
            parity_matrix_version: report.parity_matrix_version,
            mode: report.mode,
            rollback_mode: report.rollback_mode,
            scope_ref: report.scope_ref,
            scope_source_watermark: report.scope_source_watermark,
            inventory: report.inventory,
            rows: report.rows.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<M4LegacyReadParityRow> for M4LegacyReadParityRowDto {
    fn from(row: M4LegacyReadParityRow) -> Self {
        Self {
            legacy_source_kind: row.legacy_source_kind,
            legacy_item_ref: row.legacy_item_ref,
            disposition: row.disposition,
            reason_code: row.reason_code,
            canonical_source: row.canonical_source.map(Into::into),
            canonical_scope_source_watermark: row.canonical_scope_source_watermark,
            source_matches: row.source_matches,
            status_matches: row.status_matches,
            priority_reason_matches: row.priority_reason_matches,
            source_owner_watermark_matches: row.source_owner_watermark_matches,
            scope_source_watermark_matches: row.scope_source_watermark_matches,
            dedupe_key: row.dedupe_key,
            dedupe_disposition: row.dedupe_disposition,
        }
    }
}

impl From<M4LegacyCanonicalSourceRead> for M4LegacyCanonicalSourceReadDto {
    fn from(source: M4LegacyCanonicalSourceRead) -> Self {
        Self {
            source_owner_ref: source.source_owner_ref,
            scope_ref: source.scope_ref,
            source_type: source.source_type,
            canonical_source_object_id: source.canonical_source_object_id,
            source_revision: source.source_revision.to_string(),
            source_owner_watermark: source.source_owner_watermark,
            source_link: source.source_link.into(),
            source_status_code: source.source_status_code,
            priority_reason_code: source.priority_reason_code,
        }
    }
}

impl From<M4SourceLinkRead> for M4LegacyReadSourceLinkDto {
    fn from(link: M4SourceLinkRead) -> Self {
        Self {
            link_kind: link.link_kind,
            source_owner_ref: link.source_owner_ref,
            object_type: link.object_type,
            canonical_source_object_id: link.canonical_source_object_id,
            expected_source_revision: link.expected_source_revision.to_string(),
            opaque_route_ref: link.opaque_route_ref,
        }
    }
}

/// Build C08's report from an atomically re-read canonical snapshot. Missing,
/// stale, unknown, or divergent legacy candidates stay quarantined in the
/// returned read result; this function has no storage or owner side effects.
pub(crate) fn build_m4_legacy_shadow_parity_report(
    canonical_snapshot: &M4LegacyCanonicalReadSnapshot,
    legacy_candidates: &[M4LegacyReadCandidate],
) -> Result<M4LegacyReadCompatibilityReport, String> {
    validate_m4c08_canonical_snapshot(canonical_snapshot)?;

    let mut ordered_candidates = legacy_candidates.to_vec();
    ordered_candidates.sort_by(|left, right| {
        (
            left.legacy_source_kind.code(),
            left.legacy_item_ref.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.legacy_source_kind.code(),
                right.legacy_item_ref.as_deref().unwrap_or(""),
            ))
    });

    let mut rows = ordered_candidates
        .iter()
        .map(|candidate| m4c08_build_parity_row(canonical_snapshot, candidate))
        .collect::<Vec<_>>();
    m4c08_quarantine_ambiguous_legacy_identities(&mut rows)?;
    let mut seen_dedupe_keys = BTreeSet::new();
    for row in &mut rows {
        if row.disposition != "PARITY" {
            row.dedupe_disposition = "NOT_ELIGIBLE".to_string();
            continue;
        }
        let canonical_source = row
            .canonical_source
            .as_ref()
            .ok_or_else(|| "m4c08_parity_row_canonical_missing".to_string())?;
        let dedupe_key = m4c08_dedupe_key(canonical_source)?;
        row.dedupe_disposition = if seen_dedupe_keys.insert(dedupe_key.clone()) {
            "PRIMARY".to_string()
        } else {
            "DUPLICATE_DISPLAY_ONLY".to_string()
        };
        row.dedupe_key = Some(dedupe_key);
    }

    let report = M4LegacyReadCompatibilityReport {
        schema_version: M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION.to_string(),
        parity_matrix_version: M4_LEGACY_READ_PARITY_MATRIX_VERSION.to_string(),
        mode: M4_LEGACY_READ_COMPATIBILITY_MODE.to_string(),
        rollback_mode: M4_LEGACY_READ_ROLLBACK_MODE.to_string(),
        scope_ref: canonical_snapshot.scope_ref.clone(),
        scope_source_watermark: canonical_snapshot.scope_source_watermark.clone(),
        inventory: m4_legacy_read_source_inventory(),
        rows,
    };
    validate_m4c08_legacy_read_compatibility_report(&report)?;
    Ok(report)
}

/// A legacy identity is the closed pair `(legacy_source_kind, legacy_item_ref)`.
/// Once that identity exactly joins more than one distinct canonical source,
/// every row for it becomes diagnostic-only quarantine before global canonical
/// de-duplication can choose a display primary.
fn m4c08_quarantine_ambiguous_legacy_identities(
    rows: &mut [M4LegacyReadParityRow],
) -> Result<(), String> {
    let mut canonical_by_legacy_identity =
        BTreeMap::<(String, String), (String, M4LegacyCanonicalSourceRead)>::new();
    let mut ambiguous_legacy_identities = BTreeSet::new();

    for row in rows.iter() {
        if row.disposition != "PARITY" {
            continue;
        }
        let legacy_item_ref = row
            .legacy_item_ref
            .as_ref()
            .ok_or_else(|| "m4c08_parity_row_legacy_identity_missing".to_string())?;
        let canonical_source = row
            .canonical_source
            .as_ref()
            .ok_or_else(|| "m4c08_parity_row_canonical_missing".to_string())?;
        let canonical_dedupe_key = m4c08_dedupe_key(canonical_source)?;
        let legacy_identity = (row.legacy_source_kind.clone(), legacy_item_ref.clone());
        if let Some((known_dedupe_key, known_canonical_source)) =
            canonical_by_legacy_identity.get(&legacy_identity)
        {
            if known_dedupe_key != &canonical_dedupe_key
                || known_canonical_source != canonical_source
            {
                ambiguous_legacy_identities.insert(legacy_identity);
            }
        } else {
            canonical_by_legacy_identity.insert(
                legacy_identity,
                (canonical_dedupe_key, canonical_source.clone()),
            );
        }
    }

    for row in rows {
        if row.disposition != "PARITY" {
            continue;
        }
        let Some(legacy_item_ref) = row.legacy_item_ref.as_ref() else {
            continue;
        };
        if ambiguous_legacy_identities
            .contains(&(row.legacy_source_kind.clone(), legacy_item_ref.clone()))
        {
            m4c08_quarantine_legacy_identity_ambiguity(row);
        }
    }
    Ok(())
}

fn m4c08_quarantine_legacy_identity_ambiguity(row: &mut M4LegacyReadParityRow) {
    row.disposition = "QUARANTINED".to_string();
    row.reason_code = Some("M4C08_LEGACY_IDENTITY_AMBIGUOUS".to_string());
    row.canonical_source = None;
    row.canonical_scope_source_watermark = None;
    row.source_matches = false;
    row.status_matches = false;
    row.priority_reason_matches = false;
    row.source_owner_watermark_matches = false;
    row.scope_source_watermark_matches = false;
    row.dedupe_key = None;
    row.dedupe_disposition = "NOT_ELIGIBLE".to_string();
}

/// The rollback display is a filtered read-only view. It cannot resurrect an
/// owner fact, modify M4 evidence, or physically remove legacy material.
pub(crate) fn guarded_m4_legacy_read_only_fallback(
    report: &M4LegacyReadCompatibilityReport,
) -> Result<Vec<M4LegacyReadParityRow>, String> {
    validate_m4c08_legacy_read_compatibility_report(report)?;
    if report.rollback_mode != M4_LEGACY_READ_ROLLBACK_MODE {
        return Err("m4c08_rollback_mode_invalid".to_string());
    }
    Ok(report
        .rows
        .iter()
        .filter(|row| row.disposition == "PARITY" && row.dedupe_disposition == "PRIMARY")
        .cloned()
        .collect())
}

fn m4c08_build_parity_row(
    canonical_snapshot: &M4LegacyCanonicalReadSnapshot,
    candidate: &M4LegacyReadCandidate,
) -> M4LegacyReadParityRow {
    let legacy_item_ref = candidate
        .legacy_item_ref
        .as_deref()
        .filter(|value| m4_is_opaque_reference(value))
        .map(str::to_string);
    let mut row = M4LegacyReadParityRow {
        legacy_source_kind: candidate.legacy_source_kind.code().to_string(),
        legacy_item_ref,
        disposition: "QUARANTINED".to_string(),
        reason_code: None,
        canonical_source: None,
        canonical_scope_source_watermark: None,
        source_matches: false,
        status_matches: false,
        priority_reason_matches: false,
        source_owner_watermark_matches: false,
        scope_source_watermark_matches: false,
        dedupe_key: None,
        dedupe_disposition: "NOT_ELIGIBLE".to_string(),
    };

    if !m4c08_candidate_is_well_formed(candidate) {
        row.reason_code = Some("M4C08_LEGACY_CANDIDATE_INVALID".to_string());
        return row;
    }
    if candidate.scope_ref.as_deref() != Some(canonical_snapshot.scope_ref.as_str()) {
        row.reason_code = Some("M4C08_SCOPE_MISMATCH".to_string());
        return row;
    }

    let mut canonical_sources = canonical_snapshot.sources.iter().filter(|source| {
        candidate.source_owner_ref.as_deref() == Some(source.source_owner_ref.as_str())
            && candidate.scope_ref.as_deref() == Some(source.scope_ref.as_str())
            && candidate.source_type.as_deref() == Some(source.source_type.as_str())
            && candidate.canonical_source_object_id.as_deref()
                == Some(source.canonical_source_object_id.as_str())
    });
    let Some(canonical_source) = canonical_sources.next() else {
        row.reason_code = Some("M4C08_CANONICAL_SOURCE_NOT_FOUND".to_string());
        return row;
    };
    if canonical_sources.next().is_some() {
        row.reason_code = Some("M4C08_CANONICAL_SOURCE_AMBIGUOUS".to_string());
        return row;
    }

    row.canonical_source = Some(canonical_source.clone());
    row.canonical_scope_source_watermark = Some(canonical_snapshot.scope_source_watermark.clone());
    row.source_matches = candidate.source_owner_ref.as_deref()
        == Some(canonical_source.source_owner_ref.as_str())
        && candidate.scope_ref.as_deref() == Some(canonical_source.scope_ref.as_str())
        && candidate.source_type.as_deref() == Some(canonical_source.source_type.as_str())
        && candidate.canonical_source_object_id.as_deref()
            == Some(canonical_source.canonical_source_object_id.as_str())
        && candidate.source_revision == Some(canonical_source.source_revision)
        && candidate.source_link.as_ref() == Some(&canonical_source.source_link);
    row.status_matches = candidate.source_status_code.as_deref()
        == Some(canonical_source.source_status_code.as_str());
    row.priority_reason_matches = candidate.priority_reason_code.as_deref()
        == Some(canonical_source.priority_reason_code.as_str());
    row.source_owner_watermark_matches = candidate.source_owner_watermark.as_deref()
        == Some(canonical_source.source_owner_watermark.as_str());
    row.scope_source_watermark_matches = candidate.scope_source_watermark.as_deref()
        == Some(canonical_snapshot.scope_source_watermark.as_str());

    row.reason_code = if !row.source_matches {
        match candidate.source_revision {
            Some(revision) if revision < canonical_source.source_revision => {
                Some("M4C08_STALE_SOURCE_REVISION".to_string())
            }
            Some(revision) if revision > canonical_source.source_revision => {
                Some("M4C08_CANONICAL_SOURCE_REVISION_UNAVAILABLE".to_string())
            }
            _ => Some("M4C08_SOURCE_LINK_MISMATCH".to_string()),
        }
    } else if !row.source_owner_watermark_matches {
        Some("M4C08_SOURCE_OWNER_WATERMARK_MISMATCH".to_string())
    } else if !row.status_matches {
        Some("M4C08_PARITY_STATUS_MISMATCH".to_string())
    } else if !row.priority_reason_matches {
        Some("M4C08_PARITY_PRIORITY_REASON_MISMATCH".to_string())
    } else if !row.scope_source_watermark_matches {
        Some("M4C08_SCOPE_WATERMARK_MISMATCH".to_string())
    } else {
        None
    };
    if row.reason_code.is_none() {
        row.disposition = "PARITY".to_string();
    }
    row
}

fn validate_m4c08_canonical_snapshot(
    canonical_snapshot: &M4LegacyCanonicalReadSnapshot,
) -> Result<(), String> {
    if !m4c08_is_safe_reference(&canonical_snapshot.scope_ref)
        || !m4c04_is_lower_hex_digest(&canonical_snapshot.scope_source_watermark)
    {
        return Err("m4c08_canonical_snapshot_scope_invalid".to_string());
    }
    for source in &canonical_snapshot.sources {
        validate_m4c08_canonical_source(source, &canonical_snapshot.scope_ref)?;
    }
    Ok(())
}

fn validate_m4c08_canonical_source(
    source: &M4LegacyCanonicalSourceRead,
    expected_scope_ref: &str,
) -> Result<(), String> {
    if !m4c08_is_safe_reference(&source.source_owner_ref)
        || source.scope_ref != expected_scope_ref
        || source.source_type != M4_WORKFLOW_ATTENTION_SOURCE_TYPE
        || !m4c08_is_safe_reference(&source.canonical_source_object_id)
        || !m4_is_opaque_reference(&source.source_owner_watermark)
        || !m4c08_source_link_matches(
            &source.source_link,
            &source.source_owner_ref,
            &source.canonical_source_object_id,
            source.source_revision,
        )
        || !m4c08_is_source_status(&source.source_status_code)
        || m4_priority_reason_text(&source.priority_reason_code).is_err()
    {
        return Err("m4c08_canonical_source_invalid".to_string());
    }
    Ok(())
}

fn m4c08_candidate_is_well_formed(candidate: &M4LegacyReadCandidate) -> bool {
    let (
        Some(legacy_item_ref),
        Some(source_owner_ref),
        Some(scope_ref),
        Some(source_type),
        Some(canonical_source_object_id),
        Some(source_revision),
        Some(source_owner_watermark),
        Some(source_link),
        Some(source_status_code),
        Some(priority_reason_code),
        Some(scope_source_watermark),
    ) = (
        candidate.legacy_item_ref.as_deref(),
        candidate.source_owner_ref.as_deref(),
        candidate.scope_ref.as_deref(),
        candidate.source_type.as_deref(),
        candidate.canonical_source_object_id.as_deref(),
        candidate.source_revision,
        candidate.source_owner_watermark.as_deref(),
        candidate.source_link.as_ref(),
        candidate.source_status_code.as_deref(),
        candidate.priority_reason_code.as_deref(),
        candidate.scope_source_watermark.as_deref(),
    )
    else {
        return false;
    };

    m4_is_opaque_reference(legacy_item_ref)
        && m4c08_is_safe_reference(source_owner_ref)
        && m4c08_is_safe_reference(scope_ref)
        && source_type == M4_WORKFLOW_ATTENTION_SOURCE_TYPE
        && m4c08_is_safe_reference(canonical_source_object_id)
        && m4_is_opaque_reference(source_owner_watermark)
        && m4c08_source_link_matches(
            source_link,
            source_owner_ref,
            canonical_source_object_id,
            source_revision,
        )
        && m4c08_is_source_status(source_status_code)
        && m4_priority_reason_text(priority_reason_code).is_ok()
        && m4c04_is_lower_hex_digest(scope_source_watermark)
}

fn m4c08_source_link_matches(
    source_link: &M4SourceLinkRead,
    source_owner_ref: &str,
    canonical_source_object_id: &str,
    source_revision: u64,
) -> bool {
    source_link.link_kind == "INTERNAL_ROUTE"
        && source_link.source_owner_ref == source_owner_ref
        && source_link.object_type == M4_WORKFLOW_ATTENTION_OBJECT_TYPE
        && source_link.canonical_source_object_id == canonical_source_object_id
        && source_link.expected_source_revision == source_revision
        && m4_is_opaque_reference(&source_link.opaque_route_ref)
}

fn m4c08_is_source_status(value: &str) -> bool {
    matches!(
        value,
        "OPEN"
            | "BLOCKED"
            | "WAITING_USER"
            | "INFORMATIONAL"
            | "COMPLETED"
            | "CANCELLED"
            | "EXPIRED"
    )
}

fn m4c08_is_safe_reference(value: &str) -> bool {
    m4c04_is_reference_text(value)
        && !m4c04_looks_like_raw_reference(value)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn m4c08_dedupe_key(source: &M4LegacyCanonicalSourceRead) -> Result<String, String> {
    let revision = source.source_revision.to_string();
    crate::m4_secretary_domain::m4_internal_id(
        "legacy-dedupe:",
        "syn.m4.legacy-read-dedupe/v1",
        &[
            &source.source_owner_ref,
            &source.scope_ref,
            &source.source_type,
            &source.canonical_source_object_id,
            &revision,
            &source.source_owner_watermark,
        ],
    )
}

fn validate_m4c08_legacy_read_compatibility_report(
    report: &M4LegacyReadCompatibilityReport,
) -> Result<(), String> {
    if report.schema_version != M4_LEGACY_READ_COMPATIBILITY_SCHEMA_VERSION
        || report.parity_matrix_version != M4_LEGACY_READ_PARITY_MATRIX_VERSION
        || report.mode != M4_LEGACY_READ_COMPATIBILITY_MODE
        || report.rollback_mode != M4_LEGACY_READ_ROLLBACK_MODE
        || !m4c08_is_safe_reference(&report.scope_ref)
        || !m4c04_is_lower_hex_digest(&report.scope_source_watermark)
        || report.inventory != m4_legacy_read_source_inventory()
    {
        return Err("m4c08_compatibility_report_invalid".to_string());
    }
    let mut canonical_by_legacy_identity =
        BTreeMap::<(String, String), (String, M4LegacyCanonicalSourceRead)>::new();
    for row in &report.rows {
        if !M4LegacyReadSourceKind::ALL
            .iter()
            .any(|kind| kind.code() == row.legacy_source_kind)
            || row
                .legacy_item_ref
                .as_deref()
                .is_some_and(|value| !m4_is_opaque_reference(value))
        {
            return Err("m4c08_compatibility_report_row_invalid".to_string());
        }
        match row.disposition.as_str() {
            "PARITY"
                if row.reason_code.is_none()
                    && row.canonical_source.is_some()
                    && row.canonical_scope_source_watermark.as_deref()
                        == Some(report.scope_source_watermark.as_str())
                    && row.source_matches
                    && row.status_matches
                    && row.priority_reason_matches
                    && row.source_owner_watermark_matches
                    && row.scope_source_watermark_matches
                    && row.dedupe_key.as_deref().is_some_and(m4c08_is_dedupe_key)
                    && matches!(
                        row.dedupe_disposition.as_str(),
                        "PRIMARY" | "DUPLICATE_DISPLAY_ONLY"
                    ) => {}
            "QUARANTINED"
                if row.reason_code.as_deref().is_some_and(m4c08_is_reason_code)
                    && row.dedupe_key.is_none()
                    && row.dedupe_disposition == "NOT_ELIGIBLE" => {}
            _ => return Err("m4c08_compatibility_report_row_invalid".to_string()),
        }
        if let Some(canonical_source) = &row.canonical_source {
            validate_m4c08_canonical_source(canonical_source, &report.scope_ref)?;
        }
        if row.disposition == "PARITY" {
            let legacy_item_ref = row
                .legacy_item_ref
                .as_ref()
                .ok_or_else(|| "m4c08_compatibility_report_row_invalid".to_string())?;
            let canonical_source = row
                .canonical_source
                .as_ref()
                .ok_or_else(|| "m4c08_compatibility_report_row_invalid".to_string())?;
            let dedupe_key = row
                .dedupe_key
                .as_ref()
                .ok_or_else(|| "m4c08_compatibility_report_row_invalid".to_string())?;
            let legacy_identity = (row.legacy_source_kind.clone(), legacy_item_ref.clone());
            if let Some((known_dedupe_key, known_canonical_source)) =
                canonical_by_legacy_identity.get(&legacy_identity)
            {
                if known_dedupe_key != dedupe_key || known_canonical_source != canonical_source {
                    return Err("m4c08_compatibility_report_legacy_identity_ambiguous".to_string());
                }
            } else {
                canonical_by_legacy_identity.insert(
                    legacy_identity,
                    (dedupe_key.clone(), canonical_source.clone()),
                );
            }
        }
    }
    Ok(())
}

fn m4c08_is_dedupe_key(value: &str) -> bool {
    value
        .strip_prefix("legacy-dedupe:")
        .is_some_and(m4c04_is_lower_hex_digest)
}

fn m4c08_is_reason_code(value: &str) -> bool {
    matches!(
        value,
        "M4C08_LEGACY_CANDIDATE_INVALID"
            | "M4C08_SCOPE_MISMATCH"
            | "M4C08_CANONICAL_SOURCE_NOT_FOUND"
            | "M4C08_CANONICAL_SOURCE_AMBIGUOUS"
            | "M4C08_LEGACY_IDENTITY_AMBIGUOUS"
            | "M4C08_STALE_SOURCE_REVISION"
            | "M4C08_CANONICAL_SOURCE_REVISION_UNAVAILABLE"
            | "M4C08_SOURCE_LINK_MISMATCH"
            | "M4C08_SOURCE_OWNER_WATERMARK_MISMATCH"
            | "M4C08_PARITY_STATUS_MISMATCH"
            | "M4C08_PARITY_PRIORITY_REASON_MISMATCH"
            | "M4C08_SCOPE_WATERMARK_MISMATCH"
    )
}

// ===== M4C07 DailyBrief / DailyReport read protocol =======================
//
// This remains a read-only boundary.  It contains source-backed identifiers,
// hashes and scrubbed codes only; a daily read must never carry a transcript,
// prompt, provider response, memory artifact, secret, route payload, or any
// other content-bearing field.  The repository/service owns the query and
// its M4 priority/due/source-change order; this read boundary validates that
// order's safe source refs without reordering it.

pub(crate) const M4_SECRETARY_DAILY_SCHEMA_VERSION: &str = "syn.m4.secretary.daily.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SecretaryDailySchedulerRead {
    pub(crate) configuration_revision: String,
    pub(crate) iana_timezone: String,
    pub(crate) timezone_rules_version: String,
    pub(crate) current_daily_window_id: String,
    pub(crate) last_closed_daily_window_id: String,
    pub(crate) catch_up_pending_count: u64,
    pub(crate) pending_catch_up_receipt_refs: Vec<String>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SecretaryDailyBriefRead {
    pub(crate) daily_window_id: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) projector_version: String,
    pub(crate) ordered_item_refs: Vec<String>,
    pub(crate) generated_at_utc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SecretaryDailyReportRead {
    pub(crate) daily_report_id: String,
    pub(crate) daily_window_id: String,
    /// An unsigned 64-bit revision encoded as canonical base-10 ASCII so the
    /// renderer never loses precision through a JavaScript number.
    pub(crate) report_version: String,
    pub(crate) status: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) projector_version: String,
    pub(crate) ordered_item_refs: Vec<String>,
    pub(crate) supersedes_report_ref: Option<String>,
    pub(crate) generated_at_utc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct M4SecretarySchedulerRunRead {
    pub(crate) scheduler_run_id: String,
    pub(crate) configuration_revision: String,
    pub(crate) window_ref: String,
    pub(crate) scope_source_watermark_before: String,
    pub(crate) scope_source_watermark_after: String,
    pub(crate) admitted_material_event_count: u64,
    pub(crate) agent_turn_count: u64,
    pub(crate) model_invocation_count: u64,
    pub(crate) outcome_code: String,
    pub(crate) recorded_at_utc: Option<String>,
}

/// The frozen, server-owned daily read envelope consumed by the renderer.
///
/// `UNAVAILABLE` and `DISABLED` are separate variants instead of optional
/// flags so they can never be emitted together, and neither variant can leak
/// a partial ready payload.  Their only diagnostic is a scrubbed code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum M4SecretaryDailyReportEnvelope {
    Ready {
        schema_version: String,
        scheduler: M4SecretaryDailySchedulerRead,
        daily_brief: M4SecretaryDailyBriefRead,
        daily_report: M4SecretaryDailyReportRead,
        last_run: M4SecretarySchedulerRunRead,
        recovery_code: Option<String>,
    },
    Unavailable {
        schema_version: String,
        reason: String,
    },
    Disabled {
        schema_version: String,
        reason: String,
    },
}

/// Validate the M4C07 read payload without querying or mutating any state.
///
/// The ready payload carries a current-window brief and a last-closed-window
/// report.  A scheduler receipt belongs to that closed report, which is why
/// their window/ref/watermark bindings are checked here rather than trusted
/// from a renderer field.
pub(crate) fn validate_m4c07_daily_report_envelope(
    envelope: &M4SecretaryDailyReportEnvelope,
) -> Result<(), String> {
    match envelope {
        M4SecretaryDailyReportEnvelope::Ready {
            schema_version,
            scheduler,
            daily_brief,
            daily_report,
            last_run,
            recovery_code,
        } => {
            validate_m4c07_schema_version(schema_version)?;
            validate_m4c07_scheduler(scheduler)?;
            validate_m4c07_daily_brief(daily_brief)?;
            validate_m4c07_daily_report(daily_report)?;
            validate_m4c07_scheduler_run(last_run)?;
            if recovery_code
                .as_deref()
                .is_some_and(|code| !m4c07_is_scrubbed_code(code))
            {
                return Err("m4c07_daily_recovery_code_invalid".to_string());
            }

            if scheduler.current_daily_window_id != daily_brief.daily_window_id
                || scheduler.last_closed_daily_window_id != daily_report.daily_window_id
                || scheduler.current_daily_window_id == scheduler.last_closed_daily_window_id
                || last_run.configuration_revision != scheduler.configuration_revision
                || last_run.window_ref != daily_report.daily_window_id
                || last_run.scope_source_watermark_after != daily_report.scope_source_watermark
            {
                return Err("m4c07_daily_cross_object_binding_invalid".to_string());
            }
            Ok(())
        }
        M4SecretaryDailyReportEnvelope::Unavailable {
            schema_version,
            reason,
        }
        | M4SecretaryDailyReportEnvelope::Disabled {
            schema_version,
            reason,
        } => {
            validate_m4c07_schema_version(schema_version)?;
            if !m4c07_is_scrubbed_code(reason) {
                return Err("m4c07_daily_unavailable_reason_invalid".to_string());
            }
            Ok(())
        }
    }
}

fn validate_m4c07_schema_version(value: &str) -> Result<(), String> {
    if value == M4_SECRETARY_DAILY_SCHEMA_VERSION {
        Ok(())
    } else {
        Err("m4c07_daily_schema_version_invalid".to_string())
    }
}

fn validate_m4c07_scheduler(scheduler: &M4SecretaryDailySchedulerRead) -> Result<(), String> {
    if !m4c04_is_canonical_u64(&scheduler.configuration_revision)
        || !m4c04_is_iana_timezone(&scheduler.iana_timezone)
        || !m4c07_is_timezone_rules_version(&scheduler.timezone_rules_version)
        || !m4c07_is_daily_window_id(&scheduler.current_daily_window_id)
        || !m4c07_is_daily_window_id(&scheduler.last_closed_daily_window_id)
        || !m4c07_is_scheduler_status(&scheduler.status)
        || (scheduler.catch_up_pending_count == 0
            && !scheduler.pending_catch_up_receipt_refs.is_empty())
        || (scheduler.catch_up_pending_count > 0
            && scheduler.pending_catch_up_receipt_refs.is_empty())
    {
        return Err("m4c07_daily_scheduler_invalid".to_string());
    }
    let mut seen = BTreeSet::new();
    if scheduler.pending_catch_up_receipt_refs.iter().any(|value| {
        !m4c07_has_deterministic_id(value, "catch-up-truncation:") || !seen.insert(value)
    }) {
        return Err("m4c07_daily_scheduler_invalid".to_string());
    }
    Ok(())
}

fn validate_m4c07_daily_brief(brief: &M4SecretaryDailyBriefRead) -> Result<(), String> {
    if !m4c07_is_daily_window_id(&brief.daily_window_id)
        || !m4c04_is_lower_hex_digest(&brief.scope_source_watermark)
        || !m4c04_is_canonical_u64(&brief.projector_version)
        || brief
            .generated_at_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
    {
        return Err("m4c07_daily_brief_invalid".to_string());
    }
    validate_m4c07_ordered_item_refs(&brief.ordered_item_refs)
}

fn validate_m4c07_daily_report(report: &M4SecretaryDailyReportRead) -> Result<(), String> {
    if !m4c07_has_deterministic_id(&report.daily_report_id, "daily-report:")
        || !m4c07_is_daily_window_id(&report.daily_window_id)
        || !m4c04_is_canonical_u64(&report.report_version)
        || !matches!(
            report.status.as_str(),
            "GENERATED" | "SUPERSEDED" | "FAILED"
        )
        || !m4c04_is_lower_hex_digest(&report.scope_source_watermark)
        || !m4c04_is_canonical_u64(&report.projector_version)
        || report
            .supersedes_report_ref
            .as_deref()
            .is_some_and(|value| {
                !m4c07_has_deterministic_id(value, "daily-report:")
                    || value == report.daily_report_id
            })
        || report
            .generated_at_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
    {
        return Err("m4c07_daily_report_invalid".to_string());
    }
    validate_m4c07_ordered_item_refs(&report.ordered_item_refs)
}

fn validate_m4c07_scheduler_run(run: &M4SecretarySchedulerRunRead) -> Result<(), String> {
    if !m4c07_is_scheduler_run_id(&run.scheduler_run_id)
        || !m4c04_is_canonical_u64(&run.configuration_revision)
        || !m4c07_is_daily_window_id(&run.window_ref)
        || !m4c04_is_lower_hex_digest(&run.scope_source_watermark_before)
        || !m4c04_is_lower_hex_digest(&run.scope_source_watermark_after)
        || !m4c07_is_scrubbed_code(&run.outcome_code)
        || run
            .recorded_at_utc
            .as_deref()
            .is_some_and(|value| m4_parse_rfc3339_utc_key(value).is_none())
        || (run.admitted_material_event_count == 0
            && (run.agent_turn_count != 0 || run.model_invocation_count != 0))
    {
        return Err("m4c07_daily_scheduler_run_invalid".to_string());
    }
    Ok(())
}

fn validate_m4c07_ordered_item_refs(values: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !m4c07_is_source_backed_item_ref(value) || !seen.insert(value) {
            return Err("m4c07_daily_ordered_item_refs_invalid".to_string());
        }
    }
    Ok(())
}

fn m4c07_is_source_backed_item_ref(value: &str) -> bool {
    [
        "source:",
        "source-event:",
        "inbox:",
        "open-loop:",
        "personal-action:",
        "decision-projection:",
    ]
    .into_iter()
    .any(|prefix| m4c07_has_deterministic_id(value, prefix))
}

fn m4c07_is_daily_window_id(value: &str) -> bool {
    m4c07_has_deterministic_id(value, "daily-window:")
}

fn m4c07_is_scheduler_run_id(value: &str) -> bool {
    m4c07_has_deterministic_id(value, "scheduler-run:")
        || (value.starts_with("scheduler-run:") && m4_is_opaque_reference(value))
}

fn m4c07_has_deterministic_id(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(prefix)
        .is_some_and(m4c04_is_lower_hex_digest)
}

fn m4c07_is_timezone_rules_version(value: &str) -> bool {
    m4c07_has_deterministic_id(value, "timezone-rules:")
}

fn m4c07_is_scheduler_status(value: &str) -> bool {
    m4c07_is_scrubbed_code(value) && !matches!(value, "UNAVAILABLE" | "DISABLED")
}

fn m4c07_is_scrubbed_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        && ![
            "RAW",
            "TRANSCRIPT",
            "PROMPT",
            "PROVIDER",
            "SECRET",
            "CREDENTIAL",
            "TOKEN",
            "PASSWORD",
            "CALLBACK",
            "URL",
            "PATH",
            "BODY",
        ]
        .into_iter()
        .any(|forbidden| value.contains(forbidden))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(id: &str, rank: i64, due: Option<&str>, last: &str) -> M4InboxItemRead {
        M4InboxItemRead {
            inbox_item_id: id.to_string(),
            source_identity_key: format!("source-{id}"),
            source_owner_ref: "owner-a".to_string(),
            source_link: M4SourceLinkRead {
                link_kind: "INTERNAL_ROUTE".to_string(),
                source_owner_ref: "owner-a".to_string(),
                object_type: "workflow_attention".to_string(),
                canonical_source_object_id: id.to_string(),
                expected_source_revision: 1,
                opaque_route_ref: "route".to_string(),
            },
            current_source_status: "OPEN".to_string(),
            status: "NEW".to_string(),
            priority_rank: rank,
            priority_reason_code: "INFORMATIONAL".to_string(),
            priority_reason_text: "来源信息，当前无需行动".to_string(),
            due_at_utc: due.map(str::to_string),
            received_at_utc: last.to_string(),
            last_source_change_at_utc: last.to_string(),
            scrubbed_summary_ref: "summary".to_string(),
            sensitivity: "SCRUBBED_INTERNAL_REF_ONLY".to_string(),
            revision: 1,
        }
    }

    #[test]
    fn m4c03_read_order_is_rank_due_null_last_then_change_desc_and_ids() {
        let mut items = vec![
            fixture("null", 0, None, "2026-08-10T09:00:00Z"),
            fixture(
                "later",
                0,
                Some("2026-08-10T10:00:00Z"),
                "2026-08-10T12:00:00Z",
            ),
            fixture(
                "lower-rank",
                1,
                Some("2026-08-10T01:00:00Z"),
                "2026-08-10T12:00:00Z",
            ),
            fixture(
                "earlier",
                0,
                Some("2026-08-10T09:00:00Z"),
                "2026-08-10T11:00:00Z",
            ),
        ];
        sort_m4_inbox_items(&mut items).expect("sort fixed M4 items");
        assert_eq!(
            items
                .iter()
                .map(|item| item.inbox_item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["earlier", "later", "null", "lower-rank"]
        );
    }

    #[test]
    fn m4c03_read_order_compares_fractional_utc_instants_not_raw_text() {
        let mut items = vec![
            fixture("later", 0, None, "2026-08-10T09:00:00.1Z"),
            fixture("earlier", 0, None, "2026-08-10T09:00:00Z"),
        ];
        sort_m4_inbox_items(&mut items).expect("sort fractional M4 instants");
        assert_eq!(items[0].inbox_item_id, "later");
    }

    #[test]
    fn m4c08_legacy_inventory_is_complete_deterministic_and_ref_only() {
        let inventory = m4_legacy_read_source_inventory();
        assert_eq!(
            inventory
                .iter()
                .map(|entry| entry.legacy_source_kind.as_str())
                .collect::<Vec<_>>(),
            vec![
                "SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY",
                "RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION",
                "RUNTIME_ATTENTION_PROJECTION",
                "REACT_PENDING_ACTION_VISIBILITY",
                "MEMORY_DAILY_INBOX_CANDIDATE",
            ]
        );
        assert!(inventory.iter().all(|entry| {
            entry.compatibility_role == "SOURCE_REF_AND_DEDUPE_CANDIDATE_ONLY"
                && entry.write_authority == "NONE"
        }));
        let serialized = serde_json::to_string(&inventory).expect("serialize C08 inventory");
        for forbidden in [
            "payload",
            "callback",
            "transcript",
            "credential",
            "owner_state",
        ] {
            assert!(
                !serialized.to_ascii_lowercase().contains(forbidden),
                "inventory leaked forbidden field: {forbidden}"
            );
        }
    }

    #[test]
    fn m4c08_shadow_parity_releases_only_exact_rows_to_guarded_fallback() {
        let canonical = M4LegacyCanonicalSourceRead {
            source_owner_ref: "workflow-owner".to_string(),
            scope_ref: "scope:personal:primary".to_string(),
            source_type: "structured_internal_workflow_attention_ref".to_string(),
            canonical_source_object_id: "work-item-a".to_string(),
            source_revision: 7,
            source_owner_watermark: opaque("watermark", 'a'),
            source_link: m4c04_source_link("work-item-a", 7),
            source_status_code: "WAITING_USER".to_string(),
            priority_reason_code: "USER_DECISION_OR_BLOCKER".to_string(),
        };
        let snapshot = M4LegacyCanonicalReadSnapshot {
            scope_ref: "scope:personal:primary".to_string(),
            scope_source_watermark: "b".repeat(64),
            sources: vec![canonical.clone()],
        };
        let exact = M4LegacyReadCandidate {
            legacy_source_kind: M4LegacyReadSourceKind::SecretaryReadModelDeterministicSummary,
            legacy_item_ref: Some(opaque("legacy-item", 'c')),
            source_owner_ref: Some(canonical.source_owner_ref.clone()),
            scope_ref: Some(canonical.scope_ref.clone()),
            source_type: Some(canonical.source_type.clone()),
            canonical_source_object_id: Some(canonical.canonical_source_object_id.clone()),
            source_revision: Some(canonical.source_revision),
            source_owner_watermark: Some(canonical.source_owner_watermark.clone()),
            source_link: Some(canonical.source_link.clone()),
            source_status_code: Some(canonical.source_status_code.clone()),
            priority_reason_code: Some(canonical.priority_reason_code.clone()),
            scope_source_watermark: Some(snapshot.scope_source_watermark.clone()),
        };
        let inventory_only = M4LegacyReadCandidate {
            legacy_source_kind: M4LegacyReadSourceKind::MemoryDailyInboxCandidate,
            legacy_item_ref: None,
            source_owner_ref: None,
            scope_ref: None,
            source_type: None,
            canonical_source_object_id: None,
            source_revision: None,
            source_owner_watermark: None,
            source_link: None,
            source_status_code: None,
            priority_reason_code: None,
            scope_source_watermark: None,
        };
        let inventory_report = build_m4_legacy_shadow_parity_report(&snapshot, &[inventory_only])
            .expect("inventory-only C08 candidate becomes a report row");
        assert_eq!(
            inventory_report.rows[0].reason_code.as_deref(),
            Some("M4C08_LEGACY_CANDIDATE_INVALID"),
            "a missing tuple is quarantined rather than guessed"
        );

        let mut conflicting_canonical = canonical.clone();
        conflicting_canonical.source_revision = 8;
        conflicting_canonical.source_owner_watermark = opaque("watermark", 'd');
        conflicting_canonical.source_link.expected_source_revision = 8;
        let ambiguous_snapshot = M4LegacyCanonicalReadSnapshot {
            scope_ref: snapshot.scope_ref.clone(),
            scope_source_watermark: snapshot.scope_source_watermark.clone(),
            sources: vec![canonical.clone(), conflicting_canonical],
        };
        let ambiguous_report =
            build_m4_legacy_shadow_parity_report(&ambiguous_snapshot, &[exact.clone()])
                .expect("ambiguous source identity remains a report-only quarantine");
        assert_eq!(
            ambiguous_report.rows[0].reason_code.as_deref(),
            Some("M4C08_CANONICAL_SOURCE_AMBIGUOUS"),
            "an ambiguous legacy join cannot become a guarded display row"
        );

        let mut same_canonical_other_legacy_surface = exact.clone();
        same_canonical_other_legacy_surface.legacy_source_kind =
            M4LegacyReadSourceKind::RightRailNotificationAndTodoProjection;
        same_canonical_other_legacy_surface.legacy_item_ref = Some(opaque("legacy-item", 'e'));
        let canonical_dedupe_report = build_m4_legacy_shadow_parity_report(
            &snapshot,
            &[exact.clone(), same_canonical_other_legacy_surface],
        )
        .expect("different legacy surfaces may point to one canonical source");
        assert_eq!(
            canonical_dedupe_report
                .rows
                .iter()
                .filter(|row| row.disposition == "PARITY")
                .count(),
            2
        );
        assert_eq!(
            canonical_dedupe_report
                .rows
                .iter()
                .filter(|row| row.dedupe_disposition == "PRIMARY")
                .count(),
            1
        );
        assert_eq!(
            canonical_dedupe_report
                .rows
                .iter()
                .filter(|row| row.dedupe_disposition == "DUPLICATE_DISPLAY_ONLY")
                .count(),
            1,
            "different legacy kinds retain ordinary canonical de-duplication"
        );

        let mut mismatched = exact.clone();
        mismatched.legacy_source_kind = M4LegacyReadSourceKind::RuntimeAttentionProjection;
        mismatched.source_status_code = Some("OPEN".to_string());

        let report = build_m4_legacy_shadow_parity_report(&snapshot, &[mismatched, exact])
            .expect("build C08 parity report");
        assert_eq!(report.mode, "M4_PRIMARY_LEGACY_READ_ONLY_FALLBACK");
        assert_eq!(
            report.parity_matrix_version,
            M4_LEGACY_READ_PARITY_MATRIX_VERSION
        );
        assert_eq!(report.rows.len(), 2);
        let exact_row = report
            .rows
            .iter()
            .find(|row| row.disposition == "PARITY")
            .expect("exact legacy candidate remains visible");
        assert!(
            exact_row.source_matches
                && exact_row.status_matches
                && exact_row.priority_reason_matches
                && exact_row.source_owner_watermark_matches
                && exact_row.scope_source_watermark_matches
        );
        assert_eq!(
            exact_row
                .canonical_source
                .as_ref()
                .expect("parity row has canonical source")
                .source_status_code,
            "WAITING_USER"
        );
        assert_eq!(
            report
                .rows
                .iter()
                .find(|row| row.disposition == "QUARANTINED")
                .and_then(|row| row.reason_code.as_deref()),
            Some("M4C08_PARITY_STATUS_MISMATCH")
        );
        let fallback =
            guarded_m4_legacy_read_only_fallback(&report).expect("guarded compatibility fallback");
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].disposition, "PARITY");

        let transport = M4LegacyReadCompatibilityReportEnvelope::ready(report.clone());
        let serialized = serde_json::to_value(transport).expect("serialize C08 transport envelope");
        assert_eq!(serialized["status"], "READY");
        assert_eq!(
            serialized["report"]["rows"][0]["canonical_source"]["source_revision"], "7",
            "C08 transport keeps u64 source revisions as canonical strings"
        );
        assert_eq!(
            serialized["report"]["rows"][0]["canonical_source"]["source_link"]
                ["expected_source_revision"],
            "7",
            "C08 transport keeps link revisions as canonical strings"
        );
        let serialized_text =
            serde_json::to_string(&serialized).expect("render C08 transport JSON");
        for forbidden in ["payload", "transcript", "cwd", "label", "pending_action"] {
            assert!(
                !serialized_text
                    .to_ascii_lowercase()
                    .contains(&format!("\"{forbidden}\"")),
                "C08 transport leaked forbidden field name: {forbidden}"
            );
        }
    }

    fn opaque(namespace: &str, digit: char) -> String {
        format!("{namespace}:sha256:{}", digit.to_string().repeat(64))
    }

    fn deterministic(prefix: &str, digit: char) -> String {
        format!("{prefix}:{}", digit.to_string().repeat(64))
    }

    fn m4c04_source_link(object_id: &str, expected_revision: u64) -> M4SourceLinkRead {
        M4SourceLinkRead {
            link_kind: "INTERNAL_ROUTE".to_string(),
            source_owner_ref: "workflow-owner".to_string(),
            object_type: "workflow_attention".to_string(),
            canonical_source_object_id: object_id.to_string(),
            expected_source_revision: expected_revision,
            opaque_route_ref: opaque("route", 'a'),
        }
    }

    fn m4c04_personal_action(digit: char) -> M4PersonalActionRead {
        M4PersonalActionRead {
            personal_action_id: deterministic("personal-action", digit),
            explicit_user_command_ref: opaque("command", 'b'),
            title: "整理本周计划".to_string(),
            status: "OPEN".to_string(),
            due_at_utc: Some("2026-08-10T10:00:00Z".to_string()),
            revision: "0".to_string(),
        }
    }

    fn m4c04_notification(digit: char) -> M4NotificationRead {
        M4NotificationRead {
            notification_id: deterministic("notification", digit),
            source_ref: m4c04_source_link("work-item-a", 7),
            subject_ref: deterministic("open-loop", 'c'),
            notification_purpose_code: "ATTENTION_DUE".to_string(),
            delivery_channel: "IN_APP".to_string(),
            status: "PENDING".to_string(),
            created_at_utc: "2026-08-10T09:00:00Z".to_string(),
            delivered_at_utc: None,
            read_at_utc: None,
            dismissed_at_utc: None,
            revision: "0".to_string(),
        }
    }

    fn m4c04_reminder(digit: char) -> M4ReminderRead {
        M4ReminderRead {
            reminder_id: deterministic("reminder", digit),
            owner_ref: deterministic("personal-action", 'd'),
            explicit_schedule_command_id: opaque("command", 'e'),
            scheduled_for_utc: "2026-08-10T10:00:00Z".to_string(),
            iana_timezone: "Asia/Shanghai".to_string(),
            status: "SCHEDULED".to_string(),
            last_fired_at_utc: None,
            snoozed_until_utc: None,
            revision: "0".to_string(),
        }
    }

    fn m4c04_owner_writeback(status: &str) -> M4OwnerWritebackReceiptRead {
        let (scrubbed_owner_receipt_ref, error_code) = match status {
            "PENDING" => (None, None),
            "SUCCEEDED" => (Some(opaque("owner-receipt", 'f')), None),
            "FAILED" => (
                Some(opaque("owner-receipt", 'f')),
                Some("OWNER_REVISION_CONFLICT".to_string()),
            ),
            _ => (None, None),
        };
        M4OwnerWritebackReceiptRead {
            source_ref: m4c04_source_link("work-item-a", 7),
            expected_source_revision: 7,
            explicit_intent_code: "CLOSE_SOURCE_ITEM".to_string(),
            status: status.to_string(),
            scrubbed_owner_receipt_ref,
            error_code,
        }
    }

    fn m4c04_snapshot() -> M4CoordinationSnapshot {
        M4CoordinationSnapshot {
            scope_ref: "scope:personal:primary".to_string(),
            scope_source_watermark: "a".repeat(64),
            inbox_items: Vec::new(),
            open_loops: Vec::new(),
            personal_actions: vec![m4c04_personal_action('a')],
            notifications: vec![m4c04_notification('b')],
            reminders: vec![m4c04_reminder('c')],
            decisions: Vec::new(),
            owner_writeback_receipts: vec![m4c04_owner_writeback("PENDING")],
        }
    }

    #[test]
    fn m4c04_rejects_invalid_state_time_revision_sensitive_ref_and_title() {
        let mut invalid_status = m4c04_snapshot();
        invalid_status.personal_actions[0].status = "DISMISSED".to_string();
        assert!(validate_m4c04_coordination_snapshot(&invalid_status).is_err());

        let mut invalid_time = m4c04_snapshot();
        invalid_time.notifications[0].created_at_utc = "2026-08-10T09:00:00+08:00".to_string();
        assert!(validate_m4c04_coordination_snapshot(&invalid_time).is_err());

        let mut invalid_revision = m4c04_snapshot();
        invalid_revision.reminders[0].revision = "01".to_string();
        assert!(validate_m4c04_coordination_snapshot(&invalid_revision).is_err());

        let mut sensitive_ref = m4c04_snapshot();
        sensitive_ref.owner_writeback_receipts[0].status = "SUCCEEDED".to_string();
        sensitive_ref.owner_writeback_receipts[0].scrubbed_owner_receipt_ref =
            Some("https://owner.example/callback".to_string());
        assert!(validate_m4c04_coordination_snapshot(&sensitive_ref).is_err());

        let mut invalid_title = m4c04_snapshot();
        invalid_title.personal_actions[0].title = " 整理本周计划 ".to_string();
        assert!(validate_m4c04_coordination_snapshot(&invalid_title).is_err());
    }

    #[test]
    fn m4c04_title_matches_local_personal_action_constraints() {
        let mut longest_valid = m4c04_snapshot();
        longest_valid.personal_actions[0].title = "测".repeat(160);
        validate_m4c04_coordination_snapshot(&longest_valid)
            .expect("160-character local personal-action title is valid");

        for invalid_title in [
            String::new(),
            "测".repeat(161),
            "https://example.test/todo".to_string(),
            "/absolute/path/to/todo".to_string(),
            "C:\\Users\\todo".to_string(),
            "todo\nnext".to_string(),
        ] {
            let mut snapshot = m4c04_snapshot();
            snapshot.personal_actions[0].title = invalid_title;
            assert!(validate_m4c04_coordination_snapshot(&snapshot).is_err());
        }
    }

    #[test]
    fn m4c04_accepts_typed_subject_owner_refs_and_all_writeback_outcomes() {
        for status in ["PENDING", "SUCCEEDED", "FAILED"] {
            let mut snapshot = m4c04_snapshot();
            snapshot.owner_writeback_receipts = vec![m4c04_owner_writeback(status)];
            validate_m4c04_coordination_snapshot(&snapshot)
                .expect("M4C04 typed refs and writeback outcome are valid");
        }

        let mut pending_with_receipt = m4c04_snapshot();
        pending_with_receipt.owner_writeback_receipts[0].scrubbed_owner_receipt_ref =
            Some(opaque("owner-receipt", 'f'));
        assert!(validate_m4c04_coordination_snapshot(&pending_with_receipt).is_err());
    }

    #[test]
    fn m4c04_snapshot_sort_is_stable_and_owner_writeback_never_claims_owner_state() {
        let mut snapshot = m4c04_snapshot();
        snapshot.personal_actions = vec![m4c04_personal_action('b'), m4c04_personal_action('a')];
        snapshot.notifications = vec![m4c04_notification('d'), m4c04_notification('c')];
        snapshot.reminders = vec![m4c04_reminder('f'), m4c04_reminder('e')];
        let mut receipt_b = m4c04_owner_writeback("SUCCEEDED");
        receipt_b.source_ref = m4c04_source_link("work-item-b", 7);
        let mut receipt_a = m4c04_owner_writeback("SUCCEEDED");
        receipt_a.source_ref = m4c04_source_link("work-item-a", 7);
        snapshot.owner_writeback_receipts = vec![receipt_b, receipt_a];

        sort_m4c04_coordination_snapshot(&mut snapshot)
            .expect("sort valid M4C04 coordination snapshot");
        assert_eq!(
            snapshot
                .personal_actions
                .iter()
                .map(|item| item.personal_action_id.clone())
                .collect::<Vec<_>>(),
            vec![
                deterministic("personal-action", 'a'),
                deterministic("personal-action", 'b'),
            ]
        );
        assert_eq!(
            snapshot
                .notifications
                .iter()
                .map(|item| item.notification_id.clone())
                .collect::<Vec<_>>(),
            vec![
                deterministic("notification", 'c'),
                deterministic("notification", 'd'),
            ]
        );
        assert_eq!(
            snapshot
                .reminders
                .iter()
                .map(|item| item.reminder_id.clone())
                .collect::<Vec<_>>(),
            vec![
                deterministic("reminder", 'e'),
                deterministic("reminder", 'f')
            ]
        );
        assert_eq!(
            snapshot.owner_writeback_receipts[0]
                .source_ref
                .canonical_source_object_id,
            "work-item-a"
        );

        let receipt_json = serde_json::to_value(&snapshot.owner_writeback_receipts[0])
            .expect("serialize scrubbed owner writeback receipt");
        let receipt_fields = receipt_json
            .as_object()
            .expect("owner writeback receipt serializes as object");
        assert_eq!(receipt_fields.len(), 6);
        for forbidden in [
            "owner_state",
            "owner_status",
            "callback",
            "executable_payload",
            "credential",
            "url",
            "path",
        ] {
            assert!(
                receipt_fields.get(forbidden).is_none(),
                "forbidden: {forbidden}"
            );
        }
    }

    fn m4c07_daily_window(digit: char) -> String {
        deterministic("daily-window", digit)
    }

    fn m4c07_ready_envelope() -> M4SecretaryDailyReportEnvelope {
        let current_window = m4c07_daily_window('a');
        let closed_window = m4c07_daily_window('b');
        M4SecretaryDailyReportEnvelope::Ready {
            schema_version: M4_SECRETARY_DAILY_SCHEMA_VERSION.to_string(),
            scheduler: M4SecretaryDailySchedulerRead {
                configuration_revision: "2".to_string(),
                iana_timezone: "Asia/Shanghai".to_string(),
                timezone_rules_version: deterministic("timezone-rules", 'c'),
                current_daily_window_id: current_window.clone(),
                last_closed_daily_window_id: closed_window.clone(),
                catch_up_pending_count: 0,
                pending_catch_up_receipt_refs: vec![],
                status: "IDLE".to_string(),
            },
            daily_brief: M4SecretaryDailyBriefRead {
                daily_window_id: current_window,
                scope_source_watermark: "e".repeat(64),
                projector_version: "1".to_string(),
                ordered_item_refs: vec![
                    deterministic("open-loop", 'a'),
                    deterministic("inbox", 'b'),
                ],
                generated_at_utc: Some("2026-08-10T09:00:00Z".to_string()),
            },
            daily_report: M4SecretaryDailyReportRead {
                daily_report_id: deterministic("daily-report", 'c'),
                daily_window_id: closed_window.clone(),
                report_version: "1".to_string(),
                status: "GENERATED".to_string(),
                scope_source_watermark: "d".repeat(64),
                projector_version: "1".to_string(),
                ordered_item_refs: vec![
                    deterministic("source-event", 'd'),
                    deterministic("personal-action", 'e'),
                ],
                supersedes_report_ref: None,
                generated_at_utc: Some("2026-08-10T00:05:00Z".to_string()),
            },
            last_run: M4SecretarySchedulerRunRead {
                scheduler_run_id: deterministic("scheduler-run", 'f'),
                configuration_revision: "2".to_string(),
                window_ref: closed_window,
                scope_source_watermark_before: "c".repeat(64),
                scope_source_watermark_after: "d".repeat(64),
                admitted_material_event_count: 0,
                agent_turn_count: 0,
                model_invocation_count: 0,
                outcome_code: "EMPTY_WINDOW".to_string(),
                recorded_at_utc: Some("2026-08-10T00:05:00Z".to_string()),
            },
            recovery_code: None,
        }
    }

    #[test]
    fn m4c07_ready_envelope_preserves_server_priority_order_and_serializes_frozen_fields() {
        let envelope = m4c07_ready_envelope();
        validate_m4c07_daily_report_envelope(&envelope)
            .expect("synthetic M4C07 envelope validates with server priority order");

        let M4SecretaryDailyReportEnvelope::Ready {
            daily_brief,
            daily_report,
            ..
        } = &envelope
        else {
            panic!("fixture must remain ready");
        };
        // These intentionally non-lexical sequences stand for the service's
        // M4 priority/due/source-change projection and must pass unchanged.
        assert_eq!(
            daily_brief.ordered_item_refs,
            vec![deterministic("open-loop", 'a'), deterministic("inbox", 'b')]
        );
        assert_eq!(
            daily_report.ordered_item_refs,
            vec![
                deterministic("source-event", 'd'),
                deterministic("personal-action", 'e'),
            ]
        );

        let serialized = serde_json::to_value(&envelope).expect("serialize frozen daily envelope");
        let fields = serialized
            .as_object()
            .expect("daily envelope serializes as an object");
        assert_eq!(
            fields
                .get("schema_version")
                .and_then(|value| value.as_str()),
            Some(M4_SECRETARY_DAILY_SCHEMA_VERSION)
        );
        assert_eq!(
            fields.get("status").and_then(|value| value.as_str()),
            Some("READY")
        );
        for forbidden in [
            "raw_transcript",
            "transcript",
            "prompt",
            "provider_body",
            "memory_artifact",
            "secret",
            "credential",
            "route_payload",
            "callback",
        ] {
            assert!(
                fields.get(forbidden).is_none(),
                "forbidden field: {forbidden}"
            );
        }
    }

    #[test]
    fn m4c07_rejects_noncanonical_cross_bound_unsafe_and_zero_event_payloads() {
        let mut noncanonical = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { scheduler, .. } = &mut noncanonical {
            scheduler.configuration_revision = "02".to_string();
        }
        assert!(validate_m4c07_daily_report_envelope(&noncanonical).is_err());

        let mut invalid_timezone_rules_version = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { scheduler, .. } =
            &mut invalid_timezone_rules_version
        {
            scheduler.timezone_rules_version = "tzdb-2026a".to_string();
        }
        assert!(validate_m4c07_daily_report_envelope(&invalid_timezone_rules_version).is_err());

        let mut cross_window = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { daily_brief, .. } = &mut cross_window {
            daily_brief.daily_window_id = m4c07_daily_window('c');
        }
        assert!(validate_m4c07_daily_report_envelope(&cross_window).is_err());

        let mut unsafe_ref = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { daily_report, .. } = &mut unsafe_ref {
            daily_report.ordered_item_refs = vec!["https://example.invalid/raw-body".to_string()];
        }
        assert!(validate_m4c07_daily_report_envelope(&unsafe_ref).is_err());

        let mut duplicate_ref = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { daily_brief, .. } = &mut duplicate_ref {
            let duplicate = deterministic("inbox", 'b');
            daily_brief.ordered_item_refs = vec![duplicate.clone(), duplicate];
        }
        assert!(validate_m4c07_daily_report_envelope(&duplicate_ref).is_err());

        let mut nonzero_empty_window = m4c07_ready_envelope();
        if let M4SecretaryDailyReportEnvelope::Ready { last_run, .. } = &mut nonzero_empty_window {
            last_run.agent_turn_count = 1;
        }
        assert!(validate_m4c07_daily_report_envelope(&nonzero_empty_window).is_err());
    }

    #[test]
    fn m4c07_unavailable_and_disabled_are_exclusive_scrubbed_envelopes() {
        let unavailable = M4SecretaryDailyReportEnvelope::Unavailable {
            schema_version: M4_SECRETARY_DAILY_SCHEMA_VERSION.to_string(),
            reason: "M4_DAILY_STORAGE_UNAVAILABLE".to_string(),
        };
        let disabled = M4SecretaryDailyReportEnvelope::Disabled {
            schema_version: M4_SECRETARY_DAILY_SCHEMA_VERSION.to_string(),
            reason: "SCHEDULER_CONFIGURATION_DISABLED".to_string(),
        };
        validate_m4c07_daily_report_envelope(&unavailable)
            .expect("scrubbed unavailable envelope is valid");
        validate_m4c07_daily_report_envelope(&disabled)
            .expect("scrubbed disabled envelope is valid");

        for envelope in [unavailable, disabled] {
            let serialized = serde_json::to_value(&envelope).expect("serialize non-ready envelope");
            let fields = serialized
                .as_object()
                .expect("non-ready envelope serializes as an object");
            assert_eq!(fields.len(), 3);
            assert!(fields.get("reason").is_some());
            assert!(fields.get("scheduler").is_none());
            assert!(fields.get("daily_brief").is_none());
            assert!(fields.get("daily_report").is_none());
            assert!(fields.get("last_run").is_none());
        }

        let unsafe_reason = M4SecretaryDailyReportEnvelope::Unavailable {
            schema_version: M4_SECRETARY_DAILY_SCHEMA_VERSION.to_string(),
            reason: "DATABASE_PATH_PRIVATE_TMP".to_string(),
        };
        assert!(validate_m4c07_daily_report_envelope(&unsafe_reason).is_err());
    }
}
