use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
#[cfg(unix)]
use std::os::unix::{
    ffi::OsStrExt,
    fs::{MetadataExt, PermissionsExt},
};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const PROFILE_ENV: &str = "SYN_R4_ACCEPTANCE_PROFILE";
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
}

impl ProfileProcessState {
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
    let fixture = validate_root_layout(&root, &manifest)?;
    validate_manifest_runtime_identity(&manifest, &fixture.project_root)?;
    validate_fixture_contents(&fixture, &manifest)?;
    Ok(Some(runtime_paths_from_manifest(
        root,
        fixture.project_root,
    )))
}

pub(crate) fn initialize_from_env() -> Result<(), String> {
    let Some(profile_path) = std::env::var_os(PROFILE_ENV).map(PathBuf::from) else {
        return Ok(());
    };
    let context = production_context()?;
    let mut state = process_state()
        .lock()
        .map_err(|_| ERROR_UNINITIALIZED.to_string())?;
    state.initialize_from_manifest(Some(&profile_path), context)
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
    let names = entries.keys().cloned().collect::<BTreeSet<_>>();
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
