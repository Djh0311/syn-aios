//! Server-only project_index base for M1I01R01 / M1I01R02 / M1I01R03 / M3O02 / M5R00.
//!
//! This owner mints opaque `project:<uuid>` values and stores exact aliases as
//! resolver inputs. Ordinary AppState installs a server-only authority that
//! can explicitly register and read those ids. The real ordinary Tauri
//! constructor replays a pre-provisioned identity source before shared
//! product composition. M3O02 adds a restricted typed-id verifier that only
//! revalidates an already-typed `M1ProjectId` against the same ordinary
//! app-data root. It does not create ActorId, RoleRef, ScopeRef,
//! CurrentObjectRef, ExecutionChannel, PermissionProfile,
//! PermissionSnapshotRef, IdentitySnapshot, or M3 RoleSession records.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub(crate) const M1_PROJECT_INDEX_PORT_VERSION: &str = "m1.project-index.read-port.v2";
pub(crate) const M1_PROJECT_INDEX_AUTHORITY_PORT_VERSION: &str =
    "m1.project-index.authority-port.v1";
pub(crate) const M1_TYPED_PROJECT_ID_VERIFIER_PORT_VERSION: &str =
    "m1.project-index.typed-id-verifier.v1";
pub(crate) const M1_PROJECT_INDEX_UNAVAILABLE: &str = "m1_project_index_unavailable";
pub(crate) const M1_PROJECT_ID_FOREIGN_ROOT: &str = "m1_project_id_foreign_root";
pub(crate) const M1_PROJECT_INDEX_SCHEMA_VERSION: &str = "m1.project-index.registry.v2";
pub(crate) const M1_ORDINARY_APP_DATA_DIR_NAME: &str = "local.codex.governance.workbench";
pub(crate) const M1_ORDINARY_REGISTRY_RELATIVE_PATH: &str = "m1/project-index-v1.json";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME: &str =
    "m1-ordinary-project-identity-source-v1.json";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_SCHEMA_VERSION: &str =
    "m1.ordinary-project-identity-source.v1";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_MISSING: &str =
    "m1_ordinary_project_identity_source_missing";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_UNREADABLE: &str =
    "m1_ordinary_project_identity_source_unreadable";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_MALFORMED: &str =
    "m1_ordinary_project_identity_source_malformed";
pub(crate) const M1_ORDINARY_IDENTITY_SOURCE_UNSUPPORTED: &str =
    "m1_ordinary_project_identity_source_unsupported";
const M1_LOCK_RELATIVE_PATH: &str = ".m1-project-index-v1.lock";
const M1_ESTABLISHED_MARKER_RELATIVE_PATH: &str = ".m1-project-index.established";
const M1_ESTABLISHED_MARKER_VALUE: &[u8] = b"m1.project-index.established.v1\n";
const M1_RESOLVER_REVISION: u64 = 1;
const M1_LOCK_RETRY_LIMIT: usize = 256;
const M1_LOCK_RETRY_DELAY: Duration = Duration::from_millis(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1ProjectIndexError {
    pub(crate) code: String,
}

impl M1ProjectIndexError {
    pub(crate) fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    pub(crate) fn unavailable() -> Self {
        Self::new(M1_PROJECT_INDEX_UNAVAILABLE)
    }
}

impl std::fmt::Display for M1ProjectIndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M1ProjectIndexError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct M1ProjectId {
    value: String,
    issued_app_data_root: PathBuf,
}

impl M1ProjectId {
    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1RegisterIsolatedProjectRequest {
    pub(crate) exact_alias: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1RegisterExactAliasRequest {
    pub(crate) exact_alias: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1RegisteredProject {
    pub(crate) project_id: M1ProjectId,
    pub(crate) exact_alias: Option<String>,
    pub(crate) resolver_revision: u64,
    pub(crate) registry_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1ProjectRootRef {
    pub(crate) project_id: String,
    pub(crate) normalized_root_alias: String,
    pub(crate) resolver_revision: u64,
}

pub(crate) trait M1ProjectIndexReadPort {
    fn resolve_canonical_project_id(&self, claim: &str)
        -> Result<M1ProjectId, M1ProjectIndexError>;
    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError>;
    fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError>;
}

#[derive(Clone, Debug)]
pub(crate) struct M1ProjectIndexReadHandle {
    store: M1ProjectIndexStore,
}

#[derive(Clone, Debug)]
struct M1ProjectIndexStore {
    canonical_app_data_root: PathBuf,
    registry_path: PathBuf,
    lock_path: PathBuf,
    established_marker_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct M1ProjectIndexRegistry {
    schema_version: String,
    registry_revision: u64,
    projects: Vec<M1StoredProject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct M1StoredProject {
    project_id: String,
    #[serde(default)]
    exact_alias: Option<String>,
    resolver_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct M1OrdinaryIdentitySourceDocument {
    schema_version: String,
    source_id: String,
    source_revision: u64,
    projects: Vec<M1OrdinaryIdentitySourceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct M1OrdinaryIdentitySourceEntry {
    entry_id: String,
    mode: M1OrdinaryIdentitySourceMode,
    source_ref: String,
    exact_alias: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
enum M1OrdinaryIdentitySourceMode {
    #[serde(rename = "create")]
    Create,
    #[serde(rename = "migrate_legacy_project")]
    MigrateLegacyProject,
}

struct ExclusiveRegistryLock {
    path: PathBuf,
}

impl ExclusiveRegistryLock {
    fn acquire(path: &Path) -> Result<Self, M1ProjectIndexError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_lock_dir_create_failed"))?;
        }
        for _ in 0..M1_LOCK_RETRY_LIMIT {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_file) => {
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    thread::sleep(M1_LOCK_RETRY_DELAY);
                }
                Err(_) => return Err(M1ProjectIndexError::new("m1_project_index_lock_failed")),
            }
        }
        Err(M1ProjectIndexError::new("m1_project_index_lock_timeout"))
    }
}

impl Drop for ExclusiveRegistryLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl M1ProjectIndexReadHandle {
    pub(crate) fn open_ordinary_product(
        app_data_root: &Path,
    ) -> Result<Option<Self>, M1ProjectIndexError> {
        let root =
            admit_existing_clean_root(app_data_root, "m1_ordinary_app_data_root_unavailable")?;
        if root.file_name().and_then(|name| name.to_str()) != Some(M1_ORDINARY_APP_DATA_DIR_NAME) {
            return Err(M1ProjectIndexError::new(
                "m1_ordinary_app_data_root_identity_mismatch",
            ));
        }
        Self::open_from_root(root)
    }

    fn open_from_root(
        canonical_app_data_root: PathBuf,
    ) -> Result<Option<Self>, M1ProjectIndexError> {
        let store = M1ProjectIndexStore::from_root(canonical_app_data_root);
        match store.classify_registry_presence()? {
            RegistryPresence::Absent => Ok(None),
            RegistryPresence::EstablishedMissing => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_missing",
            )),
            RegistryPresence::Present => {
                let _ = store.load_registry(LoadMode::Required)?;
                Ok(Some(Self { store }))
            }
        }
    }
}

impl M1ProjectIndexReadPort for M1ProjectIndexReadHandle {
    fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_canonical_project_id(claim)
    }

    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_exact_alias(alias)
    }

    fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_project_root_ref(project_root_ref)
    }
}

pub(crate) trait M1ProjectIndexAuthorityPort {
    fn register_exact_alias(
        &self,
        request: &M1RegisterExactAliasRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError>;
    fn resolve_canonical_project_id(&self, claim: &str)
        -> Result<M1ProjectId, M1ProjectIndexError>;
    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError>;
}

#[derive(Clone, Debug)]
pub(crate) struct M1ProjectIndexAuthorityHandle {
    store: M1ProjectIndexStore,
}

impl M1ProjectIndexAuthorityHandle {
    pub(crate) fn install_ordinary_product(
        app_data_root: &Path,
    ) -> Result<Self, M1ProjectIndexError> {
        let root =
            admit_existing_clean_root(app_data_root, "m1_ordinary_app_data_root_unavailable")?;
        if root.file_name().and_then(|name| name.to_str()) != Some(M1_ORDINARY_APP_DATA_DIR_NAME) {
            return Err(M1ProjectIndexError::new(
                "m1_ordinary_app_data_root_identity_mismatch",
            ));
        }
        let store = M1ProjectIndexStore::from_root(root);
        if matches!(
            store.classify_registry_presence()?,
            RegistryPresence::Present
        ) {
            let _ = store.load_registry(LoadMode::Required)?;
        }
        Ok(Self { store })
    }

    pub(crate) fn replay_ordinary_identity_source(
        app_data_root: &Path,
    ) -> Result<(), M1ProjectIndexError> {
        let root = admit_ordinary_root_for_identity_source(app_data_root)?;
        let source = load_ordinary_identity_source(&root)?;
        let store = M1ProjectIndexStore::from_root(root);
        store.replay_ordinary_identity_source(&source)
    }

    fn require_readable_registry(&self) -> Result<(), M1ProjectIndexError> {
        match self.store.classify_registry_presence()? {
            RegistryPresence::Absent => Err(M1ProjectIndexError::unavailable()),
            RegistryPresence::EstablishedMissing => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_missing",
            )),
            RegistryPresence::Present => Ok(()),
        }
    }

    pub(crate) fn restricted_typed_project_id_verifier(&self) -> M1TypedProjectIdVerifierHandle {
        M1TypedProjectIdVerifierHandle {
            store: self.store.clone(),
        }
    }

    pub(crate) fn register_exact_alias(
        &self,
        request: &M1RegisterExactAliasRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError> {
        if is_scratch_claim(&request.exact_alias) {
            return Err(M1ProjectIndexError::new(
                "m1_project_id_scratch_claim_rejected",
            ));
        }
        if is_m5_helper_claim(&request.exact_alias) {
            return Err(M1ProjectIndexError::new(
                "m1_project_id_m5_helper_claim_rejected",
            ));
        }
        self.store
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some(request.exact_alias.clone()),
            })
    }

    pub(crate) fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.require_readable_registry()?;
        self.store.resolve_canonical_project_id(claim)
    }

    pub(crate) fn resolve_exact_alias(
        &self,
        alias: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.require_readable_registry()?;
        self.store.resolve_exact_alias(alias)
    }

    pub(crate) fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.require_readable_registry()?;
        self.store.resolve_project_root_ref(project_root_ref)
    }
}

