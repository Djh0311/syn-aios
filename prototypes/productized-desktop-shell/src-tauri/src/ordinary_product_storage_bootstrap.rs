use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const ORDINARY_TAURI_APP_DATA_DIR_NAME: &str = "local.codex.governance.workbench";
const ORDINARY_PRODUCT_DATA_DIR_NAME: &str = "CodexGovernanceWorkbench";
const WORKFLOW_STATE_RELATIVE_PATH: &str = "workflow-state/workflow-state.v0.json";
const INDEX_RELATIVE_PATH: &str = "index-kernel/codex-index.json";
const TASKS_RELATIVE_PATH: &str = "tasks/README.md";
const WORKBENCH_DB_FILE_NAME: &str = "workbench.sqlite";

pub(crate) const ORDINARY_PRODUCT_STORAGE_RESTART_REQUIRED_MARKER: &str =
    "ordinary_product_storage_restart_required";

/// Identify only the workflow-state owner path produced by the ordinary
/// product data-root resolver.  Legacy/temp JSON stores deliberately remain
/// outside this gate so their compatibility behavior is not reclassified as
/// a registered M4 source owner.
pub(crate) fn is_ordinary_product_workflow_state_path(path: &Path) -> bool {
    if !has_ordinary_product_workflow_state_path_identity(path) {
        return false;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    !metadata.file_type().is_symlink()
        && metadata.is_file()
        && fs::canonicalize(path).is_ok_and(|canonical| canonical == path)
}

/// Path identity used by the writer-side cutover fence. Unlike the owner
/// admission predicate above, this remains true if the JSON file was removed:
/// an old JsonOnly process must not recreate it after another process has
/// published DB-primary authority.
pub(crate) fn has_ordinary_product_workflow_state_path_identity(path: &Path) -> bool {
    if require_clean_absolute_path("workflow_state_path", path).is_err()
        || path.file_name().and_then(|name| name.to_str()) != Some("workflow-state.v0.json")
    {
        return false;
    }
    let Some(workflow_root) = path.parent() else {
        return false;
    };
    let Some(product_root) = workflow_root.parent() else {
        return false;
    };
    workflow_root.file_name().and_then(|name| name.to_str()) == Some("workflow-state")
        && product_root.file_name().and_then(|name| name.to_str())
            == Some(ORDINARY_PRODUCT_DATA_DIR_NAME)
        && fs::canonicalize(workflow_root).is_ok_and(|canonical| canonical == workflow_root)
        && fs::canonicalize(product_root).is_ok_and(|canonical| canonical == product_root)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductDataPaths {
    pub(crate) root: PathBuf,
    pub(crate) index_path: PathBuf,
    pub(crate) tasks_path: PathBuf,
    pub(crate) workflow_state_path: PathBuf,
}

impl ProductDataPaths {
    /// Resolve every ordinary workflow owner path from the one Tauri-owned
    /// app-data port.  The sibling keeps the existing macOS product location
    /// stable while an isolated app-data root naturally receives an isolated
    /// workflow/index/tasks root as well.
    pub(crate) fn resolve_and_materialize(
        app_data_root: &Path,
        bundled_index_path: &Path,
        bundled_tasks_path: &Path,
    ) -> Result<Self, String> {
        require_clean_absolute_path("app_data_root", app_data_root)?;
        if app_data_root.file_name().and_then(|name| name.to_str())
            != Some(ORDINARY_TAURI_APP_DATA_DIR_NAME)
        {
            return Err("ordinary_product_app_data_root_identity_mismatch".to_string());
        }
        require_existing_canonical_directory("app_data_root", app_data_root)?;
        let parent = app_data_root
            .parent()
            .ok_or_else(|| "ordinary_product_app_data_root_parent_required".to_string())?;
        require_existing_canonical_directory("app_data_parent", parent)?;

        let root = parent.join(ORDINARY_PRODUCT_DATA_DIR_NAME);
        fs::create_dir_all(&root).map_err(|error| {
            format!(
                "ordinary_product_data_root_create_failed:{}:{error}",
                root.display()
            )
        })?;
        require_existing_canonical_directory("product_data_root", &root)?;

        let index_path = root.join(INDEX_RELATIVE_PATH);
        let tasks_path = root.join(TASKS_RELATIVE_PATH);
        materialize_product_file(&root, bundled_index_path, &index_path, "index")?;
        materialize_product_file(&root, bundled_tasks_path, &tasks_path, "tasks")?;

        Ok(Self {
            workflow_state_path: root.join(WORKFLOW_STATE_RELATIVE_PATH),
            root,
            index_path,
            tasks_path,
        })
    }
}

/// Before any ordinary workflow writer runs, create the confirmed SQLite
/// primary from the complete latest JSON/sidecar snapshot.  A fresh product
/// with no workflow JSON remains JSON-only until the user invokes the ordinary
/// initialize command and restarts; that second process enters here.
pub(crate) fn cold_bootstrap_before_startup(workflow_state_path: &Path) -> Result<bool, String> {
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(workflow_state_path)?;
    match fs::symlink_metadata(&config_path) {
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "ordinary_product_storage_config_inspect_failed:{}:{error}",
                config_path.display()
            ))
        }
    }
    match fs::symlink_metadata(workflow_state_path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "ordinary_product_workflow_state_inspect_failed:{}:{error}",
                workflow_state_path.display()
            ))
        }
        Ok(_) => {}
    }
    crate::workflow_state_store::with_exclusive_workflow_state_lock(
        workflow_state_path,
        "ordinary-product-storage-cutover",
        || cold_bootstrap_before_startup_locked(workflow_state_path),
    )
}

