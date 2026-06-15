use crate::utils::fs_ops::fixture_dir;

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn sqlite_production_observation_feature_flag_disabled_uses_fallback_without_db() {
    let paths = prepare_production_paths("flag-off");
    copy_a11_fixture_to_root(&paths.fallback_root);
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let report = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        false,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect("flag disabled fallback observation");

    assert_eq!(
        report.status,
        "feature_flag_disabled_json_fallback_observation"
    );
    assert_eq!(report.observation_source, "json_fallback");
    assert!(report.degraded);
    assert!(!paths.db_path.exists());
    assert!(!report.safety_flags.production_observation_enabled);
    assert!(!report.safety_flags.product_global_read_path_changed);
    assert!(paths.observation_report_path.exists());
}

#[test]
fn sqlite_production_observation_db_stable_success_verifies_two_samples() {
    let paths = prepare_production_paths("db-stable");
    copy_a11_fixture_to_root(&paths.fallback_root);
    apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None).expect("temp db");
    let db_hash = file_hash(&paths.db_path).expect("db hash");
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let report = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(&db_hash),
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect("db stable observation");

    assert_eq!(report.status, "stable_verified");
    assert_eq!(report.observation_source, "db_limited_observation");
    assert!(!report.degraded);
    assert!(report.safety_flags.production_observation_enabled);
    assert_eq!(report.samples.len(), 2);
    assert_eq!(report.samples[0].export_hash, report.samples[1].export_hash);
    assert_eq!(
        report.samples[0].projection_hash,
        report.samples[1].projection_hash
    );
    assert!(report.export_verification.is_some());
    assert!(paths.projection_root.join("runtime-logs.v1.json").exists());
    assert!(!paths.projection_root.join("runtime-log.v1.json").exists());
}

#[test]
fn sqlite_production_observation_db_unavailable_falls_back_degraded() {
    let paths = prepare_production_paths("db-unavailable");
    copy_a11_fixture_to_root(&paths.fallback_root);
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let report = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        Some(SqliteProductionObservationFailurePoint::DbUnavailable),
    )
    .expect("db unavailable fallback");

    assert_eq!(report.status, "fallback_degraded");
    assert_eq!(report.observation_source, "json_fallback");
    assert!(report.fallback_decision.contains("db_unavailable"));
    assert!(!report.safety_flags.production_observation_enabled);
}

#[test]
fn sqlite_production_observation_schema_mismatch_falls_back_degraded() {
    let paths = prepare_production_paths("schema-mismatch");
    copy_a11_fixture_to_root(&paths.fallback_root);
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let report = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        Some(SqliteProductionObservationFailurePoint::DbSchemaMismatch),
    )
    .expect("schema mismatch fallback");

    assert_eq!(report.status, "fallback_degraded");
    assert!(report.fallback_decision.contains("db_schema_mismatch"));
}

#[test]
fn sqlite_production_observation_integrity_failure_falls_back_degraded() {
    let paths = prepare_production_paths("integrity-failure");
    copy_a11_fixture_to_root(&paths.fallback_root);
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let report = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        Some(SqliteProductionObservationFailurePoint::DbIntegrityFailure),
    )
    .expect("integrity failure fallback");

    assert_eq!(report.status, "fallback_degraded");
    assert!(report.fallback_decision.contains("db_integrity_failure"));
}

