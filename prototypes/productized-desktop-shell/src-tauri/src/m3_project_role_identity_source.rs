//! Server-only M3 ProjectRoleIdentitySource for M3O03.
//!
//! Ordinary product AppState may persist one unique actor/role/scope/object/
//! channel/permission snapshot per exact typed project and M3 project role.
//! The source does not accept path/root/alias/locator/cwd/M5 material, does
//! not mint an M5 ExecutionGrant, and does not expose a generic resolver.

#![allow(dead_code)]

use crate::m1_project_index::{M1ProjectId, M1_ORDINARY_APP_DATA_DIR_NAME};
use crate::m3_role_session::{
    owner_fingerprint_for_components, RoleSessionId, ServerResolvedBinding, Sha256Digest,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub(crate) const M3_PROJECT_ROLE_IDENTITY_SOURCE_PORT_VERSION: &str =
    "m3.project-role-identity-source.port.v1";
pub(crate) const M3_PROJECT_ROLE_IDENTITY_SOURCE_SCHEMA_VERSION: &str =
    "m3.project-role-identity-source.store.v1";
pub(crate) const M3_ORDINARY_IDENTITY_SOURCE_RELATIVE_PATH: &str =
    "m3/project-role-identity-source-v1.json";
pub(crate) const M3_IDENTITY_SOURCE_ESTABLISHED_MARKER_RELATIVE_PATH: &str =
    ".m3-project-role-identity-source.established";
const M3_IDENTITY_SOURCE_ESTABLISHED_MARKER_VALUE: &[u8] =
    b"m3.project-role-identity-source.established.v1\n";
pub(crate) const M3_IDENTITY_SOURCE_UNAVAILABLE: &str = "m3_identity_source_unavailable";
pub(crate) const M3_IDENTITY_SOURCE_MISSING: &str = "m3_project_role_identity_source_missing";
pub(crate) const M3_IDENTITY_SOURCE_CORRUPT: &str = "m3_project_role_identity_source_corrupt";
pub(crate) const M3_IDENTITY_SOURCE_DUPLICATE: &str = "m3_project_role_identity_source_duplicate";
pub(crate) const M3_IDENTITY_SOURCE_TAMPERED: &str = "m3_project_role_identity_source_tampered";
pub(crate) const M3_IDENTITY_SOURCE_VERSION_MISMATCH: &str =
    "m3_project_role_identity_source_version_mismatch";
pub(crate) const M3_IDENTITY_SOURCE_ROLE_PROJECT_MISMATCH: &str =
    "m3_project_role_identity_source_role_project_mismatch";
pub(crate) const M3_IDENTITY_SOURCE_INPUT_MISMATCH: &str =
    "m3_project_role_identity_source_input_mismatch";
pub(crate) const M3_IDENTITY_SOURCE_NOT_READABLE: &str =
    "m3_project_role_identity_source_not_readable";
pub(crate) const M3_IDENTITY_SOURCE_FORBIDDEN_MATERIAL: &str =
    "m3_project_role_identity_source_forbidden_material";

const M3_IDENTITY_SOURCE_LOCK_RELATIVE_PATH: &str = ".m3-project-role-identity-source-v1.lock";
const M3_IDENTITY_SOURCE_LOCK_RETRY_LIMIT: usize = 256;
const M3_IDENTITY_SOURCE_LOCK_RETRY_DELAY: Duration = Duration::from_millis(5);
const M3_IDENTITY_SOURCE_DOMAIN: &str = "syn.m3.project-role-identity-source.v1";
const M3_PERMISSION_PROFILE_ID: &str = "m3.project-role.permission.deny-default.v1";
const M3_ZERO_EXECUTION_CHANNEL_MATERIAL: &str =
    "syn.m3.project-role-identity-source.v1|channel|none";
const M3_DENIED_CAPABILITIES: &[&str] = &["execution", "provider", "runner", "grant"];
const M3_PERMISSION_CONSTRAINTS: &[&str] = &["zero-execution-authority", "not-an-execution-grant"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum M3ProjectRole {
    ProjectSupervisor,
    Worker,
    IndependentReviewer,
}

impl M3ProjectRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ProjectSupervisor => "ProjectSupervisor",
            Self::Worker => "Worker",
            Self::IndependentReviewer => "IndependentReviewer",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleIdentitySourceError {
    pub(crate) code: String,
}

impl M3ProjectRoleIdentitySourceError {
    pub(crate) fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    pub(crate) fn unavailable() -> Self {
        Self::new(M3_IDENTITY_SOURCE_UNAVAILABLE)
    }
}

impl std::fmt::Display for M3ProjectRoleIdentitySourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M3ProjectRoleIdentitySourceError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum IdentityRecordState {
    Prepared,
    Readable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DenyDefaultPermissionRecord {
    profile_id: String,
    allow_capabilities: Vec<String>,
    deny_capabilities: Vec<String>,
    constraints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredIdentityRecord {
    state: IdentityRecordState,
    project_id: String,
    role: M3ProjectRole,
    actor_id: String,
    role_ref: String,
    scope_ref: String,
    current_object_ref: String,
    execution_channel: String,
    permission_snapshot_ref: String,
    owner_fingerprint: String,
    role_session_id: String,
    permission: DenyDefaultPermissionRecord,
    provision_input_fingerprint: String,
    integrity_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentitySourceStoreDocument {
    schema_version: String,
    store_revision: u64,
    identities: Vec<StoredIdentityRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M3ProjectRoleIdentityBundle {
    pub(crate) project_id: String,
    pub(crate) role: M3ProjectRole,
    pub(crate) actor_id: String,
    pub(crate) role_ref: String,
    pub(crate) scope_ref: String,
    pub(crate) current_object_ref: String,
    pub(crate) execution_channel: String,
    pub(crate) permission_snapshot_ref: String,
    pub(crate) owner_fingerprint: String,
    pub(crate) role_session_id: String,
    pub(crate) readable: bool,
}

impl M3ProjectRoleIdentityBundle {
    pub(crate) fn server_binding(
        &self,
    ) -> Result<ServerResolvedBinding, M3ProjectRoleIdentitySourceError> {
        ServerResolvedBinding::from_server_canonical(
            self.actor_id.clone(),
            self.role_ref.clone(),
            self.scope_ref.clone(),
            self.current_object_ref.clone(),
            self.execution_channel.clone(),
            self.permission_snapshot_ref.clone(),
        )
        .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_TAMPERED))
    }

    pub(crate) fn bound_role_session_id(
        &self,
    ) -> Result<RoleSessionId, M3ProjectRoleIdentitySourceError> {
        RoleSessionId::try_from_canonical(self.role_session_id.clone())
            .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_TAMPERED))
    }
}

#[derive(Clone, Debug)]
struct M3ProjectRoleIdentitySourceStore {
    canonical_app_data_root: PathBuf,
    store_path: PathBuf,
    lock_path: PathBuf,
    established_marker_path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct M3ProjectRoleIdentitySourceHandle {
    store: M3ProjectRoleIdentitySourceStore,
}

struct ExclusiveSourceLock {
    path: PathBuf,
}

impl ExclusiveSourceLock {
    fn acquire(path: &Path) -> Result<Self, M3ProjectRoleIdentitySourceError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_lock_dir_create_failed",
                )
            })?;
        }
        for _ in 0..M3_IDENTITY_SOURCE_LOCK_RETRY_LIMIT {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_file) => {
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    thread::sleep(M3_IDENTITY_SOURCE_LOCK_RETRY_DELAY);
                }
                Err(_) => {
                    return Err(M3ProjectRoleIdentitySourceError::new(
                        "m3_project_role_identity_source_lock_failed",
                    ));
                }
            }
        }
        Err(M3ProjectRoleIdentitySourceError::new(
            "m3_project_role_identity_source_lock_timeout",
        ))
    }
}