fn cold_bootstrap_before_startup_locked(workflow_state_path: &Path) -> Result<bool, String> {
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(workflow_state_path)?;
    match fs::symlink_metadata(&config_path) {
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "ordinary_product_storage_config_inspect_failed:{}:{error}",
                config_path.display()
            ))
        }
    }

    match fs::symlink_metadata(workflow_state_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err("ordinary_product_workflow_state_regular_file_required".to_string())
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "ordinary_product_workflow_state_inspect_failed:{}:{error}",
                workflow_state_path.display()
            ))
        }
    }
    require_clean_absolute_path("workflow_state_path", workflow_state_path)?;
    let canonical_state = fs::canonicalize(workflow_state_path).map_err(|error| {
        format!(
            "ordinary_product_workflow_state_canonicalize_failed:{}:{error}",
            workflow_state_path.display()
        )
    })?;
    if canonical_state != workflow_state_path {
        return Err("ordinary_product_workflow_state_identity_changed".to_string());
    }
    let source_root = workflow_state_path
        .parent()
        .ok_or_else(|| "ordinary_product_workflow_state_parent_required".to_string())?;
    if source_root.file_name().and_then(|name| name.to_str()) != Some("workflow-state")
        || source_root
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            != Some(ORDINARY_PRODUCT_DATA_DIR_NAME)
    {
        return Err("ordinary_product_workflow_state_root_identity_mismatch".to_string());
    }
    require_existing_canonical_directory("workflow_state_root", source_root)?;

    let runtime_dir = config_path
        .parent()
        .ok_or_else(|| "ordinary_product_storage_runtime_parent_required".to_string())?;
    fs::create_dir_all(runtime_dir).map_err(|error| {
        format!(
            "ordinary_product_storage_runtime_create_failed:{}:{error}",
            runtime_dir.display()
        )
    })?;
    require_existing_canonical_directory("storage_runtime_root", runtime_dir)?;

    let final_db_path = runtime_dir.join(WORKBENCH_DB_FILE_NAME);
    let final_config = db_primary_config(workflow_state_path, &final_db_path);
    if final_db_path.exists() {
        require_existing_canonical_file("workbench_db", &final_db_path)?;
        require_green_reconciliation(&final_config, "existing_unpublished_db")?;
    } else {
        let staging_db_path = runtime_dir.join(format!(
            ".workbench.sqlite.bootstrap.{}.{}.tmp",
            std::process::id(),
            crate::unix_timestamp_nanos()
        ));
        let apply_result =
            crate::workbench_sqlite_apply::apply_confirmed_workbench_state_root_to_confirmed_db(
                source_root,
                source_root,
                &staging_db_path,
                &staging_db_path,
                None,
            );
        if let Err(error) = apply_result {
            let _ = fs::remove_file(&staging_db_path);
            return Err(format!("ordinary_product_storage_import_failed:{error}"));
        }
        let staging_config = db_primary_config(workflow_state_path, &staging_db_path);
        if let Err(error) = require_green_reconciliation(&staging_config, "staging_db") {
            let _ = fs::remove_file(&staging_db_path);
            return Err(error);
        }
        sync_file(&staging_db_path, "staging_db")?;

        match fs::hard_link(&staging_db_path, &final_db_path) {
            Ok(()) => {
                fs::remove_file(&staging_db_path).map_err(|error| {
                    format!(
                        "ordinary_product_storage_staging_unlink_failed:{}:{error}",
                        staging_db_path.display()
                    )
                })?;
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&staging_db_path);
                require_existing_canonical_file("workbench_db", &final_db_path)?;
            }
            Err(error) => {
                let _ = fs::remove_file(&staging_db_path);
                return Err(format!(
                    "ordinary_product_storage_db_publish_failed:{}:{error}",
                    final_db_path.display()
                ));
            }
        }
        sync_directory(runtime_dir, "storage_runtime_root")?;
        require_green_reconciliation(&final_config, "published_db")?;
    }

    // The mode declaration is deliberately the final durable write.  Until
    // this succeeds, a restart has no authority to treat SQLite as primary.
    crate::workbench_sqlite_storage_mode::install_db_primary_config_create_new(&final_config)?;
    Ok(true)
}

