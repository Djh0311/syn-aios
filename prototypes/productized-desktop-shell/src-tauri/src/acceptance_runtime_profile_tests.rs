use super::acceptance_runtime_profile::{
    resolve_paths_with_context, validate_profile_candidate_lexically, ProfileBuild,
    ProfileProcessState, ProfileValidationContext, PREPARED_ROOT_ENTRY_NAMES, PROFILE_FILENAME,
    PROFILE_PURPOSE, PROFILE_SCHEMA_VERSION,
};
use super::SessionSourceMode;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const FIXED_NOW_MS: i64 = 1_784_000_000_000;
static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct AcceptanceFixture {
    root: PathBuf,
}

impl AcceptanceFixture {
    fn new(label: &str) -> Self {
        let temp_root = fs::canonicalize(std::env::temp_dir()).expect("canonical test temp root");
        for _ in 0..16 {
            let sequence = FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let root = temp_root.join(format!(
                "syn-r4-acceptance-test-{label}-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    #[cfg(unix)]
                    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                        .expect("private fixture root");
                    return Self {
                        root: fs::canonicalize(root).expect("canonical fixture root"),
                    };
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture root failed: {error}"),
            }
        }
        panic!("unable to create unique acceptance fixture root");
    }

    fn manifest_path(&self) -> PathBuf {
        self.root.join(PROFILE_FILENAME)
    }

    fn write_manifest(&self, value: &Value) -> PathBuf {
        let path = self.manifest_path();
        fs::write(
            &path,
            serde_json::to_vec(value).expect("serialize manifest"),
        )
        .expect("write fixture manifest");
        path
    }

    #[cfg(unix)]
    fn owner_uid(&self) -> u32 {
        fs::metadata(&self.root).expect("fixture metadata").uid()
    }

    #[cfg(not(unix))]
    fn owner_uid(&self) -> u32 {
        0
    }

    fn context(&self) -> ProfileValidationContext {
        ProfileValidationContext {
            build: ProfileBuild::Debug,
            now_ms: FIXED_NOW_MS,
            current_uid: self.owner_uid(),
        }
    }
}

impl Drop for AcceptanceFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn valid_manifest(fixture: &AcceptanceFixture, run_id: &str) -> Value {
    valid_manifest_for_root(&fixture.root, run_id)
}

fn valid_manifest_for_root(root: &Path, run_id: &str) -> Value {
    let project_root = root.join(format!("fixture/SYN R4 ISOLATED ACCEPTANCE {run_id}"));
    let project_root = project_root
        .to_str()
        .expect("fixture project root must be UTF-8");
    json!({
        "schema_version": PROFILE_SCHEMA_VERSION,
        "purpose": PROFILE_PURPOSE,
        "run_id": run_id,
        "expires_at_ms": FIXED_NOW_MS + 60_000,
        "project": {
            "id": super::project_id(project_root),
            "relative_path": format!("fixture/SYN R4 ISOLATED ACCEPTANCE {run_id}"),
        },
        "workflow": {
            "id": super::default_workflow_id(project_root),
        },
        "paths": {
            "index_relative_path": "fixture/codex-index.json",
            "tasks_relative_path": "fixture/tasks.md",
            "workflow_state_relative_path": "workflow-state/workflow-state.v0.json",
            "app_data_relative_path": "app-data",
            "canvas_relative_path": "app-data/canvas-v1",
            "codex_db_relative_path": "codex-db/state.sqlite",
        },
    })
}

fn prepare_valid_fixture(fixture: &AcceptanceFixture) {
    let timestamp = "2026-07-24T00:00:00.000Z";
    let fixture_dir = fixture.root.join("fixture");
    fs::create_dir(&fixture_dir).expect("fixture directory");
    let project_directory = fixture_dir.join(format!("SYN R4 ISOLATED ACCEPTANCE {}", run_id()));
    fs::create_dir(&project_directory).expect("fixture project");
    let project_root = project_directory
        .to_str()
        .expect("fixture project root must be UTF-8");
    let project_id = super::project_id(project_root);
    let workflow_id = super::default_workflow_id(project_root);
    fs::write(
        fixture_dir.join("codex-index.json"),
        serde_json::to_vec(&json!({
            "generated_at": timestamp,
            "projects": [{
                "project_root": project_root,
                "active_hint": true,
                "thread_count": 0,
                "active_thread_count": 0,
                "archived_thread_count": 0,
                "authority_files": [],
                "handoff_files": [],
                "evidence_files": [],
                "harness_candidates": [],
                "harness_resources": [],
                "context_warnings": [],
                "warnings": [],
            }],
            "threads": [],
            "skills": [],
            "plugins": [],
            "warnings": [],
        }))
        .expect("fixture index JSON"),
    )
    .expect("fixture index");
    fs::write(fixture_dir.join("tasks.md"), "").expect("fixture tasks");
    for directory in ["workflow-state", "app-data", "codex-db", "logs"] {
        fs::create_dir(fixture.root.join(directory)).expect("allowed prepared fixture directory");
    }
    fs::write(
        fixture.root.join("workflow-state/workflow-state.v0.json"),
        serde_json::to_vec(&json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "revision": 0,
            "workspace_id": format!("workspace:{}", run_id()),
            "created_at": timestamp,
            "updated_at": timestamp,
            "source_kind": "isolated_acceptance_fixture",
            "permission_level": "user_confirmed_write",
            "projects": [{
                "project_id": project_id,
                "display_name": format!("SYN R4 ISOLATED ACCEPTANCE {}", run_id()),
                "root_path": project_root,
                "source_kind": "codex_index",
                "permission_level": "read_only",
                "created_at": timestamp,
                "updated_at": timestamp,
                "warnings": [],
            }],
            "agent_adapters": [],
            "workflows": [{
                "workflow_id": workflow_id,
                "workflow_version": 1,
                "project_id": super::project_id(project_root),
                "title": format!("SYN R4 ISOLATED ACCEPTANCE {} workflow", run_id()),
                "state": "draft",
                "source_kind": "isolated_acceptance_fixture",
                "permission_level": "user_confirmed_write",
                "model_policy": "none",
                "created_at": timestamp,
                "updated_at": timestamp,
            }],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "workflow_node_session_bindings": [],
            "workflow_node_dispatches": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": [],
        }))
        .expect("minimal workflow state JSON"),
    )
    .expect("minimal workflow state");
}

fn run_id() -> &'static str {
    "syn-r4-0123456789abcdef"
}

fn assert_error(
    result: Result<Option<super::acceptance_runtime_profile::RuntimePaths>, String>,
    expected: &str,
) {
    assert_eq!(result.expect_err("profile must fail closed"), expected);
}

fn assert_contained(root: &Path, path: &Path, label: &str) {
    assert!(
        path.strip_prefix(root).is_ok(),
        "{label} must remain below the isolated root: {}",
        path.display()
    );
}

fn parse_launcher_string_constant(source: &str, name: &str) -> Result<String, String> {
    let declaration = format!("const {name} =");
    if source.match_indices(&declaration).count() != 1 {
        return Err(format!("launcher constant {name} must occur exactly once"));
    }
    let (_, after_declaration) = source
        .split_once(&declaration)
        .ok_or_else(|| format!("launcher constant {name} missing"))?;
    let (literal, _) = after_declaration
        .split_once(';')
        .ok_or_else(|| format!("launcher constant {name} must end with semicolon"))?;
    serde_json::from_str::<String>(literal.trim())
        .map_err(|_| format!("launcher constant {name} must be a string literal"))
}

