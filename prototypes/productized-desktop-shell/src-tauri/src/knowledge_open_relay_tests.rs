use crate::knowledge_open_relay::{
    validate_main_window_ack_source, RelayAckOutcome, RelayBindingIdentity,
    RelayBindingStateForTest, RelayDispatchStatus, RelayIntentRequest, RelayTestHarness,
};

#[test]
fn r0_host_relay_needs_matching_active_grant_and_exact_ui_ack_before_opened() {
    let mut relay = RelayTestHarness::active(RelayBindingIdentity::new(
        "supervisor-conversation:r0",
        "turn:r0",
        "project:r0",
    ));
    let grant = relay.issue_grant();
    let request = RelayIntentRequest::new(
        grant,
        "supervisor-conversation:r0",
        "turn:r0",
        "project:r0",
        "research/OpenMe.md",
    );

    let pending = relay
        .accept_intent(request)
        .expect("active host grant accepts one exact intent");
    assert_eq!(pending.status(), RelayDispatchStatus::AwaitingUiAck);
    assert!(
        !pending.opened(),
        "dispatch must not claim success before the UI ack"
    );

    let opened = relay
        .acknowledge(
            pending.intent_id(),
            "research/OpenMe.md",
            RelayAckOutcome::Opened,
        )
        .expect("only the exact intent and path can settle opened");
    assert_eq!(opened.status(), RelayDispatchStatus::Opened);
    assert!(opened.opened());
}

#[test]
fn r0_host_relay_rejects_expiry_binding_mismatch_replay_and_oversize_frames() {
    let identity = RelayBindingIdentity::new("supervisor-conversation:r0", "turn:r0", "project:r0");
    let mut relay = RelayTestHarness::active(identity.clone());
    let grant = relay.issue_grant();
    assert!(relay
        .accept_intent(RelayIntentRequest::new(
            grant.clone(),
            "supervisor-conversation:r0",
            "turn:wrong",
            "project:r0",
            "research/OpenMe.md",
        ))
        .is_err());
    assert!(relay
        .accept_intent(RelayIntentRequest::oversize_for_test(
            grant.clone(),
            identity.clone()
        ))
        .is_err());

    let pending = relay
        .accept_intent(RelayIntentRequest::new(
            grant,
            "supervisor-conversation:r0",
            "turn:r0",
            "project:r0",
            "research/OpenMe.md",
        ))
        .expect("one exact request is pending");
    assert!(relay.replay_intent_for_test(pending.intent_id()).is_err());
    relay.expire_for_test();
    assert!(relay
        .acknowledge(
            pending.intent_id(),
            "research/OpenMe.md",
            RelayAckOutcome::Opened
        )
        .is_err());
}

#[test]
fn r0_host_relay_rejects_wrong_ui_ack_and_dirty_navigation_without_opened() {
    let mut relay = RelayTestHarness::active(RelayBindingIdentity::new(
        "supervisor-conversation:r0",
        "turn:r0",
        "project:r0",
    ));
    let grant = relay.issue_grant();
    let pending = relay
        .accept_intent(RelayIntentRequest::new(
            grant,
            "supervisor-conversation:r0",
            "turn:r0",
            "project:r0",
            "research/OpenMe.md",
        ))
        .expect("exact request is pending");
    assert!(relay
        .acknowledge(
            pending.intent_id(),
            "research/Other.md",
            RelayAckOutcome::Opened
        )
        .is_err());

    let rejected = relay
        .acknowledge(
            pending.intent_id(),
            "research/OpenMe.md",
            RelayAckOutcome::Rejected,
        )
        .expect("dirty navigation is an exact rejection, never an opened ack");
    assert_eq!(rejected.status(), RelayDispatchStatus::Rejected);
    assert!(!rejected.opened());
}