/// Explicit config is an operator statement, so malformed/tampered config or
/// a failed reconciliation must stop ordinary startup rather than silently
/// continuing with a second JSON writer.
pub(crate) fn initialize_for_ordinary_startup(workflow_state_path: &Path) -> Result<(), String> {
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(workflow_state_path)?;
    let explicit_config = match fs::symlink_metadata(&config_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err("ordinary_product_storage_config_regular_file_required".to_string())
        }
        Ok(_) => true,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => {
            return Err(format!(
                "ordinary_product_storage_config_inspect_failed:{}:{error}",
                config_path.display()
            ))
        }
    };

    let resolved = crate::workbench_sqlite_storage_mode::storage_mode_for(workflow_state_path);
    match (explicit_config, &resolved) {
        (
            true,
            crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(config),
        ) => {
            let product_root = workflow_state_path
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| {
                    "ordinary_product_workflow_state_product_root_required".to_string()
                })?;
            let expected = db_primary_config(
                workflow_state_path,
                &product_root
                    .join("runtime-artifacts")
                    .join(WORKBENCH_DB_FILE_NAME),
            );
            if config != &expected {
                return Err("ordinary_product_storage_config_identity_mismatch".to_string());
            }
        }
        (true, crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { reason }) => {
            return Err(format!(
                "ordinary_product_explicit_storage_config_rejected:{reason}"
            ));
        }
        (false, crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. }) => {}
        (false, crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(_)) => {
            return Err("ordinary_product_storage_mode_cache_identity_invalid".to_string());
        }
    }
    crate::workbench_sqlite_storage_mode::initialize_for_startup(workflow_state_path)
}

pub(crate) fn restart_required_after_initialization(
    workflow_state_path: &Path,
) -> Result<bool, String> {
    let config_path = crate::workbench_sqlite_storage_mode::storage_mode_path(workflow_state_path)?;
    match fs::symlink_metadata(&config_path) {
        Ok(_) => Ok(false),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!(
            "ordinary_product_storage_config_inspect_failed:{}:{error}",
            config_path.display()
        )),
    }
}

fn db_primary_config(
    workflow_state_path: &Path,
    db_path: &Path,
) -> crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
    crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig {
        workflow_state_path: workflow_state_path.to_path_buf(),
        confirmed_workflow_state_path: workflow_state_path.to_path_buf(),
        db_path: db_path.to_path_buf(),
        confirmed_db_path: db_path.to_path_buf(),
        denied_path_markers: Vec::new(),
    }
}

fn require_green_reconciliation(
    config: &crate::workbench_sqlite_storage_mode::DbPrimaryJsonProjectionConfig,
    phase: &str,
) -> Result<(), String> {
    let report = crate::workbench_sqlite_storage_mode::reconcile_db_vs_json(config)?;
    if !report.is_green() {
        return Err(format!(
            "ordinary_product_storage_reconciliation_not_green:{phase}"
        ));
    }
    Ok(())
}