fn parse_launcher_prelaunch_root_entry_names(source: &str) -> Result<Vec<String>, String> {
    const DECLARATION: &str = "const PRELAUNCH_ROOT_ENTRY_NAMES = [";
    if source.match_indices(DECLARATION).count() != 1 {
        return Err("launcher prelaunch root declaration must occur exactly once".to_string());
    }
    let (_, after_declaration) = source
        .split_once(DECLARATION)
        .ok_or_else(|| "launcher prelaunch root declaration missing".to_string())?;
    let (body, _) = after_declaration
        .split_once("];")
        .ok_or_else(|| "launcher prelaunch root declaration must close with ];".to_string())?;

    body.lines()
        .filter_map(|line| {
            let expression = line.trim();
            (!expression.is_empty()).then_some(expression)
        })
        .map(|expression| {
            let expression = expression.strip_suffix(',').ok_or_else(|| {
                format!("launcher prelaunch root entry must end with comma: {expression}")
            })?;
            if expression == "PROFILE_FILE_NAME" {
                return parse_launcher_string_constant(source, "PROFILE_FILE_NAME");
            }
            serde_json::from_str::<String>(expression).map_err(|_| {
                format!(
                    "launcher prelaunch root entry must be a known string expression: {expression}"
                )
            })
        })
        .collect()
}

fn compare_prelaunch_root_entry_sets(
    launcher_entries: &[String],
    rust_entries: &[&str],
) -> Result<(), String> {
    let launcher_set = launcher_entries.iter().cloned().collect::<BTreeSet<_>>();
    if launcher_set.len() != launcher_entries.len() {
        return Err("launcher prelaunch root entries must not contain duplicates".to_string());
    }
    let rust_set = rust_entries
        .iter()
        .map(|entry| (*entry).to_string())
        .collect::<BTreeSet<_>>();
    if rust_set.len() != rust_entries.len() {
        return Err("Rust prelaunch root entries must not contain duplicates".to_string());
    }
    if launcher_entries.len() != rust_entries.len() {
        return Err(format!(
            "prelaunch root entry count mismatch: launcher={} Rust={}",
            launcher_entries.len(),
            rust_entries.len()
        ));
    }
    if launcher_set != rust_set {
        return Err(format!(
            "prelaunch root entry set mismatch: launcher={launcher_set:?} Rust={rust_set:?}"
        ));
    }
    Ok(())
}

fn parse_launcher_build_arguments(source: &str) -> Result<Vec<String>, String> {
    const CALL_PREFIX: &str = "buildResult = await runChild(\n      tauriCliPath,\n      ";
    if source.match_indices(CALL_PREFIX).count() != 1 {
        return Err("launcher build invocation must occur exactly once".to_string());
    }
    let (_, after_call_prefix) = source
        .split_once(CALL_PREFIX)
        .ok_or_else(|| "launcher build invocation missing".to_string())?;
    let (array_expression, _) = after_call_prefix
        .split_once("\n      {")
        .ok_or_else(|| "launcher build arguments must precede child options".to_string())?;
    let array_expression = array_expression
        .trim()
        .strip_suffix(',')
        .ok_or_else(|| "launcher build arguments must end with comma".to_string())?;
    let body = array_expression
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| "launcher build arguments must be an array".to_string())?;

    body.split(',')
        .map(str::trim)
        .filter(|expression| !expression.is_empty())
        .map(|expression| {
            if expression == "BUNDLE_BUILD_CONFIG" {
                return parse_launcher_string_constant(source, "BUNDLE_BUILD_CONFIG");
            }
            serde_json::from_str::<String>(expression).map_err(|_| {
                format!("launcher build argument must be a string expression: {expression}")
            })
        })
        .collect()
}

fn parse_cargo_package_name(source: &str) -> Result<String, String> {
    let (_, after_package_header) = source
        .split_once("[package]\n")
        .ok_or_else(|| "Cargo manifest package section missing".to_string())?;
    let (package_body, _) = after_package_header
        .split_once("\n[")
        .ok_or_else(|| "Cargo manifest package section must terminate".to_string())?;
    let package_names = package_body
        .lines()
        .filter_map(|line| line.trim().strip_prefix("name = "))
        .collect::<Vec<_>>();
    if package_names.len() != 1 {
        return Err("Cargo manifest must declare exactly one package name".to_string());
    }
    serde_json::from_str::<String>(package_names[0])
        .map_err(|_| "Cargo package name must be a string literal".to_string())
}

fn assert_launcher_discovery_bundle_contract(source: &str) -> Result<(), String> {
    let tauri_config = serde_json::from_str::<Value>(include_str!("../tauri.conf.json"))
        .map_err(|_| "Tauri config must remain valid JSON".to_string())?;
    let product_name = tauri_config
        .get("productName")
        .and_then(Value::as_str)
        .ok_or_else(|| "Tauri config productName must be a string".to_string())?;
    let bundle_identifier = tauri_config
        .get("identifier")
        .and_then(Value::as_str)
        .ok_or_else(|| "Tauri config identifier must be a string".to_string())?;
    let cargo_package_name = parse_cargo_package_name(include_str!("../Cargo.toml"))?;
    let bundle_name = parse_launcher_string_constant(source, "DEBUG_APP_BUNDLE_NAME")?;
    let launcher_bundle_identifier =
        parse_launcher_string_constant(source, "DEBUG_APP_BUNDLE_IDENTIFIER")?;
    let executable_relative_path =
        parse_launcher_string_constant(source, "DEBUG_APP_EXECUTABLE_RELATIVE_PATH")?;
    let bundle_build_config = parse_launcher_string_constant(source, "BUNDLE_BUILD_CONFIG")?;

    if bundle_name != product_name {
        return Err("launcher bundle name must match Tauri productName".to_string());
    }
    if launcher_bundle_identifier != bundle_identifier {
        return Err("launcher bundle identifier must match Tauri identifier".to_string());
    }
    let expected_executable_relative_path = format!(
        "src-tauri/target/debug/bundle/macos/{product_name}.app/Contents/MacOS/{cargo_package_name}"
    );
    if executable_relative_path != expected_executable_relative_path {
        return Err(
            "launcher must direct-spawn the current debug app bundle executable".to_string(),
        );
    }
    if serde_json::from_str::<Value>(&bundle_build_config).ok()
        != Some(json!({ "bundle": { "active": true } }))
    {
        return Err("launcher bundle build config must activate only this build".to_string());
    }

    let expected_build_arguments = vec![
        "build".to_string(),
        "--debug".to_string(),
        "--bundles".to_string(),
        "app".to_string(),
        "--config".to_string(),
        bundle_build_config,
    ];
    if parse_launcher_build_arguments(source)? != expected_build_arguments {
        return Err("launcher must request exactly one debug app bundle build".to_string());
    }
    if source.contains("--no-bundle") {
        return Err("launcher must not build the discovery target with --no-bundle".to_string());
    }
    for token in [
        "const debugAppExecutablePath = resolve(\n  desktopRoot,\n  DEBUG_APP_EXECUTABLE_RELATIVE_PATH,\n);",
        "async function assertFreshDebugAppExecutable(buildStartedAtMs) {",
        "const bundleBuildStartedAtMs = Date.now();",
        "await assertFreshDebugAppExecutable(bundleBuildStartedAtMs);",
        "const diagnosedLaunch = await runDiagnosedChild(\n          debugAppExecutablePath,\n          [],",
        "target_bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,",
        "delete normalBuildEnvironment[PROFILE_ENV];",
        "[PROFILE_ENV]: join(root, PROFILE_FILE_NAME),",
    ] {
        if !source.contains(token) {
            return Err(format!("launcher discovery bundle contract token missing: {token}"));
        }
    }
    Ok(())
}