#[test]
fn sqlite_production_observation_hash_projection_manifest_and_drift_failures_block() {
    for (name, failure, expected) in [
        (
            "db-hash",
            SqliteProductionObservationFailurePoint::DbHashMismatch,
            "production_observation_blocked:db_hash_mismatch",
        ),
        (
            "fallback-hash",
            SqliteProductionObservationFailurePoint::FallbackHashMismatch,
            "production_observation_blocked:fallback_hash_mismatch",
        ),
        (
            "export-hash",
            SqliteProductionObservationFailurePoint::ExportHashMismatch,
            "production_observation_blocked:export_hash_mismatch",
        ),
        (
            "projection-missing",
            SqliteProductionObservationFailurePoint::ProjectionFileMissing,
            "production_observation_blocked:projection_file_missing",
        ),
        (
            "projection-corrupt",
            SqliteProductionObservationFailurePoint::ProjectionFileCorrupt,
            "production_observation_blocked:projection_file_corrupt",
        ),
        (
            "drift",
            SqliteProductionObservationFailurePoint::ObservationDriftBetweenSamples,
            "production_observation_blocked:projection_hash_drift",
        ),
        (
            "manifest-missing",
            SqliteProductionObservationFailurePoint::RollbackManifestMissing,
            "production_observation_blocked:rollback_manifest_missing",
        ),
        (
            "manifest-incomplete",
            SqliteProductionObservationFailurePoint::RollbackManifestIncomplete,
            "production_observation_blocked:rollback_manifest_incomplete",
        ),
    ] {
        let paths = prepare_production_paths(name);
        copy_a11_fixture_to_root(&paths.fallback_root);
        apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None).expect("temp db");
        let db_hash = file_hash(&paths.db_path).expect("db hash");
        let fallback_hash = expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_production_observation_level_a(
            LEVEL_A_OBSERVATION_MODE,
            true,
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            Some(&db_hash),
            Some(&fallback_hash),
            &allowed_production_read_models(),
            &[],
            Some(failure),
        )
        .expect_err("failure must block");

        assert!(err.contains(expected), "{name} unexpected error: {err}");
        assert!(
            !paths.observation_report_path.exists(),
            "{name} report leaked"
        );
    }
}

#[test]
fn sqlite_production_observation_mid_commit_failures_leave_no_stable_report() {
    for (name, failure, expected) in [
        (
            "after-first-sample",
            SqliteProductionObservationFailurePoint::AfterFirstSampleBeforeSecondSample,
            "after_first_sample_before_second_sample",
        ),
        (
            "after-fallback",
            SqliteProductionObservationFailurePoint::AfterFallbackSelectedBeforeReportCommit,
            "after_fallback_selected_before_report_commit",
        ),
        (
            "after-rollback",
            SqliteProductionObservationFailurePoint::AfterRollbackSelectedBeforeReportCommit,
            "after_rollback_selected_before_report_commit",
        ),
    ] {
        let paths = prepare_production_paths(name);
        copy_a11_fixture_to_root(&paths.fallback_root);
        if failure
            != SqliteProductionObservationFailurePoint::AfterFallbackSelectedBeforeReportCommit
        {
            apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None)
                .expect("temp db");
        }
        let db_hash = if paths.db_path.exists() {
            Some(file_hash(&paths.db_path).expect("db hash"))
        } else {
            None
        };
        let fallback_hash = expected_fallback_hash(&paths.fallback_root);

        let err = rehearse_production_observation_level_a(
            LEVEL_A_OBSERVATION_MODE,
            true,
            WORKFLOW_STATE_SUMMARY_READ_MODEL,
            &paths.db_path,
            &paths.fallback_root,
            &paths.projection_root,
            &paths.observation_report_path,
            &paths.rollback_manifest_path,
            db_hash.as_deref(),
            Some(&fallback_hash),
            &allowed_production_read_models(),
            &[],
            Some(failure),
        )
        .expect_err("mid commit failure");

        assert!(err.contains(expected), "{name} unexpected error: {err}");
        assert!(
            !paths.observation_report_path.exists(),
            "{name} report leaked"
        );
    }
}

#[test]
fn sqlite_production_observation_sensitive_redaction_omits_forbidden_values() {
    let paths = prepare_production_paths("sensitive");
    copy_a11_fixture_to_root(&paths.fallback_root);
    apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None).expect("temp db");
    let db_hash = file_hash(&paths.db_path).expect("db hash");
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(&db_hash),
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect("sensitive production observation");

    let mut text = fs::read_to_string(&paths.observation_report_path).expect("report");
    text.push_str(&fs::read_to_string(&paths.rollback_manifest_path).expect("manifest"));
    for entry in fs::read_dir(&paths.projection_root).expect("projection") {
        let entry = entry.expect("projection entry");
        if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
            text.push_str(&fs::read_to_string(entry.path()).expect("projection file"));
        }
    }
    for forbidden in [
        "provider credential value",
        "full transcript body",
        "rollout body payload",
        "\"prompt_body\"",
        "\"token\"",
        "\"secret\"",
    ] {
        assert!(
            !text.contains(forbidden),
            "forbidden value leaked: {forbidden}"
        );
    }
    assert!(text.contains("prompt_body:omitted"));
}

