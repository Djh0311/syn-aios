use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::{
    ffi::OsStrExt,
    fs::{MetadataExt, PermissionsExt},
};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const PROFILE_ENV: &str = "SYN_R4_ACCEPTANCE_PROFILE";
pub(crate) const REENTRY_CAPABILITY_ENV: &str = "SYN_R4_REENTRY_CAPABILITY";
pub(crate) const PROFILE_FILENAME: &str = "profile.json";
pub(crate) const PROFILE_SCHEMA_VERSION: u64 = 1;
pub(crate) const PROFILE_PURPOSE: &str = "syn-r4-isolated-runtime-profile";

const PROFILE_ROOT_PREFIX: &str = "syn-r4-acceptance-";
const INDEX_RELATIVE_PATH: &str = "fixture/codex-index.json";
const TASKS_RELATIVE_PATH: &str = "fixture/tasks.md";
const WORKFLOW_STATE_RELATIVE_PATH: &str = "workflow-state/workflow-state.v0.json";
const APP_DATA_RELATIVE_PATH: &str = "app-data";
const CANVAS_RELATIVE_PATH: &str = "app-data/canvas-v1";
const CODEX_DB_RELATIVE_PATH: &str = "codex-db/state.sqlite";
const VAULT_DIR_NAME: &str = "knowledge-vault";
const RECOVERY_DIR_NAME: &str = "knowledge-workspace-recovery";
const LOGS_DIR_NAME: &str = "logs";
// T2 崩溃恢复扩展：runtime-artifacts 为可选第 7 项（db_primary 配置/DB、验收门、重进标记的落点）。
// .r4-initialized 只能由已通过首次启动校验的受控 App 以 launcher capability 落盘；
// 绝不接受操作者预置的 run_id 文本 marker。
const RUNTIME_ARTIFACTS_DIR_NAME: &str = "runtime-artifacts";
const REENTRY_MARKER_NAME: &str = ".r4-initialized";
const REENTRY_MARKER_SCHEMA_VERSION: &str = "syn-r4-reentry-marker.v1";
const REENTRY_CAPABILITY_HEX_LENGTH: usize = 64;
const REENTRY_MARKER_MAX_BYTES: u64 = 1024;
pub(crate) const PREPARED_ROOT_ENTRY_NAMES: [&str; 6] = [
    PROFILE_FILENAME,
    "fixture",
    "workflow-state",
    "app-data",
    "codex-db",
    LOGS_DIR_NAME,
];