impl Drop for ExclusiveSourceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl M3ProjectRoleIdentitySourceHandle {
    pub(crate) fn install_ordinary_product(
        app_data_root: &Path,
    ) -> Result<Self, M3ProjectRoleIdentitySourceError> {
        let root = admit_ordinary_app_data_root(app_data_root)?;
        Ok(Self {
            store: M3ProjectRoleIdentitySourceStore::from_root(root),
        })
    }

    pub(crate) fn ordinary_app_data_root(&self) -> &Path {
        &self.store.canonical_app_data_root
    }

    pub(crate) fn prepare_or_continue_provision(
        &self,
        project_id: &M1ProjectId,
        role: M3ProjectRole,
    ) -> Result<M3ProjectRoleIdentityBundle, M3ProjectRoleIdentitySourceError> {
        let expected = derive_identity_record(project_id.as_str(), role)?;
        let _lock = ExclusiveSourceLock::acquire(&self.store.lock_path)?;
        let mut document = self.store.load_or_empty()?;
        match locate_record(&document, project_id.as_str(), role)? {
            Some(existing) => {
                verify_stored_record(existing, project_id.as_str(), role)?;
                if existing.provision_input_fingerprint != expected.provision_input_fingerprint
                    || !records_have_same_identity(existing, &expected)
                {
                    return Err(M3ProjectRoleIdentitySourceError::new(
                        M3_IDENTITY_SOURCE_INPUT_MISMATCH,
                    ));
                }
                Ok(bundle_from_record(existing))
            }
            None => {
                document.identities.push(expected.clone());
                document.store_revision = document.store_revision.saturating_add(1);
                self.store.persist(&document)?;
                Ok(bundle_from_record(&expected))
            }
        }
    }