impl M1ProjectIndexAuthorityPort for M1ProjectIndexAuthorityHandle {
    fn register_exact_alias(
        &self,
        request: &M1RegisterExactAliasRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::register_exact_alias(self, request)
    }

    fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::resolve_canonical_project_id(self, claim)
    }

    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::resolve_exact_alias(self, alias)
    }
}

impl M1ProjectIndexReadPort for M1ProjectIndexAuthorityHandle {
    fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::resolve_canonical_project_id(self, claim)
    }

    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::resolve_exact_alias(self, alias)
    }

    fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1ProjectIndexAuthorityHandle::resolve_project_root_ref(self, project_root_ref)
    }
}

/// Server-only AppState slot boundary. A missing handle is uninstalled, not a
/// reason to mint a ProjectId or expose the registry file.
pub(crate) fn require_installed_authority(
    slot: Option<&M1ProjectIndexAuthorityHandle>,
) -> Result<&dyn M1ProjectIndexAuthorityPort, M1ProjectIndexError> {
    slot.map(|handle| handle as &dyn M1ProjectIndexAuthorityPort)
        .ok_or_else(M1ProjectIndexError::unavailable)
}

/// Restricted M1 capability: revalidate an already-typed ProjectId against one
/// ordinary app-data root. It cannot register aliases or hand out storage.
pub(crate) trait M1TypedProjectIdVerifier {
    fn verify_typed_project_id(
        &self,
        project_id: &M1ProjectId,
    ) -> Result<M1ProjectId, M1ProjectIndexError>;
}

#[derive(Clone, Debug)]
pub(crate) struct M1TypedProjectIdVerifierHandle {
    store: M1ProjectIndexStore,
}

impl M1TypedProjectIdVerifierHandle {
    pub(crate) fn verify_typed_project_id(
        &self,
        project_id: &M1ProjectId,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        if project_id.issued_app_data_root != self.store.canonical_app_data_root {
            return Err(M1ProjectIndexError::new(M1_PROJECT_ID_FOREIGN_ROOT));
        }
        match self.store.classify_registry_presence()? {
            RegistryPresence::Absent => Err(M1ProjectIndexError::unavailable()),
            RegistryPresence::EstablishedMissing => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_missing",
            )),
            RegistryPresence::Present => {
                let registry = self.store.load_registry(LoadMode::Required)?;
                registry
                    .projects
                    .iter()
                    .find(|project| project.project_id == project_id.as_str())
                    .map(|project| self.store.issued_project_id(project.project_id.clone()))
                    .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_unknown"))
            }
        }
    }
}

impl M1TypedProjectIdVerifier for M1TypedProjectIdVerifierHandle {
    fn verify_typed_project_id(
        &self,
        project_id: &M1ProjectId,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        M1TypedProjectIdVerifierHandle::verify_typed_project_id(self, project_id)
    }
}