const ERROR_SCHEMA: &str = "acceptance_runtime_profile_schema_invalid";
const ERROR_ROOT: &str = "acceptance_runtime_profile_root_invalid";
const ERROR_SYMLINK: &str = "acceptance_runtime_profile_symlink_rejected";
const ERROR_PERMISSIONS: &str = "acceptance_runtime_profile_permissions_invalid";
const ERROR_OWNER: &str = "acceptance_runtime_profile_owner_invalid";
const ERROR_EXPIRED: &str = "acceptance_runtime_profile_expired";
const ERROR_REUSED: &str = "acceptance_runtime_profile_reused";
const ERROR_HARDLINK: &str = "acceptance_runtime_profile_hardlink_rejected";
const ERROR_FIXTURE: &str = "acceptance_runtime_profile_fixture_invalid";
const ERROR_UNINITIALIZED: &str = "acceptance_runtime_profile_uninitialized";
const ERROR_DUPLICATE_INITIALIZATION: &str = "acceptance_runtime_profile_duplicate_initialization";
const ERROR_NON_DEBUG: &str = "acceptance_runtime_profile_non_debug_rejected";
const ERROR_REENTRY_CAPABILITY: &str = "acceptance_runtime_profile_reentry_capability_invalid";
const ERROR_REENTRY_MARKER: &str = "acceptance_runtime_profile_reentry_marker_invalid";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProfileBuild {
    Debug,
    NonDebug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProfileValidationContext {
    pub(crate) build: ProfileBuild,
    pub(crate) now_ms: i64,
    pub(crate) current_uid: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimePaths {
    pub(crate) root: PathBuf,
    pub(crate) index_path: PathBuf,
    pub(crate) tasks_path: PathBuf,
    pub(crate) project_root: PathBuf,
    pub(crate) workflow_state_path: PathBuf,
    pub(crate) app_data_root: PathBuf,
    pub(crate) vault_root: PathBuf,
    pub(crate) recovery_backups_root: PathBuf,
    pub(crate) canvas_root: PathBuf,
    pub(crate) codex_db_path: PathBuf,
    pub(crate) app_log_dir: PathBuf,
}

impl RuntimePaths {
    pub(crate) fn session_source_mode(&self) -> crate::SessionSourceMode {
        crate::SessionSourceMode::IndexOnly
    }
}

#[derive(Default)]
pub(crate) struct ProfileProcessState {
    initialized: bool,
    paths: Option<RuntimePaths>,
    pending_first_initialization: Option<PendingFirstInitialization>,
}

#[derive(Clone, Debug)]
struct PendingFirstInitialization {
    marker_path: PathBuf,
    run_id: String,
    profile_sha256: String,
    capability_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReentryMarker {
    schema_version: String,
    run_id: String,
    profile_sha256: String,
    capability_sha256: String,
}

struct StartupResolvedProfile {
    paths: RuntimePaths,
    pending_first_initialization: Option<PendingFirstInitialization>,
}

impl ProfileProcessState {
    #[cfg(test)]
    pub(crate) fn initialize_from_manifest(
        &mut self,
        profile_manifest: Option<&Path>,
        context: ProfileValidationContext,
    ) -> Result<(), String> {
        if self.initialized {
            return Err(ERROR_DUPLICATE_INITIALIZATION.to_string());
        }
        let paths = resolve_paths_with_context(profile_manifest, context)?;
        self.paths = paths;
        self.initialized = true;
        Ok(())
    }

    pub(crate) fn initialize_from_startup_manifest(
        &mut self,
        profile_manifest: &Path,
        context: ProfileValidationContext,
        reentry_capability: &str,
    ) -> Result<(), String> {
        if self.initialized {
            return Err(ERROR_DUPLICATE_INITIALIZATION.to_string());
        }
        let resolved =
            resolve_startup_profile_with_capability(profile_manifest, context, reentry_capability)?;
        self.paths = Some(resolved.paths);
        self.pending_first_initialization = resolved.pending_first_initialization;
        self.initialized = true;
        Ok(())
    }

    pub(crate) fn finalize_first_r4_initialization(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err(ERROR_UNINITIALIZED.to_string());
        }
        let Some(pending) = self.pending_first_initialization.as_ref() else {
            return Ok(());
        };
        write_first_initialization_marker(pending)?;
        self.pending_first_initialization = None;
        Ok(())
    }

    pub(crate) fn active_paths(&self) -> Result<Option<RuntimePaths>, String> {
        if !self.initialized {
            return Err(ERROR_UNINITIALIZED.to_string());
        }
        Ok(self.paths.clone())
    }

    pub(crate) fn active_paths_for_profile_env(
        &self,
        profile_env_present: bool,
    ) -> Result<Option<RuntimePaths>, String> {
        if self.initialized {
            return self.active_paths();
        }
        if profile_env_present {
            return Err(ERROR_UNINITIALIZED.to_string());
        }
        Ok(None)
    }

    pub(crate) fn isolated_log_dir(&self) -> Result<Option<PathBuf>, String> {
        Ok(self.active_paths()?.map(|paths| paths.app_log_dir))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileManifest {
    schema_version: u64,
    purpose: String,
    run_id: String,
    expires_at_ms: i64,
    project: ManifestProject,
    workflow: ManifestWorkflow,
    paths: ManifestPaths,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestProject {
    id: String,
    relative_path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestWorkflow {
    id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestPaths {
    index_relative_path: String,
    tasks_relative_path: String,
    workflow_state_relative_path: String,
    app_data_relative_path: String,
    canvas_relative_path: String,
    codex_db_relative_path: String,
}

struct PreparedFixturePaths {
    index_path: PathBuf,
    tasks_path: PathBuf,
    workflow_state_path: PathBuf,
    project_root: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticIndexFixture {
    generated_at: String,
    projects: Vec<SyntheticIndexProject>,
    threads: Vec<Value>,
    skills: Vec<Value>,
    plugins: Vec<Value>,
    warnings: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticIndexProject {
    project_root: String,
    active_hint: bool,
    thread_count: u64,
    active_thread_count: u64,
    archived_thread_count: u64,
    authority_files: Vec<Value>,
    handoff_files: Vec<Value>,
    evidence_files: Vec<Value>,
    harness_candidates: Vec<Value>,
    harness_resources: Vec<Value>,
    context_warnings: Vec<Value>,
    warnings: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticWorkflowStateFixture {
    schema_version: String,
    workflow_version: u64,
    revision: u64,
    workspace_id: String,
    created_at: String,
    updated_at: String,
    source_kind: String,
    permission_level: String,
    projects: Vec<SyntheticWorkflowProject>,
    agent_adapters: Vec<Value>,
    workflows: Vec<SyntheticWorkflow>,
    nodes: Vec<Value>,
    edges: Vec<Value>,
    work_items: Vec<Value>,
    artifacts: Vec<Value>,
    reviews: Vec<Value>,
    workflow_node_session_bindings: Vec<Value>,
    workflow_node_dispatches: Vec<Value>,
    audit_events: Vec<Value>,
    capabilities: Vec<Value>,
    harness_resources: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticWorkflowProject {
    project_id: String,
    display_name: String,
    root_path: String,
    source_kind: String,
    permission_level: String,
    created_at: String,
    updated_at: String,
    warnings: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticWorkflow {
    workflow_id: String,
    workflow_version: u64,
    project_id: String,
    title: String,
    state: String,
    source_kind: String,
    permission_level: String,
    model_policy: String,
    created_at: String,
    updated_at: String,
}

#[cfg(test)]
pub(crate) fn resolve_paths_with_context(
    profile_manifest: Option<&Path>,
    context: ProfileValidationContext,
) -> Result<Option<RuntimePaths>, String> {
    let Some(profile_manifest) = profile_manifest else {
        return Ok(None);
    };
    if context.build != ProfileBuild::Debug {
        return Err(ERROR_NON_DEBUG.to_string());
    }

    let (root, manifest_path) = validate_profile_location(profile_manifest, context.current_uid)?;
    let manifest = read_and_validate_manifest(&manifest_path, context.now_ms)?;
    if is_reentry(&root, &manifest)? {
        let project_root = validate_reentry_layout(&root, &manifest)?;
        validate_manifest_runtime_identity(&manifest, &project_root)?;
        return Ok(Some(runtime_paths_from_manifest(root, project_root)));
    }
    let fixture = validate_root_layout(&root, &manifest)?;
    validate_manifest_runtime_identity(&manifest, &fixture.project_root)?;
    validate_fixture_contents(&fixture, &manifest)?;
    Ok(Some(runtime_paths_from_manifest(
        root,
        fixture.project_root,
    )))
}

/// Production startup uses a stronger reentry contract than the pure resolver
/// retained for old fixture-shape tests.  A real R4 process must receive the
/// launcher-created one-time capability and can only reenter through the
/// marker that this process writes after the first successful startup.
fn resolve_startup_profile_with_capability(
    profile_manifest: &Path,
    context: ProfileValidationContext,
    reentry_capability: &str,
) -> Result<StartupResolvedProfile, String> {
    if context.build != ProfileBuild::Debug {
        return Err(ERROR_NON_DEBUG.to_string());
    }
    let capability_sha256 = reentry_capability_sha256(reentry_capability)?;
    let (root, manifest_path) = validate_profile_location(profile_manifest, context.current_uid)?;
    let manifest = read_and_validate_manifest(&manifest_path, context.now_ms)?;
    let profile_sha256 = crate::utils::hash::sha256_hex_bytes(
        &fs::read(&manifest_path).map_err(|_| ERROR_SCHEMA.to_string())?,
    );
    if reentry_marker_matches(
        &root,
        &manifest,
        &profile_sha256,
        &capability_sha256,
        context.current_uid,
    )? {
        let project_root = validate_reentry_layout(&root, &manifest)?;
        validate_manifest_runtime_identity(&manifest, &project_root)?;
        return Ok(StartupResolvedProfile {
            paths: runtime_paths_from_manifest(root, project_root),
            pending_first_initialization: None,
        });
    }
    let fixture = validate_root_layout(&root, &manifest)?;
    validate_manifest_runtime_identity(&manifest, &fixture.project_root)?;
    validate_fixture_contents(&fixture, &manifest)?;
    let marker_path = root
        .join(RUNTIME_ARTIFACTS_DIR_NAME)
        .join(REENTRY_MARKER_NAME);
    Ok(StartupResolvedProfile {
        paths: runtime_paths_from_manifest(root, fixture.project_root),
        pending_first_initialization: Some(PendingFirstInitialization {
            marker_path,
            run_id: manifest.run_id,
            profile_sha256,
            capability_sha256,
        }),
    })
}

fn reentry_capability_sha256(reentry_capability: &str) -> Result<String, String> {
    if reentry_capability.len() != REENTRY_CAPABILITY_HEX_LENGTH
        || !reentry_capability
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ERROR_REENTRY_CAPABILITY.to_string());
    }
    Ok(crate::utils::hash::sha256_hex(reentry_capability))
}

fn reentry_marker_matches(
    root: &Path,
    manifest: &ProfileManifest,
    profile_sha256: &str,
    capability_sha256: &str,
    expected_uid: u32,
) -> Result<bool, String> {
    let marker = root
        .join(RUNTIME_ARTIFACTS_DIR_NAME)
        .join(REENTRY_MARKER_NAME);
    let metadata = match fs::symlink_metadata(&marker) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(ERROR_REENTRY_MARKER.to_string()),
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > REENTRY_MARKER_MAX_BYTES
    {
        return Err(ERROR_REENTRY_MARKER.to_string());
    }
    #[cfg(unix)]
    {
        if metadata.nlink() != 1 {
            return Err(ERROR_HARDLINK.to_string());
        }
        if metadata.uid() != expected_uid {
            return Err(ERROR_OWNER.to_string());
        }
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(ERROR_PERMISSIONS.to_string());
        }
    }
    let marker: ReentryMarker =
        serde_json::from_slice(&fs::read(&marker).map_err(|_| ERROR_REENTRY_MARKER.to_string())?)
            .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    if marker.schema_version != REENTRY_MARKER_SCHEMA_VERSION
        || marker.run_id != manifest.run_id
        || marker.profile_sha256 != profile_sha256
        || marker.capability_sha256 != capability_sha256
    {
        return Err(ERROR_REUSED.to_string());
    }
    Ok(true)
}

fn write_first_initialization_marker(pending: &PendingFirstInitialization) -> Result<(), String> {
    let parent = pending
        .marker_path
        .parent()
        .ok_or_else(|| ERROR_REENTRY_MARKER.to_string())?;
    match fs::create_dir(parent) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(ERROR_REENTRY_MARKER.to_string()),
    }
    let parent_metadata =
        fs::symlink_metadata(parent).map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(ERROR_REENTRY_MARKER.to_string());
    }
    #[cfg(unix)]
    {
        if parent_metadata.uid() != effective_uid()? {
            return Err(ERROR_REENTRY_MARKER.to_string());
        }
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    }
    let marker = ReentryMarker {
        schema_version: REENTRY_MARKER_SCHEMA_VERSION.to_string(),
        run_id: pending.run_id.clone(),
        profile_sha256: pending.profile_sha256.clone(),
        capability_sha256: pending.capability_sha256.clone(),
    };
    let bytes = serde_json::to_vec(&marker).map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > REENTRY_MARKER_MAX_BYTES {
        return Err(ERROR_REENTRY_MARKER.to_string());
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending.marker_path)
        .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    #[cfg(unix)]
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| ERROR_REENTRY_MARKER.to_string())?;
    Ok(())
}

/// 崩溃重进判定：marker 存在即进入重进模式；内容必须是同 run_id，否则按复用拒绝。
/// harness 从不写 marker——由验收操作者在首次校验通过后显式落盘。
#[cfg(test)]
fn is_reentry(root: &Path, manifest: &ProfileManifest) -> Result<bool, String> {
    let marker = root
        .join(RUNTIME_ARTIFACTS_DIR_NAME)
        .join(REENTRY_MARKER_NAME);
    if !marker.exists() {
        return Ok(false);
    }
    let recorded = fs::read_to_string(&marker).map_err(|_| ERROR_REUSED.to_string())?;
    if recorded.trim() != manifest.run_id {
        return Err(ERROR_REUSED.to_string());
    }
    Ok(true)
}

/// 重进模式布局校验：身份与落点仍强校验（目录/文件类型、symlink、project dir 与 run_id 对应），
/// 但不再要求全新夹具内容与空目录——崩溃后的脏 store 正是验收对象。
fn validate_reentry_layout(root: &Path, manifest: &ProfileManifest) -> Result<PathBuf, String> {
    let entries = read_direct_entries(root)?;
    for path in entries.values() {
        reject_symlink(path)?;
    }
    for required in PREPARED_ROOT_ENTRY_NAMES {
        require_directory_or_file(entries.get(required), required)?;
    }
    if let Some(runtime_artifacts) = entries.get(RUNTIME_ARTIFACTS_DIR_NAME) {
        require_directory(Some(runtime_artifacts))?;
    }
    let extra = entries
        .keys()
        .filter(|name| {
            !PREPARED_ROOT_ENTRY_NAMES.contains(&name.as_str())
                && name.as_str() != RUNTIME_ARTIFACTS_DIR_NAME
        })
        .count();
    if extra > 0 {
        return Err(ERROR_REUSED.to_string());
    }

    let fixture_dir = entries
        .get("fixture")
        .ok_or_else(|| ERROR_REUSED.to_string())?;
    require_single_link_regular_file_at(fixture_dir.join("codex-index.json"))?;
    require_single_link_regular_file_at(fixture_dir.join("tasks.md"))?;
    let project_dir = format!("SYN R4 ISOLATED ACCEPTANCE {}", manifest.run_id);
    let project_path = fixture_dir.join(&project_dir);
    require_directory(Some(&project_path))?;
    let canonical_project_root =
        fs::canonicalize(&project_path).map_err(|_| ERROR_FIXTURE.to_string())?;
    let canonical_fixture_dir =
        fs::canonicalize(fixture_dir).map_err(|_| ERROR_FIXTURE.to_string())?;
    if canonical_project_root.parent() != Some(canonical_fixture_dir.as_path()) {
        return Err(ERROR_FIXTURE.to_string());
    }

    let workflow_state_file = entries
        .get("workflow-state")
        .ok_or_else(|| ERROR_REUSED.to_string())?
        .join("workflow-state.v0.json");
    require_single_link_regular_file_at(workflow_state_file)?;
    Ok(canonical_project_root)
}

fn require_directory_or_file(path: Option<&PathBuf>, name: &str) -> Result<(), String> {
    let path = path.ok_or_else(|| ERROR_REUSED.to_string())?;
    let metadata = fs::symlink_metadata(path).map_err(|_| ERROR_REUSED.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    let expect_dir = name != PROFILE_FILENAME;
    if metadata.is_dir() != expect_dir {
        return Err(ERROR_REUSED.to_string());
    }
    Ok(())
}

fn require_single_link_regular_file_at(path: PathBuf) -> Result<(), String> {
    let metadata = fs::symlink_metadata(&path).map_err(|_| ERROR_REUSED.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    if !metadata.is_file() {
        return Err(ERROR_REUSED.to_string());
    }
    #[cfg(unix)]
    if metadata.nlink() != 1 {
        return Err(ERROR_HARDLINK.to_string());
    }
    Ok(())
}

pub(crate) fn initialize_from_env() -> Result<(), String> {
    let Some(profile_path) = std::env::var_os(PROFILE_ENV).map(PathBuf::from) else {
        return Ok(());
    };
    let reentry_capability =
        std::env::var(REENTRY_CAPABILITY_ENV).map_err(|_| ERROR_REENTRY_CAPABILITY.to_string())?;
    let context = production_context()?;
    let mut state = process_state()
        .lock()
        .map_err(|_| ERROR_UNINITIALIZED.to_string())?;
    state.initialize_from_startup_manifest(&profile_path, context, &reentry_capability)
}

/// The marker is deliberately written only after AppState and DB-primary
/// startup reconciliation have both succeeded.  A preseeded marker or a
/// startup that fails before this point cannot acquire reentry eligibility.
pub(crate) fn finalize_first_r4_initialization() -> Result<(), String> {
    let mut state = process_state()
        .lock()
        .map_err(|_| ERROR_UNINITIALIZED.to_string())?;
    state.finalize_first_r4_initialization()
}

pub(crate) fn active_paths() -> Result<Option<RuntimePaths>, String> {
    let state = process_state()
        .lock()
        .map_err(|_| ERROR_UNINITIALIZED.to_string())?;
    state.active_paths_for_profile_env(std::env::var_os(PROFILE_ENV).is_some())
}

pub(crate) fn isolated_log_dir() -> Result<Option<PathBuf>, String> {
    Ok(active_paths()?.map(|paths| paths.app_log_dir))
}

pub(crate) fn session_source_mode_for_process() -> Result<crate::SessionSourceMode, String> {
    Ok(match active_paths()? {
        Some(paths) => paths.session_source_mode(),
        None => crate::SessionSourceMode::RealWithSqliteFallback,
    })
}

fn process_state() -> &'static Mutex<ProfileProcessState> {
    static STATE: OnceLock<Mutex<ProfileProcessState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(ProfileProcessState::default()))
}

fn production_context() -> Result<ProfileValidationContext, String> {
    Ok(ProfileValidationContext {
        build: if cfg!(debug_assertions) {
            ProfileBuild::Debug
        } else {
            ProfileBuild::NonDebug
        },
        now_ms: unix_timestamp_ms(),
        current_uid: effective_uid()?,
    })
}

fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(unix)]
fn effective_uid() -> Result<u32, String> {
    extern "C" {
        fn geteuid() -> u32;
    }
    // SAFETY: geteuid has no arguments and returns the effective UID of this process.
    Ok(unsafe { geteuid() })
}

#[cfg(not(unix))]
fn effective_uid() -> Result<u32, String> {
    Err(ERROR_OWNER.to_string())
}

fn validate_profile_location(
    profile_manifest: &Path,
    expected_uid: u32,
) -> Result<(PathBuf, PathBuf), String> {
    let canonical_temp = validate_profile_candidate_lexically(profile_manifest)?;
    let root = profile_manifest
        .parent()
        .ok_or_else(|| ERROR_ROOT.to_string())?;
    let root_metadata = fs::symlink_metadata(root).map_err(|_| ERROR_ROOT.to_string())?;
    if root_metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    if !root_metadata.is_dir() {
        return Err(ERROR_ROOT.to_string());
    }

    #[cfg(unix)]
    {
        if root_metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(ERROR_PERMISSIONS.to_string());
        }
        if root_metadata.uid() != expected_uid {
            return Err(ERROR_OWNER.to_string());
        }
    }

    let canonical_root = fs::canonicalize(root).map_err(|_| ERROR_ROOT.to_string())?;
    if canonical_root.parent() != Some(canonical_temp.as_path())
        || !canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(PROFILE_ROOT_PREFIX))
    {
        return Err(ERROR_ROOT.to_string());
    }

    let manifest_metadata =
        fs::symlink_metadata(profile_manifest).map_err(|_| ERROR_ROOT.to_string())?;
    if manifest_metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    if !manifest_metadata.is_file() {
        return Err(ERROR_ROOT.to_string());
    }
    #[cfg(unix)]
    if manifest_metadata.nlink() != 1 {
        return Err(ERROR_HARDLINK.to_string());
    }
    let canonical_manifest =
        fs::canonicalize(profile_manifest).map_err(|_| ERROR_ROOT.to_string())?;
    if profile_manifest.as_os_str() != canonical_manifest.as_os_str()
        || canonical_manifest.parent() != Some(canonical_root.as_path())
        || canonical_manifest
            .file_name()
            .and_then(|name| name.to_str())
            != Some(PROFILE_FILENAME)
    {
        return Err(ERROR_ROOT.to_string());
    }
    Ok((canonical_root, canonical_manifest))
}

pub(crate) fn validate_profile_candidate_lexically(
    profile_manifest: &Path,
) -> Result<PathBuf, String> {
    if !profile_manifest.is_absolute() {
        return Err(ERROR_ROOT.to_string());
    }

    #[cfg(unix)]
    {
        let mut components = profile_manifest
            .as_os_str()
            .as_bytes()
            .split(|byte| *byte == b'/');
        if components.next() != Some(&[]) {
            return Err(ERROR_ROOT.to_string());
        }
        if components
            .any(|component| component.is_empty() || component == b"." || component == b"..")
        {
            return Err(ERROR_ROOT.to_string());
        }
    }

    #[cfg(not(unix))]
    {
        let raw = profile_manifest.as_os_str().to_string_lossy();
        if raw
            .split(['/', '\\'])
            .any(|component| component.is_empty() || component == "." || component == "..")
        {
            return Err(ERROR_ROOT.to_string());
        }
    }

    if profile_manifest.file_name().and_then(|name| name.to_str()) != Some(PROFILE_FILENAME) {
        return Err(ERROR_ROOT.to_string());
    }
    let root = profile_manifest
        .parent()
        .ok_or_else(|| ERROR_ROOT.to_string())?;
    let canonical_temp =
        fs::canonicalize(std::env::temp_dir()).map_err(|_| ERROR_ROOT.to_string())?;
    if root.parent() != Some(canonical_temp.as_path())
        || !root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(PROFILE_ROOT_PREFIX))
    {
        return Err(ERROR_ROOT.to_string());
    }

    Ok(canonical_temp)
}

fn validate_root_layout(
    root: &Path,
    manifest: &ProfileManifest,
) -> Result<PreparedFixturePaths, String> {
    let entries = read_direct_entries(root)?;
    for path in entries.values() {
        reject_symlink(path)?;
    }
    let mut names = entries.keys().cloned().collect::<BTreeSet<_>>();
    // runtime-artifacts 为可选扩展项（db_primary 配置/DB 与验收门落点），存在时必须是目录
    if names.remove(RUNTIME_ARTIFACTS_DIR_NAME) {
        require_directory(entries.get(RUNTIME_ARTIFACTS_DIR_NAME))?;
    }
    let prepared = PREPARED_ROOT_ENTRY_NAMES
        .iter()
        .map(|entry| (*entry).to_string())
        .collect::<BTreeSet<_>>();
    if names != prepared {
        return Err(ERROR_REUSED.to_string());
    }

    require_directory(entries.get("fixture"))?;
    require_directory(entries.get("workflow-state"))?;
    require_empty_directory(entries.get("app-data"))?;
    require_empty_directory(entries.get("codex-db"))?;
    require_empty_directory(entries.get(LOGS_DIR_NAME))?;

    let (index_path, tasks_path, project_root) = validate_fixture_directory(
        root,
        entries.get("fixture").expect("checked fixture"),
        &manifest.run_id,
    )?;
    let workflow_state_path = validate_workflow_state_directory(
        entries
            .get("workflow-state")
            .expect("checked workflow state"),
    )?;
    Ok(PreparedFixturePaths {
        index_path,
        tasks_path,
        workflow_state_path,
        project_root,
    })
}

fn validate_fixture_directory(
    root: &Path,
    directory: &Path,
    run_id: &str,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let canonical_directory = fs::canonicalize(directory).map_err(|_| ERROR_FIXTURE.to_string())?;
    if canonical_directory.parent() != Some(root)
        || canonical_directory.as_os_str() != directory.as_os_str()
    {
        return Err(ERROR_FIXTURE.to_string());
    }
    let entries = read_direct_entries(directory)?;
    for path in entries.values() {
        reject_symlink(path)?;
    }
    let project_dir = format!("SYN R4 ISOLATED ACCEPTANCE {run_id}");
    let expected = BTreeSet::from([
        "codex-index.json".to_string(),
        "tasks.md".to_string(),
        project_dir.clone(),
    ]);
    if entries.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(ERROR_REUSED.to_string());
    }
    let index_path =
        require_single_link_regular_file(entries.get("codex-index.json"))?.to_path_buf();
    let tasks_path = require_single_link_regular_file(entries.get("tasks.md"))?.to_path_buf();
    let project_path = require_empty_directory(entries.get(&project_dir))?;
    let canonical_project_root =
        fs::canonicalize(project_path).map_err(|_| ERROR_FIXTURE.to_string())?;
    if canonical_project_root.parent() != Some(canonical_directory.as_path())
        || canonical_project_root.as_os_str() != project_path.as_os_str()
    {
        return Err(ERROR_FIXTURE.to_string());
    }
    Ok((index_path, tasks_path, canonical_project_root))
}

fn validate_workflow_state_directory(directory: &Path) -> Result<PathBuf, String> {
    let entries = read_direct_entries(directory)?;
    for path in entries.values() {
        reject_symlink(path)?;
    }
    if entries.keys().cloned().collect::<BTreeSet<_>>()
        != BTreeSet::from(["workflow-state.v0.json".to_string()])
    {
        return Err(ERROR_REUSED.to_string());
    }
    Ok(require_single_link_regular_file(entries.get("workflow-state.v0.json"))?.to_path_buf())
}

fn require_empty_directory(path: Option<&PathBuf>) -> Result<&Path, String> {
    let path = require_directory(path)?;
    let entries = read_direct_entries(path)?;
    for entry in entries.values() {
        reject_symlink(entry)?;
    }
    if entries.is_empty() {
        Ok(path)
    } else {
        Err(ERROR_REUSED.to_string())
    }
}

fn require_directory(path: Option<&PathBuf>) -> Result<&Path, String> {
    let path = path.ok_or_else(|| ERROR_REUSED.to_string())?;
    let metadata = fs::symlink_metadata(path).map_err(|_| ERROR_REUSED.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    if metadata.is_dir() {
        Ok(path)
    } else {
        Err(ERROR_REUSED.to_string())
    }
}

fn require_single_link_regular_file(path: Option<&PathBuf>) -> Result<&Path, String> {
    let path = path.ok_or_else(|| ERROR_REUSED.to_string())?;
    let metadata = fs::symlink_metadata(path).map_err(|_| ERROR_REUSED.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err(ERROR_SYMLINK.to_string());
    }
    if metadata.is_file() {
        #[cfg(unix)]
        if metadata.nlink() != 1 {
            return Err(ERROR_HARDLINK.to_string());
        }
        Ok(path)
    } else {
        Err(ERROR_REUSED.to_string())
    }
}

fn reject_symlink(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ERROR_REUSED.to_string())?;
    if metadata.file_type().is_symlink() {
        Err(ERROR_SYMLINK.to_string())
    } else {
        Ok(())
    }
}

fn read_direct_entries(root: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let mut entries = BTreeMap::new();
    for entry in fs::read_dir(root).map_err(|_| ERROR_REUSED.to_string())? {
        let entry = entry.map_err(|_| ERROR_REUSED.to_string())?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| ERROR_REUSED.to_string())?;
        if entries.insert(name, entry.path()).is_some() {
            return Err(ERROR_REUSED.to_string());
        }
    }
    Ok(entries)
}