#[test]
fn sqlite_production_observation_idempotent_rerun_keeps_report_text() {
    let paths = prepare_production_paths("idempotent");
    copy_a11_fixture_to_root(&paths.fallback_root);
    apply_fixture_dir_to_temp_db(&paths.fallback_root, &paths.db_path, None).expect("temp db");
    let db_hash = file_hash(&paths.db_path).expect("db hash");
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(&db_hash),
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect("first production observation");
    let first = fs::read_to_string(&paths.observation_report_path).expect("first report");
    rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        true,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(&db_hash),
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect("second production observation");
    let second = fs::read_to_string(&paths.observation_report_path).expect("second report");

    assert_eq!(first, second);
}

#[test]
fn sqlite_production_observation_level_b_accepts_confirmed_non_temp_db_and_matches_fallback() {
    let paths = prepare_level_b_observation_paths("level-b-db-success");
    prepare_level_b_observation_fallback_and_db(&paths);
    let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
    let expected_fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let source_hash_before = dir_hash(&paths.fallback_root).expect("source hash before");
    let db_hash_before = file_hash(&paths.db_path).expect("db hash before");

    let db_report = run_level_b_observation(
        &paths,
        true,
        Some(&expected_db_hash),
        Some(&expected_fallback_hash),
        None,
    )
    .expect("level b db observation success");
    let source_hash_after = dir_hash(&paths.fallback_root).expect("source hash after");
    let db_hash_after = file_hash(&paths.db_path).expect("db hash after");

    assert!(!paths.db_path.starts_with(std::env::temp_dir()));
    assert_eq!(db_report.level, LEVEL_B_WORKBENCH_OWNED_STATE);
    assert_eq!(db_report.status, "stable_verified");
    assert_eq!(db_report.observation_source, "db_limited_observation");
    assert_eq!(db_report.actual_fallback_hash, expected_fallback_hash);
    assert!(!db_report.projection_hash.is_empty());
    assert_eq!(db_report.samples.len(), 2);
    assert_eq!(
        db_report.samples[0].export_hash,
        db_report.samples[1].export_hash
    );
    assert_eq!(
        db_report.samples[0].projection_hash,
        db_report.samples[1].projection_hash
    );
    assert_eq!(source_hash_before, source_hash_after);
    assert_eq!(db_hash_before, db_hash_after);
    assert_observation_safety_flags_all_false(&db_report);

    let manifest_text =
        fs::read_to_string(&paths.rollback_manifest_path).expect("read rollback manifest");
    assert!(manifest_text.contains(LEVEL_B_WORKBENCH_OWNED_STATE));
}