fn assert_launcher_bundle_integrity_contract(source: &str) -> Result<(), String> {
    let codesign_path = parse_launcher_string_constant(source, "CODESIGN_PATH")?;
    let bundle_relative_path =
        parse_launcher_string_constant(source, "DEBUG_APP_BUNDLE_RELATIVE_PATH")?;
    let executable_relative_path =
        parse_launcher_string_constant(source, "DEBUG_APP_EXECUTABLE_RELATIVE_PATH")?;
    if codesign_path != "/usr/bin/codesign" {
        return Err("launcher must use the fixed system codesign binary".to_string());
    }
    if !executable_relative_path.starts_with(&format!("{bundle_relative_path}/Contents/MacOS/")) {
        return Err("launcher executable must remain inside the sealed app bundle".to_string());
    }
    if source.contains("--ignore-resources") {
        return Err("launcher strict verification must not ignore bundle resources".to_string());
    }
    for token in [
        "const debugAppBundlePath = resolve(\n  desktopRoot,\n  DEBUG_APP_BUNDLE_RELATIVE_PATH,\n);",
        "async function sealAndVerifyDebugAppBundle(environment) {",
        "[\"--force\", \"--deep\", \"--sign\", \"-\", debugAppBundlePath],",
        "[\"--verify\", \"--deep\", \"--strict\", debugAppBundlePath],",
        "!sealResult.launched ||\n    sealResult.exit_code !== 0 ||\n    sealResult.signal !== null",
        "!verificationResult.launched ||\n    verificationResult.exit_code !== 0 ||\n    verificationResult.signal !== null",
        "throw new Error(\"fresh debug app bundle ad-hoc seal failed\");",
        "throw new Error(\"fresh debug app bundle strict verification failed\");",
        "await sealAndVerifyDebugAppBundle(normalBuildEnvironment);",
        "failureStage = \"bundle_integrity\";",
    ] {
        if !source.contains(token) {
            return Err(format!("launcher bundle integrity contract token missing: {token}"));
        }
    }
    let freshness_check = source
        .find("await assertFreshDebugAppExecutable(bundleBuildStartedAtMs);")
        .ok_or_else(|| "launcher fresh executable check missing".to_string())?;
    let seal_and_verify = source
        .find("await sealAndVerifyDebugAppBundle(normalBuildEnvironment);")
        .ok_or_else(|| "launcher bundle seal/verify call missing".to_string())?;
    let final_launch = source
        .find("const diagnosedLaunch = await runDiagnosedChild(")
        .ok_or_else(|| "launcher final launch missing".to_string())?;
    if !(freshness_check < seal_and_verify && seal_and_verify < final_launch) {
        return Err(
            "launcher must seal and strictly verify the fresh bundle before final launch"
                .to_string(),
        );
    }
    Ok(())
}

fn assert_launcher_pre_list_sigkill_diagnostic_contract(source: &str) -> Result<(), String> {
    const PARENT_SIGNAL_DECLARATION: &str =
        "const PARENT_CAPTURE_SIGNALS = [\"SIGTERM\", \"SIGINT\", \"SIGHUP\"];";
    if source.match_indices(PARENT_SIGNAL_DECLARATION).count() != 1 {
        return Err(
            "parent signal ledger must declare exactly the fixed signal set once".to_string(),
        );
    }
    for token in [
        "const PRE_LIST_SIGKILL_DIAGNOSTIC_SCHEMA_VERSION = 1;",
        PARENT_SIGNAL_DECLARATION,
        "function createPreListSigkillDiagnostic() {",
        "launcher_child_kill_attempted: false,",
        "launcher_self_signal_reraise_after_receipt: false,",
        "child_exit: pendingChildLifecycle(),",
        "child_close: pendingChildLifecycle(),",
        "process_relation: unavailableProcessRelation(),",
        "function observeParentChildProcessRelation(parentPid, childPid) {",
        "\"pid=,ppid=,pgid=,sess=\"",
        "function installParentSignalLedger() {",
        "child.once(\"exit\", (code, signal) => {",
        "child.once(\"close\", (code, signal) => {",
        "const parentSignalLedger = installParentSignalLedger();",
        "const diagnosedLaunch = await runDiagnosedChild(",
        "pre_list_sigkill_diagnostic: preListSigkillDiagnostic,",
        "process.kill(process.pid, parentSignalToReraise);",
    ] {
        if !source.contains(token) {
            return Err(format!(
                "pre-list SIGKILL diagnostic contract token missing: {token}"
            ));
        }
    }
    let diagnostic_body = source
        .split_once("function createPreListSigkillDiagnostic() {")
        .and_then(|(_, remainder)| {
            remainder.split_once("function parseParentChildProcessRelation(")
        })
        .map(|(body, _)| body)
        .ok_or_else(|| "pre-list SIGKILL diagnostic body must remain delimited".to_string())?;
    for forbidden_token in [
        "pid:",
        "path:",
        "command",
        "environment",
        "stdout",
        "stderr",
    ] {
        if diagnostic_body.contains(forbidden_token) {
            return Err(format!(
                "pre-list SIGKILL diagnostic body must not retain {forbidden_token}"
            ));
        }
    }
    for forbidden_token in [
        "child.kill(",
        "raw_ps_output:",
        "parent_pid:",
        "child_pid:",
        "command_line:",
        "environment:",
        "stdout:",
        "stderr:",
    ] {
        if source.contains(forbidden_token) {
            return Err(format!(
                "pre-list SIGKILL diagnostic must not persist {forbidden_token}"
            ));
        }
    }
    Ok(())
}

#[test]
fn acceptance_runtime_profile_valid_manifest_resolves_every_app_owned_path_under_one_root() {
    let fixture = AcceptanceFixture::new("paths");
    let manifest_path = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let paths = resolve_paths_with_context(Some(&manifest_path), fixture.context())
        .expect("valid profile")
        .expect("profile must activate");

    assert_eq!(paths.root, fixture.root);
    assert_eq!(
        paths.index_path,
        fixture.root.join("fixture/codex-index.json")
    );
    assert_eq!(paths.tasks_path, fixture.root.join("fixture/tasks.md"));
    assert_eq!(
        paths.project_root,
        fixture
            .root
            .join(format!("fixture/SYN R4 ISOLATED ACCEPTANCE {}", run_id()))
    );
    assert_eq!(
        paths.workflow_state_path,
        fixture.root.join("workflow-state/workflow-state.v0.json")
    );
    assert_eq!(paths.app_data_root, fixture.root.join("app-data"));
    assert_eq!(
        paths.vault_root,
        fixture.root.join("app-data/knowledge-vault")
    );
    assert_eq!(
        paths.recovery_backups_root,
        fixture.root.join("app-data/knowledge-workspace-recovery")
    );
    assert_eq!(paths.canvas_root, fixture.root.join("app-data/canvas-v1"));
    assert_eq!(
        paths.codex_db_path,
        fixture.root.join("codex-db/state.sqlite")
    );
    assert_eq!(paths.app_log_dir, fixture.root.join("logs"));
    for (label, path) in [
        ("index", &paths.index_path),
        ("tasks", &paths.tasks_path),
        ("project", &paths.project_root),
        ("workflow", &paths.workflow_state_path),
        ("app-data", &paths.app_data_root),
        ("vault", &paths.vault_root),
        ("recovery", &paths.recovery_backups_root),
        ("canvas", &paths.canvas_root),
        ("codex-db", &paths.codex_db_path),
        ("logs", &paths.app_log_dir),
    ] {
        assert_contained(&fixture.root, path, label);
    }
    assert_eq!(paths.session_source_mode(), SessionSourceMode::IndexOnly);
}