impl M1ProjectIndexStore {
    fn from_root(canonical_app_data_root: PathBuf) -> Self {
        let registry_path = canonical_app_data_root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH);
        let lock_path = canonical_app_data_root.join(M1_LOCK_RELATIVE_PATH);
        let established_marker_path =
            canonical_app_data_root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH);
        Self {
            canonical_app_data_root,
            registry_path,
            lock_path,
            established_marker_path,
        }
    }

    fn issued_project_id(&self, value: impl Into<String>) -> M1ProjectId {
        M1ProjectId {
            value: value.into(),
            issued_app_data_root: self.canonical_app_data_root.clone(),
        }
    }

    fn classify_registry_presence(&self) -> Result<RegistryPresence, M1ProjectIndexError> {
        match fs::symlink_metadata(&self.registry_path) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(RegistryPresence::Present),
            Ok(_) => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if self.established_marker_is_present()? {
                    return Ok(RegistryPresence::EstablishedMissing);
                }
                let registry_dir = self.registry_path.parent();
                match registry_dir {
                    Some(dir) if dir.exists() => Ok(RegistryPresence::EstablishedMissing),
                    _ => Ok(RegistryPresence::Absent),
                }
            }
            Err(_) => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_unreadable",
            )),
        }
    }

    fn established_marker_is_present(&self) -> Result<bool, M1ProjectIndexError> {
        match fs::symlink_metadata(&self.established_marker_path) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(true),
            Ok(_) => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(_) => Err(M1ProjectIndexError::new(
                "m1_project_index_registry_unreadable",
            )),
        }
    }

    fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        reject_non_canonical_project_id_claim(claim)?;
        validate_canonical_project_id(claim)?;
        let registry = self.load_registry(LoadMode::Required)?;
        registry
            .projects
            .iter()
            .find(|project| project.project_id == claim)
            .map(|project| self.issued_project_id(project.project_id.clone()))
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_unknown"))
    }

    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError> {
        let alias = validate_alias_shape(alias)?;
        let registry = self.load_registry(LoadMode::Required)?;
        let matches = registry
            .projects
            .iter()
            .filter(|project| project.exact_alias.as_deref() == Some(alias.as_str()))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [project] => Ok(self.issued_project_id(project.project_id.clone())),
            [] => Err(M1ProjectIndexError::new("m1_alias_unknown")),
            _ => Err(M1ProjectIndexError::new("m1_alias_duplicate")),
        }
    }

    fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        let project_id = self.resolve_canonical_project_id(&project_root_ref.project_id)?;
        let alias = validate_alias_shape(&project_root_ref.normalized_root_alias)?;
        let registry = self.load_registry(LoadMode::Required)?;
        let stored = registry
            .projects
            .iter()
            .find(|project| project.project_id == project_id.as_str())
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_unknown"))?;
        match stored.exact_alias.as_deref() {
            Some(stored_alias) if stored_alias == alias => {}
            Some(_) | None => return Err(M1ProjectIndexError::new("m1_alias_mismatch")),
        }
        if stored.resolver_revision != project_root_ref.resolver_revision {
            return Err(M1ProjectIndexError::new("m1_resolver_revision_stale"));
        }
        Ok(project_id)
    }

    fn register_isolated_project(
        &self,
        request: M1RegisterIsolatedProjectRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError> {
        let exact_alias = match request.exact_alias {
            Some(alias) => Some(validate_alias_shape(&alias)?),
            None => None,
        };
        let _lock = ExclusiveRegistryLock::acquire(&self.lock_path)?;
        let mut registry = match self.classify_registry_presence()? {
            RegistryPresence::Absent => empty_registry(),
            RegistryPresence::EstablishedMissing => {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_missing",
                ));
            }
            RegistryPresence::Present => self.load_registry(LoadMode::Required)?,
        };
        if let Some(alias) = exact_alias.as_deref() {
            if registry
                .projects
                .iter()
                .any(|project| project.exact_alias.as_deref() == Some(alias))
            {
                return Err(M1ProjectIndexError::new("m1_alias_duplicate"));
            }
        }

        let project_id = format!("project:{}", Uuid::new_v4());
        validate_canonical_project_id(&project_id)?;
        let next_revision = registry
            .registry_revision
            .checked_add(1)
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_index_revision_overflow"))?;
        registry.registry_revision = next_revision;
        registry.projects.push(M1StoredProject {
            project_id: project_id.clone(),
            exact_alias: exact_alias.clone(),
            resolver_revision: M1_RESOLVER_REVISION,
        });
        self.persist_registry(&registry)?;
        Ok(M1RegisteredProject {
            project_id: self.issued_project_id(project_id),
            exact_alias,
            resolver_revision: M1_RESOLVER_REVISION,
            registry_revision: registry.registry_revision,
        })
    }

    fn replay_ordinary_identity_source(
        &self,
        source: &M1OrdinaryIdentitySourceDocument,
    ) -> Result<(), M1ProjectIndexError> {
        let _lock = ExclusiveRegistryLock::acquire(&self.lock_path)?;
        let mut registry = match self.classify_registry_presence()? {
            RegistryPresence::Absent => empty_registry(),
            RegistryPresence::EstablishedMissing => {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_missing",
                ));
            }
            RegistryPresence::Present => self.load_registry(LoadMode::Required)?,
        };
        let mut added = false;
        for entry in &source.projects {
            let matches = registry
                .projects
                .iter()
                .filter(|project| project.exact_alias.as_deref() == Some(entry.exact_alias.as_str()))
                .count();
            match matches {
                1 => {}
                0 => {
                    let project_id = format!("project:{}", Uuid::new_v4());
                    validate_canonical_project_id(&project_id)?;
                    let next_revision = registry.registry_revision.checked_add(1).ok_or_else(
                        || M1ProjectIndexError::new("m1_project_index_revision_overflow"),
                    )?;
                    registry.registry_revision = next_revision;
                    registry.projects.push(M1StoredProject {
                        project_id,
                        exact_alias: Some(entry.exact_alias.clone()),
                        resolver_revision: M1_RESOLVER_REVISION,
                    });
                    added = true;
                }
                _ => {
                    return Err(M1ProjectIndexError::new(
                        "m1_project_index_registry_malformed",
                    ));
                }
            }
        }
        if added {
            self.persist_registry(&registry)?;
        }
        Ok(())
    }

    fn load_registry(&self, mode: LoadMode) -> Result<M1ProjectIndexRegistry, M1ProjectIndexError> {
        if !self.registry_path.exists() {
            return match mode {
                LoadMode::Required => Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_missing",
                )),
                LoadMode::OptionalAbsent => Ok(empty_registry()),
            };
        }
        let metadata = fs::symlink_metadata(&self.registry_path)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_unreadable"))?;
        if !metadata.file_type().is_file() {
            return Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            ));
        }
        let bytes = fs::read(&self.registry_path)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_unreadable"))?;
        let registry: M1ProjectIndexRegistry = serde_json::from_slice(&bytes)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_malformed"))?;
        if registry.schema_version != M1_PROJECT_INDEX_SCHEMA_VERSION {
            return Err(M1ProjectIndexError::new(
                "m1_project_index_registry_unsupported",
            ));
        }
        validate_loaded_registry(&registry)?;
        Ok(registry)
    }

    fn persist_registry(
        &self,
        registry: &M1ProjectIndexRegistry,
    ) -> Result<(), M1ProjectIndexError> {
        validate_loaded_registry(registry)?;
        self.persist_established_marker()?;
        let parent = self
            .registry_path
            .parent()
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_index_registry_parent_required"))?;
        fs::create_dir_all(parent)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_create_failed"))?;
        let text = serde_json::to_string_pretty(registry)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_serialize_failed"))?;
        let temp_path = parent.join(format!(".project-index-v1.{}.tmp", Uuid::new_v4().simple()));
        {
            let mut file = File::create(&temp_path)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_create_failed"))?;
            file.write_all(text.as_bytes())
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_write_failed"))?;
            file.sync_all()
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_sync_failed"))?;
        }
        fs::rename(&temp_path, &self.registry_path)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_replace_failed"))?;
        let dir = File::open(parent)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_open_failed"))?;
        dir.sync_all()
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_sync_failed"))?;
        Ok(())
    }

    fn persist_established_marker(&self) -> Result<(), M1ProjectIndexError> {
        if self.established_marker_is_present()? {
            return Ok(());
        }
        let parent = self
            .established_marker_path
            .parent()
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_index_registry_parent_required"))?;
        let temp_path = parent.join(format!(
            ".m1-project-index.established.{}.tmp",
            Uuid::new_v4().simple()
        ));
        {
            let mut file = File::create(&temp_path)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_create_failed"))?;
            file.write_all(M1_ESTABLISHED_MARKER_VALUE)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_write_failed"))?;
            file.sync_all()
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_sync_failed"))?;
        }
        fs::rename(&temp_path, &self.established_marker_path)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_replace_failed"))?;
        let dir = File::open(parent)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_open_failed"))?;
        dir.sync_all()
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_sync_failed"))?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum LoadMode {
    Required,
    OptionalAbsent,
}

#[derive(Clone, Copy)]
enum RegistryPresence {
    Absent,
    EstablishedMissing,
    Present,
}

fn empty_registry() -> M1ProjectIndexRegistry {
    M1ProjectIndexRegistry {
        schema_version: M1_PROJECT_INDEX_SCHEMA_VERSION.to_string(),
        registry_revision: 0,
        projects: Vec::new(),
    }
}

fn validate_loaded_registry(registry: &M1ProjectIndexRegistry) -> Result<(), M1ProjectIndexError> {
    if registry.schema_version != M1_PROJECT_INDEX_SCHEMA_VERSION {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_registry_unsupported",
        ));
    }
    let project_count = u64::try_from(registry.projects.len())
        .map_err(|_| M1ProjectIndexError::new("m1_project_index_revision_overflow"))?;
    if registry.registry_revision != project_count {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_registry_malformed",
        ));
    }
    let mut seen_ids = Vec::new();
    let mut seen_aliases = Vec::new();
    for project in &registry.projects {
        validate_canonical_project_id(&project.project_id)?;
        if seen_ids.contains(&project.project_id) {
            return Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            ));
        }
        seen_ids.push(project.project_id.clone());
        if project.resolver_revision != M1_RESOLVER_REVISION {
            return Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            ));
        }
        if let Some(alias) = project.exact_alias.as_deref() {
            validate_alias_shape(alias)?;
            if seen_aliases.iter().any(|item: &String| item == alias) {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_malformed",
                ));
            }
            seen_aliases.push(alias.to_string());
        }
    }
    Ok(())
}

fn validate_canonical_project_id(value: &str) -> Result<Uuid, M1ProjectIndexError> {
    let uuid = parse_prefixed_uuid("project:", value)
        .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_malformed"))?;
    if uuid.get_version() != Some(uuid::Version::Random) {
        return Err(M1ProjectIndexError::new("m1_project_id_malformed"));
    }
    Ok(uuid)
}