#[test]
fn sqlite_production_observation_level_b_rejects_invalid_confirmed_inputs() {
    let paths = prepare_level_b_observation_paths("level-b-invalid-inputs");
    prepare_level_b_observation_fallback_and_db(&paths);
    let fallback_hash = expected_fallback_hash(&paths.fallback_root);

    let err = rehearse_production_observation_level_b_workbench_owned_state(
        LEVEL_A_OBSERVATION_MODE,
        false,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.work_dir.join("other.sqlite"),
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&fallback_hash),
        &allowed_production_read_models(),
        &[],
        &level_b_observation_config(&paths),
        None,
    )
    .expect_err("unconfirmed db path must reject");
    assert!(err.contains("production_observation_level_b_confirmed_path_mismatch:db_path"));

    let mut inside_source_paths = paths.clone();
    inside_source_paths.projection_root = inside_source_paths.fallback_root.join("projection");
    inside_source_paths.rollback_manifest_path = inside_source_paths
        .fallback_root
        .join("rollback")
        .join("production-observation-rollback-manifest.json");
    inside_source_paths.observation_report_path = inside_source_paths
        .fallback_root
        .join("reports")
        .join("production-observation-report.json");
    expect_level_b_observation_err(
        &inside_source_paths,
        Some(&file_hash(&inside_source_paths.db_path).expect("db hash")),
        Some(&fallback_hash),
        None,
        "production_observation_blocked:path_inside_fallback_root_denied",
    );

    let mut inside_db_paths = paths.clone();
    let db_dir = inside_db_paths
        .db_path
        .parent()
        .expect("db dir")
        .to_path_buf();
    inside_db_paths.projection_root = db_dir.join("projection");
    inside_db_paths.rollback_manifest_path = db_dir
        .join("rollback")
        .join("production-observation-rollback-manifest.json");
    inside_db_paths.observation_report_path = db_dir
        .join("reports")
        .join("production-observation-report.json");
    expect_level_b_observation_err(
        &inside_db_paths,
        Some(&file_hash(&inside_db_paths.db_path).expect("db hash")),
        Some(&fallback_hash),
        None,
        "production_observation_blocked:path_inside_db_dir_denied",
    );

    let mut outside_report_paths = paths.clone();
    outside_report_paths.observation_report_path = paths
        .work_dir
        .parent()
        .expect("work dir parent")
        .join("outside-work")
        .join("production-observation-report.json");
    expect_level_b_observation_err(
        &outside_report_paths,
        Some(&file_hash(&outside_report_paths.db_path).expect("db hash")),
        Some(&fallback_hash),
        None,
        "production_observation_blocked:path_outside_confirmed_work_dir",
    );

    expect_level_b_observation_err(
        &paths,
        Some("sha256:not-the-confirmed-db"),
        Some(&fallback_hash),
        None,
        "production_observation_blocked:db_hash_mismatch",
    );
    assert!(!paths.observation_report_path.exists());

    let expected_db_hash = file_hash(&paths.db_path).expect("db hash");
    expect_level_b_observation_err(
        &paths,
        Some(&expected_db_hash),
        Some(&fallback_hash),
        Some(SqliteProductionObservationFailurePoint::ObservationDriftBetweenSamples),
        "production_observation_blocked:projection_hash_drift",
    );
    assert!(!paths.observation_report_path.exists());

    let err = rehearse_production_observation_level_a(
        LEVEL_A_OBSERVATION_MODE,
        false,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
        Some(&expected_fallback_hash(&paths.fallback_root)),
        &allowed_production_read_models(),
        &[],
        None,
    )
    .expect_err("Level-A must still reject non-temp DB paths");

    assert!(err.contains("temp_db_path_required"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
#[ignore = "requires explicit R3 B3 observation authorization and confirmed paths"]
fn r3_b3_observation_confirmed_paths_requires_env_authorization() {
    let confirmation = std::env::var("R3_B3_OBSERVATION_CONFIRM")
        .expect("R3_B3_OBSERVATION_CONFIRM is required for real B3 observation");
    assert_eq!(confirmation, "CONFIRMED_USER_PRESENT_2026_06_15");
    let canonical_env = |name: &str| {
        let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} is required"));
        fs::canonicalize(&value)
            .unwrap_or_else(|error| panic!("canonicalize {name} failed for {value}: {error}"))
    };
    let db_path = canonical_env("R3_B3_DB_PATH");
    let fallback_root = canonical_env("R3_B3_FALLBACK_ROOT");
    let work_dir = canonical_env("R3_B3_WORK_DIR");
    let expected_db_hash = std::env::var("R3_B3_EXPECTED_DB_HASH")
        .expect("R3_B3_EXPECTED_DB_HASH is required for real B3 observation");
    let expected_fallback_hash = std::env::var("R3_B3_EXPECTED_FALLBACK_HASH")
        .expect("R3_B3_EXPECTED_FALLBACK_HASH is required for real B3 observation");
    let projection_root = work_dir.join("projection");
    let observation_report_path = work_dir
        .join("reports")
        .join("production-observation-report.json");
    let rollback_manifest_path = work_dir
        .join("rollback")
        .join("production-observation-rollback-manifest.json");

    let paths = LevelBObservationPaths {
        db_path: db_path.clone(),
        fallback_root: fallback_root.clone(),
        work_dir: work_dir.clone(),
        projection_root: projection_root.clone(),
        rollback_manifest_path: rollback_manifest_path.clone(),
        observation_report_path,
    };
    let fallback_root_hash_before = dir_hash(&fallback_root).expect("fallback hash before");
    let db_hash_before = file_hash(&db_path).expect("db hash before");

    let report = run_level_b_observation(
        &paths,
        true,
        Some(&expected_db_hash),
        Some(&expected_fallback_hash),
        None,
    )
    .expect("R3 B3 flag-on db observation must complete");
    let fallback_root_hash_after = dir_hash(&fallback_root).expect("fallback hash after");
    let db_hash_after = file_hash(&db_path).expect("db hash after");

    assert_eq!(db_hash_before, expected_db_hash);
    assert_eq!(db_hash_after, expected_db_hash);
    assert_eq!(fallback_root_hash_before, fallback_root_hash_after);
    assert_eq!(report.status, "stable_verified");
    assert_eq!(report.observation_source, "db_limited_observation");
    assert_eq!(
        report.actual_db_hash.as_deref(),
        Some(expected_db_hash.as_str())
    );
    assert_eq!(report.actual_fallback_hash, expected_fallback_hash);
    assert!(!report.projection_hash.is_empty());
    assert_eq!(report.samples.len(), 2);
    assert_eq!(
        report.samples[0].projection_hash,
        report.samples[1].projection_hash
    );
    assert_observation_safety_flags_all_false(&report);
    println!(
            "R3_B3_DB_PATH={}\nR3_B3_DB_HASH_BEFORE={db_hash_before}\nR3_B3_DB_HASH_AFTER={db_hash_after}\nR3_B3_FALLBACK_ROOT={}\nR3_B3_FALLBACK_ROOT_HASH_BEFORE={fallback_root_hash_before}\nR3_B3_FALLBACK_ROOT_HASH_AFTER={fallback_root_hash_after}\nR3_B3_PROJECTION_HASH={}\nR3_B3_OBSERVATION_REPORT_PATH={}\nR3_B3_ROLLBACK_MANIFEST_PATH={}",
            db_path.display(),
            fallback_root.display(),
            report.projection_hash,
            paths.observation_report_path.display(),
            rollback_manifest_path.display()
        );
}

#[test]
fn sqlite_observation_stable_verifies_two_samples_and_writes_report() {
    let fixture = fixture_dir("r3-a5", "observation-export-valid-core-chain");
    let paths = prepare_paths("stable");

    let report = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("stable observation");

    assert_eq!(report.observation_status, "stable_verified");
    assert!(report.stable_verified);
    assert!(!report.degraded);
    assert_eq!(report.sample_one.export_hash, report.sample_two.export_hash);
    assert_eq!(
        report.sample_one.projection_hash,
        report.sample_two.projection_hash
    );
    assert!(paths.observation_report_path.exists());
    assert!(paths.rollback_manifest_path.exists());
    assert!(paths.projection_root.join("runtime-logs.v1.json").exists());
    assert!(!paths.projection_root.join("runtime-log.v1.json").exists());
}

#[test]
fn sqlite_observation_idempotent_rerun_keeps_stable_report_text() {
    let fixture = fixture_dir("r3-a5", "observation-export-idempotent-rerun");
    let paths = prepare_paths("idempotent");

    rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("first observation");
    let first_report = fs::read_to_string(&paths.observation_report_path).expect("first report");
    rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("second observation");
    let second_report = fs::read_to_string(&paths.observation_report_path).expect("second report");

    assert_eq!(first_report, second_report);
}

#[test]
fn sqlite_observation_export_hash_mismatch_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-export-hash-mismatch-blocked");
    let paths = prepare_paths("export-hash-mismatch");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::ExportHashMismatch),
    )
    .expect_err("export hash mismatch must block");

    assert!(err.contains("observation_blocked:export_hash_mismatch"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_projection_missing_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-projection-missing-blocked");
    let paths = prepare_paths("projection-missing");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::ProjectionFileMissing),
    )
    .expect_err("missing projection must block");

    assert!(err.contains("observation_blocked:projection_file_missing"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_projection_corrupt_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-projection-missing-blocked");
    let paths = prepare_paths("projection-corrupt");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::ProjectionFileCorrupt),
    )
    .expect_err("corrupt projection must block");

    assert!(err.contains("observation_blocked:projection_file_corrupt"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_missing_manifest_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-manifest-missing-blocked");
    let paths = prepare_paths("missing-manifest");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::RollbackManifestMissing),
    )
    .expect_err("missing manifest must block");

    assert!(err.contains("observation_blocked:rollback_manifest_missing"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_incomplete_manifest_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-manifest-incomplete-blocked");
    let paths = prepare_paths("incomplete-manifest");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::RollbackManifestIncomplete),
    )
    .expect_err("incomplete manifest must block");

    assert!(err.contains("observation_blocked:rollback_manifest_incomplete"));
    assert!(!paths.observation_report_path.exists());
    assert!(!paths.rollback_manifest_path.exists());
}