fn read_and_validate_manifest(path: &Path, now_ms: i64) -> Result<ProfileManifest, String> {
    let bytes = fs::read(path).map_err(|_| ERROR_SCHEMA.to_string())?;
    let manifest =
        serde_json::from_slice::<ProfileManifest>(&bytes).map_err(|_| ERROR_SCHEMA.to_string())?;
    if manifest.schema_version != PROFILE_SCHEMA_VERSION || manifest.purpose != PROFILE_PURPOSE {
        return Err(ERROR_SCHEMA.to_string());
    }
    validate_manifest_identity(&manifest)?;
    if manifest.expires_at_ms <= now_ms {
        return Err(ERROR_EXPIRED.to_string());
    }
    if manifest.paths.index_relative_path != INDEX_RELATIVE_PATH
        || manifest.paths.tasks_relative_path != TASKS_RELATIVE_PATH
        || manifest.paths.workflow_state_relative_path != WORKFLOW_STATE_RELATIVE_PATH
        || manifest.paths.app_data_relative_path != APP_DATA_RELATIVE_PATH
        || manifest.paths.canvas_relative_path != CANVAS_RELATIVE_PATH
        || manifest.paths.codex_db_relative_path != CODEX_DB_RELATIVE_PATH
    {
        return Err(ERROR_SCHEMA.to_string());
    }
    Ok(manifest)
}