    pub(crate) fn load_readable(
        &self,
        project_id: &M1ProjectId,
        role: M3ProjectRole,
    ) -> Result<M3ProjectRoleIdentityBundle, M3ProjectRoleIdentitySourceError> {
        let _lock = ExclusiveSourceLock::acquire(&self.store.lock_path)?;
        let document = self.store.load_required()?;
        let record = locate_record(&document, project_id.as_str(), role)?
            .ok_or_else(|| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_MISSING))?;
        verify_stored_record(record, project_id.as_str(), role)?;
        if record.state != IdentityRecordState::Readable {
            return Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_NOT_READABLE,
            ));
        }
        Ok(bundle_from_record(record))
    }

    pub(crate) fn mark_readable_if_exact_match(
        &self,
        bundle: &M3ProjectRoleIdentityBundle,
    ) -> Result<M3ProjectRoleIdentityBundle, M3ProjectRoleIdentitySourceError> {
        let _lock = ExclusiveSourceLock::acquire(&self.store.lock_path)?;
        let mut document = self.store.load_required()?;
        let index = locate_record_index(&document, &bundle.project_id, bundle.role)?;
        let existing = document
            .identities
            .get(index)
            .ok_or_else(|| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_MISSING))?;
        verify_stored_record(existing, &bundle.project_id, bundle.role)?;
        if !bundle_matches_record(bundle, existing) {
            return Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_INPUT_MISMATCH,
            ));
        }
        if existing.state == IdentityRecordState::Readable {
            return Ok(bundle_from_record(existing));
        }
        let mut updated = existing.clone();
        updated.state = IdentityRecordState::Readable;
        updated.integrity_hash = integrity_hash_for(&updated)?;
        document.identities[index] = updated.clone();
        document.store_revision = document.store_revision.saturating_add(1);
        self.store.persist(&document)?;
        Ok(bundle_from_record(&updated))
    }
}

impl M3ProjectRoleIdentitySourceStore {
    fn from_root(canonical_app_data_root: PathBuf) -> Self {
        Self {
            store_path: canonical_app_data_root.join(M3_ORDINARY_IDENTITY_SOURCE_RELATIVE_PATH),
            lock_path: canonical_app_data_root.join(M3_IDENTITY_SOURCE_LOCK_RELATIVE_PATH),
            established_marker_path: canonical_app_data_root
                .join(M3_IDENTITY_SOURCE_ESTABLISHED_MARKER_RELATIVE_PATH),
            canonical_app_data_root,
        }
    }