#[test]
fn acceptance_runtime_profile_accepts_only_the_exact_prepared_synthetic_fixture_layout() {
    let fixture = AcceptanceFixture::new("prepared");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );
    prepare_valid_fixture(&fixture);
    resolve_paths_with_context(Some(&manifest), fixture.context())
        .expect("exact synthetic fixture tree is valid");

    fs::write(fixture.root.join("unexpected-root-entry"), "stale")
        .expect("own unexpected fixture entry");
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );
}

#[test]
fn acceptance_runtime_profile_schema_purpose_and_extra_fields_fail_closed() {
    let fixture = AcceptanceFixture::new("schema");
    let mut wrong_version = valid_manifest(&fixture, run_id());
    wrong_version["schema_version"] = json!(PROFILE_SCHEMA_VERSION + 1);
    assert_error(
        resolve_paths_with_context(
            Some(&fixture.write_manifest(&wrong_version)),
            fixture.context(),
        ),
        "acceptance_runtime_profile_schema_invalid",
    );

    let mut wrong_purpose = valid_manifest(&fixture, run_id());
    wrong_purpose["purpose"] = json!("other-purpose");
    assert_error(
        resolve_paths_with_context(
            Some(&fixture.write_manifest(&wrong_purpose)),
            fixture.context(),
        ),
        "acceptance_runtime_profile_schema_invalid",
    );

    let mut extra = valid_manifest(&fixture, run_id());
    extra["unapproved_path_override"] = json!("/tmp/not-allowed");
    assert_error(
        resolve_paths_with_context(Some(&fixture.write_manifest(&extra)), fixture.context()),
        "acceptance_runtime_profile_schema_invalid",
    );
}

#[test]
fn acceptance_runtime_profile_identity_and_fixed_relative_paths_are_exact() {
    let fixture = AcceptanceFixture::new("identity");
    prepare_valid_fixture(&fixture);
    for (field, replacement) in [
        ("run_id", json!("syn-r4-uppercase0123")),
        ("project.id", json!("project:other")),
        ("project.relative_path", json!("fixture/project")),
        ("workflow.id", json!("workflow:other")),
        ("paths.index_relative_path", json!("../escape.json")),
        ("paths.canvas_relative_path", json!("app-data/other-canvas")),
    ] {
        let mut manifest = valid_manifest(&fixture, run_id());
        match field {
            "run_id" => manifest["run_id"] = replacement,
            "project.id" => manifest["project"]["id"] = replacement,
            "project.relative_path" => manifest["project"]["relative_path"] = replacement,
            "workflow.id" => manifest["workflow"]["id"] = replacement,
            "paths.index_relative_path" => manifest["paths"]["index_relative_path"] = replacement,
            "paths.canvas_relative_path" => manifest["paths"]["canvas_relative_path"] = replacement,
            _ => unreachable!(),
        }
        assert_error(
            resolve_paths_with_context(Some(&fixture.write_manifest(&manifest)), fixture.context()),
            "acceptance_runtime_profile_schema_invalid",
        );
    }
}

#[test]
fn acceptance_runtime_profile_requires_direct_canonical_temp_child_and_direct_manifest() {
    let fixture = AcceptanceFixture::new("nested");
    let nested_root = fixture.root.join("syn-r4-acceptance-nested");
    fs::create_dir(&nested_root).expect("nested root");
    #[cfg(unix)]
    fs::set_permissions(&nested_root, fs::Permissions::from_mode(0o700)).expect("nested mode");
    let nested_manifest = nested_root.join(PROFILE_FILENAME);
    fs::write(
        &nested_manifest,
        serde_json::to_vec(&valid_manifest_for_root(&nested_root, run_id())).unwrap(),
    )
    .expect("nested manifest");
    assert_error(
        resolve_paths_with_context(Some(&nested_manifest), fixture.context()),
        "acceptance_runtime_profile_root_invalid",
    );

    let wrong_name = fixture.root.join("not-profile.json");
    fs::write(
        &wrong_name,
        serde_json::to_vec(&valid_manifest(&fixture, run_id())).unwrap(),
    )
    .expect("wrong-name manifest");
    assert_error(
        resolve_paths_with_context(Some(&wrong_name), fixture.context()),
        "acceptance_runtime_profile_root_invalid",
    );

    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    let noncanonical_manifest = fixture.root.join(".").join(PROFILE_FILENAME);
    assert_ne!(manifest.as_os_str(), noncanonical_manifest.as_os_str());
    assert_error(
        resolve_paths_with_context(Some(&noncanonical_manifest), fixture.context()),
        "acceptance_runtime_profile_root_invalid",
    );
}

#[test]
fn acceptance_runtime_profile_rejects_external_or_lexically_escaped_candidates_before_root_probe() {
    for candidate in [
        PathBuf::from("relative/syn-r4-acceptance-any/profile.json"),
        PathBuf::from("/Users/untrusted/syn-r4-acceptance-any/profile.json"),
        PathBuf::from("/private/tmp/syn-r4-acceptance-any/../profile.json"),
    ] {
        assert_eq!(
            validate_profile_candidate_lexically(&candidate)
                .expect_err("non-canonical candidate must fail before root metadata"),
            "acceptance_runtime_profile_root_invalid"
        );
    }

    let fixture = AcceptanceFixture::new("lexical");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    assert_eq!(
        validate_profile_candidate_lexically(&manifest).expect("canonical candidate"),
        fs::canonicalize(std::env::temp_dir()).expect("canonical test temp root")
    );
}

#[cfg(unix)]
#[test]
fn acceptance_runtime_profile_rejects_symlinks_permissions_and_wrong_owner() {
    let fixture = AcceptanceFixture::new("symlink");
    let outside = AcceptanceFixture::new("outside");
    let root_link = fixture.root.with_file_name(format!(
        "{}-root-link",
        fixture.root.file_name().unwrap().to_string_lossy()
    ));
    symlink(&fixture.root, &root_link).expect("root symlink");
    let linked_manifest = root_link.join(PROFILE_FILENAME);
    fs::write(
        fixture.manifest_path(),
        serde_json::to_vec(&valid_manifest(&fixture, run_id())).unwrap(),
    )
    .expect("fixture manifest");
    assert_error(
        resolve_paths_with_context(Some(&linked_manifest), fixture.context()),
        "acceptance_runtime_profile_symlink_rejected",
    );
    fs::remove_file(&root_link).expect("remove own symlink");

    let manifest = fixture.manifest_path();
    fs::remove_file(&manifest).expect("remove own manifest");
    let outside_manifest = outside.write_manifest(&valid_manifest(&outside, run_id()));
    symlink(&outside_manifest, &manifest).expect("manifest symlink");
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_symlink_rejected",
    );
    fs::remove_file(&manifest).expect("remove own manifest symlink");

    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o755)).expect("relax root");
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_permissions_invalid",
    );
    fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o700)).expect("restore root");

    let mut wrong_owner_context = fixture.context();
    wrong_owner_context.current_uid = wrong_owner_context.current_uid.saturating_add(1);
    assert_error(
        resolve_paths_with_context(Some(&manifest), wrong_owner_context),
        "acceptance_runtime_profile_owner_invalid",
    );
}