fn validate_manifest_identity(manifest: &ProfileManifest) -> Result<(), String> {
    if !valid_run_id(&manifest.run_id) {
        return Err(ERROR_SCHEMA.to_string());
    }
    let expected_project_path = format!("fixture/SYN R4 ISOLATED ACCEPTANCE {}", manifest.run_id);
    if manifest.project.relative_path != expected_project_path {
        return Err(ERROR_SCHEMA.to_string());
    }
    Ok(())
}

fn validate_manifest_runtime_identity(
    manifest: &ProfileManifest,
    canonical_project_root: &Path,
) -> Result<(), String> {
    let project_root = canonical_project_root
        .to_str()
        .ok_or_else(|| ERROR_SCHEMA.to_string())?;
    if manifest.project.id != crate::project_id(project_root)
        || manifest.workflow.id != crate::default_workflow_id(project_root)
    {
        return Err(ERROR_SCHEMA.to_string());
    }
    Ok(())
}

fn valid_run_id(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("syn-r4-") else {
        return false;
    };
    hex.len() == 16
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_fixture_contents(
    fixture: &PreparedFixturePaths,
    manifest: &ProfileManifest,
) -> Result<(), String> {
    let fixture_timestamp = validate_index_fixture(&fixture.index_path, &fixture.project_root)?;
    validate_tasks_fixture(&fixture.tasks_path)?;
    validate_workflow_state_fixture(
        &fixture.workflow_state_path,
        manifest,
        &fixture.project_root,
        &fixture_timestamp,
    )
}

fn validate_index_fixture(index_path: &Path, project_root: &Path) -> Result<String, String> {
    let index = read_fixture_json::<SyntheticIndexFixture>(index_path)?;
    if !is_fixture_timestamp(&index.generated_at)
        || !index.threads.is_empty()
        || !index.skills.is_empty()
        || !index.plugins.is_empty()
        || !index.warnings.is_empty()
    {
        return Err(ERROR_FIXTURE.to_string());
    }
    let [project] = index.projects.as_slice() else {
        return Err(ERROR_FIXTURE.to_string());
    };
    let expected_root = path_as_fixture_string(project_root)?;
    if project.project_root != expected_root
        || !project.active_hint
        || project.thread_count != 0
        || project.active_thread_count != 0
        || project.archived_thread_count != 0
        || !project.authority_files.is_empty()
        || !project.handoff_files.is_empty()
        || !project.evidence_files.is_empty()
        || !project.harness_candidates.is_empty()
        || !project.harness_resources.is_empty()
        || !project.context_warnings.is_empty()
        || !project.warnings.is_empty()
    {
        return Err(ERROR_FIXTURE.to_string());
    }
    Ok(index.generated_at)
}

fn validate_tasks_fixture(tasks_path: &Path) -> Result<(), String> {
    let tasks = fs::read(tasks_path).map_err(|_| ERROR_FIXTURE.to_string())?;
    if tasks.is_empty() {
        Ok(())
    } else {
        Err(ERROR_FIXTURE.to_string())
    }
}

fn validate_workflow_state_fixture(
    workflow_state_path: &Path,
    manifest: &ProfileManifest,
    project_root: &Path,
    fixture_timestamp: &str,
) -> Result<(), String> {
    let workflow_state = read_fixture_json::<SyntheticWorkflowStateFixture>(workflow_state_path)?;
    if workflow_state.schema_version != "workflow_state_v0"
        || workflow_state.workflow_version != 1
        || workflow_state.revision != 0
        || workflow_state.workspace_id != format!("workspace:{}", manifest.run_id)
        || workflow_state.created_at != fixture_timestamp
        || workflow_state.updated_at != fixture_timestamp
        || workflow_state.source_kind != "isolated_acceptance_fixture"
        || workflow_state.permission_level != "user_confirmed_write"
        || !workflow_state.agent_adapters.is_empty()
        || !workflow_state.nodes.is_empty()
        || !workflow_state.edges.is_empty()
        || !workflow_state.work_items.is_empty()
        || !workflow_state.artifacts.is_empty()
        || !workflow_state.reviews.is_empty()
        || !workflow_state.workflow_node_session_bindings.is_empty()
        || !workflow_state.workflow_node_dispatches.is_empty()
        || !workflow_state.audit_events.is_empty()
        || !workflow_state.capabilities.is_empty()
        || !workflow_state.harness_resources.is_empty()
    {
        return Err(ERROR_FIXTURE.to_string());
    }

    let [project] = workflow_state.projects.as_slice() else {
        return Err(ERROR_FIXTURE.to_string());
    };
    let [workflow] = workflow_state.workflows.as_slice() else {
        return Err(ERROR_FIXTURE.to_string());
    };
    let project_root = path_as_fixture_string(project_root)?;
    let display_name = format!("SYN R4 ISOLATED ACCEPTANCE {}", manifest.run_id);
    if project.project_id != manifest.project.id
        || project.display_name != display_name
        || project.root_path != project_root
        || project.source_kind != "codex_index"
        || project.permission_level != "read_only"
        || project.created_at != fixture_timestamp
        || project.updated_at != fixture_timestamp
        || !project.warnings.is_empty()
        || workflow.workflow_id != manifest.workflow.id
        || workflow.workflow_version != 1
        || workflow.project_id != manifest.project.id
        || workflow.title != format!("{display_name} workflow")
        || workflow.state != "draft"
        || workflow.source_kind != "isolated_acceptance_fixture"
        || workflow.permission_level != "user_confirmed_write"
        || workflow.model_policy != "none"
        || workflow.created_at != fixture_timestamp
        || workflow.updated_at != fixture_timestamp
    {
        return Err(ERROR_FIXTURE.to_string());
    }
    Ok(())
}

fn read_fixture_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|_| ERROR_FIXTURE.to_string())?;
    serde_json::from_slice(&bytes).map_err(|_| ERROR_FIXTURE.to_string())
}

