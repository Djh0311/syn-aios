//! M4C03 read-only Attention projection DTOs and deterministic ordering.
//!
//! This module has no connection or mutation capability. The repository
//! supplies already-validated rows; ordering is repeated mechanically in Rust
//! so SQLite collation or renderer order never becomes priority authority.

use crate::m4_secretary_domain::m4_parse_rfc3339_utc_key;
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4SourceLinkRead {
    pub(crate) link_kind: String,
    pub(crate) source_owner_ref: String,
    pub(crate) object_type: String,
    pub(crate) canonical_source_object_id: String,
    pub(crate) expected_source_revision: u64,
    pub(crate) opaque_route_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M4AttentionSnapshot {
    pub(crate) scope_ref: String,
    pub(crate) scope_source_watermark: String,
    pub(crate) inbox_items: Vec<M4InboxItemRead>,
    pub(crate) open_loops: Vec<M4OpenLoopRead>,
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
}