#[test]
fn r0_host_relay_rejects_starting_and_failed_bindings_before_dispatch() {
    for binding_state in [
        RelayBindingStateForTest::Starting,
        RelayBindingStateForTest::Failed,
        RelayBindingStateForTest::Terminated,
    ] {
        let identity =
            RelayBindingIdentity::new("supervisor-conversation:r0", "turn:r0", "project:r0");
        let mut relay =
            RelayTestHarness::with_binding_state_for_test(identity.clone(), binding_state);
        let grant = relay.issue_grant();
        assert!(relay
            .accept_intent(RelayIntentRequest::new(
                grant,
                "supervisor-conversation:r0",
                "turn:r0",
                "project:r0",
                "research/OpenMe.md",
            ))
            .is_err());
    }
}

#[test]
fn r3_host_relay_rejects_run_and_project_mismatch_before_ui_dispatch() {
    let identity = RelayBindingIdentity::new("supervisor-conversation:r3", "turn:r3", "project:r3");
    for (run_id, turn_id, project_id) in [
        ("supervisor-conversation:other", "turn:r3", "project:r3"),
        ("supervisor-conversation:r3", "turn:other", "project:r3"),
        ("supervisor-conversation:r3", "turn:r3", "project:other"),
    ] {
        let mut relay = RelayTestHarness::active(identity.clone());
        let grant = relay.issue_grant();
        assert!(
            relay
                .accept_intent(RelayIntentRequest::new(
                    grant,
                    run_id,
                    turn_id,
                    project_id,
                    "research/OpenMe.md",
                ))
                .is_err(),
            "all host-bound identity fields must match before a UI dispatch"
        );
    }
}

#[test]
fn r3_host_relay_rejects_wrong_intent_and_replay_after_opened() {
    let mut relay = RelayTestHarness::active(RelayBindingIdentity::new(
        "supervisor-conversation:r3",
        "turn:r3",
        "project:r3",
    ));
    let grant = relay.issue_grant();
    let pending = relay
        .accept_intent(RelayIntentRequest::new(
            grant.clone(),
            "supervisor-conversation:r3",
            "turn:r3",
            "project:r3",
            "research/OpenMe.md",
        ))
        .expect("one exact intent is pending");

    assert!(
        relay
            .acknowledge(
                "intent:test:wrong",
                "research/OpenMe.md",
                RelayAckOutcome::Opened,
            )
            .is_err(),
        "a different intent id cannot settle the pending request"
    );
    assert!(relay
        .acknowledge(
            pending.intent_id(),
            "research/OpenMe.md",
            RelayAckOutcome::Opened,
        )
        .expect("exact acknowledgement opens once")
        .opened());
    assert!(
        relay
            .acknowledge(
                pending.intent_id(),
                "research/OpenMe.md",
                RelayAckOutcome::Opened,
            )
            .is_err(),
        "the settled intent cannot be acknowledged twice"
    );
    assert!(
        relay
            .accept_intent(RelayIntentRequest::new(
                grant,
                "supervisor-conversation:r3",
                "turn:r3",
                "project:r3",
                "research/OpenMe.md",
            ))
            .is_err(),
        "the single-use grant cannot replay after opened"
    );
}