    fn established_marker_is_present(&self) -> Result<bool, M3ProjectRoleIdentitySourceError> {
        match fs::symlink_metadata(&self.established_marker_path) {
            Ok(metadata) if metadata.file_type().is_file() => Ok(true),
            Ok(_) => Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_CORRUPT,
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(_) => Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_CORRUPT,
            )),
        }
    }

    fn persist_established_marker(&self) -> Result<(), M3ProjectRoleIdentitySourceError> {
        if self.established_marker_is_present()? {
            return Ok(());
        }
        let parent = self.established_marker_path.parent().ok_or_else(|| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_parent_required")
        })?;
        fs::create_dir_all(parent).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new(
                "m3_project_role_identity_source_dir_create_failed",
            )
        })?;
        let temp_path = parent.join(format!(
            ".m3-project-role-identity-source.established.{}.tmp",
            Uuid::new_v4().simple()
        ));
        {
            let mut file = File::create(&temp_path).map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_tmp_create_failed",
                )
            })?;
            file.write_all(M3_IDENTITY_SOURCE_ESTABLISHED_MARKER_VALUE)
                .map_err(|_| {
                    M3ProjectRoleIdentitySourceError::new(
                        "m3_project_role_identity_source_tmp_write_failed",
                    )
                })?;
            file.sync_all().map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_tmp_sync_failed",
                )
            })?;
        }
        fs::rename(&temp_path, &self.established_marker_path).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_replace_failed")
        })?;
        let dir = File::open(parent).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_dir_open_failed")
        })?;
        dir.sync_all().map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_dir_sync_failed")
        })?;
        Ok(())
    }

    fn load_or_empty(
        &self,
    ) -> Result<IdentitySourceStoreDocument, M3ProjectRoleIdentitySourceError> {
        match fs::symlink_metadata(&self.store_path) {
            Ok(metadata) if metadata.file_type().is_file() => self.load_required(),
            Ok(_) => Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_CORRUPT,
            )),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if self.established_marker_is_present()? {
                    return Err(M3ProjectRoleIdentitySourceError::new(
                        M3_IDENTITY_SOURCE_MISSING,
                    ));
                }
                Ok(IdentitySourceStoreDocument {
                    schema_version: M3_PROJECT_ROLE_IDENTITY_SOURCE_SCHEMA_VERSION.to_string(),
                    store_revision: 0,
                    identities: Vec::new(),
                })
            }
            Err(_) => Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_CORRUPT,
            )),
        }
    }

    fn load_required(
        &self,
    ) -> Result<IdentitySourceStoreDocument, M3ProjectRoleIdentitySourceError> {
        match fs::symlink_metadata(&self.store_path) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            Ok(_) => {
                return Err(M3ProjectRoleIdentitySourceError::new(
                    M3_IDENTITY_SOURCE_CORRUPT,
                ));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(M3ProjectRoleIdentitySourceError::new(
                    M3_IDENTITY_SOURCE_MISSING,
                ));
            }
            Err(_) => {
                return Err(M3ProjectRoleIdentitySourceError::new(
                    M3_IDENTITY_SOURCE_CORRUPT,
                ));
            }
        }
        let bytes = fs::read(&self.store_path)
            .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_CORRUPT))?;
        let document: IdentitySourceStoreDocument = serde_json::from_slice(&bytes)
            .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_CORRUPT))?;
        if document.schema_version != M3_PROJECT_ROLE_IDENTITY_SOURCE_SCHEMA_VERSION {
            return Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_VERSION_MISMATCH,
            ));
        }
        validate_document(&document)?;
        Ok(document)
    }

    fn persist(
        &self,
        document: &IdentitySourceStoreDocument,
    ) -> Result<(), M3ProjectRoleIdentitySourceError> {
        validate_document(document)?;
        self.persist_established_marker()?;
        let parent = self.store_path.parent().ok_or_else(|| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_parent_required")
        })?;
        fs::create_dir_all(parent).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new(
                "m3_project_role_identity_source_dir_create_failed",
            )
        })?;
        let text = serde_json::to_string_pretty(document).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new(
                "m3_project_role_identity_source_serialize_failed",
            )
        })?;
        reject_forbidden_material(&text)?;
        let temp_path = parent.join(format!(
            ".project-role-identity-source-v1.{}.tmp",
            Uuid::new_v4().simple()
        ));
        {
            let mut file = File::create(&temp_path).map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_tmp_create_failed",
                )
            })?;
            file.write_all(text.as_bytes()).map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_tmp_write_failed",
                )
            })?;
            file.sync_all().map_err(|_| {
                M3ProjectRoleIdentitySourceError::new(
                    "m3_project_role_identity_source_tmp_sync_failed",
                )
            })?;
        }
        fs::rename(&temp_path, &self.store_path).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_replace_failed")
        })?;
        let dir = File::open(parent).map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_dir_open_failed")
        })?;
        dir.sync_all().map_err(|_| {
            M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_dir_sync_failed")
        })?;
        Ok(())
    }
}

fn admit_ordinary_app_data_root(root: &Path) -> Result<PathBuf, M3ProjectRoleIdentitySourceError> {
    if !root.is_absolute()
        || root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(M3ProjectRoleIdentitySourceError::new(
            "m3_project_role_identity_source_clean_absolute_root_required",
        ));
    }
    let metadata = fs::symlink_metadata(root).map_err(|_| {
        M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_root_unavailable")
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(M3ProjectRoleIdentitySourceError::new(
            "m3_project_role_identity_source_regular_root_required",
        ));
    }
    let canonical = fs::canonicalize(root).map_err(|_| {
        M3ProjectRoleIdentitySourceError::new("m3_project_role_identity_source_root_unavailable")
    })?;
    if canonical != root {
        return Err(M3ProjectRoleIdentitySourceError::new(
            "m3_project_role_identity_source_root_identity_changed",
        ));
    }
    if canonical.file_name().and_then(|name| name.to_str()) != Some(M1_ORDINARY_APP_DATA_DIR_NAME) {
        return Err(M3ProjectRoleIdentitySourceError::new(
            "m3_project_role_identity_source_root_identity_mismatch",
        ));
    }
    Ok(canonical)
}