fn materialize_product_file(
    product_root: &Path,
    source: &Path,
    target: &Path,
    label: &str,
) -> Result<(), String> {
    match fs::symlink_metadata(target) {
        Ok(_) => return require_existing_product_file(product_root, target, label),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "ordinary_product_{label}_inspect_failed:{}:{error}",
                target.display()
            ))
        }
    }

    let bytes = fs::read(source).map_err(|error| {
        format!(
            "ordinary_product_bundled_{label}_read_failed:{}:{error}",
            source.display()
        )
    })?;
    let parent = target
        .parent()
        .ok_or_else(|| format!("ordinary_product_{label}_parent_required"))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "ordinary_product_{label}_parent_create_failed:{}:{error}",
            parent.display()
        )
    })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "ordinary_product_{label}_parent_canonicalize_failed:{}:{error}",
            parent.display()
        )
    })?;
    if !canonical_parent.starts_with(product_root) {
        return Err(format!("ordinary_product_{label}_parent_escape"));
    }

    let temporary = parent.join(format!(
        ".{}.{}.{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("product-file"),
        std::process::id(),
        crate::unix_timestamp_nanos()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary).map_err(|error| {
        format!(
            "ordinary_product_{label}_temporary_create_failed:{}:{error}",
            temporary.display()
        )
    })?;
    if let Err(error) = file.write_all(&bytes).and_then(|()| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "ordinary_product_{label}_temporary_sync_failed:{}:{error}",
            temporary.display()
        ));
    }
    drop(file);
    match fs::hard_link(&temporary, target) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            return Err(format!(
                "ordinary_product_{label}_publish_failed:{}:{error}",
                target.display()
            ));
        }
    }
    fs::remove_file(&temporary).map_err(|error| {
        format!(
            "ordinary_product_{label}_temporary_unlink_failed:{}:{error}",
            temporary.display()
        )
    })?;
    sync_directory(parent, label)?;
    require_existing_product_file(product_root, target, label)
}

fn require_existing_product_file(
    product_root: &Path,
    path: &Path,
    label: &str,
) -> Result<(), String> {
    require_existing_canonical_file(label, path)?;
    if !path.starts_with(product_root) {
        return Err(format!("ordinary_product_{label}_path_escape"));
    }
    Ok(())
}

fn require_clean_absolute_path(label: &str, path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(format!("ordinary_product_{label}_clean_absolute_required"));
    }
    Ok(())
}

fn require_existing_canonical_directory(label: &str, path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "ordinary_product_{label}_unavailable:{}:{error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("ordinary_product_{label}_directory_required"));
    }
    require_canonical_identity(label, path)
}

fn require_existing_canonical_file(label: &str, path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "ordinary_product_{label}_unavailable:{}:{error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("ordinary_product_{label}_regular_file_required"));
    }
    require_canonical_identity(label, path)
}

fn require_canonical_identity(label: &str, path: &Path) -> Result<(), String> {
    require_clean_absolute_path(label, path)?;
    let canonical = fs::canonicalize(path).map_err(|error| {
        format!(
            "ordinary_product_{label}_canonicalize_failed:{}:{error}",
            path.display()
        )
    })?;
    if canonical != path {
        return Err(format!("ordinary_product_{label}_identity_changed"));
    }
    Ok(())
}

fn sync_file(path: &Path, label: &str) -> Result<(), String> {
    OpenOptions::new()
        .read(true)
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| {
            format!(
                "ordinary_product_{label}_sync_failed:{}:{error}",
                path.display()
            )
        })
}