#[test]
fn sqlite_observation_db_integrity_failure_is_degraded_and_has_no_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-db-integrity-failure-degraded");
    let paths = prepare_paths("db-integrity");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::DbIntegrityOrSchemaMismatch),
    )
    .expect_err("db integrity failure must degrade");

    assert!(err.contains("observation_degraded"));
    assert!(!err.contains("stable_verified"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_drift_between_samples_blocks_without_stable_report() {
    let fixture = fixture_dir("r3-a5", "observation-export-valid-core-chain");
    let paths = prepare_paths("drift");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::ObservationDriftBetweenSamples),
    )
    .expect_err("observation drift must block");

    assert!(err.contains("observation_blocked:projection_hash_drift"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_failure_before_sample_creates_no_outputs() {
    let fixture = fixture_dir("r3-a5", "observation-export-valid-core-chain");
    let paths = prepare_paths("before-sample");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::BeforeObservationSample),
    )
    .expect_err("before sample failure");

    assert!(err.contains("injected_failure_before_observation_sample"));
    assert!(!paths.db_path.exists());
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_failure_after_first_export_before_second_sample_creates_no_report() {
    let fixture = fixture_dir("r3-a5", "observation-export-valid-core-chain");
    let paths = prepare_paths("after-first-export");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::AfterFirstExportBeforeSecondSample),
    )
    .expect_err("after first export failure");

    assert!(err.contains("after_first_export_before_second_sample"));
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_failure_after_rollback_selected_before_report_commit_creates_no_report() {
    let fixture = fixture_dir("r3-a5", "rollback-export-recovery-verification-dry-run");
    let paths = prepare_paths("rollback-before-report");

    let err = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        Some(SqliteObservationFailurePoint::AfterRollbackSelectedBeforeReportCommit),
    )
    .expect_err("after rollback selected failure");

    assert!(err.contains("after_rollback_selected_before_report_commit"));
    assert!(paths.rollback_manifest_path.exists());
    assert!(!paths.observation_report_path.exists());
}