fn derive_identity_record(
    project_id: &str,
    role: M3ProjectRole,
) -> Result<StoredIdentityRecord, M3ProjectRoleIdentitySourceError> {
    reject_forbidden_material(project_id)?;
    if !project_id.starts_with("project:") || project_id.contains('/') || project_id.contains('\\')
    {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_FORBIDDEN_MATERIAL,
        ));
    }
    let actor_id = sealed_ref(
        "actor",
        &format!(
            "{M3_IDENTITY_SOURCE_DOMAIN}|actor|{}|{}",
            project_id,
            role.as_str()
        ),
    )?;
    let role_ref = sealed_ref(
        "role",
        &format!("{M3_IDENTITY_SOURCE_DOMAIN}|role|{}", role.as_str()),
    )?;
    let scope_ref = sealed_ref(
        "scope",
        &format!("{M3_IDENTITY_SOURCE_DOMAIN}|scope|{project_id}"),
    )?;
    let current_object_ref = sealed_ref(
        "object",
        &format!("{M3_IDENTITY_SOURCE_DOMAIN}|object|{project_id}"),
    )?;
    let execution_channel = sealed_ref("channel", M3_ZERO_EXECUTION_CHANNEL_MATERIAL)?;
    let permission = deny_default_permission();
    let permission_snapshot_ref = sealed_ref(
        "permission",
        &format!(
            "{M3_IDENTITY_SOURCE_DOMAIN}|permission|{}|{}|{}|allow:|deny:{}|constraints:{}",
            project_id,
            role.as_str(),
            permission.profile_id,
            permission.deny_capabilities.join(","),
            permission.constraints.join(",")
        ),
    )?;
    let owner_fingerprint = owner_fingerprint_for_components(
        &actor_id,
        &role_ref,
        &scope_ref,
        &current_object_ref,
        &execution_channel,
    )
    .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_TAMPERED))?;
    let role_session_id = sealed_ref(
        "session",
        &format!(
            "{M3_IDENTITY_SOURCE_DOMAIN}|session|{}|{}",
            project_id,
            role.as_str()
        ),
    )?;
    let mut record = StoredIdentityRecord {
        state: IdentityRecordState::Prepared,
        project_id: project_id.to_string(),
        role,
        actor_id,
        role_ref,
        scope_ref,
        current_object_ref,
        execution_channel,
        permission_snapshot_ref,
        owner_fingerprint: owner_fingerprint.as_str().to_string(),
        role_session_id,
        permission,
        provision_input_fingerprint: String::new(),
        integrity_hash: String::new(),
    };
    record.provision_input_fingerprint = provision_input_fingerprint_for(&record)?;
    record.integrity_hash = integrity_hash_for(&record)?;
    Ok(record)
}

fn deny_default_permission() -> DenyDefaultPermissionRecord {
    DenyDefaultPermissionRecord {
        profile_id: M3_PERMISSION_PROFILE_ID.to_string(),
        allow_capabilities: Vec::new(),
        deny_capabilities: M3_DENIED_CAPABILITIES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        constraints: M3_PERMISSION_CONSTRAINTS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn sealed_ref(namespace: &str, material: &str) -> Result<String, M3ProjectRoleIdentitySourceError> {
    reject_forbidden_material(material)?;
    let digest = Sha256Digest::of_bytes(material.as_bytes());
    Ok(format!("{namespace}:sha256:{}", digest.as_str()))
}

fn provision_input_fingerprint_for(
    record: &StoredIdentityRecord,
) -> Result<String, M3ProjectRoleIdentitySourceError> {
    hash_components(&[
        M3_IDENTITY_SOURCE_DOMAIN,
        "provision-input",
        record.project_id.as_str(),
        record.role.as_str(),
        record.actor_id.as_str(),
        record.role_ref.as_str(),
        record.scope_ref.as_str(),
        record.current_object_ref.as_str(),
        record.execution_channel.as_str(),
        record.permission_snapshot_ref.as_str(),
        record.role_session_id.as_str(),
    ])
}

fn integrity_hash_for(
    record: &StoredIdentityRecord,
) -> Result<String, M3ProjectRoleIdentitySourceError> {
    hash_components(&[
        M3_IDENTITY_SOURCE_DOMAIN,
        "integrity",
        match record.state {
            IdentityRecordState::Prepared => "PREPARED",
            IdentityRecordState::Readable => "READABLE",
        },
        record.project_id.as_str(),
        record.role.as_str(),
        record.actor_id.as_str(),
        record.role_ref.as_str(),
        record.scope_ref.as_str(),
        record.current_object_ref.as_str(),
        record.execution_channel.as_str(),
        record.permission_snapshot_ref.as_str(),
        record.owner_fingerprint.as_str(),
        record.role_session_id.as_str(),
        record.permission.profile_id.as_str(),
        &record.permission.allow_capabilities.join(","),
        &record.permission.deny_capabilities.join(","),
        &record.permission.constraints.join(","),
        record.provision_input_fingerprint.as_str(),
    ])
}

fn hash_components(fields: &[&str]) -> Result<String, M3ProjectRoleIdentitySourceError> {
    let mut hasher = Sha256::new();
    for field in fields {
        let byte_len = u32::try_from(field.as_bytes().len())
            .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_TAMPERED))?;
        hasher.update(byte_len.to_be_bytes());
        hasher.update(field.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn locate_record<'a>(
    document: &'a IdentitySourceStoreDocument,
    project_id: &str,
    role: M3ProjectRole,
) -> Result<Option<&'a StoredIdentityRecord>, M3ProjectRoleIdentitySourceError> {
    let matches = document
        .identities
        .iter()
        .filter(|record| record.project_id == project_id && record.role == role)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [record] => Ok(Some(*record)),
        _ => Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_DUPLICATE,
        )),
    }
}