fn parse_prefixed_uuid(prefix: &str, value: &str) -> Option<Uuid> {
    let rest = value.strip_prefix(prefix)?;
    let uuid = Uuid::parse_str(rest).ok()?;
    if value == format!("{prefix}{uuid}") {
        Some(uuid)
    } else {
        None
    }
}

fn validate_alias_shape(alias: &str) -> Result<String, M1ProjectIndexError> {
    if alias.is_empty()
        || alias.len() > 1024
        || alias
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || is_caller_boolean(alias)
    {
        return Err(M1ProjectIndexError::new("m1_alias_malformed"));
    }
    if parse_prefixed_uuid("project:", alias).is_some() {
        return Err(M1ProjectIndexError::new("m1_alias_malformed"));
    }
    Ok(alias.to_string())
}

fn reject_non_canonical_project_id_claim(claim: &str) -> Result<(), M1ProjectIndexError> {
    if claim.is_empty()
        || claim
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || is_caller_boolean(claim)
    {
        return Err(M1ProjectIndexError::new("m1_project_id_malformed"));
    }
    if is_scratch_claim(claim) {
        return Err(M1ProjectIndexError::new(
            "m1_project_id_scratch_claim_rejected",
        ));
    }
    if is_m5_helper_claim(claim) {
        return Err(M1ProjectIndexError::new(
            "m1_project_id_m5_helper_claim_rejected",
        ));
    }
    if is_path_claim(claim) {
        return Err(M1ProjectIndexError::new(
            "m1_project_id_path_claim_rejected",
        ));
    }
    if parse_prefixed_uuid("project:", claim).is_some() {
        return Ok(());
    }
    if claim.starts_with("project:") {
        return Err(M1ProjectIndexError::new(
            "m1_project_id_path_claim_rejected",
        ));
    }
    Err(M1ProjectIndexError::new(
        "m1_project_id_index_locator_claim_rejected",
    ))
}

fn is_caller_boolean(value: &str) -> bool {
    matches!(
        value,
        "true" | "false" | "TRUE" | "FALSE" | "True" | "False"
    )
}

fn is_scratch_claim(value: &str) -> bool {
    value.starts_with("scratch-") || value.starts_with("project:scratch-")
}

fn is_m5_helper_claim(value: &str) -> bool {
    value.starts_with("m5:")
        || value.contains("official_project_id")
        || value.contains("resolve_project_id_from_index")
        || value.contains("m5_m3_identity")
}

fn is_path_claim(value: &str) -> bool {
    value.contains('/')
        || value.contains('\\')
        || value.starts_with('.')
        || Path::new(value).is_absolute()
}

fn admit_ordinary_root_for_identity_source(
    app_data_root: &Path,
) -> Result<PathBuf, M1ProjectIndexError> {
    if !app_data_root.is_absolute()
        || app_data_root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_clean_absolute_root_required",
        ));
    }
    if app_data_root.file_name().and_then(|name| name.to_str())
        != Some(M1_ORDINARY_APP_DATA_DIR_NAME)
    {
        return Err(M1ProjectIndexError::new(
            "m1_ordinary_app_data_root_identity_mismatch",
        ));
    }
    match fs::symlink_metadata(app_data_root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(
            M1ProjectIndexError::new("m1_project_index_regular_root_required"),
        ),
        Ok(_) => {
            let canonical = fs::canonicalize(app_data_root).map_err(|_| {
                M1ProjectIndexError::new("m1_ordinary_app_data_root_unavailable")
            })?;
            if canonical != app_data_root {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_root_identity_changed",
                ));
            }
            Ok(canonical)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Err(M1ProjectIndexError::new(M1_ORDINARY_IDENTITY_SOURCE_MISSING))
        }
        Err(_) => Err(M1ProjectIndexError::new(
            "m1_ordinary_app_data_root_unavailable",
        )),
    }
}

fn load_ordinary_identity_source(
    root: &Path,
) -> Result<M1OrdinaryIdentitySourceDocument, M1ProjectIndexError> {
    let path = root.join(M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(M1ProjectIndexError::new(M1_ORDINARY_IDENTITY_SOURCE_MISSING));
        }
        Err(_) => {
            return Err(M1ProjectIndexError::new(
                M1_ORDINARY_IDENTITY_SOURCE_UNREADABLE,
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(M1ProjectIndexError::new(
            M1_ORDINARY_IDENTITY_SOURCE_MALFORMED,
        ));
    }
    let bytes = fs::read(&path).map_err(|_| {
        M1ProjectIndexError::new(M1_ORDINARY_IDENTITY_SOURCE_UNREADABLE)
    })?;
    let document: M1OrdinaryIdentitySourceDocument = serde_json::from_slice(&bytes)
        .map_err(|_| M1ProjectIndexError::new(M1_ORDINARY_IDENTITY_SOURCE_MALFORMED))?;
    if document.schema_version != M1_ORDINARY_IDENTITY_SOURCE_SCHEMA_VERSION {
        return Err(M1ProjectIndexError::new(
            M1_ORDINARY_IDENTITY_SOURCE_UNSUPPORTED,
        ));
    }
    validate_ordinary_identity_source(&document)?;
    Ok(document)
}

fn validate_source_token(value: &str) -> Result<(), M1ProjectIndexError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(M1ProjectIndexError::new(
            M1_ORDINARY_IDENTITY_SOURCE_MALFORMED,
        ));
    }
    Ok(())
}

fn validate_ordinary_identity_source(
    document: &M1OrdinaryIdentitySourceDocument,
) -> Result<(), M1ProjectIndexError> {
    validate_source_token(&document.source_id)?;
    if document.source_revision == 0 || document.projects.is_empty() {
        return Err(M1ProjectIndexError::new(
            M1_ORDINARY_IDENTITY_SOURCE_MALFORMED,
        ));
    }
    let mut entry_ids = Vec::new();
    let mut aliases = Vec::new();
    for entry in &document.projects {
        validate_source_token(&entry.entry_id)?;
        validate_source_token(&entry.source_ref)?;
        if validate_alias_shape(&entry.exact_alias).is_err()
            || is_scratch_claim(&entry.exact_alias)
            || is_m5_helper_claim(&entry.exact_alias)
        {
            return Err(M1ProjectIndexError::new(
                M1_ORDINARY_IDENTITY_SOURCE_MALFORMED,
            ));
        }
        if entry_ids.iter().any(|item: &String| item == &entry.entry_id)
            || aliases
                .iter()
                .any(|item: &String| item == &entry.exact_alias)
        {
            return Err(M1ProjectIndexError::new(
                M1_ORDINARY_IDENTITY_SOURCE_MALFORMED,
            ));
        }
        entry_ids.push(entry.entry_id.clone());
        aliases.push(entry.exact_alias.clone());
    }
    Ok(())
}

fn admit_existing_clean_root(
    root: &Path,
    unavailable_code: &str,
) -> Result<PathBuf, M1ProjectIndexError> {
    if !root.is_absolute()
        || root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_clean_absolute_root_required",
        ));
    }
    let metadata =
        fs::symlink_metadata(root).map_err(|_| M1ProjectIndexError::new(unavailable_code))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_regular_root_required",
        ));
    }
    let canonical =
        fs::canonicalize(root).map_err(|_| M1ProjectIndexError::new(unavailable_code))?;
    if canonical != root {
        return Err(M1ProjectIndexError::new(
            "m1_project_index_root_identity_changed",
        ));
    }
    Ok(canonical)
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct M1ProjectIndexRegistrar {
    store: M1ProjectIndexStore,
}

#[cfg(test)]
impl M1ProjectIndexRegistrar {
    pub(crate) fn open_isolated_fixture(root: &Path) -> Result<Self, M1ProjectIndexError> {
        let canonical_temp = fs::canonicalize(std::env::temp_dir())
            .map_err(|_| M1ProjectIndexError::new("m1_isolated_temp_root_unavailable"))?;
        let canonical_root =
            admit_existing_clean_root(root, "m1_isolated_fixture_root_unavailable")?;
        let admitted_name = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if !canonical_root.starts_with(&canonical_temp)
            || !admitted_name.starts_with("syn-m1i01r01-")
        {
            return Err(M1ProjectIndexError::new(
                "m1_isolated_fixture_root_not_admitted",
            ));
        }
        Ok(Self {
            store: M1ProjectIndexStore::from_root(canonical_root),
        })
    }