#[test]
fn acceptance_runtime_profile_rejects_expiry_and_nonempty_reuse() {
    let fixture = AcceptanceFixture::new("freshness");
    let mut expired = valid_manifest(&fixture, run_id());
    expired["expires_at_ms"] = json!(FIXED_NOW_MS);
    assert_error(
        resolve_paths_with_context(Some(&fixture.write_manifest(&expired)), fixture.context()),
        "acceptance_runtime_profile_expired",
    );

    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    resolve_paths_with_context(Some(&manifest), fixture.context())
        .expect("prepared synthetic fixture directories remain valid");
    fs::write(
        fixture
            .root
            .join(format!("fixture/SYN R4 ISOLATED ACCEPTANCE {}", run_id()))
            .join("leftover.txt"),
        "not a fixture",
    )
    .expect("reuse sentinel");
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );
}

#[test]
fn acceptance_runtime_profile_rejects_non_synthetic_fixture_contents() {
    let index_fixture = AcceptanceFixture::new("index-content");
    let index_manifest = index_fixture.write_manifest(&valid_manifest(&index_fixture, run_id()));
    prepare_valid_fixture(&index_fixture);
    let index_path = index_fixture.root.join("fixture/codex-index.json");
    let mut index: Value =
        serde_json::from_slice(&fs::read(&index_path).expect("read index")).expect("parse index");
    index["threads"] = json!([{"id": "unexpected-session"}]);
    fs::write(
        &index_path,
        serde_json::to_vec(&index).expect("serialize index"),
    )
    .expect("rewrite own index");
    assert_error(
        resolve_paths_with_context(Some(&index_manifest), index_fixture.context()),
        "acceptance_runtime_profile_fixture_invalid",
    );

    let tasks_fixture = AcceptanceFixture::new("tasks-content");
    let tasks_manifest = tasks_fixture.write_manifest(&valid_manifest(&tasks_fixture, run_id()));
    prepare_valid_fixture(&tasks_fixture);
    fs::write(
        tasks_fixture.root.join("fixture/tasks.md"),
        "unexpected task body\n",
    )
    .expect("rewrite own tasks");
    assert_error(
        resolve_paths_with_context(Some(&tasks_manifest), tasks_fixture.context()),
        "acceptance_runtime_profile_fixture_invalid",
    );

    let workflow_fixture = AcceptanceFixture::new("workflow-content");
    let workflow_manifest =
        workflow_fixture.write_manifest(&valid_manifest(&workflow_fixture, run_id()));
    prepare_valid_fixture(&workflow_fixture);
    let workflow_path = workflow_fixture
        .root
        .join("workflow-state/workflow-state.v0.json");
    let mut workflow: Value =
        serde_json::from_slice(&fs::read(&workflow_path).expect("read workflow"))
            .expect("parse workflow");
    workflow["workflows"][0]["workflow_id"] = json!("workflow:unexpected:default");
    fs::write(
        &workflow_path,
        serde_json::to_vec(&workflow).expect("serialize workflow"),
    )
    .expect("rewrite own workflow");
    assert_error(
        resolve_paths_with_context(Some(&workflow_manifest), workflow_fixture.context()),
        "acceptance_runtime_profile_fixture_invalid",
    );
}

#[cfg(unix)]
#[test]
fn acceptance_runtime_profile_rejects_hardlinked_profile_and_fixture_files() {
    let fixture = AcceptanceFixture::new("hardlink");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let link_holder = AcceptanceFixture::new("hardlink-holder");
    for (label, source) in [
        ("profile", manifest.clone()),
        ("index", fixture.root.join("fixture/codex-index.json")),
        ("tasks", fixture.root.join("fixture/tasks.md")),
        (
            "workflow",
            fixture.root.join("workflow-state/workflow-state.v0.json"),
        ),
    ] {
        let link = link_holder.root.join(format!("{label}.link"));
        fs::hard_link(&source, &link).expect("create own test hardlink");
        assert_error(
            resolve_paths_with_context(Some(&manifest), fixture.context()),
            "acceptance_runtime_profile_hardlink_rejected",
        );
        fs::remove_file(&link).expect("remove own test hardlink");
    }
}

#[test]
fn acceptance_runtime_profile_process_state_is_immutable_and_uninitialized_getters_fail_closed() {
    let fixture = AcceptanceFixture::new("process");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let mut state = ProfileProcessState::default();

    assert_eq!(
        state
            .active_paths()
            .expect_err("uninitialized getter must close"),
        "acceptance_runtime_profile_uninitialized"
    );
    assert_eq!(
        state
            .isolated_log_dir()
            .expect_err("uninitialized log getter must close"),
        "acceptance_runtime_profile_uninitialized"
    );
    assert_eq!(
        state
            .active_paths_for_profile_env(true)
            .expect_err("profile env without initialization must not fall back"),
        "acceptance_runtime_profile_uninitialized"
    );
    assert_eq!(
        state
            .active_paths_for_profile_env(false)
            .expect("normal mode remains available without profile env"),
        None
    );

    state
        .initialize_from_manifest(Some(&manifest), fixture.context())
        .expect("first initialization");
    let active = state
        .active_paths()
        .expect("initialized getter")
        .expect("isolated profile");
    assert_eq!(active.root, fixture.root);
    assert_eq!(
        state.isolated_log_dir().expect("isolated log getter"),
        Some(fixture.root.join("logs"))
    );
    assert_eq!(
        state
            .initialize_from_manifest(Some(&manifest), fixture.context())
            .expect_err("second initialization must fail"),
        "acceptance_runtime_profile_duplicate_initialization"
    );

    let invalid_fixture = AcceptanceFixture::new("invalid-process");
    let mut invalid_manifest = valid_manifest(&invalid_fixture, run_id());
    invalid_manifest["purpose"] = json!("invalid");
    let invalid_manifest = invalid_fixture.write_manifest(&invalid_manifest);
    let mut invalid_state = ProfileProcessState::default();
    assert_eq!(
        invalid_state
            .initialize_from_manifest(Some(&invalid_manifest), invalid_fixture.context())
            .expect_err("invalid profile initialization must fail"),
        "acceptance_runtime_profile_schema_invalid"
    );
    assert_eq!(
        invalid_state
            .active_paths()
            .expect_err("failed initialization must not leave a usable state"),
        "acceptance_runtime_profile_uninitialized"
    );
}

#[test]
fn acceptance_runtime_profile_normal_mode_stays_normal_and_non_debug_rejects_profile() {
    let fixture = AcceptanceFixture::new("build-mode");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));

    assert_eq!(
        resolve_paths_with_context(None, fixture.context()).expect("normal profile resolution"),
        None
    );

    let mut release_context = fixture.context();
    release_context.build = ProfileBuild::NonDebug;
    assert_error(
        resolve_paths_with_context(Some(&manifest), release_context),
        "acceptance_runtime_profile_non_debug_rejected",
    );
}