#[test]
fn sqlite_observation_rollback_verification_is_dry_run_only() {
    let fixture = fixture_dir("r3-a5", "rollback-export-recovery-verification-dry-run");
    let paths = prepare_paths("rollback-dry-run");

    let report = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("rollback dry-run");

    assert_eq!(
        report.rollback_recovery_verification.status,
        "rollback_recovery_verification_dry_run_only"
    );
    assert!(
        report
            .rollback_recovery_verification
            .would_disable_db_read_cut
    );
    assert!(
        report
            .rollback_recovery_verification
            .would_use_last_verified_json_projection
    );
    assert!(
        report
            .rollback_recovery_verification
            .would_preserve_db_for_audit
    );
    assert!(
        report
            .rollback_recovery_verification
            .would_require_supervisor_decision
    );
    assert!(
        !report
            .rollback_recovery_verification
            .production_restore_performed
    );
    let manifest_text = fs::read_to_string(&paths.rollback_manifest_path).expect("read manifest");
    assert!(manifest_text.contains("\"production_restore_performed\":false"));
    assert!(!manifest_text.contains("\"production_restore_performed\":true"));
}

#[test]
fn sqlite_observation_report_projection_and_manifest_omit_forbidden_sensitive_fields() {
    let fixture = fixture_dir("r3-a5", "observation-sensitive-redaction");
    let paths = prepare_paths("sensitive");

    rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("sensitive observation");

    let mut text =
        fs::read_to_string(&paths.observation_report_path).expect("read observation report");
    text.push_str(&fs::read_to_string(&paths.rollback_manifest_path).expect("read manifest"));
    for entry in fs::read_dir(&paths.projection_root).expect("read projection") {
        let entry = entry.expect("projection entry");
        if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
            text.push_str(&fs::read_to_string(entry.path()).expect("read projection file"));
        }
    }
    assert!(!text.contains("provider credential value"));
    assert!(!text.contains("full transcript body"));
    assert!(!text.contains("rollout body payload"));
    assert!(!text.contains("\"prompt_body\""));
    assert!(text.contains("prompt_body:omitted"));
}