fn path_as_fixture_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| ERROR_FIXTURE.to_string())
}

fn is_fixture_timestamp(value: &str) -> bool {
    value.len() >= 20
        && value.len() <= 40
        && value.ends_with('Z')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'-' | b':' | b'.' | b'T' | b'Z'))
}

fn runtime_paths_from_manifest(root: PathBuf, project_root: PathBuf) -> RuntimePaths {
    let app_data_root = root.join(APP_DATA_RELATIVE_PATH);
    RuntimePaths {
        index_path: root.join(INDEX_RELATIVE_PATH),
        tasks_path: root.join(TASKS_RELATIVE_PATH),
        project_root,
        workflow_state_path: root.join(WORKFLOW_STATE_RELATIVE_PATH),
        vault_root: app_data_root.join(VAULT_DIR_NAME),
        recovery_backups_root: app_data_root.join(RECOVERY_DIR_NAME),
        canvas_root: root.join(CANVAS_RELATIVE_PATH),
        codex_db_path: root.join(CODEX_DB_RELATIVE_PATH),
        app_log_dir: root.join(LOGS_DIR_NAME),
        app_data_root,
        root,
    }
}

// ---- T2 崩溃恢复：debug-only 验收门 ----
// 门只在 SYN_R4_ACCEPTANCE_PROFILE 有效（debug build + 受控 /private/tmp root 强校验）
// 且操作者在 <root>/runtime-artifacts/acceptance-gates/<gate>.pause 落盘时武装。
// 普通 App 路径（无 profile）恒惰性；release 构建不含调用点（调用点全部 #[cfg(debug_assertions)]）。