#[test]
fn r3_host_relay_timeout_and_run_revocation_clear_pending_without_opened() {
    let identity = RelayBindingIdentity::new("supervisor-conversation:r3", "turn:r3", "project:r3");
    let mut relay = RelayTestHarness::active(identity.clone());

    let timed_out_grant = relay.issue_grant();
    let timed_out = relay
        .accept_intent(RelayIntentRequest::new(
            timed_out_grant.clone(),
            "supervisor-conversation:r3",
            "turn:r3",
            "project:r3",
            "research/OpenMe.md",
        ))
        .expect("exact request is pending before timeout");
    relay.expire_for_test();
    assert!(
        relay
            .acknowledge(
                timed_out.intent_id(),
                "research/OpenMe.md",
                RelayAckOutcome::Opened,
            )
            .is_err(),
        "expired UI acknowledgement cannot claim opened"
    );
    assert!(
        relay
            .accept_intent(RelayIntentRequest::new(
                timed_out_grant,
                "supervisor-conversation:r3",
                "turn:r3",
                "project:r3",
                "research/OpenMe.md",
            ))
            .is_err(),
        "timeout removes the old grant and pending intent"
    );

    let revoked_grant = relay.issue_grant();
    let revoked = relay
        .accept_intent(RelayIntentRequest::new(
            revoked_grant,
            "supervisor-conversation:r3",
            "turn:r3",
            "project:r3",
            "research/OpenMe.md",
        ))
        .expect("exact request is pending before transport terminal cleanup");
    relay.revoke_run_for_test();
    assert!(
        relay
            .acknowledge(
                revoked.intent_id(),
                "research/OpenMe.md",
                RelayAckOutcome::Opened,
            )
            .is_err(),
        "transport terminal cleanup removes pending acknowledgements"
    );
}

#[test]
fn r1_host_relay_rejects_acknowledgements_from_non_main_webviews() {
    assert!(validate_main_window_ack_source("preview").is_err());
    assert!(validate_main_window_ack_source("inspector").is_err());
    assert!(validate_main_window_ack_source("main").is_ok());
}

#[test]
fn r1_raw_manual_receipt_cannot_bypass_a_registered_transport_attempt() {
    let attempt_id = format!("knowledge-open-relay-raw-guard:{}", std::process::id());
    {
        let mut attempts = crate::conversation_transport_command_attempts()
            .lock()
            .expect("test registry lock");
        assert!(attempts
            .insert(
                attempt_id.clone(),
                crate::ConversationTransportCommandAttempt {
                    relay_attempt_id: attempt_id.clone(),
                    host_owned_cleanup_recovery: false,
                    conversation_id: "conversation:relay-test".to_string(),
                    turn_id: "turn:relay-test".to_string(),
                    profile: crate::ConversationTransportCommandAttemptProfile::Agent,
                },
            )
            .is_none());
    }

    assert_eq!(
        crate::reject_raw_managed_conversation_transport_attempt(&attempt_id)
            .expect_err("registered transport attempts have no raw manual receipt route"),
        "manual_relay_managed_conversation_attempt_protected"
    );
    crate::conversation_transport_command_attempts()
        .lock()
        .expect("test registry lock")
        .remove(&attempt_id);
}

#[test]
fn r1_raw_manual_receipt_is_closed_from_the_supervisor_pre_spawn_marker() {
    let attempt_id = format!(
        "knowledge-open-relay-pre-spawn-guard:{}",
        std::process::id()
    );
    crate::manual_relay::reserve_safe_only_manual_relay_attempt_for_test(&attempt_id)
        .expect("fixture must reserve the marker before a child exists");

    assert_eq!(
        crate::reject_raw_managed_conversation_transport_attempt(&attempt_id)
            .expect_err("outer raw guard must consult the pre-spawn marker"),
        "manual_relay_managed_conversation_attempt_protected"
    );
    assert_eq!(
        crate::manual_relay::poll_manual_relay_attempt(
            crate::manual_relay::ManualRelayPollInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "raw-fixture".to_string(),
            },
            "2026-07-23T00:00:00Z",
        )
        .expect_err("raw poll must stay closed before spawn"),
        "manual_relay_managed_conversation_attempt_protected"
    );
    assert_eq!(
        crate::manual_relay::stop_manual_relay_attempt(
            crate::manual_relay::ManualRelayStopInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "raw-fixture".to_string(),
            },
            "2026-07-23T00:00:00Z",
        )
        .expect_err("raw stop must stay closed before spawn"),
        "manual_relay_managed_conversation_attempt_protected"
    );
    crate::manual_relay::clear_safe_only_manual_relay_attempt_for_test(&attempt_id);
}
