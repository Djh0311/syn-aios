pub(crate) fn is_h4_active_attempt_status(status: &str) -> bool {
    matches!(
        status,
        "queued"
            | "running"
            | "waiting_permission"
            | "waiting_for_permission"
            | "running_stub"
            | "dry_run_running"
            | "queued_real"
            | "running_real"
            | "running_h2_phase_a"
            | "running_h2_phase_b"
            | "running_h3_b"
    )
}

pub(crate) fn h4_result_count(
    status: &str,
    readback_status: &str,
    raw: Option<i64>,
) -> Option<i64> {
    if h4_unknown_result_status(status) || h4_unknown_result_status(readback_status) {
        None
    } else {
        raw
    }
}

pub(crate) fn h4_unknown_result_status(status: &str) -> bool {
    matches!(
        status,
        "readback_unavailable"
            | "readback_failed"
            | "readback_timed_out"
            | "timed_out"
            | "not_attempted"
            | "not_attempted_stub"
            | "blocked_by_guard"
            | "blocked_waiting_authorization"
            | "codex_state_error"
            | "duplicate_blocked"
            | "user_rejected"
            | "cancel_requested"
            | "stale_cancelled"
            | "dry_run_blocked"
    )
}

pub(crate) fn h4_unknown_result_warning() -> String {
    "h4_unknown_readback_states_keep_result_count_null".to_string()
}