fn assert_static_order(source: &str, tokens: &[&str]) {
    let mut cursor = 0;
    for token in tokens {
        let relative = source[cursor..]
            .find(token)
            .unwrap_or_else(|| panic!("static contract token missing: {token}"));
        cursor += relative + token.len();
    }
}

#[test]
fn acceptance_runtime_profile_static_startup_order_and_consumer_wiring_contract() {
    let entrypoints = include_str!("index_host_app_entrypoints.rs");
    let run_start = entrypoints
        .find("pub fn run() {")
        .expect("run entrypoint must exist");
    assert_static_order(
        &entrypoints[run_start..],
        &[
            "crate::acceptance_runtime_profile::initialize_from_env()",
            "AppState::try_new()",
            "migrate_legacy_workflow_node_session_binding_ids_at",
            "exec_process_registry::reap_registered_orphans",
            "reap_supervisor_resident_stale_sessions_at",
            "workbench_sqlite_storage_mode::initialize_for_startup",
            "tauri::Builder::default()",
        ],
    );

    let lib = include_str!("lib.rs");
    for token in [
        "if let Some(paths) = acceptance_runtime_profile::active_paths()?",
        "index_path: paths.index_path,",
        "tasks_path: paths.tasks_path,",
        "workflow_state_path: paths.workflow_state_path,",
        "return paths.workflow_state_path;",
        "acceptance_runtime_profile::session_source_mode_for_process()",
        "build_snapshot_with_session_source(state, index, tasks_text, session_source_mode_for_process())",
    ] {
        assert!(
            lib.contains(token),
            "lib profile wiring token missing: {token}"
        );
    }

    let codex_db = include_str!("codex_db.rs");
    assert!(
        codex_db.contains("return paths.codex_db_path;"),
        "Codex DB must route active profile to the isolated DB path"
    );

    let knowledge_vault = include_str!("knowledge_vault.rs");
    for token in [
        "return paths.app_data_root;",
        "return paths.workflow_state_path;",
    ] {
        assert!(
            knowledge_vault.contains(token),
            "knowledge vault profile wiring token missing: {token}"
        );
    }

    let canvas_storage = include_str!("mcp/storage.rs");
    assert!(
        canvas_storage.contains("return paths.canvas_root;"),
        "Canvas must route active profile to the isolated canvas root"
    );
}

#[test]
fn acceptance_runtime_profile_launcher_build_pins_home_initial_view_before_vite_loads_dotenv() {
    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    let home_assignment = launcher
        .find("normalBuildEnvironment.VITE_STAGE_K_INITIAL_VIEW = \"home\";")
        .expect("isolated build must pin Home before Vite can load dotenv");
    let build_start = launcher
        .find("buildResult = await runChild(")
        .expect("isolated launcher build invocation must exist");
    assert!(
        home_assignment < build_start,
        "isolated build must pin Home before the normal build starts"
    );
}

#[test]
fn acceptance_runtime_profile_launcher_builds_a_fresh_app_bundle_for_sky_target_discovery() {
    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    assert_launcher_discovery_bundle_contract(launcher)
        .expect("launcher must build and direct-spawn the fresh configured app bundle");

    let bundle_disabled = launcher.replacen(
        "        \"--bundles\",\n        \"app\",",
        "        \"--no-bundle\",",
        1,
    );
    assert!(
        assert_launcher_discovery_bundle_contract(&bundle_disabled).is_err(),
        "the raw --no-bundle launch path must be rejected"
    );
    let raw_binary_target = launcher.replacen(
        "const DEBUG_APP_EXECUTABLE_RELATIVE_PATH =\n  \"src-tauri/target/debug/bundle/macos/CodexGovernanceWorkbench.app/Contents/MacOS/codex-governance-workbench\";",
        "const DEBUG_APP_EXECUTABLE_RELATIVE_PATH =\n  \"src-tauri/target/debug/codex-governance-workbench\";",
        1,
    );
    assert!(
        assert_launcher_discovery_bundle_contract(&raw_binary_target).is_err(),
        "a raw debug binary target must be rejected"
    );
    let mismatched_bundle_name = launcher.replacen(
        "const DEBUG_APP_BUNDLE_NAME = \"CodexGovernanceWorkbench\";",
        "const DEBUG_APP_BUNDLE_NAME = \"UnexpectedBundle\";",
        1,
    );
    assert!(
        assert_launcher_discovery_bundle_contract(&mismatched_bundle_name).is_err(),
        "a launcher bundle name that drifts from Tauri config must be rejected"
    );
    let mismatched_bundle_identifier = launcher.replacen(
        "const DEBUG_APP_BUNDLE_IDENTIFIER = \"local.codex.governance.workbench\";",
        "const DEBUG_APP_BUNDLE_IDENTIFIER = \"local.codex.wrong\";",
        1,
    );
    assert!(
        assert_launcher_discovery_bundle_contract(&mismatched_bundle_identifier).is_err(),
        "a launcher bundle identifier that drifts from Tauri config must be rejected"
    );
    let missing_ready_identity = launcher.replacen(
        "target_bundle_identifier: DEBUG_APP_BUNDLE_IDENTIFIER,\n",
        "",
        1,
    );
    assert!(
        assert_launcher_discovery_bundle_contract(&missing_ready_identity).is_err(),
        "the discovery-ready envelope must retain its configured bundle identity"
    );
}

#[test]
fn acceptance_runtime_profile_launcher_seals_and_strictly_verifies_fresh_bundle_before_launch() {
    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    assert_launcher_bundle_integrity_contract(launcher)
        .expect("launcher must fail closed unless the fresh app bundle passes strict verification");

    let ignored_resources = launcher.replacen(
        "[\"--verify\", \"--deep\", \"--strict\", debugAppBundlePath],",
        "[\"--verify\", \"--deep\", \"--strict\", \"--ignore-resources\", debugAppBundlePath],",
        1,
    );
    assert!(
        assert_launcher_bundle_integrity_contract(&ignored_resources).is_err(),
        "strict bundle verification must not ignore the missing resource seal"
    );
    let missing_seal = launcher.replacen(
        "await sealAndVerifyDebugAppBundle(normalBuildEnvironment);",
        "",
        1,
    );
    assert!(
        assert_launcher_bundle_integrity_contract(&missing_seal).is_err(),
        "the bundle seal/verify step must not be bypassed"
    );
    let wrong_codesign = launcher.replacen(
        "const CODESIGN_PATH = \"/usr/bin/codesign\";",
        "const CODESIGN_PATH = \"codesign\";",
        1,
    );
    assert!(
        assert_launcher_bundle_integrity_contract(&wrong_codesign).is_err(),
        "the launcher must not resolve codesign through PATH"
    );
    let accepts_failed_seal = launcher.replacen(
        "sealResult.exit_code !== 0",
        "sealResult.exit_code === 0",
        1,
    );
    assert!(
        assert_launcher_bundle_integrity_contract(&accepts_failed_seal).is_err(),
        "a nonzero bundle seal result must fail closed"
    );
    let accepts_failed_verification = launcher.replacen(
        "verificationResult.exit_code !== 0",
        "verificationResult.exit_code === 0",
        1,
    );
    assert!(
        assert_launcher_bundle_integrity_contract(&accepts_failed_verification).is_err(),
        "a nonzero strict verification result must fail closed"
    );
}