#[test]
fn sqlite_observation_export_records_per_file_verification_fields() {
    let fixture = fixture_dir("r3-a5", "observation-export-valid-core-chain");
    let paths = prepare_paths("per-file");

    let report = rehearse_fixture_observation_period(
        &fixture,
        &paths.db_path,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        None,
    )
    .expect("per-file verification");

    for file in &report.export_verification.projected_files {
        assert!(!file.path.is_empty());
        assert!(file.hash.len() >= 64);
        assert!(file.record_count > 0 || file.path == "workflow-state.v0.json");
        assert_eq!(file.redaction_status, "forbidden_sensitive_fields_omitted");
    }
    assert!(report
        .export_verification
        .projected_files
        .iter()
        .any(|file| file.path == "runtime-logs.v1.json"));
    assert!(!report
        .export_verification
        .projected_files
        .iter()
        .any(|file| file.path == "runtime-log.v1.json"));
}

struct ObservationPaths {
    db_path: PathBuf,
    projection_root: PathBuf,
    observation_report_path: PathBuf,
    rollback_manifest_path: PathBuf,
}

struct ProductionObservationPaths {
    db_path: PathBuf,
    fallback_root: PathBuf,
    projection_root: PathBuf,
    observation_report_path: PathBuf,
    rollback_manifest_path: PathBuf,
}

#[derive(Clone)]
struct LevelBObservationPaths {
    db_path: PathBuf,
    fallback_root: PathBuf,
    work_dir: PathBuf,
    projection_root: PathBuf,
    observation_report_path: PathBuf,
    rollback_manifest_path: PathBuf,
}

fn prepare_paths(name: &str) -> ObservationPaths {
    let projection_root = temp_projection_root(name);
    ObservationPaths {
        db_path: temp_db(name),
        observation_report_path: projection_root.join("observation-report.json"),
        rollback_manifest_path: projection_root.join("rollback-manifest.json"),
        projection_root,
    }
}

fn prepare_production_paths(name: &str) -> ProductionObservationPaths {
    let nanos = unique_nanos();
    let fallback_root = std::env::temp_dir().join(format!("r3-a11-fallback-{name}-{nanos}"));
    let projection_root = std::env::temp_dir().join(format!("r3-a11-projection-{name}-{nanos}"));
    ProductionObservationPaths {
        db_path: std::env::temp_dir().join(format!("r3-a11-{name}-{nanos}.sqlite")),
        fallback_root,
        observation_report_path: projection_root.join("production-observation-report.json"),
        rollback_manifest_path: projection_root
            .join("production-observation-rollback-manifest.json"),
        projection_root,
    }
}

