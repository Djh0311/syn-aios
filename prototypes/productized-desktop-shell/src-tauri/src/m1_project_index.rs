//! Server-only project_index owner for M1I01.
//!
//! Explicit isolated-project registration mints an opaque random
//! `project:<uuid>` and three distinct role identities. Roots, locators,
//! slugs, scratch tokens, caller booleans, and M5 helpers are alias or
//! resolver inputs only. Legacy index records are never imported.

#![allow(dead_code)]

use crate::m1_project_role_identity::{
    mint_prefixed_uuid, mint_project_role_identities, project_role_identity_snapshot,
    M1ProjectIdentityContext, M1ProjectRole, M1ProjectRoleIdentitySnapshot, M1StoredRoleIdentity,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub(crate) const M1_PROJECT_INDEX_PORT_VERSION: &str = "m1.project-index.authority.v1";
pub(crate) const M1_PROJECT_INDEX_SCHEMA_VERSION: &str = "m1.project-index.registry.v1";
pub(crate) const M1_ORDINARY_APP_DATA_DIR_NAME: &str = "local.codex.governance.workbench";
pub(crate) const M1_ORDINARY_REGISTRY_RELATIVE_PATH: &str = "m1/project-index-v1.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1ProjectIndexError {
    pub(crate) code: String,
}