fn sync_directory(path: &Path, label: &str) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "ordinary_product_{label}_directory_sync_failed:{}:{error}",
                path.display()
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture {
        root: PathBuf,
        app_data_root: PathBuf,
        bundled_index: PathBuf,
        bundled_tasks: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "product-line-syn-m4r-product-data-{label}-{}-{nonce}",
                std::process::id()
            ));
            let app_data_root = root.join(ORDINARY_TAURI_APP_DATA_DIR_NAME);
            let bundle = root.join("bundle");
            fs::create_dir_all(&app_data_root).expect("app data root");
            fs::create_dir_all(&bundle).expect("bundle root");
            let root = fs::canonicalize(&root).expect("canonical fixture root");
            let app_data_root = root.join(ORDINARY_TAURI_APP_DATA_DIR_NAME);
            let bundled_index = bundle.join("codex-index.json");
            let bundled_tasks = bundle.join("README.md");
            fs::write(&bundled_index, br#"{"projects":[],"sessions":[]}"#).expect("bundled index");
            fs::write(&bundled_tasks, b"# Tasks\n").expect("bundled tasks");
            Self {
                root,
                app_data_root,
                bundled_index,
                bundled_tasks,
            }
        }

        fn paths(&self) -> ProductDataPaths {
            ProductDataPaths::resolve_and_materialize(
                &self.app_data_root,
                &self.bundled_index,
                &self.bundled_tasks,
            )
            .expect("product paths")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn product_data_paths_are_sibling_rooted_and_preserve_existing_isolated_catalog() {
        let fixture = Fixture::new("paths");
        let paths = fixture.paths();
        assert_eq!(
            paths.root,
            fixture.root.join(ORDINARY_PRODUCT_DATA_DIR_NAME)
        );
        assert_eq!(
            paths.workflow_state_path,
            paths.root.join(WORKFLOW_STATE_RELATIVE_PATH)
        );
        assert!(!is_ordinary_product_workflow_state_path(
            &paths.workflow_state_path
        ));
        assert!(!has_ordinary_product_workflow_state_path_identity(
            &paths.workflow_state_path
        ));
        fs::create_dir_all(paths.workflow_state_path.parent().expect("workflow parent"))
            .expect("workflow root");
        assert!(has_ordinary_product_workflow_state_path_identity(
            &paths.workflow_state_path
        ));
        assert!(paths.index_path.starts_with(&paths.root));
        assert!(paths.tasks_path.starts_with(&paths.root));

        fs::write(&paths.index_path, br#"{"projects":[{"fixture":true}]}"#)
            .expect("isolated index replacement");
        let resolved = fixture.paths();
        assert_eq!(
            fs::read_to_string(resolved.index_path).expect("read isolated index"),
            r#"{"projects":[{"fixture":true}]}"#
        );
    }

    #[test]
    fn cold_bootstrap_publishes_config_last_with_canonical_workflow_source_binding() {
        let fixture = Fixture::new("cold-bootstrap");
        let paths = fixture.paths();
        fs::create_dir_all(paths.workflow_state_path.parent().expect("workflow parent"))
            .expect("workflow root");
        let state = crate::initial_workflow_state_json(
            "2026-08-11T00:00:00Z",
            "audit:ordinary-product-bootstrap",
            false,
            &paths.workflow_state_path,
        );
        fs::write(
            &paths.workflow_state_path,
            serde_json::to_vec_pretty(&state).expect("state json"),
        )
        .expect("workflow state");
        assert!(is_ordinary_product_workflow_state_path(
            &paths.workflow_state_path
        ));
        assert!(!is_ordinary_product_workflow_state_path(
            &fixture.root.join("legacy/workflow-state.v0.json")
        ));

        assert!(
            restart_required_after_initialization(&paths.workflow_state_path)
                .expect("restart marker")
        );
        assert!(cold_bootstrap_before_startup(&paths.workflow_state_path).expect("cold bootstrap"));
        let config_path =
            crate::workbench_sqlite_storage_mode::storage_mode_path(&paths.workflow_state_path)
                .expect("config path");
        let db_path = paths.root.join("runtime-artifacts/workbench.sqlite");
        assert!(config_path.is_file());
        assert!(db_path.is_file());
        assert!(
            !restart_required_after_initialization(&paths.workflow_state_path)
                .expect("restart marker after publish")
        );

        let connection = Connection::open(&db_path).expect("open confirmed db");
        let source_id: String = connection
            .query_row(
                "SELECT source_id FROM workflow_state_meta LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("workflow meta source id");
        assert_eq!(
            source_id,
            crate::workbench_sqlite_repository::WORKFLOW_STATE_SIDECAR_REPOSITORY_SOURCE_ID
        );
        drop(connection);

        initialize_for_ordinary_startup(&paths.workflow_state_path)
            .expect("DB-primary startup reconciliation");
        assert!(matches!(
            crate::workbench_sqlite_storage_mode::storage_mode_for(&paths.workflow_state_path),
            crate::workbench_sqlite_storage_mode::StorageMode::DbPrimaryJsonProjection(_)
        ));
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &paths.workflow_state_path,
        );
    }

    #[test]
    fn cached_json_only_writer_is_fenced_after_another_process_publishes_db_primary() {
        let fixture = Fixture::new("cached-json-only-cutover");
        let paths = fixture.paths();
        fs::create_dir_all(paths.workflow_state_path.parent().expect("workflow parent"))
            .expect("workflow root");
        let state = crate::initial_workflow_state_json(
            "2026-08-11T00:00:00Z",
            "audit:cached-json-only-cutover",
            false,
            &paths.workflow_state_path,
        );
        fs::write(
            &paths.workflow_state_path,
            serde_json::to_vec_pretty(&state).expect("state json"),
        )
        .expect("workflow state");

        assert!(matches!(
            crate::workbench_sqlite_storage_mode::storage_mode_for(&paths.workflow_state_path),
            crate::workbench_sqlite_storage_mode::StorageMode::JsonOnly { .. }
        ));
        assert!(cold_bootstrap_before_startup(&paths.workflow_state_path)
            .expect("second process publishes DB primary"));
        let before = fs::read(&paths.workflow_state_path).expect("state before stale writer");
        let mut stale_process_candidate =
            crate::read_workflow_state_value(&paths.workflow_state_path).expect("state candidate");
        stale_process_candidate["updated_at"] =
            serde_json::Value::String("stale-json-only-writer".to_string());

        let error = crate::write_validated_workflow_state(
            &paths.workflow_state_path,
            &stale_process_candidate,
        )
        .expect_err("cached JSON-only writer must stop after config publication");
        assert_eq!(error, ORDINARY_PRODUCT_STORAGE_RESTART_REQUIRED_MARKER);
        assert_eq!(
            fs::read(&paths.workflow_state_path).expect("state after stale writer"),
            before
        );
        assert!(paths
            .root
            .join("runtime-artifacts/workbench.sqlite")
            .is_file());
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &paths.workflow_state_path,
        );
    }

    #[test]
    fn busy_or_stale_workflow_lock_stops_cutover_before_db_or_config_publication() {
        let fixture = Fixture::new("cutover-lock-fence");
        let paths = fixture.paths();
        fs::create_dir_all(paths.workflow_state_path.parent().expect("workflow parent"))
            .expect("workflow root");
        let state = crate::initial_workflow_state_json(
            "2026-08-11T00:00:00Z",
            "audit:cutover-lock-fence",
            false,
            &paths.workflow_state_path,
        );
        fs::write(
            &paths.workflow_state_path,
            serde_json::to_vec_pretty(&state).expect("state json"),
        )
        .expect("workflow state");
        let before = fs::read(&paths.workflow_state_path).expect("state before lock");
        let config_path =
            crate::workbench_sqlite_storage_mode::storage_mode_path(&paths.workflow_state_path)
                .expect("config path");
        let db_path = paths.root.join("runtime-artifacts/workbench.sqlite");

        crate::workflow_state_store::with_exclusive_workflow_state_lock(
            &paths.workflow_state_path,
            "writer-held-by-test",
            || {
                let error = cold_bootstrap_before_startup(&paths.workflow_state_path)
                    .expect_err("cutover must not pass an active writer lock");
                assert!(error.contains("workflow_state_store_locked"));
                assert!(!config_path.exists());
                assert!(!db_path.exists());
                Ok(())
            },
        )
        .expect("test writer lock");
        assert_eq!(
            fs::read(&paths.workflow_state_path).expect("state after active lock"),
            before
        );

        crate::workflow_state_store::with_exclusive_workflow_state_lock(
            &paths.workflow_state_path,
            "cutover-held-by-test",
            || {
                let mut candidate = crate::read_workflow_state_value(&paths.workflow_state_path)
                    .expect("writer candidate");
                candidate["updated_at"] =
                    serde_json::Value::String("writer-during-cutover".to_string());
                let error =
                    crate::write_validated_workflow_state(&paths.workflow_state_path, &candidate)
                        .expect_err("writer must not pass an active cutover lock");
                assert!(error.contains("workflow_state_store_locked"));
                Ok(())
            },
        )
        .expect("test cutover lock");
        assert_eq!(
            fs::read(&paths.workflow_state_path).expect("state after cutover lock"),
            before
        );

        let stale_lock = paths
            .workflow_state_path
            .parent()
            .expect("workflow parent")
            .join(".workflow-state.v0.lock");
        fs::write(&stale_lock, b"crashed-owner").expect("stale lock marker");
        let error = cold_bootstrap_before_startup(&paths.workflow_state_path)
            .expect_err("stale lock requires explicit recovery");
        assert!(error.contains("workflow_state_store_locked"));
        assert!(!config_path.exists());
        assert!(!db_path.exists());
        assert_eq!(
            fs::read(&paths.workflow_state_path).expect("state after stale lock"),
            before
        );
    }

    #[test]
    fn explicit_malformed_storage_config_fails_closed() {
        let fixture = Fixture::new("malformed-config");
        let paths = fixture.paths();
        let config_path =
            crate::workbench_sqlite_storage_mode::storage_mode_path(&paths.workflow_state_path)
                .expect("config path");
        fs::create_dir_all(config_path.parent().expect("config parent"))
            .expect("runtime artifacts");
        fs::write(&config_path, b"{malformed").expect("malformed explicit config");

        let error = initialize_for_ordinary_startup(&paths.workflow_state_path)
            .expect_err("explicit malformed config must stop startup");
        assert!(error.contains("ordinary_product_explicit_storage_config_rejected"));
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &paths.workflow_state_path,
        );
    }

    #[test]
    fn ordinary_startup_rejects_cross_profile_db_before_opening_it() {
        let fixture_a = Fixture::new("profile-a");
        let fixture_b = Fixture::new("profile-b");
        let paths_a = fixture_a.paths();
        let paths_b = fixture_b.paths();
        fs::create_dir_all(
            paths_a
                .workflow_state_path
                .parent()
                .expect("workflow A parent"),
        )
        .expect("workflow A root");
        let state = crate::initial_workflow_state_json(
            "2026-08-11T00:00:00Z",
            "audit:ordinary-product-cross-profile",
            false,
            &paths_a.workflow_state_path,
        );
        fs::write(
            &paths_a.workflow_state_path,
            serde_json::to_vec_pretty(&state).expect("state json"),
        )
        .expect("workflow A state");

        let foreign_db = paths_b.root.join("runtime-artifacts/workbench.sqlite");
        fs::create_dir_all(foreign_db.parent().expect("foreign DB parent"))
            .expect("foreign runtime root");
        let sentinel = b"foreign-profile-db-must-not-be-opened";
        fs::write(&foreign_db, sentinel).expect("foreign DB sentinel");
        let config_path =
            crate::workbench_sqlite_storage_mode::storage_mode_path(&paths_a.workflow_state_path)
                .expect("profile A config path");
        fs::create_dir_all(config_path.parent().expect("profile A config parent"))
            .expect("profile A runtime root");
        fs::write(
            &config_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": crate::workbench_sqlite_storage_mode::STORAGE_MODE_SCHEMA_VERSION,
                "mode": "db_primary_json_projection",
                "workflow_state_path": paths_a.workflow_state_path,
                "confirmed_workflow_state_path": paths_a.workflow_state_path,
                "db_path": foreign_db,
                "confirmed_db_path": foreign_db,
                "denied_path_markers": [],
            }))
            .expect("cross-profile config json"),
        )
        .expect("cross-profile config");

        let error = initialize_for_ordinary_startup(&paths_a.workflow_state_path)
            .expect_err("cross-profile DB binding must stop before startup reconciliation");
        assert_eq!(error, "ordinary_product_storage_config_identity_mismatch");
        assert_eq!(
            fs::read(&foreign_db).expect("foreign DB after rejection"),
            sentinel
        );
        crate::workbench_sqlite_storage_mode::clear_storage_mode_cache_for_path_for_tests(
            &paths_a.workflow_state_path,
        );
    }
}