fn prepare_level_b_observation_paths(name: &str) -> LevelBObservationPaths {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("r3-b3a-{name}-{}", unique_nanos()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create level b root");
    let root = fs::canonicalize(root).expect("canonical level b root");
    let fallback_root = root.join("source");
    let work_dir = root.join("work");
    let db_dir = root.join("db");
    fs::create_dir_all(&work_dir).expect("create work dir");
    fs::create_dir_all(&db_dir).expect("create db dir");
    LevelBObservationPaths {
        db_path: db_dir.join("workbench-state.v1.sqlite"),
        projection_root: work_dir.join("projection"),
        rollback_manifest_path: work_dir
            .join("rollback")
            .join("production-observation-rollback-manifest.json"),
        observation_report_path: work_dir
            .join("reports")
            .join("production-observation-report.json"),
        fallback_root,
        work_dir,
    }
}

fn copy_a11_fixture_to_root(root: &Path) {
    fs::create_dir_all(root).expect("create fallback root");
    let fixture_root = fixture_dir("r3-a11", "production-observation-workflow-summary");
    for file_name in ["workflow-state.v0.json", "runtime-logs.v1.json"] {
        fs::copy(fixture_root.join(file_name), root.join(file_name))
            .unwrap_or_else(|error| panic!("copy fallback fixture {file_name}: {error}"));
    }
}

fn prepare_level_b_observation_fallback_and_db(paths: &LevelBObservationPaths) {
    copy_a11_fixture_to_root(&paths.fallback_root);
    crate::workbench_sqlite_apply::apply_confirmed_workbench_state_root_to_confirmed_db(
        &paths.fallback_root,
        &paths.fallback_root,
        &paths.db_path,
        &paths.db_path,
        None,
    )
    .expect("prepare level b confirmed db");
}

fn level_b_observation_config(paths: &LevelBObservationPaths) -> SqliteObservationLevelBConfig {
    SqliteObservationLevelBConfig {
        confirmed_db_path: paths.db_path.clone(),
        confirmed_fallback_root: paths.fallback_root.clone(),
        confirmed_work_dir: paths.work_dir.clone(),
        confirmed_projection_root: paths.projection_root.clone(),
        confirmed_rollback_manifest_path: paths.rollback_manifest_path.clone(),
        confirmed_observation_report_path: paths.observation_report_path.clone(),
    }
}

fn run_level_b_observation(
    paths: &LevelBObservationPaths,
    feature_flag_enabled: bool,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
) -> Result<SqliteProductionObservationReport, String> {
    rehearse_production_observation_level_b_workbench_owned_state(
        LEVEL_A_OBSERVATION_MODE,
        feature_flag_enabled,
        WORKFLOW_STATE_SUMMARY_READ_MODEL,
        &paths.db_path,
        &paths.fallback_root,
        &paths.projection_root,
        &paths.observation_report_path,
        &paths.rollback_manifest_path,
        expected_db_hash,
        expected_fallback_hash,
        &allowed_production_read_models(),
        &[],
        &level_b_observation_config(paths),
        failure_point,
    )
}

fn expect_level_b_observation_err(
    paths: &LevelBObservationPaths,
    expected_db_hash: Option<&str>,
    expected_fallback_hash: Option<&str>,
    failure_point: Option<SqliteProductionObservationFailurePoint>,
    expected: &str,
) {
    let err = run_level_b_observation(
        paths,
        true,
        expected_db_hash,
        expected_fallback_hash,
        failure_point,
    )
    .expect_err("level b observation must reject");
    assert!(err.contains(expected));
}

fn assert_observation_safety_flags_all_false(report: &SqliteProductionObservationReport) {
    let flags = serde_json::to_value(&report.safety_flags).expect("safety flags");
    for value in flags.as_object().expect("safety flag object").values() {
        assert_eq!(value.as_bool(), Some(false));
    }
}

fn expected_fallback_hash(root: &Path) -> String {
    let workflow = load_production_workflow_state_from_root(root).expect("fallback workflow");
    let summary = workflow_state_summary(&workflow).expect("summary");
    canonical_json_hash(&summary)
}

fn allowed_production_read_models() -> BTreeSet<String> {
    BTreeSet::from([WORKFLOW_STATE_SUMMARY_READ_MODEL.to_string()])
}

fn temp_db(name: &str) -> PathBuf {
    let nanos = unique_nanos();
    std::env::temp_dir().join(format!("r3-a5-{name}-{nanos}.sqlite"))
}

fn temp_projection_root(name: &str) -> PathBuf {
    let nanos = unique_nanos();
    std::env::temp_dir().join(format!("r3-a5-{name}-{nanos}"))
}

fn unique_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos()
}