const ACCEPTANCE_GATE_WAIT_INTERVAL: Duration = Duration::from_millis(50);
const ACCEPTANCE_GATE_WAIT_BUDGET: Duration = Duration::from_secs(120);
const M2_REFERENCE_GATE_MAX_BYTES: u64 = 2048;

#[derive(Deserialize)]
struct M2ReferenceCommandGateBinding {
    operation: String,
    attempt: String,
    command_id: String,
    nonce: String,
}

fn acceptance_gate_file(gate: &str) -> Option<PathBuf> {
    let paths = active_paths().ok().flatten()?;
    Some(
        paths
            .root
            .join(RUNTIME_ARTIFACTS_DIR_NAME)
            .join("acceptance-gates")
            .join(format!("{gate}.pause")),
    )
}

/// 门是否武装：profile 有效且门文件存在。无 profile / 无门文件恒 false。
pub(crate) fn acceptance_gate_armed(gate: &str) -> bool {
    acceptance_gate_file(gate).is_some_and(|path| path.exists())
}

/// 在确定性窗口阻塞，直到操作者移除门文件；超时 fail-closed 报错（不继续后续提交动作）。
/// 阻塞期间进程可被 SIGKILL，窗口边界 = 门文件存在期间，可观察、可重复。
pub(crate) fn acceptance_wait_for_gate_release(gate: &str) -> Result<(), String> {
    if !acceptance_gate_armed(gate) {
        return Ok(());
    }
    eprintln!("acceptance_gate_armed:{gate}: waiting for operator release");
    let deadline = std::time::Instant::now() + ACCEPTANCE_GATE_WAIT_BUDGET;
    loop {
        if !acceptance_gate_armed(gate) {
            eprintln!("acceptance_gate_released:{gate}");
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "acceptance_gate_release_timeout:{gate}: operator did not release within {:?}",
                ACCEPTANCE_GATE_WAIT_BUDGET
            ));
        }
        std::thread::sleep(ACCEPTANCE_GATE_WAIT_INTERVAL);
    }
}