#[test]
fn acceptance_runtime_profile_launcher_receipt_distinguishes_config_fixture_and_ui_observation() {
    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    for token in [
        "schema_version: \"syn_r4_isolated_preflight_receipt.v3\"",
        "home_initial_view_config_pinned:",
        "declared_fixture_path_containment:",
        "fixture_path_containment_provenance:",
        "launcher_declared_fixture_path_projection",
        "fixture_synthetic_identity_hash:",
        "profile_declared_session_source:",
        "ui_inspection_attempted: false",
        "ui_inspection_completed: false",
        "synthetic_home_verified: false",
        "screenshot_saved: false",
        "ui_inspection_failure_family: \"not_observed_by_launcher\"",
        "ui_inspection_provenance:",
        "pre_list_sigkill_diagnostic: preListSigkillDiagnostic,",
    ] {
        assert!(
            launcher.contains(token),
            "isolated launcher receipt semantic token missing: {token}"
        );
    }
    for legacy_token in ["home_initial_view_verified", "resolved_root_containment"] {
        assert!(
            !launcher.contains(legacy_token),
            "isolated launcher must not preserve the overclaimed receipt field: {legacy_token}"
        );
    }
}

#[test]
fn acceptance_runtime_profile_launcher_records_pre_list_sigkill_diagnostics_fail_closed() {
    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    assert_launcher_pre_list_sigkill_diagnostic_contract(launcher)
        .expect("launcher must persist only the fixed, redacted pre-list SIGKILL diagnostic");

    let missing_parent_signal = launcher.replacen(
        "const PARENT_CAPTURE_SIGNALS = [\"SIGTERM\", \"SIGINT\", \"SIGHUP\"];",
        "const PARENT_CAPTURE_SIGNALS = [\"SIGUSR1\", \"SIGINT\", \"SIGHUP\"];",
        1,
    );
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&missing_parent_signal).is_err(),
        "any parent-signal set drift must be rejected"
    );
    let extra_parent_signal = launcher.replacen(
        "const PARENT_CAPTURE_SIGNALS = [\"SIGTERM\", \"SIGINT\", \"SIGHUP\"];",
        "const PARENT_CAPTURE_SIGNALS = [\"SIGTERM\", \"SIGINT\", \"SIGHUP\", \"SIGUSR1\"];",
        1,
    );
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&extra_parent_signal).is_err(),
        "the parent-signal set must not silently expand"
    );
    let missing_child_close = launcher.replacen(
        "child.once(\"close\", (code, signal) => {",
        "child.once(\"disconnect\", (code, signal) => {",
        1,
    );
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&missing_child_close).is_err(),
        "the child close observer must remain mandatory"
    );
    let missing_session_projection =
        launcher.replacen("\"pid=,ppid=,pgid=,sess=\"", "\"pid=,ppid=,pgid=\"", 1);
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&missing_session_projection).is_err(),
        "the process-session projection must remain exact"
    );
    let child_kill_added = format!("{launcher}\nchild.kill(\"SIGTERM\");\n");
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&child_kill_added).is_err(),
        "the launcher must not actively terminate its final Syn child"
    );
    let raw_pid_added = format!("{launcher}\nconst leaked = {{ child_pid: 1 }};\n");
    assert!(
        assert_launcher_pre_list_sigkill_diagnostic_contract(&raw_pid_added).is_err(),
        "the diagnostic must not persist raw process identifiers"
    );
}

#[test]
fn acceptance_runtime_profile_prelaunch_layout_and_exit_contract_fail_closed() {
    let fixture = AcceptanceFixture::new("prelaunch-root-entry");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    fs::write(fixture.root.join("ui-inspection.json"), "{}")
        .expect("own root-level inspection sidecar");
    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );

    let launcher = include_str!("../../scripts/run-r4-isolated-app-preflight.mjs");
    let launcher_root_entries = parse_launcher_prelaunch_root_entry_names(launcher)
        .expect("launcher prelaunch root declaration must parse fail closed");
    let validator_source = include_str!("acceptance_runtime_profile.rs");
    let validator_body = validator_source
        .split_once("fn validate_root_layout(")
        .and_then(|(_, remainder)| remainder.split_once("fn validate_fixture_directory("))
        .map(|(body, _)| body)
        .expect("validate_root_layout body must remain present");
    assert!(
        validator_body.contains("PREPARED_ROOT_ENTRY_NAMES"),
        "Rust validator must consume the single exact prelaunch root allowlist"
    );
    compare_prelaunch_root_entry_sets(&launcher_root_entries, &PREPARED_ROOT_ENTRY_NAMES)
        .expect("launcher and Rust validator must have the same exact prelaunch root entries");

    let launcher_with_seventh = launcher.replacen(
        "  \"logs\",\n];",
        "  \"logs\",\n  \"unexpected-extra\",\n];",
        1,
    );
    let launcher_with_seventh_entries =
        parse_launcher_prelaunch_root_entry_names(&launcher_with_seventh)
            .expect("mutated launcher root declaration must still parse");
    assert!(
        compare_prelaunch_root_entry_sets(
            &launcher_with_seventh_entries,
            &PREPARED_ROOT_ENTRY_NAMES,
        )
        .expect_err("a seventh launcher root entry must be rejected")
        .contains("count mismatch"),
        "a seventh launcher root entry must fail by exact cardinality"
    );
    for (index, entry) in launcher_root_entries.iter().enumerate() {
        let entry_literal =
            serde_json::to_string(entry).expect("launcher root entry must be JSON stringable");
        let launcher_without_entry = if index == 0 {
            launcher.replacen("  PROFILE_FILE_NAME,\n", "", 1)
        } else {
            launcher.replacen(&format!("  {entry_literal},\n"), "", 1)
        };
        assert!(
            compare_prelaunch_root_entry_sets(
                &parse_launcher_prelaunch_root_entry_names(&launcher_without_entry)
                    .expect("deleted launcher root declaration must still parse"),
                &PREPARED_ROOT_ENTRY_NAMES,
            )
            .is_err(),
            "deleting launcher root entry {entry} must be rejected"
        );

        let renamed_entry = format!("renamed-{entry}");
        let renamed_entry_literal = serde_json::to_string(&renamed_entry)
            .expect("renamed launcher root entry must be JSON stringable");
        let launcher_with_renamed_entry = if index == 0 {
            launcher.replacen(
                &format!("const PROFILE_FILE_NAME = {entry_literal};"),
                &format!("const PROFILE_FILE_NAME = {renamed_entry_literal};"),
                1,
            )
        } else {
            launcher.replacen(
                &format!("  {entry_literal},\n"),
                &format!("  {renamed_entry_literal},\n"),
                1,
            )
        };
        assert!(
            compare_prelaunch_root_entry_sets(
                &parse_launcher_prelaunch_root_entry_names(&launcher_with_renamed_entry)
                    .expect("renamed launcher root declaration must still parse"),
                &PREPARED_ROOT_ENTRY_NAMES,
            )
            .is_err(),
            "renaming launcher root entry {entry} must be rejected"
        );
    }

    let mut rust_with_seventh = PREPARED_ROOT_ENTRY_NAMES.to_vec();
    rust_with_seventh.push("unexpected-extra");
    assert!(
        compare_prelaunch_root_entry_sets(&launcher_root_entries, &rust_with_seventh).is_err(),
        "a seventh Rust validator root entry must be rejected"
    );
    for entry in PREPARED_ROOT_ENTRY_NAMES {
        let rust_without_entry = PREPARED_ROOT_ENTRY_NAMES
            .iter()
            .copied()
            .filter(|candidate| *candidate != entry)
            .collect::<Vec<_>>();
        assert!(
            compare_prelaunch_root_entry_sets(&launcher_root_entries, &rust_without_entry).is_err(),
            "deleting Rust validator root entry {entry} must be rejected"
        );

        let renamed_entry = format!("renamed-{entry}");
        let rust_with_renamed_entry = PREPARED_ROOT_ENTRY_NAMES
            .iter()
            .map(|candidate| {
                if *candidate == entry {
                    renamed_entry.as_str()
                } else {
                    *candidate
                }
            })
            .collect::<Vec<_>>();
        assert!(
            compare_prelaunch_root_entry_sets(&launcher_root_entries, &rust_with_renamed_entry)
                .is_err(),
            "renaming Rust validator root entry {entry} must be rejected"
        );
    }
    for token in [
        "const UI_INSPECTION_RELATIVE_PATH = join(\"logs\", UI_INSPECTION_FILE_NAME);",
        "const uiInspectionPath = join(root, UI_INSPECTION_RELATIVE_PATH);",
        "await assertPrelaunchRootLayout(root);",
        "const ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE = 78;",
        "const ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE = 79;",
        "return \"profile_initialization_failure\";",
        "return \"app_state_initialization_failure\";",
        "return \"exit_zero_without_completed_ui_observation\";",
        "ui_observation_missing",
        "ui_observation_invalid",
    ] {
        assert!(
            launcher.contains(token),
            "isolated launcher cross-language contract token missing: {token}"
        );
    }
    for legacy_prelaunch_token in [
        "const uiInspectionPath = join(root, UI_INSPECTION_FILE_NAME);",
        "await writeJson(uiInspectionPath, pendingUiInspection(runHash));",
        "return \"normal_exit\";",
    ] {
        assert!(
            !launcher.contains(legacy_prelaunch_token),
            "isolated launcher must not retain prelaunch or exit misclassification: {legacy_prelaunch_token}"
        );
    }
    let startup_failure = launcher
        .find("const startupFailure = startupFailureFamily(launchResult);")
        .expect("fixed startup failure must be classified");
    let ui_failure = launcher
        .find("!uiInspection.ui_inspection_completed")
        .expect("missing UI observation must fail closed");
    assert!(
        startup_failure < ui_failure,
        "startup failure must be classified before missing UI observation"
    );

    let entrypoints = include_str!("index_host_app_entrypoints.rs");
    for token in [
        "const ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE: i32 = 78;",
        "const ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE: i32 = 79;",
        "fn exit_acceptance_startup_failure(exit_code: i32) -> ! {",
        "std::process::exit(exit_code);",
        "exit_acceptance_startup_failure(ACCEPTANCE_RUNTIME_PROFILE_INITIALIZATION_EXIT_CODE);",
        "exit_acceptance_startup_failure(ACCEPTANCE_APP_STATE_INITIALIZATION_EXIT_CODE);",
    ] {
        assert!(
            entrypoints.contains(token),
            "acceptance entrypoint startup failure contract token missing: {token}"
        );
    }
}