impl M1ProjectIndexError {
    fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

impl std::fmt::Display for M1ProjectIndexError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for M1ProjectIndexError {}

impl From<String> for M1ProjectIndexError {
    fn from(code: String) -> Self {
        Self { code }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct M1ProjectId(String);

impl M1ProjectId {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct M1RegisterIsolatedProjectRequest {
    pub(crate) exact_alias: Option<String>,
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

#[derive(Clone, Debug)]
pub(crate) struct M1ProjectIndexAuthority {
    canonical_app_data_root: PathBuf,
    registry_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct M1ProjectIndexRegistry {
    schema_version: String,
    registry_revision: u64,
    projects: Vec<M1StoredProject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct M1StoredProject {
    project_id: String,
    exact_alias: Option<String>,
    resolver_revision: u64,
    project_revision: u64,
    scope_id: String,
    scope_revision: u64,
    current_object_id: String,
    binding_revision: u64,
    issued_at: String,
    roles: Vec<M1StoredRoleIdentity>,
}

impl Default for M1ProjectIndexRegistry {
    fn default() -> Self {
        Self {
            schema_version: M1_PROJECT_INDEX_SCHEMA_VERSION.to_string(),
            registry_revision: 0,
            projects: Vec::new(),
        }
    }
}

impl M1ProjectIndexAuthority {
    pub(crate) fn open_ordinary_product(app_data_root: &Path) -> Result<Self, M1ProjectIndexError> {
        let root =
            admit_existing_clean_root(app_data_root, "m1_ordinary_app_data_root_unavailable")?;
        if root.file_name().and_then(|name| name.to_str()) != Some(M1_ORDINARY_APP_DATA_DIR_NAME) {
            return Err(M1ProjectIndexError::new(
                "m1_ordinary_app_data_root_identity_mismatch",
            ));
        }
        Self::finish_open(root)
    }

    #[cfg(test)]
    pub(crate) fn open_isolated_fixture(root: &Path) -> Result<Self, M1ProjectIndexError> {
        let canonical_temp = fs::canonicalize(std::env::temp_dir())
            .map_err(|_| M1ProjectIndexError::new("m1_isolated_temp_root_unavailable"))?;
        let canonical_root =
            admit_existing_clean_root(root, "m1_isolated_fixture_root_unavailable")?;
        let admitted_name = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if !canonical_root.starts_with(&canonical_temp) || !admitted_name.starts_with("syn-m1i01-")
        {
            return Err(M1ProjectIndexError::new(
                "m1_isolated_fixture_root_not_admitted",
            ));
        }
        Self::finish_open(canonical_root)
    }

    fn finish_open(canonical_app_data_root: PathBuf) -> Result<Self, M1ProjectIndexError> {
        let registry_path = canonical_app_data_root.join(M1_ORDINARY_REGISTRY_RELATIVE_PATH);
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_create_failed"))?;
            let canonical_parent = fs::canonicalize(parent)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_dir_unavailable"))?;
            if canonical_parent != parent {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_dir_identity_changed",
                ));
            }
        }
        let authority = Self {
            canonical_app_data_root,
            registry_path,
        };
        if authority.registry_path.exists() {
            let _ = authority.load_registry()?;
        } else {
            authority.persist_registry(&M1ProjectIndexRegistry::default())?;
        }
        Ok(authority)
    }

    pub(crate) fn register_isolated_project(
        &self,
        request: M1RegisterIsolatedProjectRequest,
    ) -> Result<M1RegisteredProject, M1ProjectIndexError> {
        let exact_alias = match request.exact_alias {
            Some(alias) => Some(validate_alias_for_registration(&alias)?),
            None => None,
        };
        let mut registry = self.load_registry()?;
        if let Some(alias) = exact_alias.as_deref() {
            if registry
                .projects
                .iter()
                .any(|project| project.exact_alias.as_deref() == Some(alias))
            {
                return Err(M1ProjectIndexError::new("m1_alias_duplicate"));
            }
        }

        let project_id = mint_prefixed_uuid("project:");
        validate_canonical_project_id(&project_id)?;
        let issued_at = utc_rfc3339_now()?;
        let context = M1ProjectIdentityContext {
            project_id: project_id.clone(),
            scope_id: mint_prefixed_uuid("scope:"),
            scope_revision: 1,
            current_object_id: mint_prefixed_uuid("object:"),
            binding_revision: 1,
            issued_at: issued_at.clone(),
            registry_revision: registry.registry_revision.saturating_add(1),
            project_revision: 1,
            resolver_revision: 1,
        };
        let roles = mint_project_role_identities(&context)?;
        registry.registry_revision = context.registry_revision;
        registry.projects.push(M1StoredProject {
            project_id: project_id.clone(),
            exact_alias: exact_alias.clone(),
            resolver_revision: 1,
            project_revision: 1,
            scope_id: context.scope_id,
            scope_revision: context.scope_revision,
            current_object_id: context.current_object_id,
            binding_revision: context.binding_revision,
            issued_at,
            roles: roles.to_vec(),
        });
        self.persist_registry(&registry)?;
        Ok(M1RegisteredProject {
            project_id: M1ProjectId(project_id),
            exact_alias,
            resolver_revision: 1,
            registry_revision: registry.registry_revision,
        })
    }

    pub(crate) fn resolve_canonical_project_id(
        &self,
        claim: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        reject_non_canonical_project_id_claim(claim)?;
        validate_canonical_project_id(claim)?;
        let registry = self.load_registry()?;
        registry
            .projects
            .iter()
            .find(|project| project.project_id == claim)
            .map(|project| M1ProjectId(project.project_id.clone()))
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_unknown"))
    }

    pub(crate) fn resolve_exact_alias(
        &self,
        alias: &str,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        let alias = validate_alias_for_resolution(alias)?;
        let registry = self.load_registry()?;
        let matches = registry
            .projects
            .iter()
            .filter(|project| project.exact_alias.as_deref() == Some(alias.as_str()))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [project] => Ok(M1ProjectId(project.project_id.clone())),
            [] => Err(M1ProjectIndexError::new("m1_alias_unknown")),
            _ => Err(M1ProjectIndexError::new("m1_alias_duplicate")),
        }
    }

    pub(crate) fn resolve_project_root_ref(
        &self,
        project_root_ref: &M1ProjectRootRef,
    ) -> Result<M1ProjectId, M1ProjectIndexError> {
        let project_id = self.resolve_canonical_project_id(&project_root_ref.project_id)?;
        let alias = validate_alias_for_resolution(&project_root_ref.normalized_root_alias)?;
        let registry = self.load_registry()?;
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

    pub(crate) fn project_role_identity(
        &self,
        project_id: &str,
        role: M1ProjectRole,
    ) -> Result<M1ProjectRoleIdentitySnapshot, M1ProjectIndexError> {
        let project_id = self.resolve_canonical_project_id(project_id)?;
        let registry = self.load_registry()?;
        let stored = registry
            .projects
            .iter()
            .find(|project| project.project_id == project_id.as_str())
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_id_unknown"))?;
        let stored_role = stored
            .roles
            .iter()
            .find(|item| item.role == role)
            .ok_or_else(|| M1ProjectIndexError::new("m1_project_role_identity_unavailable"))?;
        let context = M1ProjectIdentityContext {
            project_id: stored.project_id.clone(),
            scope_id: stored.scope_id.clone(),
            scope_revision: stored.scope_revision,
            current_object_id: stored.current_object_id.clone(),
            binding_revision: stored.binding_revision,
            issued_at: stored.issued_at.clone(),
            registry_revision: registry.registry_revision,
            project_revision: stored.project_revision,
            resolver_revision: stored.resolver_revision,
        };
        Ok(project_role_identity_snapshot(&context, stored_role)?)
    }

    fn load_registry(&self) -> Result<M1ProjectIndexRegistry, M1ProjectIndexError> {
        if !self.registry_path.exists() {
            return Ok(M1ProjectIndexRegistry::default());
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
            let mut file = fs::File::create(&temp_path)
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_create_failed"))?;
            file.write_all(text.as_bytes())
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_write_failed"))?;
            file.sync_all()
                .map_err(|_| M1ProjectIndexError::new("m1_project_index_tmp_sync_failed"))?;
        }
        fs::rename(&temp_path, &self.registry_path)
            .map_err(|_| M1ProjectIndexError::new("m1_project_index_registry_replace_failed"))?;
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    }
}

fn validate_loaded_registry(registry: &M1ProjectIndexRegistry) -> Result<(), M1ProjectIndexError> {
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
        if let Some(alias) = project.exact_alias.as_deref() {
            validate_alias_for_registration(alias)?;
            if seen_aliases.iter().any(|item: &String| item == alias) {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_malformed",
                ));
            }
            seen_aliases.push(alias.to_string());
        }
        if project.roles.len() != 3 {
            return Err(M1ProjectIndexError::new(
                "m1_project_index_registry_malformed",
            ));
        }
        let expected = M1ProjectRole::all();
        for (index, role) in expected.into_iter().enumerate() {
            if project.roles[index].role != role {
                return Err(M1ProjectIndexError::new(
                    "m1_project_index_registry_malformed",
                ));
            }
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

fn validate_alias_for_registration(alias: &str) -> Result<String, M1ProjectIndexError> {
    validate_alias_shape(alias, "m1_alias_malformed")
}

fn validate_alias_for_resolution(alias: &str) -> Result<String, M1ProjectIndexError> {
    validate_alias_shape(alias, "m1_alias_malformed")
}

fn validate_alias_shape(alias: &str, malformed_code: &str) -> Result<String, M1ProjectIndexError> {
    if alias.is_empty()
        || alias.len() > 1024
        || alias
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || is_caller_boolean(alias)
    {
        return Err(M1ProjectIndexError::new(malformed_code));
    }
    if parse_prefixed_uuid("project:", alias).is_some() {
        return Err(M1ProjectIndexError::new(malformed_code));
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

fn utc_rfc3339_now() -> Result<String, M1ProjectIndexError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| M1ProjectIndexError::new("m1_project_index_clock_before_epoch"))?
        .as_secs();
    Ok(unix_secs_to_rfc3339(seconds))
}

fn unix_secs_to_rfc3339(seconds: u64) -> String {
    let days = seconds / 86_400;
    let day_seconds = seconds % 86_400;
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { y + 1 } else { y } as i32;
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isolated_authority() -> (M1ProjectIndexAuthority, PathBuf) {
        let root = std::env::temp_dir().join(format!("syn-m1i01-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create isolated root");
        let canonical = fs::canonicalize(&root).expect("canonicalize isolated root");
        let authority =
            M1ProjectIndexAuthority::open_isolated_fixture(&canonical).expect("open fixture");
        (authority, canonical)
    }

    fn path_derived_project_id(project_root: &str) -> String {
        format!("project:{:x}", project_root.len())
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
        let (authority, root) = isolated_authority();
        let alias = format!("{}/isolated-project", root.display());
        let registered = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some(alias.clone()),
            })
            .expect("register");
        assert!(registered.project_id.as_str().starts_with("project:"));
        validate_canonical_project_id(registered.project_id.as_str()).expect("uuid id");
        assert_ne!(
            registered.project_id.as_str(),
            path_derived_project_id(&alias)
        );
        assert_ne!(registered.project_id.as_str(), slug_like_id(&alias));
        assert!(!registered.project_id.as_str().contains('/'));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_restart_restores_same_ids_and_identities() {
        let (authority, root) = isolated_authority();
        let registered = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/tmp/syn-m1i01-restart-alias".to_string()),
            })
            .expect("register");
        let before = M1ProjectRole::all().map(|role| {
            authority
                .project_role_identity(registered.project_id.as_str(), role)
                .expect("identity before restart")
        });
        drop(authority);

        let reopened = M1ProjectIndexAuthority::open_isolated_fixture(&root).expect("reopen");
        let after_id = reopened
            .resolve_exact_alias("/tmp/syn-m1i01-restart-alias")
            .expect("alias after restart");
        assert_eq!(after_id.as_str(), registered.project_id.as_str());
        for (role, expected) in M1ProjectRole::all().into_iter().zip(before.iter()) {
            let actual = reopened
                .project_role_identity(after_id.as_str(), role)
                .expect("identity after restart");
            assert_eq!(actual, *expected);
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_exact_alias_resolves_only_when_pre_registered() {
        let (authority, root) = isolated_authority();
        let registered = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/one".to_string()),
            })
            .expect("register");
        assert_eq!(
            authority
                .resolve_exact_alias("/exact/alias/one")
                .expect("exact hit")
                .as_str(),
            registered.project_id.as_str()
        );
        assert_eq!(
            authority
                .resolve_exact_alias("/exact/alias/ONE")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_rejects_unknown_duplicate_malformed_stale_and_mismatch() {
        let (authority, root) = isolated_authority();
        let registered = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/exact/alias/keep".to_string()),
            })
            .expect("register");

        assert_eq!(
            authority
                .resolve_exact_alias("/exact/alias/missing")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        assert_eq!(
            authority
                .register_isolated_project(M1RegisterIsolatedProjectRequest {
                    exact_alias: Some("/exact/alias/keep".to_string()),
                })
                .unwrap_err()
                .code,
            "m1_alias_duplicate"
        );
        assert_eq!(
            authority.resolve_exact_alias("").unwrap_err().code,
            "m1_alias_malformed"
        );
        assert_eq!(
            authority.resolve_exact_alias("true").unwrap_err().code,
            "m1_alias_malformed"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("project:00000000-0000-4000-8000-000000000000")
                .unwrap_err()
                .code,
            "m1_project_id_unknown"
        );
        assert_eq!(
            authority
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
            authority
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
        let (authority, root) = isolated_authority();
        let _ = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest {
                exact_alias: Some("/tmp/syn-m1i01-bound".to_string()),
            })
            .expect("register");

        assert_eq!(
            authority
                .resolve_canonical_project_id("/tmp/syn-m1i01-bound")
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id(&slug_like_id("/tmp/syn-m1i01-bound"))
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("mario-test")
                .unwrap_err()
                .code,
            "m1_project_id_index_locator_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("scratch-isolated")
                .unwrap_err()
                .code,
            "m1_project_id_scratch_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("project:scratch-isolated")
                .unwrap_err()
                .code,
            "m1_project_id_scratch_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("m5:official_project_id")
                .unwrap_err()
                .code,
            "m1_project_id_m5_helper_claim_rejected"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("true")
                .unwrap_err()
                .code,
            "m1_project_id_malformed"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_does_not_import_legacy_index_records() {
        let (authority, root) = isolated_authority();
        let legacy_index = root.join("codex-index.json");
        fs::write(
            &legacy_index,
            r#"{"projects":[{"project_root":"/legacy/never-imported"}]}"#,
        )
        .expect("write legacy index");
        assert_eq!(
            authority
                .resolve_exact_alias("/legacy/never-imported")
                .unwrap_err()
                .code,
            "m1_alias_unknown"
        );
        assert_eq!(
            authority
                .resolve_canonical_project_id("/legacy/never-imported")
                .unwrap_err()
                .code,
            "m1_project_id_path_claim_rejected"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn m1_project_index_reviewer_is_distinct_and_no_capability() {
        let (authority, root) = isolated_authority();
        let registered = authority
            .register_isolated_project(M1RegisterIsolatedProjectRequest { exact_alias: None })
            .expect("register");
        let supervisor = authority
            .project_role_identity(
                registered.project_id.as_str(),
                M1ProjectRole::ProjectSupervisor,
            )
            .expect("supervisor");
        let worker = authority
            .project_role_identity(registered.project_id.as_str(), M1ProjectRole::Worker)
            .expect("worker");
        let reviewer = authority
            .project_role_identity(
                registered.project_id.as_str(),
                M1ProjectRole::IndependentReviewer,
            )
            .expect("reviewer");
        assert_eq!(reviewer.role, M1ProjectRole::IndependentReviewer);
        assert_ne!(reviewer.actor_id, supervisor.actor_id);
        assert_ne!(reviewer.actor_id, worker.actor_id);
        assert_ne!(reviewer.session_identity_id, supervisor.session_identity_id);
        assert_ne!(reviewer.owner_fingerprint, supervisor.owner_fingerprint);
        assert!(reviewer.permission_snapshot.allow_capabilities.is_empty());
        assert!(reviewer
            .permission_snapshot
            .deny_capabilities
            .contains(&"issue_execution_grant".to_string()));
        assert_eq!(reviewer.current_object.source_owner_ref, "project_index");
        let _ = fs::remove_dir_all(root);
    }
}