/// 失败注入门：武装时返回注入错误文案，否则 None（无行为变化）。
pub(crate) fn acceptance_injected_failure(gate: &str) -> Option<String> {
    if acceptance_gate_armed(gate) {
        Some(format!("acceptance_injected_failure:{gate}"))
    } else {
        None
    }
}

/// M2's R4 crash gates are deliberately narrower than the pre-existing
/// filename-only gates.  A pause file can affect the reference command only
/// when it names the exact operation, logical command identity, fixture
/// attempt, and one-time nonce.  Startup/reconcile/outbox transactions have
/// no matching binding and therefore cannot accidentally become an S2/S3/S4
/// crash window.
fn m2_reference_gate_is_armed_for(
    gate: &str,
    operation: &str,
    attempt: &str,
    command_id: &str,
    nonce: &str,
) -> Result<bool, String> {
    let Some(path) = acceptance_gate_file(gate) else {
        return Ok(false);
    };
    if !path.exists() {
        return Ok(false);
    }
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("m2_reference_gate_metadata_failed:{gate}:{error}"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(format!("m2_reference_gate_file_invalid:{gate}"));
    }
    if metadata.len() == 0 || metadata.len() > M2_REFERENCE_GATE_MAX_BYTES {
        return Err(format!("m2_reference_gate_size_invalid:{gate}"));
    }
    let binding: M2ReferenceCommandGateBinding = serde_json::from_slice(
        &fs::read(&path).map_err(|error| format!("m2_reference_gate_read_failed:{gate}:{error}"))?,
    )
    .map_err(|_| format!("m2_reference_gate_binding_invalid:{gate}"))?;
    if binding.operation != operation
        || binding.attempt != attempt
        || binding.command_id != command_id
        || binding.nonce != nonce
    {
        return Err(format!("m2_reference_gate_binding_mismatch:{gate}"));
    }
    Ok(true)
}