#[test]
fn first_init_allows_optional_runtime_artifacts_dir() {
    let fixture = AcceptanceFixture::new("runtime-artifacts-first-init");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let runtime_artifacts = fixture.root.join("runtime-artifacts");
    fs::create_dir(&runtime_artifacts).expect("runtime artifacts dir");
    fs::write(runtime_artifacts.join("storage-mode.v1.json"), "{}").expect("operator file");

    let paths = resolve_paths_with_context(Some(&manifest), fixture.context())
        .expect("first init with runtime-artifacts dir must resolve")
        .expect("isolated profile");
    assert_eq!(paths.root, fixture.root);
}

#[test]
fn reentry_with_matching_marker_accepts_dirty_root() {
    let fixture = AcceptanceFixture::new("reentry-dirty");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    resolve_paths_with_context(Some(&manifest), fixture.context())
        .expect("first init")
        .expect("isolated profile");

    // 模拟崩溃后的脏现场：store 已变更、备份目录、日志、runtime-artifacts + 重进标记
    fs::write(
        fixture.root.join("workflow-state/workflow-state.v0.json"),
        "{\"schema_version\":\"workflow_state_v0\",\"mutated\":true}",
    )
    .expect("mutated store");
    fs::create_dir(fixture.root.join("workflow-state/backups")).expect("backups dir");
    fs::write(fixture.root.join("logs/app.log"), "log line").expect("log file");
    let runtime_artifacts = fixture.root.join("runtime-artifacts");
    fs::create_dir(&runtime_artifacts).expect("runtime artifacts dir");
    fs::write(
        runtime_artifacts.join(".r4-initialized"),
        format!("{}\n", run_id()),
    )
    .expect("reentry marker");

    let paths = resolve_paths_with_context(Some(&manifest), fixture.context())
        .expect("reentry with matching marker must resolve")
        .expect("isolated profile");
    assert_eq!(paths.root, fixture.root);
}

#[test]
fn reentry_marker_with_wrong_run_id_is_rejected() {
    let fixture = AcceptanceFixture::new("reentry-wrong-marker");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let runtime_artifacts = fixture.root.join("runtime-artifacts");
    fs::create_dir(&runtime_artifacts).expect("runtime artifacts dir");
    fs::write(
        runtime_artifacts.join(".r4-initialized"),
        "syn-r4-0000000000000000",
    )
    .expect("foreign reentry marker");

    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );
}

#[test]
fn reentry_still_rejects_unknown_extra_entries() {
    let fixture = AcceptanceFixture::new("reentry-extra-entry");
    let manifest = fixture.write_manifest(&valid_manifest(&fixture, run_id()));
    prepare_valid_fixture(&fixture);
    let runtime_artifacts = fixture.root.join("runtime-artifacts");
    fs::create_dir(&runtime_artifacts).expect("runtime artifacts dir");
    fs::write(
        runtime_artifacts.join(".r4-initialized"),
        run_id(),
    )
    .expect("reentry marker");
    fs::write(fixture.root.join("junk.txt"), "junk").expect("unknown extra entry");

    assert_error(
        resolve_paths_with_context(Some(&manifest), fixture.context()),
        "acceptance_runtime_profile_reused",
    );
}

#[test]
fn acceptance_gates_are_inert_without_initialized_profile() {
    use super::acceptance_runtime_profile::{
        acceptance_gate_armed, acceptance_injected_failure, acceptance_wait_for_gate_release,
    };

    // 测试进程的全局 profile 状态从未初始化：门必须完全惰性（普通 App 路径无故障开关）。
    assert!(!acceptance_gate_armed("pre-commit"));
    assert!(!acceptance_gate_armed("post-commit"));
    assert!(!acceptance_gate_armed("projection-fail"));
    assert_eq!(acceptance_wait_for_gate_release("pre-commit"), Ok(()));
    assert_eq!(acceptance_injected_failure("projection-fail"), None);
}