    pub(crate) fn register_isolated_project(
        &self,
        request: M1RegisterIsolatedProjectRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError> {
        self.store.register_isolated_project(request)
    }

    pub(crate) fn read_handle(&self) -> Result<M1ProjectIndexReadHandle, M1ProjectIndexError> {
        M1ProjectIndexReadHandle::open_from_root(self.store.canonical_app_data_root.clone())?
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_index_unavailable"))
    }

    pub(crate) fn restricted_typed_project_id_verifier(&self) -> M1TypedProjectIdVerifierHandle {
        M1TypedProjectIdVerifierHandle {
            store: self.store.clone(),
        }
    }
}

#[cfg(test)]
impl M1ProjectIndexReadPort for M1ProjectIndexRegistrar {
    fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_canonical_project_id(claim)
    }

    fn resolve_exact_alias(&self, alias: &str) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_exact_alias(alias)
    }

    fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        self.store.resolve_project_root_ref(project_root_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn isolated_root() -> PathBuf {
        let root = std::env::temp_dir().join(format!("syn-m1i01r01-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create isolated root");
        fs::canonicalize(&root).expect("canonicalize isolated root")
    }

    fn isolated_registrar() -> (M1ProjectIndexRegistrar, PathBuf) {
        let root = isolated_root();
        let registrar =
            M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("open registrar");
        (registrar, root)
    }

    fn slug_like_id(project_root: &str) -> String {
        let mut output = String::new();
        for character in project_root.chars() {
            if character.is_ascii_alphanumeric() {
                output.push(character.to_ascii_lowercase());
            } else if !output.ends_with('-') {
                output.push('-');
            }
        }
        format!(
            "project:{}",
            output
                .trim_matches('-')
                .chars()
                .take(96)
                .collect::<String>()
        )
    }

    #[test]
    fn m1_project_index_register_mints_opaque_uuid_not_path_derived() {
        let (registrar, root) = isolated_registrar();
        let alias = format!("{}/isolated-project", root.display());
        let registered = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some(alias.clone()),
            })
            .expect("register");
        assert!(registered.project_id.as_str().starts_with("project:"));
        validate_canonical_project_id(registered.project_id.as_str()).expect("uuid id");
        assert_ne!(registered.project_id.as_str(), slug_like_id(&alias));
        assert!(!registered.project_id.as_str().contains(&alias));
        assert!(!registered.project_id.as_str().contains('/'));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_restart_restores_same_project_id() {
        let (registrar, root) = isolated_registrar();
        let registered = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/tmp/syn-m1i01r01-restart-alias".to_string()),
            })
            .expect("register");
        drop(registrar);

        let reopened = M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("reopen");
        let after_id = reopened
            .resolve_exact_alias("/tmp/syn-m1i01r01-restart-alias")
            .expect("alias after restart");
        assert_eq!(after_id.as_str(), registered.project_id.as_str());
        let read = reopened.read_handle().expect("read after restart");
        assert_eq!(
            read.resolve_canonical_project_id(registered.project_id.as_str())
                .expect("id after restart")
                .as_str(),
            registered.project_id.as_str()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_exact_alias_resolves_only_when_pre_registered() {
        let (registrar, root) = isolated_registrar();
        let registered = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/one".to_string()),
            })
            .expect("register");
        assert_eq!(
            registrar
                .resolve_exact_alias("/exact/alias/one")
                .expect("exact hit")
                .as_str(),
            registered.project_id.as_str()
        );
        assert_eq!(
            registrar
                .resolve_exact_alias("/exact/alias/ONE")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_unknown_malformed_stale_and_mismatch() {
        let (registrar, root) = isolated_registrar();
        let registered = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/keep".to_string()),
            })
            .expect("register");

        assert_eq!(
            registrar
                .resolve_exact_alias("/exact/alias/missing")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        assert_eq!(
            registrar
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/exact/alias/keep".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_alias_duplicate"
        );
        assert_eq!(
            registrar.resolve_exact_alias("").unwrap_err().code,
            "m1_alias_malformed"
        );
        assert_eq!(
            registrar.resolve_exact_alias("true").unwrap_err().code,
            "m1_alias_malformed"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("project:00000000-0000-4000-8000-000000000000")
                .unwrap_err()
                .code,
            "m1_project_id_unknown"
        );
        assert_eq!(
            registrar
                .resolve_project_root_ref(&M1ProjectRootRef {
                    project_id: registered.project_id.as_str().to_string(),
                    normalized_root_alias: "/exact/alias/other".to_string(),
                    resolver_revision: 1,
                })
                .unwrap_err()
                .code,
            "m1_alias_mismatch"
        );
        assert_eq!(
            registrar
                .resolve_project_root_ref(&M1ProjectRootRef {
                    project_id: registered.project_id.as_str().to_string(),
                    normalized_root_alias: "/exact/alias/keep".to_string(),
                    resolver_revision: 2,
                })
                .unwrap_err()
                .code,
            "m1_resolver_revision_stale"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_path_locator_scratch_m5_and_boolean_as_project_id() {
        let (registrar, root) = isolated_registrar();
        let _ = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/tmp/syn-m1i01r01-bound".to_string()),
            })
            .expect("register");

        assert_eq!(
            registrar
                .resolve_canonical_project_id("/tmp/syn-m1i01r01-bound")
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id(&slug_like_id("/tmp/syn-m1i01r01-bound"))
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("mario-test")
                .unwrap_err()
                .code,
            "m1_project_id_index_locator_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("scratch-isolated")
                .unwrap_err()
                .code,
            "m1_project_id_scratch_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("project:scratch-isolated")
                .unwrap_err()
                .code,
            "m1_project_id_scratch_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("m5:official_project_id")
                .unwrap_err()
                .code,
            "m1_project_id_m5_helper_claim_rejected"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("true")
                .unwrap_err()
                .code,
            "m1_project_id_malformed"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_does_not_import_legacy_index_records() {
        let (registrar, root) = isolated_registrar();
        let _ = registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/registered/only".to_string()),
            })
            .expect("register");
        fs::write(
            root.join("codex-index.json"),
            r#"{"projects":[{"project_root":"/legacy/never-imported"}]}"#,
        )
        .expect("write legacy index");
        assert_eq!(
            registrar
                .resolve_exact_alias("/legacy/never-imported")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        assert_eq!(
            registrar
                .resolve_canonical_project_id("/legacy/never-imported")
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_read_open_does_not_create_blank_registry() {
        let root = isolated_root();
        let opened = M1ProjectIndexReadHandle::open_from_root(root.clone()).expect("open read");
        assert!(opened.is_none());
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        assert!(!root.join("m1").exists());
        assert!(!root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_corrupt_and_missing_established_registry() {
        let (registrar, root) = isolated_registrar();
        registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/established".to_string()),
            })
            .expect("register");
        let registry_path = root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH);

        fs::write(&registry_path, "{not-json").expect("corrupt registry");
        assert_eq!(
            M1ProjectIndexReadHandle::open_from_root(root.clone())
                .unwrap_err()
                .code,
            "m1_project_index_registry_malformed"
        );
        assert_eq!(
            registrar
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/exact/alias/after-corrupt".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_project_index_registry_malformed"
        );
        assert_eq!(
            fs::read_to_string(&registry_path).expect("retain corrupt bytes"),
            "{not-json"
        );

        fs::remove_file(&registry_path).expect("delete established registry");
        assert!(root.join("m1").exists());
        assert_eq!(
            M1ProjectIndexReadHandle::open_from_root(root.clone())
                .unwrap_err()
                .code,
            "m1_project_index_registry_missing"
        );
        assert_eq!(
            registrar
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/exact/alias/after-missing".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_project_index_registry_missing"
        );
        assert!(!registry_path.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_whole_m1_directory_loss() {
        let (registrar, root) = isolated_registrar();
        registrar
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/directory-loss".to_string()),
            })
            .expect("register");
        let marker_path = root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH);
        let registry_dir = root.join("m1");
        assert!(marker_path.is_file());
        assert_ne!(
            marker_path.parent().expect("marker parent"),
            registry_dir.as_path()
        );
        fs::remove_dir_all(&registry_dir).expect("delete established m1 directory");
        assert!(marker_path.is_file());
        assert!(!registry_dir.exists());
        assert_eq!(
            M1ProjectIndexReadHandle::open_from_root(root.clone())
                .unwrap_err()
                .code,
            "m1_project_index_registry_missing"
        );
        assert_eq!(
            registrar
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/exact/alias/after-directory-loss".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_project_index_registry_missing"
        );
        assert!(!registry_dir.exists());
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_unsupported_v1_identity_registry() {
        let (registrar, root) = isolated_registrar();
        let registry_dir = root.join("m1");
        fs::create_dir_all(&registry_dir).expect("create registry dir");
        fs::write(
            root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH),
            r#"{
              "schema_version": "m1.project-index.registry.v1",
              "registry_revision": 1,
              "projects": [{
                "project_id": "project:11111111-1111-4111-8111-111111111111",
                "exact_alias": "/legacy-v1",
                "resolver_revision": 1
              }]
            }"#,
        )
        .expect("write unsupported v1 registry");
        assert_eq!(
            M1ProjectIndexReadHandle::open_from_root(root.clone())
                .unwrap_err()
                .code,
            "m1_project_index_registry_unsupported"
        );
        assert_eq!(
            registrar
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/should-not-import-v1".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_project_index_registry_unsupported"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_v1_identity_fields_without_import() {
        let root = isolated_root();
        let registry_dir = root.join("m1");
        fs::create_dir_all(&registry_dir).expect("create registry dir");
        fs::write(
            root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH),
            r#"{
              "schema_version": "m1.project-index.registry.v2",
              "registry_revision": 1,
              "projects": [{
                "project_id": "project:11111111-1111-4111-8111-111111111111",
                "exact_alias": "/must-not-import",
                "resolver_revision": 1,
                "actor_id": "actor:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                "role_ref": "role:project_supervisor",
                "scope_ref": "scope:bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
                "identity_snapshot": {"kind":"identity"}
              }]
            }"#,
        )
        .expect("write identity-bearing registry");
        assert_eq!(
            M1ProjectIndexReadHandle::open_from_root(root.clone())
                .unwrap_err()
                .code,
            "m1_project_index_registry_malformed"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_two_authorities_same_alias_only_one_succeeds() {
        let root = isolated_root();
        let first = M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("first");
        let second = M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("second");
        let first = Arc::new(first);
        let second = Arc::new(second);
        let left = Arc::clone(&first);
        let right = Arc::clone(&second);
        let handle_one = thread::spawn(move || {
            left.register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/shared/alias".to_string()),
            })
        });
        let handle_two = thread::spawn(move || {
            right.register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/shared/alias".to_string()),
            })
        });
        let first_result = handle_one.join().expect("first join");
        let second_result = handle_two.join().expect("second join");
        let successes = [&first_result, &second_result]
            .iter()
            .filter(|result| result.is_ok())
            .count();
        let duplicates = [&first_result, &second_result]
            .iter()
            .filter(|result| {
                result
                    .as_ref()
                    .err()
                    .is_some_and(|error| error.code == "m1_alias_duplicate")
            })
            .count();
        assert_eq!(successes, 1);
        assert_eq!(duplicates, 1);
        let winner = first_result
            .as_ref()
            .ok()
            .or(second_result.as_ref().ok())
            .expect("one winner");
        assert_eq!(
            first
                .resolve_exact_alias("/shared/alias")
                .expect("stored alias")
                .as_str(),
            winner.project_id.as_str()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_two_authorities_different_aliases_keep_both_updates() {
        let root = isolated_root();
        let first = M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("first");
        let second = M1ProjectIndexRegistrar::open_isolated_fixture(&root).expect("second");
        let first = Arc::new(first);
        let second = Arc::new(second);
        let left = Arc::clone(&first);
        let right = Arc::clone(&second);
        let handle_one = thread::spawn(move || {
            left.register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/alias/alpha".to_string()),
            })
        });
        let handle_two = thread::spawn(move || {
            right.register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/alias/beta".to_string()),
            })
        });
        let first_id = handle_one.join().expect("first join").expect("alpha");
        let second_id = handle_two.join().expect("second join").expect("beta");
        assert_ne!(first_id.project_id.as_str(), second_id.project_id.as_str());
        assert_eq!(
            first
                .resolve_exact_alias("/alias/alpha")
                .expect("alpha kept")
                .as_str(),
            first_id.project_id.as_str()
        );
        assert_eq!(
            first
                .resolve_exact_alias("/alias/beta")
                .expect("beta kept")
                .as_str(),
            second_id.project_id.as_str()
        );
        let _ = fs::remove_dir_all(root);
    }

    fn ordinary_named_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!("syn-m1i01r03-{}", Uuid::new_v4()));
        let root = parent.join(M1_ORDINARY_APP_DATA_DIR_NAME);
        fs::create_dir_all(&root).expect("create ordinary named root");
        fs::canonicalize(&root).expect("canonicalize ordinary named root")
    }

    fn ordinary_app_state(app_data_root: &Path) -> crate::AppState {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        crate::AppState::try_new_with_ordinary_product_ports(
            app_data_root,
            &manifest_dir.join("../../index-kernel/codex-index.json"),
            &manifest_dir.join("../../../tasks/README.md"),
            crate::m4_secretary_conversation::M4SecretaryConversationProviderConfig::Unavailable,
        )
        .expect("ordinary product AppState must construct")
    }

    fn isolated_acceptance_app_state() -> (PathBuf, crate::AppState) {
        let root = std::env::temp_dir().join(format!("syn-m1i01r03r01-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("app-data")).expect("create isolated profile");
        let root = fs::canonicalize(&root).expect("canonicalize isolated profile");
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let paths = crate::acceptance_runtime_profile::RuntimePaths {
            root: root.clone(),
            index_path: manifest_dir.join("../../index-kernel/codex-index.json"),
            tasks_path: manifest_dir.join("../../../tasks/README.md"),
            project_root: root.join("project"),
            workflow_state_path: root.join("workflow-state.json"),
            app_data_root: root.join("app-data"),
            vault_root: root.join("vault"),
            recovery_backups_root: root.join("recovery"),
            canvas_root: root.join("canvas"),
            codex_db_path: root.join("codex.sqlite"),
            app_log_dir: root.join("logs"),
        };
        let state = crate::AppState::try_new_with_isolated_product_profile(&paths)
            .expect("isolated acceptance AppState must construct");
        (root, state)
    }

    fn assert_unavailable(result: Result<impl Sized, M1ProjectIndexError>) {
        assert_eq!(
            result.err().expect("unavailable").code,
            M1_PROJECT_INDEX_UNAVAILABLE
        );
    }

    #[test]
    fn m1_project_index_uninstalled_app_state_authority_is_unavailable() {
        let legacy = crate::AppState::try_new().expect("legacy AppState");
        assert_unavailable(legacy.m1_project_index_authority().map(|_| ()));
        let fixture = crate::AppState {
            index_path: PathBuf::from("/m1i01r03/fixture/index.json"),
            tasks_path: PathBuf::from("/m1i01r03/fixture/tasks.md"),
            workflow_state_path: PathBuf::from("/m1i01r03/fixture/workflow-state.json"),
            m3_role_session_read_runtime: Default::default(),
            m1_project_index: None,
            m3_project_role_session_authority: None,
            m5_store_path: None,
        };
        assert_unavailable(fixture.m1_project_index_authority().map(|_| ()));
    }

    #[test]
    fn m1_project_index_isolated_acceptance_app_state_authority_is_unavailable() {
        let (root, isolated) = isolated_acceptance_app_state();
        assert_unavailable(isolated.m1_project_index_authority().map(|_| ()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_ordinary_install_does_not_auto_register() {
        let root = ordinary_named_root();
        let handle =
            M1ProjectIndexAuthorityHandle::install_ordinary_product(&root).expect("install");
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        assert!(!root.join("m1").exists());
        assert!(!root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH).exists());
        assert_unavailable(handle.resolve_exact_alias("never-registered"));
        assert_unavailable(
            handle.resolve_canonical_project_id("project:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"),
        );
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_project_index_ordinary_authority_registers_and_survives_app_state_reconstruction() {
        let root = ordinary_named_root();
        let first = ordinary_app_state(&root);
        let registered = first
            .m1_project_index_authority()
            .expect("ordinary authority")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m1i01r03-explicit-alias".to_string(),
            })
            .expect("explicit register");
        validate_canonical_project_id(registered.project_id.as_str()).expect("uuid id");
        assert!(registered.project_id.as_str().starts_with("project:"));
        assert_ne!(
            registered.project_id.as_str(),
            "syn-m1i01r03-explicit-alias"
        );
        drop(first);

        let reconstructed = ordinary_app_state(&root);
        let authority = reconstructed
            .m1_project_index_authority()
            .expect("reconstructed authority");
        assert_eq!(
            authority
                .resolve_exact_alias("syn-m1i01r03-explicit-alias")
                .expect("alias after reconstruction")
                .as_str(),
            registered.project_id.as_str()
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id(registered.project_id.as_str())
                .expect("id after reconstruction")
                .as_str(),
            registered.project_id.as_str()
        );
        let persisted = fs::read_to_string(root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH))
            .expect("read persisted registry");
        assert!(!persisted.contains("actor_id"));
        assert!(!persisted.contains("role_ref"));
        assert!(!persisted.contains("identity_snapshot"));
        assert!(!persisted.contains("permission"));
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_project_index_ordinary_authority_rejects_illegitimate_alias_sources() {
        let root = ordinary_named_root();
        let handle =
            M1ProjectIndexAuthorityHandle::install_ordinary_product(&root).expect("install");
        assert_eq!(
            handle
                .register_exact_alias(&M1RegisterExactAliasRequest {
                    exact_alias: "scratch-auto".to_string(),
                })
                .unwrap_err()
                .code,
            "m1_project_id_scratch_claim_rejected"
        );
        assert_eq!(
            handle
                .register_exact_alias(&M1RegisterExactAliasRequest {
                    exact_alias: "m5:official_project_id".to_string(),
                })
                .unwrap_err()
                .code,
            "m1_project_id_m5_helper_claim_rejected"
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_typed_project_id_verifier_revalidates_same_root_and_rejects_foreign_root() {
        let root_a = ordinary_named_root();
        let root_b = ordinary_named_root();
        let authority_a =
            M1ProjectIndexAuthorityHandle::install_ordinary_product(&root_a).expect("install a");
        let authority_b =
            M1ProjectIndexAuthorityHandle::install_ordinary_product(&root_b).expect("install b");
        let registered = authority_a
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o02-same-root".to_string(),
            })
            .expect("register on a");
        let verifier_a = authority_a.restricted_typed_project_id_verifier();
        let verifier_b = authority_b.restricted_typed_project_id_verifier();
        assert_eq!(
            verifier_a
                .verify_typed_project_id(&registered.project_id)
                .expect("same root")
                .as_str(),
            registered.project_id.as_str()
        );
        assert_eq!(
            verifier_b
                .verify_typed_project_id(&registered.project_id)
                .unwrap_err()
                .code,
            M1_PROJECT_ID_FOREIGN_ROOT
        );
        let verifier_src = include_str!("m1_project_index.rs");
        assert!(verifier_src.contains("fn verify_typed_project_id"));
        assert!(
            !verifier_src.contains(&format!(
                "impl {} for {}",
                "M1ProjectIndexAuthorityPort", "M1TypedProjectIdVerifierHandle"
            )),
            "restricted verifier must not expose register/storage authority"
        );
        let _ = fs::remove_dir_all(root_a.parent().expect("parent"));
        let _ = fs::remove_dir_all(root_b.parent().expect("parent"));
    }

    #[test]
    fn m1_typed_project_id_verifier_fails_closed_on_absent_missing_corrupt_and_unknown() {
        let root = ordinary_named_root();
        let authority =
            M1ProjectIndexAuthorityHandle::install_ordinary_product(&root).expect("install");
        let registered = authority
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: "syn-m3o02-revalidate".to_string(),
            })
            .expect("register");
        let verifier = authority.restricted_typed_project_id_verifier();
        let registry_path = root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH);
        let marker_path = root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH);

        fs::write(
            &registry_path,
            r#"{
              "schema_version": "m1.project-index.registry.v2",
              "registry_revision": 0,
              "projects": []
            }"#,
        )
        .expect("empty registry");
        assert_eq!(
            verifier
                .verify_typed_project_id(&registered.project_id)
                .unwrap_err()
                .code,
            "m1_project_id_unknown"
        );

        fs::write(&registry_path, "{not-json").expect("corrupt registry");
        assert_eq!(
            verifier
                .verify_typed_project_id(&registered.project_id)
                .unwrap_err()
                .code,
            "m1_project_index_registry_malformed"
        );

        fs::remove_file(&registry_path).expect("delete registry");
        assert!(marker_path.is_file());
        assert_eq!(
            verifier
                .verify_typed_project_id(&registered.project_id)
                .unwrap_err()
                .code,
            "m1_project_index_registry_missing"
        );

        fs::remove_file(&marker_path).expect("delete marker");
        fs::remove_dir_all(root.join("m1")).expect("delete m1 dir");
        assert_eq!(
            verifier
                .verify_typed_project_id(&registered.project_id)
                .unwrap_err()
                .code,
            M1_PROJECT_INDEX_UNAVAILABLE
        );
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    const M5R00_SOURCE_ALIAS: &str = "syn-m5r00-ordinary-alias";
    const M5R00_SOURCE_REF: &str = "synthetic-legacy-ref";

    fn ordinary_identity_source_json(alias: &str, mode: &str) -> String {
        format!(
            r#"{{
              "schema_version": "m1.ordinary-project-identity-source.v1",
              "source_id": "syn-m5r00-synthetic-source",
              "source_revision": 1,
              "projects": [{{
                "entry_id": "entry-1",
                "mode": "{mode}",
                "source_ref": "{M5R00_SOURCE_REF}",
                "exact_alias": "{alias}"
              }}]
            }}"#
        )
    }

    fn write_ordinary_identity_source(root: &Path, alias: &str) {
        fs::write(
            root.join(M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME),
            ordinary_identity_source_json(alias, "create"),
        )
        .expect("write ordinary identity source");
    }

    fn registry_revision_and_bytes(root: &Path) -> (u64, Vec<u8>) {
        let bytes = fs::read(root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH))
            .expect("read persisted registry");
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse persisted registry");
        let revision = value
            .get("registry_revision")
            .and_then(serde_json::Value::as_u64)
            .expect("registry revision");
        (revision, bytes)
    }

    fn write_synthetic_ordinary_product_seeds(root: &Path) -> (PathBuf, PathBuf) {
        let seed_dir = root
            .parent()
            .expect("ordinary named root parent")
            .join("synthetic-ordinary-product-seeds");
        fs::create_dir_all(&seed_dir).expect("create synthetic seed dir");
        let index_path = seed_dir.join("codex-index.json");
        let tasks_path = seed_dir.join("README.md");
        fs::write(&index_path, r#"{"projects":[]}"#).expect("write synthetic index seed");
        fs::write(&tasks_path, "# synthetic ordinary tasks\n").expect("write synthetic tasks seed");
        (index_path, tasks_path)
    }

    fn invoke_ordinary_tauri_constructor(root: &Path) -> Result<crate::AppState, String> {
        let (index_seed, tasks_seed) = write_synthetic_ordinary_product_seeds(root);
        crate::AppState::try_new_with_tauri_ordinary_product_seeds(
            root,
            &index_seed,
            &tasks_seed,
        )
    }

    fn ordinary_tauri_constructor_error(root: &Path) -> String {
        match invoke_ordinary_tauri_constructor(root) {
            Ok(_) => panic!("ordinary Tauri constructor must fail closed"),
            Err(error) => error,
        }
    }

    fn app_state_after_ordinary_tauri_constructor(root: &Path) -> crate::AppState {
        invoke_ordinary_tauri_constructor(root)
            .expect("ordinary Tauri constructor must return Ok")
    }

    #[test]
    fn m1_ordinary_identity_source_first_tauri_constructor_registers_opaque_id() {
        let root = ordinary_named_root();
        write_ordinary_identity_source(&root, M5R00_SOURCE_ALIAS);
        fs::write(
            root.join("codex-index.json"),
            r#"{"projects":[{"project_root":"/legacy/never-imported"}]}"#,
        )
        .expect("write unused legacy index");

        let state = app_state_after_ordinary_tauri_constructor(&root);
        let project_id = state
            .m1_project_index_authority()
            .expect("ordinary authority")
            .resolve_exact_alias(M5R00_SOURCE_ALIAS)
            .expect("registered alias");
        validate_canonical_project_id(project_id.as_str()).expect("opaque uuid");
        assert!(project_id.as_str().starts_with("project:"));
        assert_ne!(project_id.as_str(), M5R00_SOURCE_ALIAS);
        assert_ne!(project_id.as_str(), slug_like_id(M5R00_SOURCE_ALIAS));
        assert!(!project_id.as_str().contains(M5R00_SOURCE_REF));
        assert!(!project_id.as_str().contains("entry-1"));
        assert_eq!(
            state
                .m1_project_index_authority()
                .expect("ordinary authority")
                .resolve_exact_alias("/legacy/never-imported")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_ordinary_identity_source_replay_is_idempotent_and_survives_app_state_rebuild() {
        let root = ordinary_named_root();
        write_ordinary_identity_source(&root, M5R00_SOURCE_ALIAS);
        let first = app_state_after_ordinary_tauri_constructor(&root);
        let first_id = first
            .m1_project_index_authority()
            .expect("ordinary authority")
            .resolve_exact_alias(M5R00_SOURCE_ALIAS)
            .expect("first resolve")
            .as_str()
            .to_string();
        let (first_revision, first_bytes) = registry_revision_and_bytes(&root);
        drop(first);

        let _second = app_state_after_ordinary_tauri_constructor(&root);
        let (second_revision, second_bytes) = registry_revision_and_bytes(&root);
        assert_eq!(first_revision, second_revision);
        assert_eq!(first_bytes, second_bytes);

        let rebuilt = app_state_after_ordinary_tauri_constructor(&root);
        let rebuilt_id = rebuilt
            .m1_project_index_authority()
            .expect("rebuilt authority")
            .resolve_exact_alias(M5R00_SOURCE_ALIAS)
            .expect("rebuilt resolve");
        assert_eq!(rebuilt_id.as_str(), first_id);
        assert_eq!(
            rebuilt
                .m1_project_index_authority()
                .expect("rebuilt authority")
                .resolve_canonical_project_id(&first_id)
                .expect("canonical after rebuild")
                .as_str(),
            first_id
        );
        let (third_revision, third_bytes) = registry_revision_and_bytes(&root);
        assert_eq!(first_revision, third_revision);
        assert_eq!(first_bytes, third_bytes);
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_ordinary_identity_source_missing_and_corrupt_fail_closed_without_registry_write() {
        let root = ordinary_named_root();
        fs::write(
            root.join("codex-index.json"),
            r#"{"projects":[{"project_root":"/legacy/never-imported"}]}"#,
        )
        .expect("write unused legacy index");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            M1_ORDINARY_IDENTITY_SOURCE_MISSING
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        assert!(!root.join("m1").exists());
        assert!(!root.join(M1_ESTABLISHED_MARKER_RELATIVE_PATH).exists());

        fs::write(root.join(M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME), "{not-json")
            .expect("write corrupt source");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            M1_ORDINARY_IDENTITY_SOURCE_MALFORMED
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());

        fs::write(
            root.join(M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME),
            ordinary_identity_source_json(M5R00_SOURCE_ALIAS, "create").replace(
                M1_ORDINARY_IDENTITY_SOURCE_SCHEMA_VERSION,
                "m1.ordinary-project-identity-source.v0",
            ),
        )
        .expect("write unsupported source");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            M1_ORDINARY_IDENTITY_SOURCE_UNSUPPORTED
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());

        fs::write(
            root.join(M1_ORDINARY_IDENTITY_SOURCE_FILE_NAME),
            ordinary_identity_source_json(M5R00_SOURCE_ALIAS, "import_legacy"),
        )
        .expect("write invalid mode");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            M1_ORDINARY_IDENTITY_SOURCE_MALFORMED
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_ordinary_identity_source_registry_missing_and_corrupt_fail_closed() {
        let root = ordinary_named_root();
        write_ordinary_identity_source(&root, M5R00_SOURCE_ALIAS);
        fs::create_dir_all(root.join("m1")).expect("create established m1 dir");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            "m1_project_index_registry_missing"
        );
        assert!(!root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH).exists());

        fs::write(root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH), "{not-json")
            .expect("write corrupt registry");
        assert_eq!(
            ordinary_tauri_constructor_error(&root),
            "m1_project_index_registry_malformed"
        );
        assert_eq!(
            fs::read_to_string(root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH))
                .expect("retain corrupt registry"),
            "{not-json"
        );
        let parent = root.parent().expect("parent").to_path_buf();
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn m1_ordinary_identity_source_replay_has_tauri_caller_and_skips_legacy_index() {
        let lib = include_str!("lib.rs");
        let tauri = lib
            .find("fn try_new_with_tauri_app_data_root(app_data_root: &Path)")
            .expect("ordinary Tauri constructor");
        let helper = lib
            .find("fn try_new_with_tauri_ordinary_product_seeds(")
            .expect("shared ordinary Tauri helper");
        let replay = lib
            .find("replay_ordinary_identity_source")
            .expect("identity source replay caller");
        let ordinary = lib
            .find("fn try_new_with_ordinary_product_ports")
            .expect("shared ordinary ports");
        assert!(tauri < helper);
        assert!(helper < replay);
        assert!(replay < ordinary);
        assert_eq!(lib.matches("replay_ordinary_identity_source").count(), 1);
        let wrapper = &lib[tauri..helper];
        assert!(wrapper.contains("try_new_with_tauri_ordinary_product_seeds"));
        assert!(wrapper.contains("../../index-kernel/codex-index.json"));
        assert!(wrapper.contains("../../../tasks/README.md"));
        assert!(!wrapper.contains("#[cfg(test)]"));
        let helper_src = &lib[helper..ordinary];
        assert!(!helper_src.contains("#[cfg(test)]"));
        assert!(helper_src.contains("replay_ordinary_identity_source"));
        assert!(helper_src.contains("try_new_with_ordinary_product_ports"));

        let production = include_str!("m1_project_index.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production owner");
        assert!(production.contains("fn replay_ordinary_identity_source"));
        assert!(production.contains("deny_unknown_fields"));
        assert!(!production.contains("codex-index.json"));
        assert!(!production.contains("fn project_id("));

        let tests = include_str!("m1_project_index.rs");
        let seed_helper = tests
            .find("fn write_synthetic_ordinary_product_seeds")
            .expect("synthetic seed helper");
        let invoke = tests
            .find("fn invoke_ordinary_tauri_constructor")
            .expect("m5r00 constructor helper");
        let invoke_end = tests[invoke..]
            .find("fn ordinary_tauri_constructor_error")
            .map(|offset| invoke + offset)
            .expect("constructor helper bound");
        let seed_src = &tests[seed_helper..invoke];
        let invoke_src = &tests[invoke..invoke_end];
        assert!(seed_src.contains(r#"{"projects":[]}"#));
        assert!(seed_src.contains("# synthetic ordinary tasks\\n"));
        assert!(!seed_src.contains("../../index-kernel/codex-index.json"));
        assert!(!seed_src.contains("fs::read"));
        assert!(invoke_src.contains("try_new_with_tauri_ordinary_product_seeds"));
        assert!(!invoke_src.contains("../../index-kernel/codex-index.json"));
        assert!(!invoke_src.contains("try_new_with_tauri_app_data_root("));
    }
}