pub(crate) fn acceptance_wait_for_m2_reference_gate_release(
    gate: &str,
    operation: &str,
    attempt: &str,
    command_id: &str,
    nonce: &str,
) -> Result<(), String> {
    if !m2_reference_gate_is_armed_for(gate, operation, attempt, command_id, nonce)? {
        return Ok(());
    }
    eprintln!("acceptance_m2_reference_gate_armed:{gate}:{operation}:{attempt}");
    let deadline = std::time::Instant::now() + ACCEPTANCE_GATE_WAIT_BUDGET;
    loop {
        if !m2_reference_gate_is_armed_for(gate, operation, attempt, command_id, nonce)? {
            eprintln!("acceptance_m2_reference_gate_released:{gate}:{operation}:{attempt}");
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "acceptance_m2_reference_gate_release_timeout:{gate}:{operation}:{attempt}"
            ));
        }
        std::thread::sleep(ACCEPTANCE_GATE_WAIT_INTERVAL);
    }
}

pub(crate) fn acceptance_injected_m2_reference_failure(
    gate: &str,
    operation: &str,
    attempt: &str,
    command_id: &str,
    nonce: &str,
) -> Result<Option<String>, String> {
    if m2_reference_gate_is_armed_for(gate, operation, attempt, command_id, nonce)? {
        return Ok(Some(format!("acceptance_injected_failure:{gate}")));
    }
    Ok(None)
}