fn locate_record_index(
    document: &IdentitySourceStoreDocument,
    project_id: &str,
    role: M3ProjectRole,
) -> Result<usize, M3ProjectRoleIdentitySourceError> {
    let matches = document
        .identities
        .iter()
        .enumerate()
        .filter(|(_, record)| record.project_id == project_id && record.role == role)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_MISSING,
        )),
        _ => Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_DUPLICATE,
        )),
    }
}

fn validate_document(
    document: &IdentitySourceStoreDocument,
) -> Result<(), M3ProjectRoleIdentitySourceError> {
    if document.schema_version != M3_PROJECT_ROLE_IDENTITY_SOURCE_SCHEMA_VERSION {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_VERSION_MISMATCH,
        ));
    }
    let mut seen = Vec::new();
    for record in &document.identities {
        let key = (record.project_id.as_str(), record.role);
        if seen.contains(&key) {
            return Err(M3ProjectRoleIdentitySourceError::new(
                M3_IDENTITY_SOURCE_DUPLICATE,
            ));
        }
        seen.push(key);
        verify_record_shape(record)?;
    }
    Ok(())
}

fn verify_record_shape(
    record: &StoredIdentityRecord,
) -> Result<(), M3ProjectRoleIdentitySourceError> {
    for value in [
        record.project_id.as_str(),
        record.actor_id.as_str(),
        record.role_ref.as_str(),
        record.scope_ref.as_str(),
        record.current_object_ref.as_str(),
        record.execution_channel.as_str(),
        record.permission_snapshot_ref.as_str(),
        record.owner_fingerprint.as_str(),
        record.role_session_id.as_str(),
        record.permission.profile_id.as_str(),
        record.provision_input_fingerprint.as_str(),
        record.integrity_hash.as_str(),
    ] {
        reject_forbidden_material(value)?;
    }
    if !record.permission.allow_capabilities.is_empty() {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_TAMPERED,
        ));
    }
    if record.permission.profile_id != M3_PERMISSION_PROFILE_ID
        || record.permission.deny_capabilities != M3_DENIED_CAPABILITIES
        || record.permission.constraints != M3_PERMISSION_CONSTRAINTS
    {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_TAMPERED,
        ));
    }
    Ok(())
}

fn verify_stored_record(
    record: &StoredIdentityRecord,
    project_id: &str,
    role: M3ProjectRole,
) -> Result<(), M3ProjectRoleIdentitySourceError> {
    verify_record_shape(record)?;
    if record.integrity_hash != integrity_hash_for(record)? {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_TAMPERED,
        ));
    }
    if record.project_id != project_id || record.role != role {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_ROLE_PROJECT_MISMATCH,
        ));
    }
    let expected = derive_identity_record(project_id, role)?;
    if record.scope_ref != expected.scope_ref
        || record.current_object_ref != expected.current_object_ref
        || record.role_ref != expected.role_ref
    {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_ROLE_PROJECT_MISMATCH,
        ));
    }
    let expected_fingerprint = owner_fingerprint_for_components(
        &record.actor_id,
        &record.role_ref,
        &record.scope_ref,
        &record.current_object_ref,
        &record.execution_channel,
    )
    .map_err(|_| M3ProjectRoleIdentitySourceError::new(M3_IDENTITY_SOURCE_TAMPERED))?;
    if record.owner_fingerprint != expected_fingerprint.as_str() {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_TAMPERED,
        ));
    }
    if !records_have_same_identity(record, &expected) {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_TAMPERED,
        ));
    }
    Ok(())
}

fn records_have_same_identity(left: &StoredIdentityRecord, right: &StoredIdentityRecord) -> bool {
    left.project_id == right.project_id
        && left.role == right.role
        && left.actor_id == right.actor_id
        && left.role_ref == right.role_ref
        && left.scope_ref == right.scope_ref
        && left.current_object_ref == right.current_object_ref
        && left.execution_channel == right.execution_channel
        && left.permission_snapshot_ref == right.permission_snapshot_ref
        && left.owner_fingerprint == right.owner_fingerprint
        && left.role_session_id == right.role_session_id
        && left.permission == right.permission
        && left.provision_input_fingerprint == right.provision_input_fingerprint
}

fn bundle_from_record(record: &StoredIdentityRecord) -> M3ProjectRoleIdentityBundle {
    M3ProjectRoleIdentityBundle {
        project_id: record.project_id.clone(),
        role: record.role,
        actor_id: record.actor_id.clone(),
        role_ref: record.role_ref.clone(),
        scope_ref: record.scope_ref.clone(),
        current_object_ref: record.current_object_ref.clone(),
        execution_channel: record.execution_channel.clone(),
        permission_snapshot_ref: record.permission_snapshot_ref.clone(),
        owner_fingerprint: record.owner_fingerprint.clone(),
        role_session_id: record.role_session_id.clone(),
        readable: record.state == IdentityRecordState::Readable,
    }
}

fn bundle_matches_record(
    bundle: &M3ProjectRoleIdentityBundle,
    record: &StoredIdentityRecord,
) -> bool {
    bundle.project_id == record.project_id
        && bundle.role == record.role
        && bundle.actor_id == record.actor_id
        && bundle.role_ref == record.role_ref
        && bundle.scope_ref == record.scope_ref
        && bundle.current_object_ref == record.current_object_ref
        && bundle.execution_channel == record.execution_channel
        && bundle.permission_snapshot_ref == record.permission_snapshot_ref
        && bundle.owner_fingerprint == record.owner_fingerprint
        && bundle.role_session_id == record.role_session_id
}

fn reject_forbidden_material(value: &str) -> Result<(), M3ProjectRoleIdentitySourceError> {
    let normalized = value.to_ascii_lowercase();
    let forbidden = [
        "/",
        "\\",
        "m5:",
        "m5_",
        "cwd",
        "locator",
        "scratch-",
        "executiongrant",
        "execution_grant",
        "official_project_id",
        "resolve_identity",
        "resolve_m4_primary_secretary",
    ];
    if forbidden.iter().any(|marker| normalized.contains(marker)) {
        return Err(M3ProjectRoleIdentitySourceError::new(
            M3_IDENTITY_SOURCE_FORBIDDEN_MATERIAL,
        ));
    }
    Ok(())
}

#[cfg(test)]
impl M3ProjectRoleIdentitySourceHandle {
    pub(crate) fn store_path_for_test(&self) -> PathBuf {
        self.store.store_path.clone()
    }

    pub(crate) fn established_marker_path_for_test(&self) -> PathBuf {
        self.store.established_marker_path.clone()
    }

    fn write_document_for_test(
        &self,
        mutate: impl FnOnce(&mut IdentitySourceStoreDocument),
    ) -> Result<(), M3ProjectRoleIdentitySourceError> {
        let _lock = ExclusiveSourceLock::acquire(&self.store.lock_path)?;
        let mut document = self.store.load_or_empty()?;
        mutate(&mut document);
        let text = serde_json::to_string_pretty(&document).expect("serialize test document");
        if let Some(parent) = self.store.store_path.parent() {
            fs::create_dir_all(parent).expect("create test store dir");
        }
        fs::write(&self.store.store_path, text).expect("write test store");
        Ok(())
    }
}

#[cfg(test)]
mod m3_project_role_identity_source_tests {
    use super::*;
    use crate::m1_project_index::{
        M1ProjectIndexAuthorityHandle, M1RegisterExactAliasRequest, M1_ORDINARY_APP_DATA_DIR_NAME,
    };

    fn ordinary_named_root() -> PathBuf {
        let parent = std::env::temp_dir().join(format!(
            "m3o03-source-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        let root = parent.join(M1_ORDINARY_APP_DATA_DIR_NAME);
        fs::create_dir_all(&root).expect("create ordinary app-data root");
        fs::canonicalize(&root).expect("canonicalize ordinary app-data root")
    }

    fn register_typed_id(app_data_root: &Path, alias: &str) -> M1ProjectId {
        M1ProjectIndexAuthorityHandle::install_ordinary_product(app_data_root)
            .expect("install m1")
            .register_exact_alias(&M1RegisterExactAliasRequest {
                exact_alias: alias.to_string(),
            })
            .expect("register typed id")
            .project_id
    }

    fn assert_code<T: std::fmt::Debug>(
        result: Result<T, M3ProjectRoleIdentitySourceError>,
        code: &str,
    ) {
        match result {
            Ok(value) => panic!("expected {code}, got {value:?}"),
            Err(error) => assert_eq!(error.code, code),
        }
    }

    #[test]
    fn source_derives_unique_deny_default_snapshot_bound_to_canonical_project() {
        let root = ordinary_named_root();
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let project_a = register_typed_id(&root, "syn-m3o03-source-a");
        let project_b = register_typed_id(&root, "syn-m3o03-source-b");
        let supervisor = source
            .prepare_or_continue_provision(&project_a, M3ProjectRole::ProjectSupervisor)
            .expect("prepare supervisor");
        let worker = source
            .prepare_or_continue_provision(&project_a, M3ProjectRole::Worker)
            .expect("prepare worker");
        let reviewer = source
            .prepare_or_continue_provision(&project_a, M3ProjectRole::IndependentReviewer)
            .expect("prepare reviewer");
        let other = source
            .prepare_or_continue_provision(&project_b, M3ProjectRole::ProjectSupervisor)
            .expect("prepare other project");

        assert_ne!(supervisor.actor_id, worker.actor_id);
        assert_ne!(supervisor.actor_id, reviewer.actor_id);
        assert_ne!(worker.actor_id, reviewer.actor_id);
        assert_ne!(supervisor.role_ref, reviewer.role_ref);
        assert_ne!(supervisor.owner_fingerprint, reviewer.owner_fingerprint);
        assert_eq!(supervisor.scope_ref, worker.scope_ref);
        assert_eq!(supervisor.current_object_ref, worker.current_object_ref);
        assert_ne!(supervisor.scope_ref, other.scope_ref);
        assert_ne!(supervisor.current_object_ref, other.current_object_ref);
        assert_eq!(supervisor.execution_channel, worker.execution_channel);
        assert!(!supervisor.readable);
        assert_eq!(
            source
                .prepare_or_continue_provision(&project_a, M3ProjectRole::ProjectSupervisor)
                .expect("idempotent prepare")
                .actor_id,
            supervisor.actor_id
        );

        let store_text = fs::read_to_string(source.store_path_for_test()).expect("read store");
        assert!(!store_text.contains(root.to_string_lossy().as_ref()));
        assert!(!store_text.contains("ExecutionGrant"));
        assert!(!store_text.contains("m5:"));
        assert!(store_text.contains(M3_PERMISSION_PROFILE_ID));
        assert!(store_text.contains("\"allow_capabilities\": []"));
        assert_code(
            source.load_readable(&project_a, M3ProjectRole::ProjectSupervisor),
            M3_IDENTITY_SOURCE_NOT_READABLE,
        );
        let readable = source
            .mark_readable_if_exact_match(&supervisor)
            .expect("mark readable");
        assert!(readable.readable);
        assert_eq!(
            source
                .load_readable(&project_a, M3ProjectRole::ProjectSupervisor)
                .expect("load readable")
                .role_session_id,
            supervisor.role_session_id
        );
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn source_fail_closed_on_missing_corrupt_duplicate_tamper_and_version() {
        let root = ordinary_named_root();
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let project_id = register_typed_id(&root, "syn-m3o03-source-faults");
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_MISSING,
        );

        fs::create_dir_all(source.store_path_for_test().parent().expect("parent"))
            .expect("create store dir");
        fs::write(source.store_path_for_test(), "{not-json").expect("corrupt");
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_CORRUPT,
        );

        fs::write(
            source.store_path_for_test(),
            r#"{
              "schema_version": "m3.project-role-identity-source.store.v0",
              "store_revision": 1,
              "identities": []
            }"#,
        )
        .expect("version");
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_VERSION_MISMATCH,
        );
        fs::remove_file(source.store_path_for_test()).expect("remove version-mismatch store");

        let prepared = source
            .prepare_or_continue_provision(&project_id, M3ProjectRole::Worker)
            .expect("prepare");
        source
            .write_document_for_test(|document| {
                if let Some(record) = document.identities.first_mut() {
                    record.actor_id = "actor:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_string();
                }
            })
            .expect("tamper actor");
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_TAMPERED,
        );

        source
            .write_document_for_test(|document| {
                let extra = document
                    .identities
                    .first()
                    .cloned()
                    .expect("existing record");
                document.identities.push(extra);
            })
            .expect("duplicate");
        assert_code(
            source.prepare_or_continue_provision(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_DUPLICATE,
        );

        let _ = prepared.role;
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn deleted_established_source_json_fails_closed_and_does_not_rebuild() {
        let root = ordinary_named_root();
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let project_id = register_typed_id(&root, "syn-m3o03-source-deleted");
        let prepared = source
            .prepare_or_continue_provision(&project_id, M3ProjectRole::Worker)
            .expect("first prepare");
        assert!(source.established_marker_path_for_test().is_file());
        fs::remove_file(source.store_path_for_test()).expect("delete established source json");
        assert!(source.established_marker_path_for_test().is_file());
        assert_code(
            source.prepare_or_continue_provision(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::Worker),
            M3_IDENTITY_SOURCE_MISSING,
        );
        assert!(!source.store_path_for_test().exists());
        let _ = prepared.role;
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }

    #[test]
    fn prepared_mismatch_cannot_be_completed_or_marked_readable() {
        let root = ordinary_named_root();
        let source = M3ProjectRoleIdentitySourceHandle::install_ordinary_product(&root)
            .expect("install source");
        let project_id = register_typed_id(&root, "syn-m3o03-source-prepared");
        let prepared = source
            .prepare_or_continue_provision(&project_id, M3ProjectRole::ProjectSupervisor)
            .expect("prepare");
        let mut mismatched = prepared.clone();
        mismatched.actor_id =
            "actor:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                .to_string();
        assert_code(
            source.mark_readable_if_exact_match(&mismatched),
            M3_IDENTITY_SOURCE_INPUT_MISMATCH,
        );
        assert_code(
            source.load_readable(&project_id, M3ProjectRole::ProjectSupervisor),
            M3_IDENTITY_SOURCE_NOT_READABLE,
        );
        let _ = fs::remove_dir_all(root.parent().expect("parent"));
    }
}
